#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

FAILURES=0
SUCCESS=0

for dir in "$SCRIPT_DIR"/lume-*/; do
    name=$(basename "$dir")

    # Skip air_quality
    if [ "$name" = "lume-air_quality" ]; then
        continue
    fi

    if [ ! -f "$dir/build.sh" ]; then
        echo "⚠️  $name: no build.sh, skipping"
        continue
    fi

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🔨 Building $name..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if bash "$dir/build.sh" 2>&1; then
        echo "✅ $name: done"
        SUCCESS=$((SUCCESS + 1))
    else
        echo "❌ $name: FAILED"
        FAILURES=$((FAILURES + 1))
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Done: $SUCCESS succeeded, $FAILURES failed"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ "$FAILURES" -gt 0 ]; then
    exit 1
fi
