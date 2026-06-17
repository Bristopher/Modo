// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{config::AdminConfig, middleware::auth::AuthContext, templates::layout::admin_layout};
use maud::{Markup, html};

pub fn strange_place_page(config: &AdminConfig, auth: &AuthContext) -> Markup {
    let content = html! {
        div class="mx-auto max-w-2xl" {
            div class="rounded-lg border border-neutral-200 bg-white p-8 shadow-sm" {
                div class="space-y-4 text-center" {
                    div class="mx-auto flex h-16 w-16 items-center justify-center \
                               rounded-full bg-neutral-100" {
                        span class="text-lg text-neutral-400" { "?" }
                    }
                    h2 class="text-lg font-semibold text-neutral-900" {
                        "You've reached a strange place"
                    }
                    p class="text-sm text-neutral-600" {
                        "You don't have access to any admin features. "
                        "Contact an administrator to request access."
                    }
                }
            }
        }
    };
    admin_layout(config, auth, "Strange Place", "", None, content)
}
