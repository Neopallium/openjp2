use super::ImageError;
use crate::params::CompressionParameters;
use libtiff_sys::*;
use openjp2::{image::opj_image, openjpeg::*};
use std::ffi::{c_void, CString};
use std::os::raw::c_char;
use std::path::Path;

// Bit depth conversion functions
fn convert_32s_to_bits<const N: u32>(src: &[i32], signed: bool) -> Vec<u8> {
  let mut dst = Vec::new();
  let mask = (1u32 << N) - 1;

  match N {
    1..=8 => {
      for &value in src {
        let v = if signed {
          value.clamp(i8::MIN as i32, i8::MAX as i32)
        } else {
          value.clamp(0, 255)
        };
        dst.push((v & (mask as i32)) as u8);
      }
    }
    9..=16 => {
      for &value in src {
        let v = if signed {
          value.clamp(i16::MIN as i32, i16::MAX as i32)
        } else {
          value.clamp(0, 65535)
        };
        let bytes = ((v & (mask as i32)) as u16).to_ne_bytes();
        dst.extend_from_slice(&bytes);
      }
    }
    _ => panic!("Unsupported bit depth: {}", N),
  }
  dst
}

fn convert_bits_to_32s<const N: u32>(src: &[u8], signed: bool) -> Vec<i32> {
  let mut dst = Vec::new();

  match N {
    1..=8 => {
      for &byte in src {
        let value = if signed {
          byte as i8 as i32
        } else {
          byte as i32
        };
        dst.push(value);
      }
    }
    9..=16 => {
      for chunk in src.chunks_exact(2) {
        let value = u16::from_ne_bytes([chunk[0], chunk[1]]);
        let value = if signed {
          value as i16 as i32
        } else {
          value as i32
        };
        dst.push(value);
      }
    }
    _ => panic!("Unsupported bit depth: {}", N),
  }
  dst
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
  let samples_per_pixel = samples_per_pixel as u32;
  let bits_per_sample = bits_per_sample as u32;
  let planar_config = planar_config as u32;

  // Validate parameters
  if samples_per_pixel == 0 || samples_per_pixel > 4 {
    unsafe { TIFFClose(tiff) };
    return Err(ImageError::InvalidFormat(
      "Invalid samples per pixel".into(),
    ));
  }

  if bits_per_sample > 16 || bits_per_sample == 0 {
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

  // Create OpenJPEG image
  let color_space = if samples_per_pixel == 1 {
    OPJ_CLRSPC_GRAY
  } else if samples_per_pixel >= 3 {
    if photometric == PHOTOMETRIC_RGB {
      OPJ_CLRSPC_SRGB
    } else {
      OPJ_CLRSPC_UNKNOWN
    }
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
  image.alloc_comps(samples_per_pixel);
  let comps = image.comps_mut().unwrap();

  // Initialize components
  for comp in comps.iter_mut() {
    comp.dx = params.subsampling().width;
    comp.dy = params.subsampling().height;
    comp.w = width;
    comp.h = height;
    comp.x0 = 0;
    comp.y0 = 0;
    comp.prec = bits_per_sample as u32;
    comp.sgnd = 0;
    comp.alpha = 0;
    comp.alloc_data();
  }

  // Detect alpha channel
  if let Some(comp) = comps.get_mut(samples_per_pixel as usize - 1) {
    comp.alpha = if samples_per_pixel == 2 || samples_per_pixel == 4 {
      1
    } else {
      0
    };
  }

  // Read image data
  let strip_size = unsafe { TIFFStripSize(tiff) as usize };
  let mut buffer = vec![0u8; strip_size];

  if planar_config == PLANARCONFIG_CONTIG {
    // Contiguous data - all samples in each strip
    for y in 0..height {
      let strip = unsafe { TIFFComputeStrip(tiff, y, 0) };
      let read = unsafe {
        TIFFReadEncodedStrip(
          tiff,
          strip,
          buffer.as_mut_ptr() as *mut c_void,
          strip_size as i64,
        )
      };
      if read < 0 {
        unsafe { TIFFClose(tiff) };
        return Err(ImageError::DecodeError("Failed to read strip".into()));
      }

      let row_data =
        &buffer[..(width * samples_per_pixel as u32 * (bits_per_sample as u32 / 8)) as usize];
      let values = convert_bits_to_32s::<16>(row_data, false);

      // Deinterleave samples into components
      for (i, comp) in comps.iter_mut().enumerate() {
        let data = comp.data_mut().unwrap();
        let offset = y * width;
        for (j, value) in values
          .iter()
          .skip(i)
          .step_by(samples_per_pixel as usize)
          .enumerate()
        {
          data[offset as usize + j] = *value;
        }
      }
    }
  } else {
    // Separate planes
    for comp_idx in 0..samples_per_pixel {
      let comp = &mut comps[comp_idx as usize];
      let data = comp.data_mut().unwrap();

      for y in 0..height {
        let strip = unsafe { TIFFComputeStrip(tiff, y, comp_idx as u16) };
        let read = unsafe {
          TIFFReadEncodedStrip(
            tiff,
            strip,
            buffer.as_mut_ptr() as *mut c_void,
            strip_size as i64,
          )
        };
        if read < 0 {
          unsafe { TIFFClose(tiff) };
          return Err(ImageError::DecodeError("Failed to read strip".into()));
        }

        let row_data = &buffer[..(width * (bits_per_sample as u32 / 8)) as usize];
        let values = convert_bits_to_32s::<16>(row_data, false);

        let offset = y * width;
        data[offset as usize..(offset + width) as usize].copy_from_slice(&values);
      }
    }
  }

  unsafe { TIFFClose(tiff) };

  // Scale for cinema mode if needed
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
  let (comp0_w, comp0_h, comp0_prec) = image.comp0_dims_prec();
  let comp0 = image.comps().unwrap().first().unwrap();

  let sgnd = comp0.sgnd;
  let adjust = if sgnd != 0 { 1 << (comp0_prec - 1) } else { 0 };
  eprintln!(
    "comp0_w: {}, comp0_h: {}, comp0_prec: {}, sgnd: {}, adjust: {}, numcomps: {}",
    comp0_w, comp0_h, comp0_prec, comp0.sgnd, adjust, numcomps
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
    TIFFSetField(tiff, TIFFTAG_IMAGEWIDTH, comp0_w);
    TIFFSetField(tiff, TIFFTAG_IMAGELENGTH, comp0_h);
    TIFFSetField(tiff, TIFFTAG_SAMPLESPERPIXEL, numcomps);
    TIFFSetField(tiff, TIFFTAG_BITSPERSAMPLE, comp0_prec);
    TIFFSetField(tiff, TIFFTAG_ORIENTATION, ORIENTATION_TOPLEFT);
    TIFFSetField(tiff, TIFFTAG_PLANARCONFIG, PLANARCONFIG_CONTIG);
    TIFFSetField(tiff, TIFFTAG_PHOTOMETRIC, photometric);
    TIFFSetField(tiff, TIFFTAG_ROWSPERSTRIP, 1u32);
  }

  let Some(comps) = image.comps_data_iter() else {
    unsafe { TIFFClose(tiff) };
    return Err(ImageError::EncodeError("No image components".into()));
  };
  let comps = comps.collect::<Vec<_>>();

  let strip_size = unsafe { TIFFStripSize(tiff) as usize };
  let row_size = (comp0_w as usize * numcomps as usize * comp0_prec as usize + 7) / 8;
  if strip_size != row_size {
    unsafe { TIFFClose(tiff) };
    return Err(ImageError::EncodeError("Invalid TIFF strip size".into()));
  }
  // Write image data
  let mut buffer = vec![0u8; strip_size];

  for y in 0..comp0_h {
    // Interleave component data for the row
    let mut idx = 0;
    for x in 0..comp0_w {
      for comp in &comps {
        let value = comp.data[y as usize * comp0_w as usize + x as usize];
        if comp0_prec <= 8 {
          buffer[idx] = value as u8;
          idx += 1;
        } else {
          let bytes = (value as u16).to_ne_bytes();
          buffer[idx..idx + 2].copy_from_slice(&bytes);
          idx += 2;
        }
      }
    }

    // Write row
    let written = unsafe {
      TIFFWriteEncodedStrip(
        tiff,
        y as u32,
        buffer.as_ptr() as *mut c_void,
        strip_size as i64,
      )
    };
    if written < 0 {
      unsafe { TIFFClose(tiff) };
      return Err(ImageError::EncodeError("Failed to write strip".into()));
    }
  }

  unsafe { TIFFClose(tiff) };
  Ok(())
}
