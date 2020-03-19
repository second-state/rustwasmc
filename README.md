<div align="center">

  <h1>SsvmUp</h1>

  <p>
    <strong>SSVM ready tool</strong>
  </p>
</div>

## About

This tool seeks to be a one-stop shop for building and working with rust-
generated WebAssembly that you would like to interop with JavaScript in [SSVM].

[SSVM]: https://github.com/second-state/SSVM

## Acknowledgment

Most of the code for this project comes from [wasm-pack].

[wasm-pack]: https://github.com/rustwasm/wasm-pack

## Install

curl https://raw.githubusercontent.com/second-state/ssvmup/master/installer/init.sh -sSf | sh

## Commands

- `build`: Generate an npm wasm pkg from a rustwasm crate

## Logging

`ssvmup` uses [`env_logger`] to produce logs when `ssvmup` runs.

To configure your log level, use the `RUST_LOG` environment variable. For example:

```
RUST_LOG=info ssvmup build
```

[`env_logger`]: https://crates.io/crates/env_logger
