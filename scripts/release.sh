#!/bin/bash
set -e

VERSION=${1:-0.1.0}

echo "🚀 Preparing release v$VERSION"

# Build the app
echo "📦 Building VTT.app..."
make clean
make package

# Calculate checksum
echo "🔐 Calculating sha256..."
SHA256=$(shasum -a 256 VTT.app.tar.gz | awk '{print $1}')
echo "SHA256: $SHA256"

# Update cask with real checksum
sed -i '' "s/REPLACE_WITH_REAL_SHA256_ON_RELEASE/$SHA256/" Casks/voice-to-text.rb
sed -i '' "s/version \".*\"/version \"$VERSION\"/" Casks/voice-to-text.rb

echo ""
echo "✅ Release ready!"
echo ""
echo "Next steps:"
echo "  1. Create GitHub release: gh release create v$VERSION VTT.app.tar.gz"
echo "  2. Test installation: brew install --cask voice-to-text"
echo "  3. Submit to homebrew-cask: https://github.com/Homebrew/homebrew-cask/blob/master/CONTRIBUTING.md"
echo ""
