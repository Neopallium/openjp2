use ::libc;
extern "C" {
  #[no_mangle]
  fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
  #[no_mangle]
  fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
  #[no_mangle]
  fn __assert_fail(
    __assertion: *const libc::c_char,
    __file: *const libc::c_char,
    __line: libc::c_uint,
    __function: *const libc::c_char,
  ) -> !;
  #[no_mangle]
  fn opj_malloc(size: size_t) -> *mut libc::c_void;
  #[no_mangle]
  fn opj_free(m: *mut libc::c_void);
}
pub type size_t = libc::c_ulong;
pub type __int32_t = libc::c_int;
pub type __uint32_t = libc::c_uint;
pub type OPJ_BOOL = libc::c_int;
pub type OPJ_FLOAT32 = libc::c_float;
pub type OPJ_BYTE = libc::c_uchar;
pub type int32_t = __int32_t;
pub type uint32_t = __uint32_t;
pub type OPJ_INT32 = int32_t;
pub type OPJ_UINT32 = uint32_t;
/*
==========================================================
   Matric inversion interface
==========================================================
*/
/* *
 * Matrix inversion.
 */
#[no_mangle]
pub unsafe extern "C" fn opj_matrix_inversion_f(
  mut pSrcMatrix: *mut OPJ_FLOAT32,
  mut pDestMatrix: *mut OPJ_FLOAT32,
  mut nb_compo: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut l_data = 0 as *mut OPJ_BYTE;
  let mut l_permutation_size =
    nb_compo.wrapping_mul(::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong as OPJ_UINT32);
  let mut l_swap_size =
    nb_compo.wrapping_mul(::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong as OPJ_UINT32);
  let mut l_total_size =
    l_permutation_size.wrapping_add((3 as libc::c_int as libc::c_uint).wrapping_mul(l_swap_size));
  let mut lPermutations = 0 as *mut OPJ_UINT32;
  let mut l_double_data = 0 as *mut OPJ_FLOAT32;
  l_data = opj_malloc(l_total_size as size_t) as *mut OPJ_BYTE;
  if l_data.is_null() {
    return 0 as libc::c_int;
  }
  lPermutations = l_data as *mut OPJ_UINT32;
  l_double_data = l_data.offset(l_permutation_size as isize) as *mut OPJ_FLOAT32;
  memset(
    lPermutations as *mut libc::c_void,
    0 as libc::c_int,
    l_permutation_size as libc::c_ulong,
  );
  if opj_lupDecompose(pSrcMatrix, lPermutations, l_double_data, nb_compo) == 0 {
    opj_free(l_data as *mut libc::c_void);
    return 0 as libc::c_int;
  }
  opj_lupInvert(
    pSrcMatrix,
    pDestMatrix,
    nb_compo,
    lPermutations,
    l_double_data,
    l_double_data.offset(nb_compo as isize),
    l_double_data.offset((2 as libc::c_int as libc::c_uint).wrapping_mul(nb_compo) as isize),
  );
  opj_free(l_data as *mut libc::c_void);
  return 1 as libc::c_int;
}
/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2008, Jerome Fimes, Communications & Systemes <jerome.fimes@c-s.fr>
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
/* *
 * LUP decomposition
 */
