// SPDX-License-Identifier: AGPL-3.0-or-later

pub const PERSON_MASK_BACKGROUND: u8 = 0;
pub const PERSON_MASK_PERSON: u8 = 255;
pub const SEGMENTATION_FRAME_BUDGET_MS: u64 = 12;
pub const SEGMENTATION_SLOW_FRAMES_CONSECUTIVE_MAX: u32 = 30;

pub trait PersonMaskSource {
    fn mask_into(&mut self, frame: &crate::yuv::I420, mask: &mut [u8]) -> bool;
}

#[derive(Debug, Default)]
pub struct SegmentationQualityGovernor {
    consecutive_slow_frames: u32,
    downgraded: bool,
}

impl SegmentationQualityGovernor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_frame_duration_ms(&mut self, duration_ms: u64) -> bool {
        assert!(self.consecutive_slow_frames < SEGMENTATION_SLOW_FRAMES_CONSECUTIVE_MAX);
        if self.downgraded {
            return false;
        }
        if duration_ms <= SEGMENTATION_FRAME_BUDGET_MS {
            self.consecutive_slow_frames = 0;
            return false;
        }
        self.consecutive_slow_frames += 1;
        assert!(self.consecutive_slow_frames <= SEGMENTATION_SLOW_FRAMES_CONSECUTIVE_MAX);
        if self.consecutive_slow_frames < SEGMENTATION_SLOW_FRAMES_CONSECUTIVE_MAX {
            return false;
        }
        self.consecutive_slow_frames = 0;
        self.downgraded = true;
        true
    }
}

pub fn create_person_mask_source(width: u32, height: u32) -> Option<Box<dyn PersonMaskSource>> {
    assert!(width >= 2);
    assert!(height >= 2);
    #[cfg(target_os = "macos")]
    {
        if let Some(source) = vision::VisionPersonMaskSource::new(width, height) {
            return Some(Box::new(source) as Box<dyn PersonMaskSource>);
        }
    }
    selfie::SelfieMaskSource::new(width, height)
        .map(|source| Box::new(source) as Box<dyn PersonMaskSource>)
}

pub fn resize_mask_bilinear(
    src: &[u8],
    src_width: usize,
    src_height: usize,
    src_stride: usize,
    dst: &mut [u8],
    dst_width: usize,
    dst_height: usize,
) {
    assert!(src_width >= 1);
    assert!(src_height >= 1);
    assert!(src_stride >= src_width);
    assert!(dst_width >= 1);
    assert!(dst_height >= 1);
    assert!(src.len() >= src_stride * (src_height - 1) + src_width);
    assert!(dst.len() >= dst_width * dst_height);

    for y in 0..dst_height {
        let sy_fixed = if dst_height == 1 {
            0
        } else {
            y * (src_height - 1) * 256 / (dst_height - 1)
        };
        let sy = sy_fixed / 256;
        let fy = (sy_fixed % 256) as u32;
        let sy_next = (sy + 1).min(src_height - 1);
        for x in 0..dst_width {
            let sx_fixed = if dst_width == 1 {
                0
            } else {
                x * (src_width - 1) * 256 / (dst_width - 1)
            };
            let sx = sx_fixed / 256;
            let fx = (sx_fixed % 256) as u32;
            let sx_next = (sx + 1).min(src_width - 1);
            let top = u32::from(src[sy * src_stride + sx]) * (256 - fx)
                + u32::from(src[sy * src_stride + sx_next]) * fx;
            let bottom = u32::from(src[sy_next * src_stride + sx]) * (256 - fx)
                + u32::from(src[sy_next * src_stride + sx_next]) * fx;
            dst[y * dst_width + x] = ((top * (256 - fy) + bottom * fy) >> 16) as u8;
        }
    }
}

mod selfie {
    use super::PersonMaskSource;
    use std::sync::{Arc, OnceLock};
    use tract_onnx::prelude::*;

    const MODEL_BYTES: &[u8] = include_bytes!("../models/selfie_segmenter_landscape.onnx");
    const MODEL_INPUT_WIDTH: usize = 256;
    const MODEL_INPUT_HEIGHT: usize = 144;
    const MODEL_INPUT_CHANNELS: usize = 3;
    const MODEL_INPUT_LEN: usize = MODEL_INPUT_WIDTH * MODEL_INPUT_HEIGHT * MODEL_INPUT_CHANNELS;
    const MODEL_CHROMA_WIDTH: usize = MODEL_INPUT_WIDTH / 2;
    const MODEL_CHROMA_HEIGHT: usize = MODEL_INPUT_HEIGHT / 2;
    const INFERENCE_FRAME_INTERVAL_FULL: u32 = 1;
    const INFERENCE_FRAME_INTERVAL_DOWNGRADED: u32 = 2;

