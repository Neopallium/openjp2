use ::libc;
extern "C" {

  fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;

  fn opj_event_msg(
    event_mgr: *mut opj_event_mgr_t,
    event_type: OPJ_INT32,
    fmt: *const libc::c_char,
    _: ...
  ) -> OPJ_BOOL;

  fn opj_free(m: *mut libc::c_void);

  fn opj_malloc(size: size_t) -> *mut libc::c_void;

  fn opj_calloc(numOfElements: size_t, sizeOfElements: size_t) -> *mut libc::c_void;
}
pub type size_t = libc::c_ulong;
pub type __int32_t = libc::c_int;
pub type __uint32_t = libc::c_uint;
pub type __int64_t = libc::c_long;
pub type __uint64_t = libc::c_ulong;
pub type OPJ_BOOL = libc::c_int;
pub type OPJ_FLOAT32 = libc::c_float;
pub type OPJ_FLOAT64 = libc::c_double;
pub type OPJ_BYTE = libc::c_uchar;
pub type int32_t = __int32_t;
pub type int64_t = __int64_t;
pub type uint32_t = __uint32_t;
pub type uint64_t = __uint64_t;
pub type OPJ_INT32 = int32_t;
pub type OPJ_UINT32 = uint32_t;
pub type OPJ_UINT64 = uint64_t;
pub type OPJ_OFF_T = int64_t;
pub type OPJ_SIZE_T = size_t;
pub type opj_msg_callback =
  Option<unsafe extern "C" fn(_: *const libc::c_char, _: *mut libc::c_void) -> ()>;
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
pub type opj_stream_private_t = opj_stream_private;

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
pub struct opj_event_mgr {
  pub m_error_data: *mut libc::c_void,
  pub m_warning_data: *mut libc::c_void,
  pub m_info_data: *mut libc::c_void,
  pub error_handler: opj_msg_callback,
  pub warning_handler: opj_msg_callback,
  pub info_handler: opj_msg_callback,
}
pub type opj_event_mgr_t = opj_event_mgr;
/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2002-2014, Universite catholique de Louvain (UCL), Belgium
 * Copyright (c) 2002-2014, Professor Benoit Macq
 * Copyright (c) 2001-2003, David Janssens
 * Copyright (c) 2002-2003, Yannick Verschueren
 * Copyright (c) 2003-2007, Francois-Olivier Devaux
 * Copyright (c) 2003-2014, Antonin Descampe
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
/* ----------------------------------------------------------------------- */
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe extern "C" fn opj_write_bytes_BE(
  mut p_buffer: *mut OPJ_BYTE,
  mut p_value: OPJ_UINT32,
  mut p_nb_bytes: OPJ_UINT32,
) {
  let mut l_data_ptr = (&mut p_value as *mut OPJ_UINT32 as *const OPJ_BYTE)
    .offset(::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong as isize)
    .offset(-(p_nb_bytes as isize));
  assert!(
    p_nb_bytes > 0 as libc::c_int as libc::c_uint
      && p_nb_bytes as libc::c_ulong <= ::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong
  );
  memcpy(
    p_buffer as *mut libc::c_void,
    l_data_ptr as *const libc::c_void,
    p_nb_bytes as libc::c_ulong,
  );
}
#[no_mangle]
pub unsafe extern "C" fn opj_write_bytes_LE(
  mut p_buffer: *mut OPJ_BYTE,
  mut p_value: OPJ_UINT32,
  mut p_nb_bytes: OPJ_UINT32,
) {
  let mut l_data_ptr = (&mut p_value as *mut OPJ_UINT32 as *const OPJ_BYTE)
    .offset(p_nb_bytes as isize)
    .offset(-(1 as libc::c_int as isize));
  let mut i: OPJ_UINT32 = 0;
  assert!(
    p_nb_bytes > 0 as libc::c_int as libc::c_uint
      && p_nb_bytes as libc::c_ulong <= ::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong
  );
  i = 0 as libc::c_int as OPJ_UINT32;
  while i < p_nb_bytes {
    let fresh0 = l_data_ptr;
    l_data_ptr = l_data_ptr.offset(-1);
    let fresh1 = p_buffer;
    p_buffer = p_buffer.offset(1);
    *fresh1 = *fresh0;
    i = i.wrapping_add(1)
  }
}
#[no_mangle]
pub unsafe extern "C" fn opj_read_bytes_BE(
  mut p_buffer: *const OPJ_BYTE,
  mut p_value: *mut OPJ_UINT32,
  mut p_nb_bytes: OPJ_UINT32,
) {
  let mut l_data_ptr = p_value as *mut OPJ_BYTE;
  assert!(
    p_nb_bytes > 0 as libc::c_int as libc::c_uint
      && p_nb_bytes as libc::c_ulong <= ::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong
  );
  *p_value = 0 as libc::c_int as OPJ_UINT32;
  memcpy(
    l_data_ptr
      .offset(::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong as isize)
      .offset(-(p_nb_bytes as isize)) as *mut libc::c_void,
    p_buffer as *const libc::c_void,
    p_nb_bytes as libc::c_ulong,
  );
}
#[no_mangle]
pub unsafe extern "C" fn opj_read_bytes_LE(
  mut p_buffer: *const OPJ_BYTE,
  mut p_value: *mut OPJ_UINT32,
  mut p_nb_bytes: OPJ_UINT32,
) {
  let mut l_data_ptr = (p_value as *mut OPJ_BYTE)
    .offset(p_nb_bytes as isize)
    .offset(-(1 as libc::c_int as isize));
  let mut i: OPJ_UINT32 = 0;
  assert!(
    p_nb_bytes > 0 as libc::c_int as libc::c_uint
      && p_nb_bytes as libc::c_ulong <= ::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong
  );
  *p_value = 0 as libc::c_int as OPJ_UINT32;
  i = 0 as libc::c_int as OPJ_UINT32;
  while i < p_nb_bytes {
    let fresh2 = p_buffer;
    p_buffer = p_buffer.offset(1);
    let fresh3 = l_data_ptr;
    l_data_ptr = l_data_ptr.offset(-1);
    *fresh3 = *fresh2;
    i = i.wrapping_add(1)
  }
}
#[no_mangle]
pub unsafe extern "C" fn opj_write_double_BE(
  mut p_buffer: *mut OPJ_BYTE,
  mut p_value: OPJ_FLOAT64,
) {
  let mut l_data_ptr = &mut p_value as *mut OPJ_FLOAT64 as *const OPJ_BYTE;
  memcpy(
    p_buffer as *mut libc::c_void,
    l_data_ptr as *const libc::c_void,
    ::std::mem::size_of::<OPJ_FLOAT64>() as libc::c_ulong,
  );
}
#[no_mangle]
pub unsafe extern "C" fn opj_write_double_LE(
  mut p_buffer: *mut OPJ_BYTE,
  mut p_value: OPJ_FLOAT64,
) {
  let mut l_data_ptr = (&mut p_value as *mut OPJ_FLOAT64 as *const OPJ_BYTE)
    .offset(::std::mem::size_of::<OPJ_FLOAT64>() as libc::c_ulong as isize)
    .offset(-(1 as libc::c_int as isize));
  let mut i: OPJ_UINT32 = 0;
  i = 0 as libc::c_int as OPJ_UINT32;
  while (i as libc::c_ulong) < ::std::mem::size_of::<OPJ_FLOAT64>() as libc::c_ulong {
    let fresh4 = l_data_ptr;
    l_data_ptr = l_data_ptr.offset(-1);
    let fresh5 = p_buffer;
    p_buffer = p_buffer.offset(1);
    *fresh5 = *fresh4;
    i = i.wrapping_add(1)
  }
}
#[no_mangle]
pub unsafe extern "C" fn opj_read_double_BE(
  mut p_buffer: *const OPJ_BYTE,
  mut p_value: *mut OPJ_FLOAT64,
) {
  let mut l_data_ptr = p_value as *mut OPJ_BYTE;
  memcpy(
    l_data_ptr as *mut libc::c_void,
    p_buffer as *const libc::c_void,
    ::std::mem::size_of::<OPJ_FLOAT64>() as libc::c_ulong,
  );
}
#[no_mangle]
pub unsafe extern "C" fn opj_read_double_LE(
  mut p_buffer: *const OPJ_BYTE,
  mut p_value: *mut OPJ_FLOAT64,
) {
  let mut l_data_ptr = (p_value as *mut OPJ_BYTE)
    .offset(::std::mem::size_of::<OPJ_FLOAT64>() as libc::c_ulong as isize)
    .offset(-(1 as libc::c_int as isize));
  let mut i: OPJ_UINT32 = 0;
  i = 0 as libc::c_int as OPJ_UINT32;
  while (i as libc::c_ulong) < ::std::mem::size_of::<OPJ_FLOAT64>() as libc::c_ulong {
    let fresh6 = p_buffer;
    p_buffer = p_buffer.offset(1);
    let fresh7 = l_data_ptr;
    l_data_ptr = l_data_ptr.offset(-1);
    *fresh7 = *fresh6;
    i = i.wrapping_add(1)
  }
}
#[no_mangle]
pub unsafe extern "C" fn opj_write_float_BE(mut p_buffer: *mut OPJ_BYTE, mut p_value: OPJ_FLOAT32) {
  let mut l_data_ptr = &mut p_value as *mut OPJ_FLOAT32 as *const OPJ_BYTE;
  memcpy(
    p_buffer as *mut libc::c_void,
    l_data_ptr as *const libc::c_void,
    ::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong,
  );
}
#[no_mangle]
pub unsafe extern "C" fn opj_write_float_LE(mut p_buffer: *mut OPJ_BYTE, mut p_value: OPJ_FLOAT32) {
  let mut l_data_ptr = (&mut p_value as *mut OPJ_FLOAT32 as *const OPJ_BYTE)
    .offset(::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong as isize)
    .offset(-(1 as libc::c_int as isize));
  let mut i: OPJ_UINT32 = 0;
  i = 0 as libc::c_int as OPJ_UINT32;
  while (i as libc::c_ulong) < ::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong {
    let fresh8 = l_data_ptr;
    l_data_ptr = l_data_ptr.offset(-1);
    let fresh9 = p_buffer;
    p_buffer = p_buffer.offset(1);
    *fresh9 = *fresh8;
    i = i.wrapping_add(1)
  }
}
#[no_mangle]
pub unsafe extern "C" fn opj_read_float_BE(
  mut p_buffer: *const OPJ_BYTE,
  mut p_value: *mut OPJ_FLOAT32,
) {
  let mut l_data_ptr = p_value as *mut OPJ_BYTE;
  memcpy(
    l_data_ptr as *mut libc::c_void,
    p_buffer as *const libc::c_void,
    ::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong,
  );
}
#[no_mangle]
pub unsafe extern "C" fn opj_read_float_LE(
  mut p_buffer: *const OPJ_BYTE,
  mut p_value: *mut OPJ_FLOAT32,
) {
  let mut l_data_ptr = (p_value as *mut OPJ_BYTE)
    .offset(::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong as isize)
    .offset(-(1 as libc::c_int as isize));
  let mut i: OPJ_UINT32 = 0;
  i = 0 as libc::c_int as OPJ_UINT32;
  while (i as libc::c_ulong) < ::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong {
    let fresh10 = p_buffer;
    p_buffer = p_buffer.offset(1);
    let fresh11 = l_data_ptr;
    l_data_ptr = l_data_ptr.offset(-1);
    *fresh11 = *fresh10;
    i = i.wrapping_add(1)
  }
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_create(
  mut p_buffer_size: OPJ_SIZE_T,
  mut l_is_input: OPJ_BOOL,
) -> *mut opj_stream_t {
  let mut l_stream = 0 as *mut opj_stream_private_t;
  l_stream = opj_calloc(
    1 as libc::c_int as size_t,
    ::std::mem::size_of::<opj_stream_private_t>() as libc::c_ulong,
  ) as *mut opj_stream_private_t;
  if l_stream.is_null() {
    return 0 as *mut opj_stream_t;
  }
  (*l_stream).m_buffer_size = p_buffer_size;
  (*l_stream).m_stored_data = opj_malloc(p_buffer_size) as *mut OPJ_BYTE;
  if (*l_stream).m_stored_data.is_null() {
    opj_free(l_stream as *mut libc::c_void);
    return 0 as *mut opj_stream_t;
  }
  (*l_stream).m_current_data = (*l_stream).m_stored_data;
  if l_is_input != 0 {
    (*l_stream).m_status |= 0x2 as libc::c_uint;
    (*l_stream).m_opj_skip = Some(
      opj_stream_read_skip
        as unsafe extern "C" fn(
          _: *mut opj_stream_private_t,
          _: OPJ_OFF_T,
          _: *mut opj_event_mgr,
        ) -> OPJ_OFF_T,
    );
    (*l_stream).m_opj_seek = Some(
      opj_stream_read_seek
        as unsafe extern "C" fn(
          _: *mut opj_stream_private_t,
          _: OPJ_OFF_T,
          _: *mut opj_event_mgr,
        ) -> OPJ_BOOL,
    )
  } else {
    (*l_stream).m_status |= 0x1 as libc::c_uint;
    (*l_stream).m_opj_skip = Some(
      opj_stream_write_skip
        as unsafe extern "C" fn(
          _: *mut opj_stream_private_t,
          _: OPJ_OFF_T,
          _: *mut opj_event_mgr,
        ) -> OPJ_OFF_T,
    );
    (*l_stream).m_opj_seek = Some(
      opj_stream_write_seek
        as unsafe extern "C" fn(
          _: *mut opj_stream_private_t,
          _: OPJ_OFF_T,
          _: *mut opj_event_mgr,
        ) -> OPJ_BOOL,
    )
  }
  (*l_stream).m_read_fn = Some(
    opj_stream_default_read
      as unsafe extern "C" fn(
        _: *mut libc::c_void,
        _: OPJ_SIZE_T,
        _: *mut libc::c_void,
      ) -> OPJ_SIZE_T,
  );
  (*l_stream).m_write_fn = Some(
    opj_stream_default_write
      as unsafe extern "C" fn(
        _: *mut libc::c_void,
        _: OPJ_SIZE_T,
        _: *mut libc::c_void,
      ) -> OPJ_SIZE_T,
  );
  (*l_stream).m_skip_fn = Some(
    opj_stream_default_skip
      as unsafe extern "C" fn(_: OPJ_OFF_T, _: *mut libc::c_void) -> OPJ_OFF_T,
  );
  (*l_stream).m_seek_fn = Some(
    opj_stream_default_seek as unsafe extern "C" fn(_: OPJ_OFF_T, _: *mut libc::c_void) -> OPJ_BOOL,
  );
  return l_stream as *mut opj_stream_t;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_default_create(mut l_is_input: OPJ_BOOL) -> *mut opj_stream_t {
  return opj_stream_create(0x100000 as libc::c_int as OPJ_SIZE_T, l_is_input);
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_destroy(mut p_stream: *mut opj_stream_t) {
  let mut l_stream = p_stream as *mut opj_stream_private_t;
  if !l_stream.is_null() {
    if (*l_stream).m_free_user_data_fn.is_some() {
      (*l_stream)
        .m_free_user_data_fn
        .expect("non-null function pointer")((*l_stream).m_user_data);
    }
    opj_free((*l_stream).m_stored_data as *mut libc::c_void);
    (*l_stream).m_stored_data = 0 as *mut OPJ_BYTE;
    opj_free(l_stream as *mut libc::c_void);
  };
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_read_function(
  mut p_stream: *mut opj_stream_t,
  mut p_function: opj_stream_read_fn,
) {
  let mut l_stream = p_stream as *mut opj_stream_private_t;
  if l_stream.is_null() || (*l_stream).m_status & 0x2 as libc::c_uint == 0 {
    return;
  }
  (*l_stream).m_read_fn = p_function;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_seek_function(
  mut p_stream: *mut opj_stream_t,
  mut p_function: opj_stream_seek_fn,
) {
  let mut l_stream = p_stream as *mut opj_stream_private_t;
  if l_stream.is_null() {
    return;
  }
  (*l_stream).m_seek_fn = p_function;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_write_function(
  mut p_stream: *mut opj_stream_t,
  mut p_function: opj_stream_write_fn,
) {
  let mut l_stream = p_stream as *mut opj_stream_private_t;
  if l_stream.is_null() || (*l_stream).m_status & 0x1 as libc::c_uint == 0 {
    return;
  }
  (*l_stream).m_write_fn = p_function;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_skip_function(
  mut p_stream: *mut opj_stream_t,
  mut p_function: opj_stream_skip_fn,
) {
  let mut l_stream = p_stream as *mut opj_stream_private_t;
  if l_stream.is_null() {
    return;
  }
  (*l_stream).m_skip_fn = p_function;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_user_data(
  mut p_stream: *mut opj_stream_t,
  mut p_data: *mut libc::c_void,
  mut p_function: opj_stream_free_user_data_fn,
) {
  let mut l_stream = p_stream as *mut opj_stream_private_t;
  if l_stream.is_null() {
    return;
  }
  (*l_stream).m_user_data = p_data;
  (*l_stream).m_free_user_data_fn = p_function;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_set_user_data_length(
  mut p_stream: *mut opj_stream_t,
  mut data_length: OPJ_UINT64,
) {
  let mut l_stream = p_stream as *mut opj_stream_private_t;
  if l_stream.is_null() {
    return;
  }
  (*l_stream).m_user_data_length = data_length;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_read_data(
  mut p_stream: *mut opj_stream_private_t,
  mut p_buffer: *mut OPJ_BYTE,
  mut p_size: OPJ_SIZE_T,
  mut p_event_mgr: *mut opj_event_mgr_t,
) -> OPJ_SIZE_T {
  let mut l_read_nb_bytes = 0 as libc::c_int as OPJ_SIZE_T;
  if (*p_stream).m_bytes_in_buffer >= p_size {
    memcpy(
      p_buffer as *mut libc::c_void,
      (*p_stream).m_current_data as *const libc::c_void,
      p_size,
    );
    (*p_stream).m_current_data = (*p_stream).m_current_data.offset(p_size as isize);
    (*p_stream).m_bytes_in_buffer = ((*p_stream).m_bytes_in_buffer as libc::c_ulong)
      .wrapping_sub(p_size) as OPJ_SIZE_T as OPJ_SIZE_T;
    l_read_nb_bytes =
      (l_read_nb_bytes as libc::c_ulong).wrapping_add(p_size) as OPJ_SIZE_T as OPJ_SIZE_T;
    (*p_stream).m_byte_offset += p_size as OPJ_OFF_T;
    return l_read_nb_bytes;
  }
  /* we are now in the case when the remaining data if not sufficient */
  if (*p_stream).m_status & 0x4 as libc::c_uint != 0 {
    l_read_nb_bytes = (l_read_nb_bytes as libc::c_ulong).wrapping_add((*p_stream).m_bytes_in_buffer)
      as OPJ_SIZE_T as OPJ_SIZE_T;
    memcpy(
      p_buffer as *mut libc::c_void,
      (*p_stream).m_current_data as *const libc::c_void,
      (*p_stream).m_bytes_in_buffer,
    );
    (*p_stream).m_current_data = (*p_stream)
      .m_current_data
      .offset((*p_stream).m_bytes_in_buffer as isize);
    (*p_stream).m_byte_offset += (*p_stream).m_bytes_in_buffer as OPJ_OFF_T;
    (*p_stream).m_bytes_in_buffer = 0 as libc::c_int as OPJ_SIZE_T;
    return if l_read_nb_bytes != 0 {
      l_read_nb_bytes
    } else {
      -(1 as libc::c_int) as OPJ_SIZE_T
    };
  }
  /* the flag is not set, we copy data and then do an actual read on the stream */
  if (*p_stream).m_bytes_in_buffer != 0 {
    l_read_nb_bytes = (l_read_nb_bytes as libc::c_ulong).wrapping_add((*p_stream).m_bytes_in_buffer)
      as OPJ_SIZE_T as OPJ_SIZE_T;
    memcpy(
      p_buffer as *mut libc::c_void,
      (*p_stream).m_current_data as *const libc::c_void,
      (*p_stream).m_bytes_in_buffer,
    );
    (*p_stream).m_current_data = (*p_stream).m_stored_data;
    p_buffer = p_buffer.offset((*p_stream).m_bytes_in_buffer as isize);
    p_size = (p_size as libc::c_ulong).wrapping_sub((*p_stream).m_bytes_in_buffer) as OPJ_SIZE_T
      as OPJ_SIZE_T;
    (*p_stream).m_byte_offset += (*p_stream).m_bytes_in_buffer as OPJ_OFF_T;
    (*p_stream).m_bytes_in_buffer = 0 as libc::c_int as OPJ_SIZE_T
  } else {
    /* case where we are already at the end of the buffer
    so reset the m_current_data to point to the start of the
    stored buffer to get ready to read from disk*/
    (*p_stream).m_current_data = (*p_stream).m_stored_data
  }
  loop {
    /* we should read less than a chunk -> read a chunk */
    if p_size < (*p_stream).m_buffer_size {
      /* we should do an actual read on the media */
      (*p_stream).m_bytes_in_buffer = (*p_stream).m_read_fn.expect("non-null function pointer")(
        (*p_stream).m_stored_data as *mut libc::c_void,
        (*p_stream).m_buffer_size,
        (*p_stream).m_user_data,
      );
      if (*p_stream).m_bytes_in_buffer == -(1 as libc::c_int) as OPJ_SIZE_T {
        /* end of stream */
        opj_event_msg(
          p_event_mgr,
          4 as libc::c_int,
          b"Stream reached its end !\n\x00" as *const u8 as *const libc::c_char,
        );
        (*p_stream).m_bytes_in_buffer = 0 as libc::c_int as OPJ_SIZE_T;
        (*p_stream).m_status |= 0x4 as libc::c_uint;
        /* end of stream */
        return if l_read_nb_bytes != 0 {
          l_read_nb_bytes
        } else {
          -(1 as libc::c_int) as OPJ_SIZE_T
        };
      } else {
        if (*p_stream).m_bytes_in_buffer < p_size {
          /* not enough data */
          l_read_nb_bytes = (l_read_nb_bytes as libc::c_ulong)
            .wrapping_add((*p_stream).m_bytes_in_buffer) as OPJ_SIZE_T
            as OPJ_SIZE_T;
          memcpy(
            p_buffer as *mut libc::c_void,
            (*p_stream).m_current_data as *const libc::c_void,
            (*p_stream).m_bytes_in_buffer,
          );
          (*p_stream).m_current_data = (*p_stream).m_stored_data;
          p_buffer = p_buffer.offset((*p_stream).m_bytes_in_buffer as isize);
          p_size = (p_size as libc::c_ulong).wrapping_sub((*p_stream).m_bytes_in_buffer)
            as OPJ_SIZE_T as OPJ_SIZE_T;
          (*p_stream).m_byte_offset += (*p_stream).m_bytes_in_buffer as OPJ_OFF_T;
          (*p_stream).m_bytes_in_buffer = 0 as libc::c_int as OPJ_SIZE_T
        } else {
          l_read_nb_bytes =
            (l_read_nb_bytes as libc::c_ulong).wrapping_add(p_size) as OPJ_SIZE_T as OPJ_SIZE_T;
          memcpy(
            p_buffer as *mut libc::c_void,
            (*p_stream).m_current_data as *const libc::c_void,
            p_size,
          );
          (*p_stream).m_current_data = (*p_stream).m_current_data.offset(p_size as isize);
          (*p_stream).m_bytes_in_buffer = ((*p_stream).m_bytes_in_buffer as libc::c_ulong)
            .wrapping_sub(p_size) as OPJ_SIZE_T
            as OPJ_SIZE_T;
          (*p_stream).m_byte_offset += p_size as OPJ_OFF_T;
          return l_read_nb_bytes;
        }
      }
    } else {
      /* direct read on the dest buffer */
      (*p_stream).m_bytes_in_buffer = (*p_stream).m_read_fn.expect("non-null function pointer")(
        p_buffer as *mut libc::c_void,
        p_size,
        (*p_stream).m_user_data,
      );
      if (*p_stream).m_bytes_in_buffer == -(1 as libc::c_int) as OPJ_SIZE_T {
        /*  end of stream */
        opj_event_msg(
          p_event_mgr,
          4 as libc::c_int,
          b"Stream reached its end !\n\x00" as *const u8 as *const libc::c_char,
        );
        (*p_stream).m_bytes_in_buffer = 0 as libc::c_int as OPJ_SIZE_T;
        (*p_stream).m_status |= 0x4 as libc::c_uint;
        /* end of stream */
        return if l_read_nb_bytes != 0 {
          l_read_nb_bytes
        } else {
          -(1 as libc::c_int) as OPJ_SIZE_T
        };
      } else {
        if (*p_stream).m_bytes_in_buffer < p_size {
          /* not enough data */
          l_read_nb_bytes = (l_read_nb_bytes as libc::c_ulong)
            .wrapping_add((*p_stream).m_bytes_in_buffer) as OPJ_SIZE_T
            as OPJ_SIZE_T;
          (*p_stream).m_current_data = (*p_stream).m_stored_data;
          p_buffer = p_buffer.offset((*p_stream).m_bytes_in_buffer as isize);
          p_size = (p_size as libc::c_ulong).wrapping_sub((*p_stream).m_bytes_in_buffer)
            as OPJ_SIZE_T as OPJ_SIZE_T;
          (*p_stream).m_byte_offset += (*p_stream).m_bytes_in_buffer as OPJ_OFF_T;
          (*p_stream).m_bytes_in_buffer = 0 as libc::c_int as OPJ_SIZE_T
        } else {
          /* we have read the exact size */
          l_read_nb_bytes = (l_read_nb_bytes as libc::c_ulong)
            .wrapping_add((*p_stream).m_bytes_in_buffer) as OPJ_SIZE_T
            as OPJ_SIZE_T;
          (*p_stream).m_byte_offset += (*p_stream).m_bytes_in_buffer as OPJ_OFF_T;
          (*p_stream).m_current_data = (*p_stream).m_stored_data;
          (*p_stream).m_bytes_in_buffer = 0 as libc::c_int as OPJ_SIZE_T;
          return l_read_nb_bytes;
        }
      }
    }
  }
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_write_data(
  mut p_stream: *mut opj_stream_private_t,
  mut p_buffer: *const OPJ_BYTE,
  mut p_size: OPJ_SIZE_T,
  mut p_event_mgr: *mut opj_event_mgr_t,
) -> OPJ_SIZE_T {
  let mut l_remaining_bytes = 0 as libc::c_int as OPJ_SIZE_T;
  let mut l_write_nb_bytes = 0 as libc::c_int as OPJ_SIZE_T;
  if (*p_stream).m_status & 0x8 as libc::c_uint != 0 {
    return -(1 as libc::c_int) as OPJ_SIZE_T;
  }
  loop {
    l_remaining_bytes = (*p_stream)
      .m_buffer_size
      .wrapping_sub((*p_stream).m_bytes_in_buffer);
    /* we have more memory than required */
    if l_remaining_bytes >= p_size {
      memcpy(
        (*p_stream).m_current_data as *mut libc::c_void,
        p_buffer as *const libc::c_void,
        p_size,
      );
      (*p_stream).m_current_data = (*p_stream).m_current_data.offset(p_size as isize);
      (*p_stream).m_bytes_in_buffer = ((*p_stream).m_bytes_in_buffer as libc::c_ulong)
        .wrapping_add(p_size) as OPJ_SIZE_T as OPJ_SIZE_T;
      l_write_nb_bytes =
        (l_write_nb_bytes as libc::c_ulong).wrapping_add(p_size) as OPJ_SIZE_T as OPJ_SIZE_T;
      (*p_stream).m_byte_offset += p_size as OPJ_OFF_T;
      return l_write_nb_bytes;
    }
    /* we copy data and then do an actual read on the stream */
    if l_remaining_bytes != 0 {
      l_write_nb_bytes = (l_write_nb_bytes as libc::c_ulong).wrapping_add(l_remaining_bytes)
        as OPJ_SIZE_T as OPJ_SIZE_T;
      memcpy(
        (*p_stream).m_current_data as *mut libc::c_void,
        p_buffer as *const libc::c_void,
        l_remaining_bytes,
      );
      (*p_stream).m_current_data = (*p_stream).m_stored_data;
      p_buffer = p_buffer.offset(l_remaining_bytes as isize);
      p_size =
        (p_size as libc::c_ulong).wrapping_sub(l_remaining_bytes) as OPJ_SIZE_T as OPJ_SIZE_T;
      (*p_stream).m_bytes_in_buffer = ((*p_stream).m_bytes_in_buffer as libc::c_ulong)
        .wrapping_add(l_remaining_bytes) as OPJ_SIZE_T
        as OPJ_SIZE_T;
      (*p_stream).m_byte_offset += l_remaining_bytes as OPJ_OFF_T
    }
    if opj_stream_flush(p_stream, p_event_mgr) == 0 {
      return -(1 as libc::c_int) as OPJ_SIZE_T;
    }
  }
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_flush(
  mut p_stream: *mut opj_stream_private_t,
  mut p_event_mgr: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* the number of bytes written on the media. */
  let mut l_current_write_nb_bytes = 0 as libc::c_int as OPJ_SIZE_T;
  (*p_stream).m_current_data = (*p_stream).m_stored_data;
  while (*p_stream).m_bytes_in_buffer != 0 {
    /* we should do an actual write on the media */
    l_current_write_nb_bytes = (*p_stream).m_write_fn.expect("non-null function pointer")(
      (*p_stream).m_current_data as *mut libc::c_void,
      (*p_stream).m_bytes_in_buffer,
      (*p_stream).m_user_data,
    );
    if l_current_write_nb_bytes == -(1 as libc::c_int) as OPJ_SIZE_T {
      (*p_stream).m_status |= 0x8 as libc::c_uint;
      opj_event_msg(
        p_event_mgr,
        4 as libc::c_int,
        b"Error on writing stream!\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0 as libc::c_int;
    }
    (*p_stream).m_current_data = (*p_stream)
      .m_current_data
      .offset(l_current_write_nb_bytes as isize);
    (*p_stream).m_bytes_in_buffer = ((*p_stream).m_bytes_in_buffer as libc::c_ulong)
      .wrapping_sub(l_current_write_nb_bytes) as OPJ_SIZE_T
      as OPJ_SIZE_T
  }
  (*p_stream).m_current_data = (*p_stream).m_stored_data;
  return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_read_skip(
  mut p_stream: *mut opj_stream_private_t,
  mut p_size: OPJ_OFF_T,
  mut p_event_mgr: *mut opj_event_mgr_t,
) -> OPJ_OFF_T {
  let mut l_skip_nb_bytes = 0 as libc::c_int as OPJ_OFF_T;
  let mut l_current_skip_nb_bytes = 0 as libc::c_int as OPJ_OFF_T;
  assert!(p_size >= 0 as libc::c_int as libc::c_long);
  if (*p_stream).m_bytes_in_buffer >= p_size as OPJ_SIZE_T {
    (*p_stream).m_current_data = (*p_stream).m_current_data.offset(p_size as isize);
    /* it is safe to cast p_size to OPJ_SIZE_T since it is <= m_bytes_in_buffer
    which is of type OPJ_SIZE_T */
    (*p_stream).m_bytes_in_buffer = ((*p_stream).m_bytes_in_buffer as libc::c_ulong)
      .wrapping_sub(p_size as OPJ_SIZE_T) as OPJ_SIZE_T
      as OPJ_SIZE_T;
    l_skip_nb_bytes += p_size;
    (*p_stream).m_byte_offset += l_skip_nb_bytes;
    return l_skip_nb_bytes;
  }
  /* we are now in the case when the remaining data if not sufficient */
  if (*p_stream).m_status & 0x4 as libc::c_uint != 0 {
    l_skip_nb_bytes += (*p_stream).m_bytes_in_buffer as OPJ_OFF_T;
    (*p_stream).m_current_data = (*p_stream)
      .m_current_data
      .offset((*p_stream).m_bytes_in_buffer as isize);
    (*p_stream).m_bytes_in_buffer = 0 as libc::c_int as OPJ_SIZE_T;
    (*p_stream).m_byte_offset += l_skip_nb_bytes;
    return if l_skip_nb_bytes != 0 {
      l_skip_nb_bytes
    } else {
      -(1 as libc::c_int) as OPJ_OFF_T
    };
  }
  /* the flag is not set, we copy data and then do an actual skip on the stream */
  if (*p_stream).m_bytes_in_buffer != 0 {
    l_skip_nb_bytes += (*p_stream).m_bytes_in_buffer as OPJ_OFF_T;
    (*p_stream).m_current_data = (*p_stream).m_stored_data;
    p_size -= (*p_stream).m_bytes_in_buffer as OPJ_OFF_T;
    (*p_stream).m_bytes_in_buffer = 0 as libc::c_int as OPJ_SIZE_T
  }
  while p_size > 0 as libc::c_int as libc::c_long {
    /* Check if we are going beyond the end of file. Most skip_fn do not */
    /* check that, but we must be careful not to advance m_byte_offset */
    /* beyond m_user_data_length, otherwise */
    /* opj_stream_get_number_byte_left() will assert. */
    if ((*p_stream).m_byte_offset + l_skip_nb_bytes + p_size) as OPJ_UINT64
      > (*p_stream).m_user_data_length
    {
      opj_event_msg(
        p_event_mgr,
        4 as libc::c_int,
        b"Stream reached its end !\n\x00" as *const u8 as *const libc::c_char,
      );
      (*p_stream).m_byte_offset += l_skip_nb_bytes;
      l_skip_nb_bytes = (*p_stream)
        .m_user_data_length
        .wrapping_sub((*p_stream).m_byte_offset as OPJ_UINT64) as OPJ_OFF_T;
      opj_stream_read_seek(
        p_stream,
        (*p_stream).m_user_data_length as OPJ_OFF_T,
        p_event_mgr,
      );
      (*p_stream).m_status |= 0x4 as libc::c_uint;
      /* end if stream */
      return if l_skip_nb_bytes != 0 {
        l_skip_nb_bytes
      } else {
        -(1 as libc::c_int) as OPJ_OFF_T
      };
    }
    /* we should do an actual skip on the media */
    l_current_skip_nb_bytes =
      (*p_stream).m_skip_fn.expect("non-null function pointer")(p_size, (*p_stream).m_user_data);
    if l_current_skip_nb_bytes == -(1 as libc::c_int) as OPJ_OFF_T {
      opj_event_msg(
        p_event_mgr,
        4 as libc::c_int,
        b"Stream reached its end !\n\x00" as *const u8 as *const libc::c_char,
      );
      (*p_stream).m_status |= 0x4 as libc::c_uint;
      (*p_stream).m_byte_offset += l_skip_nb_bytes;
      /* end if stream */
      return if l_skip_nb_bytes != 0 {
        l_skip_nb_bytes
      } else {
        -(1 as libc::c_int) as OPJ_OFF_T
      };
    }
    p_size -= l_current_skip_nb_bytes;
    l_skip_nb_bytes += l_current_skip_nb_bytes
  }
  (*p_stream).m_byte_offset += l_skip_nb_bytes;
  return l_skip_nb_bytes;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_write_skip(
  mut p_stream: *mut opj_stream_private_t,
  mut p_size: OPJ_OFF_T,
  mut p_event_mgr: *mut opj_event_mgr_t,
) -> OPJ_OFF_T {
  let mut l_is_written = 0 as libc::c_int;
  let mut l_current_skip_nb_bytes = 0 as libc::c_int as OPJ_OFF_T;
  let mut l_skip_nb_bytes = 0 as libc::c_int as OPJ_OFF_T;
  if (*p_stream).m_status & 0x8 as libc::c_uint != 0 {
    return -(1 as libc::c_int) as OPJ_OFF_T;
  }
  /* we should flush data */
  l_is_written = opj_stream_flush(p_stream, p_event_mgr);
  if l_is_written == 0 {
    (*p_stream).m_status |= 0x8 as libc::c_uint;
    (*p_stream).m_bytes_in_buffer = 0 as libc::c_int as OPJ_SIZE_T;
    return -(1 as libc::c_int) as OPJ_OFF_T;
  }
  /* then skip */
  while p_size > 0 as libc::c_int as libc::c_long {
    /* we should do an actual skip on the media */
    l_current_skip_nb_bytes =
      (*p_stream).m_skip_fn.expect("non-null function pointer")(p_size, (*p_stream).m_user_data);
    if l_current_skip_nb_bytes == -(1 as libc::c_int) as OPJ_OFF_T {
      opj_event_msg(
        p_event_mgr,
        4 as libc::c_int,
        b"Stream error!\n\x00" as *const u8 as *const libc::c_char,
      );
      (*p_stream).m_status |= 0x8 as libc::c_uint;
      (*p_stream).m_byte_offset += l_skip_nb_bytes;
      /* end if stream */
      return if l_skip_nb_bytes != 0 {
        l_skip_nb_bytes
      } else {
        -(1 as libc::c_int) as OPJ_OFF_T
      };
    }
    p_size -= l_current_skip_nb_bytes;
    l_skip_nb_bytes += l_current_skip_nb_bytes
  }
  (*p_stream).m_byte_offset += l_skip_nb_bytes;
  return l_skip_nb_bytes;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_tell(mut p_stream: *const opj_stream_private_t) -> OPJ_OFF_T {
  return (*p_stream).m_byte_offset;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_get_number_byte_left(
  mut p_stream: *const opj_stream_private_t,
) -> OPJ_OFF_T {
  assert!((*p_stream).m_byte_offset >= 0 as libc::c_int as libc::c_long);
  assert!((*p_stream).m_user_data_length >= (*p_stream).m_byte_offset as OPJ_UINT64);
  return if (*p_stream).m_user_data_length != 0 {
    ((*p_stream).m_user_data_length as OPJ_OFF_T) - (*p_stream).m_byte_offset
  } else {
    0 as libc::c_int as libc::c_long
  };
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_skip(
  mut p_stream: *mut opj_stream_private_t,
  mut p_size: OPJ_OFF_T,
  mut p_event_mgr: *mut opj_event_mgr_t,
) -> OPJ_OFF_T {
  assert!(p_size >= 0 as libc::c_int as libc::c_long);
  return (*p_stream).m_opj_skip.expect("non-null function pointer")(p_stream, p_size, p_event_mgr);
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_read_seek(
  mut p_stream: *mut opj_stream_private_t,
  mut p_size: OPJ_OFF_T,
  mut _p_event_mgr: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  (*p_stream).m_current_data = (*p_stream).m_stored_data;
  (*p_stream).m_bytes_in_buffer = 0 as libc::c_int as OPJ_SIZE_T;
  if (*p_stream).m_seek_fn.expect("non-null function pointer")(p_size, (*p_stream).m_user_data) == 0
  {
    (*p_stream).m_status |= 0x4 as libc::c_uint;
    return 0 as libc::c_int;
  } else {
    /* reset stream status */
    (*p_stream).m_status &= !(0x4 as libc::c_uint);
    (*p_stream).m_byte_offset = p_size
  }
  return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_write_seek(
  mut p_stream: *mut opj_stream_private_t,
  mut p_size: OPJ_OFF_T,
  mut p_event_mgr: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  if opj_stream_flush(p_stream, p_event_mgr) == 0 {
    (*p_stream).m_status |= 0x8 as libc::c_uint;
    return 0 as libc::c_int;
  }
  (*p_stream).m_current_data = (*p_stream).m_stored_data;
  (*p_stream).m_bytes_in_buffer = 0 as libc::c_int as OPJ_SIZE_T;
  if (*p_stream).m_seek_fn.expect("non-null function pointer")(p_size, (*p_stream).m_user_data) == 0
  {
    (*p_stream).m_status |= 0x8 as libc::c_uint;
    return 0 as libc::c_int;
  } else {
    (*p_stream).m_byte_offset = p_size
  }
  return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_seek(
  mut p_stream: *mut opj_stream_private_t,
  mut p_size: OPJ_OFF_T,
  mut p_event_mgr: *mut opj_event_mgr,
) -> OPJ_BOOL {
  assert!(p_size >= 0 as libc::c_int as libc::c_long);
  return (*p_stream).m_opj_seek.expect("non-null function pointer")(p_stream, p_size, p_event_mgr);
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_has_seek(
  mut p_stream: *const opj_stream_private_t,
) -> OPJ_BOOL {
  return ((*p_stream).m_seek_fn
    != Some(
      opj_stream_default_seek
        as unsafe extern "C" fn(_: OPJ_OFF_T, _: *mut libc::c_void) -> OPJ_BOOL,
    )) as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_default_read(
  mut _p_buffer: *mut libc::c_void,
  mut _p_nb_bytes: OPJ_SIZE_T,
  mut _p_user_data: *mut libc::c_void,
) -> OPJ_SIZE_T {
  return -(1 as libc::c_int) as OPJ_SIZE_T;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_default_write(
  mut _p_buffer: *mut libc::c_void,
  mut _p_nb_bytes: OPJ_SIZE_T,
  mut _p_user_data: *mut libc::c_void,
) -> OPJ_SIZE_T {
  return -(1 as libc::c_int) as OPJ_SIZE_T;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_default_skip(
  mut _p_nb_bytes: OPJ_OFF_T,
  mut _p_user_data: *mut libc::c_void,
) -> OPJ_OFF_T {
  return -(1 as libc::c_int) as OPJ_OFF_T;
}
#[no_mangle]
pub unsafe extern "C" fn opj_stream_default_seek(
  mut _p_nb_bytes: OPJ_OFF_T,
  mut _p_user_data: *mut libc::c_void,
) -> OPJ_BOOL {
  return 0 as libc::c_int;
}
