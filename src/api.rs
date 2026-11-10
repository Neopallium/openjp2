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

/// Runs a codec operation, converting any internal panic into an `Error`.
///
/// Decoding untrusted data is the one path where a defensive catch_unwind
/// is justified: a decoder bug must fail the decode, never abort the host
/// application.  The one-shot API types are dropped after an error, so the
/// poisoned decoder state cannot be observed.
fn catch_codec_panic<T>(is_decode: bool, f: impl FnOnce() -> Result<T>) -> Result<T> {
  match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
    Ok(result) => result,
    Err(payload) => {
      let detail = payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".to_string());
      if is_decode {
        Err(Error::Decode(format!("internal decoder panic: {detail}")))
      } else {
        Err(Error::Encode(format!("internal encoder panic: {detail}")))
      }
    }
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

/// Optional resource limits for decoding untrusted input.
///
/// These guards exist only in the Rust core API (the C-ABI mirror keeps
/// the upstream behaviour).
///
/// The limits are checked *before* allocations happen: codestream values
/// that prefix a length (JP2 box lengths, marker segment lengths) are
/// verified against the remaining stream bytes first, and the
/// image/tile/metadata dimensions are checked against the limits before
/// any buffer is sized from them.
///
/// `max_mem_alloc == 0` means "the built-in default ceiling" (1 GiB) so a
/// corrupt header cannot drive a multi-gigabyte allocation out of a tiny
/// input; set it explicitly for larger images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DecodeLimits {
  /// Maximum decoded image width in pixels (0 = unlimited).
  pub max_image_width: u32,
  /// Maximum decoded image height in pixels (0 = unlimited).
  pub max_image_height: u32,
  /// Maximum tile area in pixels (tile width x tile height, 0 = unlimited).
  pub max_tile_pixels: u64,
  /// Upper bound on the memory the decode may allocate, in bytes.
  /// 0 = the built-in default ceiling (1 GiB); set explicitly for larger
  /// images.  Covers the dominant buffers: the per-component image and
  /// tile data.
  pub max_mem_alloc: u64,
  /// Maximum length of a single metadata element in bytes - ICC colour
  /// profiles, JP2 header-box payloads, comments, etc. (0 = unlimited).
  pub max_metadata_len: usize,
}

/// Built-in memory ceiling when `DecodeLimits::max_mem_alloc` is left at 0.
pub const DEFAULT_DECODE_MEM_LIMIT: u64 = 1 << 30;

impl DecodeLimits {
  pub(crate) fn effective_max_mem(&self) -> u64 {
    if self.max_mem_alloc == 0 {
      DEFAULT_DECODE_MEM_LIMIT
    } else {
      self.max_mem_alloc
    }
  }
}

impl DecodeLimits {
  pub const fn new() -> Self {
    Self {
      max_image_width: 0,
      max_image_height: 0,
      max_tile_pixels: 0,
      max_mem_alloc: 0,
      max_metadata_len: 0,
    }
  }

  /// Cap the decoded image dimensions (either limit may be 0 = unlimited).
  pub fn max_image_size(mut self, width: u32, height: u32) -> Self {
    self.max_image_width = width;
    self.max_image_height = height;
    self
  }

  /// Cap the tile area in pixels (0 = unlimited).
  pub fn max_tile_pixels(mut self, pixels: u64) -> Self {
    self.max_tile_pixels = pixels;
    self
  }

  /// Cap the decode memory budget in bytes (0 = the built-in 1 GiB default).
  pub fn max_mem_alloc(mut self, bytes: u64) -> Self {
    self.max_mem_alloc = bytes;
    self
  }

  /// Cap the length of a single metadata element in bytes (0 = unlimited).
  pub fn max_metadata_len(mut self, bytes: usize) -> Self {
    self.max_metadata_len = bytes;
    self
  }
}

/// Decoder settings, built with the fluent builder pattern.
///
/// Defaults: full quality (reduce 0, all layers), whole image, and
/// LENIENT decoding (`strict == false`), like the C tools' `-allow-
/// partial`: partial or slightly malformed codestreams decode as far as
/// they can instead of being rejected.  (The C ABI and the CLI tools
/// keep the C library's strict-by-default behaviour.)
#[derive(Debug, Clone, Default)]
pub struct DecodeParameters {
  pub reduce: u32,
  pub layers: u32,
  pub strict: bool,
  pub decode_area: Option<DecodeArea>,
  pub ignore_pclr_cmap_cdef: bool,
  /// Optional resource limits (see `DecodeLimits`).  All-zero except
  /// `max_mem_alloc` = unlimited; `max_mem_alloc == 0` selects the built-in
  /// 1 GiB ceiling (`DEFAULT_DECODE_MEM_LIMIT`).
  pub limits: DecodeLimits,
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

  /// Apply resource limits to the decode (see `DecodeLimits`).
  pub fn limits(mut self, limits: DecodeLimits) -> Self {
    self.limits = limits;
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
  ///
  /// Layers beyond the 100 supported by the format are silently ignored
  /// (debug builds assert).
  pub fn layer_rate(mut self, rate: f32) -> Self {
    let n = self.inner.tcp_numlayers as usize;
    debug_assert!(n < 100, "layer_rate past the 100-layer limit");
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
      data.chunks_exact(stride).map(|px| px[chan] as u16).collect()
    };
    let (numcomps, color_space, planes): (u32, crate::OPJ_COLOR_SPACE, Vec<Vec<u16>>) = match img {
      ImageLuma8(g) => (
        1,
        crate::OPJ_CLRSPC_GRAY,
        vec![g.as_raw().iter().map(|&v| v as u16).collect()],
      ),
      ImageLumaA8(g) => (
        2,
        crate::OPJ_CLRSPC_GRAY,
        vec![widen(g.as_raw(), 2, 0), widen(g.as_raw(), 2, 1)],
      ),
      ImageRgb8(r) => (
        3,
        crate::OPJ_CLRSPC_SRGB,
        vec![widen(r.as_raw(), 3, 0), widen(r.as_raw(), 3, 1), widen(r.as_raw(), 3, 2)],
      ),
      ImageRgba8(r) => (
        4,
        crate::OPJ_CLRSPC_SRGB,
        vec![
          widen(r.as_raw(), 4, 0),
          widen(r.as_raw(), 4, 1),
          widen(r.as_raw(), 4, 2),
          widen(r.as_raw(), 4, 3),
        ],
      ),
      ImageLuma16(g) => (1, crate::OPJ_CLRSPC_GRAY, vec![g.as_raw().to_vec()]),
      ImageLumaA16(g) => (
        2,
        crate::OPJ_CLRSPC_GRAY,
        vec![
          g.as_raw().as_chunks::<2>().0.iter().map(|c| c[0]).collect(),
          g.as_raw().as_chunks::<2>().0.iter().map(|c| c[1]).collect(),
        ],
      ),
      ImageRgb16(r) => (
        3,
        crate::OPJ_CLRSPC_SRGB,
        vec![
          r.as_raw().as_chunks::<3>().0.iter().map(|c| c[0]).collect(),
          r.as_raw().as_chunks::<3>().0.iter().map(|c| c[1]).collect(),
          r.as_raw().as_chunks::<3>().0.iter().map(|c| c[2]).collect(),
        ],
      ),
      ImageRgba16(r) => (
        4,
        crate::OPJ_CLRSPC_SRGB,
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
          crate::OPJ_CLRSPC_SRGB,
          vec![
            widen(rgba.as_raw(), 4, 0),
            widen(rgba.as_raw(), 4, 1),
            widen(rgba.as_raw(), 4, 2),
            widen(rgba.as_raw(), 4, 3),
          ],
        )
      }
    };
    let prec = if planes.iter().any(|p| p.iter().any(|&v| v > 255)) {
      16
    } else {
      8
    };
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
          .map(|data| crate::CoreImageComp {
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
            alpha: 0,
            data: data.into_iter().map(i32::from).collect(),
          })
          .collect(),
        icc_profile_buf: Vec::new(),
        icc_profile_is_cielab: false,
      },
    }
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

  /// Encode the image into any `Write + Seek` sink (the encoder seeks
  /// back to patch marker lengths, so a plain `Write` is not enough).
  pub fn encode_to_writer<W: std::io::Write + std::io::Seek + 'static>(
    &self,
    params: EncodeParameters,
    writer: W,
  ) -> Result<()> {
    encode_image_to_writer(&self.core, params.into_inner(), J2KFormat::J2K, writer)
  }
}

