// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyMapU32 {
    pub code: u32,
    pub name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyMapU16 {
    pub code: u16,
    pub name: &'static str,
}

pub mod linux_x11 {
    use super::KeyMapU32;

    pub const KEYSYM_TABLE: &[KeyMapU32] = &[
        KeyMapU32 {
            code: 0xff1b,
            name: "Escape",
        },
        KeyMapU32 {
            code: 0xffbe,
            name: "F1",
        },
        KeyMapU32 {
            code: 0xffbf,
            name: "F2",
        },
        KeyMapU32 {
            code: 0xffc0,
            name: "F3",
        },
        KeyMapU32 {
            code: 0xffc1,
            name: "F4",
        },
        KeyMapU32 {
            code: 0xffc2,
            name: "F5",
        },
        KeyMapU32 {
            code: 0xffc3,
            name: "F6",
        },
        KeyMapU32 {
            code: 0xffc4,
            name: "F7",
        },
        KeyMapU32 {
            code: 0xffc5,
            name: "F8",
        },
        KeyMapU32 {
            code: 0xffc6,
            name: "F9",
        },
        KeyMapU32 {
            code: 0xffc7,
            name: "F10",
        },
        KeyMapU32 {
            code: 0xffc8,
            name: "F11",
        },
        KeyMapU32 {
            code: 0xffc9,
            name: "F12",
        },
        KeyMapU32 {
            code: 0xffca,
            name: "F13",
        },
        KeyMapU32 {
            code: 0xffcb,
            name: "F14",
        },
        KeyMapU32 {
            code: 0xffcc,
            name: "F15",
        },
        KeyMapU32 {
            code: 0xffcd,
            name: "F16",
        },
        KeyMapU32 {
            code: 0xffce,
            name: "F17",
        },
        KeyMapU32 {
            code: 0xffcf,
            name: "F18",
        },
        KeyMapU32 {
            code: 0xffd0,
            name: "F19",
        },
        KeyMapU32 {
            code: 0xffd1,
            name: "F20",
        },
        KeyMapU32 {
            code: 0xffd2,
            name: "F21",
        },
        KeyMapU32 {
            code: 0xffd3,
            name: "F22",
        },
        KeyMapU32 {
            code: 0xffd4,
            name: "F23",
        },
        KeyMapU32 {
            code: 0xffd5,
            name: "F24",
        },
        KeyMapU32 {
            code: 0xff61,
            name: "PrintScreen",
        },
        KeyMapU32 {
            code: 0xff14,
            name: "ScrollLock",
        },
        KeyMapU32 {
            code: 0xff13,
            name: "Pause",
        },
        KeyMapU32 {
            code: 0xff7f,
            name: "NumLock",
        },
        KeyMapU32 {
            code: 0xff67,
            name: "ContextMenu",
        },
        KeyMapU32 {
            code: 0x0060,
            name: "Backquote",
        },
        KeyMapU32 {
            code: 0x007e,
            name: "Backquote",
        },
        KeyMapU32 {
            code: 0x0031,
            name: "1",
        },
        KeyMapU32 {
            code: 0x0032,
            name: "2",
        },
        KeyMapU32 {
            code: 0x0033,
            name: "3",
        },
        KeyMapU32 {
            code: 0x0034,
            name: "4",
        },
        KeyMapU32 {
            code: 0x0035,
            name: "5",
        },
        KeyMapU32 {
            code: 0x0036,
            name: "6",
        },
        KeyMapU32 {
            code: 0x0037,
            name: "7",
        },
        KeyMapU32 {
            code: 0x0038,
            name: "8",
        },
        KeyMapU32 {
            code: 0x0039,
            name: "9",
        },
        KeyMapU32 {
            code: 0x0030,
            name: "0",
        },
        KeyMapU32 {
            code: 0x002d,
            name: "Minus",
        },
        KeyMapU32 {
            code: 0x003d,
            name: "Equal",
        },
        KeyMapU32 {
            code: 0xff08,
            name: "Backspace",
        },
        KeyMapU32 {
            code: 0xff09,
            name: "Tab",
        },
        KeyMapU32 {
            code: 0x0071,
            name: "Q",
        },
        KeyMapU32 {
            code: 0x0077,
            name: "W",
        },
        KeyMapU32 {
            code: 0x0065,
            name: "E",
        },
        KeyMapU32 {
            code: 0x0072,
            name: "R",
        },
        KeyMapU32 {
            code: 0x0074,
            name: "T",
        },
        KeyMapU32 {
            code: 0x0079,
            name: "Y",
        },
        KeyMapU32 {
            code: 0x0075,
            name: "U",
        },
        KeyMapU32 {
            code: 0x0069,
            name: "I",
        },
        KeyMapU32 {
            code: 0x006f,
            name: "O",
        },
        KeyMapU32 {
            code: 0x0070,
            name: "P",
        },
        KeyMapU32 {
            code: 0x005b,
            name: "BracketLeft",
        },
        KeyMapU32 {
            code: 0x005d,
            name: "BracketRight",
        },
        KeyMapU32 {
            code: 0x005c,
            name: "Backslash",
        },
        KeyMapU32 {
            code: 0xffe5,
            name: "CapsLock",
        },
        KeyMapU32 {
            code: 0x0061,
            name: "A",
        },
        KeyMapU32 {
            code: 0x0073,
            name: "S",
        },
        KeyMapU32 {
            code: 0x0064,
            name: "D",
        },
        KeyMapU32 {
            code: 0x0066,
            name: "F",
        },
        KeyMapU32 {
            code: 0x0067,
            name: "G",
        },
        KeyMapU32 {
            code: 0x0068,
            name: "H",
        },
        KeyMapU32 {
            code: 0x006a,
            name: "J",
        },
        KeyMapU32 {
            code: 0x006b,
            name: "K",
        },
        KeyMapU32 {
            code: 0x006c,
            name: "L",
        },
        KeyMapU32 {
            code: 0x003b,
            name: "Semicolon",
        },
        KeyMapU32 {
            code: 0x0027,
            name: "Quote",
        },
        KeyMapU32 {
            code: 0xff0d,
            name: "Enter",
        },
        KeyMapU32 {
            code: 0xffe1,
            name: "ShiftLeft",
        },
        KeyMapU32 {
            code: 0x007a,
            name: "Z",
        },
        KeyMapU32 {
            code: 0x0078,
            name: "X",
        },
        KeyMapU32 {
            code: 0x0063,
            name: "C",
        },
        KeyMapU32 {
            code: 0x0076,
            name: "V",
        },
        KeyMapU32 {
            code: 0x0062,
            name: "B",
        },
        KeyMapU32 {
            code: 0x006e,
            name: "N",
        },
        KeyMapU32 {
            code: 0x006d,
            name: "M",
        },
        KeyMapU32 {
            code: 0x002c,
            name: "Comma",
        },
        KeyMapU32 {
            code: 0x002e,
            name: "Period",
        },
        KeyMapU32 {
            code: 0x002f,
            name: "Slash",
        },
        KeyMapU32 {
            code: 0xffe2,
            name: "ShiftRight",
        },
        KeyMapU32 {
            code: 0xffe3,
            name: "ControlLeft",
        },
        KeyMapU32 {
            code: 0xffeb,
            name: "MetaLeft",
        },
        KeyMapU32 {
            code: 0xffe9,
            name: "AltLeft",
        },
        KeyMapU32 {
            code: 0x0020,
            name: "Space",
        },
        KeyMapU32 {
            code: 0xffea,
            name: "AltRight",
        },
        KeyMapU32 {
            code: 0xffec,
            name: "MetaRight",
        },
        KeyMapU32 {
            code: 0xffe4,
            name: "ControlRight",
        },
        KeyMapU32 {
            code: 0xff80,
            name: "Space",
        },
        KeyMapU32 {
            code: 0xff89,
            name: "Tab",
        },
        KeyMapU32 {
            code: 0xff8d,
            name: "NumpadEnter",
        },
        KeyMapU32 {
            code: 0xffbd,
            name: "NumpadEqual",
        },
        KeyMapU32 {
            code: 0xffaa,
            name: "NumpadMultiply",
        },
        KeyMapU32 {
            code: 0xffab,
            name: "NumpadAdd",
        },
        KeyMapU32 {
            code: 0xffac,
            name: "NumpadComma",
        },
        KeyMapU32 {
            code: 0xffad,
            name: "NumpadSubtract",
        },
        KeyMapU32 {
            code: 0xffae,
            name: "NumpadDecimal",
        },
        KeyMapU32 {
            code: 0xffaf,
            name: "NumpadDivide",
        },
        KeyMapU32 {
            code: 0xffb0,
            name: "Numpad0",
        },
        KeyMapU32 {
            code: 0xffb1,
            name: "Numpad1",
        },
        KeyMapU32 {
            code: 0xffb2,
            name: "Numpad2",
        },
        KeyMapU32 {
            code: 0xffb3,
            name: "Numpad3",
        },
        KeyMapU32 {
            code: 0xffb4,
            name: "Numpad4",
        },
        KeyMapU32 {
            code: 0xffb5,
            name: "Numpad5",
        },
        KeyMapU32 {
            code: 0xffb6,
            name: "Numpad6",
        },
        KeyMapU32 {
            code: 0xffb7,
            name: "Numpad7",
        },
        KeyMapU32 {
            code: 0xffb8,
            name: "Numpad8",
        },
        KeyMapU32 {
            code: 0xffb9,
            name: "Numpad9",
        },
        KeyMapU32 {
            code: 0xff51,
            name: "ArrowLeft",
        },
        KeyMapU32 {
            code: 0xff52,
            name: "ArrowUp",
        },
        KeyMapU32 {
            code: 0xff53,
            name: "ArrowRight",
        },
        KeyMapU32 {
            code: 0xff54,
            name: "ArrowDown",
        },
        KeyMapU32 {
            code: 0xff63,
            name: "Insert",
        },
        KeyMapU32 {
            code: 0xffff,
            name: "Delete",
        },
        KeyMapU32 {
            code: 0xff50,
            name: "Home",
        },
        KeyMapU32 {
            code: 0xff57,
            name: "End",
        },
        KeyMapU32 {
            code: 0xff55,
            name: "PageUp",
        },
        KeyMapU32 {
            code: 0xff56,
            name: "PageDown",
        },
        KeyMapU32 {
            code: 0x1008ff12,
            name: "AudioVolumeMute",
        },
        KeyMapU32 {
            code: 0x1008ff11,
            name: "AudioVolumeDown",
        },
        KeyMapU32 {
            code: 0x1008ff13,
            name: "AudioVolumeUp",
        },
        KeyMapU32 {
            code: 0x1008ff17,
            name: "MediaTrackNext",
        },
        KeyMapU32 {
            code: 0x1008ff16,
            name: "MediaTrackPrevious",
        },
        KeyMapU32 {
            code: 0x1008ff15,
            name: "MediaStop",
        },
        KeyMapU32 {
            code: 0x1008ff14,
            name: "MediaPlayPause",
        },
        KeyMapU32 {
            code: 0x1008ff26,
            name: "BrowserBack",
        },
        KeyMapU32 {
            code: 0x1008ff27,
            name: "BrowserForward",
        },
        KeyMapU32 {
            code: 0x1008ff29,
            name: "BrowserRefresh",
        },
        KeyMapU32 {
            code: 0x1008ff28,
            name: "BrowserStop",
        },
        KeyMapU32 {
            code: 0x1008ff1b,
            name: "BrowserSearch",
        },
        KeyMapU32 {
            code: 0x1008ff30,
            name: "BrowserFavorites",
        },
        KeyMapU32 {
            code: 0x1008ff18,
            name: "BrowserHome",
        },
        KeyMapU32 {
            code: 0x1008ff19,
            name: "LaunchMail",
        },
        KeyMapU32 {
            code: 0x1008ff32,
            name: "LaunchMediaPlayer",
        },
        KeyMapU32 {
            code: 0x1008ff41,
            name: "LaunchApp1",
        },
        KeyMapU32 {
            code: 0x1008ff42,
            name: "LaunchApp2",
        },
        KeyMapU32 {
            code: 0x1008ff2a,
            name: "Power",
        },
        KeyMapU32 {
            code: 0x1008ff2f,
            name: "Sleep",
        },
        KeyMapU32 {
            code: 0x1008ff2b,
            name: "WakeUp",
        },
        KeyMapU32 {
            code: 0xff23,
            name: "Convert",
        },
        KeyMapU32 {
            code: 0xff22,
            name: "NonConvert",
        },
        KeyMapU32 {
            code: 0xff2d,
            name: "KanaMode",
        },
        KeyMapU32 {
            code: 0xff31,
            name: "Lang1",
        },
        KeyMapU32 {
            code: 0xff34,
            name: "Lang2",
        },
    ];

