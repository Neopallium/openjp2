use super::ImageError;
use crate::params::CompressionParameters;
use libtiff_sys::*;
use openjp2::{image::opj_image, openjpeg::*};
use std::ffi::{c_void, CString};
use std::os::raw::c_char;
use std::path::Path;

/// BitBuffer is a simple bit buffer for reading or writing bits.
/// It is used to read or write bits from a byte buffer.
pub struct BitBuffer {
  buffer: Vec<u8>,
  /// Current bit index in the buffer.
  index: usize,
}

impl BitBuffer {
  pub fn new(len: usize) -> Self {
    BitBuffer {
      buffer: vec![0; len],
      index: 0,
    }
  }

  pub fn reset(&mut self) {
    self.index = 0;
    self.buffer.fill(0)
  }

  pub fn write(&mut self, bits: u32, value: u32) {
    // swap bytes for 16-bit values
    let value = if bits == 16 {
      (value >> 8) | ((value & 0xff) << 8)
    } else {
      value
    };
    for i in 0..bits {
      let bit = (value >> (bits - i - 1)) & 1;
      self.write_bit(bit);
    }
  }

  pub fn write_bit(&mut self, bit: u32) {
    if bit == 0 {
      self.index += 1;
      return;
    }
    let byte_index = self.index / 8;
    let bit_index = self.index % 8;
    self.buffer[byte_index] |= 1 << (7 - bit_index);
    self.index += 1;
  }

  pub fn read(&mut self, bits: u32) -> u32 {
    let mut value = 0;
    for _ in 0..bits {
      value = (value << 1) | self.read_bit();
    }
    value
  }

  pub fn read_bit(&mut self) -> u32 {
    let byte_index = self.index / 8;
    let bit_index = self.index % 8;
    let bit = (self.buffer[byte_index] >> (7 - bit_index)) & 1;
    self.index += 1;
    bit as u32
  }

  pub fn as_slice(&self) -> &[u8] {
    &self.buffer
  }

  pub fn as_mut_slice(&mut self) -> &mut [u8] {
    &mut self.buffer
  }

  pub fn as_ptr(&self) -> *const u8 {
    self.buffer.as_ptr()
  }

  pub fn as_mut_ptr(&mut self) -> *mut u8 {
    self.buffer.as_mut_ptr()
  }
}

