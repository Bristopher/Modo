// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::input::keymap::KeyMapU16;

pub const KEY_MAP: &[KeyMapU16] = &[
    KeyMapU16 {
        code: 1,
        name: "Escape",
    },
    KeyMapU16 { code: 2, name: "1" },
    KeyMapU16 { code: 3, name: "2" },
    KeyMapU16 { code: 4, name: "3" },
    KeyMapU16 { code: 5, name: "4" },
    KeyMapU16 { code: 6, name: "5" },
    KeyMapU16 { code: 7, name: "6" },
    KeyMapU16 { code: 8, name: "7" },
    KeyMapU16 { code: 9, name: "8" },
    KeyMapU16 {
        code: 10,
        name: "9",
    },
    KeyMapU16 {
        code: 11,
        name: "0",
    },
    KeyMapU16 {
        code: 12,
        name: "Minus",
    },
    KeyMapU16 {
        code: 13,
        name: "Equal",
    },
    KeyMapU16 {
        code: 14,
        name: "Backspace",
    },
    KeyMapU16 {
        code: 15,
        name: "Tab",
    },
    KeyMapU16 {
        code: 16,
        name: "Q",
    },
    KeyMapU16 {
        code: 17,
        name: "W",
    },
    KeyMapU16 {
        code: 18,
        name: "E",
    },
    KeyMapU16 {
        code: 19,
        name: "R",
    },
    KeyMapU16 {
        code: 20,
        name: "T",
    },
    KeyMapU16 {
        code: 21,
        name: "Y",
    },
    KeyMapU16 {
        code: 22,
        name: "U",
    },
    KeyMapU16 {
        code: 23,
        name: "I",
    },
    KeyMapU16 {
        code: 24,
        name: "O",
    },
    KeyMapU16 {
        code: 25,
        name: "P",
    },
    KeyMapU16 {
        code: 26,
        name: "BracketLeft",
    },
    KeyMapU16 {
        code: 27,
        name: "BracketRight",
    },
    KeyMapU16 {
        code: 28,
        name: "Enter",
    },
    KeyMapU16 {
        code: 29,
        name: "ControlLeft",
    },
    KeyMapU16 {
        code: 30,
        name: "A",
    },
    KeyMapU16 {
        code: 31,
        name: "S",
    },
    KeyMapU16 {
        code: 32,
        name: "D",
    },
    KeyMapU16 {
        code: 33,
        name: "F",
    },
    KeyMapU16 {
        code: 34,
        name: "G",
    },
    KeyMapU16 {
        code: 35,
        name: "H",
    },
    KeyMapU16 {
        code: 36,
        name: "J",
    },
    KeyMapU16 {
        code: 37,
        name: "K",
    },
    KeyMapU16 {
        code: 38,
        name: "L",
    },
    KeyMapU16 {
        code: 39,
        name: "Semicolon",
    },
    KeyMapU16 {
        code: 40,
        name: "Quote",
    },
    KeyMapU16 {
        code: 41,
        name: "Backquote",
    },
    KeyMapU16 {
        code: 42,
        name: "ShiftLeft",
    },
    KeyMapU16 {
        code: 43,
        name: "Backslash",
    },
    KeyMapU16 {
        code: 44,
        name: "Z",
    },
    KeyMapU16 {
        code: 45,
        name: "X",
    },
    KeyMapU16 {
        code: 46,
        name: "C",
    },
    KeyMapU16 {
        code: 47,
        name: "V",
    },
    KeyMapU16 {
        code: 48,
        name: "B",
    },
    KeyMapU16 {
        code: 49,
        name: "N",
    },
    KeyMapU16 {
        code: 50,
        name: "M",
    },
    KeyMapU16 {
        code: 51,
        name: "Comma",
    },
    KeyMapU16 {
        code: 52,
        name: "Period",
    },
    KeyMapU16 {
        code: 53,
        name: "Slash",
    },
    KeyMapU16 {
        code: 54,
        name: "ShiftRight",
    },
    KeyMapU16 {
        code: 55,
        name: "NumpadMultiply",
    },
    KeyMapU16 {
        code: 56,
        name: "AltLeft",
    },
    KeyMapU16 {
        code: 57,
        name: "Space",
    },
    KeyMapU16 {
        code: 58,
        name: "CapsLock",
    },
    KeyMapU16 {
        code: 59,
        name: "F1",
    },
    KeyMapU16 {
        code: 60,
        name: "F2",
    },
    KeyMapU16 {
        code: 61,
        name: "F3",
    },
    KeyMapU16 {
        code: 62,
        name: "F4",
    },
    KeyMapU16 {
        code: 63,
        name: "F5",
    },
    KeyMapU16 {
        code: 64,
        name: "F6",
    },
    KeyMapU16 {
        code: 65,
        name: "F7",
    },
    KeyMapU16 {
        code: 66,
        name: "F8",
    },
    KeyMapU16 {
        code: 67,
        name: "F9",
    },
    KeyMapU16 {
        code: 68,
        name: "F10",
    },
    KeyMapU16 {
        code: 69,
        name: "NumLock",
    },
    KeyMapU16 {
        code: 70,
        name: "ScrollLock",
    },
    KeyMapU16 {
        code: 71,
        name: "Numpad7",
    },
    KeyMapU16 {
        code: 72,
        name: "Numpad8",
    },
    KeyMapU16 {
        code: 73,
        name: "Numpad9",
    },
    KeyMapU16 {
        code: 74,
        name: "NumpadSubtract",
    },
    KeyMapU16 {
        code: 75,
        name: "Numpad4",
    },
    KeyMapU16 {
        code: 76,
        name: "Numpad5",
    },
    KeyMapU16 {
        code: 77,
        name: "Numpad6",
    },
    KeyMapU16 {
        code: 78,
        name: "NumpadAdd",
    },
    KeyMapU16 {
        code: 79,
        name: "Numpad1",
    },
    KeyMapU16 {
        code: 80,
        name: "Numpad2",
    },
    KeyMapU16 {
        code: 81,
        name: "Numpad3",
    },
    KeyMapU16 {
        code: 82,
        name: "Numpad0",
    },
    KeyMapU16 {
        code: 83,
        name: "NumpadDecimal",
    },
    KeyMapU16 {
        code: 86,
        name: "IntlBackslash",
    },
    KeyMapU16 {
        code: 119,
        name: "Pause",
    },
    KeyMapU16 {
        code: 87,
        name: "F11",
    },
    KeyMapU16 {
        code: 88,
        name: "F12",
    },
    KeyMapU16 {
        code: 89,
        name: "IntlRo",
    },
    KeyMapU16 {
        code: 90,
        name: "Lang3",
    },
    KeyMapU16 {
        code: 92,
        name: "Convert",
    },
    KeyMapU16 {
        code: 93,
        name: "KanaMode",
    },
    KeyMapU16 {
        code: 94,
        name: "NonConvert",
    },
    KeyMapU16 {
        code: 96,
        name: "NumpadEnter",
    },
    KeyMapU16 {
        code: 97,
        name: "ControlRight",
    },
    KeyMapU16 {
        code: 98,
        name: "NumpadDivide",
    },
    KeyMapU16 {
        code: 99,
        name: "PrintScreen",
    },
    KeyMapU16 {
        code: 100,
        name: "AltRight",
    },
    KeyMapU16 {
        code: 102,
        name: "Home",
    },
    KeyMapU16 {
        code: 103,
        name: "ArrowUp",
    },
    KeyMapU16 {
        code: 104,
        name: "PageUp",
    },
    KeyMapU16 {
        code: 105,
        name: "ArrowLeft",
    },
    KeyMapU16 {
        code: 106,
        name: "ArrowRight",
    },
    KeyMapU16 {
        code: 107,
        name: "End",
    },
    KeyMapU16 {
        code: 108,
        name: "ArrowDown",
    },
    KeyMapU16 {
        code: 109,
        name: "PageDown",
    },
    KeyMapU16 {
        code: 110,
        name: "Insert",
    },
    KeyMapU16 {
        code: 111,
        name: "Delete",
    },
    KeyMapU16 {
        code: 113,
        name: "AudioVolumeMute",
    },
    KeyMapU16 {
        code: 114,
        name: "AudioVolumeDown",
    },
    KeyMapU16 {
        code: 115,
        name: "AudioVolumeUp",
    },
    KeyMapU16 {
        code: 116,
        name: "Power",
    },
    KeyMapU16 {
        code: 117,
        name: "NumpadEqual",
    },
    KeyMapU16 {
        code: 121,
        name: "NumpadComma",
    },
    KeyMapU16 {
        code: 122,
        name: "Lang1",
    },
    KeyMapU16 {
        code: 123,
        name: "Lang2",
    },
    KeyMapU16 {
        code: 124,
        name: "IntlYen",
    },
    KeyMapU16 {
        code: 125,
        name: "MetaLeft",
    },
    KeyMapU16 {
        code: 126,
        name: "MetaRight",
    },
    KeyMapU16 {
        code: 127,
        name: "ContextMenu",
    },
    KeyMapU16 {
        code: 142,
        name: "Sleep",
    },
    KeyMapU16 {
        code: 143,
        name: "WakeUp",
    },
    KeyMapU16 {
        code: 148,
        name: "LaunchApp1",
    },
    KeyMapU16 {
        code: 149,
        name: "LaunchApp2",
    },
    KeyMapU16 {
        code: 155,
        name: "LaunchMail",
    },
    KeyMapU16 {
        code: 158,
        name: "BrowserBack",
    },
    KeyMapU16 {
        code: 159,
        name: "BrowserForward",
    },
    KeyMapU16 {
        code: 163,
        name: "MediaTrackNext",
    },
    KeyMapU16 {
        code: 164,
        name: "MediaPlayPause",
    },
    KeyMapU16 {
        code: 165,
        name: "MediaTrackPrevious",
    },
    KeyMapU16 {
        code: 166,
        name: "MediaStop",
    },
    KeyMapU16 {
        code: 172,
        name: "BrowserHome",
    },
    KeyMapU16 {
        code: 173,
        name: "BrowserRefresh",
    },
    KeyMapU16 {
        code: 183,
        name: "F13",
    },
    KeyMapU16 {
        code: 184,
        name: "F14",
    },
    KeyMapU16 {
        code: 185,
        name: "F15",
    },
    KeyMapU16 {
        code: 186,
        name: "F16",
    },
    KeyMapU16 {
        code: 187,
        name: "F17",
    },
    KeyMapU16 {
        code: 188,
        name: "F18",
    },
    KeyMapU16 {
        code: 189,
        name: "F19",
    },
    KeyMapU16 {
        code: 190,
        name: "F20",
    },
    KeyMapU16 {
        code: 191,
        name: "F21",
    },
    KeyMapU16 {
        code: 192,
        name: "F22",
    },
    KeyMapU16 {
        code: 193,
        name: "F23",
    },
    KeyMapU16 {
        code: 194,
        name: "F24",
    },
    KeyMapU16 {
        code: 217,
        name: "BrowserSearch",
    },
    KeyMapU16 {
        code: 226,
        name: "LaunchMediaPlayer",
    },
    KeyMapU16 {
        code: 364,
        name: "BrowserFavorites",
    },
];

