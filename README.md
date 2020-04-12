<div align="center">
  <h1>ssvmup</h1>
  <p>
    <strong>The Second State VM ready tool</strong>
  </p>
</div>

![npm](https://img.shields.io/npm/v/ssvmup)
![npm](https://img.shields.io/npm/dt/ssvmup)
![GitHub language count](https://img.shields.io/github/languages/count/second-state/ssvmup)
![GitHub top language](https://img.shields.io/github/languages/top/second-state/ssvmup)

Developers: [Getting started](https://docs.secondstate.io/server-side-webassembly/webassembly-on-the-server-side) building Rust + JavaScript hybrid apps for Node.js using the `ssvmup` tool.

## About

A one-stop tool for building Rust functions into WebAssembly (the [Second State VM, or SSVM](https://github.com/second-state/SSVM)) and then accessing these functions from Node.js JavaScript.

## Acknowledgment

This project is derived from the open source [wasm-pack].

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
