// SPDX-License-Identifier: AGPL-3.0-or-later
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=vendor/webrtc-sys/src/nvidia/NvCodec/include/cuda.h");
    println!("cargo:rustc-check-cfg=cfg(fluxer_linux_nvenc)");
    println!("cargo:rustc-check-cfg=cfg(fluxer_windows_nvenc)");
    println!("cargo:rustc-check-cfg=cfg(fluxer_windows_nvenc_encoder)");
    println!("cargo:rustc-check-cfg=cfg(fluxer_macos_videotoolbox)");

    let cuda_include_dir = vendored_cuda_include_dir();
    let cuda_version = cuda_include_dir
        .as_deref()
        .and_then(read_cuda_version)
        .unwrap_or(0);
    let is_macos = env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos");
    let has_linux_nvenc = env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux")
        && supports_webrtc_sys_nvenc_arch()
        && cuda_version > 0;
    let has_windows_nvenc = env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && supports_webrtc_sys_nvenc_arch();
    let has_windows_nvenc_encoder = has_windows_nvenc;

    println!(
        "cargo:rustc-env=FLUXER_LINUX_NVENC_COMPILED={}",
        if has_linux_nvenc { "1" } else { "0" }
    );
    println!(
        "cargo:rustc-env=FLUXER_WINDOWS_NVENC_COMPILED={}",
        if has_windows_nvenc { "1" } else { "0" }
    );
    println!(
        "cargo:rustc-env=FLUXER_WINDOWS_NVENC_ENCODER_COMPILED={}",
        if has_windows_nvenc_encoder { "1" } else { "0" }
    );
    println!("cargo:rustc-env=FLUXER_CUDA_VERSION={cuda_version}");
    if has_linux_nvenc {
        println!("cargo:rustc-cfg=fluxer_linux_nvenc");
    }
    if has_windows_nvenc {
        println!("cargo:rustc-cfg=fluxer_windows_nvenc");
    }
    if has_windows_nvenc_encoder {
        println!("cargo:rustc-cfg=fluxer_windows_nvenc_encoder");
    }
    println!(
        "cargo:rustc-env=FLUXER_MACOS_VIDEOTOOLBOX_COMPILED={}",
        if is_macos { "1" } else { "0" }
    );
    if is_macos {
        println!("cargo:rustc-cfg=fluxer_macos_videotoolbox");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=CoreMedia");
        println!("cargo:rustc-link-lib=framework=VideoToolbox");
    }

    configure_darwin_objc_linking();
    napi_build::setup();
}

fn configure_darwin_objc_linking() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }

    println!("cargo:rustc-link-arg=-ObjC");
}

fn vendored_cuda_include_dir() -> Option<PathBuf> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").ok()?;
    Some(
        PathBuf::from(manifest_dir)
            .join("vendor")
            .join("webrtc-sys")
            .join("src")
            .join("nvidia")
            .join("NvCodec")
            .join("include"),
    )
}

fn read_cuda_version(include_dir: &Path) -> Option<u32> {
    let content = std::fs::read_to_string(include_dir.join("cuda.h")).ok()?;
    content.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        match (parts.next(), parts.next(), parts.next()) {
            (Some("#define"), Some("CUDA_VERSION"), Some(value)) => value.parse().ok(),
            _ => None,
        }
    })
}

fn supports_webrtc_sys_nvenc_arch() -> bool {
    let Ok(arch) = env::var("CARGO_CFG_TARGET_ARCH") else {
        return false;
    };
    matches!(arch.as_str(), "x86_64" | "i686" | "aarch64") || arch.contains("arm")
}
