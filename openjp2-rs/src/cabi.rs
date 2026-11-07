//! The C-ABI mirror layer: builder/free helpers for the v1/v2 codestream-info
//! structures, the C-allocator pair, and the marker-info buffers the C caller
//! passes through `opj_setup_encoder` / `opj_encode`.
//!
//! Compiled only with the `c_api` feature (enabled by `openjp2-capi`).  The
//! default core build contains no unsafe code.

#![allow(unsafe_code)]

use crate::c_api_types::*;
use crate::fprintf::StringWriter;

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
