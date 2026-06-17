// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    api::types::{Archive, GuildInfo},
    config::AdminConfig,
    templates::components::archive::archives_tab as shared_archives_tab,
};
use maud::Markup;

pub fn archives_tab(
    config: &AdminConfig,
    guild: &GuildInfo,
    archives: &[Archive],
    csrf_token: &str,
) -> Markup {
    let base = &config.base_path;
    let action = format!(
        "{base}/guilds/{}?tab=archives&action=trigger_archive",
        guild.id
    );
    shared_archives_tab(
        base,
        "Guild Archives",
        &action,
        "No archives yet for this guild.",
        archives,
        csrf_token,
    )
}
