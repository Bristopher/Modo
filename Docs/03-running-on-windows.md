# 03 — Running on Windows (Path A)

Goal: stand up a working Fluxer instance on this Windows 11 machine.

## Environment audit (this machine, 2026-06-12)

| Tool | Found | Notes |
| --- | --- | --- |
| Docker | 28.3.3 + Compose v2.39.2 | **Daemon not running** — Docker Desktop must be started (Linux engine). |
| WSL2 | Ubuntu, default v2 | ✅ Available — this is the recommended route. |
| Node | v22.19.0 | ⚠️ Repo requires **Node ≥24** (`fluxer_server` engines + devenv uses `nodejs_24`). Local Node is too old for from-source runs outside devenv. |
| pnpm | 10.29.3 | ✅ Matches `packageManager` pin. |

Two implications:
1. **Nix/devenv do not run natively on Windows** → the supported path must run **inside WSL2**.
2. Your host Node (22) is below the required 24, so don't try to `pnpm dev` from PowerShell;
   devenv provides its own Node 24 inside the Nix shell.

## Recommended path: devenv inside WSL2 (full, working stack)

This is the only path that boots the **complete** stack (umbrella + Erlang gateway + NATS +
infra) — see [02-self-hosting-status.md](./02-self-hosting-status.md).

### Steps

1. **Open WSL2 Ubuntu.**

2. **Clone inside the Linux filesystem**, not under `/mnt/h` (huge perf penalty for
   node_modules + Nix; also avoids Windows line-ending/permission issues):
   ```bash
   cd ~
   git clone https://github.com/fluxerapp/fluxer.git
   cd fluxer
   ```
   > The Windows-side clone at `H:\Code\Projects\Fluxer\Pre-Self-Hosting` is fine for reading
   > code and keeping these Docs, but run the stack from the WSL copy.

3. **Install Nix + devenv** following https://devenv.sh/getting-started/ (Determinate Systems
   installer is the easy option). Then optionally `direnv` + `direnv allow` (repo ships `.envrc`).

4. **Create the runtime config** the processes expect at `config/config.json`. For local dev,
   start from the dev template:
   ```bash
   cp config/config.dev.template.json config/config.json
   ```
   The dev template targets `localhost:48763` and leaves most secrets blank (dev tolerates
   empty secrets + `dev.disable_rate_limits: true`). See [04-config-reference.md](./04-config-reference.md).
   `devenv.nix` sets `FLUXER_CONFIG=<repo>/config/config.json` automatically.

5. **Boot the stack:**
   ```bash
   devenv shell      # enters the Nix env (Node 24, Erlang 28, Rust+wasm, etc.)
   devenv up         # runs the fluxer:bootstrap task then all processes
   ```
   `devenv up` runs `scripts/dev_bootstrap.sh` first (DB/migrations/asset setup), then starts:
   caddy, fluxer_app, fluxer_server, fluxer_gateway, marketing_dev, css_watch, valkey,
   nats_core, nats_jetstream, meilisearch, livekit, mailpit.

6. **Open it:** `http://localhost:48763/`
   Dev emails (verification codes etc.) land in Mailpit at `http://localhost:48763/mailpit/`.

### Notes / gotchas
- First `devenv up` is slow: it builds the Rust→wasm client code and compiles the Erlang gateway.
- WebRTC voice on a remote VM behind an HTTP-only tunnel needs extra firewall ports
  (3478/udp, 7881/tcp, 50000-50100/udp) — see README "Voice on a remote VM". For pure localhost
  this isn't an issue.
- Process logs: `dev/logs/*.log` (e.g. `dev/logs/fluxer_gateway.log`). process-compose TUI is
  disabled (`PC_DISABLE_TUI=1`); use the logs.

## Alternative: Devcontainer (no Nix)

The repo has experimental `.devcontainer/` support (VS Code Dev Container / Codespace) that uses
Docker Compose + process-compose instead of Nix:
```bash
process-compose -f .devcontainer/process-compose.yml up
```
App at `http://localhost:48763`, Mailpit at `/mailpit/`. Bluesky OAuth is disabled there (needs
HTTPS). This still requires the Docker daemon to be running.

## Alternative: `compose.yaml` (NOT recommended yet)

`docker compose up` with the published `fluxer-server:stable` image is the lightest path but is
**incomplete**: no NATS, no Erlang gateway → no realtime messaging. Don't use it for a real
instance until extended (see [07-self-hosting-roadmap.md](./07-self-hosting-roadmap.md)). If you
only want to poke at the HTTP API surface, start Docker Desktop first, supply
`config/config.json` from the production template, and:
```bash
docker compose up valkey fluxer_server
# optional profiles:
docker compose --profile search --profile voice up
```

## Recommendation

Use **devenv in WSL2**. It's the maintainer-supported path and the only one that yields a
working chat instance today.
