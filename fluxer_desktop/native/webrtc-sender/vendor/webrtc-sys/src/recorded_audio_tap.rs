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

#[cxx::bridge(namespace = "livekit_ffi")]
pub mod ffi {
    unsafe extern "C++" {
        include!("livekit/recorded_audio_tap.h");

        fn set_recorded_audio_sink(sink: Box<RecordedAudioSinkWrapper>) -> u64;
        fn clear_recorded_audio_sink(generation: u64);
    }

    extern "Rust" {
        type RecordedAudioSinkWrapper;

        fn on_recorded_data(
            self: &RecordedAudioSinkWrapper,
            data: &[i16],
            sample_rate: i32,
            nb_channels: usize,
            nb_frames: usize,
        );
    }
}

type RecordedAudioCallback = Box<dyn Fn(&[i16], i32, usize, usize) + Send + Sync>;

pub struct RecordedAudioSinkWrapper {
    callback: RecordedAudioCallback,
}

impl RecordedAudioSinkWrapper {
    pub fn new(callback: RecordedAudioCallback) -> RecordedAudioSinkWrapper {
        RecordedAudioSinkWrapper { callback }
    }

    fn on_recorded_data(
        &self,
        data: &[i16],
        sample_rate: i32,
        nb_channels: usize,
        nb_frames: usize,
    ) {
        (self.callback)(data, sample_rate, nb_channels, nb_frames);
    }
}
