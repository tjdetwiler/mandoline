@default: run-presubmit

# Build targets for both host and wasm.
build:
    cargo build
build-wasm:
    cargo build --target=wasm32-unknown-unknown
build-all: build build-wasm

# Test targets for both host and wasm.
test:
    cargo test 
test-wasm:
    cargo test --target=wasm32-unknown-unknown
test-all: test test-wasm

# Lint/style checks.
lint:
    cargo clippy
    cargo clippy --target=wasm32-unknown-unknown
    cargo fmt --check

# Run code formatters.
format-code:
    cargo fmt

# Checks to run successfully before submit.
run-presubmit: test-all lint
