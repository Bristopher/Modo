// SPDX-License-Identifier: AGPL-3.0-or-later

use core::ffi::c_void;
use core::ptr::NonNull;

pub type IoSurfaceRaw = NonNull<c_void>;

#[cfg(target_os = "macos")]
#[link(name = "IOSurface", kind = "framework")]
unsafe extern "C" {
    fn IOSurfaceIncrementUseCount(buffer: *mut c_void);
    fn IOSurfaceDecrementUseCount(buffer: *mut c_void);
    fn IOSurfaceGetUseCount(buffer: *mut c_void) -> i32;
}

#[cfg(target_os = "macos")]
pub unsafe fn iosurface_increment_use_count(surface: IoSurfaceRaw) {
    unsafe { IOSurfaceIncrementUseCount(surface.as_ptr()) };
}

#[cfg(target_os = "macos")]
pub unsafe fn iosurface_decrement_use_count(surface: IoSurfaceRaw) {
    unsafe { IOSurfaceDecrementUseCount(surface.as_ptr()) };
}

#[cfg(target_os = "macos")]
pub unsafe fn iosurface_use_count(surface: IoSurfaceRaw) -> i32 {
    unsafe { IOSurfaceGetUseCount(surface.as_ptr()) }
}

#[cfg(not(target_os = "macos"))]
pub unsafe fn iosurface_increment_use_count(_surface: IoSurfaceRaw) {}

#[cfg(not(target_os = "macos"))]
pub unsafe fn iosurface_decrement_use_count(_surface: IoSurfaceRaw) {}

#[cfg(not(target_os = "macos"))]
pub unsafe fn iosurface_use_count(_surface: IoSurfaceRaw) -> i32 {
    0
}

pub struct IoSurfacePair {
    current: Option<IoSurfaceRaw>,
    prev: Option<IoSurfaceRaw>,
}

unsafe impl Send for IoSurfacePair {}

impl IoSurfacePair {
    pub fn new() -> Self {
        let pair = Self {
            current: None,
            prev: None,
        };
        assert!(pair.current.is_none());
        assert!(pair.prev.is_none());
        pair
    }

    pub fn has_current(&self) -> bool {
        let has = self.current.is_some();
        assert!(has == self.current.is_some());
        has
    }

    pub fn has_prev(&self) -> bool {
        let has = self.prev.is_some();
        assert!(has == self.prev.is_some());
        has
    }

    pub unsafe fn push(&mut self, new: IoSurfaceRaw) {
        unsafe { iosurface_increment_use_count(new) };
        let evicted = self.prev.take();
        let rotated = self.current.take();
        self.prev = rotated;
        self.current = Some(new);
        assert!(self.current.is_some());
        if let Some(old) = evicted {
            unsafe { iosurface_decrement_use_count(old) };
        }
    }

    pub fn take_current(&mut self) -> Option<IoSurfaceRaw> {
        let taken = self.current.take();
        assert!(self.current.is_none());
        taken
    }

    pub fn peek_current(&self) -> Option<IoSurfaceRaw> {
        self.current
    }

    pub fn peek_prev(&self) -> Option<IoSurfaceRaw> {
        self.prev
    }

    pub fn clear(&mut self) {
        let cur = self.current.take();
        let prev = self.prev.take();
        assert!(self.current.is_none());
        assert!(self.prev.is_none());
        if let Some(s) = cur {
            unsafe { iosurface_decrement_use_count(s) };
        }
        if let Some(s) = prev {
            unsafe { iosurface_decrement_use_count(s) };
        }
    }
}

