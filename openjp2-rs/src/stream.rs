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

use super::event::*;
use super::malloc::*;
use super::openjpeg::*;

extern "C" {
  fn memcpy(
    _: *mut core::ffi::c_void,
    _: *const core::ffi::c_void,
    _: usize,
  ) -> *mut core::ffi::c_void;
}

impl opj_stream_private {
  pub fn new(buffer_size: usize, is_input: bool) -> Option<Self> {
    let mut l_stream = Self {
      m_inner: None,
      m_user_data: std::ptr::null_mut(),
      m_free_user_data_fn: Default::default(),
      m_user_data_length: Default::default(),
      m_read_fn: Default::default(),
      m_write_fn: Default::default(),
      m_skip_fn: Default::default(),
      m_seek_fn: Default::default(),
      m_stored_data: std::ptr::null_mut(),
      m_current_data: std::ptr::null_mut(),
      m_opj_skip: Default::default(),
      m_opj_seek: Default::default(),
      m_bytes_in_buffer: Default::default(),
      m_byte_offset: Default::default(),
      m_buffer_size: Default::default(),
      m_status: Default::default(),
    };
    l_stream.m_buffer_size = buffer_size;
    l_stream.m_stored_data = unsafe { opj_malloc(buffer_size) as *mut OPJ_BYTE };
    if l_stream.m_stored_data.is_null() {
      return None;
    }
    l_stream.m_current_data = l_stream.m_stored_data;
    if is_input {
      l_stream.m_status |= 0x2u32;
      l_stream.m_opj_skip = Some(
        opj_stream_read_skip
          as fn(_: &mut opj_stream_private_t, _: OPJ_OFF_T, _: &mut opj_event_mgr) -> OPJ_OFF_T,
      );
      l_stream.m_opj_seek = Some(
        opj_stream_read_seek
          as fn(_: &mut opj_stream_private_t, _: OPJ_OFF_T, _: &mut opj_event_mgr) -> OPJ_BOOL,
      )
    } else {
      l_stream.m_status |= 0x1u32;
      l_stream.m_opj_skip = Some(
        opj_stream_write_skip
          as fn(_: &mut opj_stream_private_t, _: OPJ_OFF_T, _: &mut opj_event_mgr) -> OPJ_OFF_T,
      );
      l_stream.m_opj_seek = Some(
        opj_stream_write_seek
          as fn(_: &mut opj_stream_private_t, _: OPJ_OFF_T, _: &mut opj_event_mgr) -> OPJ_BOOL,
      )
    }
    l_stream.m_read_fn = Some(
      opj_stream_default_read
        as unsafe extern "C" fn(
          _: *mut core::ffi::c_void,
          _: OPJ_SIZE_T,
          _: *mut core::ffi::c_void,
        ) -> OPJ_SIZE_T,
    );
    l_stream.m_write_fn = Some(
      opj_stream_default_write
        as unsafe extern "C" fn(
          _: *mut core::ffi::c_void,
          _: OPJ_SIZE_T,
          _: *mut core::ffi::c_void,
        ) -> OPJ_SIZE_T,
    );
    l_stream.m_skip_fn = Some(
      opj_stream_default_skip
        as unsafe extern "C" fn(_: OPJ_OFF_T, _: *mut core::ffi::c_void) -> OPJ_OFF_T,
    );
    l_stream.m_seek_fn = Some(
      opj_stream_default_seek
        as unsafe extern "C" fn(_: OPJ_OFF_T, _: *mut core::ffi::c_void) -> OPJ_BOOL,
    );
    Some(l_stream)
  }
}

impl Drop for opj_stream_private {
  fn drop(&mut self) {
    unsafe {
      if self.m_free_user_data_fn.is_some() {
        self.m_free_user_data_fn.expect("non-null function pointer")(self.m_user_data);
      }
      opj_free(self.m_stored_data as *mut core::ffi::c_void);
    }
    self.m_stored_data = std::ptr::null_mut::<OPJ_BYTE>();
  }
}

