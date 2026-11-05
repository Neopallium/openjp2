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

use openjp2_core::c_api_types::*;
use openjp2_core::consts::opj::*;

use openjp2_core::{free_cstr_index_mirror, opj_codec_private_t, opj_stream_private_t, Codec, Stream, TileInfo};

use ::libc::FILE;

/* ---------------------------------------------------------------------- */
/* Functions to set the message handlers */

/// The C `opj_msg_callback` type from openjpeg.h, declared here on the ABI side: the core only
/// sees the Rust closures these functions wrap around it.
pub type opj_msg_callback_fn =
  unsafe extern "C" fn(_: *const core::ffi::c_char, _: *mut core::ffi::c_void) -> ();
pub type opj_msg_callback = Option<opj_msg_callback_fn>;

/// Wrap a C callback + user data in the core's `FnMut(&str)` sink.
fn wrap_callback(callback: opj_msg_callback, user_data: *mut core::ffi::c_void) -> impl FnMut(&str) {
  move |msg: &str| {
    let Some(callback) = callback else {
      return;
    };
    if let Ok(c_msg) = std::ffi::CString::new(msg) {
      // SAFETY: the callback is the one the caller installed, invoked with the caller's data.
      unsafe { callback(c_msg.as_ptr(), user_data) };
    }
  }
}

#[no_mangle]
pub unsafe fn opj_set_info_handler(
  p_codec: *mut opj_codec_t,
  p_callback: opj_msg_callback,
  p_user_data: *mut core::ffi::c_void,
) -> OPJ_BOOL {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.set_info_handler(wrap_callback(p_callback, p_user_data))
}

#[no_mangle]
pub unsafe fn opj_set_warning_handler(
  p_codec: *mut opj_codec_t,
  p_callback: opj_msg_callback,
  p_user_data: *mut core::ffi::c_void,
) -> OPJ_BOOL {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.set_warning_handler(wrap_callback(p_callback, p_user_data))
}

#[no_mangle]
pub unsafe fn opj_set_error_handler(
  p_codec: *mut opj_codec_t,
  p_callback: opj_msg_callback,
  p_user_data: *mut core::ffi::c_void,
) -> OPJ_BOOL {
  if p_codec.is_null() {
    return 0i32;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.set_error_handler(wrap_callback(p_callback, p_user_data))
}

/* ---------------------------------------------------------------------- */
pub const OPJ_VERSION: &str = "2.5.2";
pub const OPJ_VERSION_C: *const u8 = b"2.5.2\x00" as *const u8;

#[no_mangle]
pub fn opj_version() -> *const core::ffi::c_char {
  OPJ_VERSION_C as *const core::ffi::c_char
}

/* ---------------------------------------------------------------------- */
/* DECOMPRESSION FUNCTIONS*/
#[no_mangle]
pub fn opj_create_decompress(mut p_format: OPJ_CODEC_FORMAT) -> *mut opj_codec_t {
  if let Some(codec) = opj_codec_private_t::new_decoder(p_format) {
    let l_codec = Box::new(codec);
    Box::into_raw(l_codec) as *mut opj_codec_t
  } else {
    core::ptr::null_mut()
  }
}

#[no_mangle]
pub unsafe fn opj_set_default_decoder_parameters(mut parameters: *mut opj_dparameters_t) {
  if parameters.is_null() {
    return;
  }
  let parameters = &mut *parameters;
  parameters.set_defaults();
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
  match l_codec.read_header(p_stream) {
    Some(image) => {
      *p_image = Box::into_raw(image);
      1
    }
    None => 0,
  }
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
  let components = unsafe { core::slice::from_raw_parts(comps_indices, numcomps as usize) };
  l_codec.set_decoded_components(components, apply_color_transforms)
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
  let mut tile_info = TileInfo::default();
  if !p_data_size.is_null() {
    // Request the tile data size.
    tile_info.data_size = Some(0);
  }
  if l_codec.read_tile_header(p_stream, &mut tile_info) {
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
  } else {
    0
  }
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
  let p_data = if p_data.is_null() {
    None
  } else {
    Some(unsafe { core::slice::from_raw_parts_mut(p_data as *mut u8, p_data_size as usize) })
  };
  l_codec.decode_tile_data(p_stream, p_tile_index, p_data)
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
    core::ptr::null_mut()
  }
}

