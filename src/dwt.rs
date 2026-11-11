use core::cell::Cell;

use super::math::*;
use super::openjpeg::*;
use super::par;

thread_local! {
  /// Per-worker scratch memory for the parallel 5-3 passes (one column/row
  /// at a time; sized to the current resolution).
  static DWT_SCRATCH: core::cell::RefCell<Vec<core::cell::Cell<i32>>> =
    const { core::cell::RefCell::new(Vec::new()) };
  /// Per-worker scratch memory for the parallel 9-7 passes (the wavelet
  /// cells of one row/column group, sized to the current resolution).
  static DWT_SCRATCH_F32: core::cell::RefCell<Vec<core::cell::Cell<f32>>> =
    const { core::cell::RefCell::new(Vec::new()) };
}
use super::sparse_array::{sparse_array_read, sparse_array_write, SparseArray, SparseSample};

/// Which wavelet a forward transform runs. This replaces the pair of
/// `Option<unsafe extern "C" fn(*mut c_void, ...)>` tables the C code dispatches through, so the
/// kernels can keep their real element types (`i32` for 5-3, `f32` for 9-7)
/// instead of being erased to `void*`.
#[derive(Copy, Clone, PartialEq, Eq)]
enum ForwardWavelet {
  /// Reversible 5-3, operating on `i32` samples.
  Reversible53,
  /// Irreversible 9-7, operating on `f32` samples.
  Irreversible97,
}

/* * @name Local data structures */
/*@{*/

pub const VREG_INT_COUNT: usize = 4;

pub const PARALLEL_COLS_53: usize = 2 * VREG_INT_COUNT;

pub type Dwt<'a> = dwt_local<'a>;
/// Scratch buffer plus band geometry for one 5-3 inverse pass.
///
/// The horizontal and vertical passes of a resolution level alias the *same* scratch buffer, so
/// it is held as `&[Cell<i32>]`: that keeps `dwt_local` `Copy` and lets both views write through
/// it without any `unsafe`.
#[derive(Copy, Clone)]
pub struct dwt_local<'a> {
  pub mem: &'a [Cell<i32>],
  /// number of elements in high pass band
  pub dn: i32,
  /// number of elements in low pass band
  pub sn: i32,
  /// 0 = start on even coord, 1 = start on odd coord
  pub cas: i32,
}

impl dwt_local<'_> {
  fn sn_idx(&self, i: i32) -> i32 {
    i.clamp(0, self.sn - 1)
  }

  fn dn_idx(&self, i: i32) -> i32 {
    i.clamp(0, self.dn - 1)
  }

  fn s_idx(&self, i: i32, chunk: isize) -> isize {
    (i * 2) as isize * chunk
  }

  fn d_idx(&self, i: i32, chunk: isize) -> isize {
    (1 + (i * 2)) as isize * chunk
  }

  fn s_clamped_idx(&self, i: i32, chunk: isize) -> isize {
    self.s_idx(self.sn_idx(i), chunk)
  }

  fn d_clamped_idx(&self, i: i32, chunk: isize) -> isize {
    self.d_idx(self.dn_idx(i), chunk)
  }

  fn ss_clamped_idx(&self, i: i32, chunk: isize) -> isize {
    self.s_idx(self.dn_idx(i), chunk)
  }

  fn dd_clamped_idx(&self, i: i32, chunk: isize) -> isize {
    self.d_idx(self.sn_idx(i), chunk)
  }

  fn m(&self, i: isize) -> i32 {
    self.mem[i as usize].get()
  }

  fn m_set(&self, i: isize, value: i32) {
    self.mem[i as usize].set(value)
  }

  fn m_add(&self, i: isize, value: i32) {
    let cell = &self.mem[i as usize];
    cell.set(cell.get().wrapping_add(value))
  }

  fn m_sub(&self, i: isize, value: i32) {
    let cell = &self.mem[i as usize];
    cell.set(cell.get().wrapping_sub(value))
  }

  fn s(&self, i: i32) -> i32 {
    self.m(self.s_idx(i, 1))
  }

  fn d(&self, i: i32) -> i32 {
    self.m(self.d_idx(i, 1))
  }

  fn s_(&self, i: i32) -> i32 {
    self.m(self.s_clamped_idx(i, 1))
  }

  fn d_(&self, i: i32) -> i32 {
    self.m(self.d_clamped_idx(i, 1))
  }

  fn ss_(&self, i: i32) -> i32 {
    self.m(self.ss_clamped_idx(i, 1))
  }

  fn dd_(&self, i: i32) -> i32 {
    self.m(self.dd_clamped_idx(i, 1))
  }

  /// Inverse 5-3 wavelet transform in 1-D.
  fn decode_1(&self) {
    if self.cas == 0 {
      if self.dn > 0 || self.sn > 1 {
        for i in 0..self.sn {
          let val = self
            .d_(i - 1)
            .wrapping_add(self.d_(i))
            .wrapping_add(2)
            >> 2;
          self.m_sub(self.s_idx(i, 1), val);
        }
        for i in 0..self.dn {
          let val = self.s_(i).wrapping_add(self.s_(i + 1)) >> 1;
          self.m_add(self.d_idx(i, 1), val);
        }
      }
    } else {
      if self.sn == 0 && self.dn == 1 {
        self.m_set(self.s_idx(0, 1), self.s(0) / 2);
      } else {
        for i in 0..self.sn {
          let val = self
            .ss_(i)
            .wrapping_add(self.ss_(i + 1))
            .wrapping_add(2)
            >> 2;
          self.m_sub(self.d_idx(i, 1), val);
        }
        for i in 0..self.dn {
          let val = self.dd_(i).wrapping_add(self.dd_(i - 1)) >> 1;
          self.m_add(self.s_idx(i, 1), val);
        }
      }
    }
  }

  fn decode_partial_1(
    &self,
    win_l_x0: i32,
    win_l_x1: i32,
    win_h_x0: i32,
    win_h_x1: i32,
  ) {
    if self.cas == 0 {
      if self.dn > 0 || self.sn > 1 {
        for i in win_l_x0..win_l_x1 {
          let val = self
            .d_(i - 1)
            .wrapping_add(self.d_(i))
            .wrapping_add(2)
            >> 2;
          self.m_sub(self.s_idx(i, 1), val);
        }
        for i in win_h_x0..win_h_x1 {
          let val = self.s_(i).wrapping_add(self.s_(i + 1)) >> 1;
          self.m_add(self.d_idx(i, 1), val);
        }
      }
    } else {
      if self.sn == 0 && self.dn == 1 {
        self.m_set(self.s_idx(0, 1), self.s(0) / 2);
      } else {
        for i in win_l_x0..win_l_x1 {
          let val = self
            .d(i)
            .wrapping_sub(self.ss_(i).wrapping_add(self.ss_(i + 1)).wrapping_add(2) >> 2);
          self.m_set(self.d_idx(i, 1), val);
        }
        for i in win_h_x0..win_h_x1 {
          let val = self
            .s(i)
            .wrapping_add(self.dd_(i).wrapping_add(self.dd_(i - 1)) >> 1);
          self.m_set(self.s_idx(i, 1), val);
        }
      }
    }
  }

  fn decode_partial_1_parallel(
    &self,
    win_l_x0: i32,
    win_l_x1: i32,
    win_h_x0: i32,
    win_h_x1: i32,
  ) {
    if self.cas == 0 {
      if self.dn > 0 || self.sn > 1 {
        for i in win_l_x0..win_l_x1 {
          let s_idx = self.s_idx(i, 4);
          let d0_idx = self.d_clamped_idx(i - 1, 4);
          let d1_idx = self.d_clamped_idx(i, 4);
          for off in 0..4 {
            let val = self.m(s_idx + off).wrapping_sub(
              self
                .m(d0_idx + off)
                .wrapping_add(self.m(d1_idx + off))
                .wrapping_add(2)
                >> 2,
            );
            self.m_set(s_idx + off, val);
          }
        }
        for i in win_h_x0..win_h_x1 {
          let d_idx = self.d_idx(i, 4);
          let s0_idx = self.s_clamped_idx(i, 4);
          let s1_idx = self.s_clamped_idx(i + 1, 4);
          for off in 0..4 {
            let val = self
              .m(d_idx + off)
              .wrapping_add(self.m(s0_idx + off).wrapping_add(self.m(s1_idx + off)) >> 1);
            self.m_set(d_idx + off, val);
          }
        }
      }
    } else {
      if self.sn == 0 && self.dn == 1 {
        let s_idx = self.s_idx(0, 4);
        for off in 0..4 {
          self.m_set(s_idx + off, self.m(s_idx + off) / 2);
        }
      } else {
        for i in win_l_x0..win_l_x1 {
          let d_idx = self.d_idx(i, 4);
          let s0_idx = self.ss_clamped_idx(i, 4);
          let s1_idx = self.ss_clamped_idx(i + 1, 4);
          for off in 0..4 {
            let val = self.m(d_idx + off).wrapping_sub(
              self
                .m(s0_idx + off)
                .wrapping_add(self.m(s1_idx + off))
                .wrapping_add(2)
                >> 2,
            );
            self.m_set(d_idx + off, val);
          }
        }
        for i in win_h_x0..win_h_x1 {
          let s_idx = self.s_idx(i, 4);
          let d0_idx = self.dd_clamped_idx(i, 4);
          let d1_idx = self.dd_clamped_idx(i - 1, 4);
          for off in 0..4 {
            let val = self
              .m(s_idx + off)
              .wrapping_add(self.m(d0_idx + off).wrapping_add(self.m(d1_idx + off)) >> 1);
            self.m_set(s_idx + off, val);
          }
        }
      }
    }
  }
}

pub const NB_ELTS_V8: u32 = 8;
/// Size in bytes of what the C code calls an `opj_v8_t`: one group of `NB_ELTS_V8` floats.
const V8_SIZE: usize = NB_ELTS_V8 as usize * core::mem::size_of::<f32>();

pub type V8Dwt<'a> = v8dwt_local<'a>;
/// Scratch buffer plus band geometry for one 9-7 inverse pass.
///
/// `wavelet` is a *flat* run of `f32`s viewed in groups of `NB_ELTS_V8`: what the C code
/// calls `wavelet[k]` is `wavelet[NB_ELTS_V8 * k .. NB_ELTS_V8 * (k + 1)]` here. As with
/// [`dwt_local`], the horizontal and vertical views of a resolution level share one buffer, hence
/// `Cell`.
#[derive(Copy, Clone)]
pub struct v8dwt_local<'a> {
  pub wavelet: &'a [Cell<f32>],
  /// number of elements in high pass band
  pub dn: i32,
  /// number of elements in low pass band
  pub sn: i32,
  /// 0 = start on even coord, 1 = start on odd coord
  pub cas: i32,
  /// start coord in low pass band
  pub win_l_x0: u32,
  /// end coord in low pass band
  pub win_l_x1: u32,
  /// start coord in high pass band
  pub win_h_x0: u32,
  /// end coord in high pass band
  pub win_h_x1: u32,
}

/* From table F.4 from the standard */
const DWT_ALPHA: f32 = -1.586_134_3;
const DWT_BETA: f32 = -0.052980118;
const DWT_GAMMA: f32 = 0.882_911_1;
const DWT_DELTA: f32 = 0.443_506_87;
const DWT_K: f32 = 1.230_174_1;
const DWT_INV_K: f32 = 1.0 / 1.230_174_1;

/* <summary>                                                              */
/* This table contains the norms of the 5-3 wavelets for different bands. */
/* </summary>                                                             */
/* Note: upstream issue #493 - these arrays cover only 10 resolution
levels; see https://github.com/uclouvain/openjpeg/issues/493 */
const DWT_NORMS: [[f64; 10]; 4] = [
  [
    1.000, 1.500, 2.750, 5.375, 10.68, 21.34, 42.67, 85.33, 170.7, 341.3,
  ],
  [
    1.038, 1.592, 2.919, 5.703, 11.33, 22.64, 45.25, 90.48, 180.9, 0.,
  ],
  [
    1.038, 1.592, 2.919, 5.703, 11.33, 22.64, 45.25, 90.48, 180.9, 0.,
  ],
  [
    0.7186, 0.9218, 1.586, 3.043, 6.019, 12.01, 24.00, 47.97, 95.93, 0.,
  ],
];
/* <summary>                                                              */
/* This table contains the norms of the 9-7 wavelets for different bands. */
/* </summary>                                                             */
/* Note: upstream issue #493 - these arrays cover only 10 resolution
levels; see https://github.com/uclouvain/openjpeg/issues/493 */
const DWT_NORMS_REAL: [[f64; 10]; 4] = [
  [
    1.000, 1.965, 4.177, 8.403, 16.90, 33.84, 67.69, 135.3, 270.6, 540.9,
  ],
  [
    2.022, 3.989, 8.355, 17.04, 34.27, 68.63, 137.3, 274.6, 549.0, 0.,
  ],
  [
    2.022, 3.989, 8.355, 17.04, 34.27, 68.63, 137.3, 274.6, 549.0, 0.,
  ],
  [
    2.080, 3.865, 8.307, 17.18, 34.71, 69.59, 139.3, 278.6, 557.2, 0.,
  ],
];