/*
==========================================================
   Local functions
==========================================================
*/
unsafe extern "C" fn opj_lupDecompose(
  mut matrix: *mut OPJ_FLOAT32,
  mut permutations: *mut OPJ_UINT32,
  mut p_swap_area: *mut OPJ_FLOAT32,
  mut nb_compo: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut tmpPermutations = permutations;
  let mut dstPermutations = 0 as *mut OPJ_UINT32;
  let mut k2 = 0 as libc::c_int as OPJ_UINT32;
  let mut t: OPJ_UINT32 = 0;
  let mut temp: OPJ_FLOAT32 = 0.;
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut k: OPJ_UINT32 = 0;
  let mut p: OPJ_FLOAT32 = 0.;
  let mut lLastColum = nb_compo.wrapping_sub(1 as libc::c_int as libc::c_uint);
  let mut lSwapSize =
    nb_compo.wrapping_mul(::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong as OPJ_UINT32);
  let mut lTmpMatrix = matrix;
  let mut lColumnMatrix = 0 as *mut OPJ_FLOAT32;
  let mut lDestMatrix = 0 as *mut OPJ_FLOAT32;
  let mut offset = 1 as libc::c_int as OPJ_UINT32;
  let mut lStride = nb_compo.wrapping_sub(1 as libc::c_int as libc::c_uint);
  /*initialize permutations */
  i = 0 as libc::c_int as OPJ_UINT32;
  while i < nb_compo {
    let fresh0 = tmpPermutations;
    tmpPermutations = tmpPermutations.offset(1);
    *fresh0 = i;
    i = i.wrapping_add(1)
  }
  /* now make a pivot with column switch */
  tmpPermutations = permutations;
  k = 0 as libc::c_int as OPJ_UINT32;
  while k < lLastColum {
    p = 0.0f64 as OPJ_FLOAT32;
    /* take the middle element */
    lColumnMatrix = lTmpMatrix.offset(k as isize);
    /* make permutation with the biggest value in the column */
    i = k;
    while i < nb_compo {
      temp = if *lColumnMatrix > 0 as libc::c_int as libc::c_float {
        *lColumnMatrix
      } else {
        -*lColumnMatrix
      };
      if temp > p {
        p = temp;
        k2 = i
      }
      /* next line */
      lColumnMatrix = lColumnMatrix.offset(nb_compo as isize);
      i = i.wrapping_add(1)
    }
    /* a whole rest of 0 -> non singular */
    if p as libc::c_double == 0.0f64 {
      return 0 as libc::c_int;
    }
    /* should we permute ? */
    if k2 != k {
      /*exchange of line */
      /* k2 > k */
      dstPermutations = tmpPermutations.offset(k2 as isize).offset(-(k as isize));
      /* swap indices */
      t = *tmpPermutations;
      *tmpPermutations = *dstPermutations;
      *dstPermutations = t;
      /* and swap entire line. */
      lColumnMatrix = lTmpMatrix.offset(k2.wrapping_sub(k).wrapping_mul(nb_compo) as isize);
      memcpy(
        p_swap_area as *mut libc::c_void,
        lColumnMatrix as *const libc::c_void,
        lSwapSize as libc::c_ulong,
      );
      memcpy(
        lColumnMatrix as *mut libc::c_void,
        lTmpMatrix as *const libc::c_void,
        lSwapSize as libc::c_ulong,
      );
      memcpy(
        lTmpMatrix as *mut libc::c_void,
        p_swap_area as *const libc::c_void,
        lSwapSize as libc::c_ulong,
      );
    }
    /* now update data in the rest of the line and line after */
    lDestMatrix = lTmpMatrix.offset(k as isize);
    lColumnMatrix = lDestMatrix.offset(nb_compo as isize);
    /* take the middle element */
    let fresh1 = lDestMatrix;
    lDestMatrix = lDestMatrix.offset(1);
    temp = *fresh1;
    /* now compute up data (i.e. coeff up of the diagonal). */
    i = offset;
    while i < nb_compo {
      /*lColumnMatrix; */
      /* divide the lower column elements by the diagonal value */
      /* matrix[i][k] /= matrix[k][k]; */
      /* p = matrix[i][k] */
      p = *lColumnMatrix / temp;
      let fresh2 = lColumnMatrix;
      lColumnMatrix = lColumnMatrix.offset(1);
      *fresh2 = p;
      j = offset;
      while j < nb_compo {
        /* matrix[i][j] -= matrix[i][k] * matrix[k][j]; */
        let fresh3 = lDestMatrix;
        lDestMatrix = lDestMatrix.offset(1);
        let fresh4 = lColumnMatrix;
        lColumnMatrix = lColumnMatrix.offset(1);
        *fresh4 -= p * *fresh3;
        j = j.wrapping_add(1)
      }
      /* come back to the k+1th element */
      lDestMatrix = lDestMatrix.offset(-(lStride as isize));
      /* go to kth element of the next line */
      lColumnMatrix = lColumnMatrix.offset(k as isize);
      i = i.wrapping_add(1)
    }
    /* offset is now k+2 */
    offset = offset.wrapping_add(1);
    /* 1 element less for stride */
    lStride = lStride.wrapping_sub(1);
    /* next line */
    lTmpMatrix = lTmpMatrix.offset(nb_compo as isize);
    /* next permutation element */
    tmpPermutations = tmpPermutations.offset(1);
    k = k.wrapping_add(1)
  }
  return 1 as libc::c_int;
}
/* *
 * LUP solving
 */
