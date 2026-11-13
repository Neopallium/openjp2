//! The C-ABI encoder parameters mirror (`opj_cparameters_t`). The core uses
//! the safe owned `openjp2_core::EncoderParameters`; this module owns the
//! `#[repr(C)]` struct C callers fill in, plus the marshalling setters.

use openjp2_ctypes::*;
use openjp2_ctypes::{OPJ_BOOL, OPJ_UINT16, OPJ_UINT32};
use openjp2_ctypes::consts::opj::OPJ_PATH_LEN;

extern "C" {
  fn malloc(size: usize) -> *mut core::ffi::c_void;
  fn free(ptr: *mut core::ffi::c_void);
}

pub(crate) fn c_malloc(size: usize) -> *mut core::ffi::c_void {
  if size == 0 {
    return core::ptr::null_mut();
  }
  // SAFETY: plain C malloc.
  unsafe { malloc(size) }
}

pub(crate) fn c_free(ptr: *mut core::ffi::c_void) {
  if !ptr.is_null() {
    // SAFETY: the pointer came from c_malloc.
    unsafe { free(ptr) }
  }
}

// Note: no `Clone`/`Copy` — the `Drop` impl frees the raw pointer fields
// (`cp_matrice`, `cp_comment`, `mct_data`), so a bitwise clone would
// double-free them (the old #10 bug; the C ABI never clones this struct).
#[repr(C)]
#[derive(Debug)]
pub struct opj_cparameters {
  pub tile_size_on: OPJ_BOOL,
  pub cp_tx0: core::ffi::c_int,
  pub cp_ty0: core::ffi::c_int,
  pub cp_tdx: core::ffi::c_int,
  pub cp_tdy: core::ffi::c_int,
  pub cp_disto_alloc: core::ffi::c_int,
  pub cp_fixed_alloc: core::ffi::c_int,
  pub cp_fixed_quality: core::ffi::c_int,
  pub cp_matrice: *mut core::ffi::c_int,
  pub cp_comment: *mut core::ffi::c_char,
  pub csty: core::ffi::c_int,
  pub prog_order: OPJ_PROG_ORDER,
  pub POC: [opj_poc_t; 32],
  pub numpocs: OPJ_UINT32,
  pub tcp_numlayers: core::ffi::c_int,
  pub tcp_rates: [core::ffi::c_float; 100],
  pub tcp_distoratio: [core::ffi::c_float; 100],
  pub numresolution: core::ffi::c_int,
  pub cblockw_init: core::ffi::c_int,
  pub cblockh_init: core::ffi::c_int,
  pub mode: core::ffi::c_int,
  pub irreversible: core::ffi::c_int,
  pub roi_compno: core::ffi::c_int,
  pub roi_shift: core::ffi::c_int,
  pub res_spec: core::ffi::c_int,
  pub prcw_init: [core::ffi::c_int; 33],
  pub prch_init: [core::ffi::c_int; 33],
  pub infile: [core::ffi::c_char; OPJ_PATH_LEN as usize],
  pub outfile: [core::ffi::c_char; OPJ_PATH_LEN as usize],
  pub index_on: core::ffi::c_int,
  pub index: [core::ffi::c_char; OPJ_PATH_LEN as usize],
  pub image_offset_x0: core::ffi::c_int,
  pub image_offset_y0: core::ffi::c_int,
  pub subsampling_dx: core::ffi::c_int,
  pub subsampling_dy: core::ffi::c_int,
  pub decod_format: core::ffi::c_int,
  pub cod_format: core::ffi::c_int,
  pub jpwl_epc_on: OPJ_BOOL,
  pub jpwl_hprot_MH: core::ffi::c_int,
  pub jpwl_hprot_TPH_tileno: [core::ffi::c_int; 16],
  pub jpwl_hprot_TPH: [core::ffi::c_int; 16],
  pub jpwl_pprot_tileno: [core::ffi::c_int; 16],
  pub jpwl_pprot_packno: [core::ffi::c_int; 16],
  pub jpwl_pprot: [core::ffi::c_int; 16],
  pub jpwl_sens_size: core::ffi::c_int,
  pub jpwl_sens_addr: core::ffi::c_int,
  pub jpwl_sens_range: core::ffi::c_int,
  pub jpwl_sens_MH: core::ffi::c_int,
  pub jpwl_sens_TPH_tileno: [core::ffi::c_int; 16],
  pub jpwl_sens_TPH: [core::ffi::c_int; 16],
  pub cp_cinema: OPJ_CINEMA_MODE,
  pub max_comp_size: core::ffi::c_int,
  pub cp_rsiz: OPJ_RSIZ_CAPABILITIES,
  pub tp_on: core::ffi::c_char,
  pub tp_flag: core::ffi::c_char,
  pub tcp_mct: core::ffi::c_char,
  pub jpip_on: OPJ_BOOL,
  pub mct_data: *mut core::ffi::c_void,
  pub max_cs_size: core::ffi::c_int,
  pub rsiz: OPJ_UINT16,
}
pub type opj_cparameters_t = opj_cparameters;

