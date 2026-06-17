// SPDX-License-Identifier: AGPL-3.0-or-later

use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;

fn unsupported() -> Error {
    Error::new(
        Status::GenericFailure,
        "@fluxer/linux-screen-capture is only supported on Linux",
    )
}

#[napi(object, js_name = "LinuxScreenCaptureSource")]
pub struct LinuxScreenCaptureSource {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub app_name: Option<String>,
    pub bundle_id: Option<String>,
    pub target_pid: Option<u32>,
}

#[napi(js_name = "listSources")]
pub fn list_sources() -> Result<Vec<LinuxScreenCaptureSource>> {
    Ok(Vec::new())
}

#[napi(object, js_name = "LinuxScreenCaptureCapabilities")]
pub struct Capabilities {
    pub process: bool,
    pub system: bool,
}

#[napi(object, js_name = "LinuxScreenCaptureAvailability")]
pub struct Availability {
    pub available: bool,
    pub backend: String,
    pub reason: Option<String>,
    pub detail: Option<String>,
    pub portal_version: Option<u32>,
    pub capabilities: Capabilities,
}

#[napi(js_name = "getAvailability")]
pub fn get_availability() -> Result<Availability> {
    Ok(Availability {
        available: false,
        backend: "linux-pipewire-portal".to_string(),
        reason: Some("unsupported-platform".to_string()),
        detail: None,
        portal_version: None,
        capabilities: Capabilities {
            process: false,
            system: false,
        },
    })
}

#[napi(object, js_name = "LinuxScreenCaptureBackendInfo")]
pub struct BackendInfo {
    pub backend: String,
    pub supported: bool,
    pub reason: String,
    pub portal_version: Option<u32>,
    pub pipewire_reachable: bool,
}

#[napi(js_name = "getBackendInfo")]
pub fn get_backend_info() -> BackendInfo {
    BackendInfo {
        backend: "linux-pipewire-portal".to_string(),
        supported: false,
        reason: "@fluxer/linux-screen-capture is only supported on Linux".to_string(),
        portal_version: None,
        pipewire_reachable: false,
    }
}

#[napi]
pub struct ScreenCapture;

#[napi]
impl ScreenCapture {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Err(unsupported())
    }
}
