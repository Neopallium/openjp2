# openjp2-ctypes

C-ABI mirror types and constants for the `openjp2` crates.

This crate exists so the `openjp2` core and the `openjp2-capi` shim share
one definition of the types that appear at the C boundary:

- `opj_*_t` structs (image/component parameters, codestream info,
  codec parameters, stream parameters) — `ctypes/src/lib.rs`
- the `openjpeg.h` constants and enums — `ctypes/src/consts.rs`

The pure-Rust core re-exports these under `openjp2::ffi` (the only place
C types appear in the core's public namespace); `openjp2-capi` uses them
directly to implement the `#[no_mangle]` ABI. Nothing else should need
this crate.

## License

BSD-2-Clause, like the rest of the project.
