# Cross-Platform Windowing Example

This project aims to represent a very simple way of establishing a window and a GL context that works similarly across all of Rust's platforms. Conditional compilation is attempted to be kept to a minimum without relinquishing too much control over the environment.

## Native Run

A simple `cargo run` should suffice.

## WASM Run

This project uses wasm-bindgen and doesn't support stdweb. To build the binary, use `wasm-pack build` and then navigate to the `www` directory. `npm install` and then `npm start` should run a web server that you can view the project from.