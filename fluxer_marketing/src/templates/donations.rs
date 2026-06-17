// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{
    PageMeta, content_layout_with_footer_class,
    icons::{Icon, icon},
    tr,
};
use crate::{
    i18n::{MarketingI18n, MarketingMessageDescriptor, descriptors::*},
    invariant_text::{SWISH_BRAND_NAME, SWISH_PAYMENT_MESSAGE},
    pricing::{Currency, format_major_amount, get_base_currency, get_currency},
    request_context::{Platform, RequestContext},
};
use maud::{Markup, PreEscaped, html};

const SWISH_LOGO_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 420 420" fill-rule="evenodd" class="h-8 w-8 shrink-0" aria-hidden="true" focusable="false">
<defs>
<linearGradient id="swish-grad-1" x1="-746" y1="822.6" x2="-746.2" y2="823.1" gradientTransform="translate(224261.6 305063) scale(300.3 -370.5)" gradientUnits="userSpaceOnUse">
<stop offset="0" stop-color="#ef2131"/><stop offset="1" stop-color="#fecf2c"/>
</linearGradient>
<linearGradient id="swish-grad-2" x1="-745.4" y1="823" x2="-745.9" y2="822.1" gradientTransform="translate(204470.4 247194.2) scale(273.8 -300.2)" gradientUnits="userSpaceOnUse">
<stop offset="0" stop-color="#fbc52c"/><stop offset=".3" stop-color="#f87130"/><stop offset=".6" stop-color="#ef52e2"/><stop offset="1" stop-color="#661eec"/>
</linearGradient>
<linearGradient id="swish-grad-3" x1="-746" y1="823" x2="-745.8" y2="822.5" gradientTransform="translate(224142 305014) scale(300.3 -370.5)" gradientUnits="userSpaceOnUse">
<stop offset="0" stop-color="#78f6d8"/><stop offset=".3" stop-color="#77d1f6"/><stop offset=".6" stop-color="#70a4f3"/><stop offset="1" stop-color="#661eec"/>
</linearGradient>
<linearGradient id="swish-grad-4" x1="-746.1" y1="822.3" x2="-745.6" y2="823.2" gradientTransform="translate(204377.3 247074.5) scale(273.8 -300.2)" gradientUnits="userSpaceOnUse">
<stop offset="0" stop-color="#536eed"/><stop offset=".2" stop-color="#54c3ec"/><stop offset=".6" stop-color="#64d769"/><stop offset="1" stop-color="#fecf2c"/>
</linearGradient>
</defs>
<g>
<path fill="url(#swish-grad-1)" d="M119.3,399.2c84.3,40.3,188.3,20.4,251.2-54.5,74.5-88.8,62.9-221.1-25.8-295.5l-59,70.3c69.3,58.2,78.4,161.5,20.2,230.9-46.4,55.3-122.8,73.7-186.5,48.9"/>
<path fill="url(#swish-grad-2)" d="M119.3,399.2c84.3,40.3,188.3,20.4,251.2-54.5,7.7-9.2,14.5-18.8,20.3-28.8,9.9-61.7-11.9-126.9-63.2-169.9-13-10.9-27.2-19.8-41.9-26.5,69.3,58.2,78.4,161.5,20.2,230.9-46.4,55.3-122.8,73.7-186.5,48.9"/>
<path fill="url(#swish-grad-3)" d="M300.3,20.4C216-19.9,111.9,0,49.1,74.9c-74.5,88.8-62.9,221.1,25.8,295.5l59-70.3c-69.3-58.2-78.4-161.5-20.2-230.9C160.2,14,236.6-4.5,300.3,20.4"/>
<path fill="url(#swish-grad-4)" d="M300.3,20.4C216-19.9,111.9,0,49.1,74.9c-7.7,9.2-14.5,18.8-20.3,28.8-9.9,61.7,11.9,126.9,63.2,169.9,13,10.9,27.2,19.8,41.9,26.5-69.3-58.2-78.4-161.5-20.2-230.9C160.2,14,236.6-4.5,300.3,20.4"/>
</g>
</svg>"##;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DonationAudience {
    Individual,
    Business,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DonationCurrency {
    Usd,
    Eur,
    Brl,
    Inr,
    Pln,
    Try,
}

#[derive(Clone, Copy, Debug)]
struct DonationCurrencyOption {
    code: DonationCurrency,
    label: &'static str,
}

#[derive(Clone, Copy, Debug)]
struct DonationAmountConstraints {
    display_currency: Currency,
    minimum_amount_major: u32,
    maximum_amount_major: u32,
    preset_amounts_major: [u32; 5],
    default_preset_index: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DonatePageOptions<'a> {
    pub donation_type: Option<&'a str>,
    pub error: Option<&'a str>,
    pub currency: Option<&'a str>,
    pub swish_open: bool,
    pub swish_amount: Option<&'a str>,
}

