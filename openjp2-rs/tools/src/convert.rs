use crate::compress::CompressionParameters;
use image::{self, DynamicImage};
use openjp2::{image::opj_image, openjpeg::*};
use std::io;
use std::path::Path;

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
  let img = match params.decode_format {
    // TODO: handle raw
    //DecodeFormat::RAW | DecodeFormat::RAWL => load_raw_image(path, params)?,
    _ => load_regular_image(path)?,
  };

  // Convert the loaded image to OpenJPEG format
  convert_to_opj_image(img, params)
}

fn load_regular_image(path: &Path) -> Result<Vec<ImageComponent>, ImageError> {
  let img = image::open(path).map_err(|e| ImageError::ReadError(e.to_string()))?;

  match img {
    DynamicImage::ImageRgb8(img) => {
      let (width, height) = img.dimensions();
      let mut components = Vec::new();

      // Extract R, G, B components
      for c in 0..3 {
        let mut data = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
          for x in 0..width {
            let pixel = img.get_pixel(x, y);
            data.push(pixel[c] as i32);
          }
        }

        components.push(ImageComponent {
          data,
          width,
          height,
          precision: 8,
          signed: false,
          dx: 1,
          dy: 1,
        });
      }

      Ok(components)
    }
    DynamicImage::ImageLuma8(img) => {
      let (width, height) = img.dimensions();
      let mut data = Vec::with_capacity((width * height) as usize);

      for y in 0..height {
        for x in 0..width {
          let pixel = img.get_pixel(x, y);
          data.push(pixel[0] as i32);
        }
      }

      Ok(vec![ImageComponent {
        data,
        width,
        height,
        precision: 8,
        signed: false,
        dx: 1,
        dy: 1,
      }])
    }
    _ => Err(ImageError::InvalidFormat(
      "Unsupported image format - convert to RGB8 or Luma8 first".into(),
    )),
  }
}

fn convert_to_opj_image(
  components: Vec<ImageComponent>,
  _params: &CompressionParameters,
) -> Result<Box<opj_image>, ImageError> {
  if components.is_empty() {
    return Err(ImageError::InvalidFormat("No image components".into()));
  }

  let reference = &components[0];
  let mut image = opj_image::new();

  image.x0 = 0;
  image.y0 = 0;
  image.x1 = reference.width;
  image.y1 = reference.height;
  image.numcomps = components.len() as u32;
  image.color_space = if components.len() >= 3 {
    OPJ_CLRSPC_SRGB
  } else {
    OPJ_CLRSPC_GRAY
  };
  image.alloc_comps(image.numcomps, false);

  let comps = image.comps_mut().expect("We just allocated them");

  for (i, comp) in components.iter().enumerate() {
    let c = &mut comps[i];
    c.dx = comp.dx;
    c.dy = comp.dy;
    c.w = comp.width;
    c.h = comp.height;
    c.x0 = 0;
    c.y0 = 0;
    c.prec = comp.precision;
    c.bpp = comp.precision;
    c.sgnd = comp.signed as u32;

    let data_size = (comp.width * comp.height) as usize;
    let data = unsafe {
      std::slice::from_raw_parts_mut(
        std::alloc::alloc(std::alloc::Layout::array::<i32>(data_size).unwrap()) as *mut i32,
        data_size,
      )
    };
    data.copy_from_slice(&comp.data);
    c.data = data.as_mut_ptr();
  }

  image.comps = comps.as_mut_ptr();

  Ok(image)
}
