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

// This module is the C ABI shim; it moves wholesale to the `openjp2-capi` crate in phase 6.
#![allow(unsafe_op_in_unsafe_fn)]

use crate::image::*;
use crate::params::opj_cparameters_t;
use crate::cstr_info::{opj_codestream_index_t, opj_codestream_info_v2_t};
use openjp2_ctypes::*;

use crate::codec::Codec;
use openjp2_core::{opj_stream_private_t, TileInfo};

#[cfg(feature = "file-io")]
use ::libc::FILE;

/* ---------------------------------------------------------------------- */
/* Functions to set the message handlers */

/// The C `opj_msg_callback` type from openjpeg.h, declared here on the ABI side: the core only
/// sees the Rust closures these functions wrap around it.
pub type opj_msg_callback_fn =
  unsafe extern "C" fn(_: *const core::ffi::c_char, _: *mut core::ffi::c_void) -> ();
pub type opj_msg_callback = Option<opj_msg_callback_fn>;

/// A `Send` carrier for the C callback's user-data pointer: moving the address
/// across threads is fine; using it is the C caller's responsibility.
struct SendPtr(*mut core::ffi::c_void);
// SAFETY: the pointer is only passed through to the C callback; the core's event
// manager never dereferences it.
unsafe impl Send for SendPtr {}

/// Wrap a C callback + user data in the core's `FnMut(&str)` sink.
fn wrap_callback(
  callback: opj_msg_callback,
  user_data: *mut core::ffi::c_void,
) -> impl FnMut(&str) + Send {
  let user_data = SendPtr(user_data);
  move |msg: &str| {
    let Some(callback) = callback else {
      return;
    };
    if let Ok(c_msg) = std::ffi::CString::new(msg) {
      // SAFETY: the callback is the one the caller installed, invoked with the caller's data.
      unsafe { callback(c_msg.as_ptr(), user_data.0) };
    }
  }
}

/// Runs `f`, converting any panic into the C-style failure value.
///
/// A panic must NEVER unwind across the C ABI: that is undefined behaviour
/// in C callers.  The failure default (OPJ_FALSE / NULL / 0) signals the
/// error the same way the core's error events do.
fn ffi_guard<T: Default>(f: impl FnOnce() -> T) -> T {
  match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
    Ok(v) => v,
    Err(payload) => {
      let detail = payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".to_string());
      eprintln!("openjp2: internal panic caught at the C ABI boundary: {detail}");
      T::default()
    }
  }
}

#[no_mangle]
pub unsafe fn opj_set_info_handler(
  p_codec: *mut opj_codec_t,
  p_callback: opj_msg_callback,
  p_user_data: *mut core::ffi::c_void,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut Codec);
  l_codec.set_info_handler(wrap_callback(p_callback, p_user_data))
})}

#[no_mangle]
pub unsafe fn opj_set_warning_handler(
  p_codec: *mut opj_codec_t,
  p_callback: opj_msg_callback,
  p_user_data: *mut core::ffi::c_void,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut Codec);
  l_codec.set_warning_handler(wrap_callback(p_callback, p_user_data))
})}

#[no_mangle]
pub unsafe fn opj_set_error_handler(
  p_codec: *mut opj_codec_t,
  p_callback: opj_msg_callback,
  p_user_data: *mut core::ffi::c_void,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut Codec);
  l_codec.set_error_handler(wrap_callback(p_callback, p_user_data))
})}

/* ---------------------------------------------------------------------- */
pub const OPJ_VERSION: &str = "2.5.4";
pub const OPJ_VERSION_C: *const u8 = b"2.5.4\x00" as *const u8;

#[no_mangle]
pub fn opj_version() -> *const core::ffi::c_char {ffi_guard(|| {
  OPJ_VERSION_C as *const core::ffi::c_char
})}

/* ---------------------------------------------------------------------- */
/* DECOMPRESSION FUNCTIONS*/
#[no_mangle]
pub fn opj_create_decompress(p_format: OPJ_CODEC_FORMAT) -> *mut opj_codec_t {ffi_guard(|| {
  if let Some(codec) = Codec::new_decoder(p_format) {
    let l_codec = Box::new(codec);
    Box::into_raw(l_codec) as *mut opj_codec_t
  } else {
    core::ptr::null_mut()
  }
})}

