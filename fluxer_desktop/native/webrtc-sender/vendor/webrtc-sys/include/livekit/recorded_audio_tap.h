/*
 * Copyright 2026 LiveKit, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#pragma once

#include <atomic>
#include <cstdint>
#include <optional>

#include "api/audio/audio_device_defines.h"
#include "api/audio/audio_frame.h"
#include "common_audio/resampler/include/push_resampler.h"
#include "rust/cxx.h"

namespace livekit_ffi {

struct RecordedAudioSinkWrapper;

// Installs the process-global recorded-audio sink. The platform ADM delivers
// 10ms PCM frames on its capture thread; while a sink is installed each frame
// is resampled to 48kHz mono and handed to Rust. Returns a generation token so
// a later clear can target exactly this installation and never clobber a sink
// that a subsequent caller installed in the meantime.
uint64_t set_recorded_audio_sink(rust::Box<RecordedAudioSinkWrapper> sink);

// Removes the global recorded-audio sink only if `generation` matches the
// currently-installed sink. A stale token is a no-op.
void clear_recorded_audio_sink(uint64_t generation);

// AudioTransport interposer registered with the platform ADM in place of the
// real transport. It tees recorded microphone frames to the global sink and
// forwards every call unchanged to the real transport so the normal send and
// playout pipelines are unaffected.
class RecordingTransportProxy : public webrtc::AudioTransport {
 public:
  RecordingTransportProxy();
  ~RecordingTransportProxy();

  void set_real_transport(webrtc::AudioTransport* transport);

  int32_t RecordedDataIsAvailable(const void* audioSamples,
                                  size_t nSamples,
                                  size_t nBytesPerSample,
                                  size_t nChannels,
                                  uint32_t samplesPerSec,
                                  uint32_t totalDelayMS,
                                  int32_t clockDrift,
                                  uint32_t currentMicLevel,
                                  bool keyPressed,
                                  uint32_t& newMicLevel) override;

  int32_t RecordedDataIsAvailable(
      const void* audioSamples,
      size_t nSamples,
      size_t nBytesPerSample,
      size_t nChannels,
      uint32_t samplesPerSec,
      uint32_t totalDelayMS,
      int32_t clockDrift,
      uint32_t currentMicLevel,
      bool keyPressed,
      uint32_t& newMicLevel,
      std::optional<int64_t> estimatedCaptureTimeNS) override;

  int32_t NeedMorePlayData(size_t nSamples,
                           size_t nBytesPerSample,
                           size_t nChannels,
                           uint32_t samplesPerSec,
                           void* audioSamples,
                           size_t& nSamplesOut,
                           int64_t* elapsed_time_ms,
                           int64_t* ntp_time_ms) override;

  void PullRenderData(int bits_per_sample,
                      int sample_rate,
                      size_t number_of_channels,
                      size_t number_of_frames,
                      void* audio_data,
                      int64_t* elapsed_time_ms,
                      int64_t* ntp_time_ms) override;

 private:
  void TeeRecordedData(const void* audioSamples,
                       size_t nSamples,
                       size_t nBytesPerSample,
                       size_t nChannels,
                       uint32_t samplesPerSec);

  std::atomic<webrtc::AudioTransport*> real_transport_{nullptr};

  // Touched only on the ADM capture thread inside RecordedDataIsAvailable,
  // which WebRTC serializes, so neither member needs a lock.
  webrtc::AudioFrame capture_frame_;
  webrtc::PushResampler<int16_t> capture_resampler_;
};

}  // namespace livekit_ffi