pub(crate) unsafe fn opj_stream_read_data(
  mut p_stream: *mut opj_stream_private_t,
  mut p_buffer: *mut OPJ_BYTE,
  mut p_size: OPJ_SIZE_T,
  mut p_event_mgr: &mut opj_event_mgr,
) -> OPJ_SIZE_T {
  let p_stream = unsafe { &mut *p_stream };
  let old_byte_offset = p_stream.m_byte_offset;
  let res = if let Some(StreamInner::Reader(reader)) = &mut p_stream.m_inner {
    use std::io::Read;
    let buf = unsafe { std::slice::from_raw_parts_mut(p_buffer as *mut u8, p_size) };
    match reader.read_exact(buf) {
      Ok(_) => {
        p_stream.m_byte_offset += p_size as i64;
        p_size as OPJ_SIZE_T
      }
      Err(_err) => {
        // Maybe EOF, do a partial read.
        match reader.read(buf) {
          Ok(nb) => {
            p_stream.m_byte_offset += nb as i64;
            nb as OPJ_SIZE_T
          }
          Err(err) => {
            log::trace!("Failed to read from stream: {err}");
            -1i32 as OPJ_SIZE_T
          }
        }
      }
    }
  } else {
    opj_stream_read_data_old(p_stream, p_buffer, p_size, p_event_mgr)
  };
  log::trace!(
    "-- read_data({p_size}) = {res}, offset: old={old_byte_offset}, new={}, ptr={:?}",
    p_stream.m_byte_offset,
    p_stream as *const opj_stream_private
  );
  res
}

