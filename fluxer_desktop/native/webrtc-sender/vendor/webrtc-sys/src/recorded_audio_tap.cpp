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

#include "livekit/recorded_audio_tap.h"

#include <optional>
#include <utility>

#include "audio/remix_resample.h"
#include "common_audio/include/audio_util.h"
#include "rtc_base/checks.h"
#include "rtc_base/synchronization/mutex.h"
#include "webrtc-sys/src/recorded_audio_tap.rs.h"

namespace livekit_ffi {

namespace {

constexpr int kRecordedSinkSampleRateHz = 48000;
constexpr size_t kRecordedSinkNumChannels = 1;

// Process-global recorded-audio sink. The platform ADM is a process singleton
// owned by the livekit runtime, so the tap has to live here rather than on any
// one engine handle.
struct GlobalRecordedAudioSink {
  webrtc::Mutex lock;
  uint64_t generation = 0;
  std::optional<rust::Box<RecordedAudioSinkWrapper>> sink;
};

// Leaked on purpose: a process-global with a mutex must not be torn down during
// static destruction while the ADM capture thread may still touch it.
GlobalRecordedAudioSink& global_recorded_audio_sink() {
  static GlobalRecordedAudioSink* instance = new GlobalRecordedAudioSink();
  return *instance;
}

}  // namespace

uint64_t set_recorded_audio_sink(rust::Box<RecordedAudioSinkWrapper> sink) {
  GlobalRecordedAudioSink& global = global_recorded_audio_sink();
  webrtc::MutexLock lock(&global.lock);
  global.generation += 1;
  global.sink.emplace(std::move(sink));
  RTC_DCHECK(global.sink.has_value());
  return global.generation;
}

void clear_recorded_audio_sink(uint64_t generation) {
  GlobalRecordedAudioSink& global = global_recorded_audio_sink();
  webrtc::MutexLock lock(&global.lock);
  if (global.generation != generation) {
    return;
  }
  global.sink.reset();
}

RecordingTransportProxy::RecordingTransportProxy() {
  capture_frame_.sample_rate_hz_ = kRecordedSinkSampleRateHz;
  capture_frame_.num_channels_ = kRecordedSinkNumChannels;
  capture_frame_.samples_per_channel_ =
      webrtc::SampleRateToDefaultChannelSize(kRecordedSinkSampleRateHz);
}

RecordingTransportProxy::~RecordingTransportProxy() = default;

void RecordingTransportProxy::set_real_transport(
    webrtc::AudioTransport* transport) {
  real_transport_.store(transport, std::memory_order_release);
}

void RecordingTransportProxy::TeeRecordedData(const void* audioSamples,
                                              size_t nSamples,
                                              size_t nBytesPerSample,
                                              size_t nChannels,
                                              uint32_t samplesPerSec) {
  if (audioSamples == nullptr) {
    return;
  }
  if (nBytesPerSample != sizeof(int16_t)) {
    return;
  }
  if (nChannels == 0) {
    return;
  }
  if (samplesPerSec == 0) {
    return;
  }

  GlobalRecordedAudioSink& global = global_recorded_audio_sink();
  // TryLock keeps the ADM capture thread wait-free: a frame dropped during an
  // install/clear is acceptable, blocking the capture thread is not.
  if (!global.lock.TryLock()) {
    return;
  }
  if (global.sink.has_value()) {
    webrtc::InterleavedView<const int16_t> source(
        static_cast<const int16_t*>(audioSamples), nSamples, nChannels);
    webrtc::voe::RemixAndResample(source, static_cast<int>(samplesPerSec),
                                  &capture_resampler_, &capture_frame_);
    rust::Slice<const int16_t> samples(
        capture_frame_.data(),
        capture_frame_.num_channels() * capture_frame_.samples_per_channel());
    (*global.sink)
        ->on_recorded_data(samples, capture_frame_.sample_rate_hz(),
                           capture_frame_.num_channels(),
                           capture_frame_.samples_per_channel());
  }
  global.lock.Unlock();
}

int32_t RecordingTransportProxy::RecordedDataIsAvailable(
    const void* audioSamples,
    size_t nSamples,
    size_t nBytesPerSample,
    size_t nChannels,
    uint32_t samplesPerSec,
    uint32_t totalDelayMS,
    int32_t clockDrift,
    uint32_t currentMicLevel,
    bool keyPressed,
    uint32_t& newMicLevel) {
  TeeRecordedData(audioSamples, nSamples, nBytesPerSample, nChannels,
                  samplesPerSec);
  webrtc::AudioTransport* real =
      real_transport_.load(std::memory_order_acquire);
  if (real == nullptr) {
    newMicLevel = currentMicLevel;
    return 0;
  }
  return real->RecordedDataIsAvailable(audioSamples, nSamples, nBytesPerSample,
                                       nChannels, samplesPerSec, totalDelayMS,
                                       clockDrift, currentMicLevel, keyPressed,
                                       newMicLevel);
}

int32_t RecordingTransportProxy::RecordedDataIsAvailable(
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
    std::optional<int64_t> estimatedCaptureTimeNS) {
  TeeRecordedData(audioSamples, nSamples, nBytesPerSample, nChannels,
                  samplesPerSec);
  webrtc::AudioTransport* real =
      real_transport_.load(std::memory_order_acquire);
  if (real == nullptr) {
    newMicLevel = currentMicLevel;
    return 0;
  }
  return real->RecordedDataIsAvailable(
      audioSamples, nSamples, nBytesPerSample, nChannels, samplesPerSec,
      totalDelayMS, clockDrift, currentMicLevel, keyPressed, newMicLevel,
      estimatedCaptureTimeNS);
}

int32_t RecordingTransportProxy::NeedMorePlayData(size_t nSamples,
                                                  size_t nBytesPerSample,
                                                  size_t nChannels,
                                                  uint32_t samplesPerSec,
                                                  void* audioSamples,
                                                  size_t& nSamplesOut,
                                                  int64_t* elapsed_time_ms,
                                                  int64_t* ntp_time_ms) {
  webrtc::AudioTransport* real =
      real_transport_.load(std::memory_order_acquire);
  if (real == nullptr) {
    nSamplesOut = 0;
    if (elapsed_time_ms != nullptr) {
      *elapsed_time_ms = -1;
    }
    if (ntp_time_ms != nullptr) {
      *ntp_time_ms = -1;
    }
    return 0;
  }
  return real->NeedMorePlayData(nSamples, nBytesPerSample, nChannels,
                                samplesPerSec, audioSamples, nSamplesOut,
                                elapsed_time_ms, ntp_time_ms);
}

void RecordingTransportProxy::PullRenderData(int bits_per_sample,
                                             int sample_rate,
                                             size_t number_of_channels,
                                             size_t number_of_frames,
                                             void* audio_data,
                                             int64_t* elapsed_time_ms,
                                             int64_t* ntp_time_ms) {
  webrtc::AudioTransport* real =
      real_transport_.load(std::memory_order_acquire);
  if (real == nullptr) {
    return;
  }
  real->PullRenderData(bits_per_sample, sample_rate, number_of_channels,
                       number_of_frames, audio_data, elapsed_time_ms,
                       ntp_time_ms);
}

}  // namespace livekit_ffi
