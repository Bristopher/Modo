// SPDX-License-Identifier: AGPL-3.0-or-later

use napi_derive::napi;

const STATUS_GRANTED: &str = "granted";
const STATUS_DENIED: &str = "denied";
const STATUS_NOT_DETERMINED: &str = "not-determined";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
enum TccStatus {
    Granted,
    Denied,
    NotDetermined,
}

impl TccStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Granted => STATUS_GRANTED,
            Self::Denied => STATUS_DENIED,
            Self::NotDetermined => STATUS_NOT_DETERMINED,
        }
    }
}

#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
const IOHID_ACCESS_GRANTED: u32 = 0;
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
const IOHID_ACCESS_DENIED: u32 = 1;

#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
const fn input_monitoring_status_from_iohid(access: u32) -> TccStatus {
    match access {
        IOHID_ACCESS_GRANTED => TccStatus::Granted,
        IOHID_ACCESS_DENIED => TccStatus::Denied,
        _ => TccStatus::NotDetermined,
    }
}

fn status_string(status: TccStatus) -> String {
    status.as_str().to_owned()
}

#[napi(js_name = "screenRecordingStatus")]
pub fn screen_recording_status() -> String {
    status_string(platform::screen_recording_status())
}

#[napi(js_name = "requestScreenRecording")]
pub fn request_screen_recording() -> String {
    status_string(platform::request_screen_recording())
}

#[napi(js_name = "inputMonitoringStatus")]
pub fn input_monitoring_status() -> String {
    status_string(platform::input_monitoring_status())
}

#[napi(js_name = "requestInputMonitoring")]
pub fn request_input_monitoring() -> String {
    status_string(platform::request_input_monitoring())
}

#[cfg(target_os = "macos")]
mod platform {
    use core_graphics::access::ScreenCaptureAccess;
    use objc2_io_kit::{IOHIDCheckAccess, IOHIDRequestAccess, IOHIDRequestType};

    use super::{TccStatus, input_monitoring_status_from_iohid};

    pub(super) fn screen_recording_status() -> TccStatus {
        if ScreenCaptureAccess.preflight() {
            TccStatus::Granted
        } else {
            TccStatus::Denied
        }
    }

    pub(super) fn request_screen_recording() -> TccStatus {
        if ScreenCaptureAccess.request() {
            TccStatus::Granted
        } else {
            TccStatus::Denied
        }
    }

    pub(super) fn input_monitoring_status() -> TccStatus {
        let access = IOHIDCheckAccess(IOHIDRequestType::ListenEvent);
        input_monitoring_status_from_iohid(access.0)
    }

    pub(super) fn request_input_monitoring() -> TccStatus {
        if IOHIDRequestAccess(IOHIDRequestType::ListenEvent) {
            TccStatus::Granted
        } else {
            TccStatus::Denied
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::TccStatus;

    pub(super) fn screen_recording_status() -> TccStatus {
        TccStatus::NotDetermined
    }

    pub(super) fn request_screen_recording() -> TccStatus {
        TccStatus::NotDetermined
    }

    pub(super) fn input_monitoring_status() -> TccStatus {
        TccStatus::NotDetermined
    }

    pub(super) fn request_input_monitoring() -> TccStatus {
        TccStatus::NotDetermined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_strings_match_js_contract() {
        assert_eq!(TccStatus::Granted.as_str(), "granted");
        assert_eq!(TccStatus::Denied.as_str(), "denied");
        assert_eq!(TccStatus::NotDetermined.as_str(), "not-determined");
    }

    #[test]
    fn input_monitoring_maps_known_iohid_statuses() {
        assert_eq!(
            input_monitoring_status_from_iohid(IOHID_ACCESS_GRANTED),
            TccStatus::Granted
        );
        assert_eq!(
            input_monitoring_status_from_iohid(IOHID_ACCESS_DENIED),
            TccStatus::Denied
        );
        assert_eq!(
            input_monitoring_status_from_iohid(2),
            TccStatus::NotDetermined
        );
        assert_eq!(
            input_monitoring_status_from_iohid(u32::MAX),
            TccStatus::NotDetermined
        );
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn non_macos_exports_preserve_stub_contract() {
        assert_eq!(screen_recording_status(), "not-determined");
        assert_eq!(request_screen_recording(), "not-determined");
        assert_eq!(input_monitoring_status(), "not-determined");
        assert_eq!(request_input_monitoring(), "not-determined");
    }
}