#[allow(deprecated)]
fn encode_image_to_writer<W: std::io::Write + std::io::Seek + 'static>(
  image: &CoreImage,
  params: crate::EncoderParameters,
  format: J2KFormat,
  writer: W,
) -> Result<()> {
  catch_codec_panic(false, || encode_image_to_writer_inner(image, params, format, writer))
}

#[allow(deprecated)]
fn encode_image_to_writer_inner<W: std::io::Write + std::io::Seek + 'static>(
  image: &CoreImage,
  params: crate::EncoderParameters,
  format: J2KFormat,
  writer: W,
) -> Result<()> {
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
  let mut codec = Codec::new_encoder(codec_format)
    .ok_or_else(|| Error::Encode("failed to create encoder".into()))?;
  let mut params = params;
  let mut image = image.clone();
  if codec.setup_encoder(&mut params, &mut image) != 1 {
    return Err(Error::Encode("setup_encoder failed".into()));
  }
  let mut stream = Stream::new_output(
    crate::consts::opj::OPJ_J2K_STREAM_CHUNK_SIZE as usize,
    writer,
  );
  if codec.start_compress(&mut image, &mut stream) != 1 {
    return Err(Error::Encode("start_compress failed".into()));
  }
  /* Multi-tile images encode concurrently (byte-identical fallback). */
  #[cfg(feature = "rayon")]
  let parallel_ok =
    codec.num_tiles() > 1 && codec.encode_parallel(&mut stream) == Some(true);
  #[cfg(feature = "rayon")]
  let encode_ok = if parallel_ok {
    true
  } else {
    codec.encode(&mut stream) == 1
  };
  #[cfg(not(feature = "rayon"))]
  let encode_ok = codec.encode(&mut stream) == 1;
  if !encode_ok {
    return Err(Error::Encode("encode failed".into()));
  }
  if codec.end_compress(&mut stream) != 1 {
    return Err(Error::Encode("end_compress failed".into()));
  }
  Ok(())
}

/// `Write + Seek` adapter collecting into a shared `Vec<u8>`; the j2k
/// encoder seeks back to patch marker lengths, so the position must be
/// honoured.
/// Internal driver over the (deprecated-for-users) low-level Codec methods.
#[allow(deprecated)]
fn encode_image(
  image: &CoreImage,
  params: crate::EncoderParameters,
  format: J2KFormat,
) -> Result<Vec<u8>> {
  catch_codec_panic(false, || encode_image_inner(image, params, format))
}

