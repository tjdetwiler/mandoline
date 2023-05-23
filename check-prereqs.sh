#!/bin/sh

cargo install just
# This version needs to be kept in sync with the versions of
# wasm-bindgen
cargo install wasm-bindgen-cli --vers "0.2.86"
