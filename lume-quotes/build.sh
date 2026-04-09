#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"


echo "Building quotes_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/quotes_slide.wasm" quotes_slide.wasm
ln -sfn quotes_slide.wasm slide.wasm
ln -sfn quotes_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "quotes_slide.wasm")
echo "Done: quotes_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing quotes.vzglyd..."
rm -f quotes.vzglyd
zip -X -0 -r quotes.vzglyd manifest.json slide.wasm assets/
VZGLYD_SIZE=$(wc -c < quotes.vzglyd)
echo "Done: quotes.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-quotes"
