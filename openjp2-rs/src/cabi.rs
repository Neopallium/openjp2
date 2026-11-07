//! The C-ABI mirror layer: builder/free helpers for the v1/v2 codestream-info
//! structures, the C-allocator pair, and the marker-info buffers the C caller
//! passes through `opj_setup_encoder` / `opj_encode`.
//!
//! Compiled only with the `c_api` feature (enabled by `openjp2-capi`).  The
//! default core build contains no unsafe code.

#![allow(unsafe_code)]

use crate::c_api_types::*;
use crate::fprintf::StringWriter;
use crate::types::*;

/* ----------------------------------------------------------------------- */
/* C allocator pair                                                         */

extern "C" {
  fn malloc(_: usize) -> *mut core::ffi::c_void;
  #[cfg(not(target_os = "windows"))]
  fn free(_: *mut core::ffi::c_void);
  #[cfg(target_os = "windows")]
  fn free(_: *mut core::ffi::c_void);
}

/// Allocate `size` bytes with the C allocator.
pub fn opj_malloc(mut size: usize) -> *mut core::ffi::c_void {
  if size == 0 {
    /* prevent implementation defined behavior of malloc */
    return core::ptr::null_mut::<core::ffi::c_void>();
  }
  unsafe { malloc(size) }
}

/// Free a pointer allocated by `opj_malloc`.
pub fn opj_free(mut ptr: *mut core::ffi::c_void) {
  unsafe {
    if !ptr.is_null() {
      free(ptr);
    }
  }
}

/* ----------------------------------------------------------------------- */
/* FILE* writer for the C tool's opj_dump                                   */

impl StringWriter for ::libc::FILE {
  fn write_string(&mut self, s: &str) -> usize {
    unsafe {
      let bytes = s.as_bytes();
      let len = bytes.len();
      let nb = libc::fwrite(bytes.as_ptr() as *const ::libc::c_void, 1, len, self);
      if nb == ::libc::size_t::MAX {
        0
      } else {
        s.len()
      }
    }
  }
}

/* ----------------------------------------------------------------------- */
/* Codestream info v2                                                       */

