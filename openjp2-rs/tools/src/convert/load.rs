use super::ImageError;
use crate::compress::{CompressionParameters, ImageFileFormat};
use image::{self, DynamicImage};
use openjp2::{image::opj_image, openjpeg::*};
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::Path;

mod raw;
pub use raw::*;

// Replace existing load_image function
pub fn load_image(
  path: &Path,
  params: &CompressionParameters,
) -> Result<Box<opj_image>, ImageError> {
  match params.decode_format {
    Some(ImageFileFormat::RAW) => load_raw_image(path, params, true),
    Some(ImageFileFormat::RAWL) => load_raw_image(path, params, false),
    Some(ImageFileFormat::BMP) => load_bmp_image(path, params),
    _ => {
      let image = read_image(path)?;
      convert_from_dynamic_image(image, params)
    }
  }
}

fn read_image(path: &Path) -> Result<DynamicImage, ImageError> {
  Ok(image::open(path).map_err(|e| ImageError::ReadError(e.to_string()))?)
}

fn load_bmp_image(
  path: &Path,
  params: &CompressionParameters,
) -> Result<Box<opj_image>, ImageError> {
  // Read BMP file into memory
  let mut f = File::open(path)?;
  let mut buf = Vec::new();
  f.read_to_end(&mut buf)?;

  // Parse BMP file to get header and color table.
  let info = match parse_bmp_header(&buf) {
    Ok(info) => info,
    Err(e) => {
      eprintln!("Failed to parse BMP header of {:?}: {e:?}", path);
      Default::default()
    }
  };

  // Decode BMP image.
  let mut img = image::open(path).map_err(|e| ImageError::ReadError(e.to_string()))?;

  // Convert to grayscale if needed
  if info.grayscale {
    img = img.grayscale();
  }

  let mut img = convert_from_dynamic_image(img, params)?;

  // Set precision based on channel bitmasks from BMP header.
  if let Some(prec) = info.precisions {
    match img.comps_mut() {
      Some([gray]) => {
        gray.prec = prec.red;
      }
      Some([gray, a]) => {
        gray.prec = prec.red;
        a.prec = prec.alpha;
      }
      Some([r, g, b]) => {
        r.prec = prec.red;
        g.prec = prec.green;
        b.prec = prec.blue;
      }
      Some([r, g, b, a]) => {
        r.prec = prec.red;
        g.prec = prec.green;
        b.prec = prec.blue;
        a.prec = prec.alpha;
      }
      _ => (),
    }
  }

  Ok(img)
}