impl DonationAudience {
    pub fn from_query(value: Option<&str>) -> Self {
        if value == Some("business") {
            Self::Business
        } else {
            Self::Individual
        }
    }

    const fn id(self) -> &'static str {
        match self {
            Self::Individual => "individual",
            Self::Business => "business",
        }
    }

    const fn label(self) -> MarketingMessageDescriptor {
        match self {
            Self::Individual => DONATIONS_INDIVIDUAL_DESCRIPTOR,
            Self::Business => DONATIONS_BUSINESS_DESCRIPTOR,
        }
    }
}

impl DonationCurrency {
    const ALL: [Self; 6] = [
        Self::Usd,
        Self::Eur,
        Self::Brl,
        Self::Inr,
        Self::Pln,
        Self::Try,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Usd => "usd",
            Self::Eur => "eur",
            Self::Brl => "brl",
            Self::Inr => "inr",
            Self::Pln => "pln",
            Self::Try => "try",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Usd => "$",
            Self::Eur => "€",
            Self::Brl => "R$",
            Self::Inr => "₹",
            Self::Pln => "zł",
            Self::Try => "₺",
        }
    }

    const fn constraints(self) -> DonationAmountConstraints {
        match self {
            Self::Usd => DonationAmountConstraints {
                display_currency: Currency::Usd,
                minimum_amount_major: 5,
                maximum_amount_major: 1000,
                preset_amounts_major: [5, 25, 50, 100, 500],
                default_preset_index: 1,
            },
            Self::Eur => DonationAmountConstraints {
                display_currency: Currency::Eur,
                minimum_amount_major: 5,
                maximum_amount_major: 1000,
                preset_amounts_major: [5, 25, 50, 100, 500],
                default_preset_index: 1,
            },
            Self::Brl => DonationAmountConstraints {
                display_currency: Currency::Brl,
                minimum_amount_major: 25,
                maximum_amount_major: 5000,
                preset_amounts_major: [25, 50, 100, 250, 500],
                default_preset_index: 1,
            },
            Self::Inr => DonationAmountConstraints {
                display_currency: Currency::Inr,
                minimum_amount_major: 500,
                maximum_amount_major: 100_000,
                preset_amounts_major: [500, 1000, 2500, 5000, 10_000],
                default_preset_index: 1,
            },
            Self::Pln => DonationAmountConstraints {
                display_currency: Currency::Pln,
                minimum_amount_major: 20,
                maximum_amount_major: 4000,
                preset_amounts_major: [20, 50, 100, 250, 500],
                default_preset_index: 1,
            },
            Self::Try => DonationAmountConstraints {
                display_currency: Currency::Try,
                minimum_amount_major: 250,
                maximum_amount_major: 50_000,
                preset_amounts_major: [250, 500, 1000, 2500, 5000],
                default_preset_index: 1,
            },
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "usd" => Some(Self::Usd),
            "eur" => Some(Self::Eur),
            "brl" => Some(Self::Brl),
            "inr" => Some(Self::Inr),
            "pln" => Some(Self::Pln),
            "try" => Some(Self::Try),
            _ => None,
        }
    }
}

impl From<Currency> for DonationCurrency {
    fn from(value: Currency) -> Self {
        match value {
            Currency::Usd => Self::Usd,
            Currency::Eur => Self::Eur,
            Currency::Brl => Self::Brl,
            Currency::Inr => Self::Inr,
            Currency::Pln => Self::Pln,
            Currency::Try => Self::Try,
        }
    }
}

