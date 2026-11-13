# openjp2 — Current Public API Structure

Updated 2026-11-13, against 0.7.2. This is a factual map
of the public surface as it exists today (the *proposal* is in
`doc/redesign-proposal.md`; the earlier `doc/api-review.md` tracks the
jpeg2k-parity work).

The crate has **two API layers**:

1. **Safe high-level API** (`openjp2::api`, re-exported at the crate root) —
   the primary surface: one-shot decode/encode, builders, typed errors,
   image-crate interop.
2. **Low-level `Decoder` / `Encoder` API** (`src/codec.rs`, 0.7 split) —
   streaming/tile-level control, C-ABI-shaped signatures with a C-free
   `Result` surface.  The merged `Codec` handle moved to the
   `openjp2-capi` crate (its `opj_codec_t*` contract).

The C-ABI mirrors are **not part of this crate**: the C header's
types/constants live in the separate `openjp2-ctypes` crate, and the
`#[no_mangle]` C functions in `openjp2-capi`.  The core's public API is
C-name-free (no `OPJ_*` types/functions; only the deliberate
`ColorSpace`/`ProgressionOrder` conversion impls mention the C values).

## Layer 1 — safe high-level API

### `Error` / `Result`

```rust
pub enum Error {
    Io(std::io::Error),
    Decode(DecodeError),
    Encode(String),
    Unsupported(String),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct DecodeError {
    pub message: String,
    pub marker: Option<u16>,        // e.g. Some(0xFF51) for SIZ
    pub box_: Option<[u8; 4]>,      // e.g. Some(*b"jp2h")
    pub pos: Option<u64>,           // stream position of the failure
}
```

`Error::Decode` carries the structured decode-failure context (plus
"internal decoder panic: …" when `catch_unwind` converts a panic).  All
safe-API fallible functions return `Result`.

### `Image` (the decoded image)

```rust
// construction
Image::from_bytes(&[u8]) -> Result<Self>
Image::from_bytes_with(&[u8], DecodeParameters) -> Result<Self>
Image::from_file(path) -> Result<Self>                  // reads file to memory
Image::from_file_with(path, DecodeParameters) -> Result<Self>
Image::from_core_image(CoreImage) -> Self               // escape hatch (low-level)
Image::from_dynamic_image(&image::DynamicImage) -> Self // feature "image"
// accessors
Image::width() -> u32            // x1 - x0
Image::height() -> u32           // y1 - y0
Image::color_space() -> ColorSpace
Image::num_components() -> u32
Image::has_icc_profile() -> bool
Image::components() -> &[CoreImageComp]
Image::into_core_image(self) -> CoreImage
// encoding (by value: the encoder transforms the buffers in place;
// clone first to keep the source)
Image::encode(self, EncodeParameters) -> Result<Vec<u8>>         // J2K codestream
Image::encode_jp2(self, EncodeParameters) -> Result<Vec<u8>>     // JP2 file
Image::encode_to_writer(self, EncodeParameters, W: Write+Seek) -> Result<()>
// pixel extraction (feature "image")
Image::get_pixels(&self, alpha_default: Option<u32>) -> Result<ImageData>
// conversions (feature "image")
TryFrom<&Image> for image::DynamicImage
```

### `DecodeParameters` (builder)

```rust
DecodeParameters::new() -> Self
  .reduce(u32)                    // 2^reduce resolution reduction
  .layers(u32)                    // 0 = all quality layers
  .strict(bool)                   // default false (lenient, -allow-partial parity)
  .threads(usize)                 // 0 = ambient pool (default), n > 0 = local n-thread pool
  .limits(DecodeLimits)           // resource caps for untrusted input
  .decode_area(Option<DecodeArea>)
  .ignore_pclr_cmap_cdef()
```

### `DecodeLimits`

```rust
DecodeLimits::new()
  .max_image_size(w, h)           // 0 = unlimited
  .max_tile_pixels(pixels)        // 0 = unlimited
  .max_mem_alloc(bytes)           // 0 = built-in 1 GiB ceiling
  .max_metadata_len(bytes)        // ICC profiles, JP2 boxes, comments
// const DEFAULT_DECODE_MEM_LIMIT: u64 = 1 << 30
```

### `EncodeParameters` (builder over `EncoderParameters`)

```rust
EncodeParameters::new()
  .num_resolutions(u32)           // DWT levels + 1
  .num_layers(u32)
  .irreversible(bool)             // 9-7 vs 5-3
  .progression_order(ProgressionOrder)
  .tile_size(w, h)
  .layer_rate(f32)                // rate-distortion ratio target (<=100 layers)
  .comment(&str)
  .codeblock_width(u32) / .codeblock_height(u32)   // powers of two
  .low_level() -> &mut EncoderParameters   // escape hatch to the C-shaped fields
  .into_inner() -> EncoderParameters
```

