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
use super::j2k::*;

use super::event::opj_event_mgr;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tcd_marker_info {
  pub need_PLT: OPJ_BOOL,
  pub packet_count: OPJ_UINT32,
  pub p_packet_size: *mut OPJ_UINT32,
}
pub(crate) type opj_tcd_marker_info_t = opj_tcd_marker_info;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_stream_private {
  pub m_user_data: *mut core::ffi::c_void,
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
      _: &mut opj_event_mgr,
    ) -> OPJ_OFF_T,
  >,
  pub m_opj_seek: Option<
    unsafe extern "C" fn(
      _: *mut opj_stream_private,
      _: OPJ_OFF_T,
      _: &mut opj_event_mgr,
    ) -> OPJ_BOOL,
  >,
  pub m_bytes_in_buffer: OPJ_SIZE_T,
  pub m_byte_offset: OPJ_OFF_T,
  pub m_buffer_size: OPJ_SIZE_T,
  pub m_status: OPJ_UINT32,
}
pub(crate) type opj_stream_private_t = opj_stream_private;

pub(crate) type opj_jp2_proc =
  unsafe fn(_: &mut opj_jp2, _: *mut opj_stream_private_t, _: &mut opj_event_mgr) -> OPJ_BOOL;
pub(crate) type opj_jp2_proc_list_t = super::function_list::ProcedureList<opj_jp2_proc>;

