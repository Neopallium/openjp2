//! Safe, high-level API over the codec (see doc/api-review.md, item 13 of
//! the work plan): one-shot `Image` decoding, a `DecodeParameters` builder,
//! a typed `ColorSpace`, and a unified `Error`.  The low-level `Codec` /
//! `Stream` types remain available for streaming and tile-level use cases.

use crate::c_api_types::OPJ_COLOR_SPACE;
use crate::c_api_types::{
  OPJ_CLRSPC_CMYK, OPJ_CLRSPC_EYCC, OPJ_CLRSPC_GRAY, OPJ_CLRSPC_SRGB, OPJ_CLRSPC_SYCC,
  OPJ_CLRSPC_UNKNOWN, OPJ_CODEC_J2K, OPJ_CODEC_JP2,
};
use crate::consts::opj::OPJ_DPARAMETERS_IGNORE_PCLR_CMAP_CDEF_FLAG;
use crate::{Codec, CoreImage, CoreImageComp, J2KFormat, Stream, opj_dparameters};

/// Errors returned by the safe API.
#[derive(Debug)]
pub enum Error {
  /// I/O failure (file reads, stream errors).
  Io(std::io::Error),
  /// The codestream is corrupt or uses an unsupported feature.
  Decode(String),
  /// Invalid encode parameters or a failure while encoding.
  Encode(String),
  /// The requested feature is not supported (by the build or the format).
  Unsupported(String),
}

impl core::fmt::Display for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Error::Io(e) => write!(f, "io error: {e}"),
      Error::Decode(s) => write!(f, "decode error: {s}"),
      Error::Encode(s) => write!(f, "encode error: {s}"),
      Error::Unsupported(s) => write!(f, "unsupported: {s}"),
    }
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Error::Io(e) => Some(e),
      _ => None,
    }
  }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self {
    Error::Io(e)
  }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Safe colour-space enum (mirror of the C `OPJ_COLOR_SPACE` values).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum ColorSpace {
  #[default]
  Unknown,
  Srgb,
  Gray,
  Sycc,
  Eycc,
  Cmyk,
}

impl From<OPJ_COLOR_SPACE> for ColorSpace {
  fn from(cs: OPJ_COLOR_SPACE) -> Self {
    match cs {
      OPJ_CLRSPC_SRGB => ColorSpace::Srgb,
      OPJ_CLRSPC_GRAY => ColorSpace::Gray,
      OPJ_CLRSPC_SYCC => ColorSpace::Sycc,
      OPJ_CLRSPC_EYCC => ColorSpace::Eycc,
      OPJ_CLRSPC_CMYK => ColorSpace::Cmyk,
      _ => ColorSpace::Unknown,
    }
  }
}

impl From<ColorSpace> for OPJ_COLOR_SPACE {
  fn from(cs: ColorSpace) -> Self {
    match cs {
      ColorSpace::Unknown => OPJ_CLRSPC_UNKNOWN,
      ColorSpace::Srgb => OPJ_CLRSPC_SRGB,
      ColorSpace::Gray => OPJ_CLRSPC_GRAY,
      ColorSpace::Sycc => OPJ_CLRSPC_SYCC,
      ColorSpace::Eycc => OPJ_CLRSPC_EYCC,
      ColorSpace::Cmyk => OPJ_CLRSPC_CMYK,
    }
  }
}

/// Packet progression order (mirrors the C `OPJ_PROG_ORDER` values).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum ProgressionOrder {
  #[default]
  Lrcp,
  Rlcp,
  Rpcl,
  Pcrl,
  Cprl,
}

impl From<ProgressionOrder> for crate::c_api_types::OPJ_PROG_ORDER {
  fn from(p: ProgressionOrder) -> Self {
    use crate::c_api_types::{OPJ_CPRL, OPJ_LRCP, OPJ_PCRL, OPJ_RLCP, OPJ_RPCL};
    match p {
      ProgressionOrder::Lrcp => OPJ_LRCP,
      ProgressionOrder::Rlcp => OPJ_RLCP,
      ProgressionOrder::Rpcl => OPJ_RPCL,
      ProgressionOrder::Pcrl => OPJ_PCRL,
      ProgressionOrder::Cprl => OPJ_CPRL,
    }
  }
}

/// Rectangular decode area in image coordinates (`x0 <= x < x1`).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DecodeArea {
  pub start_x: u32,
  pub start_y: u32,
  pub end_x: u32,
  pub end_y: u32,
}