/* default coding parameters */
#[no_mangle]
pub unsafe fn opj_set_default_encoder_parameters(mut parameters: *mut opj_cparameters_t) {
  if parameters.is_null() {
    return;
  }
  unsafe {
    parameters.write_bytes(0, 1);
  }
  let parameters = &mut *parameters;
  parameters.set_defaults();
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
  if options.is_null() {
    return 1;
  }
  if p_codec.is_null() {
    return 0;
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
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
  mut nb_comps: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut l_matrix_size = nb_comps * nb_comps;
  let mut l_dc_shift_size = nb_comps;
  let encoding_matrix =
    unsafe { core::slice::from_raw_parts(pEncodingMatrix, l_matrix_size as usize) };
  let dc_shift = unsafe { core::slice::from_raw_parts(p_dc_shift, l_dc_shift_size as usize) };
  let parameters = &mut *parameters;
  parameters.set_MCT(encoding_matrix, dc_shift, nb_comps) as _
}

#[no_mangle]
pub unsafe fn opj_write_tile(
  mut p_codec: *mut opj_codec_t,
  mut p_tile_index: OPJ_UINT32,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_size: OPJ_UINT32,
  mut p_stream: *mut opj_stream_t,
) -> OPJ_BOOL {
  if p_codec.is_null() | p_stream.is_null() | p_data.is_null() {
    return 0i32;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut opj_stream_private_t) };
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  let p_data = unsafe { core::slice::from_raw_parts(p_data as *const u8, p_data_size as usize) };
  l_codec.write_tile(p_tile_index, p_data, p_stream)
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
  if output_stream.is_null() {
    return;
  }
  l_codec.dump_codec(info_flag, &mut *output_stream)
}

#[no_mangle]
pub unsafe fn opj_get_cstr_info(mut p_codec: *mut opj_codec_t) -> *mut opj_codestream_info_v2_t {
  if p_codec.is_null() {
    return core::ptr::null_mut::<opj_codestream_info_v2_t>();
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.get_cstr_info()
}

#[no_mangle]
pub unsafe fn opj_destroy_cstr_info(mut cstr_info: *mut *mut opj_codestream_info_v2_t) {
  openjp2_core::opj_destroy_cstr_info(cstr_info);
}

#[no_mangle]
pub unsafe fn opj_get_cstr_index(mut p_codec: *mut opj_codec_t) -> *mut opj_codestream_index_t {
  if p_codec.is_null() {
    return core::ptr::null_mut::<opj_codestream_index_t>();
  }
  let l_codec = &mut *(p_codec as *mut opj_codec_private_t);
  l_codec.get_cstr_index()
}

#[no_mangle]
pub unsafe fn opj_destroy_cstr_index(mut p_cstr_index: *mut *mut opj_codestream_index_t) {
  if !(*p_cstr_index).is_null() {
    // TODO: use drop.
    free_cstr_index_mirror(*p_cstr_index);
    *p_cstr_index = core::ptr::null_mut::<opj_codestream_index_t>()
  };
}

/* ----------------------------------------------------------------------- */
/* C stream objects (Phase 5.e: the C-callback adapter lives here; the    */
/* core Stream only knows about Read/Write/Seek sources).                 */

use std::cell::RefCell;
use std::io::{self as io, Error as IoError, Read, Seek, SeekFrom, Write};
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
    openjp2_core::Stream::new_input(buffer_size as usize, custom, 0)
  } else {
    openjp2_core::Stream::new_output(buffer_size as usize, custom)
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
) -> *mut opj_stream_t {
  let p_stream = Box::into_raw(new_custom_stream(p_buffer_size, l_is_input != 0)) as *mut opj_stream_t;
  log::trace!("-- create stream: {:?}", p_stream);
  p_stream
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_default_create(l_is_input: OPJ_BOOL) -> *mut opj_stream_t {
  opj_stream_create(0x100000 as OPJ_SIZE_T, l_is_input)
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_destroy(p_stream: *mut opj_stream_t) {
  if !p_stream.is_null() {
    let _ = Box::from_raw(p_stream as *mut OpjStream);
  }
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_read_function(
  p_stream: *mut opj_stream_t,
  p_function: opj_stream_read_fn,
) {
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
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_seek_function(
  p_stream: *mut opj_stream_t,
  p_function: opj_stream_seek_fn,
) {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut OpjStream) };
  if let Some(callbacks) = &p_stream.callbacks {
    callbacks.borrow_mut().m_seek_fn = p_function;
  }
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_write_function(
  p_stream: *mut opj_stream_t,
  p_function: opj_stream_write_fn,
) {
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
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_skip_function(
  p_stream: *mut opj_stream_t,
  p_function: opj_stream_skip_fn,
) {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut OpjStream) };
  if let Some(callbacks) = &p_stream.callbacks {
    callbacks.borrow_mut().m_skip_fn = p_function;
  }
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_user_data(
  p_stream: *mut opj_stream_t,
  p_data: *mut core::ffi::c_void,
  p_function: opj_stream_free_user_data_fn,
) {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut OpjStream) };
  if let Some(callbacks) = &p_stream.callbacks {
    let mut callbacks = callbacks.borrow_mut();
    callbacks.m_user_data = p_data;
    callbacks.m_free_user_data_fn = p_function;
  }
}

#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_user_data_length(
  p_stream: *mut opj_stream_t,
  data_length: OPJ_UINT64,
) {
  if p_stream.is_null() {
    return;
  }
  let p_stream = unsafe { &mut *(p_stream as *mut OpjStream) };
  p_stream.stream.set_stream_length(data_length);
}

