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

#[cfg(not(feature = "libtiff"))]
mod tiff;
#[cfg(not(feature = "libtiff"))]
pub use tiff::*;
#[cfg(feature = "libtiff")]
mod libtiff;
#[cfg(feature = "libtiff")]
pub use libtiff::*;

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
    Some(ImageFileFormat::TIF) => load_tiff_image(path, params),
    _ => {
      let image = read_image(path)?;
      convert_from_dynamic_image(image, params)
    }
  }
}

pub fn read_image(path: &Path) -> Result<DynamicImage, ImageError> {
  Ok(image::open(path).map_err(|e| ImageError::ReadError(e.to_string()))?)
}

pub fn save_image(image: &mut opj_image, path: &Path, split_comps: bool) -> Result<(), ImageError> {
  let format = ImageFileFormat::get_file_format(path)
    .map_err(|_| ImageError::InvalidFormat("Unknown file format".into()))?;

  match format {
    ImageFileFormat::RAW => save_raw_image(image, path, true),
    ImageFileFormat::RAWL => save_raw_image(image, path, false),
    ImageFileFormat::PGX => save_pgx_image(image, path),
    ImageFileFormat::PNG => save_png_image(image, path),
    ImageFileFormat::TIF => save_tiff_image(image, path),
    ImageFileFormat::PXM => save_pxm_image(image, path, split_comps),
    _ => {
      let dynamic_img = convert_to_dynamic_image(image)?;

      // Save the image based on file extension
      dynamic_img
        .save(path)
        .map_err(|e| ImageError::EncodeError(e.to_string()))
    }
  }
}

pub fn save_pxm_image(
  image: &mut opj_image,
  path: &Path,
  split_comps: bool,
) -> Result<(), ImageError> {
  let single_file = !split_comps && image.comps_match();
  if single_file {
    save_pxm_image_single(image, path)
  } else {
    save_pxm_image_multi(image, path)
  }
}

pub fn save_pxm_image_single(image: &mut opj_image, path: &Path) -> Result<(), ImageError> {
  let dynamic_img = convert_to_dynamic_image(image)?;

  // Save the image based on file extension
  dynamic_img
    .save(path)
    .map_err(|e| ImageError::EncodeError(e.to_string()))
}

pub fn save_pxm_image_multi(image: &mut opj_image, path: &Path) -> Result<(), ImageError> {
  let Some(comps) = image.comps() else {
    return Err(ImageError::InvalidFormat("No components found".into()));
  };
  let Some(stem) = path.file_stem() else {
    return Err(ImageError::InvalidFormat("Invalid file path".into()));
  };
  let stem = stem.to_string_lossy();
  // Save each component as a separate file
  for (idx, comp) in comps.iter().enumerate() {
    let comp_path = path.with_file_name(format!("{}_{}.pgm", stem, idx));
    let comp_img = convert_comp_to_dynamic_grayscale(comp)?;
    comp_img
      .save(comp_path)
      .map_err(|e| ImageError::EncodeError(e.to_string()))?;
  }
  Ok(())
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
