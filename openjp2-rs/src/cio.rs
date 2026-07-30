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

const U32_SIZE: usize = core::mem::size_of::<OPJ_UINT32>();
const F32_SIZE: usize = core::mem::size_of::<OPJ_FLOAT32>();
const F64_SIZE: usize = core::mem::size_of::<OPJ_FLOAT64>();

/// Write the low `nb_bytes` of `value` big-endian into the start of `dest`.
pub(crate) fn opj_write_bytes(dest: &mut [u8], value: OPJ_UINT32, nb_bytes: OPJ_UINT32) {
  let nb = nb_bytes as usize;
  assert!(nb > 0 && nb <= U32_SIZE);
  dest[..nb].copy_from_slice(&value.to_be_bytes()[U32_SIZE - nb..]);
}

/// Read `nb_bytes` big-endian from the start of `src`.
pub(crate) fn opj_read_bytes(src: &[u8], nb_bytes: OPJ_UINT32) -> OPJ_UINT32 {
  let nb = nb_bytes as usize;
  assert!(nb > 0 && nb <= U32_SIZE);
  let mut buf = [0u8; U32_SIZE];
  buf[U32_SIZE - nb..].copy_from_slice(&src[..nb]);
  OPJ_UINT32::from_be_bytes(buf)
}

pub(crate) fn opj_write_double(dest: &mut [u8], value: OPJ_FLOAT64) {
  dest[..F64_SIZE].copy_from_slice(&value.to_be_bytes());
}

pub(crate) fn opj_read_double(src: &[u8]) -> OPJ_FLOAT64 {
  let mut buf = [0u8; F64_SIZE];
  buf.copy_from_slice(&src[..F64_SIZE]);
  OPJ_FLOAT64::from_be_bytes(buf)
}

pub(crate) fn opj_write_float(dest: &mut [u8], value: OPJ_FLOAT32) {
  dest[..F32_SIZE].copy_from_slice(&value.to_be_bytes());
}

pub(crate) fn opj_read_float(src: &[u8]) -> OPJ_FLOAT32 {
  let mut buf = [0u8; F32_SIZE];
  buf.copy_from_slice(&src[..F32_SIZE]);
  OPJ_FLOAT32::from_be_bytes(buf)
}

// ---------------------------------------------------------------------------
// Temporary raw-pointer shims for `j2k.rs`. Deleted in phase 4.f once every
// marker handler takes slices.
// ---------------------------------------------------------------------------

pub(crate) fn opj_write_bytes_ptr(
  p_buffer: *mut OPJ_BYTE,
  p_value: OPJ_UINT32,
  p_nb_bytes: OPJ_UINT32,
) {
  let dest = unsafe { core::slice::from_raw_parts_mut(p_buffer, p_nb_bytes as usize) };
  opj_write_bytes(dest, p_value, p_nb_bytes)
}

pub(crate) fn opj_read_bytes_ptr(
  p_buffer: *const OPJ_BYTE,
  p_value: *mut OPJ_UINT32,
  p_nb_bytes: OPJ_UINT32,
) {
  let src = unsafe { core::slice::from_raw_parts(p_buffer, p_nb_bytes as usize) };
  let value = opj_read_bytes(src, p_nb_bytes);
  unsafe { *p_value = value }
}

pub(crate) fn opj_write_double_ptr(p_buffer: *mut OPJ_BYTE, p_value: OPJ_FLOAT64) {
  let dest = unsafe { core::slice::from_raw_parts_mut(p_buffer, F64_SIZE) };
  opj_write_double(dest, p_value)
}

pub(crate) fn opj_read_double_ptr(p_buffer: *const OPJ_BYTE, p_value: *mut OPJ_FLOAT64) {
  let src = unsafe { core::slice::from_raw_parts(p_buffer, F64_SIZE) };
  let value = opj_read_double(src);
  unsafe { *p_value = value }
}

pub(crate) fn opj_write_float_ptr(p_buffer: *mut OPJ_BYTE, p_value: OPJ_FLOAT32) {
  let dest = unsafe { core::slice::from_raw_parts_mut(p_buffer, F32_SIZE) };
  opj_write_float(dest, p_value)
}

pub(crate) fn opj_read_float_ptr(p_buffer: *const OPJ_BYTE, p_value: *mut OPJ_FLOAT32) {
  let src = unsafe { core::slice::from_raw_parts(p_buffer, F32_SIZE) };
  let value = opj_read_float(src);
  unsafe { *p_value = value }
}
