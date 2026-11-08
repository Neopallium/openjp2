# openjp2 Code Outline

Maps the Rust crate onto the JPEG 2000 pipeline and the design spec
(`doc/jpeg2000-design-spec.md`). The crate root is the repository root;
the C ABI layer lives in `capi/`, the CLI tools in `tools/`.

## Crate layers

| Layer | Location | Role |
|---|---|---|
| Safe Rust API | `src/codec.rs`, `src/image.rs`, `src/stream.rs`, `src/types.rs` | The public surface (`Codec`, `CoreImage`, `Stream`, `EncoderParameters`, format detection) |
| Codec drivers | `src/j2k.rs`, `src/jp2.rs` | J2K codestream and JP2 box state machines (marker/box dispatch via procedure lists) |
| Tile processing | `src/tcd.rs` | Tile decode/encode orchestration: tier-2 -> tier-1 -> inverse DWT -> MCT -> DC shift |
| Tier-1 | `src/t1.rs`, `src/mqc.rs`, `src/ht_dec.rs` | Bit-plane arithmetic coding (Part 1) and HTJ2K (T.814) |
| Tier-2 | `src/t2.rs`, `src/pi.rs`, `src/tgt.rs`, `src/bio.rs` | Packet headers, progression iteration, tag trees, bit reader |
| Transforms | `src/dwt.rs`, `src/mct.rs` | 5-3/9-7 lifting DWT, RCT/ICT + Part-2 MCT |
| Math/plumbing | `src/math.rs`, `src/cio.rs`, `src/invert.rs`, `src/sparse_array.rs`, `src/par.rs` | Int math, byte I/O, matrix inversion, sparse arrays, parallel helpers |
| ABI mirrors | `src/c_api_types.rs`, `capi/` | `#[repr(C)]` structs and `#[no_mangle]` entry points matching openjpeg.h |

## Decode pipeline (spec §3)

```
opj_read_header                jp2.rs: opj_jp2_read_header  -> j2k.rs: opj_j2k_read_header
   |  procedure list (function_list.rs: J2kProcList) executes the marker handlers
   |  j2k.rs: opj_j2k_read_header_procedure  (SOC/SIZ/COD/QCD/.../SOT-SOD scan)
   |     - SIZ: geometry, components (spec 2.2)     j2k.rs: opj_j2k_read_siz
   |     - COD/COC: coding style, MCT flag          j2k.rs: opj_j2k_read_cod / _coc
   |     - QCD/QCC: step sizes                     j2k.rs: opj_j2k_read_qcd / _qcc
   |     - TLM: tile-part lengths -> index         j2k.rs: opj_j2k_read_tlm, opj_j2k_build_tp_index_from_tlm
opj_decode
   |  per tile: j2k.rs: opj_j2k_decode_tiles -> opj_j2k_read_tile_header -> opj_j2k_decode_tile
   |     tcd.rs: opj_tcd_decode_tile
   |        t2.rs:   opj_t2_decode_packets        (spec 3.8: packet headers, tag trees)
   |        t1.rs:   opj_t1_decode_cblks          (spec 3.7: bit planes + MQ decode)
   |        dwt.rs:  opj_dwt_decode               (spec 3.3: inverse lifting)
   |        mct.rs:  opj_mct_decode               (spec 3.2: inverse RCT/ICT)
   |        tcd.rs:  opj_tcd_dc_level_shift_decode (spec 3.1)
   |        j2k.rs:  opj_j2k_update_image_data    (tile -> image)
opj_end_decompress
```

Key functions:

