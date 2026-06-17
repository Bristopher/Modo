// SPDX-License-Identifier: AGPL-3.0-or-later

pub fn has_flag(flags: u64, flag: u64) -> bool {
    flags & flag != 0
}

pub fn set_flag(flags: u64, flag: u64) -> u64 {
    flags | flag
}

pub fn clear_flag(flags: u64, flag: u64) -> u64 {
    flags & !flag
}

pub fn toggle_flag(flags: u64, flag: u64) -> u64 {
    flags ^ flag
}

pub fn list_flags(flags: u64) -> Vec<u32> {
    let mut result = Vec::new();
    for bit in 0..64 {
        if flags & (1u64 << bit) != 0 {
            result.push(bit);
        }
    }
    result
}

pub fn format_discriminator(discriminator: &str) -> String {
    let num: u16 = discriminator.parse().unwrap_or(0);
    format!("{num:04}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_discriminator_pads_short() {
        assert_eq!(format_discriminator("1"), "0001");
        assert_eq!(format_discriminator("42"), "0042");
        assert_eq!(format_discriminator("0"), "0000");
    }

    #[test]
    fn format_discriminator_four_digits() {
        assert_eq!(format_discriminator("1234"), "1234");
        assert_eq!(format_discriminator("9999"), "9999");
    }

    #[test]
    fn format_discriminator_invalid_input() {
        assert_eq!(format_discriminator(""), "0000");
        assert_eq!(format_discriminator("abc"), "0000");
    }

    #[test]
    fn has_flag_checks_bit() {
        assert!(has_flag(0b1010, 0b0010));
        assert!(!has_flag(0b1010, 0b0001));
        assert!(has_flag(0b1111, 0b0100));
    }

    #[test]
    fn set_flag_sets_bit() {
        assert_eq!(set_flag(0b1000, 0b0010), 0b1010);
        assert_eq!(set_flag(0b1010, 0b0010), 0b1010);
    }

    #[test]
    fn clear_flag_clears_bit() {
        assert_eq!(clear_flag(0b1010, 0b0010), 0b1000);
        assert_eq!(clear_flag(0b1000, 0b0010), 0b1000);
    }

    #[test]
    fn toggle_flag_toggles_bit() {
        assert_eq!(toggle_flag(0b1000, 0b0010), 0b1010);
        assert_eq!(toggle_flag(0b1010, 0b0010), 0b1000);
    }

    #[test]
    fn list_flags_returns_set_positions() {
        assert_eq!(list_flags(0b1010), vec![1, 3]);
        assert_eq!(list_flags(0b0001), vec![0]);
        assert!(list_flags(0).is_empty());
    }
}
