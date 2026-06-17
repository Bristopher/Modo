#if defined(_WIN32)

#include "cuda.h"

#include <windows.h>

namespace {

HMODULE CudaModule() {
  static HMODULE module = LoadLibraryA("nvcuda.dll");
  return module;
}

FARPROC CudaProc(const char* name) {
  HMODULE module = CudaModule();
  if (!module) {
    return nullptr;
  }
  return GetProcAddress(module, name);
}

template <typename Fn>
Fn Resolve(const char* primary, const char* fallback = nullptr) {
  FARPROC proc = CudaProc(primary);
  if (!proc && fallback) {
    proc = CudaProc(fallback);
  }
  return reinterpret_cast<Fn>(proc);
}

template <typename Fn, typename... Args>
CUresult Call(Fn fn, Args... args) {
  if (!fn) {
    return CUDA_ERROR_NOT_INITIALIZED;
  }
  return fn(args...);
}

}  // namespace

extern "C" {

CUresult CUDAAPI cuInit(unsigned int flags) {
  using Fn = CUresult(CUDAAPI*)(unsigned int);
  return Call(Resolve<Fn>("cuInit"), flags);
}

CUresult CUDAAPI cuDriverGetVersion(int* driverVersion) {
  using Fn = CUresult(CUDAAPI*)(int*);
  return Call(Resolve<Fn>("cuDriverGetVersion"), driverVersion);
}

CUresult CUDAAPI cuGetErrorName(CUresult error, const char** pStr) {
  using Fn = CUresult(CUDAAPI*)(CUresult, const char**);
  Fn fn = Resolve<Fn>("cuGetErrorName");
  if (fn) {
    return fn(error, pStr);
  }
  if (pStr) {
    *pStr = "CUDA driver API unavailable";
  }
  return CUDA_ERROR_NOT_INITIALIZED;
}

CUresult CUDAAPI cuDeviceGetCount(int* count) {
  using Fn = CUresult(CUDAAPI*)(int*);
  return Call(Resolve<Fn>("cuDeviceGetCount"), count);
}

CUresult CUDAAPI cuDeviceGet(CUdevice* device, int ordinal) {
  using Fn = CUresult(CUDAAPI*)(CUdevice*, int);
  return Call(Resolve<Fn>("cuDeviceGet"), device, ordinal);
}

CUresult CUDAAPI cuDeviceGetName(char* name, int len, CUdevice dev) {
  using Fn = CUresult(CUDAAPI*)(char*, int, CUdevice);
  return Call(Resolve<Fn>("cuDeviceGetName"), name, len, dev);
}

CUresult CUDAAPI cuDeviceGetAttribute(int* pi,
                                      CUdevice_attribute attrib,
                                      CUdevice dev) {
  using Fn = CUresult(CUDAAPI*)(int*, CUdevice_attribute, CUdevice);
  return Call(Resolve<Fn>("cuDeviceGetAttribute"), pi, attrib, dev);
}

CUresult CUDAAPI cuCtxCreate(CUcontext* pctx,
                             unsigned int flags,
                             CUdevice dev) {
  using Fn = CUresult(CUDAAPI*)(CUcontext*, unsigned int, CUdevice);
  return Call(Resolve<Fn>("cuCtxCreate_v2", "cuCtxCreate"), pctx, flags, dev);
}

CUresult CUDAAPI cuCtxCreate_v2(CUcontext* pctx,
                                unsigned int flags,
                                CUdevice dev) {
  return cuCtxCreate(pctx, flags, dev);
}

CUresult CUDAAPI cuCtxDestroy(CUcontext ctx) {
  using Fn = CUresult(CUDAAPI*)(CUcontext);
  return Call(Resolve<Fn>("cuCtxDestroy_v2", "cuCtxDestroy"), ctx);
}

CUresult CUDAAPI cuCtxDestroy_v2(CUcontext ctx) {
  return cuCtxDestroy(ctx);
}

CUresult CUDAAPI cuCtxGetCurrent(CUcontext* pctx) {
  using Fn = CUresult(CUDAAPI*)(CUcontext*);
  return Call(Resolve<Fn>("cuCtxGetCurrent"), pctx);
}

CUresult CUDAAPI cuCtxSetCurrent(CUcontext ctx) {
  using Fn = CUresult(CUDAAPI*)(CUcontext);
  return Call(Resolve<Fn>("cuCtxSetCurrent"), ctx);
}

CUresult CUDAAPI cuCtxGetDevice(CUdevice* device) {
  using Fn = CUresult(CUDAAPI*)(CUdevice*);
  return Call(Resolve<Fn>("cuCtxGetDevice"), device);
}

CUresult CUDAAPI cuCtxPushCurrent(CUcontext ctx) {
  using Fn = CUresult(CUDAAPI*)(CUcontext);
  return Call(Resolve<Fn>("cuCtxPushCurrent_v2", "cuCtxPushCurrent"), ctx);
}

CUresult CUDAAPI cuCtxPushCurrent_v2(CUcontext ctx) {
  return cuCtxPushCurrent(ctx);
}

CUresult CUDAAPI cuCtxPopCurrent(CUcontext* pctx) {
  using Fn = CUresult(CUDAAPI*)(CUcontext*);
  return Call(Resolve<Fn>("cuCtxPopCurrent_v2", "cuCtxPopCurrent"), pctx);
}

CUresult CUDAAPI cuCtxPopCurrent_v2(CUcontext* pctx) {
  return cuCtxPopCurrent(pctx);
}

CUresult CUDAAPI cuMemAlloc(CUdeviceptr* dptr, size_t bytesize) {
  using Fn = CUresult(CUDAAPI*)(CUdeviceptr*, size_t);
  return Call(Resolve<Fn>("cuMemAlloc_v2", "cuMemAlloc"), dptr, bytesize);
}

CUresult CUDAAPI cuMemAllocPitch(CUdeviceptr* dptr,
                                 size_t* pPitch,
                                 size_t WidthInBytes,
                                 size_t Height,
                                 unsigned int ElementSizeBytes) {
  using Fn = CUresult(CUDAAPI*)(CUdeviceptr*, size_t*, size_t, size_t,
                                unsigned int);
  return Call(Resolve<Fn>("cuMemAllocPitch_v2", "cuMemAllocPitch"), dptr,
              pPitch, WidthInBytes, Height, ElementSizeBytes);
}

CUresult CUDAAPI cuMemFree(CUdeviceptr dptr) {
  using Fn = CUresult(CUDAAPI*)(CUdeviceptr);
  return Call(Resolve<Fn>("cuMemFree_v2", "cuMemFree"), dptr);
}

CUresult CUDAAPI cuMemFree_v2(CUdeviceptr dptr) {
  return cuMemFree(dptr);
}

CUresult CUDAAPI cuMemcpy2D(const CUDA_MEMCPY2D* pCopy) {
  using Fn = CUresult(CUDAAPI*)(const CUDA_MEMCPY2D*);
  return Call(Resolve<Fn>("cuMemcpy2D_v2", "cuMemcpy2D"), pCopy);
}

CUresult CUDAAPI cuMemcpy2D_v2(const CUDA_MEMCPY2D* pCopy) {
  return cuMemcpy2D(pCopy);
}

CUresult CUDAAPI cuMemcpy2DUnaligned(const CUDA_MEMCPY2D* pCopy) {
  using Fn = CUresult(CUDAAPI*)(const CUDA_MEMCPY2D*);
  return Call(Resolve<Fn>("cuMemcpy2DUnaligned_v2",
                          "cuMemcpy2DUnaligned"),
              pCopy);
}

CUresult CUDAAPI cuMemcpy2DUnaligned_v2(const CUDA_MEMCPY2D* pCopy) {
  return cuMemcpy2DUnaligned(pCopy);
}

CUresult CUDAAPI cuMemcpy2DAsync(const CUDA_MEMCPY2D* pCopy,
                                 CUstream hStream) {
  using Fn = CUresult(CUDAAPI*)(const CUDA_MEMCPY2D*, CUstream);
  return Call(Resolve<Fn>("cuMemcpy2DAsync_v2", "cuMemcpy2DAsync"), pCopy,
              hStream);
}

CUresult CUDAAPI cuMemcpy2DAsync_v2(const CUDA_MEMCPY2D* pCopy,
                                    CUstream hStream) {
  return cuMemcpy2DAsync(pCopy, hStream);
}

CUresult CUDAAPI cuStreamCreate(CUstream* phStream, unsigned int Flags) {
  using Fn = CUresult(CUDAAPI*)(CUstream*, unsigned int);
  return Call(Resolve<Fn>("cuStreamCreate"), phStream, Flags);
}

CUresult CUDAAPI cuStreamSynchronize(CUstream hStream) {
  using Fn = CUresult(CUDAAPI*)(CUstream);
  return Call(Resolve<Fn>("cuStreamSynchronize"), hStream);
}

CUresult CUDAAPI cuArrayDestroy(CUarray hArray) {
  using Fn = CUresult(CUDAAPI*)(CUarray);
  return Call(Resolve<Fn>("cuArrayDestroy_v2", "cuArrayDestroy"), hArray);
}

CUresult CUDAAPI cuGraphicsEGLRegisterImage(CUgraphicsResource* pCudaResource,
                                            void* image,
                                            unsigned int flags) {
  using Fn = CUresult(CUDAAPI*)(CUgraphicsResource*, void*, unsigned int);
  return Call(Resolve<Fn>("cuGraphicsEGLRegisterImage"), pCudaResource, image,
              flags);
}

CUresult CUDAAPI cuGraphicsResourceGetMappedEglFrame(
    CUeglFrame* eglFrame,
    CUgraphicsResource resource,
    unsigned int index,
    unsigned int mipLevel) {
  using Fn = CUresult(CUDAAPI*)(CUeglFrame*, CUgraphicsResource, unsigned int,
                                unsigned int);
  return Call(Resolve<Fn>("cuGraphicsResourceGetMappedEglFrame"), eglFrame,
              resource, index, mipLevel);
}

CUresult CUDAAPI cuGraphicsUnregisterResource(CUgraphicsResource resource) {
  using Fn = CUresult(CUDAAPI*)(CUgraphicsResource);
  return Call(Resolve<Fn>("cuGraphicsUnregisterResource"), resource);
}

}  // extern "C"

#endif  // defined(_WIN32)
