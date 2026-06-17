<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
**Effective date:** 2026-04-25

## The short version

Fluxer is a chat service run by Fluxer Platform AB, a Swedish company based in Stockholm. This policy explains how we handle your data. It is binding under EU consumer protection law and part of our Terms of Service, so you can hold us to it.

- We do not sell, rent, or license your personal data. We have no advertising partners and no dealings with data brokers. Our revenue comes from Fluxer Plutonium, our optional premium subscription.
- AI does not read what you share on Fluxer. We run no AI or LLM inference over your messages, files, or voice and video calls, and none of your content is used to train or fine-tune AI models. The only automated content check is a local image classifier that helps respect explicit-content preferences.
- We do not track you around the web: no tracking cookies, no analytics SDKs, no browser fingerprinting.
- You can export your data, delete your messages, and close your account whenever you like.
- Most account data currently lives on servers in Piscataway, New Jersey, where US law, including the CLOUD Act, applies. Section 6 explains why, the privacy trade-offs, and the direction we are evaluating.

## 1. Who we are

Fluxer Platform AB is a Swedish limited liability company, organisation number 559537-3993. We operate Fluxer and related services. For GDPR purposes, we are the data controller for your personal data: we decide what data is processed and why, under the General Data Protection Regulation as implemented in Sweden, supervised by the Swedish Authority for Privacy Protection (Integritetsskyddsmyndigheten, known as IMY).

**Privacy contact:** Hampus Kraft, Founder and CEO
**Email:** <privacy@fluxer.app>
**Phone:** +46 79 101 18 18 (legal enquiries only; no phone support)
**Postal address:** Fluxer Platform AB, c/o Embassy House, Östgötagatan 12, 116 25 Stockholm, Sweden

Hampus handles privacy and data protection questions, including data subject requests.

We have not appointed a formal Data Protection Officer (GDPR Article 37) or a UK representative (UK GDPR Article 27). Both are kept under review as the service and its safety and security processing grow, and this section will be updated if that changes. In the meantime, UK residents can direct any data protection enquiry to <privacy@fluxer.app>.

## 2. What we collect, and what we do not

### 2.1 What you give us

**Account data.** Creating an account requires an email address, a username, a password, and your date of birth. Without those, we cannot sign you up. Everything else, including your avatar and bio, is optional. Passwords are stored using Argon2id, a memory-hard hashing algorithm, so even we cannot read them.

**Phone verification.** When registration triggers anti-spam checks, we may request phone verification to prevent large-scale registration abuse. The number is not linked to your account; completing verification stores only a flag saying it happened (full details in Section 7.3). SMS-based 2FA is not available for accounts registered on or after 25 April 2026.

**Content.** What you do on Fluxer: messages, files, images, voice and video calls where supported, Community data, reactions, and profile details such as your avatar, bio, and display name, plus any Communities you create or administer. All of it belongs to you.

**Support.** Support correspondence passes through Intercom, which handles the message body, attachments, anything else you choose to share, and the basic technical details (IP address, browser type) needed for support to work.

**Payments.** Stripe processes payments, not us. If you buy Fluxer Plutonium or anything else premium, we receive only what is needed to record and manage the purchase: billing country, the card's last four digits and expiry, payment status, and timestamps.

We do not ask for special-category personal data such as health, religion, race, ethnic origin, sexual orientation, political views, or trade union membership. If you choose to share any of that in a message or on your profile, it will not be used to profile you, target you, or treat you differently.

### 2.2 What we collect automatically

**Technical data:** the IP address you connect from, browser type and version, operating system, device type and identifiers, language settings, and similar details.

**Usage data:** aggregate, non-identifying records of which features get used and how often (voice calls started, files uploaded, reactions used), pages and screens visited, timestamps, session durations, crash reports, and performance metrics. Message content is not read to produce any of this; when a metric counts "messages sent", it counts the event and nothing more.

**Security and operational logs:** login attempts and authentication events, account setting changes, rate-limit triggers, API and system errors, and IP-based signals relating to spam, abuse, or unusual behaviour.

Fluxer has no advertising trackers, third-party analytics SDKs, browser fingerprinting, or cross-site tracking pixels. We do not build behavioural profiles or track which other sites you visit before or after using Fluxer.

### 2.3 What we receive from other sources

**Other users** generate data about your account when they interact with you. Someone mentioning you, adding you to a Community, or messaging you involves your username and user ID, the content of the interaction, and metadata such as timestamps.

**Service providers** relay limited operational information. Sweego reports whether transactional emails were delivered and when. Twilio reports whether SMS verification succeeded; for verification it processes the phone number needed to send the SMS, which is stored as described in Section 7.3. Stripe reports payment status and risk signals. Intercom carries support conversations and resolution status. Our infrastructure providers send alerts when they detect abuse or anomalies. IPinfo provides IP network signals for registration and abuse-prevention checks, under the conditions in Section 3.2.

