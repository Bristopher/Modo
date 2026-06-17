// SPDX-License-Identifier: AGPL-3.0-or-later

fn main() {
    napi_build::setup();

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=AVFoundation");
    }
}
