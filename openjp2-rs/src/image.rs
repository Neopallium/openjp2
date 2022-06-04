use super::math::*;
use super::openjpeg::*;
use ::libc;

use super::malloc::*;

extern "C" {
  fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;

  fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
}
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
pub(crate) unsafe fn opj_image_create0() -> *mut opj_image_t {
  let mut image = opj_calloc(
    1i32 as size_t,
    ::std::mem::size_of::<opj_image_t>() as libc::c_ulong,
  ) as *mut opj_image_t;
  return image;
}
#[no_mangle]
pub(crate) unsafe fn opj_image_create(
  mut numcmpts: OPJ_UINT32,
  mut cmptparms: *mut opj_image_cmptparm_t,
  mut clrspc: OPJ_COLOR_SPACE,
) -> *mut opj_image_t {
  let mut compno: OPJ_UINT32 = 0;
  let mut image = 0 as *mut opj_image_t;
  image = opj_calloc(
    1i32 as size_t,
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
    compno = 0 as OPJ_UINT32;
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
      if (*comp).h != 0u32
        && (*comp).w as OPJ_SIZE_T
          > (18446744073709551615u64)
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
        0i32,
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
pub(crate) unsafe fn opj_image_destroy(mut image: *mut opj_image_t) {
  if !image.is_null() {
    if !(*image).comps.is_null() {
      let mut compno: OPJ_UINT32 = 0;
      /* image components */
      compno = 0 as OPJ_UINT32;
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
pub(crate) unsafe fn opj_image_comp_header_update(
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
      .wrapping_sub(1u32)
      .wrapping_mul((*p_cp).tdx),
  );
  l_y1 = (*p_cp).ty0.wrapping_add(
    (*p_cp)
      .th
      .wrapping_sub(1u32)
      .wrapping_mul((*p_cp).tdy),
  );
  l_x1 = opj_uint_min(opj_uint_adds(l_x1, (*p_cp).tdx), (*p_image_header).x1);
  l_y1 = opj_uint_min(opj_uint_adds(l_y1, (*p_cp).tdy), (*p_image_header).y1);
  l_img_comp = (*p_image_header).comps;
  i = 0 as OPJ_UINT32;
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
pub(crate) unsafe fn opj_copy_image_header(
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
    compno = 0 as OPJ_UINT32;
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
    (*p_image_dest).numcomps = 0 as OPJ_UINT32;
    return;
  }
  compno = 0 as OPJ_UINT32;
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
      (*p_image_dest).icc_profile_len = 0 as OPJ_UINT32;
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
pub(crate) unsafe fn opj_image_tile_create(
  mut numcmpts: OPJ_UINT32,
  mut cmptparms: *mut opj_image_cmptparm_t,
  mut clrspc: OPJ_COLOR_SPACE,
) -> *mut opj_image_t {
  let mut compno: OPJ_UINT32 = 0;
  let mut image = 0 as *mut opj_image_t;
  image = opj_calloc(
    1i32 as size_t,
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
    compno = 0 as OPJ_UINT32;
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
