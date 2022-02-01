#!/bin/sh
#
C2RUST=~/opt/c2rust/target/release/c2rust

git checkout -- src

# The `--cargo` option is crashing c2rust refactor.
# So manually setup the rustc args
DEPS="`pwd`/target/debug/deps"
DEP_c2rust_bitfields=`find $DEPS -iname 'libc2rust_bitfields-*.rlib' | head -1`
DEP_libc=`find $DEPS -iname 'liblibc-*.rlib' | head -1`

$C2RUST refactor "$@" \
  rewrite_stmts 'if $e { } else { __assert_fail($a, $b, $c, $d); }' 'assert_fail($e);' \
  -- \
  src/lib.rs \
  --edition=2018 --crate-name openjp2 \
  --crate-type cdylib --crate-type staticlib --crate-type rlib \
  -L dependency=$DEPS \
  --extern c2rust_bitfields=$DEP_c2rust_bitfields \
  --extern libc=$DEP_libc

sed \
  -e 's/ assert_fail(/ assert!(/g' \
  -i src/*.rs

cargo fmt
