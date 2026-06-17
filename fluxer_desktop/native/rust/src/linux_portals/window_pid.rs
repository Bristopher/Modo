// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    NoSpaceLeft,
}

pub fn is_safe_kwin_path_segment(token: &str) -> bool {
    !token.is_empty()
        && token
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == b'_')
}

pub fn build_kwin_window_path(token: &str, capacity: usize) -> Result<String, PathError> {
    let path = format!("/org/kde/KWin/Window/{token}");
    if path.len() + 1 > capacity {
        Err(PathError::NoSpaceLeft)
    } else {
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_kwin_path_segment_accepts_plain_alnum_underscore() {
        assert!(is_safe_kwin_path_segment("abc"));
        assert!(is_safe_kwin_path_segment("123"));
        assert!(is_safe_kwin_path_segment("aZ_9"));
    }

    #[test]
    fn safe_kwin_path_segment_rejects_empty() {
        assert!(!is_safe_kwin_path_segment(""));
    }

    #[test]
    fn safe_kwin_path_segment_rejects_path_traversal_and_shell_metacharacters() {
        assert!(!is_safe_kwin_path_segment("../etc"));
        assert!(!is_safe_kwin_path_segment("a/b"));
        assert!(!is_safe_kwin_path_segment("$(rm -rf)"));
        assert!(!is_safe_kwin_path_segment("a;b"));
        assert!(!is_safe_kwin_path_segment("a-b"));
        assert!(!is_safe_kwin_path_segment("a.b"));
    }

    #[test]
    fn build_kwin_window_path_shapes_path_correctly() {
        assert_eq!(
            "/org/kde/KWin/Window/abc123",
            build_kwin_window_path("abc123", 128).unwrap()
        );
    }

    #[test]
    fn build_kwin_window_path_rejects_too_small_buffer() {
        assert_eq!(
            Err(PathError::NoSpaceLeft),
            build_kwin_window_path("abc", 8)
        );
    }
}
