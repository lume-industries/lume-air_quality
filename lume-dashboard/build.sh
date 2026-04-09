#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"


echo "Building dashboard_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/dashboard_slide.wasm" dashboard_slide.wasm
ln -sfn dashboard_slide.wasm slide.wasm
ln -sfn dashboard_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "dashboard_slide.wasm")
echo "Done: dashboard_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing dashboard.vzglyd..."
rm -f dashboard.vzglyd
zip -X -0 -r dashboard.vzglyd manifest.json slide.wasm assets/
VZGLYD_SIZE=$(wc -c < dashboard.vzglyd)
echo "Done: dashboard.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-dashboard"
