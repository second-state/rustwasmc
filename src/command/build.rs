//! Implementation of the `rustwasmc build` command.

use crate::wasm_opt;
use crate::ssvmc;
use binary_install::{Cache, Download};
use bindgen;
use build;
use cache;
use command::utils::{create_pkg_dir, get_crate_path};
use emoji;
use failure::Error;
use install::{self, InstallMode, Tool};
use license;
use lockfile::Lockfile;
use log::info;
use manifest;
use readme;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use PBAR;

/// Everything required to configure and run the `rustwasmc build` command.
#[allow(missing_docs)]
pub struct Build {
    pub crate_path: PathBuf,
    pub crate_data: manifest::CrateData,
    pub scope: Option<String>,
    pub disable_dts: bool,
    pub profile: BuildProfile,
    pub mode: InstallMode,
    pub target: String,
    pub enable_aot: bool,
    pub enable_ext: bool,
    pub run_target: String,
    pub out_dir: PathBuf,
    pub out_name: Option<String>,
    pub bindgen: Option<Download>,
    pub cache: Cache,
    pub extra_options: Vec<String>,
}

/// The build profile controls whether optimizations, debug info, and assertions
/// are enabled or disabled.
#[derive(Clone, Copy, Debug)]
pub enum BuildProfile {
    /// Enable assertions and debug info. Disable optimizations.
    Dev,
    /// Enable optimizations. Disable assertions and debug info.
    Release,
    /// Enable optimizations and debug info. Disable assertions.
    Profiling,
}

/// Everything required to configure and run the `rustwasmc build` command.
#[derive(Debug, StructOpt)]
pub struct BuildOptions {
    /// The path to the Rust crate. If not set, searches up the path from the current directory.
    #[structopt(parse(from_os_str))]
    pub path: Option<PathBuf>,

    /// The npm scope to use in package.json, if any.
    #[structopt(long = "scope", short = "s")]
    pub scope: Option<String>,

    #[structopt(long = "mode", short = "m", default_value = "normal")]
    /// Sets steps to be run. [possible values: no-install, normal, force]
    pub mode: InstallMode,

    #[structopt(long = "target", default_value = "ssvm")]
    /// Sets the runtime target. [possible values: ssvm(default), nodejs, deno].
    pub target: String,

    #[structopt(long = "enable-aot")]
    /// Enable AOT in SSVM
    pub enable_aot: bool,

    #[structopt(long = "enable-ext")]
    /// Requiring ssvm-extensions instead of ssvm
    pub enable_ext: bool,

    #[structopt(skip = true)]
    /// By default a *.d.ts file is generated for the generated JS file, but
    /// this flag will disable generating this TypeScript file.
    pub disable_dts: bool,

    #[structopt(long = "dev")]
    /// Create a development build. Enable debug info, and disable
    /// optimizations.
    pub dev: bool,

    #[structopt(long = "release")]
    /// Create a release build. Enable optimizations and disable debug info.
    pub release: bool,

    #[structopt(long = "profiling")]
    /// Create a profiling build. Enable optimizations and debug info.
    pub profiling: bool,

    #[structopt(long = "out-dir", short = "d", default_value = "pkg")]
    /// Sets the output directory with a relative path.
    pub out_dir: String,

    #[structopt(long = "out-name")]
    /// Sets the output file names. Defaults to package name.
    pub out_name: Option<String>,

    #[structopt(last = true)]
    /// List of extra options to pass to `cargo build`
    pub extra_options: Vec<String>,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            path: None,
            scope: None,
            mode: InstallMode::default(),
            target: String::from("ssvm"),
            enable_aot: false,
            enable_ext: false,
            disable_dts: true,
            dev: false,
            release: false,
            profiling: false,
            out_dir: String::from("pkg"),
            out_name: None,
            extra_options: Vec::new(),
        }
    }
}

type BuildStep = fn(&mut Build) -> Result<(), Error>;

