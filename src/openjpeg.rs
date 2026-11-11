//! Re-export hub for the C-ABI types and helpers used by the codec modules.
//!
//! Phase 6: the actual C surface (`#[no_mangle]` exports) lives in the
//! `openjp2-capi` crate; this module keeps the type re-exports the codec
//! modules rely on, plus the image-data allocation used by the owned buffers.

pub use crate::c_api_types::*;
pub use crate::consts::opj::*;
pub use crate::types::*;

/// The upstream OpenJPEG version this port mirrors.
pub const OPJ_VERSION: &str = "2.5.4";

/// Default codestream comment (the COM marker) written by the encoder.
///
/// By default the port identifies itself with its own product name and
/// crate version.  Building with the `openjpeg-version-string` feature
/// switches this to the upstream OpenJPEG product/version string, so
/// encoded files are byte-identical to the C reference (what the test
/// suite compares against).
#[cfg(feature = "openjpeg-version-string")]
pub fn default_codestream_comment() -> String {
  format!("Created by OpenJPEG version {}", OPJ_VERSION)
}

/// Default codestream comment (the COM marker) written by the encoder.
///
/// By default the port identifies itself with its own product name and
/// crate version.  Building with the `openjpeg-version-string` feature
/// switches this to the upstream OpenJPEG product/version string, so
/// encoded files are byte-identical to the C reference (what the test
/// suite compares against).
#[cfg(not(feature = "openjpeg-version-string"))]
pub fn default_codestream_comment() -> String {
  format!(
    "Created by {} version {}",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_VERSION")
  )
}

