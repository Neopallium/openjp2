# openjp2 — Current Public API Structure

Written 2026-11-10, against HEAD `136dfd0d`. This is a factual map of the
public surface as it exists today (the *proposal* is in
`doc/redesign-proposal.md`; the earlier `doc/api-review.md` tracks the
jpeg2k-parity work).

The crate has **three API layers**:

1. **Safe high-level API** (`src/api.rs`) — the primary surface: one-shot
   decode/encode, builders, typed errors.
2. **Low-level `Decoder` / `Encoder` API** (`src/codec.rs`, 0.7 split) —
   streaming/tile-level control, C-ABI-shaped; partially deprecated.  The
   merged `Codec` handle moved to the `openjp2-capi` crate (its
   `opj_codec_t*` contract).
3. **C-ABI mirrors** (the `openjp2-ctypes` workspace crate) — the C header's
   types/constants, re-exported under `openjp2::ffi` (the root re-export was
   removed in 0.7).  (The actual `#[no_mangle]` C functions live in the
   separate
   `openjp2-capi` crate.)

## Layer 1 — safe high-level API

### `Error` / `Result`

```rust
pub enum Error { Io(std::io::Error), Decode(String), Encode(String), Unsupported(String) }
pub type Result<T, E = Error> = std::result::Result<T, E>;
```

`Decode` carries the event-manager message (plus "internal decoder panic: …"
when `catch_unwind` converts a panic). All safe-API fallible functions return
`Result`.

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
// encoding
Image::encode(&self, EncodeParameters) -> Result<Vec<u8>>        // J2K codestream
Image::encode_jp2(&self, EncodeParameters) -> Result<Vec<u8>>    // JP2 file
Image::encode_to_writer(&self, EncodeParameters, W: Write+Seek) -> Result<()>
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
  .threads(usize)                 // 0 = automatic pool (default), n > 0 = local n-thread pool
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
  .layer_rate(f32)                // fixed-quality rate allocation (<=100 layers)
  .comment(&str)
  .codeblock_width(u32) / .codeblock_height(u32)   // powers of two
  .low_level() -> &mut EncoderParameters   // escape hatch to all 50 C-named fields
  .into_inner() -> EncoderParameters
```

### Enums

- `ColorSpace { Unknown, Srgb, Gray, Sycc, Eycc, Cmyk }` (+ `From` both ways with `OPJ_COLOR_SPACE`)
- `ProgressionOrder { Lrcp, Rlcp, Rpcl, Pcrl, Cprl }`
- `J2KFormat { JP2, J2K, JPT }` + free fns `detect_format(&[u8])`,
  `detect_format_from_extension`, `detect_format_from_file`

### `DecodeArea` / `DumpImage`

```rust
DecodeArea::new(start_x, start_y, end_x, end_y)
DumpImage::from_bytes / from_bytes_with / from_file / from_file_with
  .decode() -> Result<()>
  .get_codestream_index() -> Option<CodestreamIndex>
  .get_codestream_info() -> Option<CstrInfoV2>
  .dump(info_flag: i32) -> String
