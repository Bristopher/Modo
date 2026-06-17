// SPDX-License-Identifier: AGPL-3.0-or-later

use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_RENDER_TARGET, D3D11_RESOURCE_MISC_SHARED, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_DEFAULT, D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE, D3D11_VIDEO_PROCESSOR_CONTENT_DESC,
    D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC, D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0,
    D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC, D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0,
    D3D11_VIDEO_PROCESSOR_STREAM, D3D11_VIDEO_USAGE_PLAYBACK_NORMAL,
    D3D11_VPIV_DIMENSION_TEXTURE2D, D3D11_VPOV_DIMENSION_TEXTURE2D, ID3D11Device,
    ID3D11DeviceContext, ID3D11Texture2D, ID3D11VideoContext, ID3D11VideoContext1,
    ID3D11VideoDevice, ID3D11VideoProcessor, ID3D11VideoProcessorEnumerator,
    ID3D11VideoProcessorInputView, ID3D11VideoProcessorOutputView,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_COLOR_SPACE_RGB_FULL_G10_NONE_P709, DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709,
    DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020, DXGI_COLOR_SPACE_TYPE,
    DXGI_COLOR_SPACE_YCBCR_STUDIO_G22_LEFT_P709, DXGI_FORMAT_NV12, DXGI_RATIONAL, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::IDXGIResource;
use windows::core::Interface;

use crate::hdr;

fn vlog(msg: &str) {
    if crate::game_capture_abi::env_flag_enabled(crate::game_capture_abi::ENV_VERBOSE) {
        use std::io::Write;
        let _ = writeln!(std::io::stderr(), "[fluxer-nv12] {msg}");
    }
}

pub const NV12_OUTPUT_SLOT_COUNT: usize = 3;

struct Nv12OutputSlot {
    _texture: ID3D11Texture2D,
    view: ID3D11VideoProcessorOutputView,
    handle: u64,
}

pub struct Nv12GpuConverter {
    _video_device: ID3D11VideoDevice,
    video_context: ID3D11VideoContext,
    processor: ID3D11VideoProcessor,
    _enumerator: ID3D11VideoProcessorEnumerator,
    input_view: ID3D11VideoProcessorInputView,
    output_slots: [Nv12OutputSlot; NV12_OUTPUT_SLOT_COUNT],
    slot_cursor: usize,
    context: ID3D11DeviceContext,
    out_width: u32,
    out_height: u32,
}

pub struct Nv12SharedTextureFrame {
    pub handle: u64,
    pub width: u32,
    pub height: u32,
    pub dxgi_format: u32,
}

