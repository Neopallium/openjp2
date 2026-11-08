/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2005, Herve Drolon, FreeImage Team
 * Copyright (c) 2008, 2011-2012, Centre National d'Etudes Spatiales (CNES), FR
 * Copyright (c) 2012, CS Systemes d'Information, France
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

use super::c_api_types::*;
use super::consts::*;
use crate::image::CoreImage;
use super::j2k::*;

use super::event::EventMgr;

/// Safe, owned encoder parameters.  The C ABI mirror (`opj_cparameters_t`)
/// is marshalled into this by the capi crate; the tools build it directly.
#[derive(Clone, Debug)]
pub struct EncoderParameters {
  pub tile_size_on: OPJ_BOOL,
  pub cp_tx0: core::ffi::c_int,
  pub cp_ty0: core::ffi::c_int,
  pub cp_tdx: core::ffi::c_int,
  pub cp_tdy: core::ffi::c_int,
  pub cp_disto_alloc: core::ffi::c_int,
  pub cp_fixed_alloc: core::ffi::c_int,
  pub cp_fixed_quality: core::ffi::c_int,
  pub cp_matrice: Option<Vec<core::ffi::c_int>>,
  pub cp_comment: Option<Vec<u8>>,
  pub csty: core::ffi::c_int,
  pub prog_order: OPJ_PROG_ORDER,
  pub pocs: [opj_poc_t; J2K_MAX_POCS as usize],
  pub numpocs: u32,
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
  pub prcw_init: [core::ffi::c_int; OPJ_J2K_MAXRLVLS as usize],
  pub prch_init: [core::ffi::c_int; OPJ_J2K_MAXRLVLS as usize],
  pub index_on: core::ffi::c_int,
  pub image_offset_x0: core::ffi::c_int,
  pub image_offset_y0: core::ffi::c_int,
  pub subsampling_dx: core::ffi::c_int,
  pub subsampling_dy: core::ffi::c_int,
  pub decod_format: core::ffi::c_int,
  pub cod_format: core::ffi::c_int,
  pub cp_cinema: OPJ_CINEMA_MODE,
  pub max_comp_size: core::ffi::c_int,
  pub cp_rsiz: OPJ_RSIZ_CAPABILITIES,
  pub tp_on: core::ffi::c_char,
  pub tp_flag: core::ffi::c_char,
  pub tcp_mct: core::ffi::c_char,
  pub jpip_on: OPJ_BOOL,
  pub mct_data: Option<(Vec<f32>, Vec<i32>)>,
  pub max_cs_size: core::ffi::c_int,
  pub rsiz: u16,
}

impl Default for EncoderParameters {
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
      cp_matrice: None,
      cp_comment: None,
      csty: Default::default(),
      prog_order: OPJ_LRCP,
      pocs: Default::default(),
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
      prcw_init: [Default::default(); OPJ_J2K_MAXRLVLS as usize],
      prch_init: [Default::default(); OPJ_J2K_MAXRLVLS as usize],
      index_on: Default::default(),
      image_offset_x0: Default::default(),
      image_offset_y0: Default::default(),
      subsampling_dx: 1,
      subsampling_dy: 1,
      decod_format: -1,
      cod_format: -1,
      cp_cinema: OPJ_OFF,
      max_comp_size: Default::default(),
      cp_rsiz: OPJ_STD_RSIZ,
      tp_on: Default::default(),
      tp_flag: Default::default(),
      tcp_mct: Default::default(),
      jpip_on: Default::default(),
      mct_data: None,
      max_cs_size: Default::default(),
      rsiz: Default::default(),
    }
  }
}

#[derive(Clone)]
pub(crate) struct DecoderParameters {
  pub cp_reduce: u32,
  pub cp_layer: u32,
  pub decod_format: core::ffi::c_int,
  pub cod_format: core::ffi::c_int,
  pub da_x0: u32,
  pub da_x1: u32,
  pub da_y0: u32,
  pub da_y1: u32,
  pub m_verbose: OPJ_BOOL,
  pub tile_index: u32,
  pub nb_tile_to_decode: u32,
  pub flags: core::ffi::c_uint,
}

impl Default for DecoderParameters {
  fn default() -> Self {
    Self {
      cp_reduce: Default::default(),
      cp_layer: Default::default(),
      decod_format: -1,
      cod_format: -1,
      da_x0: Default::default(),
      da_x1: Default::default(),
      da_y0: Default::default(),
      da_y1: Default::default(),
      m_verbose: Default::default(),
      tile_index: Default::default(),
      nb_tile_to_decode: Default::default(),
      flags: Default::default(),
    }
  }
}

