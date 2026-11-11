//! The C-ABI codestream-info mirrors (v1 index and v2 header info) plus the
//! builders that marshal the core's safe owned structures into them.

use openjp2_ctypes::{opj_marker_info_t, opj_packet_info_t, OPJ_INT32, OPJ_PROG_ORDER, OPJ_UINT32};
use openjp2_ctypes::OPJ_OFF_T;
use openjp2_core::{CodestreamIndex, CodestreamTileIndex, CstrCompInfoV2, CstrInfoV2};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tccp_info {
  pub compno: OPJ_UINT32,
  pub csty: OPJ_UINT32,
  pub numresolutions: OPJ_UINT32,
  pub cblkw: OPJ_UINT32,
  pub cblkh: OPJ_UINT32,
  pub cblksty: OPJ_UINT32,
  pub qmfbid: OPJ_UINT32,
  pub qntsty: OPJ_UINT32,
  pub stepsizes_mant: [OPJ_UINT32; 97],
  pub stepsizes_expn: [OPJ_UINT32; 97],
  pub numgbits: OPJ_UINT32,
  pub roishift: OPJ_INT32,
  pub prcw: [OPJ_UINT32; 33],
  pub prch: [OPJ_UINT32; 33],
}
pub type opj_tccp_info_t = opj_tccp_info;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tile_info_v2 {
  pub tileno: core::ffi::c_int,
  pub csty: OPJ_UINT32,
  pub prg: OPJ_PROG_ORDER,
  pub numlayers: OPJ_UINT32,
  pub mct: OPJ_UINT32,
  pub tccp_info: *mut opj_tccp_info_t,
}
pub type opj_tile_info_v2_t = opj_tile_info_v2;

#[repr(C)]
pub struct opj_codestream_info_v2 {
  pub tx0: OPJ_UINT32,
  pub ty0: OPJ_UINT32,
  pub tdx: OPJ_UINT32,
  pub tdy: OPJ_UINT32,
  pub tw: OPJ_UINT32,
  pub th: OPJ_UINT32,
  pub nbcomps: OPJ_UINT32,
  pub m_default_tile_info: opj_tile_info_v2_t,
  pub tile_info: *mut opj_tile_info_v2_t,
}
pub type opj_codestream_info_v2_t = opj_codestream_info_v2;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tile_index {
  pub tileno: OPJ_UINT32,
  pub nb_tps: OPJ_UINT32,
  pub current_nb_tps: OPJ_UINT32,
  pub current_tpsno: OPJ_UINT32,
  pub tp_index: *mut openjp2_ctypes::opj_tp_index_t,
  pub marknum: OPJ_UINT32,
  pub marker: *mut opj_marker_info_t,
  pub maxmarknum: OPJ_UINT32,
  pub nb_packet: OPJ_UINT32,
  pub packet_index: *mut opj_packet_info_t,
}
pub type opj_tile_index_t = opj_tile_index;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_codestream_index {
  pub main_head_start: OPJ_OFF_T,
  pub main_head_end: OPJ_OFF_T,
  pub codestream_size: u64,
  pub marknum: OPJ_UINT32,
  pub marker: *mut opj_marker_info_t,
  pub maxmarknum: OPJ_UINT32,
  pub nb_of_tiles: OPJ_UINT32,
  pub tile_index: *mut opj_tile_index_t,
}
pub type opj_codestream_index_t = opj_codestream_index;

/// Builds the C-ABI v2 mirror from the core's owned header summary.
pub fn build_cstr_info_v2(info: &CstrInfoV2) -> *mut opj_codestream_info_v2_t {
  let tccps: Vec<opj_tccp_info_t> = info
    .default_tile
    .tccps
    .iter()
    .map(marshal_tccp)
    .collect();
  let boxed = Box::new(opj_codestream_info_v2 {
    tx0: info.tx0,
    ty0: info.ty0,
    tdx: info.tdx,
    tdy: info.tdy,
    tw: info.tw,
    th: info.th,
    nbcomps: info.nbcomps,
    m_default_tile_info: opj_tile_info_v2 {
      tileno: 0,
      csty: info.default_tile.csty,
      prg: info.default_tile.prg.into(),
      numlayers: info.default_tile.numlayers,
      mct: info.default_tile.mct,
      tccp_info: if tccps.is_empty() {
        core::ptr::null_mut()
      } else {
        let mut v = tccps;
        let p = v.as_mut_ptr();
        core::mem::forget(v);
        p
      },
    },
    tile_info: core::ptr::null_mut(),
  });
  Box::into_raw(boxed)
}

fn marshal_tccp(c: &CstrCompInfoV2) -> opj_tccp_info_t {
  opj_tccp_info_t {
    compno: 0,
    csty: c.csty,
    numresolutions: c.numresolutions,
    cblkw: c.cblkw,
    cblkh: c.cblkh,
    cblksty: c.cblksty,
    qmfbid: c.qmfbid,
    qntsty: c.qntsty,
    stepsizes_mant: c.stepsizes_mant,
    stepsizes_expn: c.stepsizes_expn,
    numgbits: c.numgbits,
    roishift: c.roishift,
    prcw: c.prcw,
    prch: c.prch,
  }
}

