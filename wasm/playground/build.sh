#!/usr/bin/env bash
# Build the openjp2-wasm playground: compiles the wasm crate, generates the
# JS glue into playground/pkg/ (single-threaded) and, with --threads, the
# threaded build into playground/pkg-mt/, then (optionally) serves the site.
#
# Requirements (single-threaded build):
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli   # version must match wasm-bindgen in Cargo.lock
#
# Extra requirements for --threads:
#   rustup toolchain install nightly
#   rustup +nightly component add rust-src
#   rustup +nightly target add wasm32-unknown-unknown
#
# Usage:
#   wasm/playground/build.sh              # single-threaded build only
#   wasm/playground/build.sh --threads    # + the threaded (rayon) build
#   wasm/playground/build.sh serve        # build and serve on :8080
#   wasm/playground/build.sh --threads serve
#
# The threaded build uses WebAssembly threads (SharedArrayBuffer), which
# browsers only expose to cross-origin-isolated pages, so `serve` sends
# the COOP/COEP headers.  GitHub Pages cannot set headers, so the deployed
# site runs single-threaded.

set -euo pipefail
cd "$(dirname "$0")/.."

THREADS=0
SERVE=0
for arg in "$@"; do
  case "$arg" in
    --threads) THREADS=1 ;;
    serve) SERVE=1 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

echo "==> building openjp2-wasm (wasm32-unknown-unknown, release)"
cargo build --target wasm32-unknown-unknown --release
echo "==> generating JS glue into playground/pkg/"
mkdir -p playground/pkg
wasm-bindgen target/wasm32-unknown-unknown/release/openjp2_wasm.wasm \
  --target web --out-dir playground/pkg

if [[ "$THREADS" == "1" ]]; then
  echo "==> building the threaded (rayon) variant (nightly + build-std)"
  # --shared-memory is required: wasm-bindgen-rayon posts the memory to
  # its workers, which only works for SharedArrayBuffer-backed memory.
  # Flags from the wasm-bindgen-rayon README: shared+imported memory and
  # the TLS symbols its worker transform needs.
  THREAD_FLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals'
  THREAD_FLAGS="$THREAD_FLAGS -C link-arg=--shared-memory -C link-arg=--import-memory"
  THREAD_FLAGS="$THREAD_FLAGS -C link-arg=--max-memory=2147483648"
  THREAD_FLAGS="$THREAD_FLAGS -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size"
  THREAD_FLAGS="$THREAD_FLAGS -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base"
  RUSTFLAGS="$THREAD_FLAGS" \
    cargo +nightly build --target wasm32-unknown-unknown --release \
    --features threads -Z build-std=std,panic_abort
  echo "==> generating JS glue into playground/pkg-mt/"
  mkdir -p playground/pkg-mt
  wasm-bindgen target/wasm32-unknown-unknown/release/openjp2_wasm.wasm \
    --target web --out-dir playground/pkg-mt
fi

echo "==> done.  Open wasm/playground/index.html (via a static server for"
echo "    ES-module loading), or run:  wasm/playground/build.sh serve"

if [[ "$SERVE" == "1" ]]; then
  echo "==> serving on http://localhost:8080 (COOP/COEP enabled)"
  python3 - <<'PY'
import functools
import http.server

class Handler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # required for SharedArrayBuffer / WebAssembly threads
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Resource-Policy", "same-origin")
        super().end_headers()

Handler.extensions_map[".wasm"] = "application/wasm"
http.server.ThreadingHTTPServer(
    ("", 8080), functools.partial(Handler, directory="playground")
).serve_forever()
PY
fi