pub(crate) struct TcdMarkerInfo {
  pub need_plt: OPJ_BOOL,
  pub packet_count: u32,
  pub p_packet_size: Vec<u32>,
}

impl TcdMarkerInfo {
  pub(crate) fn new(need_plt: OPJ_BOOL) -> Self {
    Self {
      need_plt,
      packet_count: 0,
      p_packet_size: Vec::new(),
    }
  }
}
pub struct Stream {
  pub(crate) m_inner: super::stream::StreamInner,
  pub(crate) m_stream_length: u64,
  pub(crate) m_byte_offset: i64,
}
pub(crate) type opj_stream_private = Stream;
pub type opj_stream_private_t = Stream;

pub(crate) type Jp2Proc =
  fn(_: &mut Jp2, _: &mut Stream, _: &mut EventMgr) -> OPJ_BOOL;
pub(crate) type Jp2ProcList = super::function_list::ProcedureList<Jp2Proc>;

pub(crate) struct Jp2 {
  pub j2k: J2k,
  pub w: u32,
  pub h: u32,
  pub bpc: u32,
  pub components: u32,
  pub unknown_color_space: u32,
  pub ipr: u32,
  pub meth: u32,
  pub approx: u32,
  pub enumcs: u32,
  pub precedence: u32,
  pub brand: u32,
  pub minversion: u32,
  pub cl: Vec<u32>,
  pub comps: Vec<Jp2Comps>,
  pub j2k_codestream_offset: i64,
  pub jpip_iptr_offset: i64,
  pub jpip_on: OPJ_BOOL,
  pub jp2_state: u32,
  pub jp2_img_state: u32,
  pub color: Jp2Color,
  pub ignore_pclr_cmap_cdef: OPJ_BOOL,
  pub has_jp2h: u8,
  pub has_ihdr: u8,
}

#[derive(Clone)]
pub(crate) struct Jp2Color {
  pub icc_profile: Option<ICCProfile>,
  pub jp2_cdef: Option<Jp2Cdef>,
  pub jp2_pclr: Option<Jp2Pclr>,
  pub jp2_has_colr: u8,
}

impl Jp2Color {
  pub fn icc_profile_len(&self) -> usize {
    match &self.icc_profile {
      Some(ICCProfile::ICC(data)) => data.len(),
      Some(ICCProfile::CIELab(data)) => data.len(),
      None => 0,
    }
  }

  pub fn icc_profile(&self) -> Option<ICCProfileRef<'_>> {
    self.icc_profile.as_ref().map(|profile| match profile {
      ICCProfile::ICC(data) => ICCProfileRef::ICC(data),
      ICCProfile::CIELab(data) => ICCProfileRef::CIELab(data),
    })
  }

  pub fn set_cielab_profile(&mut self, data: Vec<u8>) {
    self.icc_profile = Some(ICCProfile::CIELab(data));
  }

  pub fn set_icc_profile(&mut self, data: Vec<u8>) {
    self.icc_profile = Some(ICCProfile::ICC(data));
  }
}

#[derive(Clone)]
pub enum ICCProfile {
  ICC(Vec<u8>),
  CIELab(Vec<u8>),
}
pub const CIE_LAB_BYTE_SIZE: usize = 36;

impl ICCProfile {
  pub fn new_icc(data: &[u8]) -> Self {
    ICCProfile::ICC(data.to_vec())
  }

  pub fn new_cielab(data: &[u8]) -> Self {
    ICCProfile::CIELab(data.to_vec())
  }
}

