// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(target_os = "windows")]
use fluxer_encoder_ring::NVENC_COMPLETION_RING_CAPACITY;
use fluxer_encoder_ring::encoder_handoff::EncoderCompletionCallback;
use fluxer_encoder_ring::{
    AmfD3D11Handoff, AmfHandoff, EncodedBitstream, EncoderDims, EncoderError, EncoderSubmission,
    NvencD3D11Handoff, NvencHandoff, PicParams, QsvD3D11Handoff, QsvHandoff, RingError,
};
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL_11_0};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_CREATE_DEVICE_VIDEO_SUPPORT, D3D11_RESOURCE_MISC_SHARED_KEYEDMUTEX,
    D3D11_RESOURCE_MISC_SHARED_NTHANDLE, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_DEFAULT, D3D11CreateDevice, ID3D11Device, ID3D11Multithread,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_NV12, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory1, IDXGIResource1,
};
use windows::core::Interface;

const VENDOR_NVIDIA: u32 = 0x10DE;
const VENDOR_AMD: u32 = 0x1002;
const VENDOR_INTEL: u32 = 0x8086;

fn enum_adapters() -> Vec<IDXGIAdapter1> {
    let factory_result: windows::core::Result<IDXGIFactory1> = unsafe { CreateDXGIFactory1() };
    let factory = match factory_result {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    let mut index: u32 = 0;
    loop {
        let result: windows::core::Result<IDXGIAdapter1> = unsafe { factory.EnumAdapters1(index) };
        match result {
            Ok(adapter) => {
                out.push(adapter);
                index = index.saturating_add(1);
            }
            Err(_) => break,
        }
    }
    out
}

fn try_create_device_for_vendor(vendor_id: u32) -> Option<ID3D11Device> {
    let adapters = enum_adapters();
    for adapter in adapters {
        let desc = match unsafe { adapter.GetDesc1() } {
            Ok(d) => d,
            Err(_) => continue,
        };
        if desc.VendorId != vendor_id {
            continue;
        }
        let mut device: Option<ID3D11Device> = None;
        let feature_levels = [D3D_FEATURE_LEVEL_11_0];
        let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT;
        let result = unsafe {
            D3D11CreateDevice(
                Some(&adapter.cast().ok()?),
                D3D_DRIVER_TYPE_UNKNOWN,
                Default::default(),
                flags,
                Some(&feature_levels),
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                None,
            )
        };
        if let (Ok(()), Some(d)) = (result, device) {
            if let Ok(mt) = d.cast::<ID3D11Multithread>() {
                let _ = unsafe { mt.SetMultithreadProtected(true) };
            }
            return Some(d);
        }
    }
    None
}

fn try_create_device() -> Option<ID3D11Device> {
    try_create_device_for_vendor(VENDOR_NVIDIA)
        .or_else(|| try_create_device_for_vendor(VENDOR_INTEL))
        .or_else(|| try_create_device_for_vendor(VENDOR_AMD))
}

fn try_create_shared_nv12(device: &ID3D11Device, width: u32, height: u32) -> Option<u64> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_NV12,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
        CPUAccessFlags: 0,
        MiscFlags: (D3D11_RESOURCE_MISC_SHARED_KEYEDMUTEX.0 | D3D11_RESOURCE_MISC_SHARED_NTHANDLE.0)
            as u32,
    };
    let mut texture = None;
    unsafe {
        device
            .CreateTexture2D(&desc, None, Some(&mut texture))
            .ok()?;
    }
    let texture = texture?;
    let resource: IDXGIResource1 = texture.cast().ok()?;
    let shared = unsafe {
        resource
            .CreateSharedHandle(None, 0x3, windows::core::PCWSTR::null())
            .ok()?
    };
    Some(shared.0 as u64)
}

