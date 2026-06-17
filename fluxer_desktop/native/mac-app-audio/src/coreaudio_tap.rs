#![allow(non_camel_case_types, non_upper_case_globals)]

// SPDX-License-Identifier: AGPL-3.0-or-later

use core::ffi::{CStr, c_void};
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use objc2::AllocAnyThread;
use objc2::rc::Retained;
use objc2_core_audio::{
    AudioDeviceCreateIOProcID, AudioDeviceDestroyIOProcID, AudioDeviceIOProcID, AudioDeviceStart,
    AudioDeviceStop, AudioHardwareCreateAggregateDevice, AudioHardwareCreateProcessTap,
    AudioHardwareDestroyAggregateDevice, AudioHardwareDestroyProcessTap,
    AudioObjectGetPropertyData, AudioObjectID, AudioObjectPropertyAddress,
    AudioObjectSetPropertyData, CATapDescription, CATapMuteBehavior,
    kAudioHardwarePropertyTranslatePIDToProcessObject, kAudioObjectPropertyElementMain,
    kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject, kAudioObjectUnknown,
    kAudioTapPropertyDescription, kAudioTapPropertyFormat, kAudioTapPropertyUID,
};
use objc2_core_foundation::CFDictionary;
use objc2_foundation::{
    NSArray, NSBundle, NSMutableArray, NSMutableDictionary, NSNumber, NSObject, NSString, NSUUID,
};

use crate::audio_converter::{self as ac, AudioBufferList, AudioStreamBasicDescription};
use crate::foundation;
use crate::pcm_pool::{PcmFramePool, PooledPcmFrame};
use crate::process_tree;

pub type OSStatus = i32;

const NO_ERR: OSStatus = 0;
const K_AGGREGATE_DRIFT_COMPENSATION_MEDIUM_QUALITY: u32 = 0x40;

const MAX_RELATED_PROCESSES: usize = 512;
const MAX_CALLBACK_INPUT_FRAMES: u32 = 48_000;
const TARGET_SAMPLE_RATE: f64 = 48_000.0;
const TARGET_CHANNELS: u32 = 2;

const LATE_SPAWN_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

const HELPER_BUNDLE_SUFFIXES: &[&str] = &[
    ".helper",
    ".helper.Renderer",
    ".helper.GPU",
    ".helper.Plugin",
];

static AGGREGATE_UID_COUNTER: AtomicU64 = AtomicU64::new(1);
static DEBUG_COREAUDIO: AtomicBool = AtomicBool::new(false);
static DEBUG_CALLBACK_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CaptureScope {
    Process,
    System,
}

#[derive(Debug)]
pub enum CreateError {
    Unsupported,
    NoRelatedProcesses,
    NoProcessObjects,
    FoundationObjectFailed,
    CreateProcessTapFailed,
    ReadTapUidFailed,
    ReadTapFormatFailed,
    CreateAggregateDeviceFailed,
    CreateIOProcFailed,
    StartDeviceFailed,
}

