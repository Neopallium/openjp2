use smallvec::SmallVec;

/*
==========================================================
   Matric inversion interface
==========================================================
*/
/* *
 * Matrix inversion.
 */
pub(crate) fn opj_matrix_inversion_f(
  mut p_src_matrix: &mut [f32],
  mut p_dest_matrix: &mut [f32],
  mut nb_compo: usize,
) -> bool {
  let mut l_permutations = SmallVec::<[u32; 0]>::with_capacity(nb_compo);
  let mut src_tmp = SmallVec::<[f32; 0]>::with_capacity(nb_compo);
  let mut dest_tmp = SmallVec::<[f32; 0]>::with_capacity(nb_compo);
  let mut swap_area = SmallVec::<[f32; 0]>::with_capacity(nb_compo);
  l_permutations.resize(nb_compo, 0);
  src_tmp.resize(nb_compo, 0.0);
  dest_tmp.resize(nb_compo, 0.0);
  swap_area.resize(nb_compo, 0.0);
  if !opj_lup_decompose(
    p_src_matrix,
    l_permutations.as_mut_slice(),
    swap_area.as_mut_slice(),
    nb_compo,
  ) {
    return false;
  }
  opj_lup_invert(
    p_src_matrix,
    p_dest_matrix,
    nb_compo,
    l_permutations.as_slice(),
    src_tmp.as_mut_slice(),
    dest_tmp.as_mut_slice(),
    swap_area.as_mut_slice(),
  );
  true
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
fn opj_lup_decompose(
  matrix: &mut [f32],
  permutations: &mut [u32],
  p_swap_area: &mut [f32],
  nb_compo: usize,
) -> bool {
  if nb_compo == 0 {
    return false;
  }
  let l_last_column = nb_compo - 1;
  let mut l_tmp_matrix = 0usize; /* start index of the pivot row */
  let mut offset = 1usize;
  let mut l_stride = nb_compo - 1;
  // initialize permutations
  for (i, perm) in permutations.iter_mut().enumerate().take(nb_compo) {
    *perm = i as u32;
  }
  // now make a pivot with column switch
  for k in 0..l_last_column {
    // take the middle element
    let mut l_column_matrix = l_tmp_matrix + k;
    // make permutation with the biggest value in the column
    let mut p = 0.0f32;
    let mut k2 = k;
    for _i in k..nb_compo {
      let temp = matrix[l_column_matrix].abs();
      if temp > p {
        p = temp;
        k2 = _i;
      }
      // next line
      l_column_matrix += nb_compo;
    }
    // a whole rest of 0 -> non singular
    if p == 0.0 {
      return false;
    }
    // should we permute ?
    if k2 != k {
      /* k2 > k; swap the permutation indices and the entire rows. */
      permutations.swap(k, k2);
      let row_k = l_tmp_matrix;
      let row_k2 = l_tmp_matrix + (k2 - k) * nb_compo;
      p_swap_area[..nb_compo].copy_from_slice(&matrix[row_k2..row_k2 + nb_compo]);
      matrix.copy_within(row_k..row_k + nb_compo, row_k2);
      matrix[row_k..row_k + nb_compo].copy_from_slice(&p_swap_area[..nb_compo]);
    }
    // now update data in the rest of the line and line after
    let mut l_dest_matrix = l_tmp_matrix + k;
    let mut l_column_matrix = l_dest_matrix + nb_compo;
    // take the middle element
    let temp = matrix[l_dest_matrix];
    l_dest_matrix += 1;
    // now compute up data (i.e. coeff up of the diagonal).
    for _ in offset..nb_compo {
      // divide the lower column elements by the diagonal value
      // matrix[i][k] /= matrix[k][k];
      // p = matrix[i][k]
      let p = matrix[l_column_matrix] / temp;
      matrix[l_column_matrix] = p;
      l_column_matrix += 1;

      for _ in offset..nb_compo {
        // matrix[i][j] -= matrix[i][k] * matrix[k][j];
        matrix[l_column_matrix] -= p * matrix[l_dest_matrix];
        l_column_matrix += 1;
        l_dest_matrix += 1;
      }
      // come back to the k+1th element
      l_dest_matrix -= l_stride;
      // go to kth element of the next line
      l_column_matrix += k;
    }
    // offset is now k+2
    offset += 1;
    // 1 element less for stride
    l_stride -= 1;
    // next line
    l_tmp_matrix += nb_compo;
  }
  true
}
/* *
 * LUP solving
 */
fn opj_lup_solve(
  p_result: &mut [f32],
  p_matrix: &[f32],
  p_vector: &[f32],
  p_permutations: &[u32],
  nb_compo: usize,
  p_intermediate_data: &mut [f32],
) {
  assert!(nb_compo > 0);
  let mut l_line_matrix = 0usize;
  let mut l_intermediate_ptr = 0usize;
  let mut l_current_permutation_ptr = 0usize;
  for i in 0..nb_compo {
    let mut sum = 0.0f32;
    let mut l_tmp_matrix = l_line_matrix;
    let mut l_current_ptr = 0usize;
    for _ in 1..=i {
      /* sum += matrix[i][j-1] * y[j-1]; */
      sum += p_matrix[l_tmp_matrix] * p_intermediate_data[l_current_ptr];
      l_tmp_matrix += 1;
      l_current_ptr += 1;
    }
    /* y[i] = pVector[pPermutations[i]] - sum; */
    p_intermediate_data[l_intermediate_ptr] =
      p_vector[p_permutations[l_current_permutation_ptr] as usize] - sum;
    l_line_matrix += nb_compo;
    l_intermediate_ptr += 1;
    l_current_permutation_ptr += 1;
  }
  /* Back substitution: iterate k from nb_compo-1 down to 0; the diagonal of
  row k sits at index k*(nb_compo+1) (the C walks a pointer down from
  matrix + n*n - 1 by (nb_compo+1) per step).  p_result[k] is written each
  iteration, reading p_result[k+1..nb_compo] and p_intermediate_data[k]. */
  for (count, k) in (0..nb_compo).rev().enumerate() {
    let l_line_matrix = nb_compo * nb_compo - 1 - count * (nb_compo + 1);
    let mut l_tmp_matrix = l_line_matrix;
    let u = p_matrix[l_tmp_matrix];
    l_tmp_matrix += 1;
    let mut l_current_ptr = k + 1;
    let mut sum = 0.0f32;
    for _ in (k + 1)..nb_compo {
      /* sum += matrix[k][j] * x[j] */
      sum += p_matrix[l_tmp_matrix] * p_result[l_current_ptr];
      l_tmp_matrix += 1;
      l_current_ptr += 1;
    }
    /* x[k] = (y[k] - sum) / u; */
    p_result[k] = (p_intermediate_data[k] - sum) / u;
  }
}
/* *
  *LUP inversion (call with the result of lupDecompose)
  */
fn opj_lup_invert(
  p_src_matrix: &mut [f32],
  p_dest_matrix: &mut [f32],
  nb_compo: usize,
  p_permutations: &[u32],
  p_src_temp: &mut [f32],
  p_dest_temp: &mut [f32],
  p_swap_area: &mut [f32],
) {
  let nb_compo_usize = nb_compo;
  for j in 0..nb_compo {
    p_src_temp[..nb_compo_usize].fill(0.0);
    p_src_temp[j] = 1.0;
    opj_lup_solve(
      &mut p_dest_temp[..nb_compo_usize],
      &p_src_matrix[..nb_compo_usize * nb_compo_usize],
      &p_src_temp[..nb_compo_usize],
      &p_permutations[..nb_compo_usize],
      nb_compo_usize,
      &mut p_swap_area[..nb_compo_usize],
    );
    for i in 0..nb_compo_usize {
      p_dest_matrix[j + i * nb_compo_usize] = p_dest_temp[i];
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn invert(mat: &[f32], n: usize) -> Option<Vec<f32>> {
    let mut src = mat.to_vec();
    let mut dst = vec![0.0f32; n * n];
    let mut permutations = vec![0u32; n];
    let mut src_temp = vec![0.0f32; n];
    let mut dest_temp = vec![0.0f32; n];
    let mut swap_area = vec![0.0f32; n];
    if opj_lup_decompose(&mut src, &mut permutations, &mut swap_area, n) {
      opj_lup_invert(
        &mut src,
        &mut dst,
        n,
        &permutations,
        &mut src_temp,
        &mut dest_temp,
        &mut swap_area,
      );
      Some(dst)
    } else {
      None
    }
  }

  fn mat_mul(a: &[f32], b: &[f32], n: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; n * n];
    for i in 0..n {
      for j in 0..n {
        let mut acc = 0.0f32;
        for k in 0..n {
          acc += a[i * n + k] * b[k * n + j];
        }
        out[i * n + j] = acc;
      }
    }
    out
  }

  fn approx_identity(m: &[f32], n: usize) -> bool {
    for i in 0..n {
      for j in 0..n {
        let want = if i == j { 1.0 } else { 0.0 };
        if (m[i * n + j] - want).abs() > 1e-4 {
          return false;
        }
      }
    }
    true
  }

  /// Inverting the identity matrix must return the identity matrix.
  #[test]
  fn identity_inversion() {
    for n in 1..=4 {
      let mut id = vec![0.0f32; n * n];
      for i in 0..n {
        id[i * n + i] = 1.0;
      }
      let inv = invert(&id, n).expect("identity is invertible");
      assert!(approx_identity(&inv, n), "n={n}: inverse of I is I, got {inv:?}");
    }
  }

  /// A known 2x2 inverse (values from the closed form).
  #[test]
  fn known_2x2_inverse() {
    // [[2,1],[1,3]] -> [[3/5,-1/5],[-1/5,2/5]]
    let m = [2.0f32, 1.0, 1.0, 3.0];
    let inv = invert(&m, 2).expect("invertible");
    let expect = [0.6f32, -0.2, -0.2, 0.4];
    for (a, b) in inv.iter().zip(expect.iter()) {
      assert!((a - b).abs() < 1e-5, "got {inv:?}, want {expect:?}");
    }
  }

  /// M * M^-1 and M^-1 * M must be the identity (exercises the pivot swap:
  /// the first column of m3 has its maximum on row 1).
  #[test]
  fn round_trip_is_identity() {
    let cases: &[&[f32]] = &[
      &[2.0, 1.0, 1.0, 3.0],
      &[0.0, 1.0, 1.0, 2.0, 1.0, 0.0, 1.0, 0.0, 1.0],
      &[4.0, 1.0, 2.0, 1.0, 1.0, 3.0, 0.0, 0.0, 2.0, 0.0, 3.0, 1.0, 1.0, 0.0, 1.0, 2.0],
      &[5.0],
    ];
    for m in cases {
      let n = (m.len() as f64).sqrt() as usize;
      assert_eq!(n * n, m.len());
      let inv = invert(m, n).expect("matrix is invertible");
      assert!(
        approx_identity(&mat_mul(m, &inv, n), n),
        "n={n}: M*M^-1 != I for {m:?} (inv {inv:?})"
      );
      assert!(
        approx_identity(&mat_mul(&inv, m, n), n),
        "n={n}: M^-1*M != I for {m:?} (inv {inv:?})"
      );
    }
  }

  /// A matrix with a zero pivot column must be rejected (the C detects
  /// singularity only when the whole pivot column is zero), not produce
  /// garbage.
  #[test]
  fn singular_is_rejected() {
    let m = [0.0f32, 1.0, 0.0, 2.0];
    assert!(invert(&m, 2).is_none());
  }
}