    pub fn keysym_to_name(keysym: u32) -> Option<&'static str> {
        KEYSYM_TABLE
            .iter()
            .find(|entry| entry.code == keysym)
            .map(|entry| entry.name)
    }
}

pub mod windows {
    use super::KeyMapU16;

    pub const VK_TABLE: &[KeyMapU16] = &[
        KeyMapU16 {
            code: 0x1b,
            name: "Escape",
        },
        KeyMapU16 {
            code: 0x70,
            name: "F1",
        },
        KeyMapU16 {
            code: 0x71,
            name: "F2",
        },
        KeyMapU16 {
            code: 0x72,
            name: "F3",
        },
        KeyMapU16 {
            code: 0x73,
            name: "F4",
        },
        KeyMapU16 {
            code: 0x74,
            name: "F5",
        },
        KeyMapU16 {
            code: 0x75,
            name: "F6",
        },
        KeyMapU16 {
            code: 0x76,
            name: "F7",
        },
        KeyMapU16 {
            code: 0x77,
            name: "F8",
        },
        KeyMapU16 {
            code: 0x78,
            name: "F9",
        },
        KeyMapU16 {
            code: 0x79,
            name: "F10",
        },
        KeyMapU16 {
            code: 0x7a,
            name: "F11",
        },
        KeyMapU16 {
            code: 0x7b,
            name: "F12",
        },
        KeyMapU16 {
            code: 0x7c,
            name: "F13",
        },
        KeyMapU16 {
            code: 0x7d,
            name: "F14",
        },
        KeyMapU16 {
            code: 0x7e,
            name: "F15",
        },
        KeyMapU16 {
            code: 0x7f,
            name: "F16",
        },
        KeyMapU16 {
            code: 0x80,
            name: "F17",
        },
        KeyMapU16 {
            code: 0x81,
            name: "F18",
        },
        KeyMapU16 {
            code: 0x82,
            name: "F19",
        },
        KeyMapU16 {
            code: 0x83,
            name: "F20",
        },
        KeyMapU16 {
            code: 0x84,
            name: "F21",
        },
        KeyMapU16 {
            code: 0x85,
            name: "F22",
        },
        KeyMapU16 {
            code: 0x86,
            name: "F23",
        },
        KeyMapU16 {
            code: 0x87,
            name: "F24",
        },
        KeyMapU16 {
            code: 0x13,
            name: "Pause",
        },
        KeyMapU16 {
            code: 0x2c,
            name: "PrintScreen",
        },
        KeyMapU16 {
            code: 0x91,
            name: "ScrollLock",
        },
        KeyMapU16 {
            code: 0x90,
            name: "NumLock",
        },
        KeyMapU16 {
            code: 0x5d,
            name: "ContextMenu",
        },
        KeyMapU16 {
            code: 0xc0,
            name: "Backquote",
        },
        KeyMapU16 {
            code: 0x31,
            name: "1",
        },
        KeyMapU16 {
            code: 0x32,
            name: "2",
        },
        KeyMapU16 {
            code: 0x33,
            name: "3",
        },
        KeyMapU16 {
            code: 0x34,
            name: "4",
        },
        KeyMapU16 {
            code: 0x35,
            name: "5",
        },
        KeyMapU16 {
            code: 0x36,
            name: "6",
        },
        KeyMapU16 {
            code: 0x37,
            name: "7",
        },
        KeyMapU16 {
            code: 0x38,
            name: "8",
        },
        KeyMapU16 {
            code: 0x39,
            name: "9",
        },
        KeyMapU16 {
            code: 0x30,
            name: "0",
        },
        KeyMapU16 {
            code: 0xbd,
            name: "Minus",
        },
        KeyMapU16 {
            code: 0xbb,
            name: "Equal",
        },
        KeyMapU16 {
            code: 0x08,
            name: "Backspace",
        },
        KeyMapU16 {
            code: 0x09,
            name: "Tab",
        },
        KeyMapU16 {
            code: 0x51,
            name: "Q",
        },
        KeyMapU16 {
            code: 0x57,
            name: "W",
        },
        KeyMapU16 {
            code: 0x45,
            name: "E",
        },
        KeyMapU16 {
            code: 0x52,
            name: "R",
        },
        KeyMapU16 {
            code: 0x54,
            name: "T",
        },
        KeyMapU16 {
            code: 0x59,
            name: "Y",
        },
        KeyMapU16 {
            code: 0x55,
            name: "U",
        },
        KeyMapU16 {
            code: 0x49,
            name: "I",
        },
        KeyMapU16 {
            code: 0x4f,
            name: "O",
        },
        KeyMapU16 {
            code: 0x50,
            name: "P",
        },
        KeyMapU16 {
            code: 0xdb,
            name: "BracketLeft",
        },
        KeyMapU16 {
            code: 0xdd,
            name: "BracketRight",
        },
        KeyMapU16 {
            code: 0xdc,
            name: "Backslash",
        },
        KeyMapU16 {
            code: 0x14,
            name: "CapsLock",
        },
        KeyMapU16 {
            code: 0x41,
            name: "A",
        },
        KeyMapU16 {
            code: 0x53,
            name: "S",
        },
        KeyMapU16 {
            code: 0x44,
            name: "D",
        },
        KeyMapU16 {
            code: 0x46,
            name: "F",
        },
        KeyMapU16 {
            code: 0x47,
            name: "G",
        },
        KeyMapU16 {
            code: 0x48,
            name: "H",
        },
        KeyMapU16 {
            code: 0x4a,
            name: "J",
        },
        KeyMapU16 {
            code: 0x4b,
            name: "K",
        },
        KeyMapU16 {
            code: 0x4c,
            name: "L",
        },
        KeyMapU16 {
            code: 0xba,
            name: "Semicolon",
        },
        KeyMapU16 {
            code: 0xde,
            name: "Quote",
        },
        KeyMapU16 {
            code: 0x0d,
            name: "Enter",
        },
        KeyMapU16 {
            code: 0xa0,
            name: "ShiftLeft",
        },
        KeyMapU16 {
            code: 0x5a,
            name: "Z",
        },
        KeyMapU16 {
            code: 0x58,
            name: "X",
        },
        KeyMapU16 {
            code: 0x43,
            name: "C",
        },
        KeyMapU16 {
            code: 0x56,
            name: "V",
        },
        KeyMapU16 {
            code: 0x42,
            name: "B",
        },
        KeyMapU16 {
            code: 0x4e,
            name: "N",
        },
        KeyMapU16 {
            code: 0x4d,
            name: "M",
        },
        KeyMapU16 {
            code: 0xbc,
            name: "Comma",
        },
        KeyMapU16 {
            code: 0xbe,
            name: "Period",
        },
        KeyMapU16 {
            code: 0xbf,
            name: "Slash",
        },
        KeyMapU16 {
            code: 0xa1,
            name: "ShiftRight",
        },
        KeyMapU16 {
            code: 0xa2,
            name: "ControlLeft",
        },
        KeyMapU16 {
            code: 0x5b,
            name: "MetaLeft",
        },
        KeyMapU16 {
            code: 0xa4,
            name: "AltLeft",
        },
        KeyMapU16 {
            code: 0x20,
            name: "Space",
        },
        KeyMapU16 {
            code: 0xa5,
            name: "AltRight",
        },
        KeyMapU16 {
            code: 0x5c,
            name: "MetaRight",
        },
        KeyMapU16 {
            code: 0xa3,
            name: "ControlRight",
        },
        KeyMapU16 {
            code: 0x60,
            name: "Numpad0",
        },
        KeyMapU16 {
            code: 0x61,
            name: "Numpad1",
        },
        KeyMapU16 {
            code: 0x62,
            name: "Numpad2",
        },
        KeyMapU16 {
            code: 0x63,
            name: "Numpad3",
        },
        KeyMapU16 {
            code: 0x64,
            name: "Numpad4",
        },
        KeyMapU16 {
            code: 0x65,
            name: "Numpad5",
        },
        KeyMapU16 {
            code: 0x66,
            name: "Numpad6",
        },
        KeyMapU16 {
            code: 0x67,
            name: "Numpad7",
        },
        KeyMapU16 {
            code: 0x68,
            name: "Numpad8",
        },
        KeyMapU16 {
            code: 0x69,
            name: "Numpad9",
        },
        KeyMapU16 {
            code: 0x6a,
            name: "NumpadMultiply",
        },
        KeyMapU16 {
            code: 0x6b,
            name: "NumpadAdd",
        },
        KeyMapU16 {
            code: 0x6c,
            name: "NumpadComma",
        },
        KeyMapU16 {
            code: 0x6d,
            name: "NumpadSubtract",
        },
        KeyMapU16 {
            code: 0x6e,
            name: "NumpadDecimal",
        },
        KeyMapU16 {
            code: 0x6f,
            name: "NumpadDivide",
        },
        KeyMapU16 {
            code: 0x92,
            name: "NumpadEqual",
        },
        KeyMapU16 {
            code: 0x0c,
            name: "Numpad5",
        },
        KeyMapU16 {
            code: 0x25,
            name: "ArrowLeft",
        },
        KeyMapU16 {
            code: 0x26,
            name: "ArrowUp",
        },
        KeyMapU16 {
            code: 0x27,
            name: "ArrowRight",
        },
        KeyMapU16 {
            code: 0x28,
            name: "ArrowDown",
        },
        KeyMapU16 {
            code: 0x2d,
            name: "Insert",
        },
        KeyMapU16 {
            code: 0x2e,
            name: "Delete",
        },
        KeyMapU16 {
            code: 0x24,
            name: "Home",
        },
        KeyMapU16 {
            code: 0x23,
            name: "End",
        },
        KeyMapU16 {
            code: 0x21,
            name: "PageUp",
        },
        KeyMapU16 {
            code: 0x22,
            name: "PageDown",
        },
        KeyMapU16 {
            code: 0xad,
            name: "AudioVolumeMute",
        },
        KeyMapU16 {
            code: 0xae,
            name: "AudioVolumeDown",
        },
        KeyMapU16 {
            code: 0xaf,
            name: "AudioVolumeUp",
        },
        KeyMapU16 {
            code: 0xb0,
            name: "MediaTrackNext",
        },
        KeyMapU16 {
            code: 0xb1,
            name: "MediaTrackPrevious",
        },
        KeyMapU16 {
            code: 0xb2,
            name: "MediaStop",
        },
        KeyMapU16 {
            code: 0xb3,
            name: "MediaPlayPause",
        },
        KeyMapU16 {
            code: 0xa6,
            name: "BrowserBack",
        },
        KeyMapU16 {
            code: 0xa7,
            name: "BrowserForward",
        },
        KeyMapU16 {
            code: 0xa8,
            name: "BrowserRefresh",
        },
        KeyMapU16 {
            code: 0xa9,
            name: "BrowserStop",
        },
        KeyMapU16 {
            code: 0xaa,
            name: "BrowserSearch",
        },
        KeyMapU16 {
            code: 0xab,
            name: "BrowserFavorites",
        },
        KeyMapU16 {
            code: 0xac,
            name: "BrowserHome",
        },
        KeyMapU16 {
            code: 0xb4,
            name: "LaunchMail",
        },
        KeyMapU16 {
            code: 0xb5,
            name: "LaunchMediaPlayer",
        },
        KeyMapU16 {
            code: 0xb6,
            name: "LaunchApp1",
        },
        KeyMapU16 {
            code: 0xb7,
            name: "LaunchApp2",
        },
        KeyMapU16 {
            code: 0x1c,
            name: "Convert",
        },
        KeyMapU16 {
            code: 0x1d,
            name: "NonConvert",
        },
        KeyMapU16 {
            code: 0x15,
            name: "KanaMode",
        },
        KeyMapU16 {
            code: 0x5f,
            name: "Sleep",
        },
    ];

