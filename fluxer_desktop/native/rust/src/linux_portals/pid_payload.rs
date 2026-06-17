// SPDX-License-Identifier: AGPL-3.0-or-later

pub fn parse_shell_eval_pid_payload(payload: &str) -> Option<u32> {
    let bytes = payload.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if let Some(pid) = payload[start..i]
            .parse::<u64>()
            .ok()
            .and_then(|n| u32::try_from(n).ok())
            .filter(|pid| *pid != 0)
        {
            return Some(pid);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_plain_integer() {
        assert_eq!(Some(1234), parse_shell_eval_pid_payload("1234"));
    }

    #[test]
    fn extracts_integer_from_json_array_form() {
        assert_eq!(Some(4242), parse_shell_eval_pid_payload("[4242]"));
    }

    #[test]
    fn skips_zero_and_returns_next_positive() {
        assert_eq!(Some(17), parse_shell_eval_pid_payload("[0, 17]"));
    }

    #[test]
    fn returns_null_when_no_digits_present() {
        assert_eq!(None, parse_shell_eval_pid_payload("undefined"));
        assert_eq!(None, parse_shell_eval_pid_payload(""));
    }

    #[test]
    fn rejects_values_that_overflow_u32() {
        assert_eq!(None, parse_shell_eval_pid_payload("4294967296"));
    }
}
