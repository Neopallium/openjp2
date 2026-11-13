#!/usr/bin/env bash
# C-ABI compatibility check: the Rust `libopenjp2` against the C reference
# build of the same OpenJPEG version.
#
# Builds both libraries (shared, with debug info), then compares:
#   - the exported `opj_*` surface,
#   - the public function signatures (ABI shape),
#   - the public struct layouts (size / field order / offsets).
#
# Requirements: cmake, a C compiler, python3, and libabigail's `abidw`
#   (Debian/Ubuntu: `apt install abigail-tools`).
#
# Usage:
#   scripts/check_abi.sh                 # build both libs and check
#   scripts/check_abi.sh --symbols-only  # no libabigail required
#   SKIP_BUILD=1 scripts/check_abi.sh    # reuse existing builds
#   ABIDW=/path/to/abidw scripts/check_abi.sh

set -euo pipefail
cd "$(dirname "$0")/.."

C_BUILD="${C_BUILD:-target/abi-c}"
C_LIB="$C_BUILD/bin/libopenjp2.so"
RUST_LIB="capi/target/release/libopenjp2.so"
JOBS="${JOBS:-$(nproc 2>/dev/null || echo 4)}"

if [[ "${SKIP_BUILD:-0}" != "1" ]]; then
  echo "==> building the C reference library (RelWithDebInfo, shared)"
  cmake -S openjpeg -B "$C_BUILD" \
    -DBUILD_SHARED_LIBS=ON -DBUILD_TESTING=OFF -DBUILD_TOOLS=OFF \
    -DBUILD_CODEC=ON -DBUILD_PKGCONFIG_FILES=OFF \
    -DCMAKE_BUILD_TYPE=RelWithDebInfo > /dev/null
  cmake --build "$C_BUILD" -j "$JOBS" > /dev/null

  echo "==> building the Rust C-ABI library (release, with debug info)"
  ( cd capi && CARGO_PROFILE_RELEASE_DEBUG=2 cargo build --release > /dev/null )
fi

for lib in "$C_LIB" "$RUST_LIB"; do
  [[ -f "$lib" ]] || { echo "error: $lib not found" >&2; exit 2; }
done

# The type-level checks need DWARF; a stripped Rust build would silently
# degrade those checks, so verify the debug sections are present.
if ! readelf -S "$RUST_LIB" | grep -q '\.debug_info'; then
  echo "error: $RUST_LIB has no debug info (expected CARGO_PROFILE_RELEASE_DEBUG=2)" >&2
  exit 2
fi

echo "==> comparing $RUST_LIB against $C_LIB"
python3 scripts/abi_check.py \
  --header openjpeg/src/lib/openjp2/openjpeg.h \
  --c-lib "$C_LIB" \
  --rust-lib "$RUST_LIB" \
  "$@"
