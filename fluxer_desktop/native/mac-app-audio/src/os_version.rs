// SPDX-License-Identifier: AGPL-3.0-or-later

pub const SCK_MIN_MACOS: (i64, i64, i64) = (12, 3, 0);

pub const COREAUDIO_TAP_MIN_MACOS: (i64, i64, i64) = (14, 2, 0);

pub fn meets_floor(version: (i64, i64, i64), floor: (i64, i64, i64)) -> bool {
    if version.0 != floor.0 {
        return version.0 > floor.0;
    }
    if version.1 != floor.1 {
        return version.1 > floor.1;
    }
    version.2 >= floor.2
}

pub fn format_version(version: (i64, i64, i64)) -> String {
    if version.2 == 0 {
        format!("{}.{}", version.0, version.1)
    } else {
        format!("{}.{}.{}", version.0, version.1, version.2)
    }
}

#[cfg(target_os = "macos")]
pub fn current_macos_version() -> Option<(i64, i64, i64)> {
    use objc2_foundation::NSProcessInfo;
    let info = NSProcessInfo::processInfo();
    let v = info.operatingSystemVersion();
    Some((
        v.majorVersion as i64,
        v.minorVersion as i64,
        v.patchVersion as i64,
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn current_macos_version() -> Option<(i64, i64, i64)> {
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportClassification {
    pub supported: bool,
    pub sck_available: bool,
    pub coreaudio_available: bool,
    pub reason: String,
}

pub fn classify_support(detected: Option<(i64, i64, i64)>) -> SupportClassification {
    let min_sck = format_version(SCK_MIN_MACOS);
    let min_coreaudio = format_version(COREAUDIO_TAP_MIN_MACOS);
    match detected {
        None => SupportClassification {
            supported: false,
            sck_available: false,
            coreaudio_available: false,
            reason: "mac-app-audio could not detect the running macOS version. \
                 Per-app and self-excluding desktop audio capture unavailable."
                .to_owned(),
        },
        Some(v) => {
            let detected_str = format_version(v);
            let sck_ok = meets_floor(v, SCK_MIN_MACOS);
            let coreaudio_ok = meets_floor(v, COREAUDIO_TAP_MIN_MACOS);
            let supported = sck_ok || coreaudio_ok;
            let reason = if supported {
                if coreaudio_ok {
                    format!(
                        "mac-app-audio supported on macOS {detected_str} \
                         (CoreAudio process tap, requires macOS {min_coreaudio}+; \
                         ScreenCaptureKit fallback requires macOS {min_sck}+)."
                    )
                } else {
                    format!(
                        "mac-app-audio supported on macOS {detected_str} \
                         (ScreenCaptureKit per-app capture, requires macOS {min_sck}+). \
                         CoreAudio process tap requires macOS {min_coreaudio}+ \
                         and is unavailable here."
                    )
                }
            } else {
                format!(
                    "mac-app-audio requires macOS {min_sck}+ (ScreenCaptureKit). \
                     This Mac is running macOS {detected_str}. Per-app audio capture \
                     unavailable; Fluxer must not use a broader audio route that could \
                     include unrelated apps or call audio."
                )
            };
            SupportClassification {
                supported,
                sck_available: sck_ok,
                coreaudio_available: coreaudio_ok,
                reason,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meets_floor_exact_match() {
        assert!(meets_floor((12, 3, 0), SCK_MIN_MACOS));
    }

    #[test]
    fn meets_floor_higher_major() {
        assert!(meets_floor((14, 0, 0), SCK_MIN_MACOS));
    }

    #[test]
    fn meets_floor_higher_minor() {
        assert!(meets_floor((12, 4, 0), SCK_MIN_MACOS));
    }

    #[test]
    fn rejects_below_floor_minor() {
        assert!(!meets_floor((12, 2, 9), SCK_MIN_MACOS));
    }

    #[test]
    fn rejects_below_floor_major() {
        assert!(!meets_floor((11, 7, 10), SCK_MIN_MACOS));
    }

    #[test]
    fn rejects_macos_10_15_for_sck() {
        assert!(!meets_floor((10, 15, 7), SCK_MIN_MACOS));
    }

    #[test]
    fn coreaudio_floor_14_2() {
        assert!(meets_floor((14, 2, 0), COREAUDIO_TAP_MIN_MACOS));
        assert!(!meets_floor((14, 1, 9), COREAUDIO_TAP_MIN_MACOS));
        assert!(meets_floor((15, 0, 0), COREAUDIO_TAP_MIN_MACOS));
    }

    #[test]
    fn format_version_trims_zero_patch() {
        assert_eq!("12.3", format_version((12, 3, 0)));
        assert_eq!("14.2.1", format_version((14, 2, 1)));
    }

    #[test]
    fn classify_unknown_version_is_unsupported() {
        let c = classify_support(None);
        assert!(!c.supported);
        assert!(!c.sck_available);
        assert!(!c.coreaudio_available);
        assert!(c.reason.contains("could not detect"));
    }

    #[test]
    fn classify_macos_10_15_is_unsupported_and_mentions_min_version() {
        let c = classify_support(Some((10, 15, 7)));
        assert!(!c.supported);
        assert!(!c.sck_available);
        assert!(!c.coreaudio_available);
        assert!(c.reason.contains("macOS 12.3+"), "reason: {}", c.reason);
        assert!(c.reason.contains("macOS 10.15.7"), "reason: {}", c.reason);
        assert!(c.reason.contains("ScreenCaptureKit"));
    }

    #[test]
    fn classify_macos_12_3_is_sck_only() {
        let c = classify_support(Some((12, 3, 0)));
        assert!(c.supported);
        assert!(c.sck_available);
        assert!(!c.coreaudio_available);
        assert!(c.reason.contains("ScreenCaptureKit per-app capture"));
        assert!(
            c.reason
                .contains("CoreAudio process tap requires macOS 14.2+")
        );
    }

    #[test]
    fn classify_macos_14_2_has_both_backends() {
        let c = classify_support(Some((14, 2, 0)));
        assert!(c.supported);
        assert!(c.sck_available);
        assert!(c.coreaudio_available);
        assert!(c.reason.contains("CoreAudio process tap"));
    }

    #[test]
    fn classify_macos_15_is_supported() {
        let c = classify_support(Some((15, 0, 0)));
        assert!(c.supported);
        assert!(c.sck_available);
        assert!(c.coreaudio_available);
    }
}