impl DecodeArea {
  pub fn new(start_x: u32, start_y: u32, end_x: u32, end_y: u32) -> Self {
    Self {
      start_x,
      start_y,
      end_x,
      end_y,
    }
  }
}

/// Decoder settings, built with the fluent builder pattern.
#[derive(Debug, Clone, Default)]
pub struct DecodeParameters {
  pub reduce: u32,
  pub layers: u32,
  pub strict: bool,
  pub decode_area: Option<DecodeArea>,
  pub ignore_pclr_cmap_cdef: bool,
}

impl DecodeParameters {
  pub fn new() -> Self {
    Self::default()
  }

  /// Reduce the decoded image by a factor of `2^reduce`.
  pub fn reduce(mut self, reduce: u32) -> Self {
    self.reduce = reduce;
    self
  }

  /// Decode only the first `layers` quality layers (0 = all).
  pub fn layers(mut self, layers: u32) -> Self {
    self.layers = layers;
    self
  }

  /// Reject malformed codestreams instead of tolerating them.
  pub fn strict(mut self, strict: bool) -> Self {
    self.strict = strict;
    self
  }

  /// Restrict the decoded area (whole image when None).
  pub fn decode_area(mut self, area: Option<DecodeArea>) -> Self {
    self.decode_area = area;
    self
  }

  /// Ignore the palette/channel-definition boxes during JP2 colour
  /// application.
  pub fn ignore_pclr_cmap_cdef(mut self) -> Self {
    self.ignore_pclr_cmap_cdef = true;
    self
  }
}

/// Encoder settings (builder over the low-level `EncoderParameters`).
#[derive(Debug, Clone, Default)]
pub struct EncodeParameters {
  inner: crate::EncoderParameters,
}

impl EncodeParameters {
  pub fn new() -> Self {
    Self {
      inner: crate::EncoderParameters::default(),
    }
  }

  /// Number of DWT decomposition levels.
  pub fn num_resolutions(mut self, n: u32) -> Self {
    self.inner.numresolution = n as i32;
    self
  }

  /// Number of quality layers.
  pub fn num_layers(mut self, n: u32) -> Self {
    self.inner.tcp_numlayers = n as i32;
    self
  }

  /// Use the irreversible 9-7 transform (default is reversible 5-3).
  pub fn irreversible(mut self, yes: bool) -> Self {
    self.inner.irreversible = yes as i32;
    self
  }

  /// Packet progression order.
  pub fn progression_order(mut self, order: ProgressionOrder) -> Self {
    self.inner.prog_order = order.into();
    self
  }

  /// Use tiles of the given size.
  pub fn tile_size(mut self, w: u32, h: u32) -> Self {
    self.inner.tile_size_on = 1;
    self.inner.cp_tdx = w as i32;
    self.inner.cp_tdy = h as i32;
    self
  }

  /// Append a quality layer with the given compression ratio
  /// (e.g. `20.0` = 20:1); enables fixed-quality rate allocation.
  pub fn layer_rate(mut self, rate: f32) -> Self {
    let n = self.inner.tcp_numlayers as usize;
    if n < 100 {
      self.inner.cp_disto_alloc = 1;
      self.inner.tcp_rates[n] = rate;
      self.inner.tcp_numlayers += 1;
    }
    self
  }

  /// Embed a comment marker in the codestream.
  pub fn comment(mut self, comment: &str) -> Self {
    self.inner.cp_comment = Some(comment.as_bytes().to_vec());
    self
  }

  /// Code-block width (power of two, 4..=1024).
  pub fn codeblock_width(mut self, w: u32) -> Self {
    self.inner.cblockw_init = w as i32;
    self
  }

  /// Code-block height (power of two, 4..=1024).
  pub fn codeblock_height(mut self, h: u32) -> Self {
    self.inner.cblockh_init = h as i32;
    self
  }

  /// Access the underlying low-level parameters.
  pub fn low_level(&mut self) -> &mut crate::EncoderParameters {
    &mut self.inner
  }

  pub fn into_inner(self) -> crate::EncoderParameters {
    self.inner
  }
}

/// A fully decoded JPEG 2000 image (owned pixel data).
pub struct Image {
  core: CoreImage,
}

impl Image {
  /// Decode a whole image from an in-memory buffer.
  pub fn from_bytes(buf: &[u8]) -> Result<Self> {
    Self::from_bytes_with(buf, DecodeParameters::default())
  }