unsafe fn opj_stream_read_data_old(
  mut p_stream: &mut opj_stream_private_t,
  mut p_buffer: *mut OPJ_BYTE,
  mut p_size: OPJ_SIZE_T,
  mut p_event_mgr: &mut opj_event_mgr,
) -> OPJ_SIZE_T {
  let mut l_read_nb_bytes = 0 as OPJ_SIZE_T;
  if p_stream.m_bytes_in_buffer >= p_size {
    memcpy(
      p_buffer as *mut core::ffi::c_void,
      p_stream.m_current_data as *const core::ffi::c_void,
      p_size,
    );
    p_stream.m_current_data = p_stream.m_current_data.add(p_size);
    p_stream.m_bytes_in_buffer =
      p_stream.m_bytes_in_buffer.wrapping_sub(p_size) as OPJ_SIZE_T as OPJ_SIZE_T;
    l_read_nb_bytes = (l_read_nb_bytes as usize).wrapping_add(p_size) as OPJ_SIZE_T as OPJ_SIZE_T;
    p_stream.m_byte_offset += p_size as OPJ_OFF_T;
    return l_read_nb_bytes;
  }
  /* we are now in the case when the remaining data if not sufficient */
  if p_stream.m_status & 0x4u32 != 0 {
    l_read_nb_bytes = (l_read_nb_bytes as usize).wrapping_add(p_stream.m_bytes_in_buffer)
      as OPJ_SIZE_T as OPJ_SIZE_T;
    memcpy(
      p_buffer as *mut core::ffi::c_void,
      p_stream.m_current_data as *const core::ffi::c_void,
      p_stream.m_bytes_in_buffer,
    );
    p_stream.m_current_data = p_stream.m_current_data.add(p_stream.m_bytes_in_buffer);
    p_stream.m_byte_offset += p_stream.m_bytes_in_buffer as OPJ_OFF_T;
    p_stream.m_bytes_in_buffer = 0 as OPJ_SIZE_T;
    return if l_read_nb_bytes != 0 {
      l_read_nb_bytes
    } else {
      -(1i32) as OPJ_SIZE_T
    };
  }
  /* the flag is not set, we copy data and then do an actual read on the stream */
  if p_stream.m_bytes_in_buffer != 0 {
    l_read_nb_bytes = (l_read_nb_bytes as usize).wrapping_add(p_stream.m_bytes_in_buffer)
      as OPJ_SIZE_T as OPJ_SIZE_T;
    memcpy(
      p_buffer as *mut core::ffi::c_void,
      p_stream.m_current_data as *const core::ffi::c_void,
      p_stream.m_bytes_in_buffer,
    );
    p_stream.m_current_data = p_stream.m_stored_data;
    p_buffer = p_buffer.add(p_stream.m_bytes_in_buffer);
    p_size = p_size.wrapping_sub(p_stream.m_bytes_in_buffer) as OPJ_SIZE_T as OPJ_SIZE_T;
    p_stream.m_byte_offset += p_stream.m_bytes_in_buffer as OPJ_OFF_T;
    p_stream.m_bytes_in_buffer = 0 as OPJ_SIZE_T
  } else {
    /* case where we are already at the end of the buffer
    so reset the m_current_data to point to the start of the
    stored buffer to get ready to read from disk*/
    p_stream.m_current_data = p_stream.m_stored_data
  }
  loop {
    /* we should read less than a chunk -> read a chunk */
    if p_size < p_stream.m_buffer_size {
      /* we should do an actual read on the media */
      p_stream.m_bytes_in_buffer = p_stream.m_read_fn.expect("non-null function pointer")(
        p_stream.m_stored_data as *mut core::ffi::c_void,
        p_stream.m_buffer_size,
        p_stream.m_user_data,
      );
      if p_stream.m_bytes_in_buffer == -(1i32) as OPJ_SIZE_T {
        /* end of stream */
        event_msg!(p_event_mgr, EVT_INFO, "Stream reached its end !\n",);
        p_stream.m_bytes_in_buffer = 0 as OPJ_SIZE_T;
        p_stream.m_status |= 0x4u32;
        /* end of stream */
        return if l_read_nb_bytes != 0 {
          l_read_nb_bytes
        } else {
          -(1i32) as OPJ_SIZE_T
        };
      } else if p_stream.m_bytes_in_buffer < p_size {
        /* not enough data */
        l_read_nb_bytes = (l_read_nb_bytes as usize).wrapping_add(p_stream.m_bytes_in_buffer)
          as OPJ_SIZE_T as OPJ_SIZE_T;
        memcpy(
          p_buffer as *mut core::ffi::c_void,
          p_stream.m_current_data as *const core::ffi::c_void,
          p_stream.m_bytes_in_buffer,
        );
        p_stream.m_current_data = p_stream.m_stored_data;
        p_buffer = p_buffer.add(p_stream.m_bytes_in_buffer);
        p_size = p_size.wrapping_sub(p_stream.m_bytes_in_buffer) as OPJ_SIZE_T as OPJ_SIZE_T;
        p_stream.m_byte_offset += p_stream.m_bytes_in_buffer as OPJ_OFF_T;
        p_stream.m_bytes_in_buffer = 0 as OPJ_SIZE_T
      } else {
        l_read_nb_bytes =
          (l_read_nb_bytes as usize).wrapping_add(p_size) as OPJ_SIZE_T as OPJ_SIZE_T;
        memcpy(
          p_buffer as *mut core::ffi::c_void,
          p_stream.m_current_data as *const core::ffi::c_void,
          p_size,
        );
        p_stream.m_current_data = p_stream.m_current_data.add(p_size);
        p_stream.m_bytes_in_buffer =
          p_stream.m_bytes_in_buffer.wrapping_sub(p_size) as OPJ_SIZE_T as OPJ_SIZE_T;
        p_stream.m_byte_offset += p_size as OPJ_OFF_T;
        return l_read_nb_bytes;
      }
    } else {
      /* direct read on the dest buffer */
      p_stream.m_bytes_in_buffer = p_stream.m_read_fn.expect("non-null function pointer")(
        p_buffer as *mut core::ffi::c_void,
        p_size,
        p_stream.m_user_data,
      );
      if p_stream.m_bytes_in_buffer == -(1i32) as OPJ_SIZE_T {
        /*  end of stream */
        event_msg!(p_event_mgr, EVT_INFO, "Stream reached its end !\n",);
        p_stream.m_bytes_in_buffer = 0 as OPJ_SIZE_T;
        p_stream.m_status |= 0x4u32;
        /* end of stream */
        return if l_read_nb_bytes != 0 {
          l_read_nb_bytes
        } else {
          -(1i32) as OPJ_SIZE_T
        };
      } else if p_stream.m_bytes_in_buffer < p_size {
        /* not enough data */
        l_read_nb_bytes = (l_read_nb_bytes as usize).wrapping_add(p_stream.m_bytes_in_buffer)
          as OPJ_SIZE_T as OPJ_SIZE_T;
        p_stream.m_current_data = p_stream.m_stored_data;
        p_buffer = p_buffer.add(p_stream.m_bytes_in_buffer);
        p_size = p_size.wrapping_sub(p_stream.m_bytes_in_buffer) as OPJ_SIZE_T as OPJ_SIZE_T;
        p_stream.m_byte_offset += p_stream.m_bytes_in_buffer as OPJ_OFF_T;
        p_stream.m_bytes_in_buffer = 0 as OPJ_SIZE_T
      } else {
        /* we have read the exact size */
        l_read_nb_bytes = (l_read_nb_bytes as usize).wrapping_add(p_stream.m_bytes_in_buffer)
          as OPJ_SIZE_T as OPJ_SIZE_T;
        p_stream.m_byte_offset += p_stream.m_bytes_in_buffer as OPJ_OFF_T;
        p_stream.m_current_data = p_stream.m_stored_data;
        p_stream.m_bytes_in_buffer = 0 as OPJ_SIZE_T;
        return l_read_nb_bytes;
      }
    }
  }
}

