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



pub type OPJ_BOOL = i32;
pub type OPJ_FLOAT32 = f32;
pub type OPJ_BYTE = u8;
pub type OPJ_UINT16 = u16;
pub type OPJ_INT32 = i32;
pub type OPJ_UINT32 = u32;
pub type OPJ_UINT64 = u64;
pub type OPJ_OFF_T = i64;
pub type OPJ_SIZE_T = usize;
pub type opj_flag_t = u32;
pub type RSIZ_CAPABILITIES = core::ffi::c_uint;
pub const OPJ_MCT: RSIZ_CAPABILITIES = 33024;
pub const OPJ_CINEMA4K: RSIZ_CAPABILITIES = 4;
pub const OPJ_CINEMA2K: RSIZ_CAPABILITIES = 3;
pub const OPJ_STD_RSIZ: RSIZ_CAPABILITIES = 0;
pub type OPJ_RSIZ_CAPABILITIES = RSIZ_CAPABILITIES;
pub type CINEMA_MODE = core::ffi::c_uint;
pub const OPJ_CINEMA4K_24: CINEMA_MODE = 3;
pub const OPJ_CINEMA2K_48: CINEMA_MODE = 2;
pub const OPJ_CINEMA2K_24: CINEMA_MODE = 1;
pub const OPJ_OFF: CINEMA_MODE = 0;
pub type OPJ_CINEMA_MODE = CINEMA_MODE;
pub type PROG_ORDER = core::ffi::c_int;
pub const OPJ_CPRL: PROG_ORDER = 4;
pub const OPJ_PCRL: PROG_ORDER = 3;
pub const OPJ_RPCL: PROG_ORDER = 2;
pub const OPJ_RLCP: PROG_ORDER = 1;
pub const OPJ_LRCP: PROG_ORDER = 0;
pub const OPJ_PROG_UNKNOWN: PROG_ORDER = -1;
pub type OPJ_PROG_ORDER = PROG_ORDER;


/// Supported image color spaces
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum COLOR_SPACE {
  ///< not supported by the library
  OPJ_CLRSPC_UNKNOWN = -1,
  ///< not specified in the codestream
  #[default]
  OPJ_CLRSPC_UNSPECIFIED = 0,
  ///< sRGB
  OPJ_CLRSPC_SRGB = 1,
  ///< grayscale
  OPJ_CLRSPC_GRAY = 2,
  ///< YUV
  OPJ_CLRSPC_SYCC = 3,
  ///< e-YCC
  OPJ_CLRSPC_EYCC = 4,
  ///< CMYK
  OPJ_CLRSPC_CMYK = 5,
}
/// Supported image color spaces
pub use COLOR_SPACE as OPJ_COLOR_SPACE;
pub use COLOR_SPACE::*;

/// Supported codec
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum CODEC_FORMAT {
  ///< place-holder
  OPJ_CODEC_UNKNOWN = -1,
  ///< JPEG-2000 codestream : read/write
  OPJ_CODEC_J2K = 0,
  ///< JPT-stream (JPEG 2000, JPIP) : read only
  OPJ_CODEC_JPT = 1,
  ///< JP2 file format : read/write
  OPJ_CODEC_JP2 = 2,
  ///< JPP-stream (JPEG 2000, JPIP) : to be coded
  OPJ_CODEC_JPP = 3,
  ///< JPX file format (JPEG 2000 Part-2) : to be coded
  OPJ_CODEC_JPX = 4,
}
/// Supported codec
pub use CODEC_FORMAT as OPJ_CODEC_FORMAT;
pub use CODEC_FORMAT::*;

impl CODEC_FORMAT {
  pub fn from_i32(num: i32) -> Self {
    match num {
      0 => OPJ_CODEC_J2K,
      1 => OPJ_CODEC_JPT,
      2 => OPJ_CODEC_JP2,
      3 => OPJ_CODEC_JPP,
      4 => OPJ_CODEC_JPX,
      _ => OPJ_CODEC_UNKNOWN,
    }
  }
}