  /// Decode from an in-memory buffer with the given decoder settings.
  pub fn from_bytes_with(buf: &[u8], params: DecodeParameters) -> Result<Self> {
    let format = detect_format(buf)
      .ok_or_else(|| Error::Unsupported("unknown JPEG 2000 format".into()))?;
    let buf = buf.to_vec();
    let len = buf.len() as u64;
    let mut stream = Stream::new_input(
      crate::consts::opj::OPJ_J2K_STREAM_CHUNK_SIZE as usize,
      std::io::Cursor::new(buf),
      len,
    );
    decode_stream(&mut stream, format, &params)
  }

  /// Decode a whole image from a file (feature `file-io`-independent: the
  /// file is read into memory first).
  pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
    Self::from_file_with(path, DecodeParameters::default())
  }

  /// Decode from a file with the given decoder settings.
  pub fn from_file_with<P: AsRef<std::path::Path>>(path: P, params: DecodeParameters) -> Result<Self> {
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
    self.core.color_space.into()
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

  /// Encode the image to a JPEG 2000 codestream (or JP2 file when
  /// `format` is set) held in an in-memory buffer.
  pub fn encode(&self, params: EncodeParameters) -> Result<Vec<u8>> {
    encode_image(&self.core, params.into_inner(), J2KFormat::J2K)
  }

  /// Encode the image to a JP2 file in an in-memory buffer.
  pub fn encode_jp2(&self, params: EncodeParameters) -> Result<Vec<u8>> {
    encode_image(&self.core, params.into_inner(), J2KFormat::JP2)
  }
}

/// `Write + Seek` adapter collecting into a shared `Vec<u8>`; the j2k
/// encoder seeks back to patch marker lengths, so the position must be
/// honoured.
struct SharedVecWriter {
  buf: std::rc::Rc<std::cell::RefCell<Vec<u8>>>,
  pos: u64,
}

impl std::io::Write for SharedVecWriter {
  fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
    let mut buf = self.buf.borrow_mut();
    let start = self.pos as usize;
    if start + data.len() > buf.len() {
      buf.resize(start + data.len(), 0);
    }
    buf[start..start + data.len()].copy_from_slice(data);
    self.pos += data.len() as u64;
    Ok(data.len())
  }

  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}

impl std::io::Seek for SharedVecWriter {
  fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
    let new_pos = match pos {
      std::io::SeekFrom::Start(p) => p,
      std::io::SeekFrom::Current(d) => (self.pos as i64 + d) as u64,
      std::io::SeekFrom::End(d) => (self.buf.borrow().len() as i64 + d) as u64,
    };
    self.pos = new_pos;
    Ok(new_pos)
  }
}

/// Internal driver over the (deprecated-for-users) low-level Codec methods.
#[allow(deprecated)]
fn encode_image(
  image: &CoreImage,
  params: crate::EncoderParameters,
  format: J2KFormat,
) -> Result<Vec<u8>> {
  let codec_format = match format {
    J2KFormat::J2K => OPJ_CODEC_J2K,
    J2KFormat::JP2 => OPJ_CODEC_JP2,
    other => {
      return Err(Error::Unsupported(format!(
        "codec format {:?} is not supported for encoding",
        other
      )))
    }
  };
  let mut codec = Codec::new_encoder(codec_format).ok_or_else(|| Error::Encode("failed to create encoder".into()))?;
  let mut params = params;
  let mut image = image.clone();
  if codec.setup_encoder(&mut params, &mut image) != 1 {
    return Err(Error::Encode("setup_encoder failed".into()));
  }
  let writer = SharedVecWriter {
    buf: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
    pos: 0,
  };
  let out_buf = writer.buf.clone();
  let mut stream = Stream::new_output(
    crate::consts::opj::OPJ_J2K_STREAM_CHUNK_SIZE as usize,
    writer,
  );
  if codec.start_compress(&mut image, &mut stream) != 1 {
    return Err(Error::Encode("start_compress failed".into()));
  }
  if codec.encode(&mut stream) != 1 {
    return Err(Error::Encode("encode failed".into()));
  }
  if codec.end_compress(&mut stream) != 1 {
    return Err(Error::Encode("end_compress failed".into()));
  }
  let out = out_buf.borrow().clone();
  Ok(out)
}