**Public sources and fraud signals** occasionally reach us too, such as a reputation signal about a particular IP, or a risk score Stripe attaches to a transaction.

All of this is combined with what we collect directly only to run, secure, and maintain Fluxer.

## 3. How we use your information, and how we do not

### What we use it for

- **Operating Fluxer:** creating and managing your account, routing messages to the right recipients, and keeping features working end to end.
- **Security and abuse prevention:** blocking unauthorised access; investigating abuse, fraud, and spam; enforcing our Terms of Service and Community Guidelines.
- **Service communications:** security alerts, service updates, and the administrative emails your account needs to function.
- **Payments:** processing payments and managing subscriptions if you buy Plutonium or anything else premium.
- **Maintenance and improvement:** aggregate metrics showing which features are used, how performance is holding up, and where bugs occur. The data involved is error rates and feature counts, not message or file content.
- **Legal:** meeting legal requirements, responding to valid legal requests, and protecting the safety, rights, and property of our users, the public, and Fluxer.

### What we do not use it for

Your messages, files, voice or video calls, and anything else you create or share on Fluxer will never be used for:

- advertising, targeted or otherwise
- training, fine-tuning, or evaluating AI or machine learning models
- building profiles of you for ad targeting, marketing, or behavioural analysis
- sale, rental, or granting a licence to any third party for their own purposes
- mining or aggregating for commercial gain beyond operating the service

### 3.1 Lawful bases for processing (GDPR)

For readers in the EEA, the UK, or any other jurisdiction requiring a lawful basis:

**Contract necessity (Article 6(1)(b)).** Processing required to deliver the service you signed up for: delivering messages, running Communities, managing your account, processing payments, providing support.

**Legitimate interests (Article 6(1)(f)).** Service security, fraud prevention, reliability and performance, aggregate feature-use analysis, and writing to you about changes to our services or policies. Each activity has a documented assessment covering its purpose, necessity, and the balance against your rights. For example, abuse prevention uses IP signals, login events, device metadata, and rate-limit triggers; these are used only for security and administration, never for advertising or profiling. You can object at any time (Section 10).

**Legal obligations (Article 6(1)(c)).** Accounting, tax, and bookkeeping under Swedish law, responses to lawful requests from public authorities, and compliance with applicable data protection, security, and consumer laws.

**Consent (Article 6(1)(a)).** A smaller set of processing, such as optional communications or specific cookie uses on our marketing site where local law requires consent. Consent can be withdrawn at any time through your settings or by writing to us; withdrawal does not affect processing that was lawful beforehand.

### 3.2 IP address geolocation

Your IP address is used to determine approximate location (city, region, country) to:

- alert you to logins from new or unusual locations
- show you where your account is currently signed in
- spot fraud and abuse
- determine regional age requirements and access eligibility under local laws
- meet legal obligations relating to export control and sanctions

We prefer a fully local geolocation database (MaxMind GeoIP, downloaded periodically and queried entirely on our own servers, with no per-lookup network call to a third party) whenever it can answer the question.

Registration and abuse-prevention checks sometimes need IP network signals a local database cannot provide: VPN provider, commercial proxy, Tor exit-node status, residential-proxy use, and related risk indicators. For those we query IPinfo, sending only the IP address (no account identifier, user identifier, session token, or device information), so the lookup cannot be linked back to your Fluxer account. Responses are cached on our own servers, so an IP is sent at most once per cache window; stable residential IPs are cached longer, rotating proxy-pool IPs for less time. These signals are used only for security and abuse prevention, including registration checks and, in some cases, rejecting Tor or residential-proxy traffic at the API edge. They are never used for advertising, profiling, or personalisation.

**Automated regional access decisions.** Where local law requires platforms to verify user age, we rely on automated regional restrictions driven primarily by IP geolocation instead of government ID uploads or biometric scans, which are more invasive than we are willing to require for general access. This can affect whether you can use Fluxer, or specific features, from a given region.

The approach is imperfect: travel, VPNs, proxies, and unusual network setups can all produce the wrong outcome. If you believe your access has been restricted in error, write to <privacy@fluxer.app>. We will acknowledge your request promptly, conduct a human review while your account stays in its current state, and send you the outcome with the reasoning. You can put your point of view at any stage, and rights around automated decision-making under applicable law (GDPR Article 22, for instance) are honoured per Section 10. Current regional restrictions, their basis, and their effect are listed in our [Regional restrictions](/help/regional-restrictions) help article.

## 4. Who we share data with, and who we do not

Your personal data is not sold, rented, traded, or licensed to any third party. Sharing is limited to the situations below.

### 4.1 Sharing you initiate

