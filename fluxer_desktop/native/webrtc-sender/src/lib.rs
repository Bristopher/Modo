// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(unsafe_op_in_unsafe_fn)]
#![cfg_attr(
    not(all(feature = "publisher", feature = "camera-native")),
    allow(dead_code)
)]

mod audio;
mod bridge_version;
mod camera;
mod camera_background;
mod config;
mod deep_filter;
mod events;
mod hardware_encoder;
mod inbound_forwarder;
mod mask_refine;
mod native_camera;
mod person_segmentation;
mod send_control;
mod speaking;
mod stats;
mod texture_source;
mod yuv;

#[cfg(feature = "publisher")]
mod engine;

#[cfg(feature = "bench-internals")]
pub mod bench_internals {
    pub use crate::audio::DEEP_FILTER_NOISE_REDUCTION_LEVEL_MAX;
    pub use crate::deep_filter::{DEEP_FILTER_FRAME_SAMPLES, DeepFilterProcessor};
    pub use crate::mask_refine::MaskRefiner;

    pub struct BlurScratch(crate::camera_background::BlurScratch);

    impl BlurScratch {
        pub fn new(width: usize, height: usize) -> Self {
            Self(crate::camera_background::BlurScratch::new(width, height))
        }
    }

    pub fn blur_plane_masked(
        plane: &mut [u8],
        width: usize,
        height: usize,
        mask: &[u8],
        radius_pass: usize,
        scratch: &mut BlurScratch,
    ) {
        let mask = crate::camera_background::plane_mask(mask, width, 1);
        crate::camera_background::blur_plane_masked(
            plane,
            width,
            height,
            mask,
            radius_pass,
            &mut scratch.0,
        );
    }

    pub fn composite_masked_plane(
        plane: &mut [u8],
        background: &[u8],
        width: usize,
        height: usize,
        mask: &[u8],
    ) {
        let mask = crate::camera_background::plane_mask(mask, width, 1);
        crate::camera_background::composite_masked_plane(plane, background, width, height, mask);
    }
}
