// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XkbLookup {
    pub group: u32,
    pub level: u32,
}

pub fn xkb_lookup_for_base() -> XkbLookup {
    XkbLookup { group: 0, level: 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xkb_lookup_for_base_pins_group_and_level_to_unshifted_base_keysym() {
        let lookup = xkb_lookup_for_base();
        assert_eq!(0, lookup.group);
        assert_eq!(0, lookup.level);
    }
}