pub const LEFT_CTRL: u16 = 29;
pub const RIGHT_CTRL: u16 = 97;
pub const LEFT_SHIFT: u16 = 42;
pub const RIGHT_SHIFT: u16 = 54;
pub const LEFT_ALT: u16 = 56;
pub const RIGHT_ALT: u16 = 100;
pub const LEFT_META: u16 = 125;
pub const RIGHT_META: u16 = 126;

pub const BTN_LEFT: u16 = 0x110;
pub const BTN_RIGHT: u16 = 0x111;
pub const BTN_MIDDLE: u16 = 0x112;
pub const BTN_SIDE: u16 = 0x113;
pub const BTN_EXTRA: u16 = 0x114;
pub const BTN_FORWARD: u16 = 0x115;
pub const BTN_BACK: u16 = 0x116;

pub fn evdev_button_to_browser_button(code: u16) -> Option<u8> {
    match code {
        BTN_LEFT => Some(0),
        BTN_MIDDLE => Some(1),
        BTN_RIGHT => Some(2),
        BTN_SIDE | BTN_BACK => Some(3),
        BTN_EXTRA | BTN_FORWARD => Some(4),
        _ => None,
    }
}

pub fn keycode_to_name(code: u16) -> Option<&'static str> {
    KEY_MAP
        .iter()
        .find(|entry| entry.code == code)
        .map(|entry| entry.name)
}