/*@}*/
/* * @name Local static functions */
/*@{*/
/* *
Forward lazy transform (horizontal)
*/
/*
==========================================================
   local functions
==========================================================
*/
/* <summary>                             */
/* Forward lazy transform (horizontal).  */
/* </summary>                            */
/// Deinterleave a 1-D lazy transform: copy the low-pass (`sn`) and high-pass (`dn`) samples out
/// of the interleaved `a` buffer into `b`, honouring the parity (`cas`) of the first sample.
/// Generic so the 9-7 pass can reuse it over its float staging buffers.
fn opj_dwt_deinterleave_h<T: Copy>(
  a: &[T],
  a_start: usize,
  b: &mut [T],
  b_start: usize,
  dn: i32,
  sn: i32,
  cas: i32,
) {
  let sn = sn as usize;
  let dn = dn as usize;
  let cas = cas as usize;
  let mut i = 0usize;
  while i < sn {
    b[b_start + i] = a[a_start + cas + 2 * i];
    i += 1
  }
  let mut i = 0usize;
  while i < dn {
    b[b_start + sn + i] = a[a_start + 1 - cas + 2 * i];
    i += 1
  }
}
/* <summary>                             */
/* Inverse lazy transform (horizontal).  */
/* </summary>                            */
/// Inverse lazy transform (horizontal): read one interleaved row of `sn + dn` samples from the
/// tile buffer and split it into the low- and high-pass halves of the scratch, honouring `cas`.
fn opj_dwt_interleave_h(h: &Dwt, a: &[i32]) {
  let mut bi = h.cas as usize;
  for i in 0..h.sn {
    h.mem[bi].set(a[i as usize]);
    bi += 2
  }
  bi = (1 - h.cas) as usize;
  for i in 0..h.dn {
    h.mem[bi].set(a[(h.sn + i) as usize]);
    bi += 2
  }
}

/* <summary>                             */
/* Inverse lazy transform (vertical).    */
/* </summary>                            */
/// Inverse lazy transform (vertical): read one strided column of `sn + dn` samples from the tile
/// buffer (`x` = row stride) and split it into the low- and high-pass halves of the scratch.
fn opj_dwt_interleave_v(v: &Dwt, a: &[i32], a_start: usize, x: i32) {
  let mut bi = v.cas as usize;
  for i in 0..v.sn {
    v.mem[bi].set(a[a_start + (i as usize) * (x as usize)]);
    bi += 2
  }
  bi = (1 - v.cas) as usize;
  for i in 0..v.dn {
    v.mem[bi].set(a[a_start + ((v.sn + i) as usize) * (x as usize)]);
    bi += 2
  }
}

/* <summary>                            */
/* Inverse 5-3 wavelet transform in 1-D for one row. */
/* </summary>                           */
/* Performs interleave, inverse wavelet transform and copy back to buffer */
#[cfg(feature = "simd")]
fn opj_idwt53_v_simd(
  sn: i32,
  dn: i32,
  tiledp: &mut [i32],
  col_start: usize,
  stride: usize,
  nb_cols: i32,
) -> i32 {
  use wide::i32x4;
  let len = (sn + dn) as usize;
  let two = i32x4::splat(2);
  #[inline(always)]
  fn load(buf: &[i32], k: usize, c: usize, stride: usize) -> i32x4 {
    i32x4::from([buf[c + k * stride], buf[c + 1 + k * stride], buf[c + 2 + k * stride], buf[c + 3 + k * stride]])
  }
  /* Interleaved results for a 4-column group (the same layout the
  scalar path's scratch + write-back produces).  Writing the results to
  a scratch first avoids the read-after-write aliasing between the
  plain input rows (0..sn-1 = low, sn.. = high) and the interleaved
  output rows (2i = low, 2i+1 = high) - exactly how the C reference's
  SIMD kernels avoid it. */
  let mut scratch = vec![i32x4::splat(0); len];
  let mut c = col_start;
  let end = col_start + (nb_cols as usize) - (nb_cols as usize) % 4;
  while c < end {
    // low pass: s'[i] = s[i] - (d[i-1] + d[i] + 2) >> 2
    // d reads clamp to dn-1 when sn > dn (odd level heights), exactly
    // like the scalar path's dn_idx clamp.
    let d_idx = |i: usize| sn as usize + i.min(dn as usize - 1);
    let mut prev_d = load(tiledp, d_idx(0), c, stride); // d(-1) = d(0)
    for i in 0..sn as usize {
      let s = load(tiledp, i, c, stride);
      let d = load(tiledp, d_idx(i), c, stride);
      let val = (prev_d + d + two) >> 2;
      scratch[2 * i] = s - val;
      prev_d = d;
    }
    // high pass: d'[i] = d[i] + (s'[i] + s'[i+1]) >> 1
    for i in 0..dn as usize {
      let s0 = scratch[2 * i];
      let s1 = if i + 1 >= sn as usize { scratch[2 * (sn as usize - 1)] } else { scratch[2 * (i + 1)] };
      let d = load(tiledp, sn as usize + i, c, stride);
      scratch[2 * i + 1] = d + ((s0 + s1) >> 1);
    }
    // write back: row k of the group = scratch[k]
    for (k, v) in scratch.iter().enumerate() {
      let arr = v.to_array();
      let base = c + k * stride;
      tiledp[base] = arr[0];
      tiledp[base + 1] = arr[1];
      tiledp[base + 2] = arr[2];
      tiledp[base + 3] = arr[3];
    }
    c += 4;
  }
  (end - col_start) as i32
}

/// Inverse 5-3 wavelet transform in 1-D for one row: interleave, transform, copy back.
fn opj_idwt53_h(dwt: &Dwt, tiledp: &mut [i32], row_start: usize) {
  let len = (dwt.sn + dwt.dn) as usize;
  opj_dwt_interleave_h(dwt, &tiledp[row_start..row_start + len]);
  dwt.decode_1();
  for k in 0..len {
    tiledp[row_start + k] = dwt.mem[k].get();
  }
}

/* <summary>                            */
/* Inverse vertical 5-3 wavelet transform in 1-D for several columns. */
/* </summary>                           */
/* Performs interleave, inverse wavelet transform and copy back to buffer */
/// Inverse vertical 5-3 wavelet transform in 1-D for several columns: per column, interleave,
/// transform, copy back. `col_start` is the tile-buffer offset of the first column.
fn opj_idwt53_v(
  dwt: &Dwt,
  tiledp: &mut [i32],
  col_start: usize,
  stride: usize,
  nb_cols: i32,
) {
  let mut c = 0i32;
  /* SIMD path (feature `simd`): cas == 0 (even-origin columns) only;
  odd origins and short columns use the scalar path. */
  #[cfg(feature = "simd")]
  if dwt.cas == 0 && nb_cols >= 4 && dwt.sn + dwt.dn > 1 && dwt.dn > 0 {
    c = opj_idwt53_v_simd(dwt.sn, dwt.dn, tiledp, col_start, stride, nb_cols);
  }
  while c < nb_cols {
    opj_dwt_interleave_v(dwt, tiledp, col_start + c as usize, stride as i32);
    dwt.decode_1();
    let mut k = 0i32;
    while k < dwt.sn + dwt.dn {
      tiledp[col_start + (c as usize) + (k as usize) * stride] = dwt.mem[k as usize].get();
      k += 1
    }
    c += 1
  }
}

/// Multiply the first `iters_c1` even-index samples by `c1` and the first `iters_c2`
/// odd-index samples by `c2`, 8 samples per unrolled iteration.
fn opj_dwt_encode_step1_combined(
  w: &mut [f32],
  mut fw_base: usize,
  mut iters_c1: u32,
  mut iters_c2: u32,
  c1: f32,
  c2: f32,
) {
  let mut i = 0_u32;
  let iters_common = opj_uint_min(iters_c1, iters_c2);

  debug_assert!(opj_int_abs(iters_c1 as i32 - iters_c2 as i32) <= 1i32);
  while i.wrapping_add(3u32) < iters_common {
    w[fw_base] *= c1;
    w[fw_base + 1] *= c2;
    w[fw_base + 2] *= c1;
    w[fw_base + 3] *= c2;
    w[fw_base + 4] *= c1;
    w[fw_base + 5] *= c2;
    w[fw_base + 6] *= c1;
    w[fw_base + 7] *= c2;
    fw_base += 8;
    i = (i as core::ffi::c_uint).wrapping_add(4u32)
  }
  while i < iters_common {
    w[fw_base] *= c1;
    w[fw_base + 1] *= c2;
    fw_base += 2;
    i += 1;
  }
  if i < iters_c1 {
    w[fw_base] *= c1
  } else if i < iters_c2 {
    w[fw_base + 1] *= c2
  };
}
/// One lifting step of the 1-D 9-7 forward transform, walking `w` with two interleaved cursors
/// (`fl_base` for the low band, `fw_base` for the high band) instead of raw pointers.
fn opj_dwt_encode_step2(
  w: &mut [f32],
  fl_base: usize,
  mut fw_base: usize,
  end: u32,
  m: u32,
  c: f32,
) {
  let mut i: u32 = 0;
  let mut imax = opj_uint_min(end, m);
  if imax > 0u32 {
    w[fw_base - 1] += (w[fl_base] + w[fw_base]) * c;
    fw_base += 2;
    i = 1_u32;
    while i.wrapping_add(3u32) < imax {
      w[fw_base - 1] += (w[fw_base - 2] + w[fw_base]) * c;
      w[fw_base + 1] += (w[fw_base] + w[fw_base + 2]) * c;
      w[fw_base + 3] += (w[fw_base + 2] + w[fw_base + 4]) * c;
      w[fw_base + 5] += (w[fw_base + 4] + w[fw_base + 6]) * c;
      fw_base += 8;
      i = (i as core::ffi::c_uint).wrapping_add(4u32)
    }
    while i < imax {
      w[fw_base - 1] += (w[fw_base - 2] + w[fw_base]) * c;
      fw_base += 2;
      i += 1;
    }
  }
  if m < end {
    debug_assert!(m.wrapping_add(1u32) == end);
    w[fw_base - 1] += 2 as core::ffi::c_float * w[fw_base - 2] * c
  };
}
/* *
Forward 9-7 wavelet transform in 1-D
*/
/// Forward 9-7 wavelet transform in 1-D, over one staging row of `sn + dn` floats.
fn opj_dwt_encode_1_real(w: &mut [f32], dn: i32, sn: i32, cas: i32) {
  let mut a: i32 = 0;
  let mut b: i32 = 0;
  debug_assert!(dn + sn > 1i32);
  if cas == 0i32 {
    a = 0i32;
    b = 1i32
  } else {
    a = 1i32;
    b = 0i32
  }
  opj_dwt_encode_step2(
    w,
    a as usize,
    (b + 1i32) as usize,
    dn as u32,
    opj_int_min(dn, sn - b) as u32,
    DWT_ALPHA,
  );
  opj_dwt_encode_step2(
    w,
    b as usize,
    (a + 1i32) as usize,
    sn as u32,
    opj_int_min(sn, dn - a) as u32,
    DWT_BETA,
  );
  opj_dwt_encode_step2(
    w,
    a as usize,
    (b + 1i32) as usize,
    dn as u32,
    opj_int_min(dn, sn - b) as u32,
    DWT_GAMMA,
  );
  opj_dwt_encode_step2(
    w,
    b as usize,
    (a + 1i32) as usize,
    sn as u32,
    opj_int_min(sn, dn - a) as u32,
    DWT_DELTA,
  );
  if a == 0i32 {
    opj_dwt_encode_step1_combined(w, 0, sn as u32, dn as u32, DWT_INV_K, DWT_K);
  } else {
    opj_dwt_encode_step1_combined(w, 0, dn as u32, sn as u32, DWT_K, DWT_INV_K);
  };
}
/* *
Explicit calculation of the Quantization Stepsizes
*/
fn opj_dwt_encode_stepsize(
  mut stepsize: i32,
  mut numbps: i32,
  bandno_stepsize: &mut Stepsize,
) {
  let mut p: i32 = 0;
  let mut n: i32 = 0;
  p = opj_int_floorlog2(stepsize) - 13i32;
  n = 11i32 - opj_int_floorlog2(stepsize);
  bandno_stepsize.mant = if n < 0i32 {
    (stepsize) >> -n
  } else {
    (stepsize) << n
  } & 0x7ffi32;
  bandno_stepsize.expn = numbps - p;
}
/*
==========================================================
   DWT interface
==========================================================
*/
/* * Process one line for the horizontal pass of the 5x3 forward transform */
/// Process one line for the horizontal pass of the 5-3 forward transform. `row` is the tile
/// component buffer (row starts at `row_start`), `tmp` the scratch buffer.
fn opj_dwt_encode_and_deinterleave_h_one_row(
  row: &mut [i32],
  row_start: usize,
  tmp: &mut [i32],
  width: u32,
  even: OPJ_BOOL,
) {
  let sn = (width.wrapping_add(if even != 0 { 1 } else { 0 }) >> 1i32) as i32;
  let dn = width.wrapping_sub(sn as u32) as i32;
  if even != 0 {
    if width > 1u32 {
      let mut i: i32 = 0;
      i = 0i32;
      while i < sn - 1i32 {
        tmp[(sn + i) as usize] = row[row_start + (2i32 * i + 1i32) as usize]
          .wrapping_sub(
            row[row_start + (i * 2i32) as usize]
              .wrapping_add(row[row_start + ((i + 1i32) * 2i32) as usize])
              >> 1i32,
          );
        i += 1
      }
      if width.wrapping_rem(2u32) == 0u32 {
        tmp[(sn + i) as usize] = row[row_start + (2i32 * i + 1i32) as usize]
          .wrapping_sub(row[row_start + (i * 2i32) as usize])
      }
      row[row_start] = row[row_start].wrapping_add(
        (tmp[sn as usize].wrapping_add(tmp[sn as usize]).wrapping_add(2i32)) >> 2i32,
      );
      i = 1i32;
      while i < dn {
        row[row_start + i as usize] = row[row_start + (2i32 * i) as usize].wrapping_add(
          (tmp[(sn + (i - 1i32)) as usize].wrapping_add(tmp[(sn + i) as usize]).wrapping_add(2i32))
            >> 2i32,
        );
        i += 1
      }
      if width.wrapping_rem(2u32) == 1u32 {
        row[row_start + i as usize] = row[row_start + (2i32 * i) as usize].wrapping_add(
          (tmp[(sn + (i - 1i32)) as usize]
            .wrapping_add(tmp[(sn + (i - 1i32)) as usize])
            .wrapping_add(2i32))
            >> 2i32,
        )
      }
      row[row_start + sn as usize..row_start + sn as usize + dn as usize]
        .copy_from_slice(&tmp[sn as usize..sn as usize + dn as usize]);
    }
  } else if width == 1u32 {
    row[row_start] = row[row_start].wrapping_mul(2i32)
  } else {
    let mut i_0: i32 = 0;
    tmp[sn as usize] = row[row_start].wrapping_sub(row[row_start + 1]);
    i_0 = 1i32;
    while i_0 < sn {
      tmp[(sn + i_0) as usize] = row[row_start + (2i32 * i_0) as usize].wrapping_sub(
        row[row_start + (2i32 * i_0 + 1i32) as usize]
          .wrapping_add(row[row_start + (2i32 * (i_0 - 1i32) + 1i32) as usize])
          >> 1i32,
      );
      i_0 += 1
    }
    if width.wrapping_rem(2u32) == 1u32 {
      tmp[(sn + i_0) as usize] = row[row_start + (2i32 * i_0) as usize]
        .wrapping_sub(row[row_start + (2i32 * (i_0 - 1i32) + 1i32) as usize])
    }
    i_0 = 0i32;
    while i_0 < dn - 1i32 {
      row[row_start + i_0 as usize] = row[row_start + (2i32 * i_0 + 1i32) as usize].wrapping_add(
        (tmp[(sn + i_0) as usize]
          .wrapping_add(tmp[(sn + i_0 + 1i32) as usize])
          .wrapping_add(2i32))
          >> 2i32,
      );
      i_0 += 1
    }
    if width.wrapping_rem(2u32) == 0u32 {
      row[row_start + i_0 as usize] = row[row_start + (2i32 * i_0 + 1i32) as usize].wrapping_add(
        (tmp[(sn + i_0) as usize]
          .wrapping_add(tmp[(sn + i_0) as usize])
          .wrapping_add(2i32))
          >> 2i32,
      )
    }
    row[row_start + sn as usize..row_start + sn as usize + dn as usize]
      .copy_from_slice(&tmp[sn as usize..sn as usize + dn as usize]);
  };
}
/* * Process one line for the horizontal pass of the 9x7 forward transform */
/// Process one line for the horizontal pass of the 9-7 forward transform: stage the row into
/// `tmp`, transform in place, then deinterleave back into the tile component buffer.
fn opj_dwt_encode_and_deinterleave_h_one_row_real(
  row: &mut [f32],
  row_start: usize,
  tmp: &mut [f32],
  width: u32,
  even: OPJ_BOOL,
) {
  let sn = (width.wrapping_add(if even != 0 { 1 } else { 0 }) >> 1i32) as i32;
  let dn = width.wrapping_sub(sn as u32) as i32;
  if width == 1u32 {
    return;
  }
  tmp[..width as usize].copy_from_slice(&row[row_start..row_start + width as usize]);
  opj_dwt_encode_1_real(tmp, dn, sn, if even != 0 { 0i32 } else { 1i32 });
  opj_dwt_deinterleave_h(
    tmp,
    0,
    row,
    row_start,
    dn,
    sn,
    if even != 0 { 0i32 } else { 1i32 },
  );
}