#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct opj_poc {
  pub resno0: u32,
  pub compno0: u32,
  pub layno1: u32,
  pub resno1: u32,
  pub compno1: u32,
  pub layno0: u32,
  pub precno0: u32,
  pub precno1: u32,
  pub prg1: OPJ_PROG_ORDER,
  pub prg: OPJ_PROG_ORDER,
  pub progorder: [core::ffi::c_char; 5],
  pub tile: u32,
  pub tx0: u32,
  pub tx1: u32,
  pub ty0: u32,
  pub ty1: u32,
  pub layS: u32,
  pub resS: u32,
  pub compS: u32,
  pub prcS: u32,
  pub layE: u32,
  pub resE: u32,
  pub compE: u32,
  pub prcE: u32,
  pub txS: u32,
  pub txE: u32,
  pub tyS: u32,
  pub tyE: u32,
  pub dx: u32,
  pub dy: u32,
  pub lay_t: u32,
  pub res_t: u32,
  pub comp_t: u32,
  pub prc_t: u32,
  pub tx0_t: u32,
  pub ty0_t: u32,
}
pub type opj_poc_t = opj_poc;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct opj_dparameters {
  pub cp_reduce: u32,
  pub cp_layer: u32,
  pub infile: [core::ffi::c_char; 4096],
  pub outfile: [core::ffi::c_char; 4096],
  pub decod_format: core::ffi::c_int,
  pub cod_format: core::ffi::c_int,
  pub DA_x0: u32,
  pub DA_x1: u32,
  pub DA_y0: u32,
  pub DA_y1: u32,
  pub m_verbose: OPJ_BOOL,
  pub tile_index: u32,
  pub nb_tile_to_decode: u32,
  pub jpwl_correct: OPJ_BOOL,
  pub jpwl_exp_comps: core::ffi::c_int,
  pub jpwl_max_tiles: core::ffi::c_int,
  pub flags: core::ffi::c_uint,
}
pub type opj_dparameters_t = opj_dparameters;

impl Default for opj_dparameters_t {
  fn default() -> Self {
    Self {
      cp_reduce: Default::default(),
      cp_layer: Default::default(),
      infile: [Default::default(); 4096],
      outfile: [Default::default(); 4096],
      decod_format: -1,
      cod_format: -1,
      DA_x0: Default::default(),
      DA_x1: Default::default(),
      DA_y0: Default::default(),
      DA_y1: Default::default(),
      m_verbose: Default::default(),
      tile_index: Default::default(),
      nb_tile_to_decode: Default::default(),
      jpwl_correct: Default::default(),
      jpwl_exp_comps: Default::default(),
      jpwl_max_tiles: Default::default(),
      flags: Default::default(),
    }
  }
}

impl opj_dparameters_t {
  pub fn set_defaults(&mut self) {
    *self = Default::default()
  }
}

pub type opj_codec_t = *mut core::ffi::c_void;
pub type opj_stream_read_fn = Option<
  unsafe extern "C" fn(
    _: *mut core::ffi::c_void,
    _: usize,
    _: *mut core::ffi::c_void,
  ) -> usize,
>;
pub type opj_stream_write_fn = Option<
  unsafe extern "C" fn(
    _: *mut core::ffi::c_void,
    _: usize,
    _: *mut core::ffi::c_void,
  ) -> usize,
>;
pub type opj_stream_skip_fn =
  Option<unsafe extern "C" fn(_: i64, _: *mut core::ffi::c_void) -> i64>;
pub type opj_stream_seek_fn =
  Option<unsafe extern "C" fn(_: i64, _: *mut core::ffi::c_void) -> OPJ_BOOL>;
pub type opj_stream_free_user_data_fn =
  Option<unsafe extern "C" fn(_: *mut core::ffi::c_void) -> ()>;
pub type opj_stream_t = *mut core::ffi::c_void;


#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct opj_packet_info {
  pub start_pos: i64,
  pub end_ph_pos: i64,
  pub end_pos: i64,
  pub disto: core::ffi::c_double,
}
pub type opj_packet_info_t = opj_packet_info;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct opj_marker_info {
  pub type_: core::ffi::c_ushort,
  pub pos: i64,
  pub len: core::ffi::c_int,
}
pub type opj_marker_info_t = opj_marker_info;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct opj_tp_index {
  pub start_pos: i64,
  pub end_header: i64,
  pub end_pos: i64,
}
pub type opj_tp_index_t = opj_tp_index;

