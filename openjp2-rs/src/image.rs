use ::c2rust_bitfields;
use ::libc;
extern "C" {
  #[no_mangle]
  fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
  #[no_mangle]
  fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
  #[no_mangle]
  fn opj_free(m: *mut libc::c_void);
  #[no_mangle]
  fn opj_calloc(numOfElements: size_t, sizeOfElements: size_t) -> *mut libc::c_void;
  #[no_mangle]
  fn opj_malloc(size: size_t) -> *mut libc::c_void;
  #[no_mangle]
  fn opj_image_data_free(ptr: *mut libc::c_void);
  #[no_mangle]
  fn opj_image_data_alloc(size: OPJ_SIZE_T) -> *mut libc::c_void;
}
pub type size_t = libc::c_ulong;
pub type __uint16_t = libc::c_ushort;
pub type __int32_t = libc::c_int;
pub type __uint32_t = libc::c_uint;
pub type __uint64_t = libc::c_ulong;
pub type OPJ_BOOL = libc::c_int;
pub type OPJ_CHAR = libc::c_char;
pub type OPJ_FLOAT32 = libc::c_float;
pub type OPJ_FLOAT64 = libc::c_double;
pub type OPJ_BYTE = libc::c_uchar;
pub type int32_t = __int32_t;
pub type uint16_t = __uint16_t;
pub type uint32_t = __uint32_t;
pub type uint64_t = __uint64_t;
pub type OPJ_UINT16 = uint16_t;
pub type OPJ_INT32 = int32_t;
pub type OPJ_UINT32 = uint32_t;
pub type OPJ_UINT64 = uint64_t;
pub type OPJ_SIZE_T = size_t;
pub type PROG_ORDER = libc::c_int;
pub const OPJ_CPRL: PROG_ORDER = 4;
pub const OPJ_PCRL: PROG_ORDER = 3;
pub const OPJ_RPCL: PROG_ORDER = 2;
pub const OPJ_RLCP: PROG_ORDER = 1;
pub const OPJ_LRCP: PROG_ORDER = 0;
pub const OPJ_PROG_UNKNOWN: PROG_ORDER = -1;
pub type OPJ_PROG_ORDER = PROG_ORDER;
pub type COLOR_SPACE = libc::c_int;
pub const OPJ_CLRSPC_CMYK: COLOR_SPACE = 5;
pub const OPJ_CLRSPC_EYCC: COLOR_SPACE = 4;
pub const OPJ_CLRSPC_SYCC: COLOR_SPACE = 3;
pub const OPJ_CLRSPC_GRAY: COLOR_SPACE = 2;
pub const OPJ_CLRSPC_SRGB: COLOR_SPACE = 1;
pub const OPJ_CLRSPC_UNSPECIFIED: COLOR_SPACE = 0;
pub const OPJ_CLRSPC_UNKNOWN: COLOR_SPACE = -1;
pub type OPJ_COLOR_SPACE = COLOR_SPACE;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_poc {
  pub resno0: OPJ_UINT32,
  pub compno0: OPJ_UINT32,
  pub layno1: OPJ_UINT32,
  pub resno1: OPJ_UINT32,
  pub compno1: OPJ_UINT32,
  pub layno0: OPJ_UINT32,
  pub precno0: OPJ_UINT32,
  pub precno1: OPJ_UINT32,
  pub prg1: OPJ_PROG_ORDER,
  pub prg: OPJ_PROG_ORDER,
  pub progorder: [OPJ_CHAR; 5],
  pub tile: OPJ_UINT32,
  pub tx0: OPJ_INT32,
  pub tx1: OPJ_INT32,
  pub ty0: OPJ_INT32,
  pub ty1: OPJ_INT32,
  pub layS: OPJ_UINT32,
  pub resS: OPJ_UINT32,
  pub compS: OPJ_UINT32,
  pub prcS: OPJ_UINT32,
  pub layE: OPJ_UINT32,
  pub resE: OPJ_UINT32,
  pub compE: OPJ_UINT32,
  pub prcE: OPJ_UINT32,
  pub txS: OPJ_UINT32,
  pub txE: OPJ_UINT32,
  pub tyS: OPJ_UINT32,
  pub tyE: OPJ_UINT32,
  pub dx: OPJ_UINT32,
  pub dy: OPJ_UINT32,
  pub lay_t: OPJ_UINT32,
  pub res_t: OPJ_UINT32,
  pub comp_t: OPJ_UINT32,
  pub prc_t: OPJ_UINT32,
  pub tx0_t: OPJ_UINT32,
  pub ty0_t: OPJ_UINT32,
}
pub type opj_poc_t = opj_poc;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_image_comp {
  pub dx: OPJ_UINT32,
  pub dy: OPJ_UINT32,
  pub w: OPJ_UINT32,
  pub h: OPJ_UINT32,
  pub x0: OPJ_UINT32,
  pub y0: OPJ_UINT32,
  pub prec: OPJ_UINT32,
  pub bpp: OPJ_UINT32,
  pub sgnd: OPJ_UINT32,
  pub resno_decoded: OPJ_UINT32,
  pub factor: OPJ_UINT32,
  pub data: *mut OPJ_INT32,
  pub alpha: OPJ_UINT16,
}
pub type opj_image_comp_t = opj_image_comp;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_image {
  pub x0: OPJ_UINT32,
  pub y0: OPJ_UINT32,
  pub x1: OPJ_UINT32,
  pub y1: OPJ_UINT32,
  pub numcomps: OPJ_UINT32,
  pub color_space: OPJ_COLOR_SPACE,
  pub comps: *mut opj_image_comp_t,
  pub icc_profile_buf: *mut OPJ_BYTE,
  pub icc_profile_len: OPJ_UINT32,
}
pub type opj_image_t = opj_image;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_image_comptparm {
  pub dx: OPJ_UINT32,
  pub dy: OPJ_UINT32,
  pub w: OPJ_UINT32,
  pub h: OPJ_UINT32,
  pub x0: OPJ_UINT32,
  pub y0: OPJ_UINT32,
  pub prec: OPJ_UINT32,
  pub bpp: OPJ_UINT32,
  pub sgnd: OPJ_UINT32,
}
pub type opj_image_cmptparm_t = opj_image_comptparm;
pub type OPJ_BITFIELD = libc::c_uint;

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub struct opj_cp {
  pub rsiz: OPJ_UINT16,
  pub tx0: OPJ_UINT32,
  pub ty0: OPJ_UINT32,
  pub tdx: OPJ_UINT32,
  pub tdy: OPJ_UINT32,
  pub comment: *mut OPJ_CHAR,
  pub tw: OPJ_UINT32,
  pub th: OPJ_UINT32,
  pub ppm_markers_count: OPJ_UINT32,
  pub ppm_markers: *mut opj_ppx,
  pub ppm_data: *mut OPJ_BYTE,
  pub ppm_len: OPJ_UINT32,
  pub ppm_data_read: OPJ_UINT32,
  pub ppm_data_current: *mut OPJ_BYTE,
  pub ppm_buffer: *mut OPJ_BYTE,
  pub ppm_data_first: *mut OPJ_BYTE,
  pub ppm_data_size: OPJ_UINT32,
  pub ppm_store: OPJ_INT32,
  pub ppm_previous: OPJ_INT32,
  pub tcps: *mut opj_tcp_t,
  pub m_specific_param: C2RustUnnamed,
  pub strict: OPJ_BOOL,
  #[bitfield(name = "ppm", ty = "OPJ_BITFIELD", bits = "0..=0")]
  #[bitfield(name = "m_is_decoder", ty = "OPJ_BITFIELD", bits = "1..=1")]
  #[bitfield(
    name = "allow_different_bit_depth_sign",
    ty = "OPJ_BITFIELD",
    bits = "2..=2"
  )]
  pub ppm_m_is_decoder_allow_different_bit_depth_sign: [u8; 1],
  #[bitfield(padding)]
  pub c2rust_padding: [u8; 3],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed {
  pub m_dec: opj_decoding_param_t,
  pub m_enc: opj_encoding_param_t,
}
pub type opj_encoding_param_t = opj_encoding_param;

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub struct opj_encoding_param {
  pub m_max_comp_size: OPJ_UINT32,
  pub m_tp_pos: OPJ_INT32,
  pub m_matrice: *mut OPJ_INT32,
  pub m_tp_flag: OPJ_BYTE,
  #[bitfield(name = "m_disto_alloc", ty = "OPJ_BITFIELD", bits = "0..=0")]
  #[bitfield(name = "m_fixed_alloc", ty = "OPJ_BITFIELD", bits = "1..=1")]
  #[bitfield(name = "m_fixed_quality", ty = "OPJ_BITFIELD", bits = "2..=2")]
  #[bitfield(name = "m_tp_on", ty = "OPJ_BITFIELD", bits = "3..=3")]
  pub m_disto_alloc_m_fixed_alloc_m_fixed_quality_m_tp_on: [u8; 1],
  #[bitfield(padding)]
  pub c2rust_padding: [u8; 6],
}
pub type opj_decoding_param_t = opj_decoding_param;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_decoding_param {
  pub m_reduce: OPJ_UINT32,
  pub m_layer: OPJ_UINT32,
}
pub type opj_tcp_t = opj_tcp;

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub struct opj_tcp {
  pub csty: OPJ_UINT32,
  pub prg: OPJ_PROG_ORDER,
  pub numlayers: OPJ_UINT32,
  pub num_layers_to_decode: OPJ_UINT32,
  pub mct: OPJ_UINT32,
  pub rates: [OPJ_FLOAT32; 100],
  pub numpocs: OPJ_UINT32,
  pub pocs: [opj_poc_t; 32],
  pub ppt_markers_count: OPJ_UINT32,
  pub ppt_markers: *mut opj_ppx,
  pub ppt_data: *mut OPJ_BYTE,
  pub ppt_buffer: *mut OPJ_BYTE,
  pub ppt_data_size: OPJ_UINT32,
  pub ppt_len: OPJ_UINT32,
  pub distoratio: [OPJ_FLOAT32; 100],
  pub tccps: *mut opj_tccp_t,
  pub m_current_tile_part_number: OPJ_INT32,
  pub m_nb_tile_parts: OPJ_UINT32,
  pub m_data: *mut OPJ_BYTE,
  pub m_data_size: OPJ_UINT32,
  pub mct_norms: *mut OPJ_FLOAT64,
  pub m_mct_decoding_matrix: *mut OPJ_FLOAT32,
  pub m_mct_coding_matrix: *mut OPJ_FLOAT32,
  pub m_mct_records: *mut opj_mct_data_t,
  pub m_nb_mct_records: OPJ_UINT32,
  pub m_nb_max_mct_records: OPJ_UINT32,
  pub m_mcc_records: *mut opj_simple_mcc_decorrelation_data_t,
  pub m_nb_mcc_records: OPJ_UINT32,
  pub m_nb_max_mcc_records: OPJ_UINT32,
  #[bitfield(name = "cod", ty = "OPJ_BITFIELD", bits = "0..=0")]
  #[bitfield(name = "ppt", ty = "OPJ_BITFIELD", bits = "1..=1")]
  #[bitfield(name = "POC", ty = "OPJ_BITFIELD", bits = "2..=2")]
  pub cod_ppt_POC: [u8; 1],
  #[bitfield(padding)]
  pub c2rust_padding: [u8; 7],
}
pub type opj_simple_mcc_decorrelation_data_t = opj_simple_mcc_decorrelation_data;

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub struct opj_simple_mcc_decorrelation_data {
  pub m_index: OPJ_UINT32,
  pub m_nb_comps: OPJ_UINT32,
  pub m_decorrelation_array: *mut opj_mct_data_t,
  pub m_offset_array: *mut opj_mct_data_t,
  #[bitfield(name = "m_is_irreversible", ty = "OPJ_BITFIELD", bits = "0..=0")]
  pub m_is_irreversible: [u8; 1],
  #[bitfield(padding)]
  pub c2rust_padding: [u8; 7],
}
pub type opj_mct_data_t = opj_mct_data;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_mct_data {
  pub m_element_type: J2K_MCT_ELEMENT_TYPE,
  pub m_array_type: J2K_MCT_ARRAY_TYPE,
  pub m_index: OPJ_UINT32,
  pub m_data: *mut OPJ_BYTE,
  pub m_data_size: OPJ_UINT32,
}
pub type J2K_MCT_ARRAY_TYPE = MCT_ARRAY_TYPE;
pub type MCT_ARRAY_TYPE = libc::c_uint;
pub const MCT_TYPE_OFFSET: MCT_ARRAY_TYPE = 2;
pub const MCT_TYPE_DECORRELATION: MCT_ARRAY_TYPE = 1;
pub const MCT_TYPE_DEPENDENCY: MCT_ARRAY_TYPE = 0;
pub type J2K_MCT_ELEMENT_TYPE = MCT_ELEMENT_TYPE;
pub type MCT_ELEMENT_TYPE = libc::c_uint;
pub const MCT_TYPE_DOUBLE: MCT_ELEMENT_TYPE = 3;
pub const MCT_TYPE_FLOAT: MCT_ELEMENT_TYPE = 2;
pub const MCT_TYPE_INT32: MCT_ELEMENT_TYPE = 1;
pub const MCT_TYPE_INT16: MCT_ELEMENT_TYPE = 0;
pub type opj_tccp_t = opj_tccp;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tccp {
  pub csty: OPJ_UINT32,
  pub numresolutions: OPJ_UINT32,
  pub cblkw: OPJ_UINT32,
  pub cblkh: OPJ_UINT32,
  pub cblksty: OPJ_UINT32,
  pub qmfbid: OPJ_UINT32,
  pub qntsty: OPJ_UINT32,
  pub stepsizes: [opj_stepsize_t; 97],
  pub numgbits: OPJ_UINT32,
  pub roishift: OPJ_INT32,
  pub prcw: [OPJ_UINT32; 33],
  pub prch: [OPJ_UINT32; 33],
  pub m_dc_level_shift: OPJ_INT32,
}
pub type opj_stepsize_t = opj_stepsize;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_stepsize {
  pub expn: OPJ_INT32,
  pub mant: OPJ_INT32,
}
pub type opj_ppx = opj_ppx_struct;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_ppx_struct {
  pub m_data: *mut OPJ_BYTE,
  pub m_data_size: OPJ_UINT32,
}
#[inline]
unsafe extern "C" fn opj_uint_ceildivpow2(mut a: OPJ_UINT32, mut b: OPJ_UINT32) -> OPJ_UINT32 {
  return ((a as libc::c_ulong)
    .wrapping_add((1 as libc::c_uint as OPJ_UINT64) << b)
    .wrapping_sub(1 as libc::c_uint as libc::c_ulong)
    >> b) as OPJ_UINT32;
}
#[inline]
unsafe extern "C" fn opj_uint_ceildiv(mut a: OPJ_UINT32, mut b: OPJ_UINT32) -> OPJ_UINT32 {
  assert!(b != 0);
  return (a as OPJ_UINT64)
    .wrapping_add(b as libc::c_ulong)
    .wrapping_sub(1 as libc::c_int as libc::c_ulong)
    .wrapping_div(b as libc::c_ulong) as OPJ_UINT32;
}
#[inline]
unsafe extern "C" fn opj_uint_adds(mut a: OPJ_UINT32, mut b: OPJ_UINT32) -> OPJ_UINT32 {
  let mut sum = (a as OPJ_UINT64).wrapping_add(b as OPJ_UINT64);
  return -((sum >> 32 as libc::c_int) as OPJ_INT32) as OPJ_UINT32 | sum as OPJ_UINT32;
}
#[inline]
unsafe extern "C" fn opj_uint_min(mut a: OPJ_UINT32, mut b: OPJ_UINT32) -> OPJ_UINT32 {
  return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn opj_uint_max(mut a: OPJ_UINT32, mut b: OPJ_UINT32) -> OPJ_UINT32 {
  return if a > b { a } else { b };
}
/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2005, Herve Drolon, FreeImage Team
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS `AS IS'
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */
#[no_mangle]
pub unsafe extern "C" fn opj_image_create0() -> *mut opj_image_t {
  let mut image = opj_calloc(
    1 as libc::c_int as size_t,
    ::std::mem::size_of::<opj_image_t>() as libc::c_ulong,
  ) as *mut opj_image_t;
  return image;
}
#[no_mangle]
pub unsafe extern "C" fn opj_image_create(
  mut numcmpts: OPJ_UINT32,
  mut cmptparms: *mut opj_image_cmptparm_t,
  mut clrspc: OPJ_COLOR_SPACE,
) -> *mut opj_image_t {
  let mut compno: OPJ_UINT32 = 0;
  let mut image = 0 as *mut opj_image_t;
  image = opj_calloc(
    1 as libc::c_int as size_t,
    ::std::mem::size_of::<opj_image_t>() as libc::c_ulong,
  ) as *mut opj_image_t;
  if !image.is_null() {
    (*image).color_space = clrspc;
    (*image).numcomps = numcmpts;
    /* allocate memory for the per-component information */
    (*image).comps = opj_calloc(
      (*image).numcomps as size_t,
      ::std::mem::size_of::<opj_image_comp_t>() as libc::c_ulong,
    ) as *mut opj_image_comp_t;
    if (*image).comps.is_null() {
      /* TODO replace with event manager, breaks API */
      /* fprintf(stderr,"Unable to allocate memory for image.\n"); */
      opj_image_destroy(image);
      return 0 as *mut opj_image_t;
    }
    /* create the individual image components */
    compno = 0 as libc::c_int as OPJ_UINT32;
    while compno < numcmpts {
      let mut comp: *mut opj_image_comp_t =
        &mut *(*image).comps.offset(compno as isize) as *mut opj_image_comp_t;
      (*comp).dx = (*cmptparms.offset(compno as isize)).dx;
      (*comp).dy = (*cmptparms.offset(compno as isize)).dy;
      (*comp).w = (*cmptparms.offset(compno as isize)).w;
      (*comp).h = (*cmptparms.offset(compno as isize)).h;
      (*comp).x0 = (*cmptparms.offset(compno as isize)).x0;
      (*comp).y0 = (*cmptparms.offset(compno as isize)).y0;
      (*comp).prec = (*cmptparms.offset(compno as isize)).prec;
      (*comp).sgnd = (*cmptparms.offset(compno as isize)).sgnd;
      if (*comp).h != 0 as libc::c_int as libc::c_uint
        && (*comp).w as OPJ_SIZE_T
          > (18446744073709551615 as libc::c_ulong)
            .wrapping_div((*comp).h as libc::c_ulong)
            .wrapping_div(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong)
      {
        /* TODO event manager */
        opj_image_destroy(image);
        return 0 as *mut opj_image_t;
      }
      (*comp).data = opj_image_data_alloc(
        ((*comp).w as size_t)
          .wrapping_mul((*comp).h as libc::c_ulong)
          .wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
      ) as *mut OPJ_INT32;
      if (*comp).data.is_null() {
        /* TODO replace with event manager, breaks API */
        /* fprintf(stderr,"Unable to allocate memory for image.\n"); */
        opj_image_destroy(image);
        return 0 as *mut opj_image_t;
      }
      memset(
        (*comp).data as *mut libc::c_void,
        0 as libc::c_int,
        ((*comp).w as size_t)
          .wrapping_mul((*comp).h as libc::c_ulong)
          .wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
      );
      compno = compno.wrapping_add(1)
    }
  }
  return image;
}
#[no_mangle]
pub unsafe extern "C" fn opj_image_destroy(mut image: *mut opj_image_t) {
  if !image.is_null() {
    if !(*image).comps.is_null() {
      let mut compno: OPJ_UINT32 = 0;
      /* image components */
      compno = 0 as libc::c_int as OPJ_UINT32;
      while compno < (*image).numcomps {
        let mut image_comp: *mut opj_image_comp_t =
          &mut *(*image).comps.offset(compno as isize) as *mut opj_image_comp_t;
        if !(*image_comp).data.is_null() {
          opj_image_data_free((*image_comp).data as *mut libc::c_void);
        }
        compno = compno.wrapping_add(1)
      }
      opj_free((*image).comps as *mut libc::c_void);
    }
    if !(*image).icc_profile_buf.is_null() {
      opj_free((*image).icc_profile_buf as *mut libc::c_void);
    }
    opj_free(image as *mut libc::c_void);
  };
}
/* *
 * Updates the components characteristics of the image from the coding parameters.
 *
 * @param p_image_header    the image header to update.
 * @param p_cp              the coding parameters from which to update the image.
 */
#[no_mangle]
pub unsafe extern "C" fn opj_image_comp_header_update(
  mut p_image_header: *mut opj_image_t,
  mut p_cp: *const opj_cp,
) {
  let mut i: OPJ_UINT32 = 0; /* validity of p_cp members used here checked in opj_j2k_read_siz. Can't overflow. */
  let mut l_width: OPJ_UINT32 = 0; /* can't overflow */
  let mut l_height: OPJ_UINT32 = 0; /* use add saturated to prevent overflow */
  let mut l_x0: OPJ_UINT32 = 0; /* use add saturated to prevent overflow */
  let mut l_y0: OPJ_UINT32 = 0;
  let mut l_x1: OPJ_UINT32 = 0;
  let mut l_y1: OPJ_UINT32 = 0;
  let mut l_comp_x0: OPJ_UINT32 = 0;
  let mut l_comp_y0: OPJ_UINT32 = 0;
  let mut l_comp_x1: OPJ_UINT32 = 0;
  let mut l_comp_y1: OPJ_UINT32 = 0;
  let mut l_img_comp = 0 as *mut opj_image_comp_t;
  l_x0 = opj_uint_max((*p_cp).tx0, (*p_image_header).x0);
  l_y0 = opj_uint_max((*p_cp).ty0, (*p_image_header).y0);
  l_x1 = (*p_cp).tx0.wrapping_add(
    (*p_cp)
      .tw
      .wrapping_sub(1 as libc::c_uint)
      .wrapping_mul((*p_cp).tdx),
  );
  l_y1 = (*p_cp).ty0.wrapping_add(
    (*p_cp)
      .th
      .wrapping_sub(1 as libc::c_uint)
      .wrapping_mul((*p_cp).tdy),
  );
  l_x1 = opj_uint_min(opj_uint_adds(l_x1, (*p_cp).tdx), (*p_image_header).x1);
  l_y1 = opj_uint_min(opj_uint_adds(l_y1, (*p_cp).tdy), (*p_image_header).y1);
  l_img_comp = (*p_image_header).comps;
  i = 0 as libc::c_int as OPJ_UINT32;
  while i < (*p_image_header).numcomps {
    l_comp_x0 = opj_uint_ceildiv(l_x0, (*l_img_comp).dx);
    l_comp_y0 = opj_uint_ceildiv(l_y0, (*l_img_comp).dy);
    l_comp_x1 = opj_uint_ceildiv(l_x1, (*l_img_comp).dx);
    l_comp_y1 = opj_uint_ceildiv(l_y1, (*l_img_comp).dy);
    l_width = opj_uint_ceildivpow2(l_comp_x1.wrapping_sub(l_comp_x0), (*l_img_comp).factor);
    l_height = opj_uint_ceildivpow2(l_comp_y1.wrapping_sub(l_comp_y0), (*l_img_comp).factor);
    (*l_img_comp).w = l_width;
    (*l_img_comp).h = l_height;
    (*l_img_comp).x0 = l_comp_x0;
    (*l_img_comp).y0 = l_comp_y0;
    l_img_comp = l_img_comp.offset(1);
    i = i.wrapping_add(1)
  }
}
/* *
 * Copy only header of image and its component header (no data are copied)
 * if dest image have data, they will be freed
 *
 * @param   p_image_src     the src image
 * @param   p_image_dest    the dest image
 *
 */
#[no_mangle]
pub unsafe extern "C" fn opj_copy_image_header(
  mut p_image_src: *const opj_image_t,
  mut p_image_dest: *mut opj_image_t,
) {
  let mut compno: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_image_src.is_null());
  assert!(!p_image_dest.is_null());
  (*p_image_dest).x0 = (*p_image_src).x0;
  (*p_image_dest).y0 = (*p_image_src).y0;
  (*p_image_dest).x1 = (*p_image_src).x1;
  (*p_image_dest).y1 = (*p_image_src).y1;
  if !(*p_image_dest).comps.is_null() {
    compno = 0 as libc::c_int as OPJ_UINT32;
    while compno < (*p_image_dest).numcomps {
      let mut image_comp: *mut opj_image_comp_t =
        &mut *(*p_image_dest).comps.offset(compno as isize) as *mut opj_image_comp_t;
      if !(*image_comp).data.is_null() {
        opj_image_data_free((*image_comp).data as *mut libc::c_void);
      }
      compno = compno.wrapping_add(1)
    }
    opj_free((*p_image_dest).comps as *mut libc::c_void);
    (*p_image_dest).comps = 0 as *mut opj_image_comp_t
  }
  (*p_image_dest).numcomps = (*p_image_src).numcomps;
  (*p_image_dest).comps = opj_malloc(
    ((*p_image_dest).numcomps as libc::c_ulong)
      .wrapping_mul(::std::mem::size_of::<opj_image_comp_t>() as libc::c_ulong),
  ) as *mut opj_image_comp_t;
  if (*p_image_dest).comps.is_null() {
    (*p_image_dest).comps = 0 as *mut opj_image_comp_t;
    (*p_image_dest).numcomps = 0 as libc::c_int as OPJ_UINT32;
    return;
  }
  compno = 0 as libc::c_int as OPJ_UINT32;
  while compno < (*p_image_dest).numcomps {
    memcpy(
      &mut *(*p_image_dest).comps.offset(compno as isize) as *mut opj_image_comp_t
        as *mut libc::c_void,
      &mut *(*p_image_src).comps.offset(compno as isize) as *mut opj_image_comp_t
        as *const libc::c_void,
      ::std::mem::size_of::<opj_image_comp_t>() as libc::c_ulong,
    );
    let ref mut fresh0 = (*(*p_image_dest).comps.offset(compno as isize)).data;
    *fresh0 = 0 as *mut OPJ_INT32;
    compno = compno.wrapping_add(1)
  }
  (*p_image_dest).color_space = (*p_image_src).color_space;
  (*p_image_dest).icc_profile_len = (*p_image_src).icc_profile_len;
  if (*p_image_dest).icc_profile_len != 0 {
    (*p_image_dest).icc_profile_buf =
      opj_malloc((*p_image_dest).icc_profile_len as size_t) as *mut OPJ_BYTE;
    if (*p_image_dest).icc_profile_buf.is_null() {
      (*p_image_dest).icc_profile_buf = 0 as *mut OPJ_BYTE;
      (*p_image_dest).icc_profile_len = 0 as libc::c_int as OPJ_UINT32;
      return;
    }
    memcpy(
      (*p_image_dest).icc_profile_buf as *mut libc::c_void,
      (*p_image_src).icc_profile_buf as *const libc::c_void,
      (*p_image_src).icc_profile_len as libc::c_ulong,
    );
  } else {
    (*p_image_dest).icc_profile_buf = 0 as *mut OPJ_BYTE
  };
}
#[no_mangle]
pub unsafe extern "C" fn opj_image_tile_create(
  mut numcmpts: OPJ_UINT32,
  mut cmptparms: *mut opj_image_cmptparm_t,
  mut clrspc: OPJ_COLOR_SPACE,
) -> *mut opj_image_t {
  let mut compno: OPJ_UINT32 = 0;
  let mut image = 0 as *mut opj_image_t;
  image = opj_calloc(
    1 as libc::c_int as size_t,
    ::std::mem::size_of::<opj_image_t>() as libc::c_ulong,
  ) as *mut opj_image_t;
  if !image.is_null() {
    (*image).color_space = clrspc;
    (*image).numcomps = numcmpts;
    /* allocate memory for the per-component information */
    (*image).comps = opj_calloc(
      (*image).numcomps as size_t,
      ::std::mem::size_of::<opj_image_comp_t>() as libc::c_ulong,
    ) as *mut opj_image_comp_t;
    if (*image).comps.is_null() {
      opj_image_destroy(image);
      return 0 as *mut opj_image_t;
    }
    /* create the individual image components */
    compno = 0 as libc::c_int as OPJ_UINT32;
    while compno < numcmpts {
      let mut comp: *mut opj_image_comp_t =
        &mut *(*image).comps.offset(compno as isize) as *mut opj_image_comp_t;
      (*comp).dx = (*cmptparms.offset(compno as isize)).dx;
      (*comp).dy = (*cmptparms.offset(compno as isize)).dy;
      (*comp).w = (*cmptparms.offset(compno as isize)).w;
      (*comp).h = (*cmptparms.offset(compno as isize)).h;
      (*comp).x0 = (*cmptparms.offset(compno as isize)).x0;
      (*comp).y0 = (*cmptparms.offset(compno as isize)).y0;
      (*comp).prec = (*cmptparms.offset(compno as isize)).prec;
      (*comp).sgnd = (*cmptparms.offset(compno as isize)).sgnd;
      (*comp).data = 0 as *mut OPJ_INT32;
      compno = compno.wrapping_add(1)
    }
  }
  return image;
}
