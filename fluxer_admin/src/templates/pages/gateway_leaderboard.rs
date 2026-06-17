// SPDX-License-Identifier: AGPL-3.0-or-later

use super::gateway_nodes::{format_memory_from_bytes, format_node_id};
use crate::{
    config::AdminConfig,
    templates::components::{
        form::select_input,
        media::{guild_icon_url, initials},
    },
};
use maud::{Markup, html};

pub(crate) fn guild_leaderboard(
    config: &AdminConfig,
    base: &str,
    guilds: &[serde_json::Value],
    limit: u32,
) -> Markup {
    let limit_options: &[(&str, &str)] = &[
        ("100", "Top 100"),
        ("200", "Top 200"),
        ("300", "Top 300"),
        ("500", "Top 500"),
        ("1000", "Top 1000"),
    ];
    let limit_str = limit.to_string();
    html! {
        div class="rounded-lg border border-neutral-200 bg-white shadow-sm" {
            div class="flex flex-col gap-4 border-neutral-200 border-b p-6 \
                       lg:flex-row lg:items-end lg:justify-between" {
                div class="flex flex-col gap-2" {
                    h2 class="text-gray-900 tracking-tight text-base" {
                        "Guild Memory Leaderboard (Top " (limit) ")"
                    }
                    p class="text-sm text-neutral-500" {
                        "Guilds ranked by memory usage"
                    }
                }
                form method="get" action={(base) "/gateway"}
                    class="flex flex-wrap items-end gap-3" {
                    div class="min-w-[14rem]" {
                        (select_input("leaderboard_limit", "Guild count",
                            limit_options, &limit_str))
                    }
                    button type="submit"
                        class="inline-flex items-center justify-center gap-2 font-medium \
                               rounded-lg bg-neutral-50 text-neutral-700 border \
                               border-neutral-300 px-3 py-1.5 text-sm" {
                        "Apply"
                    }
                }
            }
            @if guilds.is_empty() {
                div class="p-6 text-center text-neutral-600" { "No guilds in memory" }
            } @else {
                div class="table-scroll overflow-x-auto" {
                    table class="w-full" {
                        thead class="border-neutral-200 border-b bg-neutral-50" {
                            tr {
                                th class="px-6 py-3 text-left text-neutral-600 text-xs uppercase" {
                                    "Rank"
                                }
                                th class="px-6 py-3 text-left text-neutral-600 text-xs uppercase" {
                                    "Guild"
                                }
                                th class="px-6 py-3 text-left text-neutral-600 text-xs uppercase" {
                                    "Node"
                                }
                                th class="px-6 py-3 text-right text-neutral-600 text-xs uppercase" {
                                    "RAM Usage"
                                }
                                th class="px-6 py-3 text-right text-neutral-600 text-xs uppercase" {
                                    "Members"
                                }
                                th class="px-6 py-3 text-right text-neutral-600 text-xs uppercase" {
                                    "Sessions"
                                }
                                th class="px-6 py-3 text-right text-neutral-600 text-xs uppercase" {
                                    "Presences"
                                }
                            }
                        }
                        tbody class="divide-y divide-neutral-200" {
                            @for (i, guild) in guilds.iter().enumerate() {
                                (guild_row(config, base, guild, i + 1))
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn guild_row(
    config: &AdminConfig,
    base: &str,
    guild: &serde_json::Value,
    rank: usize,
) -> Markup {
    let guild_id = guild.get("guild_id").and_then(|v| v.as_str());
    let guild_name = guild
        .get("guild_name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");
    let memory = guild.get("memory").and_then(|v| v.as_str()).unwrap_or("0");
    let members = guild
        .get("member_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let sessions = guild
        .get("session_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let presences = guild
        .get("presence_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let node_id = guild.get("node_id").and_then(|v| v.as_str()).unwrap_or("");
    let guild_icon = guild.get("guild_icon").and_then(|v| v.as_str());
    let node_label = if node_id.is_empty() {
        "-".to_owned()
    } else {
        format_node_id(node_id, 0)
    };
    let icon = guild_id
        .and_then(|gid| guild_icon_url(config, gid, guild_icon, 160, true))
        .map(|url| {
            html! {
                img src=(url) alt="" class="h-10 w-10 rounded-full object-cover";
            }
        })
        .unwrap_or_else(|| {
            html! {
                div class="flex h-10 w-10 items-center justify-center rounded-full \
                           bg-neutral-200 font-medium text-neutral-600 text-sm" {
                    (initials(guild_name))
                }
            }
        });

    html! {
        tr class="transition-colors hover:bg-neutral-50" {
            td class="whitespace-nowrap px-6 py-4 font-medium text-sm" {
                "#" (rank)
            }
            td class="whitespace-nowrap px-6 py-4" {
                @if let Some(gid) = guild_id {
                    a href={(base) "/guilds/" (gid)} class="flex items-center gap-2" {
                        (icon)
                        div {
                            div class="font-medium text-neutral-900 text-sm" { (guild_name) }
                            div class="text-neutral-500 text-xs" { (gid) }
                        }
                    }
                } @else {
                    div class="flex items-center gap-2" {
                        (icon)
                        span class="text-neutral-600 text-sm" { (guild_name) }
                    }
                }
            }
            td class="whitespace-nowrap px-6 py-4 text-sm" { (node_label) }
            td class="whitespace-nowrap px-6 py-4 text-right font-medium text-sm" {
                (format_memory_from_bytes(memory))
            }
            td class="whitespace-nowrap px-6 py-4 text-right text-sm" { (members) }
            td class="whitespace-nowrap px-6 py-4 text-right text-sm" { (sessions) }
            td class="whitespace-nowrap px-6 py-4 text-right text-sm" { (presences) }
        }
    }
}