impl Nv12GpuConverter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &ID3D11Device,
        context: &ID3D11DeviceContext,
        input: &ID3D11Texture2D,
        in_width: u32,
        in_height: u32,
        out_width: u32,
        out_height: u32,
        source_format: hdr::SourceFormat,
    ) -> Option<Self> {
        let out_width = (out_width & !1).max(2);
        let out_height = (out_height & !1).max(2);
        let video_device = device
            .cast::<ID3D11VideoDevice>()
            .inspect_err(|e| vlog(&format!("cast ID3D11VideoDevice failed: {e:?}")))
            .ok()?;
        let video_context = context
            .cast::<ID3D11VideoContext>()
            .inspect_err(|e| vlog(&format!("cast ID3D11VideoContext failed: {e:?}")))
            .ok()?;

        let content_desc = D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
            InputFrameFormat: D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
            InputFrameRate: DXGI_RATIONAL {
                Numerator: 60,
                Denominator: 1,
            },
            InputWidth: in_width,
            InputHeight: in_height,
            OutputFrameRate: DXGI_RATIONAL {
                Numerator: 60,
                Denominator: 1,
            },
            OutputWidth: out_width,
            OutputHeight: out_height,
            Usage: D3D11_VIDEO_USAGE_PLAYBACK_NORMAL,
        };
        let enumerator = unsafe { video_device.CreateVideoProcessorEnumerator(&content_desc) }
            .inspect_err(|e| vlog(&format!("CreateVideoProcessorEnumerator: {e:?}")))
            .ok()?;
        let processor = unsafe { video_device.CreateVideoProcessor(&enumerator, 0) }
            .inspect_err(|e| vlog(&format!("CreateVideoProcessor: {e:?}")))
            .ok()?;

        if let Ok(vctx1) = video_context.cast::<ID3D11VideoContext1>() {
            let input_cs = input_colour_space(source_format);
            unsafe {
                vctx1.VideoProcessorSetStreamColorSpace1(&processor, 0, input_cs);
                vctx1.VideoProcessorSetOutputColorSpace1(
                    &processor,
                    DXGI_COLOR_SPACE_YCBCR_STUDIO_G22_LEFT_P709,
                );
            }
            vlog(&format!(
                "video processor colour space set: input={} -> output=YCbCr studio Rec.709",
                input_cs.0
            ));
        } else {
            vlog("ID3D11VideoContext1 unavailable; using default SDR Rec.709 colour space");
        }

        let output_desc = D3D11_TEXTURE2D_DESC {
            Width: out_width,
            Height: out_height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_NV12,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: D3D11_RESOURCE_MISC_SHARED.0 as u32,
        };
        let mut output_slots = Vec::with_capacity(NV12_OUTPUT_SLOT_COUNT);
        for _ in 0..NV12_OUTPUT_SLOT_COUNT {
            output_slots.push(create_output_slot(
                device,
                &video_device,
                &enumerator,
                &output_desc,
            )?);
        }
        assert_eq!(
            output_slots.len(),
            NV12_OUTPUT_SLOT_COUNT,
            "all NV12 output slots created"
        );
        let Ok(output_slots) = <[Nv12OutputSlot; NV12_OUTPUT_SLOT_COUNT]>::try_from(output_slots)
        else {
            vlog("NV12 output slot count mismatch");
            return None;
        };

        let input_view_desc = D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {
            FourCC: 0,
            ViewDimension: D3D11_VPIV_DIMENSION_TEXTURE2D,
            Anonymous: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 {
                Texture2D: windows::Win32::Graphics::Direct3D11::D3D11_TEX2D_VPIV {
                    MipSlice: 0,
                    ArraySlice: 0,
                },
            },
        };
        let mut input_view = None;
        unsafe {
            video_device.CreateVideoProcessorInputView(
                input,
                &enumerator,
                &input_view_desc,
                Some(&mut input_view),
            )
        }
        .inspect_err(|e| vlog(&format!("CreateVideoProcessorInputView: {e:?}")))
        .ok()?;
        let input_view = input_view?;
        vlog(&format!(
            "NV12 converter built OK ({in_width}x{in_height} -> {out_width}x{out_height})"
        ));

        Some(Self {
            _video_device: video_device,
            video_context,
            processor,
            _enumerator: enumerator,
            input_view,
            output_slots,
            slot_cursor: 0,
            context: context.clone(),
            out_width,
            out_height,
        })
    }

    pub fn dxgi_format(&self) -> u32 {
        DXGI_FORMAT_NV12.0 as u32
    }

    pub fn convert_shared_texture(&mut self) -> Result<Nv12SharedTextureFrame, String> {
        assert!(
            self.slot_cursor < NV12_OUTPUT_SLOT_COUNT,
            "slot cursor in range"
        );
        assert!(self.out_width >= 2, "output width at least 2");
        let slot_index = self.slot_cursor;
        self.slot_cursor = (slot_index + 1) % NV12_OUTPUT_SLOT_COUNT;
        self.run_video_processor(slot_index)?;
        unsafe {
            self.context.Flush();
        }
        Ok(Nv12SharedTextureFrame {
            handle: self.output_slots[slot_index].handle,
            width: self.out_width,
            height: self.out_height,
            dxgi_format: self.dxgi_format(),
        })
    }

    fn run_video_processor(&self, slot_index: usize) -> Result<(), String> {
        assert!(slot_index < NV12_OUTPUT_SLOT_COUNT, "slot index in range");
        let mut stream = D3D11_VIDEO_PROCESSOR_STREAM {
            Enable: windows::core::BOOL(1),
            OutputIndex: 0,
            InputFrameOrField: 0,
            PastFrames: 0,
            FutureFrames: 0,
            ppPastSurfaces: std::ptr::null_mut(),
            pInputSurface: std::mem::ManuallyDrop::new(Some(self.input_view.clone())),
            ppFutureSurfaces: std::ptr::null_mut(),
            ppPastSurfacesRight: std::ptr::null_mut(),
            pInputSurfaceRight: std::mem::ManuallyDrop::new(None),
            ppFutureSurfacesRight: std::ptr::null_mut(),
        };
        let blt = unsafe {
            self.video_context.VideoProcessorBlt(
                &self.processor,
                &self.output_slots[slot_index].view,
                0,
                std::slice::from_ref(&stream),
            )
        };
        unsafe {
            std::mem::ManuallyDrop::drop(&mut stream.pInputSurface);
        }
        blt.inspect_err(|e| vlog(&format!("VideoProcessorBlt RGB->NV12: {e:?}")))
            .map_err(|e| format!("VideoProcessorBlt RGB->NV12: {e}"))
    }
}