/* * Fetch up to cols <= NB_ELTS_V8 for each line, and put them in tmpOut */
/* that has a NB_ELTS_V8 interleave factor. */
/// Generic over the sample type so the 5-3 (`i32`) and 9-7 (`f32`) passes can share
/// it without erasing their buffers to `void*`. Only moves and zero-fills samples.
fn opj_dwt_fetch_cols_vertical_pass<T: Copy + Default>(
  array: &[T],
  array_start: usize,
  tmp: &mut [T],
  height: u32,
  stride_width: u32,
  cols: u32,
) {
  if cols == NB_ELTS_V8 {
    let mut k: u32 = 0;
    while k < height {
      let src = array_start + (k * stride_width) as usize;
      tmp[(NB_ELTS_V8 * k) as usize..(NB_ELTS_V8 * (k + 1)) as usize]
        .copy_from_slice(&array[src..src + NB_ELTS_V8 as usize]);
      k += 1;
    }
  } else {
    let mut k_0: u32 = 0;
    while k_0 < height {
      let mut c: u32 = 0;
      while c < cols {
        tmp[(NB_ELTS_V8 * k_0 + c) as usize] =
          array[array_start + (c + k_0 * stride_width) as usize];
        c += 1;
      }
      while c < NB_ELTS_V8 {
        tmp[(NB_ELTS_V8 * k_0 + c) as usize] = T::default();
        c += 1;
      }
      k_0 += 1;
    }
  };
}

/* Deinterleave result of forward transform, where cols <= NB_ELTS_V8 */
/* and src contains NB_ELTS_V8 consecutive values for up to NB_ELTS_V8 */
/* columns. */
/// Deinterleave the result of a forward transform, where `cols <= NB_ELTS_V8` and `src` contains
/// NB_ELTS_V8 consecutive values per row for up to NB_ELTS_V8 columns: first the `sn` low-pass
/// rows (from sample parity `cas`), then the `dn` high-pass rows (from parity `1 - cas`).
#[inline]
fn opj_dwt_deinterleave_v_cols<T: Copy>(
  src: &[T],
  dst: &mut [T],
  dst_start: usize,
  dn: i32,
  sn: i32,
  stride_width: u32,
  cas: i32,
  cols: u32,
) {
  let mut k: i32 = 0;
  while k < 2i32 {
    let (n, src_base, dst_base) = if k == 0 {
      (
        sn as usize,
        (cas * NB_ELTS_V8 as i32) as usize,
        dst_start,
      )
    } else {
      (
        dn as usize,
        ((1 - cas) * NB_ELTS_V8 as i32) as usize,
        dst_start + sn as usize * stride_width as usize,
      )
    };
    let mut i = n;
    while i > 0 {
      i -= 1;
      let s = src_base + i * (2 * NB_ELTS_V8 as usize);
      let d = dst_base + i * stride_width as usize;
      dst[d..d + cols as usize].copy_from_slice(&src[s..s + cols as usize]);
    }
    k += 1
  }
}

/* Forward 5-3 transform, for the vertical pass, processing cols columns */
/* where cols <= NB_ELTS_V8 */
/// Forward 5-3 transform for the vertical pass, processing `cols` columns at a time
/// (`cols <= NB_ELTS_V8`). `array` is the tile component buffer, columns starting at
/// `array_start`; `tmp` is the NB_ELTS_V8-interleaved scratch buffer.
fn opj_dwt_encode_and_deinterleave_v(
  array: &mut [i32],
  array_start: usize,
  tmp: &mut [i32],
  height: u32,
  even: OPJ_BOOL,
  stride_width: u32,
  cols: u32,
) {
  let sn = (height + if even != 0 { 1 } else { 0 }) >> 1;
  let dn = height - sn;
  opj_dwt_fetch_cols_vertical_pass(array, array_start, tmp, height, stride_width, cols);

  let mut c: u32 = 0;
  let mut i: u32 = 0;

  macro_rules! sc {
    ($i:expr, $c:expr) => {
      tmp[($i * 2 * NB_ELTS_V8 + $c) as usize]
    };
  }
  macro_rules! dc {
    ($i:expr, $c:expr) => {
      tmp[((1 + $i * 2) * NB_ELTS_V8 + $c) as usize]
    };
  }

  if even != 0 {
    if height > 1 {
      i = 0_u32;
      while i + 1 < sn {
        c = 0_u32;
        while c < NB_ELTS_V8 {
          dc!(i, c) = dc!(i, c).wrapping_sub(sc!(i, c).wrapping_add(sc!(i + 1, c)) >> 1);
          c += 1
        }
        i += 1
      }
      if height.is_multiple_of(2) {
        c = 0_u32;
        while c < NB_ELTS_V8 {
          dc!(i, c) = dc!(i, c).wrapping_sub(sc!(i, c));
          c += 1
        }
      }
      c = 0_u32;
      while c < NB_ELTS_V8 {
        sc!(0, c) = sc!(0, c).wrapping_add(dc!(0, c).wrapping_add(dc!(0, c)).wrapping_add(2) >> 2);
        c += 1
      }
      i = 1_u32;
      while i < dn {
        c = 0_u32;
        while c < NB_ELTS_V8 {
          sc!(i, c) = sc!(i, c).wrapping_add(dc!(i - 1, c).wrapping_add(dc!(i, c)).wrapping_add(2) >> 2);
          c += 1
        }
        i += 1
      }
      if height % 2 == 1 {
        c = 0_u32;
        while c < NB_ELTS_V8 {
          sc!(i, c) = sc!(i, c).wrapping_add(dc!(i - 1, c).wrapping_add(dc!(i - 1, c)).wrapping_add(2) >> 2);
          c += 1
        }
      }
    }
  } else if height == 1 {
    c = 0_u32;
    while c < NB_ELTS_V8 {
      sc!(0, c) = sc!(0, c).wrapping_mul(2);
      c += 1
    }
  } else {
    c = 0_u32;
    while c < NB_ELTS_V8 {
      sc!(0, c) = sc!(0, c).wrapping_sub(dc!(0, c));
      c += 1
    }
    i = 1_u32;
    while i < sn {
      c = 0_u32;
      while c < NB_ELTS_V8 {
        sc!(i, c) = sc!(i, c).wrapping_sub(dc!(i, c).wrapping_add(dc!(i - 1, c)) >> 1);
        c += 1
      }
      i += 1
    }
    if height % 2 == 1 {
      c = 0_u32;
      while c < NB_ELTS_V8 {
        sc!(i, c) = sc!(i, c).wrapping_sub(dc!(i - 1, c));
        c += 1
      }
    }
    i = 0_u32;
    while i + 1 < dn {
      c = 0_u32;
      while c < NB_ELTS_V8 {
        dc!(i, c) = dc!(i, c).wrapping_add(sc!(i, c).wrapping_add(sc!(i + 1, c)).wrapping_add(2) >> 2);
        c += 1
      }
      i += 1
    }
    if height.is_multiple_of(2) {
      c = 0_u32;
      while c < NB_ELTS_V8 {
        dc!(i, c) = dc!(i, c).wrapping_add(sc!(i, c).wrapping_add(sc!(i, c)).wrapping_add(2) >> 2);
        c += 1
      }
    }
  }
  if cols == NB_ELTS_V8 {
    opj_dwt_deinterleave_v_cols(
      tmp,
      array,
      array_start,
      dn as i32,
      sn as i32,
      stride_width,
      if even != 0 { 0i32 } else { 1i32 },
      NB_ELTS_V8,
    );
  } else {
    opj_dwt_deinterleave_v_cols(
      tmp,
      array,
      array_start,
      dn as i32,
      sn as i32,
      stride_width,
      if even != 0 { 0i32 } else { 1i32 },
      cols,
    );
  };
}

/// Multiply every group of `NB_ELTS_V8` samples of the `fw` band by `cst`.
fn opj_v8dwt_encode_step1(tmp: &mut [f32], fw_base: usize, end: u32, cst: f32) {
  let mut i: u32 = 0;
  let mut c: u32 = 0;
  while i < end {
    c = 0_u32;
    while c < NB_ELTS_V8 {
      tmp[fw_base + (i * 2 * NB_ELTS_V8 + c) as usize] *= cst;
      c += 1
    }
    i += 1
  }
}

/// One lifting step of the 9-7 vertical pass over NB_ELTS_V8-wide bands, walking `tmp` with two
/// interleaved cursors (`fl_base` low band, `fw_base` high band) instead of raw pointers.
fn opj_v8dwt_encode_step2(
  tmp: &mut [f32],
  fl_base: usize,
  mut fw_base: usize,
  end: u32,
  m: u32,
  cst: f32,
) {
  let mut i: u32 = 0;
  let mut imax = opj_uint_min(end, m);
  let mut c: i32 = 0;
  let n = NB_ELTS_V8 as usize;
  if imax > 0u32 {
    c = 0;
    while c < NB_ELTS_V8 as i32 {
      tmp[fw_base - n + c as usize] += (tmp[fl_base + c as usize] + tmp[fw_base + c as usize]) * cst;
      c += 1
    }
    fw_base += 2 * n;
    i = 1_u32;
    while i < imax {
      c = 0;
      while c < NB_ELTS_V8 as i32 {
        tmp[fw_base - n + c as usize] +=
          (tmp[fw_base - 2 * n + c as usize] + tmp[fw_base + c as usize]) * cst;
        c += 1
      }
      fw_base += 2 * n;
      i += 1;
    }
  }
  if m < end {
    debug_assert!(m.wrapping_add(1u32) == end);
    c = 0;
    while c < NB_ELTS_V8 as i32 {
      tmp[fw_base - n + c as usize] +=
        2 as core::ffi::c_float * tmp[fw_base - 2 * n + c as usize] * cst;
      c += 1
    }
  };
}

/* Forward 9-7 transform, for the vertical pass, processing cols columns */
/* where cols <= NB_ELTS_V8 */
/// Forward 9-7 transform for the vertical pass, processing `cols` columns at a time
/// (`cols <= NB_ELTS_V8`).
fn opj_dwt_encode_and_deinterleave_v_real(
  array: &mut [f32],
  array_start: usize,
  tmp: &mut [f32],
  height: u32,
  even: OPJ_BOOL,
  stride_width: u32,
  cols: u32,
) {
  let sn = (height.wrapping_add(if even != 0 { 1 } else { 0 }) >> 1i32) as i32;
  let dn = height.wrapping_sub(sn as u32) as i32;
  let mut a: i32 = 0;
  let mut b: i32 = 0;
  if height == 1u32 {
    return;
  }
  opj_dwt_fetch_cols_vertical_pass(array, array_start, tmp, height, stride_width, cols);
  if even != 0 {
    a = 0i32;
    b = 1i32
  } else {
    a = 1i32;
    b = 0i32
  }
  opj_v8dwt_encode_step2(
    tmp,
    (a * NB_ELTS_V8 as i32) as usize,
    ((b + 1i32) * NB_ELTS_V8 as i32) as usize,
    dn as u32,
    opj_int_min(dn, sn - b) as u32,
    DWT_ALPHA,
  );
  opj_v8dwt_encode_step2(
    tmp,
    (b * NB_ELTS_V8 as i32) as usize,
    ((a + 1i32) * NB_ELTS_V8 as i32) as usize,
    sn as u32,
    opj_int_min(sn, dn - a) as u32,
    DWT_BETA,
  );
  opj_v8dwt_encode_step2(
    tmp,
    (a * NB_ELTS_V8 as i32) as usize,
    ((b + 1i32) * NB_ELTS_V8 as i32) as usize,
    dn as u32,
    opj_int_min(dn, sn - b) as u32,
    DWT_GAMMA,
  );
  opj_v8dwt_encode_step2(
    tmp,
    (b * NB_ELTS_V8 as i32) as usize,
    ((a + 1i32) * NB_ELTS_V8 as i32) as usize,
    sn as u32,
    opj_int_min(sn, dn - a) as u32,
    DWT_DELTA,
  );
  opj_v8dwt_encode_step1(tmp, (b * NB_ELTS_V8 as i32) as usize, dn as u32, DWT_K);
  opj_v8dwt_encode_step1(tmp, (a * NB_ELTS_V8 as i32) as usize, sn as u32, DWT_INV_K);
  if cols == NB_ELTS_V8 {
    opj_dwt_deinterleave_v_cols::<f32>(
      tmp,
      array,
      array_start,
      dn,
      sn,
      stride_width,
      if even != 0 { 0i32 } else { 1i32 },
      NB_ELTS_V8,
    );
  } else {
    opj_dwt_deinterleave_v_cols::<f32>(
      tmp,
      array,
      array_start,
      dn,
      sn,
      stride_width,
      if even != 0 { 0i32 } else { 1i32 },
      cols,
    );
  };
}

