// SPDX-License-Identifier: AGPL-3.0-or-later

pub fn format_user_display(
    global_name: Option<&str>,
    username: Option<&str>,
    discriminator: Option<&str>,
) -> String {
    match (global_name, username, discriminator) {
        (Some(gn), Some(un), Some("0")) => format!("{gn} (@{un})"),
        (Some(gn), _, _) => gn.to_owned(),
        (None, Some(un), Some(d)) if d != "0" => format!("{un}#{d}"),
        (None, Some(un), _) => format!("@{un}"),
        _ => "Unknown".to_owned(),
    }
}
