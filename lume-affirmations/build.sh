#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"


echo "Building affirmations_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/affirmations_slide.wasm" affirmations_slide.wasm
ln -sfn affirmations_slide.wasm slide.wasm
ln -sfn affirmations_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "affirmations_slide.wasm")
echo "Done: affirmations_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing affirmations.vzglyd..."
rm -f affirmations.vzglyd
zip -X -0 -r affirmations.vzglyd manifest.json slide.wasm assets/
VZGLYD_SIZE=$(wc -c < affirmations.vzglyd)
echo "Done: affirmations.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-affirmations"