/* <summary>                            */
/* Forward 5-3 wavelet transform in 2-D. */
/* </summary>                           */
/// Forward wavelet transform in 2-D. The tile component buffer and the NB_ELTS_V8-interleaved
/// scratch are viewed as slices for the duration of each row/column pass; the views are created
/// from the raw allocation at the boundary only (see the SAFETY notes below).
fn opj_dwt_encode_procedure(tilec: &mut TcdTileComp, wavelet: ForwardWavelet) -> OPJ_BOOL {
  let w = (tilec.x1 - tilec.x0) as u32;
  let h = (tilec.y1 - tilec.y0) as u32;
  let total_len = (w as usize).wrapping_mul(h as usize);
  if total_len > 0 && tilec.data.is_null() {
    return 0i32;
  }
  let l = tilec.numresolutions as i32 - 1i32;
  let l_data_size = opj_dwt_max_resolution(&tilec.resolutions, tilec.numresolutions) as usize;
  /* overflow check */
  if l_data_size
    > (usize::MAX).wrapping_div(NB_ELTS_V8 as usize * core::mem::size_of::<i32>())
  {
    return 0i32;
  }
  /* scratch size in elements of the wavelet's sample type */
  let scratch_len = l_data_size.wrapping_mul(NB_ELTS_V8 as usize);
  /* l_data_size is equal to 0 when numresolutions == 1 but the scratch is
  not used in that case, so do not error out */
  let mut bj: Vec<i32> = Vec::new();
  let mut bjf: Vec<f32> = Vec::new();
  match wavelet {
    ForwardWavelet::Reversible53 => {
      if bj.try_reserve_exact(scratch_len).is_err() {
        return 0i32;
      }
      bj.resize(scratch_len, 0);
    }
    ForwardWavelet::Irreversible97 => {
      if bjf.try_reserve_exact(scratch_len).is_err() {
        return 0i32;
      }
      bjf.resize(scratch_len, 0.0);
    }
  }
  /* width of the resolution level computed   */
  let mut cur = l; /* height of the resolution level computed  */
  while cur > 0 {
    let rw = (tilec.resolutions[cur as usize].x1 - tilec.resolutions[cur as usize].x0)
      as u32;
    let rh = (tilec.resolutions[cur as usize].y1 - tilec.resolutions[cur as usize].y0)
      as u32;
    let cas_row = tilec.resolutions[cur as usize].x0 & 1i32;
    let cas_col = tilec.resolutions[cur as usize].y0 & 1i32;
    /* A component can subsample to nothing inside a tile (subsampling larger
    than the tile size): its resolutions are 0-wide (or 0-high).  The C's
    row loops no-op on the zero-sized buffer (the malloc'd pointer is read
    as garbage); the port's V8 accessors would panic - skip the transform
    level entirely. */
    if rw == 0u32 || rh == 0u32 {
      cur -= 1i32;
      continue;
    }
    match wavelet {
      ForwardWavelet::Reversible53 => {
        /* Perform vertical pass */
        let mut j: u32 = 0;
        while j + NB_ELTS_V8 - 1 < rw {
          let col = j as usize;
          {
            let data = &mut tilec.data.as_mut_i32(total_len);
            let scratch = &mut bj[..];
            opj_dwt_encode_and_deinterleave_v(
              data,
              col,
              scratch,
              rh,
              (cas_col == 0i32) as core::ffi::c_int,
              w,
              NB_ELTS_V8,
            )
          }
          j += NB_ELTS_V8;
        }
        if j < rw {
          let col = j as usize;
          let data = &mut tilec.data.as_mut_i32(total_len);
          let scratch = &mut bj[..];
          opj_dwt_encode_and_deinterleave_v(
            data,
            col,
            scratch,
            rh,
            (cas_col == 0i32) as core::ffi::c_int,
            w,
            rw.wrapping_sub(j),
          )
        }
        /* Perform horizontal pass */
        j = 0_u32;
        while j < rh {
          let row_start = j.wrapping_mul(w) as usize;
          let even = if cas_row == 0i32 { 1i32 } else { 0i32 };
          {
            let data = &mut tilec.data.as_mut_i32(total_len);
            let scratch = &mut bj[..];
            opj_dwt_encode_and_deinterleave_h_one_row(data, row_start, scratch, rw, even)
          }
          j += 1;
        }
      }
      ForwardWavelet::Irreversible97 => {
        /* The 9-7 buffer is f32-typed: the vertical and horizontal passes
         * borrow it directly (the C's in-place reinterpretation, without
         * the per-level whole-tile copy). */
        /* Perform vertical pass */
        let mut j: u32 = 0;
        while j + NB_ELTS_V8 - 1 < rw {
          let col = j as usize;
          opj_dwt_encode_and_deinterleave_v_real(
            tilec.data.as_mut_f32(total_len),
            col,
            &mut bjf[..],
            rh,
            (cas_col == 0i32) as core::ffi::c_int,
            w,
            NB_ELTS_V8,
          );
          j += NB_ELTS_V8;
        }
        if j < rw {
          let col = j as usize;
          opj_dwt_encode_and_deinterleave_v_real(
            tilec.data.as_mut_f32(total_len),
            col,
            &mut bjf[..],
            rh,
            (cas_col == 0i32) as core::ffi::c_int,
            w,
            rw.wrapping_sub(j),
          );
        }
        /* Perform horizontal pass */
        j = 0_u32;
        while j < rh {
          let row_start = j.wrapping_mul(w) as usize;
          let even = if cas_row == 0i32 { 1i32 } else { 0i32 };
          opj_dwt_encode_and_deinterleave_h_one_row_real(
            tilec.data.as_mut_f32(total_len),
            row_start,
            &mut bjf[..],
            rw,
            even,
          );
          j += 1;
        }
      }
    }
    cur -= 1;
  }
  1i32
}

/* Forward 5-3 wavelet transform in 2-D. */
/* </summary>                           */

pub(crate) fn opj_dwt_encode(tilec: &mut TcdTileComp) -> OPJ_BOOL {
  opj_dwt_encode_procedure(tilec, ForwardWavelet::Reversible53)
}
/* <summary>                            */
/* Inverse 5-3 wavelet transform in 2-D. */
/* </summary>                           */

pub(crate) fn opj_dwt_decode(
  whole_tile_decoding: OPJ_BOOL,
  tilec: &mut TcdTileComp,
  numres: u32,
) -> OPJ_BOOL {
  if whole_tile_decoding != 0 {
    opj_dwt_decode_tile(tilec, numres)
  } else {
    opj_dwt_decode_partial_tile(tilec, numres)
  }
}

/* <summary>                */
/* Get norm of 5-3 wavelet. */
/* </summary>               */

pub fn opj_dwt_getnorm(mut level: u32, orient: u32) -> f64 {
  /* Band-aid to avoid a buffer overflow (upstream #493: the arrays only
  cover 10 resolution levels). */
  /* See https://github.com/uclouvain/openjpeg/issues/493 */
  if orient == 0 && level >= 10 {
    level = 9
  } else if orient > 0 && level >= 9 {
    level = 8
  }
  DWT_NORMS[orient as usize][level as usize]
}

/* <summary>                             */
/* Forward 9-7 wavelet transform in 2-D. */
/* </summary>                            */

pub(crate) fn opj_dwt_encode_real(tilec: &mut TcdTileComp) -> OPJ_BOOL {
  opj_dwt_encode_procedure(tilec, ForwardWavelet::Irreversible97)
}

/* <summary>                */
/* Get norm of 9-7 wavelet. */
/* </summary>               */

pub fn opj_dwt_getnorm_real(mut level: u32, orient: u32) -> f64 {
  /* Band-aid to avoid a buffer overflow (upstream #493: the arrays only
  cover 10 resolution levels). */
  /* See https://github.com/uclouvain/openjpeg/issues/493 */
  if orient == 0 && level >= 10 {
    level = 9
  } else if orient > 0 && level >= 9 {
    level = 8
  }
  DWT_NORMS_REAL[orient as usize][level as usize]
}

pub(crate) fn opj_dwt_calc_explicit_stepsizes(tccp: &mut Tccp, mut prec: u32) {
    let mut numbands: u32 = 0;
    let mut bandno: u32 = 0;
    numbands = (3u32)
      .wrapping_mul(tccp.numresolutions)
      .wrapping_sub(2u32);
    bandno = 0_u32;
    while bandno < numbands {
      let mut stepsize: f64 = 0.;
      let mut resno: u32 = 0;
      let mut level: u32 = 0;
      let mut orient: u32 = 0;
      let mut gain: u32 = 0;
      resno = if bandno == 0u32 {
        0u32
      } else {
        bandno
          .wrapping_sub(1u32)
          .wrapping_div(3u32)
          .wrapping_add(1u32)
      };
      orient = if bandno == 0u32 {
        0u32
      } else {
        bandno
          .wrapping_sub(1u32)
          .wrapping_rem(3u32)
          .wrapping_add(1u32)
      };
      level = tccp
        .numresolutions
        .wrapping_sub(1u32)
        .wrapping_sub(resno);
      gain = if tccp.qmfbid == 0u32 || orient == 0u32 {
        0i32
      } else if orient == 1u32 || orient == 2u32 {
        1i32
      } else {
        2i32
      } as u32;
      if tccp.qntsty == 0u32 {
        stepsize = 1.0f64
      } else {
        let mut norm = opj_dwt_getnorm_real(level, orient);
        stepsize = ((1i32) << gain) as core::ffi::c_double / norm
      }
      opj_dwt_encode_stepsize(
        (stepsize * 8192.0f64).floor() as i32,
        prec.wrapping_add(gain) as i32,
        &mut tccp.stepsizes[bandno as usize],
      );
      bandno += 1;
    }
}
/* <summary>                             */
/* Determine maximum computed resolution level for inverse wavelet transform */
/* </summary>                            */
/// Largest width/height among resolution levels 1..`i` (level 0, the full-resolution buffer, is
/// excluded).
fn opj_dwt_max_resolution(r: &[TcdResolution], mut i: u32) -> u32 {
  let mut mr = 0_u32;
  let mut idx = 1usize;
  loop {
    i = i.wrapping_sub(1);
    if i == 0 {
      break;
    }
    let w = (r[idx].x1 - r[idx].x0) as u32;
    if mr < w {
      mr = w
    }
    let w = (r[idx].y1 - r[idx].y0) as u32;
    if mr < w {
      mr = w
    }
    idx += 1;
  }
  mr
}

