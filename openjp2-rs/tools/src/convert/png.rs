use super::ImageError;
use crate::convert::*;
use crate::params::CompressionParameters;
use openjp2::image::opj_image;
use std::fs::File;
use std::io::{self, Read, Seek};
use std::path::Path;

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

pub fn save_png_image(image: &mut opj_image, path: &Path) -> Result<(), ImageError> {
  let prec = {
    let comps = image
      .comps_mut()
      .ok_or_else(|| ImageError::EncodeError("Failed to get image components".into()))?;
    let numcomps = comps.len();
    if numcomps == 0 {
      return Err(ImageError::EncodeError("No components found".into()));
    }

    let prec = comps[0].prec;

    // Clip components.
    for comp in comps.iter_mut() {
      comp.clip(prec);
    }

    // Scale components.
    if prec > 8 && prec < 16 {
      for comp in comps {
        comp.scale(16);
      }
      16
    } else if prec < 8 && numcomps > 1 {
      for comp in comps {
        comp.scale(8);
      }
      8
    } else if prec > 1 && prec < 8 && (prec == 6 || (prec & 1) == 1) {
      let prec = match prec {
        5 | 6 => 8,
        _ => prec + 1,
      };
      for comp in comps {
        comp.scale(prec);
      }
      prec
    } else {
      prec
    }
  };

  #[cfg(feature = "libpng")]
  libpng_save(image, prec, path)?;

  #[cfg(feature = "lodepng")]
  lodepng_save(image, prec, path)?;

  #[cfg(not(any(feature = "testing", feature = "lodepng")))]
  image_crate_save(image, prec, path)?;

  Ok(())
}

