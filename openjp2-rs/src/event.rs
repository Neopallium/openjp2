pub use super::consts::event::*;

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

/// Core event manager: one optional `FnMut` sink per message level. The C-callback adapter
/// (which wraps `opj_msg_callback` + user data in closures) lives in the capi crate; the core
/// itself only formats Rust strings and dispatches them to the sinks.
#[derive(Default)]
pub struct opj_event_mgr {
  error_handler: Option<Box<dyn FnMut(&str)>>,
  warning_handler: Option<Box<dyn FnMut(&str)>>,
  info_handler: Option<Box<dyn FnMut(&str)>>,
}

impl opj_event_mgr {
  pub fn set_default_event_handler(&mut self) {
    *self = Self::default();
  }

  pub fn set_info_handler(&mut self, handler: impl FnMut(&str) + 'static) {
    self.info_handler = Some(Box::new(handler));
  }

  pub fn set_warning_handler(&mut self, handler: impl FnMut(&str) + 'static) {
    self.warning_handler = Some(Box::new(handler));
  }

  pub fn set_error_handler(&mut self, handler: impl FnMut(&str) + 'static) {
    self.error_handler = Some(Box::new(handler));
  }

  pub fn is_enabled(&self, event_type: EventType) -> bool {
    match event_type {
      EventType::Error => self.error_handler.is_some(),
      EventType::Warning => self.warning_handler.is_some(),
      EventType::Info => self.info_handler.is_some(),
    }
  }

  pub fn msg_write(&mut self, event_type: EventType, msg: &str) -> i32 {
    let handler = match event_type {
      EventType::Error => &mut self.error_handler,
      EventType::Warning => &mut self.warning_handler,
      EventType::Info => &mut self.info_handler,
    };
    match handler {
      Some(handler) => {
        handler(msg);
        1
      }
      None => 0,
    }
  }
}

macro_rules! event_msg {
  ($event_mgr:expr, $event_type:expr, $fmt:expr) => {
    $event_mgr.msg_write($event_type, $fmt)
  };
  ($event_mgr:expr, $event_type:expr, $fmt:expr,) => {
    $event_mgr.msg_write($event_type, $fmt)
  };
  ($event_mgr:expr, $event_type:expr, $fmt:expr, $($arg:expr),*) => {
    event_msg!(internal $event_mgr, $event_type, $fmt, $($arg,)*)
  };
  ($event_mgr:expr, $event_type:expr, $fmt:expr, $($arg:expr,)*) => {
    event_msg!(internal $event_mgr, $event_type, $fmt, $($arg,)*)
  };
  (internal $event_mgr:expr, $event_type:expr, $($fmt:expr,)*) => {
    if $event_mgr.is_enabled($event_type) {
      let s = format!($($fmt),*);
      $event_mgr.msg_write($event_type, &s)
    } else {
      0i32
    }
  };
}