pub fn load_tiff_image(
  path: &Path,
  params: &CompressionParameters,
) -> Result<Box<opj_image>, ImageError> {
  // Open TIFF file
  let path_str = path
    .to_str()
    .ok_or_else(|| ImageError::InvalidFormat("Invalid path encoding".into()))?;
  let c_path = CString::new(path_str).unwrap();
  let tiff = unsafe { TIFFOpen(c_path.as_ptr(), b"r\0".as_ptr() as *const c_char) };
  if tiff.is_null() {
    return Err(ImageError::InvalidFormat(format!(
      "Failed to open TIFF file: {}",
      path_str
    )));
  }

  // Read TIFF header info
  let mut width: u32 = 0;
  let mut height: u32 = 0;
  let mut samples_per_pixel: u16 = 0;
  let mut bits_per_sample: u16 = 0;
  let mut photometric: u16 = 0;
  let mut planar_config: u16 = 0;

  unsafe {
    TIFFGetField(tiff, TIFFTAG_IMAGEWIDTH, &mut width);
    TIFFGetField(tiff, TIFFTAG_IMAGELENGTH, &mut height);
    TIFFGetField(tiff, TIFFTAG_SAMPLESPERPIXEL, &mut samples_per_pixel);
    TIFFGetField(tiff, TIFFTAG_BITSPERSAMPLE, &mut bits_per_sample);
    TIFFGetField(tiff, TIFFTAG_PHOTOMETRIC, &mut photometric);
    TIFFGetField(tiff, TIFFTAG_PLANARCONFIG, &mut planar_config);
  }
  let photometric = photometric as u32;
  let numcomps = samples_per_pixel as u32;
  let prec = bits_per_sample as u32;
  let planar_config = planar_config as u32;

  // Validate parameters
  if numcomps == 0 || numcomps > 4 {
    unsafe { TIFFClose(tiff) };
    return Err(ImageError::InvalidFormat(
      "Invalid samples per pixel".into(),
    ));
  }

  if prec > 16 || prec == 0 {
    unsafe { TIFFClose(tiff) };
    return Err(ImageError::InvalidFormat(
      "Unsupported bits per sample".into(),
    ));
  }

  if photometric != PHOTOMETRIC_RGB && photometric != PHOTOMETRIC_MINISBLACK {
    unsafe { TIFFClose(tiff) };
    return Err(ImageError::InvalidFormat(
      "Unsupported photometric interpretation".into(),
    ));
  }

  if width == 0 || height == 0 {
    unsafe { TIFFClose(tiff) };
    return Err(ImageError::InvalidFormat("Invalid image dimensions".into()));
  }

  // Create OpenJPEG image
  let color_space = if photometric == PHOTOMETRIC_RGB {
    OPJ_CLRSPC_SRGB
  } else if photometric == PHOTOMETRIC_MINISBLACK {
    OPJ_CLRSPC_GRAY
  } else {
    OPJ_CLRSPC_UNKNOWN
  };

  let mut image = opj_image::new();
  image.x0 = params.image_offset().x;
  image.y0 = params.image_offset().y;
  image.x1 = image.x0 + (width - 1) * params.subsampling().width + 1;
  image.y1 = image.y0 + (height - 1) * params.subsampling().height + 1;
  image.color_space = color_space;

  // Allocate components
  image.alloc_comps(numcomps);

  // Detect alpha channel
  let alpha_idx = match numcomps {
    2 => Some(1),
    4 => Some(3),
    _ => None,
  };
  // Initialize components
  let comps = image.comps_mut().expect("We just allocated the components");
  for (idx, comp) in comps.iter_mut().enumerate() {
    comp.dx = params.subsampling().width;
    comp.dy = params.subsampling().height;
    comp.w = width;
    comp.h = height;
    comp.x0 = 0;
    comp.y0 = 0;
    comp.prec = prec;
    comp.sgnd = 0;
    comp.alpha = if alpha_idx == Some(idx) { 1 } else { 0 };
    comp.alloc_data();
  }

  let strip_size = unsafe { TIFFStripSize(tiff) as usize };
  let read_strip = |buf: &mut BitBuffer, strip| {
    buf.reset();
    let read = unsafe {
      TIFFReadEncodedStrip(
        tiff,
        strip,
        buf.as_mut_ptr() as *mut c_void,
        strip_size as i64,
      )
    };
    if read < 0 {
      unsafe { TIFFClose(tiff) };
      return Err(ImageError::DecodeError("Failed to read strip".into()));
    }
    Ok(read as usize)
  };

  let num_strip = unsafe { TIFFNumberOfStrips(tiff) };
  {
    let mut comps = image
      .comps_data_mut_iter()
      .expect("We just allocated the components");
    // Read image data
    let mut buffer = BitBuffer::new(strip_size);
    if planar_config == PLANARCONFIG_CONTIG {
      let row_bit_size = width as usize * numcomps as usize * prec as usize;
      let row_size = (row_bit_size + 7) / 8;
      let row_bit_padding = row_size * 8 - row_bit_size;
      // Must have at least one component.
      // Only 1-4 components are supported.  Any additional components are ignored.
      let c0 = comps.next().expect("We just allocated the components");
      let c1 = comps.next();
      let c2 = comps.next();
      let c3 = comps.next();

      // Chunk the component data by the image width.
      // This is necessary because the TIFF library requires the data to be in strips.
      let d0 = c0.chunks_exact_mut(width as usize);
      let d1 = c1.map(|c| c.chunks_exact_mut(width as usize));
      let d2 = c2.map(|c| c.chunks_exact_mut(width as usize));
      let d3 = c3.map(|c| c.chunks_exact_mut(width as usize));

      // Contiguous data - all samples in each strip
      match (d0, d1, d2, d3) {
        (mut gray, None, None, None) => {
          // Gray
          for strip in 0..num_strip {
            let mut strip_len = read_strip(&mut buffer, strip)?;

            while strip_len >= row_size {
              let Some(gray_row) = gray.next() else {
                unsafe { TIFFClose(tiff) };
                return Err(ImageError::DecodeError("Too many rows".into()));
              };
              for gray in gray_row {
                *gray = buffer.read(prec) as i32;
              }
              buffer.read(row_bit_padding as u32);
              strip_len -= row_size;
            }
          }
        }
        (mut gray, Some(mut alpha), None, None) => {
          // Gray + Alpha
          for strip in 0..num_strip {
            let mut strip_len = read_strip(&mut buffer, strip)?;

            while strip_len >= row_size {
              let (Some(gray_row), Some(alpha_row)) = (gray.next(), alpha.next()) else {
                unsafe { TIFFClose(tiff) };
                return Err(ImageError::DecodeError("Too many rows".into()));
              };
              for (gray, alpha) in gray_row.iter_mut().zip(alpha_row.iter_mut()) {
                *gray = buffer.read(prec) as i32;
                *alpha = buffer.read(prec) as i32;
              }
              buffer.read(row_bit_padding as u32);
              strip_len -= row_size;
            }
          }
        }
        (mut red, Some(mut green), Some(mut blue), None) => {
          // RGB
          for strip in 0..num_strip {
            let mut strip_len = read_strip(&mut buffer, strip)?;

            while strip_len >= row_size {
              let (Some(red_row), Some(green_row), Some(blue_row)) =
                (red.next(), green.next(), blue.next())
              else {
                unsafe { TIFFClose(tiff) };
                return Err(ImageError::DecodeError("Too many rows".into()));
              };
              for ((red, green), blue) in red_row
                .iter_mut()
                .zip(green_row.iter_mut())
                .zip(blue_row.iter_mut())
              {
                *red = buffer.read(prec) as i32;
                *green = buffer.read(prec) as i32;
                *blue = buffer.read(prec) as i32;
              }
              buffer.read(row_bit_padding as u32);
              strip_len -= row_size;
            }
          }
        }
        (mut red, Some(mut green), Some(mut blue), Some(mut alpha)) => {
          // RGBA
          for strip in 0..num_strip {
            let mut strip_len = read_strip(&mut buffer, strip)?;

            while strip_len >= row_size {
              let (Some(red_row), Some(green_row), Some(blue_row), Some(alpha_row)) =
                (red.next(), green.next(), blue.next(), alpha.next())
              else {
                unsafe { TIFFClose(tiff) };
                return Err(ImageError::DecodeError("Too many rows".into()));
              };
              for (((red, green), blue), alpha) in red_row
                .iter_mut()
                .zip(green_row.iter_mut())
                .zip(blue_row.iter_mut())
                .zip(alpha_row.iter_mut())
              {
                *red = buffer.read(prec) as i32;
                *green = buffer.read(prec) as i32;
                *blue = buffer.read(prec) as i32;
                *alpha = buffer.read(prec) as i32;
              }
              buffer.read(row_bit_padding as u32);
              strip_len -= row_size;
            }
          }
        }
        _ => {
          unsafe { TIFFClose(tiff) };
          return Err(ImageError::EncodeError(
            "Invalid number of components".into(),
          ));
        }
      }
    } else {
      let row_bit_size = width as usize * prec as usize;
      let row_size = (row_bit_size + 7) / 8;
      let row_bit_padding = row_size * 8 - row_bit_size;
      let mut strip = 0;
      // Separate planes
      for data in comps {
        let mut rows = data.chunks_exact_mut(width as usize);

        let mut y = height;
        while strip < num_strip && y > 0 {
          let mut strip_len = read_strip(&mut buffer, strip)?;
          strip += 1;
          while strip_len >= row_size {
            let Some(row) = rows.next() else {
              unsafe { TIFFClose(tiff) };
              return Err(ImageError::DecodeError("Too many rows".into()));
            };
            for dst in row {
              *dst = buffer.read(prec) as i32;
            }
            buffer.read(row_bit_padding as u32);
            strip_len -= row_size;
            y -= 1;
          }
        }
      }
    }
  }

  unsafe { TIFFClose(tiff) };

  // Scale for cinema mode if needed
  let comps = image.comps_mut().expect("We just allocated the components");
  if params.is_cinema() && color_space == OPJ_CLRSPC_SRGB {
    for comp in comps.iter_mut() {
      comp.scale(12);
    }
  } else if let Some(target_depth) = params.target_bit_depth {
    for comp in comps.iter_mut() {
      comp.scale(target_depth);
    }
  }

  Ok(image)
}