pub type PcmCallback =
    unsafe extern "C" fn(ctx: *mut c_void, slot: *mut Option<PooledPcmFrame>, frames: u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TapDiagnostics {
    pub convert_failures: u64,
    pub dropped_buffers: u64,
}

pub struct Capture {
    pub tap_id: AudioObjectID,
    pub aggregate_device_id: AudioObjectID,
    pub io_proc_id: AudioDeviceIOProcID,
    pub input_format: AudioStreamBasicDescription,
    pub running: AtomicBool,
    pub pcm_callback: Option<PcmCallback>,
    pub pcm_callback_ctx: *mut c_void,
    pub pcm_pool: Option<PcmFramePool>,
    pub convert_failures: AtomicU64,
    pub dropped_buffers: AtomicU64,

    refresher: Option<RefresherHandle>,
}

unsafe impl Send for Capture {}
unsafe impl Sync for Capture {}

struct RefresherHandle {
    alive: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl RefresherHandle {
    fn shutdown(&mut self) {
        self.alive.store(false, Ordering::Release);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for RefresherHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn is_supported() -> bool {
    use objc2::runtime::AnyClass;
    AnyClass::get(c"CATapDescription").is_some()
}

fn getenv_set(name: &CStr) -> bool {
    unsafe { !libc::getenv(name.as_ptr()).is_null() }
}

fn collect_target_pids(pid: i32, include_process_tree: bool) -> Vec<i32> {
    if !include_process_tree {
        return vec![pid];
    }
    process_tree::collect_related_pids(pid, MAX_RELATED_PROCESSES)
}

fn collect_process_objects(pids: &[i32], skip_current_process: bool) -> Vec<AudioObjectID> {
    let self_pid = unsafe { libc::getpid() };
    let mut out = Vec::with_capacity(pids.len());
    for &pid in pids {
        if pid <= 0 {
            continue;
        }
        if skip_current_process && pid == self_pid {
            continue;
        }
        let object = translate_pid_to_process_object(pid);
        if object == kAudioObjectUnknown {
            continue;
        }
        if out.contains(&object) {
            continue;
        }
        out.push(object);
    }
    out
}

fn translate_pid_to_process_object(pid: i32) -> AudioObjectID {
    let mut out: AudioObjectID = kAudioObjectUnknown;
    let mut size: u32 = core::mem::size_of::<AudioObjectID>() as u32;
    let mut qualifier_pid = pid;
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyTranslatePIDToProcessObject,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };
    let status = unsafe {
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject as AudioObjectID,
            ptr::NonNull::from(&address),
            core::mem::size_of::<i32>() as u32,
            (&raw mut qualifier_pid) as *const _ as *const c_void,
            ptr::NonNull::from(&mut size),
            ptr::NonNull::from(&mut out).cast::<c_void>(),
        )
    };
    if status != NO_ERR {
        return kAudioObjectUnknown;
    }
    out
}

fn build_process_array(objects: &[AudioObjectID]) -> Retained<NSArray<NSNumber>> {
    let nums: Vec<Retained<NSNumber>> = objects.iter().map(|o| NSNumber::new_u32(*o)).collect();
    let refs: Vec<&NSNumber> = nums.iter().map(|n| n.as_ref()).collect();
    NSArray::from_slice(&refs)
}

fn create_tap_description(
    process_objects: &[AudioObjectID],
    pid: i32,
    scope: CaptureScope,
) -> Result<Retained<CATapDescription>, CreateError> {
    let process_array = build_process_array(process_objects);

    let alloc = CATapDescription::alloc();
    let description = match scope {
        CaptureScope::Process => unsafe {
            CATapDescription::initStereoMixdownOfProcesses(alloc, &process_array)
        },
        CaptureScope::System => unsafe {
            CATapDescription::initStereoGlobalTapButExcludeProcesses(alloc, &process_array)
        },
    };

    let excludes_by_bundle_id = if scope == CaptureScope::System {
        apply_main_bundle_id_excludes(&description)
    } else {
        false
    };
    if scope == CaptureScope::Process && process_objects.is_empty() {
        return Err(CreateError::NoProcessObjects);
    }
    if scope == CaptureScope::System && process_objects.is_empty() && !excludes_by_bundle_id {
        return Err(CreateError::NoProcessObjects);
    }

    let name = match scope {
        CaptureScope::Process => format!("Fluxer app audio tap pid {pid}"),
        CaptureScope::System => format!("Fluxer desktop audio tap excluding pid {pid}"),
    };
    unsafe {
        description.setUUID(&NSUUID::UUID());
        description.setName(&NSString::from_str(&name));
        description.setPrivate(true);
        description.setExclusive(scope == CaptureScope::System);
        description.setMuteBehavior(CATapMuteBehavior(0));
    }
    Ok(description)
}

fn apply_main_bundle_id_excludes(description: &CATapDescription) -> bool {
    use objc2::runtime::NSObjectProtocol;
    use objc2::sel;
    let obj: &NSObject = description.as_ref();
    if !obj.respondsToSelector(sel!(setBundleIDs:)) {
        return false;
    }
    let bundle_array = match build_main_bundle_id_array() {
        Some(a) => a,
        None => return false,
    };
    if bundle_array.count() == 0 {
        return false;
    }
    unsafe {
        description.setBundleIDs(&bundle_array);
        if obj.respondsToSelector(sel!(setProcessRestoreEnabled:)) {
            description.setProcessRestoreEnabled(true);
        }
    }
    true
}

fn build_main_bundle_id_array() -> Option<Retained<NSArray<NSString>>> {
    let base = copy_main_bundle_identifier()?;
    let base_str = base.to_string();
    if base_str.is_empty() || base_str.len() > 192 {
        return None;
    }
    let mut entries: Vec<Retained<NSString>> = Vec::with_capacity(1 + HELPER_BUNDLE_SUFFIXES.len());
    entries.push(base);
    for suffix in HELPER_BUNDLE_SUFFIXES {
        let helper = format!("{base_str}{suffix}");
        entries.push(NSString::from_str(&helper));
    }
    let refs: Vec<&NSString> = entries.iter().map(|s| s.as_ref()).collect();
    Some(NSArray::from_slice(&refs))
}

fn copy_main_bundle_identifier() -> Option<Retained<NSString>> {
    let bundle = NSBundle::mainBundle();
    bundle.bundleIdentifier()
}

fn copy_tap_uid(tap_id: AudioObjectID) -> Result<Retained<NSString>, CreateError> {
    let mut tap_uid: *const NSString = ptr::null();
    let mut size: u32 = core::mem::size_of::<*const NSString>() as u32;
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioTapPropertyUID,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };
    let status = unsafe {
        AudioObjectGetPropertyData(
            tap_id,
            ptr::NonNull::from(&address),
            0,
            ptr::null(),
            ptr::NonNull::from(&mut size),
            ptr::NonNull::from(&mut tap_uid).cast::<c_void>(),
        )
    };
    if status != NO_ERR || tap_uid.is_null() {
        return Err(CreateError::ReadTapUidFailed);
    }

    unsafe { Retained::from_raw(tap_uid as *mut NSString).ok_or(CreateError::ReadTapUidFailed) }
}

fn read_tap_format(tap_id: AudioObjectID) -> Result<AudioStreamBasicDescription, CreateError> {
    let mut format: AudioStreamBasicDescription = unsafe { core::mem::zeroed() };
    let mut size: u32 = core::mem::size_of::<AudioStreamBasicDescription>() as u32;
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioTapPropertyFormat,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };
    let status = unsafe {
        AudioObjectGetPropertyData(
            tap_id,
            ptr::NonNull::from(&address),
            0,
            ptr::null(),
            ptr::NonNull::from(&mut size),
            ptr::NonNull::from(&mut format).cast::<c_void>(),
        )
    };
    if status != NO_ERR {
        return Err(CreateError::ReadTapFormatFailed);
    }
    Ok(format)
}

