#!/usr/bin/env bash
# Build the openjp2-wasm playground: compiles the wasm crate, generates the
# JS glue into playground/pkg/, and (optionally) serves the site.
#
# Requirements: Rust with the wasm32-unknown-unknown target and
# wasm-bindgen-cli on PATH (its version must match the wasm-bindgen
# version in Cargo.lock):
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version "$(awk '/^name = "wasm-bindgen"$/{getline; gsub(/"/, "", $3); print $3}' Cargo.lock)"
#
# Usage:
#   wasm/playground/build.sh            # build only
#   wasm/playground/build.sh serve      # build and serve on :8080

set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> building openjp2-wasm (wasm32-unknown-unknown, release)"
cargo build --target wasm32-unknown-unknown --release

echo "==> generating JS glue into playground/pkg/"
mkdir -p playground/pkg
wasm-bindgen target/wasm32-unknown-unknown/release/openjp2_wasm.wasm \
  --target web --out-dir playground/pkg

echo "==> done.  Open wasm/playground/index.html (via a static server for"
echo "    ES-module loading), or run:  wasm/playground/build.sh serve"

if [[ "${1:-}" == "serve" ]]; then
  echo "==> serving on http://localhost:8080"
  python3 -m http.server 8080 --directory playground
fi
