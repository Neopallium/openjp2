# openjp2 — Proposed Public API & Internal Structure

Written 2026-11-10. Builds on `doc/public-api-summary.md` (current state),
`doc/internals-summary.md` (current pipelines) and `doc/api-review.md` (the
jpeg2k-parity work, mostly landed). The goal of this document: propose the
*next* iteration of the public API and the internal architecture, and record
the design questions that need your decision.

# Part A — Proposed public API (target: the unpublished 0.7)

## A.1 Guiding principles (revised 2026-11-10, with owner decisions)

1. One primary surface: the safe one-shot + builder API. The low-level API
   becomes **split `Decoder` / `Encoder` types** (owner decision: the merged
   `Codec` goes away from the core; see A.4).
2. **No C types in the core's public namespace.** All C-ABI mirror types move
   to a new shared crate **`openjp2-ctypes`** (owner decision); the core keeps
   using them internally for now (non-breaking) and re-exports them only
   under `openjp2::ffi`; the root `pub use c_api_types::*` is removed in the
   upcoming 0.7 release (0.7 has never been published, so the breaking
   changes can land directly in it — no 0.8 staging needed).
   The `openjp2-capi` crate is the only consumer that talks C types across
   the boundary.
3. Every fallible call returns `Result<_, Error>`; panics are caught at the
   boundary (already true for the safe API — extend to the new low-level API).
4. Feature-light core: `rayon` and `image`-crate interop stay optional.
   **Raw component data access is always available** (owner decision: J2K/JP2
   support more components/precisions than the `image` crate can represent) —
   the `image` feature only adds `DynamicImage`/`ImageData` conveniences.
5. `Image` keeps its name (owner decision). jpeg2k naming parity is kept only
   where it costs nothing (owner decision).

## A.2 Proposed types & methods