fn create_aggregate_description(
    tap_uid: &NSString,
    pid: i32,
) -> Result<Retained<NSMutableDictionary<NSString, NSObject>>, CreateError> {
    let tap_dict: Retained<NSMutableDictionary<NSString, NSObject>> = NSMutableDictionary::new();
    foundation::dict_set_str_key(&tap_dict, c"uid", tap_uid.as_ref());
    let drift = NSNumber::new_bool(true);
    foundation::dict_set_str_key(&tap_dict, c"drift", drift.as_ref());
    let drift_quality = NSNumber::new_u32(K_AGGREGATE_DRIFT_COMPENSATION_MEDIUM_QUALITY);
    foundation::dict_set_str_key(&tap_dict, c"drift quality", drift_quality.as_ref());

    let dict_obj: &NSObject = tap_dict.as_ref();
    let tap_list: Retained<NSMutableArray<NSObject>> = NSMutableArray::arrayWithCapacity(1);
    tap_list.addObject(dict_obj);

    let aggregate_dict: Retained<NSMutableDictionary<NSString, NSObject>> =
        NSMutableDictionary::new();
    let counter = AGGREGATE_UID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let uid_string = format!("app.fluxer.mac-app-audio.tap.{pid}.{counter}");
    let uid = NSString::from_str(&uid_string);
    let name = NSString::from_str("Fluxer app audio capture");
    foundation::dict_set_str_key(&aggregate_dict, c"uid", uid.as_ref());
    foundation::dict_set_str_key(&aggregate_dict, c"name", name.as_ref());
    let priv_n = NSNumber::new_bool(true);
    foundation::dict_set_str_key(&aggregate_dict, c"private", priv_n.as_ref());
    foundation::dict_set_str_key(&aggregate_dict, c"taps", tap_list.as_ref());
    let auto = NSNumber::new_bool(true);
    foundation::dict_set_str_key(&aggregate_dict, c"tapautostart", auto.as_ref());
    Ok(aggregate_dict)
}

