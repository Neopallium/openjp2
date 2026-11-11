//! Split of the safe API (doc/api-review.md): this module is the facade
//! re-exporting the submodules below.  See `doc/redesign-proposal.md` B.3.2.

use crate::c_api_types::OPJ_COLOR_SPACE;
use crate::c_api_types::{
  OPJ_CLRSPC_CMYK, OPJ_CLRSPC_EYCC, OPJ_CLRSPC_GRAY, OPJ_CLRSPC_SRGB, OPJ_CLRSPC_SYCC,
  OPJ_CLRSPC_UNKNOWN,
};
use crate::codec::DecoderSettings;
use crate::{CoreImage, CoreImageComp, Decoder, Encoder, J2KFormat, Stream};

mod decode;
mod dump;
mod encode;
mod error;
mod image;
#[cfg(feature = "image")]
pub mod image_conv;
mod params;
#[cfg(test)]
mod tests;

pub(crate) use decode::decode_stream;
pub use decode::detect_format;
pub use dump::DumpImage;
pub use dump::{ALL, IMG_INFO, J2K_MH_IND, J2K_MH_INFO};
pub(crate) use encode::{encode_image, encode_image_to_writer};
pub(crate) use error::catch_codec_panic;
pub use error::{DecodeError, Error, Result};
pub use image::Image;
pub use params::*;
