# openjp2-wasm

WASM bindings for the pure-Rust OpenJPEG port. This crate wraps the safe
Rust API of [`openjp2`](https://crates.io/crates/openjp2) — **not** the
legacy C ABI — so a browser can decode JPEG 2000 (`JP2`/`J2K`) files
without native code.

## Build

```sh
# one-time: add the wasm target
rustup target add wasm32-unknown-unknown

wasm-pack build wasm --target web --release
# or, without wasm-pack:
cargo build --target wasm32-unknown-unknown --release -p openjp2-wasm
wasm-bindgen target/wasm32-unknown-unknown/release/openjp2_wasm.wasm \
  --out-dir wasm/pkg --target web
```

## API

| Function | Description |
| --- | --- |
| `init()` | Installs the console panic hook (call once). |
| `version()` | The crate version string. |
| `decode_info(bytes)` | Header metadata: `{ width, height, components, prec, sgnd, numresolutions, color_space, has_icc_profile, num_layers, progression, mct, tiles_h, tiles_v, tile_w, tile_h, codeblock_w, codeblock_h, dx, dy }`. |
| `decode_to_rgba(bytes, strict?)` | Decodes to tightly packed RGBA8 (`w * h * 4` bytes). |
| `decode_probe(bytes, strict?)` | Lenient decode returning `{ width, height, complete, components, color_space, pixels }` — never fails on a truncated codestream with a readable header (the truncation-slider backend). |
| `decode_probe_opt(bytes, opts)` | Like `decode_probe`, with a `DecodeOptions` (strict, `reduce` resolution factor, `layers`, decode area, ignore-pclr). |
| `encode_rgba_to_j2k(w, h, rgba, lossless)` | Encodes RGBA8 to a J2K codestream. |
| `encode_rgba_to_jp2(w, h, rgba, lossless)` | Encodes RGBA8 to a JP2 file. |
| `encode_rgba_opt(w, h, rgba, opts)` | Encodes with an `EncodeOptions` (lossless, resolutions, layers, explicit per-layer rates, progression order, tile size, ratio, comment, container). |
| `init_thread_pool(n)` / `has_threads()` / `thread_count()` | Threaded build only: initialise and query the rayon pool (`threads` feature). |
| `codestream_structure(bytes, strict?)` | `{ thresholds, packets }` — the per-layer byte prefixes plus the packet index (six `u32`s per packet: start, end, layer, resolution, component, precinct). |
| `layer_byte_thresholds(bytes, strict?)` | Per quality layer, the smallest file prefix that still fully decodes that many layers (from the decoder's packet index). |

Pixels are always RGBA8: 1-component images become grayscale, 3-component
RGB; samples outside 8 bits are shifted/clamped. Subsampled components are
rejected (use the full `openjp2` API for those). ICC colour management is
not applied.

## WebAssembly threads (rayon)

The crate has a `threads` feature that enables the core's rayon
parallelism (parallel tile decode/encode) via
[`wasm-bindgen-rayon`](https://crates.io/crates/wasm-bindgen-rayon):

```sh
rustup toolchain install nightly
rustup +nightly component add rust-src
rustup +nightly target add wasm32-unknown-unknown
wasm/playground/build.sh --threads
```

The threaded build lands in `playground/pkg-mt/` and is loaded only when
the page is **cross-origin isolated** (`SharedArrayBuffer` available —
COOP/COEP headers); otherwise the single-threaded `pkg/` build is used.
`build.sh serve` sends those headers for local testing.  GitHub Pages
cannot set response headers, so the deployed playground stays
single-threaded (its "threads" setting explains this); the threaded build
and toggle are for local/custom deployments.

## Playground

`playground/` is the GitHub-Pages site (also served locally by
`playground/build.sh serve`):

- **Decode** — drop a JP2/J2K file to preview it on a canvas (multi-tile
  codestreams decode in parallel); settings: strict mode, quality layers,
  **resolution switching** (decode any of the codestream's DWT pyramid
  levels with one click) and — under *advanced* — a decode-area window
  and the ignore-pclr switch.
- **Encode** — drop any browser-readable image to encode it to JP2/J2K;
  settings: lossless/lossy transform, container, resolution count,
  quality layers, progression order, and — under *advanced* — tile size,
  per-layer rates, compression ratio and a COM comment.  Every setting
  has a tooltip and a `?` badge with a longer explanation.  The panel
  shows the *effective* configuration (including the automatic layer
  ladder when no rates are given), warns about conflicting settings
  (e.g. a `layers` count that a rates list overrides), and after
  encoding reports what the produced file's header actually declares.
- **Explorer** — a structural tree of the file (JP2 boxes → codestream
  markers → tile-parts; the COD node lists the progression order, layer
  and resolution counts, code-block size and transform, and each
  tile-part expands into its packets grouped layer → resolution →
  component with byte ranges) linked
  bidirectionally with the hex view (click a tree node to highlight its
  bytes, click a byte to highlight its tree node), a byte-level hex view
  where **every byte** is colour-coded and labelled with the part of the
  stream it belongs to (click one for its meaning and a link into the
  JPEG 2000 spec; unrecognized bytes are marked corrupt), a per-layer
  byte-size table (how large a truncated file must be to fully decode N
  quality layers; clicking a row sets the truncation point), buttons
  that truncate at each level of the progression order's outermost
  dimension (layer for LRCP, resolution for RLCP/RPCL, precinct for
  PCRL, component for CPRL), and a
  truncation control — a 0.01%-step slider, a byte-length field,
  ±1/10/100/1k byte buttons and size presets (500 B … 1 MiB) — that cuts
  the codestream and re-decodes it live into the preview (debounced, with
  in-place hex/tree updates), demonstrating the lenient decoder's
  tolerance for truncated files.

All decode/encode work runs in a Web Worker (`worker.js`) so the page
never freezes during a decode; where module workers are unavailable the
page falls back to running the wasm on the main thread.

The site is published by `.github/workflows/pages.yml` on every push that
touches `wasm/**` or `src/**` (requires Pages → "GitHub Actions" in the
repository settings).

## Example

`examples/index.html` is a self-contained demo: drop a JP2/J2K file to
decode and preview it on a canvas, or drop any browser-readable image to
encode it to JP2.

```js
import init, { decode_to_rgba, decode_info } from "./pkg/openjp2_wasm.js";
await init();

const bytes = new Uint8Array(await file.arrayBuffer());
const info = decode_info(bytes);
const rgba = decode_to_rgba(bytes);
const img = new ImageData(
  new Uint8ClampedArray(rgba.buffer, rgba.byteOffset, rgba.length),
  info.width,
  info.height,
);
ctx.putImageData(img, 0, 0);
```

## License

BSD-2-Clause, like the rest of the project.
