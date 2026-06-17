// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelDirection {
    Up,
    Down,
    Left,
    Right,
}

impl WheelDirection {
    pub fn delta_x(self) -> i32 {
        match self {
            Self::Left => -120,
            Self::Right => 120,
            Self::Up | Self::Down => 0,
        }
    }

    pub fn delta_y(self) -> i32 {
        match self {
            Self::Up => -120,
            Self::Down => 120,
            Self::Left | Self::Right => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseClassification {
    Button(u8),
    Wheel(WheelDirection),
    Ignored,
}

pub fn classify(x11_button: u32) -> MouseClassification {
    match x11_button {
        1 => MouseClassification::Button(0),
        2 => MouseClassification::Button(1),
        3 => MouseClassification::Button(2),
        4 => MouseClassification::Wheel(WheelDirection::Up),
        5 => MouseClassification::Wheel(WheelDirection::Down),
        6 => MouseClassification::Wheel(WheelDirection::Left),
        7 => MouseClassification::Wheel(WheelDirection::Right),
        8 => MouseClassification::Button(3),
        9 => MouseClassification::Button(4),
        _ => MouseClassification::Ignored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_buttons_map_to_browser_indices() {
        assert_eq!(MouseClassification::Button(0), classify(1));
        assert_eq!(MouseClassification::Button(1), classify(2));
        assert_eq!(MouseClassification::Button(2), classify(3));
    }

    #[test]
    fn vertical_wheel_produces_delta_y_with_120_step() {
        assert_eq!(-120, WheelDirection::Up.delta_y());
        assert_eq!(120, WheelDirection::Down.delta_y());
        assert_eq!(0, WheelDirection::Up.delta_x());
    }

    #[test]
    fn horizontal_wheel_produces_delta_x_with_120_step() {
        assert_eq!(-120, WheelDirection::Left.delta_x());
        assert_eq!(120, WheelDirection::Right.delta_x());
        assert_eq!(0, WheelDirection::Left.delta_y());
    }

    #[test]
    fn back_forward_buttons_map_to_3_and_4() {
        assert_eq!(MouseClassification::Button(3), classify(8));
        assert_eq!(MouseClassification::Button(4), classify(9));
    }

    #[test]
    fn unknown_button_numbers_are_ignored_not_silently_misrouted() {
        assert_eq!(MouseClassification::Ignored, classify(0));
        assert_eq!(MouseClassification::Ignored, classify(15));
        assert_eq!(MouseClassification::Ignored, classify(255));
    }
}