/// Internal driver over the (deprecated-for-users) low-level Codec methods.
#[allow(deprecated)]
fn decode_stream(stream: &mut Stream, format: J2KFormat, params: &DecodeParameters) -> Result<Image> {
  let codec_format = match format {
    J2KFormat::J2K => OPJ_CODEC_J2K,
    J2KFormat::JP2 => OPJ_CODEC_JP2,
    other => {
      return Err(Error::Unsupported(format!(
        "codec format {:?} is not supported for decoding",
        other
      )))
    }
  };
  let mut codec = Codec::new_decoder(codec_format).ok_or_else(|| Error::Decode("failed to create decoder".into()))?;

  let mut dparams = opj_dparameters {
    cp_reduce: params.reduce,
    cp_layer: params.layers,
    ..Default::default()
  };
  if params.ignore_pclr_cmap_cdef {
    dparams.flags |= OPJ_DPARAMETERS_IGNORE_PCLR_CMAP_CDEF_FLAG;
  }
  if codec.setup_decoder(&mut dparams) != 1 {
    return Err(Error::Decode("setup_decoder failed".into()));
  }
  if params.strict && codec.decoder_set_strict_mode(1) != 1 {
    return Err(Error::Decode("failed to set strict mode".into()));
  }

  let mut image = codec
    .read_header(stream)
    .ok_or_else(|| Error::Decode("failed to read header".into()))?;

  if let Some(area) = params.decode_area {
    let ret = codec.set_decode_area(
      &mut image,
      area.start_x as i32,
      area.start_y as i32,
      area.end_x as i32,
      area.end_y as i32,
    );
    if ret != 1 {
      return Err(Error::Decode("set_decode_area failed".into()));
    }
  }

  if codec.decode(stream, &mut image) != 1 {
    return Err(Error::Decode("decode failed".into()));
  }
  if codec.end_decompress(stream) != 1 {
    return Err(Error::Decode("end_decompress failed".into()));
  }

  Ok(Image { core: image })
}

