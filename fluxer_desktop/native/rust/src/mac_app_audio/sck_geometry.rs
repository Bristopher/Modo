// SPDX-License-Identifier: AGPL-3.0-or-later

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CGSize {
    pub width: f64,
    pub height: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CGRect {
    pub origin: CGPoint,
    pub size: CGSize,
}

impl CGRect {
    pub fn standardized(mut self) -> Self {
        if self.size.width < 0.0 {
            self.origin.x += self.size.width;
            self.size.width = -self.size.width;
        }
        if self.size.height < 0.0 {
            self.origin.y += self.size.height;
            self.size.height = -self.size.height;
        }
        self
    }

    pub fn intersection_area(a_raw: Self, b_raw: Self) -> f64 {
        let a = a_raw.standardized();
        let b = b_raw.standardized();
        if a.size.width <= 0.0
            || a.size.height <= 0.0
            || b.size.width <= 0.0
            || b.size.height <= 0.0
        {
            return 0.0;
        }
        let ax2 = a.origin.x + a.size.width;
        let ay2 = a.origin.y + a.size.height;
        let bx2 = b.origin.x + b.size.width;
        let by2 = b.origin.y + b.size.height;
        let x1 = a.origin.x.max(b.origin.x);
        let y1 = a.origin.y.max(b.origin.y);
        let x2 = ax2.min(bx2);
        let y2 = ay2.min(by2);
        if x2 <= x1 || y2 <= y1 {
            return 0.0;
        }
        (x2 - x1) * (y2 - y1)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CMTime {
    pub value: i64,
    pub timescale: i32,
    pub flags: u32,
    pub epoch: i64,
}

impl CMTime {
    pub fn seconds(value: i64, timescale: i32) -> Self {
        Self {
            value,
            timescale,
            flags: 1,
            epoch: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cgrect_intersection_handles_negative_and_disjoint_rectangles() {
        let a = CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize {
                width: 10.0,
                height: 10.0,
            },
        };
        let b = CGRect {
            origin: CGPoint { x: 5.0, y: 5.0 },
            size: CGSize {
                width: 10.0,
                height: 10.0,
            },
        };
        assert_eq!(25.0, CGRect::intersection_area(a, b));

        let c = CGRect {
            origin: CGPoint { x: 10.0, y: 10.0 },
            size: CGSize {
                width: -5.0,
                height: -5.0,
            },
        };
        assert_eq!(25.0, CGRect::intersection_area(a, c));

        let d = CGRect {
            origin: CGPoint { x: 20.0, y: 20.0 },
            size: CGSize {
                width: 2.0,
                height: 2.0,
            },
        };
        assert_eq!(0.0, CGRect::intersection_area(a, d));
    }

    #[test]
    fn core_graphics_struct_layouts_match_64_bit_darwin_abi() {
        assert_eq!(16, std::mem::size_of::<CGPoint>());
        assert_eq!(16, std::mem::size_of::<CGSize>());
        assert_eq!(32, std::mem::size_of::<CGRect>());
        assert_eq!(24, std::mem::size_of::<CMTime>());
        assert_eq!(16, std::mem::offset_of!(CMTime, epoch));
    }
}