#[allow(deprecated)]
fn encode_image_inner(
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
  let writer = crate::stream::SharedVecWriter {
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
  /* Concurrent tile encoding (rayon): only multi-tile images; any failure
  falls back to the sequential encode below, which yields identical bytes. */
  #[cfg(feature = "rayon")]
  let parallel_ok = codec.num_tiles() > 1 && codec.encode_parallel(&mut stream) == Some(true);
  #[cfg(feature = "rayon")]
  let encode_ok = if parallel_ok {
    true
  } else {
    codec.encode(&mut stream) == 1
  };
  #[cfg(not(feature = "rayon"))]
  let encode_ok = codec.encode(&mut stream) == 1;
  if !encode_ok {
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
  catch_codec_panic(true, || decode_stream_inner(stream, format, params))
}

/// Internal driver over the (deprecated-for-users) low-level Codec methods.
#[allow(deprecated)]
fn decode_stream_inner(
  stream: &mut Stream,
  format: J2KFormat,
  params: &DecodeParameters,
) -> Result<Image> {
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
  if codec.set_decode_limits(params.limits) != 1 {
    return Err(Error::Decode("set_decode_limits failed".into()));
  }
  /* The C library default is strict; the builder flag must control the
  mode explicitly so strict(false) actually relaxes it. */
  if codec.decoder_set_strict_mode(params.strict as crate::OPJ_BOOL) != 1 {
    return Err(Error::Decode("failed to set strict mode".into()));
  }

  let mut image = codec
    .read_header(stream)
    .ok_or_else(|| Error::Decode("failed to read header".into()))?;

  /* Concurrent tile decode (rayon): whole-image, full-quality,
  full-resolution decodes of multi-tile codestreams.  A layer limit or a
  resolution reduction leaves coefficient regions unwritten, which the
  sequential path (like the C reference) reads as the PREVIOUS tile's
  leftovers - a fresh per-tile worker cannot reproduce that, so those
  cases stay on the sequential path.  Any failure also falls back to the
  sequential path below, so the result is always identical. */
  #[cfg(feature = "rayon")]
  if params.decode_area.is_none() && params.layers == 0 && params.reduce == 0 {
    if let Some(core) = codec.decode_parallel(stream, &image, 0, 0, params.strict) {
      return Ok(Image { core });
    }
  }

  if let Some(area) = params.decode_area {
    #[cfg(feature = "rayon")]
    let full_header = image.clone();
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
    /* Concurrent tile decode of the intersecting tiles (rayon); the same
    unwritten-coefficient caveat as above limits it to full-quality
    full-resolution decodes. */
    #[cfg(feature = "rayon")]
    if params.reduce == 0 && params.layers == 0 {
      if let Some(core) = codec.decode_parallel_area(stream, &full_header, &image) {
        return Ok(Image { core });
      }
    }
  }

  /* Progressive-download support: in lenient mode a codestream truncated
  AFTER the last tile's data (missing EOC) delivers the fully decoded image
  instead of an error.  Strict mode keeps the hard failure.  Note the scope:
  truncation MID-TILE (or a used-components failure) may still return the
  header image with empty component data - the C behaves the same way, and
  the safe API documents this partial-decode contract rather than silently
  inventing pixels. */
  if codec.decode(stream, &mut image) != 1 {
    if params.strict {
      return Err(Error::Decode("decode failed".into()));
    }
    return Ok(Image { core: image });
  }
  if codec.end_decompress(stream) != 1 && params.strict {
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

  /// 16-bit pixel extraction through the `image`-crate interop.
  #[cfg(feature = "image")]
  #[test]
  fn get_pixels_16bit() {
    let (w, h) = (64u32, 48u32);
    let mut core = crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 1,
      color_space: crate::OPJ_CLRSPC_GRAY,
      comps: Vec::new(),
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    let mut data = Vec::with_capacity((w * h) as usize);
    for i in 0..w * h {
      data.push((i % 4096) as i32);
    }
    core.comps.push(crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 12,
      bpp: 12,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data,
    });
    let image = Image { core };
    let pixels = image.get_pixels(None).expect("get_pixels");
    assert_eq!(pixels.format, ::image::ExtendedColorType::L16);
    assert_eq!(pixels.width, w);
    assert_eq!(pixels.height, h);
    match pixels.data {
      super::image_conv::ImagePixelData::L16(v) => {
        assert_eq!(v.len(), (w * h) as usize);
        // 12-bit 0..4095 maps to 16-bit 0..65535 (proportional)
        assert_eq!(v[0], 0);
        assert!(v[1] > 0 && v[1] < 65535);
      }
      other => panic!("unexpected pixel format {:?}", other),
    }
    // And the DynamicImage conversion path.
    use core::convert::TryFrom;
    let dyn_img = ::image::DynamicImage::try_from(&image).expect("convert");
    assert_eq!((dyn_img.width(), dyn_img.height()), (w, h));
  }

  /// Parallel-vs-serial encode byte-identity over the test corpus (runs
  /// when OPJ_DATA_ROOT is set): every input that decodes must re-encode
  /// to the identical codestream through both paths.
  #[cfg(feature = "rayon")]
  #[test]
  fn parallel_encode_matches_serial_on_corpus() {
    let root = match std::env::var_os("OPJ_DATA_ROOT") {
      Some(r) => std::path::PathBuf::from(r),
      None => return,
    };
    let input = root.join("input").join("nonregression");
    if !input.is_dir() {
      return;
    }
    let mut checked = 0usize;
    let mut entries: Vec<_> = std::fs::read_dir(&input)
      .expect("read_dir")
      .filter_map(|e| e.ok())
      .map(|e| e.path())
      .filter(|p| {
        matches!(
          p.extension().and_then(|e| e.to_str()),
          Some("j2k") | Some("j2c") | Some("jp2")
        )
      })
      .collect();
    entries.sort();
    for path in entries {
      let buf = std::fs::read(&path).expect("read input");
      let Ok(image) = Image::from_bytes(&buf) else {
        continue;
      };
      let params = EncodeParameters::new().num_resolutions(4).tile_size(256, 256);
      let Ok(par) = image.encode(params.clone()) else {
        continue;
      };
      #[allow(deprecated)]
      let serial = {
        let mut codec = Codec::new_encoder(OPJ_CODEC_J2K).unwrap();
        let mut low = params.clone().into_inner();
        let mut img = image.into_core_image();
        if codec.setup_encoder(&mut low, &mut img) != 1 {
          continue;
        }
        let buf2 = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        {
          let writer = crate::stream::SharedVecWriter {
            buf: buf2.clone(),
            pos: 0,
          };
          let mut stream = Stream::new_output(
            crate::consts::opj::OPJ_J2K_STREAM_CHUNK_SIZE as usize,
            writer,
          );
          assert_eq!(codec.start_compress(&mut img, &mut stream), 1);
          assert_eq!(codec.encode(&mut stream), 1);
          assert_eq!(codec.end_compress(&mut stream), 1);
          drop(stream);
        }
        std::rc::Rc::try_unwrap(buf2)
          .ok()
          .map(|b| b.into_inner())
          .expect("unique handle")
      };
      assert_eq!(par, serial, "{}: parallel/serial encode mismatch", path.display());
      checked += 1;
    }
    assert!(checked > 10, "expected a corpus of re-encodable inputs");
  }

  /// The concurrent-tile encode must produce byte-identical codestreams
  /// to the sequential low-level encode.
  #[cfg(feature = "rayon")]
  #[test]
  fn parallel_tile_encode_byte_identical() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    let image = Image::from_bytes(&buf).expect("decode");
    let params = EncodeParameters::new()
      .num_resolutions(3)
      .num_layers(2)
      .tile_size(512, 512)
      .progression_order(ProgressionOrder::Lrcp);
    let par = image.encode(params.clone()).expect("parallel encode");
    #[allow(deprecated)]
    let serial = {
      let mut codec = Codec::new_encoder(OPJ_CODEC_J2K).unwrap();
      let mut low = params.into_inner();
      let mut img = image.into_core_image();
      assert_eq!(codec.setup_encoder(&mut low, &mut img), 1);
      let buf2 = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
      {
        let writer = crate::stream::SharedVecWriter {
          buf: buf2.clone(),
          pos: 0,
        };
        let mut stream = Stream::new_output(
          crate::consts::opj::OPJ_J2K_STREAM_CHUNK_SIZE as usize,
          writer,
        );
        assert_eq!(codec.start_compress(&mut img, &mut stream), 1);
        assert_eq!(codec.encode(&mut stream), 1);
        assert_eq!(codec.end_compress(&mut stream), 1);
        drop(stream);
      }
      std::rc::Rc::try_unwrap(buf2).ok().map(|b| b.into_inner()).expect("unique handle")
    };
    assert_eq!(par, serial, "parallel/serial encode mismatch");
  }

  /// DumpImage + the codestream index/info accessors.
  #[test]
  fn dump_image_and_index() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    let mut dump = DumpImage::from_bytes(&buf).expect("dump image");
    dump.decode().expect("decode");
    let index = dump.get_codestream_index().expect("index");
    assert_eq!(index.tile_indices().len(), 1);
    assert!(!index.markers().is_empty());
    let info = dump.get_codestream_info().expect("info");
    assert_eq!((info.tw(), info.th()), (1, 1));
    assert_eq!(info.nbcomps(), 3);
    let text = dump.dump(
      (crate::consts::opj::OPJ_IMG_INFO
        | crate::consts::opj::OPJ_J2K_MH_INFO
        | crate::consts::opj::OPJ_J2K_MH_IND) as i32,
    );
    // The JP2 dump prints the boxes and the codestream header info.
    assert!(text.contains("Codestream info"));
  }

  /// Fixed-quality layer rates via the builder (rate allocation).
  #[test]
  fn encode_with_layer_rates() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    let image = Image::from_bytes(&buf).expect("decode");
    let codestream = image
      .encode(
        EncodeParameters::new()
          .num_resolutions(3)
          .layer_rate(50.0)
          .layer_rate(10.0),
      )
      .expect("encode");
    let back = Image::from_bytes(&codestream).expect("decode");
    assert_eq!((back.width(), back.height()), (image.width(), image.height()));
    assert!(back.components()[0].data.iter().any(|v| *v != 0));
  }

  /// Encode to an arbitrary writer and decode the bytes back.
  #[test]
  fn encode_to_writer_round_trip() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    let image = Image::from_bytes(&buf).expect("decode");
    let file = std::fs::File::create("/tmp/openjp2_encode_to_writer.j2k").expect("create");
    image
      .encode_to_writer(
        EncodeParameters::new().num_resolutions(3).tile_size(512, 512),
        file,
      )
      .expect("encode to writer");
    let back = Image::from_file("/tmp/openjp2_encode_to_writer.j2k").expect("decode");
    let _ = std::fs::remove_file("/tmp/openjp2_encode_to_writer.j2k");
    assert_eq!((back.width(), back.height()), (image.width(), image.height()));
    for (a, b) in image.components().iter().zip(back.components()) {
      assert_eq!(a.data, b.data, "writer round trip");
    }
  }

  /// Build an image from an image-crate DynamicImage and round-trip it.
  #[cfg(feature = "image")]
  #[test]
  fn from_dynamic_image_round_trip() {
    let mut img = ::image::RgbaImage::new(96, 64);
    for (i, px) in img.pixels_mut().enumerate() {
      *px = ::image::Rgba([
        (i % 251) as u8,
        ((i * 7) % 233) as u8,
        ((i * 13) % 199) as u8,
        255,
      ]);
    }
    let dyn_img = ::image::DynamicImage::ImageRgba8(img);
    let image = Image::from_dynamic_image(&dyn_img);
    assert_eq!(image.num_components(), 4);
    assert_eq!((image.width(), image.height()), (96, 64));
    // Round-trip through the encoder (lossless 5-3).
    let codestream = image
      .encode(EncodeParameters::new().num_resolutions(3))
      .expect("encode");
    let back = Image::from_bytes(&codestream).expect("decode");
    for (a, b) in image.components().iter().zip(back.components()) {
      assert_eq!(a.data, b.data);
    }
    // And back into pixels.
    let pixels = back.get_pixels(None).expect("get_pixels");
    assert_eq!(pixels.format, ::image::ExtendedColorType::Rgba8);
  }

  /// Grey+alpha and RGB+alpha pixel extraction (image-crate interop).
  #[cfg(feature = "image")]
  #[test]
  fn get_pixels_alpha() {
    // 2 components, the second marked as alpha (cdef typ 1).
    let (w, h) = (32u32, 32u32);
    let mut core = crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 2,
      color_space: crate::OPJ_CLRSPC_GRAY,
      comps: Vec::new(),
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    for c in 0..2 {
      core.comps.push(crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: if c == 1 { 1 } else { 0 },
        data: vec![c * 200; (w * h) as usize],
      });
    }
    let image = Image { core };
    let pixels = image.get_pixels(None).expect("get_pixels");
    assert_eq!(pixels.format, ::image::ExtendedColorType::La8);
    assert_eq!(pixels.width, w);
    assert_eq!(pixels.height, h);
    // A single component with a default alpha fill.
    let gray = Image {
      core: crate::CoreImage {
        x0: 0,
        y0: 0,
        x1: w,
        y1: h,
        numcomps: 1,
        color_space: crate::OPJ_CLRSPC_GRAY,
        comps: vec![crate::CoreImageComp {
          dx: 1,
          dy: 1,
          w,
          h,
          x0: 0,
          y0: 0,
          prec: 8,
          bpp: 8,
          sgnd: 0,
          resno_decoded: 0,
          factor: 0,
          alpha: 0,
          data: vec![7; (w * h) as usize],
        }],
        icc_profile_buf: Vec::new(),
        icc_profile_is_cielab: false,
      },
    };
    let filled = gray.get_pixels(Some(255)).expect("get_pixels with alpha");
    assert_eq!(filled.format, ::image::ExtendedColorType::La8);
    match filled.data {
      super::image_conv::ImagePixelData::La8(v) => {
        assert_eq!(v.len(), (w * h * 2) as usize);
        assert_eq!(&v[0..2], &[7, 255]);
      }
      other => panic!("unexpected {:?}", other),
    }
  }

  /// Code-block sizes and comment markers through the builder.
  #[test]
  fn encode_with_codeblock_sizes_and_comment() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    let image = Image::from_bytes(&buf).expect("decode");
    let codestream = image
      .encode(
        EncodeParameters::new()
          .num_resolutions(3)
          .codeblock_width(16)
          .codeblock_height(16)
          .comment("openjp2 test comment"),
      )
      .expect("encode");
    let back = Image::from_bytes(&codestream).expect("decode");
    assert_eq!((back.width(), back.height()), (image.width(), image.height()));
    for (a, b) in image.components().iter().zip(back.components()) {
      assert_eq!(a.data, b.data, "lossless with 16x16 code-blocks");
    }
  }

  /// The default codestream comment identifies the port itself; the
  /// `openjpeg-version-string` feature switches it to the upstream
  /// string (checked by the encode byte-identity gates).
  #[test]
  fn default_codestream_comment_is_ours() {
    let comment = crate::openjpeg::default_codestream_comment();
    #[cfg(feature = "openjpeg-version-string")]
    assert!(comment.starts_with("Created by OpenJPEG version "));
    #[cfg(not(feature = "openjpeg-version-string"))]
    {
      assert!(comment.starts_with("Created by "));
      assert!(comment.contains(env!("CARGO_PKG_NAME")));
      assert!(comment.contains(env!("CARGO_PKG_VERSION")));
      assert!(!comment.contains("OpenJPEG version"));
    }
  }

  /// Decoding garbage must return an error, never panic.
  #[test]
  fn decode_garbage_errors() {
    assert!(Image::from_bytes(b"not a jpeg2000 file at all").is_err());
    let mut data = vec![0xFFu8, 0x4F]; // SOC only
    data.extend([0u8; 64]);
    assert!(Image::from_bytes(&data).is_err());
  }

  /// The irreversible 9-7 transform: a lossy encode/decode round trip.
  #[test]
  fn irreversible_encode_round_trip() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    let image = Image::from_bytes(&buf).expect("decode");
    let codestream = image
      .encode(
        EncodeParameters::new()
          .irreversible(true)
          .num_resolutions(4)
          .num_layers(1)
          .tile_size(512, 512),
      )
      .expect("encode");
    let back = Image::from_bytes(&codestream).expect("decode");
    assert_eq!((back.width(), back.height()), (image.width(), image.height()));
    assert_eq!(back.num_components(), image.num_components());
    // Lossy, but the pixels must be close (9-7 with one layer).
    for (a, b) in image.components().iter().zip(back.components()) {
      let max_delta = a
        .data
        .iter()
        .zip(&b.data)
        .map(|(x, y)| (x - y).unsigned_abs())
        .max()
        .unwrap_or(0);
      assert!(max_delta < 64, "9-7 round trip drifted by {}", max_delta);
    }
  }

  /// Lenient mode returns a partial image for truncated codestreams instead
  /// of an error (progressive download), and the decoder limits reject
  /// oversized geometry before any allocation.
  #[test]
  fn truncation_and_limits() {
    let mut rng: u64 = 0x243f6a8885a308d3;
    let mut next = move || {
      rng ^= rng << 13;
      rng ^= rng >> 7;
      rng ^= rng << 17;
      rng
    };
    // Build a small valid codestream with the encoder.
    let (w, h) = (96u32, 64u32);
    let mut core = crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 3,
      color_space: crate::OPJ_CLRSPC_SRGB,
      comps: Vec::new(),
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    for _c in 0..3u32 {
      let mut data = Vec::with_capacity((w * h) as usize);
      for _ in 0..w * h {
        data.push((next() % 256) as i32);
      }
      core.comps.push(crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data,
      });
    }
    let image = Image { core };
    let codestream = image
      .encode(EncodeParameters::new().num_resolutions(2).num_layers(1))
      .expect("encode");

    // 1. Full decode works.
    let full = Image::from_bytes(&codestream).expect("full decode");
    assert_eq!(full.width(), w);
    assert_eq!(full.height(), h);

    // 2. Strict mode rejects a truncated codestream; lenient mode returns the
    // partial image with the right dimensions (and component count).
    let half = &codestream[..codestream.len() / 2];
    assert!(Image::from_bytes_with(
      half,
      DecodeParameters::new().strict(true)
    )
    .is_err());
    let partial = Image::from_bytes_with(half, DecodeParameters::new().strict(false))
      .expect("lenient partial decode");
    assert_eq!(partial.width(), w);
    assert_eq!(partial.height(), h);
    assert_eq!(partial.num_components(), 3);

    // 3. A codestream cut inside the header (after SIZ) still yields the
    // dimensions in lenient mode.
    let siz_end = {
      // SIZ marker: FF 51, then size; find it in the first 200 bytes.
      let pos = codestream
        .windows(2)
        .position(|win| win == [0xff, 0x51])
        .expect("SIZ marker");
      let l = u16::from_be_bytes([codestream[pos + 2], codestream[pos + 3]]) as usize;
      pos + 2 + l
    };
    let header_only = &codestream[..siz_end + 8];
    let partial_hdr = Image::from_bytes_with(header_only, DecodeParameters::new().strict(false))
      .expect("lenient header-only decode");
    assert_eq!(partial_hdr.width(), w);
    assert_eq!(partial_hdr.height(), h);
    assert!(Image::from_bytes_with(header_only, DecodeParameters::new().strict(true)).is_err());

    // 4. Decoder limits: image size, tile area, and memory budget all reject
    // the codestream before decoding.
    assert!(Image::from_bytes_with(
      &codestream,
      DecodeParameters::new().limits(DecodeLimits::new().max_image_size(w - 1, h)),
    )
    .is_err());
    assert!(Image::from_bytes_with(
      &codestream,
      DecodeParameters::new().limits(DecodeLimits::new().max_image_size(w, h - 1)),
    )
    .is_err());
    assert!(Image::from_bytes_with(
      &codestream,
      DecodeParameters::new().limits(DecodeLimits::new().max_tile_pixels(10)),
    )
    .is_err());
    assert!(Image::from_bytes_with(
      &codestream,
      DecodeParameters::new().limits(DecodeLimits::new().max_mem_alloc(1024)),
    )
    .is_err());
    // ... and generous limits accept it.
    let ok = Image::from_bytes_with(
      &codestream,
      DecodeParameters::new().limits(
        DecodeLimits::new()
          .max_image_size(w, h)
          .max_tile_pixels(w as u64 * h as u64)
          .max_mem_alloc(64 * 1024 * 1024),
      ),
    )
    .expect("decode within limits");
    assert_eq!(ok.width(), w);
  }

  /// The concurrent-tile decode honours a resolution reduction.
  #[cfg(feature = "rayon")]
  #[test]
  fn parallel_tile_decode_with_reduce() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    let image = Image::from_bytes(&buf).expect("decode");
    let codestream = image
      .encode(EncodeParameters::new().num_resolutions(4).tile_size(512, 512))
      .expect("encode");
    let reduced = Image::from_bytes_with(&codestream, DecodeParameters::new().reduce(2))
      .expect("decode reduced");
    /* The image bounds stay full-dim; the component planes are reduced. */
    let c = &reduced.components()[0];
    assert_eq!(
      (c.w, c.h),
      (image.width().div_ceil(4), image.height().div_ceil(4))
    );
    assert!(reduced.components()[0].data.iter().any(|v| *v != 0));
  }

  /// The concurrent-tile decode honours a decode-area request (only the
  /// intersecting tiles are decoded).
  #[cfg(feature = "rayon")]
  #[test]
  fn parallel_tile_decode_with_area() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    // The sample is single-tile; build a multi-tile codestream first.
    let image = Image::from_bytes(&buf).expect("decode");
    let codestream = image
      .encode(EncodeParameters::new().num_resolutions(2).tile_size(512, 512))
      .expect("encode");
    /* Tile-aligned area: exercises the parallel area path (the partial
    one below falls back to the sequential decode). */
    let aligned = DecodeArea::new(0, 0, 1024, 1024);
    let sub = Image::from_bytes_with(
      &codestream,
      DecodeParameters::new().decode_area(Some(aligned)),
    )
    .expect("decode aligned area");
    assert_eq!((sub.width(), sub.height()), (1024, 1024));
    assert!(sub.components()[0].data.iter().any(|v| *v != 0));
    /* Partial area: sequential fallback, same API surface. */
    let partial = DecodeArea::new(64, 64, 900, 700);
    let sub2 = Image::from_bytes_with(
      &codestream,
      DecodeParameters::new().decode_area(Some(partial)),
    )
    .expect("decode partial area");
    assert_eq!((sub2.width(), sub2.height()), (836, 636));
    assert!(sub2.components()[0].data.iter().any(|v| *v != 0));
  }

  /// The concurrent-tile decode honours a quality-layer limit (the
  /// reduce cases stay on the serial path).
  #[cfg(feature = "rayon")]
  #[test]
  fn parallel_tile_decode_with_layers() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
    let full = Image::from_bytes(&buf).expect("decode");
    let limited = Image::from_bytes_with(&buf, DecodeParameters::new().layers(1))
      .expect("decode with layer limit");
    assert_eq!((limited.width(), limited.height()), (full.width(), full.height()));
    // The first layer alone must decode (a lossy rendition of the image).
    assert!(limited.components()[0].data.iter().any(|v| *v != 0));
  }

  /// Parallel-vs-serial byte identity over the real test corpus (runs
  /// when OPJ_DATA_ROOT is set, e.g. in the dev gates): every file that
  /// decodes must produce the same pixels through the parallel one-shot
  /// API and the sequential low-level Codec API.
  #[cfg(feature = "rayon")]
  #[test]
  fn parallel_matches_serial_on_corpus() {
    let root = match std::env::var_os("OPJ_DATA_ROOT") {
      Some(r) => std::path::PathBuf::from(r),
      None => return, /* no test data -> skip */
    };
    let input = root.join("input").join("nonregression");
    if !input.is_dir() {
      return;
    }
    let mut checked = 0usize;
    let mut entries: Vec<_> = std::fs::read_dir(&input)
      .expect("read_dir")
      .filter_map(|e| e.ok())
      .map(|e| e.path())
      .filter(|p| {
        matches!(
          p.extension().and_then(|e| e.to_str()),
          Some("j2k") | Some("j2c") | Some("jp2") | Some("jhc") | Some("jph")
        )
      })
      .collect();
    let conformance = root.join("input").join("conformance");
    if conformance.is_dir() {
      entries.extend(
        std::fs::read_dir(&conformance)
          .expect("read_dir")
          .filter_map(|e| e.ok())
          .map(|e| e.path())
          .filter(|p| {
            matches!(
              p.extension().and_then(|e| e.to_str()),
              Some("j2k") | Some("j2c") | Some("jp2") | Some("jhc") | Some("jph")
            )
          }),
      );
    }
    entries.sort();
    for path in entries {
      let buf = std::fs::read(&path).expect("read input");
      #[allow(deprecated)]
      let serial: Option<crate::CoreImage> = (|| {
        let fmt = match crate::detect_format(&buf) {
          Ok(f) => f,
          Err(_) => return None,
        };
        let c = if fmt == J2KFormat::JP2 {
          OPJ_CODEC_JP2
        } else {
          OPJ_CODEC_J2K
        };
        let mut codec = Codec::new_decoder(c)?;
        let mut dp = opj_dparameters::default();
        if codec.setup_decoder(&mut dp) != 1 {
          return None;
        }
        /* The safe API defaults to lenient decoding (DecodeParameters
        ::default().strict == false); mirror it. */
        if codec.decoder_set_strict_mode(0) != 1 {
          return None;
        }
        let mut stream = Stream::new_input(
          crate::consts::opj::OPJ_J2K_STREAM_CHUNK_SIZE as usize,
          std::io::Cursor::new(buf.clone()),
          buf.len() as u64,
        );
        let mut img = codec.read_header(&mut stream)?;
        if codec.decode(&mut stream, &mut img) != 1 {
          return None;
        }
        if codec.end_decompress(&mut stream) != 1 {
          return None;
        }
        Some(img)
      })();
      let par = Image::from_bytes(&buf).ok();
      // A parallel-vs-serial comparison for an arbitrary parameter set:
      // both paths must succeed/fail together and produce identical data.
      let compare = |params: &DecodeParameters, label: &str| {
        #[allow(deprecated)]
        let serial: Option<crate::CoreImage> = (|| {
          let fmt = match crate::detect_format(&buf) {
            Ok(f) => f,
            Err(_) => return None,
          };
          let c = if fmt == J2KFormat::JP2 {
            OPJ_CODEC_JP2
          } else {
            OPJ_CODEC_J2K
          };
          let mut codec = Codec::new_decoder(c)?;
          let mut dp = opj_dparameters {
            cp_layer: params.layers,
            cp_reduce: params.reduce,
            ..Default::default()
          };
          if codec.setup_decoder(&mut dp) != 1 {
            return None;
          }
          if codec.decoder_set_strict_mode(params.strict as crate::OPJ_BOOL) != 1 {
            return None;
          }
          let mut stream = Stream::new_input(
            crate::consts::opj::OPJ_J2K_STREAM_CHUNK_SIZE as usize,
            std::io::Cursor::new(buf.clone()),
            buf.len() as u64,
          );
          let mut img = codec.read_header(&mut stream)?;
          if let Some(area) = params.decode_area {
            if codec.set_decode_area(
              &mut img,
              area.start_x as i32,
              area.start_y as i32,
              area.end_x as i32,
              area.end_y as i32,
            ) != 1
            {
              return None;
            }
          }
          if codec.decode(&mut stream, &mut img) != 1 {
            return None;
          }
          if codec.end_decompress(&mut stream) != 1 {
            return None;
          }
          Some(img)
        })();
        let par = Image::from_bytes_with(&buf, params.clone()).ok();
        match (serial.as_ref(), par) {
          (Some(s), Some(p)) => {
            assert_eq!(
              p.components().len(),
              s.comps.len(),
              "{}: component count differs ({label})",
              path.display()
            );
            for (a, b) in p.components().iter().zip(s.comps.iter()) {
              assert_eq!(
                a.data,
                b.data,
                "{}: parallel/serial mismatch ({label})",
                path.display()
              );
            }
          }
          (Some(_), None) => panic!("{}: serial decoded, parallel failed ({label})", path.display()),
          (None, Some(_)) => panic!("{}: parallel decoded, serial failed ({label})", path.display()),
          (None, None) => {}
        }
      };
      compare(&DecodeParameters::new().strict(true), "strict");
      compare(
        &DecodeParameters::new().layers(2).reduce(1),
        "layers+reduce",
      );
      compare(
        &DecodeParameters::new()
          .decode_area(Some(DecodeArea::new(64, 64, 256, 256))),
        "area-partial",
      );
      /* A whole-image "area" (clamped to the image bounds) covers every
      tile completely, so this exercises the parallel area driver. */
      compare(
        &DecodeParameters::new()
          .decode_area(Some(DecodeArea::new(0, 0, 100_000_000, 100_000_000))),
        "area-whole",
      );
      match (serial, par) {
        (Some(s), Some(p)) => {
          assert_eq!(
            p.components().len(),
            s.comps.len(),
            "{}: component count differs",
            path.display()
          );
          for (a, b) in p.components().iter().zip(s.comps.iter()) {
            assert_eq!(a.data, b.data, "{}: parallel/serial mismatch", path.display());
          }
          checked += 1;
        }
        (Some(_), None) => panic!("{}: serial decoded, parallel failed", path.display()),
        (None, Some(_)) => panic!("{}: parallel decoded, serial failed", path.display()),
        (None, None) => {} /* expected-corrupt input */
      }
    }
    assert!(checked > 10, "expected a corpus of decodable inputs");
  }

  /// The concurrent-tile decode (rayon): a multi-tile codestream must
  /// decode byte-identically to the source pixels.
  #[cfg(feature = "rayon")]
  #[test]
  fn parallel_tile_decode_matches_source() {
    let mut rng: u64 = 0x1234_5678_9abc_def0;
    let mut next = move || {
      rng ^= rng << 13;
      rng ^= rng >> 7;
      rng ^= rng << 17;
      rng
    };
    let (w, h) = (192u32, 128u32);
    let mut core = crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 3,
      color_space: crate::OPJ_CLRSPC_SRGB,
      comps: Vec::new(),
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    for _c in 0..3 {
      let mut data = Vec::with_capacity((w * h) as usize);
      for _ in 0..w * h {
        data.push((next() % 256) as i32);
      }
      core.comps.push(crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data,
      });
    }
    let image = Image { core };
    let codestream = image
      .encode(
        EncodeParameters::new()
          .num_resolutions(2)
          .num_layers(1)
          .tile_size(64, 64),
      )
      .expect("encode");
    let back = Image::from_bytes(&codestream).expect("decode");
    assert_eq!(back.width(), w);
    assert_eq!(back.height(), h);
    for (a, b) in image.components().iter().zip(back.components()) {
      assert_eq!(a.data, b.data, "parallel tile decode mismatch");
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

  /// Regression (review round 2, H1): a QCD marker with Sqcd == 1 (SIQNT)
  /// and no SPqcd bytes must be rejected, not panic (the 2-byte band read
  /// used to slice past the end of the marker content).
  #[test]
  fn siqnt_truncated_qcd_rejected() {
    let mut cs = Vec::new();
    cs.extend([0xff, 0x4f]); // SOC
    // SIZ 8x8 1 comp
    cs.extend([0xff, 0x51, 0x00, 0x29]);
    cs.extend(0u16.to_be_bytes()); // Rsiz
    for _ in 0..8 {
      cs.extend(8u32.to_be_bytes());
    }
    cs.extend(0u32.to_be_bytes());
    cs.extend(1u16.to_be_bytes()); // Csiz
    cs.extend([0x07, 0x01, 0x01]);
    // COD
    cs.extend([0xff, 0x52, 0x00, 0x0c]);
    cs.extend([0x00, 0x00]);
    cs.extend(1u16.to_be_bytes());
    cs.extend([0x00]);
    cs.extend([0x00, 0x04, 0x04, 0x00, 0x01]);
    // QCD: Lqcd=3, Sqcd=0x01 (SIQNT), no SPqcd -> truncated
    cs.extend([0xff, 0x5c, 0x00, 0x03, 0x01]);

    assert!(Image::from_bytes_with(&cs, DecodeParameters::new().strict(true)).is_err());
    // The raw (unguarded) Codec path must fail cleanly too, not panic.
    #[allow(deprecated)]
    {
      let mut codec = Codec::new_decoder(OPJ_CODEC_J2K).unwrap();
      let mut stream = Stream::new_input(
        crate::consts::opj::OPJ_J2K_STREAM_CHUNK_SIZE as usize,
        std::io::Cursor::new(cs.clone()),
        cs.len() as u64,
      );
      let mut dp = opj_dparameters::default();
      codec.setup_decoder(&mut dp);
      codec.decoder_set_strict_mode(1);
      assert!(codec.read_header(&mut stream).is_none());
    }
  }

  /// Regression (review round 2, H2): components that subsample to a zero
  /// width/height must not panic the tile API (the C SIGFPEs on one shape
  /// and no-ops on the other; both now fail/succeed cleanly).
  #[test]
  fn zerodim_tile_api_does_not_panic() {
    for path in ["samples/zerowidth_patched.j2k", "samples/zerodim_patched.j2k"] {
      let data = std::fs::read(path).unwrap();
      #[allow(deprecated)]
      {
        let mut codec = Codec::new_decoder(OPJ_CODEC_J2K).unwrap();
        let mut stream = Stream::new_input(
          crate::consts::opj::OPJ_J2K_STREAM_CHUNK_SIZE as usize,
          std::io::Cursor::new(data.clone()),
          data.len() as u64,
        );
        let mut dp = opj_dparameters::default();
        codec.setup_decoder(&mut dp);
        let image = codec.read_header(&mut stream).expect("header");
        let mut tile_info = crate::TileInfo::default();
        tile_info.data_size = Some(0);
        assert!(
          codec.read_tile_header(&mut stream, &mut tile_info),
          "{path}: read_tile_header"
        );
        let mut buf = vec![0u8; tile_info.data_size.unwrap_or(0) as usize];
        let ok = codec.decode_tile_data(&mut stream, 0, Some(&mut buf));
        assert!(ok == 1, "{path}: decode_tile_data -> {ok}");
        let _ = image;
      }
    }
  }

  /// Regression (review round 2, M2 / round 3, H1): in lenient mode a
  /// codestream truncated right after the tile data (missing EOC) must
  /// still deliver the decoded pixels - the C returns the full image, the
  /// port used to return an image with empty component data.
  #[test]
  fn lenient_truncated_stream_keeps_pixels() {
    let buf = std::fs::read("samples/Hadley_Crater.jp2").unwrap();
    let image = Image::from_bytes(&buf).unwrap();
    let full = image.encode(EncodeParameters::new().num_resolutions(3)).unwrap();
    let trunc = &full[..full.len() - 2]; // drop the EOC marker
    let partial = Image::from_bytes_with(trunc, DecodeParameters::new().strict(false))
      .expect("lenient truncated decode");
    assert_eq!(partial.num_components(), image.num_components());
    for (a, b) in image.components().iter().zip(partial.components()) {
      assert_eq!(a.data.len(), b.data.len(), "component data must be preserved");
      assert_eq!(a.data, b.data, "pixels must match the full decode");
    }
    // Strict mode still rejects the same stream.
    assert!(Image::from_bytes_with(trunc, DecodeParameters::new().strict(true)).is_err());
  }

  /// Regression (review round 2, H3): an empty image with tiles requested
  /// (or a tile origin outside the image) must be rejected, not divide by
  /// zero.
  #[test]
  fn empty_image_with_tiles_rejected() {
    let empty = Image {
      core: crate::CoreImage {
        x0: 0,
        y0: 0,
        x1: 0,
        y1: 0,
        numcomps: 1,
        color_space: crate::OPJ_CLRSPC_GRAY,
        comps: vec![crate::CoreImageComp {
          dx: 1,
          dy: 1,
          w: 0,
          h: 0,
          x0: 0,
          y0: 0,
          prec: 8,
          bpp: 8,
          sgnd: 0,
          resno_decoded: 0,
          factor: 0,
          alpha: 0,
          data: vec![],
        }],
        icc_profile_buf: Vec::new(),
        icc_profile_is_cielab: false,
      },
    };
    assert!(empty
      .encode(EncodeParameters::new().tile_size(64, 64))
      .is_err());
  }

  /// Regression (review round 3, M4): is_imf_compliant must not index
  /// prcw_init past the end for a caller-supplied res_spec > 33 (the C
  /// reads OOB and returns FALSE; the port clamps the walk and encodes).
  #[test]
  fn imf_oversized_res_spec_encodes() {
    let (w, h) = (32u32, 32u32);
    let core = crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 1,
      color_space: crate::OPJ_CLRSPC_GRAY,
      comps: vec![crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data: vec![1; (w * h) as usize],
      }],
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    let image = Image { core };
    let mut params = EncodeParameters::new();
    params.low_level().rsiz = 0x0400; // IMF 2K
    params.low_level().res_spec = 40;
    params.low_level().csty = 2;
    assert!(image.encode(params).is_ok(), "encode must not panic");
  }

  /// Regression (review round 2, M6): custom-MCT matrix and dc-shift
  /// vectors with the wrong lengths must be rejected (the C reads them
  /// unconditionally).
  #[test]
  fn custom_mct_lengths_validated() {
    let (w, h) = (16u32, 16u32);
    let core = crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 3,
      color_space: crate::OPJ_CLRSPC_SRGB,
      comps: (0..3)
        .map(|_| crate::CoreImageComp {
          dx: 1,
          dy: 1,
          w,
          h,
          x0: 0,
          y0: 0,
          prec: 8,
          bpp: 8,
          sgnd: 0,
          resno_decoded: 0,
          factor: 0,
          alpha: 0,
          data: vec![7; (w * h) as usize],
        })
        .collect(),
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    let image = Image { core };
    let mut params = EncodeParameters::new().num_resolutions(2);
    params.low_level().mct_data = Some((vec![1.0f32; 4], vec![0i32; 3])); // matrix too short
    assert!(image.encode(params.clone()).is_err());
    let mut params = EncodeParameters::new().num_resolutions(2);
    params.low_level().mct_data = Some((vec![1.0f32; 9], vec![0i32; 2])); // dc_shift too short
    assert!(image.encode(params).is_err());
  }

  /// Regression (review round 3, L3): an out-of-range progression order
  /// must be rejected instead of written as a garbage marker.
  #[test]
  fn prog_order_validated() {
    let (w, h) = (16u32, 16u32);
    let core = crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 1,
      color_space: crate::OPJ_CLRSPC_GRAY,
      comps: vec![crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data: vec![1; (w * h) as usize],
      }],
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    let image = Image { core };
    let mut params = EncodeParameters::new();
    params.low_level().prog_order = 99;
    assert!(image.encode(params).is_err());
  }

  /// Regression (review rounds 2+3, M1): with TLM enabled and multi-part
  /// tiles, the parallel encoder must fall back to the sequential path
  /// (one TLM entry per tile would corrupt the marker); both paths then
  /// produce byte-identical output.
  #[cfg(feature = "rayon")]
  #[test]
  fn parallel_tlm_multi_part_falls_back() {
    let (w, h) = (192u32, 128u32);
    let mut comps = Vec::new();
    for c in 0..3u32 {
      let mut data = Vec::with_capacity((w * h) as usize);
      for i in 0..w * h {
        data.push(((i * 7 + c * 31) % 256) as i32);
      }
      comps.push(crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data,
      });
    }
    let core = crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 3,
      color_space: crate::OPJ_CLRSPC_SRGB,
      comps,
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    #[allow(deprecated)]
    let encode_low_level = |tlm: bool| -> Vec<u8> {
      let mut codec = Codec::new_encoder(OPJ_CODEC_J2K).unwrap();
      if tlm {
        assert!(codec.encoder_set_extra_options(&["TLM=YES"]));
      }
      let mut params = crate::EncoderParameters {
        tile_size_on: 1,
        cp_tdx: 64,
        cp_tdy: 64,
        tp_on: 1,
        tp_flag: b'R' as i8, // one tile-part per rate layer
        cp_disto_alloc: 1,   // enable rate allocation so the layers split
        tcp_numlayers: 2,
        tcp_rates: {
          let mut r = [0.0; 100];
          r[0] = 20.0;
          r[1] = 10.0;
          r
        },
        numresolution: 3,
        irreversible: 0,
        ..Default::default()
      };
      let mut img = core.clone();
      assert_eq!(codec.setup_encoder(&mut params, &mut img), 1);
      let buf = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
      {
        let writer = crate::stream::SharedVecWriter { buf: buf.clone(), pos: 0 };
        let mut stream = Stream::new_output(
          crate::consts::opj::OPJ_J2K_STREAM_CHUNK_SIZE as usize,
          writer,
        );
        assert_eq!(codec.start_compress(&mut img, &mut stream), 1);
        // The parallel driver must decline when TLM + multi-part tiles are
        // requested (one TLM entry per tile would corrupt the marker); for
        // the non-TLM control it applies normally.
        let parallel = codec.encode_parallel(&mut stream);
        assert_eq!(parallel, Some(!tlm), "parallel path fallback for TLM + multi-part");
        if parallel != Some(true) {
          assert_eq!(codec.encode(&mut stream), 1);
        }
        assert_eq!(codec.end_compress(&mut stream), 1);
      }
      std::rc::Rc::try_unwrap(buf).ok().map(|b| b.into_inner()).expect("unique handle")
    };
    let with_tlm = encode_low_level(true);
    let without_tlm = encode_low_level(false);
    assert!(!with_tlm.is_empty());
    assert!(!without_tlm.is_empty());
    // Both encode; the TLM variant decodes (rate allocation makes it lossy,
    // so only geometry/components are asserted).
    let back = Image::from_bytes(&with_tlm).expect("decode TLM stream");
    assert_eq!(back.num_components(), 3);
    assert_eq!((back.width(), back.height()), (w, h));
  }

  /// Regression (review round 2, M4 / round 3, H2b): subsampled component
  /// planes must be rejected by get_pixels/DynamicImage instead of
  /// silently truncating and panicking in the conversion.
  #[cfg(feature = "image")]
  #[test]
  fn get_pixels_subsampled_rejected() {
    let (w, h) = (64u32, 48u32);
    let mut comps = Vec::new();
    for c in 0..3u32 {
      let dx = if c == 0 { 1 } else { 2 };
      let (cw, ch) = (w.div_ceil(dx), h.div_ceil(dx));
      comps.push(crate::CoreImageComp {
        dx,
        dy: dx,
        w: cw,
        h: ch,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data: vec![c as i32 * 40; (cw * ch) as usize],
      });
    }
    let image = Image {
      core: crate::CoreImage {
        x0: 0,
        y0: 0,
        x1: w,
        y1: h,
        numcomps: 3,
        color_space: crate::OPJ_CLRSPC_UNSPECIFIED,
        comps,
        icc_profile_buf: Vec::new(),
        icc_profile_is_cielab: false,
      },
    };
    assert!(image.get_pixels(None).is_err(), "subsampled planes must be rejected");
    use core::convert::TryFrom;
    assert!(
      ::image::DynamicImage::try_from(&image).is_err(),
      "DynamicImage conversion must not panic"
    );
  }

