// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(target_os = "windows")]
use std::ffi::c_void;
use std::ptr;

use libloading::{Library, Symbol};
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL_11_0};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_VIDEO_SUPPORT, D3D11_SDK_VERSION,
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Multithread,
};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory1};
use windows::core::Interface;

const VENDOR_INTEL: u32 = 0x8086;

const MFX_IMPL_TYPE_HARDWARE: u32 = 2;
const MFX_ACCEL_MODE_NA: u32 = 0;
const MFX_ACCEL_MODE_VIA_D3D11: u32 = 0x0300;

const MFX_HANDLE_D3D11_DEVICE: u32 = 3;
const MFX_HANDLE_D3D11_VIDEO_DEVICE_GUESS: u32 = 11;

const MFX_VARIANT_TYPE_U32: u32 = 5;
const MFX_VARIANT_VERSION_MAJOR: u8 = 1;
const MFX_VARIANT_VERSION_MINOR: u8 = 1;

const MFX_ERR_NONE: i32 = 0;
const MFX_WRN_IN_EXECUTION: i32 = 1;

const MFX_FOURCC_NV12: u32 = u32::from_le_bytes(*b"NV12");
const MFX_CODEC_AVC: u32 = u32::from_le_bytes(*b"AVC ");
const MFX_RATECONTROL_CBR: u16 = 1;
const MFX_PICSTRUCT_PROGRESSIVE: u16 = 0x01;
const MFX_CHROMAFORMAT_YUV420: u16 = 1;
const MFX_IOPATTERN_IN_VIDEO_MEMORY: u16 = 0x01;
const MFX_IOPATTERN_IN_SYSTEM_MEMORY: u16 = 0x02;

const FILTER_PROPERTY_IMPL: &[u8] = b"mfxImplDescription.Impl\0";
const FILTER_PROPERTY_ACCEL: &[u8] = b"mfxImplDescription.AccelerationMode\0";