/// Detect the format of an in-memory buffer.
pub fn detect_format(buf: &[u8]) -> Option<J2KFormat> {
  crate::detect_format(buf).ok()
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Decodes the bundled sample and round-trips it through the encoder.
  #[test]
  fn decode_sample_and_round_trip() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    let image = Image::from_bytes(&buf).expect("decode sample");
    assert_eq!(image.num_components(), 3);
    assert!(image.width() > 0 && image.height() > 0);
    // Round-trip through the J2K encoder and back.
    let codestream = image
      .encode(
        EncodeParameters::new()
          .num_resolutions(3)
          .num_layers(1)
          .progression_order(ProgressionOrder::Lrcp)
          .comment("api test"),
      )
      .expect("encode");
    assert!(!codestream.is_empty());
    let back = Image::from_bytes(&codestream).expect("decode round-trip");
    assert_eq!(back.width(), image.width());
    assert_eq!(back.height(), image.height());
    assert_eq!(back.num_components(), image.num_components());
    // The reversible transform is lossless: pixels must match.
    for (a, b) in image.components().iter().zip(back.components()) {
      assert_eq!(a.data, b.data);
    }
  }

  #[cfg(feature = "image")]
  #[test]
  fn sample_to_dynamic_image() {
    use core::convert::TryFrom;
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    let image = Image::from_bytes(&buf).expect("decode sample");
    let pixels = image.get_pixels(None).expect("get_pixels");
    assert_eq!((pixels.width, pixels.height), (image.width(), image.height()));
    let dyn_img = ::image::DynamicImage::try_from(&image).expect("convert");
    assert_eq!(dyn_img.width(), image.width());
    assert_eq!(dyn_img.height(), image.height());
  }

  #[test]
  fn decode_area_and_layers() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    let full = Image::from_bytes(&buf).expect("decode full");
    // Decode a 64x64 corner with a resolution reduction.
    let area = DecodeArea::new(0, 0, 64, 64);
    let sub = Image::from_bytes_with(
      &buf,
      DecodeParameters::new()
        .decode_area(Some(area))
        .reduce(1)
        .layers(1)
        .strict(true),
    )
    .expect("decode area");
    assert!(sub.width() <= 64 && sub.height() <= 64);
    assert!(sub.width() > 0 && sub.height() > 0);
    assert_eq!(sub.num_components(), full.num_components());
  }

  /// Encodes random images with the reversible transform and checks the
  /// round trip is lossless, across a few sizes/precisions/component counts.
  #[test]
  fn lossless_round_trip_property() {
    let mut rng: u64 = 0x9e3779b97f4a7c15;
    let mut next = move || {
      rng ^= rng << 13;
      rng ^= rng >> 7;
      rng ^= rng << 17;
      rng
    };
    for (w, h, comps, prec) in [(17, 13, 1, 8), (64, 64, 3, 8), (33, 31, 3, 12)] {
      let mut core = crate::CoreImage {
        x0: 0,
        y0: 0,
        x1: w,
        y1: h,
        numcomps: comps,
        color_space: if comps == 1 {
          crate::OPJ_CLRSPC_GRAY
        } else {
          crate::OPJ_CLRSPC_SRGB
        },
        comps: Vec::new(),
        icc_profile_buf: Vec::new(),
        icc_profile_is_cielab: false,
      };
      for c in 0..comps {
        let dx = if c == 0 { 1 } else { 2 };
        let (cw, ch) = (w.div_ceil(dx), h.div_ceil(dx));
        let mut data = Vec::with_capacity((cw * ch) as usize);
        for _ in 0..cw * ch {
          data.push((next() % (1u64 << prec)) as i32);
        }
        core.comps.push(crate::CoreImageComp {
          dx,
          dy: dx,
          w: cw,
          h: ch,
          x0: 0,
          y0: 0,
          prec,
          bpp: prec,
          sgnd: 0,
          resno_decoded: 0,
          factor: 0,
          alpha: 0,
          data,
        });
      }
      let image = Image { core };
      let codestream = image
        .encode(EncodeParameters::new().num_resolutions(3).num_layers(1))
        .expect("encode");
      let back = Image::from_bytes(&codestream).expect("decode");
      assert_eq!(back.width(), image.width());
      assert_eq!(back.height(), image.height());
      assert_eq!(back.num_components(), comps);
      for (a, b) in image.components().iter().zip(back.components()) {
        assert_eq!(a.data, b.data, "lossless round trip failed for {w}x{h} prec {prec}");
      }
    }
  }

  /// Encode config sweep: every progression order with tiles and layers,
  /// always decoding back losslessly.
  #[test]
  fn encode_config_sweep() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    let image = Image::from_bytes(&buf).expect("decode sample");
    for order in [
      ProgressionOrder::Lrcp,
      ProgressionOrder::Rlcp,
      ProgressionOrder::Rpcl,
      ProgressionOrder::Pcrl,
      ProgressionOrder::Cprl,
    ] {
      let codestream = image
        .encode(
          EncodeParameters::new()
            .num_resolutions(2)
            .num_layers(2)
            .progression_order(order)
            .tile_size(512, 512),
        )
        .expect("encode");
      let back = Image::from_bytes(&codestream).expect("decode");
      assert_eq!(back.width(), image.width(), "order {order:?}");
      assert_eq!(back.height(), image.height(), "order {order:?}");
      for (a, b) in image.components().iter().zip(back.components()) {
        assert_eq!(a.data, b.data, "lossless for order {order:?}");
      }
    }
  }

  #[test]
  fn color_space_round_trip() {
    for cs in [
      ColorSpace::Unknown,
      ColorSpace::Srgb,
      ColorSpace::Gray,
      ColorSpace::Sycc,
      ColorSpace::Eycc,
      ColorSpace::Cmyk,
    ] {
      let c: OPJ_COLOR_SPACE = cs.into();
      let back: ColorSpace = c.into();
      assert_eq!(back, cs);
    }
  }
}

/// `image`-crate interop: pixel extraction and `DynamicImage` conversion
/// (mirrors the `jpeg2k` crate's `get_pixels`).
#[cfg(feature = "image")]
pub mod image_conv {
  use super::*;
  use core::convert::TryFrom;

  /// Pixel data in a canonical 8/16-bit layout.
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum ImagePixelData {
    L8(Vec<u8>),
    La8(Vec<u8>),
    Rgb8(Vec<u8>),
    Rgba8(Vec<u8>),
    L16(Vec<u16>),
    La16(Vec<u16>),
    Rgb16(Vec<u16>),
    Rgba16(Vec<u16>),
  }