unsafe extern "C" fn opj_lupSolve(
  mut pResult: *mut OPJ_FLOAT32,
  mut pMatrix: *mut OPJ_FLOAT32,
  mut pVector: *mut OPJ_FLOAT32,
  mut pPermutations: *mut OPJ_UINT32,
  mut nb_compo: OPJ_UINT32,
  mut p_intermediate_data: *mut OPJ_FLOAT32,
) {
  let mut k: OPJ_INT32 = 0;
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut sum: OPJ_FLOAT32 = 0.;
  let mut u: OPJ_FLOAT32 = 0.;
  let mut lStride = nb_compo.wrapping_add(1 as libc::c_int as libc::c_uint);
  let mut lCurrentPtr = 0 as *mut OPJ_FLOAT32;
  let mut lIntermediatePtr = 0 as *mut OPJ_FLOAT32;
  let mut lDestPtr = 0 as *mut OPJ_FLOAT32;
  let mut lTmpMatrix = 0 as *mut OPJ_FLOAT32;
  let mut lLineMatrix = pMatrix;
  let mut lBeginPtr = pResult
    .offset(nb_compo as isize)
    .offset(-(1 as libc::c_int as isize));
  let mut lGeneratedData = 0 as *mut OPJ_FLOAT32;
  let mut lCurrentPermutationPtr = pPermutations;
  lIntermediatePtr = p_intermediate_data;
  lGeneratedData = p_intermediate_data
    .offset(nb_compo as isize)
    .offset(-(1 as libc::c_int as isize));
  i = 0 as libc::c_int as OPJ_UINT32;
  while i < nb_compo {
    sum = 0.0f64 as OPJ_FLOAT32;
    lCurrentPtr = p_intermediate_data;
    lTmpMatrix = lLineMatrix;
    j = 1 as libc::c_int as OPJ_UINT32;
    while j <= i {
      /* sum += matrix[i][j-1] * y[j-1]; */
      let fresh5 = lTmpMatrix;
      lTmpMatrix = lTmpMatrix.offset(1);
      let fresh6 = lCurrentPtr;
      lCurrentPtr = lCurrentPtr.offset(1);
      sum += *fresh5 * *fresh6;
      j = j.wrapping_add(1)
    }
    /*y[i] = pVector[pPermutations[i]] - sum; */
    let fresh7 = lCurrentPermutationPtr;
    lCurrentPermutationPtr = lCurrentPermutationPtr.offset(1);
    let fresh8 = lIntermediatePtr;
    lIntermediatePtr = lIntermediatePtr.offset(1);
    *fresh8 = *pVector.offset(*fresh7 as isize) - sum;
    lLineMatrix = lLineMatrix.offset(nb_compo as isize);
    i = i.wrapping_add(1)
  }
  /* we take the last point of the matrix */
  lLineMatrix = pMatrix
    .offset(nb_compo.wrapping_mul(nb_compo) as isize)
    .offset(-(1 as libc::c_int as isize));
  /* and we take after the last point of the destination vector */
  lDestPtr = pResult.offset(nb_compo as isize);
  if nb_compo != 0 as libc::c_int as libc::c_uint {
  } else {
    __assert_fail(b"nb_compo != 0\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/invert.c\x00" as
                          *const u8 as *const libc::c_char,
                      252 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 104],
                                                &[libc::c_char; 104]>(b"void opj_lupSolve(OPJ_FLOAT32 *, OPJ_FLOAT32 *, OPJ_FLOAT32 *, OPJ_UINT32 *, OPJ_UINT32, OPJ_FLOAT32 *)\x00")).as_ptr());
  }
  k = nb_compo as OPJ_INT32 - 1 as libc::c_int;
  while k != -(1 as libc::c_int) {
    sum = 0.0f64 as OPJ_FLOAT32;
    lTmpMatrix = lLineMatrix;
    let fresh9 = lTmpMatrix;
    lTmpMatrix = lTmpMatrix.offset(1);
    u = *fresh9;
    let fresh10 = lDestPtr;
    lDestPtr = lDestPtr.offset(-1);
    lCurrentPtr = fresh10;
    j = (k + 1 as libc::c_int) as OPJ_UINT32;
    while j < nb_compo {
      /* sum += matrix[k][j] * x[j] */
      let fresh11 = lTmpMatrix;
      lTmpMatrix = lTmpMatrix.offset(1);
      let fresh12 = lCurrentPtr;
      lCurrentPtr = lCurrentPtr.offset(1);
      sum += *fresh11 * *fresh12;
      j = j.wrapping_add(1)
    }
    /*x[k] = (y[k] - sum) / u; */
    let fresh13 = lGeneratedData;
    lGeneratedData = lGeneratedData.offset(-1);
    let fresh14 = lBeginPtr;
    lBeginPtr = lBeginPtr.offset(-1);
    *fresh14 = (*fresh13 - sum) / u;
    lLineMatrix = lLineMatrix.offset(-(lStride as isize));
    k -= 1
  }
}
/* *
 *LUP inversion (call with the result of lupDecompose)
 */
