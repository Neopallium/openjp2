# openjp2-tools

Command-line tools for the pure-Rust OpenJPEG port: byte- and
behaviour-parity reimplementations of the upstream OpenJPEG CLIs.

| Binary | Upstream equivalent |
| --- | --- |
| `opj_compress` | `opj_compress` — encode PGM/PPM/PAM/TIFF/PNG to J2K/JP2 |
| `opj_decompress` | `opj_decompress` — decode J2K/JP2 to PGM/PPM/PAM |
| `opj_dump` | `opj_dump` — dump codestream structure and codestream info |

The tools link the core `openjp2` crate directly (no C ABI) and mirror
the upstream CLI's parameters and output formats so the ctest
non-regression suite can run against them byte-for-byte.

> **Not for release use.** This crate exists to gate C parity (see
> `scripts/check_ctest.sh`); it is not published. The modern user-facing
> CLI lives in the sibling `openjp2-cli` crate (`cli/`): `compress`,
> `decompress` and `dump` subcommands without C-parity parameters.

## Build

```sh
cargo build -p openjp2-tools
```

## Test

```sh
# conformance + decode-area suites (test data required):
OPJ_DATA_ROOT=openjpeg/data cargo test -p openjp2-tools --test conformance
OPJ_DATA_ROOT=openjpeg/data cargo test -p openjp2-tools --test tile_and_decode_area
# full non-regression suite (~2-3 min):
OPJ_DATA_ROOT=openjpeg/data cargo test -p openjp2-tools --test nonreg
```

## License

BSD-2-Clause, like the rest of the project.
