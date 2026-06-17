// Copyright 2026 LiveKit, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use webrtc_sys::recorded_audio_tap::ffi as sys;
    use webrtc_sys::recorded_audio_tap::RecordedAudioSinkWrapper;

    /// Handle to an installed recorded-audio sink. Dropping it does not clear
    /// the sink; call [`clear_recorded_audio_sink`] with this generation.
    pub type RecordedAudioSinkGeneration = u64;

    /// Installs a process-global tap on platform-ADM recorded microphone audio.
    ///
    /// `callback` is invoked on the ADM capture thread with one 48kHz mono
    /// 10ms frame (480 samples) per call: `(samples, sample_rate_hz,
    /// num_channels, samples_per_channel)`. It must be wait-free: do no
    /// allocation or blocking work, only hand the frame to a bounded queue.
    /// Returns a generation token to pass to [`clear_recorded_audio_sink`].
    pub fn set_recorded_audio_sink<F>(callback: F) -> RecordedAudioSinkGeneration
    where
        F: Fn(&[i16], i32, usize, usize) + Send + Sync + 'static,
    {
        sys::set_recorded_audio_sink(Box::new(RecordedAudioSinkWrapper::new(Box::new(callback))))
    }

    /// Removes the recorded-audio sink, but only if `generation` is still the
    /// installed one. A stale token is a no-op, so a late teardown cannot
    /// clobber a sink a newer caller installed.
    pub fn clear_recorded_audio_sink(generation: RecordedAudioSinkGeneration) {
        sys::clear_recorded_audio_sink(generation);
    }
}