#[cfg(feature = "libpng")]
fn libpng_save(image: &mut opj_image, prec: u32, path: &Path) -> Result<(), ImageError> {
  use super::BitBuffer;
  use libpng_sys::ffi::*;
  use std::ffi::CString;
  use std::os::raw::c_char;

  // Open png file
  let path_str = path
    .to_str()
    .ok_or_else(|| ImageError::InvalidFormat("Invalid path encoding".into()))?;
  let c_path = CString::new(path_str).unwrap();
  let file = unsafe { libc::fopen(c_path.as_ptr(), b"w\0".as_ptr() as *const c_char) };
  if file.is_null() {
    return Err(ImageError::EncodeError(format!(
      "Failed to create PNG file: {}",
      path_str
    )));
  }

  // Create PNG struct
  let png =
    unsafe { png_create_write_struct(PNG_LIBPNG_VER_STRING, std::ptr::null_mut(), None, None) };
  if png.is_null() {
    return Err(ImageError::EncodeError(
      "Failed to create PNG struct".into(),
    ));
  }
  let png = unsafe { &mut *png };

  // Create PNG info struct
  let info = unsafe { png_create_info_struct(png) };
  if info.is_null() {
    return Err(ImageError::EncodeError(
      "Failed to create PNG info struct".into(),
    ));
  }
  let info = unsafe { &mut *info };

  // Set error handling
  unsafe { png_set_error_fn(png, std::ptr::null_mut(), None, None) };

  // I/O initialization
  unsafe { png_init_io(png, file) };

  // Set compression parameters
  unsafe { png_set_compression_level(png, 9) };

  // Verify all components have matching parameters
  if !image.comps_match() {
    return Err(ImageError::EncodeError(
      "Components must have matching parameters".into(),
    ));
  }

  let numcomps = image.numcomps;
  let mut comps = image
    .comps_data_iter()
    .ok_or_else(|| ImageError::InvalidFormat("Missing components".into()))?;

  // Must have at least one component.
  // Only 1-4 components are supported.  Any additional components are ignored.
  let c0 = comps
    .next()
    .ok_or_else(|| ImageError::InvalidFormat("Missing components".into()))?;
  let c1 = comps.next();
  let c2 = comps.next();
  let c3 = comps.next();

  let width = c0.comp.w as usize;
  let height = c0.comp.h as usize;
  let sgnd = c0.comp.sgnd;
  let adjust = c0.adjust;
  log::debug!(
    "libpng_save: width: {}, height: {}, prec: {}, sgnd: {}, adjust: {}, numcomps: {}",
    width,
    height,
    prec,
    sgnd,
    adjust,
    numcomps
  );

  // Only support 1-4 components
  let numcomps = numcomps.min(4) as usize;

  let color_type = match numcomps {
    1 => PNG_COLOR_TYPE_GRAY,
    2 => PNG_COLOR_TYPE_GRAY_ALPHA,
    3 => PNG_COLOR_TYPE_RGB,
    4 => PNG_COLOR_TYPE_RGBA,
    _ => PNG_COLOR_TYPE_RGBA,
  };
  let is_gray = color_type == PNG_COLOR_TYPE_GRAY || color_type == PNG_COLOR_TYPE_GRAY_ALPHA;
  let bit_depth = match (is_gray, prec) {
    (true, 1 | 2 | 4 | 8 | 16) => prec as u8,
    (true, 3) => 4,
    (true, 5 | 6) => 8,
    (true, prec) if prec < 16 => 16,
    (_, 8 | 16) => prec as u8,
    (_, prec) if prec < 8 => 8,
    (_, prec) if prec < 16 => 16,
    _ => {
      log::error!(
        "Unsupported color_type {:?} with precision: {}",
        color_type,
        prec
      );
      return Err(ImageError::EncodeError("Unsupported color type".into()));
    }
  };

  // Set image header
  unsafe {
    png_set_IHDR(
      png,
      info,
      width as u32,
      height as u32,
      bit_depth as i32,
      color_type as i32,
      PNG_INTERLACE_NONE as i32,
      PNG_COMPRESSION_TYPE_BASE as i32,
      PNG_FILTER_TYPE_BASE as i32,
    );
  }

  // Set sBIT chunk
  let mut bits = png_color_8 {
    red: 0,
    green: 0,
    blue: 0,
    gray: 0,
    alpha: 0,
  };
  match color_type {
    PNG_COLOR_TYPE_GRAY => {
      bits.gray = bit_depth;
    }
    PNG_COLOR_TYPE_GRAY_ALPHA => {
      bits.gray = bit_depth;
      bits.alpha = bit_depth;
    }
    PNG_COLOR_TYPE_RGB => {
      bits.red = bit_depth;
      bits.green = bit_depth;
      bits.blue = bit_depth;
    }
    PNG_COLOR_TYPE_RGBA => {
      bits.red = bit_depth;
      bits.green = bit_depth;
      bits.blue = bit_depth;
      bits.alpha = bit_depth;
    }
    _ => {
      return Err(ImageError::EncodeError("Invalid color type".into()));
    }
  }
  unsafe { png_set_sBIT(png, info, &bits) };

  // write header
  unsafe { png_write_info(png, info) };

  // setup write buffer
  let row_stride = (width * numcomps * bit_depth as usize + 7) / 8;
  let png_row_size = unsafe { png_get_rowbytes(png, info) as usize };
  if row_stride != png_row_size {
    return Err(ImageError::EncodeError("Invalid row stride".into()));
  }
  let mut row = vec![0u8; row_stride];

  if bit_depth <= 8 {
    let adjust = |d| (d - adjust) as u8;
    let d0 = c0.data.iter().map(adjust);
    let d1 = c1.map(|c| c.data.iter().map(adjust));
    let d2 = c2.map(|c| c.data.iter().map(adjust));
    let d3 = c3.map(|c| c.data.iter().map(adjust));

    match (d0, d1, d2, d3) {
      (d0, None, None, None) => {
        let buffer = d0.map(|d| d as u8).collect::<Vec<_>>();
        if prec < 8 {
          let mut row = BitBuffer::new(row_stride);
          for row_pixels in buffer.chunks_exact(width) {
            for pixel in row_pixels {
              row.write(prec, *pixel as u32);
            }
            unsafe { png_write_row(png, row.buffer.as_ptr()) };
            row.reset();
          }
        } else {
          for row in buffer.chunks_exact(width) {
            unsafe { png_write_row(png, row.as_ptr()) };
          }
        }
      }
      (d0, Some(d1), None, None) => {
        let pixels = d0.zip(d1);
        let mut idx = 0;
        for (gray, alpha) in pixels {
          row[idx] = gray;
          row[idx + 1] = alpha;
          idx += 2;
          if idx == row_stride {
            unsafe { png_write_row(png, row.as_ptr()) };
            idx = 0;
          }
        }
      }
      (d0, Some(d1), Some(d2), None) => {
        let pixels = d0.zip(d1).zip(d2);
        let mut idx = 0;
        for ((r, g), b) in pixels {
          row[idx] = r;
          row[idx + 1] = g;
          row[idx + 2] = b;
          idx += 3;
          if idx == row_stride {
            unsafe { png_write_row(png, row.as_ptr()) };
            idx = 0;
          }
        }
      }
      (d0, Some(d1), Some(d2), Some(d3)) => {
        let pixels = d0.zip(d1).zip(d2).zip(d3);
        let mut idx = 0;
        for (((r, g), b), a) in pixels {
          row[idx] = r;
          row[idx + 1] = g;
          row[idx + 2] = b;
          row[idx + 3] = a;
          idx += 4;
          if idx == row_stride {
            unsafe { png_write_row(png, row.as_ptr()) };
            idx = 0;
          }
        }
      }
      _ => {
        return Err(ImageError::EncodeError(
          "Invalid number of components".into(),
        ));
      }
    }
  } else {
    let adjust = |d| (d - adjust) as u16;
    let d0 = c0.data.iter().map(adjust);
    let d1 = c1.map(|c| c.data.iter().map(adjust));
    let d2 = c2.map(|c| c.data.iter().map(adjust));
    let d3 = c3.map(|c| c.data.iter().map(adjust));

    match (d0, d1, d2, d3) {
      (d0, None, None, None) => {
        let mut idx = 0;
        for gray in d0 {
          row[idx] = (gray >> 8) as u8;
          row[idx + 1] = gray as u8;
          idx += 2;
          if idx == row_stride {
            unsafe { png_write_row(png, row.as_ptr()) };
            idx = 0;
          }
        }
      }
      (d0, Some(d1), None, None) => {
        let pixels = d0.zip(d1);
        let mut idx = 0;
        for (gray, alpha) in pixels {
          row[idx] = (gray >> 8) as u8;
          row[idx + 1] = gray as u8;
          row[idx + 2] = (alpha >> 8) as u8;
          row[idx + 3] = alpha as u8;
          idx += 4;
          if idx == row_stride {
            unsafe { png_write_row(png, row.as_ptr()) };
            idx = 0;
          }
        }
      }
      (d0, Some(d1), Some(d2), None) => {
        let pixels = d0.zip(d1).zip(d2);
        let mut idx = 0;
        for ((r, g), b) in pixels {
          row[idx] = (r >> 8) as u8;
          row[idx + 1] = r as u8;
          row[idx + 2] = (g >> 8) as u8;
          row[idx + 3] = g as u8;
          row[idx + 4] = (b >> 8) as u8;
          row[idx + 5] = b as u8;
          idx += 6;
          if idx == row_stride {
            unsafe { png_write_row(png, row.as_ptr()) };
            idx = 0;
          }
        }
      }
      (d0, Some(d1), Some(d2), Some(d3)) => {
        let pixels = d0.zip(d1).zip(d2).zip(d3);
        let mut idx = 0;
        for (((r, g), b), a) in pixels {
          row[idx] = (r >> 8) as u8;
          row[idx + 1] = r as u8;
          row[idx + 2] = (g >> 8) as u8;
          row[idx + 3] = g as u8;
          row[idx + 4] = (b >> 8) as u8;
          row[idx + 5] = b as u8;
          row[idx + 6] = (a >> 8) as u8;
          row[idx + 7] = a as u8;
          idx += 8;
          if idx == row_stride {
            unsafe { png_write_row(png, row.as_ptr()) };
            idx = 0;
          }
        }
      }
      _ => {
        return Err(ImageError::EncodeError(
          "Invalid number of components".into(),
        ));
      }
    }
  }

  // write end
  unsafe { png_write_end(png, info) };

  Ok(())
}

