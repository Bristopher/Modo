// SPDX-License-Identifier: AGPL-3.0-or-later

pub fn strip_animation_prefix(hash: &str) -> &str {
    hash.strip_prefix("a_").unwrap_or(hash)
}

pub fn has_animation_prefix(hash: &str) -> bool {
    hash.starts_with("a_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_animation_prefix_removes_only_virtual_animated_prefix() {
        assert_eq!("abc123", strip_animation_prefix("a_abc123"));
        assert_eq!("abc123", strip_animation_prefix("abc123"));
    }
}
