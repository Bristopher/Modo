// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    config::AdminConfig,
    templates::components::{
        badge::{BadgeVariant, badge},
        form::{csrf_input, danger_button, form_actions, submit_button},
        page_container::{card_with_header, detail_row},
    },
};
use maud::{Markup, html};

const INPUT_CLS: &str = "block w-full rounded-md border border-neutral-300 px-3 py-2 text-sm \
                          shadow-sm focus:border-brand-primary focus:outline-none focus:ring-1 \
                          focus:ring-brand-primary";

pub fn billing_tab(
    config: &AdminConfig,
    user_id: &str,
    billing: Option<&serde_json::Value>,
    invoices: Option<&serde_json::Value>,
    csrf_token: &str,
) -> Markup {
    let base = &config.base_path;
    html! {
        div class="space-y-6" {
            @if let Some(data) = billing {
                (render_billing_summary(data))
                (render_subscription(data))
                (render_payment_methods(data))
                (render_payments(data))
            } @else {
                (card_with_header("Billing", html! {
                    p class="text-sm text-neutral-500" {
                        "No billing information available for this user."
                    }
                }))
            }
            @if let Some(data) = invoices {
                (render_invoices(data))
            }

            (render_actions(base, user_id, csrf_token))
        }
    }
}

fn subscription_badge_variant(status: &str) -> BadgeVariant {
    match status {
        "active" | "trialing" => BadgeVariant::Success,
        "past_due" | "unpaid" | "incomplete" => BadgeVariant::Warning,
        "canceled" | "incomplete_expired" => BadgeVariant::Danger,
        _ => BadgeVariant::Default,
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
        .and_then(|v| v.as_str());
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
                (detail_row("Subscription", html! {
                    @if let Some(status) = sub_status {
                        (badge(status, subscription_badge_variant(status)))
                    } @else {
                        span class="text-sm text-neutral-900" { "none" }
                    }
                }))
                @if let Some(end) = period_end {
                    (detail_row("Current Period Ends", html! { (end) }))
                }
            }
        }))
    }
}

fn render_subscription(data: &serde_json::Value) -> Markup {
    let sub = match data.get("subscription") {
        Some(s) if !s.is_null() => s,
        _ => return html! {},
    };
    let status = sub
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let sub_id = sub.get("id").and_then(|v| v.as_str());
    let plan_interval = sub.get("plan_interval").and_then(|v| v.as_str());
    let period_start = sub.get("current_period_start").and_then(|v| v.as_str());
    let period_end = sub.get("current_period_end").and_then(|v| v.as_str());
    let cancel_at_period_end = sub
        .get("cancel_at_period_end")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    html! {
        (card_with_header("Subscription", html! {
            dl class="divide-y divide-neutral-100" {
                (detail_row("Status", html! {
                    (badge(status, subscription_badge_variant(status)))
                }))
                @if let Some(id) = sub_id {
                    (detail_row("ID", html! {
                        span class="text-xs" { (id) }
                    }))
                }
                @if let Some(interval) = plan_interval {
                    (detail_row("Plan Interval", html! { (interval) }))
                }
                @if let Some(start) = period_start {
                    (detail_row("Period Start", html! { (start) }))
                }
                @if let Some(end) = period_end {
                    (detail_row("Period End", html! { (end) }))
                }
                (detail_row("Cancel at Period End", html! {
                    @if cancel_at_period_end { "yes" } @else { "no" }
                }))
            }
        }))
    }
}

fn render_payment_methods(data: &serde_json::Value) -> Markup {
    let methods = data.get("payment_methods").and_then(|v| v.as_array());
    let empty = methods.is_none() || methods.is_some_and(|m| m.is_empty());
    html! {
        (card_with_header("Payment Methods", html! {
            @if empty {
                p class="text-sm text-neutral-500" { "No payment methods on file." }
            } @else if let Some(pms) = methods {
                div class="space-y-3" {
                    @for pm in pms {
                        @let pm_type = pm.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
                        @let brand = pm.get("card_brand").and_then(|v| v.as_str());
                        @let last4 = pm.get("card_last4").and_then(|v| v.as_str());
                        @let pm_id = pm.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        @let display = match (brand, last4) {
                            (Some(b), Some(l)) => format!("{b} **** {l}"),
                            _ => pm_type.to_string(),
                        };
                        div class="rounded-lg border border-neutral-200 bg-neutral-50 p-3" {
                            p class="text-sm text-neutral-900" { (display) }
                            p class="text-xs text-neutral-500" { (pm_id) }
                        }
                    }
                }
            }
        }))
    }
}

