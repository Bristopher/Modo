// SPDX-License-Identifier: AGPL-3.0-or-later

pub type Window = u64;

pub fn parse_window_token(token: &str) -> Option<Window> {
    if token.is_empty() {
        return None;
    }
    let parsed = if let Some(rest) = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
    {
        u64::from_str_radix(rest, 16).ok()?
    } else {
        token.parse::<u64>().ok()?
    };
    (parsed != 0).then_some(parsed)
}

pub fn pid_from_long(value: i64) -> Option<u32> {
    u32::try_from(value).ok().filter(|pid| *pid > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_window_token_accepts_decimal_and_hexadecimal_xids() {
        assert_eq!(Some(123), parse_window_token("123"));
        assert_eq!(Some(0x3a00007), parse_window_token("0x3a00007"));
        assert_eq!(Some(0x3a00007), parse_window_token("0X3a00007"));
    }

    #[test]
    fn parse_window_token_rejects_invalid_or_zero_xids() {
        assert_eq!(None, parse_window_token(""));
        assert_eq!(None, parse_window_token("0"));
        assert_eq!(None, parse_window_token("0x"));
        assert_eq!(None, parse_window_token("0xG"));
        assert_eq!(None, parse_window_token("../123"));
        assert_eq!(None, parse_window_token("123abc"));
    }

    #[test]
    fn pid_from_long_validates_positive_uint32_process_ids() {
        assert_eq!(Some(1), pid_from_long(1));
        assert_eq!(Some(42_424), pid_from_long(42_424));
        assert_eq!(None, pid_from_long(0));
        assert_eq!(None, pid_from_long(-1));
        assert_eq!(None, pid_from_long(u32::MAX as i64 + 1));
    }
}
