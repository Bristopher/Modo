// SPDX-License-Identifier: AGPL-3.0-or-later

use maud::{Markup, html};

pub fn error_alert(error: &str) -> Markup {
    html! {
        div class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-800" role="alert" {
            (error)
        }
    }
}

pub fn error_card(title: &str, message: &str) -> Markup {
    html! {
        div class="rounded-lg border border-neutral-200 bg-white p-4 shadow-sm sm:p-6" {
            div class="flex flex-col gap-4" {
                h3 class="text-base font-semibold text-gray-900" { (title) }
                p class="text-sm text-neutral-500" { (message) }
            }
        }
    }
}