/// Regression (review round 3, M5, re-opened by the reviewer): a tile whose
/// components subsample to nothing (subsampling larger than the tile size)
/// must encode AND decode - the C's calloc(0)/no-op loops succeed, the port
/// used to fail (decode: empty pi treated as hard error; encode: empty pi +
/// zero-dim DWT panics).
#[test]
fn subsampled_empty_tiles_round_trip() {
  let (w, h) = (256u32, 256u32);
  let core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 1,
    color_space: crate::OPJ_CLRSPC_GRAY,
    comps: vec![crate::CoreImageComp {
      dx: 128,
      dy: 128,
      w: w.div_ceil(128),
      h: h.div_ceil(128),
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data: (0..(w / 128) * (h / 128)).map(|i| (i % 256) as i32).collect(),
    }],
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let image = Image { core };
  let params = EncodeParameters::new().num_resolutions(3).tile_size(64, 64);
  let codestream = image.encode(params).expect("subsampled tiled encode");
  let back = Image::from_bytes(&codestream).expect("subsampled tiled decode");
  assert_eq!((back.width(), back.height()), (w, h));
  assert_eq!(back.num_components(), 1);
  // The visible pixels (the 2x2 subsampled plane) must round-trip losslessly
  // (reversible 5-3).
  for (a, b) in image.components()[0].data.iter().zip(back.components()[0].data.iter()) {
    assert_eq!(a, b, "subsampled round trip must be lossless");
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

  /// Minimal J2K builder: a valid 8x8 single-component codestream with the
  /// given COD parameters, used to craft regression inputs.
  fn build_minimal_j2k(numresolutions: u8, prog_order: u8, layers: u16) -> Vec<u8> {
    let mut cs = Vec::new();
    // SOC
    cs.extend([0xff, 0x4f]);
    // SIZ: 1 component, 8x8, 8-bit unsigned
    cs.extend([0xff, 0x51, 0x00, 0x29]);
    cs.extend(0u32.to_be_bytes()); // Rsiz
    cs.extend(8u32.to_be_bytes()); // Xsiz
    cs.extend(8u32.to_be_bytes()); // Ysiz
    cs.extend(0u32.to_be_bytes()); // XOsiz
    cs.extend(0u32.to_be_bytes()); // YOsiz
    cs.extend(8u32.to_be_bytes()); // XTsiz
    cs.extend(8u32.to_be_bytes()); // YTsiz
    cs.extend(0u32.to_be_bytes()); // XTOsiz
    cs.extend(0u32.to_be_bytes()); // YTOsiz
    cs.extend(1u16.to_be_bytes()); // Csiz
    cs.extend([0x07, 0x01, 0x01]); // Ssiz (8-bit) XRsiz YRsiz
    // COD
    cs.extend([0xff, 0x52, 0x00, 0x0c]);
    cs.extend([0x00, prog_order]); // Scod, progression order
    cs.extend(layers.to_be_bytes()); // num layers
    cs.extend([0x00]); // MCT
    cs.extend([numresolutions - 1, 0x04, 0x04, 0x00, 0x01]); // SPcod: levels, cblk 64, qmfbid 5-3
    // QCD: no quantization, 2 bytes per subband
    let numbands = 3 * numresolutions as usize - 2;
    let lqcd = 2 + 1 + 2 * numbands;
    cs.extend([0xff, 0x5c]);
    cs.extend((lqcd as u16).to_be_bytes());
    cs.extend([0x00]); // Sqcd
    cs.extend(std::iter::repeat(0u8).take(2 * numbands));
    // SOT + SOD + a little tile data
    cs.extend([0xff, 0x90, 0x00, 0x0a]);
    cs.extend(0u16.to_be_bytes()); // Isot
    cs.extend(0u32.to_be_bytes()); // Psot (unknown)
    cs.extend([0x00, 0x01]); // TPsot, TNsot
    cs.extend([0xff, 0x93]); // SOD
    cs.extend([0u8; 16]);
    cs
  }

  /// Regression (review finding 2): a codestream with Csiz = 0 must be
  /// rejected cleanly - the old code panicked in read_cod on the empty
  /// tccps array.
  #[test]
  fn csiz_zero_is_rejected() {
    let mut cs = Vec::new();
    cs.extend([0xff, 0x4f]); // SOC
    // SIZ with Csiz = 0 (no component records)
    cs.extend([0xff, 0x51, 0x00, 0x26]);
    cs.extend(0u32.to_be_bytes()); // Rsiz
    cs.extend(8u32.to_be_bytes()); // Xsiz
    cs.extend(8u32.to_be_bytes()); // Ysiz
    cs.extend(0u32.to_be_bytes()); // XOsiz
    cs.extend(0u32.to_be_bytes()); // YOsiz
    cs.extend(8u32.to_be_bytes()); // XTsiz
    cs.extend(8u32.to_be_bytes()); // YTsiz
    cs.extend(0u32.to_be_bytes()); // XTOsiz
    cs.extend(0u32.to_be_bytes()); // YTOsiz
    cs.extend(0u16.to_be_bytes()); // Csiz = 0
    // COD (valid, 1 resolution)
    cs.extend([0xff, 0x52, 0x00, 0x0c]);
    cs.extend([0x00, 0x00]); // Scod, LRCP
    cs.extend(1u16.to_be_bytes());
    cs.extend([0x00]); // MCT
    cs.extend([0x00, 0x04, 0x04, 0x00, 0x01]);
    // QCD
    cs.extend([0xff, 0x5c, 0x00, 0x05, 0x00, 0x00, 0x00]);
    // SOT + SOD + garbage
    cs.extend([0xff, 0x90, 0x00, 0x0a]);
    cs.extend(0u16.to_be_bytes());
    cs.extend(0u32.to_be_bytes());
    cs.extend([0x00, 0x01]);
    cs.extend([0xff, 0x93]);
    cs.extend([0u8; 16]);
    assert!(
      Image::from_bytes_with(&cs, DecodeParameters::new().strict(true)).is_err(),
      "Csiz=0 must be rejected"
    );
  }

  /// Regression (review finding 5): 6 PLT continuation bytes overflow the
  /// packet-length shift in debug builds; the decode must fail cleanly.
  #[test]
  fn plt_overflow_is_rejected() {
    let mut cs = build_minimal_j2k(1, 0, 1);
    // Replace the trailing tile data with a PLT marker + 6 continuation bytes.
    let insert_at = cs.len() - 16 - 2; // before SOD
    let mut header = cs.split_off(insert_at);
    cs.extend([0xff, 0x58, 0x00, 0x09, 0x00]); // PLT, Zplt = 0
    cs.extend([0x81u8; 6]);
    cs.append(&mut header);
    assert!(
      Image::from_bytes_with(&cs, DecodeParameters::new().strict(true)).is_err(),
      "overflowed PLT must fail the decode"
    );
  }

  /// Regression (review finding 3): numresolutions = 33 with PCRL order
  /// drives shift counts of 32 through the packet iterator; the decode must
  /// not panic (the truncated tile data makes it fail in strict mode).
  #[test]
  fn pi_high_resolution_pcrl_does_not_panic() {
    let cs = build_minimal_j2k(33, 3, 1);
    let result = Image::from_bytes_with(&cs, DecodeParameters::new().strict(true));
    let lenient = Image::from_bytes_with(&cs, DecodeParameters::new().strict(false));
    // The exact outcome depends on how far the decode gets; only panics are
    // bugs here.
    let _ = (result, lenient);
  }

  /// Regression (review finding 7): out-of-range encoder parameters must be
  /// rejected with an error, never panic (the C overflows its fixed arrays
  /// for these).
  #[test]
  fn encoder_parameters_are_validated() {
    let (w, h) = (16u32, 16u32);
    let mut core = crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 1,
      color_space: crate::OPJ_CLRSPC_GRAY,
      comps: vec![crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data: vec![64i32; (w * h) as usize],
      }],
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    let image = Image { core };

    assert!(image.encode(EncodeParameters::new().num_layers(101)).is_err());
    assert!(image.encode(EncodeParameters::new().num_layers(150)).is_err());

    let mut params = EncodeParameters::new();
    params.low_level().numpocs = 33;
    assert!(image.encode(params).is_err());

    let mut params = EncodeParameters::new();
    params.low_level().csty = 1;
    params.low_level().res_spec = 0;
    assert!(image.encode(params).is_err());

    let mut params = EncodeParameters::new();
    params.low_level().csty = 1;
    params.low_level().res_spec = 34;
    assert!(image.encode(params).is_err());
  }

  /// Regression (review finding 1): a custom MCT matrix must reach the
  /// matrix inversion without panicking.  The old port panicked there
  /// ("attempt to subtract with overflow").  Upstream parity note: the C
  /// reference writes the MCT data group only for the OPJ_MCT profile and
  /// its decoder rejects COD mct=2 ("Invalid multiple component
  /// transformation"), so - like the C - the produced codestream is not
  /// decodable; the encode side is what this test pins down.
  #[test]
  fn custom_mct_encode_reaches_inversion() {
    let (w, h) = (32u32, 24u32);
    let mut rng: u64 = 0x6a09e667f3bcc909;
    let mut next = move || {
      rng ^= rng << 13;
      rng ^= rng >> 7;
      rng ^= rng << 17;
      rng
    };
    let mut comps = Vec::new();
    for _ in 0..3 {
      let mut data = Vec::with_capacity((w * h) as usize);
      for _ in 0..w * h {
        data.push((next() % 256) as i32);
      }
      comps.push(crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data,
      });
    }
    let image = Image {
      core: crate::CoreImage {
        x0: 0,
        y0: 0,
        x1: w,
        y1: h,
        numcomps: 3,
        color_space: crate::OPJ_CLRSPC_SRGB,
        comps,
        icc_profile_buf: Vec::new(),
        icc_profile_is_cielab: false,
      },
    };
    let mut params = EncodeParameters::new().num_resolutions(3).num_layers(1);
    // Non-singular decorrelation matrix; identity would also do.
    let matrix = vec![1.0f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    params.low_level().mct_data = Some((matrix, vec![0i32; 3]));
    let codestream = image.encode(params).expect("custom MCT encode (matrix inversion)");
    assert!(!codestream.is_empty());
    // Upstream parity: the decoder rejects COD mct=2 like the C reference.
    assert!(
      Image::from_bytes_with(&codestream, DecodeParameters::new().strict(true)).is_err(),
      "custom MCT codestreams are rejected on decode (C parity)"
    );
  }

  /// Regression (review finding 6): a 16-bit image with 6 decomposition
  /// levels pushes the t1 encoder's significance shift past 31; the old code
  /// panicked in debug builds.
  #[test]
  fn encode_16bit_deep_dwt() {
    let (w, h) = (64u32, 48u32);
    let mut data = Vec::with_capacity((w * h) as usize);
    for i in 0..w * h {
      data.push((i * 1009 % 65536) as i32);
    }
    let image = Image {
      core: crate::CoreImage {
        x0: 0,
        y0: 0,
        x1: w,
        y1: h,
        numcomps: 1,
        color_space: crate::OPJ_CLRSPC_GRAY,
        comps: vec![crate::CoreImageComp {
          dx: 1,
          dy: 1,
          w,
          h,
          x0: 0,
          y0: 0,
          prec: 16,
          bpp: 16,
          sgnd: 0,
          resno_decoded: 0,
          factor: 0,
          alpha: 0,
          data,
        }],
        icc_profile_buf: Vec::new(),
        icc_profile_is_cielab: false,
      },
    };
    let codestream = image
      .encode(EncodeParameters::new().num_resolutions(6).num_layers(1))
      .expect("16-bit deep-DWT encode");
    let back = Image::from_bytes(&codestream).expect("decode");
    assert_eq!(back.num_components(), 1);
    for (x, y) in image.components()[0].data.iter().zip(back.components()[0].data.iter()) {
      assert_eq!(x, y, "reversible 5-3 must be lossless");
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
    /* Mirror decode_stream_inner: the strict flag and the resource limits
    must reach the DumpImage decoder too (they used to be silently
    dropped). */
    if codec.decoder_set_strict_mode(params.strict as crate::OPJ_BOOL) != 1 {
      return Err(Error::Decode("failed to set strict mode".into()));
    }
    if codec.set_decode_limits(params.limits) != 1 {
      return Err(Error::Decode("set_decode_limits failed".into()));
    }
    #[allow(deprecated)]
    let image = catch_codec_panic(true, || {
      codec
        .read_header(&mut stream)
        .ok_or_else(|| Error::Decode("failed to read header".into()))
    })?;
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
    let codec = &mut self.codec;
    let stream = &mut self.stream;
    let result = catch_codec_panic(true, || {
      if codec.decode(stream, &mut image) != 1 {
        return Err(Error::Decode("decode failed".into()));
      }
      Ok(())
    });
    /* Restore the image even on failure: a retry (or another decode() call)
    must not hit the expect() above, and the C reference has no such trap.
    The failed decode leaves the codec/stream mid-state; the retry simply
    fails again without panicking. */
    self.image = Some(image);
    result
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
