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
| `decode_info(bytes)` | `{ width, height, components, prec, sgnd }` without decoding pixels. |
| `decode_to_rgba(bytes, strict?)` | Decodes to tightly packed RGBA8 (`w * h * 4` bytes). |
| `encode_rgba_to_j2k(w, h, rgba, lossless)` | Encodes RGBA8 to a J2K codestream. |
| `encode_rgba_to_jp2(w, h, rgba, lossless)` | Encodes RGBA8 to a JP2 file. |

Pixels are always RGBA8: 1-component images become grayscale, 3-component
RGB; samples outside 8 bits are shifted/clamped. Subsampled components are
rejected (use the full `openjp2` API for those). ICC colour management is
not applied.

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
