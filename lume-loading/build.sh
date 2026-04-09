#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"


echo "Building loading_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/loading_slide.wasm" loading_slide.wasm
ln -sfn loading_slide.wasm slide.wasm
ln -sfn loading_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "loading_slide.wasm")
echo "Done: loading_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing loading.vzglyd..."
rm -f loading.vzglyd
zip -X -0 -r loading.vzglyd manifest.json slide.wasm assets/
VZGLYD_SIZE=$(wc -c < loading.vzglyd)
echo "Done: loading.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-loading"
