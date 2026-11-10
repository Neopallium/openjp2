# Differences from the C reference (openjpeg)

The port mirrors the C reference (`openjpeg/src/lib/openjp2`) behaviour
bit-for-bit on the test corpus (conformance + nonregression), but it is not
a literal transcription.  This page lists the intentional divergences.

## Safety and memory model

- The core crate is `#![forbid(unsafe_code)]`: no `unsafe` blocks anywhere.
  All `unsafe` lives in the `openjp2-capi` crate (the `#[no_mangle]` C ABI
  shim), which is audited against the C header contracts.
- Buffer over-reads the C performs in places (e.g. the tile-data conversion
  reading past a short component buffer) are bounds-checked and zero-filled
  instead.
- Out-of-memory is handled by Rust allocation failure (abort) like the C's
  explicit checks, except where the port pre-checks to keep the C's error
  diagnostics.

## Event reporting

- The C reference delivers diagnostics through user-registered callbacks
  (`opj_set_info_handler`/`warning`/`error`).  The core port routes messages
  through the standard `log` crate by default; the C-style callback surface
  (`Decoder`/`Encoder::set_*_handler`, plus the capi's `opj_set_*_handler`)
  is gated behind the opt-in `event-handlers` feature, which the
  `openjp2-capi` crate (and the CLI tools, which mirror the C CLI's
  callback printing) enables.  With no handler AND no `log` sink installed,
  messages are dropped (standard `log` behavior).  The message TEXT is
  identical in both modes; parallel workers' messages become visible
  through `log` where they used to be dropped (their per-worker managers
  had no handlers).

## Threading

- The C reference uses a per-codec `opj_thread_pool_t` created by
  `opj_codec_set_threads()`.  The port uses the process-wide rayon pool:
  the first successful sizing wins (`build_global` only succeeds once).
  `opj_codec_set_threads` mirrors the C's failure cases — no thread
  support and `num_threads > i32::MAX` return `OPJ_FALSE` — and otherwise
  returns `OPJ_TRUE` (the C's "FALSE after the tcd exists" nuance does
  not apply: the port's pool is process-wide by design).  A
  `num_threads == 0` call sizes nothing — the C builds a zero-worker
  (serial) pool for that codec — so use a positive count or the env vars
  below to shape parallelism.  The CLI tools size the pool from
  `-threads N` / `OPJ_NUM_THREADS` / `RAYON_NUM_THREADS`.
- The port parallelises more than the C: per-tile code-block decode/encode,
  DWT, DC shift (as in the C), plus tile-level concurrent decode and
  encode in the safe API (`Image::from_bytes` / `Image::encode`).
  Byte-identical results by construction (any unsupported layout falls
  back to the sequential path).
- At 24 threads the port decodes ~26% faster than the C reference on the
  standard benchmark file; single-threaded it is within ~13%.
- The tile-level concurrent decode only runs where it is byte-identical
  to the sequential decode by construction, and falls back to it
  otherwise: (1) decode areas that do not cover every intersecting tile
  completely (the out-of-window DWT taps of a partial window read the
  previous tile's leftover buffer in the sequential path), (2) quality
  layer limits and resolution reductions (unwritten coefficient regions
  also read cross-tile leftovers), and (3) "tolerant" codestreams whose
  packet segments overrun their packets (g1_colr.j2c): the sequential
  walk only works through the code-block state left by the previous
  tile, and a fresh per-tile decode would silently differ (detected via
  the t2 "segment too long" recovery and treated as a worker failure).

## Features

- JPWL (Part 11) and JPSEC (Part 8) marker handling is not ported
  (the C's `USE_JPWL` build option has no Rust equivalent); the markers
  parse as unknown and are rejected, like the C's default builds.
  The `jpwl_*` parameter fields remain in the C-ABI mirror for struct
  parity.
- SIMD: the C reference's NEON/AVX2/AVX512 IDWT kernels are not ported
  verbatim, but an optional `simd` feature (off by default) provides a
  safe `wide`-based kernel for the 5-3 vertical pass (~2% decode win
  at 1 thread, ~8% at 24 threads, byte-identical).  See
  `doc/simd-options.md` for the options and measurements.
- HTJ2K (T.814) decoding is ported; HTJ2K encoding is not, matching the
  C reference (decode-only).

## API and layout

- The C-ABI mirror types keep the C spellings (`opj_dparameters_t`, ...)
  behind `openjp2::ffi` / `openjp2_capi::*`; the safe API
  (`Image`, `DecodeParameters`, `EncodeParameters`, ...) is new and
  deliberately not C-shaped.
- `opj_stream_private` maps to the Rust `Stream` (reader/writer adapters);
  C custom streams round-trip through callback shims in the capi crate.
- The `opj_codec_private_t` layout is internal; only the opaque C handle
  is stable.

## Decode strictness

- The C library defaults to strict mode; the C tools decode with strict
  ON unless `-allow-partial` is given.  The port keeps that for the C
  ABI and the CLI tools, but the safe API (`Image::from_bytes` and
  friends) defaults to LENIENT decoding (`DecodeParameters::strict()`
  opts in) - a forgiving default for partial/truncated codestreams.
  Verified byte-identical to the C reference's `-allow-partial` output
  on the partial-codestream test files (e.g. v4dwt_interleave_h).

## Behavioural edge cases

- Marker validation: unknown markers in tile-part headers are rejected
  (upstream behaviour since the master sync).
- `opj_get_num_cpus()` reports `std::thread::available_parallelism()`;
  `opj_has_thread_support()` reports the `rayon` feature.
- Error messages are best-effort translations of the C diagnostics.
- CMYK->RGB in the Rust tools uses the canonical f32 + truncation
  semantics (the current upstream C tools' float code drifts by a few
  +1 pixels on this platform and disagrees with its own md5refs; the
  Rust tools match the md5refs exactly).
- The default codestream comment (COM marker) written by the encoder
  identifies the port ("Created by openjp2 version X").  Building with
  the `openjpeg-version-string` feature (off by default) writes the
  upstream OpenJPEG product/version string instead, making encoded
  files byte-identical to the C reference for test comparisons.
- The ~200 `assert!` sites in the core are ports of the C's `assert()`
  (stripped by NDEBUG in release C builds).  They encode invariants the
  C relies on too; the mutation/truncation fuzz campaigns (tens of
  thousands of decodes, zero panics) show they do not fire on corrupt
  input, and the bounds-checked Rust code fails loudly instead of
  running past them, so they are kept.
