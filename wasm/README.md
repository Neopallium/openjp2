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

The build uses the flags from the wasm-bindgen-rayon README
(`--shared-memory --import-memory` and the `__wasm_init_tls`/`__tls_*`
exports) and enables its `no-bundler` feature, since the playground is
served directly without a bundler.

The threaded build lands in `playground/pkg-mt/` and is loaded only when
the page is **cross-origin isolated** (`SharedArrayBuffer` available —
COOP/COEP headers); otherwise the single-threaded `pkg/` build is used.
`build.sh serve` sends those headers for local testing.  On GitHub Pages
(which cannot set response headers) the page includes
[coi-serviceworker](https://github.com/gzuidhof/coi-serviceworker), a
service worker that injects the headers and reloads the page once into an
isolated state, so the "threads" setting works there too (where the
browser supports it; HTTPS or localhost is required).  The vendored copy
additionally fetches with `cache: "no-store"` (and registers with
`updateViaCache: "none"`), so a new deployment is picked up immediately
instead of being masked by GitHub Pages' 10-minute HTTP cache; the very
first visit (before the worker is active) may still need one reload.

## Playground

Try it in the browser: **<https://neopallium.github.io/openjp2/>** — the
playground runs the pure-Rust decoder/encoder entirely client-side via
WebAssembly:

- **Decode** — drop a `.jp2` / `.j2k` file to preview it (multi-tile
  codestreams decode in parallel), zoom to 1:1 pixels, switch between the
  codestream's resolutions, and cap quality layers or a decode area.
- **Encode** — drop a PNG/JPEG/WebP/... image and download JP2/J2K; tune
  lossless/lossy, resolutions, quality layers (with a rate ladder),
  progression order, tiles, ratio and comment.  The panel shows the
  *effective* settings, warns about conflicts, and reports what the
  produced file's header actually declares.
- **Explorer** — every byte of the codestream labelled (boxes, markers,
  tile-parts, packets), a structure tree whose `COD` node lists the
  progression order / layer / resolution structure and whose tile-parts
  expand into individual packets, per-layer byte-size thresholds, and a
  truncation slider (0.01% steps plus +/-1/10/100/1k byte buttons) that
  re-decodes live to show how much of the image survives.

No uploads: decoding, encoding and inspection all happen in the browser.
Where the page can be cross-origin isolated, the optional threaded build
(WebAssembly rayon threads) can be enabled from the Decode panel.

`playground/` holds the site source; `playground/build.sh serve` runs it
locally (with the COOP/COEP headers).  All decode/encode work runs in a
Web Worker (`worker.js`) so the page never freezes during a decode; where
module workers are unavailable the page falls back to running the wasm on
the main thread.

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