#[no_mangle]
pub unsafe fn opj_set_default_decoder_parameters(parameters: *mut opj_dparameters_t) {ffi_guard(|| {
  if parameters.is_null() {
    return;
  }
  let parameters = &mut *parameters;
  parameters.set_defaults();
})}

#[no_mangle]
pub unsafe fn opj_codec_set_threads(
  p_codec: *mut opj_codec_t,
  num_threads: core::ffi::c_int,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut Codec);
  l_codec.set_threads(num_threads)
})}

#[no_mangle]
pub unsafe fn opj_setup_decoder(
  p_codec: *mut opj_codec_t,
  parameters: *mut opj_dparameters_t,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() || parameters.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut Codec);
  let parameters = &mut *parameters;
  l_codec.setup_decoder(parameters)
})}

#[no_mangle]
pub unsafe fn opj_decoder_set_strict_mode(
  p_codec: *mut opj_codec_t,
  strict: OPJ_BOOL,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut Codec);
  l_codec.decoder_set_strict_mode(strict)
})}

#[no_mangle]
pub unsafe fn opj_read_header(
  p_stream: *mut opj_stream_t,
  p_codec: *mut opj_codec_t,
  p_image: *mut *mut opj_image_t,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() | p_stream.is_null() | p_image.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut Codec);
  match l_codec.read_header(p_stream) {
    Some(image) => {
      *p_image = Box::into_raw(core_into_opj(image));
      1
    }
    None => 0,
  }
})}

#[no_mangle]
pub unsafe fn opj_set_decoded_components(
  p_codec: *mut opj_codec_t,
  numcomps: OPJ_UINT32,
  comps_indices: *const OPJ_UINT32,
  apply_color_transforms: OPJ_BOOL,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut Codec);
  let components = unsafe { core::slice::from_raw_parts(comps_indices, numcomps as usize) };
  l_codec.set_decoded_components(components, apply_color_transforms)
})}

#[no_mangle]
pub unsafe fn opj_decode(
  p_codec: *mut opj_codec_t,
  p_stream: *mut opj_stream_t,
  p_image: *mut opj_image_t,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() | p_stream.is_null() | p_image.is_null() {
    return 0i32;
  }
  let p_image = unsafe { &mut *p_image };
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut Codec);
  match l_codec.decode(p_stream) {
    Some(core) => {
      let fresh = core_into_opj(core);
      p_image.clear_comps();
      p_image.clear_icc_profile();
      *p_image = *fresh;
      1
    }
    None => 0,
  }
})}

#[no_mangle]
pub unsafe fn opj_set_decode_area(
  p_codec: *mut opj_codec_t,
  p_image: *mut opj_image_t,
  p_start_x: OPJ_INT32,
  p_start_y: OPJ_INT32,
  p_end_x: OPJ_INT32,
  p_end_y: OPJ_INT32,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() | p_image.is_null() {
    return 0i32;
  }
  let p_image = unsafe { &mut *p_image };
  let l_codec = &mut *(p_codec as *mut Codec);
  let mut core = core_from_opj(p_image);
  // set_decode_area only adjusts the component geometry; the caller's data
  // buffers are stale at this point (C leaves them alone and opj_decode
  // replaces them), so do not carry them through the round-trip.
  for comp in &mut core.comps {
    comp.data.clear();
  }
  let ret = l_codec.set_decode_area(&mut core, p_start_x, p_start_y, p_end_x, p_end_y);
  let fresh = core_into_opj(core);
  p_image.clear_comps();
  p_image.clear_icc_profile();
  *p_image = *fresh;
  ret
})}

#[no_mangle]
pub unsafe fn opj_read_tile_header(
  p_codec: *mut opj_codec_t,
  p_stream: *mut opj_stream_t,
  p_tile_index: *mut OPJ_UINT32,
  p_data_size: *mut OPJ_UINT32,
  p_tile_x0: *mut OPJ_INT32,
  p_tile_y0: *mut OPJ_INT32,
  p_tile_x1: *mut OPJ_INT32,
  p_tile_y1: *mut OPJ_INT32,
  p_nb_comps: *mut OPJ_UINT32,
  p_should_go_on: *mut OPJ_BOOL,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() | p_stream.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut Codec);
  match l_codec.read_tile_header(p_stream) {
    Some(tile_info) => {
      unsafe {
        if !p_tile_index.is_null() {
          *p_tile_index = tile_info.index;
        }
        if !p_data_size.is_null() {
          *p_data_size = tile_info.data_size.unwrap_or_default();
        }
        if !p_tile_x0.is_null() {
          *p_tile_x0 = tile_info.x0;
        }
        if !p_tile_y0.is_null() {
          *p_tile_y0 = tile_info.y0;
        }
        if !p_tile_x1.is_null() {
          *p_tile_x1 = tile_info.x1;
        }
        if !p_tile_y1.is_null() {
          *p_tile_y1 = tile_info.y1;
        }
        if !p_nb_comps.is_null() {
          *p_nb_comps = tile_info.nb_comps;
        }
        if !p_should_go_on.is_null() {
          *p_should_go_on = tile_info.go_on as _;
        }
      }
      1
    }
    None => 0,
  }
})}

