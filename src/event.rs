//! The event manager: ERROR/WARNING/INFO messages to the `log`
//! crate by default or the C-style callbacks (feature `event-handlers`).
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
///
/// The sinks sit behind `Mutex`es so the manager is `Sync`: the t1 codeblock jobs
/// dispatch events from rayon worker threads.
#[derive(Default)]
pub struct EventMgr {
  error_handler: Option<std::sync::Mutex<Box<dyn FnMut(&str) + Send>>>,
  warning_handler: Option<std::sync::Mutex<Box<dyn FnMut(&str) + Send>>>,
  info_handler: Option<std::sync::Mutex<Box<dyn FnMut(&str) + Send>>>,
}

impl EventMgr {
  pub fn set_default_event_handler(&mut self) {
    *self = Self::default();
  }

  pub fn set_info_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) {
    self.info_handler = Some(std::sync::Mutex::new(Box::new(handler)));
  }

  pub fn set_warning_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) {
    self.warning_handler = Some(std::sync::Mutex::new(Box::new(handler)));
  }

  pub fn set_error_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) {
    self.error_handler = Some(std::sync::Mutex::new(Box::new(handler)));
  }

  pub fn is_enabled(&self, event_type: EventType) -> bool {
    /* A registered handler OR an installed `log` sink (the default path:
    without the `event-handlers` feature the messages go to the standard
    `log` ecosystem). */
    let has_handler = match event_type {
      EventType::Error => self.error_handler.is_some(),
      EventType::Warning => self.warning_handler.is_some(),
      EventType::Info => self.info_handler.is_some(),
    };
    has_handler
      || match event_type {
        EventType::Error => log::log_enabled!(log::Level::Error),
        EventType::Warning => log::log_enabled!(log::Level::Warn),
        EventType::Info => log::log_enabled!(log::Level::Info),
      }
  }

  pub fn msg_write(&self, event_type: EventType, msg: &str) -> i32 {
    let handler = match event_type {
      EventType::Error => &self.error_handler,
      EventType::Warning => &self.warning_handler,
      EventType::Info => &self.info_handler,
    };
    match handler {
      Some(handler) => {
        /* A poisoned lock means a handler panicked mid-callback; degrade by
        keeping the (possibly broken) handler installed instead of panicking
        the host through the event path (the handler's own panics are caught
        at the API boundaries). */
        handler
          .lock()
          .unwrap_or_else(|poisoned| poisoned.into_inner())(msg);
        1
      }
      None => {
        /* No handler registered: route through the standard `log` crate
        (the stable Rust API's diagnostics path - the C-style callbacks are
        an opt-in via the `event-handlers` feature). */
        match event_type {
          EventType::Error => log::error!("{}", msg),
          EventType::Warning => log::warn!("{}", msg),
          EventType::Info => log::info!("{}", msg),
        }
        0
      }
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
