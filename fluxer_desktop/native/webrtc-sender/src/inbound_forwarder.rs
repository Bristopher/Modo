// SPDX-License-Identifier: AGPL-3.0-or-later

use futures_core::Stream;
use napi::tokio;
use napi::tokio::task::AbortHandle;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::future::poll_fn;
use std::pin::pin;

pub const INBOUND_FORWARDERS_MAX: usize = 512;

struct ForwarderEntry {
    participant_sid: String,
    handle: AbortHandle,
}

pub struct InboundForwarderRegistry {
    entries: Mutex<HashMap<String, ForwarderEntry>>,
}

impl InboundForwarderRegistry {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.lock().len()
    }

    #[cfg(test)]
    pub fn contains(&self, track_sid: &str) -> bool {
        self.entries.lock().contains_key(track_sid)
    }

    pub fn register(&self, track_sid: &str, participant_sid: &str, handle: AbortHandle) -> bool {
        if track_sid.is_empty() {
            handle.abort();
            return false;
        }
        let mut entries = self.entries.lock();
        if let Some(previous) = entries.remove(track_sid) {
            previous.handle.abort();
        }
        if entries.len() >= INBOUND_FORWARDERS_MAX {
            drop(entries);
            handle.abort();
            eprintln!(
                "webrtc-sender: inbound forwarder registry at cap {INBOUND_FORWARDERS_MAX}; \
                 refusing forwarder for track {track_sid}"
            );
            return false;
        }
        entries.insert(
            track_sid.to_string(),
            ForwarderEntry {
                participant_sid: participant_sid.to_string(),
                handle,
            },
        );
        assert!(entries.len() <= INBOUND_FORWARDERS_MAX);
        true
    }

    pub fn cancel(&self, track_sid: &str) {
        let removed = self.entries.lock().remove(track_sid);
        if let Some(entry) = removed {
            entry.handle.abort();
        }
    }

    pub fn cancel_for_participant(&self, participant_sid: &str) {
        let aborted: Vec<ForwarderEntry> = {
            let mut entries = self.entries.lock();
            let matching: Vec<String> = entries
                .iter()
                .filter(|(_, entry)| entry.participant_sid == participant_sid)
                .map(|(track_sid, _)| track_sid.clone())
                .collect();
            matching
                .into_iter()
                .filter_map(|track_sid| entries.remove(&track_sid))
                .collect()
        };
        for entry in aborted {
            entry.handle.abort();
        }
    }

    pub fn clear(&self) {
        let drained: Vec<ForwarderEntry> = {
            let mut entries = self.entries.lock();
            entries.drain().map(|(_, entry)| entry).collect()
        };
        for entry in drained {
            entry.handle.abort();
        }
    }
}