unsafe extern "C-unwind" fn io_proc(
    _in_device: AudioObjectID,
    _in_now: core::ptr::NonNull<objc2_core_audio_types::AudioTimeStamp>,
    in_input_data: core::ptr::NonNull<objc2_core_audio_types::AudioBufferList>,
    _in_input_time: core::ptr::NonNull<objc2_core_audio_types::AudioTimeStamp>,
    _out_output_data: core::ptr::NonNull<objc2_core_audio_types::AudioBufferList>,
    _in_output_time: core::ptr::NonNull<objc2_core_audio_types::AudioTimeStamp>,
    client_data: *mut c_void,
) -> OSStatus {
    if client_data.is_null() {
        return NO_ERR;
    }
    let self_ptr = client_data as *mut Capture;
    let capture = unsafe { &*self_ptr };
    if !capture.running.load(Ordering::Acquire) {
        return NO_ERR;
    }
    let local_abl = in_input_data.as_ptr() as *const AudioBufferList;
    let frames =
        match unsafe { ac::input_frame_count_for_buffer_list(&capture.input_format, local_abl) } {
            Ok(f) => f,
            Err(_) => {
                capture.convert_failures.fetch_add(1, Ordering::Relaxed);
                return NO_ERR;
            }
        };
    if frames == 0 {
        return NO_ERR;
    }
    if frames > MAX_CALLBACK_INPUT_FRAMES {
        capture.dropped_buffers.fetch_add(1, Ordering::Relaxed);
        return NO_ERR;
    }
    unsafe {
        convert_and_deliver(capture, local_abl, frames);
    }
    NO_ERR
}

