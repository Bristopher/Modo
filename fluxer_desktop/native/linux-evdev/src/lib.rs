#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

// SPDX-License-Identifier: AGPL-3.0-or-later

mod keymap;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::{EvdevHook, name_to_evdev_keycode};

#[cfg(not(target_os = "linux"))]
#[napi_derive::napi(js_name = "nameToEvdevKeycode")]
pub fn name_to_evdev_keycode(name: String) -> u32 {
    u32::from(crate::keymap::name_to_keycode(&name))
}