#[derive(Debug)]
pub enum ICCProfileRef<'a> {
  ICC(&'a [u8]),
  CIELab(&'a [u8]),
}

impl<'a> ICCProfileRef<'a> {
  pub fn as_slice(&self) -> &[u8] {
    match self {
      ICCProfileRef::ICC(data) => data,
      ICCProfileRef::CIELab(data) => data,
    }
  }

  pub fn is_cielab(&self) -> bool {
    matches!(self, ICCProfileRef::CIELab(_))
  }

  pub fn len(&self) -> usize {
    match self {
      ICCProfileRef::ICC(data) => data.len(),
      ICCProfileRef::CIELab(data) => data.len(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

#[derive(Debug)]
pub enum ICCProfileMut<'a> {
  ICC(&'a mut [u8]),
  CIELab(&'a mut [u8]),
}

impl<'a> ICCProfileMut<'a> {
  pub fn as_mut_slice(&mut self) -> &mut [u8] {
    match self {
      ICCProfileMut::ICC(data) => data,
      ICCProfileMut::CIELab(data) => data,
    }
  }

  pub fn is_cielab(&self) -> bool {
    matches!(self, ICCProfileMut::CIELab(_))
  }
}

impl ICCProfile {
  pub fn as_slice(&self) -> &[u8] {
    match self {
      ICCProfile::ICC(data) => data,
      ICCProfile::CIELab(data) => data,
    }
  }
}

#[derive(Copy, Clone)]
pub(crate) struct Jp2ChannelSign {
  pub sign: u8,
  pub size: u8,
}

#[derive(Clone)]
pub(crate) struct Jp2Pclr {
  pub entries: Vec<u32>,
  pub channel: Vec<Jp2ChannelSign>,
  pub cmap: Vec<Jp2CmapComp>,
  pub nr_entries: u16,
  pub nr_channels: u8,
}

#[derive(Copy, Clone)]
pub(crate) struct Jp2CmapComp {
  pub cmp: u16,
  pub mtyp: u8,
  pub pcol: u8,
}

#[derive(Clone)]
pub(crate) struct Jp2Cdef {
  pub info: Vec<Jp2CdefInfo>,
}
#[derive(Copy, Clone)]
pub(crate) struct Jp2CdefInfo {
  pub cn: u16,
  pub typ: u16,
  pub asoc: u16,
}

#[derive(Copy, Clone, Default)]
pub(crate) struct Jp2Comps {
  //pub depth: u32,
  //pub sgnd: u32,
  pub bpcc: u32,
}

pub(crate) type J2kProc =
  fn(_: &mut J2k, _: &mut Stream, _: &mut EventMgr) -> OPJ_BOOL;
pub(crate) type J2kProcList = super::function_list::ProcedureList<J2kProc>;

pub(crate) struct J2k {
  pub m_is_decoder: OPJ_BOOL,
  pub m_specific_param: C2RustUnnamed_2,
  pub m_private_image: Option<Box<CoreImage>>,
  pub m_output_image: Option<Box<CoreImage>>,
  pub m_cp: Cp,
  pub cstr_index: Option<Box<CodestreamIndex>>,
  pub m_current_tile_number: u32,
  pub m_tcd: Tcd,
  pub ihdr_w: u32,
  pub ihdr_h: u32,
  pub dump_state: core::ffi::c_uint,
}

#[derive(Default)]
pub(crate) struct Tcd {
  pub tp_pos: i32,
  pub tp_num: u32,
  pub cur_tp_num: u32,
  pub cur_totnum_tp: u32,
  pub cur_pino: u32,
  pub tcd_image: TcdImage,
  pub tcd_tileno: u32,
  pub m_is_decoder: bool,
  pub win_x0: u32,
  pub win_y0: u32,
  pub win_x1: u32,
  pub win_y1: u32,
  pub whole_tile_decoding: OPJ_BOOL,
  pub used_component: Vec<bool>,
}


#[derive(Clone)]
pub(crate) struct Tcp {
  pub csty: u32,
  pub prg: OPJ_PROG_ORDER,
  pub numlayers: u32,
  pub num_layers_to_decode: u32,
  pub mct: u32,
  pub rates: [f32; 100],
  pub numpocs: u32,
  pub pocs: [opj_poc_t; J2K_MAX_POCS as usize],
  pub ppt_markers: Vec<Ppx>,
  pub ppt_buffer: Vec<u8>,
  /* read cursor into `ppt_buffer`; `ppt_len` is the number of bytes left from it */
  pub ppt_data_pos: usize,
  pub ppt_len: u32,
  pub distoratio: [f32; 100],
  pub tccps: Vec<Tccp>,
  pub m_current_tile_part_number: i32,
  pub m_nb_tile_parts: u32,
  /// Tile-part data buffer: `m_data_size` bytes of codestream data plus a 2-byte margin for the
  /// MQ decoder's synthetic end marker.
  pub m_data: Vec<u8>,
  pub m_data_size: u32,
  pub mct_norms: Vec<f64>,
  pub m_mct_decoding_matrix: Vec<f32>,
  pub m_mct_coding_matrix: Vec<f32>,
  pub m_mct_records: Vec<MctData>,
  pub m_mcc_records: Vec<SimpleMccData>,
  pub cod: bool,
  pub ppt: bool,
  pub poc: bool,
}
impl Default for Tcp {
  fn default() -> Self {
    Self {
      csty: 0,
      prg: Default::default(),
      numlayers: 0,
      num_layers_to_decode: 0,
      mct: 0,
      rates: [0.0; 100],
      numpocs: 0,
      pocs: [Default::default(); J2K_MAX_POCS as usize],
      ppt_markers: Vec::new(),
      ppt_buffer: Vec::new(),
      ppt_data_pos: 0,
      ppt_len: 0,
      distoratio: [0.0; 100],
      tccps: Vec::new(),
      m_current_tile_part_number: 0,
      m_nb_tile_parts: 0,
      m_data: Vec::new(),
      m_data_size: 0,
      mct_norms: Vec::new(),
      m_mct_decoding_matrix: Vec::new(),
      m_mct_coding_matrix: Vec::new(),
      m_mct_records: Vec::new(),
      m_mcc_records: Vec::new(),
      cod: false,
      ppt: false,
      poc: false,
    }
  }
}

#[derive(Default, Clone)]
pub(crate) struct SimpleMccData {
  pub m_index: u32,
  pub m_nb_comps: u32,
  /* index into the owning tcp's `m_mct_records` */
  pub m_decorrelation_array: Option<usize>,
  /* index into the owning tcp's `m_mct_records` */
  pub m_offset_array: Option<usize>,
  pub m_is_irreversible: bool,
}
pub type J2K_MCT_ARRAY_TYPE = MCT_ARRAY_TYPE;
pub type MCT_ARRAY_TYPE = core::ffi::c_uint;
pub const MCT_TYPE_OFFSET: MCT_ARRAY_TYPE = 2;
pub const MCT_TYPE_DECORRELATION: MCT_ARRAY_TYPE = 1;
pub const MCT_TYPE_DEPENDENCY: MCT_ARRAY_TYPE = 0;

#[derive(Default, Clone)]
pub(crate) struct MctData {
  pub m_element_type: MCTElementType,
  pub m_array_type: J2K_MCT_ARRAY_TYPE,
  pub m_index: u32,
  pub m_data: Vec<u8>,
}
#[derive(Copy, Clone)]
pub(crate) struct Tccp {
  pub csty: u32,
  pub numresolutions: u32,
  pub cblkw: u32,
  pub cblkh: u32,
  pub cblksty: u32,
  pub qmfbid: u32,
  pub qntsty: u32,
  pub stepsizes: [Stepsize; OPJ_J2K_MAXBANDS as usize],
  pub numgbits: u32,
  pub roishift: i32,
  pub prcw: [u32; OPJ_J2K_MAXRLVLS as usize],
  pub prch: [u32; OPJ_J2K_MAXRLVLS as usize],
  pub m_dc_level_shift: i32,
}
impl Default for Tccp {
  fn default() -> Self {
    Self {
      csty: 0,
      numresolutions: 0,
      cblkw: 0,
      cblkh: 0,
      cblksty: 0,
      qmfbid: 0,
      qntsty: 0,
      stepsizes: [Default::default(); OPJ_J2K_MAXBANDS as usize],
      numgbits: 0,
      roishift: 0,
      prcw: [0; OPJ_J2K_MAXRLVLS as usize],
      prch: [0; OPJ_J2K_MAXRLVLS as usize],
      m_dc_level_shift: 0,
    }
  }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct Stepsize {
  pub expn: i32,
  pub mant: i32,
}
#[derive(Clone, Default)]
pub(crate) struct Ppx {
  pub m_data: Vec<u8>,
}
#[derive(Default)]
pub(crate) struct Cp {
  pub rsiz: u16,
  pub tx0: u32,
  pub ty0: u32,
  pub tdx: u32,
  pub tdy: u32,
  pub comment: Vec<u8>,
  pub tw: u32,
  pub th: u32,
  pub ppm_markers: Vec<Ppx>,
  pub ppm_buffer: Vec<u8>,
  /* read cursor into `ppm_buffer`; `ppm_len` is the number of bytes left from it */
  pub ppm_data_pos: usize,
  pub ppm_len: u32,
  pub tcps: Vec<Tcp>,
  pub m_specific_param: C2RustUnnamed_0,
  pub strict: OPJ_BOOL,
  pub ppm: bool,
  pub m_is_decoder: bool,
  pub allow_different_bit_depth_sign: bool,
}
/* This was a C union of the decoder-only and encoder-only coding parameters. A given
`Cp` is either a decoder or an encoder one, so holding both costs a few bytes and
lets the encoder side own its data instead of aliasing raw memory. */
#[derive(Default)]
pub(crate) struct C2RustUnnamed_0 {
  pub m_dec: DecodingParams,
  pub m_enc: EncodingParams,
}

/**
Rate allocation strategy
*/
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(i32)]
pub(crate) enum J2K_QUALITY_LAYER_ALLOCATION_STRATEGY {
  /// allocation by rate/distortion
  RATE_DISTORTION_RATIO = 0,
  /// allocation by fixed distortion ratio (PSNR) (fixed quality)
  FIXED_DISTORTION_RATIO = 1,
  /// allocation by fixed layer (number of passes per layer / resolution / subband)
  FIXED_LAYER = 2,
}

pub(crate) struct EncodingParams {
  pub m_max_comp_size: u32,
  pub m_tp_pos: i32,
  pub m_matrice: Vec<i32>,
  pub m_tp_flag: u8,
  pub m_quality_layer_alloc_strategy: J2K_QUALITY_LAYER_ALLOCATION_STRATEGY,
  pub m_tp_on: bool,
}
impl Default for EncodingParams {
  fn default() -> Self {
    Self {
      m_max_comp_size: 0,
      m_tp_pos: 0,
      m_matrice: Vec::new(),
      m_tp_flag: 0,
      m_quality_layer_alloc_strategy: J2K_QUALITY_LAYER_ALLOCATION_STRATEGY::RATE_DISTORTION_RATIO,
      m_tp_on: false,
    }
  }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct DecodingParams {
  pub m_reduce: u32,
  pub m_layer: u32,
}
#[derive(Default)]
pub(crate) struct TcdImage {
  pub tiles: TcdTile,
}
pub(crate) struct TcdTile {
  pub x0: i32,
  pub y0: i32,
  pub x1: i32,
  pub y1: i32,
  pub numcomps: u32,
  pub comps: Vec<TcdTileComp>,
  pub numpix: i32,
  pub distotile: f64,
  pub distolayer: [f64; 100],
  pub packno: u32,
}
impl Default for TcdTile {
  fn default() -> Self {
    Self {
      x0: Default::default(),
      y0: Default::default(),
      x1: Default::default(),
      y1: Default::default(),
      numcomps: Default::default(),
      comps: Vec::new(),
      numpix: Default::default(),
      distotile: Default::default(),
      distolayer: [Default::default(); 100],
      packno: Default::default(),
    }
  }
}

#[derive(Clone, Default, Debug)]
pub struct TileInfo {
  pub index: u32,
  pub data_size: Option<u32>,
  pub x0: i32,
  pub y0: i32,
  pub x1: i32,
  pub y1: i32,
  pub nb_comps: u32,
  pub go_on: bool,
}

/// Sample buffer of a tile component, backed by an owned Vec.
#[derive(Default)]
pub(crate) enum TileBuf {
  #[default]
  Empty,
  Owned {
    data: Vec<i32>,
    bytes: usize,
  },
}

impl TileBuf {
  /// Mutable slice over `len` samples of the buffer.
  ///
  /// # Safety contract for the caller
  /// `len` must not exceed the number of samples the buffer covers. For
  /// `Borrowed` buffers the pointer must stay valid for `len` samples.
  pub(crate) fn as_mut_slice(&mut self, len: usize) -> &mut [i32] {
    match self {
      TileBuf::Owned { data, .. } => &mut data[..len],
      TileBuf::Empty => &mut [],
    }
  }

  pub(crate) fn as_mut_ptr(&self) -> *mut i32 {
    match self {
      TileBuf::Empty => core::ptr::null_mut(),
      TileBuf::Owned { data, .. } => {
        if data.is_empty() {
          core::ptr::null_mut()
        } else {
          data.as_ptr() as *mut i32
        }
      }
    }
  }

  pub(crate) fn is_null(&self) -> bool {
    self.as_mut_ptr().is_null()
  }

  /// Byte capacity of an owned buffer; 0 for empty or borrowed ones.
  pub(crate) fn bytes(&self) -> usize {
    match *self {
      TileBuf::Owned { bytes, .. } => bytes,
      _ => 0,
    }
  }

  pub(crate) fn release(&mut self) {
    *self = TileBuf::Empty;
  }

  /// Drops any current buffer and allocates a fresh owned one. Contents are not preserved.
  pub(crate) fn alloc(&mut self, bytes: usize) -> bool {
    self.release();
    if bytes == 0 {
      return true;
    }
    let samples = bytes / core::mem::size_of::<i32>();
    let mut data: Vec<i32> = Vec::new();
    if data.try_reserve_exact(samples).is_err() {
      return false;
    }
    data.resize(samples, 0);
    *self = TileBuf::Owned { data, bytes };
    true
  }

  /// Reallocates only when the buffer is absent or too small, like the C grow path.
  pub(crate) fn ensure(&mut self, bytes: usize) -> bool {
    if !self.is_null() && bytes <= self.bytes() {
      return true;
    }
    self.alloc(bytes)
  }

  /// Hands the owned buffer over as a `Vec` of `len` samples; the caller owns it from then on.
  /// Only valid for `Owned` buffers; returns an empty Vec for `Empty`/`Borrowed` ones.
  ///
  /// # Safety contract for the caller
  /// `len` must not exceed the number of samples the buffer covers.
  pub(crate) fn take_vec(&mut self, len: usize) -> Vec<i32> {
    match core::mem::replace(self, TileBuf::Empty) {
      TileBuf::Owned { mut data, .. } => {
        data.truncate(len);
        data
      }
      _ => Vec::new(),
    }
  }
}

pub(crate) struct TcdTileComp {
  pub x0: i32,
  pub y0: i32,
  pub x1: i32,
  pub y1: i32,
  pub compno: u32,
  pub numresolutions: u32,
  pub minimum_num_resolutions: u32,
  pub resolutions: Vec<TcdResolution>,
  pub data: TileBuf,
  pub data_size_needed: usize,
  pub data_win: TileBuf,
  pub win_x0: u32,
  pub win_y0: u32,
  pub win_x1: u32,
  pub win_y1: u32,
  pub numpix: i32,
}
impl Default for TcdTileComp {
  fn default() -> Self {
    Self {
      x0: 0,
      y0: 0,
      x1: 0,
      y1: 0,
      compno: 0,
      numresolutions: 0,
      minimum_num_resolutions: 0,
      resolutions: Vec::new(),
      data: TileBuf::Empty,
      data_size_needed: 0,
      data_win: TileBuf::Empty,
      win_x0: 0,
      win_y0: 0,
      win_x1: 0,
      win_y1: 0,
      numpix: 0,
    }
  }
}

#[derive(Default)]
pub(crate) struct TcdResolution {
  pub x0: i32,
  pub y0: i32,
  pub x1: i32,
  pub y1: i32,
  pub pw: u32,
  pub ph: u32,
  pub numbands: u32,
  pub bands: [TcdBand; 3],
  pub win_x0: u32,
  pub win_y0: u32,
  pub win_x1: u32,
  pub win_y1: u32,
}
pub(crate) struct TcdBand {
  pub x0: i32,
  pub y0: i32,
  pub x1: i32,
  pub y1: i32,
  pub bandno: u32,
  pub precincts: Vec<TcdPrecinct>,
  pub numbps: i32,
  pub stepsize: f32,
}

impl Default for TcdBand {
  fn default() -> Self {
    Self {
      x0: 0,
      y0: 0,
      x1: 0,
      y1: 0,
      bandno: 0,
      precincts: Vec::new(),
      numbps: 0,
      stepsize: 0.0,
    }
  }
}
pub(crate) struct TcdPrecinct {
  pub x0: i32,
  pub y0: i32,
  pub x1: i32,
  pub y1: i32,
  pub cw: u32,
  pub ch: u32,
  pub cblks: CodeBlocks,
  pub incltree: Option<Box<TgtTree>>,
  pub imsbtree: Option<Box<TgtTree>>,
}
impl Default for TcdPrecinct {
  fn default() -> Self {
    Self {
      x0: 0,
      y0: 0,
      x1: 0,
      y1: 0,
      cw: 0,
      ch: 0,
      cblks: CodeBlocks::Empty,
      incltree: None,
      imsbtree: None,
    }
  }
}

pub(crate) struct TgtTree {
  pub numleafsh: u32,
  pub numleafsv: u32,
  pub nodes: Vec<TgtNode>,
}
#[derive(Copy, Clone, Default)]
pub(crate) struct TgtNode {
  /// Index into `TgtTree::nodes`; `None` for the root.
  pub parent: Option<u32>,
  pub value: i32,
  pub low: i32,
  pub known: u32,
}
/// Code blocks of a precinct. The variant is fixed by `Tcd::m_is_decoder` on first use.
#[derive(Clone, Default)]
pub(crate) enum CodeBlocks {
  #[default]
  Empty,
  Enc(Vec<TcdCblkEnc>),
  Dec(Vec<TcdCblkDec>),
}

impl CodeBlocks {
  pub(crate) fn len(&self) -> usize {
    match self {
      CodeBlocks::Empty => 0,
      CodeBlocks::Enc(v) => v.len(),
      CodeBlocks::Dec(v) => v.len(),
    }
  }

  /// Grow-only, like the C realloc path: existing blocks keep the buffers they own.
  pub(crate) fn grow_enc(&mut self, len: usize) {
    if matches!(self, CodeBlocks::Empty) {
      *self = CodeBlocks::Enc(Vec::new());
    }
    match self {
      CodeBlocks::Enc(v) => {
        if v.len() < len {
          v.resize(len, Default::default());
        }
      }
      _ => unreachable!("encoder precinct used as decoder precinct"),
    }
  }

  /// Grow-only, like the C realloc path: existing blocks keep the buffers they own.
  pub(crate) fn grow_dec(&mut self, len: usize) {
    if matches!(self, CodeBlocks::Empty) {
      *self = CodeBlocks::Dec(Vec::new());
    }
    match self {
      CodeBlocks::Dec(v) => {
        if v.len() < len {
          v.resize(len, Default::default());
        }
      }
      _ => unreachable!("decoder precinct used as encoder precinct"),
    }
  }

  pub(crate) fn enc_ptr(&mut self) -> *mut TcdCblkEnc {
    match self {
      CodeBlocks::Enc(v) => v.as_mut_ptr(),
      _ => core::ptr::null_mut(),
    }
  }

  pub(crate) fn dec_ptr(&mut self) -> *mut TcdCblkDec {
    match self {
      CodeBlocks::Dec(v) => v.as_mut_ptr(),
      _ => core::ptr::null_mut(),
    }
  }

  pub(crate) fn enc_mut(&mut self, i: usize) -> &mut TcdCblkEnc {
    match self {
      CodeBlocks::Enc(v) => &mut v[i],
      _ => panic!("CodeBlocks::enc_mut on a non-Enc variant"),
    }
  }

  pub(crate) fn dec_mut(&mut self, i: usize) -> &mut TcdCblkDec {
    match self {
      CodeBlocks::Dec(v) => &mut v[i],
      _ => panic!("CodeBlocks::dec_mut on a non-Dec variant"),
    }
  }
}

#[derive(Clone)]
#[derive(Default)]
pub(crate) struct TcdCblkDec {
  pub segs: Vec<TcdSeg>,
  pub chunks: Vec<TcdSegDataChunk>,
  pub x0: i32,
  pub y0: i32,
  pub x1: i32,
  pub y1: i32,
  pub mb: u32,
  pub numbps: u32,
  pub numlenbits: u32,
  pub numnewpasses: u32,
  pub numsegs: u32,
  pub real_num_segs: u32,
  pub numchunks: u32,
  /// Whether the code block data is corrupted (upstream: partial reads mark
  /// the codeblock so t1 decoding skips it instead of reading garbage).
  pub corrupted: bool,
  pub decoded_data: Vec<core::cell::Cell<i32>>,
}
#[derive(Copy, Clone)]
pub(crate) struct TcdSegDataChunk {
  /// Offset of the chunk into the tile's codestream data buffer.
  pub data_offset: usize,
  pub len: u32,
}
#[derive(Copy, Clone, Default)]
pub(crate) struct TcdSeg {
  pub len: u32,
  pub numpasses: u32,
  pub real_num_passes: u32,
  pub maxpasses: u32,
  pub numnewpasses: u32,
  pub newlen: u32,
}
#[derive(Clone)]
#[derive(Default)]
pub(crate) struct TcdCblkEnc {
  /// Compressed-data buffer. `buf[0]` is the sentinel byte the MQ coder reads through
  /// `data[-1]`; the payload the C code called `data` starts at `buf[1]`.
  pub data: Vec<u8>,
  pub layers: Vec<TcdLayer>,
  pub passes: Vec<TcdPass>,
  pub x0: i32,
  pub y0: i32,
  pub x1: i32,
  pub y1: i32,
  pub numbps: u32,
  pub numlenbits: u32,
  pub data_size: u32,
  pub numpasses: u32,
  pub numpassesinlayers: u32,
  pub totalpasses: u32,
}
impl TcdCblkEnc {
  /// Start of the payload, i.e. what the C struct stored in its `data` member.
  /// Null while no buffer has been allocated.
  pub(crate) fn data_ptr(&self) -> *mut u8 {
    if self.data.is_empty() {
      core::ptr::null_mut()
    } else {
      // The leading byte is a sentinel; the payload starts at index 1.
      self.data.as_ptr().wrapping_add(1) as *mut u8
    }
  }

  /// Payload byte at `index`, where index 0 is the first payload byte.
  pub(crate) fn data_byte(&self, index: usize) -> u8 {
    self.data[index + 1]
  }
}


#[derive(Copy, Clone, Default)]
pub(crate) struct TcdPass {
  pub rate: u32,
  pub distortiondec: f64,
  pub len: u32,
  pub term: bool,
}
#[derive(Copy, Clone, Default)]
pub(crate) struct TcdLayer {
  pub numpasses: u32,
  pub len: u32,
  pub disto: f64,
  /// Offset of the payload into `TcdCblkEnc.data` (the Vec holds a
  /// leading sentinel byte, so a C `data` pointer equals offset 1).
  pub data_offset: usize,
}
/* This was a C union of the decoder-only and encoder-only codec state. A given `J2k` is
either a decoder or an encoder one, so holding both costs a few bytes and lets each side own
its scratch buffers instead of aliasing raw memory. */
#[derive(Default)]
pub(crate) struct C2RustUnnamed_2 {
  pub m_decoder: J2kDec,
  pub m_encoder: J2kEnc,
}

#[derive(Default)]
pub(crate) struct J2kEnc {
  pub m_current_poc_tile_part_number: u32,
  pub m_current_tile_part_number: u32,
  pub m_tlm: OPJ_BOOL,
  pub m_ttlmi_is_byte: OPJ_BOOL,
  pub m_tlm_start: i64,
  pub m_tlm_sot_offsets_buffer: Vec<u8>,
  /* write cursor into `m_tlm_sot_offsets_buffer` */
  pub m_tlm_sot_offsets_current: usize,
  pub m_total_tile_parts: u32,
  pub m_encoded_tile_data: Vec<u8>,
  pub m_header_tile_data: Vec<u8>,
  pub m_plt: OPJ_BOOL,
  pub m_reserved_bytes_for_plt: u32,
  pub m_nb_comps: u32,
}

impl J2kEnc {
  /* Grow-only scratch buffer used to assemble a marker segment before it is written out.
  Mirrors the C `opj_realloc` of `m_header_tile_data`; the returned pointer is valid until
  the next call. */
  pub(crate) fn header_tile_data(&mut self, size: u32) -> *mut u8 {
    if size as usize > self.m_header_tile_data.len() {
      self.m_header_tile_data.resize(size as usize, 0);
    }
    self.m_header_tile_data.as_mut_ptr()
  }
}
#[derive(Default)]
pub(crate) struct J2kDec {
  pub m_state: J2KState,
  pub m_default_tcp: Option<Box<Tcp>>,
  pub m_header_data: Vec<u8>,
  pub m_sot_length: u32,
  pub m_start_tile_x: u32,
  pub m_start_tile_y: u32,
  pub m_end_tile_x: u32,
  pub m_end_tile_y: u32,
  pub m_tile_ind_to_dec: i32,
  pub m_last_sot_read_pos: i64,
  pub m_last_tile_part: OPJ_BOOL,
  pub m_numcomps_to_decode: u32,
  pub m_comps_indices_to_decode: Vec<u32>,
  pub m_can_decode: bool,
  pub m_discard_tiles: bool,
  pub m_skip_data: bool,
  pub m_nb_tile_parts_correction_checked: bool,
  pub m_nb_tile_parts_correction: bool,
}

impl J2kDec {
  /* Grow-only scratch buffer holding the marker segment currently being read. Mirrors the
  C `opj_realloc` of `m_header_data`. */
  pub(crate) fn header_data(&mut self, size: u32) -> &mut [u8] {
    if size as usize > self.m_header_data.len() {
      self.m_header_data.resize(size as usize, 0);
    }
    &mut self.m_header_data[..size as usize]
  }
}
