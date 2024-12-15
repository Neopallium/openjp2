use crate::compress::{CompressionParameters, DecodeFormat};
use crate::params::ParameterError;
use image::{self, DynamicImage};
use openjp2::{image::opj_image, openjpeg::*};
use std::io;
use std::path::Path;
use std::str::FromStr;

// For raw image parameters
#[derive(Clone, Debug, Default)]
pub struct RawParameters {
  pub width: u32,
  pub height: u32,
  pub num_comps: u32,
  pub bit_depth: u32,
  pub signed: bool,
  pub components: Vec<RawComponentParameters>,
}

impl FromStr for RawParameters {
  type Err = ParameterError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = s.split(&[',', '@', ':']).collect();
    if parts.len() < 5 {
      return Err(ParameterError::InvalidFormat(
        "Raw params format: width,height,ncomp,bitdepth,[s|u]@dx1,dy1:...:dxn,dyn".into(),
      ));
    }

    let width = parts[0]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid width".into()))?;
    let height = parts[1]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid height".into()))?;
    let num_comps = parts[2]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid component count".into()))?;
    let bit_depth = parts[3]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid bit depth".into()))?;
    let signed = match parts[4] {
      "s" => true,
      "u" => false,
      _ => {
        return Err(ParameterError::InvalidValue(
          "Signed flag must be 's' or 'u'".into(),
        ))
      }
    };

    let mut components = Vec::new();
    if parts.len() > 5 {
      // Parse subsampling factors
      for comp in parts[5..].iter() {
        components.push(comp.parse()?);
      }
    } else {
      // Default 1x1 subsampling for all components
      components = vec![RawComponentParameters { dx: 1, dy: 1 }; num_comps as usize];
    }

    Ok(RawParameters {
      width,
      height,
      num_comps,
      bit_depth,
      signed,
      components,
    })
  }
}

#[derive(Clone, Debug, Default)]
pub struct RawComponentParameters {
  pub dx: u32,
  pub dy: u32,
}

impl FromStr for RawComponentParameters {
  type Err = ParameterError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let factors: Vec<&str> = s.split('x').collect();
    if factors.len() != 2 {
      return Err(ParameterError::InvalidFormat(
        "Subsampling format: dx x dy".into(),
      ));
    }

    let dx = factors[0]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid dx".into()))?;
    let dy = factors[1]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid dy".into()))?;

    Ok(RawComponentParameters { dx, dy })
  }
}

// Add error types
#[derive(Debug)]
pub enum ImageError {
  InvalidFormat(String),
  ReadError(String),
  EncodeError(String),
  IOError(io::Error),
}

impl std::fmt::Display for ImageError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
      Self::ReadError(s) => write!(f, "Read error: {}", s),
      Self::EncodeError(s) => write!(f, "Encode error: {}", s),
      Self::IOError(e) => write!(f, "IO error: {}", e),
    }
  }
}

impl std::error::Error for ImageError {}

impl From<io::Error> for ImageError {
  fn from(error: io::Error) -> Self {
    ImageError::IOError(error)
  }
}

// Add this struct to represent our image data
#[derive(Debug)]
pub struct ImageComponent {
  pub data: Vec<i32>,
  pub width: u32,
  pub height: u32,
  pub precision: u32,
  pub signed: bool,
  pub dx: u32,
  pub dy: u32,
}

// Replace existing load_image function
pub fn load_image(
  path: &Path,
  params: &CompressionParameters,
) -> Result<Box<opj_image>, ImageError> {
  // Convert the loaded image to OpenJPEG format
  match params.decode_format {
    Some(DecodeFormat::RAW | DecodeFormat::RAWL) => {
      todo!("Implement loading raw image data");
    }
    _ => {
      let image = read_image(path)?;
      convert_image(image, params)
    }
  }
}

fn read_image(path: &Path) -> Result<DynamicImage, ImageError> {
  Ok(image::open(path).map_err(|e| ImageError::ReadError(e.to_string()))?)
}

