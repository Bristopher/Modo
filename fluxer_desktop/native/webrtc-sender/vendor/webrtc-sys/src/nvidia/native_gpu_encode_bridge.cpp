#include "native_gpu_encode_bridge.h"

#include "livekit/video_frame_buffer.h"
#include "rtc_base/logging.h"

#if defined(__linux__)
#include <EGL/egl.h>
#include <EGL/eglext.h>
#include <unistd.h>

#include <cstdint>
#include <vector>
#endif

namespace webrtc {
namespace {

constexpr uint32_t FourCc(char a, char b, char c, char d) {
  return static_cast<uint32_t>(a) | (static_cast<uint32_t>(b) << 8) |
         (static_cast<uint32_t>(c) << 16) |
         (static_cast<uint32_t>(d) << 24);
}

constexpr uint32_t kDrmFormatNv12 = FourCc('N', 'V', '1', '2');
constexpr uint32_t kDrmFormatXrgb8888 = FourCc('X', 'R', '2', '4');
constexpr uint32_t kDrmFormatArgb8888 = FourCc('A', 'R', '2', '4');
constexpr uint32_t kDrmFormatXbgr8888 = FourCc('X', 'B', '2', '4');
constexpr uint32_t kDrmFormatAbgr8888 = FourCc('A', 'B', '2', '4');
constexpr uint32_t kDrmFormatXrgb2101010 = FourCc('X', 'R', '3', '0');
constexpr uint32_t kDrmFormatArgb2101010 = FourCc('A', 'R', '3', '0');
constexpr uint32_t kDrmFormatXbgr2101010 = FourCc('X', 'B', '3', '0');
constexpr uint32_t kDrmFormatAbgr2101010 = FourCc('A', 'B', '3', '0');

#if defined(__linux__)

class ScopedCudaContext {
 public:
  explicit ScopedCudaContext(CUcontext context) : pushed_(false) {
    if (context && cuCtxPushCurrent(context) == CUDA_SUCCESS) {
      pushed_ = true;
    }
  }

  ~ScopedCudaContext() {
    if (pushed_) {
      CUcontext popped = nullptr;
      cuCtxPopCurrent(&popped);
    }
  }

  bool ok() const { return pushed_; }

 private:
  bool pushed_;
};

class ScopedCudaGraphicsResource {
 public:
  ~ScopedCudaGraphicsResource() {
    if (resource_) {
      cuGraphicsUnregisterResource(resource_);
    }
  }

  CUgraphicsResource* receive() { return &resource_; }
  CUgraphicsResource get() const { return resource_; }

 private:
  CUgraphicsResource resource_ = nullptr;
};

class ScopedEglImage {
 public:
  ScopedEglImage(EGLDisplay display, EGLImageKHR image)
      : display_(display), image_(image) {}

  ~ScopedEglImage() {
    if (display_ != EGL_NO_DISPLAY && image_ != EGL_NO_IMAGE_KHR) {
      auto destroy_image = reinterpret_cast<PFNEGLDESTROYIMAGEKHRPROC>(
          eglGetProcAddress("eglDestroyImageKHR"));
      if (destroy_image) {
        destroy_image(display_, image_);
      }
    }
  }

  EGLImageKHR get() const { return image_; }