unsafe fn convert_and_deliver(capture: &Capture, local_abl: *const AudioBufferList, frames: u32) {
    assert!(frames > 0);
    assert!(frames <= MAX_CALLBACK_INPUT_FRAMES);
    let Some(cb) = capture.pcm_callback else {
        return;
    };
    let Some(pool) = capture.pcm_pool.as_ref() else {
        return;
    };
    let needed = (ac::output_frame_capacity(
        frames,
        capture.input_format.m_sample_rate,
        TARGET_SAMPLE_RATE,
    ) as usize)
        * (TARGET_CHANNELS as usize);
    if needed > pool.samples_per_slot() as usize {
        capture.dropped_buffers.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let Some(mut slot) = pool.try_acquire() else {
        capture.dropped_buffers.fetch_add(1, Ordering::Relaxed);
        return;
    };
    let out_frames = unsafe {
        ac::convert_buffer_list_to_interleaved_f32(
            &capture.input_format,
            local_abl,
            frames,
            TARGET_SAMPLE_RATE,
            TARGET_CHANNELS,
            slot.unfilled_mut(),
        )
    };
    let n = match out_frames {
        Ok(0) => {
            capture.dropped_buffers.fetch_add(1, Ordering::Relaxed);
            return;
        }
        Err(_) => {
            capture.convert_failures.fetch_add(1, Ordering::Relaxed);
            return;
        }
        Ok(n) => n,
    };
    let filled = (n as usize) * (TARGET_CHANNELS as usize);
    assert!(filled <= slot.capacity());
    slot.set_filled_len(filled);
    let mut handoff = Some(slot);
    unsafe {
        cb(
            capture.pcm_callback_ctx,
            &mut handoff as *mut Option<PooledPcmFrame>,
            n,
        );
    }
}

impl Capture {
    pub fn create(
        pid: i32,
        include_process_tree: bool,
        scope: CaptureScope,
    ) -> Result<Box<Capture>, CreateError> {
        if !is_supported() {
            return Err(CreateError::Unsupported);
        }
        DEBUG_COREAUDIO.store(
            getenv_set(c"FLUXER_MAC_APP_AUDIO_DEBUG_COREAUDIO"),
            Ordering::Release,
        );
        DEBUG_CALLBACK_COUNT.store(0, Ordering::Release);

        let root_pid = if scope == CaptureScope::System {
            unsafe { libc::getpid() }
        } else {
            pid
        };
        let related_pids = collect_target_pids(root_pid, include_process_tree);
        if related_pids.is_empty() {
            return Err(CreateError::NoRelatedProcesses);
        }
        let process_objects =
            collect_process_objects(&related_pids, scope == CaptureScope::Process);

        let description = create_tap_description(&process_objects, root_pid, scope)?;
        let mut tap_id: AudioObjectID = kAudioObjectUnknown;
        let tap_status = unsafe { AudioHardwareCreateProcessTap(Some(&description), &mut tap_id) };

        if tap_status != NO_ERR || tap_id == kAudioObjectUnknown {
            return Err(CreateError::CreateProcessTapFailed);
        }

        let tap_uid = match copy_tap_uid(tap_id) {
            Ok(u) => u,
            Err(e) => {
                unsafe {
                    let _ = AudioHardwareDestroyProcessTap(tap_id);
                }
                return Err(e);
            }
        };
        let input_format = match read_tap_format(tap_id) {
            Ok(f) => f,
            Err(e) => {
                unsafe {
                    let _ = AudioHardwareDestroyProcessTap(tap_id);
                }
                return Err(e);
            }
        };
        let aggregate_description = match create_aggregate_description(&tap_uid, pid) {
            Ok(d) => d,
            Err(e) => {
                unsafe {
                    let _ = AudioHardwareDestroyProcessTap(tap_id);
                }
                return Err(e);
            }
        };
        let mut aggregate_device_id: AudioObjectID = kAudioObjectUnknown;
        let aggregate_status = unsafe {
            let dict_ref: &CFDictionary =
                &*(&*aggregate_description as *const _ as *const CFDictionary);
            AudioHardwareCreateAggregateDevice(
                dict_ref,
                ptr::NonNull::from(&mut aggregate_device_id),
            )
        };
        if aggregate_status != NO_ERR || aggregate_device_id == kAudioObjectUnknown {
            unsafe {
                let _ = AudioHardwareDestroyProcessTap(tap_id);
            }
            return Err(CreateError::CreateAggregateDeviceFailed);
        }

        let mut capture = Box::new(Capture {
            tap_id,
            aggregate_device_id,
            io_proc_id: None,
            input_format,
            running: AtomicBool::new(false),
            pcm_callback: None,
            pcm_callback_ctx: ptr::null_mut(),
            pcm_pool: None,
            convert_failures: AtomicU64::new(0),
            dropped_buffers: AtomicU64::new(0),
            refresher: None,
        });
        let mut io_proc_id: AudioDeviceIOProcID = None;
        let io_status = unsafe {
            AudioDeviceCreateIOProcID(
                aggregate_device_id,
                Some(io_proc),
                &mut *capture as *mut Capture as *mut c_void,
                ptr::NonNull::from(&mut io_proc_id),
            )
        };
        if io_status != NO_ERR || io_proc_id.is_none() {
            unsafe {
                let _ = AudioHardwareDestroyAggregateDevice(aggregate_device_id);
                let _ = AudioHardwareDestroyProcessTap(tap_id);
            }
            return Err(CreateError::CreateIOProcFailed);
        }
        capture.io_proc_id = io_proc_id;

        if scope == CaptureScope::Process && include_process_tree {
            capture.refresher =
                spawn_late_spawn_refresher(tap_id, root_pid, scope, process_objects.to_vec());
        }
        Ok(capture)
    }

    pub fn set_pcm_callback(&mut self, cb: PcmCallback, ctx: *mut c_void) {
        self.pcm_callback = Some(cb);
        self.pcm_callback_ctx = ctx;
    }

    pub fn set_pcm_pool(&mut self, pool: PcmFramePool) {
        assert!(pool.capacity() > 0);
        assert!(pool.samples_per_slot() > 0);
        self.pcm_pool = Some(pool);
    }

    pub fn diagnostics(&self) -> TapDiagnostics {
        let diagnostics = TapDiagnostics {
            convert_failures: self.convert_failures.load(Ordering::Relaxed),
            dropped_buffers: self.dropped_buffers.load(Ordering::Relaxed),
        };
        assert!(diagnostics.convert_failures <= u64::MAX / 2);
        assert!(diagnostics.dropped_buffers <= u64::MAX / 2);
        diagnostics
    }

    pub fn start(&mut self) -> Result<(), CreateError> {
        if self.io_proc_id.is_none() || self.aggregate_device_id == kAudioObjectUnknown {
            return Err(CreateError::StartDeviceFailed);
        }
        self.running.store(true, Ordering::Release);
        let status = unsafe { AudioDeviceStart(self.aggregate_device_id, self.io_proc_id) };
        if status != NO_ERR {
            self.running.store(false, Ordering::Release);
            return Err(CreateError::StartDeviceFailed);
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Release);
        if self.aggregate_device_id != kAudioObjectUnknown && self.io_proc_id.is_some() {
            unsafe {
                let _ = AudioDeviceStop(self.aggregate_device_id, self.io_proc_id);
                let _ = AudioDeviceDestroyIOProcID(self.aggregate_device_id, self.io_proc_id);
            }
            self.io_proc_id = None;
        }
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        if let Some(mut r) = self.refresher.take() {
            r.shutdown();
        }
        self.stop();
        if self.aggregate_device_id != kAudioObjectUnknown {
            unsafe {
                let _ = AudioHardwareDestroyAggregateDevice(self.aggregate_device_id);
            }
            self.aggregate_device_id = kAudioObjectUnknown;
        }
        if self.tap_id != kAudioObjectUnknown {
            unsafe {
                let _ = AudioHardwareDestroyProcessTap(self.tap_id);
            }
            self.tap_id = kAudioObjectUnknown;
        }
    }
}

fn apply_process_objects_to_tap(
    tap_id: AudioObjectID,
    objects: &[AudioObjectID],
    root_pid: i32,
    scope: CaptureScope,
) -> OSStatus {
    if tap_id == kAudioObjectUnknown {
        return -1;
    }
    let description = match create_tap_description(objects, root_pid, scope) {
        Ok(d) => d,
        Err(_) => return -1,
    };
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioTapPropertyDescription,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };

    let desc_ptr: *const CATapDescription = &*description;
    let mut desc_holder: *const CATapDescription = desc_ptr;
    unsafe {
        AudioObjectSetPropertyData(
            tap_id,
            ptr::NonNull::from(&address),
            0,
            ptr::null(),
            core::mem::size_of::<*const CATapDescription>() as u32,
            ptr::NonNull::from(&mut desc_holder).cast::<c_void>(),
        )
    }
}

