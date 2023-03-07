use super::openjpeg::*;
pub use super::consts::event::*;
use ::libc;

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

#[derive(Copy, Clone)]
pub struct opj_event_mgr {
  m_error_data: *mut libc::c_void,
  m_warning_data: *mut libc::c_void,
  m_info_data: *mut libc::c_void,
  error_handler: opj_msg_callback,
  warning_handler: opj_msg_callback,
  info_handler: opj_msg_callback,
}

impl Default for opj_event_mgr {
  fn default() -> Self {
    Self {
      m_error_data: 0 as *mut libc::c_void,
      m_warning_data: 0 as *mut libc::c_void,
      m_info_data: 0 as *mut libc::c_void,
      error_handler: None,
      info_handler: None,
      warning_handler: None,
    }
  }
}

impl opj_event_mgr {
  pub fn set_default_event_handler(&mut self) {
    self.m_error_data = 0 as *mut libc::c_void;
    self.m_warning_data = 0 as *mut libc::c_void;
    self.m_info_data = 0 as *mut libc::c_void;
    self.error_handler = None;
    self.info_handler = None;
    self.warning_handler = None;
  }

  pub fn set_info_handler(&mut self,
    mut p_callback: opj_msg_callback,
    mut p_user_data: *mut libc::c_void,
  ) {
    self.info_handler = p_callback;
    self.m_info_data = p_user_data;
  }

  pub fn set_warning_handler(&mut self,
    mut p_callback: opj_msg_callback,
    mut p_user_data: *mut libc::c_void,
  ) {
    self.warning_handler = p_callback;
    self.m_warning_data = p_user_data;
  }

  pub fn set_error_handler(&mut self,
    mut p_callback: opj_msg_callback,
    mut p_user_data: *mut libc::c_void,
  ) {
    self.error_handler = p_callback;
    self.m_error_data = p_user_data;
  }

  pub fn get_handler(&self,
    event_type: EventType,
  ) -> Option<(opj_msg_callback_fn, *mut libc::c_void)> {
    match event_type {
      EventType::Error => {
        self.error_handler.map(|h| (h, self.m_error_data))
      }
      EventType::Warning => {
        self.warning_handler.map(|h| (h, self.m_warning_data))
      }
      EventType::Info => {
        self.info_handler.map(|h| (h, self.m_info_data))
      }
    }
  }

  pub fn is_enabled(&self, event_type: EventType) -> bool {
    self.get_handler(event_type).is_some()
  }

  pub fn msg_write(&self,
    event_type: EventType,
    msg: &str,
  ) -> bool {
    let (msg_handler, l_data) = match self.get_handler(event_type) {
      Some(handler) => handler,
      None => {
        return false;
      }
    };
    let c_msg = std::ffi::CString::new(msg).unwrap();
    /* output the message to the user program */
    unsafe {
      msg_handler(c_msg.as_ptr(), l_data);
    }
    true
  }
}

macro_rules! event_msg {
  ($event_mgr:expr, $event_type:expr, $fmt:expr) => {
    if $event_mgr.msg_write($event_type, $fmt) {
      1i32
    } else {
      0i32
    }
  };
  ($event_mgr:expr, $event_type:expr, $fmt:expr, $($arg:expr),*) => {
    event_msg!(internal $event_mgr, $event_type, $fmt, $($arg,)*)
  };
  ($event_mgr:expr, $event_type:expr, $fmt:expr, $($arg:expr,)*) => {
    event_msg!(internal $event_mgr, $event_type, $fmt, $($arg,)*)
  };
  (internal $event_mgr:expr, $event_type:expr, $fmt:expr, $($arg:expr,)*) => {
    if $event_mgr.is_enabled($event_type) {
      let s = ::sprintf::sprintf!($fmt, $($arg),*);
      match &s {
        Ok(s) => if $event_mgr.msg_write($event_type, s) {
          1i32
        } else {
          0i32
        },
        Err(err) => {
          log::error!("sprintf failed: {err:?}");
          0i32
        }
      }
    } else {
      0i32
    }
  };
}
