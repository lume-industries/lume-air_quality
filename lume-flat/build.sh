#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"


echo "Building flat_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/flat_slide.wasm" flat_slide.wasm
ln -sfn flat_slide.wasm slide.wasm
ln -sfn flat_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "flat_slide.wasm")
echo "Done: flat_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing flat.vzglyd..."
rm -f flat.vzglyd
zip -X -0 -r flat.vzglyd manifest.json slide.wasm assets/
VZGLYD_SIZE=$(wc -c < flat.vzglyd)
echo "Done: flat.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-flat"
