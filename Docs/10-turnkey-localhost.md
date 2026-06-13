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

## Config rationale (verified against the code)

These choices were checked against `ServiceInitializer.tsx`, `ConfigLoader.tsx`, and the JSON
schema (`packages/config/src/schema/`), not guessed:

- **`services.server.static_dir: /usr/src/app/assets` is mandatory.** The SPA only mounts at `/`
  when `static_dir` is set (`Routes.tsx:158`, `ServiceInitializer.tsx:408`). It has **no schema
  default** ("Required in production"). The Dockerfile builds the app to `/usr/src/app/assets`,
  but its `ENV FLUXER_SERVER_STATIC_DIR=...` is **dead** — the loader only honors env vars
  prefixed `FLUXER_CONFIG__` (`EnvironmentOverrides.tsx:68`). So it must be in `config.json`.
  Without it you get an API-only server and `/` 404s.
- **Monolith mode is the default.** `instance.deployment_mode` defaults to `monolith`
  (`schema/defs/instance.json`) → `isMonolith=true`, single process, S3 mounted at `/s3`, media
  proxy in "public-only" mode. We don't set `instance`, so this applies. `internal.kv` is still
  required (`Config.tsx:24`) and is set.
- **No Meilisearch needed.** Search omitted intentionally; the image targets SQLite search.
  `integrations` defaults to `{}` and isn't required at the root. (If a run shows search errors,
  add `integrations.search` + `--profile search`.)
- **S3/media need no extra config.** Global `s3` only requires `access_key_id` +
  `secret_access_key` (seeded). `region` (`local`), `endpoint`, and all `buckets` (cdn=`fluxer`,
  uploads=`fluxer-uploads`, …) come from schema defaults; `services.s3.{data_dir,host,port}`
  default too (`./data/s3`, `0.0.0.0:3900`). The embedded S3 is mounted at `/s3` in monolith mode.
- **Secrets/keys** filled by `seed_config.mjs`; admin/proxy/cookie all have schema defaults.

> Override any config field via env without editing JSON using the `FLUXER_CONFIG__` prefix and
> `__` as the path separator, e.g. `FLUXER_CONFIG__SERVICES__SERVER__STATIC_DIR=/usr/src/app/assets`.

## Still to confirm on first real boot

1. **Gateway build** — needs network to pull `erlang:28-slim` + rebar3 deps (first build only).
2. **DB auto-migration** — the umbrella should create/migrate SQLite at the `fluxer_data` volume
   on start (no `dev_bootstrap` in prod). Watch `fluxer_server` logs for migration output.
3. **Presigned media URLs** — `s3.presigned_url_base` defaults to the internal endpoint
   (`127.0.0.1:8080/s3`). In monolith mode media is served signed via `/media`, so this should be
   fine, but verify image/attachment loading.
4. **Realtime** — confirm `fluxer_gateway` is healthy and reaching `nats` (RPC subject `rpc.api`).

Capture whatever actually happens on first boot back into this file + [07](./07-self-hosting-roadmap.md).
