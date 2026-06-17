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

#pragma once

#include <cstdint>
#include <memory>
#include <string>

#include "api/video/i420_buffer.h"
#include "api/video/i422_buffer.h"
#include "api/video/i444_buffer.h"
#include "api/video/i010_buffer.h"
#include "api/video/nv12_buffer.h"
#include "api/video/video_frame_buffer.h"

namespace livekit_ffi {
class VideoFrameBuffer;
class FluxerGpuFrameBuffer;
class PlanarYuvBuffer;
class PlanarYuv8Buffer;
class PlanarYuv16BBuffer;
class BiplanarYuvBuffer;
class BiplanarYuv8Buffer;
class I420Buffer;
class I420ABuffer;
class I422Buffer;
class I444Buffer;
class I010Buffer;
class NV12Buffer;
}  // namespace livekit_ffi

#ifdef __APPLE__
#include <CoreVideo/CoreVideo.h>
namespace livekit_ffi {
typedef __CVBuffer PlatformImageBuffer;
}  // namespace livekit_ffi
#else
namespace livekit_ffi {
typedef void PlatformImageBuffer;
}  // namespace livekit_ffi
#endif

#include "webrtc-sys/src/video_frame_buffer.rs.h"

namespace livekit_ffi {

class VideoFrameBuffer {
 public:
  explicit VideoFrameBuffer(
      webrtc::scoped_refptr<webrtc::VideoFrameBuffer> buffer);

  VideoFrameBufferType buffer_type() const;

  unsigned int width() const;
  unsigned int height() const;

  std::unique_ptr<I420Buffer> to_i420() const;

  // Requires ownership
  std::unique_ptr<I420Buffer> get_i420();
  std::unique_ptr<I420ABuffer> get_i420a();
  std::unique_ptr<I422Buffer> get_i422();
  std::unique_ptr<I444Buffer> get_i444();
  std::unique_ptr<I010Buffer> get_i010();
  std::unique_ptr<NV12Buffer> get_nv12();
  webrtc::scoped_refptr<webrtc::VideoFrameBuffer> get() const;

 protected:
  webrtc::scoped_refptr<webrtc::VideoFrameBuffer> buffer_;
};

class FluxerGpuFrameBuffer : public webrtc::VideoFrameBuffer {
 public:
  enum class Kind {
    kD3D11Texture,
    kDmaBuf,
  };

  FluxerGpuFrameBuffer(uint64_t handle,
                       uint32_t width,
                       uint32_t height,
                       uint32_t dxgi_format);
  FluxerGpuFrameBuffer(int fd0,
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
                       uint64_t device_uuid_lo);
  ~FluxerGpuFrameBuffer() override;

  Type type() const override;
  int width() const override;
  int height() const override;
  webrtc::scoped_refptr<webrtc::I420BufferInterface> ToI420() override;
  webrtc::scoped_refptr<webrtc::VideoFrameBuffer> CropAndScale(
      int offset_x,
      int offset_y,
      int crop_width,
      int crop_height,
      int scaled_width,
      int scaled_height) override;
  webrtc::scoped_refptr<webrtc::VideoFrameBuffer> GetMappedFrameBuffer(
      webrtc::ArrayView<Type> types) override;
  std::string storage_representation() const override;

  Kind kind() const { return kind_; }
  uint64_t d3d11_handle() const { return d3d11_handle_; }
  uint32_t dxgi_format() const { return dxgi_format_; }
  uint32_t drm_format() const { return drm_format_; }
  uint64_t modifier() const { return modifier_; }
  uint32_t plane_count() const { return plane_count_; }
  int fd(uint32_t plane) const;
  uint32_t stride(uint32_t plane) const;
  uint32_t offset(uint32_t plane) const;
  uint64_t device_uuid_hi() const { return device_uuid_hi_; }
  uint64_t device_uuid_lo() const { return device_uuid_lo_; }

