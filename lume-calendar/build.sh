#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building calendar-sidecar sidecar..."
cargo build --manifest-path "$SCRIPT_DIR/sidecar/Cargo.toml" --target wasm32-wasip1 --release
cp "$SCRIPT_DIR/../target/wasm32-wasip1/release/calendar-sidecar.wasm" "$SCRIPT_DIR/sidecar.wasm"
SIDECAR_SIZE=$(wc -c < "$SCRIPT_DIR/sidecar.wasm")
echo "Done: sidecar.wasm (${SIDECAR_SIZE} bytes)"

echo "Building calendar_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/calendar_slide.wasm" calendar_slide.wasm
ln -sfn calendar_slide.wasm slide.wasm
ln -sfn calendar_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "calendar_slide.wasm")
echo "Done: calendar_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing calendar.vzglyd..."
rm -f calendar.vzglyd
zip -X -0 -r calendar.vzglyd manifest.json slide.wasm sidecar.wasm assets/
VZGLYD_SIZE=$(wc -c < calendar.vzglyd)
echo "Done: calendar.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-calendar"
