//! C-ABI image types (`opj_image`, `opj_image_comp`) and the marshalling
//! helpers that cross the boundary between them and the core `CoreImage`.
//!
//! These moved out of the core in Phase 6.c: the codec modules only carry the
//! owned `CoreImage`; the C surface builds/destroys the `#[repr(C)]` mirrors
//! here.

use openjp2_ctypes::*;
pub use openjp2_core::{opj_image_cmptparm_t, opj_image_comptparm};
use openjp2_core::{CoreImage, CoreImageComp, ICCProfile, ICCProfileMut, ICCProfileRef, CIE_LAB_BYTE_SIZE};

/// Allocate `size` bytes for image component data (malloc-backed, so C
/// callers may free the buffer with `opj_image_data_free`).
pub fn opj_image_data_alloc(size: OPJ_SIZE_T) -> *mut core::ffi::c_void {
  if size == 0 {
    return core::ptr::null_mut();
  }
  crate::params::c_malloc(size)
}

/// Free an image component data buffer allocated by `opj_image_data_alloc`.
pub fn opj_image_data_free(ptr: *mut core::ffi::c_void) {
  if !ptr.is_null() {
    crate::params::c_free(ptr);
  }
}

/// Component of a C-ABI image (mirror of `CoreImageComp` with a raw data
/// buffer, so C callers can index and free it directly).
#[repr(C)]
#[derive(Copy, Clone, Debug)]
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
      data: core::ptr::null_mut(),
      alpha: 0,
    }
  }
}

