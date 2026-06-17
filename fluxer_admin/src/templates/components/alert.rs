// SPDX-License-Identifier: AGPL-3.0-or-later

use maud::{Markup, html};

#[derive(Clone, Copy)]
pub enum AlertVariant {
    Info,
    Warning,
    Error,
    Success,
}

pub fn alert(variant: AlertVariant, title: Option<&str>, content: Markup) -> Markup {
    let variant_classes = match variant {
        AlertVariant::Info => "bg-blue-50 border-blue-200 text-blue-700",
        AlertVariant::Warning => "bg-neutral-50 border-neutral-200 text-neutral-700",
        AlertVariant::Error => "bg-red-50 border-red-200 text-red-700",
        AlertVariant::Success => "bg-green-50 border-green-200 text-green-700",
    };
    html! {
        div class={"rounded-lg border p-4 " (variant_classes)} {
            @if let Some(title_text) = title {
                div class="mb-2 font-bold" { (title_text) }
            }
            div { (content) }
        }
    }
}

pub fn alert_info(content: Markup) -> Markup {
    alert(AlertVariant::Info, None, content)
}

pub fn alert_error(title: &str, content: Markup) -> Markup {
    alert(AlertVariant::Error, Some(title), content)
}

pub fn alert_success(title: &str, content: Markup) -> Markup {
    alert(AlertVariant::Success, Some(title), content)
}

pub fn alert_warning(title: &str, content: Markup) -> Markup {
    alert(AlertVariant::Warning, Some(title), content)
}
