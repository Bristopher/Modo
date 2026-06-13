# How to Run Fluxer

A self-contained guide to standing up a local Fluxer instance. Two paths:

- **Path A — Docker (recommended on Windows).** One command set, uses the turnkey localhost
  bundle (Caddy + Mailpit + auto-seeded secrets). Start here.
- **Path B — devenv (Nix).** The maintainer-supported dev environment; Linux/macOS or WSL2.

> Status: validated by config parse + seed test, **not yet runtime-booted** (Docker daemon was
> down at authoring). First run is the verification step. See the troubleshooting section.

---

## Prerequisites

| Path | Needs |
| --- | --- |
| A (Docker) | Docker Desktop **running** (Compose v2.24+; you have v28 / Compose v2.39). Node 18+ on PATH for the one-time seed script (you have v22). |
| B (devenv) | Nix + devenv. On Windows: **WSL2** (Ubuntu) — Nix doesn't run natively on Windows. |

Working directory for all commands: `H:/Code/Projects/Fluxer/Pre-Self-Hosting`

---

## Path A — Docker (turnkey localhost)

Single origin on `http://localhost:8080`. Brings up six services: **caddy, nats, valkey,
fluxer_server, fluxer_gateway, mailpit**. Caddy is the only thing bound to your host.

### 1. Create + seed the runtime config

Generates `config/config.json` from the localhost template and fills every secret + a VAPID
keypair (production has no auto-seed like dev does):

```bash
node scripts/seed_config.mjs config/config.json --from config/config.localhost.template.json
```

Re-running is safe — it only fills empty/placeholder fields. Use `--force` to regenerate all.

### 2. Build the gateway image

No public gateway image exists, so it's built locally. **First build is slow** — it compiles an
Erlang/OTP release (pulls `erlang:28-slim`, several minutes):

```bash
docker compose -f compose.yaml -f compose.localhost.yaml build fluxer_gateway
```

### 3. Boot the stack

```bash
docker compose -f compose.yaml -f compose.localhost.yaml up -d
```

### 4. Check it's healthy

```bash
docker compose -f compose.yaml -f compose.localhost.yaml ps
docker compose -f compose.yaml -f compose.localhost.yaml logs -f fluxer_server fluxer_gateway
```

Wait for `fluxer_server` and `fluxer_gateway` to report `healthy`.

### 5. Use it

- **App:** http://localhost:8080/
- **Verification emails (Mailpit):** http://localhost:8080/mailpit/ — register an account, grab
  the code here.
- **Discovery doc (sanity check):** http://localhost:8080/.well-known/fluxer — confirm
  `endpoints.gateway` is `ws://localhost:8080/gateway`.

### Tear down

```bash
docker compose -f compose.yaml -f compose.localhost.yaml down       # stop, keep data
docker compose -f compose.yaml -f compose.localhost.yaml down -v     # stop + wipe volumes
```

### Optional add-ons (profiles)

```bash
# Full-text search (Meilisearch + Elasticsearch — heavy). Set the key + add it to config.
MEILI_MASTER_KEY=$(openssl rand -hex 16) docker compose -f compose.yaml -f compose.localhost.yaml --profile search up -d

# Voice/video (LiveKit)
docker compose -f compose.yaml -f compose.localhost.yaml --profile voice up -d
```

For search, also add an `integrations.search` block to `config/config.json` pointing at
`http://meilisearch:7700` with the same key. For voice, fill `integrations.voice`.

---

## Path B — devenv (Nix) on WSL2

The supported dev stack; boots everything (incl. live rebuilds, Mailpit, LiveKit) via
process-compose. Runs at `http://localhost:48763/`.

```bash
# Inside WSL2 Ubuntu — clone in the Linux filesystem, NOT under /mnt/h (perf + permissions)
cd ~ && git clone https://github.com/fluxerapp/fluxer.git && cd fluxer

# Install Nix + devenv: https://devenv.sh/getting-started/
cp config/config.dev.template.json config/config.json   # bootstrap also does this if missing

devenv shell      # Node 24, Erlang 28, Rust+wasm, etc.
devenv up         # runs fluxer:bootstrap (auto-seeds secrets/keys) then all processes
```

- App: http://localhost:48763/
- Mailpit: http://localhost:48763/mailpit/
- Logs: `dev/logs/*.log`

Dev auto-generates all secrets, VAPID, Bluesky OAuth keys, and LiveKit config — you fill nothing.
See [08-bootstrap-and-deployment.md](./08-bootstrap-and-deployment.md).

---

## Troubleshooting / first-run watch list

| Symptom | Likely cause / fix |
| --- | --- |
| `docker compose config` error: `MEILI_MASTER_KEY is missing` | Pre-existing required var on the meilisearch service. Only matters if you use `--profile search`; set `MEILI_MASTER_KEY` in `.env`. Harmless otherwise. |
| Gateway build fails | Needs network to pull `erlang:28-slim` + rebar3 deps. Retry with connectivity. |
| `/` returns 404 (no app) | `services.server.static_dir` must be set (it's `/usr/src/app/assets` in the localhost template — the image's built SPA path). The Dockerfile's `FLUXER_SERVER_STATIC_DIR` env is ignored by the loader. |
| Server crashes on boot re: search | `integrations.search` is omitted (image targets SQLite search). If a build genuinely requires it, add it back + run `--profile search`. |
| Media uploads fail | Monolith mode mounts the S3 shim at `/s3` and serves media signed via `/media`; no MinIO needed. If broken, check `s3.presigned_url_base` (defaults to the internal endpoint). |
| Can't get a verification code | Email goes to Mailpit at `/mailpit/`. Ensure the `mailpit` service is healthy and `integrations.email.smtp.host=mailpit`. |
| Realtime not working (messages don't arrive live) | Check `fluxer_gateway` is healthy and reaching `nats`. The gateway↔API RPC runs over NATS subject `rpc.api`; both must connect. |
| `!reset` tag error on compose | Needs Docker Compose v2.24+. Check `docker compose version`. |

When you do a real first boot, capture what actually happened back into
[10-turnkey-localhost.md](./10-turnkey-localhost.md) and the roadmap
([07-self-hosting-roadmap.md](./07-self-hosting-roadmap.md)).

---

## Port reference

| Path A (Docker) | | Path B (devenv) | |
| --- | --- | --- | --- |
| Everything | `:8080` (Caddy) | Everything | `:48763` (Caddy) |
| (internal) server | 8080 | server | 49319 |
| (internal) gateway | 8082 | gateway | 49107 |
| (internal) nats | 4222 | nats core / js | 4222 / 4223 |
| (internal) valkey | 6379 | valkey | 6379 |
| (internal) mailpit | 8025 / 1025 | mailpit | 49667 / 49621 |

See [01-architecture-overview.md](./01-architecture-overview.md) for the full map.