    type SelfiePlan = TypedRunnableModel;

    fn shared_plan() -> Option<Arc<SelfiePlan>> {
        static PLAN: OnceLock<Option<Arc<SelfiePlan>>> = OnceLock::new();
        PLAN.get_or_init(|| match load_plan() {
            Ok(plan) => Some(plan),
            Err(error) => {
                eprintln!(
                    "webrtc-sender: selfie segmentation model failed to load; camera \
                     background effects fall back to the portrait ellipse: {error}"
                );
                None
            }
        })
        .clone()
    }

    fn load_plan() -> TractResult<Arc<SelfiePlan>> {
        let mut reader = std::io::Cursor::new(MODEL_BYTES);
        tract_onnx::onnx()
            .model_for_read(&mut reader)?
            .with_input_fact(
                0,
                f32::fact([
                    1,
                    MODEL_INPUT_HEIGHT,
                    MODEL_INPUT_WIDTH,
                    MODEL_INPUT_CHANNELS,
                ])
                .into(),
            )?
            .into_optimized()?
            .into_runnable()
    }

    pub struct SelfieMaskSource {
        width: u32,
        height: u32,
        plan: Arc<SelfiePlan>,
        luma_low: Vec<u8>,
        chroma_u_low: Vec<u8>,
        chroma_v_low: Vec<u8>,
        input_rgb: Vec<f32>,
        raw_mask_low: Vec<u8>,
        raw_mask_valid: bool,
        frame_counter: u32,
        inference_interval: u32,
        inference_error_logged: bool,
        governor: super::SegmentationQualityGovernor,
    }

    impl SelfieMaskSource {
        pub fn new(width: u32, height: u32) -> Option<Self> {
            assert!(width >= 2);
            assert!(height >= 2);
            let plan = shared_plan()?;
            Some(Self {
                width,
                height,
                plan,
                luma_low: vec![0; MODEL_INPUT_WIDTH * MODEL_INPUT_HEIGHT],
                chroma_u_low: vec![128; MODEL_CHROMA_WIDTH * MODEL_CHROMA_HEIGHT],
                chroma_v_low: vec![128; MODEL_CHROMA_WIDTH * MODEL_CHROMA_HEIGHT],
                input_rgb: vec![0.0; MODEL_INPUT_LEN],
                raw_mask_low: vec![0; MODEL_INPUT_WIDTH * MODEL_INPUT_HEIGHT],
                raw_mask_valid: false,
                frame_counter: 0,
                inference_interval: INFERENCE_FRAME_INTERVAL_FULL,
                inference_error_logged: false,
                governor: super::SegmentationQualityGovernor::new(),
            })
        }

        fn fill_model_input(&mut self, frame: &crate::yuv::I420) {
            assert_eq!(frame.width, self.width);
            assert_eq!(frame.height, self.height);
            let width = self.width as usize;
            let height = self.height as usize;
            super::resize_mask_bilinear(
                &frame.y,
                width,
                height,
                width,
                &mut self.luma_low,
                MODEL_INPUT_WIDTH,
                MODEL_INPUT_HEIGHT,
            );
            super::resize_mask_bilinear(
                &frame.u,
                width / 2,
                height / 2,
                width / 2,
                &mut self.chroma_u_low,
                MODEL_CHROMA_WIDTH,
                MODEL_CHROMA_HEIGHT,
            );
            super::resize_mask_bilinear(
                &frame.v,
                width / 2,
                height / 2,
                width / 2,
                &mut self.chroma_v_low,
                MODEL_CHROMA_WIDTH,
                MODEL_CHROMA_HEIGHT,
            );
            for y in 0..MODEL_INPUT_HEIGHT {
                let row = y * MODEL_INPUT_WIDTH;
                let chroma_row = (y / 2) * MODEL_CHROMA_WIDTH;
                for x in 0..MODEL_INPUT_WIDTH {
                    let luma = i32::from(self.luma_low[row + x]) - 16;
                    let cb = i32::from(self.chroma_u_low[chroma_row + x / 2]) - 128;
                    let cr = i32::from(self.chroma_v_low[chroma_row + x / 2]) - 128;
                    let r = ((298 * luma + 409 * cr + 128) >> 8).clamp(0, 255);
                    let g = ((298 * luma - 100 * cb - 208 * cr + 128) >> 8).clamp(0, 255);
                    let b = ((298 * luma + 516 * cb + 128) >> 8).clamp(0, 255);
                    let offset = (row + x) * MODEL_INPUT_CHANNELS;
                    self.input_rgb[offset] = r as f32 / 255.0;
                    self.input_rgb[offset + 1] = g as f32 / 255.0;
                    self.input_rgb[offset + 2] = b as f32 / 255.0;
                }
            }
        }

