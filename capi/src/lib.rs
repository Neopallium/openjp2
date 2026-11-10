#![allow(deprecated)] // the C ABI shim legitimately drives the low-level Codec API
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

// The C-ABI mirror types come straight from openjp2-ctypes; the core no
// longer re-exports C types at its root (0.7 breaking change).
pub use openjp2_ctypes::*;
// Re-export the core's safe surface for Rust users of this crate.
pub use openjp2_core::{
  ColorSpace, DecodeArea, DecodeLimits, DecodeParameters, DumpImage,
  EncodeParameters, Error, Image, ProgressionOrder, Result,
  Codec, opj_codec_private_t, opj_stream_private_t,
  CoreImage, CoreImageComp,
  CIE_LAB_BYTE_SIZE, ICCProfile, ICCProfileMut, ICCProfileRef,
  CodestreamIndex, CodestreamTileIndex, CstrCompInfoV2, CstrInfoV2,
  CstrTccpInfoV2, EncoderParameters, Stream, TileInfo,
  J2KFormat, detect_format, detect_format_from_extension,
  detect_format_from_extension_os_str, detect_format_from_file,
  has_thread_support, JP2_RFC3745_MAGIC,
};

pub mod image;
pub mod params;
pub mod cstr_info;
pub mod openjpeg;