impl opj_image_comp {
  /// Copy just the component properties, not the data.
  pub fn copy_props(&mut self, other: &opj_image_comp) {
    self.clear_data();
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

  /// Copy another component and its data.
  pub fn copy(&mut self, other: &opj_image_comp) -> bool {
    self.copy_props(other);
    if let Some(o_data) = other.data() {
      self.set_data(o_data)
    } else {
      true
    }
  }

  pub fn set_dims(&mut self, w: u32, h: u32) {
    if self.w == w && self.h == h {
      return;
    }
    self.w = w;
    self.h = h;
    self.clear_data();
  }

  pub fn clear_data(&mut self) {
    if !self.data.is_null() {
      opj_image_data_free(self.data as *mut core::ffi::c_void);
      self.data = core::ptr::null_mut();
    }
  }

  pub fn alloc_data(&mut self) -> bool {
    self.clear_data();
    let data_len = (self.w as usize)
      .checked_mul(self.h as usize)
      .and_then(|len| len.checked_mul(core::mem::size_of::<OPJ_INT32>()));
    match data_len {
      None => false,
      Some(data_len) => {
        unsafe {
          self.data = opj_image_data_alloc(data_len) as *mut OPJ_INT32;
          if self.data.is_null() {
            return false;
          }
          self
            .data
            .write_bytes(0, data_len / core::mem::size_of::<OPJ_INT32>());
        }
        true
      }
    }
  }

  pub fn move_data(&mut self, other: &mut opj_image_comp) {
    self.clear_data();
    self.data = other.data;
    other.data = core::ptr::null_mut();
  }

  /// Copy just the component properties from a core component, not the data.
  pub fn copy_props_from_core(&mut self, other: &CoreImageComp) {
    self.clear_data();
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

  pub fn data(&self) -> Option<&[i32]> {
    if self.data.is_null() {
      None
    } else {
      unsafe {
        Some(core::slice::from_raw_parts(
          self.data,
          self.w as usize * self.h as usize,
        ))
      }
    }
  }

  pub fn data_mut(&mut self) -> Option<&mut [i32]> {
    if self.data.is_null() {
      None
    } else {
      unsafe {
        Some(core::slice::from_raw_parts_mut(
          self.data,
          self.w as usize * self.h as usize,
        ))
      }
    }
  }

  pub fn set_data(&mut self, data: &[i32]) -> bool {
    // If the data is null, we need to allocate it.
    if self.data.is_null() && !self.alloc_data() {
      return false;
    }
    let dest = self.data_mut().expect("We just allocated this data");
    dest.copy_from_slice(data);
    true
  }

  /// Clip component data values to the allowed range for the given precision
  pub fn clip(&mut self, precision: u32) {
    let signed = self.sgnd != 0;
    if let Some(data) = self.data_mut() {
      let (min, max) = match (precision, signed) {
        (0..=31, false) => (0, (1i64 << precision) - 1),
        (0..=31, true) => {
          let max = (1i64 << (precision - 1)) - 1;
          let min = -max - 1;
          (min, max)
        }
        _ => (0, i64::MAX),
      };

      for val in data.iter_mut() {
        *val = (*val as i64).clamp(min, max) as i32;
      }
      self.prec = precision;
    }
  }

  /// Scale component data values to the target precision
  pub fn scale(&mut self, precision: u32) {
    if self.prec == precision {
      return;
    }

    if self.prec < precision {
      self.scale_up(precision);
      return;
    }

    let sgnd = self.sgnd != 0;
    let shift = self.prec - precision;
    if let Some(data) = self.data_mut() {
      if sgnd {
        for val in data.iter_mut() {
          *val >>= shift;
        }
      } else {
        for val in data.iter_mut() {
          *val = ((*val as u32) >> shift) as i32;
        }
      }
      self.prec = precision;
    }
  }

  // Scale up component values.
  fn scale_up(&mut self, precision: u32) {
    let signed = self.sgnd != 0;
    let old_prec = self.prec;
    if let Some(data) = self.data_mut() {
      if signed {
        let new_max = 1i64 << (precision - 1);
        let old_max = 1i64 << (old_prec - 1);

        for val in data.iter_mut() {
          *val = (((*val as i64) * new_max) / old_max) as i32;
        }
      } else {
        let new_max = (1u64 << precision) - 1;
        let old_max = (1u64 << old_prec) - 1;

        for val in data.iter_mut() {
          *val = (((*val as u64) * new_max) / old_max) as i32;
        }
      }
      self.prec = precision;
    }
  }
}


#[repr(C)]
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

impl core::fmt::Debug for opj_image {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("opj_image")
      .field("x0", &self.x0)
      .field("y0", &self.y0)
      .field("x1", &self.x1)
      .field("y1", &self.y1)
      .field("numcomps", &self.numcomps)
      .field("color_space", &self.color_space)
      .field("comps", &self.comps())
      .field("icc_profile", &self.icc_profile())
      .finish()
  }
}

impl Default for opj_image {
  fn default() -> Self {
    Self {
      x0: 0,
      y0: 0,
      x1: 0,
      y1: 0,
      numcomps: 0,
      color_space: Default::default(),
      comps: core::ptr::null_mut(),
      icc_profile_buf: core::ptr::null_mut(),
      icc_profile_len: 0,
    }
  }
}

impl Clone for opj_image {
  fn clone(&self) -> Self {
    let mut image = Self {
      x0: self.x0,
      y0: self.y0,
      x1: self.x1,
      y1: self.y1,
      numcomps: self.numcomps,
      color_space: self.color_space,
      ..Default::default()
    };
    if let Some(comps) = self.comps() {
      if !image.alloc_comps(comps.len() as u32) {
        return image;
      }
      if let Some(dest) = image.comps_mut() {
        for (dest, src) in dest.iter_mut().zip(comps) {
          let _ = dest.copy(src);
        }
      }
    }
    if let Some(icc_profile) = self.icc_profile() {
      if !image.copy_icc_profile(icc_profile) {
        return image;
      }
    }
    image
  }
}

impl opj_image {
  pub fn new() -> Box<Self> {
    Box::new(Self::default())
  }