#[repr(C)]
#[derive(Clone)]
pub(crate) struct opj_jp2 {
  pub j2k: opj_j2k,
  pub m_validation_list: *mut opj_jp2_proc_list_t,
  pub m_procedure_list: *mut opj_jp2_proc_list_t,
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

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_jp2_color {
  pub icc_profile_buf: *mut OPJ_BYTE,
  pub icc_profile_len: OPJ_UINT32,
  pub jp2_cdef: *mut opj_jp2_cdef_t,
  pub jp2_pclr: *mut opj_jp2_pclr_t,
  pub jp2_has_colr: OPJ_BYTE,
}
pub(crate) type opj_jp2_color_t = opj_jp2_color;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_jp2_pclr {
  pub entries: *mut OPJ_UINT32,
  pub channel_sign: *mut OPJ_BYTE,
  pub channel_size: *mut OPJ_BYTE,
  pub cmap: *mut opj_jp2_cmap_comp_t,
  pub nr_entries: OPJ_UINT16,
  pub nr_channels: OPJ_BYTE,
}
pub(crate) type opj_jp2_pclr_t = opj_jp2_pclr;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_jp2_cmap_comp {
  pub cmp: OPJ_UINT16,
  pub mtyp: OPJ_BYTE,
  pub pcol: OPJ_BYTE,
}
pub(crate) type opj_jp2_cmap_comp_t = opj_jp2_cmap_comp;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_jp2_cdef {
  pub info: *mut opj_jp2_cdef_info_t,
  pub n: OPJ_UINT16,
}
pub(crate) type opj_jp2_cdef_t = opj_jp2_cdef;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_jp2_cdef_info {
  pub cn: OPJ_UINT16,
  pub typ: OPJ_UINT16,
  pub asoc: OPJ_UINT16,
}
pub(crate) type opj_jp2_cdef_info_t = opj_jp2_cdef_info;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_jp2_comps {
  pub depth: OPJ_UINT32,
  pub sgnd: OPJ_UINT32,
  pub bpcc: OPJ_UINT32,
}
pub(crate) type opj_jp2_comps_t = opj_jp2_comps;

pub(crate) type opj_j2k_proc =
  unsafe fn(_: &mut opj_j2k, _: *mut opj_stream_private_t, _: &mut opj_event_mgr) -> OPJ_BOOL;
pub(crate) type opj_j2k_proc_list_t = super::function_list::ProcedureList<opj_j2k_proc>;

#[repr(C)]
#[derive(Clone)]
pub(crate) struct opj_j2k {
  pub m_is_decoder: OPJ_BOOL,
  pub m_specific_param: C2RustUnnamed_2,
  pub m_private_image: *mut opj_image_t,
  pub m_output_image: *mut opj_image_t,
  pub m_cp: opj_cp_t,
  pub m_procedure_list: *mut opj_j2k_proc_list_t,
  pub m_validation_list: *mut opj_j2k_proc_list_t,
  pub cstr_index: *mut opj_codestream_index_t,
  pub m_current_tile_number: OPJ_UINT32,
  pub m_tcd: *mut opj_tcd,
  pub ihdr_w: OPJ_UINT32,
  pub ihdr_h: OPJ_UINT32,
  pub dump_state: core::ffi::c_uint,
}

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub(crate) struct opj_tcd {
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
  pub win_x0: OPJ_UINT32,
  pub win_y0: OPJ_UINT32,
  pub win_x1: OPJ_UINT32,
  pub win_y1: OPJ_UINT32,
  pub whole_tile_decoding: OPJ_BOOL,
  pub used_component: *mut OPJ_BOOL,
}
pub(crate) type opj_tcd_t = opj_tcd;

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub(crate) struct opj_tcp {
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
pub(crate) type opj_tcp_t = opj_tcp;

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub(crate) struct opj_simple_mcc_decorrelation_data {
  pub m_index: OPJ_UINT32,
  pub m_nb_comps: OPJ_UINT32,
  pub m_decorrelation_array: *mut opj_mct_data_t,
  pub m_offset_array: *mut opj_mct_data_t,
  #[bitfield(name = "m_is_irreversible", ty = "OPJ_BITFIELD", bits = "0..=0")]
  pub m_is_irreversible: [u8; 1],
  #[bitfield(padding)]
  pub c2rust_padding: [u8; 7],
}
pub(crate) type opj_simple_mcc_decorrelation_data_t = opj_simple_mcc_decorrelation_data;

pub type J2K_MCT_ARRAY_TYPE = MCT_ARRAY_TYPE;
pub type MCT_ARRAY_TYPE = core::ffi::c_uint;
pub const MCT_TYPE_OFFSET: MCT_ARRAY_TYPE = 2;
pub const MCT_TYPE_DECORRELATION: MCT_ARRAY_TYPE = 1;
pub const MCT_TYPE_DEPENDENCY: MCT_ARRAY_TYPE = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_mct_data {
  pub m_element_type: MCTElementType,
  pub m_array_type: J2K_MCT_ARRAY_TYPE,
  pub m_index: OPJ_UINT32,
  pub m_data: *mut OPJ_BYTE,
  pub m_data_size: OPJ_UINT32,
}
pub(crate) type opj_mct_data_t = opj_mct_data;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tccp {
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
pub(crate) type opj_tccp_t = opj_tccp;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_stepsize {
  pub expn: OPJ_INT32,
  pub mant: OPJ_INT32,
}
pub(crate) type opj_stepsize_t = opj_stepsize;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_ppx_struct {
  pub m_data: *mut OPJ_BYTE,
  pub m_data_size: OPJ_UINT32,
}
pub(crate) type opj_ppx = opj_ppx_struct;

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub(crate) struct opj_cp {
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
pub(crate) type opj_cp_t = opj_cp;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union C2RustUnnamed_0 {
  pub m_dec: opj_decoding_param_t,
  pub m_enc: opj_encoding_param_t,
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

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub(crate) struct opj_encoding_param {
  pub m_max_comp_size: OPJ_UINT32,
  pub m_tp_pos: OPJ_INT32,
  pub m_matrice: *mut OPJ_INT32,
  pub m_tp_flag: OPJ_BYTE,
  pub m_quality_layer_alloc_strategy: J2K_QUALITY_LAYER_ALLOCATION_STRATEGY,
  pub m_tp_on: bool,
}
pub(crate) type opj_encoding_param_t = opj_encoding_param;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_decoding_param {
  pub m_reduce: OPJ_UINT32,
  pub m_layer: OPJ_UINT32,
}
pub(crate) type opj_decoding_param_t = opj_decoding_param;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tcd_image {
  pub tiles: *mut opj_tcd_tile_t,
}
pub(crate) type opj_tcd_image_t = opj_tcd_image;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tcd_tile {
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
pub(crate) type opj_tcd_tile_t = opj_tcd_tile;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tcd_tilecomp {
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
pub(crate) type opj_tcd_tilecomp_t = opj_tcd_tilecomp;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tcd_resolution {
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
pub(crate) type opj_tcd_resolution_t = opj_tcd_resolution;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tcd_band {
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
pub(crate) type opj_tcd_band_t = opj_tcd_band;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tcd_precinct {
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
pub(crate) type opj_tcd_precinct_t = opj_tcd_precinct;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tgt_tree {
  pub numleafsh: OPJ_UINT32,
  pub numleafsv: OPJ_UINT32,
  pub numnodes: OPJ_UINT32,
  pub nodes: *mut opj_tgt_node_t,
  pub nodes_size: OPJ_UINT32,
}
pub(crate) type opj_tgt_tree_t = opj_tgt_tree;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tgt_node {
  pub parent: *mut opj_tgt_node,
  pub value: OPJ_INT32,
  pub low: OPJ_INT32,
  pub known: OPJ_UINT32,
}
pub(crate) type opj_tgt_node_t = opj_tgt_node;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union C2RustUnnamed_1 {
  pub enc: *mut opj_tcd_cblk_enc_t,
  pub dec: *mut opj_tcd_cblk_dec_t,
  pub blocks: *mut core::ffi::c_void,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tcd_cblk_dec {
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
pub(crate) type opj_tcd_cblk_dec_t = opj_tcd_cblk_dec;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tcd_seg_data_chunk {
  pub data: *mut OPJ_BYTE,
  pub len: OPJ_UINT32,
}
pub(crate) type opj_tcd_seg_data_chunk_t = opj_tcd_seg_data_chunk;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tcd_seg {
  pub len: OPJ_UINT32,
  pub numpasses: OPJ_UINT32,
  pub real_num_passes: OPJ_UINT32,
  pub maxpasses: OPJ_UINT32,
  pub numnewpasses: OPJ_UINT32,
  pub newlen: OPJ_UINT32,
}
pub(crate) type opj_tcd_seg_t = opj_tcd_seg;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tcd_cblk_enc {
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
pub(crate) type opj_tcd_cblk_enc_t = opj_tcd_cblk_enc;

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub(crate) struct opj_tcd_pass {
  pub rate: OPJ_UINT32,
  pub distortiondec: OPJ_FLOAT64,
  pub len: OPJ_UINT32,
  #[bitfield(name = "term", ty = "OPJ_BITFIELD", bits = "0..=0")]
  pub term: [u8; 1],
  #[bitfield(padding)]
  pub c2rust_padding: [u8; 3],
}
pub(crate) type opj_tcd_pass_t = opj_tcd_pass;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_tcd_layer {
  pub numpasses: OPJ_UINT32,
  pub len: OPJ_UINT32,
  pub disto: OPJ_FLOAT64,
  pub data: *mut OPJ_BYTE,
}
pub(crate) type opj_tcd_layer_t = opj_tcd_layer;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union C2RustUnnamed_2 {
  pub m_decoder: opj_j2k_dec_t,
  pub m_encoder: opj_j2k_enc_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct opj_j2k_enc {
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
pub(crate) type opj_j2k_enc_t = opj_j2k_enc;

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub(crate) struct opj_j2k_dec {
  pub m_state: J2KState,
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
pub(crate) type opj_j2k_dec_t = opj_j2k_dec;
