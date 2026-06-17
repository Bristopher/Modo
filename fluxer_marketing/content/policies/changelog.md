<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
**Effective date:** 2026-04-25

This page records significant changes to our [Terms of Service](/terms), [Privacy Policy](/privacy), and [Community Guidelines](/guidelines). Current versions are linked in the footer.

## 2026-04-25

**Privacy Policy**

- Clarified that phone numbers used for account verification are not linked to Fluxer accounts or user IDs. Successful phone verification now stores only a `has_verified_phone` account flag.
- Added the retention rule for phone verification reuse prevention: Fluxer keeps an encrypted internal marker for about 30 days, used only to allow the same phone number to verify at most twice in that period, with encryption keys rotated roughly every 30 days.
- Clarified that phone verification is used for suspicious registration anti-spam checks, not as a general identity or account-linking system.
- Clarified that SMS-based 2FA is not available for accounts registered on or after 25 April 2026.

## 2026-04-18

**Privacy Policy**

These changes clarify vendors, hosting, retention, and content-safety processing. None adds a new purpose for personal data or a new category of data leaving Fluxer.

- Updated the sub-processor list to match current infrastructure. Cloudflare, OVHcloud, Hetzner, Better Stack, and Sentry were removed because they are no longer used to process personal data. Porkbun was also removed because it handles domain registration only and does not process Fluxer users' personal data (sections 2.3, 4.2, 6.1, 9, 17.4).
- Rewrote Section 3.2 on IP geolocation. Fluxer now prefers fully local geolocation databases where they are sufficient. IPinfo is used only for registration and abuse-prevention checks that need IP network signals a local database cannot reliably provide, such as VPN provider, commercial proxy, Tor exit-node status, residential-proxy use, and related risk indicators.
- Clarified that IPinfo receives only the IP address being looked up. No account identifier, user identifier, session token, device context, message content, or other Fluxer user data is sent. Responses are cached on Fluxer systems, so repeat checks for the same IP do not go out over the network during the cache window.
- Removed the separate Backblaze sub-processor entry from Sections 4.2, 6.1, and 17.4. Off-site database backups are still kept with a storage provider for disaster recovery, but those backups are encrypted with keys held only by us. The provider cannot read, index, or otherwise process the contents, so we do not treat it as a sub-processor under GDPR Article 28 (section 6.1).
- Clarified error monitoring and observability. Metrics, logs, and traces run on Fluxer-controlled infrastructure, and application error or crash data is not sent to a third-party monitoring service (section 4.2).
- Updated Section 6.1. Primary hosting and object storage for user-uploaded files both run on Vultr in Piscataway, New Jersey, USA. Voice and real-time communication servers also run on Vultr across multiple regions worldwide so calls can be handled from a region close to you. Bunny.net continues to run the user-content CDN on `fluxerusercontent.com`.
- Added a 24-hour safeguard for accidentally deleted attachments. Deleted media is retained non-visibly in user-content object storage for up to 24 hours so it can be recovered from a bad bulk-delete or similar accident. After that window, and once CDN caches have been purged, it is permanently gone (section 7.3).
- Added Section 7.8 on snapshots of content reported through the in-app report feature. Snapshots are stored in an isolated bucket, accessible only to authorised trust-and-safety and engineering staff, audit-logged, retained for up to 1 year, then deleted automatically. If specific evidence must be preserved for longer to meet a binding legal obligation, we keep only what the law requires (sections 7.8, 7.9).
- Rewrote Section 5 to put the top-line position first: Fluxer does not use AI to scan your messages, files, voice calls, or anything else you share. The explicit-content classifier is a small, non-AI image model ([OpenNSFW2](https://github.com/bhky/opennsfw2)) that runs locally on our servers, does not contribute to AI training, and exists only to respect explicit-content preferences. Added the equivalent "no AI reads your content" position to the top of the policy.
- Updated the California disclosures table in Section 17.4 to match the new sub-processor list, including IPinfo for registration and abuse-prevention IP network signals and removing Backblaze.

## 2026-04-02

**Terms of Service**

- Added a clause clarifying that Fluxer is not designed or supported for safety-critical or critical-infrastructure use, and must not be relied on for military, emergency or first-response, healthcare, sanitation, utilities, or similar high-risk operations (section 3.4).
