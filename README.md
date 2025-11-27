# ethersync-web

Browser-base daemon and editor for [ethersync](https://github.com/ethersync/ethersync).

> [!CAUTION]
> This project is in an early stage, lacks important features and may be unstable. There will be dragons!

## Running locally

[`dx serve`](https://dioxuslabs.com/learn/0.6/guide/new_app#running-the-project)

This requires the following tools to be installed:

* [dioxus-cli](https://github.com/DioxusLabs/dioxus/tree/main/packages/cli) for building the frontend
* [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/) for compiling Rust to WebAssembly
* [Clang](https://clang.llvm.org/) for compiling the C dependencies (e.g. [ring](https://github.com/briansmith/ring)) to WebAssembly

There is a Nix flake that can be used to provide the dependencies. It's convenient to automatically activate it using [`direnv`](https://direnv.net). After setting up direnv's shell hooks, put "use flake" into an `.envrc` file, and run `direnv allow`.

## Deploying

[`dx bundle`](https://dioxuslabs.com/learn/0.6/guide/bundle)

The same tools as above are necessary.

It seems that Rust 1.86 [is currently required](https://github.com/DioxusLabs/dioxus/discussions/4183) for the deploy step.

The output will be in `target/dx/ethersync-web/release/web/public` which can then be served as a static site.