#[no_mangle]
pub unsafe fn opj_decode_tile_data(
  p_codec: *mut opj_codec_t,
  p_tile_index: OPJ_UINT32,
  p_data: *mut OPJ_BYTE,
  p_data_size: OPJ_UINT32,
  p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() | p_stream.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut Codec);
  let p_data = if p_data.is_null() {
    None
  } else {
    Some(unsafe { core::slice::from_raw_parts_mut(p_data, p_data_size as usize) })
  };
  l_codec.decode_tile_data(p_stream, p_tile_index, p_data)
})}

#[no_mangle]
pub unsafe fn opj_get_decoded_tile(
  p_codec: *mut opj_codec_t,
  p_stream: *mut opj_stream_t,
  p_image: *mut opj_image_t,
  tile_index: OPJ_UINT32,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() | p_stream.is_null() | p_image.is_null() {
    return 0i32;
  }
  let p_image = unsafe { &mut *p_image };
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut Codec);
  match l_codec.get_decoded_tile(p_stream, tile_index) {
    Some(core) => {
      let fresh = core_into_opj(core);
      p_image.clear_comps();
      p_image.clear_icc_profile();
      *p_image = *fresh;
      1
    }
    None => 0,
  }
})}

#[no_mangle]
pub unsafe fn opj_set_decoded_resolution_factor(
  p_codec: *mut opj_codec_t,
  res_factor: OPJ_UINT32,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut Codec);
  l_codec.set_decoded_resolution_factor(res_factor)
})}

/* default decoding parameters */
/* ---------------------------------------------------------------------- */
/* COMPRESSION FUNCTIONS*/
#[no_mangle]
pub unsafe fn opj_create_compress(p_format: OPJ_CODEC_FORMAT) -> *mut opj_codec_t {ffi_guard(|| {
  if let Some(codec) = Codec::new_encoder(p_format) {
    let l_codec = Box::new(codec);
    Box::into_raw(l_codec) as *mut opj_codec_t
  } else {
    core::ptr::null_mut()
  }
})}

/* default coding parameters */
#[no_mangle]
pub unsafe fn opj_set_default_encoder_parameters(parameters: *mut opj_cparameters_t) {ffi_guard(|| {
  if parameters.is_null() {
    return;
  }
  unsafe {
    parameters.write_bytes(0, 1);
  }
  let parameters = &mut *parameters;
  parameters.set_defaults();
})}

#[no_mangle]
pub unsafe fn opj_setup_encoder(
  p_codec: *mut opj_codec_t,
  parameters: *mut opj_cparameters_t,
  p_image: *mut opj_image_t,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() | parameters.is_null() | p_image.is_null() {
    return 0i32;
  }
  let p_image = unsafe { &mut *p_image };
  let l_codec = &mut *(p_codec as *mut Codec);
  let parameters = &mut *parameters;
  let mut core = core_from_opj(p_image);
  let mut safe = encoder_params_from_c(parameters, core.numcomps);
  let ret = l_codec.setup_encoder(&mut safe, &mut core);
  // The codec used to free the caller's MCT buffer here; the C caller hands
  // ownership of it to opj_setup_encoder.
  if !parameters.mct_data.is_null() {
    // SAFETY: allocated with opj_malloc in opj_set_MCT.
    crate::params::c_free(parameters.mct_data);
    parameters.mct_data = core::ptr::null_mut();
  }
  let fresh = core_into_opj(core);
  p_image.clear_comps();
  p_image.clear_icc_profile();
  *p_image = *fresh;
  ret
})}

