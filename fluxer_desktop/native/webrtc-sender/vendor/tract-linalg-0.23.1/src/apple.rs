#[cfg(any(target_os = "macos", all(target_os = "ios", feature = "apple-amx-ios")))]

#[cfg(target_os = "ios")]
lazy_static::lazy_static! {
    static ref IPHONE_MODEL_MAJOR:Option<usize> = {
        use std::ffi::{c_char, c_void, CStr, CString};
        use std::ptr::null_mut;

        extern "C" {
            fn sysctlbyname(
                name: *const c_char,
                oldp: *mut c_void,
                oldlenp: *mut isize,
                newp: *mut c_void,
                newlen: isize,
            );
        }

        unsafe {
            let mut len: isize = 0;
            let name = CString::new("hw.machine").unwrap();
            sysctlbyname(name.as_ptr(), null_mut(), &mut len, null_mut(), 0);
            let mut buf = vec![0u8; len as _];
            sysctlbyname(name.as_ptr(), buf.as_mut_ptr() as _, &mut len, null_mut(), 0);
            let version = CStr::from_bytes_with_nul(&buf).unwrap().to_string_lossy().into_owned();
            let Some((major, _)) = version.trim_start_matches("iPhone").split_once(",") else { return None };
            major.parse::<usize>().ok()
        }
    };
}

#[cfg(target_os = "macos")]
pub fn has_amx() -> bool {
    true
}

#[cfg(all(target_os = "ios", feature = "apple-amx-ios"))]
fn has_amx() -> bool {
    // iPhone12,1 is the one branded "iPhone 11", with Apple A13 bionic, first CPU featuring amx
    IPHONE_MODEL_MAJOR.map(|it| it >= 12).unwrap_or(false)
}

#[inline]
#[cfg(target_os = "ios")]
pub fn has_fp16() -> bool {
    // iPhone10,1 is the one branded "iPhone 8", with Apple A11 bionic, first CPU featuring fp16
    IPHONE_MODEL_MAJOR.map(|it| it >= 10).unwrap_or(false)
}

#[inline]
#[cfg(not(target_os = "ios"))]
pub fn has_fp16() -> bool {
    cfg!(target_os = "macos")
        || cfg!(feature_cpu = "fp16")
        || *KIND == Kind::CortexA55
        || *KIND == Kind::CortexA75
        || *HAS_FP16
}
