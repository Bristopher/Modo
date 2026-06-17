// SPDX-License-Identifier: AGPL-3.0-or-later

pub const SHIFT_MASK: u64 = 1 << 17;
pub const CONTROL_MASK: u64 = 1 << 18;
pub const ALTERNATE_MASK: u64 = 1 << 19;
pub const COMMAND_MASK: u64 = 1 << 20;

const LEFT_SHIFT_KEYCODE: u16 = 0x38;
const RIGHT_SHIFT_KEYCODE: u16 = 0x3c;
const LEFT_CONTROL_KEYCODE: u16 = 0x3b;
const RIGHT_CONTROL_KEYCODE: u16 = 0x3e;
const LEFT_OPTION_KEYCODE: u16 = 0x3a;
const RIGHT_OPTION_KEYCODE: u16 = 0x3d;
const LEFT_COMMAND_KEYCODE: u16 = 0x37;
const RIGHT_COMMAND_KEYCODE: u16 = 0x36;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

pub fn from_flags(flags: u64) -> Modifiers {
    Modifiers {
        ctrl: (flags & CONTROL_MASK) != 0,
        alt: (flags & ALTERNATE_MASK) != 0,
        shift: (flags & SHIFT_MASK) != 0,
        meta: (flags & COMMAND_MASK) != 0,
    }
}

pub fn modifier_key_down_from_flags(keycode: u16, flags: u64) -> Option<bool> {
    match keycode {
        LEFT_SHIFT_KEYCODE | RIGHT_SHIFT_KEYCODE => Some((flags & SHIFT_MASK) != 0),
        LEFT_CONTROL_KEYCODE | RIGHT_CONTROL_KEYCODE => Some((flags & CONTROL_MASK) != 0),
        LEFT_OPTION_KEYCODE | RIGHT_OPTION_KEYCODE => Some((flags & ALTERNATE_MASK) != 0),
        LEFT_COMMAND_KEYCODE | RIGHT_COMMAND_KEYCODE => Some((flags & COMMAND_MASK) != 0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_flags_all_false() {
        let m = from_flags(0);
        assert!(!m.ctrl && !m.alt && !m.shift && !m.meta);
    }

    #[test]
    fn command_alone_sets_only_meta() {
        let m = from_flags(COMMAND_MASK);
        assert!(m.meta);
        assert!(!m.ctrl && !m.alt && !m.shift);
    }

    #[test]
    fn option_alone_sets_only_alt() {
        let m = from_flags(ALTERNATE_MASK);
        assert!(m.alt);
        assert!(!m.ctrl && !m.meta && !m.shift);
    }

    #[test]
    fn cmd_shift_combo() {
        let m = from_flags(COMMAND_MASK | SHIFT_MASK);
        assert!(m.meta && m.shift);
        assert!(!m.ctrl && !m.alt);
    }

    #[test]
    fn all_four_modifiers_together() {
        let m = from_flags(SHIFT_MASK | CONTROL_MASK | ALTERNATE_MASK | COMMAND_MASK);
        assert!(m.ctrl && m.alt && m.shift && m.meta);
    }

    #[test]
    fn unrelated_high_bits_ignored() {
        let m = from_flags(0xff << 32);
        assert!(!m.ctrl && !m.alt && !m.shift && !m.meta);
    }

    #[test]
    fn modifier_key_down_uses_matching_aggregate_flag() {
        assert_eq!(
            modifier_key_down_from_flags(LEFT_SHIFT_KEYCODE, SHIFT_MASK),
            Some(true)
        );
        assert_eq!(
            modifier_key_down_from_flags(RIGHT_SHIFT_KEYCODE, 0),
            Some(false)
        );
        assert_eq!(
            modifier_key_down_from_flags(LEFT_COMMAND_KEYCODE, COMMAND_MASK),
            Some(true)
        );
        assert_eq!(modifier_key_down_from_flags(0x39, 0), None);
    }
}
