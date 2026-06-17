// SPDX-License-Identifier: AGPL-3.0-or-later

use bytemuck::{Pod, Zeroable};
use fluxer_gpu_rebuild::{GpuLossCallback, GpuRebuildError};
use parking_lot::Mutex;
use std::num::NonZeroU64;
use wgpu::util::DeviceExt;

const WORKGROUP_DIM: u32 = 8;
const Y_BYTES_PER_PIXEL: u64 = 1;
const UV_BYTES_PER_PIXEL_PAIR: u64 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nv12PackError {
    DimensionsExceedMax {
        requested: (u32, u32),
        max: (u32, u32),
    },
    DimensionsNotEven {
        requested: (u32, u32),
    },
    DimensionsZero,
    YBufferTooSmall {
        required: u64,
        actual: u64,
    },
    UvBufferTooSmall {
        required: u64,
        actual: u64,
    },
    StrideNotAligned {
        stride: u32,
    },
    NotReady,
}

impl std::fmt::Display for Nv12PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DimensionsExceedMax { requested, max } => {
                write!(f, "dimensions {requested:?} exceed packer maximum {max:?}")
            }
            Self::DimensionsNotEven { requested } => {
                write!(f, "dimensions {requested:?} must be even for NV12")
            }
            Self::DimensionsZero => write!(f, "dimensions must be non-zero"),
            Self::YBufferTooSmall { required, actual } => write!(
                f,
                "Y buffer too small: required {required} bytes, actual {actual}"
            ),
            Self::UvBufferTooSmall { required, actual } => write!(
                f,
                "UV buffer too small: required {required} bytes, actual {actual}"
            ),
            Self::StrideNotAligned { stride } => {
                write!(
                    f,
                    "stride {stride} must be a multiple of 4 for word-packed writes"
                )
            }
            Self::NotReady => write!(f, "packer pipeline is released; rebuild required"),
        }
    }
}

impl std::error::Error for Nv12PackError {}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PackUniforms {
    width: u32,
    height: u32,
    stride_y: u32,
    stride_uv: u32,
}

struct PackResources {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct CachedBindGroupKey {
    source_view: wgpu::TextureView,
    y_buf: wgpu::Buffer,
    uv_buf: wgpu::Buffer,
}

struct CachedBindGroup {
    key: CachedBindGroupKey,
    bind_group: wgpu::BindGroup,
}

pub struct Nv12Packer {
    resources: Option<PackResources>,
    bind_group_cache: Mutex<Option<CachedBindGroup>>,
    max_width: u32,
    max_height: u32,
}

pub struct PackJob<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub source: &'a wgpu::TextureView,
    pub y_out: &'a wgpu::Buffer,
    pub uv_out: &'a wgpu::Buffer,
    pub dims: (u32, u32),
}

impl Nv12Packer {
    pub fn new(device: &wgpu::Device, max_width: u32, max_height: u32) -> Self {
        assert!(max_width > 0, "max_width must be positive");
        assert!(max_height > 0, "max_height must be positive");
        assert!(
            max_width.is_multiple_of(2),
            "max_width must be even for NV12"
        );
        assert!(
            max_height.is_multiple_of(2),
            "max_height must be even for NV12"
        );
        let resources = build_resources(device);
        Self {
            resources: Some(resources),
            bind_group_cache: Mutex::new(None),
            max_width,
            max_height,
        }
    }

    pub fn new_unbuilt(max_width: u32, max_height: u32) -> Self {
        assert!(max_width > 0, "max_width must be positive");
        assert!(max_height > 0, "max_height must be positive");
        assert!(
            max_width.is_multiple_of(2),
            "max_width must be even for NV12"
        );
        assert!(
            max_height.is_multiple_of(2),
            "max_height must be even for NV12"
        );
        Self {
            resources: None,
            bind_group_cache: Mutex::new(None),
            max_width,
            max_height,
        }
    }

    pub fn max_width(&self) -> u32 {
        assert!(self.max_width > 0, "max_width invariant");
        assert!(self.max_width.is_multiple_of(2), "max_width even invariant");
        self.max_width
    }

