# openjp2-cli (`openjp2` binary)

Modern command-line tools for the pure-Rust OpenJPEG port
([openjp2](../README.md)): a simplified, user-friendly interface on top of
the safe Rust API.  The C-parity tools (byte-compatible with the upstream
`opj_compress` / `opj_decompress` / `opj_dump`) live in the sibling
[`openjp2-tools`](../tools) crate.

## Install

```console
$ cargo install --locked openjp2-cli
```

The crate is named `openjp2-cli`, but it installs the **`openjp2`**
binary:

```console
$ openjp2 --version
```

To build from a checkout instead:

```console
$ cargo install --path cli
# development build, no install:
$ cargo run -p openjp2-cli -- --help
```

## Commands

### `compress` — encode an image to JP2/J2K

Reads any format the `image` crate understands (PNG, JPEG, TIFF, BMP,
TGA, WebP, PNM, ...) and writes a JPEG 2000 codestream:

```console
$ openjp2 compress photo.png photo.jp2
$ openjp2 compress photo.png out.j2k        # raw codestream
$ openjp2 compress -l -r 5 photo.png out.jp2     # lossy 9-7, 5 resolutions
$ openjp2 compress --layers 6 photo.png out.jp2   # 6 quality layers
$ openjp2 compress --rates 100,20,5,0 photo.png out.jp2  # explicit ladder
$ openjp2 compress --rate 20 photo.png out.jp2   # 20:1 ratio layer
$ openjp2 compress --tile 512x512 --order RLCP photo.png out.jp2
```

Defaults: lossless (reversible 5-3), 6 resolutions, 1 layer, single tile,
LRCP progression.

Quality layers need rate targets: a bare `--layers N` (N > 1) generates a
default geometric ladder from 100:1 to a lossless final layer; `--rates`
overrides it with an explicit comma-separated list (`0` = lossless).

### `decompress` — decode JP2/J2K to PNG

```console
$ openjp2 decompress photo.jp2               # writes photo.png
$ openjp2 decompress photo.jp2 out.jpg       # any image-crate format
$ openjp2 decompress --layers 2 --reduce 1 photo.jp2
$ openjp2 decompress --area 0,0,512,512 photo.jp2
```

Defaults: lenient mode (tolerates truncated codestreams), full quality,
all layers, auto thread count.

### `dump` — inspect the codestream structure

```console
$ openjp2 dump photo.jp2         # header, boxes, markers
$ openjp2 dump --tiles photo.jp2 # plus the tile-part index
```

## Differences from the C-parity tools

- No C-ABI parameters; only the modern safe-API options.
- No libtiff/liblcms/libpng C dependencies (PNG I/O goes through the
  `image` crate).
- Lenient decode by default (the C tools default to strict); pass
  `--strict` for strict validation.