fn convert_from_dynamic_image(
  in_img: DynamicImage,
  params: &CompressionParameters,
) -> Result<Box<opj_image>, ImageError> {
  //eprintln!("-- input image={:?}", in_img);
  // Get information from input image.
  let (width, height, numcomps, color_space, bit_depth, sgnd, alpha) = match &in_img {
    DynamicImage::ImageLuma8(img) => {
      let (width, height) = img.dimensions();
      (width, height, 1, OPJ_CLRSPC_GRAY, 8, false, None)
    }
    DynamicImage::ImageLumaA8(img) => {
      let (width, height) = img.dimensions();
      (width, height, 2, OPJ_CLRSPC_GRAY, 8, false, Some(1))
    }
    DynamicImage::ImageRgb8(img) => {
      let (width, height) = img.dimensions();
      (width, height, 3, OPJ_CLRSPC_SRGB, 8, false, None)
    }
    DynamicImage::ImageRgba8(img) => {
      let (width, height) = img.dimensions();
      (width, height, 4, OPJ_CLRSPC_SRGB, 8, false, Some(3))
    }
    DynamicImage::ImageRgb16(img) => {
      let (width, height) = img.dimensions();
      (width, height, 3, OPJ_CLRSPC_SRGB, 16, false, None)
    }
    DynamicImage::ImageRgba16(img) => {
      let (width, height) = img.dimensions();
      (width, height, 4, OPJ_CLRSPC_SRGB, 16, false, Some(3))
    }
    DynamicImage::ImageLuma16(img) => {
      let (width, height) = img.dimensions();
      (width, height, 1, OPJ_CLRSPC_GRAY, 16, false, None)
    }
    DynamicImage::ImageLumaA16(img) => {
      let (width, height) = img.dimensions();
      (width, height, 2, OPJ_CLRSPC_GRAY, 16, false, Some(1))
    }
    DynamicImage::ImageRgb32F(img) => {
      let (width, height) = img.dimensions();
      (width, height, 3, OPJ_CLRSPC_SRGB, 32, true, None)
    }
    DynamicImage::ImageRgba32F(img) => {
      let (width, height) = img.dimensions();
      (width, height, 4, OPJ_CLRSPC_SRGB, 32, true, Some(3))
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
  for (i, comp) in comps.iter_mut().enumerate() {
    comp.dx = subsampling.width;
    comp.dy = subsampling.height;
    comp.w = width;
    comp.h = height;
    comp.x0 = 0;
    comp.y0 = 0;
    comp.prec = bit_depth;
    comp.sgnd = sgnd as u32;
    comp.alpha = (Some(i as u32) == alpha) as u16;
    if !comp.alloc_data() {
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

#[derive(Debug, Default)]
struct BmpBitPrecision {
  red: u32,
  green: u32,
  blue: u32,
  alpha: u32,
}

#[derive(Debug, Default)]
struct BmpInfo {
  precisions: Option<BmpBitPrecision>,
  grayscale: bool,
}

fn parse_bmp_header(data: &[u8]) -> Result<BmpInfo, ImageError> {
  if data.len() < 54 {
    return Err(ImageError::InvalidFormat("BMP header too short".into()));
  }

  // Check BMP signature
  if &data[0..2] != b"BM" {
    return Err(ImageError::InvalidFormat("Not a BMP file".into()));
  }

  // Read header size to determine format
  let header_size = u32::from_le_bytes(data[14..18].try_into().unwrap());
  let bits_per_pixel = u16::from_le_bytes(data[28..30].try_into().unwrap());
  let compression = u32::from_le_bytes(data[30..34].try_into().unwrap());
  let clr_used = u32::from_le_bytes(data[46..50].try_into().unwrap());

  let mut precisions = None;
  let mut grayscale = false;

  // If header size is at least 52 bytes, read RGB masks.
  if header_size >= 52 {
    // Read masks
    let red_mask = u32::from_le_bytes(data[54..58].try_into().unwrap());
    let green_mask = u32::from_le_bytes(data[58..62].try_into().unwrap());
    let blue_mask = u32::from_le_bytes(data[62..66].try_into().unwrap());
    let alpha_mask = if header_size >= 56 {
      // If header size is at least 56 bytes, read alpha mask
      u32::from_le_bytes(data[66..70].try_into().unwrap())
    } else {
      0
    };

    // Convert masks to bit precisions
    let red = red_mask.count_ones();
    let green = green_mask.count_ones();
    let blue = blue_mask.count_ones();
    let alpha = alpha_mask.count_ones();
    if red + green + blue + alpha > 0 {
      precisions = Some(BmpBitPrecision {
        red,
        green,
        blue,
        alpha,
      });
    }
  }

  // For compressions BI_BITFIELDS or BI_ALPHABITFIELDS, check masks
  if compression == 3 || compression == 6 {
    // If no masks use default precisions based on bits per pixel.
    if precisions.is_none() {
      match bits_per_pixel {
        16 => {
          precisions = Some(BmpBitPrecision {
            red: 5,
            green: 6,
            blue: 5,
            alpha: 0,
          });
        }
        24 => {
          precisions = Some(BmpBitPrecision {
            red: 8,
            green: 8,
            blue: 8,
            alpha: 0,
          });
        }
        32 => {
          precisions = Some(BmpBitPrecision {
            red: 8,
            green: 8,
            blue: 8,
            alpha: if compression == 6 { 8 } else { 0 },
          });
        }
        _ => (),
      }
    }
  }

  // Check if image has color table and if it's grayscale
  if bits_per_pixel <= 8 {
    let num_colors = if clr_used > 0 {
      clr_used
    } else {
      1u32 << bits_per_pixel
    };

    let color_table_offset = 14 + header_size as usize;
    if data.len() >= color_table_offset + (num_colors as usize * 4) {
      grayscale = true;

      // Check each color table entry
      for i in 0..num_colors as usize {
        let offset = color_table_offset + (i * 4);
        let b = data[offset];
        let g = data[offset + 1];
        let r = data[offset + 2];
        // If R, G, B components differ, it's not grayscale
        if r != g || g != b {
          grayscale = false;
          break;
        }
      }
    } else {
      // For 1, 4, or 8 bits without color table, assume grayscale
      grayscale = true;
    }
  }

  Ok(BmpInfo {
    precisions,
    grayscale,
  })
}