    pub fn vk_to_name(vk: u16) -> Option<&'static str> {
        VK_TABLE
            .iter()
            .find(|entry| entry.code == vk)
            .map(|entry| entry.name)
    }
}

pub mod macos {
    use super::KeyMapU16;

    pub const KEYCODE_TABLE: &[KeyMapU16] = &[
        KeyMapU16 {
            code: 0x35,
            name: "Escape",
        },
        KeyMapU16 {
            code: 0x7a,
            name: "F1",
        },
        KeyMapU16 {
            code: 0x78,
            name: "F2",
        },
        KeyMapU16 {
            code: 0x63,
            name: "F3",
        },
        KeyMapU16 {
            code: 0x76,
            name: "F4",
        },
        KeyMapU16 {
            code: 0x60,
            name: "F5",
        },
        KeyMapU16 {
            code: 0x61,
            name: "F6",
        },
        KeyMapU16 {
            code: 0x62,
            name: "F7",
        },
        KeyMapU16 {
            code: 0x64,
            name: "F8",
        },
        KeyMapU16 {
            code: 0x65,
            name: "F9",
        },
        KeyMapU16 {
            code: 0x6d,
            name: "F10",
        },
        KeyMapU16 {
            code: 0x67,
            name: "F11",
        },
        KeyMapU16 {
            code: 0x6f,
            name: "F12",
        },
        KeyMapU16 {
            code: 0x69,
            name: "F13",
        },
        KeyMapU16 {
            code: 0x6b,
            name: "F14",
        },
        KeyMapU16 {
            code: 0x71,
            name: "F15",
        },
        KeyMapU16 {
            code: 0x6a,
            name: "F16",
        },
        KeyMapU16 {
            code: 0x40,
            name: "F17",
        },
        KeyMapU16 {
            code: 0x4f,
            name: "F18",
        },
        KeyMapU16 {
            code: 0x50,
            name: "F19",
        },
        KeyMapU16 {
            code: 0x5a,
            name: "F20",
        },
        KeyMapU16 {
            code: 0x32,
            name: "Backquote",
        },
        KeyMapU16 {
            code: 0x12,
            name: "1",
        },
        KeyMapU16 {
            code: 0x13,
            name: "2",
        },
        KeyMapU16 {
            code: 0x14,
            name: "3",
        },
        KeyMapU16 {
            code: 0x15,
            name: "4",
        },
        KeyMapU16 {
            code: 0x17,
            name: "5",
        },
        KeyMapU16 {
            code: 0x16,
            name: "6",
        },
        KeyMapU16 {
            code: 0x1a,
            name: "7",
        },
        KeyMapU16 {
            code: 0x1c,
            name: "8",
        },
        KeyMapU16 {
            code: 0x19,
            name: "9",
        },
        KeyMapU16 {
            code: 0x1d,
            name: "0",
        },
        KeyMapU16 {
            code: 0x1b,
            name: "Minus",
        },
        KeyMapU16 {
            code: 0x18,
            name: "Equal",
        },
        KeyMapU16 {
            code: 0x33,
            name: "Backspace",
        },
        KeyMapU16 {
            code: 0x30,
            name: "Tab",
        },
        KeyMapU16 {
            code: 0x0c,
            name: "Q",
        },
        KeyMapU16 {
            code: 0x0d,
            name: "W",
        },
        KeyMapU16 {
            code: 0x0e,
            name: "E",
        },
        KeyMapU16 {
            code: 0x0f,
            name: "R",
        },
        KeyMapU16 {
            code: 0x11,
            name: "T",
        },
        KeyMapU16 {
            code: 0x10,
            name: "Y",
        },
        KeyMapU16 {
            code: 0x20,
            name: "U",
        },
        KeyMapU16 {
            code: 0x22,
            name: "I",
        },
        KeyMapU16 {
            code: 0x1f,
            name: "O",
        },
        KeyMapU16 {
            code: 0x23,
            name: "P",
        },
        KeyMapU16 {
            code: 0x21,
            name: "BracketLeft",
        },
        KeyMapU16 {
            code: 0x1e,
            name: "BracketRight",
        },
        KeyMapU16 {
            code: 0x2a,
            name: "Backslash",
        },
        KeyMapU16 {
            code: 0x0a,
            name: "IntlBackslash",
        },
        KeyMapU16 {
            code: 0x39,
            name: "CapsLock",
        },
        KeyMapU16 {
            code: 0x00,
            name: "A",
        },
        KeyMapU16 {
            code: 0x01,
            name: "S",
        },
        KeyMapU16 {
            code: 0x02,
            name: "D",
        },
        KeyMapU16 {
            code: 0x03,
            name: "F",
        },
        KeyMapU16 {
            code: 0x05,
            name: "G",
        },
        KeyMapU16 {
            code: 0x04,
            name: "H",
        },
        KeyMapU16 {
            code: 0x26,
            name: "J",
        },
        KeyMapU16 {
            code: 0x28,
            name: "K",
        },
        KeyMapU16 {
            code: 0x25,
            name: "L",
        },
        KeyMapU16 {
            code: 0x29,
            name: "Semicolon",
        },
        KeyMapU16 {
            code: 0x27,
            name: "Quote",
        },
        KeyMapU16 {
            code: 0x24,
            name: "Enter",
        },
        KeyMapU16 {
            code: 0x38,
            name: "ShiftLeft",
        },
        KeyMapU16 {
            code: 0x06,
            name: "Z",
        },
        KeyMapU16 {
            code: 0x07,
            name: "X",
        },
        KeyMapU16 {
            code: 0x08,
            name: "C",
        },
        KeyMapU16 {
            code: 0x09,
            name: "V",
        },
        KeyMapU16 {
            code: 0x0b,
            name: "B",
        },
        KeyMapU16 {
            code: 0x2d,
            name: "N",
        },
        KeyMapU16 {
            code: 0x2e,
            name: "M",
        },
        KeyMapU16 {
            code: 0x2b,
            name: "Comma",
        },
        KeyMapU16 {
            code: 0x2f,
            name: "Period",
        },
        KeyMapU16 {
            code: 0x2c,
            name: "Slash",
        },
        KeyMapU16 {
            code: 0x3c,
            name: "ShiftRight",
        },
        KeyMapU16 {
            code: 0x3b,
            name: "ControlLeft",
        },
        KeyMapU16 {
            code: 0x37,
            name: "MetaLeft",
        },
        KeyMapU16 {
            code: 0x3a,
            name: "AltLeft",
        },
        KeyMapU16 {
            code: 0x31,
            name: "Space",
        },
        KeyMapU16 {
            code: 0x3d,
            name: "AltRight",
        },
        KeyMapU16 {
            code: 0x36,
            name: "MetaRight",
        },
        KeyMapU16 {
            code: 0x3e,
            name: "ControlRight",
        },
        KeyMapU16 {
            code: 0x41,
            name: "NumpadDecimal",
        },
        KeyMapU16 {
            code: 0x43,
            name: "NumpadMultiply",
        },
        KeyMapU16 {
            code: 0x45,
            name: "NumpadAdd",
        },
        KeyMapU16 {
            code: 0x47,
            name: "NumLock",
        },
        KeyMapU16 {
            code: 0x4b,
            name: "NumpadDivide",
        },
        KeyMapU16 {
            code: 0x4c,
            name: "NumpadEnter",
        },
        KeyMapU16 {
            code: 0x4e,
            name: "NumpadSubtract",
        },
        KeyMapU16 {
            code: 0x51,
            name: "NumpadEqual",
        },
        KeyMapU16 {
            code: 0x52,
            name: "Numpad0",
        },
        KeyMapU16 {
            code: 0x53,
            name: "Numpad1",
        },
        KeyMapU16 {
            code: 0x54,
            name: "Numpad2",
        },
        KeyMapU16 {
            code: 0x55,
            name: "Numpad3",
        },
        KeyMapU16 {
            code: 0x56,
            name: "Numpad4",
        },
        KeyMapU16 {
            code: 0x57,
            name: "Numpad5",
        },
        KeyMapU16 {
            code: 0x58,
            name: "Numpad6",
        },
        KeyMapU16 {
            code: 0x59,
            name: "Numpad7",
        },
        KeyMapU16 {
            code: 0x5b,
            name: "Numpad8",
        },
        KeyMapU16 {
            code: 0x5c,
            name: "Numpad9",
        },
        KeyMapU16 {
            code: 0x5d,
            name: "IntlYen",
        },
        KeyMapU16 {
            code: 0x5e,
            name: "IntlRo",
        },
        KeyMapU16 {
            code: 0x5f,
            name: "NumpadComma",
        },
        KeyMapU16 {
            code: 0x66,
            name: "Lang2",
        },
        KeyMapU16 {
            code: 0x68,
            name: "Lang1",
        },
        KeyMapU16 {
            code: 0x7b,
            name: "ArrowLeft",
        },
        KeyMapU16 {
            code: 0x7e,
            name: "ArrowUp",
        },
        KeyMapU16 {
            code: 0x7c,
            name: "ArrowRight",
        },
        KeyMapU16 {
            code: 0x7d,
            name: "ArrowDown",
        },
        KeyMapU16 {
            code: 0x72,
            name: "Insert",
        },
        KeyMapU16 {
            code: 0x75,
            name: "Delete",
        },
        KeyMapU16 {
            code: 0x73,
            name: "Home",
        },
        KeyMapU16 {
            code: 0x77,
            name: "End",
        },
        KeyMapU16 {
            code: 0x74,
            name: "PageUp",
        },
        KeyMapU16 {
            code: 0x79,
            name: "PageDown",
        },
    ];

