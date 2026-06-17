// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    api::types::Archive, config::AdminConfig,
    templates::components::archive::archives_tab as shared_archives_tab,
};
use maud::Markup;

pub fn archives_tab(
    config: &AdminConfig,
    user_id: &str,
    archives: &[Archive],
    csrf_token: &str,
) -> Markup {
    let base = &config.base_path;
    let action = format!("{base}/users/{user_id}?tab=archives&action=trigger_archive");
    shared_archives_tab(
        base,
        "User Archives",
        &action,
        "No archives yet for this user.",
        archives,
        csrf_token,
    )
}
