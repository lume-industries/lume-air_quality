#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building reminders-sidecar sidecar..."
cargo build --manifest-path "$SCRIPT_DIR/sidecar/Cargo.toml" --target wasm32-wasip1 --release
cp "$SCRIPT_DIR/../target/wasm32-wasip1/release/reminders-sidecar.wasm" "$SCRIPT_DIR/sidecar.wasm"
SIDECAR_SIZE=$(wc -c < "$SCRIPT_DIR/sidecar.wasm")
echo "Done: sidecar.wasm (${SIDECAR_SIZE} bytes)"

echo "Building reminders_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/reminders_slide.wasm" reminders_slide.wasm
ln -sfn reminders_slide.wasm slide.wasm
ln -sfn reminders_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "reminders_slide.wasm")
echo "Done: reminders_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing reminders.vzglyd..."
rm -f reminders.vzglyd
zip -X -0 -r reminders.vzglyd manifest.json slide.wasm sidecar.wasm assets/
VZGLYD_SIZE=$(wc -c < reminders.vzglyd)
echo "Done: reminders.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-reminders"
