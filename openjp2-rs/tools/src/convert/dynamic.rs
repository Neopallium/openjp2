use super::ImageError;
use crate::params::CompressionParameters;
use image::{self, DynamicImage};
use openjp2::{image::opj_image, openjpeg::*};

pub fn convert_comp_to_dynamic_grayscale(
  comp: &opj_image_comp,
) -> Result<DynamicImage, ImageError> {
  let width = comp.w;
  let height = comp.h;
  let adjust = if comp.sgnd != 0 {
    1 << (comp.prec - 1)
  } else {
    0
  };
  let Some(data) = comp.data() else {
    return Err(ImageError::InvalidFormat("Missing component data".into()));
  };

  // Convert one component to GrayScale image
  let pixels = data.iter().map(|&x| x + adjust);
  if comp.prec <= 8 {
    // Convert to ImageLuma8
    let img_buf = image::ImageBuffer::from_raw(width, height, pixels.map(|x| x as u8).collect())
      .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
    Ok(DynamicImage::ImageLuma8(img_buf))
  } else {
    // Convert to ImageLuma16
    let img_buf = image::ImageBuffer::from_raw(width, height, pixels.map(|x| x as u16).collect())
      .ok_or_else(|| ImageError::EncodeError("Failed to create image buffer".into()))?;
    Ok(DynamicImage::ImageLuma16(img_buf))
  }
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

  let color_space = match (image.numcomps, image.color_space) {
    (1 | 2, OPJ_CLRSPC_UNKNOWN | OPJ_CLRSPC_UNSPECIFIED) => OPJ_CLRSPC_GRAY,
    (3 | 4, OPJ_CLRSPC_UNKNOWN | OPJ_CLRSPC_UNSPECIFIED) => OPJ_CLRSPC_SRGB,
    (_, color_space) => color_space,
  };

  let width = c0.comp.w;
  let height = c0.comp.h;
  let adjust = c0.adjust;
  // Convert to DynamicImage based on components
  let dynamic_img = match (c0, c1, c2, c3, color_space) {
    (c0, None, None, None, OPJ_CLRSPC_GRAY) => {
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
    (gray, Some(alpha), None, None, OPJ_CLRSPC_GRAY) => {
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
    (r, Some(g), Some(b), None, OPJ_CLRSPC_SRGB | OPJ_CLRSPC_SYCC) => {
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
    (r, Some(g), Some(b), Some(a), OPJ_CLRSPC_SRGB | OPJ_CLRSPC_SYCC) => {
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
