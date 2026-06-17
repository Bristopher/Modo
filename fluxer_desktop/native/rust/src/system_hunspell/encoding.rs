// SPDX-License-Identifier: AGPL-3.0-or-later

pub fn is_utf8_encoding(encoding: &str) -> bool {
    let bytes = encoding.as_bytes();
    if bytes.is_empty() || bytes.len() > 16 {
        return false;
    }
    bytes.eq_ignore_ascii_case(b"utf-8") || bytes.eq_ignore_ascii_case(b"utf8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_utf8_encoding_matches_common_spellings() {
        assert!(is_utf8_encoding("UTF-8"));
        assert!(is_utf8_encoding("utf-8"));
        assert!(is_utf8_encoding("UTF8"));
        assert!(!is_utf8_encoding("ISO-8859-1"));
        assert!(!is_utf8_encoding(""));
    }

    #[test]
    fn is_utf8_encoding_rejects_long_or_decorated_values() {
        assert!(!is_utf8_encoding("utf-8\0"));
        assert!(!is_utf8_encoding(" utf-8"));
        assert!(!is_utf8_encoding("utf-8-with-extra"));
    }
}
