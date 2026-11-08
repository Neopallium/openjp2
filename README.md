# openjp2

A pure-Rust port of [OpenJPEG](https://github.com/uclouvain/openjpeg)
(the JPEG 2000 reference implementation), with a safe high-level API,
a C ABI drop-in for `libopenjp2`, and CLI tools.

```
src/        the openjp2 crate (core + safe API)
capi/       the openjp2-capi crate: #[no_mangle] C ABI producing libopenjp2.{a,so}
tools/      the openjp2-tools crate: opj_compress / opj_decompress / opj_dump / ...
openjpeg/   the C/CMake reference project (upstream openjpeg, tests, test data)
doc/        design spec, code outline, API review
```

## Using the safe API

```rust
use openjp2::*;

fn main() -> Result<()> {
    // One-shot decode (JP2 or J2K, auto-detected).
    let image = Image::from_file("input.jp2")?;
    println!("{}x{} {:?}", image.width(), image.height(), image.color_space());

    // Decoder options via the builder.
    let image = Image::from_file_with(
        "input.jp2",
        DecodeParameters::new()
            .layers(4) // only the first 4 quality layers
            .decode_area(Some(DecodeArea::new(0, 0, 512, 512))),
    )?;

    // Convert to an image-crate DynamicImage (feature `image`, on by default).
    let dyn_img: image::DynamicImage = (&image).try_into()?;

    // Encode (reversible 5-3 by default; lossless round-trips; multi-tile
    // encodes are parallelised automatically).
    let codestream: Vec<u8> = image.encode(EncodeParameters::new().num_resolutions(5))?;
    Ok(())
}
```

The API mirrors the [`jpeg2k`](https://crates.io/crates/jpeg2k) crate so it
can act as a thin wrapper (see `doc/api-review.md`); the C-ABI mirror types
are available under the `ffi` module.

## Parallelism

Decoding is parallel on two levels (feature `rayon`): inside each tile
(tier-1 code-block decoding, DWT, DC shift) and across tiles for whole-image,
full-quality decodes of multi-tile codestreams (the safe `Image` API decodes
every tile with a fresh codec state over an in-memory snapshot and merges the
results; ~1.5x on 25-tile files at 24 threads).  Quality-layer limits,
resolution reduction and decode-area requests are honoured as well.
Multi-tile encoding is parallel the same way (per-tile fresh encoder
states, tile-parts written in order, ~1.3x at 24 threads).  Both
parallel paths fall back to the sequential code for any input or layout
they do not support, so the output is always identical to the serial
path.

## Features

| Feature    | Default | Meaning |
|---|---|---|
| `rayon`    | on      | Parallel tier-1 codeblock coding, DWT, DC shift, tile-level decode/encode (alias: `threads`) |
| `image`    | on      | `get_pixels` / `DynamicImage` interop with the `image` crate |

Build without rayon for no-rayon targets: `cargo build --no-default-features --features image`.

## Development

- The C reference project lives in `openjpeg/` (build with
  `cmake -S openjpeg -B openjpeg/build -DUSE_RUST_LIB=ON ...`); the ctest
  suite links the **Rust** library and is the primary conformance gate
  (`scripts/check_ctest.sh`).
- Test data is a clone of
  [openjpeg-data](https://github.com/uclouvain/openjpeg-data) under
  `openjpeg/data/` (untracked).
- Gates: `cargo test --workspace` (unit + tools conformance/nonreg),
  `bash scripts/check_ctest.sh`, `cd capi && cargo test`.
- `OPJ_PROFILE=1` prints per-stage timings; `OPJ_NUM_THREADS` sizes the
  rayon pool.

## Docs

- `doc/jpeg2000-design-spec.md` — JPEG 2000 formats & algorithms (no code refs)
- `doc/code-outline.md` — module map onto the spec and the pipelines
- `doc/api-review.md` — public API review + the jpeg2k alignment plan
- `doc/differences.md` — intentional divergences from the C reference
