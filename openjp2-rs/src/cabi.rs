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

/* ----------------------------------------------------------------------- */
/* Encode-side codestream info v1 writes (called from tcd/t2)               */

/// Writes the tile stats (numpix/distotile + a fresh thresh array) into the
/// v1 mirror's tile slot.
pub(crate) fn tcd_index_set_tile_stats(
  cstr_info: *mut opj_codestream_info_t,
  tileno: u32,
  numpix: i32,
  distotile: f64,
  numlayers: u32,
) {
  if cstr_info.is_null() {
    return;
  }
  // SAFETY: `cstr_info` is the caller's v1 mirror; its tile array has tw*th entries.
  unsafe {
    let tile_info =
      &mut *(*cstr_info).tile.offset(tileno as isize) as *mut opj_tile_info_t;
    (*tile_info).numpix = numpix;
    (*tile_info).distotile = distotile;
    let mut v: Vec<core::ffi::c_double> = vec![0.0; numlayers as usize];
    (*tile_info).thresh = v.as_mut_ptr();
    core::mem::forget(v);
  }
}

/// Writes the good threshold of one layer into the v1 mirror's thresh array.
pub(crate) fn tcd_index_set_thresh(
  cstr_info: *mut opj_codestream_info_t,
  tileno: u32,
  layno: u32,
  goodthresh: f64,
) {
  if cstr_info.is_null() {
    return;
  }
  // SAFETY: the thresh array was allocated by tcd_index_set_tile_stats with
  // `numlayers` entries; `layno` stays within it.
  unsafe {
    *(*(*cstr_info).tile.offset(tileno as isize))
      .thresh
      .offset(layno as isize) = goodthresh
  };
}

/// Sets the v1 mirror's index_write flag.
pub(crate) fn tcd_index_set_write(cstr_info: *mut opj_codestream_info_t, write: i32) {
  if cstr_info.is_null() {
    return;
  }
  // SAFETY: pointer validity is the caller's contract (C-ABI mirror).
  unsafe {
    (*cstr_info).index_write = write;
  }
}

/// Initializes the v1 mirror's tile slot from the component-0 resolution
/// layout (precinct sizes/exponents and the packet array).
pub(crate) fn tcd_index_encode_tile_init(
  cstr_info: *mut opj_codestream_info_t,
  tileno: u32,
  l_num_packs: u32,
  resolutions: &[crate::types::opj_tcd_resolution_t],
  tccps: &[crate::types::opj_tccp_t],
) {
  if cstr_info.is_null() {
    return;
  }
  // SAFETY: `cstr_info` is the caller's v1 mirror; its tile array has tw*th
  // entries and the packet array is built below with the exact element count.
  unsafe {
    for (i, l_res_idx) in resolutions.iter().enumerate() {
      (*(*cstr_info).tile.offset(tileno as isize)).pw[i] = l_res_idx.pw as core::ffi::c_int;
      (*(*cstr_info).tile.offset(tileno as isize)).ph[i] = l_res_idx.ph as core::ffi::c_int;
      (*(*cstr_info).tile.offset(tileno as isize)).pdx[i] =
        tccps[i].prcw[i] as core::ffi::c_int;
      (*(*cstr_info).tile.offset(tileno as isize)).pdy[i] =
        tccps[i].prch[i] as core::ffi::c_int;
    }
    let p_packet = &mut (*(*cstr_info).tile.offset(tileno as isize)).packet;
    let n = ((*cstr_info).numcomps as usize)
      .wrapping_mul((*cstr_info).numlayers as usize)
      .wrapping_mul(l_num_packs as usize);
    let mut v: Vec<opj_packet_info_t> = vec![Default::default(); n];
    *p_packet = v.as_mut_ptr();
    core::mem::forget(v);
  }
}

/// Per-packet index bookkeeping of t2 encode (start/end positions).
pub(crate) fn t2_write_packet_index(
  cstr_info: *mut opj_codestream_info_t,
  tileno: u32,
  l_nb_bytes: i64,
  tp_on: bool,
  poc: bool,
) {
  if cstr_info.is_null() {
    return;
  }
  // SAFETY: `cstr_info` is the caller's v1 mirror; the tile/packet arrays were
  // sized by tcd_index_encode_tile_init.
  unsafe {
    if (*cstr_info).index_write != 0 {
      let info_TL: *mut opj_tile_info_t =
        &mut *(*cstr_info).tile.offset(tileno as isize) as *mut opj_tile_info_t;
      let info_PK: *mut opj_packet_info_t =
        &mut *(*info_TL).packet.offset((*cstr_info).packno as isize)
          as *mut opj_packet_info_t;
      if (*cstr_info).packno == 0 {
        (*info_PK).start_pos = ((*info_TL).end_header + 1i32) as OPJ_OFF_T
      } else {
        (*info_PK).start_pos =
          if (tp_on || poc) && (*info_PK).start_pos != 0 {
            (*info_PK).start_pos
          } else {
            ((*(*info_TL)
              .packet
              .offset(((*cstr_info).packno - 1i32) as isize))
            .end_pos)
              + 1i64
          }
      }
      (*info_PK).end_pos = (*info_PK).start_pos + l_nb_bytes - 1;
      (*info_PK).end_ph_pos += (*info_PK).start_pos - 1i64
    }
    (*cstr_info).packno += 1
  }
}

