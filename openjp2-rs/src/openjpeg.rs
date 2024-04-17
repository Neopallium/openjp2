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

pub use super::c_api_types::*;
use super::j2k::*;
pub(crate) use super::types::*;

use super::codec::*;
pub use super::image::{opj_image_create, opj_image_destroy, opj_image_tile_create};
use super::malloc::*;

#[cfg(feature = "file-io")]
use ::libc::FILE;

extern "C" {
  fn memset(_: *mut core::ffi::c_void, _: core::ffi::c_int, _: usize) -> *mut core::ffi::c_void;

  fn memcpy(
    _: *mut core::ffi::c_void,
    _: *const core::ffi::c_void,
    _: usize,
  ) -> *mut core::ffi::c_void;
}
/* ---------------------------------------------------------------------- */
/* Functions to set the message handlers */
#[no_mangle]
pub unsafe fn opj_set_info_handler(
  mut p_codec: *mut opj_codec_t,
  mut p_callback: opj_msg_callback,
  mut p_user_data: *mut core::ffi::c_void,
) -> OPJ_BOOL {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.set_info_handler(p_callback, p_user_data)
}

#[no_mangle]
pub unsafe fn opj_set_warning_handler(
  mut p_codec: *mut opj_codec_t,
  mut p_callback: opj_msg_callback,
  mut p_user_data: *mut core::ffi::c_void,
) -> OPJ_BOOL {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.set_warning_handler(p_callback, p_user_data)
}

#[no_mangle]
pub unsafe fn opj_set_error_handler(
  mut p_codec: *mut opj_codec_t,
  mut p_callback: opj_msg_callback,
  mut p_user_data: *mut core::ffi::c_void,
) -> OPJ_BOOL {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.set_error_handler(p_callback, p_user_data)
}

/* ---------------------------------------------------------------------- */
pub const OPJ_VERSION: &str = "2.5.2";
pub const OPJ_VERSION_C: *const u8 = b"2.5.2\x00" as *const u8;

#[no_mangle]
pub unsafe fn opj_version() -> *const core::ffi::c_char {
  OPJ_VERSION_C as *const core::ffi::c_char
}

/* ---------------------------------------------------------------------- */
/* DECOMPRESSION FUNCTIONS*/
#[no_mangle]
pub unsafe fn opj_create_decompress(mut p_format: OPJ_CODEC_FORMAT) -> *mut opj_codec_t {
  if let Some(codec) = opj_codec_private_t::new_decoder(p_format) {
    let l_codec = Box::new(codec);
    Box::into_raw(l_codec) as *mut opj_codec_t
  } else {
    std::ptr::null_mut()
  }
}

