// SPDX-License-Identifier: AGPL-3.0-or-later

crate::marketing_message!(
    pub const SECURITY_FOUND_SECURITY_ISSUE_DESCRIPTOR = {
        key: "security.found_security_issue",
        message: "Found a security issue?",
        comment: "Compact UI label in security, bug bounty, or responsible disclosure sections. Keep wording precise, calm, and trustworthy; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const SECURITY_RESPONSIBLE_DISCLOSURE_NOTE_DESCRIPTOR = {
        key: "security.responsible_disclosure_note",
        message: "We appreciate responsible disclosure via our security bug bounty page. We offer {premium_tier_name} codes and {bug_hunter} badges based on severity.",
        comment: "Body copy in security, bug bounty, or responsible disclosure sections. Keep wording precise, calm, and trustworthy; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const SECURITY_SECURITY_BUG_BOUNTY_DESCRIPTOR = {
        key: "security.security_bug_bounty",
        message: "Security bug bounty",
        comment: "Compact UI label in security, bug bounty, or responsible disclosure sections. Keep wording precise, calm, and trustworthy; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const SECURITY_TESTERS_ACCESS_FROM_REPORTS_DESCRIPTOR = {
        key: "security.testers_access_from_reports",
        message: "Found a bug? Check out our bug report guide to learn how to file clear, high-quality reports.",
        comment: "Body copy in security, bug bounty, or responsible disclosure sections. Keep wording precise, calm, and trustworthy; preserve placeholders exactly.",
    };
);
