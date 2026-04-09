#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building weather-sidecar sidecar..."
cargo build --manifest-path "$SCRIPT_DIR/sidecar/Cargo.toml" --target wasm32-wasip1 --release
cp "$SCRIPT_DIR/../target/wasm32-wasip1/release/weather-sidecar.wasm" "$SCRIPT_DIR/sidecar.wasm"
SIDECAR_SIZE=$(wc -c < "$SCRIPT_DIR/sidecar.wasm")
echo "Done: sidecar.wasm (${SIDECAR_SIZE} bytes)"

echo "Building weather_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/weather_slide.wasm" weather_slide.wasm
ln -sfn weather_slide.wasm slide.wasm
ln -sfn weather_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "weather_slide.wasm")
echo "Done: weather_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing weather.vzglyd..."
rm -f weather.vzglyd
zip -X -0 -r weather.vzglyd manifest.json slide.wasm sidecar.wasm assets/
VZGLYD_SIZE=$(wc -c < weather.vzglyd)
echo "Done: weather.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-weather"