#[no_mangle]
pub unsafe fn opj_set_default_decoder_parameters(mut parameters: *mut opj_dparameters_t) {
  if !parameters.is_null() {
    memset(
      parameters as *mut core::ffi::c_void,
      0i32,
      core::mem::size_of::<opj_dparameters_t>(),
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
pub unsafe fn opj_codec_set_threads(
  mut p_codec: *mut opj_codec_t,
  mut num_threads: core::ffi::c_int,
) -> OPJ_BOOL {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.set_threads(num_threads)
}

#[no_mangle]
pub unsafe fn opj_setup_decoder(
  mut p_codec: *mut opj_codec_t,
  mut parameters: *mut opj_dparameters_t,
) -> OPJ_BOOL {
  if p_codec.is_null() || parameters.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  let parameters = &mut *parameters;
  l_codec.setup_decoder(parameters)
}

#[no_mangle]
pub unsafe fn opj_decoder_set_strict_mode(
  mut p_codec: *mut opj_codec_t,
  mut strict: OPJ_BOOL,
) -> OPJ_BOOL {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.decoder_set_strict_mode(strict)
}

#[no_mangle]
pub unsafe fn opj_read_header(
  mut p_stream: *mut opj_stream_t,
  mut p_codec: *mut opj_codec_t,
  mut p_image: *mut *mut opj_image_t,
) -> OPJ_BOOL {
  if p_codec.is_null() | p_stream.is_null() | p_image.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.read_header(p_stream, p_image)
}

#[no_mangle]
pub unsafe fn opj_set_decoded_components(
  mut p_codec: *mut opj_codec_t,
  mut numcomps: OPJ_UINT32,
  mut comps_indices: *const OPJ_UINT32,
  mut apply_color_transforms: OPJ_BOOL,
) -> OPJ_BOOL {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.set_decoded_components(numcomps, comps_indices, apply_color_transforms)
}

#[no_mangle]
pub unsafe fn opj_decode(
  mut p_codec: *mut opj_codec_t,
  mut p_stream: *mut opj_stream_t,
  mut p_image: *mut opj_image_t,
) -> OPJ_BOOL {
  if p_codec.is_null() | p_stream.is_null() | p_image.is_null() {
    return 0i32;
  }
  let p_image = unsafe { &mut *p_image };
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.decode(p_stream, p_image)
}

#[no_mangle]
pub unsafe fn opj_set_decode_area(
  mut p_codec: *mut opj_codec_t,
  mut p_image: *mut opj_image_t,
  mut p_start_x: OPJ_INT32,
  mut p_start_y: OPJ_INT32,
  mut p_end_x: OPJ_INT32,
  mut p_end_y: OPJ_INT32,
) -> OPJ_BOOL {
  if p_codec.is_null() | p_image.is_null() {
    return 0i32;
  }
  let p_image = unsafe { &mut *p_image };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.set_decode_area(p_image, p_start_x, p_start_y, p_end_x, p_end_y)
}

#[no_mangle]
pub unsafe fn opj_read_tile_header(
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
  if p_codec.is_null() | p_stream.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.read_tile_header(
    p_stream,
    p_tile_index,
    p_data_size,
    p_tile_x0,
    p_tile_y0,
    p_tile_x1,
    p_tile_y1,
    p_nb_comps,
    p_should_go_on,
  )
}

#[no_mangle]
pub unsafe fn opj_decode_tile_data(
  mut p_codec: *mut opj_codec_t,
  mut p_tile_index: OPJ_UINT32,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_size: OPJ_UINT32,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if p_codec.is_null() | p_stream.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.decode_tile_data(p_stream, p_tile_index, p_data, p_data_size)
}

#[no_mangle]
pub unsafe fn opj_get_decoded_tile(
  mut p_codec: *mut opj_codec_t,
  mut p_stream: *mut opj_stream_t,
  mut p_image: *mut opj_image_t,
  mut tile_index: OPJ_UINT32,
) -> OPJ_BOOL {
  if p_codec.is_null() | p_stream.is_null() | p_image.is_null() {
    return 0i32;
  }
  let p_image = unsafe { &mut *p_image };
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.get_decoded_tile(p_stream, p_image, tile_index)
}

#[no_mangle]
pub unsafe fn opj_set_decoded_resolution_factor(
  mut p_codec: *mut opj_codec_t,
  mut res_factor: OPJ_UINT32,
) -> OPJ_BOOL {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.set_decoded_resolution_factor(res_factor)
}

/* default decoding parameters */
/* ---------------------------------------------------------------------- */
/* COMPRESSION FUNCTIONS*/
#[no_mangle]
pub unsafe fn opj_create_compress(mut p_format: OPJ_CODEC_FORMAT) -> *mut opj_codec_t {
  if let Some(codec) = opj_codec_private_t::new_encoder(p_format) {
    let l_codec = Box::new(codec);
    Box::into_raw(l_codec) as *mut opj_codec_t
  } else {
    std::ptr::null_mut()
  }
}

/* default coding parameters */
#[no_mangle]
pub unsafe fn opj_set_default_encoder_parameters(mut parameters: *mut opj_cparameters_t) {
  if !parameters.is_null() {
    memset(
      parameters as *mut core::ffi::c_void,
      0i32,
      core::mem::size_of::<opj_cparameters_t>(),
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
    (*parameters).tp_on = 0 as core::ffi::c_char;
    (*parameters).decod_format = -(1i32);
    (*parameters).cod_format = -(1i32);
    (*parameters).tcp_rates[0_usize] = 0 as core::ffi::c_float;
    (*parameters).tcp_numlayers = 0i32;
    (*parameters).cp_disto_alloc = 0i32;
    (*parameters).cp_fixed_alloc = 0i32;
    (*parameters).cp_fixed_quality = 0i32;
    (*parameters).jpip_on = 0i32
  };
}

#[no_mangle]
pub unsafe fn opj_setup_encoder(
  mut p_codec: *mut opj_codec_t,
  mut parameters: *mut opj_cparameters_t,
  mut p_image: *mut opj_image_t,
) -> OPJ_BOOL {
  if p_codec.is_null() | parameters.is_null() | p_image.is_null() {
    return 0i32;
  }
  let p_image = unsafe { &mut *p_image };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  let parameters = &mut *parameters;
  l_codec.setup_encoder(parameters, p_image)
}

/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe fn opj_encoder_set_extra_options(
  mut p_codec: *mut opj_codec_t,
  mut options: *const *const core::ffi::c_char,
) -> OPJ_BOOL {
  if p_codec.is_null() | options.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.encoder_set_extra_options(options)
}

/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe fn opj_start_compress(
  mut p_codec: *mut opj_codec_t,
  mut p_image: *mut opj_image_t,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if p_codec.is_null() | p_stream.is_null() | p_image.is_null() {
    return 0i32;
  }
  let p_image = unsafe { &mut *p_image };
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.start_compress(p_image, p_stream)
}

#[no_mangle]
pub unsafe fn opj_encode(
  mut p_codec: *mut opj_codec_t,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if p_codec.is_null() | p_stream.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.encode(p_stream)
}

#[no_mangle]
pub unsafe fn opj_end_compress(
  mut p_codec: *mut opj_codec_t,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if p_codec.is_null() | p_stream.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.end_compress(p_stream)
}

#[no_mangle]
pub unsafe fn opj_end_decompress(
  mut p_codec: *mut opj_codec_t,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if p_codec.is_null() | p_stream.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.end_decompress(p_stream)
}

#[no_mangle]
pub unsafe fn opj_set_MCT(
  mut parameters: *mut opj_cparameters_t,
  mut pEncodingMatrix: *mut OPJ_FLOAT32,
  mut p_dc_shift: *mut OPJ_INT32,
  mut pNbComp: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut l_matrix_size = pNbComp
    .wrapping_mul(pNbComp)
    .wrapping_mul(core::mem::size_of::<OPJ_FLOAT32>() as OPJ_UINT32);
  let mut l_dc_shift_size = pNbComp.wrapping_mul(core::mem::size_of::<OPJ_INT32>() as OPJ_UINT32);
  let mut l_mct_total_size = l_matrix_size.wrapping_add(l_dc_shift_size);
  /* add MCT capability */
  if (*parameters).rsiz as core::ffi::c_int & 0x8000i32 != 0 {
    (*parameters).rsiz = ((*parameters).rsiz as core::ffi::c_int | 0x100i32) as OPJ_UINT16
  } else {
    (*parameters).rsiz = (0x8000i32 | 0x100i32) as OPJ_UINT16
  }
  (*parameters).irreversible = 1i32;
  /* use array based MCT */
  (*parameters).tcp_mct = 2 as core::ffi::c_char;
  (*parameters).mct_data = opj_malloc(l_mct_total_size as size_t);
  if (*parameters).mct_data.is_null() {
    return 0i32;
  }
  memcpy(
    (*parameters).mct_data,
    pEncodingMatrix as *const core::ffi::c_void,
    l_matrix_size as usize,
  );
  memcpy(
    ((*parameters).mct_data as *mut OPJ_BYTE).offset(l_matrix_size as isize)
      as *mut core::ffi::c_void,
    p_dc_shift as *const core::ffi::c_void,
    l_dc_shift_size as usize,
  );
  1i32
}

#[no_mangle]
pub unsafe fn opj_write_tile(
  mut p_codec: *mut opj_codec_t,
  mut p_tile_index: OPJ_UINT32,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_size: OPJ_UINT32,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if p_codec.is_null() | p_stream.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.write_tile(p_tile_index, p_data, p_data_size, p_stream)
}

/* ---------------------------------------------------------------------- */
#[no_mangle]
pub unsafe fn opj_destroy_codec(mut p_codec: *mut opj_codec_t) {
  if p_codec.is_null() {
    return;
  }
  let _ = Box::from_raw(p_codec as *mut opj_codec_private_t);
}

/* ---------------------------------------------------------------------- */
#[cfg(feature = "file-io")]
#[no_mangle]
pub unsafe fn opj_dump_codec(
  mut p_codec: *mut opj_codec_t,
  mut info_flag: OPJ_INT32,
  mut output_stream: *mut FILE,
) {
  if p_codec.is_null() {
    return;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.dump_codec(info_flag, output_stream)
}

#[no_mangle]
pub unsafe fn opj_get_cstr_info(mut p_codec: *mut opj_codec_t) -> *mut opj_codestream_info_v2_t {
  if p_codec.is_null() {
    return std::ptr::null_mut::<opj_codestream_info_v2_t>();
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.get_cstr_info()
}

#[no_mangle]
pub unsafe fn opj_destroy_cstr_info(mut cstr_info: *mut *mut opj_codestream_info_v2_t) {
  if !cstr_info.is_null() {
    if !(**cstr_info).m_default_tile_info.tccp_info.is_null() {
      opj_free((**cstr_info).m_default_tile_info.tccp_info as *mut core::ffi::c_void);
    }
    if !(**cstr_info).tile_info.is_null() {
      opj_free((**cstr_info).tile_info as *mut core::ffi::c_void);
    }
    opj_free(*cstr_info as *mut core::ffi::c_void);
    *cstr_info = std::ptr::null_mut::<opj_codestream_info_v2_t>()
  };
}

#[no_mangle]
pub unsafe fn opj_get_cstr_index(mut p_codec: *mut opj_codec_t) -> *mut opj_codestream_index_t {
  if p_codec.is_null() {
    return std::ptr::null_mut::<opj_codestream_index_t>();
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.get_cstr_index()
}

#[no_mangle]
pub unsafe fn opj_destroy_cstr_index(mut p_cstr_index: *mut *mut opj_codestream_index_t) {
  if !(*p_cstr_index).is_null() {
    j2k_destroy_cstr_index(*p_cstr_index);
    *p_cstr_index = std::ptr::null_mut::<opj_codestream_index_t>()
  };
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_create(
  mut p_buffer_size: OPJ_SIZE_T,
  mut l_is_input: OPJ_BOOL,
) -> *mut opj_stream_t {
  let l_stream = Box::new(opj_stream_private::new_custom(
    p_buffer_size,
    l_is_input != 0,
  ));
  let p_stream = Box::into_raw(l_stream) as *mut opj_stream_t;
  log::trace!("-- create stream: {:?}", p_stream);
  p_stream
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_default_create(mut l_is_input: OPJ_BOOL) -> *mut opj_stream_t {
  opj_stream_create(0x100000 as OPJ_SIZE_T, l_is_input)
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_destroy(mut p_stream: *mut opj_stream_t) {
  if !p_stream.is_null() {
    let _ = Box::from_raw(p_stream as *mut opj_stream_private_t);
  }
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_read_function(
  mut p_stream: *mut opj_stream_t,
  mut p_function: opj_stream_read_fn,
) {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  if !p_stream.is_input() {
    return;
  }
  if let Some(custom) = p_stream.as_custom() {
    custom.set_read(p_function);
  }
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_seek_function(
  mut p_stream: *mut opj_stream_t,
  mut p_function: opj_stream_seek_fn,
) {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  if let Some(custom) = p_stream.as_custom() {
    custom.set_seek(p_function);
  }
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_write_function(
  mut p_stream: *mut opj_stream_t,
  mut p_function: opj_stream_write_fn,
) {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  if p_stream.is_input() {
    return;
  }
  if let Some(custom) = p_stream.as_custom() {
    custom.set_write(p_function);
  }
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_skip_function(
  mut p_stream: *mut opj_stream_t,
  mut p_function: opj_stream_skip_fn,
) {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  if let Some(custom) = p_stream.as_custom() {
    custom.set_skip(p_function);
  }
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_user_data(
  mut p_stream: *mut opj_stream_t,
  mut p_data: *mut core::ffi::c_void,
  mut p_function: opj_stream_free_user_data_fn,
) {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  if let Some(custom) = p_stream.as_custom() {
    custom.set_user_data(p_data, p_function);
  }
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_user_data_length(
  mut p_stream: *mut opj_stream_t,
  mut data_length: OPJ_UINT64,
) {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  p_stream.set_stream_length(data_length);
}

#[cfg(feature = "file-io")]
#[no_mangle]
pub unsafe fn opj_stream_create_default_file_stream(
  mut fname: *const core::ffi::c_char,
  mut p_is_read_stream: OPJ_BOOL,
) -> *mut opj_stream_t {
  opj_stream_create_file_stream(fname, 0x100000 as OPJ_SIZE_T, p_is_read_stream)
}

#[cfg(feature = "file-io")]
#[no_mangle]
pub unsafe fn opj_stream_create_file_stream(
  mut fname: *const core::ffi::c_char,
  mut p_size: OPJ_SIZE_T,
  mut p_is_read_stream: OPJ_BOOL,
) -> *mut opj_stream_t {
  if fname.is_null() {
    return std::ptr::null_mut::<opj_stream_t>();
  }
  match std::ffi::CStr::from_ptr(fname).to_str() {
    Ok(name) => {
      let l_stream = Box::new(opj_stream_private::new_file(
        name,
        p_size,
        p_is_read_stream != 0,
      ));
      Box::into_raw(l_stream) as *mut opj_stream_t
    }
    Err(err) => {
      log::error!("Failed to convert C filename to Rust String: {err}");
      return std::ptr::null_mut::<opj_stream_t>();
    }
  }
}

#[no_mangle]
pub unsafe fn opj_image_data_alloc(mut size: OPJ_SIZE_T) -> *mut core::ffi::c_void {
  /* printf("opj_image_data_alloc %p\n", ret); */
  opj_aligned_malloc(size)
}
#[no_mangle]
pub unsafe fn opj_image_data_free(mut ptr: *mut core::ffi::c_void) {
  /* printf("opj_image_data_free %p\n", ptr); */
  opj_aligned_free(ptr);
}

/* Stub implementation */
#[no_mangle]
pub fn opj_has_thread_support() -> OPJ_BOOL {
  0i32
}
#[no_mangle]
pub fn opj_get_num_cpus() -> core::ffi::c_int {
  1i32
}
