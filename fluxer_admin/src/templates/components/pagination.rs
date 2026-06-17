// SPDX-License-Identifier: AGPL-3.0-or-later

use maud::{Markup, PreEscaped, html};

pub fn pagination(base_url: &str, current_page: u32, has_more: bool, extra_params: &str) -> Markup {
    let sep = if base_url.contains('?') { "&" } else { "?" };
    let has_previous = current_page > 1;
    html! {
        div class="mt-4 flex items-center justify-between" {
            @if has_previous {
                p class="text-sm font-normal text-neutral-500" {
                    a href={
                        (base_url) (sep) "page=" (current_page - 1) (extra_params)
                    } class="text-neutral-900 underline decoration-neutral-300 hover:text-neutral-600 hover:decoration-neutral-500 hover:text-neutral-900" {
                        (PreEscaped("&larr; Previous"))
                    }
                }
            } @else {
                span {}
            }
            @if has_more {
                p class="text-sm font-normal text-neutral-500" {
                    a href={
                        (base_url) (sep) "page=" (current_page + 1) (extra_params)
                    } class="text-neutral-900 underline decoration-neutral-300 hover:text-neutral-600 hover:decoration-neutral-500 hover:text-neutral-900" {
                        (PreEscaped("Next &rarr;"))
                    }
                }
            } @else {
                span {}
            }
        }
    }
}