pub(crate) unsafe fn opj_stream_write_data(
  mut p_stream: *mut opj_stream_private_t,
  mut p_buffer: *const OPJ_BYTE,
  mut p_size: OPJ_SIZE_T,
  mut p_event_mgr: &mut opj_event_mgr,
) -> OPJ_SIZE_T {
  let p_stream = unsafe { &mut *p_stream };
  log::trace!("-- write({p_size}), offset={}", p_stream.m_byte_offset);
  if let Some(StreamInner::Writer(writer)) = &mut p_stream.m_inner {
    use std::io::Write;
    let buf = unsafe { std::slice::from_raw_parts(p_buffer as *const u8, p_size) };
    return match writer.write_all(buf) {
      Ok(_) => {
        p_stream.m_byte_offset += p_size as i64;
        p_size as OPJ_SIZE_T
      }
      Err(err) => {
        log::trace!("Failed to write to stream: {err}");
        -1i32 as OPJ_SIZE_T
      }
    };
  }
  let mut l_remaining_bytes = 0 as OPJ_SIZE_T;
  let mut l_write_nb_bytes = 0 as OPJ_SIZE_T;
  if p_stream.m_status & 0x8u32 != 0 {
    return -(1i32) as OPJ_SIZE_T;
  }
  loop {
    l_remaining_bytes = p_stream
      .m_buffer_size
      .wrapping_sub(p_stream.m_bytes_in_buffer);
    /* we have more memory than required */
    if l_remaining_bytes >= p_size {
      memcpy(
        p_stream.m_current_data as *mut core::ffi::c_void,
        p_buffer as *const core::ffi::c_void,
        p_size,
      );
      p_stream.m_current_data = p_stream.m_current_data.add(p_size);
      p_stream.m_bytes_in_buffer =
        p_stream.m_bytes_in_buffer.wrapping_add(p_size) as OPJ_SIZE_T as OPJ_SIZE_T;
      l_write_nb_bytes =
        (l_write_nb_bytes as usize).wrapping_add(p_size) as OPJ_SIZE_T as OPJ_SIZE_T;
      p_stream.m_byte_offset += p_size as OPJ_OFF_T;
      return l_write_nb_bytes;
    }
    /* we copy data and then do an actual read on the stream */
    if l_remaining_bytes != 0 {
      l_write_nb_bytes =
        (l_write_nb_bytes as usize).wrapping_add(l_remaining_bytes) as OPJ_SIZE_T as OPJ_SIZE_T;
      memcpy(
        p_stream.m_current_data as *mut core::ffi::c_void,
        p_buffer as *const core::ffi::c_void,
        l_remaining_bytes,
      );
      p_stream.m_current_data = p_stream.m_stored_data;
      p_buffer = p_buffer.add(l_remaining_bytes);
      p_size = p_size.wrapping_sub(l_remaining_bytes) as OPJ_SIZE_T as OPJ_SIZE_T;
      p_stream.m_bytes_in_buffer =
        p_stream.m_bytes_in_buffer.wrapping_add(l_remaining_bytes) as OPJ_SIZE_T as OPJ_SIZE_T;
      p_stream.m_byte_offset += l_remaining_bytes as OPJ_OFF_T
    }
    if opj_stream_flush(p_stream, p_event_mgr) == 0 {
      return -(1i32) as OPJ_SIZE_T;
    }
  }
}

