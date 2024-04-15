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

use super::openjpeg::*;

extern "C" {
  fn memcpy(
    _: *mut core::ffi::c_void,
    _: *const core::ffi::c_void,
    _: usize,
  ) -> *mut core::ffi::c_void;
}

/* ----------------------------------------------------------------------- */
/* ----------------------------------------------------------------------- */
pub(crate) unsafe fn opj_write_bytes_BE(
  mut p_buffer: *mut OPJ_BYTE,
  mut p_value: OPJ_UINT32,
  mut p_nb_bytes: OPJ_UINT32,
) {
  let mut l_data_ptr = (&mut p_value as *mut OPJ_UINT32 as *const OPJ_BYTE)
    .add(core::mem::size_of::<OPJ_UINT32>())
    .offset(-(p_nb_bytes as isize));
  assert!(p_nb_bytes > 0u32 && p_nb_bytes as usize <= core::mem::size_of::<OPJ_UINT32>());
  memcpy(
    p_buffer as *mut core::ffi::c_void,
    l_data_ptr as *const core::ffi::c_void,
    p_nb_bytes as usize,
  );
}

pub(crate) unsafe fn opj_write_bytes_LE(
  mut p_buffer: *mut OPJ_BYTE,
  mut p_value: OPJ_UINT32,
  mut p_nb_bytes: OPJ_UINT32,
) {
  let mut l_data_ptr = (&mut p_value as *mut OPJ_UINT32 as *const OPJ_BYTE)
    .offset(p_nb_bytes as isize)
    .offset(-1);
  let mut i: OPJ_UINT32 = 0;
  assert!(p_nb_bytes > 0u32 && p_nb_bytes as usize <= core::mem::size_of::<OPJ_UINT32>());
  i = 0 as OPJ_UINT32;
  while i < p_nb_bytes {
    let fresh0 = l_data_ptr;
    l_data_ptr = l_data_ptr.offset(-1);
    let fresh1 = p_buffer;
    p_buffer = p_buffer.offset(1);
    *fresh1 = *fresh0;
    i += 1;
  }
}

pub(crate) unsafe fn opj_read_bytes_BE(
  mut p_buffer: *const OPJ_BYTE,
  mut p_value: *mut OPJ_UINT32,
  mut p_nb_bytes: OPJ_UINT32,
) {
  let mut l_data_ptr = p_value as *mut OPJ_BYTE;
  assert!(p_nb_bytes > 0u32 && p_nb_bytes as usize <= core::mem::size_of::<OPJ_UINT32>());
  *p_value = 0 as OPJ_UINT32;
  memcpy(
    l_data_ptr
      .add(core::mem::size_of::<OPJ_UINT32>())
      .offset(-(p_nb_bytes as isize)) as *mut core::ffi::c_void,
    p_buffer as *const core::ffi::c_void,
    p_nb_bytes as usize,
  );
}

pub(crate) unsafe fn opj_read_bytes_LE(
  mut p_buffer: *const OPJ_BYTE,
  mut p_value: *mut OPJ_UINT32,
  mut p_nb_bytes: OPJ_UINT32,
) {
  let mut l_data_ptr = (p_value as *mut OPJ_BYTE)
    .offset(p_nb_bytes as isize)
    .offset(-1);
  let mut i: OPJ_UINT32 = 0;
  assert!(p_nb_bytes > 0u32 && p_nb_bytes as usize <= core::mem::size_of::<OPJ_UINT32>());
  *p_value = 0 as OPJ_UINT32;
  i = 0 as OPJ_UINT32;
  while i < p_nb_bytes {
    let fresh2 = p_buffer;
    p_buffer = p_buffer.offset(1);
    let fresh3 = l_data_ptr;
    l_data_ptr = l_data_ptr.offset(-1);
    *fresh3 = *fresh2;
    i += 1;
  }
}

pub(crate) unsafe fn opj_write_double_BE(mut p_buffer: *mut OPJ_BYTE, mut p_value: OPJ_FLOAT64) {
  let mut l_data_ptr = &mut p_value as *mut OPJ_FLOAT64 as *const OPJ_BYTE;
  memcpy(
    p_buffer as *mut core::ffi::c_void,
    l_data_ptr as *const core::ffi::c_void,
    core::mem::size_of::<OPJ_FLOAT64>(),
  );
}

