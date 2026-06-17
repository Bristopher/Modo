// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SysctlFailure {
    pub errno: i32,
}

pub const ENOENT: i32 = 2;

pub fn errno_message(errno: i32) -> String {
    format!("sysctlbyname failed (errno {errno})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errno_message_formats_errno_into_expected_wording() {
        assert_eq!("sysctlbyname failed (errno 22)", errno_message(22));
        assert_eq!("sysctlbyname failed (errno 2)", errno_message(ENOENT));
    }
}