    pub fn keycode_to_name(keycode: u16) -> Option<&'static str> {
        KEYCODE_TABLE
            .iter()
            .find(|entry| entry.code == keycode)
            .map(|entry| entry.name)
    }
}

pub fn fallback_name(prefix_value: impl std::fmt::Display) -> String {
    format!("Key{prefix_value}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_common_letters_map_to_single_letter_names() {
        assert_eq!(Some("A"), linux_x11::keysym_to_name(0x0061));
        assert_eq!(Some("M"), linux_x11::keysym_to_name(0x006d));
        assert_eq!(Some("Z"), linux_x11::keysym_to_name(0x007a));
    }

    #[test]
    fn linux_modifier_keysyms_produce_side_distinguished_names() {
        assert_eq!(Some("ShiftLeft"), linux_x11::keysym_to_name(0xffe1));
        assert_eq!(Some("ShiftRight"), linux_x11::keysym_to_name(0xffe2));
        assert_eq!(Some("ControlLeft"), linux_x11::keysym_to_name(0xffe3));
        assert_eq!(Some("AltLeft"), linux_x11::keysym_to_name(0xffe9));
        assert_eq!(Some("MetaLeft"), linux_x11::keysym_to_name(0xffeb));
    }

    #[test]
    fn linux_function_keys_map_across_f1_to_f12() {
        assert_eq!(Some("F1"), linux_x11::keysym_to_name(0xffbe));
        assert_eq!(Some("F12"), linux_x11::keysym_to_name(0xffc9));
        assert_eq!(Some("Pause"), linux_x11::keysym_to_name(0xff13));
        assert_eq!(Some("F13"), linux_x11::keysym_to_name(0xffca));
        assert_eq!(Some("Numpad0"), linux_x11::keysym_to_name(0xffb0));
        assert_eq!(
            Some("AudioVolumeMute"),
            linux_x11::keysym_to_name(0x1008ff12)
        );
        assert_eq!(Some("LaunchApp1"), linux_x11::keysym_to_name(0x1008ff41));
    }

    #[test]
    fn linux_arrows_and_editing_keys_round_trip() {
        assert_eq!(Some("ArrowLeft"), linux_x11::keysym_to_name(0xff51));
        assert_eq!(Some("PageDown"), linux_x11::keysym_to_name(0xff56));
        assert_eq!(Some("Delete"), linux_x11::keysym_to_name(0xffff));
    }

    #[test]
    fn linux_unknown_keysym_falls_back_to_key_number() {
        assert_eq!(None, linux_x11::keysym_to_name(0x12345));
        assert_eq!("Key74565", fallback_name(0x12345_u32));
    }

    #[test]
    fn linux_no_two_distinct_names_accidentally_share_a_keysym() {
        for (i, left) in linux_x11::KEYSYM_TABLE.iter().enumerate() {
            for right in &linux_x11::KEYSYM_TABLE[i + 1..] {
                if left.code == right.code {
                    assert_eq!(left.name, right.name);
                }
            }
        }
    }

    #[test]
    fn windows_ascii_letters_use_vk_mapping() {
        assert_eq!(Some("A"), windows::vk_to_name(0x41));
        assert_eq!(Some("Z"), windows::vk_to_name(0x5a));
        assert_eq!(Some("M"), windows::vk_to_name(0x4d));
    }

    #[test]
    fn windows_modifiers_map_to_side_distinguished_names() {
        assert_eq!(Some("ShiftLeft"), windows::vk_to_name(0xa0));
        assert_eq!(Some("ShiftRight"), windows::vk_to_name(0xa1));
        assert_eq!(Some("ControlLeft"), windows::vk_to_name(0xa2));
        assert_eq!(Some("MetaLeft"), windows::vk_to_name(0x5b));
        assert_eq!(Some("AltLeft"), windows::vk_to_name(0xa4));
    }

    #[test]
    fn windows_function_keys_cover_f1_to_f12() {
        assert_eq!(Some("F1"), windows::vk_to_name(0x70));
        assert_eq!(Some("F12"), windows::vk_to_name(0x7b));
        assert_eq!(Some("F13"), windows::vk_to_name(0x7c));
        assert_eq!(Some("Pause"), windows::vk_to_name(0x13));
    }

    #[test]
    fn windows_special_numpad_and_media_keys_map() {
        assert_eq!(Some("PrintScreen"), windows::vk_to_name(0x2c));
        assert_eq!(Some("Numpad0"), windows::vk_to_name(0x60));
        assert_eq!(Some("NumpadDivide"), windows::vk_to_name(0x6f));
        assert_eq!(Some("NumpadEqual"), windows::vk_to_name(0x92));
        assert_eq!(Some("AudioVolumeMute"), windows::vk_to_name(0xad));
        assert_eq!(Some("BrowserBack"), windows::vk_to_name(0xa6));
        assert_eq!(Some("LaunchMail"), windows::vk_to_name(0xb4));
        assert_eq!(Some("KanaMode"), windows::vk_to_name(0x15));
    }

    #[test]
    fn windows_arrows_and_editing_keys_map() {
        assert_eq!(Some("ArrowLeft"), windows::vk_to_name(0x25));
        assert_eq!(Some("PageDown"), windows::vk_to_name(0x22));
        assert_eq!(Some("Delete"), windows::vk_to_name(0x2e));
    }

    #[test]
    fn windows_unknown_vk_falls_back_to_key_number() {
        assert_eq!(None, windows::vk_to_name(0x0fff));
        assert_eq!("Key291", fallback_name(0x123_u16));
    }

    #[test]
    fn macos_letters_use_apple_hid_positions() {
        assert_eq!(Some("A"), macos::keycode_to_name(0x00));
        assert_eq!(Some("Z"), macos::keycode_to_name(0x06));
        assert_eq!(Some("M"), macos::keycode_to_name(0x2e));
    }

    #[test]
    fn macos_modifiers_map_to_side_distinguished_names() {
        assert_eq!(Some("ShiftLeft"), macos::keycode_to_name(0x38));
        assert_eq!(Some("ShiftRight"), macos::keycode_to_name(0x3c));
        assert_eq!(Some("MetaLeft"), macos::keycode_to_name(0x37));
        assert_eq!(Some("AltLeft"), macos::keycode_to_name(0x3a));
    }

    #[test]
    fn macos_function_keys_cover_f1_to_f20() {
        assert_eq!(Some("F1"), macos::keycode_to_name(0x7a));
        assert_eq!(Some("F12"), macos::keycode_to_name(0x6f));
        assert_eq!(Some("F13"), macos::keycode_to_name(0x69));
        assert_eq!(Some("F20"), macos::keycode_to_name(0x5a));
    }

    #[test]
    fn macos_numpad_and_international_keys_map() {
        assert_eq!(Some("Numpad0"), macos::keycode_to_name(0x52));
        assert_eq!(Some("NumpadEnter"), macos::keycode_to_name(0x4c));
        assert_eq!(Some("NumpadEqual"), macos::keycode_to_name(0x51));
        assert_eq!(Some("NumLock"), macos::keycode_to_name(0x47));
        assert_eq!(Some("IntlYen"), macos::keycode_to_name(0x5d));
        assert_eq!(Some("Lang1"), macos::keycode_to_name(0x68));
    }

    #[test]
    fn macos_arrows_and_editing_keys_map() {
        assert_eq!(Some("ArrowLeft"), macos::keycode_to_name(0x7b));
        assert_eq!(Some("ArrowDown"), macos::keycode_to_name(0x7d));
        assert_eq!(Some("Delete"), macos::keycode_to_name(0x75));
    }

    #[test]
    fn macos_unknown_keycode_falls_back_to_key_number() {
        assert_eq!(None, macos::keycode_to_name(0x0fff));
        assert_eq!("Key291", fallback_name(0x123_u16));
    }
}