  fn create_internal(
    cmptparms: &[opj_image_comptparm],
    clrspc: OPJ_COLOR_SPACE,
    alloc_data: bool,
  ) -> Option<Box<Self>> {
    let mut image = Self::new();
    image.color_space = clrspc;
    if !image.alloc_comps(cmptparms.len() as u32) {
      return None;
    }
    if let Some(comps) = image.comps_mut() {
      for (comp, params) in comps.iter_mut().zip(cmptparms) {
        comp.dx = params.dx;
        comp.dy = params.dy;
        comp.w = params.w;
        comp.h = params.h;
        comp.x0 = params.x0;
        comp.y0 = params.y0;
        comp.prec = params.prec;
        comp.sgnd = params.sgnd;
        if alloc_data && !comp.alloc_data() {
          return None;
        }
      }
    }
    Some(image)
  }

  pub fn create(cmptparms: &[opj_image_comptparm], clrspc: OPJ_COLOR_SPACE) -> Option<Box<Self>> {
    Self::create_internal(cmptparms, clrspc, true)
  }

  pub fn tile_create(
    cmptparms: &[opj_image_comptparm],
    clrspc: OPJ_COLOR_SPACE,
  ) -> Option<Box<Self>> {
    Self::create_internal(cmptparms, clrspc, false)
  }

  pub fn comp0_dims_prec(&self) -> (usize, usize, i32) {
    if let Some(comps) = self.comps() {
      if !comps.is_empty() {
        let comp = &comps[0];
        return (comp.w as usize, comp.h as usize, comp.prec as i32);
      }
    }
    (0, 0, 0)
  }

  /// Check if all components have the same dimensions.
  pub fn comps_same_dims(&self) -> bool {
    if let Some(comps) = self.comps() {
      if let Some((c0, comps)) = comps.split_first() {
        for comp in comps {
          if comp.w != c0.w || comp.h != c0.h || comp.dx != c0.dx || comp.dy != c0.dy {
            return false;
          }
        }
        return true;
      }
    }
    false
  }

  /// Check if all components have the same dimensions and precision.
  pub fn comps_match(&self) -> bool {
    if let Some(comps) = self.comps() {
      if let Some((c0, comps)) = comps.split_first() {
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
    }
    false
  }

  pub fn take_comps(&mut self) -> Self {
    let mut image = Self {
      x0: self.x0,
      y0: self.y0,
      x1: self.x1,
      y1: self.y1,
      numcomps: self.numcomps,
      color_space: self.color_space,
      ..Default::default()
    };
    if !self.comps.is_null() {
      image.comps = self.comps;
      image.numcomps = self.numcomps;
      self.comps = core::ptr::null_mut();
      self.numcomps = 0;
    }
    image
  }

  pub fn comps(&self) -> Option<&[opj_image_comp]> {
    if self.comps.is_null() {
      None
    } else {
      unsafe {
        Some(core::slice::from_raw_parts(
          self.comps,
          self.numcomps as usize,
        ))
      }
    }
  }

  pub fn comps_data_iter(&self) -> Option<impl Iterator<Item = &[i32]>> {
    self.comps().map(|comps| comps.iter().filter_map(|comp| comp.data()))
  }

  pub fn comps_data_mut_iter(&mut self) -> Option<impl Iterator<Item = &'_ mut [i32]>> {
    self
      .comps_mut()
      .map(|comps| comps.iter_mut().filter_map(|comp| comp.data_mut()))
  }

  pub fn comps_mut(&mut self) -> Option<&mut [opj_image_comp]> {
    if self.comps.is_null() {
      None
    } else {
      unsafe {
        Some(core::slice::from_raw_parts_mut(
          self.comps,
          self.numcomps as usize,
        ))
      }
    }
  }