        fn run_inference(&mut self, frame: &crate::yuv::I420) -> bool {
            self.fill_model_input(frame);
            let produced = self.run_model();
            if !produced && !self.inference_error_logged {
                self.inference_error_logged = true;
                eprintln!(
                    "webrtc-sender: selfie segmentation inference failed; reusing the \
                     previous person mask"
                );
            }
            produced
        }

        fn run_model(&mut self) -> bool {
            assert_eq!(self.input_rgb.len(), MODEL_INPUT_LEN);
            let Ok(tensor) = Tensor::from_shape(
                &[
                    1,
                    MODEL_INPUT_HEIGHT,
                    MODEL_INPUT_WIDTH,
                    MODEL_INPUT_CHANNELS,
                ],
                &self.input_rgb,
            ) else {
                return false;
            };
            let Ok(result) = self.plan.run(tvec!(tensor.into())) else {
                return false;
            };
            let Some(output) = result.first() else {
                return false;
            };
            let Ok(alphas) = output.to_plain_array_view::<f32>() else {
                return false;
            };
            if alphas.len() != self.raw_mask_low.len() {
                return false;
            }
            for (slot, alpha) in self.raw_mask_low.iter_mut().zip(alphas.iter()) {
                *slot = (alpha * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
            }
            true
        }

        fn record_inference_duration(&mut self, duration_ms: u64) {
            if self.governor.record_frame_duration_ms(duration_ms) {
                self.inference_interval = INFERENCE_FRAME_INTERVAL_DOWNGRADED;
                eprintln!(
                    "webrtc-sender: selfie segmentation exceeded the {}ms frame budget for {} \
                     consecutive frames; downgrading to inference every {} frames",
                    super::SEGMENTATION_FRAME_BUDGET_MS,
                    super::SEGMENTATION_SLOW_FRAMES_CONSECUTIVE_MAX,
                    INFERENCE_FRAME_INTERVAL_DOWNGRADED
                );
            }
        }
    }

    impl PersonMaskSource for SelfieMaskSource {
        fn mask_into(&mut self, frame: &crate::yuv::I420, mask: &mut [u8]) -> bool {
            let width = self.width as usize;
            let height = self.height as usize;
            assert!(mask.len() >= width * height);
            if frame.width != self.width || frame.height != self.height {
                return false;
            }
            assert!(self.inference_interval >= 1);
            let due = self.frame_counter.is_multiple_of(self.inference_interval);
            self.frame_counter = self.frame_counter.wrapping_add(1);
            if due || !self.raw_mask_valid {
                let started = std::time::Instant::now();
                if self.run_inference(frame) {
                    self.raw_mask_valid = true;
                    let duration_ms = started.elapsed().as_millis() as u64;
                    self.record_inference_duration(duration_ms);
                }
            }
            if !self.raw_mask_valid {
                return false;
            }
            super::resize_mask_bilinear(
                &self.raw_mask_low,
                MODEL_INPUT_WIDTH,
                MODEL_INPUT_HEIGHT,
                MODEL_INPUT_WIDTH,
                mask,
                width,
                height,
            );
            true
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn synthetic_frame(width: u32, height: u32) -> crate::yuv::I420 {
            let mut frame = crate::yuv::I420::new(width, height).unwrap();
            for (index, value) in frame.y.iter_mut().enumerate() {
                *value = ((index * 31 + 17) % 220) as u8 + 16;
            }
            frame.u.fill(128);
            frame.v.fill(128);
            frame
        }

        #[test]
        fn selfie_source_produces_full_range_mask_for_synthetic_frames() {
            let mut source = SelfieMaskSource::new(128, 96).expect("bundled model loads");
            let frame = synthetic_frame(128, 96);
            let mut mask = vec![0u8; 128 * 96];

            assert!(source.mask_into(&frame, &mut mask));

            assert_eq!(mask.len(), 128 * 96);
            assert!(source.raw_mask_valid);
        }

        #[test]
        fn selfie_source_rejects_mismatched_frame_dimensions() {
            let mut source = SelfieMaskSource::new(128, 96).expect("bundled model loads");
            let frame = synthetic_frame(64, 48);
            let mut mask = vec![0u8; 128 * 96];

            assert!(!source.mask_into(&frame, &mut mask));
        }

        #[test]
        fn selfie_source_reuses_cached_mask_between_inference_frames() {
            let mut source = SelfieMaskSource::new(64, 48).expect("bundled model loads");
            source.inference_interval = INFERENCE_FRAME_INTERVAL_DOWNGRADED;
            let frame = synthetic_frame(64, 48);
            let mut first = vec![0u8; 64 * 48];
            let mut second = vec![0u8; 64 * 48];

            assert!(source.mask_into(&frame, &mut first));
            assert!(source.mask_into(&frame, &mut second));

            assert_eq!(first, second);
        }
    }
}

#[cfg(target_os = "macos")]
mod vision {
    use super::PersonMaskSource;
    use core::ptr::NonNull;
    use objc2::rc::Retained;
    use objc2_core_foundation::CFRetained;
    use objc2_core_video::{
        CVPixelBuffer, CVPixelBufferGetBaseAddress, CVPixelBufferGetBaseAddressOfPlane,
        CVPixelBufferGetBytesPerRow, CVPixelBufferGetBytesPerRowOfPlane, CVPixelBufferGetHeight,
        CVPixelBufferGetPixelFormatType, CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress,
        CVPixelBufferLockFlags, CVPixelBufferUnlockBaseAddress,
    };
    use objc2_vision::{
        VNGeneratePersonSegmentationRequest, VNGeneratePersonSegmentationRequestQualityLevel,
        VNRequest, VNSequenceRequestHandler,
    };

