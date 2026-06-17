/*
 * Copyright 2025 LiveKit, Inc.
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

#include "livekit/video_frame_buffer.h"

#include "api/make_ref_counted.h"
#include "rtc_base/logging.h"

#if defined(__linux__)
#include <unistd.h>
#endif

namespace livekit_ffi {

VideoFrameBuffer::VideoFrameBuffer(
    webrtc::scoped_refptr<webrtc::VideoFrameBuffer> buffer)
    : buffer_(std::move(buffer)) {}

VideoFrameBufferType VideoFrameBuffer::buffer_type() const {
  return static_cast<VideoFrameBufferType>(buffer_->type());
}

unsigned int VideoFrameBuffer::width() const {
  return buffer_->width();
}

unsigned int VideoFrameBuffer::height() const {
  return buffer_->height();
}

std::unique_ptr<I420Buffer> VideoFrameBuffer::to_i420() const {
  return std::make_unique<I420Buffer>(buffer_->ToI420());
}

// const_cast is valid here because we take the ownership on the rust side
std::unique_ptr<I420Buffer> VideoFrameBuffer::get_i420() {
  return std::make_unique<I420Buffer>(
      webrtc::scoped_refptr<webrtc::I420BufferInterface>(
          const_cast<webrtc::I420BufferInterface*>(buffer_->GetI420())));
}

std::unique_ptr<I420ABuffer> VideoFrameBuffer::get_i420a() {
  return std::make_unique<I420ABuffer>(
      webrtc::scoped_refptr<webrtc::I420ABufferInterface>(
          const_cast<webrtc::I420ABufferInterface*>(buffer_->GetI420A())));
}

std::unique_ptr<I422Buffer> VideoFrameBuffer::get_i422() {
  return std::make_unique<I422Buffer>(
      webrtc::scoped_refptr<webrtc::I422BufferInterface>(
          const_cast<webrtc::I422BufferInterface*>(buffer_->GetI422())));
}

std::unique_ptr<I444Buffer> VideoFrameBuffer::get_i444() {
  return std::make_unique<I444Buffer>(
      webrtc::scoped_refptr<webrtc::I444BufferInterface>(
          const_cast<webrtc::I444BufferInterface*>(buffer_->GetI444())));
}

std::unique_ptr<I010Buffer> VideoFrameBuffer::get_i010() {
  return std::make_unique<I010Buffer>(
      webrtc::scoped_refptr<webrtc::I010BufferInterface>(
          const_cast<webrtc::I010BufferInterface*>(buffer_->GetI010())));
}

std::unique_ptr<NV12Buffer> VideoFrameBuffer::get_nv12() {
  return std::make_unique<NV12Buffer>(
      webrtc::scoped_refptr<webrtc::NV12BufferInterface>(
          const_cast<webrtc::NV12BufferInterface*>(buffer_->GetNV12())));
}

webrtc::scoped_refptr<webrtc::VideoFrameBuffer> VideoFrameBuffer::get() const {
  return buffer_;
}

FluxerGpuFrameBuffer::FluxerGpuFrameBuffer(uint64_t handle,
                                           uint32_t width,
                                           uint32_t height,
                                           uint32_t dxgi_format)
    : kind_(Kind::kD3D11Texture),
      width_(width),
      height_(height),
      d3d11_handle_(handle),
      dxgi_format_(dxgi_format) {}

FluxerGpuFrameBuffer::FluxerGpuFrameBuffer(int fd0,
                                           int fd1,
                                           int fd2,
                                           int fd3,
                                           uint32_t plane_count,
                                           uint32_t width,
                                           uint32_t height,
                                           uint32_t drm_format,
                                           uint64_t modifier,
                                           uint32_t stride0,
                                           uint32_t stride1,
                                           uint32_t stride2,
                                           uint32_t stride3,
                                           uint32_t offset0,
                                           uint32_t offset1,
                                           uint32_t offset2,
                                           uint32_t offset3,
                                           uint64_t device_uuid_hi,
                                           uint64_t device_uuid_lo)
    : kind_(Kind::kDmaBuf),
      width_(width),
      height_(height),
      plane_count_(plane_count),
      drm_format_(drm_format),
      modifier_(modifier),
      device_uuid_hi_(device_uuid_hi),
      device_uuid_lo_(device_uuid_lo) {
  const int input_fds[4] = {fd0, fd1, fd2, fd3};
  const uint32_t input_strides[4] = {stride0, stride1, stride2, stride3};
  const uint32_t input_offsets[4] = {offset0, offset1, offset2, offset3};
  for (uint32_t plane = 0; plane < plane_count_ && plane < 4; ++plane) {
#if defined(__linux__)
    fds_[plane] = input_fds[plane] >= 0 ? dup(input_fds[plane]) : -1;
#else
    fds_[plane] = -1;
#endif
    strides_[plane] = input_strides[plane];
    offsets_[plane] = input_offsets[plane];
  }
}

FluxerGpuFrameBuffer::~FluxerGpuFrameBuffer() {
#if defined(__linux__)
  for (int& fd : fds_) {
    if (fd >= 0) {
      close(fd);
      fd = -1;
    }
  }
#endif
}

webrtc::VideoFrameBuffer::Type FluxerGpuFrameBuffer::type() const {
  return Type::kNative;
}

int FluxerGpuFrameBuffer::width() const {
  return static_cast<int>(width_);
}

int FluxerGpuFrameBuffer::height() const {
  return static_cast<int>(height_);
}

webrtc::scoped_refptr<webrtc::I420BufferInterface>
FluxerGpuFrameBuffer::ToI420() {
  RTC_LOG(LS_WARNING)
      << "Fluxer GPU frame cannot be CPU-mapped by the WebRTC fallback path";
  return nullptr;
}

webrtc::scoped_refptr<webrtc::VideoFrameBuffer>
FluxerGpuFrameBuffer::CropAndScale(int offset_x,
                                   int offset_y,
                                   int crop_width,
                                   int crop_height,
                                   int scaled_width,
                                   int scaled_height) {
  RTC_LOG(LS_WARNING)
      << "Fluxer GPU frame cannot be cropped or scaled by WebRTC fallback";
  return nullptr;
}

webrtc::scoped_refptr<webrtc::VideoFrameBuffer>
FluxerGpuFrameBuffer::GetMappedFrameBuffer(webrtc::ArrayView<Type> types) {
  return nullptr;
}

std::string FluxerGpuFrameBuffer::storage_representation() const {
  return kind_ == Kind::kD3D11Texture ? "FluxerD3D11Texture"
                                      : "FluxerDmaBufTexture";
}

int FluxerGpuFrameBuffer::fd(uint32_t plane) const {
  return plane < 4 ? fds_[plane] : -1;
}

uint32_t FluxerGpuFrameBuffer::stride(uint32_t plane) const {
  return plane < 4 ? strides_[plane] : 0;
}

uint32_t FluxerGpuFrameBuffer::offset(uint32_t plane) const {
  return plane < 4 ? offsets_[plane] : 0;
}

const FluxerGpuFrameBuffer* AsFluxerGpuFrameBuffer(
    const webrtc::VideoFrameBuffer* buffer) {
  if (!buffer || buffer->type() != webrtc::VideoFrameBuffer::Type::kNative) {
    return nullptr;
  }
  const std::string storage = buffer->storage_representation();
  if (storage != "FluxerD3D11Texture" && storage != "FluxerDmaBufTexture") {
    return nullptr;
  }
  return static_cast<const FluxerGpuFrameBuffer*>(buffer);
}

PlanarYuvBuffer::PlanarYuvBuffer(
    webrtc::scoped_refptr<webrtc::PlanarYuvBuffer> buffer)
    : VideoFrameBuffer(buffer) {}

unsigned int PlanarYuvBuffer::chroma_width() const {
  return buffer()->ChromaWidth();
}

unsigned int PlanarYuvBuffer::chroma_height() const {
  return buffer()->ChromaHeight();
}

unsigned int PlanarYuvBuffer::stride_y() const {
  return buffer()->StrideY();
}

unsigned int PlanarYuvBuffer::stride_u() const {
  return buffer()->StrideU();
}

unsigned int PlanarYuvBuffer::stride_v() const {
  return buffer()->StrideV();
}

webrtc::PlanarYuvBuffer* PlanarYuvBuffer::buffer() const {
  return static_cast<webrtc::PlanarYuvBuffer*>(buffer_.get());
}

PlanarYuv8Buffer::PlanarYuv8Buffer(
    webrtc::scoped_refptr<webrtc::PlanarYuv8Buffer> buffer)
    : PlanarYuvBuffer(buffer) {}

const uint8_t* PlanarYuv8Buffer::data_y() const {
  return buffer()->DataY();
}

const uint8_t* PlanarYuv8Buffer::data_u() const {
  return buffer()->DataU();
}

const uint8_t* PlanarYuv8Buffer::data_v() const {
  return buffer()->DataV();
}

webrtc::PlanarYuv8Buffer* PlanarYuv8Buffer::buffer() const {
  return static_cast<webrtc::PlanarYuv8Buffer*>(buffer_.get());
}

PlanarYuv16BBuffer::PlanarYuv16BBuffer(
    webrtc::scoped_refptr<webrtc::PlanarYuv16BBuffer> buffer)
    : PlanarYuvBuffer(buffer) {}

const uint16_t* PlanarYuv16BBuffer::data_y() const {
  return buffer()->DataY();
}

const uint16_t* PlanarYuv16BBuffer::data_u() const {
  return buffer()->DataU();
}

const uint16_t* PlanarYuv16BBuffer::data_v() const {
  return buffer()->DataV();
}

webrtc::PlanarYuv16BBuffer* PlanarYuv16BBuffer::buffer() const {
  return static_cast<webrtc::PlanarYuv16BBuffer*>(buffer_.get());
}

BiplanarYuvBuffer::BiplanarYuvBuffer(
    webrtc::scoped_refptr<webrtc::BiplanarYuvBuffer> buffer)
    : VideoFrameBuffer(buffer) {}

unsigned int BiplanarYuvBuffer::chroma_width() const {
  return buffer()->ChromaWidth();
}

unsigned int BiplanarYuvBuffer::chroma_height() const {
  return buffer()->ChromaHeight();
}

unsigned int BiplanarYuvBuffer::stride_y() const {
  return buffer()->StrideY();
}

unsigned int BiplanarYuvBuffer::stride_uv() const {
  return buffer()->StrideUV();
}

webrtc::BiplanarYuvBuffer* BiplanarYuvBuffer::buffer() const {
  return static_cast<webrtc::BiplanarYuvBuffer*>(buffer_.get());
}

BiplanarYuv8Buffer::BiplanarYuv8Buffer(
    webrtc::scoped_refptr<webrtc::BiplanarYuv8Buffer> buffer)
    : BiplanarYuvBuffer(buffer) {}

const uint8_t* BiplanarYuv8Buffer::data_y() const {
  return buffer()->DataY();
}

const uint8_t* BiplanarYuv8Buffer::data_uv() const {
  return buffer()->DataUV();
}

webrtc::BiplanarYuv8Buffer* BiplanarYuv8Buffer::buffer() const {
  return static_cast<webrtc::BiplanarYuv8Buffer*>(buffer_.get());
}

I420Buffer::I420Buffer(webrtc::scoped_refptr<webrtc::I420BufferInterface> buffer)
    : PlanarYuv8Buffer(buffer) {}

webrtc::I420BufferInterface* I420Buffer::buffer() const {
  return static_cast<webrtc::I420BufferInterface*>(buffer_.get());
}

std::unique_ptr<I420Buffer> I420Buffer::scale(int scaled_width,
                                              int scaled_height) const {
  webrtc::scoped_refptr<webrtc::VideoFrameBuffer> result =
      buffer()->Scale(scaled_width, scaled_height);
  return std::make_unique<I420Buffer>(
      webrtc::scoped_refptr<webrtc::I420BufferInterface>(
          const_cast<webrtc::I420BufferInterface*>(result->GetI420())));
}

I420ABuffer::I420ABuffer(
    webrtc::scoped_refptr<webrtc::I420ABufferInterface> buffer)
    : I420Buffer(buffer) {}

unsigned int I420ABuffer::stride_a() const {
  return buffer()->StrideA();
}

const uint8_t* I420ABuffer::data_a() const {
  return buffer()->DataA();
}

webrtc::I420ABufferInterface* I420ABuffer::buffer() const {
  return static_cast<webrtc::I420ABufferInterface*>(buffer_.get());
}

std::unique_ptr<I420ABuffer> I420ABuffer::scale(int scaled_width,
                                                int scaled_height) const {
  webrtc::scoped_refptr<webrtc::VideoFrameBuffer> result =
      buffer()->Scale(scaled_width, scaled_height);
  return std::make_unique<I420ABuffer>(
      webrtc::scoped_refptr<webrtc::I420ABufferInterface>(
          const_cast<webrtc::I420ABufferInterface*>(result->GetI420A())));
}

I422Buffer::I422Buffer(webrtc::scoped_refptr<webrtc::I422BufferInterface> buffer)
    : PlanarYuv8Buffer(buffer) {}

webrtc::I422BufferInterface* I422Buffer::buffer() const {
  return static_cast<webrtc::I422BufferInterface*>(buffer_.get());
}

std::unique_ptr<I422Buffer> I422Buffer::scale(int scaled_width,
                                              int scaled_height) const {
  webrtc::scoped_refptr<webrtc::VideoFrameBuffer> result =
      buffer()->Scale(scaled_width, scaled_height);
  return std::make_unique<I422Buffer>(
      webrtc::scoped_refptr<webrtc::I422BufferInterface>(
          const_cast<webrtc::I422BufferInterface*>(result->GetI422())));
}

I444Buffer::I444Buffer(webrtc::scoped_refptr<webrtc::I444BufferInterface> buffer)
    : PlanarYuv8Buffer(buffer) {}

webrtc::I444BufferInterface* I444Buffer::buffer() const {
  return static_cast<webrtc::I444BufferInterface*>(buffer_.get());
}

std::unique_ptr<I444Buffer> I444Buffer::scale(int scaled_width,
                                              int scaled_height) const {
  webrtc::scoped_refptr<webrtc::VideoFrameBuffer> result =
      buffer()->Scale(scaled_width, scaled_height);
  return std::make_unique<I444Buffer>(
      webrtc::scoped_refptr<webrtc::I444BufferInterface>(
          const_cast<webrtc::I444BufferInterface*>(result->GetI444())));
}

I010Buffer::I010Buffer(webrtc::scoped_refptr<webrtc::I010BufferInterface> buffer)
    : PlanarYuv16BBuffer(buffer) {}

webrtc::I010BufferInterface* I010Buffer::buffer() const {
  return static_cast<webrtc::I010BufferInterface*>(buffer_.get());
}

std::unique_ptr<I010Buffer> I010Buffer::scale(int scaled_width,
                                              int scaled_height) const {
  webrtc::scoped_refptr<webrtc::VideoFrameBuffer> result =
      buffer()->Scale(scaled_width, scaled_height);
  return std::make_unique<I010Buffer>(
      webrtc::scoped_refptr<webrtc::I010BufferInterface>(
          const_cast<webrtc::I010BufferInterface*>(result->GetI010())));
}

NV12Buffer::NV12Buffer(webrtc::scoped_refptr<webrtc::NV12BufferInterface> buffer)
    : BiplanarYuv8Buffer(buffer) {}

webrtc::NV12BufferInterface* NV12Buffer::buffer() const {
  return static_cast<webrtc::NV12BufferInterface*>(buffer_.get());
}

std::unique_ptr<NV12Buffer> NV12Buffer::scale(int scaled_width,
                                              int scaled_height) const {
  webrtc::scoped_refptr<webrtc::VideoFrameBuffer> result =
      buffer()->Scale(scaled_width, scaled_height);
  return std::make_unique<NV12Buffer>(
      webrtc::scoped_refptr<webrtc::NV12BufferInterface>(
          const_cast<webrtc::NV12BufferInterface*>(result->GetNV12())));
}

std::unique_ptr<I420Buffer> copy_i420_buffer(
    const std::unique_ptr<I420Buffer>& i420) {
  return std::make_unique<I420Buffer>(webrtc::I420Buffer::Copy(*i420->get()));
}

std::unique_ptr<I420Buffer> new_i420_buffer(int width,
                                            int height,
                                            int stride_y,
                                            int stride_u,
                                            int stride_v) {
  return std::make_unique<I420Buffer>(
      webrtc::I420Buffer::Create(width, height, stride_y, stride_u, stride_v));
}

std::unique_ptr<I422Buffer> new_i422_buffer(int width,
                                            int height,
                                            int stride_y,
                                            int stride_u,
                                            int stride_v) {
  return std::make_unique<I422Buffer>(
      webrtc::I422Buffer::Create(width, height, stride_y, stride_u, stride_v));
}

std::unique_ptr<I444Buffer> new_i444_buffer(int width,
                                            int height,
                                            int stride_y,
                                            int stride_u,
                                            int stride_v) {
  return std::make_unique<I444Buffer>(
      webrtc::I444Buffer::Create(width, height, stride_y, stride_u, stride_v));
}

std::unique_ptr<I010Buffer> new_i010_buffer(int width,
                                            int height,
                                            int stride_y,
                                            int stride_u,
                                            int stride_v) {
  return std::make_unique<I010Buffer>(webrtc::make_ref_counted<webrtc::I010Buffer>(
      width, height, stride_y, stride_u, stride_v));
}

std::unique_ptr<NV12Buffer> new_nv12_buffer(int width,
                                            int height,
                                            int stride_y,
                                            int stride_uv) {
  return std::make_unique<NV12Buffer>(
      webrtc::NV12Buffer::Create(width, height, stride_y, stride_uv));
}

std::unique_ptr<VideoFrameBuffer> new_fluxer_d3d11_texture_buffer(
    uint64_t handle,
    uint32_t width,
    uint32_t height,
    uint32_t dxgi_format) {
  if (handle == 0 || width == 0 || height == 0) {
    return nullptr;
  }
  return std::make_unique<VideoFrameBuffer>(
      webrtc::make_ref_counted<FluxerGpuFrameBuffer>(handle, width, height,
                                                     dxgi_format));
}

std::unique_ptr<VideoFrameBuffer> new_fluxer_dmabuf_texture_buffer(
    int fd0,
    int fd1,
    int fd2,
    int fd3,
    uint32_t plane_count,
    uint32_t width,
    uint32_t height,
    uint32_t drm_format,
    uint64_t modifier,
    uint32_t stride0,
    uint32_t stride1,
    uint32_t stride2,
    uint32_t stride3,
    uint32_t offset0,
    uint32_t offset1,
    uint32_t offset2,
    uint32_t offset3,
    uint64_t device_uuid_hi,
    uint64_t device_uuid_lo) {
  if (plane_count == 0 || plane_count > 4 || width == 0 || height == 0) {
    return nullptr;
  }
  return std::make_unique<VideoFrameBuffer>(
      webrtc::make_ref_counted<FluxerGpuFrameBuffer>(
          fd0, fd1, fd2, fd3, plane_count, width, height, drm_format,
          modifier, stride0, stride1, stride2, stride3, offset0, offset1,
          offset2, offset3, device_uuid_hi, device_uuid_lo));
}

bool is_fluxer_gpu_buffer(const std::unique_ptr<VideoFrameBuffer>& buffer) {
  return buffer && AsFluxerGpuFrameBuffer(buffer->get().get()) != nullptr;
}

uint64_t fluxer_d3d11_texture_handle(
    const std::unique_ptr<VideoFrameBuffer>& buffer) {
  const FluxerGpuFrameBuffer* gpu =
      buffer ? AsFluxerGpuFrameBuffer(buffer->get().get()) : nullptr;
  if (!gpu || gpu->kind() != FluxerGpuFrameBuffer::Kind::kD3D11Texture) {
    return 0;
  }
  return gpu->d3d11_handle();
}

uint32_t fluxer_gpu_buffer_width(
    const std::unique_ptr<VideoFrameBuffer>& buffer) {
  const FluxerGpuFrameBuffer* gpu =
      buffer ? AsFluxerGpuFrameBuffer(buffer->get().get()) : nullptr;
  return gpu ? static_cast<uint32_t>(gpu->width()) : 0;
}

uint32_t fluxer_gpu_buffer_height(
    const std::unique_ptr<VideoFrameBuffer>& buffer) {
  const FluxerGpuFrameBuffer* gpu =
      buffer ? AsFluxerGpuFrameBuffer(buffer->get().get()) : nullptr;
  return gpu ? static_cast<uint32_t>(gpu->height()) : 0;
}

uint32_t fluxer_gpu_buffer_format(
    const std::unique_ptr<VideoFrameBuffer>& buffer) {
  const FluxerGpuFrameBuffer* gpu =
      buffer ? AsFluxerGpuFrameBuffer(buffer->get().get()) : nullptr;
  if (!gpu) {
    return 0;
  }
  return gpu->kind() == FluxerGpuFrameBuffer::Kind::kD3D11Texture
             ? gpu->dxgi_format()
             : gpu->drm_format();
}

#ifndef __APPLE__

std::unique_ptr<VideoFrameBuffer> new_native_buffer_from_platform_image_buffer(
    PlatformImageBuffer *buffer
) {
  return nullptr;
}

PlatformImageBuffer* native_buffer_to_platform_image_buffer(
    const std::unique_ptr<VideoFrameBuffer> &buffer
) {
  return nullptr;
}

#endif

}  // namespace livekit_ffi
