# 06 — Client → Gateway Connection

How the React web client (`fluxer_app`) discovers and connects to the Erlang realtime gateway
(`fluxer_gateway`). File:line refs are against the current clone (2026-06-12); the repo is
mid-refactor, so re-verify before depending on exact lines.

## 1. Discovery: how the client finds the gateway

- Client bootstrap config: `fluxer_app/src/Config.tsx`
  - `PUBLIC_BOOTSTRAP_API_ENDPOINT` (defaults to `/api`)
  - builds discovery URL `<api>/.well-known/fluxer`
- Discovery document is served by `packages/api/src/instance/InstanceController.tsx:34`
  (`GET /.well-known/fluxer`) and returns `endpoints.gateway` (plus media, static_cdn, admin,
  webapp, …) from `Config.endpoints.*`.
- Client stores it in `fluxer_app/src/stores/RuntimeConfigStore.tsx` as `gatewayEndpoint`.
  - **Relay mode:** if a relay directory URL is configured, the gateway connection is tunneled
    through a relay with multiplexed encryption (`RelayClient.tsx`, RuntimeConfigStore relay paths).

## 2. Opening the socket

- `fluxer_app/src/lib/GatewaySocket.tsx`
  - URL built (~L916-929): `gatewayEndpoint` + query params `v` (API version), `encoding=json`,
    `compress` (`zstd-stream` or `none`). e.g.
    `wss://gateway.example.com/?v=1&encoding=json&compress=zstd-stream`
  - `new WebSocket(url)` (~L388); `binaryType='arraybuffer'` when compression is on.
- **Framing/serialization:** JSON (`JSON.stringify` ~L960), optional **zstd-stream** compression
  (`GatewayCompression`). Both text and binary frames handled.

## 3. Handshake (HELLO → IDENTIFY/RESUME)

Opcode-based protocol (Discord-like): `HELLO=10, IDENTIFY=2, DISPATCH=0, HEARTBEAT=1,
HEARTBEAT_ACK=11, RESUME=...`.

1. Server sends **HELLO** with `heartbeat_interval` (gateway: `gateway_handler.erl` ~L105-108).
2. Client `handleHelloPayload` (~L593) then sends **IDENTIFY** (~L636-650):
   ```jsonc
   { "op": 2, "d": {
       "token": "flx_…",
       "properties": { "os": "...", "browser": "...", "device": "..." },
       "presence": { "status": "...", "afk": false, "mobile": false, "custom_status": null },
       "flags": 0,
       "initial_guild_id": null
   } }
   ```
   or **RESUME** (~L653-669) when re-establishing an existing session (replays from last seq).

## 4. Gateway side (Erlang)

- `fluxer_gateway/src/gateway/gateway_handler.erl`
  - parses `v`/`encoding`/`compress`, extracts client IP (~L76-90)
  - validates API version == 1 (~L97), sends HELLO (~L105-108), inits heartbeat (~L111-114)
  - decodes frames via `gateway_codec:decode` (~L143), per-opcode rate limiting (~L153-157)
  - IDENTIFY handler (~L460-483): validates payload, generates session id, calls
    `session_manager:start/2`
- Token validation → RPC back into the API (`type:"session"`), see
  [05-auth-flow.md](./05-auth-flow.md). 401 → `authentication_failed`/`invalid_token`.

## 5. Heartbeats

- Client (`GatewaySocket.tsx`):
  - interval from HELLO; first beat at ~80% interval + jitter (~L687-688)
  - HEARTBEAT payload `{ op: 1, d: <last_seq> }` (~L748-751)
  - HEARTBEAT_ACK timeout = **15000ms** (`HeartbeatAck: 15000`, ~L39); next beat scheduled after
    ACK (~L710)
  - last sequence updated on DISPATCH (~L459-461) — used for HEARTBEAT and RESUME
- On repeated ACK timeouts → reconnect with exponential backoff (RESUME if possible).

## 6. Event delivery (server → client)

The umbrella API publishes events to **NATS**; the gateway consumes them and fans out DISPATCH
frames to the relevant session sockets. This is why a working instance needs **both** NATS and
the gateway running (the `compose.yaml` gap, see [02-self-hosting-status.md](./02-self-hosting-status.md)).

## Flow diagram

```
client ──GET /.well-known/fluxer──> API        (learn endpoints.gateway)
client ──WSS ?v=1&encoding=json&compress=...──> gateway
gateway ──HELLO{heartbeat_interval}──────────> client
client  ──IDENTIFY{token,properties,presence}─> gateway
gateway ──RPC type:"session" token───────────> API   (200 ok / 401 / 429)
        ◀── READY / DISPATCH events (via NATS) ──
client  ──HEARTBEAT{seq}──> gateway ──HEARTBEAT_ACK──> client   (every interval)
```
