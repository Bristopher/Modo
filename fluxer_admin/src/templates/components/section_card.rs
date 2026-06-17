// SPDX-License-Identifier: AGPL-3.0-or-later

use maud::{Markup, html};

pub fn section_card(
    title: Option<&str>,
    description: Option<&str>,
    header_actions: Option<Markup>,
    content: Markup,
) -> Markup {
    let has_header = title.is_some() || header_actions.is_some();
    let header_row_class = if header_actions.is_some() {
        "flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between sm:gap-4"
    } else {
        "flex flex-col gap-2"
    };
    html! {
        div class="rounded-lg bg-white transition-all border border-neutral-200 p-6" {
            div class="flex flex-col gap-4" {
                @if has_header {
                    div class=(header_row_class) {
                        @if let Some(title_text) = title {
                            div class="flex min-w-0 flex-col gap-1" {
                                h2 class="text-gray-900 tracking-tight text-base" {
                                    (title_text)
                                }
                                @if let Some(desc) = description {
                                    p class="text-sm font-normal text-neutral-500" {
                                        (desc)
                                    }
                                }
                            }
                        }
                        @if let Some(actions) = header_actions {
                            div class="flex flex-shrink-0 flex-wrap items-center gap-1 sm:gap-2" {
                                (actions)
                            }
                        }
                    }
                }
                (content)
            }
        }
    }
}

pub fn section_card_simple(title: &str, content: Markup) -> Markup {
    section_card(Some(title), None, None, content)
}

pub fn section_card_with_description(title: &str, description: &str, content: Markup) -> Markup {
    section_card(Some(title), Some(description), None, content)
}
