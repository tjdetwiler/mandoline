# Path to find scad models.
model_path := "./res"

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

# This is what is run post-submit to verify the repo is healthy.
run-ci: run-presubmit

# Rebuild scad models into STLs.
rebuild-models:
	#!/usr/bin/env sh
	# Add the Homebrew path since it's not on PATH by default.
	export PATH="${PATH}:/Applications/OpenSCAD.app/Contents/MacOS"
	which openscad || {
		echo "No openscad found"
		exit -1
	}
	for scad_file in `find "{{model_path}}" -iname "*.scad"`
	do
		outname=$(dirname ${scad_file})/$(basename $scad_file .scad)
		openscad --export-format binstl -o ${outname}-bin.stl $scad_file
		openscad --export-format asciistl -o ${outname}-ascii.stl $scad_file
	done
