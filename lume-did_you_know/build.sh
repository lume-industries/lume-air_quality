#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"


echo "Building did_you_know_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/did_you_know_slide.wasm" did_you_know_slide.wasm
ln -sfn did_you_know_slide.wasm slide.wasm
ln -sfn did_you_know_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "did_you_know_slide.wasm")
echo "Done: did_you_know_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing did_you_know.vzglyd..."
rm -f did_you_know.vzglyd
zip -X -0 -r did_you_know.vzglyd manifest.json slide.wasm assets/
VZGLYD_SIZE=$(wc -c < did_you_know.vzglyd)
echo "Done: did_you_know.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-did_you_know"
