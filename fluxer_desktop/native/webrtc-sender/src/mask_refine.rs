// SPDX-License-Identifier: AGPL-3.0-or-later

const FRAME_EDGE_MAX: usize = 8192;
const MASK_REFINE_DOWNSAMPLE: usize = 4;
const GUIDED_FILTER_RADIUS_LOW: usize = 4;
const GUIDED_FILTER_EPSILON: f32 = 1e-4;
const TEMPORAL_COMBINE_RATIO: f32 = 0.7;
const TEMPORAL_UNCERTAINTY_C1: f32 = 5.68842;
const TEMPORAL_UNCERTAINTY_C2: f32 = -0.748699;
const TEMPORAL_UNCERTAINTY_C3: f32 = -57.8051;
const TEMPORAL_UNCERTAINTY_C4: f32 = 291.309;
const TEMPORAL_UNCERTAINTY_C5: f32 = -624.717;
const SHAPE_SMOOTHSTEP_EDGE_LOW: f32 = 0.55;
const SHAPE_SMOOTHSTEP_EDGE_HIGH: f32 = 0.85;
const LUT_LEN: usize = 256;

pub struct MaskRefiner {
    width: usize,
    height: usize,
    low_width: usize,
    low_height: usize,
    previous_mask: Vec<u8>,
    previous_mask_valid: bool,
    temporal_weight_lut: [u16; LUT_LEN],
    shape_lut: [u8; LUT_LEN],
    column_fixed: Vec<u32>,
    guide_low: Vec<f32>,
    mask_low: Vec<f32>,
    mean_guide: Vec<f32>,
    mean_mask: Vec<f32>,
    corr_guide_guide: Vec<f32>,
    corr_guide_mask: Vec<f32>,
    coeff_a: Vec<f32>,
    coeff_b: Vec<f32>,
    scratch: Vec<f32>,
}