pub fn donate_page(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    options: DonatePageOptions<'_>,
) -> Markup {
    let selected = DonationAudience::from_query(options.donation_type);
    let is_individual = selected == DonationAudience::Individual;
    let is_business = selected == DonationAudience::Business;
    let selected_currency = options.currency.and_then(DonationCurrency::from_code);
    let error_message = donation_query_error_message(i18n, ctx, options.error, options.currency);
    content_layout_with_footer_class(
        i18n,
        ctx,
        PageMeta {
            title: tr(i18n, ctx, DONATIONS_DONATE_LABEL_DESCRIPTOR),
            description: tr(i18n, ctx, DONATIONS_SUPPORT_MESSAGE_DESCRIPTOR),
            enable_htmx: true,
            ..Default::default()
        },
        html! {
            section class="mx-auto max-w-2xl" {
                header class="mb-10 text-center" {
                    h1 class="mb-4 font-bold text-4xl text-foreground" {
                        (tr(i18n, ctx, DONATIONS_DONATE_LABEL_DESCRIPTOR))
                    }
                    p class="text-lg text-muted-foreground" {
                        (tr(i18n, ctx, DONATIONS_SUPPORT_MESSAGE_DESCRIPTOR))
                    }
                }
                div id="donation-interaction" {
                    div class="mb-8 flex justify-center" {
                        div class="inline-flex rounded-lg bg-gray-100 p-1" {
                            (donation_tab(i18n, ctx, DonationAudience::Individual, is_individual))
                            (donation_tab(i18n, ctx, DonationAudience::Business, is_business))
                        }
                    }
                    div class="mb-6 flex flex-col items-center gap-3" {
                        (donation_form(
                            i18n,
                            ctx,
                            DonationAudience::Individual,
                            !is_individual,
                            selected_currency,
                            if is_individual { error_message.as_deref() } else { None },
                            options.swish_open,
                            options.swish_amount,
                        ))
                        (donation_form(
                            i18n,
                            ctx,
                            DonationAudience::Business,
                            !is_business,
                            selected_currency,
                            if is_business { error_message.as_deref() } else { None },
                            false,
                            None,
                        ))
                    }
                }
                div class="mt-12 border-gray-200 border-t pt-8" {
                    h2 class="mb-4 text-center font-semibold text-foreground text-lg" {
                        (tr(i18n, ctx, DONATIONS_MANAGE_TITLE_DESCRIPTOR))
                    }
                    p class="mb-4 text-center text-muted-foreground text-sm" {
                        (tr(i18n, ctx, DONATIONS_MANAGE_DESCRIPTION_DESCRIPTOR))
                    }
                    (manage_form(i18n, ctx, ""))
                }
            }
        },
        "rounded-t-3xl",
    )
}

pub fn donate_manage_page(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    email: &str,
    alert: Option<&str>,
) -> Markup {
    let alert = manage_alert_message(i18n, ctx, alert);
    content_layout_with_footer_class(
        i18n,
        ctx,
        PageMeta {
            title: tr(i18n, ctx, DONATIONS_MANAGE_TITLE_DESCRIPTOR),
            description: tr(i18n, ctx, DONATIONS_MANAGE_DESCRIPTION_DESCRIPTOR),
            enable_htmx: true,
            ..Default::default()
        },
        html! {
            section class="mx-auto max-w-2xl" {
                header class="mb-10 text-center" {
                    h1 class="mb-4 font-bold text-4xl text-foreground" { (tr(i18n, ctx, DONATIONS_MANAGE_TITLE_DESCRIPTOR)) }
                    p class="text-lg text-muted-foreground" { (tr(i18n, ctx, DONATIONS_MANAGE_DESCRIPTION_DESCRIPTOR)) }
                }
                @if let Some((message, tone)) = alert {
                    div class=(format!("mb-6 rounded-lg border p-4 text-center {}", tone.container_class)) {
                        p class=(format!("font-medium {}", tone.text_class)) { (message) }
                    }
                }
                (manage_form(i18n, ctx, email))
            }
        },
        "rounded-t-3xl",
    )
}

pub fn donate_success_page(i18n: &MarketingI18n, ctx: &RequestContext) -> Markup {
    content_layout_with_footer_class(
        i18n,
        ctx,
        PageMeta {
            title: tr(i18n, ctx, DONATIONS_SUCCESS_TITLE_DESCRIPTOR),
            description: tr(i18n, ctx, DONATIONS_SUCCESS_MESSAGE_DESCRIPTOR),
            ..Default::default()
        },
        html! {
            section class="mx-auto max-w-2xl text-center" {
                div class="mb-8" {
                    div class="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-green-100" {
                        svg class="h-10 w-10 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" {}
                        }
                    }
                    h1 class="mb-4 font-bold text-4xl text-foreground" { (tr(i18n, ctx, DONATIONS_SUCCESS_TITLE_DESCRIPTOR)) }
                    p class="text-lg text-muted-foreground" { (tr(i18n, ctx, DONATIONS_SUCCESS_MESSAGE_DESCRIPTOR)) }
                }
                div class="space-y-4" {
                    p class="text-muted-foreground" { (tr(i18n, ctx, DONATIONS_SUCCESS_EMAIL_NOTICE_DESCRIPTOR)) }
                    a class="inline-block rounded-xl bg-[#4641D9] px-8 py-3 font-semibold text-white transition-colors hover:bg-[#3d38c7]" href=(ctx.href("/donate")) {
                        (tr(i18n, ctx, DONATIONS_SUCCESS_BACK_TO_DONATE_DESCRIPTOR))
                    }
                }
            }
        },
        "rounded-t-3xl",
    )
}

const TAB_ACTIVE_CLASS: &str = "donate-tab donate-tab-active rounded-md bg-[#4641D9] px-6 py-2 font-medium text-sm text-white transition-all";
const TAB_INACTIVE_CLASS: &str = "donate-tab rounded-md px-6 py-2 font-medium text-sm text-gray-600 transition-all hover:text-gray-900";