Messages go to their recipients, Community posts are visible to members, your profile is visible to the extent you choose, and integrations you connect can access the data you grant them. Once shared, content can be saved or redistributed by other users outside Fluxer; as with any chat app, be deliberate about what you share and with whom.

### 4.2 Our service providers (processors)

A small, carefully chosen set of third parties processes data on our behalf. Each has been reviewed for privacy and security and is bound by a data processing agreement under GDPR Article 28.

_Infrastructure and storage._ Vultr provides our primary hosting, and Bunny.net operates our content delivery network for user-generated content on the `fluxerusercontent.com` domain. Section 6.1 details where data is stored.

_Security and safety._ IPinfo provides IP network signals for registration and abuse prevention under the conditions in Section 3.2. hCaptcha provides CAPTCHA challenges for bot detection.

_Third-party content._ Google provides YouTube embeds and GIF search (Tenor); KLIPY provides additional GIF search. Traffic to Tenor and KLIPY is proxied through our servers, so your IP address and device identifiers never reach them. YouTube metadata is fetched server-side (see Section 13).

_Payments and communications._ Stripe handles payment processing, Sweego (hosted in the EU) transactional email, Twilio SMS-based account verification, and Intercom our support tool. Phone numbers used for verification are never linked to Fluxer accounts (Section 7.3). Intercom handles your support messages, email address, and basic technical information under our instructions, not for its own purposes.

_Error monitoring and observability._ Our observability stack (metrics, logs, and traces) runs on infrastructure we control. No application errors or crash data are sent to any third-party monitoring service.

Data processing agreements are in place with Vultr, Bunny.net, IPinfo, hCaptcha, Stripe, Sweego, Twilio, and Intercom. A few providers act as independent controllers when you interact with them directly (Google for YouTube embeds, hCaptcha for challenge completion); in those interactions their own terms and privacy policies apply alongside ours. Additions or replacements to this list are reflected here, with material changes noted in our [changelog](/changelog).

### 4.3 When law or safety requires disclosure

Disclosure outside Fluxer happens only for:

- compliance with a valid legal obligation, legal process, or enforceable governmental request
- enforcement of our Terms of Service or other agreements
- protection of the safety, rights, or property of users, the public, or Fluxer
- detection, prevention, or handling of fraud, security, or technical issues

Where the law allows, and where notice would not create a safety, security, or legal-process risk, we try to notify affected users before disclosing data in response to a legal request, particularly when it concerns an account or its content.

### 4.4 Business transfers

If Fluxer Platform AB is part of a merger, acquisition, reorganisation, sale of assets, or similar transaction, personal data may need to be transferred. In that event:

- affected users receive at least 30 days' advance notice, where legally permitted, before personal data is transferred
- the acquiring entity is bound by this Privacy Policy for as long as it holds your data, unless it obtains your affirmative consent to a new policy
- you will have the opportunity, with clear instructions, to delete your account and request deletion of your data before the transfer takes effect
- data will not be transferred to any entity that does not agree to honour these protections

## 5. Content safety

Fluxer does not run AI or LLM inference on your content, and nothing you share is used to train or evaluate AI models, ours or anyone else's (Section 3). What we run instead is a small set of automated safety measures, plus limited human review in defined circumstances.

### 5.1 Explicit-content classification