impl MaskRefiner {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width >= 2);
        assert!(height >= 2);
        assert!(width <= FRAME_EDGE_MAX);
        assert!(height <= FRAME_EDGE_MAX);
        let low_width = (width / MASK_REFINE_DOWNSAMPLE).max(1);
        let low_height = (height / MASK_REFINE_DOWNSAMPLE).max(1);
        let low_len = low_width * low_height;
        let mut column_fixed = vec![0u32; width];
        for (x, slot) in column_fixed.iter_mut().enumerate() {
            *slot = bilinear_fixed_coord(x, width, low_width);
        }
        Self {
            width,
            height,
            low_width,
            low_height,
            previous_mask: vec![0; width * height],
            previous_mask_valid: false,
            temporal_weight_lut: temporal_weight_lut(),
            shape_lut: shape_lut(),
            column_fixed,
            guide_low: vec![0.0; low_len],
            mask_low: vec![0.0; low_len],
            mean_guide: vec![0.0; low_len],
            mean_mask: vec![0.0; low_len],
            corr_guide_guide: vec![0.0; low_len],
            corr_guide_mask: vec![0.0; low_len],
            coeff_a: vec![0.0; low_len],
            coeff_b: vec![0.0; low_len],
            scratch: vec![0.0; low_len],
        }
    }

    pub fn refine(&mut self, luma: &[u8], mask: &mut [u8]) {
        assert!(luma.len() >= self.width * self.height);
        assert!(mask.len() >= self.width * self.height);
        self.blend_temporal(mask);
        self.downsample(luma, mask);
        self.close_mask_low();
        self.solve_guided_coefficients();
        self.apply_guided_coefficients(luma, mask);
    }

    fn blend_temporal(&mut self, mask: &mut [u8]) {
        let len = self.width * self.height;
        assert!(mask.len() >= len);
        assert_eq!(self.previous_mask.len(), len);
        if self.previous_mask_valid {
            for (current, previous) in mask[..len].iter_mut().zip(self.previous_mask.iter()) {
                let new_value = i32::from(*current);
                let weight = i32::from(self.temporal_weight_lut[usize::from(*current)]);
                let delta = (i32::from(*previous) - new_value) * weight;
                *current = (new_value + ((delta + 128) >> 8)).clamp(0, 255) as u8;
            }
        }
        self.previous_mask.copy_from_slice(&mask[..len]);
        self.previous_mask_valid = true;
    }

    fn downsample(&mut self, luma: &[u8], mask: &[u8]) {
        assert_eq!(self.guide_low.len(), self.low_width * self.low_height);
        assert_eq!(self.mask_low.len(), self.guide_low.len());
        for low_y in 0..self.low_height {
            let y_start = low_y * MASK_REFINE_DOWNSAMPLE;
            let y_end = (y_start + MASK_REFINE_DOWNSAMPLE).min(self.height);
            for low_x in 0..self.low_width {
                let x_start = low_x * MASK_REFINE_DOWNSAMPLE;
                let x_end = (x_start + MASK_REFINE_DOWNSAMPLE).min(self.width);
                let mut guide_sum: u32 = 0;
                let mut mask_sum: u32 = 0;
                for y in y_start..y_end {
                    let row = y * self.width;
                    for x in x_start..x_end {
                        guide_sum += u32::from(luma[row + x]);
                        mask_sum += u32::from(mask[row + x]);
                    }
                }
                let count = ((y_end - y_start) * (x_end - x_start)) as f32;
                assert!(count >= 1.0);
                let low_offset = low_y * self.low_width + low_x;
                self.guide_low[low_offset] = guide_sum as f32 / (count * 255.0);
                self.mask_low[low_offset] = mask_sum as f32 / (count * 255.0);
            }
        }
    }

    fn close_mask_low(&mut self) {
        morph_pass_low(
            &self.mask_low,
            &mut self.scratch,
            &mut self.coeff_a,
            self.low_width,
            self.low_height,
            f32::max,
        );
        morph_pass_low(
            &self.coeff_a,
            &mut self.scratch,
            &mut self.mask_low,
            self.low_width,
            self.low_height,
            f32::min,
        );
    }

    fn solve_guided_coefficients(&mut self) {
        let len = self.low_width * self.low_height;
        assert_eq!(self.coeff_a.len(), len);
        assert_eq!(self.coeff_b.len(), len);
        let radius = GUIDED_FILTER_RADIUS_LOW;
        let w = self.low_width;
        let h = self.low_height;
        box_filter_low(
            &self.guide_low,
            &mut self.scratch,
            &mut self.mean_guide,
            w,
            h,
            radius,
        );
        box_filter_low(
            &self.mask_low,
            &mut self.scratch,
            &mut self.mean_mask,
            w,
            h,
            radius,
        );
        for i in 0..len {
            self.coeff_a[i] = self.guide_low[i] * self.guide_low[i];
            self.coeff_b[i] = self.guide_low[i] * self.mask_low[i];
        }
        box_filter_low(
            &self.coeff_a,
            &mut self.scratch,
            &mut self.corr_guide_guide,
            w,
            h,
            radius,
        );
        box_filter_low(
            &self.coeff_b,
            &mut self.scratch,
            &mut self.corr_guide_mask,
            w,
            h,
            radius,
        );
        for i in 0..len {
            let variance = self.corr_guide_guide[i] - self.mean_guide[i] * self.mean_guide[i];
            let covariance = self.corr_guide_mask[i] - self.mean_guide[i] * self.mean_mask[i];
            let a = covariance / (variance.max(0.0) + GUIDED_FILTER_EPSILON);
            self.coeff_a[i] = a;
            self.coeff_b[i] = self.mean_mask[i] - a * self.mean_guide[i];
        }
        box_filter_low(
            &self.coeff_a,
            &mut self.scratch,
            &mut self.mean_guide,
            w,
            h,
            radius,
        );
        box_filter_low(
            &self.coeff_b,
            &mut self.scratch,
            &mut self.mean_mask,
            w,
            h,
            radius,
        );
    }

    fn apply_guided_coefficients(&self, luma: &[u8], mask: &mut [u8]) {
        assert!(luma.len() >= self.width * self.height);
        assert!(mask.len() >= self.width * self.height);
        let low_w = self.low_width;
        for y in 0..self.height {
            let row_fixed = bilinear_fixed_coord(y, self.height, self.low_height);
            let sy = (row_fixed / 256) as usize;
            let fy = (row_fixed % 256) as f32 / 256.0;
            let sy_next = (sy + 1).min(self.low_height - 1);
            let row = y * self.width;
            for x in 0..self.width {
                let col_fixed = self.column_fixed[x];
                let sx = (col_fixed / 256) as usize;
                let fx = (col_fixed % 256) as f32 / 256.0;
                let sx_next = (sx + 1).min(low_w - 1);
                let a = bilinear_sample(&self.mean_guide, low_w, sx, sx_next, sy, sy_next, fx, fy);
                let b = bilinear_sample(&self.mean_mask, low_w, sx, sx_next, sy, sy_next, fx, fy);
                let q = a * (f32::from(luma[row + x]) / 255.0) + b;
                let shaped = (q * 255.0 + 0.5).clamp(0.0, 255.0) as usize;
                mask[row + x] = self.shape_lut[shaped.min(LUT_LEN - 1)];
            }
        }
    }
}

