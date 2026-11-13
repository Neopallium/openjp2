use super::*;

/// A fully decoded JPEG 2000 image (owned pixel data).
#[derive(Clone)]
pub struct Image {
  pub(crate) core: CoreImage,
}

impl Image {
  /// Decode a whole image from an in-memory buffer.
  pub fn from_bytes(buf: &[u8]) -> Result<Self> {
    Self::from_bytes_with(buf, DecodeParameters::default())
  }

  /// Decode from an in-memory buffer with the given decoder settings.
  ///
  /// Lenient mode (the default; `DecodeParameters::strict(true)` opts out)
  /// preserves the pixels decoded before a truncation point.  Note the
  /// contract's scope: truncation AFTER the last tile's data (a missing EOC)
  /// delivers the fully decoded image, but truncation mid-tile — or a
  /// used-components failure — may return the header image with EMPTY
  /// component data (earlier tiles' pixels are not moved out).  The C
  /// reference behaves the same way; check `Image::components()` data lengths
  /// when decoding untrusted partial streams.
  pub fn from_bytes_with(buf: &[u8], params: DecodeParameters) -> Result<Self> {
    let format =
      detect_format(buf).ok_or_else(|| Error::Unsupported("unknown JPEG 2000 format".into()))?;
    let mut stream = Stream::from_bytes(buf.to_vec());
    decode_stream(&mut stream, format, &params)
  }