For an explicit-content opt-out to work, we need to know which uploaded images and video thumbnails are likely to contain nudity. We use [OpenNSFW2](https://github.com/bhky/opennsfw2), an open-source image classifier running on our own servers. It is a small pretrained classifier, not a generative AI system or LLM: given an image, it returns a single probability that the image contains pornographic content. It cannot read text, generate output, retain memory across calls, or learn from what it sees. It runs entirely on our servers with no external API call, we use the published weights without fine-tuning them on your uploads, and no classifier output enters any training pipeline.

The classifier creates a per-image "likely explicit" flag, used only so delivery can respect recipients who have opted out of explicit content, for example when you send media in a DM or a Community channel. The aim is to spare people unsolicited nudity. A flag has no further consequences: the media is not banned, the account is not disabled, and no report is filed.

### 5.2 Other automated safety measures

Narrowly-scoped automated systems operate on metadata and patterns (message frequency, link structure, account age, IP reputation) rather than message content, to:

- block known malware, phishing links, and spam patterns
- detect and mitigate harassment, raiding, and coordinated abuse
- flag suspicious login attempts and account-takeover patterns
- enforce Terms of Service and Community Guidelines when they have been breached

They do not read message content and do not feed advertising or behavioural profiles.

### 5.3 Human review

Authorised staff may examine specific content when needed to investigate a user report, enforce policy, or respond to a credible safety issue. Access is controlled by role-based permissions and recorded in an audit log, so each access is attributable and reviewable internally.

### 5.4 Data Protection Impact Assessments

Where processing may create a high risk to people's rights and freedoms, we carry out Data Protection Impact Assessments (DPIAs), as GDPR Article 35 requires. Two are complete to date: the explicit-content classifier (Section 5.1) and IP-based automated access decisions (Section 3.2). Each examines necessity and proportionality, identifies risks, and documents the technical and organisational measures used to reduce them. DPIAs are revisited when we introduce or materially change processing, or on a regular schedule, whichever comes first.

## 6. Where your data lives

### 6.1 Storage locations

Our primary servers are at Vultr in Piscataway, New Jersey: your main data (account information, messages, Communities, and other persisted content) is stored there, together with the object storage for user-uploaded files. Voice and real-time communication traffic runs on Vultr across multiple regions worldwide so calls route near you. User-generated content is delivered and cached through Bunny.net edge locations worldwide (for content on `fluxerusercontent.com`).

Encrypted off-site backups of our databases exist for disaster recovery. The encryption keys are held only by us, so the backup provider cannot read, index, or otherwise process your data; that is why it is not treated as a sub-processor under GDPR Article 28.

### 6.1.1 Why our primary hosting is in the United States

At our present scale, US East Coast data centres offer a reasonable balance of connectivity, reliability, and latency for a global service. The choice is operational rather than privacy-driven, and alternatives are kept under review.

It has privacy consequences. Storing primary data in the United States brings it within reach of lawful access requests under US law, including the Clarifying Lawful Overseas Use of Data Act (the "CLOUD Act"), under which a provider subject to US jurisdiction can be required to produce data in its possession, custody, or control, even if stored outside the United States. US hosting therefore carries a foreign-government access exposure that some other setups do not. We treat this as a real privacy issue, and it is factored into our transfer impact assessments, provider reviews, and legal request procedures (Sections 6.2 and 14).

### 6.1.2 Regional hosting under evaluation

Reducing our reliance on US-hosted primary infrastructure is under active evaluation. One option is a separate EU region, where accounts and Communities could be hosted in Europe. Depending on the architecture and providers involved, that could reduce foreign-government access exposure, though not eliminate it entirely. If regional hosting ships, we will explain what it changes, what it does not, and which legal regimes still apply.

### 6.2 International data transfers

Because we operate globally and use providers in multiple countries, your data may be transferred to and processed in countries other than your own, including the United States and Canada, which may have different data protection laws.

Where the law requires it (under GDPR, for instance), safeguards are in place. Our data processing agreements include Standard Contractual Clauses approved by the European Commission or UK authorities, maintained even where other adequacy mechanisms may apply. Transfer Impact Assessments are carried out for each destination, covering the legal framework in the recipient country and the actual ability of authorities there to access data. Supplementary measures are contractual, organisational, and technical: encryption in transit and at rest, strict access controls, audit logging of access to user data, and contractual limits on provider use.

These measures have limits. We do not currently rely on jurisdiction-specific key separation, customer-controlled encryption keys, or an architecture that would make server-side data inaccessible to a provider served with a lawful order. For data we need to process on our servers to run Fluxer, these measures reduce transfer and access risk but do not eliminate it. Your data is never transferred to any third party for that party's independent advertising or marketing purposes.

## 7. Data retention

Your personal data is kept only as long as needed for the purposes in this policy, legal obligations, dispute resolution, and enforcing our agreements. Retention periods are reviewed, and data no longer needed is deleted or anonymised.

### 7.1 Active accounts

While your account is active, we hold your personal data, messages, Communities, and other content so the service can function. You can delete specific content yourself at any time.

### 7.2 Attachments and expiry

Attachments may remain available only for a limited time, depending on factors such as file size, age, and access frequency. Items saved to Saved Media are treated separately and are not subject to the same expiry. Current details are in our [help article on attachment expiry](/help/attachment-expiry).

### 7.3 Phone verification markers

Phone numbers used for account verification are not stored on your Fluxer account; when verification succeeds, your account stores only `has_verified_phone: true`.

To prevent repeated reuse during suspicious registrations, we keep an internal encrypted marker for the phone number for about 30 days. The marker contains no user ID or account reference, so it cannot be linked to an individual Fluxer account. It is used only to allow the same phone number to verify at most twice during that period, and not for SMS 2FA, recovery, advertising, profiling, contact discovery, or linking accounts together. The encryption key for these markers is rotated roughly every 30 days, with a short primary/secondary rollover so existing markers can expire naturally.

### 7.4 Deleted content

Database records (messages, account data) leave active systems quickly, typically within minutes. They may persist in encrypted backups for up to 30 days before being permanently removed.

Media attachments leave active storage typically within hours. As a short disaster-recovery safeguard against accidental deletion, the object storage behind our user-content bucket retains a non-visible copy of each deleted attachment for up to 24 hours before final erasure. During that window the attachment is invisible to you, other users, and our CDN, and only authorised operators acting on a genuine disaster-recovery scenario (such as recovery from a bad bulk delete) can restore it. Media attachments are not included in our long-term encrypted backups. Bunny.net's cache purge API is used to invalidate CDN-cached attachments as soon as possible after deletion, though short delays can occur due to rate limits and global propagation.

If you exercise your right to erasure under GDPR Article 17, the 30-day backup retention period is treated as a documented technical limitation: during that window, deleted data remains in encrypted backups that are not used for active processing and are subject to scheduled purging. Your erasure request is completed once the data has been removed from both active systems and backup cycles. Longer retention applies only where the law requires it (for example, tax or legal compliance).

### 7.5 Deleted Communities and channels

Deleting a Community or channel starts a 14-day grace period. During that window, it and all its contents (messages, attachments, roles, settings, and other associated data) are hidden from users and inaccessible through the API, media proxy, search, data exports, and bulk-deletion operations, but remain in our systems to allow recovery from accidental or unauthorised deletions. Restoration can be requested through support during the window; after 14 days, the Community or channel and all its data are permanently deleted via the procedures above, and restoration is no longer possible.

### 7.6 Inactive accounts

Accounts may be scheduled for deletion after 2 years of inactivity, with advance notice sent to the registered email address. The inactivity definition, notice schedule, and deletion process are in the [guide to deleting or disabling an account](/help/delete-account). Once deletion completes, the account can no longer be signed into and its remaining data is inaccessible, though messages you sent in Communities or direct messages may still be visible to other users unless you deleted them first or chose the message-deletion option during account deletion.

### 7.7 Payment and transaction data

Transaction records are kept for at least seven years, as required by Swedish bookkeeping law (Bokföringslag 1999:1078), and for as long thereafter as needed for legal compliance, dispute resolution, or fraud prevention. Retention is reviewed periodically. Full payment card numbers are not stored.

### 7.8 Logs and security data

Security and usage logs are kept for up to 90 days under normal conditions, then deleted or anonymised. Some may be retained longer for an active security investigation, a specific legal obligation, or an ongoing dispute, and are deleted once that reason ends. Audit logs (records of administrative actions, enforcement decisions, and account changes) are kept as long as needed for accountability, appeals, dispute resolution, and legal compliance, and are reviewed periodically.

### 7.9 Reported content snapshots

When someone uses the in-app report feature to flag a message, user, Community, or invite, we take a snapshot of the reported item so there is a stable record for investigation, action, and any appeal. A reported message snapshot typically includes the message itself, a short window of surrounding messages for context, any attachments in the report, and metadata about who reported it and why.

Report snapshots live in a separate object-storage bucket, isolated from the main user-content bucket. They are not served to end users, included in data exports, or indexed for search. Access is limited to authorised trust-and-safety and engineering staff who need them for report review, and every access is recorded in an audit trail.

Snapshots are kept for up to 1 year from the report date, after which an automated storage-lifecycle rule deletes them permanently. Deleting the original message, attachment, account, or Community does not remove the snapshot during this window, because it preserves the record needed for investigation and appeals. In rare cases where specific evidence must be kept longer to meet a binding legal obligation, only what the law requires is retained.

### 7.10 Retention at a glance

- **Account information:** while your account is active.
- **Phone verification reuse markers:** about 30 days; encrypted, no user ID, used only to allow at most two verifications per phone.
- **Messages and user content:** while your account is active, unless you delete them earlier.
- **Deleted messages (database records):** removed from active systems within minutes, and from encrypted backups within 30 days.
- **Deleted media attachments:** removed from active systems within hours; recoverable for up to 24 hours for disaster recovery; not backed up.
- **Deleted Communities and channels:** 14-day grace period, then permanently deleted per the above procedures.
- **Report snapshots (in-app reports):** up to 1 year; access limited to authorised staff and audit-logged.
- **Security and usage logs:** up to 90 days (longer only for active investigations or legal obligations).
- **Audit logs:** kept as needed; reviewed periodically.
- **Payment and transaction records:** at least 7 years (Swedish bookkeeping law); reviewed periodically after that.
- **Inactive accounts:** scheduled for deletion after 2 years of inactivity, with advance notice.

## 8. Your controls

### 8.1 Privacy dashboard

**Data export.** Available from the Data Export tab as a ZIP archive of machine-readable JSON files, covering account data, per-channel message history, payment history, and security data, along with any profile assets. Message attachments are not bundled; the export includes CDN URLs for downloading them while they remain available. Messages in Communities or channels in a deletion grace period are excluded. Current details are in [the help article on exporting your account data](/help/data-export).

**Attachment downloads.** Use the URLs in your export to keep copies before you delete messages or before attachments expire.

**Message deletion.** Individual messages can be deleted in-app; deleting a message also deletes its attachments. Bulk deletion of all your messages is available from the Data Deletion tab. It runs in the background, can take some time for large accounts, and skips messages in Communities or channels in a grace period. See [the article about requesting data deletion](/help/data-deletion).

**Account deletion.** Can be scheduled from settings and proceeds after a grace period, unless you sign back in to cancel. The [guide to deleting or disabling an account](/help/delete-account) has the full details.

### 8.2 Requests by email

To remove or correct a specific piece of data rather than delete everything, write to <privacy@fluxer.app> from the address associated with your Fluxer account, telling us clearly what you want us to do. We may ask for more information to verify your identity. If you want a copy of your data before deleting messages or your account, request an export first and wait for it to complete.

## 9. Security

Technical and organisational measures protect your personal data against accidental or unlawful destruction, loss, alteration, disclosure, and unauthorised access. Current measures include:

- standard encryption for data in transit (TLS)
- strong encryption for data at rest on our servers and backups
- professionally managed data centres with physical security
- security updates, patch management, and infrastructure hardening
- rate limiting and protections against abuse and attacks
- access controls restricting user data to authorised staff with a demonstrated need
- regular encrypted backups for disaster recovery
- audit logging of access to user data

**A note on encryption.** Nothing on Fluxer is currently end-to-end encrypted. Your data is encrypted in transit between your device and our servers, and at rest on our servers and backups, but because the service relies on server-side processing to function, message content is technically accessible to our systems while it is being handled. The same holds for real-time voice and video, which runs on Vultr across multiple regions (Section 6.1) with traffic encrypted in transit. In plain terms, you are trusting Fluxer and our hosting provider to protect that traffic.

Opt-in end-to-end encryption is planned for Personal Notes, DMs, Group DMs, and voice chats. Until that feature exists and you turn it on for a supported area, content and call media are not end-to-end encrypted on Fluxer.

**Responsible disclosure.** Security vulnerabilities can be reported through our [Security bug bounty page](/security). Responsible disclosure is appreciated and may be acknowledged publicly with your consent.

### 9.1 Data breaches

In the event of a personal data breach, we will investigate and take appropriate remedial steps. The relevant supervisory authority (IMY) will be notified within 72 hours of our becoming aware of a breach likely to pose a risk to your rights and freedoms, as GDPR Article 33 requires; affected users will be notified without undue delay where the risk is high, as Article 34 requires; and other applicable breach notification obligations will be met. Notifications will explain what happened, what data is likely affected, the likely consequences, and what you can do to protect yourself.

## 10. Your rights

Depending on where you live, you may have rights over your personal data. We honour them promptly and in good faith.

### 10.1 Under GDPR (EEA and UK)

- **Access:** confirmation of whether we process your personal data, and if so, a copy of it.
- **Rectification:** correction of personal data that is inaccurate or incomplete.
- **Erasure ("right to be forgotten"):** deletion of personal data no longer needed for the purposes it was collected for, or in other circumstances the law recognises.
- **Restriction:** restriction of processing in certain circumstances, such as while accuracy is being verified or an objection assessed.
- **Objection:** to processing based on legitimate interests. Processing stops unless compelling legitimate grounds override your interests, rights, and freedoms, or it is necessary for legal claims.
- **Data portability:** a copy of your personal data in a structured, commonly used, machine-readable format, or transmission to another controller where technically feasible. Self-service exports are a ZIP archive of JSON files, with download URLs for message attachments and certain related assets if there are any.
- **Withdrawal of consent:** at any time, for processing based on consent, without affecting the lawfulness of prior processing.
- **Automated decision-making:** where automated decisions significantly affect you (such as whether regional access rules apply), you can request human review, put your point of view, and contest the decision.

### 10.2 Under CCPA/CPRA (California residents)

California residents have the right:

- to know what personal information is collected, used, disclosed, and shared
- to delete personal information in certain circumstances
- to correct inaccurate personal information
- to opt out of the sale or sharing of personal information (no such sale or sharing occurs)
- to limit the use and disclosure of sensitive personal information
- to be free from discrimination for exercising any of the above

Additional California-specific disclosures are in Section 17.

### 10.3 Exercising your rights

Several of these rights can be exercised directly through your Privacy dashboard and account settings. Requests can also be sent to <privacy@fluxer.app>. We may need to verify your identity, for instance by asking you to reply from your registered email or provide additional details. Responses are returned within the timeframe required by applicable law, usually within 30 days, or up to 45 days where permitted. If we cannot fully comply (due to legal obligations or the rights of others, for example), we will explain why and what options remain. You can authorise an agent to submit requests on your behalf where the law permits; proof of authorisation may be requested.

### 10.4 Complaints to supervisory authorities

You have the right to lodge a complaint with your local data protection authority. In Sweden, that is the Swedish Authority for Privacy Protection (IMY) at [imy.se](https://www.imy.se); in the UK, the Information Commissioner's Office (ICO) at [ico.org.uk](https://ico.org.uk); the authority in your country of residence is also an option. You can also raise concerns with us first, so we have an opportunity to resolve them directly.

## 11. Children's privacy

### 11.1 Minimum age

Meeting the minimum age requirement in your region is a condition of using Fluxer. The general minimum, including in Sweden, is 13, though some countries set it higher; the full list is in our [help article on minimum age requirements](/help/minimum-age). Users above the minimum age but below the age of legal majority (for example, under 18) may use Fluxer, but our Terms require a parent or guardian to review and agree to them on the user's behalf.

### 11.2 Protections for younger users

Eligibility is determined from approximate geographic location and self-reported information. Users identified as under 18 may have stricter safety features enabled by default, including tighter privacy defaults and restrictions on age-restricted features. Because no user is profiled for advertising or commercial purposes on Fluxer, minors are not either. Invasive verification methods such as government ID uploads or biometric scans are not used for general access; where a legal framework would require methods we do not support, access is restricted as described in Section 3.2 and on the [Regional restrictions](/help/regional-restrictions) page.

### 11.3 If a child below the minimum age is identified

We do not knowingly collect personal information from children below the minimum age for their region; in the United States, that means children under 13, in line with the Children's Online Privacy Protection Act (COPPA). If information from such a child reaches us, we take steps to delete it and, where appropriate, the account. A parent or legal guardian who believes their child has used Fluxer without consent, or does not meet the minimum age, should write to <privacy@fluxer.app> from the child's registered email, or with sufficient proof of guardianship, to request deletion of the account and data.

## 12. Cookies and similar technologies

### 12.1 Approach

Third-party advertising and tracking cookies are not used anywhere on Fluxer (Section 2.2). Operational logging and limited feature-usage telemetry live server-side and are not used for advertising or cross-site profiling.

### 12.2 Cookies we set

A small number of cookies are set, all strictly necessary for operation and security, which under the ePrivacy Directive do not require consent:

- **`locale`:** remembers your language preference. Lasts 1 year; set on the marketing site (`fluxer.app`).
- **`csrf_token`:** protects against cross-site request forgery (CSRF) attacks. Lasts 24 hours; set on the marketing site.
- **`__flx_sudo` or `__flx_sudo_<user_id>`:** verifies your identity during sensitive account operations. Lasts 5 minutes; set in the Fluxer application. Sudo-mode cookies tied to a specific account have the user ID appended to the cookie name.

### 12.3 Client-side storage

The Fluxer application does not use cookies for authentication or session management. It uses your browser's local and session storage for preferences such as theme, media volume, and playback settings. That data stays on your device and is not sent to our servers.

### 12.4 Third-party cookies

Embedded third-party content may set its own cookies when you interact with it. hCaptcha may set cookies during CAPTCHA challenge completion, for bot detection, governed by [hCaptcha's privacy policy](https://www.hcaptcha.com/privacy). YouTube may set cookies when you play an embedded video (Section 13), governed by [Google's privacy policy](https://policies.google.com/privacy).

### 12.5 Managing cookies

Cookies can be controlled through your browser settings, though because all Fluxer cookies are strictly necessary, disabling them may stop some features from working. If non-essential cookies are ever introduced, for example analytics cookies, this section will be updated and consent obtained before they are set.

### 12.6 Opt-out preference signals

Browser-level opt-out signals such as Global Privacy Control (GPC) are honoured and recognised as valid opt-out requests as the CCPA requires. Because we do not sell or share personal information for advertising, they do not change underlying processing. Do Not Track (DNT) signals are not treated differently, as there is no industry consensus on interpreting them; in practice, Fluxer already reflects the intent behind DNT, since users are not tracked across third-party sites.

## 13. Third-party services and links

Fluxer may include links to, or integrations with, third-party services.

_GIF search (Tenor, KLIPY)._ Search queries and GIF embedding are both proxied through our servers. These providers never see your IP address or device identifiers.

_Links sent in messages._ Sending a URL in a message may cause our backend to fetch it to generate a rich embed or embedded media. Such requests identify themselves with a `User-Agent` string containing `Fluxerbot`. Site operators can block requests whose `User-Agent` contains `Fluxerbot`; doing so prevents rich embeds and embedded media from appearing in Fluxer when someone links to the site.

_YouTube links._ Video metadata is fetched server-side from the YouTube API so previews render without your device contacting YouTube. Playing an embedded video loads content directly from YouTube, which may collect information under its own privacy policy.

_Other third-party content._ Embedded third-party content may load directly from that third party upon interaction, with information collected under the third party's own terms.

Third-party services operate under their own privacy policies and data practices, which apply alongside ours when you use them.

## 14. Law enforcement and legal requests

Every legal request for user data receives careful review, with the privacy and security of the people involved as the primary consideration. Requests should be directed to <legal@fluxer.app> and must identify the requesting authority, legal basis, and scope of data requested. Overbroad, legally invalid, or inconsistent requests may be narrowed or refused. Where the law allows, and where notice would not create a safety, security, or legal-process risk, affected users are notified before disclosure so they have an opportunity to object. In genuine emergencies, disclosure may occur without prior notice where reasonably necessary to prevent harm, protect safety, or respond to an urgent situation, in line with applicable law.

## 15. Changes to this policy

This policy may be updated to reflect changes in our practices, services, or legal obligations. Material changes come with at least 30 days' advance notice through email, in-app notification, or a notice on our website, and the effective date at the top is updated. In the app, a persistent notice may link to the new version, and you may be asked to review and acknowledge the changes so we have a record that you saw them.

After the effective date, the updated policy applies to your continued use of Fluxer. If you disagree with an updated policy, you can export your data, delete your messages, and delete your account at any time, using the tools in Section 8. A [changelog](/changelog) is maintained for reference.

## 16. Contact

**Privacy and data protection:** <privacy@fluxer.app> (Hampus Kraft, Founder and CEO)
**General support:** <support@fluxer.app>

Our postal address, phone number, and all other contact routes (press, security, legal requests) are listed in Section 1 and on our [Company Information page](/company-information).

For account-related requests, write from the email address on your Fluxer account where possible; it makes verifying your identity easier and protects the account.

## 17. Additional information for California residents

This section sets out the further disclosures required by the California Consumer Privacy Act, as amended by the California Privacy Rights Act (together, "CCPA"). It applies only to California residents and sits alongside the rest of the policy.

### 17.1 Categories of personal information collected

- **Identifiers:** username, email address, user ID, IP address, device identifiers. Sources: you, automatic collection, service providers.
- **Customer records (Cal. Civ. Code § 1798.80(e)):** billing country, partial payment card details (via Stripe). Sources: you, Stripe.
- **Internet or other electronic network activity:** pages visited, features used, session timestamps, crash reports, browser type, OS. Source: automatic collection.
- **Geolocation data:** approximate location (city/region/country) derived from IP address. Source: automatic collection.
- **Audio, electronic, visual, or similar information:** voice and video communications (where supported), uploaded images and files. Source: you.
- **Inferences:** approximate region for eligibility checks, spam/abuse risk signals. Source: automatic collection.

Biometric information, professional or employment information, and education information are not collected.

### 17.2 Sensitive personal information

Of the categories of "sensitive personal information" defined by the CCPA, only account log-in credentials (email address combined with password) are collected, for the purposes of running the service and securing the account.

From 1 January 2026, the CPRA also classifies personal information of consumers under 16 as sensitive. Because Fluxer permits account creation from age 13 (depending on jurisdiction), information meeting that definition may be collected and processed. It is not used or disclosed beyond what is needed to run the service.

### 17.3 Business purposes for collection

Personal information is collected and used for the purposes described in Section 3: running the service, securing accounts, preventing abuse, processing payments, improving reliability, and meeting legal obligations.

### 17.4 Categories disclosed for a business purpose

- **Identifiers:** to infrastructure providers (Vultr, Bunny.net), Stripe for payments, Sweego and Twilio for communications, Intercom for support, and IPinfo for registration and anti-abuse IP network signals. Purposes: hosting, delivery, payments, support, registration checks, and abuse prevention.
- **Customer records:** to Stripe, for payment processing.
- **Internet or electronic network activity:** to hCaptcha and IPinfo, for bot prevention and registration and abuse-prevention IP network signals.
- **Geolocation data:** to IPinfo, only for registration and abuse-prevention IP lookups when local databases are insufficient (IP address only, no account context; results cached).
- **Audio, electronic, visual, or similar information:** to Bunny.net, for content delivery.

### 17.5 Sale and sharing of personal information

Personal information is not sold. It is not "sold" or "shared" (using the CCPA's definitions of those terms) for cross-context behavioural advertising or any other purpose. This applies to all consumers, including those under 16.

### 17.6 Retention

Each category of personal information is retained for the periods set out in Section 7.

### 17.7 Your California rights

California residents have the rights listed in Section 10: to know, to delete, to correct, to opt out of sale or sharing (none occurs), and to be free from discrimination. These rights can be exercised by writing to <privacy@fluxer.app> or through the controls described in Section 8. An authorised agent may be designated.

### 17.8 Opt-out preference signals

Global Privacy Control (GPC) and similar signals are honoured, as described in Section 12.6.
