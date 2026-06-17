// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone, Copy)]
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
    fn xkb_lookup_for_base_pins_group_level_to_zero() {
        let lookup = xkb_lookup_for_base();
        assert_eq!(lookup.group, 0);
        assert_eq!(lookup.level, 0);
    }
}