/* <summary>                            */
/* Inverse wavelet transform in 2-D.    */
/* </summary>                           */
/// Inverse 5-3 wavelet transform in 2-D. The tile component buffer is viewed as one slice for
/// the whole procedure; the row/column passes index into it with `usize` arithmetic.
fn opj_dwt_decode_tile(tilec: &mut TcdTileComp, mut numres: u32) -> OPJ_BOOL {
  let mut rw = (tilec.resolutions[0].x1 - tilec.resolutions[0].x0) as u32;
  let mut rh = (tilec.resolutions[0].y1 - tilec.resolutions[0].y0) as u32;
  let w = (tilec.resolutions[tilec.minimum_num_resolutions as usize - 1].x1
    - tilec.resolutions[tilec.minimum_num_resolutions as usize - 1].x0) as u32;

  // Not entirely sure for the return code of w == 0 which is triggered per
  // https://github.com/uclouvain/openjpeg/issues/1505
  if numres == 1 || w == 0 {
    return 1i32;
  }
  let mut h_mem_len = opj_dwt_max_resolution(&tilec.resolutions, numres) as usize;
  /* overflow check */
  if h_mem_len > (usize::MAX / PARALLEL_COLS_53 / core::mem::size_of::<i32>()) {
    return 0i32;
  }
  /* We need PARALLEL_COLS_53 times the height of the array, */
  /* since for the vertical pass */
  /* we process PARALLEL_COLS_53 columns at a time */
  h_mem_len *= PARALLEL_COLS_53;
  // The horizontal and vertical views share this scratch buffer, exactly as in the C code.
  let mut scratch: Vec<Cell<i32>> = Vec::new();
  if scratch.try_reserve_exact(h_mem_len).is_err() {
    return 0i32;
  }
  scratch.resize(h_mem_len, Cell::new(0i32));
  let mut h = Dwt {
    mem: &scratch,
    dn: 0,
    sn: 0,
    cas: 0,
  }; /* width of the resolution level computed */
  let mut v = Dwt {
    mem: &scratch,
    dn: 0,
    sn: 0,
    cas: 0,
  }; /* height of the resolution level computed */

  /* The tile data buffer is allocated for the finest decoded resolution
  (opj_tcd_decode_tile sizes it from resolutions[minimum_num_resolutions-1]),
  so the slice covers exactly that geometry, like the C code's row stride. */
  let total_len = (w as usize).wrapping_mul(
    (tilec.resolutions[tilec.minimum_num_resolutions as usize - 1].y1
      - tilec.resolutions[tilec.minimum_num_resolutions as usize - 1].y0) as usize,
  );
  if total_len > 0 && tilec.data.is_null() {
    return 0i32;
  }
  let tile_data = tilec.data.as_mut_i32(total_len);

  let mut res_idx = 1usize;
  loop {
    numres = numres.wrapping_sub(1);
    if numres == 0 {
      break;
    }
    h.sn = rw as i32;
    v.sn = rh as i32;
    rw = (tilec.resolutions[res_idx].x1 - tilec.resolutions[res_idx].x0) as u32;
    rh = (tilec.resolutions[res_idx].y1 - tilec.resolutions[res_idx].y0) as u32;
    h.dn = rw.wrapping_sub(h.sn as u32) as i32;
    h.cas = tilec.resolutions[res_idx].x0 % 2i32;
    let level_rows = rh as usize;
    if level_rows * rw as usize >= 16384 {
      /* Large levels: parallelise the in-place horizontal pass (the rows
      are disjoint and independent); smaller levels and the vertical pass
      stay sequential (the per-level parallel overhead dominates there). */
      let h_sn = h.sn;
      let h_dn = h.dn;
      let h_cas = h.cas;
      let h_len = 2 * (h_sn + h_dn) as usize;
      let stride = w as usize;
      par::for_each_chunks_mut(&mut tile_data[..level_rows * stride], stride, |_row_idx, row| {
        DWT_SCRATCH.with(|scratch| {
          let mut s = scratch.borrow_mut();
          if s.len() < h_len {
            /* The thread-local scratch is sized from header geometry: fail
            the job instead of aborting on OOM (the vec! resize would
            abort). */
            let need = h_len - s.len();
            if s.try_reserve_exact(need).is_err() {
              return;
            }
            s.resize(h_len, core::cell::Cell::new(0));
          }
          let d = Dwt {
            mem: &s[..h_len],
            dn: h_dn,
            sn: h_sn,
            cas: h_cas,
          };
          opj_dwt_interleave_h(&d, row);
          d.decode_1();
          for (cell, row_v) in d.mem[..(h_sn + h_dn) as usize].iter().zip(row.iter_mut()) {
            *row_v = cell.get();
          }
        });
      });
    } else {
      let mut j: u32 = 0;
      while j < rh {
        opj_idwt53_h(&h, tile_data, (j as usize).wrapping_mul(w as usize));
        j += 1;
      }
    }
    v.dn = rh.wrapping_sub(v.sn as u32) as i32;
    v.cas = tilec.resolutions[res_idx].y0 % 2i32;
    let mut j: u32 = 0;
    while j.wrapping_add((2i32 * 4i32) as core::ffi::c_uint) <= rw {
      opj_idwt53_v(&v, tile_data, j as usize, w as usize, 2i32 * 4i32);
      j = (j as core::ffi::c_uint).wrapping_add((2i32 * 4i32) as core::ffi::c_uint)
    }
    if j < rw {
      opj_idwt53_v(&v, tile_data, j as usize, w as usize, rw.wrapping_sub(j) as i32);
    }
    res_idx += 1;
  }
  1i32
}
fn opj_dwt_interleave_partial_h<T: SparseSample>(
  dest: &[Cell<T>],
  cas: i32,
  sa: &SparseArray,
  sa_line: u32,
  sn: u32,
  win_l_x0: u32,
  win_l_x1: u32,
  win_h_x0: u32,
  win_h_x1: u32,
) {
  let ret = sparse_array_read(
    sa,
    win_l_x0,
    sa_line,
    win_l_x1,
    sa_line.wrapping_add(1u32),
    &dest[cas as usize + 2 * win_l_x0 as usize..],
    2_u32,
    0_u32,
    true,
  );
  debug_assert!(ret);
  let ret = sparse_array_read(
    sa,
    sn.wrapping_add(win_h_x0),
    sa_line,
    sn.wrapping_add(win_h_x1),
    sa_line.wrapping_add(1u32),
    &dest[(1 - cas) as usize + 2 * win_h_x0 as usize..],
    2_u32,
    0_u32,
    true,
  );
  debug_assert!(ret);
}
fn opj_dwt_interleave_partial_v<T: SparseSample>(
  dest: &[Cell<T>],
  cas: i32,
  sa: &SparseArray,
  sa_col: u32,
  nb_cols: u32,
  sn: u32,
  win_l_y0: u32,
  win_l_y1: u32,
  win_h_y0: u32,
  win_h_y1: u32,
) {
  let ret = sparse_array_read(
    sa,
    sa_col,
    win_l_y0,
    sa_col.wrapping_add(nb_cols),
    win_l_y1,
    &dest[(cas * 4) as usize + 8 * win_l_y0 as usize..],
    1_u32,
    (2i32 * 4i32) as u32,
    true,
  );
  debug_assert!(ret);
  let ret = sparse_array_read(
    sa,
    sa_col,
    sn.wrapping_add(win_h_y0),
    sa_col.wrapping_add(nb_cols),
    sn.wrapping_add(win_h_y1),
    &dest[((1i32 - cas) * 4) as usize + 8 * win_h_y0 as usize..],
    1_u32,
    (2i32 * 4i32) as u32,
    true,
  );
  debug_assert!(ret);
}

