<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
## Reporting security issues

If you find a vulnerability in Fluxer, email **<security@fluxer.app>**.

A good report has a clear title, the affected area, the impact, and steps to reproduce. Screenshots, logs, requests, and environment details all help. The more precise the report, the faster we can fix the issue. Use test accounts where you can, and leave out real users' data and session tokens unless they are strictly needed.

Reports must include a proof of concept demonstrating impact against Fluxer's production services or a supported self-hosted setup. Repository review, local builds, forks, and modified deployments can support a report, but they are not enough on their own.

## Scope

In scope: websites, apps, and services operated by Fluxer Platform AB, including `fluxer.app`, `fluxer.gg`, `fluxer.gift`, `fluxerapp.com`, `fluxer.dev`, `fluxerusercontent.com`, `fluxerstatic.com`, `fluxer.media`, and their subdomains. Infrastructure we directly manage is also in scope, as is abuse of Fluxer features that allows unauthorised access, persistence, or data disclosure. Supported self-hosted releases are in scope when the issue reproduces in the documented setup without custom patches.

Out of scope:

- Third-party services and infrastructure we do not control.
- Physical security, social engineering, and phishing.
- DoS, flooding, resource exhaustion, and noisy scanning. An application-layer DoS provable with a few requests can be reported, just do not exploit it at scale.
- Modified, unsupported, or misconfigured self-hosted deployments, unless the issue also affects Fluxer's production services or a supported self-hosted setup.
- UI bugs, feature requests, and support issues.
- Theoretical findings, like missing best-practice headers, without a realistic attack path.

## Safe harbour

**Good-faith research that follows this policy is authorised.** We authorise good-faith research under this policy for the purposes of Swedish, EU, US, and equivalent anti-hacking laws, and we will not take legal action against you for it. If a third party takes action against you over such research, we will make clear that it was authorised under this policy. Safe harbour applies by default and is not revoked retroactively.

It does not cover extortion, intentional harm to users, service degradation, or data destruction. If you are not sure whether a test is in scope, ask first.

## Testing rules

- Only test with accounts, Communities, and data you own or have permission to use.
- Do not access, change, or delete other people's data. If you reach someone else's data by accident, stop, do not keep it, and tell us.
- Do not degrade the service, message users outside your test, scrape, flood, or brute-force.
- If a test could trigger real notifications, billing, or payments, ask us first.
- Delete any user data from your testing when you are done, and follow the law.

## What happens next

We aim to get back to you within a few days. Severity is judged on real impact: who is affected, what data is at risk, and how exploitable it is. The more severe the issue, the faster we move.

If several people report the same issue, the first clear report gets the credit. If we cannot reproduce something, we will ask for more detail before closing the report.

## Disclosure

Please hold off on public disclosure until we have confirmed and fixed the issue, typically up to 90 days. If a fix takes longer, we will keep you in the loop and agree a timeline; we will not ask for indefinite silence. If we publish an advisory, we will credit you and coordinate timing with you where we can.

## Rewards

Valid reports may earn a Bug Hunter badge and Fluxer Plutonium gift codes, scaled to severity and report quality. To stay eligible, report privately, follow this policy, and do not exploit the issue beyond demonstrating it. Fluxer staff, contractors, and their immediate family are not eligible.

## Contact

Security: <security@fluxer.app>. Everything else: <support@fluxer.app>.

Thank you for helping keep Fluxer safe.
