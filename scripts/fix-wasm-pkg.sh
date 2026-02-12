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

# Copy Node.js wrappers (CJS + ESM + types) into each pkg target
for dir in pkg/node pkg/bundler; do
  if [ -d "$dir" ]; then
    cp wasm-node-wrapper.js "$dir/node-wrapper.js"
    cp wasm-node-wrapper.mjs "$dir/node-wrapper.mjs"
    cp wasm-node-wrapper.d.ts "$dir/node-wrapper.d.ts"

    node -e "
      const pkg = require('./$dir/package.json');
      if (!pkg.exports) pkg.exports = {};
      pkg.exports['.'] = {
        import: './xcode.js',
        require: './xcode.js',
        types: './xcode.d.ts'
      };
      pkg.exports['./node'] = {
        import: './node-wrapper.mjs',
        require: './node-wrapper.js',
        types: './node-wrapper.d.ts'
      };
      if (!pkg.files.includes('node-wrapper.js')) pkg.files.push('node-wrapper.js');
      if (!pkg.files.includes('node-wrapper.mjs')) pkg.files.push('node-wrapper.mjs');
      if (!pkg.files.includes('node-wrapper.d.ts')) pkg.files.push('node-wrapper.d.ts');
      require('fs').writeFileSync('./$dir/package.json', JSON.stringify(pkg, null, 2) + '\n');
    "
    echo "Added Node.js wrappers (CJS + ESM + types) to $dir"
  fi
done
