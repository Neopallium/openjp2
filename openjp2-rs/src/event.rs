use ::libc;
extern "C" {

  fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;

  fn vsnprintf(
    _: *mut libc::c_char,
    _: libc::c_ulong,
    _: *const libc::c_char,
    _: ::std::ffi::VaList,
  ) -> libc::c_int;
}
pub type __builtin_va_list = [__va_list_tag; 1];

#[repr(C)]
#[derive(Copy, Clone)]
pub struct __va_list_tag {
  pub gp_offset: libc::c_uint,
  pub fp_offset: libc::c_uint,
  pub overflow_arg_area: *mut libc::c_void,
  pub reg_save_area: *mut libc::c_void,
}
pub type __int32_t = libc::c_int;
pub type va_list = __builtin_va_list;
pub type OPJ_BOOL = libc::c_int;
pub type int32_t = __int32_t;
pub type OPJ_INT32 = int32_t;
pub type opj_msg_callback =
  Option<unsafe extern "C" fn(_: *const libc::c_char, _: *mut libc::c_void) -> ()>;

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
/* ==========================================================
  Utility functions
==========================================================*/
/* ----------------------------------------------------------------------- */
/* *
 * Default callback function.
 * Do nothing.
 */
unsafe extern "C" fn opj_default_callback(
  mut _msg: *const libc::c_char,
  mut _client_data: *mut libc::c_void,
) {
}
/* ----------------------------------------------------------------------- */
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe extern "C" fn opj_event_msg(
  mut p_event_mgr: *mut opj_event_mgr_t,
  mut event_type: OPJ_INT32,
  mut fmt: *const libc::c_char,
  mut args: ...
) -> OPJ_BOOL {
  /* 512 bytes should be more than enough for a short message */
  let mut msg_handler: opj_msg_callback = None;
  let mut l_data = 0 as *mut libc::c_void;
  if !p_event_mgr.is_null() {
    match event_type {
      1 => {
        msg_handler = (*p_event_mgr).error_handler;
        l_data = (*p_event_mgr).m_error_data
      }
      2 => {
        msg_handler = (*p_event_mgr).warning_handler;
        l_data = (*p_event_mgr).m_warning_data
      }
      4 => {
        msg_handler = (*p_event_mgr).info_handler;
        l_data = (*p_event_mgr).m_info_data
      }
      _ => {}
    }
    if msg_handler.is_none() {
      return 0 as libc::c_int;
    }
  } else {
    return 0 as libc::c_int;
  }
  if !fmt.is_null() && !p_event_mgr.is_null() {
    let mut arg: ::std::ffi::VaListImpl;
    let mut message: [libc::c_char; 512] = [0; 512];
    memset(
      message.as_mut_ptr() as *mut libc::c_void,
      0 as libc::c_int,
      512 as libc::c_int as libc::c_ulong,
    );
    /* initialize the optional parameter list */
    arg = args.clone();
    /* parse the format string and put the result in 'message' */
    vsnprintf(
      message.as_mut_ptr(),
      512 as libc::c_int as libc::c_ulong,
      fmt,
      arg.as_va_list(),
    );
    /* force zero termination for Windows _vsnprintf() of old MSVC */
    message[(512 as libc::c_int - 1 as libc::c_int) as usize] = '\u{0}' as i32 as libc::c_char;
    /* deinitialize the optional parameter list */
    /* output the message to the user program */
    msg_handler.expect("non-null function pointer")(message.as_mut_ptr(), l_data);
  }
  return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_set_default_event_handler(mut p_manager: *mut opj_event_mgr_t) {
  (*p_manager).m_error_data = 0 as *mut libc::c_void;
  (*p_manager).m_warning_data = 0 as *mut libc::c_void;
  (*p_manager).m_info_data = 0 as *mut libc::c_void;
  (*p_manager).error_handler = Some(
    opj_default_callback
      as unsafe extern "C" fn(_: *const libc::c_char, _: *mut libc::c_void) -> (),
  );
  (*p_manager).info_handler = Some(
    opj_default_callback
      as unsafe extern "C" fn(_: *const libc::c_char, _: *mut libc::c_void) -> (),
  );
  (*p_manager).warning_handler = Some(
    opj_default_callback
      as unsafe extern "C" fn(_: *const libc::c_char, _: *mut libc::c_void) -> (),
  );
}
