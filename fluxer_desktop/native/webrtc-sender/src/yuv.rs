// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct I420 {
    pub width: u32,
    pub height: u32,
    pub y: Vec<u8>,
    pub u: Vec<u8>,
    pub v: Vec<u8>,
}

impl I420 {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        if !dims_ok(width, height) {
            return None;
        }
        let w = width as usize;
        let h = height as usize;
        Some(Self {
            width,
            height,
            y: vec![0u8; w * h],
            u: vec![0u8; (w / 2) * (h / 2)],
            v: vec![0u8; (w / 2) * (h / 2)],
        })
    }

    fn has_layout(&self, width: u32, height: u32) -> bool {
        if self.width != width || self.height != height {
            return false;
        }
        let w = width as usize;
        let h = height as usize;
        self.y.len() == w * h && self.u.len() == (w / 2) * (h / 2) && self.v.len() == self.u.len()
    }
}

pub fn tight_i420_byte_len(width: u32, height: u32) -> Option<usize> {
    if !dims_ok(width, height) {
        return None;
    }
    let w = width as usize;
    let h = height as usize;
    let y_len = w.checked_mul(h)?;
    let chroma_len = (w / 2).checked_mul(h / 2)?;
    y_len.checked_add(chroma_len.checked_mul(2)?)
}

pub fn copy_tight_i420_into(src: &[u8], width: u32, height: u32, dst: &mut I420) -> bool {
    if !dst.has_layout(width, height) {
        return false;
    }
    let Some(total_len) = tight_i420_byte_len(width, height) else {
        return false;
    };
    if src.len() != total_len {
        return false;
    }
    let y_len = (width as usize) * (height as usize);
    let chroma_len = y_len / 4;
    dst.y.copy_from_slice(&src[..y_len]);
    dst.u.copy_from_slice(&src[y_len..y_len + chroma_len]);
    dst.v.copy_from_slice(&src[y_len + chroma_len..]);
    true
}

