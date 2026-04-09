#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"


echo "Building chore_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/chore_slide.wasm" chore_slide.wasm
ln -sfn chore_slide.wasm slide.wasm
ln -sfn chore_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "chore_slide.wasm")
echo "Done: chore_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing chore.vzglyd..."
rm -f chore.vzglyd
zip -X -0 -r chore.vzglyd manifest.json slide.wasm assets/
VZGLYD_SIZE=$(wc -c < chore.vzglyd)
echo "Done: chore.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-chore"
