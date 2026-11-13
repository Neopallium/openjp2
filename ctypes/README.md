# openjp2-ctypes

C-ABI mirror types and constants for the `openjp2` crates.

This crate exists so the `openjp2` core and the `openjp2-capi` shim share
one definition of the types that appear at the C boundary:

- `opj_*_t` structs (image/component parameters, codestream info,
  codec parameters, stream parameters) — `ctypes/src/lib.rs`
- the `openjpeg.h` constants and enums — `ctypes/src/consts.rs`

The core itself is C-name-free: it uses these types only behind the
deliberate `ColorSpace`/`ProgressionOrder` conversion impls and the
C-shaped low-level parameter structs, never in its public signatures.
`openjp2-capi` uses them directly to implement the `#[no_mangle]` ABI.
Nothing else should need this crate.

## License

BSD-2-Clause, like the rest of the project.