    const PIXEL_FORMAT_NV12_FULL_RANGE: u32 = u32::from_be_bytes(*b"420f");
    const PIXEL_FORMAT_ONE_COMPONENT_8: u32 = u32::from_be_bytes(*b"L008");
    const MASK_PIXELS_MAX: usize = 8192 * 8192;

    pub struct VisionPersonMaskSource {
        width: u32,
        height: u32,
        pixel_buffer: CFRetained<CVPixelBuffer>,
        request: Retained<VNGeneratePersonSegmentationRequest>,
        requests: Retained<objc2_foundation::NSArray<VNRequest>>,
        handler: Retained<VNSequenceRequestHandler>,
        quality_governor: super::SegmentationQualityGovernor,
    }

    impl VisionPersonMaskSource {
        pub fn new(width: u32, height: u32) -> Option<Self> {
            assert!(width >= 2);
            assert!(height >= 2);
            assert!(width.is_multiple_of(2));
            assert!(height.is_multiple_of(2));
            let mut pixel_buffer_out: *mut CVPixelBuffer = core::ptr::null_mut();
            let status = unsafe {
                objc2_core_video::CVPixelBufferCreate(
                    None,
                    width as usize,
                    height as usize,
                    PIXEL_FORMAT_NV12_FULL_RANGE,
                    None,
                    NonNull::new(&mut pixel_buffer_out)?,
                )
            };
            if status != 0 {
                return None;
            }
            let pixel_buffer = unsafe { CFRetained::from_raw(NonNull::new(pixel_buffer_out)?) };
            let request = unsafe { VNGeneratePersonSegmentationRequest::new() };
            unsafe {
                request.setQualityLevel(VNGeneratePersonSegmentationRequestQualityLevel::Balanced);
                request.setOutputPixelFormat(PIXEL_FORMAT_ONE_COMPONENT_8);
            }
            let request_as_base: Retained<VNRequest> =
                Retained::into_super(Retained::into_super(Retained::into_super(request.clone())));
            let requests = objc2_foundation::NSArray::from_retained_slice(&[request_as_base]);
            assert_eq!(requests.len(), 1);
            let handler = unsafe { VNSequenceRequestHandler::new() };
            Some(Self {
                width,
                height,
                pixel_buffer,
                request,
                requests,
                handler,
                quality_governor: super::SegmentationQualityGovernor::new(),
            })
        }

