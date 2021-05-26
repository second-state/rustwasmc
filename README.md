<div align="center">
  <h1>rustwasmc</h1>
  <p>
    <strong>Rust compiler toolkit for WebAssembly apps in the WasmEdge Runtime</strong>
  </p>
</div>

![npm](https://img.shields.io/npm/v/rustwasmc)
![npm](https://img.shields.io/npm/dt/rustwasmc)
![GitHub language count](https://img.shields.io/github/languages/count/second-state/rustwasmc)
![GitHub top language](https://img.shields.io/github/languages/top/second-state/rustwasmc)

Developers: Getting started with the [Second State Functions](https://www.secondstate.io/articles/getting-started-with-function-as-a-service-in-rust/) (a serverless FaaS service), [Tencent Serverless Functions for AI inference](https://github.com/second-state/tencent-tensorflow-scf/blob/main/README-en.md), or [your own Node.js server](https://www.secondstate.io/articles/getting-started-with-rust-function/).

## About

A one-stop tool for building Rust functions into WebAssembly for deployment on the [WasmEdge Runtime](https://github.com/WasmEdge/WasmEdge). 

## Prerequisite

You will need to install the Rust compiler in order to use `rustwasmc`. Currently, we support up to Rust 1.50.0.

```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ source $HOME/.cargo/env
```

## Install

From Linux command line

```
$ curl https://raw.githubusercontent.com/second-state/rustwasmc/master/installer/init.sh -sSf | sh
```

From NPM and Node.js

```
$ npm install -g rustwasmc # Append --unsafe-perm if permission denied
```

## Usage

To build [Rust functions for Node.js](https://www.secondstate.io/articles/getting-started-with-rust-function/) applications, use the following command. See a [template application](https://github.com/second-state/wasmedge-nodejs-starter). The `rustwasmc` compiles and generates the wasm file, and the corresponding JavaScript file to call wasm functions from JavaScript. If the rust package contains only binary crate(s) and there are no library crate, the build command will only generate a `wasm-wasi` file for running as a standalone command program in the WasmEdge VM.

```
$ rustwasmc build
```

In most cases, you will want to enable AOT optimization in order to improve performance.

```
$ rustwasmc build --enable-aot
```

If you would like to use WasmEdge's extended APIs including Tensorflow, enable the extensions. Make sure that you install the `wasmedge-extensions` NPM module in this case.

```
$ rustwasmc build --enable-aot --enable-ext
```

By default, `rustwasmc` generates `wasm32` bytecode for library functions and `wasm32-wasi` bytecode for `main()` command programs. The `wasm32-wasi` bytecode allows Rust source code to access OS features such as file system, environment variables, and command line options. However, you can also force `rustwasmc` to generate `wasm32-wasi` bytecode in all cases by passing the `--wasi` flag.

```
$ rustwasmc build --wasi
```

To build Rust functions for Deno applications, use the following command. See a [template application](https://github.com/second-state/ssvm-deno-starter).

```
$ rustwasmc build --target deno
```

By default, rustwasmc will generate a directory for it's build output called pkg. If you'd like to customize this you can use the --out-dir flag.

```
$ rustwasmc build --out-dir out
```

Use clean subcommand to remove pkg and target directories.
```
$ rustwasmc clean
```

## Logging

`rustwasmc` uses [`env_logger`] to produce logs when `rustwasmc` runs.

To configure your log level, use the `RUST_LOG` environment variable. For example:

```
$ RUST_LOG=info rustwasmc build
```

[`env_logger`]: https://crates.io/crates/env_logger

## Acknowledgment

This project is derived from the open source [wasm-pack].

[wasm-pack]: https://github.com/rustwasm/wasm-pack
