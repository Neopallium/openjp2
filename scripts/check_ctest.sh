#!/usr/bin/env bash
# Runs the ctest suite and fails if any test outside the known-failure allowlist fails.
# The allowlist entries also fail with the reference C library.
set -uo pipefail

BUILD_DIR="${BUILD_DIR:-/opt/openjpeg/openjpeg/build}"
JOBS="${JOBS:-$(nproc)}"

ALLOWED=(
  "NR-DEC-issue205.jp2-43-decode-md5"
  "NR-DEC-issue208.jp2-69-decode-md5"
  "NR-DEC-issue226.j2k-74-decode-md5"
)

log="$(mktemp)"
trap 'rm -f "$log"' EXIT

( cd "$BUILD_DIR" && ctest -j"$JOBS" ) >"$log" 2>&1

# ctest lists failures as "        <id> - <name> (Failed)"
mapfile -t failed < <(sed -n 's/^[[:space:]]*[0-9]\+ - \(.*\) (\(Failed\|Timeout\|.*\))$/\1/p' "$log")

unexpected=()
for name in "${failed[@]}"; do
  ok=0
  for allow in "${ALLOWED[@]}"; do
    [[ "$name" == "$allow" ]] && ok=1 && break
  done
  (( ok )) || unexpected+=("$name")
done

missing=()
for allow in "${ALLOWED[@]}"; do
  seen=0
  for name in "${failed[@]}"; do
    [[ "$name" == "$allow" ]] && seen=1 && break
  done
  (( seen )) || missing+=("$allow")
done

grep -E '^[0-9]+% tests passed' "$log" || tail -20 "$log"

if (( ${#unexpected[@]} )); then
  echo "REGRESSION: unexpected test failures:"
  printf '  %s\n' "${unexpected[@]}"
  exit 1
fi

if (( ${#missing[@]} )); then
  echo "NOTE: known-failing tests now pass (update the allowlist):"
  printf '  %s\n' "${missing[@]}"
fi

echo "OK: no regressions (${#failed[@]} failure(s), all allowlisted)."