  pub fn set_rgb(&mut self, w: usize, h: usize, r: &[i32], g: &[i32], b: &[i32]) -> bool {
    let len = w * h;
    if r.len() != len || g.len() != len || b.len() != len {
      return false;
    }
    if let Some(comps) = self.comps_mut() {
      let w = w as u32;
      let h = h as u32;
      comps[0].set_dims(w, h);
      comps[1].set_dims(w, h);
      comps[2].set_dims(w, h);

      // Update component.
      comps[1].dx = comps[0].dx;
      comps[2].dx = comps[0].dx;
      comps[1].dy = comps[0].dy;
      comps[2].dy = comps[0].dy;

      comps[0].set_data(r);
      comps[1].set_data(g);
      comps[2].set_data(b);
    }
    self.color_space = OPJ_CLRSPC_SRGB;
    true
  }

  pub fn clear_comps(&mut self) {
    if let Some(comps) = self.comps_mut() {
      /* image components */
      for comp in comps {
        comp.clear_data();
      }
      if !self.comps.is_null() {
        // SAFETY: `comps` was built as a Vec of exactly `numcomps` components.
        unsafe {
          drop(Vec::from_raw_parts(
            self.comps,
            self.numcomps as usize,
            self.numcomps as usize,
          ))
        };
      }
      self.comps = core::ptr::null_mut();
      self.numcomps = 0;
    }
  }

  pub fn alloc_comps(&mut self, numcomps: u32) -> bool {
    self.clear_comps();
    self.numcomps = numcomps;
    {
      let mut v: Vec<opj_image_comp_t> = vec![Default::default(); numcomps as usize];
      self.comps = v.as_mut_ptr();
      core::mem::forget(v);
    }
    if self.comps.is_null() {
      self.numcomps = 0;
      return false;
    }
    true
  }

  pub fn has_icc_profile(&self) -> bool {
    !self.icc_profile_buf.is_null()
  }

  pub fn take_icc_profile(&mut self) -> Option<ICCProfile> {
    if self.icc_profile_buf.is_null() {
      None
    } else {
      unsafe {
        // A non-Null ICC Profile buffer with a length of 0 indicates that the ICC Profile is CIELab.
        let profile = if self.icc_profile_len == 0 {
          // ICC Profile is CIELab.
          Some(ICCProfile::new_cielab(core::slice::from_raw_parts(
            self.icc_profile_buf,
            CIE_LAB_BYTE_SIZE,
          )))
        } else {
          Some(ICCProfile::new_icc(core::slice::from_raw_parts(
            self.icc_profile_buf,
            self.icc_profile_len as usize,
          )))
        };
        self.clear_icc_profile();
        profile
      }
    }
  }

  pub fn icc_profile(&self) -> Option<ICCProfileRef<'_>> {
    if self.icc_profile_buf.is_null() {
      None
    } else {
      unsafe {
        // A non-Null ICC Profile buffer with a length of 0 indicates that the ICC Profile is CIELab.
        if self.icc_profile_len == 0 {
          // ICC Profile is CIELab.
          Some(ICCProfileRef::CIELab(core::slice::from_raw_parts(
            self.icc_profile_buf,
            CIE_LAB_BYTE_SIZE,
          )))
        } else {
          Some(ICCProfileRef::ICC(core::slice::from_raw_parts(
            self.icc_profile_buf,
            self.icc_profile_len as usize,
          )))
        }
      }
    }
  }