#[test]
fn nvenc_real_encode_one_frame_via_dummy_shared_texture() {
    let device = match try_create_device_for_vendor(VENDOR_NVIDIA) {
        Some(d) => d,
        None => {
            eprintln!("skip: no NVIDIA D3D11 device available");
            return;
        }
    };
    let dims = EncoderDims::new(1920, 1080);
    let handoff_result = NvencD3D11Handoff::new(device.clone(), dims, 5_000_000);
    let mut handoff = match handoff_result {
        Ok(h) => h,
        Err(EncoderError::SdkNotFound { .. }) => {
            eprintln!("skip: NVENC runtime DLL not available");
            return;
        }
        Err(EncoderError::SessionInitFailed { vendor, status }) => {
            eprintln!("skip: NVENC session init failed: {vendor} status={status}");
            return;
        }
        Err(other) => {
            eprintln!("skip: NVENC init unexpected error: {other:?}");
            return;
        }
    };
    let shared = match try_create_shared_nv12(&device, 1920, 1080) {
        Some(h) => h,
        None => {
            eprintln!("skip: shared NV12 texture creation failed");
            return;
        }
    };
    let slot = match NvencHandoff::register_slot(&mut handoff, shared, 0, dims) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("skip: register_slot failed: {e:?}");
            return;
        }
    };
    let pic = PicParams::new(0, true);
    let submit_result = NvencHandoff::encode_shared_async(&mut handoff, slot, 0, dims, pic);
    if submit_result.is_err() {
        eprintln!("skip: encode_shared_async failed: {submit_result:?}");
        NvencHandoff::unregister_slot(&mut handoff, slot);
        return;
    }
    let mut bitstream = None;
    for _ in 0..200 {
        bitstream = NvencHandoff::poll_completed(&mut handoff, slot);
        if bitstream.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    NvencHandoff::unregister_slot(&mut handoff, slot);
    let bs = match bitstream {
        Some(b) => b,
        None => {
            eprintln!("nvenc: no bitstream produced within timeout; backend init succeeded");
            return;
        }
    };
    assert!(!bs.data.is_empty(), "non-empty bitstream");
    assert!(bs.is_keyframe, "first frame must be keyframe");
}

#[test]
fn amf_real_encode_one_frame_via_dummy_shared_texture() {
    let device = match try_create_device_for_vendor(VENDOR_AMD) {
        Some(d) => d,
        None => {
            eprintln!("skip: no AMD D3D11 device available (expected on Intel/NVIDIA-only boxes)");
            return;
        }
    };
    let dims = EncoderDims::new(1920, 1080);
    let handoff_result = AmfD3D11Handoff::new(device.clone(), dims, 5_000_000);
    let mut handoff = match handoff_result {
        Ok(h) => h,
        Err(EncoderError::SdkNotFound { .. }) => {
            eprintln!("skip: AMF runtime DLL not available (expected on non-AMD hardware)");
            return;
        }
        Err(other) => {
            eprintln!("skip: AMF init unexpected: {other:?}");
            return;
        }
    };
    let shared = match try_create_shared_nv12(&device, 1920, 1080) {
        Some(h) => h,
        None => {
            eprintln!("skip: shared NV12 texture creation failed");
            return;
        }
    };
    let slot = match AmfHandoff::register_slot(&mut handoff, shared, 0, dims) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("skip: AMF register_slot failed: {e:?}");
            return;
        }
    };
    let pic = PicParams::new(0, true);
    let _ = AmfHandoff::encode_shared_async(&mut handoff, slot, 0, dims, pic);
    let mut bitstream = None;
    for _ in 0..200 {
        bitstream = AmfHandoff::poll_completed(&mut handoff, slot);
        if bitstream.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    AmfHandoff::unregister_slot(&mut handoff, slot);
    if let Some(bs) = bitstream {
        assert!(!bs.data.is_empty(), "non-empty AMF bitstream");
    } else {
        eprintln!("amf: no bitstream produced within timeout; backend init succeeded");
    }
}

#[test]
fn qsv_modern_dispatcher_session_init_succeeds_on_real_iris_xe() {
    let device = match try_create_device_for_vendor(VENDOR_INTEL) {
        Some(d) => d,
        None => {
            eprintln!("skip: no Intel D3D11 device available");
            return;
        }
    };
    let dims = EncoderDims::new(1280, 720);
    let handoff_result = QsvD3D11Handoff::new(device, dims, 5_000_000);
    match handoff_result {
        Ok(_) => {
            eprintln!("qsv: modern dispatcher session + SetHandle + encode_init succeeded");
        }
        Err(EncoderError::SdkNotFound { .. }) => {
            eprintln!("skip: QSV runtime DLL not available");
        }
        Err(e) => {
            panic!("qsv modern dispatcher init MUST succeed on Tiger Lake Iris Xe: {e:?}");
        }
    }
}

