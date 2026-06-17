/*
 * SPDX-License-Identifier: MIT
 *
 * Minimal vendored CUDA Driver API declarations used by Fluxer's native
 * LiveKit/WebRTC NVENC bridge. This is intentionally not the CUDA Toolkit
 * header; it keeps the build independent of a system CUDA SDK while still
 * compiling against the NVIDIA driver API that is loaded dynamically at
 * runtime.
 */

#ifndef __cuda_cuda_h__
#define __cuda_cuda_h__

#include <stddef.h>

#ifdef _WIN32
#define CUDAAPI __stdcall
#else
#define CUDAAPI
#endif

#ifdef __cplusplus
extern "C" {
#endif

#ifndef CUDA_VERSION
#define CUDA_VERSION 12000
#endif

typedef enum CUresult_enum {
  CUDA_SUCCESS = 0,
  CUDA_ERROR_INVALID_VALUE = 1,
  CUDA_ERROR_OUT_OF_MEMORY = 2,
  CUDA_ERROR_NOT_INITIALIZED = 3,
  CUDA_ERROR_DEINITIALIZED = 4,
  CUDA_ERROR_NOT_SUPPORTED = 801
} CUresult;

typedef int CUdevice;
typedef struct CUctx_st* CUcontext;
typedef struct CUstream_st* CUstream;
typedef struct CUarray_st* CUarray;
typedef struct CUgraphicsResource_st* CUgraphicsResource;
typedef unsigned long long CUdeviceptr;

typedef enum CUmemorytype_enum {
  CU_MEMORYTYPE_HOST = 0x01,
  CU_MEMORYTYPE_DEVICE = 0x02,
  CU_MEMORYTYPE_ARRAY = 0x03,
  CU_MEMORYTYPE_UNIFIED = 0x04
} CUmemorytype;

typedef enum CUdevice_attribute_enum {
  CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR = 75
} CUdevice_attribute;

typedef enum CUeglFrameType_enum {
  CU_EGL_FRAME_TYPE_ARRAY = 0,
  CU_EGL_FRAME_TYPE_PITCH = 1
} CUeglFrameType;

typedef enum CUeglColorFormat_enum {
  CU_EGL_COLOR_FORMAT_YUV420_PLANAR = 0x00,
  CU_EGL_COLOR_FORMAT_YUV420_SEMIPLANAR = 0x01,
  CU_EGL_COLOR_FORMAT_YUV420_SEMIPLANAR_ER = 0x02,
  CU_EGL_COLOR_FORMAT_YVU420_SEMIPLANAR = 0x03,
  CU_EGL_COLOR_FORMAT_YVU420_SEMIPLANAR_ER = 0x04,
  CU_EGL_COLOR_FORMAT_ARGB = 0x05,
  CU_EGL_COLOR_FORMAT_RGBA = 0x06,
  CU_EGL_COLOR_FORMAT_L = 0x07,
  CU_EGL_COLOR_FORMAT_R = 0x08,
  CU_EGL_COLOR_FORMAT_YUV444_PLANAR = 0x09,
  CU_EGL_COLOR_FORMAT_YUV444_SEMIPLANAR = 0x0a,
  CU_EGL_COLOR_FORMAT_YVU444_SEMIPLANAR = 0x0b,
  CU_EGL_COLOR_FORMAT_Y = 0x0c,
  CU_EGL_COLOR_FORMAT_YUVY = 0x0d,
  CU_EGL_COLOR_FORMAT_UYVY = 0x0e,
  CU_EGL_COLOR_FORMAT_ABGR = 0x0f,
  CU_EGL_COLOR_FORMAT_BGRA = 0x10,
  CU_EGL_COLOR_FORMAT_A = 0x11,
  CU_EGL_COLOR_FORMAT_RG = 0x12,
  CU_EGL_COLOR_FORMAT_AYUV = 0x13,
  CU_EGL_COLOR_FORMAT_YVU444_PLANAR = 0x14,
  CU_EGL_COLOR_FORMAT_YVU422_PLANAR = 0x15,
  CU_EGL_COLOR_FORMAT_YUV422_PLANAR = 0x16,
  CU_EGL_COLOR_FORMAT_YVU422_SEMIPLANAR = 0x17,
  CU_EGL_COLOR_FORMAT_YUV422_SEMIPLANAR = 0x18,
  CU_EGL_COLOR_FORMAT_YUYV = 0x19,
  CU_EGL_COLOR_FORMAT_UYVY_ER = 0x1a,
  CU_EGL_COLOR_FORMAT_YUYV_ER = 0x1b,
  CU_EGL_COLOR_FORMAT_YUVA = 0x1c,
  CU_EGL_COLOR_FORMAT_AYUV_ER = 0x1d,
  CU_EGL_COLOR_FORMAT_YUVA_ER = 0x1e,
  CU_EGL_COLOR_FORMAT_LAST = 0x1f
} CUeglColorFormat;

typedef enum CUarray_format_enum {
  CU_AD_FORMAT_UNSIGNED_INT8 = 0x01,
  CU_AD_FORMAT_UNSIGNED_INT16 = 0x02,
  CU_AD_FORMAT_UNSIGNED_INT32 = 0x03,
  CU_AD_FORMAT_SIGNED_INT8 = 0x08,
  CU_AD_FORMAT_SIGNED_INT16 = 0x09,
  CU_AD_FORMAT_SIGNED_INT32 = 0x0a,
  CU_AD_FORMAT_HALF = 0x10,
  CU_AD_FORMAT_FLOAT = 0x20
} CUarray_format;

typedef struct CUDA_MEMCPY2D_st {
  size_t srcXInBytes;
  size_t srcY;
  CUmemorytype srcMemoryType;
  const void* srcHost;
  CUdeviceptr srcDevice;
  CUarray srcArray;
  size_t srcPitch;
  size_t dstXInBytes;
  size_t dstY;
  CUmemorytype dstMemoryType;
  void* dstHost;
  CUdeviceptr dstDevice;
  CUarray dstArray;
  size_t dstPitch;
  size_t WidthInBytes;
  size_t Height;
} CUDA_MEMCPY2D;

typedef struct CUDA_ARRAY3D_DESCRIPTOR_st {
  size_t Width;
  size_t Height;
  size_t Depth;
  CUarray_format Format;
  unsigned int NumChannels;
  unsigned int Flags;
} CUDA_ARRAY3D_DESCRIPTOR;

typedef struct CUDA_RESOURCE_DESC_st {
  int resType;
  union {
    struct {
      CUarray hArray;
    } array;
    struct {
      CUdeviceptr devPtr;
      CUarray_format format;
      unsigned int numChannels;
      size_t sizeInBytes;
    } linear;
    struct {
      CUdeviceptr devPtr;
      CUarray_format format;
      unsigned int numChannels;
      size_t width;
      size_t height;
      size_t pitchInBytes;
    } pitch2D;
    struct {
      unsigned int reserved[32];
    } reserved;
  } res;
  unsigned int flags;
} CUDA_RESOURCE_DESC;

typedef struct CUeglFrame_st {
  union {
    CUarray pArray[3];
    void* pPitch[3];
  } frame;
  unsigned int width;
  unsigned int height;
  unsigned int depth;
  unsigned int pitch;
  unsigned int planeCount;
  unsigned int numChannels;
  CUeglFrameType frameType;
  CUeglColorFormat eglColorFormat;
  CUarray_format cuFormat;
} CUeglFrame;

#define CU_STREAM_DEFAULT 0
#define CU_GRAPHICS_MAP_RESOURCE_FLAGS_NONE 0x00

CUresult CUDAAPI cuInit(unsigned int flags);
CUresult CUDAAPI cuDriverGetVersion(int* driverVersion);
CUresult CUDAAPI cuGetErrorName(CUresult error, const char** pStr);
CUresult CUDAAPI cuDeviceGetCount(int* count);
CUresult CUDAAPI cuDeviceGet(CUdevice* device, int ordinal);
CUresult CUDAAPI cuDeviceGetName(char* name, int len, CUdevice dev);
CUresult CUDAAPI cuDeviceGetAttribute(int* pi, CUdevice_attribute attrib, CUdevice dev);
CUresult CUDAAPI cuCtxCreate(CUcontext* pctx, unsigned int flags, CUdevice dev);
CUresult CUDAAPI cuCtxCreate_v2(CUcontext* pctx, unsigned int flags, CUdevice dev);
CUresult CUDAAPI cuCtxDestroy(CUcontext ctx);
CUresult CUDAAPI cuCtxDestroy_v2(CUcontext ctx);
CUresult CUDAAPI cuCtxGetCurrent(CUcontext* pctx);
CUresult CUDAAPI cuCtxSetCurrent(CUcontext ctx);
CUresult CUDAAPI cuCtxGetDevice(CUdevice* device);
CUresult CUDAAPI cuCtxPushCurrent(CUcontext ctx);
CUresult CUDAAPI cuCtxPushCurrent_v2(CUcontext ctx);
CUresult CUDAAPI cuCtxPopCurrent(CUcontext* pctx);
CUresult CUDAAPI cuCtxPopCurrent_v2(CUcontext* pctx);
CUresult CUDAAPI cuMemAlloc(CUdeviceptr* dptr, size_t bytesize);
CUresult CUDAAPI cuMemAllocPitch(CUdeviceptr* dptr,
                                 size_t* pPitch,
                                 size_t WidthInBytes,
                                 size_t Height,
                                 unsigned int ElementSizeBytes);
CUresult CUDAAPI cuMemFree(CUdeviceptr dptr);
CUresult CUDAAPI cuMemFree_v2(CUdeviceptr dptr);
CUresult CUDAAPI cuMemcpy2D(const CUDA_MEMCPY2D* pCopy);
CUresult CUDAAPI cuMemcpy2D_v2(const CUDA_MEMCPY2D* pCopy);
CUresult CUDAAPI cuMemcpy2DUnaligned(const CUDA_MEMCPY2D* pCopy);
CUresult CUDAAPI cuMemcpy2DUnaligned_v2(const CUDA_MEMCPY2D* pCopy);
CUresult CUDAAPI cuMemcpy2DAsync(const CUDA_MEMCPY2D* pCopy, CUstream hStream);
CUresult CUDAAPI cuMemcpy2DAsync_v2(const CUDA_MEMCPY2D* pCopy, CUstream hStream);
CUresult CUDAAPI cuStreamCreate(CUstream* phStream, unsigned int Flags);
CUresult CUDAAPI cuStreamSynchronize(CUstream hStream);
CUresult CUDAAPI cuArrayDestroy(CUarray hArray);
CUresult CUDAAPI cuGraphicsEGLRegisterImage(CUgraphicsResource* pCudaResource,
                                            void* image,
                                            unsigned int flags);
CUresult CUDAAPI cuGraphicsResourceGetMappedEglFrame(CUeglFrame* eglFrame,
                                                     CUgraphicsResource resource,
                                                     unsigned int index,
                                                     unsigned int mipLevel);
CUresult CUDAAPI cuGraphicsUnregisterResource(CUgraphicsResource resource);

#ifdef __cplusplus
}
#endif

#endif
