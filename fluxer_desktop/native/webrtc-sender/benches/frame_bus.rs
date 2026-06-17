// SPDX-License-Identifier: AGPL-3.0-or-later

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fluxer_screen_frame_bus::{
    BgraFrame, EnqueueOutcome, NATIVE_SCREEN_FRAME_SINK_ACCEPTED,
    NATIVE_SCREEN_FRAME_SINK_HANDLE_MAGIC, NATIVE_SCREEN_FRAME_SINK_HANDLE_VERSION,
    NativeScreenFrameSinkHandle, Nv12Frame, ScreenFrame, ScreenFrameSink, get_sink, register_sink,
    unregister_sink,
};
use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

struct DiscardSink(AtomicU64);
impl ScreenFrameSink for DiscardSink {
    fn enqueue(&self, _frame: ScreenFrame) -> EnqueueOutcome {
        self.0.fetch_add(1, Ordering::Relaxed);
        EnqueueOutcome::Accepted
    }
}

struct NativeDiscardSink(AtomicU64);

unsafe extern "C" fn retain_native_discard_sink(context: *const c_void) {
    unsafe { Arc::increment_strong_count(context.cast::<NativeDiscardSink>()) };
}

unsafe extern "C" fn release_native_discard_sink(context: *const c_void) {
    unsafe { drop(Arc::from_raw(context.cast::<NativeDiscardSink>())) };
}

unsafe extern "C" fn enqueue_native_discard_nv12(
    context: *const c_void,
    data: *const u8,
    data_len: usize,
    _width: u32,
    _height: u32,
    _stride_y: u32,
    _stride_uv: u32,
    _timestamp_us: i64,
) -> u32 {
    if !context.is_null() && !data.is_null() && data_len > 0 {
        unsafe {
            (*context.cast::<NativeDiscardSink>())
                .0
                .fetch_add(1, Ordering::Relaxed)
        };
    }
    NATIVE_SCREEN_FRAME_SINK_ACCEPTED
}

unsafe extern "C" fn enqueue_native_discard_bgra(
    context: *const c_void,
    data: *const u8,
    data_len: usize,
    _width: u32,
    _height: u32,
    _stride: u32,
    _timestamp_us: i64,
) -> u32 {
    if !context.is_null() && !data.is_null() && data_len > 0 {
        unsafe {
            (*context.cast::<NativeDiscardSink>())
                .0
                .fetch_add(1, Ordering::Relaxed)
        };
    }
    NATIVE_SCREEN_FRAME_SINK_ACCEPTED
}

fn native_discard_handle() -> (
    Arc<NativeDiscardSink>,
    fluxer_screen_frame_bus::NativeScreenFrameSinkHandleRef,
) {
    let sink = Arc::new(NativeDiscardSink(AtomicU64::new(0)));
    let raw_context = Arc::into_raw(sink.clone()).cast::<c_void>();
    let handle = NativeScreenFrameSinkHandle {
        magic: NATIVE_SCREEN_FRAME_SINK_HANDLE_MAGIC,
        version: NATIVE_SCREEN_FRAME_SINK_HANDLE_VERSION,
        context: raw_context,
        retain: retain_native_discard_sink,
        release: release_native_discard_sink,
        enqueue_nv12: Some(enqueue_native_discard_nv12),
        enqueue_bgra: Some(enqueue_native_discard_bgra),
        enqueue_mac_cv_pixel_buffer: None,
        enqueue_dmabuf: None,
        enqueue_shared_texture: None,
    };
    let retained = unsafe { handle.retain_ref().expect("valid native sink handle") };
    unsafe { release_native_discard_sink(raw_context) };
    (sink, retained)
}

fn bench_registry_lookup(c: &mut Criterion) {
    let id = format!("bench:registry:{}", std::process::id());
    register_sink(
        id.clone(),
        Arc::new(DiscardSink(AtomicU64::new(0))) as Arc<dyn ScreenFrameSink>,
    );

    c.bench_function("frame_bus::get_sink", |b| {
        b.iter(|| {
            let s = get_sink(black_box(id.as_str()));
            black_box(s);
        })
    });

    unregister_sink(&id);
}

fn synth_nv12(width: u32, height: u32) -> ScreenFrame {
    let total = (width * height) as usize + (width * (height / 2)) as usize;
    let data = vec![0x80u8; total];
    ScreenFrame::Nv12(Nv12Frame {
        data: data.into(),
        width,
        height,
        stride_y: width,
        stride_uv: width,
        timestamp_us: 1,
    })
}

fn synth_bgra(width: u32, height: u32) -> ScreenFrame {
    let total = (width * height * 4) as usize;
    let data = vec![0xff; total];
    ScreenFrame::Bgra(BgraFrame {
        data,
        width,
        height,
        stride: width * 4,
        timestamp_us: 1,
    })
}

fn bench_discard_sink_enqueue(c: &mut Criterion) {
    let id = format!("bench:discard:{}", std::process::id());
    let sink = Arc::new(DiscardSink(AtomicU64::new(0)));
    register_sink(id.clone(), sink.clone() as Arc<dyn ScreenFrameSink>);

    let mut group = c.benchmark_group("frame_bus::enqueue_discard");
    for (label, w, h) in [
        ("nv12_320x180", 320u32, 180u32),
        ("nv12_1920x1080", 1920, 1080),
        ("nv12_3840x2160", 3840, 2160),
        ("bgra_320x180", 320, 180),
        ("bgra_1920x1080", 1920, 1080),
    ] {
        let bytes = if label.starts_with("nv12") {
            ((w * h) + (w * (h / 2))) as u64
        } else {
            (w * h * 4) as u64
        };
        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::from_parameter(label), &(), |b, _| {
            b.iter(|| {
                let frame = if label.starts_with("nv12") {
                    synth_nv12(w, h)
                } else {
                    synth_bgra(w, h)
                };
                let s = get_sink(id.as_str()).unwrap();
                let outcome = s.enqueue(black_box(frame));
                black_box(outcome);
            })
        });
    }
    group.finish();
    unregister_sink(&id);
}

fn bench_native_handle_enqueue(c: &mut Criterion) {
    let (_sink, handle) = native_discard_handle();
    let mut group = c.benchmark_group("frame_bus::native_handle_enqueue");
    for (label, w, h) in [
        ("nv12_320x180", 320u32, 180u32),
        ("nv12_1920x1080", 1920, 1080),
        ("nv12_3840x2160", 3840, 2160),
        ("bgra_320x180", 320, 180),
        ("bgra_1920x1080", 1920, 1080),
    ] {
        let bytes = if label.starts_with("nv12") {
            ((w * h) + (w * (h / 2))) as usize
        } else {
            (w * h * 4) as usize
        };
        let data = vec![0x80u8; bytes];
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), &(), |b, _| {
            b.iter(|| {
                let outcome = if label.starts_with("nv12") {
                    handle.enqueue_nv12_copy(black_box(&data), w, h, w, w, 1)
                } else {
                    handle.enqueue_bgra_copy(black_box(&data), w, h, w * 4, 1)
                };
                black_box(outcome);
            })
        });
    }
    group.finish();
}

fn bench_registry_lookup_miss(c: &mut Criterion) {
    c.bench_function("frame_bus::get_sink_miss", |b| {
        b.iter(|| {
            let s = get_sink(black_box("nonexistent"));
            black_box(s);
        })
    });
}

criterion_group!(
    benches,
    bench_registry_lookup,
    bench_registry_lookup_miss,
    bench_native_handle_enqueue,
    bench_discard_sink_enqueue
);
criterion_main!(benches);
