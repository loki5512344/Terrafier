#!/bin/bash
cargo build --release
mkdir -p dist
cp target/release/terrafier-cli dist/
cp target/release/terrafier-gui dist/
echo "Release binaries in dist/"