pub(crate) fn bilinear_fixed_coord(index: usize, full_len: usize, low_len: usize) -> u32 {
    assert!(full_len >= 1);
    assert!(low_len >= 1);
    if full_len == 1 {
        return 0;
    }
    (index * (low_len - 1) * 256 / (full_len - 1)) as u32
}

#[expect(clippy::too_many_arguments)]
pub(crate) fn bilinear_sample(
    plane: &[f32],
    width: usize,
    sx: usize,
    sx_next: usize,
    sy: usize,
    sy_next: usize,
    fx: f32,
    fy: f32,
) -> f32 {
    assert!(sy * width + sx_next < plane.len());
    assert!(sy_next * width + sx_next < plane.len());
    let top = plane[sy * width + sx] * (1.0 - fx) + plane[sy * width + sx_next] * fx;
    let bottom = plane[sy_next * width + sx] * (1.0 - fx) + plane[sy_next * width + sx_next] * fx;
    top * (1.0 - fy) + bottom * fy
}

fn temporal_uncertainty(probability: f32) -> f32 {
    assert!(probability >= 0.0);
    assert!(probability <= 1.0);
    let x = (probability - 0.5) * (probability - 0.5);
    let polynomial = x
        * (TEMPORAL_UNCERTAINTY_C1
            + x * (TEMPORAL_UNCERTAINTY_C2
                + x * (TEMPORAL_UNCERTAINTY_C3
                    + x * (TEMPORAL_UNCERTAINTY_C4 + x * TEMPORAL_UNCERTAINTY_C5))));
    1.0 - polynomial.min(1.0)
}

fn temporal_weight_lut() -> [u16; LUT_LEN] {
    let mut lut = [0u16; LUT_LEN];
    for (value, slot) in lut.iter_mut().enumerate() {
        let probability = value as f32 / 255.0;
        let weight = temporal_uncertainty(probability) * TEMPORAL_COMBINE_RATIO;
        assert!(weight >= 0.0);
        assert!(weight <= 1.0);
        *slot = (weight * 256.0 + 0.5) as u16;
    }
    lut
}

fn shape_lut() -> [u8; LUT_LEN] {
    let span = SHAPE_SMOOTHSTEP_EDGE_HIGH - SHAPE_SMOOTHSTEP_EDGE_LOW;
    assert!(span > 0.0);
    let mut lut = [0u8; LUT_LEN];
    for (value, slot) in lut.iter_mut().enumerate() {
        let probability = value as f32 / 255.0;
        let t = ((probability - SHAPE_SMOOTHSTEP_EDGE_LOW) / span).clamp(0.0, 1.0);
        let smooth = t * t * (3.0 - 2.0 * t);
        *slot = (smooth * 255.0 + 0.5) as u8;
    }
    assert_eq!(lut[0], 0);
    assert_eq!(lut[LUT_LEN - 1], 255);
    lut
}

pub(crate) fn box_filter_low(
    src: &[f32],
    scratch: &mut [f32],
    dst: &mut [f32],
    width: usize,
    height: usize,
    radius: usize,
) {
    assert!(width >= 1);
    assert!(height >= 1);
    assert!(src.len() >= width * height);
    assert!(scratch.len() >= width * height);
    assert!(dst.len() >= width * height);
    box_filter_rows_low(src, scratch, width, height, radius);
    box_filter_columns_low(scratch, dst, width, height, radius);
}

fn box_filter_rows_low(src: &[f32], dst: &mut [f32], width: usize, height: usize, radius: usize) {
    assert!(width >= 1);
    assert!(src.len() >= width * height);
    for y in 0..height {
        let row = y * width;
        let mut start = 0usize;
        let mut end = radius.min(width - 1);
        let mut sum: f32 = src[row..=row + end].iter().sum();
        for x in 0..width {
            dst[row + x] = sum / ((end - start + 1) as f32);
            let next_end = (x + 1 + radius).min(width - 1);
            if next_end > end {
                sum += src[row + next_end];
                end = next_end;
            }
            let next_start = (x + 1).saturating_sub(radius);
            if next_start > start {
                sum -= src[row + start];
                start = next_start;
            }
        }
    }
}

