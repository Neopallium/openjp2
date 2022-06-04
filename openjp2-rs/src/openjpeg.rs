use super::thread::*;
use super::cio::*;
use super::event::*;
use super::jp2::*;
use super::j2k::*;
use ::c2rust_bitfields;
use ::libc;

use super::malloc::*;

use ::libc::{
  FILE,
  fclose,
  fopen,
  fread,
  fwrite,
  fseeko,
  ftello,
};

extern "C" {
  fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;

  fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
}

pub type size_t = libc::c_ulong;

pub type OPJ_BOOL = i32;
pub type OPJ_CHAR = i8;
pub type OPJ_FLOAT32 = f32;
pub type OPJ_FLOAT64 = f64;
pub type OPJ_BYTE = u8;
pub type OPJ_INT8 = i8;
pub type OPJ_UINT8 = u8;
pub type OPJ_INT16 = i16;
pub type OPJ_UINT16 = u16;
pub type OPJ_INT32 = i32;
pub type OPJ_UINT32 = u32;
pub type OPJ_INT64 = i64;
pub type OPJ_UINT64 = u64;
pub type OPJ_OFF_T = i64;
pub type OPJ_SIZE_T = u64;
pub type opj_flag_t = u32;
pub type RSIZ_CAPABILITIES = libc::c_uint;
pub const OPJ_MCT: RSIZ_CAPABILITIES = 33024;
pub const OPJ_CINEMA4K: RSIZ_CAPABILITIES = 4;
pub const OPJ_CINEMA2K: RSIZ_CAPABILITIES = 3;
pub const OPJ_STD_RSIZ: RSIZ_CAPABILITIES = 0;
pub type OPJ_RSIZ_CAPABILITIES = RSIZ_CAPABILITIES;
pub type CINEMA_MODE = libc::c_uint;
pub const OPJ_CINEMA4K_24: CINEMA_MODE = 3;
pub const OPJ_CINEMA2K_48: CINEMA_MODE = 2;
pub const OPJ_CINEMA2K_24: CINEMA_MODE = 1;
pub const OPJ_OFF: CINEMA_MODE = 0;
pub type OPJ_CINEMA_MODE = CINEMA_MODE;
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
pub type CODEC_FORMAT = libc::c_int;
pub const OPJ_CODEC_JPX: CODEC_FORMAT = 4;
pub const OPJ_CODEC_JPP: CODEC_FORMAT = 3;
pub const OPJ_CODEC_JP2: CODEC_FORMAT = 2;
pub const OPJ_CODEC_JPT: CODEC_FORMAT = 1;
pub const OPJ_CODEC_J2K: CODEC_FORMAT = 0;
pub const OPJ_CODEC_UNKNOWN: CODEC_FORMAT = -1;
pub type OPJ_CODEC_FORMAT = CODEC_FORMAT;
pub type opj_msg_callback =
  Option<unsafe extern "C" fn(_: *const libc::c_char, _: *mut libc::c_void) -> ()>;

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
  pub tx0: OPJ_UINT32,
  pub tx1: OPJ_UINT32,
  pub ty0: OPJ_UINT32,
  pub ty1: OPJ_UINT32,
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
pub struct opj_cparameters {
  pub tile_size_on: OPJ_BOOL,
  pub cp_tx0: libc::c_int,
  pub cp_ty0: libc::c_int,
  pub cp_tdx: libc::c_int,
  pub cp_tdy: libc::c_int,
  pub cp_disto_alloc: libc::c_int,
  pub cp_fixed_alloc: libc::c_int,
  pub cp_fixed_quality: libc::c_int,
  pub cp_matrice: *mut libc::c_int,
  pub cp_comment: *mut libc::c_char,
  pub csty: libc::c_int,
  pub prog_order: OPJ_PROG_ORDER,
  pub POC: [opj_poc_t; 32],
  pub numpocs: OPJ_UINT32,
  pub tcp_numlayers: libc::c_int,
  pub tcp_rates: [libc::c_float; 100],
  pub tcp_distoratio: [libc::c_float; 100],
  pub numresolution: libc::c_int,
  pub cblockw_init: libc::c_int,
  pub cblockh_init: libc::c_int,
  pub mode: libc::c_int,
  pub irreversible: libc::c_int,
  pub roi_compno: libc::c_int,
  pub roi_shift: libc::c_int,
  pub res_spec: libc::c_int,
  pub prcw_init: [libc::c_int; 33],
  pub prch_init: [libc::c_int; 33],
  pub infile: [libc::c_char; 4096],
  pub outfile: [libc::c_char; 4096],
  pub index_on: libc::c_int,
  pub index: [libc::c_char; 4096],
  pub image_offset_x0: libc::c_int,
  pub image_offset_y0: libc::c_int,
  pub subsampling_dx: libc::c_int,
  pub subsampling_dy: libc::c_int,
  pub decod_format: libc::c_int,
  pub cod_format: libc::c_int,
  pub jpwl_epc_on: OPJ_BOOL,
  pub jpwl_hprot_MH: libc::c_int,
  pub jpwl_hprot_TPH_tileno: [libc::c_int; 16],
  pub jpwl_hprot_TPH: [libc::c_int; 16],
  pub jpwl_pprot_tileno: [libc::c_int; 16],
  pub jpwl_pprot_packno: [libc::c_int; 16],
  pub jpwl_pprot: [libc::c_int; 16],
  pub jpwl_sens_size: libc::c_int,
  pub jpwl_sens_addr: libc::c_int,
  pub jpwl_sens_range: libc::c_int,
  pub jpwl_sens_MH: libc::c_int,
  pub jpwl_sens_TPH_tileno: [libc::c_int; 16],
  pub jpwl_sens_TPH: [libc::c_int; 16],
  pub cp_cinema: OPJ_CINEMA_MODE,
  pub max_comp_size: libc::c_int,
  pub cp_rsiz: OPJ_RSIZ_CAPABILITIES,
  pub tp_on: libc::c_char,
  pub tp_flag: libc::c_char,
  pub tcp_mct: libc::c_char,
  pub jpip_on: OPJ_BOOL,
  pub mct_data: *mut libc::c_void,
  pub max_cs_size: libc::c_int,
  pub rsiz: OPJ_UINT16,
}
pub type opj_cparameters_t = opj_cparameters;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_dparameters {
  pub cp_reduce: OPJ_UINT32,
  pub cp_layer: OPJ_UINT32,
  pub infile: [libc::c_char; 4096],
  pub outfile: [libc::c_char; 4096],
  pub decod_format: libc::c_int,
  pub cod_format: libc::c_int,
  pub DA_x0: OPJ_UINT32,
  pub DA_x1: OPJ_UINT32,
  pub DA_y0: OPJ_UINT32,
  pub DA_y1: OPJ_UINT32,
  pub m_verbose: OPJ_BOOL,
  pub tile_index: OPJ_UINT32,
  pub nb_tile_to_decode: OPJ_UINT32,
  pub jpwl_correct: OPJ_BOOL,
  pub jpwl_exp_comps: libc::c_int,
  pub jpwl_max_tiles: libc::c_int,
  pub flags: libc::c_uint,
}
pub type opj_dparameters_t = opj_dparameters;
pub type opj_codec_t = *mut libc::c_void;
pub type opj_stream_read_fn = Option<
  unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_SIZE_T, _: *mut libc::c_void) -> OPJ_SIZE_T,
>;
pub type opj_stream_write_fn = Option<
  unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_SIZE_T, _: *mut libc::c_void) -> OPJ_SIZE_T,
>;
pub type opj_stream_skip_fn =
  Option<unsafe extern "C" fn(_: OPJ_OFF_T, _: *mut libc::c_void) -> OPJ_OFF_T>;
pub type opj_stream_seek_fn =
  Option<unsafe extern "C" fn(_: OPJ_OFF_T, _: *mut libc::c_void) -> OPJ_BOOL>;
pub type opj_stream_free_user_data_fn = Option<unsafe extern "C" fn(_: *mut libc::c_void) -> ()>;
pub type opj_stream_t = *mut libc::c_void;

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
pub struct opj_packet_info {
  pub start_pos: OPJ_OFF_T,
  pub end_ph_pos: OPJ_OFF_T,
  pub end_pos: OPJ_OFF_T,
  pub disto: libc::c_double,
}
pub type opj_packet_info_t = opj_packet_info;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_marker_info {
  pub type_0: libc::c_ushort,
  pub pos: OPJ_OFF_T,
  pub len: libc::c_int,
}
pub type opj_marker_info_t = opj_marker_info;

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
pub struct opj_tile_v2_info {
  pub tileno: libc::c_int,
  pub csty: OPJ_UINT32,
  pub prg: OPJ_PROG_ORDER,
  pub numlayers: OPJ_UINT32,
  pub mct: OPJ_UINT32,
  pub tccp_info: *mut opj_tccp_info_t,
}
pub type opj_tile_info_v2_t = opj_tile_v2_info;

