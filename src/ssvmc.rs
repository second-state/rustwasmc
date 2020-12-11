//! Support for downloading and executing `ssvmc`

use crate::child;
use crate::emoji;
use crate::target;
use crate::PBAR;
use binary_install::Cache;
use log::debug;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Execute `ssvmc` over wasm binaries found in `out_dir`, downloading if
/// necessary into `cache`. Passes `args` to each invocation of `ssvmc`.
pub fn run(
    cache: &Cache,
    out_dir: &Path,
    install_permitted: bool,
) -> Result<(), failure::Error> {
    let ssvmc = match find_ssvmc(cache, install_permitted)? {
        SsvmcOpt::Found(path) => path,
        SsvmcOpt::CannotInstall => {
            PBAR.info("Skipping ssvmc as no downloading was requested");
            return Ok(());
        }
        SsvmcOpt::PlatformNotSupported => {
            PBAR.info("You need Ubuntu 20.04 to compile the AOT binary. Please see https://www.secondstate.io/articles/setup-rust-nodejs/");
            return Ok(());
        }
    };

    PBAR.info("Compiling AOT binaries with `ssvmc`...");

    for file in out_dir.read_dir()? {
        let file = file?;
        let path = file.path();
        if path.extension().and_then(|s| s.to_str()) != Some("wasm") {
            continue;
        }

        let tmp = path.with_extension("so");
        let mut cmd = Command::new(&ssvmc);
        cmd.arg(&path).arg(&tmp);
        if let Err(e) = child::run(cmd, "ssvmc") {
            PBAR.info("You need Ubuntu 20.04 to compile the AOT binary. Please see https://www.secondstate.io/articles/setup-rust-nodejs/");
            return Err(e)
        }
    }

    Ok(())
}

/// Possible results of `find_ssvmc`
pub enum SsvmcOpt {
    /// Couldn't install ssvmc because downloads are forbidden
    CannotInstall,
    /// The current platform doesn't support precompiled binaries
    PlatformNotSupported,
    /// We found `ssvmc` at the specified path
    Found(PathBuf),
}

/// Attempts to find `ssvmc` in `PATH` locally, or failing that downloads a
/// precompiled binary.
///
/// Returns `Some` if a binary was found or it was successfully downloaded.
/// Returns `None` if a binary wasn't found in `PATH` and this platform doesn't
/// have precompiled binaries. Returns an error if we failed to download the
/// binary.
pub fn find_ssvmc(cache: &Cache, install_permitted: bool) -> Result<SsvmcOpt, failure::Error> {
    // First attempt to look up in PATH. If found assume it works.
    if let Ok(path) = which::which("ssvmc") {
        debug!("found ssvmc at {:?}", path);
        return Ok(SsvmcOpt::Found(path));
    }

    // ... and if that fails download a precompiled version.
    if target::LINUX && target::x86_64 {
        "x86_64-linux"
    } else {
        return Ok(SsvmcOpt::PlatformNotSupported);
    };
    let url = format!(
        "https://github.com/second-state/SSVM/releases/download/{vers}/ssvm-{vers}-linux-x64.tar.gz",
        vers = "0.7.0",
    );

    let download = |permit_install| cache.download(permit_install, "ssvmc", &["ssvmc"], &url);

    let dl = match download(false)? {
        Some(dl) => dl,
        None if !install_permitted => return Ok(SsvmcOpt::CannotInstall),
        None => {
            let msg = format!("{}Installing ssvmc...", emoji::DOWN_ARROW);
            PBAR.info(&msg);

            match download(install_permitted)? {
                Some(dl) => dl,
                None => return Ok(SsvmcOpt::CannotInstall),
            }
        }
    };

    Ok(SsvmcOpt::Found(dl.binary("ssvmc")?))
}