impl Build {
    /// Construct a build command from the given options.
    pub fn try_from_opts(build_opts: BuildOptions) -> Result<Self, Error> {
        let crate_path = get_crate_path(build_opts.path)?;
        let crate_data = manifest::CrateData::new(&crate_path, build_opts.out_name.clone())?;
        let out_dir = crate_path.join(PathBuf::from(build_opts.out_dir));

        let profile = match (build_opts.dev, build_opts.release, build_opts.profiling) {
            (false, false, false) | (false, true, false) => BuildProfile::Release,
            (true, false, false) => BuildProfile::Dev,
            (false, false, true) => BuildProfile::Profiling,
            // Unfortunately, `structopt` doesn't expose clap's `conflicts_with`
            // functionality yet, so we have to implement it ourselves.
            _ => bail!("Can only supply one of the --dev, --release, or --profiling flags"),
        };

        Ok(Build {
            crate_path,
            crate_data,
            scope: build_opts.scope,
            disable_dts: build_opts.disable_dts,
            profile,
            mode: build_opts.mode,
            target: String::from("wasm32-wasi"),
            enable_aot: build_opts.enable_aot,
            enable_ext: build_opts.enable_ext,
            run_target: build_opts.target,
            out_dir,
            out_name: build_opts.out_name,
            bindgen: None,
            cache: cache::get_rustwasmc_cache()?,
            extra_options: build_opts.extra_options,
        })
    }

    /// Configures the global binary cache used for this build
    pub fn set_cache(&mut self, cache: Cache) {
        self.cache = cache;
    }

    /// Execute this `Build` command.
    pub fn run(&mut self) -> Result<(), Error> {
        let process_steps = Build::get_process_steps(self.mode);

        let started = Instant::now();

        for (_, process_step) in process_steps {
            process_step(self)?;
        }

        let duration = crate::command::utils::elapsed(started.elapsed());
        info!("Done in {}.", &duration);
        info!(
            "Your wasm pkg is ready to publish at {}.",
            self.out_dir.display()
        );

        PBAR.info(&format!("{} Done in {}", emoji::SPARKLE, &duration));

        PBAR.info(&format!(
            "{} Your wasm pkg is ready to publish at {}.",
            emoji::PACKAGE,
            self.out_dir.display()
        ));
        Ok(())
    }

    /// Execute the "clean" command
    pub fn clean() -> Result<(), Error> {
        let bo = BuildOptions::default();
        let b = Build::try_from_opts(bo)?;
        info!("Removing the {} directory...", b.out_dir.display());
        fs::remove_dir_all(b.out_dir)?;
        info!("Removing the {} directory...", b.crate_data.target_directory().display());
        fs::remove_dir_all(b.crate_data.target_directory())?;
        Ok(())
    }