/// Converts the C-ABI encoder parameters into the core's safe owned struct.
fn encoder_params_from_c(
  parameters: &opj_cparameters_t,
  numcomps: u32,
) -> openjp2_core::EncoderParameters {
  let mut safe = openjp2_core::EncoderParameters {
    tile_size_on: parameters.tile_size_on,
    cp_tx0: parameters.cp_tx0,
    cp_ty0: parameters.cp_ty0,
    cp_tdx: parameters.cp_tdx,
    cp_tdy: parameters.cp_tdy,
    cp_disto_alloc: parameters.cp_disto_alloc,
    cp_fixed_alloc: parameters.cp_fixed_alloc,
    cp_fixed_quality: parameters.cp_fixed_quality,
    csty: parameters.csty,
    prog_order: parameters.prog_order,
    pocs: parameters.POC,
    numpocs: parameters.numpocs,
    tcp_numlayers: parameters.tcp_numlayers,
    tcp_rates: parameters.tcp_rates,
    tcp_distoratio: parameters.tcp_distoratio,
    numresolution: parameters.numresolution,
    cblockw_init: parameters.cblockw_init,
    cblockh_init: parameters.cblockh_init,
    mode: parameters.mode,
    irreversible: parameters.irreversible,
    roi_compno: parameters.roi_compno,
    roi_shift: parameters.roi_shift,
    res_spec: parameters.res_spec,
    prcw_init: parameters.prcw_init,
    prch_init: parameters.prch_init,
    index_on: parameters.index_on,
    image_offset_x0: parameters.image_offset_x0,
    image_offset_y0: parameters.image_offset_y0,
    subsampling_dx: parameters.subsampling_dx,
    subsampling_dy: parameters.subsampling_dy,
    decod_format: parameters.decod_format,
    cod_format: parameters.cod_format,
    cp_cinema: parameters.cp_cinema,
    max_comp_size: parameters.max_comp_size,
    cp_rsiz: parameters.cp_rsiz,
    tp_on: parameters.tp_on,
    tp_flag: parameters.tp_flag,
    tcp_mct: parameters.tcp_mct,
    jpip_on: parameters.jpip_on,
    max_cs_size: parameters.max_cs_size,
    rsiz: parameters.rsiz,
    ..Default::default()
  };
  safe.cp_matrice = if parameters.cp_matrice.is_null() {
    None
  } else {
    let array_len = (parameters.tcp_numlayers as usize)
      .wrapping_mul(parameters.numresolution as usize)
      .wrapping_mul(3);
    // SAFETY: cp_matrice is the caller's matrix of `array_len` ints.
    Some(unsafe { core::slice::from_raw_parts(parameters.cp_matrice, array_len) }.to_vec())
  };
  safe.cp_comment = if parameters.cp_comment.is_null() {
    None
  } else {
    // SAFETY: cp_comment is a caller-provided NUL-terminated string.
    Some(unsafe { core::ffi::CStr::from_ptr(parameters.cp_comment) }.to_bytes().to_vec())
  };
  safe.mct_data = if parameters.mct_data.is_null() {
    None
  } else {
    let l_mct_len = (numcomps * numcomps) as usize;
    // SAFETY: mct_data holds l_mct_len floats followed by numcomps ints.
    unsafe {
      let matrix: Vec<f32> =
        core::slice::from_raw_parts(parameters.mct_data as *const f32, l_mct_len).to_vec();
      let dc_shift: Vec<i32> = core::slice::from_raw_parts(
        (parameters.mct_data as *mut u8).add(l_mct_len * core::mem::size_of::<f32>()) as *const i32,
        numcomps as usize,
      )
      .to_vec();
      Some((matrix, dc_shift))
    }
  };
  safe
}

/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe fn opj_encoder_set_extra_options(
  p_codec: *mut opj_codec_t,
  options: *const *const core::ffi::c_char,
) -> OPJ_BOOL {ffi_guard(|| {
  if options.is_null() {
    return 1;
  }
  if p_codec.is_null() {
    return 0;
  }
  let l_codec = &mut *(p_codec as *mut Codec);
  let mut p_option_iter = options;
  let mut options = Vec::new();
  while !(*p_option_iter).is_null() {
    match core::ffi::CStr::from_ptr(*p_option_iter).to_str() {
      Ok(option) => {
        options.push(option);
      }
      Err(err) => {
        log::error!("Failed to convert C string to Rust string: {err}");
        return 0;
      }
    }
    p_option_iter = p_option_iter.offset(1);
  }
  l_codec.encoder_set_extra_options(options.as_slice()) as _
})}

