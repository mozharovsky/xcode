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

# Copy Node.js wrapper into pkg/node for the /node subpath
if [ -d "pkg/node" ]; then
  cp wasm-node-wrapper.js pkg/node/node-wrapper.js
  # Add exports field to pkg/node/package.json
  node -e "
    const pkg = require('./pkg/node/package.json');
    pkg.exports = {
      '.': { require: './xcode.js', types: './xcode.d.ts' },
      './node': { require: './node-wrapper.js' }
    };
    pkg.files.push('node-wrapper.js');
    require('fs').writeFileSync('./pkg/node/package.json', JSON.stringify(pkg, null, 2) + '\n');
  "
  echo "Added Node.js wrapper with open()/save() to pkg/node"
fi

# Also set up exports for bundler target
if [ -d "pkg/bundler" ]; then
  cp wasm-node-wrapper.js pkg/bundler/node-wrapper.js
  node -e "
    const pkg = require('./pkg/bundler/package.json');
    pkg.exports = {
      '.': { import: './xcode.js', types: './xcode.d.ts' },
      './node': { require: './node-wrapper.js' }
    };
    pkg.files.push('node-wrapper.js');
    require('fs').writeFileSync('./pkg/bundler/package.json', JSON.stringify(pkg, null, 2) + '\n');
  "
  echo "Added Node.js wrapper with open()/save() to pkg/bundler"
fi
