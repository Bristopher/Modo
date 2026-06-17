// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    api::types::{GuildAssetItem, GuildInfo},
    config::AdminConfig,
    templates::components::{
        badge::{BadgeVariant, badge},
        form::{csrf_input, danger_button},
        page_container::card_with_header,
    },
};
use maud::{Markup, html};

pub fn emojis_tab(
    config: &AdminConfig,
    guild: &GuildInfo,
    emojis: &[GuildAssetItem],
    csrf_token: &str,
) -> Markup {
    let base = &config.base_path;
    html! {
        (card_with_header(
            &format!("Emojis ({})", emojis.len()),
            html! {
                @if emojis.is_empty() {
                    p class="text-sm text-neutral-500" {
                        "No custom emojis found for this guild."
                    }
                } @else {
                    div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3" {
                        @for emoji in emojis {
                            (emoji_card(base, &guild.id, emoji, csrf_token))
                        }
                    }
                }
            },
        ))
    }
}

fn emoji_card(base: &str, guild_id: &str, emoji: &GuildAssetItem, csrf_token: &str) -> Markup {
    html! {
        div class="overflow-hidden rounded-lg border border-neutral-200 bg-white shadow-sm" {
            div class="flex flex-col" {
                div class="flex h-32 items-center justify-center bg-neutral-100 p-6" {
                    img src=(emoji.media_url) alt=(emoji.name)
                        class="max-h-full max-w-full object-contain"
                        loading="lazy";
                }
                div class="flex-1 space-y-3 px-4 py-3" {
                    div class="flex items-center justify-between" {
                        span class="text-sm font-semibold text-neutral-900" {
                            (emoji.name)
                        }
                        @if emoji.animated {
                            (badge("Animated", BadgeVariant::Default))
                        }
                    }
                    p class="break-words text-xs text-neutral-500" {
                        "ID: " (emoji.id)
                    }
                    a href={(base) "/users/" (emoji.creator_id)}
                        class="text-xs text-blue-600 hover:underline" {
                        "Uploader: " (emoji.creator_id)
                    }
                    form method="post"
                        action={(base) "/guilds/" (guild_id) "?tab=emojis&action=delete_emoji"} {
                        (csrf_input(csrf_token))
                        input type="hidden" name="emoji_id" value=(emoji.id);
                        (danger_button("Delete Emoji"))
                    }
                }
            }
        }
    }
}