pub fn name_to_keycode(name: &str) -> u16 {
    KEY_MAP
        .iter()
        .find(|entry| entry.name == name)
        .map_or(0, |entry| entry.code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keycode_to_name_covers_canonical_letters_and_arrows() {
        assert_eq!(Some("A"), keycode_to_name(30));
        assert_eq!(Some("Z"), keycode_to_name(44));
        assert_eq!(Some("Pause"), keycode_to_name(119));
        assert_eq!(Some("F13"), keycode_to_name(183));
        assert_eq!(Some("NumpadEnter"), keycode_to_name(96));
        assert_eq!(Some("AudioVolumeMute"), keycode_to_name(113));
        assert_eq!(Some("ArrowUp"), keycode_to_name(103));
        assert_eq!(Some("MetaLeft"), keycode_to_name(125));
        assert_eq!(None, keycode_to_name(0));
        assert_eq!(None, keycode_to_name(0xffff));
    }

    #[test]
    fn name_to_keycode_round_trips_every_entry() {
        for entry in KEY_MAP {
            assert_eq!(entry.code, name_to_keycode(entry.name));
        }
        assert_eq!(0, name_to_keycode("NoSuchKey"));
        assert_eq!(0, name_to_keycode(""));
    }

    #[test]
    fn evdev_button_to_browser_button_matches_dom_convention() {
        assert_eq!(Some(0), evdev_button_to_browser_button(BTN_LEFT));
        assert_eq!(Some(1), evdev_button_to_browser_button(BTN_MIDDLE));
        assert_eq!(Some(2), evdev_button_to_browser_button(BTN_RIGHT));
        assert_eq!(Some(3), evdev_button_to_browser_button(BTN_SIDE));
        assert_eq!(Some(3), evdev_button_to_browser_button(BTN_BACK));
        assert_eq!(Some(4), evdev_button_to_browser_button(BTN_EXTRA));
        assert_eq!(Some(4), evdev_button_to_browser_button(BTN_FORWARD));
        assert_eq!(None, evdev_button_to_browser_button(0x100));
        assert_eq!(None, evdev_button_to_browser_button(0xffff));
    }
}