unsafe extern "C" fn opj_lupInvert(
  mut pSrcMatrix: *mut OPJ_FLOAT32,
  mut pDestMatrix: *mut OPJ_FLOAT32,
  mut nb_compo: OPJ_UINT32,
  mut pPermutations: *mut OPJ_UINT32,
  mut p_src_temp: *mut OPJ_FLOAT32,
  mut p_dest_temp: *mut OPJ_FLOAT32,
  mut p_swap_area: *mut OPJ_FLOAT32,
) {
  let mut j: OPJ_UINT32 = 0;
  let mut i: OPJ_UINT32 = 0;
  let mut lCurrentPtr = 0 as *mut OPJ_FLOAT32;
  let mut lLineMatrix = pDestMatrix;
  let mut lSwapSize =
    nb_compo.wrapping_mul(::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong as OPJ_UINT32);
  j = 0 as libc::c_int as OPJ_UINT32;
  while j < nb_compo {
    let fresh15 = lLineMatrix;
    lLineMatrix = lLineMatrix.offset(1);
    lCurrentPtr = fresh15;
    memset(
      p_src_temp as *mut libc::c_void,
      0 as libc::c_int,
      lSwapSize as libc::c_ulong,
    );
    *p_src_temp.offset(j as isize) = 1.0f64 as OPJ_FLOAT32;
    opj_lupSolve(
      p_dest_temp,
      pSrcMatrix,
      p_src_temp,
      pPermutations,
      nb_compo,
      p_swap_area,
    );
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < nb_compo {
      *lCurrentPtr = *p_dest_temp.offset(i as isize);
      lCurrentPtr = lCurrentPtr.offset(nb_compo as isize);
      i = i.wrapping_add(1)
    }
    j = j.wrapping_add(1)
  }
}