/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe fn opj_start_compress(
  p_codec: *mut opj_codec_t,
  p_image: *mut opj_image_t,
  p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() | p_stream.is_null() | p_image.is_null() {
    return 0i32;
  }
  let p_image = unsafe { &mut *p_image };
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut Codec);
  let mut core = core_from_opj(p_image);
  let ret = l_codec.start_compress(&mut core, p_stream);
  let fresh = core_into_opj(core);
  p_image.clear_comps();
  p_image.clear_icc_profile();
  *p_image = *fresh;
  ret
})}

#[no_mangle]
pub unsafe fn opj_encode(
  p_codec: *mut opj_codec_t,
  p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() | p_stream.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut Codec);
  l_codec.encode(p_stream)
})}

#[no_mangle]
pub unsafe fn opj_end_compress(
  p_codec: *mut opj_codec_t,
  p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() | p_stream.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut Codec);
  l_codec.end_compress(p_stream)
})}

#[no_mangle]
pub unsafe fn opj_end_decompress(
  p_codec: *mut opj_codec_t,
  p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() | p_stream.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut Codec);
  l_codec.end_decompress(p_stream)
})}

#[no_mangle]
pub unsafe fn opj_set_MCT(
  parameters: *mut opj_cparameters_t,
  pEncodingMatrix: *mut OPJ_FLOAT32,
  p_dc_shift: *mut OPJ_INT32,
  nb_comps: OPJ_UINT32,
) -> OPJ_BOOL {ffi_guard(|| {
  let l_matrix_size = nb_comps * nb_comps;
  let l_dc_shift_size = nb_comps;
  let encoding_matrix =
    unsafe { core::slice::from_raw_parts(pEncodingMatrix, l_matrix_size as usize) };
  let dc_shift = unsafe { core::slice::from_raw_parts(p_dc_shift, l_dc_shift_size as usize) };
  let parameters = &mut *parameters;
  parameters.set_MCT(encoding_matrix, dc_shift, nb_comps) as _
})}

#[no_mangle]
pub unsafe fn opj_write_tile(
  p_codec: *mut opj_codec_t,
  p_tile_index: OPJ_UINT32,
  p_data: *mut OPJ_BYTE,
  p_data_size: OPJ_UINT32,
  p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {ffi_guard(|| {
  if p_codec.is_null() | p_stream.is_null() | p_data.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut Codec);
  let p_data = unsafe { core::slice::from_raw_parts(p_data as *const u8, p_data_size as usize) };
  l_codec.write_tile(p_tile_index, p_data, p_stream)
})}

/* ---------------------------------------------------------------------- */
#[no_mangle]
pub unsafe fn opj_destroy_codec(p_codec: *mut opj_codec_t) {ffi_guard(|| {
  if p_codec.is_null() {
    return;
  }
  let _ = Box::from_raw(p_codec as *mut Codec);
})}

/* ---------------------------------------------------------------------- */
#[cfg(feature = "file-io")]
#[no_mangle]
pub unsafe fn opj_dump_codec(
  p_codec: *mut opj_codec_t,
  info_flag: OPJ_INT32,
  output_stream: *mut FILE,
) {ffi_guard(|| {
  if p_codec.is_null() {
    return;
  }
  let l_codec = &mut *(p_codec as *mut Codec);
  if output_stream.is_null() {
    return;
  }
  let mut writer = CFile(output_stream);
  l_codec.dump_codec(info_flag, &mut writer)
})}

/// A `Write` adapter over the caller's `FILE*` for opj_dump_codec.
#[cfg(feature = "file-io")]
struct CFile(*mut FILE);

#[cfg(feature = "file-io")]
impl std::io::Write for CFile {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    // SAFETY: the FILE* is the caller's open output stream.
    let n = unsafe { libc::fwrite(buf.as_ptr() as *const libc::c_void, 1, buf.len(), self.0) };
    if n == libc::size_t::MAX {
      Ok(0)
    } else {
      Ok(n)
    }
  }

  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "file-io")]
impl openjp2_core::StringWriter for CFile {
  fn write_string(&mut self, s: &str) -> usize {
    use std::io::Write;
    self.write(s.as_bytes()).unwrap_or(0)
  }
}

