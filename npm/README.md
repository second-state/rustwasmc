<div align="center">

  <h1>rustwasmc</h1>

  <p>
    <strong>WasmEdge ready tool</strong>
  </p>
</div>

## About

This tool seeks to be a one-stop shop for building and working with rust-
generated WebAssembly that you would like to interop with JavaScript in [WasmEdge].

[WasmEdge]: https://github.com/WasmEdge/WasmEdge

## Acknowledgment

Most of the code for this project comes from [wasm-pack].

[wasm-pack]: https://github.com/rustwasm/wasm-pack

## Install

curl https://raw.githubusercontent.com/second-state/rustwasmc/master/installer/init.sh -sSf | sh

## Commands

- `build`: Generate an npm wasm pkg from a rustwasm crate

## Logging

`rustwasmc` uses [`env_logger`] to produce logs when `rustwasmc` runs.

To configure your log level, use the `RUST_LOG` environment variable. For example:

```
RUST_LOG=info rustwasmc build
```

[`env_logger`]: https://crates.io/crates/env_logger
