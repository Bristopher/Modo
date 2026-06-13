# 11 — Build Patches (from community guides) + Unraid Deployment

This doc records the fixes we applied to make the source actually **build and run** self-hosted,
cross-referenced against two community guides, plus a deployment walkthrough for an **Unraid NAS
behind a reverse proxy with a real domain** (`bigweld.duckdns.org`).

## Source guides

| Guide | Author | Where | Notes |
| --- | --- | --- | --- |
| Fluxer self-hosted deployment guide (refactor branch) | PaulMColeman | [gist e7ef82e…](https://gist.github.com/PaulMColeman/e7ef82e05035b24300d2ea1954527f10) | 20 gotchas + SSO + admin; AI-formatted |
| Self-hosting Fluxer: findings, bugs, and patches | mgabor3141 | [discussion #542](https://github.com/orgs/fluxerapp/discussions/542) | 27 issues incl. LiveKit/host-net + S3 paths |

Both confirm: the published `ghcr.io/fluxerapp/fluxer-server` image is **private**, the Dockerfile
is **stale** relative to source, and self-hosting is **pre-release**. Everything below is verified
against *our* checkout, not copied blindly — several guide line/filter references differ here.

## Patches we applied

Each is marked in-code with a `Self-host patch:` comment.

### Build blockers (without these, `docker build` fails)

| # | File | Change | Why |
| --- | --- | --- | --- |
| 1 | `.dockerignore` | Re-include `fluxer_app/scripts/build/**`; stop ignoring `emojis.json` | `**/build` rule stripped the rspack build helpers; emoji data is required at build |
| 2 | `fluxer_server/Dockerfile` (`deps`/`build`) | Whole `COPY packages/ ./packages/` instead of stale per-package list | Upstream list references removed `packages/app` and omits ~17 real packages |
| 3 | `fluxer_server/Dockerfile` (`app-build`) | Install rustup + `wasm32-unknown-unknown` + `wasm-pack`; `ca-certificates`, `build-essential` | Frontend compiles `crates/libfluxcore` → WASM; apt `rustc` lacks the wasm target |
| 4 | `fluxer_server/Dockerfile` (`app-build`) | `ARG FLUXER_BUILD_CONFIG` → `/tmp/fluxer-build-config.json`, `ENV FLUXER_CONFIG=…` | `rspack.config.mjs` throws without `FLUXER_CONFIG`; it bakes endpoint URLs into the SPA |
| 5 | `fluxer_server/Dockerfile` (`app-build`) | Added `pnpm --filter @fluxer/config generate` | App build needs the generated config schema |
| 6 | `fluxer_app/package.json` | Removed `tsgo --noEmit &&` from `build` | It runs *before* `lingui:compile`, so locale `.mjs` don't exist yet → type errors. Types are still checked in the `build` stage's `pnpm typecheck`. |
| 7 | `fluxer_server/Dockerfile` (`build`) | Added `pnpm --filter @fluxer/admin build:css` | Admin panel CSS is never compiled upstream (needed to grant premium / manage instance). **Guide said `--filter admin` — wrong here; the package is `@fluxer/admin`.** |
| 8 | `fluxer_server/Dockerfile` | `ENTRYPOINT ["pnpm","--filter","fluxer_server","start"]` | Root workspace has no `start` script |

### Asset / runtime correctness

| # | File | Change | Why |
| --- | --- | --- | --- |
| 9 | `fluxer_app/rspack.config.mjs` | `publicPath: '/'` (was `${CDN}/` in prod) | Self-hosted serves the JS/CSS bundle from its own origin, not the public CDN |
| 10 | `fluxer_app/scripts/build/rspack/static-files.mjs` | Fallback `https://fluxerstatic.com` when no static CDN set | PWA manifest icons resolve |
| 11 | `fluxer_server/src/ServiceInitializer.tsx` | Add `static_cdn` origin to CSP `imgSrc`/`styleSrc`/`fontSrc`/`connectSrc` | Emoji SVGs, fonts, icons come from `fluxerstatic.com`; CSP was blocking them |
| 12 | `config/*.template.json` | `database.sqlite_path` = **absolute** `/usr/src/app/data/fluxer.db` | Relative path resolves against pnpm's cwd (`fluxer_server/`) → DB on the wrong/ephemeral path |
| 13 | `config/*.template.json` | `services.s3.data_dir` = **absolute** `/usr/src/app/data/s3` | Same cwd trap — uploads silently land in the container's writable layer, lost on rebuild |
| 14 | `config/*.template.json` | `domain.endpoint_overrides.static_cdn` = `https://fluxerstatic.com/static` | Points emoji/font fetches at the public CDN (server independently derives the same host via `CdnEndpoints.STATIC_HOST`) |

### Design note: the domain is baked in at build time

`rspack` reads `FLUXER_CONFIG` and **hardcodes** the api/gateway/media/static URLs into the bundle.
So the **build is domain-specific**. We drive it from `.env`:

- `BASE_DOMAIN`, `PUBLIC_SCHEME`, `PUBLIC_PORT` → interpolated into the `FLUXER_BUILD_CONFIG`
  build arg in `compose.localhost.yaml`, **and** must match the seeded `config.json` domain block.
- Change the domain ⇒ **rebuild** (`docker compose … build fluxer_server`), not just restart.

## Not applied yet (only needed for specific features)

| Feature | Guide fixes | Status |
| --- | --- | --- |
| **SSO / OIDC** (Zitadel, Keycloak, Pocket ID…) | URLSearchParams `.toString()`, `includeSecret:true`, client_secret in body, unclaimed-account trait, `/auth/sso/` standalone route, callback `clearTimeout` | Skipped — we use password auth. Apply if you wire an IdP. |
| **Voice/video (LiveKit)** | webhook config, host networking (UDP range port-mapping hangs Docker), `node_ip`/DDNS, voice-states-in-READY Erlang fix | Skipped — add via `--profile voice` later; see notes below. |
| **Klipy GIF picker** | webm→gif fallback in `KlipyService.tsx` | Skipped — cosmetic third-party integration. |

## Building & running

The full localhost run steps are in **[RUNNING.md](./RUNNING.md)**. The short version, and the
production (Unraid) variant, follow.

### Localhost (smoke test on your dev box)

```bash
# .env defaults to localhost — nothing to change.
node scripts/seed_config.mjs config/config.json --from config/config.localhost.template.json
docker compose -f compose.yaml -f compose.localhost.yaml build fluxer_gateway fluxer_server
docker compose -f compose.yaml -f compose.localhost.yaml up -d
# → http://localhost:8080/ , emails at http://localhost:8080/mailpit/
```

> First `fluxer_server` build is **long** — it compiles a Rust→WASM crate, an Erlang OTP release,
> and the full SPA. Your 3 Gbps line helps with the image pulls; the compile is CPU-bound.

## Unraid deployment (real domain + reverse proxy)

Target topology (NPM and the stack share the custom Docker network **`bignet`**, so NPM reaches
Caddy by container name + internal port — no published host port):

```
browser ──https──> fluxer.bigweld.duckdns.org ──> [router :443] ──> [Nginx Proxy Manager, TLS]
                                                                         │  (both on docker net "bignet")
                                                                         │  proxy to  caddy:8080
                                                                         ▼
                                              [caddy :8080]  ← sole stack entrypoint, plain http
                                                ├─ /gateway* → fluxer_gateway:8082   (on "default" net)
                                                ├─ /mailpit/* → mailpit:8025
                                                └─ everything → fluxer_server:8080  (api, media, SPA, /admin, /s3)
                                                           │ NATS rpc.api
                                              [nats] [valkey] [mailpit]   ← private "default" net only
```

NPM terminates TLS for `https://fluxer.bigweld.duckdns.org` and forwards the whole hostname to the
stack's Caddy. Because NPM is on `bignet` and `compose.unraid.yaml` attaches `caddy` to `bignet`,
the NPM proxy target is simply **`caddy` : `8080`** (the internal container port) — **no host IP, no
published port**. Caddy does the single-origin path routing internally, so **you do not configure
per-path routes in NPM** — one proxy host, **WebSocket support ON** (for `/gateway`). The API,
gateway, NATS, Valkey, and Mailpit stay on the private `default` network, reachable only via Caddy.

### 1. Get the source onto Unraid

LFS smudge can hang clones — skip it:

```bash
GIT_LFS_SKIP_SMUDGE=1 git clone https://github.com/fluxerapp/fluxer.git
# then copy our patched files over it, or clone this patched working copy.
```

(We already have the patched tree; copy the whole project dir to the NAS, e.g. an `appdata` share.)

### 2. Point the build + config at your domain

Edit `.env` — comment the localhost block, uncomment production:

```env
BASE_DOMAIN=bigweld.duckdns.org
PUBLIC_SCHEME=https
PUBLIC_PORT=443
HOST_HTTP_PORT=8080      # what your reverse proxy forwards to
```

Seed config from the production template (already set to this domain):

```bash
node scripts/seed_config.mjs config/config.json --from config/config.duckdns.template.json
```

> Using a different subdomain instead? Change `BASE_DOMAIN` in `.env` **and** the `domain`/`s3`/
> `email` blocks in `config/config.duckdns.template.json` (or your `config.json`) to match, then
> rebuild. The two must agree or the SPA will call the wrong URLs.

### 3. Build (domain baked in) and boot

Add the `compose.unraid.yaml` override — it joins `caddy` to your existing `bignet` network and
drops the published host port. (`bignet` must already exist: `docker network create bignet`, or
make it in the Unraid Docker UI.)

```bash
docker compose -f compose.yaml -f compose.localhost.yaml -f compose.unraid.yaml build fluxer_gateway fluxer_server
docker compose -f compose.yaml -f compose.localhost.yaml -f compose.unraid.yaml up -d
docker compose -f compose.yaml -f compose.localhost.yaml -f compose.unraid.yaml ps
```

(`HOST_HTTP_PORT` in `.env` is unused with this override since nothing is published — NPM reaches
Caddy over `bignet`.)

### 4. Reverse proxy host

In Nginx Proxy Manager (which must also be attached to **`bignet`**): new proxy host
- Domain: `fluxer.bigweld.duckdns.org`
- Scheme: `http`, Forward Hostname: **`caddy`**, Forward Port: **`8080`** (container name + internal port)
- **Websockets support: ON**
- Advanced: `client_max_body_size 100M;` (for uploads/attachments)
- SSL: request a Let's Encrypt cert (DuckDNS DNS challenge easiest), Force SSL on.

Port-forward **80/443** on your router → the NPM host (for the cert + public HTTPS). **Do not** port
the internal `8080` — it isn't even published. The DuckDNS hostname (wildcard `*.bigweld.duckdns.org`
resolves automatically) must point at your WAN IP via a DuckDNS updater or router DDNS.

### 5. First account = admin

The **first** user to register gets wildcard admin ACLs (`*`) → `/admin`. Register via the app,
grab the verification code from `https://bigweld.duckdns.org/mailpit/` (swap to real SMTP later in
`config.json`'s `integrations.email`). To grant admin to others later:

```bash
docker exec fluxer_server sqlite3 /usr/src/app/data/fluxer.db \
  "INSERT INTO admin_acls (user_id, permission) VALUES ('<user-id>', '*');"
```

Premium ("Plutonium"): self-hosted unlocks all perks for everyone; the badge/status is per-user via
admin gift codes (`POST /admin/codes/gift`). See [02](./02-self-hosting-status.md).

### Running it as a Docker Hub image (optional)

The build is heavy. Build once, push to your own registry, pull on the NAS — but remember the SPA
has the domain baked in, so the image is **specific to `bigweld.duckdns.org`**:

```bash
# on a build box:
docker compose -f compose.yaml -f compose.localhost.yaml build fluxer_server fluxer_gateway
docker tag fluxer-server:local  <youruser>/fluxer-server:bigweld
docker tag fluxer-gateway:local <youruser>/fluxer-gateway:latest
docker push <youruser>/fluxer-server:bigweld
docker push <youruser>/fluxer-gateway:latest
# on Unraid: set FLUXER_SERVER_IMAGE + the gateway image to your pushed tags and `up -d` (no build).
```

(AGPLv3: if you publish a modified server others can reach, you must offer them your modified source.)

## Voice (LiveKit) — when you get to it

From mgabor3141's guide, the sharp edges:
- **Do not** map the UDP media range (`50000-50100:…/udp`) — 101 iptables rules can hang Docker's
  networking. Use `network_mode: host` for the LiveKit container instead.
- Forward **3479/udp** (TURN), **7881/tcp** (ICE TCP), **7882/udp** (RTP) on the router.
- Set `rtc.use_external_ip: true`, `node_ip: <wan-ip>` (or a DDNS-resolve entrypoint for dynamic IPs).
- Add a `webhook` block in `config/livekit.yaml` → `https://bigweld.duckdns.org/api/webhooks/livekit`,
  else participants get stuck in channels.

Our `compose.yaml` already has a `livekit` service under `--profile voice`; the host-networking +
webhook changes are still TODO there.