fn render_payments(data: &serde_json::Value) -> Markup {
    let payments = data.get("payments").and_then(|v| v.as_array());
    let empty = payments.is_none() || payments.is_some_and(|p| p.is_empty());
    html! {
        (card_with_header("Payments", html! {
            @if empty {
                p class="text-sm text-neutral-500" { "No payments recorded." }
            } @else if let Some(ps) = payments {
                div class="space-y-3" {
                    @for p in ps { (payment_row(p)) }
                }
            }
        }))
    }
}

fn payment_row(p: &serde_json::Value) -> Markup {
    let amount = p.get("amount_cents").and_then(|v| v.as_i64()).unwrap_or(0);
    let currency = p.get("currency").and_then(|v| v.as_str()).unwrap_or("");
    let status = p
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let created = p.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let display_amount = format!("{:.2} {}", amount as f64 / 100.0, currency.to_uppercase());

    let variant = match status {
        "completed" | "succeeded" => BadgeVariant::Success,
        "pending" | "processing" => BadgeVariant::Info,
        "failed" | "canceled" => BadgeVariant::Danger,
        "refunded" | "partially_refunded" => BadgeVariant::Warning,
        _ => BadgeVariant::Default,
    };

    html! {
        div class="rounded-lg border border-neutral-200 bg-neutral-50 p-4" {
            div class="flex items-center justify-between" {
                div class="flex items-center gap-2" {
                    span class="text-sm font-medium text-neutral-900" {
                        (display_amount)
                    }
                    (badge(status, variant))
                }
                span class="text-xs text-neutral-500" { (created) }
            }
        }
    }
}

fn invoice_badge_variant(status: Option<&str>) -> BadgeVariant {
    match status {
        Some("paid") => BadgeVariant::Success,
        Some("open" | "draft") => BadgeVariant::Info,
        Some("uncollectible" | "void") => BadgeVariant::Danger,
        _ => BadgeVariant::Default,
    }
}

fn render_invoices(data: &serde_json::Value) -> Markup {
    let invoices = data.get("invoices").and_then(|v| v.as_array());
    let empty = invoices.is_none() || invoices.is_some_and(|i| i.is_empty());
    let has_more = data
        .get("has_more")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    html! {
        (card_with_header("Invoices", html! {
            @if empty {
                p class="text-sm text-neutral-500" { "No invoices on file." }
            } @else if let Some(items) = invoices {
                div class="space-y-3" {
                    @for invoice in items {
                        (invoice_row(invoice))
                    }
                    @if has_more {
                        p class="text-xs text-neutral-500" {
                            "More invoices exist beyond this list."
                        }
                    }
                }
            }
        }))
    }
}