 private:
  Kind kind_;
  uint32_t width_;
  uint32_t height_;
  uint64_t d3d11_handle_ = 0;
  uint32_t dxgi_format_ = 0;
  int fds_[4] = {-1, -1, -1, -1};
  uint32_t plane_count_ = 0;
  uint32_t drm_format_ = 0;
  uint64_t modifier_ = 0;
  uint32_t strides_[4] = {0, 0, 0, 0};
  uint32_t offsets_[4] = {0, 0, 0, 0};
  uint64_t device_uuid_hi_ = 0;
  uint64_t device_uuid_lo_ = 0;
};

const FluxerGpuFrameBuffer* AsFluxerGpuFrameBuffer(
    const webrtc::VideoFrameBuffer* buffer);

class PlanarYuvBuffer : public VideoFrameBuffer {
 public:
  explicit PlanarYuvBuffer(webrtc::scoped_refptr<webrtc::PlanarYuvBuffer> buffer);

  unsigned int chroma_width() const;
  unsigned int chroma_height() const;

  unsigned int stride_y() const;
  unsigned int stride_u() const;
  unsigned int stride_v() const;

 private:
  webrtc::PlanarYuvBuffer* buffer() const;
};

class PlanarYuv8Buffer : public PlanarYuvBuffer {
 public:
  explicit PlanarYuv8Buffer(
      webrtc::scoped_refptr<webrtc::PlanarYuv8Buffer> buffer);

  const uint8_t* data_y() const;
  const uint8_t* data_u() const;
  const uint8_t* data_v() const;

 private:
  webrtc::PlanarYuv8Buffer* buffer() const;
};

class PlanarYuv16BBuffer : public PlanarYuvBuffer {
 public:
  explicit PlanarYuv16BBuffer(
      webrtc::scoped_refptr<webrtc::PlanarYuv16BBuffer> buffer);

  const uint16_t* data_y() const;
  const uint16_t* data_u() const;
  const uint16_t* data_v() const;

 private:
  webrtc::PlanarYuv16BBuffer* buffer() const;
};

class BiplanarYuvBuffer : public VideoFrameBuffer {
 public:
  explicit BiplanarYuvBuffer(
      webrtc::scoped_refptr<webrtc::BiplanarYuvBuffer> buffer);

  unsigned int chroma_width() const;
  unsigned int chroma_height() const;

  unsigned int stride_y() const;
  unsigned int stride_uv() const;

 private:
  webrtc::BiplanarYuvBuffer* buffer() const;
};

class BiplanarYuv8Buffer : public BiplanarYuvBuffer {
 public:
  explicit BiplanarYuv8Buffer(
      webrtc::scoped_refptr<webrtc::BiplanarYuv8Buffer> buffer);

  const uint8_t* data_y() const;
  const uint8_t* data_uv() const;

 private:
  webrtc::BiplanarYuv8Buffer* buffer() const;
};

class I420Buffer : public PlanarYuv8Buffer {
 public:
  explicit I420Buffer(webrtc::scoped_refptr<webrtc::I420BufferInterface> buffer);

  std::unique_ptr<I420Buffer> scale(int scaled_width, int scaled_height) const;

 private:
  webrtc::I420BufferInterface* buffer() const;
};

class I420ABuffer : public I420Buffer {
 public:
  explicit I420ABuffer(webrtc::scoped_refptr<webrtc::I420ABufferInterface> buffer);

  unsigned int stride_a() const;
  const uint8_t* data_a() const;

  std::unique_ptr<I420ABuffer> scale(int scaled_width, int scaled_height) const;

 private:
  webrtc::I420ABufferInterface* buffer() const;
};

class I422Buffer : public PlanarYuv8Buffer {
 public:
  explicit I422Buffer(webrtc::scoped_refptr<webrtc::I422BufferInterface> buffer);

  std::unique_ptr<I422Buffer> scale(int scaled_width, int scaled_height) const;

 private:
  webrtc::I422BufferInterface* buffer() const;
};

class I444Buffer : public PlanarYuv8Buffer {
 public:
  explicit I444Buffer(webrtc::scoped_refptr<webrtc::I444BufferInterface> buffer);

  std::unique_ptr<I444Buffer> scale(int scaled_width, int scaled_height) const;

 private:
  webrtc::I444BufferInterface* buffer() const;
};

class I010Buffer : public PlanarYuv16BBuffer {
 public:
  explicit I010Buffer(webrtc::scoped_refptr<webrtc::I010BufferInterface> buffer);

  std::unique_ptr<I010Buffer> scale(int scaled_width, int scaled_height) const;