    fn get_process_steps(mode: InstallMode) -> Vec<(&'static str, BuildStep)> {
        macro_rules! steps {
            ($($name:ident),+) => {
                {
                let mut steps: Vec<(&'static str, BuildStep)> = Vec::new();
                    $(steps.push((stringify!($name), Build::$name));)*
                        steps
                    }
                };
            ($($name:ident,)*) => (steps![$($name),*])
        }
        let mut steps = Vec::new();
        match &mode {
            InstallMode::Force => {}
            _ => {
                steps.extend(steps![
                    step_check_rustc_version,
                    step_check_crate_config,
                    step_check_for_wasm_target,
                ]);
            }
        }
        steps.extend(steps![
            step_build_wasm,
            step_create_dir,
            step_copy_readme,
            step_copy_license,
            step_copy_wasm,
            step_install_wasm_bindgen,
            step_run_wasm_bindgen,
            step_run_wasm_opt,
            step_run_ssvmc,
            step_create_json,
        ]);
        steps
    }

    fn step_check_rustc_version(&mut self) -> Result<(), Error> {
        info!("Checking rustc version...");
        let version = build::check_rustc_version()?;
        let msg = format!("rustc version is {}.", version);
        info!("{}", &msg);
        Ok(())
    }

    fn step_check_crate_config(&mut self) -> Result<(), Error> {
        info!("Checking crate configuration...");

        // If crate type only contains [bin], which means it will only run in wasi
        // then we don't need bindgen as well
        if !self.crate_data.check_crate_type()? {
            // ssvm only support wasm-bindgen 0.2.61
            let lockfile = Lockfile::new(&self.crate_data)?;
            let bindgen_version = lockfile.require_wasm_bindgen()?;
            if bindgen_version != "0.2.61" {
                bail!("Sorry, rustwasmc only supports wasm-bindgen 0.2.61 at this time. Please fix your Cargo.toml to wasm-bindgen = \"=0.2.61\"")
            }
        }
        info!("Crate is correctly configured.");
        Ok(())
    }

    fn step_check_for_wasm_target(&mut self) -> Result<(), Error> {
        info!("Checking for wasm-target...");
        build::wasm_target::check_for_wasm32_target(&self.target)?;
        info!("Checking for wasm-target was successful.");
        Ok(())
    }

    fn step_build_wasm(&mut self) -> Result<(), Error> {
        info!("Building wasm...");
        build::cargo_build_wasm(&self.crate_path, self.profile, &self.target, &self.extra_options)?;

        info!(
            "wasm built at {:#?}.",
            &self
                .crate_path
                .join("target")
                .join(&self.target)
                .join("release")
        );
        Ok(())
    }

    fn step_create_dir(&mut self) -> Result<(), Error> {
        info!("Creating a pkg directory...");
        create_pkg_dir(&self.out_dir)?;
        info!("Created a pkg directory at {:#?}.", &self.crate_path);
        Ok(())
    }

    fn step_create_json(&mut self) -> Result<(), Error> {
        self.crate_data.write_package_json(
            &self.out_dir,
            &self.scope,
            self.disable_dts,
        )?;
        info!(
            "Wrote a package.json at {:#?}.",
            &self.out_dir.join("package.json")
        );
        Ok(())
    }

    fn step_copy_readme(&mut self) -> Result<(), Error> {
        info!("Copying readme from crate...");
        readme::copy_from_crate(&self.crate_path, &self.out_dir)?;
        info!("Copied readme from crate to {:#?}.", &self.out_dir);
        Ok(())
    }

    fn step_copy_license(&mut self) -> Result<(), failure::Error> {
        info!("Copying license from crate...");
        license::copy_from_crate(&self.crate_data, &self.crate_path, &self.out_dir)?;
        info!("Copied license from crate to {:#?}.", &self.out_dir);
        Ok(())
    }

    fn step_install_wasm_bindgen(&mut self) -> Result<(), failure::Error> {
        // bindgen is only needed in cdylib target
        if self.crate_data.check_crate_type()? {
            return Ok(());
        }
        info!("Identifying wasm-bindgen dependency...");
        let lockfile = Lockfile::new(&self.crate_data)?;
        let bindgen_version = lockfile.require_wasm_bindgen()?;
        info!("Installing wasm-bindgen-cli...");
        let bindgen = install::download_prebuilt_or_cargo_install(
            Tool::WasmBindgen,
            &self.cache,
            &bindgen_version,
            self.mode.install_permitted()
        )?;
        self.bindgen = Some(bindgen);
        info!("Installing wasm-bindgen-cli was successful.");
        Ok(())
    }

    fn step_copy_wasm(&mut self) -> Result<(), Error> {
        // Only needed in bin target
        if !self.crate_data.check_crate_type()? {
            return Ok(());
        }

        let release_or_debug = match self.profile {
            BuildProfile::Release | BuildProfile::Profiling => "release",
            BuildProfile::Dev => "debug",
        };

        for c in self.crate_data.crate_name().iter() {
            let wasm_path = self.crate_data
                .target_directory()
                .join(&self.target)
                .join(release_or_debug)
                .join(c.as_str())
                .with_extension("wasm");
            let out_wasm_path = self.out_dir.join(c.as_str()).with_extension("wasm");
            fs::copy(&wasm_path, &out_wasm_path)?;
        }

        Ok(())
    }

    fn step_run_wasm_bindgen(&mut self) -> Result<(), Error> {
        // bindgen is only needed in cdylib target
        if self.crate_data.check_crate_type()? {
            return Ok(());
        }
        info!("Building the wasm bindings...");
        bindgen::wasm_bindgen_build(
            &self.crate_data,
            self.bindgen.as_ref().unwrap(),
            &self.out_dir,
            &self.out_name,
            self.disable_dts,
            self.profile,
            &self.target,
            &self.run_target,
            self.enable_aot,
            self.enable_ext,
        )?;
        info!("wasm bindings were built at {:#?}.", &self.out_dir);
        Ok(())
    }

    fn step_run_wasm_opt(&mut self) -> Result<(), Error> {
        let args = match self
            .crate_data
            .configured_profile(self.profile)
            .wasm_opt_args()
        {
            Some(args) => args,
            None => return Ok(()),
        };
        info!("executing wasm-opt with {:?}", args);
        wasm_opt::run(
            &self.cache,
            &self.out_dir,
            &args,
            self.mode.install_permitted(),
        ).map_err(|e| {
            format_err!(
                "{}\nTo disable `wasm-opt`, add `wasm-opt = false` to your package metadata in your `Cargo.toml`.", e
            )
        })
    }

    fn step_run_ssvmc(&mut self) -> Result<(), Error> {
        if !self.enable_aot {
            return Ok(())
        }
        ssvmc::run(
            &self.cache,
            &self.out_dir,
            self.mode.install_permitted(),
        )?;

        Ok(())
    }
}
