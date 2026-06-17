// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR missing"));

    println!("cargo:rerun-if-changed=src/styles/app.css");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=openapi-admin.json");

    generate_admin_api(&manifest_dir, &out_dir);
    build_tailwind(&manifest_dir, &out_dir);
}

fn generate_admin_api(manifest_dir: &Path, out_dir: &Path) {
    let spec_path = manifest_dir.join("openapi-admin.json");

    if !spec_path.exists() {
        eprintln!("cargo:warning=openapi-admin.json not found, skipping API generation");
        return;
    }

    let json_str = fs::read_to_string(&spec_path).expect("failed to read openapi-admin.json");
    let spec: openapiv3::OpenAPI =
        serde_json::from_str(&json_str).expect("failed to parse openapi-admin.json");

    let mut settings = progenitor::GenerationSettings::new();
    settings.with_interface(progenitor::InterfaceStyle::Positional);

    let mut generator = progenitor::Generator::new(&settings);
    let tokens = generator
        .generate_tokens(&spec)
        .expect("failed to generate admin API client");

    let content = prettyplease::unparse(
        &syn::parse2::<syn::File>(tokens).expect("failed to parse generated tokens"),
    );

    let output_path = out_dir.join("admin_api_generated.rs");
    fs::write(&output_path, content).expect("failed to write generated API code");
}

fn build_tailwind(manifest_dir: &Path, out_dir: &Path) {
    let output_dir = out_dir.join("static");
    fs::create_dir_all(&output_dir).expect("failed to create generated static dir");
    let input = manifest_dir.join("src/styles/app.css");
    let output = output_dir.join("app.css");
    let candidates = [
        manifest_dir.join("node_modules/.bin/tailwindcss"),
        manifest_dir.join("../node_modules/.bin/tailwindcss"),
    ];
    let Some(cli) = candidates.iter().find(|c| c.exists()) else {
        panic!(
            "tailwindcss CLI not found. Run `pnpm install` in fluxer_admin/.\n\
             Searched: {:?}",
            candidates
        );
    };
    let status = Command::new(cli)
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .arg("--minify")
        .arg("--cwd")
        .arg(manifest_dir)
        .status()
        .expect("failed to run tailwindcss");
    if !status.success() {
        panic!("tailwindcss failed with status {}", status);
    }
}
