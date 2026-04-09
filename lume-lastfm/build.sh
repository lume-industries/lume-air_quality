#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building lastfm-sidecar sidecar..."
cargo build --manifest-path "$SCRIPT_DIR/sidecar/Cargo.toml" --target wasm32-wasip1 --release
cp "$SCRIPT_DIR/../target/wasm32-wasip1/release/lastfm-sidecar.wasm" "$SCRIPT_DIR/sidecar.wasm"
SIDECAR_SIZE=$(wc -c < "$SCRIPT_DIR/sidecar.wasm")
echo "Done: sidecar.wasm (${SIDECAR_SIZE} bytes)"

echo "Building lastfm_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/lastfm_slide.wasm" lastfm_slide.wasm
ln -sfn lastfm_slide.wasm slide.wasm
ln -sfn lastfm_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "lastfm_slide.wasm")
echo "Done: lastfm_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing lastfm.vzglyd..."
rm -f lastfm.vzglyd
zip -X -0 -r lastfm.vzglyd manifest.json slide.wasm sidecar.wasm assets/
VZGLYD_SIZE=$(wc -c < lastfm.vzglyd)
echo "Done: lastfm.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-lastfm"
