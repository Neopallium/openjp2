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

use super::math::*;
use super::openjpeg::*;

use super::malloc::*;

extern "C" {
  fn memset(_: *mut core::ffi::c_void, _: core::ffi::c_int, _: usize) -> *mut core::ffi::c_void;
}

impl Default for opj_image_comp {
  fn default() -> Self {
    Self {
      dx: 0,
      dy: 0,
      w: 0,
      h: 0,
      x0: 0,
      y0: 0,
      prec: 0,
      bpp: 0,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      data: std::ptr::null_mut(),
      alpha: 0,
    }
  }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
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

#[repr(C)]
#[derive(Clone)]
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

impl opj_image {
  pub fn new() -> Box<Self> {
    Box::new(Self {
      x0: 0,
      y0: 0,
      x1: 0,
      y1: 0,
      numcomps: 0,
      color_space: Default::default(),
      comps: std::ptr::null_mut(),
      icc_profile_buf: std::ptr::null_mut(),
      icc_profile_len: 0,
    })
  }

  pub fn comps(&self) -> Option<&[opj_image_comp]> {
    if self.comps.is_null() {
      None
    } else {
      unsafe {
        Some(std::slice::from_raw_parts(
          self.comps,
          self.numcomps as usize,
        ))
      }
    }
  }

  pub fn comps_mut(&mut self) -> Option<&mut [opj_image_comp]> {
    if self.comps.is_null() {
      None
    } else {
      unsafe {
        Some(std::slice::from_raw_parts_mut(
          self.comps,
          self.numcomps as usize,
        ))
      }
    }
  }

  pub fn clear_comps(&mut self) {
    unsafe {
      if let Some(comps) = self.comps() {
        /* image components */
        for comp in comps {
          if !comp.data.is_null() {
            opj_image_data_free(comp.data as *mut core::ffi::c_void);
          }
        }
        opj_free(self.comps as *mut core::ffi::c_void);
        self.comps = std::ptr::null_mut();
        self.numcomps = 0;
      }
    }
  }

  pub fn alloc_comps(&mut self, numcomps: u32, clear: bool) -> bool {
    self.clear_comps();
    unsafe {
      self.numcomps = numcomps;
      self.comps = if clear {
        opj_calloc(numcomps as size_t, core::mem::size_of::<opj_image_comp_t>())
      } else {
        opj_malloc(numcomps as size_t * core::mem::size_of::<opj_image_comp_t>())
      } as *mut opj_image_comp_t;
      if self.comps.is_null() {
        return false;
      }
    }
    true
  }

  pub fn icc_profile(&self) -> Option<&[u8]> {
    if self.icc_profile_buf.is_null() {
      None
    } else {
      unsafe {
        Some(std::slice::from_raw_parts(
          self.icc_profile_buf,
          self.icc_profile_len as usize,
        ))
      }
    }
  }

  pub fn icc_profile_mut(&self) -> Option<&mut [u8]> {
    if self.icc_profile_buf.is_null() {
      None
    } else {
      unsafe {
        Some(std::slice::from_raw_parts_mut(
          self.icc_profile_buf,
          self.icc_profile_len as usize,
        ))
      }
    }
  }

  pub fn clear_icc_profile(&mut self) {
    unsafe {
      if !self.icc_profile_buf.is_null() {
        opj_free(self.icc_profile_buf as *mut core::ffi::c_void);
        self.icc_profile_buf = std::ptr::null_mut();
        self.icc_profile_len = 0;
      }
    }
  }

  fn alloc_icc_profile(&mut self, len: u32) -> bool {
    unsafe {
      self.icc_profile_buf = opj_malloc(len as size_t) as *mut OPJ_BYTE;
      if self.icc_profile_buf.is_null() {
        self.icc_profile_len = 0 as OPJ_UINT32;
        return false;
      }
      self.icc_profile_len = len;
    }
    true
  }