#[no_mangle]
pub unsafe fn opj_get_cstr_info(p_codec: *mut opj_codec_t) -> *mut opj_codestream_info_v2_t {ffi_guard(|| {
  if p_codec.is_null() {
    return core::ptr::null_mut::<opj_codestream_info_v2_t>();
  }
  let l_codec = &*(p_codec as *mut Codec);
  match l_codec.get_cstr_info_v2() {
    Some(info) => crate::cstr_info::build_cstr_info_v2(&info),
    None => core::ptr::null_mut::<opj_codestream_info_v2_t>(),
  }
})}

#[no_mangle]
pub unsafe fn opj_destroy_cstr_info(cstr_info: *mut *mut opj_codestream_info_v2_t) {ffi_guard(|| {
  crate::cstr_info::destroy_cstr_info_v2(cstr_info);
})}

#[no_mangle]
pub unsafe fn opj_get_cstr_index(p_codec: *mut opj_codec_t) -> *mut opj_codestream_index_t {ffi_guard(|| {
  if p_codec.is_null() {
    return core::ptr::null_mut::<opj_codestream_index_t>();
  }
  let l_codec = &*(p_codec as *mut Codec);
  match l_codec.get_cstr_index_owned() {
    Some(index) => crate::cstr_info::build_cstr_index(&index),
    None => core::ptr::null_mut::<opj_codestream_index_t>(),
  }
})}

#[no_mangle]
pub unsafe fn opj_destroy_cstr_index(p_cstr_index: *mut *mut opj_codestream_index_t) {ffi_guard(|| {
  if p_cstr_index.is_null() {
    return;
  }
  crate::cstr_info::destroy_cstr_index(*p_cstr_index);
  *p_cstr_index = core::ptr::null_mut::<opj_codestream_index_t>();
})}

/* ----------------------------------------------------------------------- */
/* C stream objects (Phase 5.e: the C-callback adapter lives here; the    */
/* core Stream only knows about Read/Write/Seek sources).                 */

use std::cell::RefCell;
use std::io::{self as io, Error as IoError, Read, Seek, Write};
use std::rc::Rc;

/// Shared, mutable C callback set for a C-created stream.
#[derive(Default)]
pub struct CustomCallbacks {
  pub m_user_data: *mut core::ffi::c_void,
  pub m_free_user_data_fn: opj_stream_free_user_data_fn,
  pub m_read_fn: opj_stream_read_fn,
  pub m_write_fn: opj_stream_write_fn,
  pub m_skip_fn: opj_stream_skip_fn,
  pub m_seek_fn: opj_stream_seek_fn,
}

/// Forwards core Stream I/O to the C callbacks held in a shared handle, so
/// the opj_stream_set_* functions can mutate them after creation.
pub struct CustomStream {
  callbacks: Rc<RefCell<CustomCallbacks>>,
  m_byte_offset: OPJ_OFF_T,
}

impl CustomStream {
  pub fn has_seek(&self) -> bool {
    self.callbacks.borrow().m_seek_fn.is_some()
  }
}

impl Read for CustomStream {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let callbacks = self.callbacks.borrow();
    let res = if let Some(read) = callbacks.m_read_fn {
      unsafe {
        read(
          buf.as_mut_ptr() as *mut core::ffi::c_void,
          buf.len(),
          callbacks.m_user_data,
        ) as isize
      }
    } else {
      return Err(IoError::other("Custom stream doesn't have a read function"));
    };
    drop(callbacks);
    if res >= 0 {
      self.m_byte_offset += res as i64;
      Ok(res as usize)
    } else {
      Err(IoError::other("read failed"))
    }
  }
}