pub(crate) unsafe fn opj_stream_flush(
  mut p_stream: *mut opj_stream_private_t,
  mut p_event_mgr: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let p_stream = unsafe { &mut *p_stream };
  log::trace!("-- flush(), offset={}", p_stream.m_byte_offset);
  if let Some(StreamInner::Writer(writer)) = &mut p_stream.m_inner {
    use std::io::Write;
    return match writer.flush() {
      Ok(_) => 1,
      Err(err) => {
        log::trace!("Failed to flush stream: {err}");
        0
      }
    };
  }
  /* the number of bytes written on the media. */
  let mut l_current_write_nb_bytes = 0 as OPJ_SIZE_T;
  p_stream.m_current_data = p_stream.m_stored_data;
  while p_stream.m_bytes_in_buffer != 0 {
    /* we should do an actual write on the media */
    l_current_write_nb_bytes = p_stream.m_write_fn.expect("non-null function pointer")(
      p_stream.m_current_data as *mut core::ffi::c_void,
      p_stream.m_bytes_in_buffer,
      p_stream.m_user_data,
    );
    if l_current_write_nb_bytes == -(1i32) as OPJ_SIZE_T {
      p_stream.m_status |= 0x8u32;
      event_msg!(p_event_mgr, EVT_INFO, "Error on writing stream!\n",);
      return 0i32;
    }
    p_stream.m_current_data = p_stream.m_current_data.add(l_current_write_nb_bytes);
    p_stream.m_bytes_in_buffer = p_stream
      .m_bytes_in_buffer
      .wrapping_sub(l_current_write_nb_bytes) as OPJ_SIZE_T
      as OPJ_SIZE_T
  }
  p_stream.m_current_data = p_stream.m_stored_data;
  1i32
}

