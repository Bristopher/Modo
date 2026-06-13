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

Status: **largely done** — see [09-docker-compose-extended.md](./09-docker-compose-extended.md).

- [x] **Add NATS** (JetStream) matching `services.nats.*`. Done (internal-only, `nats_data` vol).
- [x] **Add the Erlang gateway** service. Done — built locally from `./fluxer_gateway`
      (confirmed no public image exists), exposes 8082, healthchecks `/_health`. RPC + events
      run over NATS, not HTTP.
- [x] **Provide a `.env.example`.** Done (`FLUXER_GATEWAY_PORT`, `NATS_AUTH_TOKEN`, etc.).
- [ ] Decide media storage: the prod template points `s3.endpoint` at the server's own
      `:8080/s3` shim — confirm that embedded S3 is enabled in the image, or add MinIO.
- [ ] Add a front proxy (Caddy/Traefik) terminating TLS and routing `/`, `/api`, `/media`,
      and the gateway WebSocket to the right services, so `domain.base_domain` works over HTTPS.
- [ ] Add a `config.json` generator/bootstrap that fills secrets + VAPID for production (dev's
      auto-seed has no compose equivalent).

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

## Open questions — status

Resolved (see [08-bootstrap-and-deployment.md](./08-bootstrap-and-deployment.md)):

- [x] **Is `fluxer-gateway` published to GHCR?** **No.** Only `fluxer-server` is on public GHCR
      (`release-server.yaml`). The gateway deploys over SSH to private infra
      (`deploy-gateway.yaml`). Self-hosters **must build it** from `fluxer_gateway/Dockerfile`.
- [x] **What does `dev_bootstrap.sh` do?** Creates `config.json` from the dev template,
      auto-seeds all secrets, generates VAPID + Bluesky OAuth keys, renders LiveKit config,
      symlinks the NSFW ONNX model. A production deploy has **no equivalent** — must be scripted.
- [x] **Are dev secrets manual?** No — bootstrap seeds them. **Production secrets are manual.**

Still open:

- [ ] Does the umbrella image embed the S3 shim, or is external object storage mandatory in prod?
      (prod template points `s3.endpoint` at the server's own `:8080/s3`)
- [ ] How are client assets (Rust→wasm, CSS) built/served in prod vs the dev `css_watch` +
      Vite dev server? (`deploy-app.yaml` builds the app image — inspect for the prod asset path)
- [ ] Confirm the current prod config shape vs `packages/config/src/ConfigSchema.json` — the
      bootstrap references `services.queue.secret` and a deprecated top-level `.gateway` not in
      the prod template.
- [ ] Federation/relay (`fluxer_relay*`, `federation.enabled`) — out of scope for a basic
      self-host, but the connection-initiation secret + `/.well-known/fluxer` are already wired.

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
