# 05 — Auth Flow

How a user authenticates and how that credential is later validated by the gateway. File:line
references are against the current clone (2026-06-12); verify before relying on exact lines —
the repo is mid-refactor.

## Login endpoint

- **Route:** `POST /auth/login` — `packages/api/src/auth/AuthController.tsx:150`
- **Middleware:** LocalAuth, Captcha, RateLimit
- **Request:** `{ email, password, captcha_token?, captcha_type? }`
- **Response:**
  - No MFA → `{ mfa: false, user_id, token, theme? }`
  - MFA → `{ mfa: true, ticket, sms, totp, webauthn, allowed_methods, sms_phone_hint }`

Service logic: `packages/api/src/auth/services/AuthLoginService.tsx` — looks up user by email,
verifies password, branches on MFA, and on success calls `createAuthSession({ user, request })`.

## Password hashing

- `packages/api/src/utils/PasswordUtils.tsx`
- **Argon2** (`argon2.hash` / `argon2.verify`). Test mode uses reduced cost
  (memory 1024, time 1, parallelism 1).

## Token issuance

- `packages/api/src/auth/services/AuthSessionService.tsx` — `createAuthSession`:
  - generates token via `generateAuthToken()`
  - writes an auth-session record: `{ user_id, session_id_hash: SHA256(token), created_at,
    approx_last_used_at, client_ip, client_user_agent, client_is_desktop, version: 1 }`
- **Token format** — `packages/api/src/auth/services/AuthUtilityService.tsx`:
  - 27 random bytes → base62, padded to ≥36 chars, prefixed `flx_`
  - Final shape: **`flx_<36-char-alphanumeric>`**
  - `getTokenIdHash(token) = SHA256(token)` is what's stored/looked up (the raw token is never
    persisted server-side).

So the DB stores only the **SHA256 hash** of the token; the plaintext `flx_...` lives only on
the client.

## Client-side storage & use

- `fluxer_app/src/actions/AuthenticationActionCreators.tsx` — captures `token` from the login
  response into the auth store.
- The token is then handed to the realtime layer and sent in the gateway **IDENTIFY** payload
  (see [06-client-gateway-connection.md](./06-client-gateway-connection.md)).

## How the gateway validates a token

The Erlang gateway does **not** check the DB directly — it calls back into the API over RPC:

- `fluxer_gateway/src/session/session_manager_shard.erl` builds an RPC request:
  ```erlang
  #{ <<"type">> => <<"session">>,
     <<"token">> => Token,
     <<"version">> => ApiVersion,
     <<"ip">> => ClientIP }
  ```
  via `rpc_client:call/1`.
- API response handling: HTTP **200** → user data + auth session info; **401** → `invalid_token`;
  **429** → `rate_limited`.
- On success the gateway creates a session process (`fluxer_gateway/src/session/session.erl`)
  storing the **hashed** token (`utils:hash_token(Token)`); subsequent heartbeat/token re-checks
  go through `{token_verify, Token}`.

## Related: instance-to-instance / connections

`auth.connection_initiation_secret` (config) and `packages/api/src/connection/*` implement a
domain-verified connection/SSO flow (e.g. `DomainConnectionVerifier.tsx`, `SsoUtils.tsx`) using
the `/.well-known/fluxer` discovery document. Relevant for federation/relay, not for basic
local login.

## Summary diagram

```
POST /auth/login {email,password}
        │  argon2.verify
        ▼
createAuthSession ──> token = flx_<36>   (DB stores SHA256(token))
        │
        ▼ (client keeps plaintext token)
WS IDENTIFY {token}
        │
gateway ──RPC type:"session", token──> API
                                         │ 200 → ok (+user)  401 → invalid_token
        ◀────────────────────────────────┘
session process holds hash_token(token)
```
