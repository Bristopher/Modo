#![allow(non_snake_case)]

// SPDX-License-Identifier: AGPL-3.0-or-later

use objc2::rc::Retained;
use objc2_core_foundation::CGRect;
use objc2_core_media::{CMTime, CMTimeFlags};
use objc2_foundation::NSString;
use objc2_screen_capture_kit::{
    SCDisplay, SCRunningApplication, SCStream, SCStreamConfiguration, SCWindow,
};

pub use objc2_screen_capture_kit::SCStreamOutputType;

pub fn cgrect_standardized(r: CGRect) -> CGRect {
    let mut out = r;
    if out.size.width < 0.0 {
        out.origin.x += out.size.width;
        out.size.width = -out.size.width;
    }
    if out.size.height < 0.0 {
        out.origin.y += out.size.height;
        out.size.height = -out.size.height;
    }
    out
}

pub fn cgrect_intersection_area(a_raw: CGRect, b_raw: CGRect) -> f64 {
    let a = cgrect_standardized(a_raw);
    let b = cgrect_standardized(b_raw);
    if a.size.width <= 0.0 || a.size.height <= 0.0 || b.size.width <= 0.0 || b.size.height <= 0.0 {
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

pub fn cmtime_seconds(value: i64, timescale: i32) -> CMTime {
    CMTime {
        value,
        timescale,
        flags: CMTimeFlags(1),
        epoch: 0,
    }
}

pub fn sc_running_application_process_id(app: &SCRunningApplication) -> i32 {
    unsafe { app.processID() }
}

pub fn sc_running_application_bundle_identifier(app: &SCRunningApplication) -> Retained<NSString> {
    unsafe { app.bundleIdentifier() }
}

pub fn sc_running_application_name(app: &SCRunningApplication) -> Retained<NSString> {
    unsafe { app.applicationName() }
}

pub fn sc_display_frame(display: &SCDisplay) -> CGRect {
    unsafe { display.frame() }
}

pub fn sc_window_owning_application(win: &SCWindow) -> Option<Retained<SCRunningApplication>> {
    unsafe { win.owningApplication() }
}

pub fn sc_window_frame(win: &SCWindow) -> CGRect {
    unsafe { win.frame() }
}

pub fn cfg_set_captures_audio(cfg: &SCStreamConfiguration, v: bool) {
    unsafe { cfg.setCapturesAudio(v) }
}
pub fn cfg_set_excludes_current_process_audio(cfg: &SCStreamConfiguration, v: bool) {
    unsafe { cfg.setExcludesCurrentProcessAudio(v) }
}
pub fn cfg_set_sample_rate(cfg: &SCStreamConfiguration, v: isize) {
    unsafe { cfg.setSampleRate(v) }
}
pub fn cfg_set_channel_count(cfg: &SCStreamConfiguration, v: isize) {
    unsafe { cfg.setChannelCount(v) }
}
pub fn cfg_set_queue_depth(cfg: &SCStreamConfiguration, v: isize) {
    unsafe { cfg.setQueueDepth(v) }
}
pub fn cfg_set_width(cfg: &SCStreamConfiguration, v: usize) {
    unsafe { cfg.setWidth(v) }
}
pub fn cfg_set_height(cfg: &SCStreamConfiguration, v: usize) {
    unsafe { cfg.setHeight(v) }
}
pub fn cfg_set_shows_cursor(cfg: &SCStreamConfiguration, v: bool) {
    unsafe { cfg.setShowsCursor(v) }
}
pub fn cfg_set_minimum_frame_interval(cfg: &SCStreamConfiguration, t: CMTime) {
    unsafe { cfg.setMinimumFrameInterval(t) }
}

pub fn cfg_set_capture_dynamic_range_sdr_if_available(cfg: &SCStreamConfiguration) {
    use objc2::runtime::NSObjectProtocol;
    use objc2::sel;
    let obj: &objc2::runtime::NSObject = cfg.as_ref();
    if obj.respondsToSelector(sel!(setCaptureDynamicRange:)) {
        unsafe {
            cfg.setCaptureDynamicRange(objc2_screen_capture_kit::SCCaptureDynamicRange(0));
        }
    }
}

pub fn cfg_set_stream_name_if_available(cfg: &SCStreamConfiguration, name: &NSString) {
    use objc2::runtime::NSObjectProtocol;
    use objc2::sel;
    let obj: &objc2::runtime::NSObject = cfg.as_ref();
    if obj.respondsToSelector(sel!(setStreamName:)) {
        unsafe {
            cfg.setStreamName(Some(name));
        }
    }
}

pub fn sc_stream_add_stream_output(
    stream: &SCStream,
    output: &objc2::runtime::ProtocolObject<dyn objc2_screen_capture_kit::SCStreamOutput>,
    kind: SCStreamOutputType,
    queue: Option<&dispatch2::DispatchQueue>,
) -> Result<(), Retained<objc2_foundation::NSError>> {
    unsafe { stream.addStreamOutput_type_sampleHandlerQueue_error(output, kind, queue) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_core_foundation::{CGPoint, CGSize};

    #[test]
    fn cgrect_intersection_handles_negative_and_disjoint() {
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
        assert_eq!(25.0, cgrect_intersection_area(a, b));

        let c = CGRect {
            origin: CGPoint { x: 10.0, y: 10.0 },
            size: CGSize {
                width: -5.0,
                height: -5.0,
            },
        };
        assert_eq!(25.0, cgrect_intersection_area(a, c));

        let d = CGRect {
            origin: CGPoint { x: 20.0, y: 20.0 },
            size: CGSize {
                width: 2.0,
                height: 2.0,
            },
        };
        assert_eq!(0.0, cgrect_intersection_area(a, d));
    }
}