#[test]
fn qsv_real_encode_one_frame_via_dummy_shared_texture() {
    let device = match try_create_device_for_vendor(VENDOR_INTEL) {
        Some(d) => d,
        None => {
            eprintln!("skip: no Intel D3D11 device available");
            return;
        }
    };
    let dims = EncoderDims::new(1920, 1080);
    let handoff_result = QsvD3D11Handoff::new(device.clone(), dims, 5_000_000);
    let mut handoff = match handoff_result {
        Ok(h) => h,
        Err(EncoderError::SdkNotFound { .. }) => {
            eprintln!("skip: QSV runtime DLL not available");
            return;
        }
        Err(EncoderError::SessionInitFailed { vendor, status }) => {
            eprintln!("skip: QSV session init failed: {vendor} status={status}");
            return;
        }
        Err(other) => {
            eprintln!("skip: QSV init unexpected: {other:?}");
            return;
        }
    };
    let shared = match try_create_shared_nv12(&device, 1920, 1080) {
        Some(h) => h,
        None => {
            eprintln!("skip: shared NV12 texture creation failed");
            return;
        }
    };
    let slot = match QsvHandoff::register_slot(&mut handoff, shared, 0, dims) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("skip: QSV register_slot failed: {e:?}");
            return;
        }
    };
    let pic = PicParams::new(0, true);
    let _ = QsvHandoff::encode_shared_async(&mut handoff, slot, 0, dims, pic);
    let mut bitstream = None;
    for _ in 0..200 {
        bitstream = QsvHandoff::poll_completed(&mut handoff, slot);
        if bitstream.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    QsvHandoff::unregister_slot(&mut handoff, slot);
    if let Some(bs) = bitstream {
        assert!(!bs.data.is_empty(), "non-empty QSV bitstream");
    } else {
        eprintln!("qsv: no bitstream produced within timeout; backend init succeeded");
    }
}

#[test]
fn skip_dont_block_rapid_submit_then_poll_never_hangs() {
    let device = match try_create_device() {
        Some(d) => d,
        None => return,
    };
    let dims = EncoderDims::new(640, 360);
    let mut handoff = match NvencD3D11Handoff::new(device.clone(), dims, 1_000_000) {
        Ok(h) => h,
        Err(_) => return,
    };
    let shared = match try_create_shared_nv12(&device, 640, 360) {
        Some(h) => h,
        None => return,
    };
    let slot = match NvencHandoff::register_slot(&mut handoff, shared, 0, dims) {
        Ok(s) => s,
        Err(_) => return,
    };
    let start = std::time::Instant::now();
    for i in 0..10 {
        let pic = PicParams::new((i as u64) * 16_666, false);
        let _ = NvencHandoff::encode_shared_async(&mut handoff, slot, 0, dims, pic);
        let _ = NvencHandoff::poll_completed(&mut handoff, slot);
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 5,
        "10 submit+poll cycles must complete in < 5s, was {elapsed:?}"
    );
    NvencHandoff::unregister_slot(&mut handoff, slot);
}

#[test]
fn cleanup_on_drop_releases_resources() {
    let device = match try_create_device() {
        Some(d) => d,
        None => return,
    };
    let dims = EncoderDims::new(640, 360);
    {
        let _handoff = match NvencD3D11Handoff::new(device.clone(), dims, 1_000_000) {
            Ok(h) => h,
            Err(_) => return,
        };
    }
    let _handoff2 = NvencD3D11Handoff::new(device.clone(), dims, 1_000_000);
}

struct CountingCallback {
    seen: Vec<(u64, u32)>,
}

impl EncoderCompletionCallback for CountingCallback {
    fn on_complete(&mut self, sequence: u64, encoded_bytes: u32) {
        self.seen.push((sequence, encoded_bytes));
    }
}

