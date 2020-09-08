//! Functionality related to running `wasm-bindgen`.

use binary_install::Download;
use child;
use command::build::{BuildProfile};
use failure::{self, ResultExt};
use manifest::CrateData;
use std::path::{Path};
use std::process::Command;

/// Run the `wasm-bindgen` CLI to generate bindings for the current crate's
/// `.wasm`.
pub fn wasm_bindgen_build(
    data: &CrateData,
    bindgen: &Download,
    out_dir: &Path,
    out_name: &Option<String>,
    disable_dts: bool,
    profile: BuildProfile,
    target: &str,
    run_target: &str,
    enable_aot: bool,
) -> Result<(), failure::Error> {
    let release_or_debug = match profile {
        BuildProfile::Release | BuildProfile::Profiling => "release",
        BuildProfile::Dev => "debug",
    };

    let out_dir = out_dir.to_str().unwrap();

    let dts_arg = if disable_dts {
        "--no-typescript"
    } else {
        "--typescript"
    };
    let bindgen_path = bindgen.binary("wasm-bindgen")?;

    for c in data.crate_name().iter() {
        let mut cmd = Command::new(&bindgen_path);
        cmd.arg("--out-dir")
            .arg(out_dir)
            .arg(dts_arg);

        cmd.arg("--target").arg(run_target);

        if let Some(value) = out_name {
            cmd.arg("--out-name").arg(value);
        }

        let profile = data.configured_profile(profile);
        if profile.wasm_bindgen_debug_js_glue() {
            cmd.arg("--debug");
        }
        if !profile.wasm_bindgen_demangle_name_section() {
            cmd.arg("--no-demangle");
        }
        if profile.wasm_bindgen_dwarf_debug_info() {
            cmd.arg("--keep-debug");
        }
        if enable_aot {
            cmd.arg("--enable-aot");
        }
        let wasm_path = data
            .target_directory()
            .join(target)
            .join(release_or_debug)
            .join(c.as_str())
            .with_extension("wasm");
        cmd.arg(&wasm_path);
        child::run(cmd, "wasm-bindgen").context("Running the wasm-bindgen CLI")?;
    }

    Ok(())
}
