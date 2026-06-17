// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_foundation::NSError;
use objc2_screen_capture_kit::{SCShareableContent, SCStream};

pub const DEFAULT_TIMEOUT_NS: u64 = 30 * 1_000_000_000;

#[derive(Debug, Eq, PartialEq)]
pub enum AsyncError {
    AsyncTimedOut,
    SCKReturnedError,
}

struct WakerInner {
    state: Mutex<WakerState>,
    cv: Condvar,
}

struct WakerState {
    done: bool,
    failed: bool,
    err: Option<Retained<NSError>>,
    content: Option<Retained<SCShareableContent>>,
}

unsafe impl Send for WakerInner {}
unsafe impl Sync for WakerInner {}

fn new_waker() -> Arc<WakerInner> {
    Arc::new(WakerInner {
        state: Mutex::new(WakerState {
            done: false,
            failed: false,
            err: None,
            content: None,
        }),
        cv: Condvar::new(),
    })
}

fn lock_state(w: &WakerInner) -> MutexGuard<'_, WakerState> {
    w.state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn wait_deadline(w: &Arc<WakerInner>, timeout_ns: u64) -> bool {
    let s = lock_state(w);
    if s.done {
        return true;
    }
    let dur = Duration::from_nanos(timeout_ns);
    match w.cv.wait_timeout(s, dur) {
        Ok((state, _)) => state.done,
        Err(poisoned) => poisoned.into_inner().0.done,
    }
}

fn retain_error(err: *mut NSError) -> Result<Option<Retained<NSError>>, AsyncError> {
    if err.is_null() {
        Ok(None)
    } else {
        unsafe { Retained::retain(err) }
            .map(Some)
            .ok_or(AsyncError::SCKReturnedError)
    }
}

fn retain_content(
    content: *mut SCShareableContent,
) -> Result<Option<Retained<SCShareableContent>>, AsyncError> {
    if content.is_null() {
        Ok(None)
    } else {
        unsafe { Retained::retain(content) }
            .map(Some)
            .ok_or(AsyncError::SCKReturnedError)
    }
}

fn complete(
    waker: &WakerInner,
    err: Option<Retained<NSError>>,
    content: Option<Retained<SCShareableContent>>,
    failed: bool,
) {
    let mut s = lock_state(waker);
    s.err = err;
    s.content = content;
    s.failed = failed;
    s.done = true;
    waker.cv.notify_all();
}

pub fn await_ns_error_block_start(stream: &SCStream, timeout_ns: u64) -> Result<(), AsyncError> {
    let waker = new_waker();
    let waker_cb = waker.clone();
    let blk = RcBlock::new(move |err: *mut NSError| {
        let (err_opt, failed) = match retain_error(err) {
            Ok(err_opt) => (err_opt, false),
            Err(_) => (None, true),
        };
        complete(&waker_cb, err_opt, None, failed);
    });

    unsafe {
        stream.startCaptureWithCompletionHandler(Some(&blk));
    }

    if !wait_deadline(&waker, timeout_ns) {
        return Err(AsyncError::AsyncTimedOut);
    }
    let s = lock_state(&waker);
    if s.failed || s.err.is_some() {
        return Err(AsyncError::SCKReturnedError);
    }
    Ok(())
}

pub fn await_ns_error_block_stop(stream: &SCStream, timeout_ns: u64) -> Result<(), AsyncError> {
    let waker = new_waker();
    let waker_cb = waker.clone();
    let blk = RcBlock::new(move |err: *mut NSError| {
        let (err_opt, failed) = match retain_error(err) {
            Ok(err_opt) => (err_opt, false),
            Err(_) => (None, true),
        };
        complete(&waker_cb, err_opt, None, failed);
    });

    unsafe {
        stream.stopCaptureWithCompletionHandler(Some(&blk));
    }

    if !wait_deadline(&waker, timeout_ns) {
        return Err(AsyncError::AsyncTimedOut);
    }
    let s = lock_state(&waker);
    if s.failed || s.err.is_some() {
        return Err(AsyncError::SCKReturnedError);
    }
    Ok(())
}

pub fn start_capture(stream: &SCStream, timeout_ns: u64) -> Result<(), AsyncError> {
    await_ns_error_block_start(stream, timeout_ns)
}

pub fn stop_capture(stream: &SCStream, timeout_ns: u64) -> Result<(), AsyncError> {
    await_ns_error_block_stop(stream, timeout_ns)
}

pub struct ShareableContent {
    pub content: Retained<SCShareableContent>,
}

pub fn get_shareable_content(
    excluding_desktop_windows: bool,
    on_screen_windows_only: bool,
    timeout_ns: u64,
) -> Result<ShareableContent, AsyncError> {
    let waker = new_waker();
    let waker_cb = waker.clone();
    let blk = RcBlock::new(move |content: *mut SCShareableContent, err: *mut NSError| {
        let (err_opt, err_failed) = match retain_error(err) {
            Ok(err_opt) => (err_opt, false),
            Err(_) => (None, true),
        };
        let (content_opt, content_failed) = match retain_content(content) {
            Ok(content_opt) => (content_opt, false),
            Err(_) => (None, true),
        };
        complete(
            &waker_cb,
            err_opt,
            content_opt,
            err_failed || content_failed,
        );
    });

    unsafe {
        SCShareableContent::getShareableContentExcludingDesktopWindows_onScreenWindowsOnly_completionHandler(
            excluding_desktop_windows,
            on_screen_windows_only,
            &blk,
        );
    }

    if !wait_deadline(&waker, timeout_ns) {
        return Err(AsyncError::AsyncTimedOut);
    }
    let mut s = lock_state(&waker);
    if s.failed || s.err.is_some() {
        return Err(AsyncError::SCKReturnedError);
    }
    match s.content.take() {
        Some(content) => Ok(ShareableContent { content }),
        None => Err(AsyncError::SCKReturnedError),
    }
}