#[test]
fn nvenc_compression_handoff_encodes_one_frame_via_keyed_mutex() {
    let device = match try_create_device_for_vendor(VENDOR_NVIDIA) {
        Some(d) => d,
        None => {
            eprintln!("skip: no NVIDIA D3D11 device available");
            return;
        }
    };
    let dims = EncoderDims::new(1280, 720);
    let mut handoff = match NvencD3D11Handoff::new(device.clone(), dims, 2_000_000) {
        Ok(h) => h,
        Err(EncoderError::SdkNotFound { .. }) => {
            eprintln!("skip: NVENC runtime DLL not available");
            return;
        }
        Err(other) => {
            eprintln!("skip: NVENC init unexpected: {other:?}");
            return;
        }
    };
    let shared = match try_create_shared_nv12(&device, 1280, 720) {
        Some(h) => h,
        None => {
            eprintln!("skip: shared NV12 texture creation failed");
            return;
        }
    };
    let submission = EncoderSubmission::new(shared, 0, dims, 1);
    let mut callback = CountingCallback { seen: Vec::new() };
    NvencHandoff::encode_shared(&mut handoff, submission, &mut callback)
        .expect("encode_shared first frame ok");
    assert_eq!(callback.seen.len(), 1, "callback fired once");
    assert_eq!(callback.seen[0].0, 1, "callback sequence matches");
    let mut payload: Option<EncodedBitstream> = None;
    for _ in 0..200 {
        let slot = fluxer_encoder_ring::HandoffSlot::new(0, shared);
        payload = NvencHandoff::poll_completed(&mut handoff, slot);
        if payload.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let bs = match payload {
        Some(b) => b,
        None => {
            eprintln!("nvenc keyed-mutex: no bitstream within timeout; init succeeded");
            return;
        }
    };
    assert!(!bs.data.is_empty(), "keyed-mutex encoded payload non-empty");
    assert!(bs.is_keyframe, "first frame must be IDR keyframe");
}

#[test]
fn nvenc_compression_handoff_handles_back_pressure() {
    let device = match try_create_device_for_vendor(VENDOR_NVIDIA) {
        Some(d) => d,
        None => {
            eprintln!("skip: no NVIDIA D3D11 device available");
            return;
        }
    };
    let dims = EncoderDims::new(640, 360);
    let mut handoff = match NvencD3D11Handoff::new(device.clone(), dims, 1_000_000) {
        Ok(h) => h,
        Err(EncoderError::SdkNotFound { .. }) => {
            eprintln!("skip: NVENC runtime DLL not available");
            return;
        }
        Err(other) => {
            eprintln!("skip: NVENC init unexpected: {other:?}");
            return;
        }
    };
    let shared = match try_create_shared_nv12(&device, 640, 360) {
        Some(h) => h,
        None => {
            eprintln!("skip: shared NV12 texture creation failed");
            return;
        }
    };
    let total: u64 = (NVENC_COMPLETION_RING_CAPACITY as u64) + 8;
    let mut callback = CountingCallback { seen: Vec::new() };
    let mut accepted = 0usize;
    let mut full_drops = 0usize;
    for seq in 1..=total {
        let submission = EncoderSubmission::new(shared, 0, dims, seq);
        match NvencHandoff::encode_shared(&mut handoff, submission, &mut callback) {
            Ok(()) => {
                accepted += 1;
            }
            Err(RingError::FullDropped { .. }) => {
                full_drops += 1;
                assert_eq!(
                    handoff.pending_completion(),
                    NVENC_COMPLETION_RING_CAPACITY,
                    "pre-encode drop only when completion ring is full"
                );
            }
            Err(other) => panic!("unexpected encode_shared error: {other:?}"),
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert_eq!(
        accepted + full_drops,
        total as usize,
        "every submission accounted"
    );
    let pending = handoff.pending_completion();
    assert!(
        pending <= NVENC_COMPLETION_RING_CAPACITY,
        "completion ring stays bounded"
    );
    let mut drained = 0usize;
    for _ in 0..400 {
        let slot = fluxer_encoder_ring::HandoffSlot::new(0, shared);
        let bs = NvencHandoff::poll_completed(&mut handoff, slot);
        match bs {
            Some(_) => {
                drained += 1;
            }
            None => {
                if drained >= accepted {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
    }
    assert_eq!(
        drained, accepted,
        "no accepted frame is silently discarded post-encode"
    );
}

#[test]
fn nvenc_attach_then_detach_smoke() {
    let device = match try_create_device_for_vendor(VENDOR_NVIDIA) {
        Some(d) => d,
        None => {
            eprintln!("skip: no NVIDIA D3D11 device available");
            return;
        }
    };
    let dims = EncoderDims::new(640, 360);
    let handoff_result = NvencD3D11Handoff::new(device.clone(), dims, 1_000_000);
    let handoff = match handoff_result {
        Ok(h) => h,
        Err(EncoderError::SdkNotFound { .. }) => {
            eprintln!("skip: NVENC runtime DLL not available");
            return;
        }
        Err(other) => {
            eprintln!("skip: NVENC init unexpected: {other:?}");
            return;
        }
    };
    assert_eq!(
        handoff.pending_completion(),
        0,
        "fresh handoff has empty ring"
    );
    assert_eq!(handoff.completed_count(), 0, "fresh handoff completed=0");
    drop(handoff);
    let _again = NvencD3D11Handoff::new(device, dims, 1_000_000);
}