  /// Decode a whole image from a file (feature `file-io`-independent: the
  /// file is read into memory first).
  pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
    Self::from_file_with(path, DecodeParameters::default())
  }

  /// Decode from a file with the given decoder settings.
  pub fn from_file_with<P: AsRef<std::path::Path>>(
    path: P,
    params: DecodeParameters,
  ) -> Result<Self> {
    let buf = std::fs::read(path)?;
    Self::from_bytes_with(&buf, params)
  }

  /// Width of the decoded image.
  pub fn width(&self) -> u32 {
    self.core.x1.wrapping_sub(self.core.x0)
  }

  /// Height of the decoded image.
  pub fn height(&self) -> u32 {
    self.core.y1.wrapping_sub(self.core.y0)
  }

  /// The image colour space.
  pub fn color_space(&self) -> ColorSpace {
    self.core.color_space
  }

  /// Horizontal origin of the image (`core.x0`).
  pub fn x_offset(&self) -> u32 {
    self.core.x0
  }

  /// Vertical origin of the image (`core.y0`).
  pub fn y_offset(&self) -> u32 {
    self.core.y0
  }

  /// Save the image to a JPEG 2000 file; the format is detected from the
  /// file extension (`.j2k`/`.j2c`/`.jpc`/`.jhc` = codestream,
  /// `.jp2`/`.jph` = JP2 container).
  ///
  /// The image is cloned first (`encode*` consume the image, since the
  /// encoder transforms the component buffers in place).
  pub fn save_as_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
    self.save_as_file_with(path, EncodeParameters::new())
  }

  /// Like [`Self::save_as_file`] with explicit encoder parameters.
  pub fn save_as_file_with<P: AsRef<std::path::Path>>(
    &self,
    path: P,
    params: EncodeParameters,
  ) -> Result<()> {
    let path = path.as_ref();
    let ext = path.extension().map(|ext| ext.to_string_lossy());
    let format = crate::detect_format_from_extension(ext.as_deref()).map_err(Error::Unsupported)?;
    let bytes = match format {
      J2KFormat::JP2 => self.clone().encode_jp2(params),
      J2KFormat::J2K => self.clone().encode(params),
      other => {
        return Err(Error::Unsupported(format!(
          "format {:?} is not supported for encoding",
          other
        )))
      }
    }?;
    std::fs::write(path, bytes).map_err(Error::Io)
  }

  /// Number of components.
  pub fn num_components(&self) -> u32 {
    self.core.numcomps
  }

  /// Whether an ICC profile is attached.
  pub fn has_icc_profile(&self) -> bool {
    !self.core.icc_profile_buf.is_empty()
  }

  /// The component planes.
  pub fn components(&self) -> &[CoreImageComp] {
    &self.core.comps
  }

  /// The raw decoded data (escape hatch to the low-level API).
  pub fn into_core_image(self) -> CoreImage {
    self.core
  }

  /// Build an image from raw component planes (the input side of
  /// `encode`/`encode_jp2`).
  pub fn from_core_image(core: CoreImage) -> Self {
    Self { core }
  }

  /// Build an image from an `image`-crate `DynamicImage` (feature
  /// `image`): each channel becomes a component plane (8- or 16-bit).
  #[cfg(feature = "image")]
  pub fn from_dynamic_image(img: &::image::DynamicImage) -> Self {
    use ::image::DynamicImage::*;
    let (w, h) = (img.width(), img.height());
    // Channels as plain u16 planes (8-bit data widened).
    let widen = |data: &[u8], stride: usize, chan: usize| -> Vec<u16> {
      data
        .chunks_exact(stride)
        .map(|px| px[chan] as u16)
        .collect()
    };
    let (numcomps, color_space, planes): (u32, crate::api::ColorSpace, Vec<Vec<u16>>) = match img {
      ImageLuma8(g) => (
        1,
        crate::api::ColorSpace::Gray,
        vec![g.as_raw().iter().map(|&v| v as u16).collect()],
      ),
      ImageLumaA8(g) => (
        2,
        crate::api::ColorSpace::Gray,
        vec![widen(g.as_raw(), 2, 0), widen(g.as_raw(), 2, 1)],
      ),
      ImageRgb8(r) => (
        3,
        crate::api::ColorSpace::Srgb,
        vec![
          widen(r.as_raw(), 3, 0),
          widen(r.as_raw(), 3, 1),
          widen(r.as_raw(), 3, 2),
        ],
      ),
      ImageRgba8(r) => (
        4,
        crate::api::ColorSpace::Srgb,
        vec![
          widen(r.as_raw(), 4, 0),
          widen(r.as_raw(), 4, 1),
          widen(r.as_raw(), 4, 2),
          widen(r.as_raw(), 4, 3),
        ],
      ),
      ImageLuma16(g) => (1, crate::api::ColorSpace::Gray, vec![g.as_raw().to_vec()]),
      ImageLumaA16(g) => (
        2,
        crate::api::ColorSpace::Gray,
        vec![
          g.as_raw().as_chunks::<2>().0.iter().map(|c| c[0]).collect(),
          g.as_raw().as_chunks::<2>().0.iter().map(|c| c[1]).collect(),
        ],
      ),
      ImageRgb16(r) => (
        3,
        crate::api::ColorSpace::Srgb,
        vec![
          r.as_raw().as_chunks::<3>().0.iter().map(|c| c[0]).collect(),
          r.as_raw().as_chunks::<3>().0.iter().map(|c| c[1]).collect(),
          r.as_raw().as_chunks::<3>().0.iter().map(|c| c[2]).collect(),
        ],
      ),
      ImageRgba16(r) => (
        4,
        crate::api::ColorSpace::Srgb,
        vec![
          r.as_raw().as_chunks::<4>().0.iter().map(|c| c[0]).collect(),
          r.as_raw().as_chunks::<4>().0.iter().map(|c| c[1]).collect(),
          r.as_raw().as_chunks::<4>().0.iter().map(|c| c[2]).collect(),
          r.as_raw().as_chunks::<4>().0.iter().map(|c| c[3]).collect(),
        ],
      ),
      _ => {
        let rgba = img.to_rgba8();
        (
          4,
          crate::api::ColorSpace::Srgb,
          vec![
            widen(rgba.as_raw(), 4, 0),
            widen(rgba.as_raw(), 4, 1),
            widen(rgba.as_raw(), 4, 2),
            widen(rgba.as_raw(), 4, 3),
          ],
        )
      }
    };
    /* The precision follows the SOURCE format (a 16-bit image stays 16-bit
    even when all its values are small); the alpha plane of La/Rgba images
    is marked. */
    let prec = if matches!(
      img,
      ImageLuma16(_) | ImageLumaA16(_) | ImageRgb16(_) | ImageRgba16(_)
    ) {
      16
    } else {
      8
    };
    let has_alpha = numcomps == 2 || numcomps == 4;
    Self {
      core: crate::CoreImage {
        x0: 0,
        y0: 0,
        x1: w,
        y1: h,
        numcomps,
        color_space,
        comps: planes
          .into_iter()
          .enumerate()
          .map(|(idx, data)| crate::CoreImageComp {
            dx: 1,
            dy: 1,
            w,
            h,
            x0: 0,
            y0: 0,
            prec,
            bpp: prec,
            sgnd: 0,
            resno_decoded: 0,
            factor: 0,
            alpha: if has_alpha && idx as u32 == numcomps - 1 {
              1
            } else {
              0
            },
            data: data.into_iter().map(i32::from).collect(),
          })
          .collect(),
        icc_profile_buf: Vec::new(),
        icc_profile_is_cielab: false,
      },
    }
  }

  /// Encode the image to a JPEG 2000 codestream held in an in-memory
  /// buffer.  Takes the image by value: the encoder transforms the
  /// component buffers in place, so no copy is made (clone first to keep
  /// the source).
  pub fn encode(self, params: EncodeParameters) -> Result<Vec<u8>> {
    encode_image(self.core, params.into_inner(), J2KFormat::J2K)
  }

  /// Encode the image to a JP2 file in an in-memory buffer.  By-value like
  /// `encode` (clone first to keep the source).
  pub fn encode_jp2(self, params: EncodeParameters) -> Result<Vec<u8>> {
    encode_image(self.core, params.into_inner(), J2KFormat::JP2)
  }

  /// Encode the image into any `Write + Seek` sink (the encoder seeks
  /// back to patch marker lengths, so a plain `Write` is not enough).
  /// By-value like `encode`.
  pub fn encode_to_writer<W: std::io::Write + std::io::Seek + 'static>(
    self,
    params: EncodeParameters,
    writer: W,
  ) -> Result<()> {
    encode_image_to_writer(self.core, params.into_inner(), J2KFormat::J2K, writer)
  }
}

impl std::fmt::Debug for Image {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Image")
      .field("width", &self.width())
      .field("height", &self.height())
      .field("x_offset", &self.x_offset())
      .field("y_offset", &self.y_offset())
      .field("color_space", &self.color_space())
      .field("num_components", &self.num_components())
      .field("has_icc_profile", &self.has_icc_profile())
      .finish()
  }
}