#[repr(C)]
#[derive(Copy, Clone)]
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
pub struct opj_tp_info {
  pub tp_start_pos: libc::c_int,
  pub tp_end_header: libc::c_int,
  pub tp_end_pos: libc::c_int,
  pub tp_start_pack: libc::c_int,
  pub tp_numpacks: libc::c_int,
}
pub type opj_tp_info_t = opj_tp_info;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tile_info {
  pub thresh: *mut libc::c_double,
  pub tileno: libc::c_int,
  pub start_pos: libc::c_int,
  pub end_header: libc::c_int,
  pub end_pos: libc::c_int,
  pub pw: [libc::c_int; 33],
  pub ph: [libc::c_int; 33],
  pub pdx: [libc::c_int; 33],
  pub pdy: [libc::c_int; 33],
  pub packet: *mut opj_packet_info_t,
  pub numpix: libc::c_int,
  pub distotile: libc::c_double,
  pub marknum: libc::c_int,
  pub marker: *mut opj_marker_info_t,
  pub maxmarknum: libc::c_int,
  pub num_tps: libc::c_int,
  pub tp: *mut opj_tp_info_t,
}
pub type opj_tile_info_t = opj_tile_info;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_codestream_info {
  pub D_max: libc::c_double,
  pub packno: libc::c_int,
  pub index_write: libc::c_int,
  pub image_w: libc::c_int,
  pub image_h: libc::c_int,
  pub prog: OPJ_PROG_ORDER,
  pub tile_x: libc::c_int,
  pub tile_y: libc::c_int,
  pub tile_Ox: libc::c_int,
  pub tile_Oy: libc::c_int,
  pub tw: libc::c_int,
  pub th: libc::c_int,
  pub numcomps: libc::c_int,
  pub numlayers: libc::c_int,
  pub numdecompos: *mut libc::c_int,
  pub marknum: libc::c_int,
  pub marker: *mut opj_marker_info_t,
  pub maxmarknum: libc::c_int,
  pub main_head_start: libc::c_int,
  pub main_head_end: libc::c_int,
  pub codestream_size: libc::c_int,
  pub tile: *mut opj_tile_info_t,
}
pub type opj_codestream_info_t = opj_codestream_info;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tcd_marker_info {
  pub need_PLT: OPJ_BOOL,
  pub packet_count: OPJ_UINT32,
  pub p_packet_size: *mut OPJ_UINT32,
}
pub type opj_tcd_marker_info_t = opj_tcd_marker_info;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tp_index {
  pub start_pos: OPJ_OFF_T,
  pub end_header: OPJ_OFF_T,
  pub end_pos: OPJ_OFF_T,
}
pub type opj_tp_index_t = opj_tp_index;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tile_index {
  pub tileno: OPJ_UINT32,
  pub nb_tps: OPJ_UINT32,
  pub current_nb_tps: OPJ_UINT32,
  pub current_tpsno: OPJ_UINT32,
  pub tp_index: *mut opj_tp_index_t,
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
  pub codestream_size: OPJ_UINT64,
  pub marknum: OPJ_UINT32,
  pub marker: *mut opj_marker_info_t,
  pub maxmarknum: OPJ_UINT32,
  pub nb_of_tiles: OPJ_UINT32,
  pub tile_index: *mut opj_tile_index_t,
}
pub type opj_codestream_index_t = opj_codestream_index;
pub type opj_event_mgr_t = opj_event_mgr;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_event_mgr {
  pub m_error_data: *mut libc::c_void,
  pub m_warning_data: *mut libc::c_void,
  pub m_info_data: *mut libc::c_void,
  pub error_handler: opj_msg_callback,
  pub warning_handler: opj_msg_callback,
  pub info_handler: opj_msg_callback,
}
pub type opj_codec_private_t = opj_codec_private;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_codec_private {
  pub m_codec_data: C2RustUnnamed,
  pub m_codec: *mut libc::c_void,
  pub m_event_mgr: opj_event_mgr_t,
  pub is_decompressor: OPJ_BOOL,
  pub opj_dump_codec:
    Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_INT32, _: *mut FILE) -> ()>,
  pub opj_get_codec_info:
    Option<unsafe extern "C" fn(_: *mut libc::c_void) -> *mut opj_codestream_info_v2_t>,
  pub opj_get_codec_index:
    Option<unsafe extern "C" fn(_: *mut libc::c_void) -> *mut opj_codestream_index_t>,
  pub opj_set_threads:
    Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_UINT32) -> OPJ_BOOL>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed {
  pub m_decompression: opj_decompression,
  pub m_compression: opj_compression,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_compression {
  pub opj_start_compress: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: *mut opj_stream_private,
      _: *mut opj_image,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub opj_encode: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: *mut opj_stream_private,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub opj_write_tile: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: OPJ_UINT32,
      _: *mut OPJ_BYTE,
      _: OPJ_UINT32,
      _: *mut opj_stream_private,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub opj_end_compress: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: *mut opj_stream_private,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub opj_destroy: Option<unsafe extern "C" fn(_: *mut libc::c_void) -> ()>,
  pub opj_setup_encoder: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: *mut opj_cparameters_t,
      _: *mut opj_image,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub opj_encoder_set_extra_options: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: *const *const libc::c_char,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_stream_private {
  pub m_user_data: *mut libc::c_void,
  pub m_free_user_data_fn: opj_stream_free_user_data_fn,
  pub m_user_data_length: OPJ_UINT64,
  pub m_read_fn: opj_stream_read_fn,
  pub m_write_fn: opj_stream_write_fn,
  pub m_skip_fn: opj_stream_skip_fn,
  pub m_seek_fn: opj_stream_seek_fn,
  pub m_stored_data: *mut OPJ_BYTE,
  pub m_current_data: *mut OPJ_BYTE,
  pub m_opj_skip: Option<
    unsafe extern "C" fn(
      _: *mut opj_stream_private,
      _: OPJ_OFF_T,
      _: *mut opj_event_mgr,
    ) -> OPJ_OFF_T,
  >,
  pub m_opj_seek: Option<
    unsafe extern "C" fn(
      _: *mut opj_stream_private,
      _: OPJ_OFF_T,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub m_bytes_in_buffer: OPJ_SIZE_T,
  pub m_byte_offset: OPJ_OFF_T,
  pub m_buffer_size: OPJ_SIZE_T,
  pub m_status: OPJ_UINT32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_decompression {
  pub opj_read_header: Option<
    unsafe extern "C" fn(
      _: *mut opj_stream_private,
      _: *mut libc::c_void,
      _: *mut *mut opj_image_t,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub opj_decode: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: *mut opj_stream_private,
      _: *mut opj_image_t,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub opj_read_tile_header: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: *mut OPJ_UINT32,
      _: *mut OPJ_UINT32,
      _: *mut OPJ_INT32,
      _: *mut OPJ_INT32,
      _: *mut OPJ_INT32,
      _: *mut OPJ_INT32,
      _: *mut OPJ_UINT32,
      _: *mut OPJ_BOOL,
      _: *mut opj_stream_private,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub opj_decode_tile_data: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: OPJ_UINT32,
      _: *mut OPJ_BYTE,
      _: OPJ_UINT32,
      _: *mut opj_stream_private,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub opj_end_decompress: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: *mut opj_stream_private,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub opj_destroy: Option<unsafe extern "C" fn(_: *mut libc::c_void) -> ()>,
  pub opj_setup_decoder:
    Option<unsafe extern "C" fn(_: *mut libc::c_void, _: *mut opj_dparameters_t) -> ()>,
  pub opj_decoder_set_strict_mode:
    Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_BOOL) -> ()>,
  pub opj_set_decode_area: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: *mut opj_image_t,
      _: OPJ_INT32,
      _: OPJ_INT32,
      _: OPJ_INT32,
      _: OPJ_INT32,
      _: *mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub opj_get_decoded_tile: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: *mut opj_stream_private_t,
      _: *mut opj_image_t,
      _: *mut opj_event_mgr,
      _: OPJ_UINT32,
    ) -> OPJ_BOOL,
  >,
  pub opj_set_decoded_resolution_factor: Option<
    unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_UINT32, _: *mut opj_event_mgr_t) -> OPJ_BOOL,
  >,
  pub opj_set_decoded_components: Option<
    unsafe extern "C" fn(
      _: *mut libc::c_void,
      _: OPJ_UINT32,
      _: *const OPJ_UINT32,
      _: *mut opj_event_mgr_t,
    ) -> OPJ_BOOL,
  >,
}
pub type opj_stream_private_t = opj_stream_private;
pub type opj_jp2_t = opj_jp2;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_jp2 {
  pub j2k: *mut opj_j2k_t,
  pub m_validation_list: *mut opj_procedure_list,
  pub m_procedure_list: *mut opj_procedure_list,
  pub w: OPJ_UINT32,
  pub h: OPJ_UINT32,
  pub numcomps: OPJ_UINT32,
  pub bpc: OPJ_UINT32,
  pub C: OPJ_UINT32,
  pub UnkC: OPJ_UINT32,
  pub IPR: OPJ_UINT32,
  pub meth: OPJ_UINT32,
  pub approx: OPJ_UINT32,
  pub enumcs: OPJ_UINT32,
  pub precedence: OPJ_UINT32,
  pub brand: OPJ_UINT32,
  pub minversion: OPJ_UINT32,
  pub numcl: OPJ_UINT32,
  pub cl: *mut OPJ_UINT32,
  pub comps: *mut opj_jp2_comps_t,
  pub j2k_codestream_offset: OPJ_OFF_T,
  pub jpip_iptr_offset: OPJ_OFF_T,
  pub jpip_on: OPJ_BOOL,
  pub jp2_state: OPJ_UINT32,
  pub jp2_img_state: OPJ_UINT32,
  pub color: opj_jp2_color_t,
  pub ignore_pclr_cmap_cdef: OPJ_BOOL,
  pub has_jp2h: OPJ_BYTE,
  pub has_ihdr: OPJ_BYTE,
}
pub type opj_jp2_color_t = opj_jp2_color;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_jp2_color {
  pub icc_profile_buf: *mut OPJ_BYTE,
  pub icc_profile_len: OPJ_UINT32,
  pub jp2_cdef: *mut opj_jp2_cdef_t,
  pub jp2_pclr: *mut opj_jp2_pclr_t,
  pub jp2_has_colr: OPJ_BYTE,
}
pub type opj_jp2_pclr_t = opj_jp2_pclr;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_jp2_pclr {
  pub entries: *mut OPJ_UINT32,
  pub channel_sign: *mut OPJ_BYTE,
  pub channel_size: *mut OPJ_BYTE,
  pub cmap: *mut opj_jp2_cmap_comp_t,
  pub nr_entries: OPJ_UINT16,
  pub nr_channels: OPJ_BYTE,
}
pub type opj_jp2_cmap_comp_t = opj_jp2_cmap_comp;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_jp2_cmap_comp {
  pub cmp: OPJ_UINT16,
  pub mtyp: OPJ_BYTE,
  pub pcol: OPJ_BYTE,
}
pub type opj_jp2_cdef_t = opj_jp2_cdef;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_jp2_cdef {
  pub info: *mut opj_jp2_cdef_info_t,
  pub n: OPJ_UINT16,
}
pub type opj_jp2_cdef_info_t = opj_jp2_cdef_info;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_jp2_cdef_info {
  pub cn: OPJ_UINT16,
  pub typ: OPJ_UINT16,
  pub asoc: OPJ_UINT16,
}
pub type opj_jp2_comps_t = opj_jp2_comps;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_jp2_comps {
  pub depth: OPJ_UINT32,
  pub sgnd: OPJ_UINT32,
  pub bpcc: OPJ_UINT32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_procedure_list {
  pub m_nb_procedures: OPJ_UINT32,
  pub m_nb_max_procedures: OPJ_UINT32,
  pub m_procedures: *mut opj_procedure,
}
pub type opj_procedure = Option<unsafe extern "C" fn() -> ()>;
pub type opj_j2k_t = opj_j2k;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_j2k {
  pub m_is_decoder: OPJ_BOOL,
  pub m_specific_param: C2RustUnnamed_2,
  pub m_private_image: *mut opj_image_t,
  pub m_output_image: *mut opj_image_t,
  pub m_cp: opj_cp_t,
  pub m_procedure_list: *mut opj_procedure_list_t,
  pub m_validation_list: *mut opj_procedure_list_t,
  pub cstr_index: *mut opj_codestream_index_t,
  pub m_current_tile_number: OPJ_UINT32,
  pub m_tcd: *mut opj_tcd,
  pub m_tp: *mut opj_thread_pool_t,
  pub ihdr_w: OPJ_UINT32,
  pub ihdr_h: OPJ_UINT32,
  pub dump_state: libc::c_uint,
}
pub type opj_tcd_t = opj_tcd;

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub struct opj_tcd {
  pub tp_pos: OPJ_INT32,
  pub tp_num: OPJ_UINT32,
  pub cur_tp_num: OPJ_UINT32,
  pub cur_totnum_tp: OPJ_UINT32,
  pub cur_pino: OPJ_UINT32,
  pub tcd_image: *mut opj_tcd_image_t,
  pub image: *mut opj_image_t,
  pub cp: *mut opj_cp_t,
  pub tcp: *mut opj_tcp_t,
  pub tcd_tileno: OPJ_UINT32,
  #[bitfield(name = "m_is_decoder", ty = "OPJ_BITFIELD", bits = "0..=0")]
  pub m_is_decoder: [u8; 1],
  #[bitfield(padding)]
  pub c2rust_padding: [u8; 3],
  pub thread_pool: *mut opj_thread_pool_t,
  pub win_x0: OPJ_UINT32,
  pub win_y0: OPJ_UINT32,
  pub win_x1: OPJ_UINT32,
  pub win_y1: OPJ_UINT32,
  pub whole_tile_decoding: OPJ_BOOL,
  pub used_component: *mut OPJ_BOOL,
}
pub type OPJ_BITFIELD = libc::c_uint;
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
pub type opj_cp_t = opj_cp;

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
  pub m_specific_param: C2RustUnnamed_0,
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
pub union C2RustUnnamed_0 {
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
pub type opj_tcd_image_t = opj_tcd_image;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tcd_image {
  pub tiles: *mut opj_tcd_tile_t,
}
pub type opj_tcd_tile_t = opj_tcd_tile;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tcd_tile {
  pub x0: OPJ_INT32,
  pub y0: OPJ_INT32,
  pub x1: OPJ_INT32,
  pub y1: OPJ_INT32,
  pub numcomps: OPJ_UINT32,
  pub comps: *mut opj_tcd_tilecomp_t,
  pub numpix: OPJ_INT32,
  pub distotile: OPJ_FLOAT64,
  pub distolayer: [OPJ_FLOAT64; 100],
  pub packno: OPJ_UINT32,
}
pub type opj_tcd_tilecomp_t = opj_tcd_tilecomp;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tcd_tilecomp {
  pub x0: OPJ_INT32,
  pub y0: OPJ_INT32,
  pub x1: OPJ_INT32,
  pub y1: OPJ_INT32,
  pub compno: OPJ_UINT32,
  pub numresolutions: OPJ_UINT32,
  pub minimum_num_resolutions: OPJ_UINT32,
  pub resolutions: *mut opj_tcd_resolution_t,
  pub resolutions_size: OPJ_UINT32,
  pub data: *mut OPJ_INT32,
  pub ownsData: OPJ_BOOL,
  pub data_size_needed: size_t,
  pub data_size: size_t,
  pub data_win: *mut OPJ_INT32,
  pub win_x0: OPJ_UINT32,
  pub win_y0: OPJ_UINT32,
  pub win_x1: OPJ_UINT32,
  pub win_y1: OPJ_UINT32,
  pub numpix: OPJ_INT32,
}
pub type opj_tcd_resolution_t = opj_tcd_resolution;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tcd_resolution {
  pub x0: OPJ_INT32,
  pub y0: OPJ_INT32,
  pub x1: OPJ_INT32,
  pub y1: OPJ_INT32,
  pub pw: OPJ_UINT32,
  pub ph: OPJ_UINT32,
  pub numbands: OPJ_UINT32,
  pub bands: [opj_tcd_band_t; 3],
  pub win_x0: OPJ_UINT32,
  pub win_y0: OPJ_UINT32,
  pub win_x1: OPJ_UINT32,
  pub win_y1: OPJ_UINT32,
}
pub type opj_tcd_band_t = opj_tcd_band;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tcd_band {
  pub x0: OPJ_INT32,
  pub y0: OPJ_INT32,
  pub x1: OPJ_INT32,
  pub y1: OPJ_INT32,
  pub bandno: OPJ_UINT32,
  pub precincts: *mut opj_tcd_precinct_t,
  pub precincts_data_size: OPJ_UINT32,
  pub numbps: OPJ_INT32,
  pub stepsize: OPJ_FLOAT32,
}
pub type opj_tcd_precinct_t = opj_tcd_precinct;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tcd_precinct {
  pub x0: OPJ_INT32,
  pub y0: OPJ_INT32,
  pub x1: OPJ_INT32,
  pub y1: OPJ_INT32,
  pub cw: OPJ_UINT32,
  pub ch: OPJ_UINT32,
  pub cblks: C2RustUnnamed_1,
  pub block_size: OPJ_UINT32,
  pub incltree: *mut opj_tgt_tree_t,
  pub imsbtree: *mut opj_tgt_tree_t,
}
pub type opj_tgt_tree_t = opj_tgt_tree;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tgt_tree {
  pub numleafsh: OPJ_UINT32,
  pub numleafsv: OPJ_UINT32,
  pub numnodes: OPJ_UINT32,
  pub nodes: *mut opj_tgt_node_t,
  pub nodes_size: OPJ_UINT32,
}
pub type opj_tgt_node_t = opj_tgt_node;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tgt_node {
  pub parent: *mut opj_tgt_node,
  pub value: OPJ_INT32,
  pub low: OPJ_INT32,
  pub known: OPJ_UINT32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed_1 {
  pub enc: *mut opj_tcd_cblk_enc_t,
  pub dec: *mut opj_tcd_cblk_dec_t,
  pub blocks: *mut libc::c_void,
}
pub type opj_tcd_cblk_dec_t = opj_tcd_cblk_dec;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tcd_cblk_dec {
  pub segs: *mut opj_tcd_seg_t,
  pub chunks: *mut opj_tcd_seg_data_chunk_t,
  pub x0: OPJ_INT32,
  pub y0: OPJ_INT32,
  pub x1: OPJ_INT32,
  pub y1: OPJ_INT32,
  pub Mb: OPJ_UINT32,
  pub numbps: OPJ_UINT32,
  pub numlenbits: OPJ_UINT32,
  pub numnewpasses: OPJ_UINT32,
  pub numsegs: OPJ_UINT32,
  pub real_num_segs: OPJ_UINT32,
  pub m_current_max_segs: OPJ_UINT32,
  pub numchunks: OPJ_UINT32,
  pub numchunksalloc: OPJ_UINT32,
  pub decoded_data: *mut OPJ_INT32,
}
pub type opj_tcd_seg_data_chunk_t = opj_tcd_seg_data_chunk;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tcd_seg_data_chunk {
  pub data: *mut OPJ_BYTE,
  pub len: OPJ_UINT32,
}
pub type opj_tcd_seg_t = opj_tcd_seg;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tcd_seg {
  pub len: OPJ_UINT32,
  pub numpasses: OPJ_UINT32,
  pub real_num_passes: OPJ_UINT32,
  pub maxpasses: OPJ_UINT32,
  pub numnewpasses: OPJ_UINT32,
  pub newlen: OPJ_UINT32,
}
pub type opj_tcd_cblk_enc_t = opj_tcd_cblk_enc;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tcd_cblk_enc {
  pub data: *mut OPJ_BYTE,
  pub layers: *mut opj_tcd_layer_t,
  pub passes: *mut opj_tcd_pass_t,
  pub x0: OPJ_INT32,
  pub y0: OPJ_INT32,
  pub x1: OPJ_INT32,
  pub y1: OPJ_INT32,
  pub numbps: OPJ_UINT32,
  pub numlenbits: OPJ_UINT32,
  pub data_size: OPJ_UINT32,
  pub numpasses: OPJ_UINT32,
  pub numpassesinlayers: OPJ_UINT32,
  pub totalpasses: OPJ_UINT32,
}
pub type opj_tcd_pass_t = opj_tcd_pass;

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub struct opj_tcd_pass {
  pub rate: OPJ_UINT32,
  pub distortiondec: OPJ_FLOAT64,
  pub len: OPJ_UINT32,
  #[bitfield(name = "term", ty = "OPJ_BITFIELD", bits = "0..=0")]
  pub term: [u8; 1],
  #[bitfield(padding)]
  pub c2rust_padding: [u8; 3],
}
pub type opj_tcd_layer_t = opj_tcd_layer;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_tcd_layer {
  pub numpasses: OPJ_UINT32,
  pub len: OPJ_UINT32,
  pub disto: OPJ_FLOAT64,
  pub data: *mut OPJ_BYTE,
}
pub type opj_procedure_list_t = opj_procedure_list;

#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed_2 {
  pub m_decoder: opj_j2k_dec_t,
  pub m_encoder: opj_j2k_enc_t,
}
pub type opj_j2k_enc_t = opj_j2k_enc;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_j2k_enc {
  pub m_current_poc_tile_part_number: OPJ_UINT32,
  pub m_current_tile_part_number: OPJ_UINT32,
  pub m_TLM: OPJ_BOOL,
  pub m_Ttlmi_is_byte: OPJ_BOOL,
  pub m_tlm_start: OPJ_OFF_T,
  pub m_tlm_sot_offsets_buffer: *mut OPJ_BYTE,
  pub m_tlm_sot_offsets_current: *mut OPJ_BYTE,
  pub m_total_tile_parts: OPJ_UINT32,
  pub m_encoded_tile_data: *mut OPJ_BYTE,
  pub m_encoded_tile_size: OPJ_UINT32,
  pub m_header_tile_data: *mut OPJ_BYTE,
  pub m_header_tile_data_size: OPJ_UINT32,
  pub m_PLT: OPJ_BOOL,
  pub m_reserved_bytes_for_PLT: OPJ_UINT32,
  pub m_nb_comps: OPJ_UINT32,
}
pub type opj_j2k_dec_t = opj_j2k_dec;

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub struct opj_j2k_dec {
  pub m_state: OPJ_UINT32,
  pub m_default_tcp: *mut opj_tcp_t,
  pub m_header_data: *mut OPJ_BYTE,
  pub m_header_data_size: OPJ_UINT32,
  pub m_sot_length: OPJ_UINT32,
  pub m_start_tile_x: OPJ_UINT32,
  pub m_start_tile_y: OPJ_UINT32,
  pub m_end_tile_x: OPJ_UINT32,
  pub m_end_tile_y: OPJ_UINT32,
  pub m_tile_ind_to_dec: OPJ_INT32,
  pub m_last_sot_read_pos: OPJ_OFF_T,
  pub m_last_tile_part: OPJ_BOOL,
  pub m_numcomps_to_decode: OPJ_UINT32,
  pub m_comps_indices_to_decode: *mut OPJ_UINT32,
  #[bitfield(name = "m_can_decode", ty = "OPJ_BITFIELD", bits = "0..=0")]
  #[bitfield(name = "m_discard_tiles", ty = "OPJ_BITFIELD", bits = "1..=1")]
  #[bitfield(name = "m_skip_data", ty = "OPJ_BITFIELD", bits = "2..=2")]
  #[bitfield(
    name = "m_nb_tile_parts_correction_checked",
    ty = "OPJ_BITFIELD",
    bits = "3..=3"
  )]
  #[bitfield(
    name = "m_nb_tile_parts_correction",
    ty = "OPJ_BITFIELD",
    bits = "4..=4"
  )]
  pub m_can_decode_m_discard_tiles_m_skip_data_m_nb_tile_parts_correction_checked_m_nb_tile_parts_correction:
    [u8; 1],
  #[bitfield(padding)]
  pub c2rust_padding: [u8; 7],
}
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
/* _WIN32 */
/* ---------------------------------------------------------------------- */
/* Functions to set the message handlers */
#[no_mangle]
pub(crate) unsafe fn opj_set_info_handler(
  mut p_codec: *mut opj_codec_t,
  mut p_callback: opj_msg_callback,
  mut p_user_data: *mut libc::c_void,
) -> OPJ_BOOL {
  let mut l_codec = p_codec as *mut opj_codec_private_t;
  if l_codec.is_null() {
    return 0i32;
  }
  (*l_codec).m_event_mgr.info_handler = p_callback;
  (*l_codec).m_event_mgr.m_info_data = p_user_data;
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_set_warning_handler(
  mut p_codec: *mut opj_codec_t,
  mut p_callback: opj_msg_callback,
  mut p_user_data: *mut libc::c_void,
) -> OPJ_BOOL {
  let mut l_codec = p_codec as *mut opj_codec_private_t;
  if l_codec.is_null() {
    return 0i32;
  }
  (*l_codec).m_event_mgr.warning_handler = p_callback;
  (*l_codec).m_event_mgr.m_warning_data = p_user_data;
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_set_error_handler(
  mut p_codec: *mut opj_codec_t,
  mut p_callback: opj_msg_callback,
  mut p_user_data: *mut libc::c_void,
) -> OPJ_BOOL {
  let mut l_codec = p_codec as *mut opj_codec_private_t;
  if l_codec.is_null() {
    return 0i32;
  }
  (*l_codec).m_event_mgr.error_handler = p_callback;
  (*l_codec).m_event_mgr.m_error_data = p_user_data;
  return 1i32;
}
/* ---------------------------------------------------------------------- */
unsafe extern "C" fn opj_read_from_file(
  mut p_buffer: *mut libc::c_void,
  mut p_nb_bytes: OPJ_SIZE_T,
  mut p_user_data: *mut libc::c_void,
) -> OPJ_SIZE_T {
  let mut p_file = p_user_data as *mut FILE;
  let l_nb_read = fread(
    p_buffer,
    1,
    p_nb_bytes as usize,
    p_file,
  ) as u64;
  return if l_nb_read != 0 {
    l_nb_read
  } else {
    -(1i32) as OPJ_SIZE_T
  };
}
unsafe extern "C" fn opj_get_data_length_from_file(
  mut p_user_data: *mut libc::c_void,
) -> OPJ_UINT64 {
  let mut p_file = p_user_data as *mut FILE;
  let mut file_length = 0 as OPJ_OFF_T;
  fseeko(p_file, 0, 2);
  file_length = ftello(p_file);
  fseeko(p_file, 0, 0);
  return file_length as OPJ_UINT64;
}
unsafe extern "C" fn opj_write_from_file(
  mut p_buffer: *mut libc::c_void,
  mut p_nb_bytes: OPJ_SIZE_T,
  mut p_user_data: *mut libc::c_void,
) -> OPJ_SIZE_T {
  let mut p_file = p_user_data as *mut FILE;
  return fwrite(
    p_buffer,
    1,
    p_nb_bytes as usize,
    p_file,
  ) as u64;
}
unsafe extern "C" fn opj_skip_from_file(
  mut p_nb_bytes: OPJ_OFF_T,
  mut p_user_data: *mut libc::c_void,
) -> OPJ_OFF_T {
  let mut p_file = p_user_data as *mut FILE;
  if fseeko(p_file, p_nb_bytes, 1i32) != 0 {
    return -(1i32) as OPJ_OFF_T;
  }
  return p_nb_bytes;
}
unsafe extern "C" fn opj_seek_from_file(
  mut p_nb_bytes: OPJ_OFF_T,
  mut p_user_data: *mut libc::c_void,
) -> OPJ_BOOL {
  let mut p_file = p_user_data as *mut FILE;
  if fseeko(p_file, p_nb_bytes, 0i32) != 0 {
    return 0i32;
  }
  return 1i32;
}
unsafe extern "C" fn opj_close_from_file(mut p_user_data: *mut libc::c_void) {
  let mut p_file = p_user_data as *mut FILE;
  fclose(p_file);
}
/* ---------------------------------------------------------------------- */
/* _WIN32 */
/* ---------------------------------------------------------------------- */
#[no_mangle]
pub(crate) unsafe fn opj_version() -> *const libc::c_char {
  return b"2.5.0\x00" as *const u8 as *const libc::c_char;
}
/* ---------------------------------------------------------------------- */
/* DECOMPRESSION FUNCTIONS*/
#[no_mangle]
pub(crate) unsafe fn opj_create_decompress(mut p_format: OPJ_CODEC_FORMAT) -> *mut opj_codec_t {
  let mut l_codec = 0 as *mut opj_codec_private_t;
  l_codec = opj_calloc(
    1i32 as size_t,
    ::std::mem::size_of::<opj_codec_private_t>() as libc::c_ulong,
  ) as *mut opj_codec_private_t;
  if l_codec.is_null() {
    return 0 as *mut opj_codec_t;
  }
  (*l_codec).is_decompressor = 1i32;
  match p_format as libc::c_int {
    0 => {
      (*l_codec).opj_dump_codec = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_j2k_t, _: OPJ_INT32, _: *mut FILE) -> ()>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_INT32, _: *mut FILE) -> ()>,
      >(Some(
        j2k_dump as unsafe extern "C" fn(_: *mut opj_j2k_t, _: OPJ_INT32, _: *mut FILE) -> (),
      ));
      (*l_codec).opj_get_codec_info = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_j2k_t) -> *mut opj_codestream_info_v2_t>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void) -> *mut opj_codestream_info_v2_t>,
      >(Some(
        j2k_get_cstr_info
          as unsafe extern "C" fn(_: *mut opj_j2k_t) -> *mut opj_codestream_info_v2_t,
      ));
      (*l_codec).opj_get_codec_index = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_j2k_t) -> *mut opj_codestream_index_t>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void) -> *mut opj_codestream_index_t>,
      >(Some(
        j2k_get_cstr_index
          as unsafe extern "C" fn(_: *mut opj_j2k_t) -> *mut opj_codestream_index_t,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_decode = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_stream_private,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_decode
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_end_decompress = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_stream_private,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_end_decompress
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_read_header = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_stream_private_t,
            _: *mut opj_j2k_t,
            _: *mut *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut opj_stream_private,
            _: *mut libc::c_void,
            _: *mut *mut opj_image_t,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_read_header
          as unsafe extern "C" fn(
            _: *mut opj_stream_private_t,
            _: *mut opj_j2k_t,
            _: *mut *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_destroy = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_j2k_t) -> ()>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void) -> ()>,
      >(Some(
        opj_j2k_destroy as unsafe extern "C" fn(_: *mut opj_j2k_t) -> (),
      ));
      (*l_codec).m_codec_data.m_decompression.opj_setup_decoder = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_j2k_t, _: *mut opj_dparameters_t) -> ()>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void, _: *mut opj_dparameters_t) -> ()>,
      >(Some(
        opj_j2k_setup_decoder
          as unsafe extern "C" fn(_: *mut opj_j2k_t, _: *mut opj_dparameters_t) -> (),
      ));
      (*l_codec)
        .m_codec_data
        .m_decompression
        .opj_decoder_set_strict_mode = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_j2k_t, _: OPJ_BOOL) -> ()>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_BOOL) -> ()>,
      >(Some(
        opj_j2k_decoder_set_strict_mode
          as unsafe extern "C" fn(_: *mut opj_j2k_t, _: OPJ_BOOL) -> (),
      ));
      (*l_codec).m_codec_data.m_decompression.opj_read_tile_header = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_BOOL,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_BOOL,
            _: *mut opj_stream_private,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_read_tile_header
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_BOOL,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_decode_tile_data = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: OPJ_UINT32,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: OPJ_UINT32,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_stream_private,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_decode_tile
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: OPJ_UINT32,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_set_decode_area = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_image_t,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_image_t,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_set_decode_area
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_image_t,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_get_decoded_tile = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
            _: OPJ_UINT32,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr,
            _: OPJ_UINT32,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_get_tile
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
            _: OPJ_UINT32,
          ) -> OPJ_BOOL,
      ));
      (*l_codec)
        .m_codec_data
        .m_decompression
        .opj_set_decoded_resolution_factor = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_set_decoded_resolution_factor
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec)
        .m_codec_data
        .m_decompression
        .opj_set_decoded_components = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: OPJ_UINT32,
            _: *const OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: OPJ_UINT32,
            _: *const OPJ_UINT32,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_set_decoded_components
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: OPJ_UINT32,
            _: *const OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).opj_set_threads = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_j2k_t, _: OPJ_UINT32) -> OPJ_BOOL>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_UINT32) -> OPJ_BOOL>,
      >(Some(
        opj_j2k_set_threads as unsafe extern "C" fn(_: *mut opj_j2k_t, _: OPJ_UINT32) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec = opj_j2k_create_decompress() as *mut libc::c_void;
      if (*l_codec).m_codec.is_null() {
        opj_free(l_codec as *mut libc::c_void);
        return 0 as *mut opj_codec_t;
      }
    }
    2 => {
      /* get a JP2 decoder handle */
      (*l_codec).opj_dump_codec = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_jp2_t, _: OPJ_INT32, _: *mut FILE) -> ()>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_INT32, _: *mut FILE) -> ()>,
      >(Some(
        jp2_dump as unsafe extern "C" fn(_: *mut opj_jp2_t, _: OPJ_INT32, _: *mut FILE) -> (),
      ));
      (*l_codec).opj_get_codec_info = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_jp2_t) -> *mut opj_codestream_info_v2_t>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void) -> *mut opj_codestream_info_v2_t>,
      >(Some(
        jp2_get_cstr_info
          as unsafe extern "C" fn(_: *mut opj_jp2_t) -> *mut opj_codestream_info_v2_t,
      ));
      (*l_codec).opj_get_codec_index = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_jp2_t) -> *mut opj_codestream_index_t>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void) -> *mut opj_codestream_index_t>,
      >(Some(
        jp2_get_cstr_index
          as unsafe extern "C" fn(_: *mut opj_jp2_t) -> *mut opj_codestream_index_t,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_decode = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_stream_private,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_decode
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_end_decompress = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_stream_private,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_end_decompress
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_read_header = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_stream_private_t,
            _: *mut opj_jp2_t,
            _: *mut *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut opj_stream_private,
            _: *mut libc::c_void,
            _: *mut *mut opj_image_t,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_read_header
          as unsafe extern "C" fn(
            _: *mut opj_stream_private_t,
            _: *mut opj_jp2_t,
            _: *mut *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_read_tile_header = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_BOOL,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_BOOL,
            _: *mut opj_stream_private,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_read_tile_header
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_INT32,
            _: *mut OPJ_UINT32,
            _: *mut OPJ_BOOL,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_decode_tile_data = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: OPJ_UINT32,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: OPJ_UINT32,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_stream_private,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_decode_tile
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: OPJ_UINT32,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_destroy = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_jp2_t) -> ()>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void) -> ()>,
      >(Some(
        opj_jp2_destroy as unsafe extern "C" fn(_: *mut opj_jp2_t) -> (),
      ));
      (*l_codec).m_codec_data.m_decompression.opj_setup_decoder = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_jp2_t, _: *mut opj_dparameters_t) -> ()>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void, _: *mut opj_dparameters_t) -> ()>,
      >(Some(
        opj_jp2_setup_decoder
          as unsafe extern "C" fn(_: *mut opj_jp2_t, _: *mut opj_dparameters_t) -> (),
      ));
      (*l_codec)
        .m_codec_data
        .m_decompression
        .opj_decoder_set_strict_mode = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_jp2_t, _: OPJ_BOOL) -> ()>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_BOOL) -> ()>,
      >(Some(
        opj_jp2_decoder_set_strict_mode
          as unsafe extern "C" fn(_: *mut opj_jp2_t, _: OPJ_BOOL) -> (),
      ));
      (*l_codec).m_codec_data.m_decompression.opj_set_decode_area = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_image_t,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_image_t,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_set_decode_area
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_image_t,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: OPJ_INT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_decompression.opj_get_decoded_tile = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
            _: OPJ_UINT32,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr,
            _: OPJ_UINT32,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_get_tile
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
            _: OPJ_UINT32,
          ) -> OPJ_BOOL,
      ));
      (*l_codec)
        .m_codec_data
        .m_decompression
        .opj_set_decoded_resolution_factor = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_set_decoded_resolution_factor
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec)
        .m_codec_data
        .m_decompression
        .opj_set_decoded_components = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: OPJ_UINT32,
            _: *const OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: OPJ_UINT32,
            _: *const OPJ_UINT32,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_set_decoded_components
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: OPJ_UINT32,
            _: *const OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).opj_set_threads = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_jp2_t, _: OPJ_UINT32) -> OPJ_BOOL>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_UINT32) -> OPJ_BOOL>,
      >(Some(
        opj_jp2_set_threads as unsafe extern "C" fn(_: *mut opj_jp2_t, _: OPJ_UINT32) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec = opj_jp2_create(1i32) as *mut libc::c_void;
      if (*l_codec).m_codec.is_null() {
        opj_free(l_codec as *mut libc::c_void);
        return 0 as *mut opj_codec_t;
      }
    }
    -1 | 1 | _ => {
      opj_free(l_codec as *mut libc::c_void);
      return 0 as *mut opj_codec_t;
    }
  }
  opj_set_default_event_handler(&mut (*l_codec).m_event_mgr);
  return l_codec as *mut opj_codec_t;
}
#[no_mangle]
pub(crate) unsafe fn opj_set_default_decoder_parameters(
  mut parameters: *mut opj_dparameters_t,
) {
  if !parameters.is_null() {
    memset(
      parameters as *mut libc::c_void,
      0i32,
      ::std::mem::size_of::<opj_dparameters_t>() as libc::c_ulong,
    );
    /* UniPG>> */
    /* USE_JPWL */
    /* <<UniPG */
    (*parameters).cp_layer = 0 as OPJ_UINT32;
    (*parameters).cp_reduce = 0 as OPJ_UINT32;
    (*parameters).decod_format = -(1i32);
    (*parameters).cod_format = -(1i32);
    (*parameters).flags = 0u32
  };
}
#[no_mangle]
pub(crate) unsafe fn opj_codec_set_threads(
  mut p_codec: *mut opj_codec_t,
  mut num_threads: libc::c_int,
) -> OPJ_BOOL {
  if !p_codec.is_null() && num_threads >= 0i32 {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    return (*l_codec)
      .opj_set_threads
      .expect("non-null function pointer")(
      (*l_codec).m_codec, num_threads as OPJ_UINT32
    );
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_setup_decoder(
  mut p_codec: *mut opj_codec_t,
  mut parameters: *mut opj_dparameters_t,
) -> OPJ_BOOL {
  if !p_codec.is_null() && !parameters.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    if (*l_codec).is_decompressor == 0 {
      opj_event_msg(
        &mut (*l_codec).m_event_mgr as *mut opj_event_mgr_t,
        1i32,
        b"Codec provided to the opj_setup_decoder function is not a decompressor handler.\n\x00"
          as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*l_codec)
      .m_codec_data
      .m_decompression
      .opj_setup_decoder
      .expect("non-null function pointer")((*l_codec).m_codec, parameters);
    return 1i32;
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_decoder_set_strict_mode(
  mut p_codec: *mut opj_codec_t,
  mut strict: OPJ_BOOL,
) -> OPJ_BOOL {
  if !p_codec.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    if (*l_codec).is_decompressor == 0 {
      opj_event_msg(&mut (*l_codec).m_event_mgr as *mut opj_event_mgr_t,
                          1i32,
                          b"Codec provided to the opj_decoder_set_strict_mode function is not a decompressor handler.\n\x00"
                              as *const u8 as *const libc::c_char);
      return 0i32;
    }
    (*l_codec)
      .m_codec_data
      .m_decompression
      .opj_decoder_set_strict_mode
      .expect("non-null function pointer")((*l_codec).m_codec, strict);
    return 1i32;
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_read_header(
  mut p_stream: *mut opj_stream_t,
  mut p_codec: *mut opj_codec_t,
  mut p_image: *mut *mut opj_image_t,
) -> OPJ_BOOL {
  if !p_codec.is_null() && !p_stream.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    let mut l_stream = p_stream as *mut opj_stream_private_t;
    if (*l_codec).is_decompressor == 0 {
      opj_event_msg(
        &mut (*l_codec).m_event_mgr as *mut opj_event_mgr_t,
        1i32,
        b"Codec provided to the opj_read_header function is not a decompressor handler.\n\x00"
          as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    return (*l_codec)
      .m_codec_data
      .m_decompression
      .opj_read_header
      .expect("non-null function pointer")(
      l_stream,
      (*l_codec).m_codec,
      p_image,
      &mut (*l_codec).m_event_mgr,
    );
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_set_decoded_components(
  mut p_codec: *mut opj_codec_t,
  mut numcomps: OPJ_UINT32,
  mut comps_indices: *const OPJ_UINT32,
  mut apply_color_transforms: OPJ_BOOL,
) -> OPJ_BOOL {
  if !p_codec.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    if (*l_codec).is_decompressor == 0 {
      opj_event_msg(&mut (*l_codec).m_event_mgr as *mut opj_event_mgr_t,
                          1i32,
                          b"Codec provided to the opj_set_decoded_components function is not a decompressor handler.\n\x00"
                              as *const u8 as *const libc::c_char);
      return 0i32;
    }
    if apply_color_transforms != 0 {
      opj_event_msg(
        &mut (*l_codec).m_event_mgr as *mut opj_event_mgr_t,
        1i32,
        b"apply_color_transforms = OPJ_TRUE is not supported.\n\x00" as *const u8
          as *const libc::c_char,
      );
      return 0i32;
    }
    return (*l_codec)
      .m_codec_data
      .m_decompression
      .opj_set_decoded_components
      .expect("non-null function pointer")(
      (*l_codec).m_codec,
      numcomps,
      comps_indices,
      &mut (*l_codec).m_event_mgr,
    );
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_decode(
  mut p_codec: *mut opj_codec_t,
  mut p_stream: *mut opj_stream_t,
  mut p_image: *mut opj_image_t,
) -> OPJ_BOOL {
  if !p_codec.is_null() && !p_stream.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    let mut l_stream = p_stream as *mut opj_stream_private_t;
    if (*l_codec).is_decompressor == 0 {
      return 0i32;
    }
    return (*l_codec)
      .m_codec_data
      .m_decompression
      .opj_decode
      .expect("non-null function pointer")(
      (*l_codec).m_codec,
      l_stream,
      p_image,
      &mut (*l_codec).m_event_mgr,
    );
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_set_decode_area(
  mut p_codec: *mut opj_codec_t,
  mut p_image: *mut opj_image_t,
  mut p_start_x: OPJ_INT32,
  mut p_start_y: OPJ_INT32,
  mut p_end_x: OPJ_INT32,
  mut p_end_y: OPJ_INT32,
) -> OPJ_BOOL {
  if !p_codec.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    if (*l_codec).is_decompressor == 0 {
      return 0i32;
    }
    return (*l_codec)
      .m_codec_data
      .m_decompression
      .opj_set_decode_area
      .expect("non-null function pointer")(
      (*l_codec).m_codec,
      p_image,
      p_start_x,
      p_start_y,
      p_end_x,
      p_end_y,
      &mut (*l_codec).m_event_mgr,
    );
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_read_tile_header(
  mut p_codec: *mut opj_codec_t,
  mut p_stream: *mut opj_stream_t,
  mut p_tile_index: *mut OPJ_UINT32,
  mut p_data_size: *mut OPJ_UINT32,
  mut p_tile_x0: *mut OPJ_INT32,
  mut p_tile_y0: *mut OPJ_INT32,
  mut p_tile_x1: *mut OPJ_INT32,
  mut p_tile_y1: *mut OPJ_INT32,
  mut p_nb_comps: *mut OPJ_UINT32,
  mut p_should_go_on: *mut OPJ_BOOL,
) -> OPJ_BOOL {
  if !p_codec.is_null() && !p_stream.is_null() && !p_data_size.is_null() && !p_tile_index.is_null()
  {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    let mut l_stream = p_stream as *mut opj_stream_private_t;
    if (*l_codec).is_decompressor == 0 {
      return 0i32;
    }
    return (*l_codec)
      .m_codec_data
      .m_decompression
      .opj_read_tile_header
      .expect("non-null function pointer")(
      (*l_codec).m_codec,
      p_tile_index,
      p_data_size,
      p_tile_x0,
      p_tile_y0,
      p_tile_x1,
      p_tile_y1,
      p_nb_comps,
      p_should_go_on,
      l_stream,
      &mut (*l_codec).m_event_mgr,
    );
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_decode_tile_data(
  mut p_codec: *mut opj_codec_t,
  mut p_tile_index: OPJ_UINT32,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_size: OPJ_UINT32,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if !p_codec.is_null() && !p_data.is_null() && !p_stream.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    let mut l_stream = p_stream as *mut opj_stream_private_t;
    if (*l_codec).is_decompressor == 0 {
      return 0i32;
    }
    return (*l_codec)
      .m_codec_data
      .m_decompression
      .opj_decode_tile_data
      .expect("non-null function pointer")(
      (*l_codec).m_codec,
      p_tile_index,
      p_data,
      p_data_size,
      l_stream,
      &mut (*l_codec).m_event_mgr,
    );
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_get_decoded_tile(
  mut p_codec: *mut opj_codec_t,
  mut p_stream: *mut opj_stream_t,
  mut p_image: *mut opj_image_t,
  mut tile_index: OPJ_UINT32,
) -> OPJ_BOOL {
  if !p_codec.is_null() && !p_stream.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    let mut l_stream = p_stream as *mut opj_stream_private_t;
    if (*l_codec).is_decompressor == 0 {
      return 0i32;
    }
    return (*l_codec)
      .m_codec_data
      .m_decompression
      .opj_get_decoded_tile
      .expect("non-null function pointer")(
      (*l_codec).m_codec,
      l_stream,
      p_image,
      &mut (*l_codec).m_event_mgr,
      tile_index,
    );
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_set_decoded_resolution_factor(
  mut p_codec: *mut opj_codec_t,
  mut res_factor: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut l_codec = p_codec as *mut opj_codec_private_t;
  if l_codec.is_null() {
    return 0i32;
  }
  return (*l_codec)
    .m_codec_data
    .m_decompression
    .opj_set_decoded_resolution_factor
    .expect("non-null function pointer")(
    (*l_codec).m_codec,
    res_factor,
    &mut (*l_codec).m_event_mgr,
  );
}
/* default decoding parameters */
/* ---------------------------------------------------------------------- */
/* COMPRESSION FUNCTIONS*/
#[no_mangle]
pub(crate) unsafe fn opj_create_compress(mut p_format: OPJ_CODEC_FORMAT) -> *mut opj_codec_t {
  let mut l_codec = 0 as *mut opj_codec_private_t;
  l_codec = opj_calloc(
    1i32 as size_t,
    ::std::mem::size_of::<opj_codec_private_t>() as libc::c_ulong,
  ) as *mut opj_codec_private_t;
  if l_codec.is_null() {
    return 0 as *mut opj_codec_t;
  }
  (*l_codec).is_decompressor = 0i32;
  match p_format as libc::c_int {
    0 => {
      (*l_codec).m_codec_data.m_compression.opj_encode = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_stream_private,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_encode
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_compression.opj_end_compress = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_stream_private,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_end_compress
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_compression.opj_start_compress = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_stream_private,
            _: *mut opj_image,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_start_compress
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_compression.opj_write_tile = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: OPJ_UINT32,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: OPJ_UINT32,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_stream_private,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_write_tile
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: OPJ_UINT32,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_compression.opj_destroy = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_j2k_t) -> ()>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void) -> ()>,
      >(Some(
        opj_j2k_destroy as unsafe extern "C" fn(_: *mut opj_j2k_t) -> (),
      ));
      (*l_codec).m_codec_data.m_compression.opj_setup_encoder = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_cparameters_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_cparameters_t,
            _: *mut opj_image,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_setup_encoder
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_cparameters_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec)
        .m_codec_data
        .m_compression
        .opj_encoder_set_extra_options = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *const *const libc::c_char,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *const *const libc::c_char,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_j2k_encoder_set_extra_options
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *const *const libc::c_char,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).opj_set_threads = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_j2k_t, _: OPJ_UINT32) -> OPJ_BOOL>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_UINT32) -> OPJ_BOOL>,
      >(Some(
        opj_j2k_set_threads as unsafe extern "C" fn(_: *mut opj_j2k_t, _: OPJ_UINT32) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec = opj_j2k_create_compress() as *mut libc::c_void;
      if (*l_codec).m_codec.is_null() {
        opj_free(l_codec as *mut libc::c_void);
        return 0 as *mut opj_codec_t;
      }
    }
    2 => {
      /* get a JP2 decoder handle */
      (*l_codec).m_codec_data.m_compression.opj_encode = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_stream_private,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_encode
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_compression.opj_end_compress = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_stream_private,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_end_compress
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_compression.opj_start_compress = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_stream_private,
            _: *mut opj_image,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_start_compress
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_compression.opj_write_tile = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: OPJ_UINT32,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: OPJ_UINT32,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_stream_private,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_write_tile
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: OPJ_UINT32,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec_data.m_compression.opj_destroy = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_jp2_t) -> ()>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void) -> ()>,
      >(Some(
        opj_jp2_destroy as unsafe extern "C" fn(_: *mut opj_jp2_t) -> (),
      ));
      (*l_codec).m_codec_data.m_compression.opj_setup_encoder = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_cparameters_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *mut opj_cparameters_t,
            _: *mut opj_image,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_setup_encoder
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *mut opj_cparameters_t,
            _: *mut opj_image_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec)
        .m_codec_data
        .m_compression
        .opj_encoder_set_extra_options = ::std::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *const *const libc::c_char,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        Option<
          unsafe extern "C" fn(
            _: *mut libc::c_void,
            _: *const *const libc::c_char,
            _: *mut opj_event_mgr,
          ) -> OPJ_BOOL,
        >,
      >(Some(
        opj_jp2_encoder_set_extra_options
          as unsafe extern "C" fn(
            _: *mut opj_jp2_t,
            _: *const *const libc::c_char,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ));
      (*l_codec).opj_set_threads = ::std::mem::transmute::<
        Option<unsafe extern "C" fn(_: *mut opj_jp2_t, _: OPJ_UINT32) -> OPJ_BOOL>,
        Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_UINT32) -> OPJ_BOOL>,
      >(Some(
        opj_jp2_set_threads as unsafe extern "C" fn(_: *mut opj_jp2_t, _: OPJ_UINT32) -> OPJ_BOOL,
      ));
      (*l_codec).m_codec = opj_jp2_create(0i32) as *mut libc::c_void;
      if (*l_codec).m_codec.is_null() {
        opj_free(l_codec as *mut libc::c_void);
        return 0 as *mut opj_codec_t;
      }
    }
    -1 | 1 | _ => {
      opj_free(l_codec as *mut libc::c_void);
      return 0 as *mut opj_codec_t;
    }
  }
  opj_set_default_event_handler(&mut (*l_codec).m_event_mgr);
  return l_codec as *mut opj_codec_t;
}
#[no_mangle]
pub(crate) unsafe fn opj_set_default_encoder_parameters(
  mut parameters: *mut opj_cparameters_t,
) {
  if !parameters.is_null() {
    memset(
      parameters as *mut libc::c_void,
      0i32,
      ::std::mem::size_of::<opj_cparameters_t>() as libc::c_ulong,
    );
    /* UniPG>> */
    /* USE_JPWL */
    /* <<UniPG */
    (*parameters).cp_cinema = OPJ_OFF;
    (*parameters).rsiz = 0 as OPJ_UINT16;
    (*parameters).max_comp_size = 0i32;
    (*parameters).numresolution = 6i32;
    (*parameters).cp_rsiz = OPJ_STD_RSIZ;
    (*parameters).cblockw_init = 64i32;
    (*parameters).cblockh_init = 64i32;
    (*parameters).prog_order = OPJ_LRCP;
    (*parameters).roi_compno = -(1i32);
    (*parameters).subsampling_dx = 1i32;
    (*parameters).subsampling_dy = 1i32;
    (*parameters).tp_on = 0 as libc::c_char;
    (*parameters).decod_format = -(1i32);
    (*parameters).cod_format = -(1i32);
    (*parameters).tcp_rates[0 as usize] = 0 as libc::c_float;
    (*parameters).tcp_numlayers = 0i32;
    (*parameters).cp_disto_alloc = 0i32;
    (*parameters).cp_fixed_alloc = 0i32;
    (*parameters).cp_fixed_quality = 0i32;
    (*parameters).jpip_on = 0i32
  };
}
#[no_mangle]
pub(crate) unsafe fn opj_setup_encoder(
  mut p_codec: *mut opj_codec_t,
  mut parameters: *mut opj_cparameters_t,
  mut p_image: *mut opj_image_t,
) -> OPJ_BOOL {
  if !p_codec.is_null() && !parameters.is_null() && !p_image.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    if (*l_codec).is_decompressor == 0 {
      return (*l_codec)
        .m_codec_data
        .m_compression
        .opj_setup_encoder
        .expect("non-null function pointer")(
        (*l_codec).m_codec,
        parameters,
        p_image,
        &mut (*l_codec).m_event_mgr,
      );
    }
  }
  return 0i32;
}
/* default coding parameters */
/* DEPRECATED */
/* DEPRECATED */
/* no ROI */
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub(crate) unsafe fn opj_encoder_set_extra_options(
  mut p_codec: *mut opj_codec_t,
  mut options: *const *const libc::c_char,
) -> OPJ_BOOL {
  if !p_codec.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    if (*l_codec).is_decompressor == 0 {
      return (*l_codec)
        .m_codec_data
        .m_compression
        .opj_encoder_set_extra_options
        .expect("non-null function pointer")(
        (*l_codec).m_codec,
        options,
        &mut (*l_codec).m_event_mgr,
      );
    }
  }
  return 0i32;
}
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub(crate) unsafe fn opj_start_compress(
  mut p_codec: *mut opj_codec_t,
  mut p_image: *mut opj_image_t,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if !p_codec.is_null() && !p_stream.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    let mut l_stream = p_stream as *mut opj_stream_private_t;
    if (*l_codec).is_decompressor == 0 {
      return (*l_codec)
        .m_codec_data
        .m_compression
        .opj_start_compress
        .expect("non-null function pointer")(
        (*l_codec).m_codec,
        l_stream,
        p_image,
        &mut (*l_codec).m_event_mgr,
      );
    }
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_encode(
  mut p_info: *mut opj_codec_t,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if !p_info.is_null() && !p_stream.is_null() {
    let mut l_codec = p_info as *mut opj_codec_private_t;
    let mut l_stream = p_stream as *mut opj_stream_private_t;
    if (*l_codec).is_decompressor == 0 {
      return (*l_codec)
        .m_codec_data
        .m_compression
        .opj_encode
        .expect("non-null function pointer")(
        (*l_codec).m_codec,
        l_stream,
        &mut (*l_codec).m_event_mgr,
      );
    }
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_end_compress(
  mut p_codec: *mut opj_codec_t,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if !p_codec.is_null() && !p_stream.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    let mut l_stream = p_stream as *mut opj_stream_private_t;
    if (*l_codec).is_decompressor == 0 {
      return (*l_codec)
        .m_codec_data
        .m_compression
        .opj_end_compress
        .expect("non-null function pointer")(
        (*l_codec).m_codec,
        l_stream,
        &mut (*l_codec).m_event_mgr,
      );
    }
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_end_decompress(
  mut p_codec: *mut opj_codec_t,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if !p_codec.is_null() && !p_stream.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    let mut l_stream = p_stream as *mut opj_stream_private_t;
    if (*l_codec).is_decompressor == 0 {
      return 0i32;
    }
    return (*l_codec)
      .m_codec_data
      .m_decompression
      .opj_end_decompress
      .expect("non-null function pointer")(
      (*l_codec).m_codec,
      l_stream,
      &mut (*l_codec).m_event_mgr,
    );
  }
  return 0i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_set_MCT(
  mut parameters: *mut opj_cparameters_t,
  mut pEncodingMatrix: *mut OPJ_FLOAT32,
  mut p_dc_shift: *mut OPJ_INT32,
  mut pNbComp: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut l_matrix_size = pNbComp
    .wrapping_mul(pNbComp)
    .wrapping_mul(::std::mem::size_of::<OPJ_FLOAT32>() as OPJ_UINT32);
  let mut l_dc_shift_size =
    pNbComp.wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as OPJ_UINT32);
  let mut l_mct_total_size = l_matrix_size.wrapping_add(l_dc_shift_size);
  /* add MCT capability */
  if (*parameters).rsiz as libc::c_int & 0x8000i32 != 0 {
    (*parameters).rsiz = ((*parameters).rsiz as libc::c_int | 0x100i32) as OPJ_UINT16
  } else {
    (*parameters).rsiz = (0x8000i32 | 0x100i32) as OPJ_UINT16
  }
  (*parameters).irreversible = 1i32;
  /* use array based MCT */
  (*parameters).tcp_mct = 2 as libc::c_char;
  (*parameters).mct_data = opj_malloc(l_mct_total_size as size_t);
  if (*parameters).mct_data.is_null() {
    return 0i32;
  }
  memcpy(
    (*parameters).mct_data,
    pEncodingMatrix as *const libc::c_void,
    l_matrix_size as libc::c_ulong,
  );
  memcpy(
    ((*parameters).mct_data as *mut OPJ_BYTE).offset(l_matrix_size as isize) as *mut libc::c_void,
    p_dc_shift as *const libc::c_void,
    l_dc_shift_size as libc::c_ulong,
  );
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_write_tile(
  mut p_codec: *mut opj_codec_t,
  mut p_tile_index: OPJ_UINT32,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_size: OPJ_UINT32,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if !p_codec.is_null() && !p_stream.is_null() && !p_data.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    let mut l_stream = p_stream as *mut opj_stream_private_t;
    if (*l_codec).is_decompressor != 0 {
      return 0i32;
    }
    return (*l_codec)
      .m_codec_data
      .m_compression
      .opj_write_tile
      .expect("non-null function pointer")(
      (*l_codec).m_codec,
      p_tile_index,
      p_data,
      p_data_size,
      l_stream,
      &mut (*l_codec).m_event_mgr,
    );
  }
  return 0i32;
}
/* ---------------------------------------------------------------------- */
#[no_mangle]
pub(crate) unsafe fn opj_destroy_codec(mut p_codec: *mut opj_codec_t) {
  if !p_codec.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    if (*l_codec).is_decompressor != 0 {
      (*l_codec)
        .m_codec_data
        .m_decompression
        .opj_destroy
        .expect("non-null function pointer")((*l_codec).m_codec);
    } else {
      (*l_codec)
        .m_codec_data
        .m_compression
        .opj_destroy
        .expect("non-null function pointer")((*l_codec).m_codec);
    }
    (*l_codec).m_codec = 0 as *mut libc::c_void;
    opj_free(l_codec as *mut libc::c_void);
  };
}
/* ---------------------------------------------------------------------- */
#[no_mangle]
pub(crate) unsafe fn opj_dump_codec(
  mut p_codec: *mut opj_codec_t,
  mut info_flag: OPJ_INT32,
  mut output_stream: *mut FILE,
) {
  if !p_codec.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    (*l_codec)
      .opj_dump_codec
      .expect("non-null function pointer")((*l_codec).m_codec, info_flag, output_stream);
    return;
  };
}
#[no_mangle]
pub(crate) unsafe fn opj_get_cstr_info(
  mut p_codec: *mut opj_codec_t,
) -> *mut opj_codestream_info_v2_t {
  if !p_codec.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    return (*l_codec)
      .opj_get_codec_info
      .expect("non-null function pointer")((*l_codec).m_codec);
  }
  return 0 as *mut opj_codestream_info_v2_t;
}
#[no_mangle]
pub(crate) unsafe fn opj_destroy_cstr_info(mut cstr_info: *mut *mut opj_codestream_info_v2_t) {
  if !cstr_info.is_null() {
    if !(**cstr_info).m_default_tile_info.tccp_info.is_null() {
      opj_free((**cstr_info).m_default_tile_info.tccp_info as *mut libc::c_void);
    }
    if !(**cstr_info).tile_info.is_null() {
      opj_free((**cstr_info).tile_info as *mut libc::c_void);
    }
    opj_free(*cstr_info as *mut libc::c_void);
    *cstr_info = 0 as *mut opj_codestream_info_v2_t
  };
}
#[no_mangle]
pub(crate) unsafe fn opj_get_cstr_index(
  mut p_codec: *mut opj_codec_t,
) -> *mut opj_codestream_index_t {
  if !p_codec.is_null() {
    let mut l_codec = p_codec as *mut opj_codec_private_t;
    return (*l_codec)
      .opj_get_codec_index
      .expect("non-null function pointer")((*l_codec).m_codec);
  }
  return 0 as *mut opj_codestream_index_t;
}
#[no_mangle]
pub(crate) unsafe fn opj_destroy_cstr_index(
  mut p_cstr_index: *mut *mut opj_codestream_index_t,
) {
  if !(*p_cstr_index).is_null() {
    j2k_destroy_cstr_index(*p_cstr_index);
    *p_cstr_index = 0 as *mut opj_codestream_index_t
  };
}
#[no_mangle]
pub(crate) unsafe fn opj_stream_create_default_file_stream(
  mut fname: *const libc::c_char,
  mut p_is_read_stream: OPJ_BOOL,
) -> *mut opj_stream_t {
  return opj_stream_create_file_stream(
    fname,
    0x100000 as OPJ_SIZE_T,
    p_is_read_stream,
  );
}
#[no_mangle]
pub(crate) unsafe fn opj_stream_create_file_stream(
  mut fname: *const libc::c_char,
  mut p_size: OPJ_SIZE_T,
  mut p_is_read_stream: OPJ_BOOL,
) -> *mut opj_stream_t {
  let mut l_stream = 0 as *mut opj_stream_t;
  let mut p_file = 0 as *mut FILE;
  let mut mode = 0 as *const libc::c_char;
  if fname.is_null() {
    return 0 as *mut opj_stream_t;
  }
  if p_is_read_stream != 0 {
    mode = b"rb\x00" as *const u8 as *const libc::c_char
  } else {
    mode = b"wb\x00" as *const u8 as *const libc::c_char
  }
  p_file = fopen(fname, mode);
  if p_file.is_null() {
    return 0 as *mut opj_stream_t;
  }
  l_stream = opj_stream_create(p_size, p_is_read_stream);
  if l_stream.is_null() {
    fclose(p_file);
    return 0 as *mut opj_stream_t;
  }
  opj_stream_set_user_data(
    l_stream,
    p_file as *mut libc::c_void,
    Some(opj_close_from_file as unsafe extern "C" fn(_: *mut libc::c_void) -> ()),
  );
  opj_stream_set_user_data_length(
    l_stream,
    opj_get_data_length_from_file(p_file as *mut libc::c_void),
  );
  opj_stream_set_read_function(
    l_stream,
    Some(
      opj_read_from_file
        as unsafe extern "C" fn(
          _: *mut libc::c_void,
          _: OPJ_SIZE_T,
          _: *mut libc::c_void,
        ) -> OPJ_SIZE_T,
    ),
  );
  opj_stream_set_write_function(
    l_stream,
    ::std::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut libc::c_void,
          _: OPJ_SIZE_T,
          _: *mut libc::c_void,
        ) -> OPJ_SIZE_T,
      >,
      opj_stream_write_fn,
    >(Some(
      opj_write_from_file
        as unsafe extern "C" fn(
          _: *mut libc::c_void,
          _: OPJ_SIZE_T,
          _: *mut libc::c_void,
        ) -> OPJ_SIZE_T,
    )),
  );
  opj_stream_set_skip_function(
    l_stream,
    Some(
      opj_skip_from_file as unsafe extern "C" fn(_: OPJ_OFF_T, _: *mut libc::c_void) -> OPJ_OFF_T,
    ),
  );
  opj_stream_set_seek_function(
    l_stream,
    Some(
      opj_seek_from_file as unsafe extern "C" fn(_: OPJ_OFF_T, _: *mut libc::c_void) -> OPJ_BOOL,
    ),
  );
  return l_stream;
}
#[no_mangle]
pub(crate) unsafe fn opj_image_data_alloc(mut size: OPJ_SIZE_T) -> *mut libc::c_void {
  let mut ret = opj_aligned_malloc(size);
  /* printf("opj_image_data_alloc %p\n", ret); */
  return ret;
}
#[no_mangle]
pub(crate) unsafe fn opj_image_data_free(mut ptr: *mut libc::c_void) {
  /* printf("opj_image_data_free %p\n", ptr); */
  opj_aligned_free(ptr);
}
