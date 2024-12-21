use crate::compress::{CompressionParameters, ImageFileFormat, MCTMode};
use crate::params::ParameterError;
use image::{self, DynamicImage};
use openjp2::{image::opj_image, openjpeg::*};
use std::io::{self, BufWriter, Read, Write};
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
  match params.decode_format {
    Some(ImageFileFormat::RAW) => load_raw_image(path, params, true),
    Some(ImageFileFormat::RAWL) => load_raw_image(path, params, false),
    _ => {
      let image = read_image(path)?;
      convert_from_dynamic_image(image, params)
    }
  }
}

fn load_raw_image(
  path: &Path,
  params: &CompressionParameters,
  big_endian: bool,
) -> Result<Box<opj_image>, ImageError> {
  // Get raw parameters from compression parameters
  let raw_params = params.raw_params.as_ref().ok_or_else(|| {
    ImageError::InvalidFormat("Raw parameters required for RAW/RAWL format".into())
  })?;

  // Validate parameters
  if raw_params.width == 0
    || raw_params.height == 0
    || raw_params.num_comps == 0
    || raw_params.bit_depth == 0
  {
    return Err(ImageError::InvalidFormat(
      "Invalid raw image parameters. Use -F option.".into(),
    ));
  }

  // Calculate buffer size needed for one component
  let bytes_per_sample = match raw_params.bit_depth {
    bd if bd <= 8 => 1,
    bd if bd <= 16 => 2,
    bd if bd <= 32 => 4,
    _ => {
      return Err(ImageError::InvalidFormat(
        "Bit depth > 32 not supported".into(),
      ))
    }
  };

  let num_pixels = (raw_params.width * raw_params.height) as usize;
  let buffer_size = num_pixels * bytes_per_sample;

  // Create image and initialize components
  let mut image = opj_image::new();
  image.color_space = if raw_params.num_comps == 1 {
    OPJ_CLRSPC_GRAY
  } else if raw_params.num_comps >= 3 {
    match &params.mct_mode {
      Some(MCTMode::None) => OPJ_CLRSPC_SYCC,
      None | Some(MCTMode::RGB2YCC) => OPJ_CLRSPC_SRGB,
      _ => OPJ_CLRSPC_UNKNOWN,
    }
  } else {
    OPJ_CLRSPC_UNKNOWN
  };

  // Set image parameters
  let offset = params.image_offset();
  let subsampling = params.subsampling();

  image.x0 = offset.x;
  image.y0 = offset.y;
  image.x1 = offset.x + (raw_params.width - 1) * subsampling.width + 1;
  image.y1 = offset.y + (raw_params.height - 1) * subsampling.height + 1;

  // Allocate components
  image.alloc_comps(raw_params.num_comps);
  let comps = image.comps_mut().expect("We just allocated the components");

  // Initialize components
  for (i, comp) in comps.iter_mut().enumerate() {
    let raw_comp = raw_params
      .components
      .get(i)
      .unwrap_or(&RawComponentParameters { dx: 1, dy: 1 });

    comp.dx = subsampling.width * raw_comp.dx;
    comp.dy = subsampling.height * raw_comp.dy;
    comp.w = raw_params.width;
    comp.h = raw_params.height;
    comp.x0 = 0;
    comp.y0 = 0;
    comp.prec = raw_params.bit_depth;
    comp.sgnd = raw_params.signed as u32;

    if !comp.alloc_data() {
      return Err(ImageError::InvalidFormat(
        "Failed to allocate component data".into(),
      ));
    }
  }

  // Allocate single reusable buffer for reading component data
  let mut buffer = vec![0u8; buffer_size];
  let mut file = std::fs::File::open(path)?;

  // Read and process one component at a time
  for comp in comps.iter_mut() {
    // Read data for this component
    file.read_exact(&mut buffer)?;

    let data = comp.data_mut().expect("We just allocated it");

    match raw_params.bit_depth {
      bd if bd <= 8 => {
        for (dst, &src) in data.iter_mut().zip(buffer.iter()) {
          *dst = if raw_params.signed {
            src as i8 as i32
          } else {
            src as i32
          };
        }
      }
      bd if bd <= 16 => {
        let from_bytes = if big_endian {
          u16::from_be_bytes
        } else {
          u16::from_le_bytes
        };
        for (dst, bytes) in data.iter_mut().zip(buffer.chunks_exact(2)) {
          let value = from_bytes([bytes[0], bytes[1]]);
          *dst = if raw_params.signed {
            value as i16 as i32
          } else {
            value as i32
          };
        }
      }
      bd if bd <= 32 => {
        let from_bytes = if big_endian {
          u32::from_be_bytes
        } else {
          u32::from_le_bytes
        };
        for (dst, bytes) in data.iter_mut().zip(buffer.chunks_exact(4)) {
          let value = from_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
          *dst = value as i32;
        }
      }
      _ => unreachable!(),
    }
  }

  Ok(image)
}

fn read_image(path: &Path) -> Result<DynamicImage, ImageError> {
  Ok(image::open(path).map_err(|e| ImageError::ReadError(e.to_string()))?)
}

