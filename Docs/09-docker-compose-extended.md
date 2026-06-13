# 09 — Extended Docker Compose Path

We extended the upstream `compose.yaml` so it can run a **complete** instance (the original
shipped only Valkey + the umbrella, missing NATS and the gateway — see
[02-self-hosting-status.md](./02-self-hosting-status.md)).

## What changed (`compose.yaml`)

| Change | Why |
| --- | --- |
| **Added `nats` service** (JetStream, internal-only, `nats_data` volume, healthcheck on :8222) | Backbone for inter-service messaging. Gateway↔API RPC (`rpc.api`) and event fan-out all run over NATS. |
| **Added `fluxer_gateway` service** (built from `./fluxer_gateway`, image `fluxer-gateway:local`) | Realtime WebSocket gateway. No public image exists, so it's built locally. Reads the same `config.json`, exposes WS port 8082, healthchecks `/_health`. |
| **`fluxer_server` now `depends_on: nats`** | The umbrella must reach NATS to answer `rpc.api` and publish events. |
| **Added `nats_data` volume** | JetStream persistence. |
| **Added `.env.example`** | Documents `FLUXER_GATEWAY_PORT`, `NATS_AUTH_TOKEN`, and the existing knobs. |

Default `docker compose up` now starts: **valkey, nats, fluxer_server, fluxer_gateway**.
`meilisearch`/`elasticsearch` remain behind `--profile search`; `livekit` behind `--profile voice`.

Validated with `docker compose config -q` (parses clean; requires `MEILI_MASTER_KEY` set only
because the pre-existing meilisearch service hard-requires it via `${MEILI_MASTER_KEY:?}`).

## How the wiring works

```
client ──WSS──> fluxer_gateway (8082)
                     │  rpc.api (token validate, events)
                     ▼
                   nats (4222, JetStream)
                     ▲
                     │  subscribes rpc.api, publishes events
fluxer_server (8080) ┘   (umbrella: api/admin/app_proxy/media)
        │
     valkey (cache/ratelimit)
```

Both `fluxer_server` and `fluxer_gateway` mount `./config:/usr/src/app/config:ro` and read
`FLUXER_CONFIG=/usr/src/app/config/config.json`. The gateway pulls `services.gateway.*`,
`services.nats.*`, and `auth.vapid.*` from that same file
(`fluxer_gateway/src/gateway/fluxer_gateway_config.erl`).

## Bring-up

> Requires Docker Desktop **running** (the daemon was not up during authoring — see
> [03-running-on-windows.md](./03-running-on-windows.md)). The gateway build pulls
> `erlang:28-slim` and compiles an OTP release (several minutes the first time).

```bash
cp .env.example .env                       # adjust ports if needed
cp config/config.production.template.json config/config.json
# fill secrets in config/config.json (see 04-config-reference.md) — there is NO
# auto-seed for production like dev has.

docker compose build fluxer_gateway        # first build is slow (Erlang release)
docker compose up -d valkey nats fluxer_server fluxer_gateway
docker compose ps                          # all should become healthy
docker compose logs -f fluxer_gateway
```

Optional add-ons:
```bash
MEILI_MASTER_KEY=... docker compose --profile search up -d   # Meili + Elasticsearch
docker compose --profile voice up -d                          # LiveKit
```

## Remaining caveats (not yet turnkey)

These are config/topology concerns, not missing services:

1. **Endpoints / discovery.** Clients fetch `/.well-known/fluxer` for `endpoints.gateway` etc.
   Those derive from `domain.*` + `Config.endpoints.*`. For anything beyond localhost you need a
   **reverse proxy (Caddy/Traefik)** terminating TLS and routing `/`, `/api`, `/media`, and the
   gateway WebSocket so the discovered URLs are actually reachable. `compose.yaml` does not yet
   include that proxy.
2. **NATS auth is off by default** (internal network only). Enable token auth for hardening
   (`.env` `NATS_AUTH_TOKEN` + `--auth` flag + matching `services.nats.auth_token`).
3. **Production secrets are manual** — no bootstrap auto-seed (see
   [08-bootstrap-and-deployment.md](./08-bootstrap-and-deployment.md)). Generate with
   `openssl rand -hex 32` and a VAPID keypair.
4. **Media/S3** — prod template points `s3.endpoint` at the umbrella's own `:8080/s3`; confirm
   the embedded S3 shim is enabled in the image, or point at external object storage / MinIO.
5. **Gateway port consistency** — `FLUXER_GATEWAY_PORT` (host) and `services.gateway.port`
   (container, 8082) must agree; the healthcheck assumes 8082.

## Next steps to make it turnkey

- Add a Caddy/Traefik service + a localhost-friendly `config.json` (base_domain `localhost`)
  so the whole thing works end-to-end without external DNS/TLS.
- Add a production bootstrap script that seeds secrets + VAPID like dev does.
- See [07-self-hosting-roadmap.md](./07-self-hosting-roadmap.md) P1–P2.
