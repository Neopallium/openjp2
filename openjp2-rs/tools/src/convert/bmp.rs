use super::convert_from_dynamic_image;
use super::ImageError;
use crate::params::CompressionParameters;
use openjp2::image::opj_image;
use openjp2::COLOR_SPACE::OPJ_CLRSPC_SRGB;
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
  log::debug!("BMP header info: {:?}", info);

  // Only needed to handle corrupted `issue982.bmp` file.
  if info.is_non_contiguous() {
    return info.read_image(&buf);
  }

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
pub struct BmpBitPrecision {
  pub red_shift: u32,
  pub green_shift: u32,
  pub blue_shift: u32,
  pub alpha_shift: u32,
  pub red_mask: u32,
  pub green_mask: u32,
  pub blue_mask: u32,
  pub alpha_mask: u32,
  pub red: u32,
  pub green: u32,
  pub blue: u32,
  pub alpha: u32,
  pub contiguous: bool,
}

impl BmpBitPrecision {
  pub fn from_masks(
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
    alpha_mask: u32,
  ) -> Option<Self> {
    let (red_shift, red, r_contig) = bmp_mask_get_shift_and_prec(red_mask);
    let (green_shift, green, g_contig) = bmp_mask_get_shift_and_prec(green_mask);
    let (blue_shift, blue, b_contig) = bmp_mask_get_shift_and_prec(blue_mask);
    let (alpha_shift, alpha, a_contig) = bmp_mask_get_shift_and_prec(alpha_mask);
    if red + green + blue + alpha > 0 {
      Some(BmpBitPrecision {
        red_shift,
        green_shift,
        blue_shift,
        alpha_shift,
        red_mask,
        green_mask,
        blue_mask,
        alpha_mask,
        red,
        green,
        blue,
        alpha,
        contiguous: r_contig && g_contig && b_contig && a_contig,
      })
    } else {
      None
    }
  }
}

#[derive(Debug, Default)]
pub struct BmpInfo {
  pub width: usize,
  pub height: usize,
  pub data_offset: usize,
  pub compression: u32,
  pub bits_per_pixel: u16,
  pub stride: usize,
  pub precisions: Option<BmpBitPrecision>,
  pub grayscale: bool,
}

impl BmpInfo {
  pub fn is_non_contiguous(&self) -> bool {
    match &self.precisions {
      Some(prec) => !prec.contiguous,
      None => false,
    }
  }

  pub fn read_raw_data(&self, data: &[u8]) -> Result<Vec<u8>, ImageError> {
    let len = self.height * self.stride;
    match data.get(self.data_offset..self.data_offset + len) {
      Some(data) => Ok(data.to_vec()),
      None => Err(ImageError::InvalidFormat(
        "BMP data offset out of bounds".into(),
      )),
    }
  }

  pub fn read_image(&self, data: &[u8]) -> Result<Box<opj_image>, ImageError> {
    let len = self.height * self.stride;
    let src = match data.get(self.data_offset..self.data_offset + len) {
      Some(data) => data,
      None => {
        return Err(ImageError::InvalidFormat(
          "BMP data offset out of bounds".into(),
        ));
      }
    };

    let mut image = opj_image::new();
    image.color_space = OPJ_CLRSPC_SRGB;
    image.x0 = 0;
    image.y0 = 0;
    image.x1 = self.width as u32;
    image.y1 = self.height as u32;

    let Some(prec) = &self.precisions else {
      return Err(ImageError::InvalidFormat("BMP precisions not set".into()));
    };
    if prec.alpha > 0 {
      image.numcomps = 4;
    } else {
      image.numcomps = 3;
    };

    // Allocate components
    if !image.alloc_comps(image.numcomps) {
      return Err(ImageError::InvalidFormat(
        "Failed to allocate components".into(),
      ));
    }
    let comps = image.comps_mut().expect("We just allocated the components");
    for (idx, comp) in comps.iter_mut().enumerate() {
      comp.dx = 1;
      comp.dy = 1;
      comp.w = self.width as u32;
      comp.h = self.height as u32;
      comp.x0 = 0;
      comp.y0 = 0;
      comp.sgnd = 0;
      match idx {
        0 => {
          comp.prec = prec.red;
          comp.alpha = 0;
        }
        1 => {
          comp.prec = prec.green;
          comp.alpha = 0;
        }
        2 => {
          comp.prec = prec.blue;
          comp.alpha = 0;
        }
        3 => {
          comp.prec = prec.alpha;
          comp.alpha = 1;
        }
        _ => (),
      }
      if !comp.alloc_data() {
        return Err(ImageError::InvalidFormat(
          "Failed to allocate component data".into(),
        ));
      }
    }

    {
      // Get component data and write image from BMP data.
      let mut comps = image
        .comps_data_mut_iter()
        .expect("We just allocated all components data");
      let d_red = comps.next().expect("We just allocated all components data");
      let d_green = comps.next().expect("We just allocated all components data");
      let d_blue = comps.next().expect("We just allocated all components data");

      let src = src
        .chunks_exact(4)
        .map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]]));
      match comps.next() {
        Some(d_alpha) => {
          let src_and_rgba = src.zip(
            d_red
              .iter_mut()
              .zip(d_green.iter_mut())
              .zip(d_blue.iter_mut())
              .zip(d_alpha.iter_mut()),
          );
          for (src, (((red, green), blue), alpha)) in src_and_rgba {
            *red = ((src & prec.red_mask) >> prec.red_shift) as i32;
            *green = ((src & prec.green_mask) >> prec.green_shift) as i32;
            *blue = ((src & prec.blue_mask) >> prec.blue_shift) as i32;
            *alpha = ((src & prec.alpha_mask) >> prec.alpha_shift) as i32;
          }
        }
        None => {
          let src_and_rgb = src.zip(
            d_red
              .iter_mut()
              .zip(d_green.iter_mut())
              .zip(d_blue.iter_mut()),
          );
          for (src, ((red, green), blue)) in src_and_rgb {
            *red = ((src & prec.red_mask) >> prec.red_shift) as i32;
            *green = ((src & prec.green_mask) >> prec.green_shift) as i32;
            *blue = ((src & prec.blue_mask) >> prec.blue_shift) as i32;
          }
        }
      }
    }

    Ok(image)
  }
}

