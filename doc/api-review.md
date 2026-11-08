# openjp2 Public API Review & Proposal

Goal: make the `openjp2` crate's public API follow Rust best practices and
align it with the [`jpeg2k`](https://crates.io/crates/jpeg2k) crate
(v0.10.1, same author) so that `jpeg2k` can become a thin wrapper /
re-export of `openjp2` instead of binding `openjpeg-sys` (the C library).

## 1. Current `openjp2` public surface (problems)

The crate currently exports (via `src/lib.rs`):

- `pub use c_api_types::*` — the whole C-ABI mirror: `OPJ_BOOL` (= i32),
  `opj_dparameters`, `opj_image_cmptparm`, `CODEC_FORMAT`, `OPJ_CLRSPC_*`
  constants, ... i.e. the *C header shape* leaks into every user's
  namespace.
- `Codec` — a monolithic struct with 25+ methods mixing setup, streaming
  decode, tile decode, encode and dump concerns; returns `OPJ_BOOL`
  (`i32`) and `Option<CoreImage>` instead of `Result<_, Error>`.
- `CoreImage`/`CoreImageComp` — an owned image with `Vec<i32>` component
  data (fine), but the API does not distinguish "header-only" from
  "decoded" images, and there is no typed colour-space enum on the safe
  side (it uses the C `OPJ_COLOR_SPACE`).
- `EncoderParameters` — a safe owned params struct (good) but with the
  C field names (`tcp_numlayers`, `numresolution`, ...) and 50+ public
  fields; no builder, no validation on assignment.
- `Stream` — Read/Write/Seek adapter (good); `detect_format_from_file` /
  `detect_format` functions at the crate root (ok, but naming/errors are
  inconsistent: `Result<J2KFormat, String>`).
- No `Error` type: failures are `Option::None`, `0`, or `Result<_, String>`.

Pain points, in order of severity:

1. No unified error type; callers cannot match on failure reasons.
2. `OPJ_BOOL`-returning APIs (i32 success codes).
3. The C mirror types in the default namespace.
4. `EncoderParameters` is a 50-field bag, not a builder.
5. Decode flow requires manually threading `CoreImage` through
   `read_header -> decode -> end_decompress` (three calls) instead of a
   one-shot `Image::from_bytes` style API.
6. No `ColorSpace` type on the safe side; no typed component view.

## 2. The `jpeg2k` crate's shape (the target)

`jpeg2k` wraps `openjpeg-sys` with:

```rust
// error.rs
pub enum Error { Io(io::Error), Decode(String), Encode(String), Unsupported(String), ... }
pub type Result<T, E = Error> = std::result::Result<T, E>;

// format.rs
pub enum J2KFormat { J2K, JP2, JPT, JPP, JPX, Unknown }
pub fn j2k_detect_format(buf: &[u8]) -> Result<J2KFormat>;
pub fn j2k_detect_format_from_extension(ext: Option<&OsStr>) -> Result<J2KFormat>;

// j2k_image.rs
pub struct Image { /* ... */ }
impl Image {
  pub fn from_bytes(buf: &[u8]) -> Result<Self>;
  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>;
  pub fn from_bytes_with(buf: &[u8], params: DecodeParameters) -> Result<Self>;
  pub fn from_file_with<P: AsRef<Path>>(path: P, params: DecodeParameters) -> Result<Self>;
  pub fn width(&self) -> u32; pub fn height(&self) -> u32;
  pub fn orig_width(&self) -> u32; pub fn orig_height(&self) -> u32;
  pub fn color_space(&self) -> ColorSpace;
  pub fn num_components(&self) -> u32;
  pub fn has_icc_profile(&self) -> bool;
  pub fn components(&self) -> &[ImageComponent];
  pub fn get_pixels(&self, alpha_default: Option<u32>) -> Result<ImageData>;
}
pub struct ImageComponent { /* width/height/precision/bpp/alpha/signed + data() -> &[i32], data_u8/u16 iterators */ }

// codec.rs
pub struct DecodeParameters { /* builder */ }
impl DecodeParameters {
  pub fn new() -> Self;
  pub fn reduce(mut self, reduce: u32) -> Self;
  pub fn strict(mut self, strict: bool) -> Self;
  pub fn layers(mut self, layers: u32) -> Self;
  pub fn decode_area(mut self, area: Option<DecodeArea>) -> Self;
  pub fn ignore_pclr_cmap_cdef(mut self) -> Self;
}
pub struct DecodeArea { pub start_x: u32, pub start_y: u32, pub end_x: u32, pub end_y: u32 }
pub struct EncodeParameters(/* wraps opj_cparameters */);
pub struct CodestreamIndex(/* wraps opj_codestream_index_t */);   // markers()/tile_indices()
pub struct CodestreamInfo(/* wraps opj_codestream_info_v2_t */);  // tile_info()/default_tile_info()
pub struct TileCodingParamInfo(/* wraps opj_tccp_info_t */);      // csty()/prg()/numlayers()/mct()/tccp_infos()
pub struct DumpImage<'a>(/* dump helper */);
```

Cargo features: `default = ["image", "threads", "file-io", "openjpeg-sys"]`.

This shape is exactly what a safe wrapper should look like: one-shot
`Image` construction, a `DecodeParameters` builder, typed `Result`
errors, `&[ImageComponent]` slice views, a `ColorSpace` enum, and
`image`-crate integration (`get_pixels`/`ImageData`).

## 3. Proposed new `openjp2` API

### 3.1 Errors

```rust
pub enum Error {
  Io(std::io::Error),
  Decode(String),      // corrupt / unsupported codestream
  Encode(String),      // invalid encode parameters / IO during encode
  Unsupported(String), // e.g. format/feature not supported by the build
}
pub type Result<T, E = Error> = std::result::Result<T, E>;
```

All public fallible functions return `Result<..>`. The C ABI layer keeps
returning `OPJ_BOOL` (the C contract) and maps to/from `Error` at the
boundary.

### 3.2 Types

```rust
pub enum ColorSpace { Unknown, Srgb, Gray, Sycc, Eycc, Cmyk }   // safe mirror of OPJ_CLRSPC_*
pub struct Image { /* owned */ }
pub struct ImageComponent { /* owned component */ }
pub struct DecodeArea { pub start_x: u32, pub start_y: u32, pub end_x: u32, pub end_y: u32 }
pub struct DecodeParameters { /* builder: reduce/strict/layers/decode_area/ignore_pclr_cmap_cdef */ }
pub struct EncodeParameters { /* builder over the current EncoderParameters fields */ }
pub struct CodestreamIndex / CodestreamInfo / TileCodingParamInfo  // owned safe accessors
pub enum J2KFormat { J2K, JP2, JPT, JPP, JPX, Unknown }
```

### 3.3 High-level entry points

```rust
impl Image {
  pub fn from_bytes(buf: &[u8]) -> Result<Self>;
  pub fn from_bytes_with(buf: &[u8], params: DecodeParameters) -> Result<Self>;
  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>;               // feature file-io
  pub fn from_file_with<P: AsRef<Path>>(path: P, params: DecodeParameters) -> Result<Self>;
  pub fn width(&self) -> u32;  pub fn height(&self) -> u32;
  pub fn color_space(&self) -> ColorSpace;
  pub fn num_components(&self) -> u32;
  pub fn has_icc_profile(&self) -> bool;
  pub fn components(&self) -> &[ImageComponent];
  pub fn encode(&self, params: EncodeParameters) -> Result<Vec<u8>>;       // or to_stream
}

pub fn detect_format(buf: &[u8]) -> Option<J2KFormat>;
pub fn detect_format_from_extension(ext: Option<&OsStr>) -> Option<J2KFormat>;
```

The existing `Codec` remains for the streaming/tile-level use cases
(opj_decode_tile_data, opj_get_decoded_tile parity) but its methods get
`Result` returns and it is re-exported from a `codec` module rather than
the crate root.

### 3.4 What stays as-is

- `Stream` (Read/Write/Seek) — keep, add `from_reader` (PR #6's idea).
- `CodestreamIndex`/`CstrInfoV2`/`CstrTccpInfoV2` safe structs — keep,
  wrap them in the accessor types above.
- The C mirror types move behind a `pub mod ffi` (`openjp2::ffi::*`)
  instead of `pub use c_api_types::*`, so the C names stop polluting the
  default namespace while staying available for C-ABI users.
- Cargo features: rename/align with `jpeg2k` — `default = ["image",
  "threads", "file-io"]` where `threads` = rayon and `image` = the
  `image`-crate interop (`ImageData`, `DynamicImage` conversions).

### 3.5 Making `jpeg2k` a thin wrapper

With the above, `jpeg2k` reduces to:

- re-exports of `openjp2`'s `Image`/`DecodeParameters`/`EncodeParameters`/
  `Error`/`J2KFormat`/index types;
- its `openjpeg-sys` dependency dropped entirely (no C toolchain needed);
- its `ImageData`/`image` interop moves into `openjp2` behind the `image`
  feature (or stays in `jpeg2k` as an extension trait).

Migration path (non-breaking first, breaking second):

1. Add `Error`/`Result` and the new types alongside the old API; deprecate
   the `OPJ_BOOL`-returning methods with `#[deprecated]`.
2. Add the `Image::from_*` one-shot API + `DecodeParameters` builder.
3. Move the C mirrors to `pub mod ffi` and re-export for compatibility.
4. Release 0.8 with the old API removed.

## 4. Open questions

- `Image` name collision: `openjp2::Image` vs `image::DynamicImage` — fine
  (different crates), but consider `Jp2Image` if ambiguity hurts.
- Keep `Codec` public at the root or behind `codec::`? Proposal: root
  `Image`-centric API; `Codec` re-exported for advanced use.
- `get_pixels`/`ImageData` in `openjp2` itself (pulling in the `image`
  crate) or left to wrappers? Proposal: behind the `image` feature to
  keep the core dependency-free.
- Naming parity: match `jpeg2k`'s exact fn names (`from_bytes`,
  `color_space`, `components`) so wrapper code is mechanical.

---

## Implementation status (2026-11-08 marathon)

| Proposal | Status |
|---|---|
| `Error`/`Result` | ✅ `openjp2::{Error, Result}` (`src/api.rs`) |
| `ColorSpace` | ✅ safe enum with `From` conversions |
| `DecodeArea`/`DecodeParameters` builder | ✅ (reduce/layers/strict/decode_area/ignore_pclr_cmap_cdef) |
| `Image` one-shot API | ✅ from_bytes/from_bytes_with/from_file/from_file_with + accessors + into_core_image |
| `EncodeParameters` builder + `Image::encode`/`encode_jp2` | ✅ (num_resolutions/num_layers/irreversible/codeblock sizes/progression_order/tile_size/layer_rate/comment) |
| `image`-crate interop | ✅ behind the `image` feature: `ImageData`/`ImagePixelData`, `get_pixels`, `TryFrom<&Image> for DynamicImage` |
| Index/info accessors | ✅ `CodestreamIndex`/`CstrInfoV2` jpeg2k-style accessors |
| `DumpImage` | ✅ (decode/index/info/dump text) |
| `ffi` module for the C mirrors | ✅ `openjp2::ffi` (the old glob re-exports stay for compatibility) |
| `threads` feature alias | ✅ (`threads = ["rayon"]`) |
| jpeg2k thin-wrapper PoC | ✅ `/tmp/jpeg2k-openjp2` runs jpeg2k's README/convert/save/dump examples over openjp2 with no `openjpeg-sys` |
| `Codec` Result-ification + deprecations | ✅ deprecations done (47c60c7f); the `Result`-typed `Codec` itself stays deferred (breaking-change prep, migration steps 1-2) |
| Component `data_u8`/`data_u16` iterators on the openjp2 side | ✅ `CoreImageComp::data_u8`/`data_u16` (feature `image`) |
| Concurrent tile decode / encode | ✅ whole-image multi-tile decodes and multi-tile encodes run tiles in parallel (rayon), byte-identical to the serial paths (04d9170a, dea21597, 93ae8fd8) |

The remaining open questions from §4 are unchanged; `openjp2::Image`
keeps the `image`-crate name ambiguity accepted for now.  The
intentional divergences from the C reference (threading model,
removed jpwl/jpspec features, SIMD, safety hardening) are listed in
`doc/differences.md`.
