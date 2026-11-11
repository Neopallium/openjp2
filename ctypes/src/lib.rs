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

// The C-ABI mirror types and constants keep the C header's spellings.
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

//! C-ABI mirror types and constants for the OpenJPEG `openjpeg.h` surface.
//!
//! The `openjp2` core depends on this crate for the C-header-shaped types it
//! still uses internally (the migration to Rust primitives is incremental);
//! the `openjp2-capi` crate is the only consumer that passes these types
//! across the FFI boundary.

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

use crate::consts::opj::OPJ_PATH_LEN;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct opj_dparameters {
  pub cp_reduce: u32,
  pub cp_layer: u32,
  pub infile: [core::ffi::c_char; OPJ_PATH_LEN as usize],
  pub outfile: [core::ffi::c_char; OPJ_PATH_LEN as usize],
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
      infile: [Default::default(); OPJ_PATH_LEN as usize],
      outfile: [Default::default(); OPJ_PATH_LEN as usize],
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
  unsafe extern "C" fn(_: *mut core::ffi::c_void, _: usize, _: *mut core::ffi::c_void) -> usize,
>;
pub type opj_stream_write_fn = Option<
  unsafe extern "C" fn(_: *mut core::ffi::c_void, _: usize, _: *mut core::ffi::c_void) -> usize,
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

pub mod consts {
  /// Constants mirroring the public `openjpeg.h` defines.
  pub mod opj {

    pub const OPJ_TRUE: u32 = 1;
    pub const OPJ_FALSE: u32 = 0;
    /**< Maximum allowed size for filenames */
    pub const OPJ_PATH_LEN: u32 = 4096;
    /**< Number of maximum resolution level authorized */
    pub const OPJ_J2K_MAXRLVLS: u32 = 33;
    /**< Number of maximum sub-band linked to number of resolution level */
    pub const OPJ_J2K_MAXBANDS: u32 = 3 * OPJ_J2K_MAXRLVLS - 2;
    pub const OPJ_J2K_DEFAULT_NB_SEGS: u32 = 10;
    /** 1 mega by default */
    pub const OPJ_J2K_STREAM_CHUNK_SIZE: u32 = 0x100000;
    pub const OPJ_J2K_DEFAULT_HEADER_SIZE: u32 = 1000;
    pub const OPJ_J2K_MCC_DEFAULT_NB_RECORDS: u32 = 10;
    pub const OPJ_J2K_MCT_DEFAULT_NB_RECORDS: u32 = 10;
    /**< Maximum number of tile parts expected by JPWL: increase at your will */
    pub const JPWL_MAX_NO_TILESPECS: u32 = 16;
    /**< Maximum number of packet parts expected by JPWL: increase at your will */
    pub const JPWL_MAX_NO_PACKSPECS: u32 = 16;
    /**< Maximum number of JPWL markers: increase at your will */
    pub const JPWL_MAX_NO_MARKERS: u32 = 512;
    /**< index file name used when JPWL is on */
    pub const JPWL_PRIVATEINDEX_NAME: &str = "jpwl_index_privatefilename";
    /**< Expect this number of components, so you'll find better the first EPB */
    pub const JPWL_EXPECTED_COMPONENTS: u32 = 3;
    /**< Expect this maximum number of tiles, to avoid some crashes */
    pub const JPWL_MAXIMUM_TILES: u32 = 8192;
    /**< Expect this maximum number of bit errors in marker id's */
    pub const JPWL_MAXIMUM_HAMMING: u32 = 2;
    /**< Expect this maximum number of bytes for composition of EPBs */
    pub const JPWL_MAXIMUM_EPB_ROOM: u32 = 65450;
    /**< Basic image information provided to the user */
    pub const OPJ_IMG_INFO: u32 = 1;
    /**< Codestream information based only on the main header */
    pub const OPJ_J2K_MH_INFO: u32 = 2;
    /**< Tile information based on the current tile header */
    pub const OPJ_J2K_TH_INFO: u32 = 4;
    /**< Tile/Component information of all tiles */
    pub const OPJ_J2K_TCH_INFO: u32 = 8;
    /**< Codestream index based only on the main header */
    pub const OPJ_J2K_MH_IND: u32 = 16;
    /**< Tile index based on the current tile */
    pub const OPJ_J2K_TH_IND: u32 = 32;
    /**< JP2 file information */
    pub const OPJ_JP2_INFO: u32 = 128;
    /**< JP2 file index */
    pub const OPJ_JP2_IND: u32 = 256;
    /** no profile, conform to 15444-1 */
    pub const OPJ_PROFILE_NONE: u32 = 0x0000;
    /** Profile 0 as described in 15444-1,Table A.45 */
    pub const OPJ_PROFILE_0: u32 = 0x0001;
    /** Profile 1 as described in 15444-1,Table A.45 */
    pub const OPJ_PROFILE_1: u32 = 0x0002;
    /** At least 1 extension defined in 15444-2 (Part-2) */
    pub const OPJ_PROFILE_PART2: u32 = 0x8000;
    /** 2K cinema profile defined in 15444-1 AMD1 */
    pub const OPJ_PROFILE_CINEMA_2K: u32 = 0x0003;
    /** 4K cinema profile defined in 15444-1 AMD1 */
    pub const OPJ_PROFILE_CINEMA_4K: u32 = 0x0004;
    /** Scalable 2K cinema profile defined in 15444-1 AMD2 */
    pub const OPJ_PROFILE_CINEMA_S2K: u32 = 0x0005;
    /** Scalable 4K cinema profile defined in 15444-1 AMD2 */
    pub const OPJ_PROFILE_CINEMA_S4K: u32 = 0x0006;
    /** Long term storage cinema profile defined in 15444-1 AMD2 */
    pub const OPJ_PROFILE_CINEMA_LTS: u32 = 0x0007;
    /** Single Tile Broadcast profile defined in 15444-1 AMD3 */
    pub const OPJ_PROFILE_BC_SINGLE: u32 = 0x0100;
    /** Multi Tile Broadcast profile defined in 15444-1 AMD3 */
    pub const OPJ_PROFILE_BC_MULTI: u32 = 0x0200;
    /** Multi Tile Reversible Broadcast profile defined in 15444-1 AMD3 */
    pub const OPJ_PROFILE_BC_MULTI_R: u32 = 0x0300;
    /** 2K Single Tile Lossy IMF profile defined in 15444-1 AMD 8 */
    pub const OPJ_PROFILE_IMF_2K: u32 = 0x0400;
    /** 4K Single Tile Lossy IMF profile defined in 15444-1 AMD 8 */
    pub const OPJ_PROFILE_IMF_4K: u32 = 0x0500;
    /** 8K Single Tile Lossy IMF profile defined in 15444-1 AMD 8 */
    pub const OPJ_PROFILE_IMF_8K: u32 = 0x0600;
    /** 2K Single/Multi Tile Reversible IMF profile defined in 15444-1 AMD 8 */
    pub const OPJ_PROFILE_IMF_2K_R: u32 = 0x0700;
    /** 4K Single/Multi Tile Reversible IMF profile defined in 15444-1 AMD 8 */
    pub const OPJ_PROFILE_IMF_4K_R: u32 = 0x0800;
    /** 8K Single/Multi Tile Reversible IMF profile defined in 15444-1 AMD 8 */
    pub const OPJ_PROFILE_IMF_8K_R: u32 = 0x0900;
    /** No Part-2 extension */
    pub const OPJ_EXTENSION_NONE: u32 = 0x0000;
    /** Custom MCT support */
    pub const OPJ_EXTENSION_MCT: u32 = 0x0100;

    pub fn opj_is_cinema(v: u32) -> bool {
      (OPJ_PROFILE_CINEMA_2K..=OPJ_PROFILE_CINEMA_S4K).contains(&v)
    }

    pub fn opj_is_storage(v: u32) -> bool {
      v == OPJ_PROFILE_CINEMA_LTS
    }

    pub fn opj_is_broadcast(v: u32) -> bool {
      (OPJ_PROFILE_BC_SINGLE..=(OPJ_PROFILE_BC_MULTI_R | 0x000b)).contains(&v)
    }

    pub fn opj_is_imf(v: u32) -> bool {
      (OPJ_PROFILE_IMF_2K..=(OPJ_PROFILE_IMF_8K_R | 0x009b)).contains(&v)
    }

    pub fn opj_is_part2(v: u32) -> bool {
      v & OPJ_PROFILE_PART2 != 0
    }

    /** Extract IMF profile without mainlevel/sublevel */
    pub fn opj_get_imf_profile(v: u32) -> u32 {
      v & 0xff00
    }

    /** Extract IMF main level */
    pub fn opj_get_imf_mainlevel(v: u32) -> u32 {
      v & 0xf
    }

    /** Extract IMF sub level */
    pub fn opj_get_imf_sublevel(v: u32) -> u32 {
      (v >> 4) & 0xf
    }

    /** Maximum main level */
    pub const OPJ_IMF_MAINLEVEL_MAX: u32 = 11;
    /** MSamples/sec for IMF main level 1 */
    pub const OPJ_IMF_MAINLEVEL_1_MSAMPLESEC: u32 = 65;
    /** MSamples/sec for IMF main level 2 */
    pub const OPJ_IMF_MAINLEVEL_2_MSAMPLESEC: u32 = 130;
    /** MSamples/sec for IMF main level 3 */
    pub const OPJ_IMF_MAINLEVEL_3_MSAMPLESEC: u32 = 195;
    /** MSamples/sec for IMF main level 4 */
    pub const OPJ_IMF_MAINLEVEL_4_MSAMPLESEC: u32 = 260;
    /** MSamples/sec for IMF main level 5 */
    pub const OPJ_IMF_MAINLEVEL_5_MSAMPLESEC: u32 = 520;
    /** MSamples/sec for IMF main level 6 */
    pub const OPJ_IMF_MAINLEVEL_6_MSAMPLESEC: u32 = 1200;
    /** MSamples/sec for IMF main level 7 */
    pub const OPJ_IMF_MAINLEVEL_7_MSAMPLESEC: u32 = 2400;
    /** MSamples/sec for IMF main level 8 */
    pub const OPJ_IMF_MAINLEVEL_8_MSAMPLESEC: u32 = 4800;
    /** MSamples/sec for IMF main level 9 */
    pub const OPJ_IMF_MAINLEVEL_9_MSAMPLESEC: u32 = 9600;
    /** MSamples/sec for IMF main level 10 */
    pub const OPJ_IMF_MAINLEVEL_10_MSAMPLESEC: u32 = 19200;
    /** MSamples/sec for IMF main level 11 */
    pub const OPJ_IMF_MAINLEVEL_11_MSAMPLESEC: u32 = 38400;
    /** Mbits/s for IMF sub level 1 */
    pub const OPJ_IMF_SUBLEVEL_1_MBITSSEC: u32 = 200;
    /** Mbits/s for IMF sub level 2 */
    pub const OPJ_IMF_SUBLEVEL_2_MBITSSEC: u32 = 400;
    /** Mbits/s for IMF sub level 3 */
    pub const OPJ_IMF_SUBLEVEL_3_MBITSSEC: u32 = 800;
    /** Mbits/s for IMF sub level 4 */
    pub const OPJ_IMF_SUBLEVEL_4_MBITSSEC: u32 = 1600;
    /** Mbits/s for IMF sub level 5 */
    pub const OPJ_IMF_SUBLEVEL_5_MBITSSEC: u32 = 3200;
    /** Mbits/s for IMF sub level 6 */
    pub const OPJ_IMF_SUBLEVEL_6_MBITSSEC: u32 = 6400;
    /** Mbits/s for IMF sub level 7 */
    pub const OPJ_IMF_SUBLEVEL_7_MBITSSEC: u32 = 12800;
    /** Mbits/s for IMF sub level 8 */
    pub const OPJ_IMF_SUBLEVEL_8_MBITSSEC: u32 = 25600;
    /** Mbits/s for IMF sub level 9 */
    pub const OPJ_IMF_SUBLEVEL_9_MBITSSEC: u32 = 51200;
    /** Maximum codestream length for 24fps */
    pub const OPJ_CINEMA_24_CS: u32 = 1302083;
    /** Maximum codestream length for 48fps */
    pub const OPJ_CINEMA_48_CS: u32 = 651041;
    /** Maximum size per color component for 2K & 4K @ 24fps */
    pub const OPJ_CINEMA_24_COMP: u32 = 1041666;
    /** Maximum size per color component for 2K @ 48fps */
    pub const OPJ_CINEMA_48_COMP: u32 = 520833;
    pub const OPJ_DPARAMETERS_IGNORE_PCLR_CMAP_CDEF_FLAG: u32 = 0x0001;
    pub const OPJ_DPARAMETERS_DUMP_FLAG: u32 = 0x0002;
    pub const OPJ_STREAM_READ: u32 = OPJ_TRUE;
    pub const OPJ_STREAM_WRITE: u32 = OPJ_FALSE;
  }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct opj_image_comptparm {
  pub dx: u32,
  pub dy: u32,
  pub w: u32,
  pub h: u32,
  pub x0: u32,
  pub y0: u32,
  pub prec: u32,
  pub bpp: u32,
  pub sgnd: u32,
}
pub type opj_image_cmptparm_t = opj_image_comptparm;
