// Split of the safe API (doc/api-review.md): see the facade in api.rs
// and doc/redesign-proposal.md B.3.2.

use super::*;

/// The colour space a codestream declares (the JP2 `colr`/COD value).
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
  /// Unknown / not declared.
  #[default]
  Unknown,
  /// sRGB.
  Srgb,
  /// Grayscale.
  Gray,
  /// YCbCr (`SYCC`).
  Sycc,
  /// Extended YCC (`EYCC`).
  Eycc,
  /// CMYK.
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
/// Packet progression order (the COD marker's progression field).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum ProgressionOrder {
  /// Layer, resolution, component, precinct.
  #[default]
  Lrcp,
  /// Resolution, layer, component, precinct.
  Rlcp,
  /// Resolution, precinct, component, layer.
  Rpcl,
  /// Precinct, component, resolution, layer.
  Pcrl,
  /// Component, precinct, resolution, layer.
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
  /// Inclusive x origin.
  pub start_x: u32,
  /// Inclusive y origin.
  pub start_y: u32,
  /// Exclusive x end.
  pub end_x: u32,
  /// Exclusive y end.
  pub end_y: u32,
}

impl DecodeArea {
  /// A new area (clamped to the image by the decoder).
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
  /// All-zero limits (everything allowed, subject to the built-in
  /// 1 GiB decode-memory ceiling).
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
  /// Resolution reduction (`2^reduce`); `0` = full resolution.
  pub reduce: u32,
  /// Only decode the first `layers` quality layers; `0` = all.
  pub layers: u32,
  /// Reject malformed codestreams instead of tolerating them.
  pub strict: bool,
  /// Restrict the decoded area (whole image when `None`).
  pub decode_area: Option<DecodeArea>,
  /// Ignore the JP2 palette/channel-definition boxes.
  pub ignore_pclr_cmap_cdef: bool,
  /// Per-decode thread limit: `0` = automatic (the process-wide rayon
  /// pool), `n > 0` = a local n-thread pool built for this decode.  The
  /// default (`0`) preserves the current process-wide behaviour; without
  /// the `rayon` feature the value is inert.
  pub threads: usize,
  /// Optional resource limits (see `DecodeLimits`).  All-zero except
  /// `max_mem_alloc` = unlimited; `max_mem_alloc == 0` selects the built-in
  /// 1 GiB ceiling (`DEFAULT_DECODE_MEM_LIMIT`).
  pub limits: DecodeLimits,
}

impl DecodeParameters {
  /// Default parameters: lenient, full image, automatic threading.
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

  /// Limit the number of worker threads used for this decode (`0` = the
  /// process-wide rayon pool, the default; `n > 0` = a local n-thread pool
  /// for this decode only).  The result is byte-identical either way.
  pub fn threads(mut self, n: usize) -> Self {
    self.threads = n;
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
  /// Default parameters: lossless, single layer, one tile.
  pub fn new() -> Self {
    Self {
      inner: crate::EncoderParameters::default(),
    }
  }

  /// Number of resolutions (DWT decomposition levels + 1).
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
  /// (e.g. `20.0` = 20:1); enables rate-distortion ratio allocation (the
  /// C's `-q` option).
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
  /// Mutable access to the low-level C-mirror parameters (advanced
  /// knobs: custom MCT, cinema profiles, ...).
  pub fn low_level(&mut self) -> &mut crate::EncoderParameters {
    &mut self.inner
  }

  /// Consumes the builder into the low-level parameter struct.
  pub fn into_inner(self) -> crate::EncoderParameters {
    self.inner
  }
}