const QSV_DLL_NAME_VPL: &str = "libvpl.dll";

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct MfxStructVersion {
    minor: u8,
    major: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
union MfxVariantData {
    u32_: u32,
    u64_: u64,
    ptr: *mut c_void,
    pad: [u8; 16],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MfxVariant {
    version: MfxStructVersion,
    type_: u32,
    data: MfxVariantData,
}

#[repr(C)]
#[derive(Default)]
struct MfxFrameInfo {
    reserved: [u32; 4],
    channel_id: u16,
    bit_depth_luma: u16,
    bit_depth_chroma: u16,
    shift: u16,
    frame_id_temporal: u16,
    frame_id_priority: u16,
    frame_id_view: u16,
    frame_id_quality: u16,
    four_cc: u32,
    width: u16,
    height: u16,
    crop_x: u16,
    crop_y: u16,
    crop_w: u16,
    crop_h: u16,
    frame_rate_extn: u32,
    frame_rate_extd: u32,
    reserved3: u16,
    aspect_ratio_w: u16,
    aspect_ratio_h: u16,
    pic_struct: u16,
    chroma_format: u16,
    reserved2: u16,
}

#[repr(C)]
#[derive(Default)]
struct MfxInfoMfx {
    reserved: [u32; 7],
    low_power: u16,
    brc_param_multiplier: u16,
    frame_info: MfxFrameInfo,
    codec_id: u32,
    codec_profile: u16,
    codec_level: u16,
    num_thread: u16,
    target_usage: u16,
    gop_pic_size: u16,
    gop_ref_dist: u16,
    gop_opt_flag: u16,
    idr_interval: u16,
    rate_control_method: u16,
    init_qp: u16,
    buffer_size_in_kb: u16,
    target_kbps: u16,
    max_kbps: u16,
    num_slice: u16,
    num_ref_frame: u16,
    encoded_order: u16,
    union_pad: [u16; 15],
}

#[repr(C)]
struct MfxVideoParam {
    alloc_id: u32,
    reserved: [u32; 2],
    reserved3: u16,
    async_depth: u16,
    mfx: MfxInfoMfx,
    protected: u16,
    io_pattern: u16,
    ext_param: *mut c_void,
    num_ext_param: u16,
    reserved2: u16,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct MfxPlatform {
    code_name: u16,
    device_id: u16,
    media_adapter_type: u16,
    reserved: [u16; 13],
}

type MfxLoad = unsafe extern "C" fn() -> *mut c_void;
type MfxUnload = unsafe extern "C" fn(*mut c_void);
type MfxCreateConfig = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type MfxSetConfigFilterProperty = unsafe extern "C" fn(*mut c_void, *const u8, MfxVariant) -> i32;
type MfxCreateSession = unsafe extern "C" fn(*mut c_void, u32, *mut *mut c_void) -> i32;
type MfxClose = unsafe extern "C" fn(*mut c_void) -> i32;
type MfxSetHandle = unsafe extern "C" fn(*mut c_void, u32, *mut c_void) -> i32;
type MfxQueryImpl = unsafe extern "C" fn(*mut c_void, *mut i32) -> i32;
type MfxQueryPlatform = unsafe extern "C" fn(*mut c_void, *mut MfxPlatform) -> i32;
type MfxEncodeInit = unsafe extern "C" fn(*mut c_void, *mut MfxVideoParam) -> i32;
type MfxEncodeClose = unsafe extern "C" fn(*mut c_void) -> i32;
type MfxEncodeQuery =
    unsafe extern "C" fn(*mut c_void, *mut MfxVideoParam, *mut MfxVideoParam) -> i32;
type MfxGetHandle = unsafe extern "C" fn(*mut c_void, u32, *mut *mut c_void) -> i32;

struct Dispatcher {
    library: Library,
}

impl Dispatcher {
    fn load() -> Option<Self> {
        let library = unsafe { Library::new(QSV_DLL_NAME_VPL) }.ok()?;
        Some(Self { library })
    }

    fn modern_session(&self, accel_mode: u32) -> (Option<(*mut c_void, *mut c_void)>, Vec<String>) {
        let mut log = Vec::new();
        let load: Symbol<'_, MfxLoad> = match unsafe { self.library.get(b"MFXLoad\0") } {
            Ok(s) => s,
            Err(e) => {
                log.push(format!("MFXLoad symbol missing: {e:?}"));
                return (None, log);
            }
        };
        let create_config: Symbol<'_, MfxCreateConfig> =
            match unsafe { self.library.get(b"MFXCreateConfig\0") } {
                Ok(s) => s,
                Err(e) => {
                    log.push(format!("MFXCreateConfig missing: {e:?}"));
                    return (None, log);
                }
            };
        let set_prop: Symbol<'_, MfxSetConfigFilterProperty> =
            match unsafe { self.library.get(b"MFXSetConfigFilterProperty\0") } {
                Ok(s) => s,
                Err(e) => {
                    log.push(format!("MFXSetConfigFilterProperty missing: {e:?}"));
                    return (None, log);
                }
            };
        let create_session: Symbol<'_, MfxCreateSession> =
            match unsafe { self.library.get(b"MFXCreateSession\0") } {
                Ok(s) => s,
                Err(e) => {
                    log.push(format!("MFXCreateSession missing: {e:?}"));
                    return (None, log);
                }
            };
        let loader = unsafe { load() };
        if loader.is_null() {
            log.push("MFXLoad returned NULL".to_string());
            return (None, log);
        }
        log.push(format!("MFXLoad ok loader={loader:p}"));
        let impl_cfg = unsafe { create_config(loader) };
        if impl_cfg.is_null() {
            log.push("MFXCreateConfig(impl) returned NULL".to_string());
            return (None, log);
        }
        let impl_variant = MfxVariant {
            version: MfxStructVersion {
                minor: MFX_VARIANT_VERSION_MINOR,
                major: MFX_VARIANT_VERSION_MAJOR,
            },
            type_: MFX_VARIANT_TYPE_U32,
            data: MfxVariantData {
                u32_: MFX_IMPL_TYPE_HARDWARE,
            },
        };
        let status_impl =
            unsafe { set_prop(impl_cfg, FILTER_PROPERTY_IMPL.as_ptr(), impl_variant) };
        log.push(format!(
            "SetConfigFilterProperty(Impl=HARDWARE) status={status_impl}"
        ));
        if status_impl != MFX_ERR_NONE {
            return (None, log);
        }
        let accel_cfg = unsafe { create_config(loader) };
        if accel_cfg.is_null() {
            log.push("MFXCreateConfig(accel) returned NULL".to_string());
            return (None, log);
        }
        let accel_variant = MfxVariant {
            version: MfxStructVersion {
                minor: MFX_VARIANT_VERSION_MINOR,
                major: MFX_VARIANT_VERSION_MAJOR,
            },
            type_: MFX_VARIANT_TYPE_U32,
            data: MfxVariantData { u32_: accel_mode },
        };
        let status_accel =
            unsafe { set_prop(accel_cfg, FILTER_PROPERTY_ACCEL.as_ptr(), accel_variant) };
        log.push(format!(
            "SetConfigFilterProperty(AccelerationMode={accel_mode:#x}) status={status_accel}"
        ));
        if status_accel != MFX_ERR_NONE {
            return (None, log);
        }
        let mut session: *mut c_void = ptr::null_mut();
        let status_create = unsafe { create_session(loader, 0, &mut session) };
        log.push(format!(
            "MFXCreateSession status={status_create} session={session:p}"
        ));
        if status_create != MFX_ERR_NONE || session.is_null() {
            return (None, log);
        }
        (Some((loader, session)), log)
    }

    fn unload(&self, loader: *mut c_void) {
        let Ok(unload) = (unsafe { self.library.get::<MfxUnload>(b"MFXUnload\0") }) else {
            return;
        };
        if !loader.is_null() {
            unsafe { unload(loader) };
        }
    }

    fn close(&self, session: *mut c_void) {
        let Ok(close) = (unsafe { self.library.get::<MfxClose>(b"MFXClose\0") }) else {
            return;
        };
        if !session.is_null() {
            unsafe {
                let _ = close(session);
            }
        }
    }
}

fn try_create_intel_device() -> Option<(ID3D11Device, ID3D11DeviceContext)> {
    let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }.ok()?;
    let mut idx: u32 = 0;
    loop {
        let adapter: IDXGIAdapter1 = match unsafe { factory.EnumAdapters1(idx) } {
            Ok(a) => a,
            Err(_) => return None,
        };
        idx = idx.saturating_add(1);
        let desc = match unsafe { adapter.GetDesc1() } {
            Ok(d) => d,
            Err(_) => continue,
        };
        if desc.VendorId != VENDOR_INTEL {
            continue;
        }
        let mut device: Option<ID3D11Device> = None;
        let mut ctx: Option<ID3D11DeviceContext> = None;
        let feature_levels = [D3D_FEATURE_LEVEL_11_0];
        let cast_result = adapter.cast::<windows::Win32::Graphics::Dxgi::IDXGIAdapter>();
        let cast_adapter = match cast_result {
            Ok(a) => a,
            Err(_) => continue,
        };
        let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT;
        let result = unsafe {
            D3D11CreateDevice(
                Some(&cast_adapter),
                D3D_DRIVER_TYPE_UNKNOWN,
                Default::default(),
                flags,
                Some(&feature_levels),
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut ctx),
            )
        };
        if let (Ok(()), Some(d), Some(c)) = (result, device, ctx) {
            if let Ok(mt) = d.cast::<ID3D11Multithread>() {
                let _ = unsafe { mt.SetMultithreadProtected(true) };
            }
            return Some((d, c));
        }
    }
}

