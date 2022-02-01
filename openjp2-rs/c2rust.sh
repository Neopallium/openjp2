#!/bin/sh
#
C2RUST=~/opt/c2rust/target/release/c2rust

#rm -rf Cargo.* rust-toolchain build.rs lib.rs src target

$C2RUST transpile --reduce-type-annotations \
  --emit-modules --emit-build-files \
  --filter 'lib/openjp2/*' \
  -o ./ ./compile_commands.json -- \
  -DSTANDARD_SLOW_VERSION -U__SSE__ -U__SSE2__ -U__SSE4_1__