 private:
  EGLDisplay display_ = EGL_NO_DISPLAY;
  EGLImageKHR image_ = EGL_NO_IMAGE_KHR;
};

bool EnsureEglInitialized(EGLDisplay* out_display) {
  EGLDisplay display = eglGetDisplay(EGL_DEFAULT_DISPLAY);
  if (display == EGL_NO_DISPLAY) {
    RTC_LOG(LS_WARNING) << "Fluxer native GPU encode: no EGL display";
    return false;
  }
  EGLint major = 0;
  EGLint minor = 0;
  if (!eglInitialize(display, &major, &minor)) {
    RTC_LOG(LS_WARNING) << "Fluxer native GPU encode: eglInitialize failed";
    return false;
  }
  *out_display = display;
  return true;
}

void AppendPlaneAttributes(std::vector<EGLint>* attrs,
                           int plane,
                           int fd,
                           uint32_t offset,
                           uint32_t stride,
                           uint64_t modifier) {
  const EGLint fd_attrs[4] = {EGL_DMA_BUF_PLANE0_FD_EXT,
                              EGL_DMA_BUF_PLANE1_FD_EXT,
                              EGL_DMA_BUF_PLANE2_FD_EXT,
                              EGL_DMA_BUF_PLANE3_FD_EXT};
  const EGLint offset_attrs[4] = {EGL_DMA_BUF_PLANE0_OFFSET_EXT,
                                  EGL_DMA_BUF_PLANE1_OFFSET_EXT,
                                  EGL_DMA_BUF_PLANE2_OFFSET_EXT,
                                  EGL_DMA_BUF_PLANE3_OFFSET_EXT};
  const EGLint pitch_attrs[4] = {EGL_DMA_BUF_PLANE0_PITCH_EXT,
                                 EGL_DMA_BUF_PLANE1_PITCH_EXT,
                                 EGL_DMA_BUF_PLANE2_PITCH_EXT,
                                 EGL_DMA_BUF_PLANE3_PITCH_EXT};
  const EGLint mod_lo_attrs[4] = {EGL_DMA_BUF_PLANE0_MODIFIER_LO_EXT,
                                  EGL_DMA_BUF_PLANE1_MODIFIER_LO_EXT,
                                  EGL_DMA_BUF_PLANE2_MODIFIER_LO_EXT,
                                  EGL_DMA_BUF_PLANE3_MODIFIER_LO_EXT};
  const EGLint mod_hi_attrs[4] = {EGL_DMA_BUF_PLANE0_MODIFIER_HI_EXT,
                                  EGL_DMA_BUF_PLANE1_MODIFIER_HI_EXT,
                                  EGL_DMA_BUF_PLANE2_MODIFIER_HI_EXT,
                                  EGL_DMA_BUF_PLANE3_MODIFIER_HI_EXT};

  attrs->push_back(fd_attrs[plane]);
  attrs->push_back(fd);
  attrs->push_back(offset_attrs[plane]);
  attrs->push_back(static_cast<EGLint>(offset));
  attrs->push_back(pitch_attrs[plane]);
  attrs->push_back(static_cast<EGLint>(stride));
  if (modifier != 0) {
    attrs->push_back(mod_lo_attrs[plane]);
    attrs->push_back(static_cast<EGLint>(modifier & 0xffffffffu));
    attrs->push_back(mod_hi_attrs[plane]);
    attrs->push_back(static_cast<EGLint>((modifier >> 32) & 0xffffffffu));
  }
}

EGLImageKHR CreateDmabufEglImage(
    EGLDisplay display,
    const livekit_ffi::FluxerGpuFrameBuffer& native) {
  auto create_image = reinterpret_cast<PFNEGLCREATEIMAGEKHRPROC>(
      eglGetProcAddress("eglCreateImageKHR"));
  if (!create_image) {
    RTC_LOG(LS_WARNING)
        << "Fluxer native GPU encode: eglCreateImageKHR unavailable";
    return EGL_NO_IMAGE_KHR;
  }

  std::vector<EGLint> attrs = {
      EGL_WIDTH,
      native.width(),
      EGL_HEIGHT,
      native.height(),
      EGL_LINUX_DRM_FOURCC_EXT,
      static_cast<EGLint>(native.drm_format()),
  };
  const uint32_t plane_count = native.plane_count();
  if (plane_count == 0 || plane_count > 4) {
    return EGL_NO_IMAGE_KHR;
  }
  for (uint32_t plane = 0; plane < plane_count; ++plane) {
    if (native.fd(plane) < 0 || native.stride(plane) == 0) {
      return EGL_NO_IMAGE_KHR;
    }
    AppendPlaneAttributes(&attrs, static_cast<int>(plane), native.fd(plane),
                          native.offset(plane), native.stride(plane),
                          native.modifier());
  }
  attrs.push_back(EGL_NONE);

  return create_image(display, EGL_NO_CONTEXT, EGL_LINUX_DMA_BUF_EXT, nullptr,
                      attrs.data());
}

bool DrmFormatToNvencFormat(uint32_t drm_format,
                            NV_ENC_BUFFER_FORMAT* nvenc_format) {
  switch (drm_format) {
    case kDrmFormatNv12:
      *nvenc_format = NV_ENC_BUFFER_FORMAT_NV12;
      return true;
    case kDrmFormatXrgb8888:
    case kDrmFormatArgb8888:
      *nvenc_format = NV_ENC_BUFFER_FORMAT_ARGB;
      return true;
    case kDrmFormatXbgr8888:
    case kDrmFormatAbgr8888:
      *nvenc_format = NV_ENC_BUFFER_FORMAT_ABGR;
      return true;
    case kDrmFormatXrgb2101010:
    case kDrmFormatArgb2101010:
      *nvenc_format = NV_ENC_BUFFER_FORMAT_ARGB10;
      return true;
    case kDrmFormatXbgr2101010:
    case kDrmFormatAbgr2101010:
      *nvenc_format = NV_ENC_BUFFER_FORMAT_ABGR10;
      return true;
    default:
      return false;
  }
}

bool EglFrameLooksLikeSupportedInput(
    const livekit_ffi::FluxerGpuFrameBuffer& native,
    const CUeglFrame& egl_frame) {
  if (egl_frame.frameType != CU_EGL_FRAME_TYPE_PITCH ||
      egl_frame.width != static_cast<unsigned int>(native.width()) ||
      egl_frame.height != static_cast<unsigned int>(native.height()) ||
      egl_frame.pitch == 0 || !egl_frame.frame.pPitch[0]) {
    return false;
  }
  if (native.drm_format() != kDrmFormatNv12) {
    return egl_frame.planeCount == 1;
  }
  if (egl_frame.planeCount <= 1) {
    return true;
  }
  const uintptr_t y = reinterpret_cast<uintptr_t>(egl_frame.frame.pPitch[0]);
  const uintptr_t uv = reinterpret_cast<uintptr_t>(egl_frame.frame.pPitch[1]);
  return uv == y + static_cast<uintptr_t>(egl_frame.pitch) *
                     static_cast<uintptr_t>(native.height());
}

bool TryEncodeDmabuf(NvEncoder* encoder,
                     CUcontext cu_context,
                     const livekit_ffi::FluxerGpuFrameBuffer& native,
                     NV_ENC_PIC_PARAMS* pic_params,
                     std::vector<std::vector<uint8_t>>* bit_stream) {
  NV_ENC_BUFFER_FORMAT nvenc_format = NV_ENC_BUFFER_FORMAT_UNDEFINED;
  if (encoder->GetDeviceType() != NV_ENC_DEVICE_TYPE_CUDA ||
      !DrmFormatToNvencFormat(native.drm_format(), &nvenc_format)) {
    return false;
  }

  EGLDisplay display = EGL_NO_DISPLAY;
  if (!EnsureEglInitialized(&display)) {
    return false;
  }
  ScopedEglImage image(display, CreateDmabufEglImage(display, native));
  if (image.get() == EGL_NO_IMAGE_KHR) {
    RTC_LOG(LS_WARNING)
        << "Fluxer native GPU encode: failed to import DMA-BUF as EGLImage";
    return false;
  }

  ScopedCudaContext current(cu_context);
  if (!current.ok()) {
    RTC_LOG(LS_WARNING)
        << "Fluxer native GPU encode: failed to make CUDA context current";
    return false;
  }

  ScopedCudaGraphicsResource resource;
  CUresult cu_result = cuGraphicsEGLRegisterImage(
      resource.receive(), image.get(), CU_GRAPHICS_MAP_RESOURCE_FLAGS_NONE);
  if (cu_result != CUDA_SUCCESS) {
    const char* name = nullptr;
    cuGetErrorName(cu_result, &name);
    RTC_LOG(LS_WARNING)
        << "Fluxer native GPU encode: cuGraphicsEGLRegisterImage failed: "
        << (name ? name : "unknown");
    return false;
  }

  CUeglFrame egl_frame = {};
  cu_result = cuGraphicsResourceGetMappedEglFrame(&egl_frame, resource.get(),
                                                  0, 0);
  if (cu_result != CUDA_SUCCESS) {
    const char* name = nullptr;
    cuGetErrorName(cu_result, &name);
    RTC_LOG(LS_WARNING)
        << "Fluxer native GPU encode: cuGraphicsResourceGetMappedEglFrame "
           "failed: "
        << (name ? name : "unknown");
    return false;
  }

  if (!EglFrameLooksLikeSupportedInput(native, egl_frame)) {
    RTC_LOG(LS_WARNING)
        << "Fluxer native GPU encode: imported DMA-BUF has unsupported layout";
    return false;
  }

  NvEncExternalInputFrame external = {};
  external.resource = egl_frame.frame.pPitch[0];
  external.resourceType = NV_ENC_INPUT_RESOURCE_TYPE_CUDADEVICEPTR;
  external.width = native.width();
  external.height = native.height();
  external.pitch = static_cast<int>(egl_frame.pitch);
  external.bufferFormat = nvenc_format;

  encoder->EncodeExternalFrame(external, *bit_stream, pic_params);
  return true;
}

#endif  // defined(__linux__)

}  // namespace

bool TryEncodeNativeGpuFrame(NvEncoder* encoder,
                             CUcontext cu_context,
                             const VideoFrame& input_frame,
                             NV_ENC_PIC_PARAMS* pic_params,
                             std::vector<std::vector<uint8_t>>* bit_stream) {
  if (!encoder || !bit_stream) {
    return false;
  }
  const auto input_buffer = input_frame.video_frame_buffer();
  const auto* native =
      livekit_ffi::AsFluxerGpuFrameBuffer(input_buffer ? input_buffer.get()
                                                       : nullptr);
  if (!native) {
    return false;
  }

#if defined(__linux__)
  if (native->kind() == livekit_ffi::FluxerGpuFrameBuffer::Kind::kDmaBuf) {
    return TryEncodeDmabuf(encoder, cu_context, *native, pic_params,
                           bit_stream);
  }
#endif

  return false;
}

}  // namespace webrtc
