// SPDX-License-Identifier: AGPL-3.0-or-later

use image::{AnimationDecoder, ImageReader, imageops::FilterType};
use std::sync::Arc;

const MAX_CUSTOM_BACKGROUND_BYTES: u64 = 10 * 1024 * 1024;
const FRAME_EDGE_MAX: usize = 8192;
pub const CUSTOM_BACKGROUND_ANIMATION_FRAMES_MAX: usize = 64;
pub const CUSTOM_BACKGROUND_ANIMATION_BYTES_MAX: usize = 64 * 1024 * 1024;
const CUSTOM_BACKGROUND_FRAME_DELAY_MS_MIN: u64 = 20;
const CUSTOM_BACKGROUND_FRAME_DELAY_MS_MAX: u64 = 10_000;
const CUSTOM_BACKGROUND_FRAME_DELAY_MS_DEFAULT: u64 = 100;
const STATIC_BACKGROUND_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];
const ANIMATED_BACKGROUND_EXTENSIONS: &[&str] = &["gif", "webp"];
const VIDEO_BACKGROUND_EXTENSIONS: &[&str] = &["mp4"];
pub const CAMERA_EFFECT_STRENGTH_MAX: u32 = 100;
pub const CAMERA_EFFECT_STRENGTH_DEFAULT: u32 = 50;
const BLUR_STRENGTH_FACTOR_PERMILLE_MIN: u32 = 250;
const BLUR_STRENGTH_FACTOR_PERMILLE_MID: u32 = 1000;
const BLUR_STRENGTH_FACTOR_PERMILLE_MAX: u32 = 2500;
const BLUR_STRENGTH_FACTOR_PERMILLE_SCALE: u32 = 1000;
const BLUR_RADIUS_SCALED_MIN: usize = 1;
const BLUR_RADIUS_SCALED_MAX: usize = 60;
const BLUR_DOWNSAMPLE: usize = 2;
const BLUR_BOX_PASSES: usize = 3;
const BLUR_PASS_RADIUS_NUMERATOR: usize = 5;
const BLUR_PASS_RADIUS_DENOMINATOR: usize = 17;
const BLUR_WEIGHT_EPSILON: f32 = 0.5;
const PORTRAIT_ELLIPSE_INNER_PERMILLE: i64 = 900;
const PORTRAIT_ELLIPSE_OUTER_PERMILLE: i64 = 1200;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CameraBackgroundMode {
    None,
    Blur,
    Custom,
}