pub fn save_tiff_image(image: &mut opj_image, path: &Path) -> Result<(), ImageError> {
  // Open TIFF file
  let path_str = path
    .to_str()
    .ok_or_else(|| ImageError::InvalidFormat("Invalid path encoding".into()))?;
  let c_path = CString::new(path_str).unwrap();
  let tiff = unsafe { TIFFOpen(c_path.as_ptr(), b"wb\0".as_ptr() as *const c_char) };
  if tiff.is_null() {
    return Err(ImageError::EncodeError(format!(
      "Failed to create TIFF file: {}",
      path_str
    )));
  }

  let mut numcomps = image.numcomps;
  if numcomps == 0 {
    unsafe { TIFFClose(tiff) };
    return Err(ImageError::EncodeError(
      "Invalid number of components".into(),
    ));
  }

  // Verify all components have matching parameters
  if !image.comps_match() {
    unsafe { TIFFClose(tiff) };
    return Err(ImageError::EncodeError(
      "Components must have matching parameters".into(),
    ));
  }

  // Clip components.
  if let Some(comps) = image.comps_mut() {
    let prec = comps[0].prec;
    for comp in comps.iter_mut() {
      comp.clip(prec);
    }
  }

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

  let width = c0.comp.w;
  let height = c0.comp.h;
  let prec = c0.comp.prec;
  let sgnd = c0.comp.sgnd;
  let adjust = c0.adjust;
  log::debug!(
    "save_tiff: width: {}, height: {}, prec: {}, sgnd: {}, adjust: {}, numcomps: {}",
    width,
    height,
    prec,
    sgnd,
    adjust,
    numcomps
  );
  let photometric = if image.color_space == OPJ_CLRSPC_CMYK {
    if numcomps < 4 {
      return Err(ImageError::EncodeError(
        "CMYK images shall be composed of at least 4 planes".into(),
      ));
    }
    if numcomps > 4 {
      numcomps = 4; /* Alpha not supported */
    }
    PHOTOMETRIC_SEPARATED
  } else if numcomps > 2 {
    if numcomps > 4 {
      numcomps = 4;
    }
    PHOTOMETRIC_RGB
  } else {
    PHOTOMETRIC_MINISBLACK
  };
  // Set TIFF tags
  unsafe {
    TIFFSetField(tiff, TIFFTAG_IMAGEWIDTH, width);
    TIFFSetField(tiff, TIFFTAG_IMAGELENGTH, height);
    TIFFSetField(tiff, TIFFTAG_SAMPLESPERPIXEL, numcomps);
    TIFFSetField(tiff, TIFFTAG_BITSPERSAMPLE, prec);
    TIFFSetField(tiff, TIFFTAG_ORIENTATION, ORIENTATION_TOPLEFT);
    TIFFSetField(tiff, TIFFTAG_PLANARCONFIG, PLANARCONFIG_CONTIG);
    TIFFSetField(tiff, TIFFTAG_PHOTOMETRIC, photometric);
    TIFFSetField(tiff, TIFFTAG_ROWSPERSTRIP, 1u32);
  }

  // Chunk the component data by the image width.
  // This is necessary because the TIFF library requires the data to be in strips.
  let d0 = c0.data.chunks_exact(width as usize);
  let d1 = c1.map(|c| c.data.chunks_exact(width as usize));
  let d2 = c2.map(|c| c.data.chunks_exact(width as usize));
  let d3 = c3.map(|c| c.data.chunks_exact(width as usize));

  let strip_size = unsafe { TIFFStripSize(tiff) as usize };
  let row_size = (width as usize * numcomps as usize * prec as usize + 7) / 8;
  if strip_size != row_size {
    unsafe { TIFFClose(tiff) };
    return Err(ImageError::EncodeError("Invalid TIFF strip size".into()));
  }
  let write_strip = |buf: &mut BitBuffer, y| {
    // Write row
    let written = unsafe {
      TIFFWriteEncodedStrip(
        tiff,
        y as u32,
        buf.as_ptr() as *mut c_void,
        strip_size as i64,
      )
    };
    buf.reset();
    if written < 0 {
      unsafe { TIFFClose(tiff) };
      return Err(ImageError::EncodeError("Failed to write strip".into()));
    }
    Ok(())
  };

  // Write image data using BitBuffer
  let mut buffer = BitBuffer::new(strip_size);
  match (d0, d1, d2, d3) {
    (d0, None, None, None) => {
      // Write image data with a single component
      for (y, row) in d0.enumerate() {
        for gray in row {
          let gray = (gray - adjust) as u32;
          buffer.write(prec, gray);
        }
        write_strip(&mut buffer, y as u32)?;
      }
    }
    (d0, Some(d1), None, None) => {
      // Write image data with two components
      for (y, (row0, row1)) in d0.zip(d1).enumerate() {
        for (gray, alpha) in row0.iter().zip(row1.iter()) {
          let gray = (gray - adjust) as u32;
          let alpha = (alpha - adjust) as u32;
          buffer.write(prec, gray);
          buffer.write(prec, alpha);
        }
        write_strip(&mut buffer, y as u32)?;
      }
    }
    (d0, Some(d1), Some(d2), None) => {
      // Write image data with three components
      for (y, (row0, (row1, row2))) in d0.zip(d1.zip(d2)).enumerate() {
        for ((red, green), blue) in row0.iter().zip(row1.iter()).zip(row2.iter()) {
          let red = (red - adjust) as u32;
          let green = (green - adjust) as u32;
          let blue = (blue - adjust) as u32;
          buffer.write(prec, red);
          buffer.write(prec, green);
          buffer.write(prec, blue);
        }
        write_strip(&mut buffer, y as u32)?;
      }
    }
    (d0, Some(d1), Some(d2), Some(d3)) => {
      // Write image data with four components
      for (y, (row0, (row1, (row2, row3)))) in d0.zip(d1.zip(d2.zip(d3))).enumerate() {
        for (((red, green), blue), alpha) in row0
          .iter()
          .zip(row1.iter())
          .zip(row2.iter())
          .zip(row3.iter())
        {
          let red = (red - adjust) as u32;
          let green = (green - adjust) as u32;
          let blue = (blue - adjust) as u32;
          let alpha = (alpha - adjust) as u32;
          buffer.write(prec, red);
          buffer.write(prec, green);
          buffer.write(prec, blue);
          buffer.write(prec, alpha);
        }
        write_strip(&mut buffer, y as u32)?;
      }
    }
    _ => {
      unsafe { TIFFClose(tiff) };
      return Err(ImageError::EncodeError(
        "Invalid number of components".into(),
      ));
    }
  }

  unsafe { TIFFClose(tiff) };
  Ok(())
}
