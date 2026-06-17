// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    collections::{HashMap, HashSet},
    os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use evdev::{Device, EventType, KeyCode};
use napi::{
    Env, Status,
    bindgen_prelude::{Function, Object, Result, ToNapiValue},
    sys,
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode, UnknownReturnValue},
};
use napi_derive::napi;
use nix::sys::eventfd::{EfdFlags, EventFd};
use polling::{Event as PollEvent, Events, Poller};

use crate::keymap;

const EXIT_KEY: usize = 0;
const MONITOR_KEY: usize = 1;
const DEVICE_KEY_BASE: usize = 2;
const DEV_INPUT_PREFIX: &str = "/dev/input/event";
const EVENT_QUEUE_LIMIT: usize = 1024;

fn poll_key_for_device_fd(fd: RawFd) -> Option<usize> {
    usize::try_from(fd).ok()?.checked_add(DEVICE_KEY_BASE)
}

fn device_fd_from_poll_key(key: usize) -> Option<RawFd> {
    let fd = key.checked_sub(DEVICE_KEY_BASE)?;
    RawFd::try_from(fd).ok()
}

#[derive(Debug)]
pub enum NativeEvent {
    Key {
        kind: KeyKind,
        keycode: u16,
        key_name: &'static str,
        ctrl: bool,
        alt: bool,
        shift: bool,
        meta: bool,
    },
    Mouse {
        kind: MouseKind,
        button: u8,
        ctrl: bool,
        alt: bool,
        shift: bool,
        meta: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum KeyKind {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseKind {
    Down,
    Up,
}

impl ToNapiValue for NativeEvent {
    unsafe fn to_napi_value(raw_env: sys::napi_env, event: Self) -> Result<sys::napi_value> {
        let env = Env::from_raw(raw_env);
        let mut object = Object::new(&env)?;
        match event {
            Self::Key {
                kind,
                keycode,
                key_name,
                ctrl,
                alt,
                shift,
                meta,
            } => {
                object.set(
                    "type",
                    match kind {
                        KeyKind::Down => "keydown",
                        KeyKind::Up => "keyup",
                    },
                )?;
                object.set("keycode", u32::from(keycode))?;
                object.set("keyName", key_name)?;
                object.set("ctrlKey", ctrl)?;
                object.set("altKey", alt)?;
                object.set("shiftKey", shift)?;
                object.set("metaKey", meta)?;
            }
            Self::Mouse {
                kind,
                button,
                ctrl,
                alt,
                shift,
                meta,
            } => {
                object.set(
                    "type",
                    match kind {
                        MouseKind::Down => "mousedown",
                        MouseKind::Up => "mouseup",
                    },
                )?;
                object.set("button", u32::from(button))?;
                object.set("ctrlKey", ctrl)?;
                object.set("altKey", alt)?;
                object.set("shiftKey", shift)?;
                object.set("metaKey", meta)?;
            }
        }
        unsafe { <Object<'_> as ToNapiValue>::to_napi_value(raw_env, object) }
    }
}

type EventTsfn = Arc<
    ThreadsafeFunction<
        NativeEvent,
        UnknownReturnValue,
        NativeEvent,
        Status,
        false,
        true,
        EVENT_QUEUE_LIMIT,
    >,
>;

struct ExitFd {
    fd: EventFd,
}

impl ExitFd {
    fn new() -> std::io::Result<Self> {
        let fd = EventFd::from_value_and_flags(0, EfdFlags::EFD_CLOEXEC | EfdFlags::EFD_NONBLOCK)
            .map_err(std::io::Error::from)?;
        Ok(Self { fd })
    }

    fn signal(&self) {
        let _ = self.fd.write(1);
    }

    fn drain(&self) {
        let _ = self.fd.read();
    }

    fn as_borrowed(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

fn resolve_seat() -> String {
    std::env::var("XDG_SEAT")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "seat0".to_string())
}

fn read_seat_from_device(device: &udev::Device) -> Option<String> {
    if let Some(value) = device.property_value("ID_SEAT") {
        return Some(value.to_string_lossy().into_owned());
    }
    let mut parent = device.parent();
    while let Some(p) = parent {
        if let Some(value) = p.property_value("ID_SEAT") {
            return Some(value.to_string_lossy().into_owned());
        }
        parent = p.parent();
    }
    Some("seat0".to_string())
}

fn lookup_input_seat(sysname: &str) -> Option<String> {
    let mut enumerator = udev::Enumerator::new().ok()?;
    enumerator.match_subsystem("input").ok()?;
    enumerator.match_sysname(sysname).ok()?;
    let device = enumerator.scan_devices().ok()?.next()?;
    read_seat_from_device(&device)
}

struct OpenedDevice {
    device: Device,
    path: PathBuf,
}

struct Reader {
    poller: Arc<Poller>,
    exit_fd: Arc<ExitFd>,
    udev_handle_available: bool,
    seat: String,
    devices: HashMap<RawFd, OpenedDevice>,
    held_keys: HashSet<u16>,
    callback: EventTsfn,
    stop: Arc<AtomicBool>,
}

impl Reader {
    fn new(
        callback: EventTsfn,
        stop: Arc<AtomicBool>,
        exit_fd: Arc<ExitFd>,
    ) -> std::io::Result<Self> {
        let poller = Arc::new(Poller::new()?);
        unsafe {
            poller.add(&exit_fd.as_borrowed(), PollEvent::readable(EXIT_KEY))?;
        }
        let seat = resolve_seat();
        let udev_handle_available = udev::Enumerator::new().is_ok();

        Ok(Self {
            poller,
            exit_fd,
            udev_handle_available,
            seat,
            devices: HashMap::new(),
            held_keys: HashSet::new(),
            callback,
            stop,
        })
    }

    fn try_attach_monitor(&self) -> Option<udev::MonitorSocket> {
        let socket = udev::MonitorBuilder::new()
            .and_then(|b| b.match_subsystem("input"))
            .and_then(|b| b.listen())
            .ok()?;
        let monitor_fd = socket.as_raw_fd();
        let borrowed = unsafe { BorrowedFd::borrow_raw(monitor_fd) };
        if unsafe { self.poller.add(&borrowed, PollEvent::readable(MONITOR_KEY)) }.is_ok() {
            Some(socket)
        } else {
            None
        }
    }

    fn device_is_on_our_seat(&self, sysname: &str) -> bool {
        if !self.udev_handle_available || self.seat.is_empty() {
            return true;
        }
        match lookup_input_seat(sysname) {
            Some(found) => found == self.seat,
            None => true,
        }
    }

    fn open_all_devices(&mut self) {
        let entries = match std::fs::read_dir("/dev/input") {
            Ok(entries) => entries,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = match name.to_str() {
                Some(s) => s,
                None => continue,
            };
            if !name_str.starts_with("event") {
                continue;
            }
            if !self.device_is_on_our_seat(name_str) {
                continue;
            }
            let path = entry.path();
            let _ = self.open_device(&path);
        }
    }

    fn open_device(&mut self, path: &Path) -> std::io::Result<()> {
        if self
            .devices
            .values()
            .any(|opened| opened.path.as_path() == path)
        {
            return Ok(());
        }
        let device = Device::open(path)?;
        if !device_has_routable_input(&device) {
            return Ok(());
        }
        let _ = device.set_nonblocking(true);
        let fd = device.as_raw_fd();
        let poll_key = poll_key_for_device_fd(fd).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "input device fd cannot be represented as a poll key",
            )
        })?;
        unsafe {
            let borrowed = BorrowedFd::borrow_raw(fd);
            self.poller.add(&borrowed, PollEvent::readable(poll_key))?;
        }
        self.devices.insert(
            fd,
            OpenedDevice {
                device,
                path: path.to_path_buf(),
            },
        );
        Ok(())
    }

    fn close_device_by_path(&mut self, path: &Path) {
        let fd = self
            .devices
            .iter()
            .find(|(_, opened)| opened.path.as_path() == path)
            .map(|(fd, _)| *fd);
        if let Some(fd) = fd {
            self.close_device_by_fd(fd);
        }
    }

    fn close_device_by_fd(&mut self, fd: RawFd) {
        if let Some(opened) = self.devices.remove(&fd) {
            let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
            let _ = self.poller.delete(borrowed);
            drop(opened);
        }
    }

    fn run(&mut self) {
        let monitor = self.try_attach_monitor();
        let mut events = Events::new();
        loop {
            if self.stop.load(Ordering::Acquire) {
                break;
            }
            events.clear();
            if self.poller.wait(&mut events, None).is_err() {
                break;
            }
            let mut device_fds_to_drain: Vec<RawFd> = Vec::new();
            let mut drain_monitor = false;
            let mut got_exit = false;
            for event in events.iter() {
                match event.key {
                    EXIT_KEY => got_exit = true,
                    MONITOR_KEY => drain_monitor = true,
                    fd_key => {
                        if let Some(fd) = device_fd_from_poll_key(fd_key) {
                            device_fds_to_drain.push(fd);
                        }
                    }
                }
            }
            if got_exit {
                self.exit_fd.drain();
                break;
            }

            self.rearm(monitor.as_ref(), drain_monitor, &device_fds_to_drain);

            if drain_monitor && let Some(monitor) = monitor.as_ref() {
                self.drain_monitor(monitor);
            }
            for fd in device_fds_to_drain {
                self.drain_device(fd);
            }
        }

        if let Some(monitor) = monitor.as_ref() {
            let borrowed = unsafe { BorrowedFd::borrow_raw(monitor.as_raw_fd()) };
            let _ = self.poller.delete(borrowed);
        }
    }

    fn rearm(
        &self,
        monitor: Option<&udev::MonitorSocket>,
        drain_monitor: bool,
        device_fds: &[RawFd],
    ) {
        let _ = self
            .poller
            .modify(self.exit_fd.as_borrowed(), PollEvent::readable(EXIT_KEY));
        if drain_monitor && let Some(monitor) = monitor {
            let borrowed = unsafe { BorrowedFd::borrow_raw(monitor.as_raw_fd()) };
            let _ = self
                .poller
                .modify(borrowed, PollEvent::readable(MONITOR_KEY));
        }
        for fd in device_fds {
            if self.devices.contains_key(fd) {
                let Some(poll_key) = poll_key_for_device_fd(*fd) else {
                    continue;
                };
                let borrowed = unsafe { BorrowedFd::borrow_raw(*fd) };
                let _ = self.poller.modify(borrowed, PollEvent::readable(poll_key));
            }
        }
    }

    fn drain_monitor(&mut self, monitor: &udev::MonitorSocket) {
        let mut pending: Vec<(String, PathBuf, Option<String>)> = Vec::new();
        for event in monitor.iter() {
            let action = match event.action() {
                Some(a) => a.to_string_lossy().into_owned(),
                None => continue,
            };
            let devnode = match event.devnode() {
                Some(p) => p.to_path_buf(),
                None => continue,
            };
            let subsystem = event.subsystem().map(|s| s.to_string_lossy().into_owned());
            if subsystem.as_deref() != Some("input") {
                continue;
            }
            let devnode_str = devnode.to_string_lossy().into_owned();
            if !devnode_str.starts_with(DEV_INPUT_PREFIX) {
                continue;
            }
            let seat = read_seat_from_device(&event);
            pending.push((action, devnode, seat));
        }
        for (action, devnode, seat) in pending {
            match action.as_str() {
                "add" => {
                    if !self.seat.is_empty()
                        && let Some(seat) = seat.as_deref()
                        && seat != self.seat
                    {
                        continue;
                    }
                    let _ = self.open_device(&devnode);
                }
                "remove" => self.close_device_by_path(&devnode),
                _ => {}
            }
        }
    }

    fn drain_device(&mut self, fd: RawFd) {
        let mut decoded: Vec<(u16, i32)> = Vec::new();
        let mut device_dead = false;
        {
            let Some(opened) = self.devices.get_mut(&fd) else {
                return;
            };
            loop {
                let fetch_result = opened.device.fetch_events();
                match fetch_result {
                    Ok(events) => {
                        for ev in events {
                            if ev.event_type() != EventType::KEY {
                                continue;
                            }
                            decoded.push((ev.code(), ev.value()));
                        }
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => {
                        device_dead = true;
                        break;
                    }
                }
            }
        }
        if device_dead {
            self.close_device_by_fd(fd);
            return;
        }
        for (code, value) in decoded {
            if value == 2 {
                continue;
            }
            self.translate_event(code, value == 1);
        }
    }

    fn translate_event(&mut self, code: u16, is_press: bool) {
        if let Some(button) = keymap::evdev_button_to_browser_button(code) {
            let event = NativeEvent::Mouse {
                kind: if is_press {
                    MouseKind::Down
                } else {
                    MouseKind::Up
                },
                button,
                ctrl: self.modifier_state_ctrl(),
                alt: self.modifier_state_alt(),
                shift: self.modifier_state_shift(),
                meta: self.modifier_state_meta(),
            };
            self.dispatch(event);
            return;
        }
        let key_name = match keymap::keycode_to_name(code) {
            Some(name) => name,
            None => return,
        };
        if is_press {
            self.held_keys.insert(code);
        } else {
            self.held_keys.remove(&code);
        }
        let event = NativeEvent::Key {
            kind: if is_press { KeyKind::Down } else { KeyKind::Up },
            keycode: code,
            key_name,
            ctrl: self.modifier_state_ctrl(),
            alt: self.modifier_state_alt(),
            shift: self.modifier_state_shift(),
            meta: self.modifier_state_meta(),
        };
        self.dispatch(event);
    }

    fn dispatch(&self, event: NativeEvent) {
        let status = self
            .callback
            .call(event, ThreadsafeFunctionCallMode::NonBlocking);
        if status == Status::Closing {
            self.stop.store(true, Ordering::Release);
        }
    }

    fn modifier_state_ctrl(&self) -> bool {
        self.held_keys.contains(&keymap::LEFT_CTRL) || self.held_keys.contains(&keymap::RIGHT_CTRL)
    }
    fn modifier_state_alt(&self) -> bool {
        self.held_keys.contains(&keymap::LEFT_ALT) || self.held_keys.contains(&keymap::RIGHT_ALT)
    }
    fn modifier_state_shift(&self) -> bool {
        self.held_keys.contains(&keymap::LEFT_SHIFT)
            || self.held_keys.contains(&keymap::RIGHT_SHIFT)
    }
    fn modifier_state_meta(&self) -> bool {
        self.held_keys.contains(&keymap::LEFT_META) || self.held_keys.contains(&keymap::RIGHT_META)
    }
}

fn is_routable_key_code(code: u16) -> bool {
    keymap::keycode_to_name(code).is_some()
        || keymap::evdev_button_to_browser_button(code).is_some()
}

fn device_has_routable_input(device: &Device) -> bool {
    device.supported_keys().is_some_and(|keys| {
        keys.iter()
            .any(|key: KeyCode| is_routable_key_code(key.code()))
    })
}

impl Drop for Reader {
    fn drop(&mut self) {
        let fds: Vec<RawFd> = self.devices.keys().copied().collect();
        for fd in fds {
            let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
            let _ = self.poller.delete(borrowed);
        }

        let _ = self.poller.delete(self.exit_fd.as_borrowed());
    }
}

struct HookInner {
    stop: Option<Arc<AtomicBool>>,
    exit_fd: Option<Arc<ExitFd>>,
    thread: Option<JoinHandle<()>>,
    opened: bool,
}

impl HookInner {
    fn stop_and_join(&mut self) {
        if let Some(stop) = &self.stop {
            stop.store(true, Ordering::Release);
        }
        if let Some(exit_fd) = &self.exit_fd {
            exit_fd.signal();
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        self.stop = None;
        self.exit_fd = None;
        self.opened = false;
    }
}

#[napi]
pub struct EvdevHook {
    callback: EventTsfn,
    inner: Mutex<HookInner>,
}

#[napi]
impl EvdevHook {
    #[napi(constructor)]
    pub fn new(on_event: Function<NativeEvent, UnknownReturnValue>) -> Result<Self> {
        let callback = Arc::new(
            on_event
                .build_threadsafe_function::<NativeEvent>()
                .weak::<true>()
                .callee_handled::<false>()
                .max_queue_size::<EVENT_QUEUE_LIMIT>()
                .build()
                .map_err(|err| {
                    generic_error(format!("failed to build callback: {}", err.reason))
                })?,
        );
        Ok(Self {
            callback,
            inner: Mutex::new(HookInner {
                stop: None,
                exit_fd: None,
                thread: None,
                opened: false,
            }),
        })
    }

    #[napi]
    pub fn start(&self) -> Result<bool> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| generic_error("hook lock poisoned"))?;
        if inner.thread.is_some() {
            return Ok(inner.opened);
        }
        let stop = Arc::new(AtomicBool::new(false));
        let exit_fd = Arc::new(
            ExitFd::new()
                .map_err(|err| generic_error(format!("failed to allocate eventfd: {err}")))?,
        );
        let mut reader = Reader::new(self.callback.clone(), stop.clone(), exit_fd.clone())
            .map_err(|err| generic_error(format!("evdev start failed: {err}")))?;

        reader.open_all_devices();
        let opened = !reader.devices.is_empty();

        let join = thread::Builder::new()
            .name("fluxer-linux-evdev-reader".to_string())
            .spawn(move || {
                let mut reader = reader;
                reader.run();
            })
            .map_err(|err| generic_error(format!("failed to spawn reader thread: {err}")))?;

        inner.stop = Some(stop);
        inner.exit_fd = Some(exit_fd);
        inner.thread = Some(join);
        inner.opened = opened;
        Ok(opened)
    }

    #[napi]
    pub fn stop(&self) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| generic_error("hook lock poisoned"))?;
        inner.stop_and_join();
        Ok(())
    }
}

