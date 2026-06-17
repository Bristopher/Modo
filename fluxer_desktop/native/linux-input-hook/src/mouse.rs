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
            _ => 0,
        }
    }

    pub fn delta_y(self) -> i32 {
        match self {
            Self::Up => -120,
            Self::Down => 120,
            _ => 0,
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
        assert_eq!(classify(1), MouseClassification::Button(0));
        assert_eq!(classify(2), MouseClassification::Button(1));
        assert_eq!(classify(3), MouseClassification::Button(2));
    }

    #[test]
    fn vertical_wheel_delta_y_120_step() {
        assert_eq!(classify(4), MouseClassification::Wheel(WheelDirection::Up));
        assert_eq!(WheelDirection::Up.delta_y(), -120);
        assert_eq!(WheelDirection::Down.delta_y(), 120);
        assert_eq!(WheelDirection::Up.delta_x(), 0);
    }

    #[test]
    fn horizontal_wheel_delta_x_120_step() {
        assert_eq!(WheelDirection::Left.delta_x(), -120);
        assert_eq!(WheelDirection::Right.delta_x(), 120);
        assert_eq!(WheelDirection::Left.delta_y(), 0);
    }

    #[test]
    fn back_forward_buttons_map_to_3_and_4() {
        assert_eq!(classify(8), MouseClassification::Button(3));
        assert_eq!(classify(9), MouseClassification::Button(4));
    }

    #[test]
    fn unknown_buttons_are_ignored() {
        assert_eq!(classify(0), MouseClassification::Ignored);
        assert_eq!(classify(15), MouseClassification::Ignored);
        assert_eq!(classify(255), MouseClassification::Ignored);
    }
}