/// Sets the packet's end-of-header position (encode).
pub(crate) fn t2_packet_index_end_ph(
  cstr_info: *mut opj_codestream_info_t,
  tileno: u32,
  pos: i64,
) {
  if cstr_info.is_null() {
    return;
  }
  // SAFETY: see t2_write_packet_index.
  unsafe {
    if (*cstr_info).index_write != 0 {
      let info_PK: *mut opj_packet_info_t = &mut *(*(*cstr_info).tile.offset(tileno as isize))
        .packet
        .offset((*cstr_info).packno as isize)
        as *mut opj_packet_info_t;
      (*info_PK).end_ph_pos = pos
    }
  }
}

/// Accumulates the packet distortion and tracks D_max (encode).
pub(crate) fn t2_packet_index_disto(
  cstr_info: *mut opj_codestream_info_t,
  tileno: u32,
  disto: f64,
) {
  if cstr_info.is_null() {
    return;
  }
  // SAFETY: see t2_write_packet_index.
  unsafe {
    if (*cstr_info).index_write != 0 {
      let info_PK: *mut opj_packet_info_t =
        &mut *(*(*cstr_info).tile.offset(tileno as isize))
          .packet
          .offset((*cstr_info).packno as isize) as *mut opj_packet_info_t;
      (*info_PK).disto += disto;
      if (*cstr_info).D_max < (*info_PK).disto {
        (*cstr_info).D_max = (*info_PK).disto
      }
    }
  }
}

/// Sets the caller-provided packet-info's end-of-header position.
pub(crate) fn t2_pack_info_end_ph(p_pack_info: *mut opj_packet_info_t, l_pos: i64) {
  if p_pack_info.is_null() {
    return;
  }
  // SAFETY: `p_pack_info` is the caller's packet-info slot.
  unsafe {
    (*p_pack_info).end_ph_pos = l_pos;
  }
}

/* ----------------------------------------------------------------------- */
/* Encoder parameter marshalling (setup_encoder reads)                     */

/// Copies the caller-provided fixed-quality matrix (`cp_matrice`) into an
/// owned buffer. Returns None when no matrix is set.
pub(crate) fn cparams_matrice(
  parameters: &opj_cparameters_t,
  array_len: usize,
) -> Option<Vec<core::ffi::c_int>> {
  if parameters.cp_matrice.is_null() {
    return None;
  }
  // SAFETY: `cp_matrice` is the caller's matrix of `array_len` ints.
  Some(unsafe { core::slice::from_raw_parts(parameters.cp_matrice, array_len) }.to_vec())
}

/// Copies the caller-provided comment string into an owned byte buffer.
pub(crate) fn cparams_comment(parameters: &opj_cparameters_t) -> Option<Vec<u8>> {
  if parameters.cp_comment.is_null() {
    return None;
  }
  // SAFETY: `cp_comment` is a caller-provided NUL-terminated string.
  Some(
    unsafe { core::ffi::CStr::from_ptr(parameters.cp_comment) }
      .to_bytes()
      .to_vec(),
  )
}

/// Copies the caller-provided MCT coding matrix and returns it together with
/// the pointer to the dc-shift array that follows it in the same buffer.
pub(crate) fn cparams_mct_data(
  parameters: &opj_cparameters_t,
  numcomps: u32,
) -> Option<(Vec<OPJ_FLOAT32>, *mut OPJ_INT32)> {
  if parameters.mct_data.is_null() {
    return None;
  }
  let l_mct_len = (numcomps * numcomps) as usize;
  let l_mct_size = l_mct_len.wrapping_mul(core::mem::size_of::<OPJ_FLOAT32>());
  // SAFETY: `mct_data` holds `l_mct_len` floats followed by `numcomps` ints.
  unsafe {
    let l_dc_shift = (parameters.mct_data as *mut OPJ_BYTE).add(l_mct_size) as *mut OPJ_INT32;
    let matrix: Vec<OPJ_FLOAT32> =
      core::slice::from_raw_parts(parameters.mct_data as *const OPJ_FLOAT32, l_mct_len).to_vec();
    Some((matrix, l_dc_shift))
  }
}

/// Copies the dc-shift array that follows the MCT matrix in the caller's buffer.
pub(crate) fn cparams_dc_shift(dc_shift: *mut OPJ_INT32, numcomps: u32) -> Vec<OPJ_INT32> {
  // SAFETY: `dc_shift` points at `numcomps` ints following the MCT matrix.
  unsafe { core::slice::from_raw_parts(dc_shift, numcomps as usize) }.to_vec()
}
