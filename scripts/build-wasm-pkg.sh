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

# ── 2. Embed WASM as base64 ──────────────────────────────────────────────────

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

# ── 3. Copy wrapper + types ──────────────────────────────────────────────────

cp wasm-node-wrapper.mjs "pkg/web/index.mjs"
cp wasm-node-wrapper.d.ts "pkg/web/index.d.ts"
cp types.d.ts "pkg/web/types.d.ts"
echo "Copied wrapper as index.mjs + types"

# ── 4. Set exports map ───────────────────────────────────────────────────────

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

  // Files to publish
  const extra = ['index.mjs', 'index.d.ts', 'types.d.ts', 'xcode_bg_wasm_inline.js'];
  for (const f of extra) {
    if (!pkg.files.includes(f)) pkg.files.push(f);
  }

  require('fs').writeFileSync('./pkg/web/package.json', JSON.stringify(pkg, null, 2) + '\n');
"
echo "Set exports: '.' and './node' → index.mjs"
