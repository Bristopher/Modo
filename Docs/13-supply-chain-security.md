# 13 — Supply-Chain Safety (npm attacks)

How this build holds up against the 2025-era npm supply-chain attacks (the qix
`chalk`/`debug` compromise, the self-replicating "Shai-Hulud" worm, and the typosquat/
postinstall-stealer wave). Those attacks all share a shape: a **malicious package version**
is published, your install pulls it, and a **lifecycle script (postinstall)** runs that
exfiltrates npm/GitHub/cloud tokens and, in the worm's case, republishes itself.

## What already protects this repo

Fluxer's `pnpm-workspace.yaml` ships with the exact mitigations the security community
recommended after those incidents — verified present:

| Setting | Value | What it stops |
| --- | --- | --- |
| `minimumReleaseAge` | `1440` (24 h) | **The single best defense.** pnpm refuses any package version younger than 24 hours. Malicious versions are typically caught and yanked within hours — this build never sees them. |
| `strictDepBuilds` | `true` | Lifecycle scripts run **only** for explicitly allow-listed packages (`allowBuilds`). A random dependency's `postinstall` — the actual infection vector — **cannot execute**. |
| `allowBuilds` | explicit list | The allow-list: ~16 packages (sharp, esbuild, argon2, …). Everything else is script-blocked. We *removed* `wasm-pack` from it. |
| `blockExoticSubdeps` | `true` | Blocks non-registry deps (git/http/tarball URLs) pulled in transitively — a common malware delivery path. |
| `trustPolicy` | `no-downgrade` | Prevents version-downgrade attacks (pinning you back to a vulnerable release). |
| `pnpm-lock.yaml` + `--frozen-lockfile` | — | The Dockerfile installs **exact pinned versions** from the lockfile. No floating ranges, no surprise upgrades at build time. |

## Spot-check: known-compromised packages are NOT present

Checked the lockfile against the Sept 2025 qix/`chalk`-`debug` malicious versions and the
Shai-Hulud worm's seed packages. Every transitive dep is on a **safe** version:

| Package | Malicious version | Pinned here |
| --- | --- | --- |
| chalk | 5.6.1 | 4.1.2 ✅ |
| debug | 4.4.2 | 4.4.3 (post-fix) / 2.6.9 ✅ |
| ansi-styles | 6.2.2 | 6.2.3 ✅ |
| strip-ansi | 7.1.1 | 7.1.2 ✅ |
| color-convert | 3.1.1 | 2.0.1 ✅ |
| ansi-regex | 6.2.1 | 6.2.2 ✅ |
| supports-color | 10.2.1 | 7.2.0 ✅ |
| error-ex | 1.3.3 | 1.3.4 ✅ |
| is-arrayish | 0.3.3 | 0.2.1 ✅ |
| @ctrl/tinycolor | (worm seed) | not in tree ✅ |
| simple-swizzle | 0.2.3 | not in tree ✅ |

Re-run the scan any time:
```bash
for pkg in chalk debug ansi-styles color-convert strip-ansi supports-color ansi-regex "@ctrl/tinycolor"; do
  esc=$(echo "$pkg" | sed 's/[]\/[]/\\&/g')
  echo "$pkg: $(grep -oE "^  ${esc}@[0-9][^:]*" pnpm-lock.yaml | sed 's/^  //' | sort -u | tr '\n' ' ')"
done
```

## Our patches don't widen the attack surface

- **`wasm-pack: false`** — we *removed* a package from the build-script allow-list. Strictly
  safer (one fewer postinstall can run).
- **Every external binary fetch is pinned + checksummed.** Both `wasm-pack` (app-build) and
  `rebar3` (both gateway-build stages) are verified against a SHA-256 (`sha256sum -c`), so a
  swapped or tampered artifact fails the build instead of running. These are the only
  non-registry downloads in the build.
- **No secrets at build time.** The build container only sees `FLUXER_BUILD_CONFIG` (your
  public domain — not secret). `config/config.json` (the keyring) is mounted to the
  **runtime** container via volume, never baked into the image. So even a hypothetical
  build-time stealer has no tokens to take.
- **Blast radius = the build container.** `docker build` runs isolated; it has no access to
  your host npm token, SSH keys, or `~/.aws`. The worm's propagation step (republish to npm)
  needs an npm auth token that simply isn't in the container.

## Staying safe going forward

1. **Don't `--force` or regenerate the lockfile casually.** `--frozen-lockfile` is your
   friend; it's what pins you to vetted versions. Only update deps deliberately.
2. **Keep `minimumReleaseAge` ≥ 1440.** If you ever bump dependencies, the 24 h cooldown is
   what buys time for a bad release to be caught before you pull it.
3. **Don't add packages to `allowBuilds`** unless you know why they need a build script.
4. **`pnpm audit`** periodically for known CVEs (separate from the worm threat, but useful).
5. If you build on the NAS, do it as an unprivileged user and don't have an npm publish token
   present in that shell/environment.
