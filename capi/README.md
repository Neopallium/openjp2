# openjp2-capi

The C ABI for the pure-Rust OpenJPEG port: a `#[no_mangle]` reimplementation
of the upstream `openjpeg.h` entry points, producing `libopenjp2.{a,so}`.

The shim is thin and total:

- every exported function is panic-guarded (`ffi_guard`), so a panic can
  never unwind into C;
- the merged `Codec` handle preserves the C contract of one
  `opj_codec_t*` per open codec, including the C's wrong-direction error
  events;
- `opj_dparameters_t` / `opj_cparameters_t` are marshalled into the core's
  native settings at the boundary.

The C ABI is a compatibility layer for existing OpenJPEG consumers; new
Rust code should use the safe `openjp2` API directly.

## Features

| Feature | Default | Meaning |
| --- | --- | --- |
| `file-io` | on | `FILE*`-based streams and the FILE dump writer (needs `libc`; disable for no_std/WASM targets — callback streams still work) |
| `openjpeg-version-string` | off | write the upstream OpenJPEG version string into encoded COM markers (byte-parity with the C encoder) |

## Build

The crate produces `libopenjp2.a` / `libopenjp2.so` under the name the
upstream CMake build expects; the C/CMake reference project in
`openjpeg/` links it (`-DUSE_RUST_LIB=ON`) and runs the upstream ctest
suite against it — the project's primary conformance gate.

## License

BSD-2-Clause, like the rest of the project.
