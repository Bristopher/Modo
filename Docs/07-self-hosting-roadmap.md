# 07 — Self-Hosting Roadmap / Next Steps

Concrete work to get from "runs via devenv on a dev box" to "reproducible self-host anyone can
deploy." Ordered roughly by priority. Re-check against upstream before doing big work — the
maintainer is doing a refactor that will likely ship an official self-host path
(`docs.fluxer.app/self-hosting`).

## P0 — Get a working instance locally (validate the stack)

- [ ] Run the **devenv path in WSL2** end-to-end (see [03-running-on-windows.md](./03-running-on-windows.md)).
- [ ] Confirm: web app loads at `:48763`, account creation works (verification code via Mailpit),
      send a message in a channel, see it arrive in a second browser session (proves
      gateway + NATS realtime path).
- [ ] Capture the actual startup order / bootstrap steps from `scripts/dev_bootstrap.sh` and
      `scripts/dev_gateway.sh` into a runbook.

## P1 — Make the Docker `compose.yaml` actually complete

The current `compose.yaml` cannot run a working instance. To fix:

- [ ] **Add NATS** service (core + JetStream) matching `services.nats.*` in the config.
- [ ] **Add the Erlang gateway** service. Needs a published image
      (`fluxer_gateway/Dockerfile` exists — check whether `ghcr.io/fluxerapp/fluxer-gateway`
      is published; if not, build locally). Wire `services.gateway.port: 8082` and
      `media_proxy_endpoint`.
- [ ] Decide media storage: the prod template points `s3.endpoint` at the server's own
      `:8080/s3` shim — confirm that embedded S3 is enabled in the image, or add MinIO.
- [ ] Add a front proxy (Caddy/Traefik) terminating TLS and routing `/`, `/api`, `/media`,
      and the gateway WebSocket to the right services, so `domain.base_domain` works over HTTPS.
- [ ] Provide a `.env.example` and a `config.json` generator that fills all the secrets from
      [04-config-reference.md](./04-config-reference.md).

## P2 — Reproducible config + secret bootstrap

- [ ] Script secret generation (`openssl rand -hex 32` ×N, VAPID keypair, Meili master key,
      NATS auth token) → emit a filled `config.json` from the production template.
- [ ] Document the minimum viable config (single-node SQLite, no Cassandra, search optional,
      voice optional) vs the full config.
- [ ] Verify which `integrations` are truly optional (gif/klipy/tenor, search, voice) and gate
      them so a bare instance starts without external API keys.

## P3 — Production hardening

- [ ] TLS + real domain (`domain.public_scheme: https`).
- [ ] Real SMTP provider (replace Mailpit) for verification/notification emails.
- [ ] LiveKit deployment for voice/video (`fluxer_devops/livekitctl`) + the UDP/TCP firewall
      ports (3478/udp, 7881/tcp, 50000-50100/udp).
- [ ] Backups for SQLite (`./data/fluxer.db`) + Valkey AOF + Meilisearch data.
- [ ] Decide SQLite vs Cassandra threshold (Cassandra migrations: `fluxer_api/scripts/CassandraMigrate.tsx`).
- [ ] Admin panel pass: set tiers/limits (self-host is fully unlocked, no paywall).

## Open questions to resolve from the code

- [ ] Is `fluxer-gateway` published to GHCR, or must self-hosters compile Erlang? (check
      `.github/workflows` + `fluxer_gateway/Dockerfile`)
- [ ] Does the umbrella image embed the S3 shim, or is external object storage mandatory in prod?
- [ ] What exactly does `scripts/dev_bootstrap.sh` do that a production deploy must replicate
      (DB init, search index creation, key generation, asset build)?
- [ ] How are client assets (Rust→wasm, CSS) built/served in prod vs the dev `css_watch` +
      Vite dev server?
- [ ] Federation/relay (`fluxer_relay*`, `federation.enabled`) — out of scope for a basic
      self-host, but note the connection-initiation secret + `/.well-known/fluxer` are already
      wired.

## Useful source pointers

| Topic | Path |
| --- | --- |
| Umbrella entry | `fluxer_server/src/startServer.tsx`, `fluxer_server/src/index.tsx` |
| Dev orchestration | `devenv.nix`, `scripts/dev_bootstrap.sh`, `scripts/dev_gateway.sh` |
| Docker (incomplete) | `compose.yaml`, `fluxer_server/Dockerfile`, `fluxer_gateway/Dockerfile` |
| Config schema | `packages/config/src/ConfigSchema.json` |
| Discovery doc | `packages/api/src/instance/InstanceController.tsx` |
| Auth | `packages/api/src/auth/**` |
| Gateway | `fluxer_gateway/src/**` (Erlang) |
| Cassandra migrations | `fluxer_api/scripts/CassandraMigrate.tsx` |
| LiveKit bootstrap | `fluxer_devops/livekitctl/` |