    pub fn max_height(&self) -> u32 {
        assert!(self.max_height > 0, "max_height invariant");
        assert!(
            self.max_height.is_multiple_of(2),
            "max_height even invariant"
        );
        self.max_height
    }

    pub fn is_built(&self) -> bool {
        let built = self.resources.is_some();
        assert!(
            self.max_width > 0,
            "max_width must be positive while introspecting state"
        );
        assert!(
            self.max_height > 0,
            "max_height must be positive while introspecting state"
        );
        built
    }

    pub fn y_plane_size(width: u32, height: u32) -> u64 {
        assert!(width > 0, "y_plane_size width must be positive");
        assert!(height > 0, "y_plane_size height must be positive");
        u64::from(width) * u64::from(height) * Y_BYTES_PER_PIXEL
    }

    pub fn uv_plane_size(width: u32, height: u32) -> u64 {
        assert!(width > 0, "uv_plane_size width must be positive");
        assert!(height > 0, "uv_plane_size height must be positive");
        assert!(width.is_multiple_of(2), "uv plane requires even width");
        assert!(height.is_multiple_of(2), "uv plane requires even height");
        u64::from(width) * u64::from(height) * UV_BYTES_PER_PIXEL_PAIR / 4
    }

    pub fn pack(&self, mut job: PackJob<'_>) -> Result<(), Nv12PackError> {
        let (width, height) = job.dims;
        let resources = match self.resources.as_ref() {
            Some(r) => r,
            None => return Err(Nv12PackError::NotReady),
        };
        self.validate(width, height, job.y_out, job.uv_out)?;
        assert!(width <= self.max_width, "validated width must respect max");
        assert!(
            height <= self.max_height,
            "validated height must respect max"
        );
        let bind_group = self.acquire_bind_group(resources, &job);
        record_pack_pass(resources, &bind_group, &mut job);
        Ok(())
    }

    fn acquire_bind_group(&self, resources: &PackResources, job: &PackJob<'_>) -> wgpu::BindGroup {
        let key = CachedBindGroupKey {
            source_view: job.source.clone(),
            y_buf: job.y_out.clone(),
            uv_buf: job.uv_out.clone(),
        };
        let mut cache = self.bind_group_cache.lock();
        if let Some(cached) = cache.as_ref()
            && cached.key == key
        {
            return cached.bind_group.clone();
        }
        let bind_group = build_bind_group(resources, job);
        *cache = Some(CachedBindGroup {
            key,
            bind_group: bind_group.clone(),
        });
        assert!(
            cache.is_some(),
            "bind group cache must be populated after rebuild"
        );
        bind_group
    }

    pub fn cached_bind_group_count(&self) -> usize {
        let cache = self.bind_group_cache.lock();
        match cache.as_ref() {
            Some(_) => 1,
            None => 0,
        }
    }

    fn validate(
        &self,
        width: u32,
        height: u32,
        y_out: &wgpu::Buffer,
        uv_out: &wgpu::Buffer,
    ) -> Result<(), Nv12PackError> {
        assert!(self.max_width > 0, "validate requires positive max_width");
        assert!(self.max_height > 0, "validate requires positive max_height");
        if width == 0 {
            return Err(Nv12PackError::DimensionsZero);
        }
        if height == 0 {
            return Err(Nv12PackError::DimensionsZero);
        }
        if width > self.max_width {
            return Err(Nv12PackError::DimensionsExceedMax {
                requested: (width, height),
                max: (self.max_width, self.max_height),
            });
        }
        if height > self.max_height {
            return Err(Nv12PackError::DimensionsExceedMax {
                requested: (width, height),
                max: (self.max_width, self.max_height),
            });
        }
        if !width.is_multiple_of(2) {
            return Err(Nv12PackError::DimensionsNotEven {
                requested: (width, height),
            });
        }
        if !height.is_multiple_of(2) {
            return Err(Nv12PackError::DimensionsNotEven {
                requested: (width, height),
            });
        }
        if !width.is_multiple_of(4) {
            return Err(Nv12PackError::StrideNotAligned { stride: width });
        }
        let required_y = Self::y_plane_size(width, height);
        let actual_y = y_out.size();
        if actual_y < required_y {
            return Err(Nv12PackError::YBufferTooSmall {
                required: required_y,
                actual: actual_y,
            });
        }
        let required_uv = Self::uv_plane_size(width, height);
        let actual_uv = uv_out.size();
        if actual_uv < required_uv {
            return Err(Nv12PackError::UvBufferTooSmall {
                required: required_uv,
                actual: actual_uv,
            });
        }
        Ok(())
    }
}