/// Free a codestream info v2 structure (core-side helper; the C ABI export
/// lives in openjp2-capi).
///
/// # Safety
/// `cstr_info` must be NULL or point to a pointer previously filled by
/// `j2k_get_cstr_info`.
pub unsafe fn opj_destroy_cstr_info(cstr_info: *mut *mut opj_codestream_info_v2_t) {
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

/* ----------------------------------------------------------------------- */
/* Codestream info builders (v1/v2 mirrors)                                 */

pub(crate) fn j2k_get_cstr_info(p_j2k: &mut opj_j2k) -> *mut opj_codestream_info_v2_t {
  let numcomps = p_j2k.m_private_image.as_deref().expect("m_private_image").numcomps;
  // SAFETY: zeroed is the C calloc state the callers expect for the pointer fields.
  let mut cstr_info = Box::new(unsafe { core::mem::zeroed::<opj_codestream_info_v2_t>() });
  cstr_info.nbcomps = p_j2k.m_private_image.as_deref().expect("m_private_image").numcomps;
  cstr_info.tx0 = p_j2k.m_cp.tx0;
  cstr_info.ty0 = p_j2k.m_cp.ty0;
  cstr_info.tdx = p_j2k.m_cp.tdx;
  cstr_info.tdy = p_j2k.m_cp.tdy;
  cstr_info.tw = p_j2k.m_cp.tw;
  cstr_info.th = p_j2k.m_cp.th;
  cstr_info.tile_info = core::ptr::null_mut::<opj_tile_info_v2_t>();
  let l_default_tile = p_j2k
    .m_specific_param
    .m_decoder
    .m_default_tcp
    .as_deref()
    .expect("decoder m_default_tcp is always set");
  cstr_info.m_default_tile_info.csty = l_default_tile.csty;
  cstr_info.m_default_tile_info.prg = l_default_tile.prg;
  cstr_info.m_default_tile_info.numlayers = l_default_tile.numlayers;
  cstr_info.m_default_tile_info.mct = l_default_tile.mct;
  {
    let mut v: Vec<opj_tccp_info_t> = vec![unsafe { core::mem::zeroed() }; cstr_info.nbcomps as usize];
    cstr_info.m_default_tile_info.tccp_info = v.as_mut_ptr();
    core::mem::forget(v);
  }
  let tccp_infos = unsafe {
    core::slice::from_raw_parts_mut(
      cstr_info.m_default_tile_info.tccp_info,
      numcomps as usize,
    )
  };
  let mut compno: OPJ_UINT32 = 0;
  while compno < numcomps {
    let l_tccp = &l_default_tile.tccps[compno as usize];
    let l_tccp_info = &mut tccp_infos[compno as usize];
    /* coding style*/
    l_tccp_info.csty = l_tccp.csty;
    l_tccp_info.numresolutions = l_tccp.numresolutions;
    l_tccp_info.cblkw = l_tccp.cblkw;
    l_tccp_info.cblkh = l_tccp.cblkh;
    l_tccp_info.cblksty = l_tccp.cblksty;
    l_tccp_info.qmfbid = l_tccp.qmfbid;
    if l_tccp.numresolutions < 33u32 {
      l_tccp_info.prch[..l_tccp.numresolutions as usize]
        .copy_from_slice(&l_tccp.prch[..l_tccp.numresolutions as usize]);
      l_tccp_info.prcw[..l_tccp.numresolutions as usize]
        .copy_from_slice(&l_tccp.prcw[..l_tccp.numresolutions as usize]);
    }
    /* quantization style*/
    l_tccp_info.qntsty = l_tccp.qntsty;
    l_tccp_info.numgbits = l_tccp.numgbits;
    let numbands: OPJ_INT32 = if l_tccp.qntsty == 1u32 {
      1i32
    } else {
      (l_tccp.numresolutions as OPJ_INT32 * 3i32) - 2i32
    };
    if numbands < 3i32 * 33i32 - 2i32 {
      let mut bandno: OPJ_INT32 = 0;
      while bandno < numbands {
        l_tccp_info.stepsizes_mant[bandno as usize] = l_tccp.stepsizes[bandno as usize].mant as OPJ_UINT32;
        l_tccp_info.stepsizes_expn[bandno as usize] = l_tccp.stepsizes[bandno as usize].expn as OPJ_UINT32;
        bandno += 1
      }
    }
    /* RGN value*/
    l_tccp_info.roishift = l_tccp.roishift;
    compno += 1;
  }
  Box::into_raw(cstr_info)
}

pub(crate) fn j2k_get_cstr_index(p_j2k: &mut opj_j2k) -> *mut opj_codestream_index_t {
  let cstr_index = p_j2k.cstr_index.as_deref().expect("cstr_index");
  let mut l_cstr_index = Box::new(unsafe { core::mem::zeroed::<opj_codestream_index_t>() });
  l_cstr_index.main_head_start = cstr_index.main_head_start;
  l_cstr_index.main_head_end = cstr_index.main_head_end;
  l_cstr_index.codestream_size = cstr_index.codestream_size;
  l_cstr_index.marknum = cstr_index.markers.len() as OPJ_UINT32;
  if !cstr_index.markers.is_empty() {
    let mut v = cstr_index.markers.clone();
    l_cstr_index.marker = v.as_mut_ptr();
    core::mem::forget(v);
  } else {
    l_cstr_index.marker = core::ptr::null_mut::<opj_marker_info_t>()
  }
  l_cstr_index.nb_of_tiles = cstr_index.tiles.len() as OPJ_UINT32;
  if cstr_index.tiles.is_empty() {
    l_cstr_index.tile_index = core::ptr::null_mut::<opj_tile_index_t>()
  } else {
    let mut v: Vec<opj_tile_index_t> =
      vec![unsafe { core::mem::zeroed() }; cstr_index.tiles.len()];
    l_cstr_index.tile_index = v.as_mut_ptr();
    let tile_indices = unsafe {
      core::slice::from_raw_parts_mut(l_cstr_index.tile_index, cstr_index.tiles.len())
    };
    for (it_tile, tile) in cstr_index.tiles.iter().enumerate() {
      let dest = &mut tile_indices[it_tile];
      /* Tile markers */
      dest.marknum = tile.markers.len() as OPJ_UINT32;
      if !tile.markers.is_empty() {
        let mut mv = tile.markers.clone();
        dest.marker = mv.as_mut_ptr();
        core::mem::forget(mv);
      } else {
        dest.marker = core::ptr::null_mut::<opj_marker_info_t>()
      }
      /* Tile part index */
      dest.nb_tps = tile.nb_tps;
      if !tile.tp_index.is_empty() {
        let mut tv = tile.tp_index.clone();
        dest.tp_index = tv.as_mut_ptr();
        core::mem::forget(tv);
      } else {
        dest.tp_index = core::ptr::null_mut::<opj_tp_index_t>()
      }
      /* Packet index (NOT USED) */
      dest.nb_packet = 0 as OPJ_UINT32;
      dest.packet_index = core::ptr::null_mut::<opj_packet_info_t>();
    }
    core::mem::forget(v);
  }
  Box::into_raw(l_cstr_index)
}

pub(crate) fn jp2_get_cstr_index(mut p_jp2: &mut opj_jp2) -> *mut opj_codestream_index_t {
  j2k_get_cstr_index(&mut p_jp2.j2k)
}

pub(crate) fn jp2_get_cstr_info(mut p_jp2: &mut opj_jp2) -> *mut opj_codestream_info_v2_t {
  j2k_get_cstr_info(&mut p_jp2.j2k)
}

/// Free a C-ABI codestream index mirror built by `j2k_get_cstr_index`.
///
/// # Safety
/// `p_cstr_ind` must be NULL or point to a `Box` produced by
/// `j2k_get_cstr_index`.
pub unsafe fn free_cstr_index_mirror(p_cstr_ind: *mut opj_codestream_index_t) {
  if p_cstr_ind.is_null() {
    return;
  }
  let cstr_ind = unsafe { Box::from_raw(p_cstr_ind) };
  if !cstr_ind.marker.is_null() {
    // SAFETY: built as a Vec of exactly `marknum` entries.
    unsafe {
      drop(Vec::from_raw_parts(
        cstr_ind.marker,
        cstr_ind.marknum as usize,
        cstr_ind.marknum as usize,
      ))
    };
  }
  if !cstr_ind.tile_index.is_null() {
    let n = cstr_ind.nb_of_tiles as usize;
    for tile in unsafe { core::slice::from_raw_parts_mut(cstr_ind.tile_index, n) } {
      if !tile.packet_index.is_null() {
        // SAFETY: built as a Vec of exactly `nb_packet` entries.
        unsafe {
          drop(Vec::from_raw_parts(
            tile.packet_index,
            tile.nb_packet as usize,
            tile.nb_packet as usize,
          ))
        };
      }
      if !tile.tp_index.is_null() {
        // SAFETY: built as a Vec of exactly `nb_tps` entries.
        unsafe {
          drop(Vec::from_raw_parts(tile.tp_index, tile.nb_tps as usize, tile.nb_tps as usize))
        };
      }
      if !tile.marker.is_null() {
        // SAFETY: built as a Vec of exactly `marknum` entries.
        unsafe {
          drop(Vec::from_raw_parts(
            tile.marker,
            tile.marknum as usize,
            tile.marknum as usize,
          ))
        };
      }
    }
    // SAFETY: built as a Vec of exactly `nb_of_tiles` entries.
    unsafe { drop(Vec::from_raw_parts(cstr_ind.tile_index, n, n)) };
  }
}

/* ----------------------------------------------------------------------- */
/* Tile marker info (PLT packet sizes)                                      */