/// Frees a v2 mirror built by `build_cstr_info_v2`.
pub fn destroy_cstr_info_v2(cstr_info: *mut *mut opj_codestream_info_v2_t) {
  if cstr_info.is_null() {
    return;
  }
  // SAFETY: the pointee was produced by build_cstr_info_v2.
  unsafe {
    let info = Box::from_raw(*cstr_info);
    if !info.m_default_tile_info.tccp_info.is_null() {
      let n = info.nbcomps as usize;
      drop(Vec::from_raw_parts(info.m_default_tile_info.tccp_info, n, n));
    }
    *cstr_info = core::ptr::null_mut();
  }
}

/// Builds the C-ABI v1 index mirror from the core's owned index.
pub fn build_cstr_index(index: &CodestreamIndex) -> *mut opj_codestream_index_t {
  let boxed = Box::new(opj_codestream_index {
    main_head_start: index.main_head_start,
    main_head_end: index.main_head_end,
    codestream_size: index.codestream_size,
    marknum: index.markers.len() as OPJ_UINT32,
    marker: clone_to_ptr(&index
      .markers
      .iter()
      .map(|m| openjp2_ctypes::opj_marker_info {
        type_: m.type_,
        pos: m.pos,
        len: m.len,
      })
      .collect::<Vec<_>>()),
    maxmarknum: index.markers.len() as OPJ_UINT32,
    nb_of_tiles: index.tiles.len() as OPJ_UINT32,
    tile_index: if index.tiles.is_empty() {
      core::ptr::null_mut()
    } else {
      let tiles: Vec<opj_tile_index_t> = index.tiles.iter().map(marshal_tile).collect();
      let mut v = tiles;
      let p = v.as_mut_ptr();
      core::mem::forget(v);
      p
    },
  });
  Box::into_raw(boxed)
}

fn marshal_tile(tile: &CodestreamTileIndex) -> opj_tile_index_t {
  opj_tile_index_t {
    tileno: tile.tileno,
    nb_tps: tile.nb_tps,
    current_nb_tps: tile.current_nb_tps,
    current_tpsno: tile.current_tpsno,
    tp_index: clone_to_ptr(&tile
      .tp_index
      .iter()
      .map(|t| openjp2_ctypes::opj_tp_index {
        start_pos: t.start_pos,
        end_header: t.end_header,
        end_pos: t.end_pos,
      })
      .collect::<Vec<_>>()),
    marknum: tile.markers.len() as OPJ_UINT32,
    marker: clone_to_ptr(&tile
      .markers
      .iter()
      .map(|m| openjp2_ctypes::opj_marker_info {
        type_: m.type_,
        pos: m.pos,
        len: m.len,
      })
      .collect::<Vec<_>>()),
    maxmarknum: tile.markers.len() as OPJ_UINT32,
    nb_packet: 0,
    packet_index: core::ptr::null_mut(),
  }
}

fn clone_to_ptr<T: Copy>(v: &[T]) -> *mut T {
  if v.is_empty() {
    return core::ptr::null_mut();
  }
  let mut c = v.to_vec();
  let p = c.as_mut_ptr();
  core::mem::forget(c);
  p
}

/// Frees an index mirror built by `build_cstr_index`.
pub fn destroy_cstr_index(p_cstr_ind: *mut opj_codestream_index_t) {
  if p_cstr_ind.is_null() {
    return;
  }
  // SAFETY: the pointee was produced by build_cstr_index.
  unsafe {
    let cstr_ind = Box::from_raw(p_cstr_ind);
    if !cstr_ind.marker.is_null() {
      let n = cstr_ind.marknum as usize;
      drop(Vec::from_raw_parts(cstr_ind.marker, n, n));
    }
    if !cstr_ind.tile_index.is_null() {
      let n = cstr_ind.nb_of_tiles as usize;
      for tile in core::slice::from_raw_parts_mut(cstr_ind.tile_index, n) {
        if !tile.tp_index.is_null() {
          /* The tp_index allocation is sized `current_nb_tps` (the core keeps
          tp_index.len() == current_nb_tps); `nb_tps` is the *declared* part
          count and can be smaller (even 0) when a later SOT declares no
          parts.  The C code frees with opj_free, which is length-less;
          Vec::from_raw_parts needs the real allocation size. */
          let m = tile.current_nb_tps as usize;
          drop(Vec::from_raw_parts(tile.tp_index, m, m));
        }
        if !tile.marker.is_null() {
          let m = tile.marknum as usize;
          drop(Vec::from_raw_parts(tile.marker, m, m));
        }
        if !tile.packet_index.is_null() {
          let m = tile.nb_packet as usize;
          drop(Vec::from_raw_parts(tile.packet_index, m, m));
        }
      }
      drop(Vec::from_raw_parts(cstr_ind.tile_index, n, n));
    }
  }
}
