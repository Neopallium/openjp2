use super::convert_from_dynamic_image;
use super::ImageError;
use crate::compress::CompressionParameters;
use openjp2::image::opj_image;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn load_bmp_image(
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