  /// Decoded pixels with their colour format.
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub format: ::image::ExtendedColorType,
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
    let len = (comp.w as usize) * (comp.h as usize);
    let data = &comp.data[..len];
    let prec = comp.prec;
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
    let len = (comp.w as usize) * (comp.h as usize);
    let data = &comp.data[..len];
    let prec = comp.prec;
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
          GrayImage::from_vec(width, height, d).expect("pixel buffer length"),
        ),
        ImagePixelData::La8(d) => DynamicImage::ImageLumaA8(
          GrayAlphaImage::from_vec(width, height, d).expect("pixel buffer length"),
        ),
        ImagePixelData::Rgb8(d) => DynamicImage::ImageRgb8(
          RgbImage::from_vec(width, height, d).expect("pixel buffer length"),
        ),
        ImagePixelData::Rgba8(d) => DynamicImage::ImageRgba8(
          RgbaImage::from_vec(width, height, d).expect("pixel buffer length"),
        ),
        ImagePixelData::L16(d) => DynamicImage::ImageLuma16(
          ImageBuffer::from_vec(width, height, d).expect("pixel buffer length"),
        ),
        ImagePixelData::La16(d) => DynamicImage::ImageLumaA16(
          ImageBuffer::from_vec(width, height, d).expect("pixel buffer length"),
        ),
        ImagePixelData::Rgb16(d) => DynamicImage::ImageRgb16(
          ImageBuffer::from_vec(width, height, d).expect("pixel buffer length"),
        ),
        ImagePixelData::Rgba16(d) => DynamicImage::ImageRgba16(
          ImageBuffer::from_vec(width, height, d).expect("pixel buffer length"),
        ),
      })
    }
  }
}

/// Header/decode driver for the codestream index/info and the dump text
/// (the jpeg2k `DumpImage` equivalent).
pub struct DumpImage {
  codec: Codec,
  stream: Stream,
  image: Option<CoreImage>,
}

impl DumpImage {
  #[allow(deprecated)]
  pub fn from_bytes(buf: &[u8]) -> Result<Self> {
    Self::from_bytes_with(buf, DecodeParameters::default())
  }

  pub fn from_bytes_with(buf: &[u8], params: DecodeParameters) -> Result<Self> {
    let format = detect_format(buf)
      .ok_or_else(|| Error::Unsupported("unknown JPEG 2000 format".into()))?;
    let buf = buf.to_vec();
    let len = buf.len() as u64;
    let mut stream = Stream::new_input(
      crate::consts::opj::OPJ_J2K_STREAM_CHUNK_SIZE as usize,
      std::io::Cursor::new(buf),
      len,
    );
    let codec_format = match format {
      J2KFormat::J2K => OPJ_CODEC_J2K,
      J2KFormat::JP2 => OPJ_CODEC_JP2,
      other => {
        return Err(Error::Unsupported(format!(
          "codec format {:?} is not supported",
          other
        )))
      }
    };
    let mut codec = Codec::new_decoder(codec_format)
      .ok_or_else(|| Error::Decode("failed to create decoder".into()))?;
    let mut dparams = opj_dparameters {
      cp_reduce: params.reduce,
      cp_layer: params.layers,
      ..Default::default()
    };
    if params.ignore_pclr_cmap_cdef {
      dparams.flags |= OPJ_DPARAMETERS_IGNORE_PCLR_CMAP_CDEF_FLAG;
    }
    if codec.setup_decoder(&mut dparams) != 1 {
      return Err(Error::Decode("setup_decoder failed".into()));
    }
    #[allow(deprecated)]
    let image = codec
      .read_header(&mut stream)
      .ok_or_else(|| Error::Decode("failed to read header".into()))?;
    Ok(Self {
      codec,
      stream,
      image: Some(image),
    })
  }

  pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
    let buf = std::fs::read(path)?;
    Self::from_bytes(&buf)
  }

  pub fn from_file_with<P: AsRef<std::path::Path>>(path: P, params: DecodeParameters) -> Result<Self> {
    let buf = std::fs::read(path)?;
    Self::from_bytes_with(&buf, params)
  }

  /// Decode the tile data (needed to populate the codestream index with
  /// the tile markers).
  #[allow(deprecated)]
  pub fn decode(&mut self) -> Result<()> {
    let mut image = self.image.take().expect("image is Some");
    if self.codec.decode(&mut self.stream, &mut image) != 1 {
      return Err(Error::Decode("decode failed".into()));
    }
    self.image = Some(image);
    Ok(())
  }

  /// The codestream index (tile-parts, markers, packets).
  pub fn get_codestream_index(&self) -> Option<crate::CodestreamIndex> {
    self.codec.get_cstr_index_owned()
  }

  /// The codestream header info (v2).
  pub fn get_codestream_info(&self) -> Option<crate::CstrInfoV2> {
    self.codec.get_cstr_info_v2()
  }

  /// The dump text for the given info flag.
  pub fn dump(&self, info_flag: i32) -> String {
    let mut out = String::new();
    self.codec.dump_codec(info_flag, &mut out);
    out
  }
}
