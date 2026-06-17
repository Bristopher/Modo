// SPDX-License-Identifier: AGPL-3.0-or-later

pub const SCK_MIN_MACOS: (i64, i64, i64) = (12, 3, 0);

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
    pub reason: String,
}

pub fn classify_support(detected: Option<(i64, i64, i64)>) -> SupportClassification {
    let min_sck = format_version(SCK_MIN_MACOS);
    match detected {
        None => SupportClassification {
            supported: false,
            sck_available: false,
            reason: "mac-screen-capture could not detect the running macOS version. \
                 Native screen capture unavailable."
                .to_owned(),
        },
        Some(v) => {
            let detected_str = format_version(v);
            let sck_ok = meets_floor(v, SCK_MIN_MACOS);
            let reason = if sck_ok {
                format!(
                    "mac-screen-capture supported on macOS {detected_str} \
                     (ScreenCaptureKit, requires macOS {min_sck}+)."
                )
            } else {
                format!(
                    "mac-screen-capture requires macOS {min_sck}+ (ScreenCaptureKit). \
                     This Mac is running macOS {detected_str}. Native cursor-hidden \
                     screen capture unavailable; fall back to getDisplayMedia."
                )
            };
            SupportClassification {
                supported: sck_ok,
                sck_available: sck_ok,
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
    fn rejects_below_floor() {
        assert!(!meets_floor((12, 2, 9), SCK_MIN_MACOS));
        assert!(!meets_floor((11, 7, 10), SCK_MIN_MACOS));
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
        assert!(c.reason.contains("could not detect"));
    }

    #[test]
    fn classify_macos_11_is_unsupported() {
        let c = classify_support(Some((11, 7, 10)));
        assert!(!c.supported);
        assert!(!c.sck_available);
        assert!(c.reason.contains("macOS 12.3+"));
        assert!(c.reason.contains("macOS 11.7.10"));
    }

    #[test]
    fn classify_macos_12_3_is_supported() {
        let c = classify_support(Some((12, 3, 0)));
        assert!(c.supported);
        assert!(c.sck_available);
        assert!(c.reason.contains("ScreenCaptureKit"));
    }
}