#[no_mangle]
pub unsafe fn opj_stream_create_default_file_stream(
  fname: *const core::ffi::c_char,
  p_is_read_stream: OPJ_BOOL,
) -> *mut opj_stream_t {
  opj_stream_create_file_stream(fname, 0x100000 as OPJ_SIZE_T, p_is_read_stream)
}

#[no_mangle]
pub unsafe fn opj_stream_create_file_stream(
  fname: *const core::ffi::c_char,
  p_size: OPJ_SIZE_T,
  p_is_read_stream: OPJ_BOOL,
) -> *mut opj_stream_t {
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
      return core::ptr::null_mut::<opj_stream_t>();
    }
  }
}

#[no_mangle]
pub unsafe fn opj_image_data_alloc(size: OPJ_SIZE_T) -> *mut core::ffi::c_void {
  openjp2_core::opj_image_data_alloc(size)
}

#[no_mangle]
pub unsafe fn opj_image_data_free(ptr: *mut core::ffi::c_void) {
  openjp2_core::opj_image_data_free(ptr);
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

/* ----------------------------------------------------------------------- */
/* Image object exports (moved from the core crate in Phase 6)             */

#[no_mangle]
pub fn opj_image_create(
  numcmpts: OPJ_UINT32,
  cmptparms: *mut openjp2_core::image::opj_image_cmptparm_t,
  clrspc: OPJ_COLOR_SPACE,
) -> *mut opj_image_t {
  openjp2_core::opj_image_create(numcmpts, cmptparms, clrspc)
}

#[no_mangle]
pub fn opj_image_destroy(image: *mut opj_image_t) {
  openjp2_core::opj_image_destroy(image);
}

#[no_mangle]
pub fn opj_image_tile_create(
  numcmpts: OPJ_UINT32,
  cmptparms: *mut openjp2_core::image::opj_image_cmptparm_t,
  clrspc: OPJ_COLOR_SPACE,
) -> *mut opj_image_t {
  openjp2_core::opj_image_tile_create(numcmpts, cmptparms, clrspc)
}
