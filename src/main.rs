#![allow(clippy::redundant_closure, clippy::redundant_pattern_matching)]

extern crate atty;
extern crate env_logger;
#[macro_use]
extern crate failure;
extern crate human_panic;
extern crate log;
extern crate structopt;
extern crate rustwasmc;
extern crate which;

use std::env;
use std::panic;
use structopt::StructOpt;
use rustwasmc::{
    command::run_rustwasmc,
    Cli, PBAR,
};

mod installer;

fn main() {
    env_logger::init();

    setup_panic_hooks();

    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        for cause in e.iter_causes() {
            eprintln!("Caused by: {}", cause);
        }
        ::std::process::exit(1);
    }
}

fn run() -> Result<(), failure::Error> {
    if let Ok(me) = env::current_exe() {
        // If we're actually running as the installer then execute our
        // self-installation, otherwise just continue as usual.
        if me
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("executable should have a filename")
            .starts_with("rustwasmc-init")
        {
            installer::install();
        }
    }

    let args = Cli::from_args();

    PBAR.set_log_level(args.log_level);

    if args.quiet {
        PBAR.set_quiet(true);
    }

    run_rustwasmc(args.cmd)?;

    Ok(())
}

fn setup_panic_hooks() {
    let meta = human_panic::Metadata {
        version: env!("CARGO_PKG_VERSION").into(),
        name: env!("CARGO_PKG_NAME").into(),
        authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
        homepage: env!("CARGO_PKG_HOMEPAGE").into(),
    };

    let default_hook = panic::take_hook();

    if let Err(_) = env::var("RUST_BACKTRACE") {
        panic::set_hook(Box::new(move |info: &panic::PanicInfo| {
            // First call the default hook that prints to standard error.
            default_hook(info);

            // Then call human_panic.
            let file_path = human_panic::handle_dump(&meta, info);
            human_panic::print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
        }));
    }
}