fn spawn_late_spawn_refresher(
    tap_id: AudioObjectID,
    root_pid: i32,
    scope: CaptureScope,
    initial_objects: Vec<AudioObjectID>,
) -> Option<RefresherHandle> {
    let alive = Arc::new(AtomicBool::new(true));
    let alive_for_thread = alive.clone();
    let thread = thread::Builder::new()
        .name("fluxer-mac-tap-refresh".into())
        .spawn(move || {
            run_late_spawn_refresher(tap_id, root_pid, scope, initial_objects, alive_for_thread);
        })
        .ok()?;
    Some(RefresherHandle {
        alive,
        thread: Some(thread),
    })
}

fn run_late_spawn_refresher(
    tap_id: AudioObjectID,
    root_pid: i32,
    scope: CaptureScope,
    initial_objects: Vec<AudioObjectID>,
    alive: Arc<AtomicBool>,
) {
    let mut previous: Vec<AudioObjectID> = initial_objects;
    previous.sort_unstable();
    previous.dedup();

    while alive.load(Ordering::Acquire) {
        thread::sleep(LATE_SPAWN_REFRESH_INTERVAL);
        if !alive.load(Ordering::Acquire) {
            break;
        }
        let related = process_tree::collect_related_pids(root_pid, MAX_RELATED_PROCESSES);
        if related.is_empty() {
            continue;
        }
        let mut current = collect_process_objects(&related, scope == CaptureScope::Process);
        current.sort_unstable();
        current.dedup();
        if current == previous {
            continue;
        }

        if current.is_empty() {
            continue;
        }
        let status = apply_process_objects_to_tap(tap_id, &current, root_pid, scope);
        if status == NO_ERR {
            previous = current;
        }
    }
}

pub fn coreaudio_error_message(e: &CreateError) -> &'static str {
    match e {
        CreateError::Unsupported => "CoreAudio process taps unavailable",
        CreateError::NoRelatedProcesses => "No related process for selected app",
        CreateError::NoProcessObjects => "No CoreAudio process object for selected app",
        CreateError::FoundationObjectFailed => "CoreAudio tap configuration failed",
        CreateError::CreateProcessTapFailed => "CoreAudio process tap creation failed",
        CreateError::ReadTapUidFailed => "CoreAudio process tap UID lookup failed",
        CreateError::ReadTapFormatFailed => "CoreAudio process tap format lookup failed",
        CreateError::CreateAggregateDeviceFailed => {
            "CoreAudio process tap aggregate device creation failed"
        }
        CreateError::CreateIOProcFailed => "CoreAudio process tap IOProc creation failed",
        CreateError::StartDeviceFailed => "CoreAudio process tap start failed",
    }
}

#[cfg(test)]
mod io_proc_diagnostics_tests {
    use super::*;
    use crate::audio_converter::AudioBuffer;
    use core::mem::size_of;
    use core::ptr::NonNull;
    use core::sync::atomic::AtomicU32 as TestAtomicU32;
    use objc2_core_audio_types::AudioTimeStamp;

