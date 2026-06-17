// SPDX-License-Identifier: AGPL-3.0-or-later

pub const DEFAULT_WIDTH: u32 = 1280;
pub const DEFAULT_HEIGHT: u32 = 720;
pub const DEFAULT_FPS: u32 = 30;
const MAX_CAMERA_DEVICE_ALIASES: usize = 4;
#[cfg(feature = "camera-native")]
const MAX_CAMERA_FORMAT_ATTEMPTS: usize = 16;
#[cfg(feature = "camera-native")]
const MAX_COMPATIBLE_CAMERA_FORMAT_CANDIDATES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CameraSelector {
    Index(u32),
    Id(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CameraRequest {
    pub selector: CameraSelector,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub mirror: bool,
    pub background: crate::camera_background::CameraBackgroundConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CameraDevice {
    pub device_id: String,
    pub label: String,
    pub description: String,
    pub index: Option<u32>,
    pub device_id_aliases: Vec<String>,
}

fn push_camera_device_alias(aliases: &mut Vec<String>, alias: &str) {
    assert!(aliases.len() <= MAX_CAMERA_DEVICE_ALIASES);
    let trimmed = alias.trim();
    if trimmed.is_empty() {
        return;
    }
    for existing in aliases.iter().take(MAX_CAMERA_DEVICE_ALIASES) {
        if existing == trimmed {
            return;
        }
    }
    if aliases.len() < MAX_CAMERA_DEVICE_ALIASES {
        aliases.push(trimmed.to_string());
    }
    assert!(aliases.len() <= MAX_CAMERA_DEVICE_ALIASES);
}

fn camera_device_aliases(device_id: &str, index: &str) -> Vec<String> {
    let mut aliases = Vec::with_capacity(2);
    assert_eq!(aliases.len(), 0);
    push_camera_device_alias(&mut aliases, device_id);
    push_camera_device_alias(&mut aliases, index);
    assert!(aliases.len() <= MAX_CAMERA_DEVICE_ALIASES);
    aliases
}

impl CameraRequest {
    pub fn from_opts(
        device_id: Option<&str>,
        width: Option<u32>,
        height: Option<u32>,
        frame_rate: Option<u32>,
        mirror: bool,
        background: crate::camera_background::CameraBackgroundConfig,
    ) -> Self {
        let selector = match device_id
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != "default")
        {
            Some(raw) => raw
                .parse::<u32>()
                .map(CameraSelector::Index)
                .unwrap_or_else(|_| CameraSelector::Id(raw.to_string())),
            None => CameraSelector::Index(0),
        };
        let width = match width {
            Some(w) if w >= 2 => w & !1,
            _ => DEFAULT_WIDTH,
        };
        let height = match height {
            Some(h) if h >= 2 => h & !1,
            _ => DEFAULT_HEIGHT,
        };
        let fps = match frame_rate {
            Some(f) if f > 0 => f,
            _ => DEFAULT_FPS,
        };
        Self {
            selector,
            width,
            height,
            fps,
            mirror,
            background,
        }
    }
}

#[cfg(feature = "camera-native")]
fn accepted_camera_formats() -> &'static [nokhwa::utils::FrameFormat] {
    use nokhwa::pixel_format::{FormatDecoder, RgbFormat};
    use nokhwa::utils::FrameFormat;
    const FORMATS: &[FrameFormat] = &[
        FrameFormat::NV12,
        FrameFormat::YUYV,
        FrameFormat::RAWRGB,
        FrameFormat::RAWBGR,
        FrameFormat::MJPEG,
    ];
    let formats = <RgbFormat as FormatDecoder>::FORMATS;
    assert!(FORMATS.iter().all(|format| formats.contains(format)));
    assert!(formats.len() <= MAX_CAMERA_FORMAT_ATTEMPTS);
    FORMATS
}

#[cfg(feature = "camera-native")]
fn camera_format_priority(frame_format: nokhwa::utils::FrameFormat) -> usize {
    accepted_camera_formats()
        .iter()
        .position(|candidate| *candidate == frame_format)
        .unwrap_or(accepted_camera_formats().len())
}

#[cfg(feature = "camera-native")]
fn camera_format_score(
    format: &nokhwa::utils::CameraFormat,
    request: &CameraRequest,
) -> (u64, u32, usize) {
    let resolution = format.resolution();
    let width_delta = u64::from(resolution.width().abs_diff(request.width));
    let height_delta = u64::from(resolution.height().abs_diff(request.height));
    (
        width_delta * width_delta + height_delta * height_delta,
        format.frame_rate().abs_diff(request.fps),
        camera_format_priority(format.format()),
    )
}

#[cfg(feature = "camera-native")]
fn select_best_camera_format(
    formats: &[nokhwa::utils::CameraFormat],
    request: &CameraRequest,
) -> Option<nokhwa::utils::CameraFormat> {
    formats
        .iter()
        .take(MAX_COMPATIBLE_CAMERA_FORMAT_CANDIDATES)
        .filter(|format| accepted_camera_formats().contains(&format.format()))
        .min_by_key(|format| camera_format_score(format, request))
        .copied()
}

#[cfg(feature = "camera-native")]
struct CameraFrameConverter {
    i420: crate::yuv::I420,
}

#[cfg(feature = "camera-native")]
impl CameraFrameConverter {
    fn new(width: u32, height: u32) -> Option<Self> {
        Some(Self {
            i420: crate::yuv::I420::new(width, height)?,
        })
    }

    fn width(&self) -> u32 {
        self.i420.width
    }

    fn height(&self) -> u32 {
        self.i420.height
    }

    fn convert<'a>(
        &'a mut self,
        frame: &nokhwa::Buffer,
        width: u32,
        height: u32,
    ) -> Option<&'a mut crate::yuv::I420> {
        if self.i420.width != width || self.i420.height != height {
            return None;
        }
        if !camera_frame_to_i420_into(frame, width, height, &mut self.i420) {
            return None;
        }
        Some(&mut self.i420)
    }

    fn convert_nv12<'a>(
        &'a mut self,
        src: &[u8],
        width: u32,
        height: u32,
    ) -> Option<&'a crate::yuv::I420> {
        if self.i420.width != width || self.i420.height != height {
            return None;
        }
        if !crate::yuv::nv12_to_i420_into(src, width, height, width, width, &mut self.i420) {
            return None;
        }
        Some(&self.i420)
    }
}

