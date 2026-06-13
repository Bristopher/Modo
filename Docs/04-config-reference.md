# 04 — Config Reference

Fluxer reads a single JSON config file. The path comes from the **`FLUXER_CONFIG`** env var
(devenv sets it to `<repo>/config/config.json`; the Docker image expects
`/usr/src/app/config/config.json`). The JSON Schema lives at
`packages/config/src/ConfigSchema.json` (referenced via `config/config.schema.json`).

Templates in `config/`:
- `config.dev.template.json` — local dev (localhost:48763, blank secrets OK)
- `config.production.template.json` — production starting point (placeholders to fill)
- `config.test.json` — test runner config
- `livekit.example.yaml` — LiveKit server config sample

## Top-level sections

| Section | Purpose |
| --- | --- |
| `env` | `development` \| `production` |
| `domain` | `base_domain`, `public_scheme`, `public_port` — how clients address the instance |
| `database` | `backend` (`sqlite` default), `sqlite_path` (e.g. `./data/fluxer.db`); Cassandra optional |
| `internal` | `kv` (Valkey/Redis URL), `kv_mode` (`standalone`) |
| `s3` | `access_key_id`, `secret_access_key`, `endpoint` — object storage for media |
| `services` | per-service config: `server`, `media_proxy`, `admin`, `marketing`, `gateway`, `nats` |
| `auth` | `sudo_mode_secret`, `connection_initiation_secret`, `vapid` keypair, optional `bluesky` |
| `integrations` | `search` (Meilisearch), `email`/`smtp`, `voice` (LiveKit), `gif` (klipy/tenor) |
| `discovery`, `dev`, `instance`, `federation` | misc (dev rate-limit toggle, federation flag, instance key path) |

## Services block detail

```jsonc
"services": {
  "server":       { "port": 8080, "host": "0.0.0.0" },          // umbrella HTTP
  "media_proxy":  { "secret_key": "<64-hex>" },                  // signs media URLs
  "admin":        { "secret_key_base": "<64-hex>",
                    "oauth_client_secret": "<64-hex>" },
  "marketing":    { "enabled": true, "secret_key_base": "<64-hex>" },
  "gateway":      { "port": 8082,                                // Erlang WS service
                    "admin_reload_secret": "<64-hex>",
                    "media_proxy_endpoint": "http://127.0.0.1:8080/media" },
  "nats":         { "core_url": "nats://nats:4222",
                    "jetstream_url": "nats://nats:4222",
                    "auth_token": "<token>" }
}
```

> Note: the **gateway** and **nats** are configured here as separate network services. The
> umbrella `fluxer_server` does not embed them. This is why `compose.yaml` (which omits both) is
> incomplete — see [02-self-hosting-status.md](./02-self-hosting-status.md).

## Endpoints / discovery

The API exposes the canonical discovery document at **`GET /.well-known/fluxer`**
(`packages/api/src/instance/InstanceController.tsx:34`). It returns `endpoints.{gateway, media,
static_cdn, marketing, admin, invite, gift, webapp, ...}` sourced from `Config.endpoints.*`.
Clients fetch this to learn where the gateway WebSocket lives — see
[06-client-gateway-connection.md](./06-client-gateway-connection.md).

## Secrets checklist (production)

Generate independently; never reuse:

```bash
# 64-char hex secret (repeat for each *_secret / secret_key / secret_key_base)
openssl rand -hex 32

# VAPID keys for Web Push (auth.vapid)
npx web-push generate-vapid-keys
```

Fields to fill in `config.production.template.json`:
- `s3.access_key_id`, `s3.secret_access_key`
- `services.media_proxy.secret_key`
- `services.admin.secret_key_base`, `services.admin.oauth_client_secret`
- `services.marketing.secret_key_base`
- `services.gateway.admin_reload_secret`
- `services.nats.auth_token`
- `auth.sudo_mode_secret`, `auth.connection_initiation_secret`
- `auth.vapid.public_key`, `auth.vapid.private_key`
- `integrations.search.api_key` (Meilisearch master key)

## Dev vs production differences

| | dev template | production template |
| --- | --- | --- |
| `env` | `development` | `production` |
| `base_domain` | `localhost` (port 48763) | `chat.example.com` (https/443) |
| Secrets | mostly blank, allowed | all must be set |
| `dev.disable_rate_limits` | `true` | (absent) |
| Email | Mailpit SMTP (localhost:49621) | real SMTP provider |
| Extra dev keys | `bluesky`, `klipy`, `tenor`, `voice` wired for local | minimal |
| Ports | 49xxx range (Caddy fronts 48763) | 8080 server / 8082 gateway |
