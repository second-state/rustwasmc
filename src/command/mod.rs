//! CLI command structures, parsing, and execution.
#![allow(clippy::redundant_closure)]

pub mod build;
pub mod utils;

use self::build::{Build, BuildOptions};
use failure::Error;
use log::info;
use std::result;

/// The various kinds of commands that `rustwasmc` can execute.
#[derive(Debug, StructOpt)]
pub enum Command {
    /// 🏗️  build your npm package!
    #[structopt(name = "build")]
    Build(BuildOptions),

    /// clean the pkg and target dir
    #[structopt(name = "clean")]
    Clean{},
}

/// Run a command with the given logger!
pub fn run_rustwasmc(command: Command) -> result::Result<(), Error> {
    // Run the correct command based off input and store the result of it so that we can clear
    // the progress bar then return it
    match command {
        Command::Build(build_opts) => {
            info!("Running build command...");
            // Check rust toolchain first
            let o = std::process::Command::new("rustc").arg("--version").output();
            match o {
                Err(_e) => bail!("Please follow instructions to install Rust language tools first. https://www.secondstate.io/articles/ssvmup/ Thank you."),
                _ => {}
            }
            Build::try_from_opts(build_opts).and_then(|mut b| b.run())
        }
        Command::Clean{} => {
            Build::clean()
        }
    }
}
