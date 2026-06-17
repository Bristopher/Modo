// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    config::AdminConfig,
    middleware::auth::AuthContext,
    templates::{
        components::{form::checkbox, page_container::page_header},
        layout::admin_layout,
        pages::blocklist_helpers::{
            BlocklistActionVariant, blocklist_action_card, blocklist_text_field,
        },
    },
};
use maud::{Markup, html};

pub fn url_domain_bans_page(
    config: &AdminConfig,
    auth: &AuthContext,
    flash: Option<&crate::api::types::FlashMessage>,
    csrf_token: &str,
) -> Markup {
    let base = &config.base_path;
    let content = html! {
        (page_header("URL Domain Blocklist", None))
        div class="grid gap-6 lg:grid-cols-2" {
            (ban_card(base, csrf_token))
            (check_card(base, csrf_token))
        }
        div class="mt-6" {
            (unban_card(base, csrf_token))
        }
    };
    admin_layout(
        config,
        auth,
        "URL Domain Blocklist",
        "url-domain-bans",
        flash,
        content,
    )
}

fn ban_card(base: &str, csrf_token: &str) -> Markup {
    let action_url = format!("{base}/url-domain-bans?action=ban&_csrf={csrf_token}");
    blocklist_action_card(
        "Ban URL Domain",
        &action_url,
        csrf_token,
        html! {
            (blocklist_text_field("domain", "Domain", "example.com", true))
            (checkbox("match_subdomains", "true", "Match subdomains (e.g. sub.example.com)", true, true))
            (blocklist_text_field("audit_log_reason", "Private reason (audit log, optional)", "Why is this ban being applied?", false))
        },
        "Ban Domain",
        BlocklistActionVariant::Primary,
    )
}

fn check_card(base: &str, csrf_token: &str) -> Markup {
    let action_url = format!("{base}/url-domain-bans?action=check&_csrf={csrf_token}");
    blocklist_action_card(
        "Check Domain Ban Status",
        &action_url,
        csrf_token,
        html! {
            (blocklist_text_field("domain", "Domain", "example.com", true))
        },
        "Check Status",
        BlocklistActionVariant::Primary,
    )
}

fn unban_card(base: &str, csrf_token: &str) -> Markup {
    let action_url = format!("{base}/url-domain-bans?action=unban&_csrf={csrf_token}");
    blocklist_action_card(
        "Remove Domain Ban",
        &action_url,
        csrf_token,
        html! {
            (blocklist_text_field("domain", "Domain", "example.com", true))
            (blocklist_text_field("audit_log_reason", "Private reason (audit log, optional)", "Why is this ban being removed?", false))
        },
        "Unban Domain",
        BlocklistActionVariant::Danger,
    )
}
