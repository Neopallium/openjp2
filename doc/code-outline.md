# openjp2 Code Outline

Maps the Rust crate onto the JPEG 2000 pipeline and the design spec
(`doc/jpeg2000-design-spec.md`). The crate root is the repository root;
the C ABI layer lives in `capi/` (with the C types in `ctypes/`), the
C-parity CLI tools in `tools/`, the modern CLI in `cli/`, and the WASM
bindings + playground in `wasm/`.

## Crate layers

| Layer | Location | Role |
|---|---|---|
| Safe Rust API | `src/api/` (public `api` module, re-exported at the root), `src/codec.rs`, `src/image.rs`, `src/stream.rs`, `src/types.rs` | The public surface (`Image`, `DecodeParameters`, `EncodeParameters`, `Decoder`, `Encoder`, `CoreImage`, `Stream`, typed errors, format detection) |
| Codec drivers | `src/j2k.rs`, `src/jp2.rs` | J2K codestream and JP2 box state machines (match-based marker/box dispatch) |
| Tile processing | `src/tcd.rs` | Tile decode/encode orchestration: tier-2 -> tier-1 -> inverse DWT -> MCT -> DC shift |
| Tier-1 | `src/t1.rs`, `src/mqc.rs`, `src/ht_dec.rs` | Bit-plane arithmetic coding (Part 1) and HTJ2K (T.814) |
| Tier-2 | `src/t2.rs`, `src/pi.rs`, `src/tgt.rs`, `src/bio.rs` | Packet headers, progression iteration, tag trees, bit reader |
| Transforms | `src/dwt.rs`, `src/mct.rs` | 5-3/9-7 lifting DWT, RCT/ICT + Part-2 MCT |
| Math/plumbing | `src/math.rs`, `src/cio.rs`, `src/invert.rs`, `src/sparse_array.rs`, `src/par.rs` | Int math, byte I/O, matrix inversion, sparse arrays, parallel helpers |
| C ABI | `ctypes/`, `capi/` | `#[repr(C)]` structs and `openjpeg.h` constants (`openjp2-ctypes`) + `#[no_mangle]` entry points (`openjp2-capi`) |

## Decode pipeline (spec §3)

```
Decoder::read_header          jp2.rs: jp2_read_header  -> j2k.rs: j2k_read_header
   |  j2k.rs: j2k_read_header_procedure  (SOC/SIZ/COD/QCD/.../SOT-SOD scan)
   |     - SIZ: geometry, components (spec 2.2)     j2k.rs: j2k_read_siz
   |     - COD/COC: coding style, MCT flag          j2k.rs: j2k_read_cod / _coc
   |     - QCD/QCC: step sizes                     j2k.rs: j2k_read_qcd / _qcc
   |     - TLM: tile-part lengths -> index         j2k.rs: j2k_read_tlm, j2k_build_tp_index_from_tlm
Decoder::decode
   |  per tile: j2k.rs: j2k_decode_tiles -> j2k_read_tile_header -> j2k_decode_tile
   |     tcd.rs: tcd_decode_tile
   |        t2.rs:   t2_decode_packets        (spec 3.8: packet headers, tag trees)
   |        t1.rs:   t1_decode_cblks          (spec 3.7: bit planes + MQ decode)
   |        dwt.rs:  dwt_decode               (spec 3.3: inverse lifting)
   |        mct.rs:  mct_decode               (spec 3.2: inverse RCT/ICT)
   |        tcd.rs:  tcd_dc_level_shift_decode (spec 3.1)
   |        j2k.rs:  j2k_update_image_data    (tile -> image)
Decoder::end_decompress
```

Key functions:

