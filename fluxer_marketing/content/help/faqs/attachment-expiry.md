Fluxer automatically expires older attachments. Smaller files stay available for longer, while larger files expire sooner. If someone opens a message with a file that is close to expiry, we extend its availability so it remains accessible.

## How expiry is decided

The timer starts when you upload the file. Files of 5 MB or smaller keep links for about three years, the longest window. Files near 500 MB keep links for about 14 days, the shortest. In between, larger files get shorter windows. Files above 500 MB are not accepted right now.

## Extending availability when accessed

If a message with a file is loaded and the remaining time falls inside the renewal window, we move the expiry forwards. The renewal window depends on size: small files can renew up to about 30 days, while the largest files renew up to about 7 days.

One view is enough to refresh a file. You do not need to click or download it. Multiple views inside the same window do not stack. The total lifetime is capped to the size-based budget, so repeated renewals cannot keep a file available indefinitely.

## What happens after expiry

We regularly sweep expired attachments and delete them from our CDN and storage. The same removal mechanism applies as for attachments you delete yourself: the file leaves active storage within hours and is irretrievable after the 24-hour disaster-recovery window described in [section 7.4 of our Privacy Policy](/privacy). Attachments are not included in our long-term backups.

## Why we expire attachments

Large media is expensive to store indefinitely, so expiry keeps storage fair for everyone. Clearing older uploads also reduces the chance that sensitive files remain accessible for longer than needed.

## Keeping important files

If you need a file, download it before it expires. For full account exports, including attachment URLs, see [exporting your account data](/help/data-export).

## Frequently asked questions

### Does Plutonium extend file expiry?

Not at the moment. The same attachment expiry limits apply to all users.

### Do I need to click or download a file to keep it available?

No. Viewing the message in chat or search is enough.

### What about Saved Media?

Saved Media lets you keep up to 50 files, or 500 with Plutonium. Saved Media is not subject to attachment expiry.

### Can I hide the expiry indicator?

Yes. Go to User Settings > Messages & Media > Media and switch off "Show Attachment Expiry Indicator".