fn opj_stream_read_skip(
  mut p_stream: &mut opj_stream_private_t,
  mut p_size: OPJ_OFF_T,
  mut p_event_mgr: &mut opj_event_mgr,
) -> OPJ_OFF_T {
  log::trace!("-- read_skip({p_size}), offset={}", p_stream.m_byte_offset);
  let mut l_skip_nb_bytes = 0 as OPJ_OFF_T;
  let mut l_current_skip_nb_bytes = 0 as OPJ_OFF_T;
  assert!(p_size >= 0i64);
  unsafe {
    if p_stream.m_bytes_in_buffer >= p_size as OPJ_SIZE_T {
      p_stream.m_current_data = p_stream.m_current_data.offset(p_size as isize);
      /* it is safe to cast p_size to OPJ_SIZE_T since it is <= m_bytes_in_buffer
      which is of type OPJ_SIZE_T */
      p_stream.m_bytes_in_buffer = p_stream
        .m_bytes_in_buffer
        .wrapping_sub(p_size as OPJ_SIZE_T) as OPJ_SIZE_T
        as OPJ_SIZE_T;
      l_skip_nb_bytes += p_size;
      p_stream.m_byte_offset += l_skip_nb_bytes;
      return l_skip_nb_bytes;
    }
    /* we are now in the case when the remaining data if not sufficient */
    if p_stream.m_status & 0x4u32 != 0 {
      l_skip_nb_bytes += p_stream.m_bytes_in_buffer as OPJ_OFF_T;
      p_stream.m_current_data = p_stream.m_current_data.add(p_stream.m_bytes_in_buffer);
      p_stream.m_bytes_in_buffer = 0 as OPJ_SIZE_T;
      p_stream.m_byte_offset += l_skip_nb_bytes;
      return if l_skip_nb_bytes != 0 {
        l_skip_nb_bytes
      } else {
        -(1i32) as OPJ_OFF_T
      };
    }
    /* the flag is not set, we copy data and then do an actual skip on the stream */
    if p_stream.m_bytes_in_buffer != 0 {
      l_skip_nb_bytes += p_stream.m_bytes_in_buffer as OPJ_OFF_T;
      p_stream.m_current_data = p_stream.m_stored_data;
      p_size -= p_stream.m_bytes_in_buffer as OPJ_OFF_T;
      p_stream.m_bytes_in_buffer = 0 as OPJ_SIZE_T
    }
    while p_size > 0i64 {
      /* Check if we are going beyond the end of file. Most skip_fn do not */
      /* check that, but we must be careful not to advance m_byte_offset */
      /* beyond m_user_data_length, otherwise */
      /* opj_stream_get_number_byte_left() will assert. */
      if (p_stream.m_byte_offset + l_skip_nb_bytes + p_size) as OPJ_UINT64
        > p_stream.m_user_data_length
      {
        event_msg!(p_event_mgr, EVT_INFO, "Stream reached its end !\n",);
        p_stream.m_byte_offset += l_skip_nb_bytes;
        l_skip_nb_bytes = p_stream
          .m_user_data_length
          .wrapping_sub(p_stream.m_byte_offset as OPJ_UINT64)
          as OPJ_OFF_T;
        opj_stream_read_seek(
          p_stream,
          p_stream.m_user_data_length as OPJ_OFF_T,
          p_event_mgr,
        );
        p_stream.m_status |= 0x4u32;
        /* end if stream */
        return if l_skip_nb_bytes != 0 {
          l_skip_nb_bytes
        } else {
          -(1i32) as OPJ_OFF_T
        };
      }
      /* we should do an actual skip on the media */
      l_current_skip_nb_bytes =
        p_stream.m_skip_fn.expect("non-null function pointer")(p_size, p_stream.m_user_data);
      if l_current_skip_nb_bytes == -(1i32) as OPJ_OFF_T {
        event_msg!(p_event_mgr, EVT_INFO, "Stream reached its end !\n",);
        p_stream.m_status |= 0x4u32;
        p_stream.m_byte_offset += l_skip_nb_bytes;
        /* end if stream */
        return if l_skip_nb_bytes != 0 {
          l_skip_nb_bytes
        } else {
          -(1i32) as OPJ_OFF_T
        };
      }
      p_size -= l_current_skip_nb_bytes;
      l_skip_nb_bytes += l_current_skip_nb_bytes
    }
    p_stream.m_byte_offset += l_skip_nb_bytes;
    l_skip_nb_bytes
  }
}

