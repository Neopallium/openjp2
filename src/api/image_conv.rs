// Split of the safe API (doc/api-review.md): see the facade in api.rs
// and doc/redesign-proposal.md B.3.2.

use core::convert::TryFrom;
use super::{ColorSpace, CoreImageComp, Error, Image, Result};

  /// Pixel data in a canonical 8/16-bit layout.
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum ImagePixelData {
    /// 8-bit luminance.
    L8(Vec<u8>),
    /// 8-bit luminance + alpha.
    La8(Vec<u8>),
    /// 8-bit RGB.
    Rgb8(Vec<u8>),
    /// 8-bit RGBA.
    Rgba8(Vec<u8>),
    /// 16-bit luminance.
    L16(Vec<u16>),
    /// 16-bit luminance + alpha.
    La16(Vec<u16>),
    /// 16-bit RGB.
    Rgb16(Vec<u16>),
    /// 16-bit RGBA.
    Rgba16(Vec<u16>),
  }

  /// Decoded pixels with their colour format.
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub struct ImageData {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// The pixel layout.
    pub format: ::image::ExtendedColorType,
    /// The pixel data.
    pub data: ImagePixelData,
  }

  impl Image {
    /// Convert the component planes into canonical pixels.
    ///
    /// `alpha_default` fills a missing alpha channel with a constant value
    /// (jpeg2k-compatible: for a 1/3-component image, an extra alpha
    /// channel with the given value is emitted).
    pub fn get_pixels(&self, alpha_default: Option<u32>) -> Result<ImageData> {
      let comps = &self.core.comps;
      let (width, height) = comps
        .first()
        .map(|c| (c.w, c.h))
        .ok_or_else(|| Error::Unsupported("image has no components".into()))?;
      /* Legally subsampled codestreams (XRsiz/YRsiz > 1) have component
      planes of different dimensions; the per-plane iterators below zip and
      would silently truncate.  Refuse them instead of returning an
      ImageData shorter than its contract (and panicking in the
      DynamicImage conversion). */
      if !self.core.comps_same_dims() {
        return Err(Error::Unsupported(
          "components have different dimensions (subsampled image); convert each plane separately"
            .into(),
        ));
      }
      /* A partially decoded image (lenient/truncated codestream) has
      component data shorter than the declared geometry; refuse it too
      instead of silently returning an ImageData shorter than w*h. */
      if comps
        .iter()
        .any(|c| c.data.len() != (c.w as usize) * (c.h as usize))
      {
        return Err(Error::Unsupported(
          "component data is truncated (partial decode); the image must be fully decoded"
            .into(),
        ));
      }
      let max_prec = comps.iter().fold(0u32, |max, c| max.max(c.prec));
      let has_alpha = comps.iter().any(|c| c.alpha != 0);
      match &self.color_space() {
        ColorSpace::Unknown => { /* infer from the component count below */ }
        ColorSpace::Srgb | ColorSpace::Gray => {}
        cs => return Err(Error::Unsupported(format!("color space {cs:?}"))),
      }

      let data = match (comps.as_slice(), has_alpha, max_prec) {
        ([r], _, 1..=8) => {
          if let Some(alpha) = alpha_default {
            (
              ::image::ExtendedColorType::La8,
              ImagePixelData::La8(
                comp_u8(r).flat_map(|r| [r, alpha as u8]).collect(),
              ),
            )
          } else {
            (
              ::image::ExtendedColorType::L8,
              ImagePixelData::L8(comp_u8(r).collect()),
            )
          }
        }
        ([r], _, 9..=16) => {
          if let Some(alpha) = alpha_default {
            (
              ::image::ExtendedColorType::La16,
              ImagePixelData::La16(
                comp_u16(r).flat_map(|r| [r, alpha as u16]).collect(),
              ),
            )
          } else {
            (
              ::image::ExtendedColorType::L16,
              ImagePixelData::L16(comp_u16(r).collect()),
            )
          }
        }
        ([r, a], true, 1..=8) => (
          ::image::ExtendedColorType::La8,
          ImagePixelData::La8(
            comp_u8(r)
              .zip(comp_u8(a))
              .flat_map(|(r, a)| [r, a])
              .collect(),
          ),
        ),
        ([r, a], true, 9..=16) => (
          ::image::ExtendedColorType::La16,
          ImagePixelData::La16(
            comp_u16(r)
              .zip(comp_u16(a))
              .flat_map(|(r, a)| [r, a])
              .collect(),
          ),
        ),
        ([r, g, b], false, 1..=8) => {
          if let Some(alpha) = alpha_default {
            (
              ::image::ExtendedColorType::Rgba8,
              ImagePixelData::Rgba8(
                comp_u8(r)
                  .zip(comp_u8(g).zip(comp_u8(b)))
                  .flat_map(|(r, (g, b))| [r, g, b, alpha as u8])
                  .collect(),
              ),
            )
          } else {
            (
              ::image::ExtendedColorType::Rgb8,
              ImagePixelData::Rgb8(
                comp_u8(r)
                  .zip(comp_u8(g).zip(comp_u8(b)))
                  .flat_map(|(r, (g, b))| [r, g, b])
                  .collect(),
              ),
            )
          }
        }
        ([r, g, b], false, 9..=16) => {
          if let Some(alpha) = alpha_default {
            (
              ::image::ExtendedColorType::Rgba16,
              ImagePixelData::Rgba16(
                comp_u16(r)
                  .zip(comp_u16(g).zip(comp_u16(b)))
                  .flat_map(|(r, (g, b))| [r, g, b, alpha as u16])
                  .collect(),
              ),
            )
          } else {
            (
              ::image::ExtendedColorType::Rgb16,
              ImagePixelData::Rgb16(
                comp_u16(r)
                  .zip(comp_u16(g).zip(comp_u16(b)))
                  .flat_map(|(r, (g, b))| [r, g, b])
                  .collect(),
              ),
            )
          }
        }
        ([r, g, b, a], _, 1..=8) => (
          ::image::ExtendedColorType::Rgba8,
          ImagePixelData::Rgba8(
            comp_u8(r)
              .zip(comp_u8(g).zip(comp_u8(b).zip(comp_u8(a))))
              .flat_map(|(r, (g, (b, a)))| [r, g, b, a])
              .collect(),
          ),
        ),
        ([r, g, b, a], _, 9..=16) => (
          ::image::ExtendedColorType::Rgba16,
          ImagePixelData::Rgba16(
            comp_u16(r)
              .zip(comp_u16(g).zip(comp_u16(b).zip(comp_u16(a))))
              .flat_map(|(r, (g, (b, a)))| [r, g, b, a])
              .collect(),
          ),
        ),
        _ => {
          return Err(Error::Unsupported(format!(
            "unsupported component/precision combination: {} components, max precision {}",
            comps.len(),
            max_prec
          )))
        }
      };

      Ok(ImageData {
        width,
        height,
        format: data.0,
        data: data.1,
      })
    }
  }

  /// Scales a component to unsigned 8-bit values.
  pub(crate) fn comp_u8(comp: &CoreImageComp) -> impl Iterator<Item = u8> + '_ {
    // Bound the walk by the data actually present, not by w*h: a partial
    // (lenient/truncated) decode legitimately leaves the plane shorter than
    // the declared geometry, and the slice must not panic.
    let len = ((comp.w as usize) * (comp.h as usize)).min(comp.data.len());
    let data = &comp.data[..len];
    let prec = comp.prec.clamp(1, 31);
    let signed = comp.sgnd != 0;
    data.iter().map(move |p| {
      if signed {
        let old_max = 1i64 << (prec - 1);
        const NEW_MAX: i64 = 1 << 7;
        ((((*p as i64) * NEW_MAX) / old_max) + NEW_MAX) as u8
      } else {
        let old_max = (1u64 << prec) - 1;
        const NEW_MAX: u64 = (1 << 8) - 1;
        (((*p as u64) * NEW_MAX) / old_max) as u8
      }
    })
  }

  /// Scales a component to unsigned 16-bit values.
  pub(crate) fn comp_u16(comp: &CoreImageComp) -> impl Iterator<Item = u16> + '_ {
    // See comp_u8: bound by the available data, clamp the precision.
    let len = ((comp.w as usize) * (comp.h as usize)).min(comp.data.len());
    let data = &comp.data[..len];
    let prec = comp.prec.clamp(1, 31);
    let signed = comp.sgnd != 0;
    data.iter().map(move |p| {
      if signed {
        let old_max = 1i64 << (prec - 1);
        const NEW_MAX: i64 = 1 << 15;
        ((((*p as i64) * NEW_MAX) / old_max) + NEW_MAX) as u16
      } else {
        let old_max = (1u64 << prec) - 1;
        const NEW_MAX: u64 = (1 << 16) - 1;
        (((*p as u64) * NEW_MAX) / old_max) as u16
      }
    })
  }

  impl TryFrom<&Image> for ::image::DynamicImage {
    type Error = Error;

    fn try_from(img: &Image) -> Result<::image::DynamicImage> {
      use ::image::*;
      let ImageData {
        width,
        height,
        data,
        ..
      } = img.get_pixels(None)?;
      Ok(match data {
        ImagePixelData::L8(d) => DynamicImage::ImageLuma8(
          GrayImage::from_vec(width, height, d).ok_or_else(|| Error::Unsupported("pixel buffer length mismatch".into()))?,
        ),
        ImagePixelData::La8(d) => DynamicImage::ImageLumaA8(
          GrayAlphaImage::from_vec(width, height, d).ok_or_else(|| Error::Unsupported("pixel buffer length mismatch".into()))?,
        ),
        ImagePixelData::Rgb8(d) => DynamicImage::ImageRgb8(
          RgbImage::from_vec(width, height, d).ok_or_else(|| Error::Unsupported("pixel buffer length mismatch".into()))?,
        ),
        ImagePixelData::Rgba8(d) => DynamicImage::ImageRgba8(
          RgbaImage::from_vec(width, height, d).ok_or_else(|| Error::Unsupported("pixel buffer length mismatch".into()))?,
        ),
        ImagePixelData::L16(d) => DynamicImage::ImageLuma16(
          ImageBuffer::from_vec(width, height, d).ok_or_else(|| Error::Unsupported("pixel buffer length mismatch".into()))?,
        ),
        ImagePixelData::La16(d) => DynamicImage::ImageLumaA16(
          ImageBuffer::from_vec(width, height, d).ok_or_else(|| Error::Unsupported("pixel buffer length mismatch".into()))?,
        ),
        ImagePixelData::Rgb16(d) => DynamicImage::ImageRgb16(
          ImageBuffer::from_vec(width, height, d).ok_or_else(|| Error::Unsupported("pixel buffer length mismatch".into()))?,
        ),
        ImagePixelData::Rgba16(d) => DynamicImage::ImageRgba16(
          ImageBuffer::from_vec(width, height, d).ok_or_else(|| Error::Unsupported("pixel buffer length mismatch".into()))?,
        ),
      })
    }
  }