fn bmp_mask_get_shift_and_prec(mask: u32) -> (u32, u32, bool) {
  let shift = mask.trailing_zeros();
  let prec = (mask >> shift).trailing_ones();
  let remain = mask >> (shift + prec);
  (shift, prec, remain == 0)
}

fn parse_bmp_header(data: &[u8]) -> Result<BmpInfo, ImageError> {
  if data.len() < 54 {
    return Err(ImageError::InvalidFormat("BMP header too short".into()));
  }

  // Check BMP signature
  if &data[0..2] != b"BM" {
    return Err(ImageError::InvalidFormat("Not a BMP file".into()));
  }

  // File header.
  // Get offset to image data
  let data_offset = u32::from_le_bytes(data[10..14].try_into().unwrap()) as usize;

  // Read header size to determine format
  let header_size = u32::from_le_bytes(data[14..18].try_into().unwrap()) as usize;
  let width = u32::from_le_bytes(data[18..22].try_into().unwrap()) as usize;
  let height = u32::from_le_bytes(data[22..26].try_into().unwrap()) as usize;
  let bits_per_pixel = u16::from_le_bytes(data[28..30].try_into().unwrap());
  let compression = u32::from_le_bytes(data[30..34].try_into().unwrap());
  let clr_used = u32::from_le_bytes(data[46..50].try_into().unwrap()) as usize;

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
    log::debug!(
      "RGB masks: {:08x} {:08x} {:08x} {:08x}",
      red_mask,
      green_mask,
      blue_mask,
      alpha_mask
    );

    // Convert masks to bit precisions
    precisions = BmpBitPrecision::from_masks(red_mask, green_mask, blue_mask, alpha_mask);
  }

  // For compressions BI_BITFIELDS or BI_ALPHABITFIELDS, check masks
  if compression == 3 || compression == 6 {
    // If no masks use default precisions based on bits per pixel.
    if precisions.is_none() {
      match bits_per_pixel {
        16 => {
          precisions = BmpBitPrecision::from_masks(0xf800, 0x07e0, 0x001f, 0);
        }
        24 => {
          precisions = BmpBitPrecision::from_masks(0xff0000, 0x00ff00, 0x0000ff, 0);
        }
        32 => {
          let alpha = if compression == 6 { 0xff000000 } else { 0 };
          precisions = BmpBitPrecision::from_masks(0xff0000, 0x00ff00, 0x0000ff, alpha);
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
      1usize << bits_per_pixel
    };

    let color_table_offset = 14 + header_size;
    if data.len() >= color_table_offset + (num_colors * 4) {
      grayscale = true;

      // Check each color table entry
      for i in 0..num_colors {
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

  // Calculate row stride.
  let stride = (width * bits_per_pixel as usize + 31) / 32 * 4;

  Ok(BmpInfo {
    width,
    height,
    stride,
    data_offset,
    compression,
    bits_per_pixel,
    precisions,
    grayscale,
  })
}
