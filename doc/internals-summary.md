# openjp2 — Internal Structure Summary

Updated 2026-11-13, against 0.7.1. Complements `doc/code-outline.md`
(a module↔spec map) with a data-flow view of the two pipelines. The code is a
faithful, `#![forbid(unsafe_code)]` port of OpenJPEG 2.5.x; module names map
1:1 to the C files (`src/lib/openjp2/*.c`).

## 1. Shared building blocks (both pipelines)

| Module | Role |
|---|---|
| `types.rs` | Central data model: `Cp`/`Tcp`/`Tccp` (coding parameters), `Tcd`/`TcdTile`/`TcdTileComp`/`TcdResolution`/`TcdBand`/`TcdPrecinct` (tile geometry), `CodeBlocks::Enc(Vec<TcdCblkEnc>)`/`Dec(Vec<TcdCblkDec>)` (per-precinct codeblocks), `TileBuf` (owned `Vec<i32>` sample buffer), `J2k`/`Jp2` codec state, `EncoderParameters`/`DecoderParameters`. |
| `stream.rs` | `Stream`: `BufReader`/`BufWriter` over `Box<dyn ReadSeek/WriteSeek>`, tracked byte offset, `get_bytes_left` vs the declared length, seek. The C-callback adapter lives in the capi crate. |
| `cio.rs` | Big-endian fixed-width reads/writes (`read_bytes`, `write_bytes`, float/double). |
| `bio.rs` | Bit-level I/O (used by tier-2 packet headers and tag trees). |
| `mqc.rs` | MQ arithmetic coder, enc + dec, MQ + raw/bypass modes. Owns `Vec<Cell<u8>>` (payload buffer; `buf[0]` is the encode sentinel, re-zeroed in `init_enc`); 19 context states padded to a 256-entry value array for branch-free indexing. |
| `tgt.rs` | Tag trees (encode/decode) for inclusion/msb packets. |
| `t1_luts.rs`, `t1_ht_luts.rs` | Generated lookup tables (contexts, NMSEDEC, ZC/SC/SPB, HT VLC). |
| `math.rs` | Wrapping/ceildiv/clamp helpers (the C's overflow semantics). |
| `event.rs` | `EventMgr`: `Mutex`-wrapped `FnMut(&str)` sinks per level (Sync for rayon workers). |
| `par.rs` | Rayon helpers (`map_collect`, `for_each`, `for_each_chunks_mut`, `for_each_range`, `configure_pool`); sequential fallbacks when the feature is off. |
| `sparse_array.rs` | Sparse sample array (HTJ2K blocks): `Vec<Option<Vec<i32>>>` blocks, region read/write with strides. |
| `mct.rs` | Multi-component transforms: reversible (i32), irreversible (f32 bit-pattern in i32), custom matrices (13-bit fixed point; `invert.rs` provides the LUP inverse). |
| `invert.rs` | LUP matrix inversion (used only for custom-MCT encode). |
| `dwt.rs` | 5-3 and 9-7 forward/inverse DWT, whole-tile and partial; 8-element-interleaved ("V8") row layout; 9-7 uses `f32` scratch (the C reinterprets the i32 buffer in place — safe Rust converts). |
| `t1.rs` | Tier-1: significance/refinement/cleanup passes, enc+dec, MQ+raw. Workspace is a `thread_local!` `T1` (flags + data + Mqc) reused across codeblocks. |
| `ht_dec.rs` | HTJ2K (T.814) block decoder (MEL/VLC/mag-sign, `sparse_array` output). |
| `t2.rs` | Tier-2: packet header codec (bio/tag-trees), packet assembly/parsing; fills `TcdCblkDec.segs/chunks` on decode, writes packet data on encode. |
| `pi.rs` | Packet iterator for the 5 progression orders (LRCP/RLCP/RPCL/PCRL/CPRL); drives both t2 directions. |
| `tcd.rs` | Tile coder: geometry construction (resolutions/bands/precincts/codeblocks), orchestration of dwt/mct/dc-shift/t1/t2, rate allocation (encode), tile↔image data transfer (`update_tile_data`, `copy_tile_data`). |
| `j2k.rs` | Codestream state machine: every marker read/write handler, main/tile-part header loops, tile decode loop, parallel tile decode/encode drivers, `setup_encoder/decoder`. |
| `jp2.rs` | JP2 box reader/writer (jp/ ftyp/ jp2h/ ihdr/ colr/ bpcc/ pclr/ cdef/ cmap/ jp2c), colour post-processing (`check_color`, `apply_pclr`, `apply_cdef`). |
| `codec.rs` | `Decoder`/`Encoder` over the `CodecFormat` enum (`J2k`/`Jp2`), each with an `EventMgr`; shared surface (handlers, `set_threads`, dump, cstr accessors, `get_tile_geometry`) via an internal macro + parallel dispatch hooks. The merged `Codec` handle lives in `openjp2-capi` (`capi/src/codec.rs`). |
| `api/` | The public `api` module (re-exported at the root): `Image`, the parameter builders, typed errors, the dump driver, image-crate interop, and `catch_unwind` guards around the decode/encode drivers. |
| `consts.rs` | The C-mirror marker/flag constants under native names (`J2K_MS_*`, `J2K_MAXRLVLS`, …). |
| `fprintf.rs` | `StringWriter`: the string sink used by the dump text. |
| `openjpeg.rs` | Crate-private re-export hub for the codec modules (`Stream`, `CoreImage`, …). |

Key structural decisions (recorded so they are not rediscovered):
- Codeblock payloads live in `tcp.m_data` (the tile-part buffer); each
  `TcdSegDataChunk` stores a `data_offset` + `len` into it.
- t1 decode **always copies** chunks into the thread-local `Mqc.buf` (the
  C's zero-copy single-chunk path was dropped); t1 encode copies the Mqc
  payload into `TcdCblkEnc.data` after the passes.
- `TileBuf` is `Vec<i32>`; the 9-7 paths convert through `f32::from_bits`/
  `to_bits` (whole-tile scratch lives once in `Tcd.dwt_f32_scratch`).
- Encode/decode codeblock arrays are grow-only (`CodeBlocks::grow_enc/dec`)
  to mirror the C's realloc reuse.

## 2. Decoder pipeline

```
Image::from_bytes / DumpImage
 └─ decode_stream (catch_unwind)  [api/]
     ├─ Decoder::new + setup_decoder + set_decode_limits + strict
     ├─ read_header
     │   ├─ J2K: marker loop (j2k.rs read_header_procedure)
     │   │     SOC → SIZ/COD/QCD/… (dispatch table J2KMarker::handler) → SOT break
     │   │     • SIZ: CoreImage comps (prec/dx/dy validated 1..=31/1..=255),
     │   │       Cp tiles + per-tile tccps, DecodeLimits checks (explicit mem cap,
     │   │       always-on tccps/Tcp accounting), codestream-index setup
     │   │     • COD/QCD/COC/QCC: tcp/tccp params; POC/PLT/PPM/TLM/COM/MCT/…
     │   └─ JP2: box loop (jp2.rs read_header_procedure) → jp2h boxes
     │         (ihdr/colr/pclr/cdef/cmap) → jp2c → the J2K header; metadata
     │         length caps; then j2k read
     ├─ (rayon) Decoder::decode_parallel / decode_parallel_area [codec.rs]
     │     per-tile fresh codec states over an Arc<Vec<u8>> snapshot;
     │     SOT pre-scan (TLM index or FF90 scan); workers run the serial
     │     tile pipeline below; master merges via update_image_data and
     │     re-applies JP2 colour post-processing.  Conservative guards:
     │     whole-image/area-covering tiles only, layer/reduce cases fall
     │     back to serial (cross-tile buffer-reuse parity).
     └─ serial decode (j2k.rs: j2k_decode)
           └─ decode_tiles loop (procedure list)
                ├─ read_tile_header: tile-part loop → tcp.m_data (Psot-clamped
                │     read; try_reserve-guarded allocation), SOT/COD/COC/QCD/…
                ├─ j2k_decode_tile
                │    ├─ tcd_init_decode_tile: geometry (resolutions/bands/
                │    │     precincts/codeblocks, incl. the issue726 stored-
                │    │     geometry fix), tile buffer sizing (try_reserve)
                │    ├─ t2 decode (t2.rs + pi.rs): packet headers via bio +
                │    │     tag trees → per-codeblock segs/chunks (data_offset
                │    │     into tcp.m_data); sets resno_decoded
                │    ├─ t1 decode (t1.rs): per codeblock, MQ/raw decode →
                │    │     coefficients into tilec.data (whole tile) or
                │    │     decoded_data (window); parallel over codeblocks
                │    ├─ dwt decode (dwt.rs): inverse 5-3/9-7, whole-tile or
                │    │     partial-window paths
                │    ├─ mct decode (mct.rs; only when tcp.mct != 0)
                │    └─ dc level shift decode (tcd.rs)
                ├─ update_image_data (j2k.rs): tile buffers → output image
                │     comps (resno_decoded, win/full geometry, memory cap)
                └─ lenient handling: "Stream too short" returns strict?0:1;
                      read_tile_header failure breaks the loop in lenient mode
           └─ move_data_from_codec_to_output_image (tcd tile data → p_image)
```

Lenient (progressive-download) behavior: a truncated main header after SIZ
yields `m_header_partial` (dims known, decode skipped); truncated tile data
returns the partially decoded image instead of an error; strict mode keeps the
hard failures.

## 3. Encoder pipeline

```
Image::encode / encode_jp2 / encode_to_writer
 └─ encode_image* (catch_unwind) [api/]
     ├─ Encoder::new + setup_encoder (j2k.rs)
     │     • parameter validation (layers 0..=100, numpocs ≤ 32,
     │       res_spec×numresolution shift guard, prog order, prec 1..=31,
     │       cblk sizes, tile grid, mct_data lengths)
     │     • Cp/Tcp/Tccp construction; MCT: standard norms or custom matrix
     │       (invert.rs LUP) + dc shifts; IMF/cinema profile forcing;
     │       TLM setup; comment (COM)
     ├─ start_compress (j2k.rs procedure list: init_info → SOC → SIZ → COD →
     │     QCD → COC× → QCC× → [TLM] → [POC] → regions → [COM] →
     │     [MCT group, rsiz-gated] → create_tcd → update_rates)
     │     (JP2 wrapper writes signature + ftyp + jp2h boxes first)
     ├─ encode
     │   ├─ (rayon) j2k_encode_tiles_parallel: per-tile fresh J2k states
     │   │     (cp/image cloned; workers read the shared image) write their
     │   │     tile-parts to per-worker buffers; master writes them in order
     │   │     and updates TLM.  Falls back to serial when TLM is on and
     │   │     tiles are multi-part (per-tile TLM recording is only correct
     │   │     for single-part tiles).
     │   └─ serial (write_first_tile_part / write_all_tile_parts), per tile:
     │         ├─ tcd init (encode geometry) + copy_tile_data (input samples
     │         │     → tilec.data, precision-aware)
     │         ├─ dc level shift encode → mct encode → dwt encode (forward)
     │         ├─ t1 encode: sigpass → refpass → clnpass (MQ or raw),
     │         │     parallel over codeblocks, per-pass rate/distortion
     │         ├─ rate allocation (tcd.rs): rate/distortion ratio or fixed
     │         │     quality/distortion thresholds → per-layer pass counts
     │         │     (`makelayer`, `makelayer_fixed`)
     │         └─ t2 encode: packet headers (bio + tag trees) + data →
     │               per-codeblock layers/data; tile-part splitting
     │               (tp_flag R/C, POC)
     └─ end_compress: EOC, TLM finalisation (write_updated_tlm);
           (JP2 wrapper writes the jp2c box length by seeking back)
```

Parallel-vs-serial byte identity is guaranteed by construction: workers are
pure per-tile functions of `(cp, image, tile_no)`, and the master performs
all ordering-sensitive work (markers, TLM) sequentially.
