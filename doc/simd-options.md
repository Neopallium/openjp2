# SIMD options for the Rust port

What the C reference vectorizes, what the port can use (no embedded
ASM), what was measured, and what is implemented.

## The C reference's SIMD coverage

From the synced upstream code (`src/lib/openjp2`):

| Site | ISA | What it does |
|---|---|---|
| `dwt.c` `opj_idwt53_v_cas0_mcols_SIMD` | SSE2 / AVX2 / AVX512 | 5-3 vertical pass, 8/16/32 columns at once with shuffle-based interleaving; writes to a scratch buffer, caller memcpys back |
| `dwt.c` `opj_v8dwt_decode_step1/2_neon` | NEON (aarch64) | v8 DWT decode |
| `mct.c` `opj_mct_decode`/`encode` | SSE2 / SSE4.1 | reversible + irreversible MCT |
| `t1.c` encode stepsize scaling | SSE2 | float conversion in the encoder only |

The C's tier-1 codec (the actual hotspot, ~75% of its decode profile) is
scalar macro machines — the SIMD does not touch it.  The port's profile
matches: t1 = 70-92% of decode time, dwt = 8% (1 thread) / 22%
(24 threads).

## Rust options (stable toolchain, no inline ASM)

- **`std::arch` intrinsics (`core::arch`, the `stdarch` crate)** - exact
  C-equivalent intrinsics.  Every intrinsic is an `unsafe fn`, which
  conflicts with the core crate's `#![forbid(unsafe_code)]`: they could
  only live in a separate crate (with its own audited unsafe) or behind
  a feature that relaxes the rule.  Not used.
- **`std::simd` (portable SIMD)** - the safe, target-agnostic API.
  Still nightly-only (`#![feature(portable_simd)]` is rejected on
  rustc 1.99.1 stable - verified).  Its stable backport is the
  `core_simd` crate.  Watch for stabilization.
- **`wide` (1.7.1)** - safe explicit-width SIMD types (`i32x4`,
  `i32x8`, `f32x8`, ...) with SSE2/AVX/AVX512/NEON/WASM backends
  selected at compile time.  The call site is 100% safe Rust, so the
  core crate keeps `#![forbid(unsafe_code)]`.  **Implemented and
  measured** (below).
- **`pulp` (0.22.3)** - safe generic SIMD with runtime feature
  detection (`pulp::with_simd` picks the best ISA per thread); used by
  `faer`.  A candidate if runtime dispatch becomes a requirement.
- **`multiversion` (0.9.2)** - automatic function multi-versioning and
  runtime dispatch (the bodies still need intrinsics, i.e. unsafe).
- **`simdeez` (3.0.1)** - small SIMD abstraction crate.
- **Auto-vectorization** - building with `-C target-cpu=native` and
  letting LLVM vectorize the scalar loops.  Measured: zero gain on the
  port (the DWT's `Cell`-scratch strided gather/scatter loops defeat
  the vectorizer), so an explicit kernel is required.

Runtime CPU detection (if ever needed) is available safely through
`std::is_x86_feature_detected!` plus `#[target_feature]` wrappers;
`wide` compiles per-target instead, which is simpler and sufficient for
the current use.

## Implemented: `simd` feature (off by default)

- New cargo feature `simd` (pass-throughs on the tools and capi
  crates) adding the optional `wide` dependency.
- A `wide`-based kernel for the 5-3 inverse DWT **vertical pass**
  (`opj_idwt53_v_simd` in `src/dwt.rs`), processing 4 columns at once
  with `i32x4`:
  - reads the plain column layout (rows 0..sn-1 = low, sn.. = high),
    writes the interleaved result to a scratch buffer and copies back -
    the same scheme the C's SIMD uses to avoid the read-after-write
    aliasing between the layouts;
  - the `d(i)` read clamps to `dn-1` when `sn > dn` (odd level
    heights), mirroring the scalar path's `dn_idx` clamp;
  - only runs for `cas == 0`, `dn > 0`, `len > 1`, `nb_cols >= 4`;
    everything else (odd origins, edge columns, short bands) stays on
    the scalar path.
- On aarch64 the same `wide` code lowers to NEON automatically (the C
  has a hand-written NEON v8 path; ours is the 5-3 only).

## Measurements (Bretagne2.j2k, release, best-of-5)

| config | 1 thread | 24 threads |
|---|---|---|
| scalar (no simd) | 0.83 s | 0.13 s |
| + `simd` feature | 0.81 s | 0.12 s |
| + `-C target-cpu=native` (AVX2/AVX512) | 0.82-0.83 s | 0.13 s |
| 8-wide (`i32x8`) variant | 0.82-0.87 s | 0.13 s |
| C reference | 0.74 s | 0.16-0.19 s |

- The micro-benchmark of the kernel alone shows ~4x over the scalar
  column lift; end-to-end the gain is ~2% (1 thread) / ~8%
  (24 threads) because the vertical pass is a small slice of decode
  and is gather/scatter (memory) bound.
- The 8-wide/AVX2 variant and the native auto-vectorized build add
  nothing further (same memory bound); the 4-wide kernel captures the
  available win and was kept.
- With the feature the port decodes ~35% faster than the C reference
  at 24 threads (was ~19-26%).
- Byte-identity: all 207 nonregression + conformance inputs decode
  byte-identically to the scalar build (and the corpus equivalence
  tests pass with the feature enabled).

## Remaining levers, in order of value

1. **t1 unrolled macro machines** (scalar) - the same rewrite the C
   uses for its 75%-of-profile tier-1 codec; the only remaining
   large win (~13% single-thread parity gap).  Not SIMD.
2. **MCT SIMD** (`wide` i32x4 is a drop-in for `opj_mct_decode`) -
   trivial to add, but the MCT is <1% of decode time.
3. **Runtime dispatch** (`pulp`/`multiversion`) - only if a single
   binary must serve multiple ISA levels; today per-target builds are
   simpler.
4. **`std::simd`** when it stabilizes - would replace `wide` with
   zero behavioural change.