impl GpuLossCallback for Nv12Packer {
    fn release(&mut self) {
        assert!(self.max_width > 0, "release precondition: max_width valid");
        assert!(
            self.max_height > 0,
            "release precondition: max_height valid"
        );
        self.resources = None;
        {
            let mut cache = self.bind_group_cache.lock();
            *cache = None;
            assert!(
                cache.is_none(),
                "release postcondition: cache must be cleared"
            );
        }
        assert!(!self.is_built(), "release postcondition: must be unbuilt");
    }

    fn rebuild(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Result<(), GpuRebuildError> {
        assert!(self.max_width > 0, "rebuild precondition: max_width valid");
        assert!(
            self.max_height > 0,
            "rebuild precondition: max_height valid"
        );
        if self.resources.is_some() {
            return Err(GpuRebuildError::OwnerInvariantBroken {
                reason: "rebuild without prior release",
            });
        }
        let resources = build_resources(device);
        self.resources = Some(resources);
        assert!(self.is_built(), "rebuild postcondition: must be built");
        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.is_built()
    }

    fn debug_label(&self) -> &'static str {
        "nv12_gpu_pack.packer"
    }
}

fn record_pack_pass(
    resources: &PackResources,
    bind_group: &wgpu::BindGroup,
    job: &mut PackJob<'_>,
) {
    let (width, height) = job.dims;
    assert!(
        width.is_multiple_of(2),
        "record_pack_pass width even precondition"
    );
    assert!(
        height.is_multiple_of(2),
        "record_pack_pass height even precondition"
    );
    let uniforms = PackUniforms {
        width,
        height,
        stride_y: width,
        stride_uv: width,
    };
    job.queue
        .write_buffer(&resources.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    job.encoder
        .clear_buffer(job.y_out, 0, Some(Nv12Packer::y_plane_size(width, height)));
    job.encoder.clear_buffer(
        job.uv_out,
        0,
        Some(Nv12Packer::uv_plane_size(width, height)),
    );
    let mut pass = job
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("nv12_gpu_pack.pass"),
            timestamp_writes: None,
        });
    pass.set_pipeline(&resources.pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    let groups_x = (width / 2).div_ceil(WORKGROUP_DIM);
    let groups_y = (height / 2).div_ceil(WORKGROUP_DIM);
    assert!(groups_x > 0, "record_pack_pass groups_x positive");
    assert!(groups_y > 0, "record_pack_pass groups_y positive");
    pass.dispatch_workgroups(groups_x, groups_y, 1);
}

fn build_bind_group(resources: &PackResources, job: &PackJob<'_>) -> wgpu::BindGroup {
    job.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("nv12_gpu_pack.bind_group"),
        layout: &resources.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(job.source),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: job.y_out.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: job.uv_out.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: resources.uniform_buffer.as_entire_binding(),
            },
        ],
    })
}

fn build_resources(device: &wgpu::Device) -> PackResources {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("nv12_gpu_pack.shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });
    let bind_group_layout = create_bind_group_layout(device);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("nv12_gpu_pack.pipeline_layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("nv12_gpu_pack.pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("pack_nv12"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("nv12_gpu_pack.uniforms"),
        contents: bytemuck::bytes_of(&PackUniforms {
            width: 0,
            height: 0,
            stride_y: 0,
            stride_uv: 0,
        }),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    PackResources {
        pipeline,
        bind_group_layout,
        uniform_buffer,
    }
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("nv12_gpu_pack.bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(std::mem::size_of::<PackUniforms>() as u64),
                },
                count: None,
            },
        ],
    })
}

