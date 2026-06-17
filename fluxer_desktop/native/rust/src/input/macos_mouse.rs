// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CgEventType {
    LeftMouseDown = 1,
    LeftMouseUp = 2,
    RightMouseDown = 3,
    RightMouseUp = 4,
    MouseMoved = 5,
    LeftMouseDragged = 6,
    RightMouseDragged = 7,
    KeyDown = 10,
    KeyUp = 11,
    FlagsChanged = 12,
    ScrollWheel = 22,
    OtherMouseDown = 25,
    OtherMouseUp = 26,
    OtherMouseDragged = 27,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    Button(u8),
    Ignored,
}

pub fn classify(event_type: CgEventType, other_button: u32) -> Classification {
    match event_type {
        CgEventType::LeftMouseDown | CgEventType::LeftMouseUp => Classification::Button(0),
        CgEventType::RightMouseDown | CgEventType::RightMouseUp => Classification::Button(2),
        CgEventType::OtherMouseDown | CgEventType::OtherMouseUp => match other_button {
            2 => Classification::Button(1),
            3 => Classification::Button(3),
            4 => Classification::Button(4),
            _ => Classification::Ignored,
        },
        _ => Classification::Ignored,
    }
}

pub fn is_down(event_type: CgEventType) -> bool {
    matches!(
        event_type,
        CgEventType::LeftMouseDown | CgEventType::RightMouseDown | CgEventType::OtherMouseDown
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_and_right_buttons_map_to_0_and_2() {
        assert_eq!(
            Classification::Button(0),
            classify(CgEventType::LeftMouseDown, 0)
        );
        assert_eq!(
            Classification::Button(2),
            classify(CgEventType::RightMouseUp, 0)
        );
    }

    #[test]
    fn middle_button_maps_to_1() {
        assert_eq!(
            Classification::Button(1),
            classify(CgEventType::OtherMouseDown, 2)
        );
    }

    #[test]
    fn back_forward_map_to_3_and_4() {
        assert_eq!(
            Classification::Button(3),
            classify(CgEventType::OtherMouseDown, 3)
        );
        assert_eq!(
            Classification::Button(4),
            classify(CgEventType::OtherMouseUp, 4)
        );
    }

    #[test]
    fn unknown_other_button_is_ignored_not_silently_misrouted() {
        assert_eq!(
            Classification::Ignored,
            classify(CgEventType::OtherMouseDown, 99)
        );
    }

    #[test]
    fn is_down_distinguishes_press_from_release() {
        assert!(is_down(CgEventType::LeftMouseDown));
        assert!(!is_down(CgEventType::LeftMouseUp));
        assert!(is_down(CgEventType::OtherMouseDown));
        assert!(!is_down(CgEventType::MouseMoved));
    }
}
