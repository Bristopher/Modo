# 12 — Setup, Changing Domain & Moving Machines

The operational guide. Three procedures: **fresh setup**, **change the domain**, and **move to a
new machine**. Read the mental model first — it explains *why* a domain change needs a rebuild and
*what* you must never lose when migrating.

---

## Mental model: build-time vs runtime vs proxy

The domain lives in **three** places and they must all agree:

| Where | What it is | When it's read | Change requires |
| --- | --- | --- | --- |
| **`.env`** (`BASE_DOMAIN`, `PUBLIC_SCHEME`, `PUBLIC_PORT`) | Feeds the `FLUXER_BUILD_CONFIG` build arg | At **`docker build`** — rspack hardcodes api/gateway/media URLs into the SPA bundle | **Rebuild** `fluxer_server` |
| **`config/config.json`** (`domain` block, `s3.endpoint`, email `from_email`) | Runtime server config | At **container start** | Restart |
| **NPM / reverse proxy** (proxy host + TLS cert) | Public TLS entrypoint | Live | Edit proxy host |

> The big gotcha: the **frontend bakes the domain in at build time**. If you change the domain in
> `config.json` but don't rebuild, the SPA keeps calling the *old* URLs and the app breaks. **Domain
> change = rebuild, not just restart.**

What is **NOT** domain-specific (safe to keep across domain changes): the secrets, the database, and
uploaded files.

What you must **NEVER regenerate** once you have real users (doing so breaks sessions, push
notifications, signed media URLs, admin login):
- `services.media_proxy.secret_key`
- `services.admin.secret_key_base`, `services.admin.oauth_client_secret`
- `auth.sudo_mode_secret`, `auth.connection_initiation_secret`, `auth.vapid.*`
- `s3.access_key_id`, `s3.secret_access_key`

These all live in `config/config.json`. **Back it up.** It is the keyring for your instance.

---

> **Compose files — which `-f` flags to use.** Every `docker compose` command below shows the
> localhost pair. **On Unraid, append `-f compose.unraid.yaml`** to each, e.g.:
> ```bash
> docker compose -f compose.yaml -f compose.localhost.yaml -f compose.unraid.yaml up -d
> ```
> That override joins `caddy` to your custom `bignet` network and drops the published host port, so
> Nginx Proxy Manager (also on `bignet`) proxies straight to **`caddy:8080`** — container name +
> internal port, no host IP. `bignet` must already exist (`docker network create bignet`).

## A. Fresh setup (new machine, new domain)

### Prerequisites
- Docker + Docker Compose v2.24+ (Unraid: the Docker service + the Compose Manager plugin, or run
  compose from the CLI).
- Node 18+ on the machine where you run the one-time seed script (only needed to generate secrets).
- A reverse proxy (Nginx Proxy Manager / SWAG) and a domain pointing at your WAN IP.

### 1. Get the patched source onto the machine
Copy this whole project directory (it contains all the self-host patches) to the host — e.g. an
Unraid `appdata` share. If cloning fresh from upstream instead, you'd have to re-apply the patches
in [11-build-patches-and-unraid.md](./11-build-patches-and-unraid.md), so prefer copying this tree.

```bash
# avoids a git-lfs smudge hang if you ever do clone upstream:
GIT_LFS_SKIP_SMUDGE=1 git clone https://github.com/fluxerapp/fluxer.git
```

### 2. Set the domain in `.env`
Edit `.env` — comment the localhost block, set production:
```env
BASE_DOMAIN=fluxer.bigweld.duckdns.org
PUBLIC_SCHEME=https
PUBLIC_PORT=443
HOST_HTTP_PORT=8080
```