- **MQ coder** (`mqc.rs`, spec 3.7.1): `opj_mqc_decode_macro`/`opj_mqc_encode_macro`
  (the DECODE/ENCODE decision), `opj_mqc_codemps_macro`/`_codelps_macro`
  (interval update + probability transitions), `opj_mqc_renormd_macro`/
  `opj_mqc_renorme_macro` (renormalisation), `opj_mqc_bytein_macro`/
  `opj_mqc_byteout` (byte I/O with bit-stuffing).  State table:
  `MQC_STATES` (94 entries, spec's 47 states x 2 MPS variants).
- **Tier-1** (`t1.rs`, spec 3.7): `opj_t1_decode_cblk` drives the three
  passes per bit plane: `opj_t1_dec_sigpass_*`, `opj_t1_dec_refpass_*`,
  `opj_t1_dec_clnpass_*` (with MQ and RAW variants).  The context/flag
  arrays are `flags` (sigma + visited), `data_` (reconstructed values).
  LUTs (`t1_luts.rs`): `LUT_CTXNO_ZC`/`LUT_CTXNO_SC` (context numbers),
  `LUT_SPB` (significance-propagation bits), `LUT_NMSEDEC_*`
  (distortion estimates for rate allocation).
- **HTJ2K** (`ht_dec.rs`, spec §4): MEL/VLC decode of cleanup quad
  strips; `vlc_tbl0/1` tables in `t1_ht_luts.rs`; mixed HT/MQ dispatch in
  `opj_t1_ht_decode_cblk` vs `opj_t1_decode_cblk`.
- **Tier-2** (`t2.rs`, `pi.rs`, `tgt.rs`, `bio.rs`):
  `opj_t2_decode_packets` walks the progression (spec 3.8);
  `PiIterator` (`pi.rs`) computes the packet order for LRCP/RLCP/RPCL/
  PCRL/CPRL; `opj_t2_read_packet_header` decodes inclusion/zero-plane/
  segment info from the tag trees (`tgt.rs`) with `bio.rs` as the
  bit-pump; `opj_t2_read_packet_data` extracts the code-block segments.
- **DWT** (`dwt.rs`, spec 3.3): `dwt_local` (5-3 lifting:
  `decode_1`/`decode_2`, spec's predict/update) and `v8dwt_local` (9-7
  lifting, `DWT_ALPHA/BETA/GAMMA/DELTA/K` constants); `opj_dwt_decode`/
  `opj_dwt_encode` run the per-level horizontal/vertical passes
  (`opj_dwt_fetch_cols_vertical_pass` etc.).
- **Component transform** (`mct.rs`, spec 3.2): `opj_mct_encode`/
  `opj_mct_decode` for RCT/ICT, `MCT_NORMS`/`MCT_NORMS_REAL` for the
  distortion weighting; Part-2 multi-component transforms live in
  `opj_mct_encode_custom` and the `m_mct_records` in `Tcp`.

## Encode pipeline (spec §6)

```
opj_setup_encoder   j2k.rs: opj_j2k_setup_encoder   (validate + derive coding params)
opj_start_compress  j2k.rs: opj_j2k_start_compress -> opj_j2k_write_header procedures
opj_encode          per tile: j2k.rs: opj_j2k_encode_tile -> tcd.rs: opj_tcd_encode_tile
                       tcd.rs: opj_tcd_dc_level_shift_encode
                       tcd.rs: opj_tcd_mct_encode
                       tcd.rs: opj_tcd_dwt_encode          (forward 5-3/9-7)
                       t1.rs:  opj_t1_encode_cblks         (spec 3.7, MQ encode)
                       tcd.rs: opj_tcd_rate_allocate_encode (spec 6.4: layers/truncation)
                       t2.rs:  opj_t2_encode_packets       (spec 3.8, progression + EPH/SOP)
opj_end_compress    j2k.rs: opj_j2k_end_compress -> EOC + TLM/PLT index markers
```

## JP2 layer (`jp2.rs` + `src/j2k/dump.rs`, spec §5)

- Box I/O: `Jp2BoxHeader` (`length`/`ty`/`header_length`,
  `content_length`; XLBox handling, spec 5.2) read by
  `opj_jp2_read_header_procedure`.
- Box handlers: `opj_jp2_read_jp2h` (ihdr/colr/bpcc/pclr/cmap/cdef/res),
  `opj_jp2_read_colr` (spec 5.2: meth 1 enumerated / meth 2 ICC /
  CIELab-restricted), `opj_jp2_read_cmap`/`_pclr`/`_cdef`
  (palette/alpha/channel semantics).
- Colour application: `opj_jp2_apply_pclr` (palette), `opj_jp2_apply_cdef`
  (channel types), `opj_jp2_check_color` (colour space + ICC).
- Writer: `opj_jp2_write_*` box emitters mirror the reader; the encoder
  builds ihdr from the image, colr from the ICC profile or enumerated
  colour space.
- `src/j2k/dump.rs`: `opj_j2k_dump_*` — the header/codestream/tile-index
  dump used by `opj_dump` and `opj_dump_codec` (including the
  `OJP_DO_NOT_DISPLAY_TILE_INDEX_IF_TLM` test knob).

## Shared structures (`src/types.rs`)

- `J2k` / `J2kDec` / `J2kEnc`: the codec state (marker dispatch state
  `J2KState`, decoding parameters, default tile coding parameters, TLM
  info, tile-part correction flags).
- `Cp`/`Tcp`/`Tccp`/`Stepsize`: coding/tile/component coding parameters.
- `Tcd`/`TcdImage`/`TcdTile`/`TcdTileComp`/`TcdResolution`/`TcdBand`/
  `TcdPrecinct`/`TcdCblkDec`/`TcdCblkEnc`: the tile cache (decoded
  coefficients, code-block segments, encoded passes).
- `CodestreamIndex`/`CodestreamTileIndex`: the owned codestream index
  (markers, tile-parts, packets) surfaced through the safe API and
  marshalled to the C mirrors by `capi/src/cstr_info.rs`.
- `TlmInfo`/`TlmTilePartInfo`: TLM marker tile-part entries.

## C ABI (`capi/`, mirrors of `openjpeg/src/lib/openjp2/openjpeg.h`)

- `capi/src/openjpeg.rs`: the `opj_*` entry points. Streams are either
  custom C-callback streams (`CustomStream` forwarding read/write/seek/
  skip to the caller) or (feature `file-io`) file streams; images round-
  trip through `core_from_opj`/`core_into_opj`.
- `capi/src/image.rs`: `opj_image`/`opj_image_comp` mirrors, malloc-backed
  component data, ICC/CIELab buffer marshalling.
- `capi/src/params.rs`: `opj_cparameters_t` mirror with
  `set_defaults`/`set_comment`/`set_MCT`; `capi/src/cstr_info.rs`: the
  codestream-info v1/v2 and index mirrors + builders/destroyers.

## Parallelism (`src/par.rs`)

Rayon (feature `rayon`, default on) parallelises: tier-1 code-block
decode/encode jobs, per-component DWT/DC-shift/MCT, and the tile->image
update. `map_collect`/`for_each`/`zip_map_collect` fall back to
sequential iterators without the feature; small job sets (< 8) stay
sequential even with rayon.