#[cfg(feature = "camera-native")]
fn camera_frame_to_i420_into(
    frame: &nokhwa::Buffer,
    width: u32,
    height: u32,
    dst: &mut crate::yuv::I420,
) -> bool {
    use nokhwa::utils::FrameFormat;
    match frame.source_frame_format() {
        FrameFormat::NV12 => {
            crate::yuv::nv12_to_i420_into(frame.buffer(), width, height, width, width, dst)
        }
        FrameFormat::YUYV => {
            crate::yuv::yuyv_to_i420_into(frame.buffer(), width, height, width * 2, dst)
        }
        FrameFormat::RAWRGB => crate::yuv::rgb_to_i420_into(frame.buffer(), width, height, dst),
        FrameFormat::RAWBGR => crate::yuv::bgr_to_i420_into(frame.buffer(), width, height, dst),
        FrameFormat::MJPEG | FrameFormat::GRAY => {
            decode_camera_frame_to_i420_into(frame, width, height, dst)
        }
    }
}

fn mirror_i420_in_place(frame: &mut crate::yuv::I420) {
    assert!(frame.width >= 2);
    assert!(frame.height >= 2);
    mirror_plane_rows(&mut frame.y, frame.width as usize, frame.height as usize);
    mirror_plane_rows(
        &mut frame.u,
        (frame.width / 2) as usize,
        (frame.height / 2) as usize,
    );
    mirror_plane_rows(
        &mut frame.v,
        (frame.width / 2) as usize,
        (frame.height / 2) as usize,
    );
}

fn mirror_plane_rows(plane: &mut [u8], width: usize, rows: usize) {
    assert!(width >= 1);
    assert!(rows >= 1);
    assert!(plane.len() >= width.saturating_mul(rows));
    for row in 0..rows {
        let start = row * width;
        let end = start + width;
        plane[start..end].reverse();
    }
}

#[cfg(feature = "camera-native")]
fn decode_camera_frame_to_i420_into(
    frame: &nokhwa::Buffer,
    width: u32,
    height: u32,
    dst: &mut crate::yuv::I420,
) -> bool {
    use nokhwa::pixel_format::RgbFormat;
    let Ok(rgb) = frame.decode_image::<RgbFormat>() else {
        return false;
    };
    if rgb.width() != width || rgb.height() != height {
        return false;
    }
    crate::yuv::rgb_to_i420_into(&rgb.into_raw(), width, height, dst)
}

#[cfg(all(test, feature = "camera-native"))]
mod camera_native_tests {
    use super::*;
    use nokhwa::Buffer;
    use nokhwa::utils::{CameraFormat, FrameFormat, Resolution};

    fn request(width: u32, height: u32, fps: u32) -> CameraRequest {
        CameraRequest {
            selector: CameraSelector::Index(0),
            width,
            height,
            fps,
            mirror: false,
            background: crate::camera_background::CameraBackgroundConfig::default(),
        }
    }

