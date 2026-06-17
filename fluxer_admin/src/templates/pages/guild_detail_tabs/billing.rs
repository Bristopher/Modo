// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    config::AdminConfig,
    templates::components::{
        form::{csrf_input, danger_button, form_actions, submit_button},
        page_container::{card_with_header, detail_row},
    },
};
use maud::{Markup, html};

pub fn billing_tab(
    config: &AdminConfig,
    guild_id: &str,
    billing: Option<&serde_json::Value>,
    csrf_token: &str,
) -> Markup {
    let base = &config.base_path;
    html! {
        div class="space-y-6" {
            @if let Some(data) = billing {
                (render_billing_summary(data))
            } @else {
                (card_with_header("Billing", html! {
                    p class="text-sm text-neutral-500" {
                        "No billing information available for this guild."
                    }
                }))
            }

            (card_with_header("Billing Actions", html! {
                div class="space-y-4" {
                    form method="post"
                        action={(base) "/guilds/" (guild_id) "?tab=billing&action=refresh_billing"}
                        class="block" {
                        (csrf_input(csrf_token))
                        (form_actions(html! {
                            (submit_button("Refresh Billing Data"))
                        }))
                    }

                    form method="post"
                        action={(base) "/guilds/" (guild_id) "?tab=billing&action=cancel_subscription"} {
                        (csrf_input(csrf_token))
                        div class="space-y-3" {
                            input type="text" name="reason" placeholder="Reason (optional)"
                                class="block w-full rounded-md border border-neutral-300 px-3 \
                                       py-2 text-sm shadow-sm focus:border-brand-primary \
                                       focus:outline-none focus:ring-1 focus:ring-brand-primary";
                            (form_actions(html! {
                                (danger_button("Cancel Subscription"))
                            }))
                        }
                    }
                }
            }))
        }
    }
}

fn render_billing_summary(data: &serde_json::Value) -> Markup {
    let customer_id = data
        .get("stripe_customer_id")
        .and_then(|v| v.as_str())
        .unwrap_or("\u{2014}");
    let sub_status = data
        .get("subscription")
        .and_then(|s| s.get("status"))
        .and_then(|v| v.as_str())
        .unwrap_or("none");
    let period_end = data
        .get("subscription")
        .and_then(|s| s.get("current_period_end"))
        .and_then(|v| v.as_str());

    html! {
        (card_with_header("Summary", html! {
            dl class="divide-y divide-neutral-100" {
                (detail_row("Stripe Customer", html! {
                    span class="text-xs" { (customer_id) }
                }))
                (detail_row("Subscription Status", html! {
                    span class="inline-flex items-center rounded-full px-2 py-0.5 text-xs \
                                font-medium bg-neutral-100 text-neutral-700" {
                        (sub_status)
                    }
                }))
                @if let Some(end) = period_end {
                    (detail_row("Current Period Ends", html! { (end) }))
                }
            }
        }))
    }
}