        fn downgrade_to_fast_quality(&self) {
            unsafe {
                self.request
                    .setQualityLevel(VNGeneratePersonSegmentationRequestQualityLevel::Fast);
            }
            eprintln!(
                "webrtc-sender: person segmentation exceeded the {}ms frame budget for {} \
                 consecutive frames; downgrading Vision quality from balanced to fast",
                super::SEGMENTATION_FRAME_BUDGET_MS,
                super::SEGMENTATION_SLOW_FRAMES_CONSECUTIVE_MAX
            );
        }

        fn fill_pixel_buffer(&self, frame: &crate::yuv::I420) -> bool {
            assert_eq!(frame.width, self.width);
            assert_eq!(frame.height, self.height);
            let width = self.width as usize;
            let height = self.height as usize;
            let lock_flags = CVPixelBufferLockFlags(0);
            let lock_status =
                unsafe { CVPixelBufferLockBaseAddress(&self.pixel_buffer, lock_flags) };
            if lock_status != 0 {
                return false;
            }
            let y_base = CVPixelBufferGetBaseAddressOfPlane(&self.pixel_buffer, 0);
            let y_stride = CVPixelBufferGetBytesPerRowOfPlane(&self.pixel_buffer, 0);
            let uv_base = CVPixelBufferGetBaseAddressOfPlane(&self.pixel_buffer, 1);
            let uv_stride = CVPixelBufferGetBytesPerRowOfPlane(&self.pixel_buffer, 1);
            if y_base.is_null() || uv_base.is_null() || y_stride < width || uv_stride < width {
                let _ = unsafe { CVPixelBufferUnlockBaseAddress(&self.pixel_buffer, lock_flags) };
                return false;
            }
            let chroma_width = width / 2;
            let chroma_height = height / 2;
            unsafe {
                let y_base = y_base as *mut u8;
                for row in 0..height {
                    let src = &frame.y[row * width..row * width + width];
                    core::ptr::copy_nonoverlapping(src.as_ptr(), y_base.add(row * y_stride), width);
                }
                let uv_base = uv_base as *mut u8;
                for row in 0..chroma_height {
                    let dst_row = uv_base.add(row * uv_stride);
                    for col in 0..chroma_width {
                        let chroma_index = row * chroma_width + col;
                        dst_row.add(col * 2).write(frame.u[chroma_index]);
                        dst_row.add(col * 2 + 1).write(frame.v[chroma_index]);
                    }
                }
            }
            let unlock_status =
                unsafe { CVPixelBufferUnlockBaseAddress(&self.pixel_buffer, lock_flags) };
            unlock_status == 0
        }

        fn copy_observation_mask(
            mask_buffer: &CVPixelBuffer,
            mask: &mut [u8],
            width: usize,
            height: usize,
        ) -> bool {
            if CVPixelBufferGetPixelFormatType(mask_buffer) != PIXEL_FORMAT_ONE_COMPONENT_8 {
                return false;
            }
            let lock_flags = CVPixelBufferLockFlags::ReadOnly;
            if unsafe { CVPixelBufferLockBaseAddress(mask_buffer, lock_flags) } != 0 {
                return false;
            }
            let src_width = CVPixelBufferGetWidth(mask_buffer);
            let src_height = CVPixelBufferGetHeight(mask_buffer);
            let src_stride = CVPixelBufferGetBytesPerRow(mask_buffer);
            let base = CVPixelBufferGetBaseAddress(mask_buffer);
            let valid = !base.is_null()
                && src_width >= 1
                && src_height >= 1
                && src_stride >= src_width
                && src_width * src_height <= MASK_PIXELS_MAX;
            if valid {
                let src = unsafe {
                    core::slice::from_raw_parts(
                        base as *const u8,
                        src_stride * (src_height - 1) + src_width,
                    )
                };
                super::resize_mask_bilinear(
                    src, src_width, src_height, src_stride, mask, width, height,
                );
            }
            let _ = unsafe { CVPixelBufferUnlockBaseAddress(mask_buffer, lock_flags) };
            valid
        }
    }

    impl PersonMaskSource for VisionPersonMaskSource {
        fn mask_into(&mut self, frame: &crate::yuv::I420, mask: &mut [u8]) -> bool {
            let started = std::time::Instant::now();
            let produced = self.mask_into_timed(frame, mask);
            let duration_ms = started.elapsed().as_millis() as u64;
            if self.quality_governor.record_frame_duration_ms(duration_ms) {
                self.downgrade_to_fast_quality();
            }
            produced
        }
    }

