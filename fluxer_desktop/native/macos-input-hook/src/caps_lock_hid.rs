// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ffi::c_void;
use std::ptr;

use core_foundation::base::{CFAllocatorRef, CFRelease, TCFType, kCFAllocatorDefault};
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::number::CFNumber;
use core_foundation::runloop::{CFRunLoop, CFRunLoopRef, kCFRunLoopCommonModes};
use core_foundation::string::{CFString, CFStringRef};

const KEYBOARD_USAGE_PAGE: u32 = 0x07;
const CAPS_LOCK_USAGE: u32 = 0x39;
const GENERIC_DESKTOP_USAGE_PAGE: i32 = 0x01;
const KEYBOARD_DEVICE_USAGE: i32 = 0x06;
const HID_OPTIONS_NONE: u32 = 0;
const KERN_SUCCESS: i32 = 0;

type IOHIDManagerRef = *mut c_void;
type IOHIDValueRef = *mut c_void;
type IOHIDElementRef = *mut c_void;
type IOHIDValueCallback = extern "C" fn(*mut c_void, i32, *mut c_void, IOHIDValueRef);

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOHIDManagerCreate(allocator: CFAllocatorRef, options: u32) -> IOHIDManagerRef;
    fn IOHIDManagerSetDeviceMatching(manager: IOHIDManagerRef, matching: CFDictionaryRef);
    fn IOHIDManagerSetInputValueMatching(manager: IOHIDManagerRef, matching: CFDictionaryRef);
    fn IOHIDManagerRegisterInputValueCallback(
        manager: IOHIDManagerRef,
        callback: Option<IOHIDValueCallback>,
        context: *mut c_void,
    );
    fn IOHIDManagerScheduleWithRunLoop(
        manager: IOHIDManagerRef,
        run_loop: CFRunLoopRef,
        mode: CFStringRef,
    );
    fn IOHIDManagerUnscheduleFromRunLoop(
        manager: IOHIDManagerRef,
        run_loop: CFRunLoopRef,
        mode: CFStringRef,
    );
    fn IOHIDManagerOpen(manager: IOHIDManagerRef, options: u32) -> i32;
    fn IOHIDManagerClose(manager: IOHIDManagerRef, options: u32) -> i32;
    fn IOHIDValueGetElement(value: IOHIDValueRef) -> IOHIDElementRef;
    fn IOHIDValueGetIntegerValue(value: IOHIDValueRef) -> isize;
    fn IOHIDElementGetUsagePage(element: IOHIDElementRef) -> u32;
    fn IOHIDElementGetUsage(element: IOHIDElementRef) -> u32;
}

struct CapsLockContext {
    emit: Box<dyn Fn(bool)>,
}

extern "C" fn caps_lock_input_value_callback(
    context: *mut c_void,
    result: i32,
    _sender: *mut c_void,
    value: IOHIDValueRef,
) {
    if result != KERN_SUCCESS || context.is_null() || value.is_null() {
        return;
    }
    let element = unsafe { IOHIDValueGetElement(value) };
    if element.is_null() {
        return;
    }
    let usage_page = unsafe { IOHIDElementGetUsagePage(element) };
    let usage = unsafe { IOHIDElementGetUsage(element) };
    if usage_page != KEYBOARD_USAGE_PAGE || usage != CAPS_LOCK_USAGE {
        return;
    }
    let pressed = unsafe { IOHIDValueGetIntegerValue(value) } != 0;
    let caps_context = unsafe { &*context.cast::<CapsLockContext>() };
    (caps_context.emit)(pressed);
}

fn usage_matching_dictionary(entries: &[(&'static str, i32)]) -> CFDictionary<CFString, CFNumber> {
    let pairs: Vec<(CFString, CFNumber)> = entries
        .iter()
        .map(|(key, value)| (CFString::from_static_string(key), CFNumber::from(*value)))
        .collect();
    CFDictionary::from_CFType_pairs(&pairs)
}

pub struct CapsLockHidListener {
    manager: IOHIDManagerRef,
    context: *mut CapsLockContext,
    run_loop: CFRunLoop,
}

impl CapsLockHidListener {
    pub fn start(emit: Box<dyn Fn(bool)>) -> Option<Self> {
        let manager = unsafe { IOHIDManagerCreate(kCFAllocatorDefault, HID_OPTIONS_NONE) };
        if manager.is_null() {
            return None;
        }
        let device_matching = usage_matching_dictionary(&[
            ("DeviceUsagePage", GENERIC_DESKTOP_USAGE_PAGE),
            ("DeviceUsage", KEYBOARD_DEVICE_USAGE),
        ]);
        #[allow(clippy::cast_possible_wrap)]
        let value_matching = usage_matching_dictionary(&[
            ("UsagePage", KEYBOARD_USAGE_PAGE as i32),
            ("Usage", CAPS_LOCK_USAGE as i32),
        ]);
        let context = Box::into_raw(Box::new(CapsLockContext { emit }));
        let run_loop = CFRunLoop::get_current();
        unsafe {
            IOHIDManagerSetDeviceMatching(manager, device_matching.as_concrete_TypeRef());
            IOHIDManagerSetInputValueMatching(manager, value_matching.as_concrete_TypeRef());
            IOHIDManagerRegisterInputValueCallback(
                manager,
                Some(caps_lock_input_value_callback),
                context.cast(),
            );
            IOHIDManagerScheduleWithRunLoop(
                manager,
                run_loop.as_concrete_TypeRef(),
                kCFRunLoopCommonModes,
            );
            let status = IOHIDManagerOpen(manager, HID_OPTIONS_NONE);
            if status != KERN_SUCCESS {
                IOHIDManagerUnscheduleFromRunLoop(
                    manager,
                    run_loop.as_concrete_TypeRef(),
                    kCFRunLoopCommonModes,
                );
                IOHIDManagerRegisterInputValueCallback(manager, None, ptr::null_mut());
                CFRelease(manager.cast());
                drop(Box::from_raw(context));
                return None;
            }
        }
        Some(Self {
            manager,
            context,
            run_loop,
        })
    }
}

impl Drop for CapsLockHidListener {
    fn drop(&mut self) {
        unsafe {
            IOHIDManagerUnscheduleFromRunLoop(
                self.manager,
                self.run_loop.as_concrete_TypeRef(),
                kCFRunLoopCommonModes,
            );
            IOHIDManagerRegisterInputValueCallback(self.manager, None, ptr::null_mut());
            let _ = IOHIDManagerClose(self.manager, HID_OPTIONS_NONE);
            CFRelease(self.manager.cast());
            drop(Box::from_raw(self.context));
        }
    }
}
