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
