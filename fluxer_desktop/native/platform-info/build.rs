// SPDX-License-Identifier: AGPL-3.0-or-later

fn main() {
    napi_build::setup();

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
    }
}
