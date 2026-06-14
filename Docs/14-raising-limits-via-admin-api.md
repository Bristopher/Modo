# Raising File-Size / Message-Length Limits Past the Admin-Panel UI Clamps

## What this is

The Fluxer admin panel UI clamps limit inputs (file size capped at **500MB** / `524288000`,
message length at **4000** chars). Those caps are **UI-only**. The backend imposes no maximum,
so you can write higher limits directly through the admin API and they apply live.

This document records exactly how we did it on the self-hosted instance.

## Why it works (code references)

- `packages/schema/src/domains/admin/AdminSchemas.tsx` — `LimitConfigUpdateRequest` validates
  each limit value as `z.number().min(0)`. **No max.**
- `packages/api/src/constants/LimitConfig.tsx` — `sanitizeLimitConfigForInstance` normalizes
  structure and strips the premium tier for self-host, but does **not** clamp values on read.
- `packages/api/src/admin/controllers/LimitConfigAdminController.tsx` — `/admin/limit-config/update`
  calls `limitConfigService.updateConfig(...)`, which applies immediately (no restart).

So the UI clamp is purely client-side. Bypass it by calling the API directly.

## The critical gotcha: the `Admin` auth scheme

Admin API keys (format `fa_<id>_<secret>`) authenticate with the **`Admin `** Authorization
scheme, **NOT `Bearer `**.

See `packages/api/src/middleware/UserMiddleware.tsx` (~line 63): the header parser only routes
`fa_...` keys to admin-key validation when the header reads `Authorization: Admin <key>`.
Sending the key as `Bearer` falls through to the OAuth bearer path → no user resolved → **401**.

This single detail was the cause of repeated `401`/`403` failures before we got it right.

## Endpoints

POST, registered on the `/api` app:

- `POST http://caddy:8080/api/admin/limit-config/get`  — returns `{ limit_config, ... }`
- `POST http://caddy:8080/api/admin/limit-config/update` — body `{ "limit_config": <snapshot> }`

## Procedure

1. In the admin panel → **API Keys**, create an admin API key with the WILDCARD `*` ACL.
2. Run this inside the server container (replace the key). It GETs the current config,
   raises the two limits across every rule, and POSTs it back:

```bash
docker exec -e AK="fa_PASTE_KEY_HERE" fluxer_server node -e '(async()=>{
  const KEY=process.env.AK;
  const FILE=5*1024*1024*1024, MSG=10000;            // 5GB, 10k chars
  const h={"Authorization":"Admin "+KEY,"Content-Type":"application/json"};
  const base="http://caddy:8080/api/admin";
  const g=await fetch(base+"/limit-config/get",{method:"POST",headers:h,body:"{}"});
  console.log("get status",g.status);
  if(!g.ok){console.log((await g.text()).slice(0,300));return}
  const j=await g.json(); const cfg=j.limit_config;
  for(const rule of cfg.rules){
    rule.limits.max_attachment_file_size=FILE;
    rule.limits.max_message_length=MSG;
  }
  const u=await fetch(base+"/limit-config/update",{method:"POST",headers:h,body:JSON.stringify({limit_config:cfg})});
  console.log("update status",u.status);
  console.log((await u.text()).slice(0,200));
})()'
```

Expected output: `get status 200` then `update status 200`. Change applies immediately.

For **30GB**, set `FILE=30*1024*1024*1024`.

## Required companion change — NPM body-size cap

Raising the Fluxer limit alone is not enough. The upload still passes through Nginx Proxy
Manager, whose default `client_max_body_size` (~1MB) rejects large POSTs with `413` **before**
they reach Fluxer.

In **Nginx Proxy Manager → Fluxer proxy host → Advanced → Custom Nginx Configuration**:

```nginx
client_max_body_size 0;
proxy_request_buffering off;
proxy_read_timeout 3600s;
proxy_send_timeout 3600s;
client_body_timeout 3600s;
send_timeout 3600s;
```

- `client_max_body_size 0` — removes the body-size cap entirely.
- `proxy_request_buffering off` — streams the upload instead of buffering the whole file to
  disk first (matters for multi-GB files).
- `client_body_timeout` / `send_timeout` / `proxy_*_timeout` — the **timeout is the real-world
  killer**, not the size cap (see below). Default proxy timeouts (~30–60s) sever large uploads
  mid-stream.

### Observed failure (real case, 2026-06-14)

An 851MB upload (`Content-Length: 851359274`) failed with a generic client "message didn't go
through". The Caddy log showed it was **not a 413** — the body streamed fine:

```
POST /api/v1/channels/.../messages
Content-Length: 851359274   (851 MB)
bytes_read:     623101500   (623 MB received before the cut)
duration:       30.24 s
status:         0           (connection severed mid-upload)
```

623MB arrived in ~30s (≈165 Mbit/s) before a **30-second proxy timeout** guillotined the
connection at the 3/4 mark. A 413 body-size block would have rejected it instantly at 0 bytes;
instead it streamed most of the file then timed out. Fix = the timeout directives above, not the
body-size cap.

## Diagnosing a failed upload (which layer rejected it)

```bash
docker logs fluxer_server --since 5m 2>&1 | grep -iE "413|payload|too large|attachment|upload|limit" | tail -30
docker logs caddy --since 5m 2>&1 | tail -30
docker logs Nginx-Proxy-Manager-Official --since 5m 2>&1 | grep -iE "413|client_max|too large|fluxer" | tail -30
```

- `413` in the **NPM** log → proxy body-size cap (apply the Advanced config above).
- Request never reaches **fluxer_server** → blocked upstream (NPM or Caddy).
- Request arrives at **fluxer_server** then errors → app-side (limit not applied, or S3/storage).

## Client-side 30-second upload timeout (desktop app patch)

After the limit config and NPM timeouts were correct, an 851MB upload **still** died at exactly
~30.18s. Server logs (Caddy) showed the request streaming fine (`bytes_read` ~620–690MB of 851MB)
then severed with `status: 0` — and raising NPM timeouts to 3600s changed nothing. The cut was
**client-side**.

Cause: `fluxer_app/src/lib/HttpClient.tsx` has `private defaultTimeoutMs = 30000`. Every REST
request, including the multipart message-with-attachment POST, gets `xhr.timeout = 30000`. A large
upload can't finish the transfer in 30s, so the **browser/Electron client aborts itself**. No
server-side change can fix this.

Fix (local patch — requires a desktop rebuild): in `performXHRRequest`, skip the timeout when the
body is an upload (FormData/Blob/ArrayBuffer), keeping the 30s default for normal JSON calls:

```js
const isUploadBody = body instanceof FormData || body instanceof Blob || body instanceof ArrayBuffer;
if (config.timeout && config.timeout > 0 && !isUploadBody) {
    xhr.timeout = config.timeout;
}
```

This is a modification to **tracked source** — it will need re-applying after upstream pulls
(see `docs/11-build-patches-and-unraid.md`). After patching, rebuild the desktop client for the
change to take effect.

## Caveat

Single multi-GB browser uploads have **no resume** — a connection blip restarts from zero.
~5GB is the practical edge for one HTTP POST. The config will *accept* 30GB, but reliable
30GB transfers need a chunked / S3-multipart mechanism, not just a higher limit.
