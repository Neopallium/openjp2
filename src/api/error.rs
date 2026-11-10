// Split of the safe API (doc/api-review.md): see the facade in api.rs
// and doc/redesign-proposal.md B.3.2.

//! Safe, high-level API over the codec (see doc/api-review.md, item 13 of
//! the work plan): one-shot `Image` decoding, a `DecodeParameters` builder,
//! a typed `ColorSpace`, and a unified `Error`.  The low-level `Decoder` /
//! `Encoder` types (the merged `Codec` handle lives in the `openjp2-capi`
//! crate) keep the C-style step API for incremental use.
//! `Stream` types remain available for streaming and tile-level use cases.

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
pub(crate) fn catch_codec_panic<T>(is_decode: bool, f: impl FnOnce() -> Result<T>) -> Result<T> {
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
