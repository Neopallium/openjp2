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

pub struct ImageCompRef<'a> {
  /// Component.
  pub comp: &'a CoreImageComp,
  /// Adjustment value to convert from signed to unsigned.
  pub adjust: i32,
  /// Component data.
  pub data: &'a [i32],
}

pub(crate) fn opj_image_comp_header_update(p_image_header: &mut CoreImage, p_cp: &opj_cp) {
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
  for comp in &mut p_image_header.comps {
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

pub(crate) fn opj_copy_image_header_to_core(p_image_src: &CoreImage, p_image_dest: &mut CoreImage) {
  *p_image_dest = p_image_src.clone_header_only();
}

#[derive(Clone, Default)]
pub struct CoreImageComp {
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
  pub alpha: OPJ_UINT16,
  pub data: Vec<OPJ_INT32>,
}

#[derive(Clone, Default)]
pub struct CoreImage {
  pub x0: OPJ_UINT32,
  pub y0: OPJ_UINT32,
  pub x1: OPJ_UINT32,
  pub y1: OPJ_UINT32,
  pub numcomps: OPJ_UINT32,
  pub color_space: OPJ_COLOR_SPACE,
  pub comps: Vec<CoreImageComp>,
  pub icc_profile_buf: Vec<OPJ_BYTE>,
  /// A non-empty profile buffer with this flag means the profile is CIELab
  /// (mirrors the C ABI convention of icc_profile_len == 0).
  pub icc_profile_is_cielab: bool,
}

impl CoreImageComp {
  /// Data accessor kept for compatibility with the C-ABI image API.
  pub fn data(&self) -> Option<&[i32]> {
    Some(&self.data)
  }

  pub fn data_mut(&mut self) -> Option<&mut [i32]> {
    Some(&mut self.data)
  }

  /// Allocate (zeroed) data for w * h samples.
  pub fn alloc_data(&mut self) -> bool {
    self.data = vec![0; (self.w as usize).wrapping_mul(self.h as usize)];
    true
  }

  pub fn clear_data(&mut self) {
    self.data.clear();
  }

  pub fn set_dims(&mut self, w: u32, h: u32) {
    if self.w == w && self.h == h {
      return;
    }
    self.w = w;
    self.h = h;
    self.data.clear();
  }

  pub fn set_data(&mut self, data: &[i32]) -> bool {
    self.data = data.to_vec();
    true
  }

  /// Copy component properties (not data).
  pub fn copy_props(&mut self, other: &CoreImageComp) {
    self.data.clear();
    self.dx = other.dx;
    self.dy = other.dy;
    self.w = other.w;
    self.h = other.h;
    self.x0 = other.x0;
    self.y0 = other.y0;
    self.prec = other.prec;
    self.bpp = other.bpp;
    self.sgnd = other.sgnd;
    self.resno_decoded = other.resno_decoded;
    self.factor = other.factor;
    self.alpha = other.alpha;
  }

  /// Copy properties and data from another core component.
  pub fn copy(&mut self, other: &CoreImageComp) -> bool {
    *self = other.clone();
    true
  }

  /// Clip component samples to `precision` bits.
  pub fn clip(&mut self, precision: u32) {
    let signed = self.sgnd != 0;
    let (min, max) = match (precision, signed) {
      (0..=31, false) => (0, (1i64 << precision) - 1),
      (0..=31, true) => {
        let max = (1i64 << (precision - 1)) - 1;
        let min = -max - 1;
        (min, max)
      }
      _ => (0, i64::MAX),
    };
    for val in self.data.iter_mut() {
      *val = (*val as i64).clamp(min, max) as i32;
    }
    self.prec = precision;
  }

  /// Scale component samples to `precision` bits.
  pub fn scale(&mut self, precision: u32) {
    if self.prec == precision {
      return;
    }
    if self.prec < precision {
      let signed = self.sgnd != 0;
      let old_prec = self.prec;
      if signed {
        let new_max = 1i64 << (precision - 1);
        let old_max = 1i64 << (old_prec - 1);
        for val in self.data.iter_mut() {
          *val = (((*val as i64) * new_max) / old_max) as i32;
        }
      } else {
        let new_max = (1u64 << precision) - 1;
        let old_max = (1u64 << old_prec) - 1;
        for val in self.data.iter_mut() {
          *val = (((*val as u64) * new_max) / old_max) as i32;
        }
      }
      self.prec = precision;
      return;
    }
    let sgnd = self.sgnd != 0;
    let shift = self.prec - precision;
    if sgnd {
      for val in self.data.iter_mut() {
        *val >>= shift;
      }
    } else {
      for val in self.data.iter_mut() {
        *val = ((*val as u32) >> shift) as i32;
      }
    }
    self.prec = precision;
  }
}

impl CoreImage {
  pub fn new() -> CoreImage {
    CoreImage::default()
  }

  /// Header-only image from component parameters (no data), like
  /// opj_image_tile_create.
  pub fn tile_create(
    cmptparms: &[opj_image_comptparm],
    clrspc: OPJ_COLOR_SPACE,
  ) -> Option<CoreImage> {
    let mut image = CoreImage::default();
    image.color_space = clrspc;
    if !image.alloc_comps(cmptparms.len() as u32) {
      return None;
    }
    for (comp, params) in image.comps.iter_mut().zip(cmptparms) {
      comp.dx = params.dx;
      comp.dy = params.dy;
      comp.w = params.w;
      comp.h = params.h;
      comp.x0 = params.x0;
      comp.y0 = params.y0;
      comp.prec = params.prec;
      comp.sgnd = params.sgnd;
    }
    Some(image)
  }

  /// Compatibility accessors over the owned Vecs.
  pub fn comps(&self) -> Option<&[CoreImageComp]> {
    Some(&self.comps)
  }

  pub fn comps_mut(&mut self) -> Option<&mut [CoreImageComp]> {
    Some(&mut self.comps)
  }

  pub fn comps_data_iter(&self) -> Option<impl Iterator<Item = ImageCompRef<'_>>> {
    Some(self.comps.iter().map(|comp| ImageCompRef {
      comp,
      adjust: if comp.sgnd != 0 {
        1 << (comp.prec - 1)
      } else {
        0
      },
      data: &comp.data,
    }))
  }

  pub fn take_comps(&mut self) -> CoreImage {
    let mut image = CoreImage::default();
    image.x0 = self.x0;
    image.y0 = self.y0;
    image.x1 = self.x1;
    image.y1 = self.y1;
    image.numcomps = self.numcomps;
    image.color_space = self.color_space;
    image.comps = core::mem::take(&mut self.comps);
    self.numcomps = 0;
    image
  }

  pub fn set_rgb(&mut self, w: usize, h: usize, r: &[i32], g: &[i32], b: &[i32]) -> bool {
    let len = w * h;
    if r.len() != len || g.len() != len || b.len() != len || self.comps.len() < 3 {
      return false;
    }
    let w = w as u32;
    let h = h as u32;
    self.comps[0].set_dims(w, h);
    self.comps[1].set_dims(w, h);
    self.comps[2].set_dims(w, h);
    /* Update component. */
    self.comps[1].dx = self.comps[0].dx;
    self.comps[2].dx = self.comps[0].dx;
    self.comps[1].dy = self.comps[0].dy;
    self.comps[2].dy = self.comps[0].dy;
    self.comps[0].set_data(r);
    self.comps[1].set_data(g);
    self.comps[2].set_data(b);
    self.color_space = OPJ_CLRSPC_SRGB;
    true
  }

  pub fn comps_data_mut_iter(&mut self) -> Option<impl Iterator<Item = &mut [i32]>> {
    Some(self.comps.iter_mut().map(|c| &mut c.data[..]))
  }

  pub fn take_icc_profile(&mut self) -> Option<ICCProfile> {
    if self.icc_profile_buf.is_empty() && !self.icc_profile_is_cielab {
      None
    } else {
      let buf = core::mem::take(&mut self.icc_profile_buf);
      let profile = if self.icc_profile_is_cielab {
        ICCProfile::new_cielab(&buf)
      } else {
        ICCProfile::new_icc(&buf)
      };
      self.icc_profile_is_cielab = false;
      Some(profile)
    }
  }

  pub fn clear_icc_profile(&mut self) {
    self.icc_profile_buf.clear();
    self.icc_profile_is_cielab = false;
  }

  /// Deep copy of a C-ABI image (header, component properties, data and ICC profile).
  /// Copy of the image header (component properties, empty data).
  pub(crate) fn clone_header_only(&self) -> CoreImage {
    CoreImage {
      x0: self.x0,
      y0: self.y0,
      x1: self.x1,
      y1: self.y1,
      numcomps: self.numcomps,
      color_space: self.color_space,
      comps: self
        .comps
        .iter()
        .map(|c| CoreImageComp {
          dx: c.dx,
          dy: c.dy,
          w: c.w,
          h: c.h,
          x0: c.x0,
          y0: c.y0,
          prec: c.prec,
          bpp: c.bpp,
          sgnd: c.sgnd,
          resno_decoded: c.resno_decoded,
          factor: c.factor,
          alpha: c.alpha,
          data: Vec::new(),
        })
        .collect(),
      icc_profile_buf: self.icc_profile_buf.clone(),
      icc_profile_is_cielab: self.icc_profile_is_cielab,
    }
  }

  pub fn alloc_comps(&mut self, numcomps: u32) -> bool {
    self.comps.clear();
    self.numcomps = numcomps;
    self.comps.resize(numcomps as usize, CoreImageComp::default());
    true
  }

  pub fn comp0_dims_prec(&self) -> (usize, usize, i32) {
    if let Some(comp) = self.comps.first() {
      return (comp.w as usize, comp.h as usize, comp.prec as i32);
    }
    (0, 0, 0)
  }

  /// Check if all components have the same dimensions.
  pub fn comps_same_dims(&self) -> bool {
    if let Some((c0, comps)) = self.comps.split_first() {
      for comp in comps {
        if comp.w != c0.w || comp.h != c0.h || comp.dx != c0.dx || comp.dy != c0.dy {
          return false;
        }
      }
      return true;
    }
    false
  }

  /// Check if all components have the same dimensions and precision.
  pub fn comps_match(&self) -> bool {
    if let Some((c0, comps)) = self.comps.split_first() {
      for comp in comps {
        if comp.w != c0.w
          || comp.h != c0.h
          || comp.dx != c0.dx
          || comp.dy != c0.dy
          || comp.prec != c0.prec
          || comp.sgnd != c0.sgnd
        {
          return false;
        }
      }
      return true;
    }
    false
  }
}

