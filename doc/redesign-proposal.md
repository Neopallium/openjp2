# openjp2 — Proposed Public API & Internal Structure

Written 2026-11-10. Builds on `doc/public-api-summary.md` (current state),
`doc/internals-summary.md` (current pipelines) and `doc/api-review.md` (the
jpeg2k-parity work, mostly landed). The goal of this document: propose the
*next* iteration of the public API and the internal architecture, and record
the design questions that need your decision.

# Part A — Proposed public API (v0.8)

## A.1 Guiding principles

1. One primary surface: the safe one-shot + builder API. The low-level
   `Codec` becomes a typed, `Result`-returning advanced API behind a
   `raw`/`codec` module, no longer deprecated-in-place.
2. No C types in the default namespace: `OPJ_*` mirrors exist **only** under
   `openjp2::ffi` (kept for the capi crate; normal users never import them).
3. Every fallible call returns `Result<_, Error>`; panics are caught at the
   boundary (already true for the safe API — extend to the new low-level API).
4. Feature-light core: `image`-crate interop and rayon stay optional.

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
  #[cfg(feature = "image")] pub fn get_pixels(&self) -> Result<ImageData>; // drop alpha_default
  #[cfg(feature = "image")] pub fn to_dynamic_image(&self) -> Result<DynamicImage>;
}

// NEW: typed component view (wraps CoreImageComp, hides resno_decoded etc.)
pub struct ImageComponent {
  pub fn width/height/precision/signed/subsampling(dx,dy)/is_alpha;
  pub fn data(&self) -> ComponentData<'_>;   // enum below
  pub fn to_u8(&self) -> impl Iterator<Item=u8>;   // precision-aware scaling
  pub fn to_u16(&self) -> impl Iterator<Item=u16>;
}
pub enum ComponentData<'a> { I32(&'a [i32]) }   // future: F32 for 9-7 raw?

// ---- decode configuration ----
pub struct DecodeParameters { /* builder as today + */ }
impl DecodeParameters {
  pub fn new() -> Self;
  pub fn reduce / layers / strict / limits / decode_area / ignore_pclr_cmap_cdef;
  // NEW:
  pub fn threads(self, n: usize) -> Self;      // explicit opt-in for tile parallelism
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

// ---- the typed low-level API (replaces the deprecated Codec methods) ----
pub mod raw {
  pub struct Decoder { codec: Codec }   // J2K or JP2 behind the enum
  impl Decoder {
    pub fn new(format: J2KFormat) -> Result<Self>;
    pub fn set_strict / set_limits / set_components / set_decode_area /
        set_resolution_factor;
    pub fn read_header(&mut self, &mut Stream) -> Result<HeaderImage>;
    pub fn decode(&mut self, &mut Stream) -> Result<CoreImage>;
    pub fn read_tile_header(&mut self, &mut Stream) -> Result<TileInfo>;
    pub fn decode_tile_data(&mut self, &mut Stream, tile: u32, out: Option<&mut [u8]>) -> Result<()>;
    pub fn get_decoded_tile(&mut self, &mut Stream, tile: u32) -> Result<CoreImage>;
    pub fn cstr_info(&self) -> Option<CodestreamInfo>;
    pub fn cstr_index(&self) -> Option<CodestreamIndex>;
  }
  pub struct Encoder { codec: Codec }
  impl Encoder {
    pub fn new(format: J2KFormat) -> Result<Self>;
    pub fn setup(&mut self, &mut EncodeParameters, &mut CoreImage) -> Result<()>;
    pub fn start(&mut self, &mut CoreImage, &mut Stream) -> Result<()>;
    pub fn encode(&mut self, &mut Stream) -> Result<()>;
    pub fn write_tile(&mut self, tile: u32, &[u8], &mut Stream) -> Result<()>;
    pub fn finish(&mut self, &mut Stream) -> Result<()>;
    pub fn extra_options(&mut self, &[&str]) -> Result<()>;
  }
}
```

Notes:
- `Codec` stays `pub` (re-exported) for the capi crate and exotic users, but
  the deprecated methods are removed in 0.8; `raw::{Decoder,Encoder}` wrap it
  and convert everything to `Result` (a panic inside is still caught by the
  existing guards).
- `HeaderImage` = a `CoreImage` with empty component data (header-only) so
  callers cannot mistake it for decoded pixels.

# Part B — Proposed internal structure

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
   unify with `raw::Decoder`, the `LayerRate` builder doc/behavior, the
   `Jp2BoxType::UUID` constant typo, `GUARD_BITS` parse parity.

# Part C — Design questions for you

1. **Image naming.** Keep `openjp2::Image` (collides conceptually with
   `image::DynamicImage` but never in the same namespace) or rename to
   `DecodedImage`/`Jp2Image`? The jpeg2k wrapper parity argues for keeping
   `Image`.
2. **Breaking change timing.** The typed `raw::{Decoder,Encoder}` + removal
   of the deprecated `Codec` methods is a 0.8 breaking change. Do it in one
   release, or keep the deprecated shims for one more cycle?
3. **Error granularity.** Is the structured `DecodeError { marker/box, pos }`
   worth it, or is `String` sufficient for 0.8?
4. **Encoder surface completeness.** Do you want the full builder (precincts,
   ROI, tile-parts, TLM, guard bits, rates) in `EncodeParameters`, or should
   advanced knobs stay on `low_level()`?
5. **Custom MCT decode.** Extend beyond C parity (accept mct=2 on decode,
   since we can) or stay strict-parity? (See B.2.4.)
6. **Component data typing.** Is `ComponentData::I32` enough, or should the
   API expose the raw f32 planes for 9-7 files (post-B.1.4)?
7. **Threading default.** Keep automatic tile parallelism in the one-shot API
   (today's behavior), or make it opt-in (`DecodeParameters::threads`)?
   Today's parallelism is byte-identical and bounded, but implicit.
8. **`get_pixels` alpha.** Drop the `alpha_default: Option<u32>` parameter
   (A.2) in favour of a separate `with_alpha` variant, or keep jpeg2k's
   signature exactly?
9. **Unsafe policy for the DWT reinterpret.** B.1.4 avoids unsafe via a typed
   `TileBuf`; alternatively a single `#[allow(unsafe_code)]` reinterpret
   block would be smaller (~15 sites). Is `forbid(unsafe_code)` absolute, or
   is a scoped, documented exception acceptable for the i32/f32 view?
10. **Parallel-decode rework (B.1.6).** Worth the effort now (removes the
    snapshot copy + prescan hazard), or defer until the memory audit asks
    for it?
11. **Procedure-list removal (B.1.1).** The lists mirror the C's design and
    helped parity review; is a mechanical simplification acceptable, or do
    you prefer keeping the C mirror as-is for future upstream syncs?
12. **Docs.** Should these three docs (public-api-summary, internals-summary,
    redesign-proposal) replace `api-review.md` + `code-outline.md`, or live
    alongside them?
