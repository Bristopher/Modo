# 02 — Self-Hosting Status (as of 2026-06-12)

## TL;DR

Fluxer is runnable from source today via **`devenv up`**, but it is **pre-release for
self-hosting**. The maintainer's README says so directly, and the `Self-hosting` section is
literally `TBD`. The "easy" Docker `compose.yaml` path is **incomplete** — it cannot run a
working instance on its own.

## What upstream says

From `README.md` (CAUTION block):

> "please wait a little longer before you dive deep into the current codebase or try to set up
> self-hosting. I'm aware the current stack isn't very lightweight."

And the Self-hosting section body is just `TBD`, pointing to `docs.fluxer.app/self-hosting`
(the intended future path). Self-hosted deployments are promised to be fully unlocked (no
"Plutonium"/paywall); tiers/limits configurable in the admin panel.

## What works today

- **Full stack via devenv** (`devenv up`): provisions Node 24, Erlang 28, Rust+wasm, Valkey,
  NATS core+JetStream, Meilisearch, LiveKit, Mailpit, Caddy, and all Fluxer processes via
  process-compose. This is the supported path and boots a complete, working instance at
  `http://localhost:48763/`.
- **SQLite default storage** — no external DB needed for a single-node instance.
- **Config templates** exist for dev and production (`config/*.template.json`).
- **A prebuilt server image** is published: `ghcr.io/fluxerapp/fluxer-server:stable`
  (referenced by `compose.yaml`).

## The critical gap: `compose.yaml` is not a complete instance

`compose.yaml` defines only:

- `valkey` (always)
- `fluxer_server` (the umbrella image, port 8080)
- `meilisearch` + `elasticsearch` (behind `search` profile)
- `livekit` (behind `voice` profile)

**Missing from `compose.yaml` but required by the production config template
(`config.production.template.json`):**

| Required by config | In `compose.yaml`? | Impact if missing |
| --- | --- | --- |
| **NATS** (`services.nats.core_url: nats://nats:4222`) | ❌ No | Inter-service eventing breaks; gateway can't fan out events |
| **fluxer_gateway** (Erlang, `services.gateway.port: 8082`) | ❌ No | **No realtime** — messages, typing, presence won't deliver over WebSocket |
| **S3-compatible store** (`s3.endpoint`) | Partial (points at `:8080/s3` on the server itself) | Media uploads depend on the server's embedded S3 shim being enabled |

So: **upstream's** `docker compose up` gives you the HTTP API + web app, but **no working
realtime gateway and no NATS**, which means it is not a usable chat instance. This is the core
reason self-hosting is still "TBD".

> **Update (this fork):** `compose.yaml` has been extended to add the missing **NATS** and
> **fluxer_gateway** services (gateway built locally from `./fluxer_gateway`). Default
> `docker compose up` now runs valkey + nats + fluxer_server + fluxer_gateway. See
> [09-docker-compose-extended.md](./09-docker-compose-extended.md). Remaining gaps are config/
> topology (reverse-proxy + TLS for non-localhost discovery, production secret seeding), not
> missing services.

## Secrets you must generate either way

The production template has these placeholders (all must be replaced — see
[04-config-reference.md](./04-config-reference.md)):

- `services.media_proxy.secret_key` — 64-char hex
- `services.admin.secret_key_base`, `services.admin.oauth_client_secret` — 64-char hex
- `services.marketing.secret_key_base` — 64-char hex
- `services.gateway.admin_reload_secret` — 64-char hex
- `services.nats.auth_token` — NATS auth token
- `auth.sudo_mode_secret`, `auth.connection_initiation_secret` — 64-char hex
- `auth.vapid.public_key` / `private_key` — Web Push VAPID keypair
- `s3.access_key_id` / `secret_access_key`
- `integrations.search.api_key` — Meilisearch master key

## Bottom line for "stand it up now"

- **Want a working instance now** → use **devenv on WSL2** (Linux). See
  [03-running-on-windows.md](./03-running-on-windows.md). It is the only path that includes the
  Erlang gateway + NATS out of the box.
- **Want the Docker path** → it needs to be extended first (add NATS + a gateway image/build).
  Tracked in [07-self-hosting-roadmap.md](./07-self-hosting-roadmap.md).
