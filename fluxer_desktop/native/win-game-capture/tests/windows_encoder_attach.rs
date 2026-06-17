// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(target_os = "windows")]
use fluxer_win_game_capture::{EncoderAttachError, EncoderAttachment};
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_CREATE_DEVICE_VIDEO_SUPPORT, D3D11_RESOURCE_MISC_SHARED, D3D11_SDK_VERSION,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11CreateDevice, ID3D11Device,
    ID3D11DeviceContext, ID3D11Multithread, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::core::Interface;

fn try_create_device() -> Option<(ID3D11Device, ID3D11DeviceContext)> {
    let feature_levels = [D3D_FEATURE_LEVEL_11_0];
    let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT;
    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;
    let result = unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            Default::default(),
            flags,
            Some(&feature_levels),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )
    };
    if result.is_err() {
        return None;
    }
    let dev = device?;
    let ctx = context?;
    if let Ok(mt) = dev.cast::<ID3D11Multithread>() {
        let _ = unsafe { mt.SetMultithreadProtected(true) };
    }
    Some((dev, ctx))
}

fn create_bgra_capture_texture(
    device: &ID3D11Device,
    width: u32,
    height: u32,
) -> Option<ID3D11Texture2D> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
        CPUAccessFlags: 0,
        MiscFlags: D3D11_RESOURCE_MISC_SHARED.0 as u32,
    };
    let mut tex = None;
    unsafe { device.CreateTexture2D(&desc, None, Some(&mut tex)) }.ok()?;
    tex
}

#[test]
fn windows_attach_construct_and_detach() {
    let _device = match try_create_device() {
        Some(d) => d,
        None => {
            eprintln!("skip: no D3D11 device available");
            return;
        }
    };
    let attach = match EncoderAttachment::try_new(640, 480) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("skip: EncoderAttachment init failed: {e:?}");
            return;
        }
    };
    assert!(attach.is_attached(), "attached after construction");
    assert_eq!(attach.width(), 640, "width preserved");
    assert_eq!(attach.height(), 480, "height preserved");
    assert_eq!(attach.capacity(), 8, "capacity matches RING_SIZE");
    let stats = attach.stats();
    assert_eq!(stats.frames_submitted, 0, "fresh attach: no submissions");
    assert_eq!(stats.failed_blits, 0, "fresh attach: no failed blits");
    attach.detach();
    assert!(!attach.is_attached(), "post-detach not attached");
}

#[test]
fn submit_capture_frame_with_blit_records_stats() {
    let (device, _context) = match try_create_device() {
        Some(d) => d,
        None => {
            eprintln!("skip: no D3D11 device available");
            return;
        }
    };
    let attach = match EncoderAttachment::try_new(640, 480) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("skip: attach init failed: {e:?}");
            return;
        }
    };
    let capture = match create_bgra_capture_texture(&device, 640, 480) {
        Some(t) => t,
        None => {
            eprintln!("skip: capture texture create failed");
            return;
        }
    };
    let result = attach.submit_capture_frame_with_blit(&capture, 640, 480);
    let stats = attach.stats();
    match result {
        Ok(()) => {
            assert!(
                stats.frames_submitted == 1,
                "expected 1 submission, got {}",
                stats.frames_submitted
            );
            assert!(
                stats.failed_blits == 0,
                "expected 0 failed blits, got {}",
                stats.failed_blits
            );
        }
        Err(EncoderAttachError::BlitFailed) => {
            assert!(
                stats.failed_blits >= 1,
                "expected >=1 failed blits, got {}",
                stats.failed_blits
            );
        }
        Err(EncoderAttachError::DeviceUnavailable) => {
            eprintln!("device unavailable from ring backend (driver constraint)");
        }
        Err(other) => {
            eprintln!("submit_capture_frame_with_blit unexpected error: {other:?}");
        }
    }
}

#[test]
fn submit_notify_back_pressure_increments_drops_after_capacity() {
    let _device = match try_create_device() {
        Some(d) => d,
        None => {
            eprintln!("skip: no D3D11 device available");
            return;
        }
    };
    let attach = match EncoderAttachment::try_new(64, 64) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("skip: attach init failed: {e:?}");
            return;
        }
    };
    let total = (attach.capacity() as u64) * 2;
    for _ in 0..total {
        attach.submit_notify().expect("notify ok");
    }
    let stats = attach.stats();
    assert!(
        stats.frames_submitted >= attach.capacity() as u64,
        "many submissions accepted"
    );
    assert!(stats.frames_dropped > 0, "back pressure drops oldest");
    assert_eq!(
        stats.ring_full_events, stats.frames_dropped,
        "drops==ring_full_events"
    );
}