impl Default for InboundForwarderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn spawn_drain_forwarder<S, F>(stream: S, mut on_item: F) -> AbortHandle
where
    S: Stream + Send + 'static,
    S::Item: Send,
    F: FnMut(S::Item) + Send + 'static,
{
    let task = tokio::spawn(async move {
        let mut stream = pin!(stream);
        loop {
            let item = poll_fn(|cx| stream.as_mut().poll_next(cx)).await;
            match item {
                Some(item) => on_item(item),
                None => return,
            }
        }
    });
    task.abort_handle()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::task::{Context, Poll};
    use tokio::runtime::Builder;
    use tokio::sync::Notify;

    struct CloseSignal {
        closed: AtomicBool,
        notify: Notify,
    }

    impl CloseSignal {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                closed: AtomicBool::new(false),
                notify: Notify::new(),
            })
        }

        fn is_closed(&self) -> bool {
            self.closed.load(Ordering::SeqCst)
        }

        fn fire(&self) {
            self.closed.store(true, Ordering::SeqCst);
            self.notify.notify_one();
        }

        async fn wait(&self) {
            if self.is_closed() {
                return;
            }
            self.notify.notified().await;
            assert!(self.is_closed());
        }
    }

    struct FakeVideoStream {
        signal: Arc<CloseSignal>,
    }

    impl Stream for FakeVideoStream {
        type Item = u64;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Pending
        }
    }

    impl Drop for FakeVideoStream {
        fn drop(&mut self) {
            self.signal.fire();
        }
    }

    struct CountedStream {
        items: VecDeque<u64>,
        count: Arc<AtomicUsize>,
        done: Arc<Notify>,
    }

    impl Stream for CountedStream {
        type Item = u64;

        fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            match self.items.pop_front() {
                Some(item) => {
                    self.count.fetch_add(1, Ordering::SeqCst);
                    Poll::Ready(Some(item))
                }
                None => {
                    self.done.notify_one();
                    Poll::Ready(None)
                }
            }
        }
    }

    fn runtime() -> tokio::runtime::Runtime {
        Builder::new_multi_thread()
            .worker_threads(1)
            .build()
            .expect("tokio runtime")
    }

    fn dispatch_track_subscribed(
        registry: &InboundForwarderRegistry,
        track_sid: &str,
        participant_sid: &str,
    ) -> Arc<CloseSignal> {
        let (signal, registered) = dispatch_with_outcome(registry, track_sid, participant_sid);
        assert!(registered);
        signal
    }

    fn dispatch_with_outcome(
        registry: &InboundForwarderRegistry,
        track_sid: &str,
        participant_sid: &str,
    ) -> (Arc<CloseSignal>, bool) {
        let signal = CloseSignal::new();
        let stream = FakeVideoStream {
            signal: signal.clone(),
        };
        let handle = spawn_drain_forwarder(stream, |_frame| {});
        let registered = registry.register(track_sid, participant_sid, handle);
        (signal, registered)
    }

    #[test]
    fn teardown_symmetry_closes_the_stream() {
        let runtime = runtime();
        runtime.block_on(async {
            let registry = InboundForwarderRegistry::new();
            let signal = dispatch_track_subscribed(&registry, "TR_x", "PA_one");

            assert_eq!(registry.len(), 1);
            assert!(!signal.is_closed());

            registry.cancel("TR_x");
            assert_eq!(registry.len(), 0);

            signal.wait().await;
            assert!(signal.is_closed());
        });
    }

    #[test]
    fn repeated_subscribe_unsubscribe_never_accumulates() {
        let runtime = runtime();
        runtime.block_on(async {
            let registry = InboundForwarderRegistry::new();
            let cycles = 32usize;
            let mut signals = Vec::with_capacity(cycles);

            for _ in 0..cycles {
                let signal = dispatch_track_subscribed(&registry, "TR_x", "PA_one");
                assert_eq!(registry.len(), 1);
                registry.cancel("TR_x");
                assert_eq!(registry.len(), 0);
                signal.wait().await;
                signals.push(signal);
            }

            assert_eq!(registry.len(), 0);
            let closed_count = signals.iter().filter(|signal| signal.is_closed()).count();
            assert_eq!(closed_count, cycles);
        });
    }

    #[test]
    fn double_subscribe_keeps_one_live_forwarder() {
        let runtime = runtime();
        runtime.block_on(async {
            let registry = InboundForwarderRegistry::new();
            let first_signal = dispatch_track_subscribed(&registry, "TR_x", "PA_one");
            let second_signal = dispatch_track_subscribed(&registry, "TR_x", "PA_one");

            assert_eq!(registry.len(), 1);
            first_signal.wait().await;
            assert!(first_signal.is_closed());
            assert!(!second_signal.is_closed());

            registry.cancel("TR_x");
            assert_eq!(registry.len(), 0);
            second_signal.wait().await;
            assert!(second_signal.is_closed());
        });
    }

    #[test]
    fn participant_disconnect_tears_down_all_forwarders() {
        let runtime = runtime();
        runtime.block_on(async {
            let registry = InboundForwarderRegistry::new();
            let video_signal = dispatch_track_subscribed(&registry, "TR_video", "PA_one");
            let audio_signal = dispatch_track_subscribed(&registry, "TR_audio", "PA_one");
            let other_signal = dispatch_track_subscribed(&registry, "TR_other", "PA_two");
            assert_eq!(registry.len(), 3);

            registry.cancel_for_participant("PA_one");
            assert_eq!(registry.len(), 1);
            assert!(registry.contains("TR_other"));

            video_signal.wait().await;
            audio_signal.wait().await;
            assert!(video_signal.is_closed());
            assert!(audio_signal.is_closed());
            assert!(!other_signal.is_closed());

            registry.clear();
            assert_eq!(registry.len(), 0);
            other_signal.wait().await;
            assert!(other_signal.is_closed());
        });
    }

    #[test]
    fn clear_tears_down_every_forwarder() {
        let runtime = runtime();
        runtime.block_on(async {
            let registry = InboundForwarderRegistry::new();
            let mut signals = Vec::new();
            for index in 0..8 {
                let track_sid = format!("TR_{index}");
                signals.push(dispatch_track_subscribed(&registry, &track_sid, "PA_one"));
            }
            assert_eq!(registry.len(), 8);

            registry.clear();
            assert_eq!(registry.len(), 0);

            for signal in &signals {
                signal.wait().await;
            }
            assert!(signals.iter().all(|signal| signal.is_closed()));
        });
    }

    #[test]
    fn drain_forwarder_invokes_callback_per_item() {
        let runtime = runtime();
        runtime.block_on(async {
            let count = Arc::new(AtomicUsize::new(0));
            let polled = Arc::new(AtomicUsize::new(0));
            let polled_in_task = polled.clone();
            let done = Arc::new(Notify::new());
            let stream = CountedStream {
                items: VecDeque::from(vec![1u64, 2, 3, 4]),
                count: count.clone(),
                done: done.clone(),
            };
            let _handle = spawn_drain_forwarder(stream, move |_item| {
                polled_in_task.fetch_add(1, Ordering::SeqCst);
            });
            done.notified().await;

            assert_eq!(count.load(Ordering::SeqCst), 4);
            assert_eq!(polled.load(Ordering::SeqCst), 4);
        });
    }

    #[test]
    fn register_refuses_and_closes_forwarder_at_cap() {
        let runtime = runtime();
        runtime.block_on(async {
            let registry = InboundForwarderRegistry::new();
            for index in 0..INBOUND_FORWARDERS_MAX {
                let track_sid = format!("TR_{index}");
                let (_signal, registered) = dispatch_with_outcome(&registry, &track_sid, "PA_one");
                assert!(registered);
            }
            assert_eq!(registry.len(), INBOUND_FORWARDERS_MAX);

            let (overflow_signal, registered) =
                dispatch_with_outcome(&registry, "TR_overflow", "PA_one");
            assert!(!registered);
            assert_eq!(registry.len(), INBOUND_FORWARDERS_MAX);
            assert!(!registry.contains("TR_overflow"));

            overflow_signal.wait().await;
            assert!(overflow_signal.is_closed());

            registry.clear();
            assert_eq!(registry.len(), 0);
        });
    }

    #[test]
    fn register_refuses_and_closes_forwarder_for_empty_sid() {
        let runtime = runtime();
        runtime.block_on(async {
            let registry = InboundForwarderRegistry::new();
            let (signal, registered) = dispatch_with_outcome(&registry, "", "PA_one");
            assert!(!registered);
            assert_eq!(registry.len(), 0);

            signal.wait().await;
            assert!(signal.is_closed());
        });
    }
}