  pub fn copy_icc_profile(&mut self, icc_profile: &[u8]) -> bool {
    if icc_profile.len() == 0 {
      self.clear_icc_profile();
      return true;
    }
    if self.alloc_icc_profile(icc_profile.len() as u32) {
      if let Some(dest) = self.icc_profile_mut() {
        dest.copy_from_slice(icc_profile);
        return true;
      }
    }
    false
  }
}

impl Drop for opj_image {
  fn drop(&mut self) {
    self.clear_comps();
    self.clear_icc_profile();
  }
}

pub(crate) fn opj_image_create0() -> *mut opj_image_t {
  let image = opj_image::new();
  Box::into_raw(image)
}

#[no_mangle]
pub fn opj_image_create(
  mut numcmpts: OPJ_UINT32,
  mut cmptparms: *mut opj_image_cmptparm_t,
  mut clrspc: OPJ_COLOR_SPACE,
) -> *mut opj_image_t {
  assert!(!cmptparms.is_null());
  let mut image = opj_image::new();
  let cmptparms = unsafe { std::slice::from_raw_parts(cmptparms, numcmpts as usize) };
  image.color_space = clrspc;
  /* allocate memory for the per-component information */
  if !image.alloc_comps(numcmpts, true) {
    /* TODO replace with event manager, breaks API */
    /* fprintf(stderr,"Unable to allocate memory for image.\n"); */
    return std::ptr::null_mut::<opj_image_t>();
  }
  /* create the individual image components */
  let comps = image.comps_mut().unwrap();
  for (comp, params) in comps.iter_mut().zip(cmptparms) {
    comp.dx = params.dx;
    comp.dy = params.dy;
    comp.w = params.w;
    comp.h = params.h;
    comp.x0 = params.x0;
    comp.y0 = params.y0;
    comp.prec = params.prec;
    comp.sgnd = params.sgnd;
    if comp.h != 0u32
      && comp.w as OPJ_SIZE_T
        > (usize::MAX)
          .wrapping_div(comp.h as usize)
          .wrapping_div(core::mem::size_of::<OPJ_INT32>())
    {
      /* TODO event manager */
      return std::ptr::null_mut::<opj_image_t>();
    }
    unsafe {
      comp.data = opj_image_data_alloc(
        (comp.w as size_t)
          .wrapping_mul(comp.h as usize)
          .wrapping_mul(core::mem::size_of::<OPJ_INT32>()),
      ) as *mut OPJ_INT32;
      if comp.data.is_null() {
        /* TODO replace with event manager, breaks API */
        /* fprintf(stderr,"Unable to allocate memory for image.\n"); */
        return std::ptr::null_mut::<opj_image_t>();
      }
      memset(
        comp.data as *mut core::ffi::c_void,
        0i32,
        (comp.w as size_t)
          .wrapping_mul(comp.h as usize)
          .wrapping_mul(core::mem::size_of::<OPJ_INT32>()),
      );
    }
  }
  Box::into_raw(image)
}

#[no_mangle]
pub fn opj_image_destroy(mut image: *mut opj_image_t) {
  if !image.is_null() {
    // Convert back to a boxed value and drop it.
    let _ = unsafe { Box::from_raw(image) };
  }
}

/* *
 * Updates the components characteristics of the image from the coding parameters.
 *
 * @param p_image_header    the image header to update.
 * @param p_cp              the coding parameters from which to update the image.
 */
pub(crate) fn opj_image_comp_header_update(
  mut p_image_header: *mut opj_image_t,
  mut p_cp: *const opj_cp,
) {
  assert!(!p_image_header.is_null());
  assert!(!p_cp.is_null());
  let (p_image_header, p_cp) = unsafe { (&mut *p_image_header, &*p_cp) };
  let l_x0 = opj_uint_max(p_cp.tx0, p_image_header.x0);
  let l_y0 = opj_uint_max(p_cp.ty0, p_image_header.y0);
  let l_x1 = p_cp
    .tx0
    .wrapping_add(p_cp.tw.wrapping_sub(1u32).wrapping_mul(p_cp.tdx));
  let l_y1 = p_cp
    .ty0
    .wrapping_add(p_cp.th.wrapping_sub(1u32).wrapping_mul(p_cp.tdy));
  let l_x1 = opj_uint_min(opj_uint_adds(l_x1, p_cp.tdx), p_image_header.x1);
  let l_y1 = opj_uint_min(opj_uint_adds(l_y1, p_cp.tdy), p_image_header.y1);
  if let Some(comps) = p_image_header.comps_mut() {
    for comp in comps {
      let l_comp_x0 = opj_uint_ceildiv(l_x0, comp.dx);
      let l_comp_y0 = opj_uint_ceildiv(l_y0, comp.dy);
      let l_comp_x1 = opj_uint_ceildiv(l_x1, comp.dx);
      let l_comp_y1 = opj_uint_ceildiv(l_y1, comp.dy);
      let l_width = opj_uint_ceildivpow2(l_comp_x1.wrapping_sub(l_comp_x0), comp.factor);
      let l_height = opj_uint_ceildivpow2(l_comp_y1.wrapping_sub(l_comp_y0), comp.factor);
      comp.w = l_width;
      comp.h = l_height;
      comp.x0 = l_comp_x0;
      comp.y0 = l_comp_y0;
    }
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
pub(crate) fn opj_copy_image_header(
  mut p_image_src: *const opj_image_t,
  mut p_image_dest: *mut opj_image_t,
) {
  let (p_image_src, p_image_dest) = unsafe {
    /* preconditions */
    assert!(!p_image_src.is_null());
    assert!(!p_image_dest.is_null());

    let p_image_src = &*p_image_src;
    let p_image_dest = &mut *p_image_dest;
    (p_image_src, p_image_dest)
  };
  p_image_dest.x0 = p_image_src.x0;
  p_image_dest.y0 = p_image_src.y0;
  p_image_dest.x1 = p_image_src.x1;
  p_image_dest.y1 = p_image_src.y1;
  if !p_image_dest.alloc_comps(p_image_src.numcomps, false) {
    p_image_dest.comps = std::ptr::null_mut::<opj_image_comp_t>();
    p_image_dest.numcomps = 0 as OPJ_UINT32;
    return;
  }
  if let Some(src) = p_image_src.comps() {
    if let Some(dest) = p_image_dest.comps_mut() {
      for (src, dest) in src.iter().zip(dest) {
        *dest = *src;
        dest.data = std::ptr::null_mut::<OPJ_INT32>();
      }
    }
  }
  p_image_dest.color_space = p_image_src.color_space;
  if let Some(icc_profile) = p_image_src.icc_profile() {
    if !p_image_dest.copy_icc_profile(icc_profile) {
      return;
    }
  }
}

#[no_mangle]
pub fn opj_image_tile_create(
  mut numcmpts: OPJ_UINT32,
  mut cmptparms: *mut opj_image_cmptparm_t,
  mut clrspc: OPJ_COLOR_SPACE,
) -> *mut opj_image_t {
  assert!(!cmptparms.is_null());
  let mut image = opj_image::new();
  let cmptparms = unsafe { std::slice::from_raw_parts(cmptparms, numcmpts as usize) };
  image.color_space = clrspc;
  /* allocate memory for the per-component information */
  if !image.alloc_comps(numcmpts, true) {
    return std::ptr::null_mut::<opj_image_t>();
  }
  /* create the individual image components */
  let comps = image.comps_mut().unwrap();
  for (comp, params) in comps.iter_mut().zip(cmptparms) {
    comp.dx = params.dx;
    comp.dy = params.dy;
    comp.w = params.w;
    comp.h = params.h;
    comp.x0 = params.x0;
    comp.y0 = params.y0;
    comp.prec = params.prec;
    comp.sgnd = params.sgnd;
    comp.data = std::ptr::null_mut::<OPJ_INT32>();
  }
  Box::into_raw(image)
}
