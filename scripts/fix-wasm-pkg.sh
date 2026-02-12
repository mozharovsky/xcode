#!/bin/bash
# Fix wasm-pack generated package.json files with correct name/metadata
set -e

VERSION=$(node -p "require('./package.json').version")

for dir in pkg/web pkg/node pkg/bundler; do
  if [ -f "$dir/package.json" ]; then
    node -e "
      const pkg = require('./$dir/package.json');
      pkg.name = '@xcodekit/xcode-wasm';
      pkg.version = '$VERSION';
      pkg.description = 'Parse, manipulate, and serialize Xcode .pbxproj files (WASM build)';
      pkg.repository = { type: 'git', url: 'https://github.com/mozharovsky/xcode' };
      pkg.keywords = ['xcode', 'pbxproj', 'ios', 'apple', 'parser', 'wasm', 'rust'];
      pkg.license = 'MIT';
      require('fs').writeFileSync('./$dir/package.json', JSON.stringify(pkg, null, 2) + '\n');
    "
    echo "Fixed $dir/package.json → @xcodekit/xcode-wasm@$VERSION"
  fi
done

# Copy shared types.d.ts into each pkg target
for dir in pkg/web pkg/node pkg/bundler; do
  if [ -d "$dir" ]; then
    cp types.d.ts "$dir/types.d.ts"
    node -e "
      const pkg = require('./$dir/package.json');
      if (!pkg.files.includes('types.d.ts')) pkg.files.push('types.d.ts');
      if (!pkg.exports) pkg.exports = {};
      pkg.exports['./types'] = { types: './types.d.ts' };
      require('fs').writeFileSync('./$dir/package.json', JSON.stringify(pkg, null, 2) + '\n');
    "
    echo "Added types.d.ts to $dir"
  fi
done

# Copy ESM wrapper + types into pkg/web (published target, --target web)
if [ -d "pkg/web" ]; then
  cp wasm-node-wrapper.mjs "pkg/web/node-wrapper.mjs"
  cp wasm-node-wrapper.d.ts "pkg/web/node-wrapper.d.ts"

  node -e "
    const pkg = require('./pkg/web/package.json');
    if (!pkg.exports) pkg.exports = {};
    pkg.exports['.'] = {
      types: './xcode.d.ts',
      import: './xcode.js',
      default: './xcode.js'
    };
    pkg.exports['./node'] = {
      types: './node-wrapper.d.ts',
      import: './node-wrapper.mjs',
      default: './node-wrapper.mjs'
    };
    if (!pkg.files.includes('node-wrapper.mjs')) pkg.files.push('node-wrapper.mjs');
    if (!pkg.files.includes('node-wrapper.d.ts')) pkg.files.push('node-wrapper.d.ts');
    require('fs').writeFileSync('./pkg/web/package.json', JSON.stringify(pkg, null, 2) + '\n');
  "
  echo "Added ESM wrapper + types to pkg/web"
fi

# Copy CJS wrapper + types into pkg/node (local dev/testing, --target nodejs)
if [ -d "pkg/node" ]; then
  cp wasm-node-wrapper.js "pkg/node/node-wrapper.js"
  cp wasm-node-wrapper.d.ts "pkg/node/node-wrapper.d.ts"

  node -e "
    const pkg = require('./pkg/node/package.json');
    if (!pkg.exports) pkg.exports = {};
    pkg.exports['.'] = {
      require: './xcode.js',
      types: './xcode.d.ts'
    };
    pkg.exports['./node'] = {
      require: './node-wrapper.js',
      types: './node-wrapper.d.ts'
    };
    if (!pkg.files.includes('node-wrapper.js')) pkg.files.push('node-wrapper.js');
    if (!pkg.files.includes('node-wrapper.d.ts')) pkg.files.push('node-wrapper.d.ts');
    require('fs').writeFileSync('./pkg/node/package.json', JSON.stringify(pkg, null, 2) + '\n');
  "
  echo "Added CJS wrapper + types to pkg/node"
fi
