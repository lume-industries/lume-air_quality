#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building afl-sidecar sidecar..."
cargo build --manifest-path "$SCRIPT_DIR/sidecar/Cargo.toml" --target wasm32-wasip1 --release
cp "$SCRIPT_DIR/../target/wasm32-wasip1/release/afl-sidecar.wasm" "$SCRIPT_DIR/sidecar.wasm"
SIDECAR_SIZE=$(wc -c < "$SCRIPT_DIR/sidecar.wasm")
echo "Done: sidecar.wasm (${SIDECAR_SIZE} bytes)"

echo "Building afl_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/afl_slide.wasm" afl_slide.wasm
ln -sfn afl_slide.wasm slide.wasm
ln -sfn afl_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "afl_slide.wasm")
echo "Done: afl_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing afl.vzglyd..."
rm -f afl.vzglyd
zip -X -0 -r afl.vzglyd manifest.json slide.wasm sidecar.wasm assets/
VZGLYD_SIZE=$(wc -c < afl.vzglyd)
echo "Done: afl.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-afl"