impl Write for CustomStream {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let callbacks = self.callbacks.borrow();
    let res = if let Some(write) = callbacks.m_write_fn {
      unsafe {
        write(
          buf.as_ptr() as *mut core::ffi::c_void,
          buf.len(),
          callbacks.m_user_data,
        ) as isize
      }
    } else {
      return Err(IoError::other(
        "Custom stream doesn't have a write function",
      ));
    };
    drop(callbacks);
    if res >= 0 {
      self.m_byte_offset += res as i64;
      Ok(res as usize)
    } else {
      Err(IoError::other("write failed"))
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

impl Seek for CustomStream {
  fn seek(&mut self, pos: std::io::SeekFrom) -> io::Result<u64> {
    log::trace!("-- CustomStream.seek({pos:?})");
    match pos {
      std::io::SeekFrom::Start(offset) => {
        let callbacks = self.callbacks.borrow();
        let res = if let Some(seek) = callbacks.m_seek_fn {
          unsafe { seek(offset as i64, callbacks.m_user_data) }
        } else {
          return Err(IoError::other(
            "Custom stream doesn't have a seek function",
          ));
        };
        drop(callbacks);
        if res != 0 {
          self.m_byte_offset = offset as i64;
          Ok(offset)
        } else {
          Err(IoError::other("seek failed"))
        }
      }
      std::io::SeekFrom::Current(offset) => {
        let callbacks = self.callbacks.borrow();
        let res = if let Some(skip) = callbacks.m_skip_fn {
          unsafe { skip(offset, callbacks.m_user_data) }
        } else {
          return Err(IoError::other(
            "Custom stream doesn't have a skip function",
          ));
        };
        drop(callbacks);
        if res != -1 {
          self.m_byte_offset += offset;
          Ok(self.m_byte_offset as u64)
        } else {
          Err(IoError::other("skip failed"))
        }
      }
      std::io::SeekFrom::End(_offset) => Err(IoError::other("Unsupported seek from end")),
    }
  }
}

/// A C-visible stream: the core `Stream` plus (for C-callback streams) the
/// shared callback handle the opj_stream_set_* functions mutate.
pub struct OpjStream {
  stream: openjp2_core::Stream,
  callbacks: Option<Rc<RefCell<CustomCallbacks>>>,
  is_input: bool,
}

impl Drop for OpjStream {
  fn drop(&mut self) {
    if let Some(callbacks) = &self.callbacks {
      let callbacks = callbacks.borrow();
      if let Some(free) = callbacks.m_free_user_data_fn {
        unsafe {
          free(callbacks.m_user_data);
        }
      }
    }
  }
}

fn new_custom_stream(buffer_size: OPJ_SIZE_T, is_input: bool) -> Box<OpjStream> {
  let callbacks = Rc::new(RefCell::new(CustomCallbacks::default()));
  let custom = CustomStream {
    callbacks: callbacks.clone(),
    m_byte_offset: 0,
  };
  let stream = if is_input {
    openjp2_core::Stream::new_input(buffer_size, custom, 0)
  } else {
    openjp2_core::Stream::new_output(buffer_size, custom)
  };
  Box::new(OpjStream {
    stream,
    callbacks: Some(callbacks),
    is_input,
  })
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_create(
  p_buffer_size: OPJ_SIZE_T,
  l_is_input: OPJ_BOOL,
) -> *mut opj_stream_t {ffi_guard(|| {
  let p_stream = Box::into_raw(new_custom_stream(p_buffer_size, l_is_input != 0)) as *mut opj_stream_t;
  log::trace!("-- create stream: {:?}", p_stream);
  p_stream
})}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_default_create(l_is_input: OPJ_BOOL) -> *mut opj_stream_t {ffi_guard(|| {
  opj_stream_create(openjp2_ctypes::consts::opj::OPJ_J2K_STREAM_CHUNK_SIZE as OPJ_SIZE_T, l_is_input)
})}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_destroy(p_stream: *mut opj_stream_t) {ffi_guard(|| {
  if !p_stream.is_null() {
    let _ = Box::from_raw(p_stream as *mut OpjStream);
  }
})}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_read_function(
  p_stream: *mut opj_stream_t,
  p_function: opj_stream_read_fn,
) {ffi_guard(|| {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut OpjStream) };
  if !p_stream.is_input {
    return;
  }
  if let Some(callbacks) = &p_stream.callbacks {
    callbacks.borrow_mut().m_read_fn = p_function;
  }
})}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_seek_function(
  p_stream: *mut opj_stream_t,
  p_function: opj_stream_seek_fn,
) {ffi_guard(|| {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut OpjStream) };
  if let Some(callbacks) = &p_stream.callbacks {
    callbacks.borrow_mut().m_seek_fn = p_function;
  }
})}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_write_function(
  p_stream: *mut opj_stream_t,
  p_function: opj_stream_write_fn,
) {ffi_guard(|| {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut OpjStream) };
  if p_stream.is_input {
    return;
  }
  if let Some(callbacks) = &p_stream.callbacks {
    callbacks.borrow_mut().m_write_fn = p_function;
  }
})}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_skip_function(
  p_stream: *mut opj_stream_t,
  p_function: opj_stream_skip_fn,
) {ffi_guard(|| {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut OpjStream) };
  if let Some(callbacks) = &p_stream.callbacks {
    callbacks.borrow_mut().m_skip_fn = p_function;
  }
})}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_user_data(
  p_stream: *mut opj_stream_t,
  p_data: *mut core::ffi::c_void,
  p_function: opj_stream_free_user_data_fn,
) {ffi_guard(|| {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut OpjStream) };
  if let Some(callbacks) = &p_stream.callbacks {
    let mut callbacks = callbacks.borrow_mut();
    callbacks.m_user_data = p_data;
    callbacks.m_free_user_data_fn = p_function;
  }
})}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_user_data_length(
  p_stream: *mut opj_stream_t,
  data_length: OPJ_UINT64,
) {ffi_guard(|| {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut OpjStream) };
  p_stream.stream.set_stream_length(data_length);
})}