```rust
// ---- errors: add structured variants, keep strings as context ----
pub enum Error {
  Io(std::io::Error),
  Decode(DecodeError),   // { message: String, marker: Option<u16>, box_: Option<[u8;4]>, pos: Option<u64> }
  Encode(String),
  Unsupported(String),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

// ---- the decoded image ----
pub struct Image { /* as today, but with a decoded-component view */ }
impl Image {
  pub fn from_bytes(buf: &[u8]) -> Result<Self>;
  pub fn from_bytes_with(buf: &[u8], params: &DecodeParameters) -> Result<Self>;
  pub fn from_file(path) / from_file_with(path, params) -> Result<Self>;
  pub fn width/height/orig_width/orig_height/color_space/num_components;
  pub fn has_icc_profile(&self) -> bool;
  pub fn components(&self) -> &[ImageComponent];     // NEW: typed wrapper
  pub fn into_core_image(self) -> CoreImage;          // escape hatch
  pub fn encode(&self, params: &EncodeParameters) -> Result<Vec<u8>>;
  pub fn encode_jp2(&self, params: &EncodeParameters) -> Result<Vec<u8>>;
  pub fn encode_to_writer<W: Write+Seek>(&self, params, w) -> Result<()>;
  #[cfg(feature = "image")] pub fn get_pixels(&self, alpha_default: Option<u32>) -> Result<ImageData>;
      // keep the jpeg2k signature (owner decision); see doc/component-data-types.md
      // for the noted shortcomings (u32 alpha truncated for 8/16-bit output,
      // the Option-parameter shape is un-idiomatic for a reader method).
  #[cfg(feature = "image")] pub fn to_dynamic_image(&self) -> Result<DynamicImage>;
}

// NEW: typed component view (wraps CoreImageComp, hides resno_decoded etc.)
// Raw access is available WITHOUT any cargo feature.
pub struct ImageComponent {
  pub fn width/height/precision/signed/subsampling(dx,dy)/is_alpha;
  pub fn data(&self) -> &[i32];                       // raw, always available
  pub fn to_u8(&self) -> impl Iterator<Item=u8>;       // precision-aware, no feature
  pub fn to_u16(&self) -> impl Iterator<Item=u16>;     // (today's data_u8/u16, un-gated)
}

// ---- decode configuration ----
pub struct DecodeParameters { /* builder as today + */ }
impl DecodeParameters {
  pub fn new() -> Self;
  pub fn reduce / layers / strict / limits / decode_area / ignore_pclr_cmap_cdef;
  // NEW:
  pub fn threads(self, n: usize) -> Self;      // 0 = automatic (the configured rayon pool);
                                              // n > 0 = limit THIS decode to a local
                                              // n-thread rayon pool (owner decision)
}

// ---- encode configuration: fill the builder out to the C surface ----
pub struct EncodeParameters { inner: EncoderParameters }
impl EncodeParameters {
  pub fn new() -> Self;
  pub fn num_resolutions / num_layers / irreversible / progression_order /
      tile_size / layer_rate / comment / codeblock_width / codeblock_height;
  // NEW (currently only reachable via low_level()):
  pub fn precinct_size(self, w: u8, h: u8) -> Self;   // PPx/PPy for all resolutions
  pub fn precinct_sizes(self, &[(u8,u8)]) -> Self;    // per-resolution
  pub fn roi(self, compno: i32, shift: i32) -> Self;
  pub fn tile_parts(self, on: bool, mode: TilePartMode) -> Self; // R/C
  pub fn tlm(self, on: bool) -> Self;
  pub fn guard_bits(self, n: u32) -> Self;
  pub fn mct_matrix(self, matrix: &[f32], dc_shift: &[i32]) -> Self; // custom MCT
  pub fn rate(self, max_comp_size: u32) -> Self;
  pub fn codestream_comment(self, Option<&str>) -> Self; // distinguish "" from None
  pub fn low_level(&mut self) -> &mut EncoderParameters; // escape hatch stays
}

// ---- codestream introspection (wrap the raw structs) ----
pub struct CodestreamInfo { /* wraps CstrInfoV2 */ }
impl CodestreamInfo {
  pub fn grid(&self) -> (u32,u32,u32,u32,u32,u32); // tx0 ty0 tdx tdy tw th
  pub fn nb_comps(&self) -> u32;
  pub fn default_tile_info(&self) -> TileCodingParamInfo;
  pub fn tile_info(&self) -> TileCodingParamInfo;
}
pub struct CodestreamIndex { /* wraps the current struct, accessor-only */ }
  pub fn tile_indices(&self) -> &[TileIndex];
  pub fn markers(&self) -> &[MarkerInfo];       // NEW: expose the marker list
pub struct DumpImage { /* as today + set_decode_limits already wired */ }

// ---- the split low-level API (owner decision: Codec is split; the merged
//      type moves to the capi crate) ----
pub struct Decoder { inner: DecoderInner, event_mgr: EventMgr }
impl Decoder {
  pub fn new(format: J2KFormat) -> Result<Self>;
  pub fn set_strict / set_limits / set_components / set_decode_area /
      set_resolution_factor / set_threads;
  pub fn set_info_handler / set_warning_handler / set_error_handler;
  pub fn read_header(&mut self, &mut Stream) -> Result<HeaderImage>;
  pub fn decode(&mut self, &mut Stream) -> Result<CoreImage>;
  pub fn read_tile_header(&mut self, &mut Stream) -> Result<TileInfo>;
  pub fn decode_tile_data(&mut self, &mut Stream, tile: u32, out: Option<&mut [u8]>) -> Result<()>;
  pub fn get_decoded_tile(&mut self, &mut Stream, tile: u32) -> Result<CoreImage>;
  pub fn cstr_info(&self) -> Option<CodestreamInfo>;
  pub fn cstr_index(&self) -> Option<CodestreamIndex>;
  pub fn dump(&self, info_flag: i32) -> String;
}
pub struct Encoder { inner: EncoderInner, event_mgr: EventMgr }
impl Encoder {
  pub fn new(format: J2KFormat) -> Result<Self>;
  pub fn set_info_handler / set_warning_handler / set_error_handler / set_threads;
  pub fn setup(&mut self, &mut EncodeParameters, &mut CoreImage) -> Result<()>;
  pub fn start(&mut self, &mut CoreImage, &mut Stream) -> Result<()>;
  pub fn encode(&mut self, &mut Stream) -> Result<()>;
  pub fn write_tile(&mut self, tile: u32, &[u8], &mut Stream) -> Result<()>;
  pub fn finish(&mut self, &mut Stream) -> Result<()>;
  pub fn extra_options(&mut self, &[&str]) -> Result<()>;
  pub fn cstr_info(&self) -> Option<CodestreamInfo>;
  pub fn cstr_index(&self) -> Option<CodestreamIndex>;
  pub fn dump(&self, info_flag: i32) -> String;
}
// The openjp2-capi crate defines ITS OWN merged type to keep the single
// opj_codec_t* pointer contract, and marshals the C-shaped parameters there:
//
//   capi::Codec { enum { Dec(openjp2::Decoder), Enc(openjp2::Encoder) } }
//
// so the core's public signatures contain NO C types (opj_dparameters and
// friends are translated inside the capi crate).
```