fn build_video_params_with(
    width: u16,
    height: u16,
    io_pattern: u16,
    target_usage: u16,
    low_power: u16,
) -> MfxVideoParam {
    let aligned_w = (width + 15) & !15;
    let aligned_h = (height + 15) & !15;
    let info = MfxFrameInfo {
        four_cc: MFX_FOURCC_NV12,
        width: aligned_w,
        height: aligned_h,
        crop_w: width,
        crop_h: height,
        frame_rate_extn: 30,
        frame_rate_extd: 1,
        aspect_ratio_w: 1,
        aspect_ratio_h: 1,
        pic_struct: MFX_PICSTRUCT_PROGRESSIVE,
        chroma_format: MFX_CHROMAFORMAT_YUV420,
        ..Default::default()
    };
    let mfx = MfxInfoMfx {
        frame_info: info,
        codec_id: MFX_CODEC_AVC,
        target_usage,
        low_power,
        gop_pic_size: 30,
        gop_ref_dist: 1,
        rate_control_method: MFX_RATECONTROL_CBR,
        target_kbps: 5_000,
        max_kbps: 5_000,
        num_slice: 1,
        num_ref_frame: 1,
        ..Default::default()
    };
    MfxVideoParam {
        alloc_id: 0,
        reserved: [0; 2],
        reserved3: 0,
        async_depth: 1,
        mfx,
        protected: 0,
        io_pattern,
        ext_param: ptr::null_mut(),
        num_ext_param: 0,
        reserved2: 0,
    }
}