```

### `ImageData` / `ImagePixelData` (feature "image")

`ImageData { width, height, format: ExtendedColorType, data: ImagePixelData }`;
`ImagePixelData` is an enum of `L8/La8/Rgb8/Rgba8` (`Vec<u8>`) and
`L16/La16/Rgb16/Rgba16` (`Vec<u16>`).

## Layer 2 — low-level `Decoder` / `Encoder` (0.7: C-free `Result` surface; the merged `Codec` lives in `openjp2-capi`)

```rust
// construction (J2KFormat; Result-returning)
Decoder::new(J2KFormat) -> Result<Self>
Encoder::new(J2KFormat) -> Result<Self>
// events (shared; infallible)
.set_info_handler / .set_warning_handler / .set_error_handler(FnMut(&str)+Send+'static)
// threading (sizes the process-wide rayon pool; first call wins)
.set_threads(i32) -> Result<()>
// dump / header info (shared)
.dump_codec<W: StringWriter>(info_flag, &mut W)
.cstr_info() -> Option<CstrInfoV2>          // the C-ABI opj_get_cstr_info source
.cstr_index() -> Option<CodestreamIndex>    // the C-ABI opj_get_cstr_index source
.get_tile_geometry() -> (tdx, tdy, cblkw, cblkh)
// decoder
Decoder::apply_settings(&DecoderSettings) -> Result<()>   // layers/reduce/
                                                          // ignore_pclr_cmap_cdef/dump_state
Decoder::set_strict(bool) / set_limits(DecodeLimits) / set_components(&[u32]) /
    set_decode_area(&mut CoreImage, Option<DecodeArea>) / set_resolution_factor(u32) -> Result<()>
Decoder::read_header(&mut Stream) -> Result<HeaderImage>  // header-only (empty data)
Decoder::decode(&mut Stream) -> Result<CoreImage>         // lenient mode returns the
                                                          // partial image (documented contract)
Decoder::end_decompress(&mut Stream) -> Result<()>
Decoder::read_tile_header(&mut Stream) -> Result<TileInfo>   // always requests the data size
Decoder::decode_tile_data(&mut Stream, tile, Option<&mut [u8]>) -> Result<()>
Decoder::get_decoded_tile(&mut Stream, tile) -> Result<CoreImage>
// encoder
Encoder::setup(&mut EncoderParameters, &mut CoreImage) -> Result<()>
Encoder::extra_options(&[&str]) -> Result<()>              // "TLM=YES", GUARD_BITS, ...
Encoder::start(&mut CoreImage, &mut Stream) -> Result<()>
Encoder::encode(&mut Stream) -> Result<()>
Encoder::finish(&mut Stream) -> Result<()>
Encoder::write_tile(tile, &[u8], &mut Stream) -> Result<()>
```

`DecoderSettings` is the native subset of the C's `opj_dparameters_t` the
core consumes (`cp_layer`, `cp_reduce`, the two `flags` bits); the
`openjp2-capi` crate translates the C struct there and keeps the single
`opj_codec_t*` pointer contract via its merged
`capi::Codec { Dec(core::Decoder), Enc(core::Encoder) }` (wrong-direction
calls reproduce the C's error events through a hidden `emit_error_event`
seam).  No C types appear in the core's public signatures.

Internal parallel hooks (`pub(crate)`, `cfg(feature = "rayon")`): `num_tiles`,
`encode_parallel` (Encoder), `decode_parallel`, `decode_parallel_area`
(Decoder) — used by the safe API, byte-identical to the serial paths, with
conservative fallbacks.

## Layer 3 — C-ABI mirrors (re-exported at the root and under `ffi`)

- Scalar aliases: `OPJ_BOOL`(=i32), `OPJ_UINT32`, `OPJ_INT32`, `OPJ_SIZE_T`, …
- Enums: `COLOR_SPACE`/`OPJ_COLOR_SPACE`, `CODEC_FORMAT`/`OPJ_CODEC_FORMAT`,
  `OPJ_PROG_ORDER` constants (`OPJ_LRCP`…), `OPJ_RSIZ_CAPABILITIES`,
  `OPJ_CINEMA_MODE`.
- Structs: `opj_dparameters_t`, `opj_image_cmptparm_t`, `opj_poc_t`,
  `opj_packet_info_t`, `opj_marker_info_t`, `opj_tp_index_t`; function-pointer
  typedefs for the C stream callbacks.
- Constants modules: `consts::cio`, `consts::event`, `consts::j2k`, `consts::opj`
  (marker values, `OPJ_J2K_MAXRLVLS`, `OPJ_J2K_STREAM_CHUNK_SIZE`, …).

## Supporting public types

- `Stream` — `Read+Write+Seek` adapter (`new_file`/`new_input`/`new_output`,
  `read`/`write`/`flush`/`tell`/`get_bytes_left`/`skip`/`seek`/`has_seek`,
  `set_stream_length`); implements `std::io::Read`/`Write`.
- `CoreImage` / `CoreImageComp` — owned decoded image: `comps: Vec<CoreImageComp>`
  with `data: Vec<i32>`, `prec`, `sgnd`, `dx/dy`, `w/h`, `x0/y0`, `resno_decoded`,
  `factor`, `alpha`; methods `tile_create`, `alloc_comps`, `set_rgb`,
  `take_comps`, `take_icc_profile`, `comps_same_dims`, `comps_match`,
  `clone_header_only` (pub(crate)); per-comp `alloc_data`/`set_data`/`copy`/
  `clip`/`scale`/`data_u8`/`data_u16`.
- `ICCProfile { ICC(Vec<u8>), CIELab(Vec<u8>) }` + `ICCProfileRef`/`ICCProfileMut`
  views, `CIE_LAB_BYTE_SIZE = 36`.
- `TileInfo` — tile header info for `read_tile_header`.
- `CodestreamIndex` / `CodestreamTileIndex` (`tileno`, `nb_tile_parts`,
  `tile_indices()`, …) and `CstrInfoV2` / `CstrTccpInfoV2` / `CstrCompInfoV2`
  (`tx0/ty0/tdx/tdy/tw/th`, `nbcomps`, `csty`, `prg`, `numlayers`, `mct`,
  `tccp_infos`, `default_tile_info`, `tile_info`).
- `StringWriter` trait (string sink for `dump_codec`).
- `has_thread_support() -> OPJ_BOOL`; format-detection consts
  (`JP2_RFC3745_MAGIC`, `JP2_MAGIC`, `J2K_CODESTREAM_MAGIC`).

## Cargo features

`default = ["rayon", "image"]`; `rayon`, `threads` (alias of rayon), `image`
(image-crate interop), `simd` (`wide`-crate DWT kernels),
`openjpeg-version-string` (byte-identical COM comment to the C reference).
The `openjp2-capi` crate is a separate workspace and depends on the core with
no features.
