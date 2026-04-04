#!/bin/bash
# Build Vibe Island for all platforms
# Run from the project root

set -e

echo "=== Vibe Island Build ==="

# Install JS deps
if command -v bun &>/dev/null; then
    bun install
else
    npm install
fi

# Build for current platform
echo "Building for $(uname -s)..."
npx tauri build

echo ""
echo "=== Build complete ==="
echo "Output: src-tauri/target/release/bundle/"
ls -la src-tauri/target/release/bundle/*/ 2>/dev/null || echo "(check build output)"
