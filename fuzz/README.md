# Fuzzing the J2K/JP2 decoders

libFuzzer targets for the decoder hardening work. The campaign goal:
malformed, truncated or malicious codestreams must never panic the
decoders, and the limit-constrained mode must stay within its memory
budget.

## Prerequisites

- `cargo-fuzz` (installed)
- a nightly toolchain (ASan instrumentation needs `-Zsanitizer`):
  `rustup toolchain install nightly --profile minimal`

## Targets

- `fuzz_decode` - decodes every input in three modes: lenient
  (progressive-friendly), strict, and limit-constrained
  (`DecodeLimits`: 2048x2048 image, 32 MB memory, 1 MB metadata).

The corpus (`corpus/fuzz_decode/`) is seeded by the encoder
(`cargo run --bin gen_seeds`): small J2K/JP2 codestreams, reversible and
irreversible, gray and RGB. `regression_sot_oom.jp2` and
`regression_tile_alloc.jp2` are crash regressions found by the campaign
(a lying SOD Psot that drove a 4.3 GB allocation, and a mutated SIZ whose
761 M-pixel tile drove a 3.0 GB allocation; both are now rejected before
allocating). `regression_csiz_zero.j2k`, `regression_plt_overflow.j2k` and
`regression_pi33_pcrl.j2k` are regressions for the code-review fixes
(Csiz=0, the PLT length-shift overflow, and the pcrl packet-iterator
shift overflow).

## Structure-aware mutator + dictionary

The target registers an `LLVMFuzzerCustomMutator` (`fuzz_mutator!` in
`fuzz_targets/fuzz_decode.rs`) that parses the J2K marker / JP2 box layout
and rewrites header fields (SIZ geometry/precision, COD numlayers/
numresolutions/cblk sizes, Sqcd, Psot, IHDR dims) with extreme values
(0/1/2, 0x7f/0x80, 0x7fff/0x8000, 0x7fffffff/0xffffffff), keeping the
containing marker length valid so the mutation reaches the deep handler
instead of being rejected at the framing check. Roughly half the mutations
fall back to libFuzzer's byte-level mutator, fed by `fuzz_decode.dict`
(marker segments, JP2 box types, boundary values).

## Run

```bash
cd fuzz
# 20-minute campaign (ALWAYS cap the time: slow corpus entries can drop the
# execution rate to ~20/s and make an uncapped -runs budget run for hours)
cargo +nightly fuzz run fuzz_decode -- -max_len=131072 -dict=fuzz_decode.dict -max_total_time=1200
# reproduce a finding:
cargo +nightly fuzz run fuzz_decode -- artifacts/fuzz_decode/<name> -runs=1
# minimize a finding:
cargo +nightly fuzz tmin fuzz_decode artifacts/fuzz_decode/<name>
```

## Notes

- The unset `DecodeLimits::max_mem_alloc` falls back to a built-in 1 GiB
  ceiling (see `DEFAULT_DECODE_MEM_LIMIT`): the decode-stage buffers
  (tile data, output image data) are always bounded, so a corrupt header
  cannot drive a multi-gigabyte allocation out of a tiny input. Set an
  explicit limit for larger images; the SIZ-stage check now always applies
  (the effective ceiling, default included) and also accounts for the
  per-tile coding-parameter arrays.
- Lengths that prefix data in the stream (JP2 box lengths, marker segment
  lengths, the SOD tile-part length) are checked against the remaining
  stream bytes before any allocation; the lenient SOD path clamps a lying
  Psot to what is actually available (progressive download).
- The fuzz crate is intentionally NOT part of the workspace (it needs the
  nightly toolchain and links `libfuzzer-sys`).
- The safe API (`Image::from_bytes*`, `DumpImage`) and every C-ABI export
  are wrapped in `catch_unwind`: a decoder bug fails the decode (or returns
  the C failure value) instead of aborting the host process or unwinding
  into C.
