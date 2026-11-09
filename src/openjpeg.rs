//! Re-export hub for the C-ABI types and helpers used by the codec modules.
//!
//! Phase 6: the actual C surface (`#[no_mangle]` exports) lives in the
//! `openjp2-capi` crate; this module keeps the type re-exports the codec
//! modules rely on, plus the image-data allocation used by the owned buffers.

pub use crate::c_api_types::*;
pub use crate::consts::opj::*;
pub use crate::types::*;

pub const OPJ_VERSION: &str = "2.5.4";

