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
| 8b | `pnpm-workspace.yaml` + `Dockerfile` (`app-build`) | Set `wasm-pack: false` in `allowBuilds`; fetch the `v0.14.0` binary from `github.com/wasm-bindgen/wasm-pack` and `rm` the npm bin shim | The npm `wasm-pack@0.14.0` postinstall downloads from `github.com/drager/wasm-pack`, which **moved** to `wasm-bindgen/wasm-pack` — the old URL 404s and (with `strictDepBuilds: true`) fails the entire `pnpm install`. Not in either guide — surfaced on first real build (the repo move post-dates them). |
| 8c | `packages/api/src/infrastructure/GatewayService.tsx:264` | `new ServiceUnavailableError({message: '…'})` instead of a bare string | Upstream type error — `ServiceUnavailableError` takes an options object, not a string. `pnpm typecheck` (the `build` stage gate) fails on it (`TS2559`). Every other call site already passes an object. Not in either guide. |

### Asset / runtime correctness

| # | File | Change | Why |
| --- | --- | --- | --- |
| 9 | `fluxer_app/rspack.config.mjs` | `publicPath: '/'` (was `${CDN}/` in prod) | Self-hosted serves the JS/CSS bundle from its own origin, not the public CDN |
| 10 | `fluxer_app/scripts/build/rspack/static-files.mjs` | Fallback `https://fluxerstatic.com` when no static CDN set | PWA manifest icons resolve |
| 11 | `fluxer_server/src/ServiceInitializer.tsx` | Add `static_cdn` origin to CSP `imgSrc`/`styleSrc`/`fontSrc`/`connectSrc` | Emoji SVGs, fonts, icons come from `fluxerstatic.com`; CSP was blocking them |
| 12 | `config/*.template.json` | `database.sqlite_path` = **absolute** `/usr/src/app/data/fluxer.db` | Relative path resolves against pnpm's cwd (`fluxer_server/`) → DB on the wrong/ephemeral path |
| 13 | `config/*.template.json` | `services.s3.data_dir` = **absolute** `/usr/src/app/data/s3` | Same cwd trap — uploads silently land in the container's writable layer, lost on rebuild |
| 14 | `config/*.template.json` | `domain.endpoint_overrides.static_cdn` = `https://fluxerstatic.com/static` | Points emoji/font fetches at the public CDN (server independently derives the same host via `CdnEndpoints.STATIC_HOST`) |
| 15 | `config/*.template.json` | `services.queue.data_dir` = **absolute** `/usr/src/app/data/queue` | Same cwd trap as #12/#13 — the durable job-queue store (`queue_service.data_dir` default `./data/queue`) would otherwise sit on the ephemeral layer and lose in-flight jobs on rebuild |

### Self-host behavior changes (intentional divergences from upstream defaults)

These deliberately change how the app behaves by default for this deployment. Noted here so a
future me (or an upstream merge) knows they were on purpose, not accidental.

| # | What changed | File(s) | Why |
| --- | --- | --- | --- |
| B1 | **NSFW image scanning is OFF by default** here. Added a `FLUXER_DISABLE_NSFW` env var (`true`/`1`/`yes`) that makes `NSFWDetectionService` skip loading the ONNX model and report all media as non-NSFW. | `packages/media_proxy/src/lib/NSFWDetectionService.tsx`; wired in `unraid/compose.yaml` + staged `.env` (`FLUXER_DISABLE_NSFW=true`) | The image is built slim (`INCLUDE_NSFW_ML=false`), so `/opt/data/model.onnx` doesn't exist — but `initialize()` upstream reads it unconditionally and crashes the server on boot. The env var lets us run model-free. Set to `false` (and build with `INCLUDE_NSFW_ML=true`) to re-enable scanning. |
| B2 | **Voice/LiveKit is always-on** (removed the `profiles: ['voice']` gate). | `unraid/compose.yaml` | This deploy always wants voice; the profile gate meant a plain `compose up` / Compose Manager "Up" skipped LiveKit and forced a per-launch flag. Removing it makes LiveKit a normal service. |
| B3 | **Large uploads aren't aborted at 30s.** The REST client's `defaultTimeoutMs = 30000` was applied to *every* request, including the multipart message-with-attachment POST, so any upload that took >30s to stream was killed client-side (`xhr.timeout`). Patched `performXHRRequest` to detect upload bodies (`FormData`/`Blob`/`ArrayBuffer`) and give them a bounded 1h ceiling instead of the 30s default; normal JSON calls keep 30s. | `fluxer_app/src/lib/HttpClient.tsx` (~line 753) | Needed for attachments above ~a few hundred MB. Pairs with the admin limit-config bump (see [14](./14-raising-limits-via-admin-api.md)) and the NPM body/timeout config below. **Requires a desktop/web rebuild** — it's bundled frontend. 1h (not unbounded) so a half-open dead connection still has an upper limit; the server discards partial uploads anyway (buffer-then-store, no orphans). |

