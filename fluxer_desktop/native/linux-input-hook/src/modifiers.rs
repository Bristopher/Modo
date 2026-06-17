// SPDX-License-Identifier: AGPL-3.0-or-later

pub const SHIFT_MASK: u32 = 1 << 0;
pub const CONTROL_MASK: u32 = 1 << 2;
pub const MOD1_MASK: u32 = 1 << 3;
pub const MOD4_MASK: u32 = 1 << 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

pub fn from_state(state: u32) -> Modifiers {
    Modifiers {
        ctrl: (state & CONTROL_MASK) != 0,
        alt: (state & MOD1_MASK) != 0,
        shift: (state & SHIFT_MASK) != 0,
        meta: (state & MOD4_MASK) != 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_bits_all_false() {
        let m = from_state(0);
        assert!(!m.ctrl && !m.alt && !m.shift && !m.meta);
    }

    #[test]
    fn shift_alone() {
        let m = from_state(SHIFT_MASK);
        assert!(m.shift);
        assert!(!m.ctrl && !m.alt && !m.meta);
    }

    #[test]
    fn all_four_modifiers_together() {
        let m = from_state(SHIFT_MASK | CONTROL_MASK | MOD1_MASK | MOD4_MASK);
        assert!(m.ctrl && m.alt && m.shift && m.meta);
    }

    #[test]
    fn lock_and_numlock_ignored() {
        let m = from_state((1 << 1) | (1 << 4));
        assert!(!m.ctrl && !m.alt && !m.shift && !m.meta);
    }

    #[test]
    fn mod1_is_alt_mod4_is_meta() {
        let a = from_state(MOD1_MASK);
        assert!(a.alt && !a.meta);
        let b = from_state(MOD4_MASK);
        assert!(b.meta && !b.alt);
    }
}
