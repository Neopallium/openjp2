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

  #[cfg(feature = "lodepng")]
  lodepng_save(image, prec, path)?;

  #[cfg(not(any(feature = "testing", feature = "lodepng")))]
  image_crate_save(image, prec, path)?;

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
    eprintln!("bit_depth <= 8");
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
    eprintln!("prec > 8: encode as 16 bit");
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