fn convert_image(
  in_img: DynamicImage,
  params: &CompressionParameters,
) -> Result<Box<opj_image>, ImageError> {
  // Get information from input image.
  let (width, height, numcomps, color_space, bit_depth, sgnd) = match &in_img {
    DynamicImage::ImageLuma8(img) => {
      let (width, height) = img.dimensions();
      (width, height, 1, OPJ_CLRSPC_GRAY, 8, false)
    }
    DynamicImage::ImageLumaA8(img) => {
      let (width, height) = img.dimensions();
      (width, height, 2, OPJ_CLRSPC_GRAY, 8, false)
    }
    DynamicImage::ImageRgb8(img) => {
      let (width, height) = img.dimensions();
      (width, height, 3, OPJ_CLRSPC_SRGB, 8, false)
    }
    DynamicImage::ImageRgba8(img) => {
      let (width, height) = img.dimensions();
      (width, height, 4, OPJ_CLRSPC_SRGB, 8, false)
    }
    DynamicImage::ImageRgb16(img) => {
      let (width, height) = img.dimensions();
      (width, height, 3, OPJ_CLRSPC_SRGB, 16, false)
    }
    DynamicImage::ImageRgba16(img) => {
      let (width, height) = img.dimensions();
      (width, height, 4, OPJ_CLRSPC_SRGB, 16, false)
    }
    DynamicImage::ImageLuma16(img) => {
      let (width, height) = img.dimensions();
      (width, height, 1, OPJ_CLRSPC_GRAY, 16, false)
    }
    DynamicImage::ImageLumaA16(img) => {
      let (width, height) = img.dimensions();
      (width, height, 2, OPJ_CLRSPC_GRAY, 16, false)
    }
    DynamicImage::ImageRgb32F(img) => {
      let (width, height) = img.dimensions();
      (width, height, 3, OPJ_CLRSPC_SRGB, 32, true)
    }
    DynamicImage::ImageRgba32F(img) => {
      let (width, height) = img.dimensions();
      (width, height, 4, OPJ_CLRSPC_SRGB, 32, true)
    }
    _ => {
      return Err(ImageError::InvalidFormat(
        "Unsupported image format".to_string(),
      ))
    }
  };

  let mut image = opj_image::new();

  let offset = params.image_offset();
  let subsampling = params.subsampling();

  image.x0 = offset.x;
  image.y0 = offset.y;
  image.x1 = offset.x + (width - 1) * subsampling.width + 1;
  image.y1 = offset.y + (height - 1) * subsampling.height + 1;
  image.color_space = color_space;
  image.alloc_comps(numcomps);

  let comps = image.comps_mut().expect("We just allocated them");
  // Initialize components
  for comps in comps.iter_mut() {
    comps.dx = subsampling.width;
    comps.dy = subsampling.height;
    comps.w = width;
    comps.h = height;
    comps.x0 = 0;
    comps.y0 = 0;
    comps.prec = bit_depth;
    comps.sgnd = sgnd as u32;
    if !comps.alloc_data() {
      return Err(ImageError::InvalidFormat(
        "Failed to allocate component data".into(),
      ));
    }
  }

  // Get mutable references to component data.
  let mut data = comps
    .into_iter()
    .map(|c| c.data_mut().expect("We just allocated it"));

  match in_img {
    DynamicImage::ImageLuma8(img) => {
      let grey = data.next().expect("We just allocated all the components");
      for (grey, pixel) in grey.into_iter().zip(img.pixels()) {
        *grey = pixel[0] as i32;
      }
    }
    DynamicImage::ImageLumaA8(img) => {
      // get each components data.
      let grey = data.next().expect("We just allocated all the components");
      let alpha = data.next().expect("We just allocated all the components");
      // zip the components data to access them together.
      let data = grey.into_iter().zip(alpha.into_iter());

      // iterate over the pixels and assign the pixel values to the components data.
      for ((grey, alpha), pixel) in data.zip(img.pixels()) {
        *grey = pixel[0] as i32;
        *alpha = pixel[1] as i32;
      }
    }
    DynamicImage::ImageRgb8(img) => {
      // get each components data.
      let r = data.next().expect("We just allocated all the components");
      let g = data.next().expect("We just allocated all the components");
      let b = data.next().expect("We just allocated all the components");
      // zip the components data to access them together.
      let data = r.into_iter().zip(g.into_iter()).zip(b.into_iter());

      // iterate over the pixels and assign the pixel values to the components data.
      for (((r, g), b), pixel) in data.zip(img.pixels()) {
        *r = pixel[0] as i32;
        *g = pixel[1] as i32;
        *b = pixel[2] as i32;
      }
    }
    DynamicImage::ImageRgba8(img) => {
      // get each components data.
      let r = data.next().expect("We just allocated all the components");
      let g = data.next().expect("We just allocated all the components");
      let b = data.next().expect("We just allocated all the components");
      let a = data.next().expect("We just allocated all the components");
      // zip the components data to access them together.
      let data = r
        .into_iter()
        .zip(g.into_iter())
        .zip(b.into_iter())
        .zip(a.into_iter());

      // iterate over the pixels and assign the pixel values to the components data.
      for ((((r, g), b), a), pixel) in data.zip(img.pixels()) {
        *r = pixel[0] as i32;
        *g = pixel[1] as i32;
        *b = pixel[2] as i32;
        *a = pixel[3] as i32;
      }
    }
    DynamicImage::ImageLuma16(img) => {
      let grey = data.next().expect("We just allocated all the components");
      for (grey, pixel) in grey.into_iter().zip(img.pixels()) {
        *grey = pixel[0] as i32;
      }
    }
    DynamicImage::ImageLumaA16(img) => {
      // get each components data.
      let grey = data.next().expect("We just allocated all the components");
      let alpha = data.next().expect("We just allocated all the components");
      // zip the components data to access them together.
      let data = grey.into_iter().zip(alpha.into_iter());

      // iterate over the pixels and assign the pixel values to the components data.
      for ((grey, alpha), pixel) in data.zip(img.pixels()) {
        *grey = pixel[0] as i32;
        *alpha = pixel[1] as i32;
      }
    }
    DynamicImage::ImageRgb16(img) => {
      // get each components data.
      let r = data.next().expect("We just allocated all the components");
      let g = data.next().expect("We just allocated all the components");
      let b = data.next().expect("We just allocated all the components");
      // zip the components data to access them together.
      let data = r.into_iter().zip(g.into_iter()).zip(b.into_iter());

      // iterate over the pixels and assign the pixel values to the components data.
      for (((r, g), b), pixel) in data.zip(img.pixels()) {
        *r = pixel[0] as i32;
        *g = pixel[1] as i32;
        *b = pixel[2] as i32;
      }
    }
    DynamicImage::ImageRgba16(img) => {
      // get each components data.
      let r = data.next().expect("We just allocated all the components");
      let g = data.next().expect("We just allocated all the components");
      let b = data.next().expect("We just allocated all the components");
      let a = data.next().expect("We just allocated all the components");
      // zip the components data to access them together.
      let data = r
        .into_iter()
        .zip(g.into_iter())
        .zip(b.into_iter())
        .zip(a.into_iter());

      // iterate over the pixels and assign the pixel values to the components data.
      for ((((r, g), b), a), pixel) in data.zip(img.pixels()) {
        *r = pixel[0] as i32;
        *g = pixel[1] as i32;
        *b = pixel[2] as i32;
        *a = pixel[3] as i32;
      }
    }
    _ => {
      return Err(ImageError::InvalidFormat(
        "Unsupported image format - convert to RGB8 or Luma8 first".into(),
      ));
    }
  }
  Ok(image)
}