fn donation_tab(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    audience: DonationAudience,
    selected: bool,
) -> Markup {
    let audience_id = audience.id();
    let class = if selected {
        TAB_ACTIVE_CLASS
    } else {
        TAB_INACTIVE_CLASS
    };
    html! {
        a
            id=(format!("donate-tab-{audience_id}"))
            href=(ctx.href(&format!("/donate?type={audience_id}")))
            hx-get=(ctx.href(&format!("/donate?type={audience_id}")))
            hx-select="#donation-interaction"
            hx-target="#donation-interaction"
            hx-swap="outerHTML"
            hx-push-url="true"
            data-donate-tab=(audience_id)
            aria-current=[selected.then_some("page")]
            class=(class) {
            (tr(i18n, ctx, audience.label()))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn donation_form(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    audience: DonationAudience,
    hidden: bool,
    selected_currency: Option<DonationCurrency>,
    error_message: Option<&str>,
    swish_open: bool,
    swish_amount: Option<&str>,
) -> Markup {
    let currencies = donation_currencies(&ctx.country_code);
    let default_currency = currencies
        .first()
        .map(|currency| currency.code)
        .unwrap_or(DonationCurrency::Usd);
    let selected_currency = selected_currency.unwrap_or(default_currency);
    let audience_id = audience.id();
    let error_id = format!("donation-error-{audience_id}");
    html! {
        div id=(format!("donate-content-{audience_id}")) class=(if hidden { "donate-content hidden w-full" } else { "donate-content w-full" }) {
            form
                class="mx-auto max-w-lg space-y-6"
                method="post"
                action=(ctx.href("/_donations/checkout"))
                hx-post=(ctx.href("/_donations/checkout"))
                hx-target=(format!("#donation-error-{audience_id}"))
                hx-swap="outerHTML"
                hx-push-url="false" {
                input type="hidden" name="audience" value=(audience_id);
                div {
                    label for=(format!("donation-email-{audience_id}")) class="mb-2 block text-sm font-medium text-gray-950" {
                        (tr(i18n, ctx, DONATIONS_FORM_EMAIL_DESCRIPTOR))
                    }
                    input id=(format!("donation-email-{audience_id}")) name="email" type="email" required placeholder=(tr(i18n, ctx, DONATIONS_FORM_EMAIL_PLACEHOLDER_DESCRIPTOR)) class="w-full rounded-lg border border-gray-300 px-4 py-3 text-gray-950 focus:border-[#4641d9] focus:outline-none";
                }
                (donation_amount_fieldset(i18n, ctx, audience, selected_currency))
                fieldset {
                    legend class="mb-2 block text-sm font-medium text-gray-950" { (tr(i18n, ctx, DONATIONS_FORM_DONATION_TYPE_DESCRIPTOR)) }
                    div class="grid grid-cols-3 gap-2" {
                        (donation_interval_radio(i18n, ctx, "once", DONATIONS_FORM_ONE_TIME_DESCRIPTOR, true))
                        (donation_interval_radio(i18n, ctx, "month", DONATIONS_FORM_MONTHLY_DESCRIPTOR, false))
                        (donation_interval_radio(i18n, ctx, "year", DONATIONS_FORM_YEARLY_DESCRIPTOR, false))
                    }
                }
                fieldset {
                    legend class="mb-2 block text-sm font-medium text-gray-950" { (tr(i18n, ctx, DONATIONS_FORM_CURRENCY_DESCRIPTOR)) }
                    div class="grid grid-cols-2 gap-2 sm:grid-cols-3" {
                        @for currency in currencies {
                            (donation_currency_radio(ctx, audience, currency, currency.code == selected_currency))
                        }
                    }
                }
                button type="submit" id=(format!("donate-btn-{audience_id}")) class="w-full rounded-xl bg-[#4641d9] py-3 font-semibold text-white transition-colors hover:bg-[#3832b8] disabled:cursor-not-allowed disabled:opacity-60" {
                    (tr(i18n, ctx, DONATIONS_DONATE_ACTION_DESCRIPTOR))
                }
                @if audience == DonationAudience::Individual {
                    div class="flex items-center gap-3 py-1 text-xs font-semibold uppercase text-gray-500" {
                        span class="h-px flex-1 bg-gray-200" {}
                        span { (tr(i18n, ctx, DONATIONS_FORM_OR_LABEL_DESCRIPTOR)) }
                        span class="h-px flex-1 bg-gray-200" {}
                    }
                    a id="swish-donate-link" href="#swish-modal-backdrop" aria-controls="swish-modal-backdrop" class="flex h-12 w-full items-center justify-center gap-2 rounded-xl border border-gray-200 bg-white px-6 font-semibold text-gray-900 transition-colors hover:bg-gray-50" {
                        (swish_logo_svg("donate-link"))
                        (SWISH_BRAND_NAME)
                    }
                }
                @if let Some(message) = error_message {
                    (donation_error_message(&error_id, message))
                } @else {
                    (donation_empty_message(&error_id))
                }
            }
            @if audience == DonationAudience::Individual {
                (swish_modal(i18n, ctx, swish_open, swish_amount))
            }
        }
    }
}

pub fn donation_amount_fieldset_fragment(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    audience: DonationAudience,
    currency: DonationCurrency,
) -> Markup {
    donation_amount_fieldset(i18n, ctx, audience, currency)
}

fn donation_amount_fieldset(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    audience: DonationAudience,
    currency: DonationCurrency,
) -> Markup {
    let constraints = currency.constraints();
    let audience_id = audience.id();
    html! {
        fieldset id=(format!("donation-amount-fieldset-{audience_id}")) {
            legend class="mb-2 block text-sm font-medium text-gray-950" { (tr(i18n, ctx, DONATIONS_FORM_AMOUNT_DESCRIPTOR)) }
            div class="flex flex-col gap-2" {
                @for (preset_index, amount) in constraints.preset_amounts_major.iter().enumerate() {
                    label class="block" {
                        input
                            class="peer sr-only"
                            type="radio"
                            name="amount_major"
                            value=(*amount)
                            checked[preset_index == constraints.default_preset_index];
                        span
                            id=(format!("amount-btn-{audience_id}-{preset_index}"))
                            class="block w-full rounded-lg border border-gray-200 px-4 py-2 text-left font-medium text-gray-700 transition peer-checked:border-[#4641d9] peer-checked:text-[#4641d9] hover:border-[#4641d9] hover:text-[#4641d9]" {
                            (format_donation_amount(*amount, currency))
                        }
                    }
                }
                label class="block" {
                    input class="peer sr-only" type="radio" name="amount_major" value="custom";
                    span
                        id=(format!("amount-btn-{audience_id}-custom"))
                        class="block w-full rounded-lg border border-gray-200 px-4 py-2 text-left font-medium text-gray-700 transition peer-checked:border-[#4641d9] peer-checked:text-[#4641d9] hover:border-[#4641d9] hover:text-[#4641d9]" {
                        (tr(i18n, ctx, DONATIONS_FORM_AMOUNT_OTHER_DESCRIPTOR))
                    }
                    input
                        id=(format!("custom-amount-{audience_id}"))
                        name="custom_amount_major"
                        type="number"
                        min=(constraints.minimum_amount_major)
                        max=(constraints.maximum_amount_major)
                        step="1"
                        inputmode="numeric"
                        placeholder=(amount_placeholder(i18n, ctx, currency))
                        class="mt-2 hidden w-full rounded-lg border border-gray-300 px-4 py-3 text-gray-950 peer-checked:block focus:border-[#4641d9] focus:outline-none";
                }
            }
            p id=(format!("donation-minimum-{audience_id}")) class="mt-2 text-sm text-gray-500" {
                (minimum_message(i18n, ctx, currency))
            }
        }
    }
}

fn donation_interval_radio(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    interval: &str,
    label: MarketingMessageDescriptor,
    selected: bool,
) -> Markup {
    html! {
        label class="block" {
            input class="peer sr-only" type="radio" name="interval" value=(interval) checked[selected];
            span class="block rounded-lg border-2 border-gray-200 px-3 py-2 text-center font-medium text-gray-700 peer-checked:border-[#4641d9] peer-checked:bg-[#4641d9] peer-checked:text-white" {
                (tr(i18n, ctx, label))
            }
        }
    }
}

fn donation_currency_radio(
    ctx: &RequestContext,
    audience: DonationAudience,
    currency: DonationCurrencyOption,
    selected: bool,
) -> Markup {
    let audience_id = audience.id();
    let currency_code = currency.code.code();
    html! {
        label class="block" {
            input
                class="peer sr-only"
                type="radio"
                name="currency"
                value=(currency_code)
                checked[selected]
                hx-get=(ctx.href(&format!("/_donations/amounts?audience={audience_id}&currency={currency_code}")))
                hx-trigger="change"
                hx-target=(format!("#donation-amount-fieldset-{audience_id}"))
                hx-swap="outerHTML"
                hx-params="none"
                hx-push-url="false";
            span class="block rounded-lg border-2 border-gray-200 px-4 py-2 text-center font-medium text-gray-700 peer-checked:border-[#4641d9] peer-checked:bg-[#4641d9] peer-checked:text-white" {
                (currency.label)
            }
        }
    }
}

fn manage_form(i18n: &MarketingI18n, ctx: &RequestContext, email: &str) -> Markup {
    html! {
        form
            class="mx-auto flex max-w-md gap-2"
            method="post"
            action=(ctx.href("/_donations/request-link"))
            hx-post=(ctx.href("/_donations/request-link"))
            hx-target="#manage-message"
            hx-swap="outerHTML"
            hx-push-url="false" {
            label for="manage-email" class="sr-only" {
                (tr(i18n, ctx, DONATIONS_FORM_EMAIL_DESCRIPTOR))
            }
            input id="manage-email" name="email" type="email" required placeholder=(tr(i18n, ctx, DONATIONS_FORM_EMAIL_PLACEHOLDER_DESCRIPTOR)) value=(email) class="min-w-0 flex-1 rounded-lg border border-gray-200 px-4 py-2 text-gray-950 focus:border-[#4641d9] focus:outline-none";
            button id="send-link-btn" type="submit" class="rounded-lg bg-gray-800 px-6 py-2 font-medium text-white transition hover:bg-gray-700 disabled:cursor-not-allowed disabled:opacity-60" {
                (tr(i18n, ctx, DONATIONS_FORM_SEND_LINK_DESCRIPTOR))
            }
        }
        (manage_empty_message())
    }
}

fn swish_modal(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    _open: bool,
    raw_amount: Option<&str>,
) -> Markup {
    let amount = parse_swish_amount(raw_amount).unwrap_or(50);
    let mobile = matches!(ctx.platform, Platform::Ios | Platform::Android);
    let title = if mobile {
        DONATIONS_SWISH_OPEN_TITLE_DESCRIPTOR
    } else {
        DONATIONS_SWISH_SCAN_TITLE_DESCRIPTOR
    };
    html! {
        div id="swish-modal-backdrop" class="pwa-modal-backdrop" {
            div class="pwa-modal swish-modal" role="dialog" aria-modal="true" aria-labelledby="swish-modal-title" {
                div class="flex h-full flex-col" {
                    div class="flex items-center justify-between p-6 pb-4" {
                        h2 id="swish-modal-title" class="text-xl font-bold text-gray-950" {
                            (tr(i18n, ctx, title))
                        }
                        a href="#_" id="swish-close" class="rounded-lg p-2 text-gray-600 hover:bg-gray-100 hover:text-gray-900" aria-label=(tr(i18n, ctx, NAVIGATION_CLOSE_DESCRIPTOR)) {
                            (icon(Icon::X, "h-5 w-5"))
                        }
                    }
                    div class="flex flex-col items-center gap-4 px-6 pb-6" {
                        (swish_payment_fragment(i18n, ctx, Some(&amount.to_string())))
                    }
                }
            }
        }
    }
}

pub fn swish_payment_fragment(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    raw_amount: Option<&str>,
) -> Markup {
    let amount = parse_swish_amount(raw_amount).unwrap_or(50);
    let mobile = matches!(ctx.platform, Platform::Ios | Platform::Android);
    html! {
        div id="swish-payment-fragment" class="flex w-full flex-col items-center gap-4" aria-live="polite" {
            @if mobile {
                div class="flex h-24 w-24 items-center justify-center rounded-3xl bg-gray-50" {
                    (swish_logo_svg("modal-link"))
                }
                p class="text-center text-sm text-gray-600" {
                    (tr(i18n, ctx, DONATIONS_SWISH_MOBILE_INSTRUCTIONS_DESCRIPTOR))
                }
                (swish_amount_form(i18n, ctx, amount, DONATIONS_SWISH_UPDATE_AMOUNT_DESCRIPTOR))
                a href=(swish_url(amount)) class="flex h-12 w-full items-center justify-center gap-2 rounded-xl bg-[#4641D9] px-6 font-semibold text-white transition-colors hover:bg-[#3832B8]" {
                    (swish_logo_svg("mobile-action"))
                    (tr(i18n, ctx, DONATIONS_SWISH_OPEN_ACTION_DESCRIPTOR))
                }
            } @else {
                p class="text-center text-sm text-gray-600" {
                    (tr(i18n, ctx, DONATIONS_SWISH_INSTRUCTIONS_DESCRIPTOR))
                }
                (swish_qr_fragment(i18n, ctx, Some(&amount.to_string())))
                (swish_amount_form(i18n, ctx, amount, DONATIONS_SWISH_UPDATE_QR_DESCRIPTOR))
            }
        }
    }
}

fn swish_amount_form(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    amount: u32,
    submit_label: MarketingMessageDescriptor,
) -> Markup {
    html! {
        form method="get" action=(ctx.href("/donate#swish-modal-backdrop")) hx-get=(ctx.href("/_swish/payment")) hx-target="#swish-payment-fragment" hx-swap="outerHTML" hx-push-url="false" class="flex flex-wrap items-center justify-center gap-3 text-sm text-gray-950" {
            input type="hidden" name="type" value="individual";
            label for="swish-amount" class="font-medium" {
                (tr(i18n, ctx, DONATIONS_FORM_AMOUNT_DESCRIPTOR)) " (SEK)"
            }
            input
                id="swish-amount"
                name="swish_amount"
                type="number"
                inputmode="numeric"
                min="1"
                max="999999"
                step="1"
                value=(amount)
                class="w-28 rounded-lg border border-gray-300 px-3 py-1.5 text-gray-950 focus:border-[#4641d9] focus:outline-none";
            button type="submit" class="rounded-lg bg-gray-900 px-3 py-1.5 font-semibold text-white hover:bg-gray-700" {
                (tr(i18n, ctx, submit_label))
            }
        }
    }
}

pub fn donation_amount_from_form(
    amount_major: &str,
    custom_amount_major: Option<&str>,
    currency: DonationCurrency,
) -> Option<u32> {
    let raw_amount = if amount_major == "custom" {
        custom_amount_major?
    } else {
        amount_major
    };
    let constraints = currency.constraints();
    parse_bounded_amount(
        raw_amount,
        constraints.minimum_amount_major,
        constraints.maximum_amount_major,
    )
}

pub fn donation_error_message(id: &str, message: &str) -> Markup {
    html! {
        p id=(id) role="alert" class="text-center text-sm text-red-600" { (message) }
    }
}

pub fn donation_empty_message(id: &str) -> Markup {
    html! {
        p id=(id) aria-live="polite" class="hidden text-center text-sm text-red-600" {}
    }
}

fn manage_empty_message() -> Markup {
    html! {
        p id="manage-message" class="mt-2 hidden text-center text-sm" {}
    }
}

pub fn manage_message_fragment(i18n: &MarketingI18n, ctx: &RequestContext, alert: &str) -> Markup {
    let Some((message, tone)) = manage_alert_message(i18n, ctx, Some(alert)) else {
        return manage_empty_message();
    };
    if alert == "sent" {
        html! {
            p id="manage-message" aria-live="polite" class=(format!("mt-2 text-center text-sm {}", tone.text_class)) {
                (message)
            }
        }
    } else {
        html! {
            p id="manage-message" role="alert" class=(format!("mt-2 text-center text-sm {}", tone.text_class)) {
                (message)
            }
        }
    }
}

fn swish_logo_svg(id_suffix: &str) -> PreEscaped<String> {
    let mut svg = SWISH_LOGO_SVG.to_owned();
    for index in 1..=4 {
        let original = format!("swish-grad-{index}");
        let scoped = format!("swish-grad-{index}-{id_suffix}");
        svg = svg.replace(&original, &scoped);
    }
    PreEscaped(svg)
}

#[derive(Clone, Copy)]
struct AlertTone {
    container_class: &'static str,
    text_class: &'static str,
}

fn donation_query_error_message(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    error: Option<&str>,
    currency: Option<&str>,
) -> Option<String> {
    donation_error_text(
        i18n,
        ctx,
        error?,
        currency.and_then(DonationCurrency::from_code),
    )
}

fn donation_error_text(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    error: &str,
    currency: Option<DonationCurrency>,
) -> Option<String> {
    match error {
        "invalid_email" => Some(i18n.text(ctx.locale, DONATIONS_ERRORS_INVALID_EMAIL_DESCRIPTOR)),
        "invalid_amount" => {
            let currency = currency.unwrap_or(DonationCurrency::Usd);
            Some(invalid_amount_message(i18n, ctx, currency))
        }
        "network" => Some(i18n.text(ctx.locale, DONATIONS_ERRORS_NETWORK_DESCRIPTOR)),
        "generic" => Some(i18n.text(ctx.locale, DONATIONS_ERRORS_GENERIC_DESCRIPTOR)),
        _ => None,
    }
}

pub fn donation_checkout_error_fragment(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    audience: DonationAudience,
    error: &str,
    currency: Option<DonationCurrency>,
) -> Markup {
    let message = donation_error_text(i18n, ctx, error, currency)
        .unwrap_or_else(|| i18n.text(ctx.locale, DONATIONS_ERRORS_GENERIC_DESCRIPTOR));
    donation_error_message(&format!("donation-error-{}", audience.id()), &message)
}

fn manage_alert_message(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    alert: Option<&str>,
) -> Option<(String, AlertTone)> {
    let warning = AlertTone {
        container_class: "border-orange-200 bg-orange-50",
        text_class: "text-orange-900",
    };
    let error = AlertTone {
        container_class: "border-red-200 bg-red-50",
        text_class: "text-red-900",
    };
    let success = AlertTone {
        container_class: "border-green-200 bg-green-50",
        text_class: "text-green-900",
    };
    match alert? {
        "active_subscription" => Some((
            i18n.text(
                ctx.locale,
                DONATIONS_ERRORS_ACTIVE_SUBSCRIPTION_EXISTS_DESCRIPTOR,
            ),
            warning,
        )),
        "sent" => Some((
            i18n.text(ctx.locale, DONATIONS_MANAGE_SUCCESS_DESCRIPTOR),
            success,
        )),
        "invalid_email" => Some((
            i18n.text(ctx.locale, DONATIONS_ERRORS_INVALID_EMAIL_DESCRIPTOR),
            error,
        )),
        "network" => Some((
            i18n.text(ctx.locale, DONATIONS_ERRORS_NETWORK_DESCRIPTOR),
            error,
        )),
        "generic" => Some((
            i18n.text(ctx.locale, DONATIONS_ERRORS_GENERIC_DESCRIPTOR),
            error,
        )),
        _ => None,
    }
}

pub fn swish_qr_fragment(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    raw_amount: Option<&str>,
) -> Markup {
    let Some(amount) = parse_swish_amount(raw_amount) else {
        return html! {
            div id="swish-qr-container" class="flex h-72 w-72 items-center justify-center rounded-lg border border-gray-200 bg-white p-3" {
                span id="swish-qr-status" class="text-center text-sm text-red-500" {
                    (tr(i18n, ctx, DONATIONS_SWISH_QR_FAILED_DESCRIPTOR))
                }
            }
        };
    };
    html! {
        div id="swish-qr-container" class="flex h-72 w-72 items-center justify-center rounded-lg border border-gray-200 bg-white p-3" {
            img
                id="swish-qr-image"
                src=(ctx.href(&format!("/_swish/qr?amount={amount}")))
                alt=(tr(i18n, ctx, DONATIONS_SWISH_QR_ALT_DESCRIPTOR))
                class="h-full w-full";
        }
    }
}

fn parse_swish_amount(raw: Option<&str>) -> Option<u32> {
    let raw = raw.unwrap_or("50");
    parse_bounded_amount(raw, 1, 999_999)
}

fn parse_bounded_amount(raw: &str, minimum: u32, maximum: u32) -> Option<u32> {
    if raw.is_empty() || raw.len() > 6 || !raw.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let amount = raw.parse::<u32>().ok()?;
    if (minimum..=maximum).contains(&amount) {
        Some(amount)
    } else {
        None
    }
}

fn donation_currencies(country_code: &str) -> Vec<DonationCurrencyOption> {
    let mut ordered = Vec::new();
    for currency in [
        DonationCurrency::from(get_currency(country_code)),
        DonationCurrency::from(get_base_currency(country_code)),
    ]
    .into_iter()
    .chain(DonationCurrency::ALL)
    {
        if ordered
            .iter()
            .all(|option: &DonationCurrencyOption| option.code != currency)
        {
            ordered.push(DonationCurrencyOption {
                code: currency,
                label: currency.label(),
            });
        }
    }
    ordered
}

fn amount_placeholder(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    currency: DonationCurrency,
) -> String {
    let minimum = format_donation_amount(currency.constraints().minimum_amount_major, currency);
    let maximum = format_donation_amount(currency.constraints().maximum_amount_major, currency);
    i18n.text_with(
        ctx.locale,
        DONATIONS_FORM_AMOUNT_PLACEHOLDER_DESCRIPTOR,
        &[("minimum", &minimum), ("maximum", &maximum)],
    )
}

fn invalid_amount_message(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    currency: DonationCurrency,
) -> String {
    let minimum = format_donation_amount(currency.constraints().minimum_amount_major, currency);
    let maximum = format_donation_amount(currency.constraints().maximum_amount_major, currency);
    i18n.text_with(
        ctx.locale,
        DONATIONS_ERRORS_INVALID_AMOUNT_DESCRIPTOR,
        &[("minimum", &minimum), ("maximum", &maximum)],
    )
}

fn minimum_message(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    currency: DonationCurrency,
) -> String {
    let minimum = format_donation_amount(currency.constraints().minimum_amount_major, currency);
    i18n.text_with(
        ctx.locale,
        DONATIONS_MINIMUM_DONATION_DESCRIPTOR,
        &[("minimum", &minimum)],
    )
}

fn format_donation_amount(amount: u32, currency: DonationCurrency) -> String {
    format_major_amount(amount, currency.constraints().display_currency)
}

fn swish_url(amount: u32) -> String {
    let payload = serde_json::json!({
        "version": 1,
        "payee": {"value": "1232376820", "editable": false},
        "amount": {"value": amount, "editable": true},
        "message": {"value": SWISH_PAYMENT_MESSAGE, "editable": false},
    })
    .to_string();
    format!("swish://payment?data={}", urlencoding::encode(&payload))
}
