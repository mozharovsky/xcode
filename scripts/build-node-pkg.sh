#!/bin/bash
# Build @xcodekit/xcode-node → pkg/xcode-node/
set -e

if [ ! -f "index.js" ] || [ ! -f "index.d.ts" ]; then
  echo "Error: index.js/index.d.ts not found. Run: npx napi build --platform --release"
  exit 1
fi

OUT="pkg/xcode-node"
rm -rf "$OUT"
mkdir -p "$OUT"

cp npm/xcode-node/package.json "$OUT/package.json"
cp README.md "$OUT/README.md"
cp index.js "$OUT/index.js"
cp index.d.ts "$OUT/index.d.ts"
cp types.d.ts "$OUT/types.d.ts"
echo "Built $OUT/"
