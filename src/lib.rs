#![allow(dead_code)]
#![allow(non_camel_case_types)]
// The C-ABI mirror types keep the C header's field spellings; those names
// carry a scoped #![allow(non_snake_case)] in c_api_types.rs itself. All
// c2rust-residue identifiers were renamed to Rust style.
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![deny(unsafe_op_in_unsafe_fn)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! A pure-Rust port of [OpenJPEG](https://github.com/uclouvain/openjpeg) with a
//! safe high-level API, a C ABI drop-in, and CLI tools.
//!
//! # Examples
//!
//! ## Decode a JP2/J2K image
//!
//! ```
//! use openjp2::*;
//!
//! fn main() -> Result<()> {
//!     // One-shot decode (JP2 or J2K, auto-detected; multi-tile codestreams
//!     // decode tiles in parallel).
//!     let image = Image::from_file("samples/Hadley_Crater.jp2")?;
//!     println!("{}x{} {:?}", image.width(), image.height(), image.color_space());
//!     Ok(())
//! }
//! ```
//!
//! ## Encode to JP2/J2K
//!
//! ```
//! use openjp2::*;
//!
//! fn main() -> Result<()> {
//!     let image = Image::from_file("samples/Hadley_Crater.jp2")?;
//!
//!     // Raw codestream (J2K); the default 5-3 wavelet is reversible (lossless).
//!     let codestream: Vec<u8> = image.clone().encode(EncodeParameters::new())?;
//!
//!     // Or wrapped in the JP2 container.
//!     let jp2: Vec<u8> = image.clone().encode_jp2(EncodeParameters::new().num_resolutions(3))?;
//!
//!     // Both round-trip back to an image.
//!     let back = Image::from_bytes(&jp2)?;
//!     assert_eq!((back.width(), back.height()), (image.width(), image.height()));
//!     Ok(())
//! }
//! ```
//!
//! ## Encode from another image format
//!
//! The `image` feature (default) bridges to the [`image`](https://crates.io/crates/image)
//! crate: decode a PNG/JPEG/… and write it back as JPEG 2000.
//!
//! ```no_run
//! use openjp2::*;
//!
//! fn main() -> Result<()> {
//!     let source = ::image::open("photo.png").expect("decode the input image");
//!     let jp2 = Image::from_dynamic_image(&source)
//!         .encode_jp2(EncodeParameters::new().num_resolutions(5))?;
//!     std::fs::write("photo.jp2", jp2)?;
//!     Ok(())
//! }
//! ```
//!
//! ## Decode the header only
//!
//! ```
//! use openjp2::*;
//!
//! fn main() -> Result<()> {
//!     // Inspect the image dimensions without decoding the pixel data.
//!     let buf = std::fs::read("samples/Hadley_Crater.jp2")?;
//!     let mut decoder = Decoder::new(J2KFormat::JP2)?;
//!     let mut stream = Stream::from_bytes(buf);
//!     let header = decoder.read_header(&mut stream)?;
//!     println!("{}x{} ({} components)", header.width(), header.height(), header.num_components());
//!     Ok(())
//! }
//! ```
//!
//! # Design specification
//!
//! The JPEG 2000 formats and algorithms this port implements are
//! documented in the [`design_spec`] module (a documentation-only
//! module, kept off the main crate page).

use openjp2_ctypes as c_api_types;

/// The JPEG 2000 design specification: the codestream structure, the
/// decode/encode pipelines, and the formats (Part 1, JP2, HTJ2K), written
/// without code references.
///
/// This module exists for documentation only; it contains no items.
pub mod design_spec {
  #![doc = include_str!("../doc/jpeg2000-design-spec.md")]
}
/// C-mirror constants (`openjpeg.h` values grouped by domain).
pub(crate) mod consts;
mod types;

#[macro_use]
mod event;

#[macro_use]
mod fprintf;
pub use fprintf::StringWriter;

/// The image data model (`CoreImage`, `CoreImageComp`, ICC profiles).
pub mod image;
/// Byte streams for the codecs (`Stream` and its constructors).
pub mod stream;
// The C-parity re-export hub (ctypes + consts + internal types) is
// crate-private: the C surface lives in openjp2-ctypes / openjp2-capi.
pub(crate) mod openjpeg;

/// The high-level safe API: one-shot `Image` decode/encode, the
/// `DecodeParameters`/`EncodeParameters` builders, `Decoder`/`Encoder`
/// settings, typed errors and the `image`-crate interop.
pub mod api;

/// Whether the build includes the `rayon` feature (the C-ABI
/// `has_thread_support` reports this).
pub fn has_thread_support() -> bool {
  cfg!(feature = "rayon")
}

#[cfg(feature = "image")]
pub use api::image_conv::{ImageData, ImagePixelData};
pub use api::{
  ColorSpace, DecodeArea, DecodeError, DecodeLimits, DecodeParameters, DumpImage, EncodeParameters,
  Error, Image, ProgressionOrder, Result,
};

pub use api::EncoderParameters;
pub use codec::TileInfo;
pub use codec::{Decoder, DecoderSettings, Encoder, HeaderImage};
pub use image::{CoreImage, CoreImageComp};
pub use image::{ICCProfile, ICCProfileMut, ICCProfileRef, CIE_LAB_BYTE_SIZE};
pub use j2k::{
  CodestreamIndex, CodestreamTileIndex, CstrCompInfoV2, CstrInfoV2, CstrTccpInfoV2, MarkerInfo,
  PacketInfo, TpIndex,
};
pub use stream::Stream;
pub use types::Poc;

/// Magic bytes for JP2 RFC3745.
pub const JP2_RFC3745_MAGIC: &[u8] = &[
  0x00, 0x00, 0x00, 0x0c, 0x6a, 0x50, 0x20, 0x20, 0x0d, 0x0a, 0x87, 0x0a,
];
/// The JP2 signature box contents (`0d 0a 87 0a`).
pub const JP2_MAGIC: &[u8] = &[0x0d, 0x0a, 0x87, 0x0a];
/// Magic bytes for J2K Codestream.
pub const J2K_CODESTREAM_MAGIC: &[u8] = &[0xff, 0x4f, 0xff, 0x51];

/// Supported Jpeg 2000 formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum J2KFormat {
  /// The JP2 container (ISO/IEC 15444-1 Annex I).
  JP2,
  /// A raw codestream (Part 1).
  J2K,
  /// JPT (Part 3): recognized but unsupported by this port.
  JPT,
}

/// Detect Jpeg 2000 format from magic bytes.
pub fn detect_format(buf: &[u8]) -> Result<J2KFormat, String> {
  if buf.starts_with(JP2_RFC3745_MAGIC) || buf.starts_with(JP2_MAGIC) {
    Ok(J2KFormat::JP2)
  } else if buf.starts_with(J2K_CODESTREAM_MAGIC) {
    Ok(J2KFormat::J2K)
  } else {
    Err("Can't detect image format from bytes".into())
  }
}

/// Detect Jpeg 2000 format from file extension.
pub fn detect_format_from_extension(ext: Option<&str>) -> Result<J2KFormat, String> {
  let lower_ext = ext.map(|e| e.to_ascii_lowercase());
  match lower_ext.as_deref() {
    Some("jp2") => Ok(J2KFormat::JP2),
    Some("jpt") => Ok(J2KFormat::JPT),
    Some("j2k") | Some("j2c") | Some("jpc") => Ok(J2KFormat::J2K),
    // HTJ2K with JP2 boxes
    Some("jph") => Ok(J2KFormat::JP2),
    // HTJ2K codestream
    Some("jhc") => Ok(J2KFormat::J2K),
    Some(ext) => Err(format!("Unknown file extension: {}", ext)),
    None => Err("No file extension".into()),
  }
}

/// Detects the format from a file extension (`OsStr` flavour).
pub fn detect_format_from_extension_os_str(
  ext: Option<&std::ffi::OsStr>,
) -> Result<J2KFormat, String> {
  let ext = ext.map(|ext| ext.to_string_lossy());
  detect_format_from_extension(ext.as_deref())
}

fn detect_from_file_magic<P: AsRef<std::path::Path>>(path: P) -> Result<J2KFormat, String> {
  use std::io::Read;
  // Read magic bytes from file
  let mut buf = [0; 12];
  let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
  file.read_exact(&mut buf).map_err(|e| e.to_string())?;

  // Detect format from magic bytes
  detect_format(&buf)
}

/// Detects the format of a file (magic bytes, then extension).
pub fn detect_format_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<J2KFormat, String> {
  let path = path.as_ref();
  let ext = path.extension().map(|ext| ext.to_string_lossy());
  let ext_format = detect_format_from_extension(ext.as_deref())?;
  if ext_format == J2KFormat::JPT {
    return Ok(J2KFormat::JPT);
  }

  // Detect format from magic bytes
  let Ok(magic_format) = detect_from_file_magic(path) else {
    return Ok(ext_format);
  };

  // Log warning if magic bytes and file extension don't match
  if ext_format != magic_format {
    eprintln!(
      "Warning: File {:?} extension format ({:?}) doesn't match magic bytes format ({:?})",
      path, ext_format, magic_format
    );
  }

  Ok(magic_format)
}

mod bio;
mod cio;
mod codec;
mod dwt;
mod ht_dec;
mod invert;
mod j2k;
mod jp2;
mod math;
mod mct;
mod mqc;
mod par;
mod pi;
mod sparse_array;
mod t1;
mod t1_ht_luts;
mod t1_luts;
mod t2;
mod tcd;
mod tgt;
