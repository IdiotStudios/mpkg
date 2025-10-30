#!/bin/bash

set -e

echo "Building for linux..."
cargo build --release --target x86_64-unknown-linux-gnu

echo "Building for Windows..."
cargo build --release --target x86_64-pc-windows-gnu

# macOs (needs toolchain/linker)
# echo "Building for Mac..."
# cargo build --release --target x86_64-apple-darwin

echo "All Builds Finished!"