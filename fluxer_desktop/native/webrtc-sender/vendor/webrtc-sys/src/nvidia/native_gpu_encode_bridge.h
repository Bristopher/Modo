#ifndef WEBRTC_NVIDIA_NATIVE_GPU_ENCODE_BRIDGE_H_
#define WEBRTC_NVIDIA_NATIVE_GPU_ENCODE_BRIDGE_H_

#include <cuda.h>

#include <cstdint>
#include <vector>

#include "NvEncoder/NvEncoder.h"
#include "api/video/video_frame.h"

namespace webrtc {

bool TryEncodeNativeGpuFrame(NvEncoder* encoder,
                             CUcontext cu_context,
                             const VideoFrame& input_frame,
                             NV_ENC_PIC_PARAMS* pic_params,
                             std::vector<std::vector<uint8_t>>* bit_stream);

}  // namespace webrtc

#endif  // WEBRTC_NVIDIA_NATIVE_GPU_ENCODE_BRIDGE_H_
