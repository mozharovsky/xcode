#!/bin/bash
# Fix wasm-pack generated package for @xcodekit/xcode-wasm.
#
# Single target (--target web), single entry point. WASM binary is inlined
# as base64 so bundlers (Bun, esbuild, etc.) don't need to resolve .wasm files.
set -e

VERSION=$(node -p "require('./package.json').version")

if [ ! -f "pkg/web/package.json" ]; then
  echo "Error: pkg/web not found. Run: wasm-pack build --target web --out-dir pkg/web"
  exit 1
fi

# ── 1. Fix package metadata ──────────────────────────────────────────────────

node -e "
  const pkg = require('./pkg/web/package.json');
  pkg.name = '@xcodekit/xcode-wasm';
  pkg.version = '$VERSION';
  pkg.description = 'Parse, manipulate, and serialize Xcode .pbxproj files (WASM build)';
  pkg.repository = { type: 'git', url: 'https://github.com/mozharovsky/xcode' };
  pkg.keywords = ['xcode', 'pbxproj', 'ios', 'apple', 'parser', 'wasm', 'rust'];
  pkg.license = 'MIT';
  require('fs').writeFileSync('./pkg/web/package.json', JSON.stringify(pkg, null, 2) + '\n');
"
echo "Fixed pkg/web/package.json → @xcodekit/xcode-wasm@$VERSION"

# ── 2. Optimize WASM with wasm-opt ────────────────────────────────────────────

if command -v wasm-opt &> /dev/null; then
  BEFORE=$(wc -c < pkg/web/xcode_bg.wasm)
  wasm-opt pkg/web/xcode_bg.wasm -o pkg/web/xcode_bg.wasm -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals
  AFTER=$(wc -c < pkg/web/xcode_bg.wasm)
  echo "wasm-opt: $(( BEFORE / 1024 )) KB → $(( AFTER / 1024 )) KB (saved $(( (BEFORE - AFTER) / 1024 )) KB)"
else
  echo "wasm-opt not found — skipping (install binaryen for smaller WASM)"
fi

# ── 3. Strip dead xcode_bg.wasm URL from init() ──────────────────────────────
# The wasm-bindgen init() has a fallback: `if (A === void 0) A = new URL("xcode_bg.wasm", import.meta.url)`
# Since we always pass inlined bytes, this never executes — but bundlers still
# see the URL and try to resolve the file. Remove it to avoid warnings/errors.

node -e "
  const fs = require('fs');
  let js = fs.readFileSync('./pkg/web/xcode.js', 'utf8');
  const before = js.length;
  js = js.replace(/new URL\(['\"]xcode_bg\.wasm['\"],\s*import\.meta\.url\)/g, 'undefined');
  if (js.length !== before) {
    fs.writeFileSync('./pkg/web/xcode.js', js);
    console.log('Stripped dead xcode_bg.wasm URL reference from xcode.js');
  } else {
    console.log('No xcode_bg.wasm URL reference found (already clean)');
  }
"

# ── 4. Embed WASM as base64 ──────────────────────────────────────────────────

node -e "
  const fs = require('fs');
  const wasm = fs.readFileSync('./pkg/web/xcode_bg.wasm');
  const b64 = wasm.toString('base64');
  const js = '// Auto-generated — do not edit. Contains xcode_bg.wasm as base64.\n' +
             'const bytes = Buffer.from(\"' + b64 + '\", \"base64\");\n' +
             'export default bytes;\n';
  fs.writeFileSync('./pkg/web/xcode_bg_wasm_inline.js', js);
  console.log('Embedded WASM as base64 (' + Math.round(wasm.length / 1024) + ' KB → ' + Math.round(js.length / 1024) + ' KB)');
"

# ── 5. Copy wrapper + types ──────────────────────────────────────────────────

cp wasm-node-wrapper.mjs "pkg/web/index.mjs"
cp wasm-node-wrapper.d.ts "pkg/web/index.d.ts"
cp types.d.ts "pkg/web/types.d.ts"
echo "Copied wrapper as index.mjs + types"

# ── 6. Set exports map ───────────────────────────────────────────────────────

node -e "
  const pkg = require('./pkg/web/package.json');

  // Single entry — both '.' and './node' point to the same file
  const entry = {
    types: './index.d.ts',
    import: './index.mjs',
    default: './index.mjs'
  };
  pkg.exports = {
    '.': entry,
    './node': entry,
    './types': { types: './types.d.ts' }
  };

  // Files to publish (drop raw .wasm — it's inlined as base64)
  pkg.files = pkg.files.filter(f => !f.endsWith('.wasm'));
  const extra = ['index.mjs', 'index.d.ts', 'types.d.ts', 'xcode_bg_wasm_inline.js'];
  for (const f of extra) {
    if (!pkg.files.includes(f)) pkg.files.push(f);
  }

  require('fs').writeFileSync('./pkg/web/package.json', JSON.stringify(pkg, null, 2) + '\n');
"

rm -f pkg/web/xcode_bg.wasm
echo "Set exports: '.' and './node' → index.mjs (dropped raw .wasm)"
