// SPDX-License-Identifier: AGPL-3.0-or-later

pub const PROCESS_LOOPBACK_MIN_BUILD: u32 = 20_348;

pub fn supports_process_loopback(major: u32, minor: u32, build: u32) -> bool {
    if major > 10 {
        return true;
    }
    major == 10 && minor == 0 && build >= PROCESS_LOOPBACK_MIN_BUILD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_loopback_follows_microsoft_documented_windows_build_floor() {
        assert!(!supports_process_loopback(10, 0, 19_045));
        assert!(!supports_process_loopback(
            10,
            0,
            PROCESS_LOOPBACK_MIN_BUILD - 1
        ));
        assert!(supports_process_loopback(10, 0, PROCESS_LOOPBACK_MIN_BUILD));
        assert!(supports_process_loopback(10, 0, 22_000));
        assert!(supports_process_loopback(11, 0, 0));
    }
}