Notes:
- The split replaces today's `Codec` entirely in the core. Internally
  `Decoder`/`Encoder` hold the `CodecFormat` enum (J2K/JP2) plus an
  `EventMgr`; the six methods they share (handlers, `set_threads`, `dump`,
  cstr accessors) are duplicated or factored into a tiny `EventHandlers`
  helper — there is no user-visible merged type in the core anymore.
- `HeaderImage` = a `CoreImage` with empty component data (header-only) so
  callers cannot mistake it for decoded pixels.
- The safe API (`Image`, `DumpImage`) is rewritten on top of `Decoder`
  (and the encode drivers on `Encoder`); nothing user-visible changes there.
- The capi crate's merged `Codec` keeps today's `opj_codec_t*` ABI and is
  where `opj_dparameters_t`/`opj_image_cmptparm_t` marshalling lives.

# Part B — Proposed internal structure

## B.0 Codec split & C-type extraction (owner decisions)

1. **Split `Codec` (core)** — ✅ **LANDED (793ff501)**: `codec.rs` now exposes
   `Decoder` and `Encoder`, each owning the `CodecFormat` enum + an
   `EventMgr`.  (The original item text follows: replace `codec.rs`'s
   `CodecType`/`CodecFormat` facade with two public types, `Decoder` and
   `Encoder`, each owning the `CodecFormat` enum + an `EventMgr`.) The 24 methods partition cleanly
   (verified): 14 → `Decoder`, 7 → `Encoder`, 6 shared (3 handlers,
   `set_threads`, `dump_codec`, cstr accessors) via a small internal
   `EventHandlers`/`Threads` helper. The safe API drivers switch from
   `Codec` to the split types; the `opj_codec_private_t` alias disappears
   from the core.
2. **Merged `Codec` lives in the capi crate.** — ✅ **LANDED (793ff501)**: the
   capi defines
   `Codec { Dec(core::Decoder), Enc(core::Encoder) }` to keep the
   single `opj_codec_t*` pointer contract, and becomes the *only* place where
   the C-shaped parameters (`opj_dparameters_t`, `opj_image_cmptparm_t`,
   `opj_cparameters_t`) are marshalled. The core's public signatures become
   C-type-free.
3. **`openjp2-ctypes` crate (new workspace member).** — ✅ **LANDED
   (b0538729 + 603aec32)**: the C-ABI mirror types, the `consts::opj`
   constants and `opj_image_comptparm` moved; the core keeps a private
   `c_api_types` alias for its internal use and re-exports the types only
   under `openjp2::ffi`.  (Original item text: moves all of today's
   `c_api_types.rs` plus the C-header-mirror constants (`consts::{opj, cio,
   event}`, the ABI constants in `consts::j2k`) and `opj_image_comptparm`.)
   Ownership inventory (verified): the core *internally* uses `OPJ_BOOL`
   (~280 sites in 12 modules), `OPJ_COLOR_SPACE`, `OPJ_PROG_ORDER`+constants,
   `opj_poc_t`, `CODEC_FORMAT`, `opj_dparameters_t`, RSIZ/CINEMA modes —
   these keep working by depending on the crate; the purely-C items
   (`opj_codec_t`, `opj_stream_t`, callback fn-pointer typedefs,
   `opj_packet_info_t`, `opj_marker_info_t`, `opj_tp_index_t`) are used only
   by the capi and move wholesale. Root `pub use c_api_types::*` is removed
   in the 0.7 release (kept under `ffi` as a re-export of the ctypes crate).
   **Phasing:** (a) extract + core depends + capi depends (non-breaking);
   (b) 0.7 (unpublished, so breaking changes are fine): drop the root
       re-export, split `Codec`, move the marshalling;
   (c) optional later: replace `OPJ_BOOL` etc. *inside* the core with Rust
   types (bool/i32/enums) so the core no longer imports the ctypes crate at
   all.