fn box_filter_columns_low(
    src: &[f32],
    dst: &mut [f32],
    width: usize,
    height: usize,
    radius: usize,
) {
    assert!(width >= 1);
    assert!(width <= FRAME_EDGE_MAX);
    assert!(height >= 1);
    let mut sums = [0.0f32; FRAME_EDGE_MAX];
    let mut start = 0usize;
    let mut end = radius.min(height - 1);
    for y in 0..=end {
        let row = y * width;
        for x in 0..width {
            sums[x] += src[row + x];
        }
    }
    for y in 0..height {
        let scale = 1.0 / ((end - start + 1) as f32);
        let row = y * width;
        for x in 0..width {
            dst[row + x] = sums[x] * scale;
        }
        let next_end = (y + 1 + radius).min(height - 1);
        if next_end > end {
            let next_row = next_end * width;
            for x in 0..width {
                sums[x] += src[next_row + x];
            }
            end = next_end;
        }
        let next_start = (y + 1).saturating_sub(radius);
        if next_start > start {
            let previous_row = start * width;
            for x in 0..width {
                sums[x] -= src[previous_row + x];
            }
            start = next_start;
        }
    }
}

fn morph_pass_low(
    src: &[f32],
    scratch: &mut [f32],
    dst: &mut [f32],
    width: usize,
    height: usize,
    select: fn(f32, f32) -> f32,
) {
    assert!(width >= 1);
    assert!(height >= 1);
    assert!(src.len() >= width * height);
    assert!(scratch.len() >= width * height);
    assert!(dst.len() >= width * height);
    for y in 0..height {
        let row = y * width;
        for x in 0..width {
            let left = src[row + x.saturating_sub(1)];
            let right = src[row + (x + 1).min(width - 1)];
            scratch[row + x] = select(select(left, src[row + x]), right);
        }
    }
    for y in 0..height {
        let above = y.saturating_sub(1) * width;
        let below = (y + 1).min(height - 1) * width;
        let row = y * width;
        for x in 0..width {
            dst[row + x] = select(
                select(scratch[above + x], scratch[row + x]),
                scratch[below + x],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient_luma(width: usize, height: usize) -> Vec<u8> {
        let mut luma = vec![0u8; width * height];
        for (index, value) in luma.iter_mut().enumerate() {
            *value = ((index % width) * 255 / (width - 1).max(1)) as u8;
        }
        luma
    }

    fn left_half_mask(width: usize, height: usize) -> Vec<u8> {
        let mut mask = vec![0u8; width * height];
        for y in 0..height {
            for x in 0..width / 2 {
                mask[y * width + x] = 255;
            }
        }
        mask
    }

    #[test]
    fn temporal_weight_lut_smooths_uncertain_values_and_trusts_confident_ones() {
        let lut = temporal_weight_lut();

        assert!(lut[128] >= 170);
        assert!(lut[128] <= 182);
        assert!(lut[0] <= 8);
        assert!(lut[255] <= 8);
        assert!(lut[64] > lut[16]);
    }

    #[test]
    fn temporal_blend_pulls_uncertain_pixels_toward_previous_mask() {
        let mut refiner = MaskRefiner::new(8, 8);
        let mut first = vec![128u8; 64];
        refiner.blend_temporal(&mut first);
        let mut second = vec![128u8; 64];
        second[0] = 255;
        second[1] = 130;

        refiner.blend_temporal(&mut second);

        assert_eq!(second[0], 255);
        assert!(second[1] < 130);
        assert_eq!(second[63], 128);
    }

    #[test]
    fn temporal_blend_first_frame_passes_mask_through_unchanged() {
        let mut refiner = MaskRefiner::new(8, 8);
        let mut mask = vec![37u8; 64];

        refiner.blend_temporal(&mut mask);

        assert!(mask.iter().all(|value| *value == 37));
        assert!(refiner.previous_mask_valid);
    }

    #[test]
    fn shape_lut_is_monotonic_and_saturates_at_both_ends() {
        let lut = shape_lut();

        for value in 1..LUT_LEN {
            assert!(lut[value] >= lut[value - 1]);
        }
        assert_eq!(lut[(255.0 * SHAPE_SMOOTHSTEP_EDGE_LOW) as usize - 4], 0);
        assert_eq!(lut[(255.0 * SHAPE_SMOOTHSTEP_EDGE_HIGH) as usize + 4], 255);
    }

    #[test]
    fn refine_keeps_solid_person_and_background_regions_saturated() {
        let width = 64usize;
        let height = 48usize;
        let mut refiner = MaskRefiner::new(width, height);
        let luma = {
            let mut luma = vec![32u8; width * height];
            for y in 0..height {
                for x in width / 2..width {
                    luma[y * width + x] = 224;
                }
            }
            luma
        };
        let mut mask = left_half_mask(width, height);

        refiner.refine(&luma, &mut mask);

        assert_eq!(mask[24 * width], 255);
        assert_eq!(mask[24 * width + 4], 255);
        assert_eq!(mask[24 * width + width - 1], 0);
        assert_eq!(mask[24 * width + width - 5], 0);
    }

    #[test]
    fn refine_snaps_mask_transition_to_the_luma_edge() {
        let width = 64usize;
        let height = 48usize;
        let mut refiner = MaskRefiner::new(width, height);
        let mut luma = vec![16u8; width * height];
        for y in 0..height {
            for x in 0..width / 2 {
                luma[y * width + x] = 240;
            }
        }
        let mut blurry_mask = vec![0u8; width * height];
        for y in 0..height {
            for x in 0..width {
                let distance = (width as i32 / 2 - x as i32).clamp(-12, 12);
                blurry_mask[y * width + x] = (127 + distance * 10).clamp(0, 255) as u8;
            }
        }

        refiner.refine(&luma, &mut blurry_mask);

        let row = 24 * width;
        assert!(blurry_mask[row + width / 2 - 8] > 220);
        assert!(blurry_mask[row + width / 2 + 8] < 35);
    }

    #[test]
    fn refine_fills_small_holes_inside_the_person() {
        let width = 64usize;
        let height = 48usize;
        let mut refiner = MaskRefiner::new(width, height);
        let luma = vec![128u8; width * height];
        let mut mask = vec![255u8; width * height];
        mask[24 * width + 32] = 0;

        refiner.refine(&luma, &mut mask);

        assert!(mask[24 * width + 32] > 200);
        assert_eq!(mask[0], 255);
    }

    #[test]
    fn refine_handles_minimum_dimensions_without_panicking() {
        let mut refiner = MaskRefiner::new(2, 2);
        let luma = vec![128u8; 4];
        let mut mask = vec![255u8; 4];

        refiner.refine(&luma, &mut mask);

        assert_eq!(mask.len(), 4);
        assert!(mask.iter().all(|value| *value == 255));
    }

    #[test]
    fn box_filter_low_preserves_constant_planes_exactly() {
        let width = 9usize;
        let height = 7usize;
        let src = vec![0.625f32; width * height];
        let mut scratch = vec![0.0f32; width * height];
        let mut dst = vec![0.0f32; width * height];

        box_filter_low(&src, &mut scratch, &mut dst, width, height, 4);

        for value in dst {
            assert!((value - 0.625).abs() < 1e-6);
        }
    }

    #[test]
    fn morph_close_removes_single_pixel_pits_and_keeps_plateaus() {
        let width = 8usize;
        let height = 8usize;
        let mut src = vec![1.0f32; width * height];
        src[3 * width + 3] = 0.0;
        let mut scratch = vec![0.0f32; width * height];
        let mut maxed = vec![0.0f32; width * height];
        let mut closed = vec![0.0f32; width * height];

        morph_pass_low(&src, &mut scratch, &mut maxed, width, height, f32::max);
        morph_pass_low(&maxed, &mut scratch, &mut closed, width, height, f32::min);

        assert!(closed[3 * width + 3] > 0.99);
        assert!(closed[0] > 0.99);
    }

    #[test]
    fn refine_converges_to_stable_mask_over_repeated_identical_frames() {
        let width = 32usize;
        let height = 24usize;
        let mut refiner = MaskRefiner::new(width, height);
        let luma = gradient_luma(width, height);
        let raw = left_half_mask(width, height);
        let mut previous_output = vec![0u8; width * height];
        for iteration in 0..8 {
            let mut mask = raw.clone();
            refiner.refine(&luma, &mut mask);
            if iteration == 7 {
                let drift: i32 = mask
                    .iter()
                    .zip(previous_output.iter())
                    .map(|(a, b)| (i32::from(*a) - i32::from(*b)).abs())
                    .sum();
                assert!(drift <= (width * height) as i32);
            }
            previous_output.copy_from_slice(&mask);
        }
        assert_eq!(previous_output[12 * width], 255);
    }
}