- **MQ coder** (`mqc.rs`, spec 3.7.1): `mqc_decode_macro`/`mqc_encode_macro`
  (the DECODE/ENCODE decision), `mqc_codemps_macro`/`_codelps_macro`
  (interval update + probability transitions), `mqc_renormd_macro`/
  `mqc_renorme_macro` (renormalisation), `mqc_bytein_macro`/
  `mqc_byteout` (byte I/O with bit-stuffing).  State table:
  `MQC_STATES` (94 entries, spec's 47 states x 2 MPS variants).
- **Tier-1** (`t1.rs`, spec 3.7): `t1_decode_cblk` drives the three
  passes per bit plane: `t1_dec_sigpass_*`, `t1_dec_refpass_*`,
  `t1_dec_clnpass_*` (with MQ and RAW variants).  The context/flag
  arrays are `flags` (sigma + visited), `data_` (reconstructed values).
  LUTs (`t1_luts.rs`): `LUT_CTXNO_ZC`/`LUT_CTXNO_SC` (context numbers),
  `LUT_SPB` (significance-propagation bits), `LUT_NMSEDEC_*`
  (distortion estimates for rate allocation).
- **HTJ2K** (`ht_dec.rs`, spec §4): MEL/VLC decode of cleanup quad
  strips; `vlc_tbl0/1` tables in `t1_ht_luts.rs`; mixed HT/MQ dispatch in
  `t1_ht_decode_cblk` vs `t1_decode_cblk`.
- **Tier-2** (`t2.rs`, `pi.rs`, `tgt.rs`, `bio.rs`):
  `t2_decode_packets` walks the progression (spec 3.8);
  `PiIterator` (`pi.rs`) computes the packet order for LRCP/RLCP/RPCL/
  PCRL/CPRL; `t2_read_packet_header` decodes inclusion/zero-plane/
  segment info from the tag trees (`tgt.rs`) with `bio.rs` as the
  bit-pump; `t2_read_packet_data` extracts the code-block segments.
- **DWT** (`dwt.rs`, spec 3.3): `dwt_local` (5-3 lifting:
  `decode_1`/`decode_2`, spec's predict/update) and `v8dwt_local` (9-7
  lifting, `DWT_ALPHA/BETA/GAMMA/DELTA/K` constants); `dwt_decode`/
  `dwt_encode` run the per-level horizontal/vertical passes
  (`dwt_fetch_cols_vertical_pass` etc.).
- **Component transform** (`mct.rs`, spec 3.2): `mct_encode`/
  `mct_decode` for RCT/ICT, `MCT_NORMS`/`MCT_NORMS_REAL` for the
  distortion weighting; Part-2 multi-component transforms live in
  `mct_encode_custom` and the `m_mct_records` in `Tcp`.

## Encode pipeline (spec §6)

```
Encoder::setup      j2k.rs: j2k_setup_encoder   (validate + derive coding params)
Encoder::start      j2k.rs: start_compress -> j2k_write_header procedures
Encoder::encode     per tile: j2k.rs: j2k_encode_tile -> tcd.rs: tcd_encode_tile
                       tcd.rs: tcd_dc_level_shift_encode
                       tcd.rs: tcd_mct_encode
                       tcd.rs: tcd_dwt_encode          (forward 5-3/9-7)
                       t1.rs:  t1_encode_cblks         (spec 3.7, MQ encode)
                       tcd.rs: tcd_rate_allocate_encode (spec 6.4: layers/truncation)
                       t2.rs:  t2_encode_packets       (spec 3.8, progression + EPH/SOP)
Encoder::finish     j2k.rs: j2k_end_compress -> EOC + TLM/PLT index markers
```

## JP2 layer (`jp2.rs` + `src/j2k/dump.rs`, spec §5)

- Box I/O: `Jp2BoxHeader` (`length`/`ty`/`header_length`,
  `content_length`; XLBox handling, spec 5.2) read by
  `jp2_read_header_procedure`.
- Box handlers: `jp2_read_jp2h` (ihdr/colr/bpcc/pclr/cmap/cdef/res),
  `jp2_read_colr` (spec 5.2: meth 1 enumerated / meth 2 ICC /
  CIELab-restricted), `jp2_read_cmap`/`_pclr`/`_cdef`
  (palette/alpha/channel semantics).
- Colour application: `jp2_apply_pclr` (palette), `jp2_apply_cdef`
  (channel types), `jp2_check_color` (colour space + ICC).
- Writer: `jp2_write_*` box emitters mirror the reader; the encoder
  builds ihdr from the image, colr from the ICC profile or enumerated
  colour space.
- `src/j2k/dump.rs`: `j2k_dump_*` — the header/codestream/tile-index
  dump used by `opj_dump` and `dump_codec` (including the
  `J2K_DO_NOT_DISPLAY_TILE_INDEX_IF_TLM` test knob).

## Shared structures (`src/types.rs`)

- `J2k` / `J2kDec` / `J2kEnc`: the codec state (marker dispatch state
  `J2KState`, decoding parameters, default tile coding parameters, TLM
  info, tile-part correction flags).
- `Cp`/`Tcp`/`Tccp`/`Stepsize`: coding/tile/component coding parameters.
- `Tcd`/`TcdImage`/`TcdTile`/`TcdTileComp`/`TcdResolution`/`TcdBand`/
  `TcdPrecinct`/`TcdCblkDec`/`TcdCblkEnc`: the tile cache (decoded
  coefficients, code-block segments, encoded passes).
- `CodestreamIndex`/`CodestreamTileIndex`/`PacketInfo`/`TpIndex`/
  `MarkerInfo`: the owned codestream index (markers, tile-parts, packet
  positions + layer/resolution/component/precinct identity) surfaced
  through the safe API.  The C mirror (`capi/src/cstr_info.rs`) exposes
  the positions only, like upstream.
- `TlmInfo`/`TlmTilePartInfo`: TLM marker tile-part entries.

## C ABI (`capi/`, mirrors of `openjpeg/src/lib/openjp2/openjpeg.h`)

- `ctypes/`: the `#[repr(C)]` mirrors and `openjpeg.h` constants the
  C ABI needs (the core itself is C-name-free).
- `capi/src/openjpeg.rs`: the `opj_*` entry points. Streams are either
  custom C-callback streams (`CustomStream` forwarding read/write/seek/
  skip to the caller) or (feature `file-io`) file streams; images round-
  trip through `core_from_opj`/`core_into_opj`.
- `capi/src/image.rs`: `opj_image`/`opj_image_comp` mirrors, malloc-backed
  component data, ICC/CIELab buffer marshalling.
- `capi/src/params.rs`: `opj_cparameters_t` mirror with
  `set_defaults`/`set_comment`/`set_MCT`; `capi/src/cstr_info.rs`: the
  codestream-info v1/v2 and index mirrors + builders/destroyers.

The encoder mirrors this: `j2k_encode_tile_worker` +
`j2k_encode_tiles_parallel` (in `src/j2k.rs`, wired through
`Encoder::encode_parallel` from `encode_image` in `src/api/encode.rs`) encode
each tile into a per-worker memory buffer via the existing
pre_write_tile / tile-data extraction / post_write_tile sequence and
write the tile-parts sequentially (TLM state updated in order), so
the codestream is byte-identical to the serial encode.

## Parallelism (`src/par.rs` + the tile decode driver)

Additionally, the safe one-shot API decodes multi-tile codestreams with
tiles in parallel (`j2k_decode_tiles_parallel` in `src/j2k.rs`, wired
through `Decoder::decode_parallel` in `src/codec.rs` from `decode_stream`
in `src/api/decode.rs`): the codestream is snapshotted into an `Arc<Vec<u8>>`,
each tile's first SOT is located (TLM-built index when valid, else an
FF90/Isot scan - bit-stuffing guarantees SOT markers never appear in
coded data), and every tile is decoded by `j2k_decode_tile_worker`
(a fresh decoder over an independent cursor at the tile's first SOT).
The per-tile caches are merged in order with
`j2k_update_image_data` (`j2k_merge_tile_results`), and JP2 files
re-run `jp2_apply_color_postprocessing` (palette/cdef) afterwards,
exactly like the serial `jp2_decode`.  The parallel path only runs
where it is byte-identical to the serial one by construction:
whole-image full-quality decodes, tile-aligned decode areas, and
multi-tile encodes.  Everything else - partial windows, layer limits,
resolution reduction, and "tolerant" codestreams whose packet segments
overrun their packets (detected via the t2 lenient recovery) - returns
None and the caller falls back to the serial decode, which reproduces
the C reference's cross-tile buffer-reuse behaviour.

## Parallelism (`src/par.rs`)

Rayon (feature `rayon`, default on) parallelises: tier-1 code-block
decode/encode jobs, per-component DWT/DC-shift/MCT, and the tile->image
update. `map_collect`/`for_each`/`zip_map_collect` fall back to
sequential iterators without the feature; small job sets (< 8) stay
sequential even with rayon.