## B.1 Decoder

1. **Kill the procedure lists.** `opj_j2k_setup_decoding`'s list has exactly
   one entry; the header writer/reader lists are fixed sequences. Replace
   `ProcedureList` with plain method calls (simpler, greppable, no fn-pointer
   table). Low risk, mechanical.
2. **Marker classification cleanup (review_3 M3).** Make
   `EPH/SOC/SOD/EOC` behave like the C's `read_unk` (skip + rescan) so corrupt
   streams recover identically; special-case `SOD` explicitly in the
   tile-part loop where it terminates the header (study the C's
   `opj_j2k_read_unk` + `read_tile_header` interplay first — this is the one
   item the fix round left open on purpose).
3. **Thread-local T1 → owned per-job workspace.** Replace
   `thread_local! { T1 }` with an `Arc<T1Pool>` (one workspace per rayon
   worker, reused across codeblocks within a tile) or a per-call `T1` when
   there is no rayon. This removes the decode→encode `Mqc.buf[0]` sharing
   hazard *structurally* (the sentinel fix stays as defense) and makes the
   encoder state independent of the previous decode on the same thread.
4. **Close the f32 reinterpret gap without unsafe.** `TileBuf` becomes
   `enum TileBuf { I32(Vec<i32>), F32(Vec<f32>) }` selected by the tilecomp's
   `qmfbid`: the 9-7 DWT/MCT/dc-shift borrow `&mut [f32]` directly, the t1
   write-back and `update_tile_data` use `from_bits`/`to_bits` at the seams.
   Drops the ~14 MB whole-tile 9-7 scratch (the C reinterprets in place) —
   the last remaining decode-memory delta vs the C.
5. **Fallible-allocation sweep (review findings).** `try_reserve` for the TLM
   `m_tile_part_infos`, `check_poc_val`'s matrix, pi's `include` mask
   (allocate lazily or cap), the dwt scratch (post-item 4 it shrinks), and
   the tag-tree node vector — every decode-path allocation sized from header
   values fails gracefully instead of aborting.
6. **Parallel decode v2.** Replace the `Arc<Vec<u8>>` snapshot + raw
   `FF 90 00 0A` prescan with the TLM/tp_index seek plan already used for
   decode areas, or a double-buffered tile-header reader; eliminate the
   review_3 L13 prescan false-positive class and the full-stream copy
   (B.1-memory).
7. **Structured errors.** Thread a `pos`/`marker` context into `event_msg!`
   at the marker loop and box loop so `Error::Decode` can carry where the
   failure happened (helps users and the fuzz triage).

## B.2 Encoder

1. **Unify serial/parallel tile encoding around one pure function.**
   Extract `encode_tile(cp, image, tile_no) -> Result<Vec<u8>>` (the current
   worker body, minus the stream writes); the serial path calls it in a loop,
   the parallel path calls it under rayon, and the master stays the only
   writer of markers/TLM. This makes the byte-identity guarantee structural
   and lets the TLM multi-part guard live in one place.
2. **Tier-1 encode workspace isolation** (same as B.1.3): no cross-tile
   `Mqc` reuse through the thread-local.
3. **Fix the remaining encoder parity gaps tracked from the reviews**
   (each with a regression test): fixed-layer rate allocation index clamp
   (review_2 LOW-4 / review_3 L11), `numpix` as `OPJ_SIZE_T` (L12), empty
   COM comment flag (L4), `set_threads` return-value parity (L2), encode-side
   codestream index recording (currently absent — `opj_get_cstr_index` after
   encode yields nothing).
