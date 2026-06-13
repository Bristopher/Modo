# 01 — Architecture Overview

Fluxer is a single **pnpm + Nix monorepo** containing the client, the backend, and all infra
orchestration. It is a Discord-like chat/VoIP platform.

## Workspace layout

From `pnpm-workspace.yaml` the active TS workspaces are: `packages/*`, `fluxer_admin`,
`fluxer_api`, `fluxer_app`, `fluxer_app_proxy`, `fluxer_gateway`, `fluxer_integration`,
`fluxer_marketing`, `fluxer_media_proxy`, `fluxer_relay_directory`, `fluxer_server`.
`fluxer_docs` and `fluxer_desktop` are explicitly **excluded** from the workspace (built separately).

| Directory | Role | Language |
| --- | --- | --- |
| `fluxer_server` | **Umbrella backend** — combines all TS backend services into one Node process for self-hosters (see `fluxer_server/package.json` description) | TS/Node ≥24 |
| `fluxer_api` | Core API (Hono HTTP). Auth, instance, connections, limits, unfurler, etc. | TS/Hono |
| `fluxer_admin` | Admin panel (tiers, limits, instance config) | TS |
| `fluxer_app` | React web client (also bundled into desktop) | React 19 |
| `fluxer_app_proxy` | Serves/proxies the web app | TS |
| `fluxer_media_proxy` | Media/attachment proxy + transforms | TS (sharp/ffmpeg) |
| `fluxer_marketing` | Marketing site | TS |
| `fluxer_gateway` | **Realtime WebSocket gateway** — message routing + presence. Separate service. | **Erlang/OTP 28** |
| `fluxer_relay` / `fluxer_relay_directory` | Relay infrastructure (federation groundwork, encrypted multiplexed transport) | TS |
| `fluxer_integration` | Third-party integrations | TS |
| `fluxer_desktop` | Electron wrapper around the web client | Electron |
| `fluxer_devops` | Deploy helpers (e.g. `livekitctl`) | mixed |
| `packages/*` | Shared internal libs: `@fluxer/config`, `@fluxer/hono`, `@fluxer/logger`, `@fluxer/nats`, `@fluxer/s3`, `@fluxer/kv_client`, `@fluxer/errors`, `@fluxer/constants`, `api`, etc. | TS |
| `fluxer_static`, `config`, `dev`, `scripts`, `tsconfigs` | static assets, config templates, dev tooling | — |

## The umbrella: what `fluxer_server` actually is

`fluxer_server/package.json` depends on (workspace refs):
`@fluxer/admin`, `@fluxer/api`, `@fluxer/app_proxy`, `@fluxer/media_proxy`, `@fluxer/config`,
`@fluxer/hono`, `@fluxer/initialization`, `@fluxer/kv_client`, `@fluxer/nats`, `@fluxer/s3`,
`@fluxer/logger`, `@fluxer/sentry`, etc.

Entry point: `fluxer_server/src/startServer.tsx` → `createFluxerServer()` → `initialize()` →
`start()`. So **one Node process serves the API, admin, app proxy, media proxy, and marketing**.

What it does **not** contain: the **Erlang gateway** and **NATS**. Those run as their own
processes. The umbrella talks to the gateway and to NATS over the network.

## Tech stack (from README)

- TypeScript / Node.js + **Hono** for all HTTP services
- **Erlang/OTP** for the realtime WebSocket gateway
- **React** + **Electron** for client / desktop
- **Rust → WebAssembly** for perf-critical client code
- **SQLite** default storage, optional **Cassandra** for distributed deployments
- **Meilisearch** for full-text search (Elasticsearch also wired in `compose.yaml`)
- **Valkey** (Redis-compatible) for cache / rate limiting / ephemeral coordination
- **NATS** (core + JetStream) for inter-service messaging / event streaming
- **LiveKit** for voice/video

## Service / port map (dev, from `devenv.nix` + `config/config.dev.template.json`)

| Process | Dev port | Notes |
| --- | --- | --- |
| Caddy (front door) | **48763** | Reverse proxy; this is the URL you open |
| fluxer_app (Vite dev) | 49427 | proxied by Caddy |
| fluxer_server (umbrella) | 49319 | API/admin/app_proxy/media |
| fluxer_gateway (Erlang) | 49107 | WebSocket |
| marketing_dev | 49531 | |
| Valkey | 6379 | |
| NATS core | 4222 | |
| NATS JetStream | 4223 | |
| Meilisearch | 7700 | |
| LiveKit | 7880 (+ 7881 tcp, 3478/udp, 50000-50100/udp) | voice/video |
| Mailpit | SMTP 49621 / UI 49667 (`/mailpit/`) | captures dev emails |

In **production** template (`config.production.template.json`): server on `8080`,
gateway on `8082`, Valkey via `redis://valkey:6379`, NATS via `nats://nats:4222`.

## Request/data flow (high level)

```
Browser ──HTTP──> Caddy ──> fluxer_app (static/SPA)
   │                         └─> fluxer_server (/api, /admin, /media, /s3)
   │
   └──WSS──> fluxer_gateway (Erlang)
                 │  validates client token via RPC ─> fluxer_server API
                 └─ pub/sub via NATS  <──>  fluxer_server (publishes events)

Storage:  SQLite (default)  |  Valkey (cache/ratelimit)  |  Meilisearch (search)
Voice:    client <──WebRTC──> LiveKit  (signaling proxied, media direct)
```

See [06-client-gateway-connection.md](./06-client-gateway-connection.md) for the connection
detail and [05-auth-flow.md](./05-auth-flow.md) for auth.