impl CameraBackgroundMode {
    pub fn from_bridge_value(value: Option<&str>) -> Result<Self, String> {
        match value.map(str::trim).filter(|value| !value.is_empty()) {
            None | Some("none") | Some("non") => Ok(Self::None),
            Some("blur") => Ok(Self::Blur),
            Some("custom") => Ok(Self::Custom),
            Some(_) => Err("unsupported native camera background mode".to_string()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CameraBackgroundCustomMediaKind {
    Static,
    Animated,
    Video,
}

impl CameraBackgroundCustomMediaKind {
    pub fn from_bridge_value(value: Option<&str>) -> Result<Option<Self>, String> {
        match value.map(str::trim).filter(|value| !value.is_empty()) {
            None => Ok(None),
            Some("static") => Ok(Some(Self::Static)),
            Some("animated") => Ok(Some(Self::Animated)),
            Some("video") => Ok(Some(Self::Video)),
            Some(_) => Err("unsupported native custom camera background media kind".to_string()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CameraBackgroundCustomSource {
    pub path: String,
    pub media_kind: CameraBackgroundCustomMediaKind,
}

#[derive(Default)]
struct LiveBackgroundState {
    frame: Option<crate::yuv::I420>,
    spare: Option<crate::yuv::I420>,
    target_dims: Option<(u32, u32)>,
}

#[derive(Clone, Default)]
pub struct CameraBackgroundLiveSlot(Arc<parking_lot::Mutex<LiveBackgroundState>>);

impl PartialEq for CameraBackgroundLiveSlot {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for CameraBackgroundLiveSlot {}

impl std::fmt::Debug for CameraBackgroundLiveSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CameraBackgroundLiveSlot")
    }
}

impl CameraBackgroundLiveSlot {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn store(&self, frame: crate::yuv::I420) -> bool {
        assert!(frame.width >= 2);
        assert!(frame.height >= 2);
        let target_dims = self.0.lock().target_dims;
        let frame = match target_dims {
            Some((width, height)) if (frame.width, frame.height) != (width, height) => {
                match resize_i420(&frame, width, height) {
                    Some(resized) => resized,
                    None => return false,
                }
            }
            _ => frame,
        };
        self.0.lock().frame = Some(frame);
        true
    }

    pub fn store_tight_i420(&self, data: &[u8], width: u32, height: u32) -> bool {
        assert!(width >= 2);
        assert!(height >= 2);
        let mut state = self.0.lock();
        if let Some((target_width, target_height)) = state.target_dims
            && (width, height) != (target_width, target_height)
        {
            drop(state);
            return self.store_resized_tight_i420(data, width, height, target_width, target_height);
        }
        let reuse = matches!(state.frame.as_ref(), Some(frame) if frame.width == width && frame.height == height);
        if !reuse {
            state.frame = crate::yuv::I420::new(width, height);
        }
        let Some(frame) = state.frame.as_mut() else {
            return false;
        };
        crate::yuv::copy_tight_i420_into(data, width, height, frame)
    }

    fn store_resized_tight_i420(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        target_width: u32,
        target_height: u32,
    ) -> bool {
        assert!(target_width >= 2);
        assert!(target_height >= 2);
        let spare = self.0.lock().spare.take();
        let mut resized = match spare {
            Some(frame) if frame.width == target_width && frame.height == target_height => frame,
            _ => match crate::yuv::I420::new(target_width, target_height) {
                Some(frame) => frame,
                None => return false,
            },
        };
        if !resize_tight_i420_into(data, width, height, &mut resized) {
            return false;
        }
        let mut state = self.0.lock();
        if state.target_dims != Some((target_width, target_height)) {
            return false;
        }
        state.spare = state
            .frame
            .replace(resized)
            .filter(|frame| frame.width == target_width && frame.height == target_height);
        true
    }

    pub fn clear(&self) {
        self.0.lock().frame = None;
        assert!(self.0.lock().frame.is_none());
    }

    fn set_target_dims(&self, width: u32, height: u32) {
        assert!(width >= 2);
        assert!(height >= 2);
        let mismatched = {
            let mut state = self.0.lock();
            state.target_dims = Some((width, height));
            match state.frame.as_ref() {
                Some(frame) if frame.width != width || frame.height != height => state.frame.take(),
                _ => None,
            }
        };
        let Some(frame) = mismatched else {
            return;
        };
        let Some(resized) = resize_i420(&frame, width, height) else {
            return;
        };
        let mut state = self.0.lock();
        if state.target_dims == Some((width, height)) && state.frame.is_none() {
            state.frame = Some(resized);
        }
    }

    fn with_latest<R>(&self, callback: impl FnOnce(Option<&crate::yuv::I420>) -> R) -> R {
        let state = self.0.lock();
        callback(state.frame.as_ref())
    }
}

fn resize_tight_i420_into(
    data: &[u8],
    width: u32,
    height: u32,
    dst: &mut crate::yuv::I420,
) -> bool {
    assert!(width >= 2);
    assert!(height >= 2);
    assert!(dst.width >= 2);
    assert!(dst.height >= 2);
    let w = width as usize;
    let h = height as usize;
    let cw = w / 2;
    let ch = h / 2;
    if data.len() < w * h + 2 * cw * ch {
        return false;
    }
    let (y, chroma) = data.split_at(w * h);
    let (u, v) = chroma.split_at(cw * ch);
    let dw = dst.width as usize;
    let dh = dst.height as usize;
    crate::person_segmentation::resize_mask_bilinear(y, w, h, w, &mut dst.y, dw, dh);
    crate::person_segmentation::resize_mask_bilinear(u, cw, ch, cw, &mut dst.u, dw / 2, dh / 2);
    crate::person_segmentation::resize_mask_bilinear(v, cw, ch, cw, &mut dst.v, dw / 2, dh / 2);
    true
}

fn resize_i420(src: &crate::yuv::I420, width: u32, height: u32) -> Option<crate::yuv::I420> {
    let mut dst = crate::yuv::I420::new(width, height)?;
    let src_width = src.width as usize;
    let src_height = src.height as usize;
    let dst_width = width as usize;
    let dst_height = height as usize;
    crate::person_segmentation::resize_mask_bilinear(
        &src.y, src_width, src_height, src_width, &mut dst.y, dst_width, dst_height,
    );
    crate::person_segmentation::resize_mask_bilinear(
        &src.u,
        src_width / 2,
        src_height / 2,
        src_width / 2,
        &mut dst.u,
        dst_width / 2,
        dst_height / 2,
    );
    crate::person_segmentation::resize_mask_bilinear(
        &src.v,
        src_width / 2,
        src_height / 2,
        src_width / 2,
        &mut dst.v,
        dst_width / 2,
        dst_height / 2,
    );
    Some(dst)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CameraBackgroundConfig {
    pub mode: CameraBackgroundMode,
    pub custom: Option<CameraBackgroundCustomSource>,
    pub live_background: Option<CameraBackgroundLiveSlot>,
    pub blur_strength: u32,
}

impl Default for CameraBackgroundConfig {
    fn default() -> Self {
        Self {
            mode: CameraBackgroundMode::None,
            custom: None,
            live_background: None,
            blur_strength: CAMERA_EFFECT_STRENGTH_DEFAULT,
        }
    }
}

pub fn clamp_camera_effect_strength(value: Option<u32>) -> u32 {
    let strength = value
        .unwrap_or(CAMERA_EFFECT_STRENGTH_DEFAULT)
        .min(CAMERA_EFFECT_STRENGTH_MAX);
    assert!(strength <= CAMERA_EFFECT_STRENGTH_MAX);
    strength
}

impl CameraBackgroundConfig {
    pub fn from_bridge_values(
        mode: Option<&str>,
        custom_media_path: Option<&str>,
        custom_media_kind: Option<&str>,
        blur_strength: Option<u32>,
    ) -> Result<Self, String> {
        let mode = CameraBackgroundMode::from_bridge_value(mode)?;
        let media_kind = CameraBackgroundCustomMediaKind::from_bridge_value(custom_media_kind)?;
        let blur_strength = clamp_camera_effect_strength(blur_strength);
        if mode != CameraBackgroundMode::Custom {
            return Ok(Self {
                mode,
                custom: None,
                live_background: None,
                blur_strength,
            });
        }
        let path = custom_media_path
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .ok_or_else(|| "custom native camera background requires a media path".to_string())?;
        let media_kind = media_kind
            .ok_or_else(|| "custom native camera background requires a media kind".to_string())?;
        Ok(Self {
            mode,
            custom: Some(CameraBackgroundCustomSource {
                path: path.to_string(),
                media_kind,
            }),
            live_background: None,
            blur_strength,
        })
    }

    pub fn ensure_supported_for_publish(&self) -> Result<(), String> {
        if self.mode != CameraBackgroundMode::Custom {
            return Ok(());
        }
        let source = self
            .custom
            .as_ref()
            .ok_or_else(|| "custom native camera background requires a media source".to_string())?;
        match source.media_kind {
            CameraBackgroundCustomMediaKind::Static => validate_static_background_source(source),
            CameraBackgroundCustomMediaKind::Animated => {
                validate_timed_background_source(source, ANIMATED_BACKGROUND_EXTENSIONS, "animated")
            }
            CameraBackgroundCustomMediaKind::Video => {
                validate_timed_background_source(source, VIDEO_BACKGROUND_EXTENSIONS, "video")
            }
        }
    }
}

enum CustomBackgroundSource {
    Static(crate::yuv::I420),
    Animated(AnimatedBackground),
    Live(CameraBackgroundLiveSlot),
}

struct AnimatedBackground {
    frames: Vec<crate::yuv::I420>,
    schedule_end_us: Vec<u64>,
    anchor_timestamp_us: Option<i64>,
}

impl AnimatedBackground {
    fn new(frames_with_delays: Vec<(crate::yuv::I420, u64)>) -> Option<Self> {
        if frames_with_delays.is_empty() {
            return None;
        }
        assert!(frames_with_delays.len() <= CUSTOM_BACKGROUND_ANIMATION_FRAMES_MAX);
        let mut frames = Vec::with_capacity(frames_with_delays.len());
        let mut schedule_end_us = Vec::with_capacity(frames_with_delays.len());
        let mut total_us: u64 = 0;
        for (frame, delay_ms) in frames_with_delays {
            total_us = total_us.saturating_add(clamp_animation_delay_ms(delay_ms) * 1_000);
            frames.push(frame);
            schedule_end_us.push(total_us);
        }
        assert_eq!(frames.len(), schedule_end_us.len());
        assert!(total_us > 0);
        Some(Self {
            frames,
            schedule_end_us,
            anchor_timestamp_us: None,
        })
    }

    fn frame_at(&mut self, timestamp_us: i64) -> &crate::yuv::I420 {
        assert!(!self.frames.is_empty());
        assert_eq!(self.frames.len(), self.schedule_end_us.len());
        let anchor = *self.anchor_timestamp_us.get_or_insert(timestamp_us);
        let elapsed_us = timestamp_us.saturating_sub(anchor).max(0) as u64;
        let index = animation_frame_index(&self.schedule_end_us, elapsed_us);
        &self.frames[index]
    }
}

fn clamp_animation_delay_ms(delay_ms: u64) -> u64 {
    if delay_ms == 0 {
        return CUSTOM_BACKGROUND_FRAME_DELAY_MS_DEFAULT;
    }
    delay_ms.clamp(
        CUSTOM_BACKGROUND_FRAME_DELAY_MS_MIN,
        CUSTOM_BACKGROUND_FRAME_DELAY_MS_MAX,
    )
}

fn animation_frame_index(schedule_end_us: &[u64], elapsed_us: u64) -> usize {
    assert!(!schedule_end_us.is_empty());
    assert!(schedule_end_us.len() <= CUSTOM_BACKGROUND_ANIMATION_FRAMES_MAX);
    let total_us = *schedule_end_us.last().unwrap();
    assert!(total_us > 0);
    let looped_us = elapsed_us % total_us;
    schedule_end_us
        .iter()
        .position(|end_us| looped_us < *end_us)
        .unwrap_or(schedule_end_us.len() - 1)
}

pub struct CameraBackgroundTransform {
    config: CameraBackgroundConfig,
    width: u32,
    height: u32,
    radius_pass_luma: usize,
    radius_pass_chroma: usize,
    custom: Option<CustomBackgroundSource>,
    mask_source: Option<Box<dyn crate::person_segmentation::PersonMaskSource>>,
    person_mask: Vec<u8>,
    person_mask_scratch: Vec<u8>,
    refiner: crate::mask_refine::MaskRefiner,
    blur_scratch: BlurScratch,
}

impl CameraBackgroundTransform {
    pub fn new(config: CameraBackgroundConfig, width: u32, height: u32) -> Result<Self, String> {
        let mask_source = if config.mode == CameraBackgroundMode::None {
            None
        } else {
            crate::person_segmentation::create_person_mask_source(width, height)
        };
        Self::with_mask_source(config, width, height, mask_source)
    }

    pub fn with_mask_source(
        config: CameraBackgroundConfig,
        width: u32,
        height: u32,
        mask_source: Option<Box<dyn crate::person_segmentation::PersonMaskSource>>,
    ) -> Result<Self, String> {
        assert!(width >= 2);
        assert!(height >= 2);
        if width as usize > FRAME_EDGE_MAX || height as usize > FRAME_EDGE_MAX {
            return Err("camera frame dimensions exceed the background transform cap".to_string());
        }
        let radius = scaled_blur_radius(blur_radius(width, height), config.blur_strength);
        assert!(radius > 0);
        let radius_pass_luma = blur_pass_radius(radius);
        let radius_pass_chroma = (radius_pass_luma / 2).max(1);
        let custom = load_custom_background_source(&config, width, height)?;
        let plane_len = (width as usize) * (height as usize);
        let mut person_mask = vec![crate::person_segmentation::PERSON_MASK_BACKGROUND; plane_len];
        fill_portrait_ellipse_mask(&mut person_mask, width as usize, height as usize);
        Ok(Self {
            config,
            width,
            height,
            radius_pass_luma,
            radius_pass_chroma,
            custom,
            mask_source,
            person_mask,
            person_mask_scratch: vec![0; plane_len],
            refiner: crate::mask_refine::MaskRefiner::new(width as usize, height as usize),
            blur_scratch: BlurScratch::new(width as usize, height as usize),
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn refresh_person_mask(&mut self, frame: &crate::yuv::I420) {
        let Some(source) = self.mask_source.as_mut() else {
            return;
        };
        let plane_len = (self.width as usize) * (self.height as usize);
        assert_eq!(self.person_mask.len(), plane_len);
        assert_eq!(self.person_mask_scratch.len(), plane_len);
        if source.mask_into(frame, &mut self.person_mask_scratch) {
            self.refiner.refine(&frame.y, &mut self.person_mask_scratch);
            std::mem::swap(&mut self.person_mask, &mut self.person_mask_scratch);
        }
    }

    pub fn apply_i420(&mut self, frame: &mut crate::yuv::I420, timestamp_us: i64) -> bool {
        assert!(frame.width >= 2);
        assert!(frame.height >= 2);
        if self.config.mode == CameraBackgroundMode::None {
            return true;
        }
        if frame.width != self.width || frame.height != self.height {
            return false;
        }
        self.refresh_person_mask(frame);
        if self.config.mode == CameraBackgroundMode::Custom {
            let Some(custom) = self.custom.as_mut() else {
                return false;
            };
            return apply_custom_background(frame, custom, &self.person_mask, timestamp_us);
        }
        blur_i420_background(
            frame,
            &self.person_mask,
            self.radius_pass_luma,
            self.radius_pass_chroma,
            &mut self.blur_scratch,
        )
    }
}

fn apply_custom_background(
    frame: &mut crate::yuv::I420,
    custom: &mut CustomBackgroundSource,
    mask: &[u8],
    timestamp_us: i64,
) -> bool {
    assert!(frame.width >= 2);
    assert!(frame.height >= 2);
    match custom {
        CustomBackgroundSource::Static(background) => {
            composite_i420_background(frame, background, mask);
            true
        }
        CustomBackgroundSource::Animated(animated) => {
            let background = animated.frame_at(timestamp_us);
            composite_i420_background(frame, background, mask);
            true
        }
        CustomBackgroundSource::Live(slot) => {
            slot.with_latest(|background| {
                if let Some(background) = background
                    && background.width == frame.width
                    && background.height == frame.height
                {
                    composite_i420_background(frame, background, mask);
                }
            });
            true
        }
    }
}

type CameraBackgroundTransformFactory =
    fn(CameraBackgroundConfig, u32, u32) -> Result<CameraBackgroundTransform, String>;

pub struct CameraBackgroundStage {
    config: CameraBackgroundConfig,
    factory: CameraBackgroundTransformFactory,
    transform: Option<CameraBackgroundTransform>,
    attempted_dims: Option<(u32, u32)>,
}

impl CameraBackgroundStage {
    pub fn new(config: CameraBackgroundConfig) -> Self {
        Self::with_factory(config, CameraBackgroundTransform::new)
    }

    pub fn with_factory(
        config: CameraBackgroundConfig,
        factory: CameraBackgroundTransformFactory,
    ) -> Self {
        Self {
            config,
            factory,
            transform: None,
            attempted_dims: None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.mode != CameraBackgroundMode::None
    }

    pub fn apply_i420(&mut self, frame: &mut crate::yuv::I420, timestamp_us: i64) -> bool {
        assert!(frame.width >= 2);
        assert!(frame.height >= 2);
        if !self.is_enabled() {
            return true;
        }
        let dims = (frame.width, frame.height);
        let needs_transform = match self.transform.as_ref() {
            Some(transform) => transform.dimensions() != dims,
            None => true,
        };
        if needs_transform {
            if self.attempted_dims == Some(dims) {
                return true;
            }
            self.attempted_dims = Some(dims);
            self.transform = (self.factory)(self.config.clone(), dims.0, dims.1).ok();
        }
        let Some(transform) = self.transform.as_mut() else {
            return true;
        };
        if transform.dimensions() != dims {
            return true;
        }
        transform.apply_i420(frame, timestamp_us)
    }
}

fn validate_static_background_source(source: &CameraBackgroundCustomSource) -> Result<(), String> {
    assert!(source.media_kind == CameraBackgroundCustomMediaKind::Static);
    validate_background_file(source, STATIC_BACKGROUND_EXTENSIONS, "static")?;
    let reader = ImageReader::open(&source.path)
        .map_err(|_| "static native camera background is not readable".to_string())?;
    let reader = reader
        .with_guessed_format()
        .map_err(|_| "static native camera background format is invalid".to_string())?;
    let dimensions = reader
        .into_dimensions()
        .map_err(|_| "static native camera background dimensions are invalid".to_string())?;
    if dimensions.0 < 2 || dimensions.1 < 2 {
        return Err("static native camera background dimensions are too small".to_string());
    }
    Ok(())
}

fn validate_timed_background_source(
    source: &CameraBackgroundCustomSource,
    extensions: &[&str],
    label: &str,
) -> Result<(), String> {
    assert!(!extensions.is_empty());
    assert!(!label.trim().is_empty());
    validate_background_file(source, extensions, label)
}

fn validate_background_file(
    source: &CameraBackgroundCustomSource,
    extensions: &[&str],
    label: &str,
) -> Result<(), String> {
    assert!(!extensions.is_empty());
    assert!(!label.trim().is_empty());
    let metadata = std::fs::metadata(&source.path)
        .map_err(|_| format!("{label} native camera background is not readable"))?;
    if !metadata.is_file() {
        return Err(format!("{label} native camera background is not a file"));
    }
    if metadata.len() == 0 || metadata.len() > MAX_CUSTOM_BACKGROUND_BYTES {
        return Err(format!("{label} native camera background size is invalid"));
    }
    let extension = std::path::Path::new(&source.path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !extensions.iter().any(|candidate| *candidate == extension) {
        return Err(format!(
            "{label} native camera background type is unsupported"
        ));
    }
    Ok(())
}

fn load_custom_background_source(
    config: &CameraBackgroundConfig,
    width: u32,
    height: u32,
) -> Result<Option<CustomBackgroundSource>, String> {
    if config.mode != CameraBackgroundMode::Custom {
        return Ok(None);
    }
    let source = config
        .custom
        .as_ref()
        .ok_or_else(|| "custom native camera background requires a media source".to_string())?;
    match source.media_kind {
        CameraBackgroundCustomMediaKind::Video => {
            let slot = config.live_background.clone().unwrap_or_default();
            slot.set_target_dims(width, height);
            Ok(Some(CustomBackgroundSource::Live(slot)))
        }
        CameraBackgroundCustomMediaKind::Animated => {
            let frames = decode_animated_background_frames(
                &source.path,
                width,
                height,
                CUSTOM_BACKGROUND_ANIMATION_FRAMES_MAX,
                CUSTOM_BACKGROUND_ANIMATION_BYTES_MAX,
            );
            if let Some(animated) = AnimatedBackground::new(frames) {
                return Ok(Some(CustomBackgroundSource::Animated(animated)));
            }
            Ok(Some(CustomBackgroundSource::Static(
                decode_static_background_i420(&source.path, width, height)?,
            )))
        }
        CameraBackgroundCustomMediaKind::Static => Ok(Some(CustomBackgroundSource::Static(
            decode_static_background_i420(&source.path, width, height)?,
        ))),
    }
}

fn decode_static_background_i420(
    path: &str,
    width: u32,
    height: u32,
) -> Result<crate::yuv::I420, String> {
    let image = ImageReader::open(path)
        .map_err(|_| "custom native camera background is not readable".to_string())?
        .with_guessed_format()
        .map_err(|_| "custom native camera background format is invalid".to_string())?
        .decode()
        .map_err(|_| "custom native camera background decode failed".to_string())?;
    let resized = image.resize_exact(width, height, FilterType::Triangle);
    crate::yuv::rgb_to_i420(&resized.to_rgb8().into_raw(), width, height)
        .ok_or_else(|| "custom native camera background conversion failed".to_string())
}

fn open_animation_frames(path: &str) -> Option<image::Frames<'static>> {
    use image::codecs::gif::GifDecoder;
    use image::codecs::webp::WebPDecoder;
    let format = ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .format()?;
    let reader = std::io::BufReader::new(std::fs::File::open(path).ok()?);
    match format {
        image::ImageFormat::Gif => Some(GifDecoder::new(reader).ok()?.into_frames()),
        image::ImageFormat::WebP => {
            let decoder = WebPDecoder::new(reader).ok()?;
            if !decoder.has_animation() {
                return None;
            }
            Some(decoder.into_frames())
        }
        _ => None,
    }
}

fn animation_frame_delay_ms(delay: image::Delay) -> u64 {
    let (numer, denom) = delay.numer_denom_ms();
    if denom == 0 {
        return CUSTOM_BACKGROUND_FRAME_DELAY_MS_DEFAULT;
    }
    clamp_animation_delay_ms((u64::from(numer) + u64::from(denom) / 2) / u64::from(denom))
}

fn decode_animated_background_frames(
    path: &str,
    width: u32,
    height: u32,
    frames_max: usize,
    bytes_max: usize,
) -> Vec<(crate::yuv::I420, u64)> {
    assert!(frames_max >= 1);
    assert!(frames_max <= CUSTOM_BACKGROUND_ANIMATION_FRAMES_MAX);
    assert!(bytes_max >= 1);
    let Some(frames) = open_animation_frames(path) else {
        return Vec::new();
    };
    let mut decoded = Vec::new();
    let mut decoded_bytes: usize = 0;
    for frame in frames {
        let Ok(frame) = frame else {
            break;
        };
        let delay_ms = animation_frame_delay_ms(frame.delay());
        let buffer = frame.into_buffer();
        decoded_bytes = decoded_bytes.saturating_add(buffer.as_raw().len());
        if decoded_bytes > bytes_max {
            break;
        }
        let resized = image::DynamicImage::ImageRgba8(buffer)
            .resize_exact(width, height, FilterType::Triangle)
            .to_rgb8()
            .into_raw();
        let Some(i420) = crate::yuv::rgb_to_i420(&resized, width, height) else {
            continue;
        };
        decoded.push((i420, delay_ms));
        if decoded.len() >= frames_max {
            break;
        }
    }
    assert!(decoded.len() <= frames_max);
    decoded
}

fn blur_radius(width: u32, height: u32) -> usize {
    let radius = width.min(height) / 32;
    radius.clamp(4, 24) as usize
}

const _: () = assert!(
    24 * (BLUR_STRENGTH_FACTOR_PERMILLE_MAX as usize)
        / (BLUR_STRENGTH_FACTOR_PERMILLE_SCALE as usize)
        == BLUR_RADIUS_SCALED_MAX
);

fn blur_strength_factor_permille(strength: u32) -> u32 {
    assert!(strength <= CAMERA_EFFECT_STRENGTH_MAX);
    if strength <= CAMERA_EFFECT_STRENGTH_DEFAULT {
        let span = BLUR_STRENGTH_FACTOR_PERMILLE_MID - BLUR_STRENGTH_FACTOR_PERMILLE_MIN;
        BLUR_STRENGTH_FACTOR_PERMILLE_MIN + span * strength / CAMERA_EFFECT_STRENGTH_DEFAULT
    } else {
        let span = BLUR_STRENGTH_FACTOR_PERMILLE_MAX - BLUR_STRENGTH_FACTOR_PERMILLE_MID;
        let above_default = strength - CAMERA_EFFECT_STRENGTH_DEFAULT;
        BLUR_STRENGTH_FACTOR_PERMILLE_MID
            + span * above_default / (CAMERA_EFFECT_STRENGTH_MAX - CAMERA_EFFECT_STRENGTH_DEFAULT)
    }
}

pub(crate) fn scaled_blur_radius(base_radius: usize, strength: u32) -> usize {
    assert!(base_radius >= BLUR_RADIUS_SCALED_MIN);
    assert!(base_radius <= BLUR_RADIUS_SCALED_MAX);
    let strength = strength.min(CAMERA_EFFECT_STRENGTH_MAX);
    let factor = blur_strength_factor_permille(strength);
    assert!(factor >= BLUR_STRENGTH_FACTOR_PERMILLE_MIN);
    assert!(factor <= BLUR_STRENGTH_FACTOR_PERMILLE_MAX);
    let scaled = base_radius * (factor as usize) / (BLUR_STRENGTH_FACTOR_PERMILLE_SCALE as usize);
    scaled.clamp(BLUR_RADIUS_SCALED_MIN, BLUR_RADIUS_SCALED_MAX)
}

#[derive(Clone, Copy)]
pub(crate) struct PlaneMask<'mask> {
    values: &'mask [u8],
    width: usize,
    scale: usize,
}

pub(crate) fn plane_mask(mask: &[u8], frame_width: usize, scale: usize) -> PlaneMask<'_> {
    assert!(scale == 1 || scale == 2);
    assert!(frame_width >= 1);
    PlaneMask {
        values: mask,
        width: frame_width,
        scale,
    }
}

pub(crate) fn blur_pass_radius(scaled_radius: usize) -> usize {
    assert!(scaled_radius >= BLUR_RADIUS_SCALED_MIN);
    assert!(scaled_radius <= BLUR_RADIUS_SCALED_MAX);
    (scaled_radius * BLUR_PASS_RADIUS_NUMERATOR / BLUR_PASS_RADIUS_DENOMINATOR).max(1)
}

pub(crate) struct BlurScratch {
    value: Vec<f32>,
    value_alternate: Vec<f32>,
    weight: Vec<f32>,
    weight_alternate: Vec<f32>,
    box_scratch: Vec<f32>,
    column_fixed: Vec<u32>,
}

impl BlurScratch {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        assert!(width >= 2);
        assert!(height >= 2);
        let low_len = width.div_ceil(BLUR_DOWNSAMPLE) * height.div_ceil(BLUR_DOWNSAMPLE);
        Self {
            value: vec![0.0; low_len],
            value_alternate: vec![0.0; low_len],
            weight: vec![0.0; low_len],
            weight_alternate: vec![0.0; low_len],
            box_scratch: vec![0.0; low_len],
            column_fixed: vec![0; width],
        }
    }
}

pub(crate) fn blur_i420_background(
    frame: &mut crate::yuv::I420,
    mask: &[u8],
    radius_pass_luma: usize,
    radius_pass_chroma: usize,
    scratch: &mut BlurScratch,
) -> bool {
    assert!(radius_pass_luma >= 1);
    assert!(radius_pass_chroma >= 1);
    let width = frame.width as usize;
    let height = frame.height as usize;
    if frame.y.len() != width * height {
        return false;
    }
    if frame.u.len() != (width / 2) * (height / 2) || frame.v.len() != frame.u.len() {
        return false;
    }
    let luma_mask = plane_mask(mask, width, 1);
    let chroma_mask = plane_mask(mask, width, 2);
    blur_plane_masked(
        &mut frame.y,
        width,
        height,
        luma_mask,
        radius_pass_luma,
        scratch,
    );
    let chroma_width = width / 2;
    let chroma_height = height / 2;
    blur_plane_masked(
        &mut frame.u,
        chroma_width,
        chroma_height,
        chroma_mask,
        radius_pass_chroma,
        scratch,
    );
    blur_plane_masked(
        &mut frame.v,
        chroma_width,
        chroma_height,
        chroma_mask,
        radius_pass_chroma,
        scratch,
    );
    true
}

pub(crate) fn blur_plane_masked(
    plane: &mut [u8],
    plane_width: usize,
    plane_height: usize,
    mask: PlaneMask<'_>,
    radius_pass: usize,
    scratch: &mut BlurScratch,
) {
    assert!(plane_width >= 1);
    assert!(plane_height >= 1);
    assert_eq!(plane.len(), plane_width * plane_height);
    assert!(radius_pass >= 1);
    let low_width = plane_width.div_ceil(BLUR_DOWNSAMPLE);
    let low_height = plane_height.div_ceil(BLUR_DOWNSAMPLE);
    assert!(scratch.value.len() >= low_width * low_height);
    downsample_masked_plane(plane, plane_width, plane_height, mask, scratch);
    for _ in 0..BLUR_BOX_PASSES {
        crate::mask_refine::box_filter_low(
            &scratch.value,
            &mut scratch.box_scratch,
            &mut scratch.value_alternate,
            low_width,
            low_height,
            radius_pass,
        );
        std::mem::swap(&mut scratch.value, &mut scratch.value_alternate);
        crate::mask_refine::box_filter_low(
            &scratch.weight,
            &mut scratch.box_scratch,
            &mut scratch.weight_alternate,
            low_width,
            low_height,
            radius_pass,
        );
        std::mem::swap(&mut scratch.weight, &mut scratch.weight_alternate);
    }
    normalize_blurred_background(scratch, low_width * low_height);
    composite_blurred_background(
        plane,
        plane_width,
        plane_height,
        mask,
        low_width,
        low_height,
        scratch,
    );
}

fn normalize_blurred_background(scratch: &mut BlurScratch, low_len: usize) {
    assert!(scratch.value.len() >= low_len);
    assert!(scratch.weight.len() >= low_len);
    for (value, weight) in scratch.value[..low_len]
        .iter_mut()
        .zip(scratch.weight[..low_len].iter())
    {
        if *weight > BLUR_WEIGHT_EPSILON {
            *value = (*value / *weight).clamp(0.0, 255.0);
        } else {
            *value = 0.0;
        }
    }
}

fn downsample_masked_plane(
    plane: &[u8],
    plane_width: usize,
    plane_height: usize,
    mask: PlaneMask<'_>,
    scratch: &mut BlurScratch,
) {
    let low_width = plane_width.div_ceil(BLUR_DOWNSAMPLE);
    let low_height = plane_height.div_ceil(BLUR_DOWNSAMPLE);
    assert!(scratch.value.len() >= low_width * low_height);
    assert!(scratch.weight.len() >= low_width * low_height);
    for low_y in 0..low_height {
        let y_start = low_y * BLUR_DOWNSAMPLE;
        let y_end = (y_start + BLUR_DOWNSAMPLE).min(plane_height);
        for low_x in 0..low_width {
            let x_start = low_x * BLUR_DOWNSAMPLE;
            let x_end = (x_start + BLUR_DOWNSAMPLE).min(plane_width);
            let mut value_sum: f32 = 0.0;
            let mut weight_sum: f32 = 0.0;
            for y in y_start..y_end {
                let row = y * plane_width;
                let mask_row = y * mask.scale * mask.width;
                for x in x_start..x_end {
                    let person = mask.values[mask_row + x * mask.scale];
                    let weight = f32::from(255 - person);
                    value_sum += f32::from(plane[row + x]) * weight;
                    weight_sum += weight;
                }
            }
            let low_offset = low_y * low_width + low_x;
            scratch.value[low_offset] = value_sum;
            scratch.weight[low_offset] = weight_sum;
        }
    }
}

fn composite_blurred_background(
    plane: &mut [u8],
    plane_width: usize,
    plane_height: usize,
    mask: PlaneMask<'_>,
    low_width: usize,
    low_height: usize,
    scratch: &mut BlurScratch,
) {
    assert!(low_width >= 1);
    assert!(low_height >= 1);
    assert!(scratch.column_fixed.len() >= plane_width);
    for (x, slot) in scratch.column_fixed[..plane_width].iter_mut().enumerate() {
        *slot = crate::mask_refine::bilinear_fixed_coord(x, plane_width, low_width);
    }
    for y in 0..plane_height {
        let row_fixed = crate::mask_refine::bilinear_fixed_coord(y, plane_height, low_height);
        let sy = (row_fixed / 256) as usize;
        let fy = (row_fixed % 256) as f32 / 256.0;
        let sy_next = (sy + 1).min(low_height - 1);
        let row = y * plane_width;
        let mask_row = y * mask.scale * mask.width;
        for x in 0..plane_width {
            let person = mask.values[mask_row + x * mask.scale];
            if person == crate::person_segmentation::PERSON_MASK_PERSON {
                continue;
            }
            let col_fixed = scratch.column_fixed[x];
            let sx = (col_fixed / 256) as usize;
            let fx = (col_fixed % 256) as f32 / 256.0;
            let sx_next = (sx + 1).min(low_width - 1);
            let value = crate::mask_refine::bilinear_sample(
                &scratch.value,
                low_width,
                sx,
                sx_next,
                sy,
                sy_next,
                fx,
                fy,
            );
            let background = value.clamp(0.0, 255.0) as u16;
            let alpha = u16::from(person);
            let offset = row + x;
            plane[offset] =
                ((u16::from(plane[offset]) * alpha + background * (255 - alpha)) / 255) as u8;
        }
    }
}

fn composite_i420_background(frame: &mut crate::yuv::I420, custom: &crate::yuv::I420, mask: &[u8]) {
    assert_eq!(frame.width, custom.width);
    assert_eq!(frame.height, custom.height);
    let width = frame.width as usize;
    let height = frame.height as usize;
    let luma_mask = plane_mask(mask, width, 1);
    let chroma_mask = plane_mask(mask, width, 2);
    composite_masked_plane(&mut frame.y, &custom.y, width, height, luma_mask);
    composite_masked_plane(&mut frame.u, &custom.u, width / 2, height / 2, chroma_mask);
    composite_masked_plane(&mut frame.v, &custom.v, width / 2, height / 2, chroma_mask);
}

pub(crate) fn composite_masked_plane(
    plane: &mut [u8],
    background: &[u8],
    width: usize,
    height: usize,
    mask: PlaneMask<'_>,
) {
    assert!(width >= 1);
    assert!(height >= 1);
    assert_eq!(plane.len(), width * height);
    assert_eq!(background.len(), width * height);
    assert!(mask.scale == 1 || mask.scale == 2);
    assert!(mask.width >= width * mask.scale);
    assert!(mask.values.len() >= mask.width * height * mask.scale);

    for y in 0..height {
        let mask_row = y * mask.scale * mask.width;
        for x in 0..width {
            let person = u16::from(mask.values[mask_row + x * mask.scale]);
            if person == u16::from(crate::person_segmentation::PERSON_MASK_PERSON) {
                continue;
            }
            let offset = y * width + x;
            if person == u16::from(crate::person_segmentation::PERSON_MASK_BACKGROUND) {
                plane[offset] = background[offset];
                continue;
            }
            let background_alpha = 255 - person;
            plane[offset] = ((u16::from(plane[offset]) * person
                + u16::from(background[offset]) * background_alpha)
                / 255) as u8;
        }
    }
}

pub(crate) fn fill_portrait_ellipse_mask(mask: &mut [u8], width: usize, height: usize) {
    assert!(width >= 1);
    assert!(height >= 1);
    assert!(mask.len() >= width * height);
    let cx = (width as i64) / 2;
    let cy = ((height as i64) * 9) / 20;
    let rx = ((width as i64) * 3).max(1) / 10;
    let ry = ((height as i64) * 9).max(1) / 20;
    let inner = PORTRAIT_ELLIPSE_INNER_PERMILLE;
    let outer = PORTRAIT_ELLIPSE_OUTER_PERMILLE;
    assert!(outer > inner);

    for y in 0..height {
        let dy = ((y as i64 - cy) * 1000) / ry.max(1);
        for x in 0..width {
            let dx = ((x as i64 - cx) * 1000) / rx.max(1);
            let distance = (dx * dx + dy * dy) / 1000;
            let offset = y * width + x;
            if distance <= inner {
                mask[offset] = crate::person_segmentation::PERSON_MASK_PERSON;
            } else if distance >= outer {
                mask[offset] = crate::person_segmentation::PERSON_MASK_BACKGROUND;
            } else {
                let ramp = (distance - inner) * 255 / (outer - inner);
                mask[offset] = (255 - ramp) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::person_segmentation::{
        PERSON_MASK_BACKGROUND, PERSON_MASK_PERSON, PersonMaskSource,
    };
    use image::{ImageBuffer, Rgb};
    use std::path::PathBuf;

    struct LeftHalfPersonMask {
        responses: Vec<bool>,
        calls: usize,
    }

    impl LeftHalfPersonMask {
        fn new(responses: Vec<bool>) -> Self {
            assert!(!responses.is_empty());
            Self {
                responses,
                calls: 0,
            }
        }
    }

    impl PersonMaskSource for LeftHalfPersonMask {
        fn mask_into(&mut self, frame: &crate::yuv::I420, mask: &mut [u8]) -> bool {
            assert!(self.calls < self.responses.len());
            let succeed = self.responses[self.calls];
            self.calls += 1;
            if !succeed {
                return false;
            }
            let width = frame.width as usize;
            let height = frame.height as usize;
            assert!(mask.len() >= width * height);
            for y in 0..height {
                for x in 0..width {
                    mask[y * width + x] = if x < width / 2 {
                        PERSON_MASK_PERSON
                    } else {
                        PERSON_MASK_BACKGROUND
                    };
                }
            }
            true
        }
    }

    fn blur_transform_with_mask(
        width: u32,
        height: u32,
        responses: Vec<bool>,
    ) -> CameraBackgroundTransform {
        CameraBackgroundTransform::with_mask_source(
            CameraBackgroundConfig {
                mode: CameraBackgroundMode::Blur,
                custom: None,
                live_background: None,
                ..CameraBackgroundConfig::default()
            },
            width,
            height,
            Some(Box::new(LeftHalfPersonMask::new(responses))),
        )
        .unwrap()
    }

    fn gradient_rgb(width: usize, height: usize) -> Vec<u8> {
        let mut frame = vec![0u8; width * height * 3];
        for y in 0..height {
            for x in 0..width {
                let offset = (y * width + x) * 3;
                frame[offset] = (x * 4).min(255) as u8;
                frame[offset + 1] = (y * 4).min(255) as u8;
                frame[offset + 2] = ((x + y) * 2).min(255) as u8;
            }
        }
        frame
    }

    fn gradient_i420(width: u32, height: u32) -> crate::yuv::I420 {
        let rgb = gradient_rgb(width as usize, height as usize);
        crate::yuv::rgb_to_i420(&rgb, width, height).unwrap()
    }

    fn write_temp_png(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("fluxer-camera-background-{name}.png"));
        let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(4, 4);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = Rgb([(x * 40) as u8, (y * 40) as u8, 128]);
        }
        image.save(&path).unwrap();
        path
    }

    fn write_temp_bytes(name: &str, extension: &str, bytes: &[u8]) -> PathBuf {
        assert!(!name.trim().is_empty());
        assert!(!extension.trim().is_empty());
        assert!(!bytes.is_empty());
        let path =
            std::env::temp_dir().join(format!("fluxer-camera-background-{name}.{extension}"));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn none_mode_leaves_frame_unchanged() {
        let mut transform = CameraBackgroundTransform::new(
            CameraBackgroundConfig {
                mode: CameraBackgroundMode::None,
                custom: None,
                live_background: None,
                ..CameraBackgroundConfig::default()
            },
            32,
            24,
        )
        .unwrap();
        let mut frame = gradient_i420(32, 24);
        let original = frame.clone();

        assert!(transform.apply_i420(&mut frame, 0));

        assert_eq!(frame.y, original.y);
        assert_eq!(frame.u, original.u);
        assert_eq!(frame.v, original.v);
    }

    #[test]
    fn blur_rejects_dimension_changes_without_allocating() {
        let mut transform = CameraBackgroundTransform::with_mask_source(
            CameraBackgroundConfig {
                mode: CameraBackgroundMode::Blur,
                custom: None,
                live_background: None,
                ..CameraBackgroundConfig::default()
            },
            64,
            48,
            None,
        )
        .unwrap();
        let mut frame = gradient_i420(32, 24);

        assert!(!transform.apply_i420(&mut frame, 0));
    }

    #[test]
    fn bridge_values_accept_none_alias_blur_and_custom_media() {
        let none =
            CameraBackgroundConfig::from_bridge_values(Some("non"), None, None, None).unwrap();
        assert_eq!(none.mode, CameraBackgroundMode::None);

        let blur =
            CameraBackgroundConfig::from_bridge_values(Some("blur"), None, None, None).unwrap();
        assert_eq!(blur.mode, CameraBackgroundMode::Blur);

        let custom = CameraBackgroundConfig::from_bridge_values(
            Some("custom"),
            Some("/tmp/background.webm"),
            Some("video"),
            None,
        )
        .unwrap();
        assert_eq!(custom.mode, CameraBackgroundMode::Custom);
        assert_eq!(
            custom.custom.as_ref().map(|source| &source.media_kind),
            Some(&CameraBackgroundCustomMediaKind::Video)
        );
    }

    #[test]
    fn custom_background_requires_media_path_and_accepts_timed_media_files() {
        assert!(
            CameraBackgroundConfig::from_bridge_values(Some("custom"), None, Some("static"), None,)
                .is_err()
        );
        let gif_path = write_temp_bytes("animated-custom", "gif", b"GIF89a");
        let mp4_path = write_temp_bytes("video-custom", "mp4", b"\0\0\0\x18ftypmp42");
        let animated = CameraBackgroundConfig::from_bridge_values(
            Some("custom"),
            gif_path.to_str(),
            Some("animated"),
            None,
        )
        .unwrap();
        let video = CameraBackgroundConfig::from_bridge_values(
            Some("custom"),
            mp4_path.to_str(),
            Some("video"),
            None,
        )
        .unwrap();

        assert!(animated.ensure_supported_for_publish().is_ok());
        assert!(video.ensure_supported_for_publish().is_ok());
        let _ = std::fs::remove_file(gif_path);
        let _ = std::fs::remove_file(mp4_path);
    }

    #[test]
    fn custom_background_rejects_unsupported_extensions() {
        let path = write_temp_bytes("unsupported-custom", "webm", b"not-webm");
        let video = CameraBackgroundConfig::from_bridge_values(
            Some("custom"),
            path.to_str(),
            Some("video"),
            None,
        )
        .unwrap();

        let error = video.ensure_supported_for_publish().unwrap_err();

        assert!(error.contains("type is unsupported"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn blur_mode_transforms_i420_without_rgb_decode() {
        let mut transform = CameraBackgroundTransform::with_mask_source(
            CameraBackgroundConfig {
                mode: CameraBackgroundMode::Blur,
                custom: None,
                live_background: None,
                ..CameraBackgroundConfig::default()
            },
            64,
            48,
            None,
        )
        .unwrap();
        let mut frame = gradient_i420(64, 48);
        let original = frame.clone();
        let center = 22 * 64 + 32;
        let corner = 0usize;

        assert!(transform.apply_i420(&mut frame, 0));

        assert_eq!(frame.y[center], original.y[center]);
        assert_ne!(frame.y[corner], original.y[corner]);
    }

    #[test]
    fn static_custom_background_composites_i420_from_predecoded_image() {
        let path = write_temp_png("static-custom-i420");
        let custom = CameraBackgroundConfig::from_bridge_values(
            Some("custom"),
            path.to_str(),
            Some("static"),
            None,
        )
        .unwrap();
        let mut transform =
            CameraBackgroundTransform::with_mask_source(custom, 64, 48, None).unwrap();
        let mut frame = crate::yuv::I420 {
            width: 64,
            height: 48,
            y: vec![235; 64 * 48],
            u: vec![128; 32 * 24],
            v: vec![128; 32 * 24],
        };
        let center = 22 * 64 + 32;
        let corner = 0usize;

        assert!(transform.apply_i420(&mut frame, 0));

        assert_eq!(frame.y[center], 235);
        assert_ne!(frame.y[corner], 235);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn person_mask_keeps_person_pixels_sharp_regardless_of_frame_position() {
        let mut transform = blur_transform_with_mask(64, 48, vec![true]);
        let mut frame = gradient_i420(64, 48);
        let original = frame.clone();
        let person_corner = 0usize;
        let background_corner = 63usize;

        assert!(transform.apply_i420(&mut frame, 0));

        assert_eq!(frame.y[person_corner], original.y[person_corner]);
        assert_ne!(frame.y[background_corner], original.y[background_corner]);
    }

    #[test]
    fn person_mask_drives_chroma_planes_at_half_resolution() {
        let mut transform = blur_transform_with_mask(64, 48, vec![true]);
        let mut frame = gradient_i420(64, 48);
        let original = frame.clone();
        let person_chroma = 0usize;
        let background_chroma = 31usize;

        assert!(transform.apply_i420(&mut frame, 0));

        assert_eq!(frame.u[person_chroma], original.u[person_chroma]);
        assert_ne!(frame.u[background_chroma], original.u[background_chroma]);
    }

    #[test]
    fn person_mask_failure_falls_back_to_portrait_ellipse() {
        let mut transform = blur_transform_with_mask(64, 48, vec![false]);
        let mut frame = gradient_i420(64, 48);
        let original = frame.clone();
        let portrait_center = 22 * 64 + 32;
        let corner = 0usize;

        assert!(transform.apply_i420(&mut frame, 0));

        assert_eq!(frame.y[portrait_center], original.y[portrait_center]);
        assert_ne!(frame.y[corner], original.y[corner]);
    }

    #[test]
    fn person_mask_failure_reuses_previous_valid_mask() {
        let mut transform = blur_transform_with_mask(64, 48, vec![true, false]);
        let mut first = gradient_i420(64, 48);
        assert!(transform.apply_i420(&mut first, 0));
        let mut second = gradient_i420(64, 48);
        let original = second.clone();
        let person_corner = 0usize;
        let background_corner = 63usize;

        assert!(transform.apply_i420(&mut second, 0));

        assert_eq!(second.y[person_corner], original.y[person_corner]);
        assert_ne!(second.y[background_corner], original.y[background_corner]);
    }

    #[test]
    fn person_mask_composites_custom_background_only_outside_person() {
        let path = write_temp_png("masked-custom-i420");
        let custom = CameraBackgroundConfig::from_bridge_values(
            Some("custom"),
            path.to_str(),
            Some("static"),
            None,
        )
        .unwrap();
        let mut transform = CameraBackgroundTransform::with_mask_source(
            custom,
            64,
            48,
            Some(Box::new(LeftHalfPersonMask::new(vec![true]))),
        )
        .unwrap();
        let mut frame = crate::yuv::I420 {
            width: 64,
            height: 48,
            y: vec![235; 64 * 48],
            u: vec![128; 32 * 24],
            v: vec![128; 32 * 24],
        };
        let person_corner = 0usize;
        let background_corner = 63usize;

        assert!(transform.apply_i420(&mut frame, 0));

        assert_eq!(frame.y[person_corner], 235);
        assert_ne!(frame.y[background_corner], 235);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn animated_custom_background_uses_first_decoded_frame() {
        let path = write_temp_png("animated-first-frame");
        let renamed = path.with_extension("gif");
        let _ = std::fs::remove_file(&renamed);
        let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(4, 4);
        for pixel in image.pixels_mut() {
            *pixel = Rgb([10, 20, 30]);
        }
        image.save(&renamed).unwrap();
        let custom = CameraBackgroundConfig::from_bridge_values(
            Some("custom"),
            renamed.to_str(),
            Some("animated"),
            None,
        )
        .unwrap();

        let transform = CameraBackgroundTransform::with_mask_source(custom, 64, 48, None);

        assert!(transform.is_ok());
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(renamed);
    }

    #[test]
    fn stage_degrades_to_raw_frames_when_transform_creation_fails() {
        let mut stage = CameraBackgroundStage::with_factory(
            CameraBackgroundConfig {
                mode: CameraBackgroundMode::Custom,
                custom: Some(CameraBackgroundCustomSource {
                    path: "/nonexistent/background.png".to_string(),
                    media_kind: CameraBackgroundCustomMediaKind::Static,
                }),
                live_background: None,
                ..CameraBackgroundConfig::default()
            },
            CameraBackgroundTransform::new,
        );
        let mut frame = gradient_i420(64, 48);
        let original = frame.clone();

        assert!(stage.apply_i420(&mut frame, 0));
        assert!(stage.apply_i420(&mut frame, 0));

        assert_eq!(frame.y, original.y);
    }

    #[test]
    fn stage_recreates_transform_when_frame_dimensions_change() {
        fn factory_without_mask(
            config: CameraBackgroundConfig,
            width: u32,
            height: u32,
        ) -> Result<CameraBackgroundTransform, String> {
            CameraBackgroundTransform::with_mask_source(config, width, height, None)
        }
        let mut stage = CameraBackgroundStage::with_factory(
            CameraBackgroundConfig {
                mode: CameraBackgroundMode::Blur,
                custom: None,
                live_background: None,
                ..CameraBackgroundConfig::default()
            },
            factory_without_mask,
        );
        let mut first = gradient_i420(64, 48);
        assert!(stage.apply_i420(&mut first, 0));
        assert_ne!(first.y[0], gradient_i420(64, 48).y[0]);

        let mut second = gradient_i420(32, 24);
        let original = second.clone();
        assert!(stage.apply_i420(&mut second, 0));

        assert_ne!(second.y[0], original.y[0]);
    }

    #[test]
    fn stage_passes_frames_through_when_background_disabled() {
        let mut stage = CameraBackgroundStage::new(CameraBackgroundConfig::default());
        let mut frame = gradient_i420(64, 48);
        let original = frame.clone();

        assert!(!stage.is_enabled());
        assert!(stage.apply_i420(&mut frame, 0));

        assert_eq!(frame.y, original.y);
    }

    fn solid_i420(width: u32, height: u32, luma: u8) -> crate::yuv::I420 {
        let mut frame = crate::yuv::I420::new(width, height).unwrap();
        frame.y.fill(luma);
        frame.u.fill(128);
        frame.v.fill(128);
        frame
    }

    fn write_temp_gif(name: &str, colors: &[[u8; 3]], delay_ms: u32) -> PathBuf {
        use image::codecs::gif::GifEncoder;
        use image::{Delay, Frame, Rgba, RgbaImage};
        assert!(!colors.is_empty());
        assert!(delay_ms > 0);
        let path = std::env::temp_dir().join(format!("fluxer-camera-background-{name}.gif"));
        let file = std::fs::File::create(&path).unwrap();
        let mut encoder = GifEncoder::new(file);
        for color in colors {
            let mut image = RgbaImage::new(4, 4);
            for pixel in image.pixels_mut() {
                *pixel = Rgba([color[0], color[1], color[2], 255]);
            }
            encoder
                .encode_frame(Frame::from_parts(
                    image,
                    0,
                    0,
                    Delay::from_numer_denom_ms(delay_ms, 1),
                ))
                .unwrap();
        }
        path
    }

    #[test]
    fn animation_delay_clamps_to_named_bounds_and_defaults_zero() {
        assert_eq!(
            clamp_animation_delay_ms(0),
            CUSTOM_BACKGROUND_FRAME_DELAY_MS_DEFAULT
        );
        assert_eq!(
            clamp_animation_delay_ms(1),
            CUSTOM_BACKGROUND_FRAME_DELAY_MS_MIN
        );
        assert_eq!(clamp_animation_delay_ms(50), 50);
        assert_eq!(
            clamp_animation_delay_ms(60_000),
            CUSTOM_BACKGROUND_FRAME_DELAY_MS_MAX
        );
    }

    #[test]
    fn animation_frame_index_follows_cumulative_schedule() {
        let schedule = [100_000u64, 250_000, 400_000];

        assert_eq!(animation_frame_index(&schedule, 0), 0);
        assert_eq!(animation_frame_index(&schedule, 99_999), 0);
        assert_eq!(animation_frame_index(&schedule, 100_000), 1);
        assert_eq!(animation_frame_index(&schedule, 249_999), 1);
        assert_eq!(animation_frame_index(&schedule, 250_000), 2);
        assert_eq!(animation_frame_index(&schedule, 399_999), 2);
    }

    #[test]
    fn animation_frame_index_wraps_around_loop() {
        let schedule = [100_000u64, 200_000];

        assert_eq!(animation_frame_index(&schedule, 200_000), 0);
        assert_eq!(animation_frame_index(&schedule, 350_000), 1);
        assert_eq!(animation_frame_index(&schedule, 1_000_000_000), 0);
    }

    #[test]
    fn animated_decode_respects_frame_and_byte_caps() {
        let colors = [
            [255u8, 0, 0],
            [0, 255, 0],
            [0, 0, 255],
            [255, 255, 0],
            [0, 255, 255],
        ];
        let path = write_temp_gif("caps", &colors, 100);
        let path_str = path.to_str().unwrap();
        let frame_rgba_bytes = 4 * 4 * 4;

        let all = decode_animated_background_frames(
            path_str,
            16,
            16,
            CUSTOM_BACKGROUND_ANIMATION_FRAMES_MAX,
            CUSTOM_BACKGROUND_ANIMATION_BYTES_MAX,
        );
        let frame_capped = decode_animated_background_frames(
            path_str,
            16,
            16,
            3,
            CUSTOM_BACKGROUND_ANIMATION_BYTES_MAX,
        );
        let byte_capped = decode_animated_background_frames(
            path_str,
            16,
            16,
            CUSTOM_BACKGROUND_ANIMATION_FRAMES_MAX,
            frame_rgba_bytes * 2 + 1,
        );

        assert_eq!(all.len(), colors.len());
        assert!(all.iter().all(|(_, delay_ms)| *delay_ms == 100));
        assert_eq!(frame_capped.len(), 3);
        assert_eq!(byte_capped.len(), 2);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn animated_gif_background_selects_frames_deterministically_from_timestamps() {
        let path = write_temp_gif("two-frame", &[[0, 0, 0], [255, 255, 255]], 100);
        let custom = CameraBackgroundConfig::from_bridge_values(
            Some("custom"),
            path.to_str(),
            Some("animated"),
            None,
        )
        .unwrap();
        let mut transform =
            CameraBackgroundTransform::with_mask_source(custom, 64, 48, None).unwrap();
        let corner = 0usize;

        let mut first = solid_i420(64, 48, 128);
        assert!(transform.apply_i420(&mut first, 5_000_000));
        let mut second = solid_i420(64, 48, 128);
        assert!(transform.apply_i420(&mut second, 5_150_000));
        let mut wrapped = solid_i420(64, 48, 128);
        assert!(transform.apply_i420(&mut wrapped, 5_250_000));

        assert!(first.y[corner] < 64, "dark frame luma {}", first.y[corner]);
        assert!(
            second.y[corner] > 192,
            "bright frame luma {}",
            second.y[corner]
        );
        assert_eq!(wrapped.y[corner], first.y[corner]);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn video_background_composites_latest_pushed_live_frame() {
        let slot = CameraBackgroundLiveSlot::new();
        let config = CameraBackgroundConfig {
            mode: CameraBackgroundMode::Custom,
            custom: Some(CameraBackgroundCustomSource {
                path: "/tmp/background.mp4".to_string(),
                media_kind: CameraBackgroundCustomMediaKind::Video,
            }),
            live_background: Some(slot.clone()),
            ..CameraBackgroundConfig::default()
        };
        let mut transform =
            CameraBackgroundTransform::with_mask_source(config, 64, 48, None).unwrap();
        let original = solid_i420(64, 48, 128);
        let corner = 0usize;
        let center = 22 * 64 + 32;

        let mut passthrough = original.clone();
        assert!(transform.apply_i420(&mut passthrough, 0));
        assert_eq!(passthrough.y, original.y);

        assert!(slot.store(solid_i420(64, 48, 40)));
        let mut composited = original.clone();
        assert!(transform.apply_i420(&mut composited, 33_000));
        assert_eq!(composited.y[corner], 40);
        assert_eq!(composited.y[center], 128);

        slot.clear();
        let mut cleared = original.clone();
        assert!(transform.apply_i420(&mut cleared, 66_000));
        assert_eq!(cleared.y, original.y);
    }

    #[test]
    fn video_background_without_engine_slot_passes_frames_through() {
        let config = CameraBackgroundConfig {
            mode: CameraBackgroundMode::Custom,
            custom: Some(CameraBackgroundCustomSource {
                path: "/nonexistent/background.mp4".to_string(),
                media_kind: CameraBackgroundCustomMediaKind::Video,
            }),
            live_background: None,
            ..CameraBackgroundConfig::default()
        };
        let mut transform =
            CameraBackgroundTransform::with_mask_source(config, 64, 48, None).unwrap();
        let original = solid_i420(64, 48, 128);
        let mut frame = original.clone();

        assert!(transform.apply_i420(&mut frame, 0));

        assert_eq!(frame.y, original.y);
    }

    #[test]
    fn live_slot_resizes_pushed_frames_to_camera_dims_on_store() {
        let slot = CameraBackgroundLiveSlot::new();
        slot.set_target_dims(64, 48);

        assert!(slot.store(solid_i420(32, 24, 200)));

        slot.with_latest(|frame| {
            let frame = frame.expect("stored frame present");
            assert_eq!((frame.width, frame.height), (64, 48));
            assert_eq!(frame.y[0], 200);
        });
    }

    #[test]
    fn live_slot_resizes_existing_frame_when_target_dims_arrive() {
        let slot = CameraBackgroundLiveSlot::new();
        assert!(slot.store(solid_i420(32, 24, 75)));

        slot.set_target_dims(64, 48);

        slot.with_latest(|frame| {
            let frame = frame.expect("stored frame retained");
            assert_eq!((frame.width, frame.height), (64, 48));
            assert_eq!(frame.y[0], 75);
        });
    }

    #[test]
    fn live_slot_keeps_only_the_latest_pushed_frame() {
        let slot = CameraBackgroundLiveSlot::new();
        slot.set_target_dims(64, 48);

        assert!(slot.store(solid_i420(64, 48, 10)));
        assert!(slot.store(solid_i420(64, 48, 20)));

        slot.with_latest(|frame| {
            assert_eq!(frame.expect("latest frame present").y[0], 20);
        });
    }

    #[test]
    fn live_slot_store_tight_i420_reuses_same_size_plane_buffers() {
        let slot = CameraBackgroundLiveSlot::new();
        let first = [1u8, 2, 3, 4, 5, 6];
        let second = [7u8, 8, 9, 10, 11, 12];

        assert!(slot.store_tight_i420(&first, 2, 2));
        let first_ptrs = slot.with_latest(|frame| {
            let frame = frame.expect("first frame present");
            (frame.y.as_ptr(), frame.u.as_ptr(), frame.v.as_ptr())
        });
        assert!(slot.store_tight_i420(&second, 2, 2));

        slot.with_latest(|frame| {
            let frame = frame.expect("second frame present");
            assert_eq!(frame.y.as_ptr(), first_ptrs.0);
            assert_eq!(frame.u.as_ptr(), first_ptrs.1);
            assert_eq!(frame.v.as_ptr(), first_ptrs.2);
            assert_eq!(frame.y, vec![7, 8, 9, 10]);
            assert_eq!(frame.u, vec![11]);
            assert_eq!(frame.v, vec![12]);
        });
    }

    #[test]
    fn live_slot_resizes_tight_frames_to_target_dims_and_recycles_buffers() {
        let slot = CameraBackgroundLiveSlot::new();
        slot.set_target_dims(64, 48);
        let tight = |y: u8, u: u8, v: u8| {
            let mut data = vec![y; 32 * 24];
            data.resize(32 * 24 + 16 * 12, u);
            data.resize(32 * 24 + 2 * 16 * 12, v);
            data
        };

        assert!(slot.store_tight_i420(&tight(40, 90, 160), 32, 24));
        let first_ptr = slot.with_latest(|frame| {
            let frame = frame.expect("resized frame stored");
            assert_eq!((frame.width, frame.height), (64, 48));
            assert_eq!(frame.y[0], 40);
            assert_eq!(frame.u[0], 90);
            assert_eq!(frame.v[0], 160);
            frame.y.as_ptr()
        });

        assert!(slot.store_tight_i420(&tight(10, 20, 30), 32, 24));
        slot.with_latest(|frame| {
            assert_eq!(frame.expect("second frame stored").y[0], 10);
        });

        assert!(slot.store_tight_i420(&tight(50, 60, 70), 32, 24));
        slot.with_latest(|frame| {
            let frame = frame.expect("third frame stored");
            assert_eq!(frame.y[0], 50);
            assert_eq!(frame.y.as_ptr(), first_ptr);
        });
    }

    #[test]
    fn live_slot_rejects_truncated_tight_frames_needing_resize() {
        let slot = CameraBackgroundLiveSlot::new();
        slot.set_target_dims(64, 48);
        let short = vec![128u8; 32 * 24];
        assert!(!slot.store_tight_i420(&short, 32, 24));
        slot.with_latest(|frame| assert!(frame.is_none()));
    }

    #[test]
    fn scaled_blur_radius_spans_quarter_to_two_and_a_half_times_base() {
        assert_eq!(scaled_blur_radius(24, 0), 6);
        assert_eq!(scaled_blur_radius(24, CAMERA_EFFECT_STRENGTH_DEFAULT), 24);
        assert_eq!(scaled_blur_radius(24, CAMERA_EFFECT_STRENGTH_MAX), 60);
        assert_eq!(scaled_blur_radius(4, 0), 1);
        assert_eq!(scaled_blur_radius(4, CAMERA_EFFECT_STRENGTH_DEFAULT), 4);
        assert_eq!(scaled_blur_radius(4, CAMERA_EFFECT_STRENGTH_MAX), 10);
    }

    #[test]
    fn scaled_blur_radius_clamps_to_named_bounds() {
        assert_eq!(scaled_blur_radius(1, 0), BLUR_RADIUS_SCALED_MIN);
        assert_eq!(
            scaled_blur_radius(BLUR_RADIUS_SCALED_MAX, CAMERA_EFFECT_STRENGTH_MAX),
            BLUR_RADIUS_SCALED_MAX
        );
        assert_eq!(
            scaled_blur_radius(10, CAMERA_EFFECT_STRENGTH_MAX + 100),
            scaled_blur_radius(10, CAMERA_EFFECT_STRENGTH_MAX)
        );
    }

    #[test]
    fn scaled_blur_radius_is_monotonic_in_strength() {
        let mut previous = 0usize;
        for strength in 0..=CAMERA_EFFECT_STRENGTH_MAX {
            let radius = scaled_blur_radius(24, strength);
            assert!(radius >= previous);
            previous = radius;
        }
    }

    #[test]
    fn bridge_values_default_and_clamp_effect_strengths() {
        let defaults =
            CameraBackgroundConfig::from_bridge_values(Some("blur"), None, None, None).unwrap();
        assert_eq!(defaults.blur_strength, CAMERA_EFFECT_STRENGTH_DEFAULT);

        let clamped =
            CameraBackgroundConfig::from_bridge_values(Some("blur"), None, None, Some(250))
                .unwrap();
        assert_eq!(clamped.blur_strength, CAMERA_EFFECT_STRENGTH_MAX);

        let explicit =
            CameraBackgroundConfig::from_bridge_values(Some("blur"), None, None, Some(10)).unwrap();
        assert_eq!(explicit.blur_strength, 10);
    }

    #[test]
    fn masked_blur_keeps_person_colors_out_of_the_blurred_background() {
        let mut transform = blur_transform_with_mask(64, 48, vec![true]);
        let mut frame = solid_i420(64, 48, 16);
        for y in 0..48usize {
            for x in 0..32usize {
                frame.y[y * 64 + x] = 235;
            }
        }

        assert!(transform.apply_i420(&mut frame, 0));

        assert_eq!(frame.y[24 * 64 + 20], 235);
        assert_eq!(frame.y[24 * 64 + 48], 16);
        assert_eq!(frame.y[24 * 64 + 63], 16);
    }

    #[test]
    fn blur_strength_scales_the_effective_blur_radius() {
        fn blurred_corner(strength: u32) -> u8 {
            let config = CameraBackgroundConfig::from_bridge_values(
                Some("blur"),
                None,
                None,
                Some(strength),
            )
            .unwrap();
            let mut transform =
                CameraBackgroundTransform::with_mask_source(config, 640, 360, None).unwrap();
            let mut frame = gradient_i420(640, 360);
            assert!(transform.apply_i420(&mut frame, 0));
            frame.y[0]
        }

        let weak = blurred_corner(0);
        let default = blurred_corner(CAMERA_EFFECT_STRENGTH_DEFAULT);
        let strong = blurred_corner(CAMERA_EFFECT_STRENGTH_MAX);
        assert!(weak < default);
        assert!(default < strong);
    }
}