    impl VisionPersonMaskSource {
        fn mask_into_timed(&mut self, frame: &crate::yuv::I420, mask: &mut [u8]) -> bool {
            let width = self.width as usize;
            let height = self.height as usize;
            assert!(mask.len() >= width * height);
            if frame.width != self.width || frame.height != self.height {
                return false;
            }
            if !self.fill_pixel_buffer(frame) {
                return false;
            }
            let performed = unsafe {
                self.handler
                    .performRequests_onCVPixelBuffer_error(&self.requests, &self.pixel_buffer)
            };
            if performed.is_err() {
                return false;
            }
            let Some(results) = (unsafe { self.request.results() }) else {
                return false;
            };
            let Some(observation) = results.firstObject() else {
                return false;
            };
            let mask_buffer = unsafe { observation.pixelBuffer() };
            Self::copy_observation_mask(&mask_buffer, mask, width, height)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_identity_returns_same_values() {
        let src = vec![0u8, 64, 128, 255];
        let mut dst = vec![0u8; 4];

        resize_mask_bilinear(&src, 2, 2, 2, &mut dst, 2, 2);

        assert_eq!(dst, src);
    }

    #[test]
    fn resize_upscales_with_interpolated_midpoints() {
        let src = vec![0u8, 255, 0, 255];
        let mut dst = vec![0u8; 9];

        resize_mask_bilinear(&src, 2, 2, 2, &mut dst, 3, 3);

        assert_eq!(dst[0], 0);
        assert_eq!(dst[2], 255);
        assert!(dst[1] > 100);
        assert!(dst[1] < 156);
    }

    #[test]
    fn resize_honours_source_stride_padding() {
        let src = vec![10u8, 20, 99, 99, 30, 40, 99, 99];
        let mut dst = vec![0u8; 4];

        resize_mask_bilinear(&src, 2, 2, 4, &mut dst, 2, 2);

        assert_eq!(dst, vec![10, 20, 30, 40]);
    }

    #[test]
    fn resize_collapses_to_single_pixel_average_free() {
        let src = vec![200u8; 16];
        let mut dst = vec![0u8; 1];

        resize_mask_bilinear(&src, 4, 4, 4, &mut dst, 1, 1);

        assert_eq!(dst, vec![200]);
    }

    #[test]
    fn mask_constants_span_full_alpha_range() {
        assert_eq!(PERSON_MASK_BACKGROUND, 0);
        assert_eq!(PERSON_MASK_PERSON, 255);
    }

    #[test]
    fn segmentation_governor_downgrades_after_consecutive_slow_frames() {
        let mut governor = SegmentationQualityGovernor::new();

        for _ in 1..SEGMENTATION_SLOW_FRAMES_CONSECUTIVE_MAX {
            assert!(!governor.record_frame_duration_ms(SEGMENTATION_FRAME_BUDGET_MS + 1));
        }

        assert!(governor.record_frame_duration_ms(SEGMENTATION_FRAME_BUDGET_MS + 1));
    }

    #[test]
    fn segmentation_governor_resets_count_after_a_frame_within_budget() {
        let mut governor = SegmentationQualityGovernor::new();

        for _ in 1..SEGMENTATION_SLOW_FRAMES_CONSECUTIVE_MAX {
            assert!(!governor.record_frame_duration_ms(SEGMENTATION_FRAME_BUDGET_MS + 1));
        }
        assert!(!governor.record_frame_duration_ms(SEGMENTATION_FRAME_BUDGET_MS));
        for _ in 1..SEGMENTATION_SLOW_FRAMES_CONSECUTIVE_MAX {
            assert!(!governor.record_frame_duration_ms(SEGMENTATION_FRAME_BUDGET_MS + 1));
        }

        assert!(governor.record_frame_duration_ms(SEGMENTATION_FRAME_BUDGET_MS + 1));
    }

    #[test]
    fn segmentation_governor_downgrades_only_once() {
        let mut governor = SegmentationQualityGovernor::new();
        for _ in 0..SEGMENTATION_SLOW_FRAMES_CONSECUTIVE_MAX - 1 {
            assert!(!governor.record_frame_duration_ms(SEGMENTATION_FRAME_BUDGET_MS + 1));
        }
        assert!(governor.record_frame_duration_ms(SEGMENTATION_FRAME_BUDGET_MS + 1));

        for _ in 0..SEGMENTATION_SLOW_FRAMES_CONSECUTIVE_MAX * 2 {
            assert!(!governor.record_frame_duration_ms(SEGMENTATION_FRAME_BUDGET_MS + 100));
        }
    }
}