Rate targets are what split code-block passes across quality layers: a
bare layer count leaves everything in the final layer (so decoders see no
difference between N layers and all of them).  The CLI and the WASM
bindings therefore generate a geometric ladder (100:1 … lossless) when
layers > 1 without explicit rates.

### Enums

- `ColorSpace { Unknown, Srgb, Gray, Sycc, Eycc, Cmyk }` (+ `From` both
  ways with `openjp2-ctypes`' `OPJ_COLOR_SPACE`).
- `ProgressionOrder { Lrcp, Rlcp, Rpcl, Pcrl, Cprl }`.
- `J2KFormat { JP2, J2K, JPT }` (JPT is recognized but unsupported) +
  free fns `detect_format(&[u8])`, `detect_format_from_extension`,
  `detect_format_from_file`.

### `DecodeArea` / `DumpImage`

```rust
DecodeArea::new(start_x, start_y, end_x, end_y)
DumpImage::from_bytes / from_bytes_with / from_file / from_file_with
  .decode() -> Result<()>
  .get_codestream_index() -> Option<CodestreamIndex>
  .get_codestream_info() -> Option<CstrInfoV2>
  .dump(info_flag: i32) -> String
// public dump flags: IMG_INFO, J2K_MH_INFO, J2K_MH_IND, ALL
```

### `ImageData` / `ImagePixelData` (feature "image")

`ImageData { width, height, format: ExtendedColorType, data: ImagePixelData }`;
`ImagePixelData` is an enum of `L8/La8/Rgb8/Rgba8` (`Vec<u8>`) and
`L16/La16/Rgb16/Rgba16` (`Vec<u16>`).

## Layer 2 — low-level `Decoder` / `Encoder`

```rust
// construction (J2KFormat; Result-returning)
Decoder::new(J2KFormat) -> Result<Self>
Encoder::new(J2KFormat) -> Result<Self>
// events (shared; feature "event-handlers" for the C-style callbacks)
.set_info_handler / .set_warning_handler / .set_error_handler(FnMut(&str)+Send+'static)
// threading (sizes the process-wide rayon pool; first call wins)
.set_threads(i32) -> Result<()>
// dump / header info (shared)
.dump_codec<W: StringWriter>(info_flag, &mut W)
.cstr_info() -> Option<CstrInfoV2>
.cstr_index() -> Option<CodestreamIndex>    // marker/tile-part/packet structure
.get_tile_geometry() -> (tdx, tdy, cblkw, cblkh)
// decoder
Decoder::apply_settings(&DecoderSettings) -> Result<()>   // layers/reduce/
                                                          // ignore_pclr_cmap_cdef/dump_state/threads
Decoder::set_strict(bool) / set_limits(DecodeLimits) / set_components(&[u32]) /
    set_decode_area(&mut CoreImage, Option<DecodeArea>) / set_resolution_factor(u32) -> Result<()>
Decoder::read_header(&mut Stream) -> Result<HeaderImage>  // header-only (empty data)
Decoder::decode(&mut Stream) -> Result<CoreImage>         // lenient mode returns the
                                                          // partial image (documented contract)
Decoder::end_decompress(&mut Stream) -> Result<()>
Decoder::read_tile_header(&mut Stream) -> Result<TileInfo>
Decoder::decode_tile_data(&mut Stream, tile, Option<&mut [u8]>) -> Result<()>
Decoder::get_decoded_tile(&mut Stream, tile) -> Result<CoreImage>
// encoder
Encoder::setup(&mut EncoderParameters, &mut CoreImage) -> Result<()>
Encoder::extra_options(&[&str]) -> Result<()>             // "TLM=YES", GUARD_BITS, ...
Encoder::start(&mut CoreImage, &mut Stream) -> Result<()>
Encoder::encode(&mut Stream) -> Result<()>
Encoder::finish(&mut Stream) -> Result<()>
Encoder::write_tile(tile, &[u8], &mut Stream) -> Result<()>
```

`DecoderSettings` is the native subset of the C's `opj_dparameters_t` the
core consumes (`layers`, `reduce`, `ignore_pclr_cmap_cdef`, `dump_state`,
`threads`); the `openjp2-capi` crate translates the C struct there and
keeps the single `opj_codec_t*` pointer contract via its merged
`capi::Codec { Dec(core::Decoder), Enc(core::Encoder) }` (wrong-direction
calls reproduce the C's error events through a hidden `emit_error_event`
seam).

Internal parallel hooks (`pub(crate)`, `cfg(feature = "rayon")`):
`num_tiles`, `encode_parallel` (Encoder), `decode_parallel`,
`decode_parallel_area` (Decoder) — used by the safe API, byte-identical to
the serial paths, with conservative fallbacks.

### Codestream structure (index) types

- `CodestreamIndex { main_head_start, main_head_end, codestream_size,
  markers: Vec<MarkerInfo>, tiles: Vec<CodestreamTileIndex> }` with
  `packets()` (every packet of every tile) and `layer_byte_thresholds()`
  (for each quality layer, the smallest file prefix that still contains
  all its packets — "bytes needed to fully decode N layers").
- `CodestreamTileIndex { tileno, nb_tps, current_nb_tps, current_tpsno,
  tp_index: Vec<TpIndex>, markers, nb_packet, packet_index }`.
- `PacketInfo { start_pos, end_ph_pos, end_pos, disto, layno, resno,
  compno, precno }` — the decoder fills it during a decode (the C
  allocates/frees this array but never populates it; the packet identity
  is a Rust-API extension).
- `TpIndex { start_pos, end_header, end_pos }`,
  `MarkerInfo { type_, pos, len }`.
- `CstrInfoV2` / `CstrTccpInfoV2` / `CstrCompInfoV2` (`tx0/ty0/tdx/tdy/tw/th`,
  `nbcomps`, `csty`, `prg`, `numlayers`, `mct`, `tccps`, `numresolutions`, …).

## Supporting public types

- `Stream` — file/reader/writer adapters (`new_file`/`new_input`/
  `new_output`/`from_bytes`, `read`/`write`/`flush`/`tell`/
  `get_bytes_left`/`skip`/`seek`/`has_seek`/`set_stream_length`);
  implements `std::io::Read`/`Write`.
- `CoreImage` / `CoreImageComp` — owned decoded image: `comps:
  Vec<CoreImageComp>` with `data: Vec<i32>`, `prec`, `sgnd`, `dx/dy`,
  `w/h`, `x0/y0`, `resno_decoded`, `factor`, `alpha`; methods
  `tile_create`, `alloc_comps`, `set_rgb`, `take_comps`,
  `take_icc_profile`, `comps_same_dims`, `comps_match`,
  `clone_header_only` (pub(crate)); per-comp `alloc_data`/`set_data`/
  `copy`/`clip`/`scale`/`data_u8`/`data_u16`.
- `ICCProfile { ICC(Vec<u8>), CIELab(Vec<u8>) }` + `ICCProfileRef`/
  `ICCProfileMut` views, `CIE_LAB_BYTE_SIZE = 36`.
- `TileInfo` — tile header info for `read_tile_header`.
- `Poc` — a progression-order change (native replacement for the C's
  `opj_poc_t`).
- `StringWriter` trait (string sink for `dump_codec`).
- `has_thread_support() -> bool`; format-detection consts
  (`JP2_RFC3745_MAGIC`, `JP2_MAGIC`, `J2K_CODESTREAM_MAGIC`).

## Cargo features

`default = ["rayon", "image"]`; `rayon` (parallel tile decode/encode),
`threads` (alias of rayon), `event-handlers` (C-style callbacks),
`image` (image-crate interop), `simd` (`wide`-crate DWT kernels, off by
default), `serde` (Serialize/Deserialize for `ImageData`,
`ImagePixelData`, `DecodeArea`, `DecodeLimits`, `ColorSpace` and
`ProgressionOrder`; forwards to `image/serde`), `openjpeg-version-string`
(byte-identical COM comment to the C reference).

### jpeg2k-compatibility additions

The 0.7.x line also carries the additive API that `jpeg2k` re-exports:
`Image::save_as_file`/`save_as_file_with` (extension-detected format),
`Image::x_offset()`/`y_offset()`/`orig_width()`/`orig_height()` + `Debug`, `CoreImageComp`'s
`width`/`height`/`precision`/`bpp`/`is_alpha`/`is_signed` accessors,
`DecodeArea`'s `Default` + `FromStr` (`"x0:y0:x1:y1"`), `Debug` on the
codestream index/info structs, `ImagePixelData::len()`/`is_empty()`, and
`Decoder::header()` (the header image after `read_header`).

## Sibling crates

| Crate | Contents |
| --- | --- |
| `openjp2-capi` | `#[no_mangle]` C ABI (`libopenjp2` drop-in) + the merged `opj_codec_t*` handle |
| `openjp2-ctypes` | C-ABI mirror types and `openjpeg.h` constants |
| `openjp2-tools` | C-parity `opj_compress` / `opj_decompress` / `opj_dump` |
| `openjp2-cli` | modern `openjp2 compress` / `decompress` / `dump` (the package installs the `openjp2` binary) |
| `openjp2-wasm` | WASM bindings + the GitHub-Pages playground |