fn convert_from_dynamic_image(
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

pub fn save_image(image: &opj_image, path: &Path) -> Result<(), ImageError> {
  let format = ImageFileFormat::get_file_format(path)
    .map_err(|_| ImageError::InvalidFormat("Unknown file format".into()))?;

  match format {
    ImageFileFormat::RAW => save_raw_image(image, path, true),
    ImageFileFormat::RAWL => save_raw_image(image, path, false),
    _ => {
      let dynamic_img = convert_to_dynamic_image(image)?;

      // Save the image based on file extension
      dynamic_img
        .save(path)
        .map_err(|e| ImageError::EncodeError(e.to_string()))
    }
  }
}

pub fn save_raw_image(image: &opj_image, path: &Path, big_endian: bool) -> Result<(), ImageError> {
  let file = std::fs::File::create(path)?;
  let mut writer = BufWriter::new(file);

  // Check that the image components have matching dimensions, sampling factors, bit depth and signedness.
  if !image.comps_match() {
    return Err(ImageError::InvalidFormat(
      "Mismatched component parameters".into(),
    ));
  }

  let Some(comps) = image.comps_data_iter() else {
    return Err(ImageError::InvalidFormat("Missing components".into()));
  };

  // Write each component's data
  for comp in comps {
    let precision = comp.comp.prec;
    let mask = (1 << precision) - 1;
    let signed = comp.comp.sgnd != 0;

    match precision {
      p if p <= 8 => {
        let (min, max) = if signed {
          (i8::MIN as i32, i8::MAX as i32)
        } else {
          (0, 255)
        };
        // Write 8-bit values
        for &value in comp.data.iter() {
          let value = (value.clamp(min, max) & mask) as u8;
          writer.write_all(&[value])?;
        }
      }
      p if p <= 16 => {
        let (min, max) = if signed {
          (i16::MIN as i32, i16::MAX as i32)
        } else {
          (0, 65535)
        };
        // Write 16-bit values
        for &value in comp.data.iter() {
          let value = (value.clamp(min, max) & mask) as u16;

          let bytes = if big_endian {
            value.to_be_bytes()
          } else {
            value.to_le_bytes()
          };
          writer.write_all(&bytes)?;
        }
      }
      p if p <= 32 => {
        // Write 32-bit values
        for &value in comp.data.iter() {
          let bytes = if big_endian {
            value.to_be_bytes()
          } else {
            value.to_le_bytes()
          };
          writer.write_all(&bytes)?;
        }
      }
      _ => {
        return Err(ImageError::InvalidFormat(format!(
          "Unsupported bit depth: {}",
          precision
        )));
      }
    }
  }

  // Ensure all data is flushed
  writer.flush()?;
  Ok(())
}

pub fn convert_to_dynamic_image(image: &opj_image) -> Result<DynamicImage, ImageError> {
  let mut comps = image
    .comps_data_iter()
    .ok_or_else(|| ImageError::InvalidFormat("Missing components".into()))?;

  // Must have at least one component
  let c0 = comps
    .next()
    .ok_or_else(|| ImageError::InvalidFormat("Missing components".into()))?;
  let c1 = comps.next();
  let c2 = comps.next();
  let c3 = comps.next();

  // Only support 1-4 component images
  if comps.next().is_some() {
    return Err(ImageError::InvalidFormat(
      "Unsupported number of components".into(),
    ));
  }

  // The components must have matching parameters.
  if !image.comps_match() {
    return Err(ImageError::InvalidFormat(
      "Mismatched component parameters".into(),
    ));
  }

  let width = c0.comp.w;
  let height = c0.comp.h;
  let adjust = c0.adjust;
  // Convert to DynamicImage based on components
  let dynamic_img = match (c0, c1, c2, c3, image.color_space) {
    (c0, None, None, None, OPJ_CLRSPC_GRAY) => {
      // Grayscale image

      let pixels = c0.data.iter().map(|&x| x + adjust);
      if c0.comp.prec <= 8 {
        // Convert to ImageLuma8
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|x| x as u8).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageLuma8(img_buf)
      } else {
        // Convert to ImageLuma16
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|x| x as u16).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageLuma16(img_buf)
      }
    }
    (gray, Some(alpha), None, None, OPJ_CLRSPC_GRAY) => {
      // Grayscale with alpha

      let pixels = gray
        .data
        .iter()
        .zip(alpha.data.iter())
        .map(|(g, a)| [g + adjust, a + adjust])
        .flatten();
      if gray.comp.prec <= 8 {
        // Convert to ImageLumaA8
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|p| p as u8).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageLumaA8(img_buf)
      } else {
        // Convert to ImageLumaA16
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|p| p as u16).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageLumaA16(img_buf)
      }
    }
    (r, Some(g), Some(b), None, OPJ_CLRSPC_SRGB | OPJ_CLRSPC_SYCC) => {
      // RGB image

      let pixels = r
        .data
        .iter()
        .zip(g.data.iter())
        .zip(b.data.iter())
        .map(|((r, g), b)| [r + adjust, g + adjust, b + adjust])
        .flatten();
      if r.comp.prec <= 8 {
        // Convert to ImageRgb8
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|p| p as u8).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageRgb8(img_buf)
      } else {
        // Convert to ImageRgb16
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|p| p as u16).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageRgb16(img_buf)
      }
    }
    (r, Some(g), Some(b), Some(a), OPJ_CLRSPC_SRGB | OPJ_CLRSPC_SYCC) => {
      // RGBA image

      let pixels = r
        .data
        .iter()
        .zip(g.data.iter())
        .zip(b.data.iter())
        .zip(a.data.iter())
        .map(|(((r, g), b), a)| [r + adjust, g + adjust, b + adjust, a + adjust])
        .flatten();
      if r.comp.prec <= 8 {
        // Convert to ImageRgba8
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|p| p as u8).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageRgba8(img_buf)
      } else {
        // Convert to ImageRgba16
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|p| p as u16).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageRgba16(img_buf)
      }
    }
    _ => {
      return Err(ImageError::InvalidFormat(format!(
        "Unsupported image format: {} components, colorspace {:?}",
        image.numcomps, image.color_space
      )))
    }
  };

  Ok(dynamic_img)
}