    #[test]
    fn raw_format_selection_prefers_nv12_over_yuyv_and_mjpeg() {
        let formats = vec![
            CameraFormat::new(Resolution::new(1280, 720), FrameFormat::MJPEG, 30),
            CameraFormat::new(Resolution::new(1280, 720), FrameFormat::YUYV, 30),
            CameraFormat::new(Resolution::new(1280, 720), FrameFormat::NV12, 30),
        ];

        let selected = select_best_camera_format(&formats, &request(1280, 720, 30)).unwrap();

        assert_eq!(selected.format(), FrameFormat::NV12);
    }

    #[test]
    fn raw_format_selection_prefers_nearest_resolution_before_format_priority() {
        let formats = vec![
            CameraFormat::new(Resolution::new(1920, 1080), FrameFormat::NV12, 30),
            CameraFormat::new(Resolution::new(1024, 768), FrameFormat::YUYV, 30),
        ];

        let selected = select_best_camera_format(&formats, &request(1280, 720, 30)).unwrap();

        assert_eq!(selected.resolution(), Resolution::new(1024, 768));
        assert_eq!(selected.format(), FrameFormat::YUYV);
    }

    #[test]
    fn camera_frame_to_i420_accepts_raw_nv12_without_rgb_decode() {
        let data = [1u8, 2, 3, 4, 10, 20];
        let frame = Buffer::new(Resolution::new(2, 2), &data, FrameFormat::NV12);
        let mut converter = CameraFrameConverter::new(2, 2).unwrap();

        let i420 = converter.convert(&frame, 2, 2).unwrap();

        assert_eq!(i420.y, vec![1, 2, 3, 4]);
        assert_eq!(i420.u, vec![10]);
        assert_eq!(i420.v, vec![20]);
    }

    #[test]
    fn camera_frame_to_i420_accepts_raw_yuyv_without_rgb_decode() {
        let data = [1u8, 10, 2, 20, 3, 30, 4, 40];
        let frame = Buffer::new(Resolution::new(2, 2), &data, FrameFormat::YUYV);
        let mut converter = CameraFrameConverter::new(2, 2).unwrap();

        let i420 = converter.convert(&frame, 2, 2).unwrap();

        assert_eq!(i420.y, vec![1, 2, 3, 4]);
        assert_eq!(i420.u, vec![20]);
        assert_eq!(i420.v, vec![30]);
    }

    #[test]
    fn native_camera_transport_contract_includes_platform_zero_copy_buffers() {
        assert_eq!(
            crate::native_camera::required_transport_names(),
            ["cvPixelBuffer", "d3d11Texture", "dmabuf"]
        );
    }
}

#[cfg(feature = "publisher")]
#[cfg_attr(not(feature = "camera-native"), allow(dead_code))]
#[derive(Clone, Copy, Debug)]
pub struct OpenedCamera {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

#[cfg(feature = "publisher")]
pub type LocalVideoFrameSink = Box<dyn Fn(&crate::yuv::I420, i64) + Send>;

#[cfg(feature = "publisher")]
pub type LocalVideoFrameSinkActive = Box<dyn Fn() -> bool + Send>;

#[cfg(feature = "publisher")]
pub struct CameraCaptureSinks {
    pub source: livekit::webrtc::video_source::native::NativeVideoSource,
    pub frame_sink: LocalVideoFrameSink,
    pub frame_sink_active: LocalVideoFrameSinkActive,
}

#[cfg(all(
    feature = "publisher",
    feature = "camera-native",
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
pub use live::spawn_capture_worker;

#[cfg(all(
    feature = "publisher",
    not(all(
        feature = "camera-native",
        any(target_os = "windows", target_os = "macos", target_os = "linux")
    ))
))]
pub fn spawn_capture_worker(
    _request: CameraRequest,
    result_tx: std::sync::mpsc::Sender<Result<OpenedCamera, String>>,
    _source_rx: std::sync::mpsc::Receiver<CameraCaptureSinks>,
    _stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    let _ = result_tx.send(Err(
        "native camera capture is not enabled for this platform build".to_string(),
    ));
}

#[cfg(all(
    feature = "publisher",
    feature = "camera-native",
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
pub use live::list_devices;

#[cfg(all(
    feature = "publisher",
    not(all(
        feature = "camera-native",
        any(target_os = "windows", target_os = "macos", target_os = "linux")
    ))
))]
pub fn list_devices() -> Result<Vec<CameraDevice>, String> {
    Ok(Vec::new())
}

#[cfg(all(
    feature = "publisher",
    feature = "camera-native",
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
mod live {
    use super::{
        CameraCaptureSinks, CameraDevice, CameraFrameConverter, CameraRequest, CameraSelector,
        OpenedCamera, camera_device_aliases,
    };
    use crate::yuv;
    use livekit::webrtc::video_frame::{I420Buffer, NV12Buffer, VideoFrame, VideoRotation};
    use livekit::webrtc::video_source::native::NativeVideoSource;
    use nokhwa::pixel_format::RgbFormat;
    use nokhwa::utils::{
        ApiBackend, CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType,
        Resolution,
    };
    use nokhwa::{Camera, query};
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::Sender;
    #[cfg(target_os = "macos")]
    use std::sync::mpsc::channel;
    use std::time::{Duration, Instant};