#[cfg(feature = "lodepng")]
fn lodepng_save(image: &mut opj_image, prec: u32, path: &Path) -> Result<(), ImageError> {
  use lodepng::{ChunkPosition, ColorType, FilterStrategy, GreyAlpha};
  use rgb::{Rgb, Rgba};

  // Verify all components have matching parameters
  if !image.comps_match() {
    return Err(ImageError::EncodeError(
      "Components must have matching parameters".into(),
    ));
  }

  let numcomps = image.numcomps;
  let mut comps = image
    .comps_data_iter()
    .ok_or_else(|| ImageError::InvalidFormat("Missing components".into()))?;

  // Must have at least one component.
  // Only 1-4 components are supported.  Any additional components are ignored.
  let c0 = comps
    .next()
    .ok_or_else(|| ImageError::InvalidFormat("Missing components".into()))?;
  let c1 = comps.next();
  let c2 = comps.next();
  let c3 = comps.next();

  let width = c0.comp.w as usize;
  let height = c0.comp.h as usize;
  let sgnd = c0.comp.sgnd;
  let adjust = c0.adjust;
  log::debug!(
    "lodepng_save: width: {}, height: {}, prec: {}, sgnd: {}, adjust: {}, numcomps: {}",
    width,
    height,
    prec,
    sgnd,
    adjust,
    numcomps
  );

  let color_type = match numcomps {
    1 => ColorType::GREY,
    2 => ColorType::GREY_ALPHA,
    3 => ColorType::RGB,
    4 => ColorType::RGBA,
    _ => ColorType::RGBA,
  };
  let bit_depth = match (color_type, prec) {
    (ColorType::GREY, 1 | 2 | 4 | 8 | 16) => prec,
    (ColorType::GREY, 3) => 4,
    (ColorType::GREY, 5 | 6) => 8,
    (ColorType::GREY, prec) if prec < 16 => 16,
    (_, 8 | 16) => prec,
    (_, prec) if prec < 8 => 8,
    (_, prec) if prec < 16 => 16,
    _ => {
      log::error!(
        "Unsupported color_type {:?} with precision: {}",
        color_type,
        prec
      );
      return Err(ImageError::EncodeError("Unsupported color type".into()));
    }
  };

  let bits = vec![bit_depth as u8; numcomps.min(4) as usize];

  // Create encoder with specified color type and bit depth
  let mut encoder = lodepng::Encoder::new();
  {
    let color_mode = encoder.info_raw_mut();
    color_mode.colortype = color_type;
    color_mode
      .try_set_bitdepth(bit_depth)
      .map_err(|e| ImageError::EncodeError(e.to_string()))?;
    let color = color_mode.clone();
    let info = encoder.info_png_mut();
    info.color = color;
    info
      .create_chunk(ChunkPosition::IHDR, b"sBIT", &bits)
      .map_err(|e| ImageError::EncodeError(e.to_string()))?;

    let settings = encoder.settings_mut();
    settings.set_level(9); // 9=Best
    settings.auto_convert = false;
    settings.filter_strategy = FilterStrategy::ZERO;
  }

  if bit_depth <= 8 {
    let adjust = |d| (d - adjust) as u8;
    let d0 = c0.data.iter().map(adjust);
    let d1 = c1.map(|c| c.data.iter().map(adjust));
    let d2 = c2.map(|c| c.data.iter().map(adjust));
    let d3 = c3.map(|c| c.data.iter().map(adjust));

    match (d0, d1, d2, d3) {
      (d0, None, None, None) => {
        let buffer = d0.map(|d| d as u8).collect::<Vec<_>>();
        encoder
          .encode_file(path, &buffer, width, height)
          .map_err(|e| ImageError::EncodeError(e.to_string()))?;
      }
      (d0, Some(d1), None, None) => {
        let pixels = d0.zip(d1);
        let buffer = pixels
          .map(|(d0, d1)| GreyAlpha(d0 as u8, d1 as u8))
          .collect::<Vec<_>>();
        encoder
          .encode_file(path, &buffer, width, height)
          .map_err(|e| ImageError::EncodeError(e.to_string()))?;
      }
      (d0, Some(d1), Some(d2), None) => {
        let pixels = d0.zip(d1).zip(d2);
        let buffer = pixels
          .map(|((r, g), b)| Rgb { r, g, b })
          .collect::<Vec<_>>();
        encoder
          .encode_file(path, &buffer, width, height)
          .map_err(|e| ImageError::EncodeError(e.to_string()))?;
      }
      (d0, Some(d1), Some(d2), Some(d3)) => {
        let pixels = d0.zip(d1).zip(d2).zip(d3);
        let buffer = pixels
          .map(|(((r, g), b), a)| Rgba { r, g, b, a })
          .collect::<Vec<_>>();
        encoder
          .encode_file(path, &buffer, width, height)
          .map_err(|e| ImageError::EncodeError(e.to_string()))?;
      }
      _ => {
        return Err(ImageError::EncodeError(
          "Invalid number of components".into(),
        ));
      }
    }
  } else {
    let adjust = |d| (d - adjust) as u16;
    let d0 = c0.data.iter().map(adjust);
    let d1 = c1.map(|c| c.data.iter().map(adjust));
    let d2 = c2.map(|c| c.data.iter().map(adjust));
    let d3 = c3.map(|c| c.data.iter().map(adjust));

    match (d0, d1, d2, d3) {
      (d0, None, None, None) => {
        let buffer = d0.collect::<Vec<_>>();
        encoder
          .encode_file(path, &buffer, width, height)
          .map_err(|e| ImageError::EncodeError(e.to_string()))?;
      }
      (d0, Some(d1), None, None) => {
        let pixels = d0.zip(d1);
        let buffer = pixels.map(|(d0, d1)| GreyAlpha(d0, d1)).collect::<Vec<_>>();
        encoder
          .encode_file(path, &buffer, width, height)
          .map_err(|e| ImageError::EncodeError(e.to_string()))?;
      }
      (d0, Some(d1), Some(d2), None) => {
        let pixels = d0.zip(d1).zip(d2);
        let buffer = pixels
          .map(|((r, g), b)| Rgb { r, g, b })
          .collect::<Vec<_>>();
        encoder
          .encode_file(path, &buffer, width, height)
          .map_err(|e| ImageError::EncodeError(e.to_string()))?;
      }
      (d0, Some(d1), Some(d2), Some(d3)) => {
        let pixels = d0.zip(d1).zip(d2).zip(d3);
        let buffer = pixels
          .map(|(((r, g), b), a)| Rgba { r, g, b, a })
          .collect::<Vec<_>>();
        encoder
          .encode_file(path, &buffer, width, height)
          .map_err(|e| ImageError::EncodeError(e.to_string()))?;
      }
      _ => {
        return Err(ImageError::EncodeError(
          "Invalid number of components".into(),
        ));
      }
    }
  }

  Ok(())
}

#[cfg(not(any(feature = "testing", feature = "lodepng")))]
fn image_crate_save(image: &mut opj_image, _prec: u32, path: &Path) -> Result<(), ImageError> {
  use image::codecs::png::{CompressionType, FilterType, PngEncoder};

  let dynamic_img = convert_to_dynamic_image(image)?;
  let file = File::create(path).map_err(|e| ImageError::EncodeError(e.to_string()))?;
  let encoder = PngEncoder::new_with_quality(file, CompressionType::Best, FilterType::NoFilter);
  dynamic_img
    .write_with_encoder(encoder)
    .map_err(|e| ImageError::EncodeError(e.to_string()))
}