impl Drop for EvdevHook {
    fn drop(&mut self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.stop_and_join();
        }
    }
}

#[napi(js_name = "nameToEvdevKeycode")]
pub fn name_to_evdev_keycode(name: String) -> u32 {
    u32::from(keymap::name_to_keycode(&name))
}

fn generic_error(reason: impl Into<String>) -> napi::Error {
    napi::Error::new(Status::GenericFailure, reason.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_poll_keys_are_legal_and_outside_fd_range() {
        assert_eq!(EXIT_KEY, 0);
        assert_eq!(MONITOR_KEY, 1);
        assert_ne!(EXIT_KEY, MONITOR_KEY);
        assert_eq!(poll_key_for_device_fd(0), Some(DEVICE_KEY_BASE));
        assert_eq!(device_fd_from_poll_key(DEVICE_KEY_BASE), Some(0));
        assert_eq!(device_fd_from_poll_key(EXIT_KEY), None);
        assert_eq!(device_fd_from_poll_key(MONITOR_KEY), None);
    }

    #[test]
    fn poller_accepts_exit_control_key() {
        let poller = Poller::new().expect("create poller");
        let exit_fd = ExitFd::new().expect("create exit fd");
        unsafe {
            poller
                .add(&exit_fd.as_borrowed(), PollEvent::readable(EXIT_KEY))
                .expect("register exit fd");
        }
        poller
            .delete(exit_fd.as_borrowed())
            .expect("delete exit fd");
    }

    #[test]
    fn routable_key_code_filter_keeps_keyboards_and_dom_mouse_buttons() {
        assert!(is_routable_key_code(KeyCode::KEY_A.code()));
        assert!(is_routable_key_code(KeyCode::KEY_LEFTCTRL.code()));
        assert!(is_routable_key_code(KeyCode::BTN_LEFT.code()));
        assert!(is_routable_key_code(KeyCode::BTN_FORWARD.code()));
    }

    #[test]
    fn routable_key_code_filter_ignores_tablet_pad_and_tool_buttons() {
        assert!(!is_routable_key_code(KeyCode::BTN_0.code()));
        assert!(!is_routable_key_code(KeyCode::BTN_TOOL_PEN.code()));
        assert!(!is_routable_key_code(KeyCode::BTN_STYLUS.code()));
    }
}
