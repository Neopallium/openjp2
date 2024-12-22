use super::ImageError;
use crate::params::{CompressionParameters, ImageFileFormat};
use image::{self, DynamicImage};
use openjp2::{image::opj_image, openjpeg::*};
use std::fs::File;
use std::io::{self, Read, Seek};
use std::path::Path;

mod raw;
pub use raw::*;
mod bmp;
pub use bmp::*;

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

pub fn load_png_image(
  path: &Path,
  params: &CompressionParameters,
) -> Result<Box<opj_image>, ImageError> {
  let image = read_image(path)?;
  let mut img = convert_from_dynamic_image(image, params)?;

  // Read PNG header to determine bit depth and color type
  let (bit_depth, color_type) = parse_png_header(path)?;

  // Set precision based on bit depth and color type
  if color_type == 0 && bit_depth < 8 {
    // Grayscale with bit depth < 8
    img
      .comps_mut()
      .expect("We just allocated them")
      .iter_mut()
      .for_each(|comp| {
        comp.prec = bit_depth as u32;
      });
  }

  Ok(img)
}

pub fn read_image(path: &Path) -> Result<DynamicImage, ImageError> {
  Ok(image::open(path).map_err(|e| ImageError::ReadError(e.to_string()))?)
}

fn parse_png_header(path: &Path) -> Result<(u8, u8), ImageError> {
  let mut file = File::open(path).map_err(|e| ImageError::ReadError(e.to_string()))?;
  let mut header = [0u8; 29]; // PNG signature (8) + IHDR length (4) + "IHDR" (4) + IHDR data (13)

  file
    .read_exact(&mut header)
    .map_err(|e| ImageError::ReadError(e.to_string()))?;

  // Verify PNG signature
  if &header[0..8] != [137, 80, 78, 71, 13, 10, 26, 10] {
    return Err(ImageError::InvalidFormat("Not a valid PNG file".into()));
  }

  // IHDR chunk starts at offset 8
  // Verify IHDR chunk type
  if &header[12..16] != b"IHDR" {
    return Err(ImageError::InvalidFormat("Invalid PNG header".into()));
  }

  // Extract bit depth and color type from IHDR
  let mut bit_depth = header[24]; // 8th byte of IHDR data
  let color_type = header[25]; // 9th byte of IHDR data

  // Skip IHDR CRC
  file
    .seek(io::SeekFrom::Current(4))
    .map_err(|e| ImageError::ReadError(e.to_string()))?;

  // Read chunks until we find tRNS or IDAT
  let mut chunk_length = [0u8; 4];
  let mut chunk_type = [0u8; 4];

  loop {
    // Read chunk length
    if file.read_exact(&mut chunk_length).is_err() {
      break;
    }
    let length = u32::from_be_bytes(chunk_length);

    // Read chunk type
    if file.read_exact(&mut chunk_type).is_err() {
      break;
    }

    if &chunk_type == b"IDAT" {
      break;
    }

    if &chunk_type == b"tRNS" {
      // If we find tRNS chunk, force bit depth to 8
      bit_depth = 8;
      break;
    }

    // Skip chunk data and CRC
    file
      .seek(io::SeekFrom::Current(length as i64 + 4))
      .map_err(|e| ImageError::ReadError(e.to_string()))?;
  }

  Ok((bit_depth, color_type))
}

pub fn convert_from_dynamic_image(
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
