#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"


echo "Building budget_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/budget_slide.wasm" budget_slide.wasm
ln -sfn budget_slide.wasm slide.wasm
ln -sfn budget_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "budget_slide.wasm")
echo "Done: budget_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing budget.vzglyd..."
rm -f budget.vzglyd
zip -X -0 -r budget.vzglyd manifest.json slide.wasm assets/
VZGLYD_SIZE=$(wc -c < budget.vzglyd)
echo "Done: budget.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-budget"