    fn make_idle_capture(input_format: AudioStreamBasicDescription) -> Box<Capture> {
        Box::new(Capture {
            tap_id: kAudioObjectUnknown,
            aggregate_device_id: kAudioObjectUnknown,
            io_proc_id: None,
            input_format,
            running: AtomicBool::new(true),
            pcm_callback: None,
            pcm_callback_ctx: ptr::null_mut(),
            pcm_pool: None,
            convert_failures: AtomicU64::new(0),
            dropped_buffers: AtomicU64::new(0),
            refresher: None,
        })
    }

    fn make_test_pool() -> PcmFramePool {
        PcmFramePool::new(2, 8_192).expect("test pool builds")
    }

    fn call_io_proc(capture: &Capture, abl: &mut AudioBufferList) -> OSStatus {
        let mut ts_now: AudioTimeStamp = unsafe { core::mem::zeroed() };
        let mut ts_input: AudioTimeStamp = unsafe { core::mem::zeroed() };
        let mut ts_output: AudioTimeStamp = unsafe { core::mem::zeroed() };
        let mut out_abl = AudioBufferList {
            m_number_buffers: 0,
            buffers: [AudioBuffer {
                m_number_channels: 0,
                m_data_byte_size: 0,
                m_data: ptr::null_mut(),
            }],
        };
        unsafe {
            io_proc(
                0,
                NonNull::from(&mut ts_now),
                NonNull::new(
                    abl as *mut AudioBufferList as *mut objc2_core_audio_types::AudioBufferList,
                )
                .expect("input abl non-null"),
                NonNull::from(&mut ts_input),
                NonNull::new(
                    &mut out_abl as *mut AudioBufferList
                        as *mut objc2_core_audio_types::AudioBufferList,
                )
                .expect("output abl non-null"),
                NonNull::from(&mut ts_output),
                capture as *const Capture as *mut c_void,
            )
        }
    }

    fn interleaved_abl(data: &mut [f32]) -> AudioBufferList {
        AudioBufferList {
            m_number_buffers: 1,
            buffers: [AudioBuffer {
                m_number_channels: TARGET_CHANNELS,
                m_data_byte_size: (data.len() * size_of::<f32>()) as u32,
                m_data: data.as_mut_ptr() as *mut c_void,
            }],
        }
    }

    #[test]
    fn convert_failures_counted_for_unsupported_format() {
        let capture = make_idle_capture(AudioStreamBasicDescription::default());
        let mut data = [0.0_f32; 8];
        let mut abl = interleaved_abl(&mut data);
        let status = call_io_proc(&capture, &mut abl);
        assert_eq!(status, NO_ERR);
        let diagnostics = capture.diagnostics();
        assert_eq!(diagnostics.convert_failures, 1);
        assert_eq!(diagnostics.dropped_buffers, 0);
    }

    unsafe extern "C" fn noop_pcm_cb(
        _ctx: *mut c_void,
        _slot: *mut Option<PooledPcmFrame>,
        _frames: u32,
    ) {
    }

    #[test]
    fn dropped_buffers_counted_when_pool_exhausted() {
        let mut capture = make_idle_capture(ac::build_input_asbd(48_000.0, TARGET_CHANNELS, false));
        let pool = make_test_pool();
        capture.set_pcm_callback(noop_pcm_cb, ptr::null_mut());
        capture.set_pcm_pool(pool.clone());
        let mut held = Vec::with_capacity(pool.capacity() as usize);
        for _ in 0..pool.capacity() {
            held.push(pool.try_acquire().expect("slot in capacity"));
        }
        let mut data = [0.25_f32; 96];
        let mut abl = interleaved_abl(&mut data);
        let status = call_io_proc(&capture, &mut abl);
        drop(held);
        assert_eq!(status, NO_ERR);
        let diagnostics = capture.diagnostics();
        assert_eq!(diagnostics.dropped_buffers, 1);
        assert_eq!(diagnostics.convert_failures, 0);
    }

    #[test]
    fn dropped_buffers_counted_when_slot_too_small_for_conversion() {
        let mut capture = make_idle_capture(ac::build_input_asbd(48_000.0, TARGET_CHANNELS, false));
        let pool = PcmFramePool::new(2, 16).expect("tiny pool builds");
        capture.set_pcm_callback(noop_pcm_cb, ptr::null_mut());
        capture.set_pcm_pool(pool);
        let mut data = [0.25_f32; 96];
        let mut abl = interleaved_abl(&mut data);
        let status = call_io_proc(&capture, &mut abl);
        assert_eq!(status, NO_ERR);
        let diagnostics = capture.diagnostics();
        assert_eq!(diagnostics.dropped_buffers, 1);
        assert_eq!(diagnostics.convert_failures, 0);
    }

