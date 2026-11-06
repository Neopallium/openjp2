//! Re-export hub for the C-ABI types and helpers used by the codec modules.
//!
//! Phase 6: the actual C surface (`#[no_mangle]` exports) lives in the
//! `openjp2-capi` crate; this module keeps the type re-exports the codec
//! modules rely on, plus the image-data allocation used by the owned buffers.

pub use crate::c_api_types::*;
pub use crate::consts::opj::*;
pub use crate::types::*;

/// Allocate `size` bytes for image component data. The C ABI export of this
/// function lives in `openjp2-capi`; the codec uses it for the buffers the C
/// callers may free or replace.
pub fn opj_image_data_alloc(size: OPJ_SIZE_T) -> *mut core::ffi::c_void {
  unsafe { crate::malloc::opj_malloc(size) }
}

/// Free an image component data buffer allocated by `opj_image_data_alloc`.
pub fn opj_image_data_free(ptr: *mut core::ffi::c_void) {
  unsafe { crate::malloc::opj_free(ptr) }
}

pub const OPJ_VERSION: &str = "2.5.2";

/// Free a codestream info v2 structure (core-side helper; the C ABI export
/// lives in openjp2-capi).
pub fn opj_destroy_cstr_info(cstr_info: *mut *mut opj_codestream_info_v2_t) {
  if !cstr_info.is_null() {
    let info = unsafe { Box::from_raw(*cstr_info) };
    if !info.m_default_tile_info.tccp_info.is_null() {
      // SAFETY: built as a Vec of exactly `nbcomps` entries (capacity == length).
      unsafe {
        drop(Vec::from_raw_parts(
          info.m_default_tile_info.tccp_info,
          info.nbcomps as usize,
          info.nbcomps as usize,
        ))
      };
    }
    if !info.tile_info.is_null() {
      // SAFETY: built as a Vec of exactly `tw * th` entries.
      let n = (info.tw as usize).wrapping_mul(info.th as usize);
      unsafe { drop(Vec::from_raw_parts(info.tile_info, n, n)) };
    }
    unsafe { *cstr_info = core::ptr::null_mut::<opj_codestream_info_v2_t>() };
  }
}