impl Default for IoSurfacePair {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for IoSurfacePair {
    fn drop(&mut self) {
        let cur = self.current.take();
        let prev = self.prev.take();
        if let Some(s) = cur {
            unsafe { iosurface_decrement_use_count(s) };
        }
        if let Some(s) = prev {
            unsafe { iosurface_decrement_use_count(s) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ptr::NonNull;

    fn fake_surface(addr: usize) -> IoSurfaceRaw {
        assert!(addr != 0);
        NonNull::new(addr as *mut c_void).expect("non-null fake surface")
    }

    #[test]
    fn new_pair_is_empty() {
        let pair = IoSurfacePair::new();
        assert!(!pair.has_current());
        assert!(!pair.has_prev());
    }

    #[test]
    fn take_current_on_empty_returns_none() {
        let mut pair = IoSurfacePair::new();
        assert!(pair.take_current().is_none());
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn push_rotates_current_to_prev_offplatform() {
        let mut pair = IoSurfacePair::new();
        let a = fake_surface(0x1);
        let b = fake_surface(0x2);
        unsafe {
            pair.push(a);
        }
        assert_eq!(pair.peek_current(), Some(a));
        assert!(pair.peek_prev().is_none());
        unsafe {
            pair.push(b);
        }
        assert_eq!(pair.peek_current(), Some(b));
        assert_eq!(pair.peek_prev(), Some(a));
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn take_current_clears_slot_offplatform() {
        let mut pair = IoSurfacePair::new();
        let a = fake_surface(0x10);
        unsafe {
            pair.push(a);
        }
        let taken = pair.take_current().expect("current is set");
        assert_eq!(taken, a);
        assert!(pair.peek_current().is_none());
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn drop_pair_is_safe_offplatform() {
        let mut pair = IoSurfacePair::new();
        unsafe {
            pair.push(fake_surface(0x100));
            pair.push(fake_surface(0x200));
        }
        drop(pair);
    }
}

#[cfg(all(test, target_os = "macos"))]
mod macos_tests {
    use super::*;
    use core::ffi::c_void;
    use core::ptr::NonNull;
    use objc2_core_foundation::{
        CFDictionary, CFNumber, CFRetained, CFString, kCFAllocatorDefault,
        kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks,
    };

    #[link(name = "IOSurface", kind = "framework")]
    unsafe extern "C" {
        fn IOSurfaceCreate(properties: *const CFDictionary) -> *mut c_void;
    }

    fn cf_number_i32(v: i32) -> CFRetained<CFNumber> {
        CFNumber::new_i32(v)
    }

    fn cf_string(s: &'static str) -> CFRetained<CFString> {
        CFString::from_static_str(s)
    }

    fn make_iosurface() -> IoSurfaceRaw {
        let width_key = cf_string("IOSurfaceWidth");
        let height_key = cf_string("IOSurfaceHeight");
        let bpe_key = cf_string("IOSurfaceBytesPerElement");
        let pf_key = cf_string("IOSurfacePixelFormat");
        let width_val = cf_number_i32(32);
        let height_val = cf_number_i32(32);
        let bpe_val = cf_number_i32(4);
        let pf_val = cf_number_i32(i32::from_be_bytes(*b"BGRA"));
        let keys: [*const c_void; 4] = [
            CFRetained::as_ptr(&width_key).as_ptr() as *const c_void,
            CFRetained::as_ptr(&height_key).as_ptr() as *const c_void,
            CFRetained::as_ptr(&bpe_key).as_ptr() as *const c_void,
            CFRetained::as_ptr(&pf_key).as_ptr() as *const c_void,
        ];
        let vals: [*const c_void; 4] = [
            CFRetained::as_ptr(&width_val).as_ptr() as *const c_void,
            CFRetained::as_ptr(&height_val).as_ptr() as *const c_void,
            CFRetained::as_ptr(&bpe_val).as_ptr() as *const c_void,
            CFRetained::as_ptr(&pf_val).as_ptr() as *const c_void,
        ];
        let dict_opt = unsafe {
            CFDictionary::new(
                kCFAllocatorDefault,
                keys.as_ptr() as *mut *const c_void,
                vals.as_ptr() as *mut *const c_void,
                4,
                &kCFTypeDictionaryKeyCallBacks,
                &kCFTypeDictionaryValueCallBacks,
            )
        };
        let dict = dict_opt.expect("CFDictionary::new returned non-null");
        let dict_ptr = CFRetained::as_ptr(&dict).as_ptr() as *const CFDictionary;
        let raw = unsafe { IOSurfaceCreate(dict_ptr) };
        assert!(!raw.is_null(), "IOSurfaceCreate must produce a surface");
        NonNull::new(raw).expect("non-null IOSurface")
    }

    #[test]
    fn push_increments_use_count_and_take_current_does_not_release() {
        let a = make_iosurface();
        let before = unsafe { iosurface_use_count(a) };
        let mut pair = IoSurfacePair::new();
        unsafe {
            pair.push(a);
        }
        let after_push = unsafe { iosurface_use_count(a) };
        assert_eq!(after_push, before + 1);
        let taken = pair.take_current().expect("current set");
        assert_eq!(taken, a);
        let after_take = unsafe { iosurface_use_count(a) };
        assert_eq!(after_take, before + 1);
        unsafe { iosurface_decrement_use_count(a) };
        let after_balance = unsafe { iosurface_use_count(a) };
        assert_eq!(after_balance, before);
    }

    #[test]
    fn push_rotates_and_drop_releases_both_slots() {
        let a = make_iosurface();
        let b = make_iosurface();
        let before_a = unsafe { iosurface_use_count(a) };
        let before_b = unsafe { iosurface_use_count(b) };
        let mut pair = IoSurfacePair::new();
        unsafe {
            pair.push(a);
            pair.push(b);
        }
        assert_eq!(pair.peek_current(), Some(b));
        assert_eq!(pair.peek_prev(), Some(a));
        assert_eq!(unsafe { iosurface_use_count(a) }, before_a + 1);
        assert_eq!(unsafe { iosurface_use_count(b) }, before_b + 1);
        drop(pair);
        assert_eq!(unsafe { iosurface_use_count(a) }, before_a);
        assert_eq!(unsafe { iosurface_use_count(b) }, before_b);
    }

    #[test]
    fn third_push_evicts_oldest_and_releases_it() {
        let a = make_iosurface();
        let b = make_iosurface();
        let c = make_iosurface();
        let before_a = unsafe { iosurface_use_count(a) };
        let mut pair = IoSurfacePair::new();
        unsafe {
            pair.push(a);
            pair.push(b);
            pair.push(c);
        }
        assert_eq!(unsafe { iosurface_use_count(a) }, before_a);
        assert_eq!(pair.peek_current(), Some(c));
        assert_eq!(pair.peek_prev(), Some(b));
    }

    #[test]
    fn clear_releases_both_slots() {
        let a = make_iosurface();
        let b = make_iosurface();
        let before_a = unsafe { iosurface_use_count(a) };
        let before_b = unsafe { iosurface_use_count(b) };
        let mut pair = IoSurfacePair::new();
        unsafe {
            pair.push(a);
            pair.push(b);
        }
        pair.clear();
        assert_eq!(unsafe { iosurface_use_count(a) }, before_a);
        assert_eq!(unsafe { iosurface_use_count(b) }, before_b);
        assert!(pair.peek_current().is_none());
        assert!(pair.peek_prev().is_none());
    }
}