fn try_set_handle(
    dispatcher: &Dispatcher,
    session: *mut c_void,
    handle_type: u32,
    handle_ptr: *mut c_void,
) -> Result<i32, String> {
    let sym: Symbol<'_, MfxSetHandle> =
        unsafe { dispatcher.library.get(b"MFXVideoCORE_SetHandle\0") }
            .map_err(|e| format!("MFXVideoCORE_SetHandle missing: {e:?}"))?;
    let status = unsafe { sym(session, handle_type, handle_ptr) };
    Ok(status)
}

fn try_query_impl(dispatcher: &Dispatcher, session: *mut c_void) -> Result<(i32, i32), String> {
    let sym: Symbol<'_, MfxQueryImpl> = unsafe { dispatcher.library.get(b"MFXQueryIMPL\0") }
        .map_err(|e| format!("MFXQueryIMPL missing: {e:?}"))?;
    let mut out: i32 = 0;
    let status = unsafe { sym(session, &mut out) };
    Ok((status, out))
}

fn try_query_platform(
    dispatcher: &Dispatcher,
    session: *mut c_void,
) -> Result<(i32, MfxPlatform), String> {
    let sym: Symbol<'_, MfxQueryPlatform> =
        unsafe { dispatcher.library.get(b"MFXVideoCORE_QueryPlatform\0") }
            .map_err(|e| format!("MFXVideoCORE_QueryPlatform missing: {e:?}"))?;
    let mut p = MfxPlatform::default();
    let status = unsafe { sym(session, &mut p) };
    Ok((status, p))
}

fn try_encode_init(
    dispatcher: &Dispatcher,
    session: *mut c_void,
    width: u16,
    height: u16,
    io_pattern: u16,
) -> Result<i32, String> {
    try_encode_init_with(dispatcher, session, width, height, io_pattern, 4, 0)
}

fn try_encode_init_with(
    dispatcher: &Dispatcher,
    session: *mut c_void,
    width: u16,
    height: u16,
    io_pattern: u16,
    target_usage: u16,
    low_power: u16,
) -> Result<i32, String> {
    let query: Symbol<'_, MfxEncodeQuery> =
        unsafe { dispatcher.library.get(b"MFXVideoENCODE_Query\0") }
            .map_err(|e| format!("MFXVideoENCODE_Query missing: {e:?}"))?;
    let init: Symbol<'_, MfxEncodeInit> =
        unsafe { dispatcher.library.get(b"MFXVideoENCODE_Init\0") }
            .map_err(|e| format!("MFXVideoENCODE_Init missing: {e:?}"))?;
    let mut params = build_video_params_with(width, height, io_pattern, target_usage, low_power);
    let mut query_out = build_video_params_with(width, height, io_pattern, target_usage, low_power);
    let q_status = unsafe { query(session, &mut params, &mut query_out) };
    eprintln!(
        "  MFXVideoENCODE_Query(io={io_pattern:#x},tu={target_usage},lp={low_power}) status={q_status}"
    );
    if q_status != MFX_ERR_NONE && q_status != MFX_WRN_IN_EXECUTION && q_status != -3 {
        eprintln!("    Query returned hard error {q_status}; skipping Init to avoid AV");
        return Ok(q_status);
    }
    let init_status = unsafe { init(session, &mut query_out) };
    Ok(init_status)
}

fn try_get_handle(
    dispatcher: &Dispatcher,
    session: *mut c_void,
    handle_type: u32,
) -> Result<(i32, *mut c_void), String> {
    let sym: Symbol<'_, MfxGetHandle> =
        unsafe { dispatcher.library.get(b"MFXVideoCORE_GetHandle\0") }
            .map_err(|e| format!("MFXVideoCORE_GetHandle missing: {e:?}"))?;
    let mut out: *mut c_void = ptr::null_mut();
    let status = unsafe { sym(session, handle_type, &mut out) };
    Ok((status, out))
}

fn try_encode_close(dispatcher: &Dispatcher, session: *mut c_void) {
    if let Ok(sym) = unsafe {
        dispatcher
            .library
            .get::<MfxEncodeClose>(b"MFXVideoENCODE_Close\0")
    } {
        unsafe {
            let _ = sym(session);
        }
    }
}

