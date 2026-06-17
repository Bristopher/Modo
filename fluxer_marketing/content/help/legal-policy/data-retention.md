We keep as little as we can, for as short a time as we can, and you can delete most things yourself. Our [Privacy Policy](/privacy) is the main reference for what we collect and how long we keep it. Section 7 covers retention in full; this article summarises the points people ask about most.

## What we keep while your account is active

While your account exists, we hold the things needed to run it: your username, email address, password hash, date of birth, and the content you have created (messages, uploads, Communities, profile details). You can delete most of this yourself at any time, and you can close the account entirely.

A few things are commonly assumed to be kept that we do not actually keep:

- **Phone numbers.** Phone verification is handled by Twilio. The number is not written to your account. Internally we hold only an encrypted marker for about 30 days, with no user ID attached, used solely to stop the same number being reused more than twice during a suspicious-registration check. After that window the marker expires.
- **Card numbers.** Payments go through Stripe. We never see or store the full card number.
- **Message content for analytics or AI.** Aggregate metrics count events, not content. Nothing you share is used to train AI models.

## What you can remove yourself

- **Individual messages and attachments.** Delete them in the app. Deleting a message also deletes its attachments.
- **All your messages in bulk.** Privacy Dashboard > Data Deletion. See [requesting data deletion](/help/data-deletion).
- **Your whole account.** Settings > Account. See [how to delete or disable your account](/help/delete-account). After a 14-day grace period (during which signing in cancels the deletion), the account is removed.
- **A specific piece of data.** Email <privacy@fluxer.app> from your registered address.

## What happens when you delete something

- **A message or account record** is removed from active systems within minutes. It may persist in encrypted backups for up to 30 days, then it is permanently removed.
- **An attachment** is removed from active storage within hours. It is recoverable by authorised operators for up to 24 hours for disaster recovery only, then permanently erased. Attachments are not included in our long-term backups.
- **A Community or channel** enters a 14-day grace period during which it and everything in it (messages, attachments, roles, settings) is hidden and inaccessible, then it is permanently deleted using the rules above.
- **Your account** gets a 14-day grace period (cancellable by signing in), then identifying data is removed. Backup purge follows the 30-day cycle above.

Content you sent in Communities or direct messages may remain visible to the other people who received it after your account is gone, but it is no longer linked to you. If you want it gone first, delete it before closing the account or choose the message-deletion option during account deletion.

## What we keep longer, and why

A few kinds of data outlive your account, but only for specific, narrow reasons:

- **Report snapshots.** When someone reports a message, user, Community, or invite, we snapshot the reported item so there is a stable record for investigation and appeals. Snapshots live in an isolated bucket, are not served to users or included in exports, and are deleted after one year. Deleting the original does not remove the snapshot during that window.
- **Security and usage logs.** Up to 90 days under normal conditions. Specific logs may be kept longer only for an active security investigation, a legal obligation, or an ongoing dispute.
- **Audit logs.** Records of administrative actions and enforcement decisions are kept as long as needed for accountability and appeals, and reviewed periodically.
- **Payment and transaction records.** Kept at least seven years, as Swedish bookkeeping law (Bokföringslag 1999:1078) requires. Full card numbers are not stored.
- **Photo IDs sent to support** (for an age appeal or a date-of-birth correction): deleted within 60 days after the request is closed.
- **Support correspondence.** Held in Intercom for as long as needed to handle the conversation and any follow-up, then deleted on review.
- **Backups.** Encrypted, off-site, kept on a rolling cycle of up to about 30 days, then overwritten.
- **Legal records.** Anything else we are specifically required by law to keep, for the period the law requires.

Aggregated or anonymised information that can no longer identify you may be kept indefinitely to understand service trends.

## Inactive accounts

Accounts may be scheduled for deletion after two years of inactivity, with advance notice to the registered email address before deletion proceeds. See [how to delete or disable your account](/help/delete-account) for the exact criteria and notice schedule.

## Your rights

You can export your data, delete your messages, and close your account from the Privacy Dashboard and account settings. Anything you cannot do through the app can be requested at <privacy@fluxer.app> from your registered email.

If you are in the EEA or UK, the GDPR also gives you rights of access, rectification, erasure, restriction, portability, and objection, as well as rights around automated decisions. California residents have parallel rights under CCPA/CPRA. Both are described in section 10 of the [Privacy Policy](/privacy), along with how to exercise them and how to lodge a complaint with your local supervisory authority.