### 3. Make a config template for your domain, then seed
The seeder fills secrets but **does not** set the domain — that comes from the template. Either edit
`config/config.duckdns.template.json` (it's already set to `fluxer.bigweld.duckdns.org`) or copy it:

```bash
cp config/config.duckdns.template.json config/config.mydomain.template.json
# edit base_domain, s3.endpoint, and email from_email in that file to your domain
node scripts/seed_config.mjs config/config.json --from config/config.mydomain.template.json
```

This writes `config/config.json` with fresh secrets + a VAPID keypair. **Save this file somewhere
safe** (it's gitignored).

### 4. Build (domain baked in) and boot
```bash
docker compose -f compose.yaml -f compose.localhost.yaml build fluxer_gateway fluxer_server
docker compose -f compose.yaml -f compose.localhost.yaml up -d
docker compose -f compose.yaml -f compose.localhost.yaml ps
```

### 5. Reverse proxy + ports
- Router: forward **80** and **443** TCP → your NPM host. (Not 8080 — it's never published.)
- NPM (must be on `bignet`) proxy host: `your.domain` → scheme `http`, forward host **`caddy`**,
  port **`8080`**, **Websockets ON**, request a Let's Encrypt cert (DuckDNS DNS challenge is
  easiest), Force SSL on. Advanced: `client_max_body_size 100M;`

### 6. First account = admin
Register in the app, get the code from `https://your.domain/mailpit/`, done. First user gets `/admin`.

---

## B. Change the domain (new domain, same machine & data)

You keep your database, uploads, and secrets — only the public hostname changes.

**Checklist:**

1. **`.env`** — set the new `BASE_DOMAIN` (and `PUBLIC_SCHEME`/`PUBLIC_PORT` if they change).
2. **`config/config.json`** — edit, *in place* (do **not** re-seed; that would touch secrets):
   - `domain.base_domain`
   - `s3.endpoint` (`https://NEW.domain/s3`)
   - `integrations.email.from_email` (optional)
   Leave every secret untouched.
3. **Rebuild** the server so the SPA picks up the new URLs:
   ```bash
   docker compose -f compose.yaml -f compose.localhost.yaml build fluxer_server
   docker compose -f compose.yaml -f compose.localhost.yaml up -d
   ```
   (No need to rebuild `fluxer_gateway` — it's domain-agnostic.)
4. **Reverse proxy** — update the proxy host's Domain Names to the new hostname and request a new
   cert for it. Update router forwards only if the public IP/ports changed.
5. **DNS** — point the new domain at your WAN IP (DuckDNS wildcard subdomains resolve automatically).

> Only changing the **subdomain** under the same DuckDNS base (e.g. `fluxer.` → `chat.`)? Same steps
> — DuckDNS already resolves `*.bigweld.duckdns.org`, so DNS needs nothing; just `.env` +
> `config.json` + rebuild + NPM hostname/cert.

**Quick edit helper** (in-place domain swap in config.json without disturbing secrets):
```bash
node -e "const f='config/config.json',c=JSON.parse(require('fs').readFileSync(f)); \
  const d='chat.bigweld.duckdns.org'; \
  c.domain.base_domain=d; c.s3.endpoint='https://'+d+'/s3'; \
  if(c.integrations?.email) c.integrations.email.from_email='noreply@'+d; \
  require('fs').writeFileSync(f, JSON.stringify(c,null,'\t')); console.log('updated to',d)"
```

---

## C. Move to a new machine (keep everything)

You're migrating the instance — same domain (or change it too via section B after). You must carry
over **three** things: the **patched source tree**, **`config/config.json`** (secrets), and the
**data volume** (SQLite DB + uploaded files).

### What holds the data
Our compose uses a named volume **`fluxer_data`** mounted at `/usr/src/app/data` inside the
container. It contains:
- `fluxer.db` — the SQLite database (users, messages, guilds, …)
- `s3/` — uploaded files / attachments / avatars

### 1. On the OLD machine — stop and back up
```bash
docker compose -f compose.yaml -f compose.localhost.yaml down

# back up the config (secrets) — just copy the file
cp config/config.json ~/fluxer-config-backup.json

# back up the data volume to a tarball
docker run --rm -v fluxer_data:/data -v "$PWD":/backup alpine \
  tar czf /backup/fluxer_data.tgz -C /data .
```
You now have: the project dir, `fluxer_data.tgz`, and `config/config.json`. Copy all three to the
new machine.

### 2. On the NEW machine — restore
```bash
# put the project tree in place, with config/config.json restored (same secrets!)
# recreate the named volume and load the tarball into it:
docker volume create fluxer_data
docker run --rm -v fluxer_data:/data -v "$PWD":/backup alpine \
  tar xzf /backup/fluxer_data.tgz -C /data

# build (image isn't transferable unless you pushed it — see 11's Docker Hub note) and boot
docker compose -f compose.yaml -f compose.localhost.yaml build fluxer_gateway fluxer_server
docker compose -f compose.yaml -f compose.localhost.yaml up -d
```
If the domain is unchanged, `.env` and `config.json` already match — you're done once the reverse
proxy on the new machine points at it. If the domain changes too, also do section B.

> **Unraid tip:** instead of the named volume you can bind-mount data to a share for easier
> backup/snapshotting. In `compose.yaml`, change the `fluxer_server` volume
> `fluxer_data:/usr/src/app/data` to `/mnt/user/appdata/fluxer/data:/usr/src/app/data` (and drop
> `fluxer_data` from the `volumes:` block). Then your DB + uploads live under
> `/mnt/user/appdata/fluxer/` and the Unraid backup plugins cover them. Do this **before** first
> boot, or migrate the volume contents into the share first.

### Don't forget
- **`config/config.json` must come with you.** A fresh seed makes *new* secrets → all existing
  sessions log out, push notifications die, previously-signed media URLs break, admin OAuth resets.
- The **built image** is machine-local unless you pushed it to a registry. Either rebuild on the new
  host (the patches are in the source tree) or pull your pushed tag (see [11](./11-build-patches-and-unraid.md)).

---

## Backups (do this regularly, not just on migration)

Two things, both small:
```bash
# 1. the keyring
cp config/config.json /path/to/backups/fluxer-config-$(date +%F).json   # bash; on PowerShell adjust

# 2. the data volume
docker run --rm -v fluxer_data:/data -v "$PWD":/backup alpine \
  tar czf /backup/fluxer_data-$(date +%F).tgz -C /data .
```
Restore = section C step 2. Test a restore at least once before you rely on it.

---

## Troubleshooting domain changes

| Symptom | Cause | Fix |
| --- | --- | --- |
| App loads but login/API calls fail or hit the old URL | Changed `config.json`/`.env` but didn't rebuild | Rebuild `fluxer_server` (the domain is compiled into the SPA) |
| Realtime / typing / new messages don't arrive live | Websockets not enabled on the proxy host, or `/gateway` not reaching the gateway | Turn on Websockets in NPM; check `fluxer_gateway` is healthy |
| Cert errors / "not secure" | Cert is for the old hostname | Request a new Let's Encrypt cert for the new domain in NPM |
| Emoji/fonts missing | CSP / static CDN — should be patched already | Confirm the patch in [11](./11-build-patches-and-unraid.md) (#11) is present |
| Uploads vanish after a rebuild | `s3.data_dir` relative, or data on a non-persistent path | Ensure `services.s3.data_dir` is absolute and `fluxer_data` volume is mounted |
