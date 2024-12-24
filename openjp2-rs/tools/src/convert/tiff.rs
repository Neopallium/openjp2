use super::*;
use crate::params::CompressionParameters;
use openjp2::image::opj_image;
use std::path::Path;

pub fn load_tiff_image(
  path: &Path,
  params: &CompressionParameters,
) -> Result<Box<opj_image>, ImageError> {
  let tif_image = read_image(path)?;
  let mut image = convert_from_dynamic_image(tif_image, params)?;
  let scaled = if params.is_cinema() {
    // If Cinematic mode was set and the image is RGB(A) rescale
    // to 12 bits per component to comply with cinema profiles.
    let comps = image
      .comps_mut()
      .ok_or_else(|| ImageError::EncodeError("Failed to get image components".into()))?;
    if comps[0].prec != 12 {
      for comp in comps {
        comp.scale(12);
      }
      true
    } else {
      // It was already 12 bits per component.
      false
    }
  } else {
    false
  };
  // If it wasn't scale for Cinematic mode, check if the CLI requested a target bit depth.
  if !scaled {
    if let Some(target_bit_depth) = params.target_bit_depth {
      let comps = image
        .comps_mut()
        .ok_or_else(|| ImageError::EncodeError("Failed to get image components".into()))?;
      for comp in comps {
        comp.scale(target_bit_depth);
      }
    }
  }
  Ok(image)
}

pub fn save_tif_image(image: &mut opj_image, path: &Path) -> Result<(), ImageError> {
  {
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
  }

  let dynamic_img = convert_to_dynamic_image(image)?;
  dynamic_img
    .save(path)
    .map_err(|e| ImageError::EncodeError(e.to_string()))
}
