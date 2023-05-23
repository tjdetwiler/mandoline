@default: run-presubmit

# Build for host (native)
build:
    cargo build
# Build for wasm
build-wasm:
    cargo build --target=wasm32-unknown-unknown
# Build both native and wasm.
build-all: build build-wasm

# Test host (native)
test:
    cargo test 
# Test wasm
test-wasm:
    cargo test --target=wasm32-unknown-unknown
# Test both native and wasm.
test-all: test test-wasm

# Populate wasm files in stl-viewer/pkg
package-wasm:
    cd stl-viewer && wasm-pack build --target web

# Run the native application.
run:
    cargo run
# Run the wgpu application.
run-wasm: package-wasm
	python -m http.server --directory stl-viewer/pkg 8000

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
