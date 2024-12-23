use super::ImageError;
use crate::params::ImageFileFormat;
use image::{self, DynamicImage};
use openjp2::{image::opj_image, openjpeg::*};
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn save_image(image: &opj_image, path: &Path) -> Result<(), ImageError> {
  let format = ImageFileFormat::get_file_format(path)
    .map_err(|_| ImageError::InvalidFormat("Unknown file format".into()))?;

  match format {
    ImageFileFormat::RAW => save_raw_image(image, path, true),
    ImageFileFormat::RAWL => save_raw_image(image, path, false),
    ImageFileFormat::PGX => save_pgx_image(image, path),
    _ => {
      let dynamic_img = convert_to_dynamic_image(image)?;

      // Save the image based on file extension
      dynamic_img
        .save(path)
        .map_err(|e| ImageError::EncodeError(e.to_string()))
    }
  }
}

pub fn save_raw_image(image: &opj_image, path: &Path, big_endian: bool) -> Result<(), ImageError> {
  let file = std::fs::File::create(path)?;
  let mut writer = BufWriter::new(file);

  // Check that the image components have matching dimensions, sampling factors, bit depth and signedness.
  if !image.comps_match() {
    return Err(ImageError::InvalidFormat(
      "Mismatched component parameters".into(),
    ));
  }

  let Some(comps) = image.comps_data_iter() else {
    return Err(ImageError::InvalidFormat("Missing components".into()));
  };

  // Write each component's data
  for comp in comps {
    let precision = comp.comp.prec;
    let mask = (1 << precision) - 1;
    let signed = comp.comp.sgnd != 0;

    match precision {
      p if p <= 8 => {
        let (min, max) = if signed {
          (i8::MIN as i32, i8::MAX as i32)
        } else {
          (0, 255)
        };
        // Write 8-bit values
        for &value in comp.data.iter() {
          let value = (value.clamp(min, max) & mask) as u8;
          writer.write_all(&[value])?;
        }
      }
      p if p <= 16 => {
        let (min, max) = if signed {
          (i16::MIN as i32, i16::MAX as i32)
        } else {
          (0, 65535)
        };
        // Write 16-bit values
        for &value in comp.data.iter() {
          let value = (value.clamp(min, max) & mask) as u16;

          let bytes = if big_endian {
            value.to_be_bytes()
          } else {
            value.to_le_bytes()
          };
          writer.write_all(&bytes)?;
        }
      }
      p if p <= 32 => {
        // Write 32-bit values
        for &value in comp.data.iter() {
          let bytes = if big_endian {
            value.to_be_bytes()
          } else {
            value.to_le_bytes()
          };
          writer.write_all(&bytes)?;
        }
      }
      _ => {
        return Err(ImageError::InvalidFormat(format!(
          "Unsupported bit depth: {}",
          precision
        )));
      }
    }
  }

  // Ensure all data is flushed
  writer.flush()?;
  Ok(())
}

pub fn save_pgx_image(image: &opj_image, path: &Path) -> Result<(), ImageError> {
  // For multi-component images we need to save each component as a separate file
  let need_suffix = image.numcomps > 1;
  let stem = path
    .file_stem()
    .and_then(|s| s.to_str())
    .ok_or_else(|| ImageError::InvalidFormat("Invalid path".into()))?;
  let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("pgx");

  for (comp_idx, comp_data) in image
    .comps_data_iter()
    .ok_or_else(|| ImageError::InvalidFormat("Missing components".into()))?
    .enumerate()
  {
    // Create filename with component suffix if needed
    let comp_path = if need_suffix {
      path.with_file_name(format!("{}_{}.{}", stem, comp_idx, ext))
    } else {
      path.to_path_buf()
    };

    let file = std::fs::File::create(&comp_path)?;
    let mut writer = BufWriter::new(file);

    // Write PGX header
    let sign_char = if comp_data.comp.sgnd != 0 { '-' } else { '+' };
    write!(
      writer,
      "PG ML {} {} {} {}\n",
      sign_char, comp_data.comp.prec, comp_data.comp.w, comp_data.comp.h
    )?;

    let precision = comp_data.comp.prec;
    let signed = comp_data.comp.sgnd != 0;
    let adjust = comp_data.adjust;

    if precision <= 8 {
      // Write 8-bit values
      let (min, max) = if signed { (-128, 127) } else { (0, 255) };

      for &value in comp_data.data.iter() {
        let value = (value + adjust).clamp(min, max) as u8;
        writer.write_all(&[value])?;
      }
    } else if precision <= 16 {
      // Write 16-bit values in big-endian order
      let (min, max) = if signed { (-32768, 32767) } else { (0, 65535) };

      for &value in comp_data.data.iter() {
        let value = (value + adjust).clamp(min, max) as u16;
        writer.write_all(&value.to_be_bytes())?;
      }
    } else {
      return Err(ImageError::InvalidFormat(
        "PGX format only supports up to 16 bits per component".into(),
      ));
    }

    // Ensure all data is written
    writer.flush()?;
  }

  Ok(())
}

