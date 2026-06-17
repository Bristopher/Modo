// SPDX-License-Identifier: AGPL-3.0-or-later

use criterion::{Criterion, criterion_group, criterion_main};
use fluxer_nv12_gpu_pack::{Nv12Packer, PackJob, try_acquire_device};

struct BenchFixture {
    device: wgpu::Device,
    queue: wgpu::Queue,
    view: wgpu::TextureView,
    y_buf: wgpu::Buffer,
    uv_buf: wgpu::Buffer,
    width: u32,
    height: u32,
}

fn make_fixture(width: u32, height: u32) -> Option<BenchFixture> {
    let (device, queue, _instance) = try_acquire_device()?;
    let pixels = vec![128u8; (width * height * 4) as usize];
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("nv12_gpu_pack.bench_source"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let y_size = Nv12Packer::y_plane_size(width, height);
    let uv_size = Nv12Packer::uv_plane_size(width, height);
    let y_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("nv12_gpu_pack.bench_y"),
        size: y_size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let uv_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("nv12_gpu_pack.bench_uv"),
        size: uv_size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    Some(BenchFixture {
        device,
        queue,
        view,
        y_buf,
        uv_buf,
        width,
        height,
    })
}

fn submit_pack_with_buffers(
    fixture: &BenchFixture,
    packer: &Nv12Packer,
    y_buf: &wgpu::Buffer,
    uv_buf: &wgpu::Buffer,
) {
    let mut encoder = fixture
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("nv12_gpu_pack.bench_encoder"),
        });
    let result = packer.pack(PackJob {
        device: &fixture.device,
        queue: &fixture.queue,
        encoder: &mut encoder,
        source: &fixture.view,
        y_out: y_buf,
        uv_out: uv_buf,
        dims: (fixture.width, fixture.height),
    });
    assert!(result.is_ok());
    fixture.queue.submit(std::iter::once(encoder.finish()));
    let _ = fixture.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
}

fn make_buffer_pair(fixture: &BenchFixture, label: &str) -> (wgpu::Buffer, wgpu::Buffer) {
    let y_size = Nv12Packer::y_plane_size(fixture.width, fixture.height);
    let uv_size = Nv12Packer::uv_plane_size(fixture.width, fixture.height);
    let y_buf = fixture.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: y_size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let uv_buf = fixture.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: uv_size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    (y_buf, uv_buf)
}

fn bench_pack_1080p_cache_hit(c: &mut Criterion) {
    let Some(fixture) = make_fixture(1920, 1080) else {
        eprintln!("nv12-gpu-pack bench: no wgpu adapter, skipping");
        return;
    };
    let packer = Nv12Packer::new(&fixture.device, fixture.width, fixture.height);
    submit_pack_with_buffers(&fixture, &packer, &fixture.y_buf, &fixture.uv_buf);
    c.bench_function("nv12_pack_1080p_cache_hit", |b| {
        b.iter(|| submit_pack_with_buffers(&fixture, &packer, &fixture.y_buf, &fixture.uv_buf));
    });
}

fn bench_pack_1080p_cache_miss(c: &mut Criterion) {
    let Some(fixture) = make_fixture(1920, 1080) else {
        eprintln!("nv12-gpu-pack bench: no wgpu adapter, skipping");
        return;
    };
    let packer = Nv12Packer::new(&fixture.device, fixture.width, fixture.height);
    let (y_b, uv_b) = make_buffer_pair(&fixture, "nv12_gpu_pack.bench_alt");
    c.bench_function("nv12_pack_1080p_cache_miss", |b| {
        b.iter(|| {
            submit_pack_with_buffers(&fixture, &packer, &fixture.y_buf, &fixture.uv_buf);
            submit_pack_with_buffers(&fixture, &packer, &y_b, &uv_b);
        });
    });
}

criterion_group!(
    benches,
    bench_pack_1080p_cache_hit,
    bench_pack_1080p_cache_miss
);
criterion_main!(benches);
