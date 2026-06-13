# 08 — Bootstrap & Deployment Model

What `devenv up` actually does before the processes start, and how upstream ships images. This
resolves most of the open questions from [07-self-hosting-roadmap.md](./07-self-hosting-roadmap.md).

## `scripts/dev_bootstrap.sh` (the `fluxer:bootstrap` task)

Runs once before the process-compose processes (see `devenv.nix` `tasks."fluxer:bootstrap"`).
Steps, in order (`main()` at end of script):

1. **`prepare_log_dir`** — `mkdir -p dev/logs`.
2. **`check_config`** — if `config/config.json` is missing, **copies it from
   `config.dev.template.json`**. So you don't have to create it manually for dev.
3. **`ensure_core_secrets`** — **auto-generates all dev secrets** with `node:crypto` and writes
   them into `config.json` via `jq`, *only* where the value is empty or a known placeholder.
   Covers: `s3.access_key_id/secret_access_key`, `services.media_proxy.secret_key`,
   `services.admin.secret_key_base/oauth_client_secret`, `services.marketing.secret_key_base`,
   `services.gateway.admin_reload_secret`, `services.queue.secret` (if present),
   `integrations.search.api_key` (Meili), `auth.sudo_mode_secret`,
   `auth.connection_initiation_secret`, SMTP password, LiveKit `voice.api_key/api_secret`.
   Also **deletes a deprecated top-level `.gateway`** key if present, and syncs the Meili master
   key to `dev/meilisearch_master_key`.
4. **`ensure_vapid_keys`** — generates an EC P-256 **VAPID keypair** (Web Push) if missing/invalid.
5. **`generate_bluesky_oauth_keys`** — generates an ES256 (P-256) PKCS#8 key at
   `dev/bluesky_oauth_key.pem` and wires `auth.bluesky.keys`.
6. **`generate_livekit_config`** — renders `dev/livekit.yaml` from `dev/livekit.template.yaml`
   (API key/secret, webhook, node IP). For non-localhost `base_domain` it resolves the public IP
   via ifconfig.me/ipify for direct media.
7. **`setup_model_symlink`** — symlinks the NSFW ONNX model
   `fluxer_media_proxy/data/model.onnx` → `fluxer_server/data/model.onnx`. **If the model file is
   absent, NSFW detection is disabled but boot continues.**

### Implication for self-hosting
Dev "just works" because bootstrap seeds everything. A **production** deploy has no equivalent
auto-seed step in `compose.yaml` — you must generate secrets + VAPID + LiveKit config yourself
(see [04-config-reference.md](./04-config-reference.md)). A production bootstrap script is a gap
worth filling (roadmap P2). Note the bootstrap also references config keys not in the prod
template: **`services.queue.secret`** and the deprecated top-level **`.gateway`** — confirm the
current prod shape against `packages/config/src/ConfigSchema.json` before scripting it.

## How upstream builds & ships (CI, `.github/workflows/`)

| Component | Workflow | Where it goes |
| --- | --- | --- |
| **fluxer-server** (umbrella) | `release-server.yaml` | **Public GHCR**: `ghcr.io/fluxerapp/fluxer-server` (tags: `latest`, `nightly`, `vX`, `sha-…`). This is the only publicly pulled image. |
| **gateway** | `deploy-gateway.yaml` / `restart-gateway.yaml` | **Not public.** Deploys over **SSH to the maintainer's own server** (`webfactory/ssh-agent`, `secrets.SERVER_IP/SERVER_USER`), building/running the image on that host. No public registry tag. |
| admin / api / app / marketing / media-proxy / static-proxy | `deploy-*.yaml` | `docker/build-push-action` to the deploy target (private infra) — these mirror the umbrella's constituent services for the production cluster. |
| relay / relay-directory | `release-relay*.yaml` | GHCR (`REGISTRY: ghcr.io`). |
| livekitctl | `release-livekitctl.yaml` | release artifact. |
| desktop | `build-desktop.yaml` | desktop app build. |
| Cassandra | `migrate-cassandra.yaml`, `test-cassandra-backup.yaml` | DB migration/backup jobs. |

### Key takeaways
- **The Erlang gateway has no public image.** A self-hoster must build it from
  `fluxer_gateway/Dockerfile` (erlang:28-slim → rebar3 prod release; `EXPOSE 8080 8081`; entry
  `scripts/docker_entrypoint.sh`, which `envsubst`s `config/sys.config.template`).
- The published `fluxer-server:stable` umbrella + a locally-built gateway + NATS + Valkey is the
  minimum set for a working Dockerized instance. `compose.yaml` currently ships only the first two
  of those four.
- Production runs as **separate microservice images** (admin/api/app/media-proxy deployed
  individually); the **umbrella image is the self-hoster convenience packaging** of the same TS
  services.

## Gateway Docker entry

`fluxer_gateway/Dockerfile`:
- build stage `erlang:28-slim`, installs rebar3 3.24.0, `rebar3 compile --deps_only`, then
  `rebar3 as prod release` after rendering `config/sys.config` from
  `config/sys.config.template` via `envsubst` (LOGGER_LEVEL arg).
- runtime stage copies `_build/prod/rel/fluxer_gateway`, runs as non-root `fluxer` user,
  `EXPOSE 8080 8081`, entrypoint `bin/docker_entrypoint.sh`.

To add it to compose: build context `./fluxer_gateway`, mount/inject the gateway config, expose
its port, and point the umbrella's `services.gateway.*` + clients' discovered `endpoints.gateway`
at it.
