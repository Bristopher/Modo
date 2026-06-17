// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::templates::components::form::{csrf_input, form_actions};
use maud::{Markup, html};

pub(crate) enum BlocklistActionVariant {
    Primary,
    Danger,
}

pub(crate) fn blocklist_text_field(
    name: &str,
    label: &str,
    placeholder: &str,
    required: bool,
) -> Markup {
    html! {
        div class="space-y-1" {
            label for=(name) class="block text-sm font-medium text-neutral-700" {
                (label)
            }
            input type="text" id=(name) name=(name) placeholder=(placeholder)
                required[required]
                class="block w-full rounded-md border border-neutral-300 px-3 py-2 \
                       text-sm shadow-sm focus:border-brand-primary focus:outline-none \
                       focus:ring-1 focus:ring-brand-primary";
        }
    }
}

pub(crate) fn blocklist_action_card(
    title: &str,
    action_url: &str,
    csrf_token: &str,
    fields: Markup,
    button_label: &str,
    variant: BlocklistActionVariant,
) -> Markup {
    let button_class = match variant {
        BlocklistActionVariant::Primary => {
            "inline-flex items-center rounded-md bg-brand-primary px-4 py-2 text-sm \
             font-medium text-white shadow-sm hover:bg-brand-primary-dark"
        }
        BlocklistActionVariant::Danger => {
            "inline-flex items-center rounded-md bg-red-600 px-4 py-2 text-sm \
             font-medium text-white shadow-sm hover:bg-red-700"
        }
    };
    html! {
        div class="rounded-lg border border-neutral-200 bg-white p-4 shadow-sm sm:p-6" {
            h3 class="text-base font-medium text-neutral-900 mb-4" {
                (title)
            }
            form method="post" action=(action_url)
                hx-post=(action_url) hx-target="#flash-container" hx-swap="innerHTML" {
                (csrf_input(csrf_token))
                div class="space-y-4" {
                    (fields)
                    (form_actions(html! {
                        button type="submit" class=(button_class) {
                            (button_label)
                        }
                    }))
                }
            }
        }
    }
}
