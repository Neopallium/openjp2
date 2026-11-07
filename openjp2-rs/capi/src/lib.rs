//! C ABI for the Rust OpenJPEG port.
//!
//! This crate holds the `#[no_mangle]`/`#[repr(C)]` surface that produces the
//! drop-in `libopenjp2.a`/`libopenjp2.so`. The safe Rust API lives in the
//! `openjp2` core crate.

// The C ABI mirror must keep the libopenjp2 identifier spellings.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
// Every public fn here is a thin C-ABI shim; the Safety story is per-call-site.
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

// Re-export the core's public surface (FFI types, codec handle, image, stream).
pub use openjp2_core::*;

pub mod image;
pub mod openjpeg;
