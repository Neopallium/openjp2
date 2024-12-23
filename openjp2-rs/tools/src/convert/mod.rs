mod raw;
pub use raw::*;
mod bmp;
pub use bmp::*;
mod pgx;
pub use pgx::*;
mod png;
pub use png::*;
mod dynamic;
pub use dynamic::*;

use crate::params::CompressionParameters;
use crate::params::ImageFileFormat;
use image::{self, DynamicImage};
use openjp2::image::opj_image;
use std::path::Path;

// Replace existing load_image function
pub fn load_image(
  path: &Path,
  params: &CompressionParameters,
) -> Result<Box<opj_image>, ImageError> {
  match params.decode_format {
    Some(ImageFileFormat::RAW) => load_raw_image(path, params, true),
    Some(ImageFileFormat::RAWL) => load_raw_image(path, params, false),
    Some(ImageFileFormat::BMP) => load_bmp_image(path, params),
    Some(ImageFileFormat::PNG) => load_png_image(path, params),
    _ => {
      let image = read_image(path)?;
      convert_from_dynamic_image(image, params)
    }
  }
}

pub fn read_image(path: &Path) -> Result<DynamicImage, ImageError> {
  Ok(image::open(path).map_err(|e| ImageError::ReadError(e.to_string()))?)
}

pub fn save_image(image: &opj_image, path: &Path) -> Result<(), ImageError> {
  let format = ImageFileFormat::get_file_format(path)
    .map_err(|_| ImageError::InvalidFormat("Unknown file format".into()))?;

  match format {
    ImageFileFormat::RAW => save_raw_image(image, path, true),
    ImageFileFormat::RAWL => save_raw_image(image, path, false),
    ImageFileFormat::PGX => save_pgx_image(image, path),
    _ => {
      let dynamic_img = convert_to_dynamic_image(image)?;

      // Save the image based on file extension
      dynamic_img
        .save(path)
        .map_err(|e| ImageError::EncodeError(e.to_string()))
    }
  }
}

// Add error types
#[derive(Debug)]
pub enum ImageError {
  InvalidFormat(String),
  ReadError(String),
  EncodeError(String),
  DecodeError(String),
  IOError(std::io::Error),
}

impl std::fmt::Display for ImageError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
      Self::ReadError(s) => write!(f, "Read error: {}", s),
      Self::EncodeError(s) => write!(f, "Encode error: {}", s),
      Self::DecodeError(s) => write!(f, "Decode error: {}", s),
      Self::IOError(e) => write!(f, "IO error: {}", e),
    }
  }
}

impl std::error::Error for ImageError {}

impl From<std::io::Error> for ImageError {
  fn from(error: std::io::Error) -> Self {
    ImageError::IOError(error)
  }
}