pub fn convert_to_dynamic_image(image: &opj_image) -> Result<DynamicImage, ImageError> {
  let mut comps = image
    .comps_data_iter()
    .ok_or_else(|| ImageError::InvalidFormat("Missing components".into()))?;

  // Must have at least one component
  let c0 = comps
    .next()
    .ok_or_else(|| ImageError::InvalidFormat("Missing components".into()))?;
  let c1 = comps.next();
  let c2 = comps.next();
  let c3 = comps.next();

  // Only support 1-4 component images
  if comps.next().is_some() {
    return Err(ImageError::InvalidFormat(
      "Unsupported number of components".into(),
    ));
  }

  // The components must have matching parameters.
  if !image.comps_match() {
    return Err(ImageError::InvalidFormat(
      "Mismatched component parameters".into(),
    ));
  }

  let width = c0.comp.w;
  let height = c0.comp.h;
  let adjust = c0.adjust;
  // Convert to DynamicImage based on components
  let dynamic_img = match (c0, c1, c2, c3, image.color_space) {
    (c0, None, None, None, OPJ_CLRSPC_GRAY | OPJ_CLRSPC_UNSPECIFIED) => {
      // Grayscale image

      let pixels = c0.data.iter().map(|&x| x + adjust);
      if c0.comp.prec <= 8 {
        // Convert to ImageLuma8
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|x| x as u8).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageLuma8(img_buf)
      } else {
        // Convert to ImageLuma16
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|x| x as u16).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageLuma16(img_buf)
      }
    }
    (gray, Some(alpha), None, None, OPJ_CLRSPC_GRAY | OPJ_CLRSPC_UNSPECIFIED) => {
      // Grayscale with alpha

      let pixels = gray
        .data
        .iter()
        .zip(alpha.data.iter())
        .map(|(g, a)| [g + adjust, a + adjust])
        .flatten();
      if gray.comp.prec <= 8 {
        // Convert to ImageLumaA8
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|p| p as u8).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageLumaA8(img_buf)
      } else {
        // Convert to ImageLumaA16
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|p| p as u16).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageLumaA16(img_buf)
      }
    }
    (r, Some(g), Some(b), None, OPJ_CLRSPC_SRGB | OPJ_CLRSPC_SYCC | OPJ_CLRSPC_UNSPECIFIED) => {
      // RGB image

      let pixels = r
        .data
        .iter()
        .zip(g.data.iter())
        .zip(b.data.iter())
        .map(|((r, g), b)| [r + adjust, g + adjust, b + adjust])
        .flatten();
      if r.comp.prec <= 8 {
        // Convert to ImageRgb8
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|p| p as u8).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageRgb8(img_buf)
      } else {
        // Convert to ImageRgb16
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|p| p as u16).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageRgb16(img_buf)
      }
    }
    (r, Some(g), Some(b), Some(a), OPJ_CLRSPC_SRGB | OPJ_CLRSPC_SYCC | OPJ_CLRSPC_UNSPECIFIED) => {
      // RGBA image

      let pixels = r
        .data
        .iter()
        .zip(g.data.iter())
        .zip(b.data.iter())
        .zip(a.data.iter())
        .map(|(((r, g), b), a)| [r + adjust, g + adjust, b + adjust, a + adjust])
        .flatten();
      if r.comp.prec <= 8 {
        // Convert to ImageRgba8
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|p| p as u8).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageRgba8(img_buf)
      } else {
        // Convert to ImageRgba16
        let img_buf =
          image::ImageBuffer::from_raw(width, height, pixels.map(|p| p as u16).collect())
            .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
        DynamicImage::ImageRgba16(img_buf)
      }
    }
    _ => {
      return Err(ImageError::InvalidFormat(format!(
        "Unsupported image format: {} components, colorspace {:?}",
        image.numcomps, image.color_space
      )))
    }
  };

  Ok(dynamic_img)
}