### Desktop installer behavior (`scripts/Build-FluxerDesktop.ps1`)

The Windows desktop build script wraps electron-builder and injects a generated NSIS include
(`.eb-installer.generated.nsh`, created/cleaned per-run so the tracked `electron-builder.config.cjs`
stays clean). Two installer behaviors are baked in via that include:

| Feature | How | Why |
| --- | --- | --- |
| **Auto-close running app on install** | `customInit` + `customInstall` macros run `taskkill /F /T /IM "<productName>.exe"`. Electron names the main process *and* all GPU/renderer helpers `<productName>.exe` on Windows, so one image-name kill clears every file lock. `customInit` fires in `.onInit` (before electron-builder's app-running check), `customInstall` repeats it right before file copy. | electron-builder's assisted installer otherwise stops with *"\<app\> cannot be closed. Please close it manually and click Retry"* when the app is open (commonly minimized to the tray). The kill makes reinstalls one-click. **Baked into every build.** |
| **Seed the server URL (`-DefaultServer <url>`)** | `customInstall` writes `{"app_url":"<url>"}` to `%APPDATA%\<storageDir>\settings.json` **on first install only** (`IfFileExists` guard preserves a user who later switched servers). `<storageDir>` is `fluxercanary` for canary/branded builds, `fluxer` for stable. | The fresh install opens already pointed at the self-hosted instance — no need to run `scripts/Switch-FluxerInstance.ps1`. Only emitted when `-DefaultServer` is passed. |

Build invocation for this deployment (brand implies `-Canary`; brand name must match the installed
app so the auto-kill targets the right exe):

```powershell
.\scripts\Build-FluxerDesktop.ps1 -Brand "Fluxer Bigweld" -DefaultServer "https://fluxer.bigweld.duckdns.org"
```

> The auto-kill lives in the *installer*, so it only takes effect from the **next** build onward —
> the currently-installed installer can't kill for an install that predates the feature.

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
| **Postgres database backend** | — none exist — | **Not possible without forking.** `database.backend` is `sqlite`\|`cassandra` only; zero Postgres code in the tree. SQLite is the correct single-node backend (faster than a containerized SQL server here); Cassandra is the scale-out path. Adding Postgres = a new storage adapter + perpetual upstream-merge maintenance. |

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

## Data persistence — the relative-path trap (we lost the DB once; don't repeat it)

**What happened (2026-06-14):** a routine `fluxer_server` rebuild wiped the entire database — the
account and all server data. Root cause was a chain of three things, every one of which has to be
right or your data evaporates on the next rebuild:

1. **Relative paths resolve against the wrong cwd.** `fluxer_server` runs via
   `pnpm --filter fluxer_server start`, so its working directory is **`/usr/src/app/fluxer_server/`**,
   not `/usr/src/app/`. A config value of `"./data/fluxer.db"` therefore lands at
   `/usr/src/app/fluxer_server/data/fluxer.db` — **inside the container's writable layer**, which is
   deleted when the container is recreated (every `build` / image change / `up -d` with a new image).
2. **The persistent volume mounts elsewhere.** The data volume mounts at **`/usr/src/app/data`** — a
   *different* directory from where the relative path resolved. So the DB was never on the volume.
3. **Three separate stores all default to relative paths.** It's not just the DB:
   - `database.sqlite_path` default `./data/fluxer.db` (the DB)
   - `services.s3.data_dir` default `./data/s3` (**every uploaded file** — the built-in S3 store)
   - `services.queue.data_dir` default `./data/queue` (durable job queue)

   If a config omits the `services.s3` / `services.queue` blocks entirely (as a minimal hand-written
   `config.json` easily can), they fall back to those relative defaults and ride the ephemeral layer.

**The fix — two halves, both required:**

- **Config:** all three paths pinned **absolute** under `/usr/src/app/data` (patches #12, #13, #15).
- **Mount:** the data directory is a **host bind-mount**, not a Docker named volume. In
  `unraid/compose.yaml` the `fluxer_server` volume is `${FLUXER_APPDATA:-.}/data:/usr/src/app/data`
  (was the named volume `fluxer_data`). This puts the DB/uploads/queue at
  `/mnt/user/nvme_array/appdata/Fluxer/unraid/data/` — a real directory on the array:
  - survives `docker compose down -v` (named volumes don't),
  - visible on disk and covered by **CA Appdata Backup**,
  - obvious where it lives (no `docker volume inspect` archaeology).

> A named volume would *also* survive plain rebuilds — but it's invisible, not in Appdata Backup, and
> one `down -v` deletes it. The bind-mount is the belt-and-suspenders choice after getting burned.

**Verify it actually persisted** (after first register + one upload):

```bash
# inside the container — all three should exist and grow
docker exec fluxer_server sh -c 'ls -la /usr/src/app/data /usr/src/app/data/s3 /usr/src/app/data/queue'
# on the HOST — the real proof it's on the array, not the ephemeral layer
ls -la /mnt/user/nvme_array/appdata/Fluxer/unraid/data/
```

If `fluxer.db` grows past ~4 KB and an `s3/` tree appears **on the host**, you're safe. If the host
dir is empty but the app works, you're writing to the ephemeral layer — stop and fix before the next
rebuild.

## Database backend — SQLite vs Cassandra (no Postgres)

Fluxer's data layer supports exactly two backends (`database.backend` enum in
`packages/config/src/ConfigSchema.json`): **`sqlite`** and **`cassandra`**. There is **no Postgres
support anywhere in the codebase** — it's not a config toggle, it's an unwritten storage adapter, so
"swap in Postgres for performance" isn't a compose change, it's a fork.

For a single self-hosted box, **SQLite is the right and faster choice** — an in-process embedded DB
with no network hop beats a containerized SQL server for one node. Cassandra is upstream's
*horizontal-scale* path (multi-node, JVM, heavy) and is overkill here. See the "Not applied" table.

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
- Advanced (Custom Nginx Configuration — for large uploads/attachments):
  ```nginx
  client_max_body_size 0;        # no proxy-level size cap (Fluxer enforces its own limit)
  proxy_request_buffering off;    # stream the upload instead of buffering the whole file first
  proxy_read_timeout 3600s;
  proxy_send_timeout 3600s;
  client_body_timeout 3600s;      # the 30s-ish default severs large uploads mid-stream
  send_timeout 3600s;
  ```
  These must go in the **proxy host's** Advanced tab, not global Settings. See
  [14](./14-raising-limits-via-admin-api.md) for the full upload-limit story (NPM is only one of
  three layers — also the Fluxer limit-config and the client-side timeout patch B3).
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

## Email (SMTP) — wired

The duckdns template uses real Gmail SMTP (`smtp.gmail.com:465`, secure). The app **password is
never committed**: it lives in `.env` as `FLUXER_SMTP_PASSWORD` (gitignored) and is injected into
the server at runtime via `FLUXER_CONFIG__INTEGRATIONS__EMAIL__SMTP__PASSWORD` (set on the
`fluxer_server` service in `compose.yaml`). To change it, edit `.env` and `up -d` again — no rebuild.

> Gmail requires an **App Password** (not your account password) with 2FA enabled, and the
> `from_email` must be the authenticated account or a configured alias. Strip the spaces Google
> shows. If mail silently fails, check the server logs and that the App Password is still valid.

## Voice (LiveKit) — wired

Voice is fully configured under `--profile voice`. What was set up:

- **`config/livekit.template.yaml`** → rendered to `config/livekit.yaml` (gitignored) by the seeder,
  with the API key/secret matching `integrations.voice` in `config.json`.
- **Single muxed UDP port** (`rtc.udp_port: 7882`) instead of the `50000-50100` range — that range
  maps to 101 iptables rules and can hang the Docker daemon (mgabor3141's guide). One port, one
  rule. No `network_mode: host` needed.
- **`rtc.use_external_ip: true`** — LiveKit auto-detects your WAN IP via STUN at startup (re-detected
  on restart, so it tolerates your dynamic DuckDNS IP). No hardcoded `node_ip`.
- **Webhook** → `http://fluxer_server:8080/api/webhooks/livekit` (internal container address), so
  participant join/leave events reconcile and users don't get stuck in channels.
- **Caddy** routes `wss://<domain>/livekit` → `livekit:7880` (signaling only; media is direct UDP).

### Bring up voice

```bash
# seed (generates the voice key/secret + renders config/livekit.yaml)
node scripts/seed_config.mjs config/config.json --from config/config.duckdns.template.json
# start the stack WITH the voice profile
docker compose -f compose.yaml -f compose.localhost.yaml -f compose.unraid.yaml --profile voice up -d
```

### Router port-forwards for voice (in addition to 80/443)

| Port | Proto | Purpose |
| --- | --- | --- |
| 7882 | UDP | Primary RTC media (muxed) |
| 7881 | TCP | ICE TCP fallback (clients that can't do UDP) |
| 3479 | UDP | Embedded TURN relay |

> **TURN port note:** the LiveKit/TURN default is **3478**, but on this NAS that port is already
> taken by `nextcloud-aio-talk` and `Screego`, so we moved TURN to **3479**. This is set in three
> places that must agree: `config/livekit.yaml` → `turn.udp_port: 3479`, the `livekit` `ports:`
> mapping in `unraid/compose.yaml` (`'3479:3479/udp'`), and the router forward (3479/udp). If you
> ever free up 3478, you can move it back by syncing those three.

These go to the Unraid host; the `livekit` container publishes them. Signaling (7880) does **not**
need forwarding — it rides the existing 443 → Caddy → `livekit:7880` path. If voice connects but
has no audio, it's almost always one of these three UDP/TCP forwards missing, or `use_external_ip`
detecting the wrong IP (set `rtc.node_ip: <wan-ip>` in the template and re-seed `--force` to pin it).