fn opj_stream_write_skip(
  mut p_stream: &mut opj_stream_private_t,
  mut p_size: OPJ_OFF_T,
  mut p_event_mgr: &mut opj_event_mgr,
) -> OPJ_OFF_T {
  log::trace!("-- write_skip({p_size}), offset={}", p_stream.m_byte_offset);
  let mut l_is_written = 0i32;
  let mut l_current_skip_nb_bytes = 0 as OPJ_OFF_T;
  let mut l_skip_nb_bytes = 0 as OPJ_OFF_T;
  if p_stream.m_status & 0x8u32 != 0 {
    return -(1i32) as OPJ_OFF_T;
  }
  unsafe {
    /* we should flush data */
    l_is_written = opj_stream_flush(p_stream, p_event_mgr);
    if l_is_written == 0 {
      p_stream.m_status |= 0x8u32;
      p_stream.m_bytes_in_buffer = 0 as OPJ_SIZE_T;
      return -(1i32) as OPJ_OFF_T;
    }
    /* then skip */
    while p_size > 0i64 {
      /* we should do an actual skip on the media */
      l_current_skip_nb_bytes =
        p_stream.m_skip_fn.expect("non-null function pointer")(p_size, p_stream.m_user_data);
      if l_current_skip_nb_bytes == -(1i32) as OPJ_OFF_T {
        event_msg!(p_event_mgr, EVT_INFO, "Stream error!\n",);
        p_stream.m_status |= 0x8u32;
        p_stream.m_byte_offset += l_skip_nb_bytes;
        /* end if stream */
        return if l_skip_nb_bytes != 0 {
          l_skip_nb_bytes
        } else {
          -(1i32) as OPJ_OFF_T
        };
      }
      p_size -= l_current_skip_nb_bytes;
      l_skip_nb_bytes += l_current_skip_nb_bytes
    }
    p_stream.m_byte_offset += l_skip_nb_bytes;
    l_skip_nb_bytes
  }
}

pub(crate) unsafe fn opj_stream_tell(mut p_stream: *mut opj_stream_private_t) -> OPJ_OFF_T {
  let p_stream = unsafe { &mut *p_stream };
  log::trace!("-- tell() = {}", p_stream.m_byte_offset);
  p_stream.m_byte_offset
}

pub(crate) unsafe fn opj_stream_get_number_byte_left(
  mut p_stream: *mut opj_stream_private_t,
) -> OPJ_OFF_T {
  let p_stream = unsafe { &mut *p_stream };
  log::trace!("-- byte_left(), offset={}", p_stream.m_byte_offset);
  assert!(p_stream.m_byte_offset >= 0i64);
  assert!(p_stream.m_user_data_length >= p_stream.m_byte_offset as OPJ_UINT64);
  let nb = if p_stream.m_user_data_length != 0 {
    (p_stream.m_user_data_length as OPJ_OFF_T) - p_stream.m_byte_offset
  } else {
    0i64
  };
  log::trace!("-- get_number_byte_left() = {}", nb);
  nb
}

pub(crate) fn opj_stream_skip(
  mut p_stream: *mut opj_stream_private_t,
  mut p_size: OPJ_OFF_T,
  mut p_event_mgr: &mut opj_event_mgr,
) -> OPJ_OFF_T {
  let p_stream = unsafe { &mut *p_stream };
  if let Some(inner) = &mut p_stream.m_inner {
    use std::io::{Seek, SeekFrom};
    let res = match inner {
      StreamInner::Reader(reader) => reader.seek_relative(p_size).map(|_| (p_stream.m_byte_offset + p_size) as u64),
      StreamInner::Writer(writer) => writer.seek(SeekFrom::Current(p_size as _)),
    };
    return match res {
      Ok(offset) => {
        p_stream.m_byte_offset = offset as i64;
        // TODO: return number of bytes skipped.
        log::trace!("-- skip({p_size}) = {}", p_size);
        p_size
      }
      Err(err) => {
        log::trace!("Failed to skip stream: {err}");
        -1i32 as _
      }
    };
  }
  assert!(p_size >= 0i64);
  let off = p_stream.m_opj_skip.expect("non-null function pointer")(p_stream, p_size, p_event_mgr);
  log::trace!("-- skip({p_size}) = {}", off);
  off
}

fn opj_stream_read_seek(
  mut p_stream: &mut opj_stream_private_t,
  mut p_size: OPJ_OFF_T,
  mut _p_event_mgr: &mut opj_event_mgr,
) -> OPJ_BOOL {
  log::trace!("-- read_seek({p_size}), offset={}", p_stream.m_byte_offset);
  unsafe {
    p_stream.m_current_data = p_stream.m_stored_data;
    p_stream.m_bytes_in_buffer = 0 as OPJ_SIZE_T;
    if p_stream.m_seek_fn.expect("non-null function pointer")(p_size, p_stream.m_user_data) == 0 {
      p_stream.m_status |= 0x4u32;
      return 0i32;
    } else {
      /* reset stream status */
      p_stream.m_status &= !(0x4u32);
      p_stream.m_byte_offset = p_size
    }
  }
  1i32
}

