#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building news-sidecar sidecar..."
cargo build --manifest-path "$SCRIPT_DIR/sidecar/Cargo.toml" --target wasm32-wasip1 --release
cp "$SCRIPT_DIR/../target/wasm32-wasip1/release/news-sidecar.wasm" "$SCRIPT_DIR/sidecar.wasm"
SIDECAR_SIZE=$(wc -c < "$SCRIPT_DIR/sidecar.wasm")
echo "Done: sidecar.wasm (${SIDECAR_SIZE} bytes)"

echo "Building news_slide.wasm..."
cargo build --target wasm32-wasip1 --release
cp "../target/wasm32-wasip1/release/news_slide.wasm" news_slide.wasm
ln -sfn news_slide.wasm slide.wasm
ln -sfn news_slide.json manifest.json
SLIDE_SIZE=$(wc -c < "news_slide.wasm")
echo "Done: news_slide.wasm (${SLIDE_SIZE} bytes)"

echo "Packing news.vzglyd..."
rm -f news.vzglyd
zip -X -0 -r news.vzglyd manifest.json slide.wasm sidecar.wasm assets/
VZGLYD_SIZE=$(wc -c < news.vzglyd)
echo "Done: news.vzglyd (${VZGLYD_SIZE} bytes)"
echo "Run with:"
echo "  cargo run --manifest-path ../lume/Cargo.toml -- --scene ../lume-news"
