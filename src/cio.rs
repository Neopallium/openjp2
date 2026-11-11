//! Big-endian byte I/O helpers (`read_bytes`/`write_bytes`).
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

const U32_SIZE: usize = core::mem::size_of::<u32>();
const F32_SIZE: usize = core::mem::size_of::<f32>();
const F64_SIZE: usize = core::mem::size_of::<f64>();

/// Write the low `nb_bytes` of `value` big-endian into the start of `dest`.
pub(crate) fn write_bytes(dest: &mut [u8], value: u32, nb_bytes: u32) {
  let nb = nb_bytes as usize;
  debug_assert!(nb > 0 && nb <= U32_SIZE);
  dest[..nb].copy_from_slice(&value.to_be_bytes()[U32_SIZE - nb..]);
}

/// Read `nb_bytes` big-endian from the start of `src`.
pub(crate) fn read_bytes(src: &[u8], nb_bytes: u32) -> u32 {
  let nb = nb_bytes as usize;
  debug_assert!(nb > 0 && nb <= U32_SIZE);
  let mut buf = [0u8; U32_SIZE];
  buf[U32_SIZE - nb..].copy_from_slice(&src[..nb]);
  u32::from_be_bytes(buf)
}

pub(crate) fn write_double(dest: &mut [u8], value: f64) {
  dest[..F64_SIZE].copy_from_slice(&value.to_be_bytes());
}

pub(crate) fn read_double(src: &[u8]) -> f64 {
  let mut buf = [0u8; F64_SIZE];
  buf.copy_from_slice(&src[..F64_SIZE]);
  f64::from_be_bytes(buf)
}

pub(crate) fn write_float(dest: &mut [u8], value: f32) {
  dest[..F32_SIZE].copy_from_slice(&value.to_be_bytes());
}

pub(crate) fn read_float(src: &[u8]) -> f32 {
  let mut buf = [0u8; F32_SIZE];
  buf.copy_from_slice(&src[..F32_SIZE]);
  f32::from_be_bytes(buf)
}
