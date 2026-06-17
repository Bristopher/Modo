// SPDX-License-Identifier: AGPL-3.0-or-later

fn main() {
    napi_build::setup();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    match target_os.as_str() {
        "macos" => {}
        "windows" => {
            println!("cargo:rustc-link-arg-cdylib=/DELAYLOAD:webauthn.dll");
            println!("cargo:rustc-link-arg-cdylib=delayimp.lib");
        }
        "linux" => {
            println!("cargo:rustc-link-arg-cdylib=-Wl,--disable-new-dtags");
            println!("cargo:rustc-link-arg-cdylib=-Wl,-rpath,$ORIGIN");
        }
        _ => {}
    }
}
