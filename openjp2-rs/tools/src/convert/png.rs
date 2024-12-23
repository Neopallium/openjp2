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