pub(crate) unsafe fn opj_write_double_LE(mut p_buffer: *mut OPJ_BYTE, mut p_value: OPJ_FLOAT64) {
  let mut l_data_ptr = (&mut p_value as *mut OPJ_FLOAT64 as *const OPJ_BYTE)
    .add(core::mem::size_of::<OPJ_FLOAT64>())
    .offset(-1);
  let mut i: OPJ_UINT32 = 0;
  i = 0 as OPJ_UINT32;
  while (i as usize) < core::mem::size_of::<OPJ_FLOAT64>() {
    let fresh4 = l_data_ptr;
    l_data_ptr = l_data_ptr.offset(-1);
    let fresh5 = p_buffer;
    p_buffer = p_buffer.offset(1);
    *fresh5 = *fresh4;
    i += 1;
  }
}

pub(crate) unsafe fn opj_read_double_BE(
  mut p_buffer: *const OPJ_BYTE,
  mut p_value: *mut OPJ_FLOAT64,
) {
  let mut l_data_ptr = p_value as *mut OPJ_BYTE;
  memcpy(
    l_data_ptr as *mut core::ffi::c_void,
    p_buffer as *const core::ffi::c_void,
    core::mem::size_of::<OPJ_FLOAT64>(),
  );
}

pub(crate) unsafe fn opj_read_double_LE(
  mut p_buffer: *const OPJ_BYTE,
  mut p_value: *mut OPJ_FLOAT64,
) {
  let mut l_data_ptr = (p_value as *mut OPJ_BYTE)
    .add(core::mem::size_of::<OPJ_FLOAT64>())
    .offset(-1);
  let mut i: OPJ_UINT32 = 0;
  i = 0 as OPJ_UINT32;
  while (i as usize) < core::mem::size_of::<OPJ_FLOAT64>() {
    let fresh6 = p_buffer;
    p_buffer = p_buffer.offset(1);
    let fresh7 = l_data_ptr;
    l_data_ptr = l_data_ptr.offset(-1);
    *fresh7 = *fresh6;
    i += 1;
  }
}

pub(crate) unsafe fn opj_write_float_BE(mut p_buffer: *mut OPJ_BYTE, mut p_value: OPJ_FLOAT32) {
  let mut l_data_ptr = &mut p_value as *mut OPJ_FLOAT32 as *const OPJ_BYTE;
  memcpy(
    p_buffer as *mut core::ffi::c_void,
    l_data_ptr as *const core::ffi::c_void,
    core::mem::size_of::<OPJ_FLOAT32>(),
  );
}

pub(crate) unsafe fn opj_write_float_LE(mut p_buffer: *mut OPJ_BYTE, mut p_value: OPJ_FLOAT32) {
  let mut l_data_ptr = (&mut p_value as *mut OPJ_FLOAT32 as *const OPJ_BYTE)
    .add(core::mem::size_of::<OPJ_FLOAT32>())
    .offset(-1);
  let mut i: OPJ_UINT32 = 0;
  i = 0 as OPJ_UINT32;
  while (i as usize) < core::mem::size_of::<OPJ_FLOAT32>() {
    let fresh8 = l_data_ptr;
    l_data_ptr = l_data_ptr.offset(-1);
    let fresh9 = p_buffer;
    p_buffer = p_buffer.offset(1);
    *fresh9 = *fresh8;
    i += 1;
  }
}

pub(crate) unsafe fn opj_read_float_BE(
  mut p_buffer: *const OPJ_BYTE,
  mut p_value: *mut OPJ_FLOAT32,
) {
  let mut l_data_ptr = p_value as *mut OPJ_BYTE;
  memcpy(
    l_data_ptr as *mut core::ffi::c_void,
    p_buffer as *const core::ffi::c_void,
    core::mem::size_of::<OPJ_FLOAT32>(),
  );
}

pub(crate) unsafe fn opj_read_float_LE(
  mut p_buffer: *const OPJ_BYTE,
  mut p_value: *mut OPJ_FLOAT32,
) {
  let mut l_data_ptr = (p_value as *mut OPJ_BYTE)
    .add(core::mem::size_of::<OPJ_FLOAT32>())
    .offset(-1);
  let mut i: OPJ_UINT32 = 0;
  i = 0 as OPJ_UINT32;
  while (i as usize) < core::mem::size_of::<OPJ_FLOAT32>() {
    let fresh10 = p_buffer;
    p_buffer = p_buffer.offset(1);
    let fresh11 = l_data_ptr;
    l_data_ptr = l_data_ptr.offset(-1);
    *fresh11 = *fresh10;
    i += 1;
  }
}
