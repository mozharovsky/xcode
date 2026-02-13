#!/bin/bash
# Build @xcodekit/xcode-wasm → pkg/xcode-wasm/
set -e

SRC="pkg/wasm-build"
OUT="pkg/xcode-wasm"

if [ ! -f "$SRC/xcode_bg.wasm" ]; then
  echo "Error: $SRC not found. Run: wasm-pack build --target web --out-dir $SRC"
  exit 1
fi

rm -rf "$OUT"
mkdir -p "$OUT"

# ── 1. Optimize WASM with wasm-opt ───────────────────────────────────────────

if command -v wasm-opt &> /dev/null; then
  BEFORE=$(wc -c < "$SRC/xcode_bg.wasm")
  wasm-opt "$SRC/xcode_bg.wasm" -o "$SRC/xcode_bg.wasm" -Oz \
    --enable-bulk-memory --enable-nontrapping-float-to-int \
    --enable-sign-ext --enable-mutable-globals
  AFTER=$(wc -c < "$SRC/xcode_bg.wasm")
  echo "wasm-opt: $(( BEFORE / 1024 )) KB → $(( AFTER / 1024 )) KB"
else
  echo "wasm-opt not found — skipping (install binaryen for smaller WASM)"
fi

# ── 2. Strip dead xcode_bg.wasm URL from init() ─────────────────────────────

node -e "
  const fs = require('fs');
  let js = fs.readFileSync('./$SRC/xcode.js', 'utf8');
  const before = js.length;
  js = js.replace(/new URL\(['\"]xcode_bg\.wasm['\"],\s*import\.meta\.url\)/g, 'undefined');
  if (js.length !== before) {
    fs.writeFileSync('./$SRC/xcode.js', js);
    console.log('Stripped dead xcode_bg.wasm URL reference');
  }
"

# ── 3. Embed WASM as base64 ─────────────────────────────────────────────────

node -e "
  const fs = require('fs');
  const wasm = fs.readFileSync('./$SRC/xcode_bg.wasm');
  const b64 = wasm.toString('base64');
  const js = '// Auto-generated — do not edit. Contains xcode_bg.wasm as base64.\n' +
             'const bytes = Buffer.from(\"' + b64 + '\", \"base64\");\n' +
             'export default bytes;\n';
  fs.writeFileSync('./$OUT/xcode_bg_wasm_inline.js', js);
  console.log('Embedded WASM as base64 (' + Math.round(wasm.length / 1024) + ' KB → ' + Math.round(js.length / 1024) + ' KB)');
"

# ── 4. Assemble package ─────────────────────────────────────────────────────

cp npm/xcode-wasm/package.json "$OUT/package.json"
cp README.md "$OUT/README.md"
cp "$SRC/xcode.js" "$OUT/xcode.js"
cp "$SRC/xcode.d.ts" "$OUT/xcode.d.ts"
cp "$SRC/xcode_bg.wasm.d.ts" "$OUT/xcode_bg.wasm.d.ts"
cp npm/xcode-wasm/wrapper.mjs "$OUT/index.mjs"
cp npm/xcode-wasm/wrapper.d.ts "$OUT/index.d.ts"
cp types.d.ts "$OUT/types.d.ts"
echo "Built $OUT/"