    const MAX_CAMERA_DEVICE_FALLBACK_CANDIDATES: usize = 64;
    const CAMERA_FRAME_FAILURE_RETRY_DELAY: Duration = Duration::from_millis(5);
    const CAMERA_FRAME_FAILURES_CONSECUTIVE_MAX: u32 = 600;

    #[cfg(target_os = "macos")]
    fn ensure_camera_runtime_initialized() -> Result<(), String> {
        if nokhwa::nokhwa_check() {
            return Ok(());
        }
        let (tx, rx) = channel();
        nokhwa::nokhwa_initialize(move |ok| {
            let _ = tx.send(ok);
        });
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(true) => Ok(()),
            Ok(false) => Err("camera permission denied".to_string()),
            Err(_) => Err("camera permission request timed out".to_string()),
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn ensure_camera_runtime_initialized() -> Result<(), String> {
        Ok(())
    }

    fn camera_index_for_request(request: &CameraRequest) -> CameraIndex {
        match &request.selector {
            CameraSelector::Index(index) => CameraIndex::Index(*index),
            CameraSelector::Id(id) => CameraIndex::String(id.clone()),
        }
    }

    fn camera_device_id_matches(requested_id: &str, device_id: &str, index: &str) -> bool {
        assert!(!requested_id.trim().is_empty());
        assert_eq!(requested_id, requested_id.trim());
        assert!(MAX_CAMERA_DEVICE_FALLBACK_CANDIDATES > 0);
        if requested_id == device_id.trim() {
            return true;
        }
        requested_id == index.trim()
    }

    fn fallback_camera_index_for_request(request: &CameraRequest) -> Option<CameraIndex> {
        let CameraSelector::Id(requested_id) = &request.selector else {
            return None;
        };
        let requested_id = requested_id.trim();
        if requested_id.is_empty() {
            return None;
        }
        let devices = query(ApiBackend::Auto).ok()?;
        for device in devices
            .into_iter()
            .take(MAX_CAMERA_DEVICE_FALLBACK_CANDIDATES)
        {
            let index_string = device.index().as_string();
            let misc = device.misc();
            let device_id = if misc.trim().is_empty() {
                index_string.as_str()
            } else {
                misc.as_str()
            };
            if !camera_device_id_matches(requested_id, device_id, &index_string) {
                continue;
            }
            if let Ok(index) = device.index().as_index() {
                return Some(CameraIndex::Index(index));
            }
        }
        None
    }

    fn requested_camera_format(
        request: &CameraRequest,
        frame_format: FrameFormat,
    ) -> RequestedFormat<'static> {
        let camera_format = CameraFormat::new(
            Resolution::new(request.width, request.height),
            frame_format,
            request.fps,
        );
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(camera_format))
    }

    fn open_stream(mut camera: Camera) -> Result<(Camera, OpenedCamera), String> {
        camera
            .open_stream()
            .map_err(|e| format!("open camera stream: {e}"))?;
        let resolution = camera.resolution();
        let fps = camera.frame_rate();
        let opened = OpenedCamera {
            width: resolution.width() & !1,
            height: resolution.height() & !1,
            fps,
        };
        Ok((camera, opened))
    }

    fn open_best_compatible_camera(
        index: CameraIndex,
        request: &CameraRequest,
    ) -> Result<(Camera, OpenedCamera), String> {
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
        let mut camera =
            Camera::new(index, requested).map_err(|error| format!("open camera: {error}"))?;
        let formats = camera
            .compatible_camera_formats()
            .map_err(|error| format!("query compatible formats: {error}"))?;
        let best_format = super::select_best_camera_format(&formats, request).ok_or_else(|| {
            format!(
                "no compatible RGB-decodable camera format for {}x{}@{}",
                request.width, request.height, request.fps
            )
        })?;
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(best_format));
        camera
            .set_camera_requset(requested)
            .map_err(|error| format!("set selected format {best_format:?}: {error}"))?;
        open_stream(camera).map_err(|error| format!("{error}; selected format {best_format:?}"))
    }

    fn open_camera_with_index(
        index: CameraIndex,
        request: &CameraRequest,
    ) -> Result<(Camera, OpenedCamera), String> {
        assert!(request.width >= 2);
        assert!(request.height >= 2);
        assert!(request.fps > 0);
        let mut errors = Vec::new();
        match open_best_compatible_camera(index.clone(), request) {
            Ok(opened) => return Ok(opened),
            Err(error) => errors.push(format!("best compatible format: {error}")),
        }
        for frame_format in super::accepted_camera_formats()
            .iter()
            .take(super::MAX_CAMERA_FORMAT_ATTEMPTS)
        {
            let requested = requested_camera_format(request, *frame_format);
            match Camera::new(index.clone(), requested) {
                Ok(camera) => match open_stream(camera) {
                    Ok(opened) => return Ok(opened),
                    Err(error) => errors.push(format!("{frame_format}: {error}")),
                },
                Err(error) => errors.push(format!("{frame_format}: {error}")),
            }
        }

        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
        let mut camera = Camera::new(index, requested).map_err(|error| {
            format!(
                "open camera: {error}; tried requested formats: {}",
                errors.join("; ")
            )
        })?;
        match camera.compatible_camera_formats() {
            Ok(formats) => {
                if let Some(best_format) = super::select_best_camera_format(&formats, request) {
                    let request =
                        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(best_format));
                    if let Err(error) = camera.set_camera_requset(request) {
                        errors.push(format!("set fallback format {best_format:?}: {error}"));
                    }
                }
            }
            Err(error) => errors.push(format!("query fallback formats: {error}")),
        }
        open_stream(camera)
            .map_err(|error| format!("{error}; tried requested formats: {}", errors.join("; ")))
    }

    fn open_camera(request: &CameraRequest) -> Result<(Camera, OpenedCamera), String> {
        ensure_camera_runtime_initialized()?;
        let primary_index = camera_index_for_request(request);
        match open_camera_with_index(primary_index.clone(), request) {
            Ok(opened) => Ok(opened),
            Err(primary_error) => match fallback_camera_index_for_request(request) {
                Some(fallback_index) if fallback_index != primary_index => {
                    open_camera_with_index(fallback_index.clone(), request).map_err(|fallback_error| {
                        format!(
                            "open camera by requested selector failed: {primary_error}; fallback index {} failed: {fallback_error}",
                            fallback_index.as_string()
                        )
                    })
                }
                _ => Err(primary_error),
            },
        }
    }

    pub fn list_devices() -> Result<Vec<CameraDevice>, String> {
        let devices = query(ApiBackend::Auto).map_err(|e| format!("query cameras: {e}"))?;
        Ok(devices
            .into_iter()
            .map(|device| {
                let misc = device.misc();
                let index_string = device.index().as_string();
                let device_id = if misc.trim().is_empty() {
                    index_string.clone()
                } else {
                    misc
                };
                CameraDevice {
                    device_id_aliases: camera_device_aliases(&device_id, &index_string),
                    device_id,
                    label: device.human_name(),
                    description: device.description().to_string(),
                    index: device.index().as_index().ok(),
                }
            })
            .collect())
    }

    pub fn spawn_capture_worker(
        request: CameraRequest,
        result_tx: Sender<Result<OpenedCamera, String>>,
        source_rx: std::sync::mpsc::Receiver<CameraCaptureSinks>,
        stop: Arc<AtomicBool>,
    ) {
        std::thread::spawn(move || {
            elevate_capture_thread_priority();
            let (mut camera, opened) = match open_camera(&request) {
                Ok(pair) => pair,
                Err(e) => {
                    let _ = result_tx.send(Err(e));
                    return;
                }
            };
            if result_tx.send(Ok(opened)).is_err() {
                return;
            }
            let sinks = match source_rx.recv() {
                Ok(s) => s,
                Err(_) => return,
            };
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                run_capture_loop(&mut camera, opened, &request, &sinks, &stop);
            }));
            if outcome.is_err() {
                eprintln!("webrtc-sender: camera capture worker panicked; stopping capture");
            }
            let _ = camera.stop_stream();
        });
    }

    #[cfg(target_os = "macos")]
    fn elevate_capture_thread_priority() {
        use std::ffi::{c_int, c_uint};
        const QOS_CLASS_USER_INTERACTIVE: c_uint = 0x21;
        const _: () = assert!(QOS_CLASS_USER_INTERACTIVE == 0x21);
        unsafe extern "C" {
            fn pthread_set_qos_class_self_np(qos_class: c_uint, relative_priority: c_int) -> c_int;
        }
        let _ = unsafe { pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0) };
    }

    #[cfg(target_os = "windows")]
    fn elevate_capture_thread_priority() {
        use std::ffi::{c_int, c_void};
        const THREAD_PRIORITY_HIGHEST: c_int = 2;
        const _: () = assert!(THREAD_PRIORITY_HIGHEST == 2);
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetCurrentThread() -> *mut c_void;
            fn SetThreadPriority(thread: *mut c_void, priority: c_int) -> c_int;
        }
        let _ = unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST) };
    }

    #[cfg(target_os = "linux")]
    fn elevate_capture_thread_priority() {}

    fn run_capture_loop(
        camera: &mut Camera,
        opened: OpenedCamera,
        request: &CameraRequest,
        sinks: &CameraCaptureSinks,
        stop: &AtomicBool,
    ) {
        let start = Instant::now();
        let fps = opened.fps.max(1);
        assert!(fps >= 1);
        let mut deadline = Instant::now();
        let mut background_stage =
            crate::camera_background::CameraBackgroundStage::new(request.background.clone());
        let mut frame_converter: Option<CameraFrameConverter> = None;
        let mut consecutive_failures: u32 = 0;
        while !stop.load(Ordering::Acquire) {
            let frame = match camera.frame() {
                Ok(f) => f,
                Err(error) => {
                    if camera_frame_failure_cap_reached(&mut consecutive_failures) {
                        eprintln!(
                            "webrtc-sender: camera frame capture failed \
                             {CAMERA_FRAME_FAILURES_CONSECUTIVE_MAX} times in a row; \
                             stopping capture: {error}"
                        );
                        break;
                    }
                    std::thread::sleep(CAMERA_FRAME_FAILURE_RETRY_DELAY);
                    continue;
                }
            };
            consecutive_failures = 0;
            let fw = frame.resolution().width() & !1;
            let fh = frame.resolution().height() & !1;
            if stop.load(Ordering::Acquire) {
                break;
            }
            let timestamp_us = start.elapsed().as_micros() as i64;
            let converter_needs_init = match frame_converter.as_ref() {
                Some(converter) => converter.width() != fw || converter.height() != fh,
                None => true,
            };
            if converter_needs_init {
                frame_converter = CameraFrameConverter::new(fw, fh);
            }
            let Some(converter) = frame_converter.as_mut() else {
                continue;
            };
            if !request.mirror
                && !background_stage.is_enabled()
                && try_publish_raw_camera_frame(sinks, &frame, fw, fh, timestamp_us, converter)
            {
                pace_camera_frame(fps, &mut deadline);
                continue;
            }
            let Some(i420) = converter.convert(&frame, fw, fh) else {
                continue;
            };
            if !background_stage.apply_i420(i420, timestamp_us) {
                continue;
            }
            if request.mirror {
                super::mirror_i420_in_place(i420);
            }
            capture_i420(&sinks.source, i420, timestamp_us);
            (sinks.frame_sink)(i420, timestamp_us);

            pace_camera_frame(fps, &mut deadline);
        }
    }

    fn camera_frame_failure_cap_reached(consecutive_failures: &mut u32) -> bool {
        assert!(*consecutive_failures < CAMERA_FRAME_FAILURES_CONSECUTIVE_MAX);
        *consecutive_failures += 1;
        assert!(*consecutive_failures <= CAMERA_FRAME_FAILURES_CONSECUTIVE_MAX);
        *consecutive_failures == CAMERA_FRAME_FAILURES_CONSECUTIVE_MAX
    }

    fn pace_camera_frame(fps: u32, deadline: &mut Instant) {
        assert!(fps >= 1);
        let interval = Duration::from_secs_f64(1.0 / f64::from(fps));
        assert!(interval > Duration::ZERO);
        let next = next_pacing_deadline(*deadline, interval, Instant::now());
        let now = Instant::now();
        if next > now {
            std::thread::sleep(next - now);
        }
        *deadline = next;
    }

    fn next_pacing_deadline(deadline: Instant, interval: Duration, now: Instant) -> Instant {
        assert!(interval > Duration::ZERO);
        let next = deadline + interval;
        if next < now { now } else { next }
    }

    fn try_publish_raw_camera_frame(
        sinks: &CameraCaptureSinks,
        frame: &nokhwa::Buffer,
        width: u32,
        height: u32,
        timestamp_us: i64,
        converter: &mut CameraFrameConverter,
    ) -> bool {
        assert!(width >= 2);
        assert!(height >= 2);
        if frame.source_frame_format() != FrameFormat::NV12 {
            return false;
        }
        if !capture_tight_nv12(&sinks.source, frame.buffer(), width, height, timestamp_us) {
            return false;
        }
        if !(sinks.frame_sink_active)() {
            return true;
        }
        if let Some(i420) = converter.convert_nv12(frame.buffer(), width, height) {
            (sinks.frame_sink)(i420, timestamp_us);
        }
        true
    }

    fn capture_tight_nv12(
        source: &NativeVideoSource,
        data: &[u8],
        width: u32,
        height: u32,
        timestamp_us: i64,
    ) -> bool {
        let y_len = (width as usize) * (height as usize);
        let uv_len = y_len / 2;
        if data.len() < y_len + uv_len {
            return false;
        }
        let mut buffer = NV12Buffer::with_strides(width, height, width, width);
        let (dst_stride_y, dst_stride_uv) = buffer.strides();
        let (dst_y, dst_uv) = buffer.data_mut();
        if !yuv::copy_nv12_planes(
            data,
            width,
            height,
            width,
            width,
            dst_y,
            dst_uv,
            dst_stride_y,
            dst_stride_uv,
        ) {
            return false;
        }
        source.capture_frame(&VideoFrame {
            rotation: VideoRotation::VideoRotation0,
            timestamp_us,
            frame_metadata: None,
            buffer,
        });
        true
    }

    fn capture_i420(source: &NativeVideoSource, frame: &yuv::I420, timestamp_us: i64) {
        let mut buffer = I420Buffer::new(frame.width, frame.height);
        let (stride_y, stride_u, stride_v) = buffer.strides();
        {
            let (dy, du, dv) = buffer.data_mut();
            copy_plane(
                dy,
                &frame.y,
                frame.width as usize,
                stride_y as usize,
                frame.height as usize,
            );
            let cw = (frame.width / 2) as usize;
            let ch = (frame.height / 2) as usize;
            copy_plane(du, &frame.u, cw, stride_u as usize, ch);
            copy_plane(dv, &frame.v, cw, stride_v as usize, ch);
        }
        let video_frame = VideoFrame {
            rotation: VideoRotation::VideoRotation0,
            timestamp_us,
            frame_metadata: None,
            buffer,
        };
        source.capture_frame(&video_frame);
    }

    fn copy_plane(dst: &mut [u8], src: &[u8], width: usize, dst_stride: usize, rows: usize) {
        for row in 0..rows {
            let s = row * width;
            let d = row * dst_stride;
            if s + width <= src.len() && d + width <= dst.len() {
                dst[d..d + width].copy_from_slice(&src[s..s + width]);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn request(width: u32, height: u32, fps: u32) -> CameraRequest {
            CameraRequest {
                selector: CameraSelector::Index(0),
                width,
                height,
                fps,
                mirror: false,
                background: crate::camera_background::CameraBackgroundConfig::default(),
            }
        }

        #[test]
        fn fallback_format_selection_uses_decoder_supported_non_mjpeg_modes() {
            let formats = vec![
                CameraFormat::new(Resolution::new(640, 480), FrameFormat::MJPEG, 30),
                CameraFormat::new(Resolution::new(1280, 720), FrameFormat::YUYV, 30),
                CameraFormat::new(Resolution::new(1920, 1080), FrameFormat::NV12, 60),
            ];

            let selected =
                crate::camera::select_best_camera_format(&formats, &request(1280, 720, 30))
                    .unwrap();

            assert_eq!(selected.resolution(), Resolution::new(1280, 720));
            assert_eq!(selected.format(), FrameFormat::YUYV);
            assert_eq!(selected.frame_rate(), 30);
        }

        #[test]
        fn fallback_format_selection_prefers_nearest_resolution_before_format_priority() {
            let formats = vec![
                CameraFormat::new(Resolution::new(1920, 1080), FrameFormat::MJPEG, 30),
                CameraFormat::new(Resolution::new(1024, 768), FrameFormat::YUYV, 30),
            ];

            let selected =
                crate::camera::select_best_camera_format(&formats, &request(1280, 720, 30))
                    .unwrap();

            assert_eq!(selected.resolution(), Resolution::new(1024, 768));
            assert_eq!(selected.format(), FrameFormat::YUYV);
        }

        #[test]
        fn next_pacing_deadline_advances_by_one_interval_when_on_schedule() {
            let interval = Duration::from_millis(33);
            let deadline = Instant::now();
            let now = deadline;

            let next = next_pacing_deadline(deadline, interval, now);

            assert_eq!(next, deadline + interval);
        }

        #[test]
        fn next_pacing_deadline_resets_to_now_when_behind_schedule() {
            let interval = Duration::from_millis(10);
            let deadline = Instant::now();
            let now = deadline + Duration::from_millis(500);

            let next = next_pacing_deadline(deadline, interval, now);

            assert_eq!(next, now);
        }

        #[test]
        fn next_pacing_deadline_does_not_drift_across_consecutive_frames() {
            let interval = Duration::from_millis(20);
            let start = Instant::now();
            let mut deadline = start;

            for _ in 0..3 {
                deadline = next_pacing_deadline(deadline, interval, start);
            }

            assert_eq!(deadline, start + interval * 3);
        }

        #[test]
        fn camera_frame_failure_cap_trips_only_at_consecutive_max() {
            let mut consecutive_failures: u32 = 0;

            for _ in 1..CAMERA_FRAME_FAILURES_CONSECUTIVE_MAX {
                assert!(!camera_frame_failure_cap_reached(&mut consecutive_failures));
            }

            assert!(camera_frame_failure_cap_reached(&mut consecutive_failures));
            assert_eq!(consecutive_failures, CAMERA_FRAME_FAILURES_CONSECUTIVE_MAX);
        }

        #[test]
        fn camera_frame_failure_cap_resets_after_success() {
            let mut consecutive_failures: u32 = 0;
            assert!(!camera_frame_failure_cap_reached(&mut consecutive_failures));
            assert_eq!(consecutive_failures, 1);

            consecutive_failures = 0;

            assert!(!camera_frame_failure_cap_reached(&mut consecutive_failures));
            assert_eq!(consecutive_failures, 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_opts_applies_defaults() {
        let req = CameraRequest::from_opts(
            None,
            None,
            None,
            None,
            false,
            crate::camera_background::CameraBackgroundConfig::default(),
        );
        assert_eq!(
            req,
            CameraRequest {
                selector: CameraSelector::Index(0),
                width: 1280,
                height: 720,
                fps: 30,
                mirror: false,
                background: crate::camera_background::CameraBackgroundConfig::default(),
            }
        );
    }

    #[test]
    fn from_opts_parses_device_index() {
        assert_eq!(
            CameraRequest::from_opts(
                Some("2"),
                None,
                None,
                None,
                false,
                crate::camera_background::CameraBackgroundConfig::default(),
            )
            .selector,
            CameraSelector::Index(2)
        );
        assert_eq!(
            CameraRequest::from_opts(
                Some(" 3 "),
                None,
                None,
                None,
                false,
                crate::camera_background::CameraBackgroundConfig::default(),
            )
            .selector,
            CameraSelector::Index(3)
        );
        assert_eq!(
            CameraRequest::from_opts(
                Some(""),
                None,
                None,
                None,
                false,
                crate::camera_background::CameraBackgroundConfig::default(),
            )
            .selector,
            CameraSelector::Index(0)
        );
        assert_eq!(
            CameraRequest::from_opts(
                Some("default"),
                None,
                None,
                None,
                false,
                crate::camera_background::CameraBackgroundConfig::default(),
            )
            .selector,
            CameraSelector::Index(0)
        );
    }

    #[test]
    fn from_opts_preserves_string_device_ids() {
        assert_eq!(
            CameraRequest::from_opts(
                Some("front"),
                None,
                None,
                None,
                false,
                crate::camera_background::CameraBackgroundConfig::default(),
            )
            .selector,
            CameraSelector::Id("front".to_string())
        );
    }

    #[test]
    fn from_opts_forces_even_dims_and_honours_overrides() {
        let req = CameraRequest::from_opts(
            Some("1"),
            Some(641),
            Some(481),
            Some(24),
            true,
            crate::camera_background::CameraBackgroundConfig::default(),
        );
        assert_eq!(
            req,
            CameraRequest {
                selector: CameraSelector::Index(1),
                width: 640,
                height: 480,
                fps: 24,
                mirror: true,
                background: crate::camera_background::CameraBackgroundConfig::default(),
            }
        );
    }

    #[test]
    fn from_opts_rejects_degenerate_dims_and_fps() {
        let req = CameraRequest::from_opts(
            None,
            Some(0),
            Some(1),
            Some(0),
            false,
            crate::camera_background::CameraBackgroundConfig::default(),
        );
        assert_eq!(
            req,
            CameraRequest {
                selector: CameraSelector::Index(0),
                width: 1280,
                height: 720,
                fps: 30,
                mirror: false,
                background: crate::camera_background::CameraBackgroundConfig::default(),
            }
        );
    }

    #[test]
    fn mirror_i420_in_place_reverses_luma_and_chroma_rows() {
        let mut frame = crate::yuv::I420 {
            width: 4,
            height: 4,
            y: (0u8..16).collect(),
            u: vec![16, 17, 18, 19],
            v: vec![20, 21, 22, 23],
        };

        mirror_i420_in_place(&mut frame);

        assert_eq!(
            frame.y,
            vec![3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12]
        );
        assert_eq!(frame.u, vec![17, 16, 19, 18]);
        assert_eq!(frame.v, vec![21, 20, 23, 22]);
    }

    #[test]
    fn camera_device_aliases_include_distinct_device_id_and_index() {
        let aliases = camera_device_aliases("native-camera-id", "0");
        assert_eq!(
            aliases,
            vec!["native-camera-id".to_string(), "0".to_string()]
        );
    }

    #[test]
    fn camera_device_aliases_deduplicate_empty_and_repeated_values() {
        let aliases = camera_device_aliases("0", "0");
        assert_eq!(aliases, vec!["0".to_string()]);
        let aliases = camera_device_aliases("", "2");
        assert_eq!(aliases, vec!["2".to_string()]);
    }
}