pub fn try_acquire_device() -> Option<(wgpu::Device, wgpu::Queue, wgpu::Instance)> {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::all() | wgpu::Backends::SECONDARY;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .ok()?;
    let device_result = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("nv12_gpu_pack.test_device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::default(),
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::default(),
    }));
    match device_result {
        Ok((device, queue)) => Some((device, queue, instance)),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluxer_gpu_rebuild::{GpuLossRegistry, RebuildOutcome};

    struct GpuCtx {
        device: wgpu::Device,
        queue: wgpu::Queue,
        _instance: wgpu::Instance,
    }

    fn gpu_ctx() -> Option<GpuCtx> {
        let (device, queue, instance) = try_acquire_device()?;
        Some(GpuCtx {
            device,
            queue,
            _instance: instance,
        })
    }

    fn make_source_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> wgpu::Texture {
        assert_eq!(
            pixels.len() as u64,
            u64::from(width) * u64::from(height) * 4
        );
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nv12_gpu_pack.test_source"),
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
            pixels,
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
        texture
    }

    fn make_storage_buffer(device: &wgpu::Device, size: u64, label: &str) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn read_buffer(device: &wgpu::Device, queue: &wgpu::Queue, src: &wgpu::Buffer) -> Vec<u8> {
        let size = src.size();
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nv12_gpu_pack.test_staging"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("nv12_gpu_pack.test_readback"),
        });
        encoder.copy_buffer_to_buffer(src, 0, &staging, 0, size);
        queue.submit(std::iter::once(encoder.finish()));
        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        let _ = device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        let recv = rx.recv();
        assert!(recv.is_ok(), "channel must deliver map result");
        let map_result = recv.expect("map result channel");
        assert!(map_result.is_ok(), "map_async must succeed");
        let data = {
            let range = slice.get_mapped_range();
            range.to_vec()
        };
        staging.unmap();
        data
    }

    fn pack_and_readback(
        ctx: &GpuCtx,
        packer: &Nv12Packer,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> (Vec<u8>, Vec<u8>) {
        let y_size = Nv12Packer::y_plane_size(width, height);
        let uv_size = Nv12Packer::uv_plane_size(width, height);
        let y_buf = make_storage_buffer(&ctx.device, y_size, "y");
        let uv_buf = make_storage_buffer(&ctx.device, uv_size, "uv");
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("nv12_gpu_pack.test_encoder"),
            });
        let result = packer.pack(PackJob {
            device: &ctx.device,
            queue: &ctx.queue,
            encoder: &mut encoder,
            source: &view,
            y_out: &y_buf,
            uv_out: &uv_buf,
            dims: (width, height),
        });
        assert!(result.is_ok(), "pack must succeed: {result:?}");
        ctx.queue.submit(std::iter::once(encoder.finish()));
        let y = read_buffer(&ctx.device, &ctx.queue, &y_buf);
        let uv = read_buffer(&ctx.device, &ctx.queue, &uv_buf);
        (y, uv)
    }

    #[test]
    fn solid_gray_packs_to_expected_bt709_limited() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let width: u32 = 16;
        let height: u32 = 8;
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        for chunk in pixels.chunks_exact_mut(4) {
            chunk[0] = 128;
            chunk[1] = 128;
            chunk[2] = 128;
            chunk[3] = 255;
        }
        let tex = make_source_texture(&ctx.device, &ctx.queue, width, height, &pixels);
        let packer = Nv12Packer::new(&ctx.device, width, height);
        let (y, uv) = pack_and_readback(&ctx, &packer, &tex, width, height);
        for value in &y {
            assert!(
                (124..=128).contains(value),
                "Y value {value} out of expected range for gray 128"
            );
        }
        for value in &uv {
            assert!(
                (126..=130).contains(value),
                "UV value {value} out of expected range for neutral gray"
            );
        }
    }

    #[test]
    fn solid_red_v_is_above_neutral() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let width: u32 = 16;
        let height: u32 = 8;
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        for chunk in pixels.chunks_exact_mut(4) {
            chunk[0] = 255;
            chunk[1] = 0;
            chunk[2] = 0;
            chunk[3] = 255;
        }
        let tex = make_source_texture(&ctx.device, &ctx.queue, width, height, &pixels);
        let packer = Nv12Packer::new(&ctx.device, width, height);
        let (_y, uv) = pack_and_readback(&ctx, &packer, &tex, width, height);
        for pair in uv.chunks_exact(2) {
            let u = pair[0];
            let v = pair[1];
            assert!(u < 128, "expected U below neutral for pure red, got {u}");
            assert!(
                v > 200,
                "expected V well above neutral for pure red, got {v}"
            );
        }
    }

    #[test]
    fn rejects_dims_exceeding_max() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let max_w: u32 = 8;
        let max_h: u32 = 8;
        let packer = Nv12Packer::new(&ctx.device, max_w, max_h);
        let y_size = Nv12Packer::y_plane_size(16, 16);
        let uv_size = Nv12Packer::uv_plane_size(16, 16);
        let y_buf = make_storage_buffer(&ctx.device, y_size, "y");
        let uv_buf = make_storage_buffer(&ctx.device, uv_size, "uv");
        let dummy_pixels = vec![0u8; (max_w * max_h * 4) as usize];
        let tex = make_source_texture(&ctx.device, &ctx.queue, max_w, max_h, &dummy_pixels);
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let result = packer.pack(PackJob {
            device: &ctx.device,
            queue: &ctx.queue,
            encoder: &mut encoder,
            source: &view,
            y_out: &y_buf,
            uv_out: &uv_buf,
            dims: (16, 16),
        });
        assert!(matches!(
            result,
            Err(Nv12PackError::DimensionsExceedMax { .. })
        ));
    }

    #[test]
    fn rejects_odd_dims() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let packer = Nv12Packer::new(&ctx.device, 32, 32);
        let y_buf = make_storage_buffer(&ctx.device, 64, "y");
        let uv_buf = make_storage_buffer(&ctx.device, 64, "uv");
        let dummy_pixels = vec![0u8; (8 * 8 * 4) as usize];
        let tex = make_source_texture(&ctx.device, &ctx.queue, 8, 8, &dummy_pixels);
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let result = packer.pack(PackJob {
            device: &ctx.device,
            queue: &ctx.queue,
            encoder: &mut encoder,
            source: &view,
            y_out: &y_buf,
            uv_out: &uv_buf,
            dims: (7, 8),
        });
        assert!(matches!(
            result,
            Err(Nv12PackError::DimensionsNotEven { .. })
        ));
    }

    #[test]
    fn determinism_two_runs_same_input_same_output() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let width: u32 = 32;
        let height: u32 = 16;
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        for (i, chunk) in pixels.chunks_exact_mut(4).enumerate() {
            let v = (i % 251) as u8;
            chunk[0] = v;
            chunk[1] = v.wrapping_mul(2);
            chunk[2] = v.wrapping_add(31);
            chunk[3] = 255;
        }
        let tex = make_source_texture(&ctx.device, &ctx.queue, width, height, &pixels);
        let packer = Nv12Packer::new(&ctx.device, width, height);
        let (y1, uv1) = pack_and_readback(&ctx, &packer, &tex, width, height);
        let (y2, uv2) = pack_and_readback(&ctx, &packer, &tex, width, height);
        assert_eq!(y1, y2, "Y plane must be deterministic");
        assert_eq!(uv1, uv2, "UV plane must be deterministic");
    }

    #[test]
    fn buffer_too_small_is_rejected() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let packer = Nv12Packer::new(&ctx.device, 64, 64);
        let y_buf = make_storage_buffer(&ctx.device, 16, "y");
        let uv_buf = make_storage_buffer(&ctx.device, 16, "uv");
        let pixels = vec![0u8; (16 * 16 * 4) as usize];
        let tex = make_source_texture(&ctx.device, &ctx.queue, 16, 16, &pixels);
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let result = packer.pack(PackJob {
            device: &ctx.device,
            queue: &ctx.queue,
            encoder: &mut encoder,
            source: &view,
            y_out: &y_buf,
            uv_out: &uv_buf,
            dims: (16, 16),
        });
        assert!(matches!(result, Err(Nv12PackError::YBufferTooSmall { .. })));
    }

    #[test]
    fn packer_is_ready_only_when_built() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let mut packer = Nv12Packer::new(&ctx.device, 32, 32);
        assert!(
            packer.is_ready(),
            "freshly constructed packer must be ready"
        );
        packer.release();
        assert!(!packer.is_ready(), "released packer must not be ready");
        let rebuilt = packer.rebuild(&ctx.device, &ctx.queue);
        assert!(rebuilt.is_ok(), "rebuild on a fresh device must succeed");
        assert!(packer.is_ready(), "rebuilt packer must be ready");
    }

    #[test]
    fn pack_between_release_and_rebuild_returns_not_ready() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let mut packer = Nv12Packer::new(&ctx.device, 16, 16);
        packer.release();
        let y_buf = make_storage_buffer(&ctx.device, 256, "y");
        let uv_buf = make_storage_buffer(&ctx.device, 256, "uv");
        let dummy_pixels = vec![0u8; (16 * 16 * 4) as usize];
        let tex = make_source_texture(&ctx.device, &ctx.queue, 16, 16, &dummy_pixels);
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let result = packer.pack(PackJob {
            device: &ctx.device,
            queue: &ctx.queue,
            encoder: &mut encoder,
            source: &view,
            y_out: &y_buf,
            uv_out: &uv_buf,
            dims: (16, 16),
        });
        assert!(matches!(result, Err(Nv12PackError::NotReady)));
    }

    #[test]
    fn registry_release_then_rebuild_invokes_callback_in_order() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let registry = GpuLossRegistry::new();
        let packer = Box::new(Nv12Packer::new(&ctx.device, 16, 16));
        let _guard = registry.register(packer);
        let report = registry.handle_device_lost(&ctx.device, &ctx.queue);
        assert_eq!(report.released_count, 1);
        assert_eq!(report.rebuilt_count, 1);
        assert_eq!(report.failed_count, 0);
        assert!(report.is_total_success());
    }

    #[test]
    fn after_loss_handling_packer_can_pack_again() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let width: u32 = 16;
        let height: u32 = 8;
        let pixels = vec![128u8; (width * height * 4) as usize];
        let tex = make_source_texture(&ctx.device, &ctx.queue, width, height, &pixels);
        let mut packer = Nv12Packer::new(&ctx.device, width, height);
        packer.release();
        packer
            .rebuild(&ctx.device, &ctx.queue)
            .expect("rebuild must succeed after release");
        let (y, _uv) = pack_and_readback(&ctx, &packer, &tex, width, height);
        assert!(!y.is_empty(), "Y plane must be non-empty after rebuild");
        assert!(
            (124..=128).contains(&y[0]),
            "Y value out of range for gray 128 after rebuild: {}",
            y[0]
        );
    }

    #[test]
    fn determinism_across_rebuild_round_trip() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let width: u32 = 32;
        let height: u32 = 16;
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        for (i, chunk) in pixels.chunks_exact_mut(4).enumerate() {
            let v = (i % 251) as u8;
            chunk[0] = v;
            chunk[1] = v.wrapping_mul(2);
            chunk[2] = v.wrapping_add(31);
            chunk[3] = 255;
        }
        let tex = make_source_texture(&ctx.device, &ctx.queue, width, height, &pixels);
        let mut packer = Nv12Packer::new(&ctx.device, width, height);
        let (y1, uv1) = pack_and_readback(&ctx, &packer, &tex, width, height);
        packer.release();
        packer
            .rebuild(&ctx.device, &ctx.queue)
            .expect("rebuild must succeed");
        let (y2, uv2) = pack_and_readback(&ctx, &packer, &tex, width, height);
        assert_eq!(y1, y2, "Y plane must match across rebuild round-trip");
        assert_eq!(uv1, uv2, "UV plane must match across rebuild round-trip");
    }

    #[test]
    fn double_rebuild_without_release_is_owner_invariant_error() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let mut packer = Nv12Packer::new(&ctx.device, 16, 16);
        let outcome = packer.rebuild(&ctx.device, &ctx.queue);
        assert!(matches!(
            outcome,
            Err(GpuRebuildError::OwnerInvariantBroken { .. })
        ));
    }

    struct FailingRebuildOwner {
        ready: bool,
    }

    impl FailingRebuildOwner {
        fn new() -> Self {
            Self { ready: true }
        }
    }

    impl GpuLossCallback for FailingRebuildOwner {
        fn release(&mut self) {
            let was_ready = self.ready;
            assert!(
                was_ready,
                "FailingRebuildOwner release must observe ready state",
            );
            self.ready = false;
            assert!(!self.ready, "FailingRebuildOwner postcondition");
        }
        fn rebuild(
            &mut self,
            _device: &wgpu::Device,
            _queue: &wgpu::Queue,
        ) -> Result<(), GpuRebuildError> {
            assert!(
                !self.ready,
                "rebuild must run after release in FailingRebuildOwner"
            );
            Err(GpuRebuildError::DeviceRejected {
                reason: "synthetic second-device failure",
            })
        }
        fn is_ready(&self) -> bool {
            self.ready
        }
        fn debug_label(&self) -> &'static str {
            "test.failing_rebuild_owner"
        }
    }

    #[test]
    fn failed_rebuild_surfaces_in_report_and_owner_reports_not_ready() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let registry = GpuLossRegistry::new();
        let _guard = registry.register(Box::new(FailingRebuildOwner::new()));
        let report = registry.handle_device_lost(&ctx.device, &ctx.queue);
        assert_eq!(report.released_count, 1);
        assert_eq!(report.failed_count, 1);
        assert_eq!(report.rebuilt_count, 0);
        assert!(!report.is_total_success());
        let failed = report
            .outcomes
            .iter()
            .filter(|o| matches!(o, RebuildOutcome::Failed { .. }))
            .count();
        assert_eq!(failed, 1);
    }

    #[test]
    fn multiple_packers_release_lifo_rebuild_fifo() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let registry = GpuLossRegistry::new();
        let mut guards = Vec::new();
        for _ in 0..3u32 {
            let packer = Box::new(Nv12Packer::new(&ctx.device, 16, 16));
            guards.push(registry.register(packer));
        }
        assert_eq!(registry.len(), 3);
        let report = registry.handle_device_lost(&ctx.device, &ctx.queue);
        assert_eq!(report.released_count, 3);
        assert_eq!(report.rebuilt_count, 3);
        assert_eq!(report.failed_count, 0);
        for outcome in &report.outcomes {
            assert!(matches!(outcome, RebuildOutcome::Rebuilt { .. }));
        }
        drop(guards);
    }

    fn pack_one_pass(
        ctx: &GpuCtx,
        packer: &Nv12Packer,
        view: &wgpu::TextureView,
        y_buf: &wgpu::Buffer,
        uv_buf: &wgpu::Buffer,
        width: u32,
        height: u32,
    ) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("nv12_gpu_pack.cache_test_encoder"),
            });
        let result = packer.pack(PackJob {
            device: &ctx.device,
            queue: &ctx.queue,
            encoder: &mut encoder,
            source: view,
            y_out: y_buf,
            uv_out: uv_buf,
            dims: (width, height),
        });
        assert!(result.is_ok(), "pack must succeed: {result:?}");
        ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    #[test]
    fn cache_hit_when_pack_called_twice_with_same_inputs() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let width: u32 = 32;
        let height: u32 = 16;
        let pixels = vec![64u8; (width * height * 4) as usize];
        let tex = make_source_texture(&ctx.device, &ctx.queue, width, height, &pixels);
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let packer = Nv12Packer::new(&ctx.device, width, height);
        let y_buf = make_storage_buffer(&ctx.device, Nv12Packer::y_plane_size(width, height), "y");
        let uv_buf =
            make_storage_buffer(&ctx.device, Nv12Packer::uv_plane_size(width, height), "uv");
        assert_eq!(packer.cached_bind_group_count(), 0);
        pack_one_pass(&ctx, &packer, &view, &y_buf, &uv_buf, width, height);
        assert_eq!(packer.cached_bind_group_count(), 1);
        pack_one_pass(&ctx, &packer, &view, &y_buf, &uv_buf, width, height);
        assert_eq!(packer.cached_bind_group_count(), 1);
    }

    #[test]
    fn cache_miss_when_buffers_change() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let width: u32 = 32;
        let height: u32 = 16;
        let pixels = vec![64u8; (width * height * 4) as usize];
        let tex = make_source_texture(&ctx.device, &ctx.queue, width, height, &pixels);
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let packer = Nv12Packer::new(&ctx.device, width, height);
        let y_buf_a =
            make_storage_buffer(&ctx.device, Nv12Packer::y_plane_size(width, height), "y_a");
        let uv_buf_a = make_storage_buffer(
            &ctx.device,
            Nv12Packer::uv_plane_size(width, height),
            "uv_a",
        );
        pack_one_pass(&ctx, &packer, &view, &y_buf_a, &uv_buf_a, width, height);
        assert_eq!(packer.cached_bind_group_count(), 1);
        let y_buf_b =
            make_storage_buffer(&ctx.device, Nv12Packer::y_plane_size(width, height), "y_b");
        let uv_buf_b = make_storage_buffer(
            &ctx.device,
            Nv12Packer::uv_plane_size(width, height),
            "uv_b",
        );
        pack_one_pass(&ctx, &packer, &view, &y_buf_b, &uv_buf_b, width, height);
        assert_eq!(packer.cached_bind_group_count(), 1);
        pack_one_pass(&ctx, &packer, &view, &y_buf_a, &uv_buf_a, width, height);
        assert_eq!(packer.cached_bind_group_count(), 1);
    }

    #[test]
    fn cache_invalidated_on_release() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let width: u32 = 32;
        let height: u32 = 16;
        let pixels = vec![64u8; (width * height * 4) as usize];
        let tex = make_source_texture(&ctx.device, &ctx.queue, width, height, &pixels);
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut packer = Nv12Packer::new(&ctx.device, width, height);
        let y_buf = make_storage_buffer(&ctx.device, Nv12Packer::y_plane_size(width, height), "y");
        let uv_buf =
            make_storage_buffer(&ctx.device, Nv12Packer::uv_plane_size(width, height), "uv");
        pack_one_pass(&ctx, &packer, &view, &y_buf, &uv_buf, width, height);
        assert_eq!(packer.cached_bind_group_count(), 1);
        packer.release();
        assert_eq!(packer.cached_bind_group_count(), 0);
        packer
            .rebuild(&ctx.device, &ctx.queue)
            .expect("rebuild must succeed");
        assert_eq!(packer.cached_bind_group_count(), 0);
        pack_one_pass(&ctx, &packer, &view, &y_buf, &uv_buf, width, height);
        assert_eq!(packer.cached_bind_group_count(), 1);
    }

    #[test]
    fn cache_replaces_when_source_view_changes() {
        let Some(ctx) = gpu_ctx() else {
            eprintln!("nv12-gpu-pack: no wgpu adapter, skipping");
            return;
        };
        let width: u32 = 32;
        let height: u32 = 16;
        let pixels = vec![64u8; (width * height * 4) as usize];
        let tex_a = make_source_texture(&ctx.device, &ctx.queue, width, height, &pixels);
        let tex_b = make_source_texture(&ctx.device, &ctx.queue, width, height, &pixels);
        let view_a = tex_a.create_view(&wgpu::TextureViewDescriptor::default());
        let view_b = tex_b.create_view(&wgpu::TextureViewDescriptor::default());
        let packer = Nv12Packer::new(&ctx.device, width, height);
        let y_buf = make_storage_buffer(&ctx.device, Nv12Packer::y_plane_size(width, height), "y");
        let uv_buf =
            make_storage_buffer(&ctx.device, Nv12Packer::uv_plane_size(width, height), "uv");
        pack_one_pass(&ctx, &packer, &view_a, &y_buf, &uv_buf, width, height);
        assert_eq!(packer.cached_bind_group_count(), 1);
        pack_one_pass(&ctx, &packer, &view_b, &y_buf, &uv_buf, width, height);
        assert_eq!(packer.cached_bind_group_count(), 1);
    }
}