#[cfg(feature = "file-io")]
#[no_mangle]
pub unsafe fn opj_stream_create_default_file_stream(
  fname: *const core::ffi::c_char,
  p_is_read_stream: OPJ_BOOL,
) -> *mut opj_stream_t {ffi_guard(|| {
  opj_stream_create_file_stream(fname, 0x100000 as OPJ_SIZE_T, p_is_read_stream)
})}

#[cfg(feature = "file-io")]
#[no_mangle]
pub unsafe fn opj_stream_create_file_stream(
  fname: *const core::ffi::c_char,
  p_size: OPJ_SIZE_T,
  p_is_read_stream: OPJ_BOOL,
) -> *mut opj_stream_t {ffi_guard(|| {
  if fname.is_null() {
    return core::ptr::null_mut::<opj_stream_t>();
  }
  match core::ffi::CStr::from_ptr(fname).to_str() {
    Ok(name) => {
      let is_input = p_is_read_stream != 0;
      let stream = match openjp2_core::Stream::new_file(name, p_size, is_input) {
        Ok(stream) => stream,
        Err(err) => {
          log::error!("Failed to open file stream: {err}");
          return core::ptr::null_mut::<opj_stream_t>();
        }
      };
      Box::into_raw(Box::new(OpjStream {
        stream,
        callbacks: None,
        is_input,
      })) as *mut opj_stream_t
    }
    Err(err) => {
      log::error!("Failed to convert C filename to Rust String: {err}");
      core::ptr::null_mut::<opj_stream_t>()
    }
  }
})}

#[no_mangle]
pub unsafe fn opj_image_data_alloc(size: OPJ_SIZE_T) -> *mut core::ffi::c_void {ffi_guard(|| {
  crate::image::opj_image_data_alloc(size)
})}

#[no_mangle]
pub unsafe fn opj_image_data_free(ptr: *mut core::ffi::c_void) {ffi_guard(|| {
  crate::image::opj_image_data_free(ptr);
})}

#[no_mangle]
pub fn opj_has_thread_support() -> OPJ_BOOL {ffi_guard(|| {
  openjp2_core::has_thread_support()
})}

#[no_mangle]
pub fn opj_get_num_cpus() -> core::ffi::c_int {ffi_guard(|| {
  std::thread::available_parallelism()
    .map(|n| n.get() as core::ffi::c_int)
    .unwrap_or(1i32)
})}

/* ----------------------------------------------------------------------- */
/* Image object exports (moved from the core crate in Phase 6)             */

#[no_mangle]
pub fn opj_image_create(
  numcmpts: OPJ_UINT32,
  cmptparms: *mut opj_image_cmptparm_t,
  clrspc: OPJ_COLOR_SPACE,
) -> *mut opj_image_t {ffi_guard(|| {
  crate::image::opj_image_create(numcmpts, cmptparms, clrspc)
})}

#[no_mangle]
pub fn opj_image_destroy(image: *mut opj_image_t) {ffi_guard(|| {
  crate::image::opj_image_destroy(image);
})}

#[no_mangle]
pub fn opj_image_tile_create(
  numcmpts: OPJ_UINT32,
  cmptparms: *mut opj_image_cmptparm_t,
  clrspc: OPJ_COLOR_SPACE,
) -> *mut opj_image_t {ffi_guard(|| {
  crate::image::opj_image_tile_create(numcmpts, cmptparms, clrspc)
})}