impl Default for opj_cparameters_t {
  fn default() -> Self {
    Self {
      tile_size_on: Default::default(),
      cp_tx0: Default::default(),
      cp_ty0: Default::default(),
      cp_tdx: Default::default(),
      cp_tdy: Default::default(),
      cp_disto_alloc: Default::default(),
      cp_fixed_alloc: Default::default(),
      cp_fixed_quality: Default::default(),
      cp_matrice: core::ptr::null_mut(),
      cp_comment: core::ptr::null_mut(),
      csty: Default::default(),
      prog_order: OPJ_LRCP,
      POC: Default::default(),
      numpocs: Default::default(),
      tcp_numlayers: Default::default(),
      tcp_rates: [Default::default(); 100],
      tcp_distoratio: [Default::default(); 100],
      numresolution: 6i32,
      cblockw_init: 64i32,
      cblockh_init: 64i32,
      mode: Default::default(),
      irreversible: Default::default(),
      roi_compno: -1,
      roi_shift: Default::default(),
      res_spec: Default::default(),
      prcw_init: [Default::default(); 33],
      prch_init: [Default::default(); 33],
      infile: [0; OPJ_PATH_LEN as usize],
      outfile: [0; OPJ_PATH_LEN as usize],
      index_on: Default::default(),
      index: [0; OPJ_PATH_LEN as usize],
      image_offset_x0: Default::default(),
      image_offset_y0: Default::default(),
      subsampling_dx: 1,
      subsampling_dy: 1,
      decod_format: -1,
      cod_format: -1,
      jpwl_epc_on: 0,
      jpwl_hprot_MH: 0,
      jpwl_hprot_TPH_tileno: [0; 16],
      jpwl_hprot_TPH: [0; 16],
      jpwl_pprot_tileno: [0; 16],
      jpwl_pprot_packno: [0; 16],
      jpwl_pprot: [0; 16],
      jpwl_sens_size: 0,
      jpwl_sens_addr: 0,
      jpwl_sens_range: 0,
      jpwl_sens_MH: 0,
      jpwl_sens_TPH_tileno: [0; 16],
      jpwl_sens_TPH: [0; 16],
      cp_cinema: OPJ_OFF,
      max_comp_size: Default::default(),
      cp_rsiz: OPJ_STD_RSIZ,
      tp_on: Default::default(),
      tp_flag: Default::default(),
      tcp_mct: Default::default(),
      jpip_on: Default::default(),
      mct_data: core::ptr::null_mut(),
      max_cs_size: Default::default(),
      rsiz: Default::default(),
    }
  }
}


impl opj_cparameters_t {
  pub fn set_defaults(&mut self) {
    *self = Default::default()
  }

  pub fn set_comment(&mut self, comment: &str) {
    let comment = std::ffi::CString::new(comment).expect("CString::new failed");
    if !self.cp_comment.is_null() {
      // SAFETY: a previous set_comment allocated this string.
      unsafe {
        drop(std::ffi::CString::from_raw(self.cp_comment));
      }
    }
    self.cp_comment = comment.into_raw();
  }

  pub fn set_MCT(
    &mut self,
    encoding_matrix: &[f32],
    dc_shift: &[i32],
    nb_comps: u32,
  ) -> bool {
    let l_matrix_size = core::mem::size_of_val(encoding_matrix);
    let l_dc_shift_size = core::mem::size_of_val(dc_shift);
    let l_mct_total_size = l_matrix_size + l_dc_shift_size;
    /* add MCT capability */
    if self.rsiz as core::ffi::c_int & 0x8000i32 != 0 {
      self.rsiz = (self.rsiz as core::ffi::c_int | 0x100i32) as OPJ_UINT16
    } else {
      self.rsiz = (0x8000i32 | 0x100i32) as OPJ_UINT16
    }
    self.irreversible = 1i32;
    /* use array based MCT */
    self.tcp_mct = 2 as core::ffi::c_char;
    self.mct_data = c_malloc(l_mct_total_size);
    if self.mct_data.is_null() {
      return false;
    }
    // SAFETY: mct_data was allocated with exactly l_mct_total_size bytes above.
    unsafe {
      core::slice::from_raw_parts_mut(self.mct_data as *mut f32, encoding_matrix.len())
        .copy_from_slice(encoding_matrix);
      core::slice::from_raw_parts_mut(
        (self.mct_data as *mut u8).add(l_matrix_size) as *mut i32,
        nb_comps as usize,
      )
      .copy_from_slice(dc_shift);
    }
    true
  }

  pub fn get_matrix_mut(&mut self, nb_comps: usize) -> &mut [f32] {
    let l_matrix_size = nb_comps * nb_comps;
    // SAFETY: the buffer was sized for nb_comps^2 floats by set_MCT.
    unsafe { core::slice::from_raw_parts_mut(self.mct_data as *mut f32, l_matrix_size) }
  }

  pub fn get_dc_shift_mut(&mut self, nb_comps: usize) -> &mut [i32] {
    let l_matrix_byte_size = nb_comps * nb_comps * core::mem::size_of::<OPJ_FLOAT32>();
    // SAFETY: the dc shifts follow the matrix in the set_MCT buffer.
    unsafe {
      core::slice::from_raw_parts_mut(
        (self.mct_data as *mut u8).add(l_matrix_byte_size) as *mut i32,
        nb_comps,
      )
    }
  }
}

impl Drop for opj_cparameters {
  fn drop(&mut self) {
    if !self.cp_matrice.is_null() {
      c_free(self.cp_matrice as *mut core::ffi::c_void);
      self.cp_matrice = core::ptr::null_mut();
    }
    if !self.cp_comment.is_null() {
      // SAFETY: allocated by set_comment.
      unsafe {
        drop(std::ffi::CString::from_raw(self.cp_comment));
      }
      self.cp_comment = core::ptr::null_mut();
    }
    if !self.mct_data.is_null() {
      c_free(self.mct_data);
      self.mct_data = core::ptr::null_mut();
    }
  }
}