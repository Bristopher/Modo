# 10 — Turnkey Localhost (Docker)

A one-origin, single-command local instance built on top of the extended compose
([09](./09-docker-compose-extended.md)). Adds a **Caddy** front door + **Mailpit**, and seeds
secrets automatically, so you can register and chat without external DNS, TLS, or API keys.

> **Not yet runtime-verified.** Everything below is staged and syntax/seed-validated, but the
> Docker daemon was down during authoring, so the stack has not actually been booted. First run
> is the verification step — see "When you come back to run it".

## What this adds

| File | Purpose |
| --- | --- |
| `config/config.localhost.template.json` | Combined-mode config: `base_domain=localhost`, `http`, `:8080`, marketing off, Mailpit email, no search/voice. |
| `config/Caddyfile` | Front door on `:8080`; `/gateway*`→gateway (strip prefix), `/mailpit/*`→mailpit, everything else→umbrella. Mirrors `dev/Caddyfile.dev`. |
| `compose.localhost.yaml` | Override adding `caddy` + `mailpit`, and `!reset []` on the server/gateway host ports so Caddy is the sole entrypoint. |
| `scripts/seed_config.mjs` | Node (no `jq`) seeder — fills all secrets + a VAPID keypair into `config.json`. The production equivalent of dev's auto-seed. |

Validated: `docker compose -f compose.yaml -f compose.localhost.yaml config -q` parses; default
services = caddy, nats, valkey, fluxer_server, fluxer_gateway, mailpit; only `:8080` is published.

## Why these choices

- **Single origin / path routing.** The client derives every endpoint from
  `domain.{base_domain,public_scheme,public_port}` (`packages/config/src/EndpointDerivation.tsx`):
  `ws://localhost:8080/gateway`, `http://localhost:8080/api`, `/media`, and the SPA at `/`. Caddy
  must listen on the same port as `public_port`. No subdomains, no `endpoint_overrides` needed.
- **Combined mode.** The umbrella (`fluxer_server`) serves api + admin + app_proxy + media + the
  SPA in one process. `internal` is only schema-required for `instance.deployment_mode:
  microservices`; we don't set that, so combined mode applies. (`internal.kv` is still set for
  Valkey.)
- **Marketing off.** The umbrella does **not** bundle `@fluxer/marketing`
  (`fluxer_server/package.json`), so `services.marketing.enabled=false` avoids a missing service.
- **Mailpit included.** Account registration emails a verification code; without an SMTP sink you
  can't complete signup. Mailpit captures it at `/mailpit/`.
- **NATS no-auth.** Internal Docker network only (no host port). `services.nats.auth_token=""`.
  Enable token auth later for hardening (see [09](./09-docker-compose-extended.md)).
- **Services not host-exposed.** Only Caddy binds the host; server/gateway/nats/valkey/mailpit
  talk over the compose network. `!reset []` needs Docker Compose v2.24+ (you have v2.39).

## When you come back to run it

After starting Docker Desktop:

```bash
cd H:/Code/Projects/Fluxer/Pre-Self-Hosting

# 1. Create + seed the runtime config (writes config/config.json)
node scripts/seed_config.mjs config/config.json --from config/config.localhost.template.json

# 2. Build the gateway image (first build compiles an Erlang OTP release — minutes)
docker compose -f compose.yaml -f compose.localhost.yaml build fluxer_gateway

# 3. Boot the stack
docker compose -f compose.yaml -f compose.localhost.yaml up -d

# 4. Watch health
docker compose -f compose.yaml -f compose.localhost.yaml ps
docker compose -f compose.yaml -f compose.localhost.yaml logs -f fluxer_server fluxer_gateway
```

Then:
- App: `http://localhost:8080/`
- Discovery: `http://localhost:8080/.well-known/fluxer` (check `endpoints.gateway` == `ws://localhost:8080/gateway`)
- Verification emails: `http://localhost:8080/mailpit/`

Tear down: `docker compose -f compose.yaml -f compose.localhost.yaml down` (add `-v` to wipe data).

## Likely first-run issues to watch for (unverified)

1. **Umbrella serving the SPA at `/`** — assumed combined mode serves the built app via
   app_proxy. If `/` 404s, the app assets may need a separate build/serve step; check
   `fluxer_server` Routes + `deploy-app.yaml` for the prod asset path.
2. **Missing `integrations.search`** — omitted to avoid running Meilisearch. If the server hard-
   requires it, add it back and run `--profile search` (note: that also starts Elasticsearch).
3. **S3/media** — `s3.endpoint` points at the umbrella's own `:8080/s3`; confirm the embedded S3
   shim is enabled in the image, else media uploads fail and you'll want MinIO.
4. **Gateway build** — needs network to pull `erlang:28-slim` + rebar3 deps.
5. **First-run DB init** — dev uses `dev_bootstrap.sh` for migrations; the umbrella image should
   self-migrate on start, but verify SQLite at the `fluxer_data` volume gets created.

Capture whatever actually happens on first boot back into this file + [07](./07-self-hosting-roadmap.md).
