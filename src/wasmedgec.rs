//! Support for downloading and executing `wasmedgec`

use crate::child;
use crate::emoji;
use crate::target;
use crate::PBAR;
use binary_install::Cache;
use log::debug;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Execute `wasmedgec` over wasm binaries found in `out_dir`, downloading if
/// necessary into `cache`. Passes `args` to each invocation of `wasmedgec`.
pub fn run(
    cache: &Cache,
    out_dir: &Path,
    install_permitted: bool,
) -> Result<(), failure::Error> {
    let wasmedgec = match find_wasmedgec(cache, install_permitted)? {
        SsvmcOpt::Found(path) => path,
        SsvmcOpt::CannotInstall => {
            PBAR.info("Skipping wasmedgec as no downloading was requested");
            return Ok(());
        }
        SsvmcOpt::PlatformNotSupported => {
            PBAR.info("You need Ubuntu 20.04 to compile the AOT binary. Please see https://www.secondstate.io/articles/setup-rust-nodejs/");
            return Ok(());
        }
    };

    PBAR.info("Compiling AOT binaries with `wasmedgec`...");

    for file in out_dir.read_dir()? {
        let file = file?;
        let path = file.path();
        if path.extension().and_then(|s| s.to_str()) != Some("wasm") {
            continue;
        }

        let tmp = path.with_extension("so");
        let mut cmd = Command::new(&wasmedgec);
        cmd.arg(&path).arg(&tmp);
        if let Err(e) = child::run(cmd, "wasmedgec") {
            PBAR.info("You need Ubuntu 20.04 to compile the AOT binary. Please see https://www.secondstate.io/articles/setup-rust-nodejs/");
            return Err(e)
        }
    }

    Ok(())
}

/// Possible results of `find_wasmedgec`
pub enum SsvmcOpt {
    /// Couldn't install wasmedgec because downloads are forbidden
    CannotInstall,
    /// The current platform doesn't support precompiled binaries
    PlatformNotSupported,
    /// We found `wasmedgec` at the specified path
    Found(PathBuf),
}

/// Attempts to find `wasmedgec` in `PATH` locally, or failing that downloads a
/// precompiled binary.
///
/// Returns `Some` if a binary was found or it was successfully downloaded.
/// Returns `None` if a binary wasn't found in `PATH` and this platform doesn't
/// have precompiled binaries. Returns an error if we failed to download the
/// binary.
pub fn find_wasmedgec(cache: &Cache, install_permitted: bool) -> Result<SsvmcOpt, failure::Error> {
    // First attempt to look up in PATH. If found assume it works.
    if let Ok(path) = which::which("wasmedgec") {
        debug!("found wasmedgec at {:?}", path);
        return Ok(SsvmcOpt::Found(path));
    }

    // ... and if that fails download a precompiled version.
    if target::LINUX && target::x86_64 {
        "x86_64-linux"
    } else {
        return Ok(SsvmcOpt::PlatformNotSupported);
    };
    let url = format!(
        "https://github.com/WasmEdge/WasmEdge/releases/download/{vers}/WasmEdge-{vers}-manylinux2014_x86_64.tar.gz",
        vers = "0.8.0",
    );

    let download = |permit_install| cache.download(permit_install, "wasmedgec", &["wasmedgec"], &url);

    let dl = match download(false)? {
        Some(dl) => dl,
        None if !install_permitted => return Ok(SsvmcOpt::CannotInstall),
        None => {
            let msg = format!("{}Installing wasmedgec...", emoji::DOWN_ARROW);
            PBAR.info(&msg);

            match download(install_permitted)? {
                Some(dl) => dl,
                None => return Ok(SsvmcOpt::CannotInstall),
            }
        }
    };

    Ok(SsvmcOpt::Found(dl.binary("wasmedgec")?))
}