#[test]
fn qsv_tiger_lake_probe_struct_sizes() {
    eprintln!("=== STRUCT SIZES (Rust) ===");
    eprintln!("MfxFrameInfo = {}", std::mem::size_of::<MfxFrameInfo>());
    eprintln!("MfxInfoMfx = {}", std::mem::size_of::<MfxInfoMfx>());
    eprintln!("MfxVideoParam = {}", std::mem::size_of::<MfxVideoParam>());
    assert!(
        std::mem::size_of::<MfxFrameInfo>() == 68,
        "MfxFrameInfo size must be 68"
    );
    assert!(
        std::mem::size_of::<MfxInfoMfx>() >= 168,
        "MfxInfoMfx >= 168 (= sizeof mfxInfoVPP)"
    );
}

#[test]
fn qsv_tiger_lake_probe_experiment_1_preconditions() {
    let dispatcher = match Dispatcher::load() {
        Some(d) => d,
        None => {
            eprintln!("skip: libvpl.dll not loadable");
            return;
        }
    };
    eprintln!("=== EXPERIMENT 1: preconditions (QueryIMPL + QueryPlatform) ===");
    let (session_opt, log) = dispatcher.modern_session(MFX_ACCEL_MODE_VIA_D3D11);
    for line in &log {
        eprintln!("  {line}");
    }
    let (loader, session) = match session_opt {
        Some(s) => s,
        None => {
            eprintln!("EXP1 result: modern session not created; cannot probe preconditions");
            return;
        }
    };
    match try_query_impl(&dispatcher, session) {
        Ok((status, impl_val)) => {
            eprintln!("  MFXQueryIMPL: status={status} impl={impl_val:#x}");
        }
        Err(e) => eprintln!("  MFXQueryIMPL error: {e}"),
    }
    match try_query_platform(&dispatcher, session) {
        Ok((status, p)) => {
            eprintln!(
                "  MFXVideoCORE_QueryPlatform: status={status} code_name={} device_id={:#x} media_adapter_type={}",
                p.code_name, p.device_id, p.media_adapter_type
            );
        }
        Err(e) => eprintln!("  MFXVideoCORE_QueryPlatform error: {e}"),
    }
    dispatcher.close(session);
    dispatcher.unload(loader);
    eprintln!("EXP1 done");
}

#[test]
fn qsv_tiger_lake_probe_experiment_2_variant_accel_modes() {
    let dispatcher = match Dispatcher::load() {
        Some(d) => d,
        None => {
            eprintln!("skip: libvpl.dll not loadable");
            return;
        }
    };
    let (device, _ctx) = match try_create_intel_device() {
        Some(d) => d,
        None => {
            eprintln!("skip: no Intel D3D11 device");
            return;
        }
    };
    eprintln!("=== EXPERIMENT 2: SetHandle handle-type variants ===");
    let (session_opt, log) = dispatcher.modern_session(MFX_ACCEL_MODE_VIA_D3D11);
    for line in &log {
        eprintln!("  {line}");
    }
    let (loader, session) = match session_opt {
        Some(s) => s,
        None => {
            eprintln!("EXP2 result: modern session not created");
            return;
        }
    };
    let raw_device = device.as_raw();
    let exp2a = try_set_handle(&dispatcher, session, MFX_HANDLE_D3D11_DEVICE, raw_device);
    eprintln!(
        "  EXP2a: SetHandle(MFX_HANDLE_D3D11_DEVICE=3) -> {:?}",
        exp2a
    );
    let exp2b = try_set_handle(
        &dispatcher,
        session,
        MFX_HANDLE_D3D11_VIDEO_DEVICE_GUESS,
        raw_device,
    );
    eprintln!(
        "  EXP2b: SetHandle(MFX_HANDLE_D3D11_VIDEO_DEVICE_GUESS=11) -> {:?}",
        exp2b
    );
    dispatcher.close(session);
    dispatcher.unload(loader);
    eprintln!("EXP2 done");
    assert!(exp2a.is_ok(), "MFXVideoCORE_SetHandle symbol resolves");
}