4. **Custom MCT end-to-end.** The inversion is fixed; upstream cannot decode
   COD mct=2 (both C and port reject it). Decide: keep C parity (encode-only
   feature, as today) or emit the MCT data group + accept mct=2 on decode as
   a deliberate, documented extension.
5. **Precision/clipping hygiene.** Centralize the "prec 1..=31" and clipping
   rules in one place (setup_encoder + `CoreImageComp::clip`) so new encode
   entry points can't bypass validation.

## B.3 Cross-cutting

1. **`Stream` for in-memory bytes.** Add `Stream::from_bytes(&[u8])` (no
   `BufReader<Cursor>` indirection) for the common case; keep `new_input` for
   arbitrary `Read+Seek`.
2. **Module hygiene.** `api.rs` (2240+ lines incl. tests) splits into
   `api/{error,decode,encode,image_conv,dump}.rs`; the 200-line test modules
   stay in the parent or move to `tests/`.
3. **Panic-catch scope.** Keep `catch_unwind` at the safe-API and C-ABI
   boundaries (do not wrap individual internal calls).
4. **Remaining open review items** (schedule inside this redesign rather than
   as one-offs): t1 refinement-pass wrapping (review_2 M7), `opj_lrintf` NaN
   parity (review_3 M12 — needs an empirical trigger first), `DumpImage`
   unify with `Decoder`, the `LayerRate` builder doc/behavior, the
   `Jp2BoxType::UUID` constant typo, `GUARD_BITS` parse parity.

# Part C — Design questions

## Decided (owner, 2026-11-10)

1. **Image naming** → keep `openjp2::Image`.
2. **Codec split** → core `Decoder`/`Encoder`; the merged `Codec` moves into
   the capi crate. The deprecated methods go with it in 0.7 (unpublished, so
   the breaking change lands there directly — no 0.8 staging).
3. **C-ABI types** → all move to a new `openjp2-ctypes` crate; the capi crate
   is their only consumer at the boundary.
4. **`image` crate** → stays optional; raw component access always available.
5. **jpeg2k naming parity** → keep only where free.
6. **API style** → follow the Rust API guidelines
   (https://rust-lang.github.io/api-guidelines/) as much as possible; the
   proposal in Part A should be re-checked against them before
   implementation (C-INTERMEDIATE, C-GOOD-ERR, C-NO-OUT, naming, builders).
7. **Encoder knobs** → common knobs on the `EncodeParameters` builder;
   advanced knobs stay on `low_level()`.
8. **Custom MCT decode** → extend beyond C parity (accept mct=2 on decode),
   but only AFTER the API refactor.
9. **Component data typing** → i32-only for 0.7; the future design for
   f32/i32 (and possibly u32 / higher-resolution pixels) is written up in
   `doc/component-data-types.md`.
10. **Threading** → the user can limit the threads used for a decode, while
    automatic (the configured rayon pool) stays supported:
    `DecodeParameters::threads(n)` with `0` = automatic, `n > 0` = a local
    n-thread pool for that decode.
11. **`get_pixels` alpha** → keep the jpeg2k signature
    (`alpha_default: Option<u32>`); the shortcomings are noted in
    `doc/component-data-types.md`.
12. **Unsafe policy for the DWT reinterpret (typed `TileBuf` vs a scoped
    exception)** → deferred (question was unclear; revisit with a concrete
    writeup when B.1.4 is scheduled).
13. **Parallel-decode rework (B.1.6)** → deferred until after the split.
14. **Procedure-list removal (B.1.1)** → yes, remove them.
15. **Docs layout** → keep all five (+ the new component-data doc).
16. **`OPJ_*` internals (B.0 phase c)** → incremental migration to primitive
    Rust types; where `OPJ_BOOL` signals an error it becomes
    `Result<(), Error>` (incrementally is fine).

No questions remain open for the 0.7 cycle; items 8, 12 and 13 are explicitly
deferred and will be revisited with a concrete writeup each.