fn create_output_slot(
    device: &ID3D11Device,
    video_device: &ID3D11VideoDevice,
    enumerator: &ID3D11VideoProcessorEnumerator,
    output_desc: &D3D11_TEXTURE2D_DESC,
) -> Option<Nv12OutputSlot> {
    assert!(output_desc.Width >= 2, "output width at least 2");
    assert!(output_desc.Height >= 2, "output height at least 2");
    let mut output_texture = None;
    unsafe { device.CreateTexture2D(output_desc, None, Some(&mut output_texture)) }
        .inspect_err(|e| vlog(&format!("CreateTexture2D NV12 output: {e:?}")))
        .ok()?;
    let output_texture = output_texture?;
    let resource: IDXGIResource = output_texture
        .cast()
        .inspect_err(|e| {
            vlog(&format!(
                "QueryInterface IDXGIResource for NV12 output: {e:?}"
            ))
        })
        .ok()?;
    let shared_handle = unsafe { resource.GetSharedHandle() }
        .inspect_err(|e| vlog(&format!("GetSharedHandle NV12 output: {e:?}")))
        .ok()?;
    if shared_handle.is_invalid() {
        vlog("GetSharedHandle NV12 output returned an invalid handle");
        return None;
    }
    let shared_handle = shared_handle.0 as usize as u64;

    let output_view_desc = D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
        ViewDimension: D3D11_VPOV_DIMENSION_TEXTURE2D,
        Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
            Texture2D: windows::Win32::Graphics::Direct3D11::D3D11_TEX2D_VPOV { MipSlice: 0 },
        },
    };
    let mut output_view = None;
    unsafe {
        video_device.CreateVideoProcessorOutputView(
            &output_texture,
            enumerator,
            &output_view_desc,
            Some(&mut output_view),
        )
    }
    .inspect_err(|e| vlog(&format!("CreateVideoProcessorOutputView: {e:?}")))
    .ok()?;
    let output_view = output_view?;

    Some(Nv12OutputSlot {
        _texture: output_texture,
        view: output_view,
        handle: shared_handle,
    })
}

fn input_colour_space(source_format: hdr::SourceFormat) -> DXGI_COLOR_SPACE_TYPE {
    match source_format {
        hdr::SourceFormat::R10G10B10A2 { hdr: true } => DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020,
        hdr::SourceFormat::Rgba16Float { hdr: true } => DXGI_COLOR_SPACE_RGB_FULL_G10_NONE_P709,
        _ => DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709,
    }
}

unsafe impl Send for Nv12GpuConverter {}

#[cfg(test)]
mod tests {
    use super::*;

    fn cs_value(source_format: hdr::SourceFormat) -> i32 {
        input_colour_space(source_format).0
    }

    #[test]
    fn eight_bit_sources_use_sdr_rec709_colour_space() {
        assert_eq!(
            cs_value(hdr::SourceFormat::Bgra8),
            DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709.0
        );
        assert_eq!(
            cs_value(hdr::SourceFormat::Rgba8),
            DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709.0
        );
    }

    #[test]
    fn unflagged_high_precision_sources_stay_sdr_rec709() {
        assert_eq!(
            cs_value(hdr::SourceFormat::R10G10B10A2 { hdr: false }),
            DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709.0
        );
        assert_eq!(
            cs_value(hdr::SourceFormat::Rgba16Float { hdr: false }),
            DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709.0
        );
    }

    #[test]
    fn ten_bit_hdr_uses_pq_rec2020_input_space() {
        assert_eq!(
            cs_value(hdr::SourceFormat::R10G10B10A2 { hdr: true }),
            DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020.0
        );
    }

    #[test]
    fn fp16_hdr_uses_linear_extended_rec709_input_space() {
        assert_eq!(
            cs_value(hdr::SourceFormat::Rgba16Float { hdr: true }),
            DXGI_COLOR_SPACE_RGB_FULL_G10_NONE_P709.0
        );
    }
}