#[test]
fn qsv_tiger_lake_probe_experiment_3_skip_set_handle() {
    let dispatcher = match Dispatcher::load() {
        Some(d) => d,
        None => {
            eprintln!("skip: libvpl.dll not loadable");
            return;
        }
    };
    let (device, _ctx) = match try_create_intel_device() {
        Some(d) => d,
        None => {
            eprintln!("skip: no Intel D3D11 device");
            return;
        }
    };
    eprintln!("=== EXPERIMENT 3: SetHandle with VIDEO_SUPPORT device + encoder init ===");
    let (session_opt, log) = dispatcher.modern_session(MFX_ACCEL_MODE_VIA_D3D11);
    for line in &log {
        eprintln!("  {line}");
    }
    let (loader, session) = match session_opt {
        Some(s) => s,
        None => {
            eprintln!("EXP3 result: modern session not created");
            return;
        }
    };
    match try_query_impl(&dispatcher, session) {
        Ok((status, impl_val)) => {
            eprintln!("  pre-set MFXQueryIMPL: status={status} impl={impl_val:#x}");
        }
        Err(e) => eprintln!("  MFXQueryIMPL error: {e}"),
    }
    let raw_device = device.as_raw();
    let set_status = try_set_handle(&dispatcher, session, MFX_HANDLE_D3D11_DEVICE, raw_device);
    eprintln!(
        "  EXP3-set: SetHandle(D3D11_DEVICE=3, VIDEO_SUPPORT+BGRA, multithread-protected) -> {:?}",
        set_status
    );
    match try_get_handle(&dispatcher, session, MFX_HANDLE_D3D11_DEVICE) {
        Ok((status, hdl)) => {
            eprintln!(
                "  post-set GetHandle(MFX_HANDLE_D3D11_DEVICE) -> status={status} handle={hdl:p}"
            );
        }
        Err(e) => eprintln!("  GetHandle error: {e}"),
    }
    let init_a = try_encode_init(
        &dispatcher,
        session,
        1280,
        720,
        MFX_IOPATTERN_IN_VIDEO_MEMORY,
    );
    eprintln!(
        "  EXP3a (IO_VIDEO_MEMORY, target_usage=4, low_power=0): init -> {:?}",
        init_a
    );
    let init_b = try_encode_init_with(
        &dispatcher,
        session,
        1280,
        720,
        MFX_IOPATTERN_IN_VIDEO_MEMORY,
        7,
        0x10,
    );
    eprintln!(
        "  EXP3b (IO_VIDEO_MEMORY, target_usage=7 best-speed, low_power=ON=0x10): init -> {:?}",
        init_b
    );
    let init_c = try_encode_init_with(
        &dispatcher,
        session,
        1280,
        720,
        MFX_IOPATTERN_IN_VIDEO_MEMORY,
        4,
        0x10,
    );
    eprintln!(
        "  EXP3c (IO_VIDEO_MEMORY, target_usage=4 balanced, low_power=ON=0x10): init -> {:?}",
        init_c
    );
    try_encode_close(&dispatcher, session);
    dispatcher.close(session);
    dispatcher.unload(loader);
    eprintln!("EXP3 done");
    assert!(init_a.is_ok(), "init_a symbol resolved");
    assert!(init_b.is_ok(), "init_b symbol resolved");
    assert!(init_c.is_ok(), "init_c symbol resolved");
}

#[test]
fn qsv_tiger_lake_probe_experiment_4_software_fallback() {
    let dispatcher = match Dispatcher::load() {
        Some(d) => d,
        None => {
            eprintln!("skip: libvpl.dll not loadable");
            return;
        }
    };
    eprintln!("=== EXPERIMENT 4: software-only (ACCEL_MODE_NA) session ===");
    let (session_opt, log) = dispatcher.modern_session(MFX_ACCEL_MODE_NA);
    for line in &log {
        eprintln!("  {line}");
    }
    let (loader, session) = match session_opt {
        Some(s) => s,
        None => {
            eprintln!("EXP4 result: software session not created");
            return;
        }
    };
    match try_query_impl(&dispatcher, session) {
        Ok((status, impl_val)) => {
            eprintln!("  MFXQueryIMPL: status={status} impl={impl_val:#x}");
        }
        Err(e) => eprintln!("  MFXQueryIMPL error: {e}"),
    }
    let init_status = try_encode_init(
        &dispatcher,
        session,
        1280,
        720,
        MFX_IOPATTERN_IN_SYSTEM_MEMORY,
    );
    eprintln!("  EXP4: MFXVideoENCODE_Init software -> {:?}", init_status);
    try_encode_close(&dispatcher, session);
    dispatcher.close(session);
    dispatcher.unload(loader);
    eprintln!("EXP4 done");
    assert!(
        init_status.is_ok(),
        "encode init symbol resolved in software"
    );
}
