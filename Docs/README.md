# Fluxer Self-Hosting — Working Docs

Internal research notes for standing up and continuing development of a self-hosted Fluxer
instance from the public monorepo (`github.com/fluxerapp/fluxer`).

> Status as of **2026-06-13**: the upstream repo explicitly says self-hosting is **pre-release**
> ("TBD" in the README), and the published server image is private + the Dockerfile is stale. We
> applied **~14 source patches** (cross-referenced against two community guides) so the image
> builds and runs from source — see **[11-build-patches-and-unraid.md](./11-build-patches-and-unraid.md)**.
> These docs capture what works today, what is missing, and concrete next steps.
> See [02-self-hosting-status.md](./02-self-hosting-status.md).

## Index

| Doc | What it covers |
| --- | --- |
| **[RUNNING.md](./RUNNING.md)** | **Start here to run it** — step-by-step Docker (localhost) and devenv (WSL2) instructions + troubleshooting |
| [01-architecture-overview.md](./01-architecture-overview.md) | Monorepo map, every service, ports, and how they fit together |
| [02-self-hosting-status.md](./02-self-hosting-status.md) | What works today, the critical gap in `compose.yaml`, what's blocking a clean self-host |
| [03-running-on-windows.md](./03-running-on-windows.md) | Path A — standing it up on this Windows 11 box (devenv/WSL2 vs Docker), env audit |
| [04-config-reference.md](./04-config-reference.md) | `config.json` breakdown, every secret you must generate, dev vs prod templates |
| [05-auth-flow.md](./05-auth-flow.md) | Login, token format, password hashing, session issuance/validation (with file refs) |
| [06-client-gateway-connection.md](./06-client-gateway-connection.md) | How the React client discovers + connects to the Erlang gateway, handshake, heartbeat |
| [07-self-hosting-roadmap.md](./07-self-hosting-roadmap.md) | Concrete TODO list to make a complete, reproducible self-host |
| [08-bootstrap-and-deployment.md](./08-bootstrap-and-deployment.md) | What `devenv up` bootstraps (auto-seeded secrets/keys) + how upstream CI ships images (gateway is build-only) |
| [09-docker-compose-extended.md](./09-docker-compose-extended.md) | The extended `compose.yaml` (added NATS + gateway) — what changed, how to bring it up, remaining caveats |
| [10-turnkey-localhost.md](./10-turnkey-localhost.md) | One-command local instance: Caddy + Mailpit override, localhost config, secret seeder, run steps (**not yet runtime-verified**) |
| [11-build-patches-and-unraid.md](./11-build-patches-and-unraid.md) | **The ~14 source patches** that make the image build/run (from the PaulMColeman + mgabor3141 guides) + an **Unraid + reverse-proxy + duckdns** deployment walkthrough |
| [12-setup-and-migration.md](./12-setup-and-migration.md) | **Operations guide** — fresh setup, **changing the domain** (why it needs a rebuild), **moving to a new machine** (what secrets/data to carry), and backups |
| [13-supply-chain-security.md](./13-supply-chain-security.md) | **npm supply-chain safety** — the built-in pnpm defenses (24h release-age gate, build-script allowlist), a lockfile scan vs. the 2025 compromised packages, and why our patches don't widen the attack surface |

## Quick orientation

- **One umbrella backend**: `fluxer_server` (TypeScript) bundles api + admin + app_proxy +
  media_proxy + marketing into a single Node process.
- **One separate realtime service**: `fluxer_gateway` (Erlang/OTP) — the WebSocket gateway.
  It is **not** part of the umbrella and is **not** in `compose.yaml`.
- **Client**: `fluxer_app` (React), served behind Caddy in dev.
- **Infra**: Valkey, NATS (core + JetStream), Meilisearch, LiveKit, plus Mailpit in dev.

The only fully-supported way to run the complete stack today is **`devenv up`** (Nix), which
orchestrates all of the above via process-compose. The Docker `compose.yaml` is incomplete.