    #[test]
    fn dropped_buffers_counted_for_oversize_input() {
        let capture = make_idle_capture(ac::build_input_asbd(48_000.0, TARGET_CHANNELS, false));
        let mut data = [0.0_f32; 8];
        let bytes_per_frame = (size_of::<f32>() as u32) * TARGET_CHANNELS;
        let mut abl = interleaved_abl(&mut data);
        abl.buffers[0].m_data_byte_size = (MAX_CALLBACK_INPUT_FRAMES + 1) * bytes_per_frame;
        let status = call_io_proc(&capture, &mut abl);
        assert_eq!(status, NO_ERR);
        let diagnostics = capture.diagnostics();
        assert_eq!(diagnostics.dropped_buffers, 1);
        assert_eq!(diagnostics.convert_failures, 0);
    }

    #[test]
    fn stopped_capture_counts_nothing() {
        let capture = make_idle_capture(AudioStreamBasicDescription::default());
        capture.running.store(false, Ordering::Release);
        let mut data = [0.0_f32; 8];
        let mut abl = interleaved_abl(&mut data);
        let status = call_io_proc(&capture, &mut abl);
        assert_eq!(status, NO_ERR);
        let diagnostics = capture.diagnostics();
        assert_eq!(diagnostics.convert_failures, 0);
        assert_eq!(diagnostics.dropped_buffers, 0);
    }

    static OBSERVED_FRAMES: TestAtomicU32 = TestAtomicU32::new(0);
    static OBSERVED_FILLED: TestAtomicU32 = TestAtomicU32::new(0);

    unsafe extern "C" fn observing_pcm_cb(
        _ctx: *mut c_void,
        slot: *mut Option<PooledPcmFrame>,
        frames: u32,
    ) {
        assert!(!slot.is_null());
        let frame = unsafe { (*slot).take() }.expect("slot delivered");
        assert_eq!(
            frame.filled_len(),
            (frames as usize) * (TARGET_CHANNELS as usize)
        );
        assert_eq!(frame.data_slice()[0], 0.25);
        OBSERVED_FILLED.store(frame.filled_len() as u32, Ordering::Release);
        OBSERVED_FRAMES.store(frames, Ordering::Release);
    }

    #[test]
    fn successful_conversion_counts_nothing_and_invokes_callback() {
        OBSERVED_FRAMES.store(0, Ordering::Release);
        OBSERVED_FILLED.store(0, Ordering::Release);
        let mut capture = make_idle_capture(ac::build_input_asbd(48_000.0, TARGET_CHANNELS, false));
        let pool = make_test_pool();
        capture.set_pcm_callback(observing_pcm_cb, ptr::null_mut());
        capture.set_pcm_pool(pool.clone());
        let mut data = [0.25_f32; 96];
        let mut abl = interleaved_abl(&mut data);
        let status = call_io_proc(&capture, &mut abl);
        assert_eq!(status, NO_ERR);
        let diagnostics = capture.diagnostics();
        assert_eq!(diagnostics.convert_failures, 0);
        assert_eq!(diagnostics.dropped_buffers, 0);
        assert_eq!(OBSERVED_FRAMES.load(Ordering::Acquire), 48);
        assert_eq!(OBSERVED_FILLED.load(Ordering::Acquire), 96);
        assert_eq!(pool.stats().in_flight, 0);
        assert_eq!(pool.stats().released, 1);
    }

    #[test]
    fn untaken_slot_returns_to_pool() {
        let mut capture = make_idle_capture(ac::build_input_asbd(48_000.0, TARGET_CHANNELS, false));
        let pool = make_test_pool();
        capture.set_pcm_callback(noop_pcm_cb, ptr::null_mut());
        capture.set_pcm_pool(pool.clone());
        let mut data = [0.25_f32; 96];
        let mut abl = interleaved_abl(&mut data);
        let status = call_io_proc(&capture, &mut abl);
        assert_eq!(status, NO_ERR);
        assert_eq!(pool.stats().in_flight, 0);
        assert_eq!(pool.stats().acquired, 1);
        assert_eq!(pool.stats().released, 1);
    }
}