fn invoice_row(invoice: &serde_json::Value) -> Markup {
    let amount = invoice
        .get("amount_paid")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let currency = invoice
        .get("currency")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let status = invoice.get("status").and_then(|v| v.as_str());
    let created = invoice
        .get("created")
        .and_then(|v| v.as_i64())
        .map(format_unix_timestamp)
        .unwrap_or_default();
    let display_amount = format_amount(amount, currency);
    let status_label = status.unwrap_or("unknown");
    let billing_reason = invoice.get("billing_reason").and_then(|v| v.as_str());
    let invoice_id = invoice.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let subscription_id = invoice.get("subscription_id").and_then(|v| v.as_str());
    let payment_intent_id = invoice.get("payment_intent_id").and_then(|v| v.as_str());
    let charge_id = invoice.get("charge_id").and_then(|v| v.as_str());
    let hosted_invoice_url = invoice.get("hosted_invoice_url").and_then(|v| v.as_str());
    let invoice_pdf = invoice.get("invoice_pdf").and_then(|v| v.as_str());
    html! {
        div class="rounded-lg border border-neutral-200 bg-neutral-50 p-4" {
            div class="space-y-3" {
                div class="flex items-center justify-between gap-3" {
                    div class="flex items-center gap-2" {
                        span class="text-sm font-medium text-neutral-900" {
                            (display_amount)
                        }
                        (badge(status_label, invoice_badge_variant(status)))
                    }
                    span class="text-xs text-neutral-500" { (created) }
                }
                @if let Some(reason) = billing_reason {
                    p class="text-sm text-neutral-500" { (reason) }
                }
                dl class="space-y-1" {
                    (compact_detail_row("id", invoice_id))
                    @if let Some(id) = subscription_id {
                        (compact_detail_row("subscription", id))
                    }
                    @if let Some(id) = payment_intent_id {
                        (compact_detail_row("payment_intent", id))
                    }
                    @if let Some(id) = charge_id {
                        (compact_detail_row("charge", id))
                    }
                }
                @if hosted_invoice_url.is_some() || invoice_pdf.is_some() {
                    div class="flex items-center gap-3 text-sm" {
                        @if let Some(url) = hosted_invoice_url {
                            a href=(url) target="_blank" rel="noreferrer noopener"
                                class="text-blue-600 hover:text-blue-800 hover:underline" {
                                "View"
                            }
                        }
                        @if let Some(url) = invoice_pdf {
                            a href=(url) target="_blank" rel="noreferrer noopener"
                                class="text-blue-600 hover:text-blue-800 hover:underline" {
                                "PDF"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn compact_detail_row(label: &str, value: &str) -> Markup {
    html! {
        div class="grid grid-cols-1 gap-1 text-xs sm:grid-cols-3" {
            dt class="text-neutral-500" { (label) }
            dd class="break-all text-neutral-700 sm:col-span-2" { (value) }
        }
    }
}

fn format_amount(amount_minor: i64, currency: &str) -> String {
    let code = currency.trim().to_uppercase();
    if code.is_empty() {
        format!("{:.2}", amount_minor as f64 / 100.0)
    } else {
        format!("{:.2} {code}", amount_minor as f64 / 100.0)
    }
}

fn format_unix_timestamp(value: i64) -> String {
    time::OffsetDateTime::from_unix_timestamp(value)
        .ok()
        .and_then(|ts| {
            ts.format(&time::format_description::well_known::Rfc3339)
                .ok()
        })
        .unwrap_or_else(|| value.to_string())
}

fn render_actions(base: &str, user_id: &str, csrf_token: &str) -> Markup {
    html! {
        (card_with_header("Billing Actions", html! {
            div class="space-y-4" {
                form method="post"
                    action={(base) "/users/" (user_id) "?tab=billing&action=cancel_subscription_now"} {
                    (csrf_input(csrf_token))
                    div class="space-y-3" {
                        p class="text-sm text-neutral-700" {
                            "Cancel subscription immediately, no refund."
                        }
                        input type="text" name="reason" placeholder="Reason (optional)"
                            class=(INPUT_CLS);
                        (form_actions(html! {
                            (danger_button("Cancel Now"))
                        }))
                    }
                }

                form method="post"
                    action={(base) "/users/" (user_id) "?tab=billing&action=cancel_subscription"} {
                    (csrf_input(csrf_token))
                    div class="space-y-3" {
                        p class="text-sm text-neutral-700" {
                            "Cancel at renewal (access until period end)."
                        }
                        (form_actions(html! {
                            (submit_button("Cancel at Renewal"))
                        }))
                    }
                }

                form method="post"
                    action={(base) "/users/" (user_id) "?tab=billing&action=refund_payment"} {
                    (csrf_input(csrf_token))
                    div class="space-y-3" {
                        p class="text-sm font-medium text-neutral-700" {
                            "Manual Refund"
                        }
                        div class="grid grid-cols-1 gap-3 sm:grid-cols-2" {
                            input type="text" name="payment_intent_id"
                                placeholder="pi_..." required
                                class=(INPUT_CLS);
                            input type="number" name="amount_cents" min="1"
                                placeholder="Amount cents (blank = full)"
                                class=(INPUT_CLS);
                        }
                        input type="text" name="reason"
                            placeholder="Reason (optional)"
                            class=(INPUT_CLS);
                        (form_actions(html! {
                            (danger_button("Refund"))
                        }))
                    }
                }
            }
        }))
    }
}