fn opj_dwt_get_band_coordinates(
  tilec: &TcdTileComp,
  resno: u32,
  bandno: u32,
  tcx0: u32,
  tcy0: u32,
  tcx1: u32,
  tcy1: u32,
  tbx0: Option<&mut u32>,
  tby0: Option<&mut u32>,
  tbx1: Option<&mut u32>,
  tby1: Option<&mut u32>,
) {
  /* Compute number of decomposition for this band. See table F-1 */
  let nb = if resno == 0u32 {
    tilec.numresolutions.wrapping_sub(1u32)
  } else {
    tilec.numresolutions.wrapping_sub(resno)
  };
  /* Map above tile-based coordinates to sub-band-based coordinates per */
  /* equation B-15 of the standard */
  let x0b = bandno & 1u32;
  let y0b = bandno >> 1i32;
  if let Some(tbx0) = tbx0 {
    *tbx0 = if nb == 0u32 {
      tcx0
    } else if tcx0 <= ((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(x0b) {
      0u32
    } else {
      opj_uint_ceildivpow2(
        tcx0.wrapping_sub(((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(x0b)),
        nb,
      )
    }
  }
  if let Some(tby0) = tby0 {
    *tby0 = if nb == 0u32 {
      tcy0
    } else if tcy0 <= ((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(y0b) {
      0u32
    } else {
      opj_uint_ceildivpow2(
        tcy0.wrapping_sub(((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(y0b)),
        nb,
      )
    }
  }
  if let Some(tbx1) = tbx1 {
    *tbx1 = if nb == 0u32 {
      tcx1
    } else if tcx1 <= ((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(x0b) {
      0u32
    } else {
      opj_uint_ceildivpow2(
        tcx1.wrapping_sub(((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(x0b)),
        nb,
      )
    }
  }
  if let Some(tby1) = tby1 {
    *tby1 = if nb == 0u32 {
      tcy1
    } else if tcy1 <= ((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(y0b) {
      0u32
    } else {
      opj_uint_ceildivpow2(
        tcy1.wrapping_sub(((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(y0b)),
        nb,
      )
    }
  };
}
fn opj_dwt_segment_grow(
  filter_width: u32,
  max_size: u32,
  start: &mut u32,
  end: &mut u32,
) {
  *start = opj_uint_subs(*start, filter_width);
  *end = opj_uint_adds(*end, filter_width);
  *end = opj_uint_min(*end, max_size);
}
/// Copy every decoded code-block of the lowest `numres` resolutions into a sparse array over
/// the finest of those resolutions, so the partial inverse transform can work on the window only.
fn opj_dwt_init_sparse_array(tilec: &TcdTileComp, numres: u32) -> Option<SparseArray> {
  let tr_max = &tilec.resolutions[numres.wrapping_sub(1u32) as usize];
  let w = (tr_max.x1 - tr_max.x0) as u32;
  let h = (tr_max.y1 - tr_max.y0) as u32;
  let mut sa = SparseArray::new(
    w,
    h,
    opj_uint_min(w, 64_u32),
    opj_uint_min(h, 64_u32),
  )?;
  let mut resno: u32 = 0;
  while resno < numres {
    let res = &tilec.resolutions[resno as usize];
    let mut bandno: u32 = 0;
    while bandno < res.numbands {
      let band = &res.bands[bandno as usize];
      let mut precno: u32 = 0;
      while precno < res.pw.wrapping_mul(res.ph) {
        let precinct = &band.precincts[precno as usize];
        let mut cblkno: u32 = 0;
        while cblkno < precinct.cw.wrapping_mul(precinct.ch) {
          if let CodeBlocks::Dec(cblks) = &precinct.cblks {
            let cblk = &cblks[cblkno as usize];
            if !cblk.decoded_data.is_empty() {
              let mut x = (cblk.x0 - band.x0) as u32;
              let mut y = (cblk.y0 - band.y0) as u32;
              let cblk_w = (cblk.x1 - cblk.x0) as u32;
              let cblk_h = (cblk.y1 - cblk.y0) as u32;
              if band.bandno & 1u32 != 0 {
                let pres = &tilec.resolutions[resno.wrapping_sub(1u32) as usize];
                x = x.wrapping_add((pres.x1 - pres.x0) as u32)
              }
              if band.bandno & 2u32 != 0 {
                let pres = &tilec.resolutions[resno.wrapping_sub(1u32) as usize];
                y = y.wrapping_add((pres.y1 - pres.y0) as u32)
              }
              if !sparse_array_write(
                &mut sa,
                x,
                y,
                x.wrapping_add(cblk_w),
                y.wrapping_add(cblk_h),
                cblk.decoded_data.as_slice(),
                1_u32,
                cblk_w,
                true,
              ) {
                return None;
              }
            }
          }
          cblkno += 1;
        }
        precno += 1;
      }
      bandno += 1;
    }
    resno += 1;
  }
  Some(sa)
}

fn opj_dwt_decode_partial_tile(tilec: &mut TcdTileComp, mut numres: u32) -> OPJ_BOOL {
  let mut resno: u32 = 0;
  /* This value matches the maximum left/right extension given in tables */
  /* F.2 and F.3 of the standard. */
  let filter_width = 2u32; /* width of the resolution level computed */
  let tr_max_idx = numres.wrapping_sub(1u32) as usize; /* height of the resolution level computed */
  let mut rw = (tilec.resolutions[0].x1 - tilec.resolutions[0].x0) as u32;
  let mut rh = (tilec.resolutions[0].y1 - tilec.resolutions[0].y0) as u32;
  let mut h_mem_size: usize = 0;
  /* Compute the intersection of the area of interest, expressed in tile coordinates */
  /* with the tile coordinates */
  let mut win_tcx0 = tilec.win_x0;
  let mut win_tcy0 = tilec.win_y0;
  let mut win_tcx1 = tilec.win_x1;
  let mut win_tcy1 = tilec.win_y1;
  if tilec.resolutions[tr_max_idx].x0 == tilec.resolutions[tr_max_idx].x1
    || tilec.resolutions[tr_max_idx].y0 == tilec.resolutions[tr_max_idx].y1
  {
    return 1i32;
  }
  let mut sa = if let Some(sa) = opj_dwt_init_sparse_array(&*tilec, numres) {
    sa
  } else {
    return 0i32;
  };
  if numres == 1u32 {
    let tr_max = &tilec.resolutions[tr_max_idx];
    let win_w = tr_max.win_x1.wrapping_sub(tr_max.win_x0) as usize;
    let win_h = tr_max.win_y1.wrapping_sub(tr_max.win_y0) as usize;
    if win_w == 0 || win_h == 0 {
      // Empty window (e.g. the tile does not intersect the decoding area): nothing to read back,
      // and `data_win` may be null, so skip the read entirely (the forgiving path did the same).
      return 1i32;
    }
    // SAFETY: `data_win` covers the window of interest (`win_w * win_h` samples).
    let win = tilec.data_win.as_mut_i32(win_w.wrapping_mul(win_h));
    let ret = sparse_array_read(
      &sa,
      tr_max.win_x0.wrapping_sub(tr_max.x0 as u32),
      tr_max.win_y0.wrapping_sub(tr_max.y0 as u32),
      tr_max.win_x1.wrapping_sub(tr_max.x0 as u32),
      tr_max.win_y1.wrapping_sub(tr_max.y0 as u32),
      Cell::from_mut(win).as_slice_of_cells(),
      1_u32,
      tr_max.win_x1.wrapping_sub(tr_max.win_x0),
      true,
    );
    debug_assert!(ret);
    return 1i32;
  }
  h_mem_size = opj_dwt_max_resolution(&tilec.resolutions, numres) as usize;
  /* overflow check */
  /* in vertical pass, we process 4 columns at a time; the scratch holds
  4 x h_mem_size i32 elements, so the guard must divide by 4 * sizeof(i32)
  (the C writes SIZE_MAX / (4 * sizeof(int32_t)); the port's previous
  left-to-right MAX/4*4 expression was a dead check). */
  if h_mem_size > (usize::MAX).wrapping_div(4 * core::mem::size_of::<i32>()) {
    return 0i32;
  }
  h_mem_size *= 4;
  // The horizontal and vertical views share this scratch buffer, exactly as in the C code.
  let mut scratch: Vec<Cell<i32>> = Vec::new();
  if scratch.try_reserve_exact(h_mem_size).is_err() {
    return 0i32;
  }
  scratch.resize(h_mem_size, Cell::new(0i32));
  let mut h = Dwt {
    mem: &scratch,
    dn: 0,
    sn: 0,
    cas: 0,
  };
  let mut v = Dwt {
    mem: &scratch,
    dn: 0,
    sn: 0,
    cas: 0,
  };
  resno = 1_u32;
  while resno < numres {
    let mut i: u32 = 0;
    let mut j: u32 = 0;
    /* Window of interest subband-based coordinates */
    let mut win_ll_x0: u32 = 0;
    let mut win_ll_y0: u32 = 0;
    let mut win_ll_x1: u32 = 0;
    let mut win_ll_y1: u32 = 0;
    let mut win_hl_x0: u32 = 0;
    let mut win_hl_x1: u32 = 0;
    let mut win_lh_y0: u32 = 0;
    let mut win_lh_y1: u32 = 0;
    /* Window of interest tile-resolution-based coordinates */
    let mut win_tr_x0: u32 = 0;
    let mut win_tr_x1: u32 = 0;
    let mut win_tr_y0: u32 = 0;
    let mut win_tr_y1: u32 = 0;
    /* Tile-resolution subband-based coordinates */
    let mut tr_ll_x0: u32 = 0;
    let mut tr_ll_y0: u32 = 0;
    let mut tr_hl_x0: u32 = 0;
    let mut tr_lh_y0: u32 = 0;
    let tr = &tilec.resolutions[resno as usize];
    h.sn = rw as i32;
    v.sn = rh as i32;
    rw = (tr.x1 - tr.x0) as u32;
    rh = (tr.y1 - tr.y0) as u32;
    h.dn = rw.wrapping_sub(h.sn as u32) as i32;
    h.cas = tr.x0 % 2i32;
    v.dn = rh.wrapping_sub(v.sn as u32) as i32;
    v.cas = tr.y0 % 2i32;
    /* Get the subband coordinates for the window of interest */
    /* LL band */
    opj_dwt_get_band_coordinates(
      &*tilec,
      resno,
      0_u32,
      win_tcx0,
      win_tcy0,
      win_tcx1,
      win_tcy1,
      Some(&mut win_ll_x0),
      Some(&mut win_ll_y0),
      Some(&mut win_ll_x1),
      Some(&mut win_ll_y1),
    );
    /* HL band */
    opj_dwt_get_band_coordinates(
      &*tilec,
      resno,
      1_u32,
      win_tcx0,
      win_tcy0,
      win_tcx1,
      win_tcy1,
      Some(&mut win_hl_x0),
      None,
      Some(&mut win_hl_x1),
      None,
    );
    /* LH band */
    opj_dwt_get_band_coordinates(
      &*tilec,
      resno,
      2_u32,
      win_tcx0,
      win_tcy0,
      win_tcx1,
      win_tcy1,
      None,
      Some(&mut win_lh_y0),
      None,
      Some(&mut win_lh_y1),
    );
    /* Beware: band index for non-LL0 resolution are 0=HL, 1=LH and 2=HH */
    tr_ll_x0 = tr.bands[1_usize].x0 as u32;
    tr_ll_y0 = tr.bands[0_usize].y0 as u32;
    tr_hl_x0 = tr.bands[0_usize].x0 as u32;
    tr_lh_y0 = tr.bands[1_usize].y0 as u32;
    /* Subtract the origin of the bands for this tile, to the subwindow */
    /* of interest band coordinates, so as to get them relative to the */
    /* tile */
    win_ll_x0 = opj_uint_subs(win_ll_x0, tr_ll_x0);
    win_ll_y0 = opj_uint_subs(win_ll_y0, tr_ll_y0);
    win_ll_x1 = opj_uint_subs(win_ll_x1, tr_ll_x0);
    win_ll_y1 = opj_uint_subs(win_ll_y1, tr_ll_y0);
    win_hl_x0 = opj_uint_subs(win_hl_x0, tr_hl_x0);
    win_hl_x1 = opj_uint_subs(win_hl_x1, tr_hl_x0);
    win_lh_y0 = opj_uint_subs(win_lh_y0, tr_lh_y0);
    win_lh_y1 = opj_uint_subs(win_lh_y1, tr_lh_y0);
    opj_dwt_segment_grow(
      filter_width,
      h.sn as u32,
      &mut win_ll_x0,
      &mut win_ll_x1,
    );
    opj_dwt_segment_grow(
      filter_width,
      h.dn as u32,
      &mut win_hl_x0,
      &mut win_hl_x1,
    );
    opj_dwt_segment_grow(
      filter_width,
      v.sn as u32,
      &mut win_ll_y0,
      &mut win_ll_y1,
    );
    opj_dwt_segment_grow(
      filter_width,
      v.dn as u32,
      &mut win_lh_y0,
      &mut win_lh_y1,
    );
    /* Compute the tile-resolution-based coordinates for the window of interest */
    if h.cas == 0i32 {
      win_tr_x0 = opj_uint_min(
        (2u32).wrapping_mul(win_ll_x0),
        (2u32).wrapping_mul(win_hl_x0).wrapping_add(1u32),
      );
      win_tr_x1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_ll_x1),
          (2u32).wrapping_mul(win_hl_x1).wrapping_add(1u32),
        ),
        rw,
      )
    } else {
      win_tr_x0 = opj_uint_min(
        (2u32).wrapping_mul(win_hl_x0),
        (2u32).wrapping_mul(win_ll_x0).wrapping_add(1u32),
      );
      win_tr_x1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_hl_x1),
          (2u32).wrapping_mul(win_ll_x1).wrapping_add(1u32),
        ),
        rw,
      )
    }
    if v.cas == 0i32 {
      win_tr_y0 = opj_uint_min(
        (2u32).wrapping_mul(win_ll_y0),
        (2u32).wrapping_mul(win_lh_y0).wrapping_add(1u32),
      );
      win_tr_y1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_ll_y1),
          (2u32).wrapping_mul(win_lh_y1).wrapping_add(1u32),
        ),
        rh,
      )
    } else {
      win_tr_y0 = opj_uint_min(
        (2u32).wrapping_mul(win_lh_y0),
        (2u32).wrapping_mul(win_ll_y0).wrapping_add(1u32),
      );
      win_tr_y1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_lh_y1),
          (2u32).wrapping_mul(win_ll_y1).wrapping_add(1u32),
        ),
        rh,
      )
    }
    j = 0_u32;
    while j < rh {
      if j >= win_ll_y0 && j < win_ll_y1
        || j >= win_lh_y0.wrapping_add(v.sn as u32)
          && j < win_lh_y1.wrapping_add(v.sn as u32)
      {
        /* Avoids dwt.c:1584:44 (in opj_dwt_decode_partial_1): runtime error: */
        /* signed integer overflow: -1094795586 + -1094795586 cannot be represented in type 'int' */
        /* on opj_decompress -i  ../../openjpeg/MAPA.jp2 -o out.tif -d 0,0,256,256 */
        /* This is less extreme than memsetting the whole buffer to 0 */
        /* although we could potentially do better with better handling of edge conditions */
        if win_tr_x1 >= 1u32 && win_tr_x1 < rw {
          h.m_set(win_tr_x1.wrapping_sub(1u32) as isize, 0i32)
        }
        if win_tr_x1 < rw {
          h.m_set(win_tr_x1 as isize, 0i32)
        }
        opj_dwt_interleave_partial_h(
          h.mem,
          h.cas,
          &sa,
          j,
          h.sn as u32,
          win_ll_x0,
          win_ll_x1,
          win_hl_x0,
          win_hl_x1,
        );
        h.decode_partial_1(
          win_ll_x0 as i32,
          win_ll_x1 as i32,
          win_hl_x0 as i32,
          win_hl_x1 as i32,
        );
        if !sparse_array_write(
          &mut sa,
          win_tr_x0,
          j,
          win_tr_x1,
          j.wrapping_add(1u32),
          &h.mem[win_tr_x0 as usize..],
          1_u32,
          0_u32,
          true,
        ) {
          return 0i32;
        }
      }
      j += 1;
    }
    i = win_tr_x0;
    while i < win_tr_x1 {
      let mut nb_cols = opj_uint_min(4u32, win_tr_x1.wrapping_sub(i));
      opj_dwt_interleave_partial_v(
        v.mem,
        v.cas,
        &sa,
        i,
        nb_cols,
        v.sn as u32,
        win_ll_y0,
        win_ll_y1,
        win_lh_y0,
        win_lh_y1,
      );
      v.decode_partial_1_parallel(
        win_ll_y0 as i32,
        win_ll_y1 as i32,
        win_lh_y0 as i32,
        win_lh_y1 as i32,
      );
      if !sparse_array_write(
        &mut sa,
        i,
        win_tr_y0,
        i.wrapping_add(nb_cols),
        win_tr_y1,
        &v.mem[(4u32).wrapping_mul(win_tr_y0) as usize..],
        1_u32,
        4_u32,
        true,
      ) {
        return 0i32;
      }
      i = (i as core::ffi::c_uint).wrapping_add(nb_cols)
    }
    resno += 1;
  }
  let tr_max = &tilec.resolutions[tr_max_idx];
  let win_w = tr_max.win_x1.wrapping_sub(tr_max.win_x0) as usize;
  let win_h = tr_max.win_y1.wrapping_sub(tr_max.win_y0) as usize;
  if win_w == 0 || win_h == 0 {
    // Empty window (e.g. the tile does not intersect the decoding area): nothing to read back,
    // and `data_win` may be null, so skip the read entirely (the forgiving path did the same).
    return 1i32;
  }
  // SAFETY: `data_win` covers the window of interest (`win_w * win_h` samples).
  let win = tilec.data_win.as_mut_i32(win_w.wrapping_mul(win_h));
  let ret_0 = sparse_array_read(
    &sa,
    tr_max.win_x0.wrapping_sub(tr_max.x0 as u32),
    tr_max.win_y0.wrapping_sub(tr_max.y0 as u32),
    tr_max.win_x1.wrapping_sub(tr_max.x0 as u32),
    tr_max.win_y1.wrapping_sub(tr_max.y0 as u32),
    Cell::from_mut(win).as_slice_of_cells(),
    1_u32,
    tr_max.win_x1.wrapping_sub(tr_max.win_x0),
    true,
  );
  debug_assert!(ret_0);
  1i32
}

/// Inverse lazy transform (horizontal) of the 9-7 pass: stage a block of `remaining_height`
/// rows from the tile buffer into the scratch, low band first then high band.
fn opj_v8dwt_interleave_h(
  dwt: &V8Dwt,
  a: &[f32],
  mut a_start: usize,
  width: u32,
  remaining_height: u32,
) {
  let mut bi = (NB_ELTS_V8 * dwt.cas as u32) as usize;
  let mut x0 = dwt.win_l_x0;
  let mut x1 = dwt.win_l_x1;
  // The C code branches between a "fast" and a "slow" path here; both write exactly
  // `min(remaining_height, NB_ELTS_V8)` values per column, so the branch is redundant.
  let rows = remaining_height.min(NB_ELTS_V8) as usize;
  for _ in 0..2 {
    for i in x0..x1 {
      let dst = bi + (i * 2 * NB_ELTS_V8) as usize;
      for r in 0..rows {
        dwt.wavelet[dst + r].set(a[a_start + i as usize + r * width as usize]);
      }
    }
    bi = (NB_ELTS_V8 * (1i32 - dwt.cas) as u32) as usize;
    // Same as the C's `a += dwt->sn`: the high band of this level starts `sn` samples later.
    a_start += dwt.sn as usize;
    x0 = dwt.win_h_x0;
    x1 = dwt.win_h_x1;
  }
}

fn opj_v8dwt_interleave_partial_h(
  mut dwt: &V8Dwt,
  sa: &SparseArray,
  sa_line: u32,
  remaining_height: u32,
) {
  let l_base = (NB_ELTS_V8 * dwt.cas as u32 + 2 * NB_ELTS_V8 * dwt.win_l_x0) as usize;
  let h_base =
    (NB_ELTS_V8 * (1i32 - dwt.cas) as u32 + 2 * NB_ELTS_V8 * dwt.win_h_x0) as usize;
  for i in 0..remaining_height {
    let ret = sparse_array_read(
      sa,
      dwt.win_l_x0,
      sa_line.wrapping_add(i),
      dwt.win_l_x1,
      sa_line.wrapping_add(i).wrapping_add(1u32),
      &dwt.wavelet[l_base + i as usize..],
      2 * NB_ELTS_V8,
      0_u32,
      true,
    );
    debug_assert!(ret);
    let ret = sparse_array_read(
      sa,
      (dwt.sn as u32).wrapping_add(dwt.win_h_x0),
      sa_line.wrapping_add(i),
      (dwt.sn as u32).wrapping_add(dwt.win_h_x1),
      sa_line.wrapping_add(i).wrapping_add(1u32),
      &dwt.wavelet[h_base + i as usize..],
      2 * NB_ELTS_V8,
      0_u32,
      true,
    );
    debug_assert!(ret);
  }
}

#[inline]
/// Inverse lazy transform (vertical) of the 9-7 pass: stage `nb_elts_read` columns from the tile
/// buffer into the scratch, low band first then high band.
fn opj_v8dwt_interleave_v(
  dwt: &V8Dwt,
  a: &[f32],
  mut a_start: usize,
  width: u32,
  nb_elts_read: u32,
) {
  let mut bi = (NB_ELTS_V8 * dwt.cas as u32) as usize;
  let mut i: u32 = 0;
  i = dwt.win_l_x0;
  while i < dwt.win_l_x1 {
    let dst = bi + (2 * NB_ELTS_V8 * i) as usize;
    let src = a_start + (i as usize).wrapping_mul(width as usize);
    for e in 0..nb_elts_read as usize {
      dwt.wavelet[dst + e].set(a[src + e]);
    }
    i += 1;
  }
  a_start += (dwt.sn as usize).wrapping_mul(width as usize);
  bi = (NB_ELTS_V8 * (1i32 - dwt.cas) as u32) as usize;
  i = dwt.win_h_x0;
  while i < dwt.win_h_x1 {
    let dst = bi + (2 * NB_ELTS_V8 * i) as usize;
    let src = a_start + (i as usize).wrapping_mul(width as usize);
    for e in 0..nb_elts_read as usize {
      dwt.wavelet[dst + e].set(a[src + e]);
    }
    i += 1;
  }
}
fn opj_v8dwt_interleave_partial_v(
  mut dwt: &V8Dwt,
  sa: &SparseArray,
  sa_col: u32,
  nb_elts_read: u32,
) {
  let l_base = (NB_ELTS_V8 * dwt.cas as u32 + 2 * NB_ELTS_V8 * dwt.win_l_x0) as usize;
  let h_base =
    (NB_ELTS_V8 * (1i32 - dwt.cas) as u32 + 2 * NB_ELTS_V8 * dwt.win_h_x0) as usize;
  let ret = sparse_array_read(
    sa,
    sa_col,
    dwt.win_l_x0,
    sa_col.wrapping_add(nb_elts_read),
    dwt.win_l_x1,
    &dwt.wavelet[l_base..],
    1_u32,
    2 * NB_ELTS_V8,
    true,
  );
  debug_assert!(ret);
  let ret = sparse_array_read(
    sa,
    sa_col,
    (dwt.sn as u32).wrapping_add(dwt.win_h_x0),
    sa_col.wrapping_add(nb_elts_read),
    (dwt.sn as u32).wrapping_add(dwt.win_h_x1),
    &dwt.wavelet[h_base..],
    1_u32,
    2 * NB_ELTS_V8,
    true,
  );
  debug_assert!(ret);
}

/// `w` is the flat wavelet buffer and `w_base` the float index the C code's `opj_v8_t*` argument
/// pointed at (its `opj_v8_t` index times `NB_ELTS_V8`).
fn opj_v8dwt_decode_step1(
  w: &[Cell<f32>],
  w_base: usize,
  mut start: u32,
  mut end: u32,
  c: f32,
) {
  /* To be adapted if NB_ELTS_V8 changes */
  for i in start..end {
    let base = w_base + (i * 2 * NB_ELTS_V8) as usize;
    for e in 0..NB_ELTS_V8 as usize {
      let cell = &w[base + e];
      cell.set(cell.get() * c);
    }
  }
}

/// `l_base` / `w_base` are the float indices the C code's two `opj_v8_t*` arguments pointed at.
fn opj_v8dwt_decode_step2(
  w: &[Cell<f32>],
  mut l_base: usize,
  mut w_base: usize,
  mut start: u32,
  mut end: u32,
  mut m: u32,
  mut c: f32,
) {
  const STEP: usize = (2 * NB_ELTS_V8) as usize;
  const N: usize = NB_ELTS_V8 as usize;
  let mut imax = opj_uint_min(end, m);
  if start > 0u32 {
    w_base += STEP * start as usize;
    l_base = w_base - STEP
  }
  /* To be adapted if NB_ELTS_V8 changes */
  for _ in start..imax {
    for e in 0..N {
      let cell = &w[w_base - N + e];
      cell.set(cell.get() + (w[l_base + e].get() + w[w_base + e].get()) * c);
    }
    l_base = w_base;
    w_base += STEP;
  }
  if m < end {
    debug_assert!(m.wrapping_add(1u32) == end);
    c += c;
    for e in 0..N {
      let cell = &w[w_base - N + e];
      cell.set(cell.get() + w[l_base + e].get() * c);
    }
  };
}
/* <summary>                             */
/* Inverse 9-7 wavelet transform in 1-D. */
/* </summary>                            */
fn opj_v8dwt_decode(mut dwt: &V8Dwt) {
  let mut a: i32 = 0;
  let mut b: i32 = 0;
  /* BUG_WEIRD_TWO_INVK (look for this identifier in tcd.c) */
  /* Historic value for 2 / DWT_INV_K */
  /* Normally, we should use invK, but if we do so, we have failures in the */
  /* conformance test, due to MSE and peak errors significantly higher than */
  /* accepted value */
  /* Due to using two_inv_k instead of invK, we have to compensate in tcd.c */
  /* the computation of the stepsize for the non LL subbands */
  let two_inv_k = 1.625_732_4_f32;
  if dwt.cas == 0i32 {
    if !(dwt.dn > 0i32 || dwt.sn > 1i32) {
      return;
    }
    a = 0i32;
    b = 1i32
  } else {
    if !(dwt.sn > 0i32 || dwt.dn > 1i32) {
      return;
    }
    a = 1i32;
    b = 0i32
  }
  // The C code indexes `wavelet` in `opj_v8_t` units; here the buffer is flat floats.
  let group = |k: i32| (NB_ELTS_V8 as i32 * k) as usize;
  opj_v8dwt_decode_step1(dwt.wavelet, group(a), dwt.win_l_x0, dwt.win_l_x1, DWT_K);
  opj_v8dwt_decode_step1(dwt.wavelet, group(b), dwt.win_h_x0, dwt.win_h_x1, two_inv_k);
  opj_v8dwt_decode_step2(
    dwt.wavelet,
    group(b),
    group(a + 1),
    dwt.win_l_x0,
    dwt.win_l_x1,
    opj_int_min(dwt.sn, dwt.dn - a) as u32,
    -DWT_DELTA,
  );
  opj_v8dwt_decode_step2(
    dwt.wavelet,
    group(a),
    group(b + 1),
    dwt.win_h_x0,
    dwt.win_h_x1,
    opj_int_min(dwt.dn, dwt.sn - b) as u32,
    -DWT_GAMMA,
  );
  opj_v8dwt_decode_step2(
    dwt.wavelet,
    group(b),
    group(a + 1),
    dwt.win_l_x0,
    dwt.win_l_x1,
    opj_int_min(dwt.sn, dwt.dn - a) as u32,
    -DWT_BETA,
  );
  opj_v8dwt_decode_step2(
    dwt.wavelet,
    group(a),
    group(b + 1),
    dwt.win_h_x0,
    dwt.win_h_x1,
    opj_int_min(dwt.dn, dwt.sn - b) as u32,
    -DWT_ALPHA,
  );
}

/* <summary>                             */
/* Inverse 9-7 wavelet transform in 2-D. */
/* </summary>                            */
fn opj_dwt_decode_tile_97(tilec: &mut TcdTileComp, mut numres: u32) -> OPJ_BOOL {
  let mut rw = (tilec.resolutions[0].x1 - tilec.resolutions[0].x0) as u32;
  let mut rh = (tilec.resolutions[0].y1 - tilec.resolutions[0].y0) as u32;
  let w = (tilec.resolutions[tilec.minimum_num_resolutions as usize - 1].x1
    - tilec.resolutions[tilec.minimum_num_resolutions as usize - 1].x0) as u32;
  let mut l_data_size: usize = 0;
  if numres == 1u32 {
    return 1i32;
  }
  l_data_size = opj_dwt_max_resolution(&tilec.resolutions, numres) as usize;
  /* overflow check */
  if l_data_size > (usize::MAX).wrapping_div(V8_SIZE) {
    return 0i32;
  }
  // The horizontal and vertical views share this scratch buffer, exactly as in the C code.
  let mut scratch: Vec<Cell<f32>> = Vec::new();
  let scratch_len = l_data_size * NB_ELTS_V8 as usize;
  if scratch.try_reserve_exact(scratch_len).is_err() {
    return 0i32;
  }
  scratch.resize(scratch_len, Cell::new(0f32));
  let mut h = V8Dwt {
    wavelet: &scratch,
    dn: 0,
    sn: 0,
    cas: 0,
    win_l_x0: 0,
    win_l_x1: 0,
    win_h_x0: 0,
    win_h_x1: 0,
  }; /* width of the resolution level computed */
  let mut v = V8Dwt {
    wavelet: &scratch,
    dn: 0,
    sn: 0,
    cas: 0,
    win_l_x0: 0,
    win_l_x1: 0,
    win_h_x0: 0,
    win_h_x1: 0,
  }; /* height of the resolution level computed */

  /* The tile data buffer is allocated for the finest decoded resolution
  (opj_tcd_decode_tile sizes it from resolutions[minimum_num_resolutions-1]),
  so the slice covers exactly that geometry, like the C code's row stride. */
  let total_len = (w as usize).wrapping_mul(
    (tilec.resolutions[tilec.minimum_num_resolutions as usize - 1].y1
      - tilec.resolutions[tilec.minimum_num_resolutions as usize - 1].y0) as usize,
  );
  if total_len > 0 && tilec.data.is_null() {
    return 0i32;
  }
  /* The 9-7 buffer is f32-typed: borrow it directly (the C's in-place
  reinterpretation; no whole-tile copy). */
  let tile_data = tilec.data.as_mut_f32(total_len);

  let mut res_idx = 1usize;
  loop {
    numres = numres.wrapping_sub(1);
    if numres == 0 {
      break;
    }
    let mut aj_start = 0usize;
    let mut j: u32 = 0;
    h.sn = rw as i32;
    v.sn = rh as i32;
    rw = (tilec.resolutions[res_idx].x1 - tilec.resolutions[res_idx].x0) as u32;
    rh = (tilec.resolutions[res_idx].y1 - tilec.resolutions[res_idx].y0) as u32;
    h.dn = rw.wrapping_sub(h.sn as u32) as i32;
    h.cas = tilec.resolutions[res_idx].x0 % 2i32;
    h.win_l_x0 = 0_u32;
    h.win_l_x1 = h.sn as u32;
    h.win_h_x0 = 0_u32;
    h.win_h_x1 = h.dn as u32;
    if (rw as usize).wrapping_mul(rh as usize) >= 16384 {
      /* Large levels: parallelise the horizontal pass over the 8-row groups
      (disjoint row bands); smaller levels stay sequential (the per-level
      parallel overhead dominates there). */
      let h_len = (rw as usize) * NB_ELTS_V8 as usize;
      let (h_sn, h_dn, h_cas) = (h.sn, h.dn, h.cas);
      let stride = w as usize;
      par::for_each_chunks_mut(
        &mut tile_data[..rh as usize * stride],
        stride * NB_ELTS_V8 as usize,
        |_gi, chunk| {
          DWT_SCRATCH_F32.with(|scratch| {
            let mut s = scratch.borrow_mut();
            if s.len() < h_len {
              let need = h_len - s.len();
              if s.try_reserve_exact(need).is_err() {
                return;
              }
              s.resize(h_len, core::cell::Cell::new(0.0));
            }
            let d = v8dwt_local {
              wavelet: &s[..h_len],
              dn: h_dn,
              sn: h_sn,
              cas: h_cas,
              win_l_x0: 0,
              win_l_x1: h_sn as u32,
              win_h_x0: 0,
              win_h_x1: h_dn as u32,
            };
            let remaining = (chunk.len() / stride) as u32;
            opj_v8dwt_interleave_h(&d, chunk, 0, stride as u32, remaining);
            opj_v8dwt_decode(&d);
            for k in 0..rw as usize {
              let g = k * NB_ELTS_V8 as usize;
              for r in 0..remaining as usize {
                chunk[k + r * stride] = d.wavelet[g + r].get();
              }
            }
          });
        },
      );
    } else {
      j = 0_u32;
      while j + (NB_ELTS_V8 - 1) < rh {
        opj_v8dwt_interleave_h(&h, tile_data, aj_start, w, NB_ELTS_V8);
        opj_v8dwt_decode(&h);
        /* To be adapted if NB_ELTS_V8 changes */
        for k in 0..rw as usize {
          let g = k * NB_ELTS_V8 as usize;
          for r in 0..NB_ELTS_V8 as usize {
            tile_data[aj_start + k + r * w as usize] = h.wavelet[g + r].get();
          }
        }
        aj_start += (w * NB_ELTS_V8) as usize;
        j += NB_ELTS_V8;
      }
      if j < rh {
        opj_v8dwt_interleave_h(&h, tile_data, aj_start, w, rh.wrapping_sub(j));
        opj_v8dwt_decode(&h);
        let mut k_0: u32 = 0;
        while k_0 < rw {
          let mut l: u32 = 0;
          while l < rh.wrapping_sub(j) {
            tile_data[aj_start + (k_0 as usize) + (w as usize) * (l as usize)] =
              h.wavelet[(k_0 * NB_ELTS_V8 + l) as usize].get();
            l += 1;
          }
          k_0 += 1;
        }
      }
    }
    v.dn = rh.wrapping_sub(v.sn as u32) as i32;
    v.cas = tilec.resolutions[res_idx].y0 % 2i32;
    v.win_l_x0 = 0_u32;
    v.win_l_x1 = v.sn as u32;
    v.win_h_x0 = 0_u32;
    v.win_h_x1 = v.dn as u32;
    aj_start = 0usize;
    j = rw;
    while j > (NB_ELTS_V8 - 1) {
      opj_v8dwt_interleave_v(&v, tile_data, aj_start, w, NB_ELTS_V8);
      opj_v8dwt_decode(&v);
      let mut k_1: u32 = 0;
      while k_1 < rh {
        let dst = (k_1 as usize).wrapping_mul(w as usize);
        let g = (k_1 * NB_ELTS_V8) as usize;
        for e in 0..NB_ELTS_V8 as usize {
          tile_data[aj_start + dst + e] = v.wavelet[g + e].get();
        }
        k_1 += 1;
      }
      aj_start += NB_ELTS_V8 as usize;
      j -= NB_ELTS_V8;
    }
    if rw & (NB_ELTS_V8 - 1) != 0 {
      let mut k_2: u32 = 0;
      j = rw & (NB_ELTS_V8 - 1);
      opj_v8dwt_interleave_v(&v, tile_data, aj_start, w, j);
      opj_v8dwt_decode(&v);
      k_2 = 0_u32;
      while k_2 < rh {
        let dst = (k_2 as usize).wrapping_mul(w as usize);
        let g = (k_2 * NB_ELTS_V8) as usize;
        for e in 0..j as usize {
          tile_data[aj_start + dst + e] = v.wavelet[g + e].get();
        }
        k_2 += 1;
      }
    }
    res_idx += 1;
  }
  1i32
}
fn opj_dwt_decode_partial_97(tilec: &mut TcdTileComp, mut numres: u32) -> OPJ_BOOL {
  let mut resno: u32 = 0;
  /* This value matches the maximum left/right extension given in tables */
  /* F.2 and F.3 of the standard. Note: in opj_tcd_is_subband_area_of_interest() */
  /* we currently use 3. */
  let filter_width = 4u32; /* width of the resolution level computed */
  let tr_max_idx = numres.wrapping_sub(1u32) as usize; /* height of the resolution level computed */
  let mut rw = (tilec.resolutions[0].x1 - tilec.resolutions[0].x0) as u32;
  let mut rh = (tilec.resolutions[0].y1 - tilec.resolutions[0].y0) as u32;
  let mut l_data_size: usize = 0;
  /* Compute the intersection of the area of interest, expressed in tile coordinates */
  /* with the tile coordinates */
  let mut win_tcx0 = tilec.win_x0;
  let mut win_tcy0 = tilec.win_y0;
  let mut win_tcx1 = tilec.win_x1;
  let mut win_tcy1 = tilec.win_y1;
  if tilec.resolutions[tr_max_idx].x0 == tilec.resolutions[tr_max_idx].x1
    || tilec.resolutions[tr_max_idx].y0 == tilec.resolutions[tr_max_idx].y1
  {
    return 1i32;
  }
  let mut sa = if let Some(sa) = opj_dwt_init_sparse_array(&*tilec, numres) {
    sa
  } else {
    return 0i32;
  };
  if numres == 1u32 {
    let tr_max = &tilec.resolutions[tr_max_idx];
    let win_w = tr_max.win_x1.wrapping_sub(tr_max.win_x0) as usize;
    let win_h = tr_max.win_y1.wrapping_sub(tr_max.win_y0) as usize;
    if win_w == 0 || win_h == 0 {
      // Empty window (e.g. the tile does not intersect the decoding area): nothing to read back,
      // and `data_win` may be null, so skip the read entirely (the forgiving path did the same).
      return 1i32;
    }
    // SAFETY: `data_win` covers the window of interest (`win_w * win_h` samples).
    let win = tilec.data_win.as_mut_f32(win_w.wrapping_mul(win_h));
    let ret = sparse_array_read(
      &sa,
      tr_max.win_x0.wrapping_sub(tr_max.x0 as u32),
      tr_max.win_y0.wrapping_sub(tr_max.y0 as u32),
      tr_max.win_x1.wrapping_sub(tr_max.x0 as u32),
      tr_max.win_y1.wrapping_sub(tr_max.y0 as u32),
      Cell::from_mut(win).as_slice_of_cells(),
      1_u32,
      tr_max.win_x1.wrapping_sub(tr_max.win_x0),
      true,
    );
    debug_assert!(ret);
    return 1i32;
  }
  l_data_size = opj_dwt_max_resolution(&tilec.resolutions, numres) as usize;
  /* overflow check */
  if l_data_size > (usize::MAX).wrapping_div(V8_SIZE) {
    return 0i32;
  }
  // The horizontal and vertical views share this scratch buffer, exactly as in the C code.
  let mut scratch: Vec<Cell<f32>> = Vec::new();
  let scratch_len = l_data_size * NB_ELTS_V8 as usize;
  if scratch.try_reserve_exact(scratch_len).is_err() {
    return 0i32;
  }
  scratch.resize(scratch_len, Cell::new(0f32));
  let mut h = V8Dwt {
    wavelet: &scratch,
    dn: 0,
    sn: 0,
    cas: 0,
    win_l_x0: 0,
    win_l_x1: 0,
    win_h_x0: 0,
    win_h_x1: 0,
  };
  let mut v = V8Dwt {
    wavelet: &scratch,
    dn: 0,
    sn: 0,
    cas: 0,
    win_l_x0: 0,
    win_l_x1: 0,
    win_h_x0: 0,
    win_h_x1: 0,
  };
  resno = 1_u32;
  while resno < numres {
    let mut j: u32 = 0;
    /* Window of interest subband-based coordinates */
    let mut win_ll_x0: u32 = 0;
    let mut win_ll_y0: u32 = 0;
    let mut win_ll_x1: u32 = 0;
    let mut win_ll_y1: u32 = 0;
    let mut win_hl_x0: u32 = 0;
    let mut win_hl_x1: u32 = 0;
    let mut win_lh_y0: u32 = 0;
    let mut win_lh_y1: u32 = 0;
    /* Window of interest tile-resolution-based coordinates */
    let mut win_tr_x0: u32 = 0;
    let mut win_tr_x1: u32 = 0;
    let mut win_tr_y0: u32 = 0;
    let mut win_tr_y1: u32 = 0;
    /* Tile-resolution subband-based coordinates */
    let mut tr_ll_x0: u32 = 0;
    let mut tr_ll_y0: u32 = 0;
    let mut tr_hl_x0: u32 = 0;
    let mut tr_lh_y0: u32 = 0;
    let tr = &tilec.resolutions[resno as usize];
    h.sn = rw as i32;
    v.sn = rh as i32;
    rw = (tr.x1 - tr.x0) as u32;
    rh = (tr.y1 - tr.y0) as u32;
    h.dn = rw.wrapping_sub(h.sn as u32) as i32;
    h.cas = tr.x0 % 2i32;
    v.dn = rh.wrapping_sub(v.sn as u32) as i32;
    v.cas = tr.y0 % 2i32;
    /* Get the subband coordinates for the window of interest */
    /* LL band */
    opj_dwt_get_band_coordinates(
      &*tilec,
      resno,
      0_u32,
      win_tcx0,
      win_tcy0,
      win_tcx1,
      win_tcy1,
      Some(&mut win_ll_x0),
      Some(&mut win_ll_y0),
      Some(&mut win_ll_x1),
      Some(&mut win_ll_y1),
    );
    /* HL band */
    opj_dwt_get_band_coordinates(
      &*tilec,
      resno,
      1_u32,
      win_tcx0,
      win_tcy0,
      win_tcx1,
      win_tcy1,
      Some(&mut win_hl_x0),
      None,
      Some(&mut win_hl_x1),
      None,
    );
    /* LH band */
    opj_dwt_get_band_coordinates(
      &*tilec,
      resno,
      2_u32,
      win_tcx0,
      win_tcy0,
      win_tcx1,
      win_tcy1,
      None,
      Some(&mut win_lh_y0),
      None,
      Some(&mut win_lh_y1),
    );
    /* Beware: band index for non-LL0 resolution are 0=HL, 1=LH and 2=HH */
    tr_ll_x0 = tr.bands[1_usize].x0 as u32;
    tr_ll_y0 = tr.bands[0_usize].y0 as u32;
    tr_hl_x0 = tr.bands[0_usize].x0 as u32;
    tr_lh_y0 = tr.bands[1_usize].y0 as u32;
    /* Subtract the origin of the bands for this tile, to the subwindow */
    /* of interest band coordinates, so as to get them relative to the */
    /* tile */
    win_ll_x0 = opj_uint_subs(win_ll_x0, tr_ll_x0);
    win_ll_y0 = opj_uint_subs(win_ll_y0, tr_ll_y0);
    win_ll_x1 = opj_uint_subs(win_ll_x1, tr_ll_x0);
    win_ll_y1 = opj_uint_subs(win_ll_y1, tr_ll_y0);
    win_hl_x0 = opj_uint_subs(win_hl_x0, tr_hl_x0);
    win_hl_x1 = opj_uint_subs(win_hl_x1, tr_hl_x0);
    win_lh_y0 = opj_uint_subs(win_lh_y0, tr_lh_y0);
    win_lh_y1 = opj_uint_subs(win_lh_y1, tr_lh_y0);
    opj_dwt_segment_grow(
      filter_width,
      h.sn as u32,
      &mut win_ll_x0,
      &mut win_ll_x1,
    );
    opj_dwt_segment_grow(
      filter_width,
      h.dn as u32,
      &mut win_hl_x0,
      &mut win_hl_x1,
    );
    opj_dwt_segment_grow(
      filter_width,
      v.sn as u32,
      &mut win_ll_y0,
      &mut win_ll_y1,
    );
    opj_dwt_segment_grow(
      filter_width,
      v.dn as u32,
      &mut win_lh_y0,
      &mut win_lh_y1,
    );
    /* Compute the tile-resolution-based coordinates for the window of interest */
    if h.cas == 0i32 {
      win_tr_x0 = opj_uint_min(
        (2u32).wrapping_mul(win_ll_x0),
        (2u32).wrapping_mul(win_hl_x0).wrapping_add(1u32),
      );
      win_tr_x1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_ll_x1),
          (2u32).wrapping_mul(win_hl_x1).wrapping_add(1u32),
        ),
        rw,
      )
    } else {
      win_tr_x0 = opj_uint_min(
        (2u32).wrapping_mul(win_hl_x0),
        (2u32).wrapping_mul(win_ll_x0).wrapping_add(1u32),
      );
      win_tr_x1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_hl_x1),
          (2u32).wrapping_mul(win_ll_x1).wrapping_add(1u32),
        ),
        rw,
      )
    }
    if v.cas == 0i32 {
      win_tr_y0 = opj_uint_min(
        (2u32).wrapping_mul(win_ll_y0),
        (2u32).wrapping_mul(win_lh_y0).wrapping_add(1u32),
      );
      win_tr_y1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_ll_y1),
          (2u32).wrapping_mul(win_lh_y1).wrapping_add(1u32),
        ),
        rh,
      )
    } else {
      win_tr_y0 = opj_uint_min(
        (2u32).wrapping_mul(win_lh_y0),
        (2u32).wrapping_mul(win_ll_y0).wrapping_add(1u32),
      );
      win_tr_y1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_lh_y1),
          (2u32).wrapping_mul(win_ll_y1).wrapping_add(1u32),
        ),
        rh,
      )
    }
    h.win_l_x0 = win_ll_x0;
    h.win_l_x1 = win_ll_x1;
    h.win_h_x0 = win_hl_x0;
    h.win_h_x1 = win_hl_x1;
    j = 0_u32;
    while j + (NB_ELTS_V8 - 1) < rh {
      if j + (NB_ELTS_V8 - 1) >= win_ll_y0 && j < win_ll_y1
        || j + (NB_ELTS_V8 - 1) >= win_lh_y0.wrapping_add(v.sn as u32)
          && j < win_lh_y1.wrapping_add(v.sn as u32)
      {
        opj_v8dwt_interleave_partial_h(&h, &sa, j, opj_uint_min(NB_ELTS_V8, rh.wrapping_sub(j)));
        opj_v8dwt_decode(&h);
        if !sparse_array_write(
          &mut sa,
          win_tr_x0,
          j,
          win_tr_x1,
          j.wrapping_add(8u32),
          &h.wavelet[(win_tr_x0 * NB_ELTS_V8) as usize..],
          NB_ELTS_V8,
          1_u32,
          true,
        ) {
          return 0i32;
        }
      }
      j += NB_ELTS_V8;
    }
    if j < rh
      && (j + (NB_ELTS_V8 - 1) >= win_ll_y0 && j < win_ll_y1
        || j + (NB_ELTS_V8 - 1) >= win_lh_y0.wrapping_add(v.sn as u32)
          && j < win_lh_y1.wrapping_add(v.sn as u32))
    {
      opj_v8dwt_interleave_partial_h(&h, &sa, j, rh.wrapping_sub(j));
      opj_v8dwt_decode(&h);
      if !sparse_array_write(
        &mut sa,
        win_tr_x0,
        j,
        win_tr_x1,
        rh,
        &h.wavelet[(win_tr_x0 * NB_ELTS_V8) as usize..],
        NB_ELTS_V8,
        1_u32,
        true,
      ) {
        return 0i32;
      }
    }
    v.win_l_x0 = win_ll_y0;
    v.win_l_x1 = win_ll_y1;
    v.win_h_x0 = win_lh_y0;
    v.win_h_x1 = win_lh_y1;
    j = win_tr_x0;
    while j < win_tr_x1 {
      let mut nb_elts = opj_uint_min(NB_ELTS_V8, win_tr_x1.wrapping_sub(j));
      opj_v8dwt_interleave_partial_v(&v, &sa, j, nb_elts);
      opj_v8dwt_decode(&v);
      if !sparse_array_write(
        &mut sa,
        j,
        win_tr_y0,
        j.wrapping_add(nb_elts),
        win_tr_y1,
        &h.wavelet[(win_tr_y0 * NB_ELTS_V8) as usize..],
        1_u32,
        NB_ELTS_V8,
        true,
      ) {
        return 0i32;
      }
      j += NB_ELTS_V8;
    }
    resno += 1;
  }
  let tr_max = &tilec.resolutions[tr_max_idx];
  let win_w = tr_max.win_x1.wrapping_sub(tr_max.win_x0) as usize;
  let win_h = tr_max.win_y1.wrapping_sub(tr_max.win_y0) as usize;
  if win_w == 0 || win_h == 0 {
    // Empty window (e.g. the tile does not intersect the decoding area): nothing to read back,
    // and `data_win` may be null, so skip the read entirely (the forgiving path did the same).
    return 1i32;
  }
  // SAFETY: `data_win` covers the window of interest (`win_w * win_h` samples).
  let win = tilec.data_win.as_mut_f32(win_w.wrapping_mul(win_h));
  let ret_0 = sparse_array_read(
    &sa,
    tr_max.win_x0.wrapping_sub(tr_max.x0 as u32),
    tr_max.win_y0.wrapping_sub(tr_max.y0 as u32),
    tr_max.win_x1.wrapping_sub(tr_max.x0 as u32),
    tr_max.win_y1.wrapping_sub(tr_max.y0 as u32),
    Cell::from_mut(win).as_slice_of_cells(),
    1_u32,
    tr_max.win_x1.wrapping_sub(tr_max.win_x0),
    true,
  );
  debug_assert!(ret_0);
  1i32
}

pub(crate) fn opj_dwt_decode_real(
  whole_tile_decoding: OPJ_BOOL,
  tilec: &mut TcdTileComp,
  numres: u32,
) -> OPJ_BOOL {
  if whole_tile_decoding != 0 {
    opj_dwt_decode_tile_97(tilec, numres)
  } else {
    opj_dwt_decode_partial_97(tilec, numres)
  }
}
