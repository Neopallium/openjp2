// The copyright in this software is being made available under the 2-clauses
// BSD License, included below. This software may be subject to other third
// party and contributor rights, including patent rights, and no such rights
// are granted under this license.
//
// Copyright (c) 2002-2014, Universite catholique de Louvain (UCL), Belgium
// Copyright (c) 2002-2014, Professor Benoit Macq
// Copyright (c) 2001-2003, David Janssens
// Copyright (c) 2002-2003, Yannick Verschueren
// Copyright (c) 2003-2007, Francois-Olivier Devaux
// Copyright (c) 2003-2014, Antonin Descampe
// Copyright (c) 2005, Herve Drolon, FreeImage Team
// Copyright (c) 2008, 2011-2012, Centre National d'Etudes Spatiales (CNES), FR
// Copyright (c) 2012, CS Systemes d'Information, France
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS `AS IS'
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED.  IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
//! Component transforms: the standard RCT/ICT kernels and the
//! custom MCT matrix kernels. See `doc/jpeg2000-design-spec.md` §3.2.
use super::openjpeg::*;

#[inline]
fn int_fix_mul(mut a: i32, mut b: i32) -> i32 {
  let mut temp = a as i64 * b as i64;
  temp += 4096i64;

  /* The C's asserts here are NDEBUG-stripped in release builds; debug_assert
  matches that behaviour (the range check remains as a debug aid). */
  debug_assert!(temp >> 13i32 <= 0x7fffffff_i64);
  debug_assert!(temp >> 13i32 >= -0x7fffffff_i64 - 1_i64);
  (temp >> 13i32) as i32
}
//
//
/* <summary> */
/* This table contains the norms of the basis function of the reversible MCT. */
/* </summary> */
static MCT_NORMS: [f64; 3] = [1.732f64, 0.8292f64, 0.8292f64];
/* <summary> */
/* This table contains the norms of the basis function of the irreversible MCT. */
/* </summary> */
static MCT_NORMS_REAL: [f64; 3] = [1.732f64, 1.805f64, 1.573f64];

pub(crate) fn mct_get_mct_norms() -> &'static [f64] {
  &MCT_NORMS
}

pub(crate) fn mct_get_mct_norms_real() -> &'static [f64] {
  &MCT_NORMS_REAL
}
/* <summary> */
/* Forward reversible MCT. */
/* </summary> */

pub(crate) fn mct_encode(c0: &mut [i32], c1: &mut [i32], c2: &mut [i32]) {
  let len = c0.len().min(c1.len()).min(c2.len());
  for i in 0..len {
    let r = c0[i];
    let g = c1[i];
    let b = c2[i];
    /* The C's int arithmetic wraps on hardware (UB in the standard);
    match it explicitly so debug builds do not panic on extreme samples. */
    c0[i] = r.wrapping_add(g.wrapping_mul(2i32)).wrapping_add(b) >> 2i32;
    c1[i] = b.wrapping_sub(g);
    c2[i] = r.wrapping_sub(g);
  }
}
/* <summary> */
/* Inverse reversible MCT. */
/* </summary> */

pub(crate) fn mct_decode(c0: &mut [i32], c1: &mut [i32], c2: &mut [i32]) {
  let len = c0.len().min(c1.len()).min(c2.len());
  for i in 0..len {
    let y = c0[i];
    let u = c1[i];
    let v = c2[i];
    let g = y.wrapping_sub(u.wrapping_add(v) >> 2i32);
    c0[i] = v.wrapping_add(g);
    c1[i] = g;
    c2[i] = u.wrapping_add(g);
  }
}
/* <summary> */
/* Get norm of basis function of reversible MCT. */
/* </summary> */

pub(crate) fn mct_getnorm(mut compno: u32) -> f64 {
  MCT_NORMS[compno as usize]
}
/* <summary> */
/* Forward irreversible MCT. */
/* </summary> */

pub(crate) fn mct_encode_real(c0: &mut [f32], c1: &mut [f32], c2: &mut [f32]) {
  let len = c0.len().min(c1.len()).min(c2.len());
  for i in 0..len {
    let r = c0[i];
    let g = c1[i];
    let b = c2[i];
    c0[i] = 0.299f32 * r + 0.587f32 * g + 0.114f32 * b;
    c1[i] = -0.16875f32 * r - 0.331260f32 * g + 0.5f32 * b;
    c2[i] = 0.5f32 * r - 0.41869f32 * g - 0.08131f32 * b;
  }
}
/* <summary> */
/* Inverse irreversible MCT. */
/* </summary> */

pub(crate) fn mct_decode_real(c0: &mut [f32], c1: &mut [f32], c2: &mut [f32]) {
  let len = c0.len().min(c1.len()).min(c2.len());
  for i in 0..len {
    let y = c0[i];
    let u = c1[i];
    let v = c2[i];
    c0[i] = y + v * 1.402f32;
    c1[i] = y - u * 0.34413f32 - v * 0.71414f32;
    c2[i] = y + u * 1.772f32;
  }
}
/* <summary> */
/* Get norm of basis function of irreversible MCT. */
/* </summary> */

pub(crate) fn mct_getnorm_real(mut compno: u32) -> f64 {
  MCT_NORMS_REAL[compno as usize]
}

#[allow(clippy::needless_range_loop)]
pub(crate) fn mct_encode_custom(
  coding_matrix: &[f32],
  n: usize,
  data: &mut [&mut [i32]],
  _is_signed: u32,
) -> OpResult {
  let nb_comp = data.len();
  let multiplicator = ((1i32) << 13i32) as f32;
  let matrix: Vec<i32> = coding_matrix[..nb_comp * nb_comp]
    .iter()
    .map(|&v| (v * multiplicator) as i32)
    .collect();
  let mut current = vec![0_i32; nb_comp];
  for i in 0..n {
    for j in 0..nb_comp {
      current[j] = data[j][i];
    }
    for j in 0..nb_comp {
      let mut acc = 0i32;
      for k in 0..nb_comp {
        acc += int_fix_mul(matrix[j * nb_comp + k], current[k]);
      }
      data[j][i] = acc;
    }
  }
  Ok(())
}

#[allow(clippy::needless_range_loop)]
pub(crate) fn mct_decode_custom(
  decoding_matrix: &[f32],
  n: usize,
  data: &mut [&mut [f32]],
  _is_signed: u32,
) -> OpResult {
  let nb_comp = data.len();
  let mut current = vec![0 as f32; nb_comp];
  for i in 0..n {
    for j in 0..nb_comp {
      current[j] = data[j][i];
    }
    for j in 0..nb_comp {
      let mut acc = 0 as f32;
      for k in 0..nb_comp {
        acc += decoding_matrix[j * nb_comp + k] * current[k];
      }
      data[j][i] = acc;
    }
  }
  Ok(())
}

pub(crate) fn calculate_norms(norms: &mut [f64], nb_comps: u32, matrix: &[f32]) {
  let nb_comps = nb_comps as usize;
  for (i, norm) in norms.iter_mut().enumerate().take(nb_comps) {
    let mut acc = 0 as f64;
    for k in 0..nb_comps {
      let value = matrix[k * nb_comps + i] as f64;
      acc += value * value;
    }
    *norm = acc.sqrt();
  }
}