fn dims_ok(width: u32, height: u32) -> bool {
    width >= 2 && height >= 2 && width.is_multiple_of(2) && height.is_multiple_of(2)
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn nv12_to_i420(
    src: &[u8],
    width: u32,
    height: u32,
    stride_y: u32,
    stride_uv: u32,
) -> Option<I420> {
    let mut dst = I420::new(width, height)?;
    if !nv12_to_i420_into(src, width, height, stride_y, stride_uv, &mut dst) {
        return None;
    }
    Some(dst)
}

pub fn nv12_to_i420_into(
    src: &[u8],
    width: u32,
    height: u32,
    stride_y: u32,
    stride_uv: u32,
    dst: &mut I420,
) -> bool {
    if !dims_ok(width, height) {
        return false;
    }
    if !dst.has_layout(width, height) {
        return false;
    }
    let w = width as usize;
    let h = height as usize;
    let cw = w / 2;
    let ch = h / 2;
    let sy = stride_y.max(width) as usize;
    let suv = stride_uv.max(width) as usize;

    let Some(uv_offset) = sy.checked_mul(h) else {
        return false;
    };
    let Some(uv_len) = suv.checked_mul(ch) else {
        return false;
    };
    let Some(needed) = uv_offset.checked_add(uv_len) else {
        return false;
    };
    if src.len() < needed {
        return false;
    }

    for row in 0..h {
        let s = row * sy;
        dst.y[row * w..row * w + w].copy_from_slice(&src[s..s + w]);
    }

    for row in 0..ch {
        let base = uv_offset + row * suv;
        for x in 0..cw {
            dst.u[row * cw + x] = src[base + 2 * x];
            dst.v[row * cw + x] = src[base + 2 * x + 1];
        }
    }

    true
}

#[allow(clippy::too_many_arguments)]
pub fn copy_nv12_planes(
    src: &[u8],
    width: u32,
    height: u32,
    stride_y: u32,
    stride_uv: u32,
    dst_y: &mut [u8],
    dst_uv: &mut [u8],
    dst_stride_y: u32,
    dst_stride_uv: u32,
) -> bool {
    if !dims_ok(width, height) {
        return false;
    }
    let w = width as usize;
    let h = height as usize;
    let ch = h / 2;
    let sy = stride_y.max(width) as usize;
    let suv = stride_uv.max(width) as usize;
    let dsy = dst_stride_y as usize;
    let dsuv = dst_stride_uv as usize;
    if dsy < w || dsuv < w {
        return false;
    }

    let Some(uv_offset) = sy.checked_mul(h) else {
        return false;
    };
    let Some(uv_len) = suv.checked_mul(ch) else {
        return false;
    };
    let Some(needed) = uv_offset.checked_add(uv_len) else {
        return false;
    };
    if src.len() < needed || dst_y.len() < dsy * h || dst_uv.len() < dsuv * ch {
        return false;
    }

    for row in 0..h {
        let s = row * sy;
        let d = row * dsy;
        dst_y[d..d + w].copy_from_slice(&src[s..s + w]);
    }
    for row in 0..ch {
        let s = uv_offset + row * suv;
        let d = row * dsuv;
        dst_uv[d..d + w].copy_from_slice(&src[s..s + w]);
    }
    true
}

#[cfg(test)]
pub fn yuyv_to_i420(src: &[u8], width: u32, height: u32, stride: u32) -> Option<I420> {
    let mut dst = I420::new(width, height)?;
    if !yuyv_to_i420_into(src, width, height, stride, &mut dst) {
        return None;
    }
    Some(dst)
}

pub fn yuyv_to_i420_into(src: &[u8], width: u32, height: u32, stride: u32, dst: &mut I420) -> bool {
    if !dims_ok(width, height) {
        return false;
    }
    if !dst.has_layout(width, height) {
        return false;
    }
    let w = width as usize;
    let h = height as usize;
    let cw = w / 2;
    let ch = h / 2;
    let stride = stride.max(width * 2) as usize;
    if src.len() < stride * h {
        return false;
    }

    for row in 0..h {
        let row_base = row * stride;
        for pair in 0..cw {
            let src_offset = row_base + pair * 4;
            let dst_offset = row * w + pair * 2;
            dst.y[dst_offset] = src[src_offset];
            dst.y[dst_offset + 1] = src[src_offset + 2];
        }
    }
    for cy in 0..ch {
        for cx in 0..cw {
            let top = (cy * 2) * stride + cx * 4;
            let bottom = (cy * 2 + 1) * stride + cx * 4;
            dst.u[cy * cw + cx] =
                ((u16::from(src[top + 1]) + u16::from(src[bottom + 1])) / 2) as u8;
            dst.v[cy * cw + cx] =
                ((u16::from(src[top + 3]) + u16::from(src[bottom + 3])) / 2) as u8;
        }
    }

    true
}

fn clamp_u8(value: i32) -> u8 {
    value.clamp(0, 255) as u8
}

fn rgb_to_y(r: i32, g: i32, b: i32) -> u8 {
    clamp_u8(((66 * r + 129 * g + 25 * b + 128) >> 8) + 16)
}
fn rgb_to_u(r: i32, g: i32, b: i32) -> u8 {
    clamp_u8(((-38 * r - 74 * g + 112 * b + 128) >> 8) + 128)
}
fn rgb_to_v(r: i32, g: i32, b: i32) -> u8 {
    clamp_u8(((112 * r - 94 * g - 18 * b + 128) >> 8) + 128)
}

#[cfg(test)]
pub fn bgra_to_i420(src: &[u8], width: u32, height: u32, stride: u32) -> Option<I420> {
    let mut dst = I420::new(width, height)?;
    if !bgra_to_i420_planes(
        src,
        width,
        height,
        stride,
        &mut dst.y,
        &mut dst.u,
        &mut dst.v,
        width,
        width / 2,
        width / 2,
    ) {
        return None;
    }
    Some(dst)
}

#[allow(clippy::too_many_arguments)]
pub fn bgra_to_i420_planes(
    src: &[u8],
    width: u32,
    height: u32,
    stride: u32,
    dst_y: &mut [u8],
    dst_u: &mut [u8],
    dst_v: &mut [u8],
    dst_stride_y: u32,
    dst_stride_u: u32,
    dst_stride_v: u32,
) -> bool {
    if !dims_ok(width, height) {
        return false;
    }
    let w = width as usize;
    let h = height as usize;
    let cw = w / 2;
    let ch = h / 2;
    let stride = stride.max(width * 4) as usize;
    if src.len() < stride * h {
        return false;
    }
    let dsy = dst_stride_y as usize;
    let dsu = dst_stride_u as usize;
    let dsv = dst_stride_v as usize;
    if dsy < w || dsu < cw || dsv < cw {
        return false;
    }
    if dst_y.len() < dsy * h || dst_u.len() < dsu * ch || dst_v.len() < dsv * ch {
        return false;
    }

    let px = |row: usize, col: usize| -> (i32, i32, i32) {
        let o = row * stride + col * 4;
        let b = src[o] as i32;
        let g = src[o + 1] as i32;
        let r = src[o + 2] as i32;
        (r, g, b)
    };

    for row in 0..h {
        for col in 0..w {
            let (r, g, b) = px(row, col);
            dst_y[row * dsy + col] = rgb_to_y(r, g, b);
        }
    }
    for cy in 0..ch {
        for cx in 0..cw {
            let mut rs = 0;
            let mut gs = 0;
            let mut bs = 0;
            for dy in 0..2 {
                for dx in 0..2 {
                    let (r, g, b) = px(cy * 2 + dy, cx * 2 + dx);
                    rs += r;
                    gs += g;
                    bs += b;
                }
            }
            let (r, g, b) = (rs / 4, gs / 4, bs / 4);
            dst_u[cy * dsu + cx] = rgb_to_u(r, g, b);
            dst_v[cy * dsv + cx] = rgb_to_v(r, g, b);
        }
    }

    true
}

#[cfg_attr(not(feature = "camera-native"), allow(dead_code))]
pub fn rgb_to_i420(src: &[u8], width: u32, height: u32) -> Option<I420> {
    let mut dst = I420::new(width, height)?;
    if !rgb_to_i420_into(src, width, height, &mut dst) {
        return None;
    }
    Some(dst)
}

pub fn rgb_to_i420_into(src: &[u8], width: u32, height: u32, dst: &mut I420) -> bool {
    if !dims_ok(width, height) {
        return false;
    }
    if !dst.has_layout(width, height) {
        return false;
    }
    let w = width as usize;
    let h = height as usize;
    let cw = w / 2;
    let ch = h / 2;
    let stride = w * 3;
    if src.len() < stride * h {
        return false;
    }

    let px = |row: usize, col: usize| -> (i32, i32, i32) {
        let o = row * stride + col * 3;
        let r = src[o] as i32;
        let g = src[o + 1] as i32;
        let b = src[o + 2] as i32;
        (r, g, b)
    };

    for row in 0..h {
        for col in 0..w {
            let (r, g, b) = px(row, col);
            dst.y[row * w + col] = rgb_to_y(r, g, b);
        }
    }
    for cy in 0..ch {
        for cx in 0..cw {
            let mut rs = 0;
            let mut gs = 0;
            let mut bs = 0;
            for dy in 0..2 {
                for dx in 0..2 {
                    let (r, g, b) = px(cy * 2 + dy, cx * 2 + dx);
                    rs += r;
                    gs += g;
                    bs += b;
                }
            }
            let (r, g, b) = (rs / 4, gs / 4, bs / 4);
            dst.u[cy * cw + cx] = rgb_to_u(r, g, b);
            dst.v[cy * cw + cx] = rgb_to_v(r, g, b);
        }
    }

    true
}

#[cfg_attr(not(feature = "camera-native"), allow(dead_code))]
pub fn bgr_to_i420_into(src: &[u8], width: u32, height: u32, dst: &mut I420) -> bool {
    if !dims_ok(width, height) {
        return false;
    }
    if !dst.has_layout(width, height) {
        return false;
    }
    let w = width as usize;
    let h = height as usize;
    let cw = w / 2;
    let ch = h / 2;
    let stride = w * 3;
    if src.len() < stride * h {
        return false;
    }

    let px = |row: usize, col: usize| -> (i32, i32, i32) {
        let o = row * stride + col * 3;
        let b = src[o] as i32;
        let g = src[o + 1] as i32;
        let r = src[o + 2] as i32;
        (r, g, b)
    };

    for row in 0..h {
        for col in 0..w {
            let (r, g, b) = px(row, col);
            dst.y[row * w + col] = rgb_to_y(r, g, b);
        }
    }
    for cy in 0..ch {
        for cx in 0..cw {
            let mut rs = 0;
            let mut gs = 0;
            let mut bs = 0;
            for dy in 0..2 {
                for dx in 0..2 {
                    let (r, g, b) = px(cy * 2 + dy, cx * 2 + dx);
                    rs += r;
                    gs += g;
                    bs += b;
                }
            }
            let (r, g, b) = (rs / 4, gs / 4, bs / 4);
            dst.u[cy * cw + cx] = rgb_to_u(r, g, b);
            dst.v[cy * cw + cx] = rgb_to_v(r, g, b);
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_odd_or_tiny_dimensions() {
        assert!(nv12_to_i420(&[0u8; 64], 3, 2, 3, 3).is_none());
        assert!(nv12_to_i420(&[0u8; 64], 2, 1, 2, 2).is_none());
        assert!(bgra_to_i420(&[0u8; 256], 2, 3, 8).is_none());
        assert!(bgra_to_i420(&[0u8; 256], 0, 2, 0).is_none());
    }

    #[test]
    fn nv12_short_buffer_is_rejected() {
        assert!(nv12_to_i420(&[0u8; 5], 2, 2, 2, 2).is_none());
        assert!(nv12_to_i420(&[0u8; 6], 2, 2, 2, 2).is_some());
    }

    #[test]
    fn nv12_packed_2x2_deinterleaves() {
        let src = [1u8, 2, 3, 4, 10, 20];
        let out = nv12_to_i420(&src, 2, 2, 2, 2).unwrap();
        assert_eq!(out.y, vec![1, 2, 3, 4]);
        assert_eq!(out.u, vec![10]);
        assert_eq!(out.v, vec![20]);
        assert_eq!((out.width / 2, out.height / 2), (1, 1));
    }

    #[test]
    fn yuyv_2x2_deinterleaves_and_vertically_averages_chroma() {
        let src = [1u8, 10, 2, 20, 3, 30, 4, 40];
        let out = yuyv_to_i420(&src, 2, 2, 4).unwrap();

        assert_eq!(out.y, vec![1, 2, 3, 4]);
        assert_eq!(out.u, vec![20]);
        assert_eq!(out.v, vec![30]);
    }

    #[test]
    fn yuyv_respects_row_padding() {
        let src = [1u8, 10, 2, 20, 99, 99, 3, 30, 4, 40, 88, 88];
        let out = yuyv_to_i420(&src, 2, 2, 6).unwrap();

        assert_eq!(out.y, vec![1, 2, 3, 4]);
        assert_eq!(out.u, vec![20]);
        assert_eq!(out.v, vec![30]);
    }

    #[test]
    fn nv12_4x4_deinterleaves_two_chroma_columns() {
        let mut src = Vec::new();
        src.extend(0u8..16);
        src.extend([100, 101, 102, 103, 104, 105, 106, 107]);
        let out = nv12_to_i420(&src, 4, 4, 4, 4).unwrap();
        assert_eq!(out.y, (0u8..16).collect::<Vec<_>>());
        assert_eq!(out.u, vec![100, 102, 104, 106]);
        assert_eq!(out.v, vec![101, 103, 105, 107]);
    }

    #[test]
    fn nv12_respects_row_padding() {
        let src = [1u8, 2, 0xFF, 0xFF, 3, 4, 0xFF, 0xFF, 10, 20, 0xFF, 0xFF];
        let out = nv12_to_i420(&src, 2, 2, 4, 4).unwrap();
        assert_eq!(out.y, vec![1, 2, 3, 4]);
        assert_eq!(out.u, vec![10]);
        assert_eq!(out.v, vec![20]);
    }

    #[test]
    fn copy_nv12_planes_preserves_nv12_layout() {
        let src = [1u8, 2, 3, 4, 10, 20];
        let mut y = [0u8; 4];
        let mut uv = [0u8; 2];
        assert!(copy_nv12_planes(&src, 2, 2, 2, 2, &mut y, &mut uv, 2, 2));
        assert_eq!(y, [1, 2, 3, 4]);
        assert_eq!(uv, [10, 20]);
    }

    #[test]
    fn copy_nv12_planes_respects_destination_stride() {
        let src = [1u8, 2, 0xFF, 0xFF, 3, 4, 0xFF, 0xFF, 10, 20, 0xFF, 0xFF];
        let mut y = [0u8; 8];
        let mut uv = [0u8; 4];
        assert!(copy_nv12_planes(&src, 2, 2, 4, 4, &mut y, &mut uv, 4, 4));
        assert_eq!(y, [1, 2, 0, 0, 3, 4, 0, 0]);
        assert_eq!(uv, [10, 20, 0, 0]);
    }

    #[test]
    fn copy_nv12_planes_rejects_short_buffers() {
        let src = [0u8; 6];
        let mut y = [0u8; 3];
        let mut uv = [0u8; 2];
        assert!(!copy_nv12_planes(&src, 2, 2, 2, 2, &mut y, &mut uv, 2, 2));
        let mut y = [0u8; 4];
        assert!(!copy_nv12_planes(
            &src[..5],
            2,
            2,
            2,
            2,
            &mut y,
            &mut uv,
            2,
            2
        ));
    }

    fn solid_bgra(width: u32, height: u32, b: u8, g: u8, r: u8) -> Vec<u8> {
        let mut v = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..(width * height) {
            v.extend([b, g, r, 255]);
        }
        v
    }

    fn near(a: u8, b: u8, tol: i32) -> bool {
        (a as i32 - b as i32).abs() <= tol
    }

    #[test]
    fn bgra_black_white_grey_levels() {
        let black = bgra_to_i420(&solid_bgra(2, 2, 0, 0, 0), 2, 2, 8).unwrap();
        assert!(near(black.y[0], 16, 1), "black Y={}", black.y[0]);
        assert!(near(black.u[0], 128, 1) && near(black.v[0], 128, 1));

        let white = bgra_to_i420(&solid_bgra(2, 2, 255, 255, 255), 2, 2, 8).unwrap();
        assert!(near(white.y[0], 235, 2), "white Y={}", white.y[0]);
        assert!(near(white.u[0], 128, 2) && near(white.v[0], 128, 2));
    }

    #[test]
    fn bgra_primaries_have_expected_chroma_signs() {
        let red = bgra_to_i420(&solid_bgra(2, 2, 0, 0, 255), 2, 2, 8).unwrap();
        assert!(red.v[0] > 200, "red V={}", red.v[0]);
        let blue = bgra_to_i420(&solid_bgra(2, 2, 255, 0, 0), 2, 2, 8).unwrap();
        assert!(blue.u[0] > 200, "blue U={}", blue.u[0]);
        let green = bgra_to_i420(&solid_bgra(2, 2, 0, 255, 0), 2, 2, 8).unwrap();
        assert!(
            green.u[0] < 60 && green.v[0] < 60,
            "green U={} V={}",
            green.u[0],
            green.v[0]
        );
    }

    #[test]
    fn bgra_plane_sizes() {
        let out = bgra_to_i420(&solid_bgra(8, 6, 10, 20, 30), 8, 6, 32).unwrap();
        assert_eq!(out.y.len(), 8 * 6);
        assert_eq!(out.u.len(), 4 * 3);
        assert_eq!(out.v.len(), 4 * 3);
    }

    #[test]
    fn bgra_golden_2x2_solid_colour_exact_bytes() {
        let src = solid_bgra(2, 2, 32, 64, 128);
        let out = bgra_to_i420(&src, 2, 2, 8).unwrap();
        assert_eq!(out.width, 2);
        assert_eq!(out.height, 2);
        assert_eq!(out.y, vec![84, 84, 84, 84]);
        assert_eq!(out.u, vec![105]);
        assert_eq!(out.v, vec![158]);
    }

    #[test]
    fn bgra_golden_strided_2x2_skips_row_padding() {
        let mut src = vec![0xFFu8; 16 * 2];
        for px in 0..2 {
            let o = px * 4;
            src[o..o + 4].copy_from_slice(&[0, 0, 0, 255]);
        }
        for px in 0..2 {
            let o = 16 + px * 4;
            src[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
        let out = bgra_to_i420(&src, 2, 2, 16).unwrap();
        assert!(
            near(out.y[0], 16, 1) && near(out.y[1], 16, 1),
            "row0 Y={:?}",
            &out.y[0..2]
        );
        assert!(
            near(out.y[2], 235, 2) && near(out.y[3], 235, 2),
            "row1 Y={:?}",
            &out.y[2..4]
        );
        assert!(
            near(out.u[0], 128, 2) && near(out.v[0], 128, 2),
            "U={} V={}",
            out.u[0],
            out.v[0]
        );
    }

    #[test]
    fn bgra_to_i420_planes_matches_tight_conversion_with_destination_padding() {
        let src = solid_bgra(4, 2, 32, 64, 128);
        let tight = bgra_to_i420(&src, 4, 2, 16).unwrap();
        let mut y = [0u8; 10];
        let mut u = [0u8; 4];
        let mut v = [0u8; 4];
        assert!(bgra_to_i420_planes(
            &src, 4, 2, 16, &mut y, &mut u, &mut v, 5, 2, 2
        ));
        assert_eq!(&y[0..4], &tight.y[0..4]);
        assert_eq!(&y[5..9], &tight.y[4..8]);
        assert_eq!(&u[0..2], &tight.u[0..2]);
        assert_eq!(&v[0..2], &tight.v[0..2]);
    }

    #[test]
    fn copy_tight_i420_into_reuses_existing_plane_storage() {
        let mut dst = I420::new(4, 2).unwrap();
        let ptrs = (dst.y.as_ptr(), dst.u.as_ptr(), dst.v.as_ptr());
        let src: Vec<u8> = (0u8..12).collect();

        assert_eq!(tight_i420_byte_len(4, 2), Some(12));
        assert!(copy_tight_i420_into(&src, 4, 2, &mut dst));

        assert_eq!(dst.y.as_ptr(), ptrs.0);
        assert_eq!(dst.u.as_ptr(), ptrs.1);
        assert_eq!(dst.v.as_ptr(), ptrs.2);
        assert_eq!(dst.y, vec![0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(dst.u, vec![8, 9]);
        assert_eq!(dst.v, vec![10, 11]);
    }

    fn solid_rgb(width: u32, height: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
        let mut v = Vec::with_capacity((width * height * 3) as usize);
        for _ in 0..(width * height) {
            v.extend([r, g, b]);
        }
        v
    }

    #[test]
    fn rgb_rejects_odd_dims_and_short_buffer() {
        assert!(rgb_to_i420(&[0u8; 64], 3, 2).is_none());
        assert!(rgb_to_i420(&[0u8; 64], 2, 1).is_none());
        assert!(rgb_to_i420(&[0u8; 11], 2, 2).is_none());
        assert!(rgb_to_i420(&[0u8; 12], 2, 2).is_some());
    }

    #[test]
    fn rgb_golden_2x2_solid_colour_matches_bgra_path() {
        let out = rgb_to_i420(&solid_rgb(2, 2, 128, 64, 32), 2, 2).unwrap();
        assert_eq!(out.width, 2);
        assert_eq!(out.height, 2);
        assert_eq!(out.y, vec![84, 84, 84, 84]);
        assert_eq!(out.u, vec![105]);
        assert_eq!(out.v, vec![158]);
    }

    #[test]
    fn rgb_plane_sizes_and_levels() {
        let black = rgb_to_i420(&solid_rgb(2, 2, 0, 0, 0), 2, 2).unwrap();
        assert!(near(black.y[0], 16, 1));
        assert!(near(black.u[0], 128, 1) && near(black.v[0], 128, 1));
        let white = rgb_to_i420(&solid_rgb(4, 4, 255, 255, 255), 4, 4).unwrap();
        assert_eq!(white.y.len(), 16);
        assert_eq!(white.u.len(), 4);
        assert_eq!(white.v.len(), 4);
        assert!(near(white.y[0], 235, 2));
    }

    #[test]
    fn nv12_golden_4x2_packed_to_i420() {
        let mut src = Vec::new();
        src.extend(0u8..8);
        src.extend([40, 41, 42, 43]);
        let out = nv12_to_i420(&src, 4, 2, 4, 4).unwrap();
        assert_eq!(out.width, 4);
        assert_eq!(out.height, 2);
        assert_eq!(out.y, (0u8..8).collect::<Vec<_>>());
        assert_eq!(out.u, vec![40, 42]);
        assert_eq!(out.v, vec![41, 43]);
    }
}
