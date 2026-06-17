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

pub fn stickers_tab(
    config: &AdminConfig,
    guild: &GuildInfo,
    stickers: &[GuildAssetItem],
    csrf_token: &str,
) -> Markup {
    let base = &config.base_path;
    html! {
        (card_with_header(
            &format!("Stickers ({})", stickers.len()),
            html! {
                @if stickers.is_empty() {
                    p class="text-sm text-neutral-500" {
                        "No stickers found for this guild."
                    }
                } @else {
                    div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3" {
                        @for sticker in stickers {
                            (sticker_card(base, &guild.id, sticker, csrf_token))
                        }
                    }
                }
            },
        ))
    }
}

fn sticker_card(base: &str, guild_id: &str, sticker: &GuildAssetItem, csrf_token: &str) -> Markup {
    html! {
        div class="overflow-hidden rounded-lg border border-neutral-200 bg-white shadow-sm" {
            div class="flex flex-col" {
                div class="flex h-32 items-center justify-center bg-neutral-100 p-6" {
                    img src=(sticker.media_url) alt=(sticker.name)
                        class="max-h-full max-w-full object-contain"
                        loading="lazy";
                }
                div class="flex-1 space-y-1 px-4 py-3" {
                    div class="flex items-center justify-between gap-2" {
                        span class="text-sm font-semibold text-neutral-900" {
                            (sticker.name)
                        }
                        (badge(
                            if sticker.animated { "Animated" } else { "Static" },
                            BadgeVariant::Default,
                        ))
                    }
                    p class="break-words text-xs text-neutral-500" {
                        "ID: " (sticker.id)
                    }
                    a href={(base) "/users/" (sticker.creator_id)}
                        class="text-xs text-blue-600 hover:underline" {
                        "Uploader: " (sticker.creator_id)
                    }
                    form method="post"
                        action={(base) "/guilds/" (guild_id) "?tab=stickers&action=delete_sticker"}
                        class="mt-4" {
                        (csrf_input(csrf_token))
                        input type="hidden" name="sticker_id" value=(sticker.id);
                        (danger_button("Delete Sticker"))
                    }
                }
            }
        }
    }
}