 private:
  webrtc::I010BufferInterface* buffer() const;
};

class NV12Buffer : public BiplanarYuv8Buffer {
 public:
  explicit NV12Buffer(webrtc::scoped_refptr<webrtc::NV12BufferInterface> buffer);

  std::unique_ptr<NV12Buffer> scale(int scaled_width, int scaled_height) const;

 private:
  webrtc::NV12BufferInterface* buffer() const;
};

std::unique_ptr<I420Buffer> copy_i420_buffer(
    const std::unique_ptr<I420Buffer>& i420);
std::unique_ptr<I420Buffer> new_i420_buffer(int width, int height, int stride_y, int stride_u, int stride_v);
std::unique_ptr<I422Buffer> new_i422_buffer(int width, int height, int stride_y, int stride_u, int stride_v);
std::unique_ptr<I444Buffer> new_i444_buffer(int width, int height, int stride_y, int stride_u, int stride_v);
std::unique_ptr<I010Buffer> new_i010_buffer(int width, int height, int stride_y, int stride_u, int stride_v);
std::unique_ptr<NV12Buffer> new_nv12_buffer(int width, int height, int stride_y, int stride_uv);

std::unique_ptr<VideoFrameBuffer> new_fluxer_d3d11_texture_buffer(
    uint64_t handle, uint32_t width, uint32_t height, uint32_t dxgi_format);
std::unique_ptr<VideoFrameBuffer> new_fluxer_dmabuf_texture_buffer(
    int fd0, int fd1, int fd2, int fd3, uint32_t plane_count,
    uint32_t width, uint32_t height, uint32_t drm_format, uint64_t modifier,
    uint32_t stride0, uint32_t stride1, uint32_t stride2, uint32_t stride3,
    uint32_t offset0, uint32_t offset1, uint32_t offset2, uint32_t offset3,
    uint64_t device_uuid_hi, uint64_t device_uuid_lo);
bool is_fluxer_gpu_buffer(const std::unique_ptr<VideoFrameBuffer>& buffer);
uint64_t fluxer_d3d11_texture_handle(const std::unique_ptr<VideoFrameBuffer>& buffer);
uint32_t fluxer_gpu_buffer_width(const std::unique_ptr<VideoFrameBuffer>& buffer);
uint32_t fluxer_gpu_buffer_height(const std::unique_ptr<VideoFrameBuffer>& buffer);
uint32_t fluxer_gpu_buffer_format(const std::unique_ptr<VideoFrameBuffer>& buffer);

std::unique_ptr<VideoFrameBuffer> new_native_buffer_from_platform_image_buffer(PlatformImageBuffer *buffer);
PlatformImageBuffer* native_buffer_to_platform_image_buffer(const std::unique_ptr<VideoFrameBuffer> &);

static const VideoFrameBuffer* yuv_to_vfb(const PlanarYuvBuffer* yuv) {
  return yuv;
}

static const VideoFrameBuffer* biyuv_to_vfb(const BiplanarYuvBuffer* biyuv) {
  return biyuv;
}

static const PlanarYuvBuffer* yuv8_to_yuv(const PlanarYuv8Buffer* yuv8) {
  return yuv8;
}

static const PlanarYuvBuffer* yuv16b_to_yuv(const PlanarYuv16BBuffer* yuv16) {
  return yuv16;
}

static const BiplanarYuvBuffer* biyuv8_to_biyuv(
    const BiplanarYuv8Buffer* biyuv8) {
  return biyuv8;
}

static const PlanarYuv8Buffer* i420_to_yuv8(const I420Buffer* i420) {
  return i420;
}

static const PlanarYuv8Buffer* i420a_to_yuv8(const I420ABuffer* i420a) {
  return i420a;
}

static const PlanarYuv8Buffer* i422_to_yuv8(const I422Buffer* i422) {
  return i422;
}

static const PlanarYuv8Buffer* i444_to_yuv8(const I444Buffer* i444) {
  return i444;
}

static const PlanarYuv16BBuffer* i010_to_yuv16b(const I010Buffer* i010) {
  return i010;
}

static const BiplanarYuv8Buffer* nv12_to_biyuv8(const NV12Buffer* nv12) {
  return nv12;
}

static std::unique_ptr<VideoFrameBuffer> _unique_video_frame_buffer() {
  return nullptr;
}

}  // namespace livekit_ffi