  pub fn icc_profile_mut(&self) -> Option<ICCProfileMut<'_>> {
    if self.icc_profile_buf.is_null() {
      None
    } else {
      unsafe {
        // A non-Null ICC Profile buffer with a length of 0 indicates that the ICC Profile is CIELab.
        if self.icc_profile_len == 0 {
          // ICC Profile is CIELab.
          Some(ICCProfileMut::CIELab(core::slice::from_raw_parts_mut(
            self.icc_profile_buf,
            CIE_LAB_BYTE_SIZE,
          )))
        } else {
          Some(ICCProfileMut::ICC(core::slice::from_raw_parts_mut(
            self.icc_profile_buf,
            self.icc_profile_len as usize,
          )))
        }
      }
    }
  }

  pub fn clear_icc_profile(&mut self) {
    if !self.icc_profile_buf.is_null() {
      // A non-null buffer with a length of 0 is the C ABI's CIELab marker;
      // those buffers are always CIE_LAB_BYTE_SIZE bytes long.
      let len = if self.icc_profile_len == 0 {
        CIE_LAB_BYTE_SIZE
      } else {
        self.icc_profile_len as usize
      };
      // SAFETY: `icc_profile_buf` was built as a Vec of exactly `len` bytes.
      unsafe {
        drop(Vec::from_raw_parts(self.icc_profile_buf, len, len))
      };
      self.icc_profile_buf = core::ptr::null_mut();
      self.icc_profile_len = 0;
    }
  }

  fn alloc_icc_profile(&mut self, len: usize) -> Option<&mut [u8]> {
    {
      let mut v: Vec<OPJ_BYTE> = vec![0; len];
      self.icc_profile_buf = v.as_mut_ptr();
      core::mem::forget(v);
    }
    if self.icc_profile_buf.is_null() {
      self.icc_profile_len = 0 as OPJ_UINT32;
      return None;
    }
    self.icc_profile_len = len as u32;
    Some(unsafe { core::slice::from_raw_parts_mut(self.icc_profile_buf, len) })
  }

  pub fn copy_icc_profile<'a>(&mut self, icc_profile: ICCProfileRef<'a>) -> bool {
    if icc_profile.is_empty() {
      self.clear_icc_profile();
      return true;
    }
    if let Some(dest) = self.alloc_icc_profile(icc_profile.len()) {
      match icc_profile {
        ICCProfileRef::CIELab(src) => {
          dest.copy_from_slice(src);
          self.icc_profile_len = 0;
        }
        ICCProfileRef::ICC(src) => {
          dest.copy_from_slice(src);
        }
      }
      return true;
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



pub fn opj_image_create(
  numcmpts: OPJ_UINT32,
  cmptparms: *mut opj_image_cmptparm_t,
  clrspc: OPJ_COLOR_SPACE,
) -> *mut opj_image_t {
  if numcmpts > 0 && cmptparms.is_null() {
    return core::ptr::null_mut();
  }
  let cmptparms = unsafe { core::slice::from_raw_parts(cmptparms, numcmpts as usize) };
  if let Some(image) = opj_image::create(cmptparms, clrspc) {
    Box::into_raw(image)
  } else {
    core::ptr::null_mut()
  }
}

pub fn opj_image_destroy(image: *mut opj_image_t) {
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
/* *
 * Copy only header of image and its component header (no data are copied)
 * if dest image have data, they will be freed
 *
 * @param   p_image_src     the src image
 * @param   p_image_dest    the dest image
 *
 */

/// Copy the header of an owned core image and its component headers into a
/// C-ABI image (no data copied; dest component data are dropped).
pub fn opj_image_tile_create(
  numcmpts: OPJ_UINT32,
  cmptparms: *mut opj_image_cmptparm_t,
  clrspc: OPJ_COLOR_SPACE,
) -> *mut opj_image_t {
  if numcmpts > 0 && cmptparms.is_null() {
    return core::ptr::null_mut();
  }
  let cmptparms = unsafe { core::slice::from_raw_parts(cmptparms, numcmpts as usize) };
  if let Some(image) = opj_image::tile_create(cmptparms, clrspc) {
    Box::into_raw(image)
  } else {
    core::ptr::null_mut()
  }
}

pub fn core_from_opj(image: &opj_image) -> CoreImage {
    let mut core = CoreImage {
      x0: image.x0,
      y0: image.y0,
      x1: image.x1,
      y1: image.y1,
      numcomps: image.numcomps,
      color_space: image.color_space,
      comps: Vec::with_capacity(image.numcomps as usize),
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    if let Some(comps) = image.comps() {
      for comp in comps {
        core.comps.push(CoreImageComp {
          dx: comp.dx,
          dy: comp.dy,
          w: comp.w,
          h: comp.h,
          x0: comp.x0,
          y0: comp.y0,
          prec: comp.prec,
          bpp: comp.bpp,
          sgnd: comp.sgnd,
          resno_decoded: comp.resno_decoded,
          factor: comp.factor,
          alpha: comp.alpha,
          data: comp.data().map(|data| data.to_vec()).unwrap_or_default(),
        });
      }
    }
    if !image.icc_profile_buf.is_null() {
      // A non-null buffer with a length of 0 is the C ABI's CIELab marker.
      if image.icc_profile_len == 0 {
        core.icc_profile_is_cielab = true;
        core.icc_profile_buf = unsafe {
          core::slice::from_raw_parts(image.icc_profile_buf, CIE_LAB_BYTE_SIZE).to_vec()
        };
      } else {
        core.icc_profile_buf = unsafe {
          core::slice::from_raw_parts(image.icc_profile_buf, image.icc_profile_len as usize).to_vec()
        };
      }
    }
    core
  }

  /// Allocate a fresh C-ABI image. Component data uses `opj_image_data_alloc`
  /// so C callers may free/replace it, mirroring `opj_image_create`.
pub fn core_into_opj(core: CoreImage) -> Box<opj_image> {
    let mut image = opj_image::new();
    image.x0 = core.x0;
    image.y0 = core.y0;
    image.x1 = core.x1;
    image.y1 = core.y1;
    image.numcomps = core.numcomps;
    image.color_space = core.color_space;
    if !image.alloc_comps(core.numcomps) {
      return image;
    }
    if let Some(dest_comps) = image.comps_mut() {
      for (dest, src) in dest_comps.iter_mut().zip(&core.comps) {
        dest.dx = src.dx;
        dest.dy = src.dy;
        dest.w = src.w;
        dest.h = src.h;
        dest.x0 = src.x0;
        dest.y0 = src.y0;
        dest.prec = src.prec;
        dest.bpp = src.bpp;
        dest.sgnd = src.sgnd;
        dest.resno_decoded = src.resno_decoded;
        dest.factor = src.factor;
        dest.alpha = src.alpha;
        if !src.data.is_empty() {
          // Drop any stale buffer the destination may hold (its size may not
          // match the updated w/h) and allocate fresh from the new dims.
          dest.clear_data();
          if !dest.set_data(&src.data) {
            return image;
          }
        }
      }
    }
    if core.icc_profile_is_cielab {
      image.copy_icc_profile(ICCProfileRef::CIELab(&core.icc_profile_buf));
    } else if !core.icc_profile_buf.is_empty() {
      image.copy_icc_profile(ICCProfileRef::ICC(&core.icc_profile_buf));
    }
    image
  }




#[cfg(test)]
mod image_tests {
  use super::*;

  #[test]
  fn round_trip() {
    let params = [
      opj_image_comptparm { dx: 1, dy: 1, w: 4, h: 3, x0: 0, y0: 0, prec: 8, bpp: 8, sgnd: 0 },
      opj_image_comptparm { dx: 1, dy: 1, w: 4, h: 3, x0: 0, y0: 0, prec: 8, bpp: 8, sgnd: 0 },
      opj_image_comptparm { dx: 1, dy: 1, w: 4, h: 3, x0: 0, y0: 0, prec: 8, bpp: 8, sgnd: 0 },
    ];
    let mut image = opj_image::create(&params, OPJ_CLRSPC_SRGB).unwrap();
    image.x0 = 1;
    image.y0 = 2;
    image.x1 = 5;
    image.y1 = 5;
    let core = core_from_opj(&image);
    let back = core_into_opj(core);
    assert_eq!(back.x0, 1);
    assert_eq!(back.y0, 2);
    assert_eq!(back.x1, 5);
    assert_eq!(back.y1, 5);
    assert_eq!(back.numcomps, 3);
    for comp in back.comps().unwrap() {
      assert_eq!((comp.w, comp.h), (4, 3));
    }
  }
}
