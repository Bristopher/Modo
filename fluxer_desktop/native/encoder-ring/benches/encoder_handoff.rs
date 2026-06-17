// SPDX-License-Identifier: AGPL-3.0-or-later

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fluxer_encoder_ring::{
    EncoderDims, EncoderSubmission, NotImplementedHandoff, NvencHandoff, RingError,
    apply_dts_offset, compute_dts_offset_us,
};

struct NoopCallback;
impl fluxer_encoder_ring::encoder_handoff::EncoderCompletionCallback for NoopCallback {
    fn on_complete(&mut self, _sequence: u64, _encoded_bytes: u32) {}
}

fn bench_stub_encode_round_trip(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoder_handoff");
    group.sample_size(60);
    group.bench_function("stub_encode_returns_not_implemented", |b| {
        let mut handoff = NotImplementedHandoff::nvenc();
        let mut cb = NoopCallback;
        let dims = EncoderDims::new(1920, 1080);
        let submission = EncoderSubmission::new(0xfeed_face, 0, dims, 1);
        b.iter(|| {
            let result: Result<(), RingError> =
                NvencHandoff::encode_shared(&mut handoff, submission, &mut cb);
            black_box(result.err());
        });
    });
    group.finish();
}

fn bench_dts_offset_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoder_handoff");
    group.sample_size(120);
    group.bench_function("dts_offset_compute_apply", |b| {
        b.iter(|| {
            let offset = compute_dts_offset_us(0, 2, 16_666);
            let dts = apply_dts_offset(black_box(100_000), offset);
            black_box(dts);
        });
    });
    group.finish();
}

#[cfg(target_os = "windows")]
fn bench_nvenc_single_frame_round_trip(c: &mut Criterion) {
    use fluxer_encoder_ring::{NvencD3D11Handoff, PicParams};
    use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0};
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        D3D11_RESOURCE_MISC_SHARED_KEYEDMUTEX, D3D11_RESOURCE_MISC_SHARED_NTHANDLE,
        D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11CreateDevice,
        ID3D11Device,
    };
    use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_NV12, DXGI_SAMPLE_DESC};
    use windows::Win32::Graphics::Dxgi::IDXGIResource1;
    use windows::core::Interface;
    let mut device: Option<ID3D11Device> = None;
    let feature_levels = [D3D_FEATURE_LEVEL_11_0];
    let _ = unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            Default::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&feature_levels),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            None,
        )
    };
    let device = match device {
        Some(d) => d,
        None => return,
    };
    let dims = EncoderDims::new(1920, 1080);
    let handoff_result = NvencD3D11Handoff::new(device.clone(), dims, 5_000_000);
    let mut handoff = match handoff_result {
        Ok(h) => h,
        Err(_) => return,
    };
    let desc = D3D11_TEXTURE2D_DESC {
        Width: 1920,
        Height: 1080,
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
    let _ = unsafe { device.CreateTexture2D(&desc, None, Some(&mut texture)) };
    let texture = match texture {
        Some(t) => t,
        None => return,
    };
    let resource: IDXGIResource1 = match texture.cast() {
        Ok(r) => r,
        Err(_) => return,
    };
    let shared =
        match unsafe { resource.CreateSharedHandle(None, 0x3, windows::core::PCWSTR::null()) } {
            Ok(h) => h.0 as u64,
            Err(_) => return,
        };
    let slot = match handoff.register_slot(shared, 0, dims) {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut counter: u64 = 0;
    let mut group = c.benchmark_group("encoder_handoff");
    group.sample_size(20);
    group.bench_function("nvenc_1080p_nv12_single_frame", |b| {
        b.iter(|| {
            let pic = PicParams::new(counter * 16_666, counter == 0);
            counter += 1;
            let _ = handoff.encode_shared_async(slot, 0, dims, pic);
            let _ = handoff.poll_completed(slot);
        });
    });
    group.finish();
    handoff.unregister_slot(slot);
}

#[cfg(not(target_os = "windows"))]
fn bench_nvenc_single_frame_round_trip(_c: &mut Criterion) {}

criterion_group!(
    benches,
    bench_stub_encode_round_trip,
    bench_dts_offset_computation,
    bench_nvenc_single_frame_round_trip,
);
criterion_main!(benches);
