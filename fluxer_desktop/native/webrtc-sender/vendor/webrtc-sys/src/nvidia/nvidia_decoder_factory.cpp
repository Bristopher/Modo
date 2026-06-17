#include "nvidia_decoder_factory.h"

#include <modules/video_coding/codecs/h264/include/h264.h>

#include <memory>

#include "cuda_context.h"
#include "h264_decoder_impl.h"
#include "h265_decoder_impl.h"
#include "rtc_base/logging.h"

namespace webrtc {

constexpr char kSdpKeyNameCodecImpl[] = "implementation_name";
constexpr char kCodecName[] = "NvCodec";

static int GetCudaDeviceCapabilityMajorVersion(CUcontext context) {
  cuCtxSetCurrent(context);

  CUdevice device;
  cuCtxGetDevice(&device);

  int major;
  cuDeviceGetAttribute(&major, CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR,
                       device);

  return major;
}

static bool CanDecodeWithNvdec(CUcontext context,
                               cudaVideoCodec codec,
                               const char* codec_name) {
  CUVIDDECODECAPS caps = {};
  caps.eCodecType = codec;
  caps.eChromaFormat = cudaVideoChromaFormat_420;
  caps.nBitDepthMinus8 = 0;

  CUresult result = cuCtxPushCurrent(context);
  if (result != CUDA_SUCCESS) {
    RTC_LOG(LS_WARNING) << "NVDEC " << codec_name
                        << " capability probe failed on cuCtxPushCurrent: "
                        << result;
    return false;
  }

  result = cuvidGetDecoderCaps(&caps);
  CUresult pop_result = cuCtxPopCurrent(nullptr);
  if (pop_result != CUDA_SUCCESS) {
    RTC_LOG(LS_WARNING) << "NVDEC " << codec_name
                        << " capability probe failed on cuCtxPopCurrent: "
                        << pop_result;
    return false;
  }
  if (result != CUDA_SUCCESS) {
    RTC_LOG(LS_WARNING) << "NVDEC " << codec_name
                        << " capability probe failed on cuvidGetDecoderCaps: "
                        << result;
    return false;
  }

  const bool supports_nv12 =
      (caps.nOutputFormatMask & (1 << cudaVideoSurfaceFormat_NV12)) != 0;
  if (!caps.bIsSupported || !supports_nv12) {
    RTC_LOG(LS_WARNING) << "NVDEC " << codec_name
                        << " is unavailable on this GPU.";
    return false;
  }

  return true;
}

std::vector<SdpVideoFormat> SupportedNvDecoderCodecs(CUcontext context) {
  std::vector<SdpVideoFormat> supportedFormats;
  const bool h264_supported =
      CanDecodeWithNvdec(context, cudaVideoCodec_H264, "H264");
  const bool hevc_supported =
      CanDecodeWithNvdec(context, cudaVideoCodec_HEVC, "HEVC");

  // HardwareGeneration Kepler is 3.x
  // https://docs.nvidia.com/deploy/cuda-compatibility/index.html#faq
  // Kepler support h264 profile Main, Highprofile up to Level4.1
  // https://docs.nvidia.com/video-technologies/video-codec-sdk/nvdec-video-decoder-api-prog-guide/index.html#video-decoder-capabilities__table_o3x_fms_3lb
  if (h264_supported && GetCudaDeviceCapabilityMajorVersion(context) <= 3) {
    supportedFormats = {
        CreateH264Format(webrtc::H264Profile::kProfileHigh,
                         webrtc::H264Level::kLevel4_1, "1"),
        CreateH264Format(webrtc::H264Profile::kProfileMain,
                         webrtc::H264Level::kLevel4_1, "1"),
    };
  } else if (h264_supported) {
    supportedFormats = {
        // Constrained Baseline Profile does not support NvDecoder, but WebRTC
        // uses this profile by default,
        // so it must be returned in this method.
        CreateH264Format(webrtc::H264Profile::kProfileConstrainedBaseline,
                         webrtc::H264Level::kLevel5_1, "1"),
        CreateH264Format(webrtc::H264Profile::kProfileBaseline,
                         webrtc::H264Level::kLevel5_1, "1"),
        CreateH264Format(webrtc::H264Profile::kProfileHigh,
                         webrtc::H264Level::kLevel5_1, "1"),
        CreateH264Format(webrtc::H264Profile::kProfileMain,
                         webrtc::H264Level::kLevel5_1, "1"),
    };
  }

  if (hevc_supported) {
    supportedFormats.push_back(SdpVideoFormat("H265"));
    supportedFormats.push_back(SdpVideoFormat("HEVC"));
  }

  for (auto& format : supportedFormats) {
    format.parameters.emplace(kSdpKeyNameCodecImpl, kCodecName);
  }

  return supportedFormats;
}

NvidiaVideoDecoderFactory::NvidiaVideoDecoderFactory()
    : cu_context_(livekit_ffi::CudaContext::GetInstance()) {
  if (cu_context_->IsInitialized() || cu_context_->Initialize()) {
    supported_formats_ = SupportedNvDecoderCodecs(cu_context_->GetContext());
  } else {
    RTC_LOG(LS_ERROR) << "Failed to initialize CUDA context.";
  }
  RTC_LOG(LS_INFO) << "NvidiaVideoDecoderFactory created with "
                   << supported_formats_.size() << " supported formats.";
}

NvidiaVideoDecoderFactory::~NvidiaVideoDecoderFactory() {}

bool NvidiaVideoDecoderFactory::IsSupported() {
  if (!livekit_ffi::CudaContext::IsAvailable()) {
    RTC_LOG(LS_WARNING) << "Cuda Context is not available.";
    return false;
  }

  livekit_ffi::CudaContext* context = livekit_ffi::CudaContext::GetInstance();
  if (!context->IsInitialized() && !context->Initialize()) {
    RTC_LOG(LS_WARNING) << "CUDA context initialization failed, NVDEC disabled.";
    return false;
  }

  const bool supported =
      !SupportedNvDecoderCodecs(context->GetContext()).empty();
  if (supported) {
    std::cout << "Nvidia Decoder is supported." << std::endl;
  } else {
    RTC_LOG(LS_WARNING) << "No supported NVDEC codecs found, NVDEC disabled.";
    context->Shutdown();
  }
  return supported;
}

std::unique_ptr<VideoDecoder> NvidiaVideoDecoderFactory::Create(
    const Environment& env,
    const SdpVideoFormat& format) {
  // Check if the requested format is supported.
  for (const auto& supported_format : supported_formats_) {
    if (format.IsSameCodec(supported_format)) {
      // If the format is supported, create and return the decoder.
      if (!cu_context_) {
        cu_context_ = livekit_ffi::CudaContext::GetInstance();
        if (!cu_context_->Initialize()) {
          RTC_LOG(LS_ERROR) << "Failed to initialize CUDA context.";
          return nullptr;
        }
      }
      if (format.name == "H264") {
        RTC_LOG(LS_INFO) << "Using NVIDIA HW decoder (NVDEC) for H264";
        return std::make_unique<NvidiaH264DecoderImpl>(cu_context_->GetContext());
      }
      if (format.name == "H265" || format.name == "HEVC") {
        RTC_LOG(LS_INFO) << "Using NVIDIA HW decoder (NVDEC) for H265/HEVC";
        return std::make_unique<NvidiaH265DecoderImpl>(cu_context_->GetContext());
      }
    }
  }
  return nullptr;
}

std::vector<SdpVideoFormat> NvidiaVideoDecoderFactory::GetSupportedFormats()
    const {
  return supported_formats_;
}

}  // namespace webrtc
