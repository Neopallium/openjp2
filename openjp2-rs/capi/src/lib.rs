//! C ABI for the Rust OpenJPEG port.
//!
//! This crate holds the `#[no_mangle]`/`#[repr(C)]` surface that produces the
//! drop-in `libopenjp2.a`/`libopenjp2.so`. The safe Rust API lives in the
//! `openjp2` core crate.

// Re-export the core's public surface (FFI types, codec handle, image, stream).
pub use openjp2_core::*;

pub mod openjpeg;