fn opj_stream_write_seek(
  mut p_stream: &mut opj_stream_private_t,
  mut p_size: OPJ_OFF_T,
  mut p_event_mgr: &mut opj_event_mgr,
) -> OPJ_BOOL {
  log::trace!("-- write_seek({p_size}), offset={}", p_stream.m_byte_offset);
  unsafe {
    if opj_stream_flush(p_stream, p_event_mgr) == 0 {
      p_stream.m_status |= 0x8u32;
      return 0i32;
    }
    p_stream.m_current_data = p_stream.m_stored_data;
    p_stream.m_bytes_in_buffer = 0 as OPJ_SIZE_T;
    if p_stream.m_seek_fn.expect("non-null function pointer")(p_size, p_stream.m_user_data) == 0 {
      p_stream.m_status |= 0x8u32;
      return 0i32;
    } else {
      p_stream.m_byte_offset = p_size
    }
    1i32
  }
}

pub(crate) fn opj_stream_seek(
  mut p_stream: *mut opj_stream_private_t,
  mut p_size: OPJ_OFF_T,
  mut p_event_mgr: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let p_stream = unsafe { &mut *p_stream };
  if let Some(inner) = &mut p_stream.m_inner {
    use std::io::{Seek, SeekFrom};
    let res = match inner {
      StreamInner::Reader(reader) => reader.seek_relative(p_size - p_stream.m_byte_offset).map(|_| p_size as _),
      StreamInner::Writer(writer) => writer.seek(SeekFrom::Start(p_size as _)),
    };
    return match res {
      Ok(_) => {
        p_stream.m_byte_offset = p_size;
        log::trace!("-- seek({p_size}) = {}", 1);
        1
      }
      Err(err) => {
        log::trace!("Failed to seek stream: {err}");
        0
      }
    };
  }
  assert!(p_size >= 0i64);
  let res = p_stream.m_opj_seek.expect("non-null function pointer")(p_stream, p_size, p_event_mgr);
  log::trace!("-- seek({p_size}) = {}", res);
  res
}

pub(crate) fn opj_stream_has_seek(mut p_stream: *const opj_stream_private_t) -> OPJ_BOOL {
  let p_stream = unsafe { &*p_stream };
  (p_stream.m_seek_fn
    != Some(
      opj_stream_default_seek
        as unsafe extern "C" fn(_: OPJ_OFF_T, _: *mut core::ffi::c_void) -> OPJ_BOOL,
    )) as core::ffi::c_int
}

pub(crate) unsafe extern "C" fn opj_stream_default_read(
  mut _p_buffer: *mut core::ffi::c_void,
  mut _p_nb_bytes: OPJ_SIZE_T,
  mut _p_user_data: *mut core::ffi::c_void,
) -> OPJ_SIZE_T {
  -(1i32) as OPJ_SIZE_T
}

pub(crate) unsafe extern "C" fn opj_stream_default_write(
  mut _p_buffer: *mut core::ffi::c_void,
  mut _p_nb_bytes: OPJ_SIZE_T,
  mut _p_user_data: *mut core::ffi::c_void,
) -> OPJ_SIZE_T {
  -(1i32) as OPJ_SIZE_T
}

pub(crate) unsafe extern "C" fn opj_stream_default_skip(
  mut _p_nb_bytes: OPJ_OFF_T,
  mut _p_user_data: *mut core::ffi::c_void,
) -> OPJ_OFF_T {
  -(1i32) as OPJ_OFF_T
}

pub(crate) unsafe extern "C" fn opj_stream_default_seek(
  mut _p_nb_bytes: OPJ_OFF_T,
  mut _p_user_data: *mut core::ffi::c_void,
) -> OPJ_BOOL {
  0i32
}
