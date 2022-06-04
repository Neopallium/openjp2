use super::math::*;
use super::openjpeg::*;
use super::sparse_array::*;
use super::thread::*;
use ::libc;

use super::malloc::*;

extern "C" {
  fn floor(_: libc::c_double) -> libc::c_double;

  fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
}
/* Where void* is a OPJ_INT32* for 5x3 and OPJ_FLOAT32* for 9x7 */
pub type opj_encode_and_deinterleave_h_one_row_fnptr_type = Option<
  unsafe extern "C" fn(
    _: *mut libc::c_void,
    _: *mut libc::c_void,
    _: OPJ_UINT32,
    _: OPJ_BOOL,
  ) -> (),
>;
/* Forward transform, for the vertical pass, processing cols columns */
/* where cols <= NB_ELTS_V8 */
/* Where void* is a OPJ_INT32* for 5x3 and OPJ_FLOAT32* for 9x7 */
pub type opj_encode_and_deinterleave_v_fnptr_type = Option<
  unsafe extern "C" fn(
    _: *mut libc::c_void,
    _: *mut libc::c_void,
    _: OPJ_UINT32,
    _: OPJ_BOOL,
    _: OPJ_UINT32,
    _: OPJ_UINT32,
  ) -> (),
>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_dwt_encode_h_job_t {
  pub h: opj_dwt_t,
  pub rw: OPJ_UINT32,
  pub w: OPJ_UINT32,
  pub tiledp: *mut OPJ_INT32,
  pub min_j: OPJ_UINT32,
  pub max_j: OPJ_UINT32,
  pub p_function: opj_encode_and_deinterleave_h_one_row_fnptr_type,
}
/* * @name Local data structures */
/*@{*/
pub type opj_dwt_t = dwt_local;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct dwt_local {
  pub mem: *mut OPJ_INT32,
  pub dn: OPJ_INT32,
  pub sn: OPJ_INT32,
  pub cas: OPJ_INT32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_dwt_encode_v_job_t {
  pub v: opj_dwt_t,
  pub rh: OPJ_UINT32,
  pub w: OPJ_UINT32,
  pub tiledp: *mut OPJ_INT32,
  pub min_j: OPJ_UINT32,
  pub max_j: OPJ_UINT32,
  pub p_encode_and_deinterleave_v: opj_encode_and_deinterleave_v_fnptr_type,
}
pub type opj_sparse_array_int32_t = opj_sparse_array_int32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_dwt_decode_v_job_t {
  pub v: opj_dwt_t,
  pub rh: OPJ_UINT32,
  pub w: OPJ_UINT32,
  pub tiledp: *mut OPJ_INT32,
  pub min_j: OPJ_UINT32,
  pub max_j: OPJ_UINT32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_dwt_decode_h_job_t {
  pub h: opj_dwt_t,
  pub rw: OPJ_UINT32,
  pub w: OPJ_UINT32,
  pub tiledp: *mut OPJ_INT32,
  pub min_j: OPJ_UINT32,
  pub max_j: OPJ_UINT32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union opj_v8_t {
  pub f: [OPJ_FLOAT32; 8],
}
pub type opj_v8dwt_t = v8dwt_local;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct v8dwt_local {
  pub wavelet: *mut opj_v8_t,
  pub dn: OPJ_INT32,
  pub sn: OPJ_INT32,
  pub cas: OPJ_INT32,
  pub win_l_x0: OPJ_UINT32,
  pub win_l_x1: OPJ_UINT32,
  pub win_h_x0: OPJ_UINT32,
  pub win_h_x1: OPJ_UINT32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_dwt97_decode_v_job_t {
  pub v: opj_v8dwt_t,
  pub rh: OPJ_UINT32,
  pub w: OPJ_UINT32,
  pub aj: *mut OPJ_FLOAT32,
  pub nb_columns: OPJ_UINT32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_dwt97_decode_h_job_t {
  pub h: opj_v8dwt_t,
  pub rw: OPJ_UINT32,
  pub w: OPJ_UINT32,
  pub aj: *mut OPJ_FLOAT32,
  pub nb_rows: OPJ_UINT32,
}

/* number of elements in high pass band */
/* number of elements in low pass band */
/* 0 = start on even coord, 1 = start on odd coord */
/* number of elements in high pass band */
/* number of elements in low pass band */
/* 0 = start on even coord, 1 = start on odd coord */
/* start coord in low pass band */
/* end coord in low pass band */
/* start coord in high pass band */
/* end coord in high pass band */

/* From table F.4 from the standard */
const opj_dwt_alpha: f32 = -1.586134342;
const opj_dwt_beta: f32 = -0.052980118;
const opj_dwt_gamma: f32 = 0.882911075;
const opj_dwt_delta: f32 = 0.443506852;
const opj_K: f32 = 1.230174105;
const opj_invK: f32 = (1.0 / 1.230174105);

/* <summary>                                                              */
/* This table contains the norms of the 5-3 wavelets for different bands. */
/* </summary>                                                             */
/* FIXME! the array should really be extended up to 33 resolution levels */
/* See https://github.com/uclouvain/openjpeg/issues/493 */
const opj_dwt_norms: [[f64; 10]; 4] = [
  [
    1.000, 1.500, 2.750, 5.375, 10.68, 21.34, 42.67, 85.33, 170.7,
    341.3,
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
/* FIXME! the array should really be extended up to 33 resolution levels */
/* See https://github.com/uclouvain/openjpeg/issues/493 */
const opj_dwt_norms_real: [[f64; 10]; 4] = [
  [
    1.000, 1.965, 4.177, 8.403, 16.90, 33.84, 67.69, 135.3, 270.6,
    540.9,
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
unsafe fn opj_dwt_deinterleave_h(
  mut a: *const OPJ_INT32,
  mut b: *mut OPJ_INT32,
  mut dn: OPJ_INT32,
  mut sn: OPJ_INT32,
  mut cas: OPJ_INT32,
) {
  let mut i: OPJ_INT32 = 0;
  let mut l_dest = b;
  let mut l_src = a.offset(cas as isize);
  i = 0i32;
  while i < sn {
    let fresh0 = l_dest;
    l_dest = l_dest.offset(1);
    *fresh0 = *l_src;
    l_src = l_src.offset(2);
    i += 1
  }
  l_dest = b.offset(sn as isize);
  l_src = a.offset(1).offset(-(cas as isize));
  i = 0i32;
  while i < dn {
    let fresh1 = l_dest;
    l_dest = l_dest.offset(1);
    *fresh1 = *l_src;
    l_src = l_src.offset(2);
    i += 1
  }
}
/* <summary>                             */
/* Inverse lazy transform (horizontal).  */
/* </summary>                            */
unsafe fn opj_dwt_interleave_h(mut h: *const opj_dwt_t, mut a: *mut OPJ_INT32) {
  let mut ai: *const OPJ_INT32 = a;
  let mut bi = (*h).mem.offset((*h).cas as isize);
  let mut i = (*h).sn;
  loop {
    let fresh2 = i;
    i = i - 1;
    if !(fresh2 != 0) {
      break;
    }
    let fresh3 = ai;
    ai = ai.offset(1);
    *bi = *fresh3;
    bi = bi.offset(2)
  }
  ai = a.offset((*h).sn as isize);
  bi = (*h)
    .mem
    .offset(1)
    .offset(-((*h).cas as isize));
  i = (*h).dn;
  loop {
    let fresh4 = i;
    i = i - 1;
    if !(fresh4 != 0) {
      break;
    }
    let fresh5 = ai;
    ai = ai.offset(1);
    *bi = *fresh5;
    bi = bi.offset(2)
  }
}
/* <summary>                             */
/* Inverse lazy transform (vertical).    */
/* </summary>                            */
unsafe fn opj_dwt_interleave_v(
  mut v: *const opj_dwt_t,
  mut a: *mut OPJ_INT32,
  mut x: OPJ_INT32,
) {
  let mut ai: *const OPJ_INT32 = a;
  let mut bi = (*v).mem.offset((*v).cas as isize);
  let mut i = (*v).sn;
  loop {
    let fresh6 = i;
    i = i - 1;
    if !(fresh6 != 0) {
      break;
    }
    *bi = *ai;
    bi = bi.offset(2);
    ai = ai.offset(x as isize)
  }
  ai = a.offset(((*v).sn as libc::c_ulong).wrapping_mul(x as OPJ_SIZE_T) as isize);
  bi = (*v)
    .mem
    .offset(1)
    .offset(-((*v).cas as isize));
  i = (*v).dn;
  loop {
    let fresh7 = i;
    i = i - 1;
    if !(fresh7 != 0) {
      break;
    }
    *bi = *ai;
    bi = bi.offset(2);
    ai = ai.offset(x as isize)
  }
}
/* STANDARD_SLOW_VERSION */
/* <summary>                            */
/* Inverse 5-3 wavelet transform in 1-D. */
/* </summary>                           */
unsafe fn opj_dwt_decode_1_(
  mut a: *mut OPJ_INT32,
  mut dn: OPJ_INT32,
  mut sn: OPJ_INT32,
  mut cas: OPJ_INT32,
) {
  let mut i: OPJ_INT32 = 0;
  if cas == 0 {
    if dn > 0i32 || sn > 1i32 {
      /* NEW :  CASE ONE ELEMENT */
      i = 0i32;
      while i < sn {
        let ref mut fresh8 = *a.offset((i * 2i32) as isize);
        *fresh8 -= (if (i - 1i32) < 0i32 {
          *a.offset((1i32 + 0i32 * 2i32) as isize)
        } else {
          (if i - 1i32 >= dn {
            *a.offset((1i32 + (dn - 1i32) * 2i32) as isize)
          } else {
            *a.offset((1i32 + (i - 1i32) * 2i32) as isize)
          })
        }) + (if i < 0i32 {
          *a.offset((1i32 + 0i32 * 2i32) as isize)
        } else {
          (if i >= dn {
            *a.offset((1i32 + (dn - 1i32) * 2i32) as isize)
          } else {
            *a.offset((1i32 + i * 2i32) as isize)
          })
        }) + 2i32
          >> 2i32;
        i += 1
      }
      i = 0i32;
      while i < dn {
        let ref mut fresh9 = *a.offset((1i32 + i * 2i32) as isize);
        *fresh9 += (if i < 0i32 {
          *a.offset((0i32 * 2i32) as isize)
        } else {
          (if i >= sn {
            *a.offset(((sn - 1i32) * 2i32) as isize)
          } else {
            *a.offset((i * 2i32) as isize)
          })
        }) + (if (i + 1i32) < 0i32 {
          *a.offset((0i32 * 2i32) as isize)
        } else {
          (if i + 1i32 >= sn {
            *a.offset(((sn - 1i32) * 2i32) as isize)
          } else {
            *a.offset(((i + 1i32) * 2i32) as isize)
          })
        }) >> 1i32;
        i += 1
      }
    }
  } else if sn == 0 && dn == 1i32 {
    /* NEW :  CASE ONE ELEMENT */
    let ref mut fresh10 = *a.offset((0i32 * 2i32) as isize);
    *fresh10 /= 2i32
  } else {
    i = 0i32;
    while i < sn {
      let ref mut fresh11 = *a.offset((1i32 + i * 2i32) as isize);
      *fresh11 -= (if i < 0i32 {
        *a.offset((0i32 * 2i32) as isize)
      } else {
        (if i >= dn {
          *a.offset(((dn - 1i32) * 2i32) as isize)
        } else {
          *a.offset((i * 2i32) as isize)
        })
      }) + (if (i + 1i32) < 0i32 {
        *a.offset((0i32 * 2i32) as isize)
      } else {
        (if i + 1i32 >= dn {
          *a.offset(((dn - 1i32) * 2i32) as isize)
        } else {
          *a.offset(((i + 1i32) * 2i32) as isize)
        })
      }) + 2i32
        >> 2i32;
      i += 1
    }
    i = 0i32;
    while i < dn {
      let ref mut fresh12 = *a.offset((i * 2i32) as isize);
      *fresh12 += (if i < 0i32 {
        *a.offset((1i32 + 0i32 * 2i32) as isize)
      } else {
        (if i >= sn {
          *a.offset((1i32 + (sn - 1i32) * 2i32) as isize)
        } else {
          *a.offset((1i32 + i * 2i32) as isize)
        })
      }) + (if (i - 1i32) < 0i32 {
        *a.offset((1i32 + 0i32 * 2i32) as isize)
      } else {
        (if i - 1i32 >= sn {
          *a.offset((1i32 + (sn - 1i32) * 2i32) as isize)
        } else {
          *a.offset((1i32 + (i - 1i32) * 2i32) as isize)
        })
      }) >> 1i32;
      i += 1
    }
  };
}
unsafe fn opj_dwt_decode_1(mut v: *const opj_dwt_t) {
  opj_dwt_decode_1_((*v).mem, (*v).dn, (*v).sn, (*v).cas);
}
/* STANDARD_SLOW_VERSION */
/* !defined(STANDARD_SLOW_VERSION) */
/* <summary>                            */
/* Inverse 5-3 wavelet transform in 1-D for one row. */
/* </summary>                           */
/* Performs interleave, inverse wavelet transform and copy back to buffer */
unsafe fn opj_idwt53_h(mut dwt: *const opj_dwt_t, mut tiledp: *mut OPJ_INT32) {
  /* For documentation purpose */
  opj_dwt_interleave_h(dwt, tiledp);
  opj_dwt_decode_1(dwt);
  memcpy(
    tiledp as *mut libc::c_void,
    (*dwt).mem as *const libc::c_void,
    (((*dwt).sn + (*dwt).dn) as OPJ_UINT32 as libc::c_ulong)
      .wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
  );
}
/* (defined(__SSE2__) || defined(__AVX2__)) && !defined(STANDARD_SLOW_VERSION) */
/* !defined(STANDARD_SLOW_VERSION) */
/* <summary>                            */
/* Inverse vertical 5-3 wavelet transform in 1-D for several columns. */
/* </summary>                           */
/* Performs interleave, inverse wavelet transform and copy back to buffer */
unsafe fn opj_idwt53_v(
  mut dwt: *const opj_dwt_t,
  mut tiledp_col: *mut OPJ_INT32,
  mut stride: OPJ_SIZE_T,
  mut nb_cols: OPJ_INT32,
) {
  /* For documentation purpose */
  let mut k: OPJ_INT32 = 0;
  let mut c: OPJ_INT32 = 0;
  c = 0i32;
  while c < nb_cols {
    opj_dwt_interleave_v(dwt, tiledp_col.offset(c as isize), stride as OPJ_INT32);
    opj_dwt_decode_1(dwt);
    k = 0i32;
    while k < (*dwt).sn + (*dwt).dn {
      *tiledp_col.offset(
        (c as u64).wrapping_add((k as u64).wrapping_mul(stride)) as isize,
      ) = *(*dwt).mem.offset(k as isize);
      k += 1
    }
    c += 1
  }
}
unsafe fn opj_dwt_encode_step1_combined(
  mut fw: *mut OPJ_FLOAT32,
  mut iters_c1: OPJ_UINT32,
  mut iters_c2: OPJ_UINT32,
  c1: OPJ_FLOAT32,
  c2: OPJ_FLOAT32,
) {
  let mut i = 0 as OPJ_UINT32;
  let iters_common = opj_uint_min(iters_c1, iters_c2);

  assert!(
    fw as OPJ_SIZE_T & 0xfu64 == 0u64
  );
  assert!(opj_int_abs(iters_c1 as OPJ_INT32 - iters_c2 as OPJ_INT32) <= 1i32);
  while i.wrapping_add(3u32) < iters_common {
    let ref mut fresh13 = *fw.offset(0);
    *fresh13 *= c1;
    let ref mut fresh14 = *fw.offset(1);
    *fresh14 *= c2;
    let ref mut fresh15 = *fw.offset(2);
    *fresh15 *= c1;
    let ref mut fresh16 = *fw.offset(3);
    *fresh16 *= c2;
    let ref mut fresh17 = *fw.offset(4);
    *fresh17 *= c1;
    let ref mut fresh18 = *fw.offset(5);
    *fresh18 *= c2;
    let ref mut fresh19 = *fw.offset(6);
    *fresh19 *= c1;
    let ref mut fresh20 = *fw.offset(7);
    *fresh20 *= c2;
    fw = fw.offset(8);
    i =
      (i as libc::c_uint).wrapping_add(4u32) as OPJ_UINT32
  }
  while i < iters_common {
    let ref mut fresh21 = *fw.offset(0);
    *fresh21 *= c1;
    let ref mut fresh22 = *fw.offset(1);
    *fresh22 *= c2;
    fw = fw.offset(2);
    i = i.wrapping_add(1)
  }
  if i < iters_c1 {
    let ref mut fresh23 = *fw.offset(0);
    *fresh23 *= c1
  } else if i < iters_c2 {
    let ref mut fresh24 = *fw.offset(1);
    *fresh24 *= c2
  };
}
unsafe fn opj_dwt_encode_step2(
  mut fl: *mut OPJ_FLOAT32,
  mut fw: *mut OPJ_FLOAT32,
  mut end: OPJ_UINT32,
  mut m: OPJ_UINT32,
  mut c: OPJ_FLOAT32,
) {
  let mut i: OPJ_UINT32 = 0;
  let mut imax = opj_uint_min(end, m);
  if imax > 0u32 {
    let ref mut fresh25 = *fw.offset(-(1i32) as isize);
    *fresh25 += (*fl.offset(0) + *fw.offset(0)) * c;
    fw = fw.offset(2);
    i = 1 as OPJ_UINT32;
    while i.wrapping_add(3u32) < imax {
      let ref mut fresh26 = *fw.offset(-(1i32) as isize);
      *fresh26 +=
        (*fw.offset(-(2i32) as isize) + *fw.offset(0)) * c;
      let ref mut fresh27 = *fw.offset(1);
      *fresh27 +=
        (*fw.offset(0) + *fw.offset(2)) * c;
      let ref mut fresh28 = *fw.offset(3);
      *fresh28 +=
        (*fw.offset(2) + *fw.offset(4)) * c;
      let ref mut fresh29 = *fw.offset(5);
      *fresh29 +=
        (*fw.offset(4) + *fw.offset(6)) * c;
      fw = fw.offset(8);
      i = (i as libc::c_uint).wrapping_add(4u32) as OPJ_UINT32
        as OPJ_UINT32
    }
    while i < imax {
      let ref mut fresh30 = *fw.offset(-(1i32) as isize);
      *fresh30 +=
        (*fw.offset(-(2i32) as isize) + *fw.offset(0)) * c;
      fw = fw.offset(2);
      i = i.wrapping_add(1)
    }
  }
  if m < end {
    assert!(m.wrapping_add(1u32) == end);
    let ref mut fresh31 = *fw.offset(-(1i32) as isize);
    *fresh31 += 2 as libc::c_float * *fw.offset(-(2i32) as isize) * c
  };
}
/* *
Forward 9-7 wavelet transform in 1-D
*/
unsafe fn opj_dwt_encode_1_real(
  mut aIn: *mut libc::c_void,
  mut dn: OPJ_INT32,
  mut sn: OPJ_INT32,
  mut cas: OPJ_INT32,
) {
  let mut w = aIn as *mut OPJ_FLOAT32;
  let mut a: OPJ_INT32 = 0;
  let mut b: OPJ_INT32 = 0;
  assert!(dn + sn > 1i32);
  if cas == 0i32 {
    a = 0i32;
    b = 1i32
  } else {
    a = 1i32;
    b = 0i32
  }
  opj_dwt_encode_step2(
    w.offset(a as isize),
    w.offset(b as isize).offset(1),
    dn as OPJ_UINT32,
    opj_int_min(dn, sn - b) as OPJ_UINT32,
    opj_dwt_alpha,
  );
  opj_dwt_encode_step2(
    w.offset(b as isize),
    w.offset(a as isize).offset(1),
    sn as OPJ_UINT32,
    opj_int_min(sn, dn - a) as OPJ_UINT32,
    opj_dwt_beta,
  );
  opj_dwt_encode_step2(
    w.offset(a as isize),
    w.offset(b as isize).offset(1),
    dn as OPJ_UINT32,
    opj_int_min(dn, sn - b) as OPJ_UINT32,
    opj_dwt_gamma,
  );
  opj_dwt_encode_step2(
    w.offset(b as isize),
    w.offset(a as isize).offset(1),
    sn as OPJ_UINT32,
    opj_int_min(sn, dn - a) as OPJ_UINT32,
    opj_dwt_delta,
  );
  if a == 0i32 {
    opj_dwt_encode_step1_combined(w, sn as OPJ_UINT32, dn as OPJ_UINT32, opj_invK, opj_K);
  } else {
    opj_dwt_encode_step1_combined(w, dn as OPJ_UINT32, sn as OPJ_UINT32, opj_K, opj_invK);
  };
}
/* *
Explicit calculation of the Quantization Stepsizes
*/
unsafe fn opj_dwt_encode_stepsize(
  mut stepsize: OPJ_INT32,
  mut numbps: OPJ_INT32,
  mut bandno_stepsize: *mut opj_stepsize_t,
) {
  let mut p: OPJ_INT32 = 0;
  let mut n: OPJ_INT32 = 0;
  p = opj_int_floorlog2(stepsize) - 13i32;
  n = 11i32 - opj_int_floorlog2(stepsize);
  (*bandno_stepsize).mant = (if n < 0i32 {
    (stepsize) >> -n
  } else {
    (stepsize) << n
  }) & 0x7ffi32;
  (*bandno_stepsize).expn = numbps - p;
}
/*
==========================================================
   DWT interface
==========================================================
*/
/* * Process one line for the horizontal pass of the 5x3 forward transform */
unsafe extern "C" fn opj_dwt_encode_and_deinterleave_h_one_row(
  mut rowIn: *mut libc::c_void,
  mut tmpIn: *mut libc::c_void,
  mut width: OPJ_UINT32,
  mut even: OPJ_BOOL,
) {
  let mut row = rowIn as *mut OPJ_INT32;
  let mut tmp = tmpIn as *mut OPJ_INT32;
  let sn = (width.wrapping_add(
    (if even != 0 {
      1i32
    } else {
      0i32
    }) as libc::c_uint,
  ) >> 1i32) as OPJ_INT32;
  let dn = width.wrapping_sub(sn as OPJ_UINT32) as OPJ_INT32;
  if even != 0 {
    if width > 1u32 {
      let mut i: OPJ_INT32 = 0;
      i = 0i32;
      while i < sn - 1i32 {
        *tmp.offset((sn + i) as isize) = *row
          .offset((2i32 * i + 1i32) as isize)
          - (*row.offset((i * 2i32) as isize)
            + *row.offset(((i + 1i32) * 2i32) as isize)
            >> 1i32);
        i += 1
      }
      if width.wrapping_rem(2u32) == 0u32 {
        *tmp.offset((sn + i) as isize) = *row
          .offset((2i32 * i + 1i32) as isize)
          - *row.offset((i * 2i32) as isize)
      }
      let ref mut fresh32 = *row.offset(0);
      *fresh32 +=
        *tmp.offset(sn as isize) + *tmp.offset(sn as isize) + 2i32 >> 2i32;
      i = 1i32;
      while i < dn {
        *row.offset(i as isize) = *row.offset((2i32 * i) as isize)
          + (*tmp.offset((sn + (i - 1i32)) as isize)
            + *tmp.offset((sn + i) as isize)
            + 2i32
            >> 2i32);
        i += 1
      }
      if width.wrapping_rem(2u32) == 1u32 {
        *row.offset(i as isize) = *row.offset((2i32 * i) as isize)
          + (*tmp.offset((sn + (i - 1i32)) as isize)
            + *tmp.offset((sn + (i - 1i32)) as isize)
            + 2i32
            >> 2i32)
      }
      memcpy(
        row.offset(sn as isize) as *mut libc::c_void,
        tmp.offset(sn as isize) as *const libc::c_void,
        (dn as OPJ_SIZE_T).wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
      );
    }
  } else if width == 1u32 {
    let ref mut fresh33 = *row.offset(0);
    *fresh33 *= 2i32
  } else {
    let mut i_0: OPJ_INT32 = 0;
    *tmp.offset((sn + 0i32) as isize) =
      *row.offset(0) - *row.offset(1);
    i_0 = 1i32;
    while i_0 < sn {
      *tmp.offset((sn + i_0) as isize) = *row.offset((2i32 * i_0) as isize)
        - (*row.offset((2i32 * i_0 + 1i32) as isize)
          + *row.offset((2i32 * (i_0 - 1i32) + 1i32) as isize)
          >> 1i32);
      i_0 += 1
    }
    if width.wrapping_rem(2u32) == 1u32 {
      *tmp.offset((sn + i_0) as isize) = *row.offset((2i32 * i_0) as isize)
        - *row.offset((2i32 * (i_0 - 1i32) + 1i32) as isize)
    }
    i_0 = 0i32;
    while i_0 < dn - 1i32 {
      *row.offset(i_0 as isize) = *row.offset((2i32 * i_0 + 1i32) as isize)
        + (*tmp.offset((sn + i_0) as isize)
          + *tmp.offset((sn + i_0 + 1i32) as isize)
          + 2i32
          >> 2i32);
      i_0 += 1
    }
    if width.wrapping_rem(2u32) == 0u32 {
      *row.offset(i_0 as isize) = *row.offset((2i32 * i_0 + 1i32) as isize)
        + (*tmp.offset((sn + i_0) as isize) + *tmp.offset((sn + i_0) as isize) + 2i32
          >> 2i32)
    }
    memcpy(
      row.offset(sn as isize) as *mut libc::c_void,
      tmp.offset(sn as isize) as *const libc::c_void,
      (dn as OPJ_SIZE_T).wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
    );
  };
}
/* * Process one line for the horizontal pass of the 9x7 forward transform */
unsafe extern "C" fn opj_dwt_encode_and_deinterleave_h_one_row_real(
  mut rowIn: *mut libc::c_void,
  mut tmpIn: *mut libc::c_void,
  mut width: OPJ_UINT32,
  mut even: OPJ_BOOL,
) {
  let mut row = rowIn as *mut OPJ_FLOAT32;
  let mut tmp = tmpIn as *mut OPJ_FLOAT32;
  let sn = (width.wrapping_add(
    (if even != 0 {
      1i32
    } else {
      0i32
    }) as libc::c_uint,
  ) >> 1i32) as OPJ_INT32;
  let dn = width.wrapping_sub(sn as OPJ_UINT32) as OPJ_INT32;
  if width == 1u32 {
    return;
  }
  memcpy(
    tmp as *mut libc::c_void,
    row as *const libc::c_void,
    (width as libc::c_ulong).wrapping_mul(::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong),
  );
  opj_dwt_encode_1_real(
    tmp as *mut libc::c_void,
    dn,
    sn,
    if even != 0 {
      0i32
    } else {
      1i32
    },
  );
  opj_dwt_deinterleave_h(
    tmp as *mut OPJ_INT32,
    row as *mut OPJ_INT32,
    dn,
    sn,
    if even != 0 {
      0i32
    } else {
      1i32
    },
  );
}
unsafe extern "C" fn opj_dwt_encode_h_func(
  mut user_data: *mut libc::c_void,
  mut _tls: *mut opj_tls_t,
) {
  let mut j: OPJ_UINT32 = 0;
  let mut job = 0 as *mut opj_dwt_encode_h_job_t;
  job = user_data as *mut opj_dwt_encode_h_job_t;
  j = (*job).min_j;
  while j < (*job).max_j {
    let mut aj = (*job).tiledp.offset(j.wrapping_mul((*job).w) as isize);
    Some((*job).p_function.expect("non-null function pointer")).expect("non-null function pointer")(
      aj as *mut libc::c_void,
      (*job).h.mem as *mut libc::c_void,
      (*job).rw,
      if (*job).h.cas == 0i32 {
        1i32
      } else {
        0i32
      },
    );
    j = j.wrapping_add(1)
  }
  opj_aligned_free((*job).h.mem as *mut libc::c_void);
  opj_free(job as *mut libc::c_void);
}
unsafe extern "C" fn opj_dwt_encode_v_func(
  mut user_data: *mut libc::c_void,
  mut _tls: *mut opj_tls_t,
) {
  let mut j: OPJ_UINT32 = 0;
  let mut job = 0 as *mut opj_dwt_encode_v_job_t;
  job = user_data as *mut opj_dwt_encode_v_job_t;
  j = (*job).min_j;
  while j
    .wrapping_add(8u32)
    .wrapping_sub(1u32)
    < (*job).max_j
  {
    Some(
      (*job)
        .p_encode_and_deinterleave_v
        .expect("non-null function pointer"),
    )
    .expect("non-null function pointer")(
      (*job).tiledp.offset(j as isize) as *mut libc::c_void,
      (*job).v.mem as *mut libc::c_void,
      (*job).rh,
      ((*job).v.cas == 0i32) as libc::c_int,
      (*job).w,
      8 as OPJ_UINT32,
    );
    j =
      (j as libc::c_uint).wrapping_add(8u32) as OPJ_UINT32
  }
  if j < (*job).max_j {
    Some(
      (*job)
        .p_encode_and_deinterleave_v
        .expect("non-null function pointer"),
    )
    .expect("non-null function pointer")(
      (*job).tiledp.offset(j as isize) as *mut libc::c_void,
      (*job).v.mem as *mut libc::c_void,
      (*job).rh,
      ((*job).v.cas == 0i32) as libc::c_int,
      (*job).w,
      (*job).max_j.wrapping_sub(j),
    );
  }
  opj_aligned_free((*job).v.mem as *mut libc::c_void);
  opj_free(job as *mut libc::c_void);
}
/* * Fetch up to cols <= NB_ELTS_V8 for each line, and put them in tmpOut */
/* that has a NB_ELTS_V8 interleave factor. */
unsafe fn opj_dwt_fetch_cols_vertical_pass(
  mut arrayIn: *const libc::c_void,
  mut tmpOut: *mut libc::c_void,
  mut height: OPJ_UINT32,
  mut stride_width: OPJ_UINT32,
  mut cols: OPJ_UINT32,
) {
  let mut array = arrayIn as *const OPJ_INT32;
  let mut tmp = tmpOut as *mut OPJ_INT32;
  if cols == 8u32 {
    let mut k: OPJ_UINT32 = 0;
    k = 0 as OPJ_UINT32;
    while k < height {
      memcpy(
        tmp.offset((8u32).wrapping_mul(k) as isize)
          as *mut libc::c_void,
        array.offset(k.wrapping_mul(stride_width) as isize) as *const libc::c_void,
        (8u64)
          .wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
      );
      k = k.wrapping_add(1)
    }
  } else {
    let mut k_0: OPJ_UINT32 = 0;
    k_0 = 0 as OPJ_UINT32;
    while k_0 < height {
      let mut c: OPJ_UINT32 = 0;
      c = 0 as OPJ_UINT32;
      while c < cols {
        *tmp.offset(
          (8u32)
            .wrapping_mul(k_0)
            .wrapping_add(c) as isize,
        ) = *array.offset(c.wrapping_add(k_0.wrapping_mul(stride_width)) as isize);
        c = c.wrapping_add(1)
      }
      while c < 8u32 {
        *tmp.offset(
          (8u32)
            .wrapping_mul(k_0)
            .wrapping_add(c) as isize,
        ) = 0i32;
        c = c.wrapping_add(1)
      }
      k_0 = k_0.wrapping_add(1)
    }
  };
}
/* Deinterleave result of forward transform, where cols <= NB_ELTS_V8 */
/* and src contains NB_ELTS_V8 consecutive values for up to NB_ELTS_V8 */
/* columns. */
#[inline]
unsafe fn opj_dwt_deinterleave_v_cols(
  mut src: *const OPJ_INT32,
  mut dst: *mut OPJ_INT32,
  mut dn: OPJ_INT32,
  mut sn: OPJ_INT32,
  mut stride_width: OPJ_UINT32,
  mut cas: OPJ_INT32,
  mut cols: OPJ_UINT32,
) {
  let mut k: OPJ_INT32 = 0; /* fallthru */
  let mut i = sn; /* fallthru */
  let mut l_dest = dst; /* fallthru */
  let mut l_src = src.offset((cas * 8i32) as isize); /* fallthru */
  let mut c: OPJ_UINT32 = 0; /* fallthru */
  k = 0i32; /* fallthru */
  while k < 2i32 {
    loop {
      let fresh34 = i;
      i = i - 1;
      if !(fresh34 != 0) {
        break;
      }
      if cols == 8u32 {
        memcpy(
          l_dest as *mut libc::c_void,
          l_src as *const libc::c_void,
          (8u64)
            .wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
        );
      } else {
        c = 0 as OPJ_UINT32;
        let mut current_block_16: u64;
        match cols {
          7 => {
            *l_dest.offset(c as isize) = *l_src.offset(c as isize);
            c = c.wrapping_add(1);
            current_block_16 = 3516197883607697062;
          }
          6 => {
            current_block_16 = 3516197883607697062;
          }
          5 => {
            current_block_16 = 3520964086993316036;
          }
          4 => {
            current_block_16 = 5597948028301642222;
          }
          3 => {
            current_block_16 = 6886095072222541387;
          }
          2 => {
            current_block_16 = 12368470615728410476;
          }
          _ => {
            current_block_16 = 8007710116773720393;
          }
        }
        match current_block_16 {
          3516197883607697062 => {
            *l_dest.offset(c as isize) = *l_src.offset(c as isize);
            c = c.wrapping_add(1);
            current_block_16 = 3520964086993316036;
          }
          _ => {}
        }
        match current_block_16 {
          3520964086993316036 => {
            *l_dest.offset(c as isize) = *l_src.offset(c as isize);
            c = c.wrapping_add(1);
            current_block_16 = 5597948028301642222;
          }
          _ => {}
        }
        match current_block_16 {
          5597948028301642222 => {
            *l_dest.offset(c as isize) = *l_src.offset(c as isize);
            c = c.wrapping_add(1);
            current_block_16 = 6886095072222541387;
          }
          _ => {}
        }
        match current_block_16 {
          6886095072222541387 => {
            *l_dest.offset(c as isize) = *l_src.offset(c as isize);
            c = c.wrapping_add(1);
            current_block_16 = 12368470615728410476;
          }
          _ => {}
        }
        match current_block_16 {
          12368470615728410476 => {
            *l_dest.offset(c as isize) = *l_src.offset(c as isize);
            c = c.wrapping_add(1)
          }
          _ => {}
        }
        *l_dest.offset(c as isize) = *l_src.offset(c as isize)
      }
      l_dest = l_dest.offset(stride_width as isize);
      l_src = l_src.offset((2i32 * 8i32) as isize)
    }
    l_dest = dst.offset((sn as OPJ_SIZE_T).wrapping_mul(stride_width as OPJ_SIZE_T) as isize);
    l_src = src.offset(((1i32 - cas) * 8i32) as isize);
    i = dn;
    k += 1
  }
}
/* Forward 5-3 transform, for the vertical pass, processing cols columns */
/* where cols <= NB_ELTS_V8 */
unsafe extern "C" fn opj_dwt_encode_and_deinterleave_v(
  mut arrayIn: *mut libc::c_void,
  mut tmpIn: *mut libc::c_void,
  mut height: OPJ_UINT32,
  mut even: OPJ_BOOL,
  mut stride_width: OPJ_UINT32,
  mut cols: OPJ_UINT32,
) {
  let mut array = arrayIn as *mut OPJ_INT32;
  let mut tmp = tmpIn as *mut OPJ_INT32;
  let sn = height.wrapping_add(
    (if even != 0 {
      1i32
    } else {
      0i32
    }) as libc::c_uint,
  ) >> 1i32;
  let dn = height.wrapping_sub(sn);
  opj_dwt_fetch_cols_vertical_pass(arrayIn, tmpIn, height, stride_width, cols);
  if even != 0 {
    let mut c: OPJ_UINT32 = 0;
    if height > 1u32 {
      let mut i: OPJ_UINT32 = 0;
      i = 0 as OPJ_UINT32;
      while i.wrapping_add(1u32) < sn {
        c = 0 as OPJ_UINT32;
        while c < 8u32 {
          let ref mut fresh35 = *tmp.offset(
            (1u32)
              .wrapping_add(i.wrapping_mul(2u32))
              .wrapping_mul(8u32)
              .wrapping_add(c) as isize,
          );
          *fresh35 -= *tmp.offset(
            i.wrapping_mul(2u32)
              .wrapping_mul(8u32)
              .wrapping_add(c) as isize,
          ) + *tmp.offset(
            i.wrapping_add(1u32)
              .wrapping_mul(2u32)
              .wrapping_mul(8u32)
              .wrapping_add(c) as isize,
          ) >> 1i32;
          c = c.wrapping_add(1)
        }
        i = i.wrapping_add(1)
      }
      if height.wrapping_rem(2u32) == 0u32 {
        c = 0 as OPJ_UINT32;
        while c < 8u32 {
          let ref mut fresh36 = *tmp.offset(
            (1u32)
              .wrapping_add(i.wrapping_mul(2u32))
              .wrapping_mul(8u32)
              .wrapping_add(c) as isize,
          );
          *fresh36 -= *tmp.offset(
            i.wrapping_mul(2u32)
              .wrapping_mul(8u32)
              .wrapping_add(c) as isize,
          );
          c = c.wrapping_add(1)
        }
      }
      c = 0 as OPJ_UINT32;
      while c < 8u32 {
        let ref mut fresh37 = *tmp.offset(
          ((0i32 * 2i32 * 8i32) as libc::c_uint).wrapping_add(c)
            as isize,
        );
        *fresh37 += *tmp.offset(
          (((1i32 + 0i32 * 2i32) * 8i32)
            as libc::c_uint)
            .wrapping_add(c) as isize,
        ) + *tmp.offset(
          (((1i32 + 0i32 * 2i32) * 8i32)
            as libc::c_uint)
            .wrapping_add(c) as isize,
        ) + 2i32
          >> 2i32;
        c = c.wrapping_add(1)
      }
      i = 1 as OPJ_UINT32;
      while i < dn {
        c = 0 as OPJ_UINT32;
        while c < 8u32 {
          let ref mut fresh38 = *tmp.offset(
            i.wrapping_mul(2u32)
              .wrapping_mul(8u32)
              .wrapping_add(c) as isize,
          );
          *fresh38 += *tmp.offset(
            (1u32)
              .wrapping_add(
                i.wrapping_sub(1u32)
                  .wrapping_mul(2u32),
              )
              .wrapping_mul(8u32)
              .wrapping_add(c) as isize,
          ) + *tmp.offset(
            (1u32)
              .wrapping_add(i.wrapping_mul(2u32))
              .wrapping_mul(8u32)
              .wrapping_add(c) as isize,
          ) + 2i32
            >> 2i32;
          c = c.wrapping_add(1)
        }
        i = i.wrapping_add(1)
      }
      if height.wrapping_rem(2u32) == 1u32 {
        c = 0 as OPJ_UINT32;
        while c < 8u32 {
          let ref mut fresh39 = *tmp.offset(
            i.wrapping_mul(2u32)
              .wrapping_mul(8u32)
              .wrapping_add(c) as isize,
          );
          *fresh39 += *tmp.offset(
            (1u32)
              .wrapping_add(
                i.wrapping_sub(1u32)
                  .wrapping_mul(2u32),
              )
              .wrapping_mul(8u32)
              .wrapping_add(c) as isize,
          ) + *tmp.offset(
            (1u32)
              .wrapping_add(
                i.wrapping_sub(1u32)
                  .wrapping_mul(2u32),
              )
              .wrapping_mul(8u32)
              .wrapping_add(c) as isize,
          ) + 2i32
            >> 2i32;
          c = c.wrapping_add(1)
        }
      }
    }
  } else {
    let mut c_0: OPJ_UINT32 = 0;
    if height == 1u32 {
      c_0 = 0 as OPJ_UINT32;
      while c_0 < 8u32 {
        let ref mut fresh40 = *tmp.offset(
          ((0i32 * 2i32 * 8i32) as libc::c_uint)
            .wrapping_add(c_0) as isize,
        );
        *fresh40 *= 2i32;
        c_0 = c_0.wrapping_add(1)
      }
    } else {
      let mut i_0: OPJ_UINT32 = 0;
      c_0 = 0 as OPJ_UINT32;
      while c_0 < 8u32 {
        let ref mut fresh41 = *tmp.offset(
          ((0i32 * 2i32 * 8i32) as libc::c_uint)
            .wrapping_add(c_0) as isize,
        );
        *fresh41 -= *tmp.offset(
          (((1i32 + 0i32 * 2i32) * 8i32)
            as libc::c_uint)
            .wrapping_add(c_0) as isize,
        );
        c_0 = c_0.wrapping_add(1)
      }
      i_0 = 1 as OPJ_UINT32;
      while i_0 < sn {
        c_0 = 0 as OPJ_UINT32;
        while c_0 < 8u32 {
          let ref mut fresh42 = *tmp.offset(
            i_0
              .wrapping_mul(2u32)
              .wrapping_mul(8u32)
              .wrapping_add(c_0) as isize,
          );
          *fresh42 -= *tmp.offset(
            (1u32)
              .wrapping_add(i_0.wrapping_mul(2u32))
              .wrapping_mul(8u32)
              .wrapping_add(c_0) as isize,
          ) + *tmp.offset(
            (1u32)
              .wrapping_add(
                i_0
                  .wrapping_sub(1u32)
                  .wrapping_mul(2u32),
              )
              .wrapping_mul(8u32)
              .wrapping_add(c_0) as isize,
          ) >> 1i32;
          c_0 = c_0.wrapping_add(1)
        }
        i_0 = i_0.wrapping_add(1)
      }
      if height.wrapping_rem(2u32) == 1u32 {
        c_0 = 0 as OPJ_UINT32;
        while c_0 < 8u32 {
          let ref mut fresh43 = *tmp.offset(
            i_0
              .wrapping_mul(2u32)
              .wrapping_mul(8u32)
              .wrapping_add(c_0) as isize,
          );
          *fresh43 -= *tmp.offset(
            (1u32)
              .wrapping_add(
                i_0
                  .wrapping_sub(1u32)
                  .wrapping_mul(2u32),
              )
              .wrapping_mul(8u32)
              .wrapping_add(c_0) as isize,
          );
          c_0 = c_0.wrapping_add(1)
        }
      }
      i_0 = 0 as OPJ_UINT32;
      while i_0.wrapping_add(1u32) < dn {
        c_0 = 0 as OPJ_UINT32;
        while c_0 < 8u32 {
          let ref mut fresh44 = *tmp.offset(
            (1u32)
              .wrapping_add(i_0.wrapping_mul(2u32))
              .wrapping_mul(8u32)
              .wrapping_add(c_0) as isize,
          );
          *fresh44 += *tmp.offset(
            i_0
              .wrapping_mul(2u32)
              .wrapping_mul(8u32)
              .wrapping_add(c_0) as isize,
          ) + *tmp.offset(
            i_0
              .wrapping_add(1u32)
              .wrapping_mul(2u32)
              .wrapping_mul(8u32)
              .wrapping_add(c_0) as isize,
          ) + 2i32
            >> 2i32;
          c_0 = c_0.wrapping_add(1)
        }
        i_0 = i_0.wrapping_add(1)
      }
      if height.wrapping_rem(2u32) == 0u32 {
        c_0 = 0 as OPJ_UINT32;
        while c_0 < 8u32 {
          let ref mut fresh45 = *tmp.offset(
            (1u32)
              .wrapping_add(i_0.wrapping_mul(2u32))
              .wrapping_mul(8u32)
              .wrapping_add(c_0) as isize,
          );
          *fresh45 += *tmp.offset(
            i_0
              .wrapping_mul(2u32)
              .wrapping_mul(8u32)
              .wrapping_add(c_0) as isize,
          ) + *tmp.offset(
            i_0
              .wrapping_mul(2u32)
              .wrapping_mul(8u32)
              .wrapping_add(c_0) as isize,
          ) + 2i32
            >> 2i32;
          c_0 = c_0.wrapping_add(1)
        }
      }
    }
  }
  if cols == 8u32 {
    opj_dwt_deinterleave_v_cols(
      tmp,
      array,
      dn as OPJ_INT32,
      sn as OPJ_INT32,
      stride_width,
      if even != 0 {
        0i32
      } else {
        1i32
      },
      8 as OPJ_UINT32,
    );
  } else {
    opj_dwt_deinterleave_v_cols(
      tmp,
      array,
      dn as OPJ_INT32,
      sn as OPJ_INT32,
      stride_width,
      if even != 0 {
        0i32
      } else {
        1i32
      },
      cols,
    );
  };
}
unsafe fn opj_v8dwt_encode_step1(
  mut fw: *mut OPJ_FLOAT32,
  mut end: OPJ_UINT32,
  cst: OPJ_FLOAT32,
) {
  let mut i: OPJ_UINT32 = 0;
  let mut c: OPJ_UINT32 = 0;
  i = 0 as OPJ_UINT32;
  while i < end {
    c = 0 as OPJ_UINT32;
    while c < 8u32 {
      let ref mut fresh46 = *fw.offset(
        i.wrapping_mul(2u32)
          .wrapping_mul(8u32)
          .wrapping_add(c) as isize,
      );
      *fresh46 *= cst;
      c = c.wrapping_add(1)
    }
    i = i.wrapping_add(1)
  }
}
unsafe fn opj_v8dwt_encode_step2(
  mut fl: *mut OPJ_FLOAT32,
  mut fw: *mut OPJ_FLOAT32,
  mut end: OPJ_UINT32,
  mut m: OPJ_UINT32,
  mut cst: OPJ_FLOAT32,
) {
  let mut i: OPJ_UINT32 = 0;
  let mut imax = opj_uint_min(end, m);
  let mut c: OPJ_INT32 = 0;
  if imax > 0u32 {
    c = 0i32;
    while c < 8i32 {
      let ref mut fresh47 = *fw.offset((-(1i32) * 8i32 + c) as isize);
      *fresh47 += (*fl.offset((0i32 * 8i32 + c) as isize)
        + *fw.offset((0i32 * 8i32 + c) as isize))
        * cst;
      c += 1
    }
    fw = fw.offset((2i32 * 8i32) as isize);
    i = 1 as OPJ_UINT32;
    while i < imax {
      c = 0i32;
      while c < 8i32 {
        let ref mut fresh48 = *fw.offset((-(1i32) * 8i32 + c) as isize);
        *fresh48 += (*fw.offset((-(2i32) * 8i32 + c) as isize)
          + *fw.offset((0i32 * 8i32 + c) as isize))
          * cst;
        c += 1
      }
      fw = fw.offset((2i32 * 8i32) as isize);
      i = i.wrapping_add(1)
    }
  }
  if m < end {
    assert!(m.wrapping_add(1u32) == end);
    c = 0i32;
    while c < 8i32 {
      let ref mut fresh49 = *fw.offset((-(1i32) * 8i32 + c) as isize);
      *fresh49 += 2 as libc::c_float
        * *fw.offset((-(2i32) * 8i32 + c) as isize)
        * cst;
      c += 1
    }
  };
}
/* Forward 9-7 transform, for the vertical pass, processing cols columns */
/* where cols <= NB_ELTS_V8 */
unsafe extern "C" fn opj_dwt_encode_and_deinterleave_v_real(
  mut arrayIn: *mut libc::c_void,
  mut tmpIn: *mut libc::c_void,
  mut height: OPJ_UINT32,
  mut even: OPJ_BOOL,
  mut stride_width: OPJ_UINT32,
  mut cols: OPJ_UINT32,
) {
  let mut array = arrayIn as *mut OPJ_FLOAT32;
  let mut tmp = tmpIn as *mut OPJ_FLOAT32;
  let sn = (height.wrapping_add(
    (if even != 0 {
      1i32
    } else {
      0i32
    }) as libc::c_uint,
  ) >> 1i32) as OPJ_INT32;
  let dn = height.wrapping_sub(sn as OPJ_UINT32) as OPJ_INT32;
  let mut a: OPJ_INT32 = 0;
  let mut b: OPJ_INT32 = 0;
  if height == 1u32 {
    return;
  }
  opj_dwt_fetch_cols_vertical_pass(arrayIn, tmpIn, height, stride_width, cols);
  if even != 0 {
    a = 0i32;
    b = 1i32
  } else {
    a = 1i32;
    b = 0i32
  }
  opj_v8dwt_encode_step2(
    tmp.offset((a * 8i32) as isize),
    tmp.offset(((b + 1i32) * 8i32) as isize),
    dn as OPJ_UINT32,
    opj_int_min(dn, sn - b) as OPJ_UINT32,
    opj_dwt_alpha,
  );
  opj_v8dwt_encode_step2(
    tmp.offset((b * 8i32) as isize),
    tmp.offset(((a + 1i32) * 8i32) as isize),
    sn as OPJ_UINT32,
    opj_int_min(sn, dn - a) as OPJ_UINT32,
    opj_dwt_beta,
  );
  opj_v8dwt_encode_step2(
    tmp.offset((a * 8i32) as isize),
    tmp.offset(((b + 1i32) * 8i32) as isize),
    dn as OPJ_UINT32,
    opj_int_min(dn, sn - b) as OPJ_UINT32,
    opj_dwt_gamma,
  );
  opj_v8dwt_encode_step2(
    tmp.offset((b * 8i32) as isize),
    tmp.offset(((a + 1i32) * 8i32) as isize),
    sn as OPJ_UINT32,
    opj_int_min(sn, dn - a) as OPJ_UINT32,
    opj_dwt_delta,
  );
  opj_v8dwt_encode_step1(
    tmp.offset((b * 8i32) as isize),
    dn as OPJ_UINT32,
    opj_K,
  );
  opj_v8dwt_encode_step1(
    tmp.offset((a * 8i32) as isize),
    sn as OPJ_UINT32,
    opj_invK,
  );
  if cols == 8u32 {
    opj_dwt_deinterleave_v_cols(
      tmp as *mut OPJ_INT32,
      array as *mut OPJ_INT32,
      dn,
      sn,
      stride_width,
      if even != 0 {
        0i32
      } else {
        1i32
      },
      8 as OPJ_UINT32,
    );
  } else {
    opj_dwt_deinterleave_v_cols(
      tmp as *mut OPJ_INT32,
      array as *mut OPJ_INT32,
      dn,
      sn,
      stride_width,
      if even != 0 {
        0i32
      } else {
        1i32
      },
      cols,
    );
  };
}
/* <summary>                            */
/* Forward 5-3 wavelet transform in 2-D. */
/* </summary>                           */
#[inline]
unsafe fn opj_dwt_encode_procedure(
  mut tp: *mut opj_thread_pool_t,
  mut tilec: *mut opj_tcd_tilecomp_t,
  mut p_encode_and_deinterleave_v: opj_encode_and_deinterleave_v_fnptr_type,
  mut p_encode_and_deinterleave_h_one_row: opj_encode_and_deinterleave_h_one_row_fnptr_type,
) -> OPJ_BOOL {
  let mut i: OPJ_INT32 = 0;
  let mut bj = 0 as *mut OPJ_INT32;
  let mut w: OPJ_UINT32 = 0;
  let mut l: OPJ_INT32 = 0;
  let mut l_data_size: OPJ_SIZE_T = 0;
  let mut l_cur_res = 0 as *mut opj_tcd_resolution_t;
  let mut l_last_res = 0 as *mut opj_tcd_resolution_t;
  let num_threads = opj_thread_pool_get_thread_count(tp);
  let mut tiledp = (*tilec).data;
  w = ((*tilec).x1 - (*tilec).x0) as OPJ_UINT32;
  l = (*tilec).numresolutions as OPJ_INT32 - 1i32;
  l_cur_res = (*tilec).resolutions.offset(l as isize);
  l_last_res = l_cur_res.offset(-1);
  l_data_size = opj_dwt_max_resolution((*tilec).resolutions, (*tilec).numresolutions) as OPJ_SIZE_T;
  /* overflow check */
  if l_data_size
    > (18446744073709551615u64).wrapping_div(
      (8u64)
        .wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
    )
  {
    /* FIXME event manager error callback */
    return 0i32;
  }
  l_data_size = (l_data_size as libc::c_ulong).wrapping_mul(
    (8u64)
      .wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
  ) as OPJ_SIZE_T as OPJ_SIZE_T;
  bj = opj_aligned_32_malloc(l_data_size) as *mut OPJ_INT32;
  /* l_data_size is equal to 0 when numresolutions == 1 but bj is not used */
  /* in that case, so do not error out */
  if l_data_size != 0u64 && bj.is_null() {
    return 0i32;
  } /* width of the resolution level computed   */
  i = l; /* height of the resolution level computed  */
  loop {
    let fresh50 = i; /* width of the resolution level once lower than computed one                                       */
    i = i - 1; /* height of the resolution level once lower than computed one                                      */
    if !(fresh50 != 0) {
      break; /* 0 = non inversion on horizontal filtering 1 = inversion between low-pass and high-pass filtering */
    } /* 0 = non inversion on vertical filtering 1 = inversion between low-pass and high-pass filtering   */
    let mut j: OPJ_UINT32 = 0;
    let mut rw: OPJ_UINT32 = 0;
    let mut rh: OPJ_UINT32 = 0;
    let mut rw1: OPJ_UINT32 = 0;
    let mut rh1: OPJ_UINT32 = 0;
    let mut cas_col: OPJ_INT32 = 0;
    let mut cas_row: OPJ_INT32 = 0;
    let mut dn: OPJ_INT32 = 0;
    let mut sn: OPJ_INT32 = 0;
    rw = ((*l_cur_res).x1 - (*l_cur_res).x0) as OPJ_UINT32;
    rh = ((*l_cur_res).y1 - (*l_cur_res).y0) as OPJ_UINT32;
    rw1 = ((*l_last_res).x1 - (*l_last_res).x0) as OPJ_UINT32;
    rh1 = ((*l_last_res).y1 - (*l_last_res).y0) as OPJ_UINT32;
    cas_row = (*l_cur_res).x0 & 1i32;
    cas_col = (*l_cur_res).y0 & 1i32;
    sn = rh1 as OPJ_INT32;
    dn = rh.wrapping_sub(rh1) as OPJ_INT32;
    /* Perform vertical pass */
    if num_threads <= 1i32 || rw < (2i32 * 8i32) as libc::c_uint
    {
      j = 0 as OPJ_UINT32;
      while j
        .wrapping_add(8u32)
        .wrapping_sub(1u32)
        < rw
      {
        p_encode_and_deinterleave_v.expect("non-null function pointer")(
          tiledp.offset(j as isize) as *mut libc::c_void,
          bj as *mut libc::c_void,
          rh,
          (cas_col == 0i32) as libc::c_int,
          w,
          8 as OPJ_UINT32,
        );
        j = (j as libc::c_uint).wrapping_add(8u32) as OPJ_UINT32
          as OPJ_UINT32
      }
      if j < rw {
        p_encode_and_deinterleave_v.expect("non-null function pointer")(
          tiledp.offset(j as isize) as *mut libc::c_void,
          bj as *mut libc::c_void,
          rh,
          (cas_col == 0i32) as libc::c_int,
          w,
          rw.wrapping_sub(j),
        );
      }
    } else {
      let mut num_jobs = num_threads as OPJ_UINT32;
      let mut step_j: OPJ_UINT32 = 0;
      if rw < num_jobs {
        num_jobs = rw
      }
      step_j = rw
        .wrapping_div(num_jobs)
        .wrapping_div(8u32)
        .wrapping_mul(8u32);
      j = 0 as OPJ_UINT32;
      while j < num_jobs {
        let mut job = 0 as *mut opj_dwt_encode_v_job_t;
        job = opj_malloc(::std::mem::size_of::<opj_dwt_encode_v_job_t>() as libc::c_ulong)
          as *mut opj_dwt_encode_v_job_t;
        if job.is_null() {
          opj_thread_pool_wait_completion(tp, 0i32);
          opj_aligned_free(bj as *mut libc::c_void);
          return 0i32;
        }
        (*job).v.mem = opj_aligned_32_malloc(l_data_size) as *mut OPJ_INT32;
        if (*job).v.mem.is_null() {
          opj_thread_pool_wait_completion(tp, 0i32);
          opj_free(job as *mut libc::c_void);
          opj_aligned_free(bj as *mut libc::c_void);
          return 0i32;
        }
        (*job).v.dn = dn;
        (*job).v.sn = sn;
        (*job).v.cas = cas_col;
        (*job).rh = rh;
        (*job).w = w;
        (*job).tiledp = tiledp;
        (*job).min_j = j.wrapping_mul(step_j);
        (*job).max_j = if j.wrapping_add(1u32) == num_jobs {
          rw
        } else {
          j.wrapping_add(1u32)
            .wrapping_mul(step_j)
        };
        (*job).p_encode_and_deinterleave_v = p_encode_and_deinterleave_v;
        opj_thread_pool_submit_job(
          tp,
          Some(
            opj_dwt_encode_v_func
              as unsafe extern "C" fn(_: *mut libc::c_void, _: *mut opj_tls_t) -> (),
          ),
          job as *mut libc::c_void,
        );
        j = j.wrapping_add(1)
      }
      opj_thread_pool_wait_completion(tp, 0i32);
    }
    sn = rw1 as OPJ_INT32;
    dn = rw.wrapping_sub(rw1) as OPJ_INT32;
    /* Perform horizontal pass */
    if num_threads <= 1i32 || rh <= 1u32 {
      j = 0 as OPJ_UINT32; /* this can overflow */
      while j < rh {
        let mut aj = tiledp.offset(j.wrapping_mul(w) as isize);
        Some(p_encode_and_deinterleave_h_one_row.expect("non-null function pointer"))
          .expect("non-null function pointer")(
          aj as *mut libc::c_void,
          bj as *mut libc::c_void,
          rw,
          if cas_row == 0i32 {
            1i32
          } else {
            0i32
          },
        );
        j = j.wrapping_add(1)
      }
    } else {
      let mut num_jobs_0 = num_threads as OPJ_UINT32;
      let mut step_j_0: OPJ_UINT32 = 0;
      if rh < num_jobs_0 {
        num_jobs_0 = rh
      }
      step_j_0 = rh.wrapping_div(num_jobs_0);
      j = 0 as OPJ_UINT32;
      while j < num_jobs_0 {
        let mut job_0 = 0 as *mut opj_dwt_encode_h_job_t;
        job_0 = opj_malloc(::std::mem::size_of::<opj_dwt_encode_h_job_t>() as libc::c_ulong)
          as *mut opj_dwt_encode_h_job_t;
        if job_0.is_null() {
          opj_thread_pool_wait_completion(tp, 0i32);
          opj_aligned_free(bj as *mut libc::c_void);
          return 0i32;
        }
        (*job_0).h.mem = opj_aligned_32_malloc(l_data_size) as *mut OPJ_INT32;
        if (*job_0).h.mem.is_null() {
          opj_thread_pool_wait_completion(tp, 0i32);
          opj_free(job_0 as *mut libc::c_void);
          opj_aligned_free(bj as *mut libc::c_void);
          return 0i32;
        }
        (*job_0).h.dn = dn;
        (*job_0).h.sn = sn;
        (*job_0).h.cas = cas_row;
        (*job_0).rw = rw;
        (*job_0).w = w;
        (*job_0).tiledp = tiledp;
        (*job_0).min_j = j.wrapping_mul(step_j_0);
        (*job_0).max_j = j.wrapping_add(1u32).wrapping_mul(step_j_0);
        if j == num_jobs_0.wrapping_sub(1u32) {
          /* this will take care of the overflow */
          (*job_0).max_j = rh
        }
        (*job_0).p_function = p_encode_and_deinterleave_h_one_row;
        opj_thread_pool_submit_job(
          tp,
          Some(
            opj_dwt_encode_h_func
              as unsafe extern "C" fn(_: *mut libc::c_void, _: *mut opj_tls_t) -> (),
          ),
          job_0 as *mut libc::c_void,
        );
        j = j.wrapping_add(1)
      }
      opj_thread_pool_wait_completion(tp, 0i32);
    }
    l_cur_res = l_last_res;
    l_last_res = l_last_res.offset(-1)
  }
  opj_aligned_free(bj as *mut libc::c_void);
  return 1i32;
}
/* Forward 5-3 wavelet transform in 2-D. */
/* </summary>                           */
#[no_mangle]
pub(crate) unsafe fn opj_dwt_encode(
  mut p_tcd: *mut opj_tcd_t,
  mut tilec: *mut opj_tcd_tilecomp_t,
) -> OPJ_BOOL {
  return opj_dwt_encode_procedure(
    (*p_tcd).thread_pool,
    tilec,
    Some(
      opj_dwt_encode_and_deinterleave_v
        as unsafe extern "C" fn(
          _: *mut libc::c_void,
          _: *mut libc::c_void,
          _: OPJ_UINT32,
          _: OPJ_BOOL,
          _: OPJ_UINT32,
          _: OPJ_UINT32,
        ) -> (),
    ),
    Some(
      opj_dwt_encode_and_deinterleave_h_one_row
        as unsafe extern "C" fn(
          _: *mut libc::c_void,
          _: *mut libc::c_void,
          _: OPJ_UINT32,
          _: OPJ_BOOL,
        ) -> (),
    ),
  );
}
/* <summary>                            */
/* Inverse 5-3 wavelet transform in 2-D. */
/* </summary>                           */
#[no_mangle]
pub(crate) unsafe fn opj_dwt_decode(
  mut p_tcd: *mut opj_tcd_t,
  mut tilec: *mut opj_tcd_tilecomp_t,
  mut numres: OPJ_UINT32,
) -> OPJ_BOOL {
  if (*p_tcd).whole_tile_decoding != 0 {
    return opj_dwt_decode_tile((*p_tcd).thread_pool, tilec, numres);
  } else {
    return opj_dwt_decode_partial_tile(tilec, numres);
  };
}

/* <summary>                */
/* Get norm of 5-3 wavelet. */
/* </summary>               */
#[no_mangle]
pub fn opj_dwt_getnorm(
  mut level: OPJ_UINT32,
  orient: OPJ_UINT32,
) -> OPJ_FLOAT64 {
  /* FIXME ! This is just a band-aid to avoid a buffer overflow */
  /* but the array should really be extended up to 33 resolution levels */
  /* See https://github.com/uclouvain/openjpeg/issues/493 */
  if orient == 0 && level >= 10 {
    level = 9
  } else if orient > 0 && level >= 9 {
    level = 8
  }
  return opj_dwt_norms[orient as usize][level as usize];
}

/* <summary>                             */
/* Forward 9-7 wavelet transform in 2-D. */
/* </summary>                            */
#[no_mangle]
pub(crate) unsafe fn opj_dwt_encode_real(
  mut p_tcd: *mut opj_tcd_t,
  mut tilec: *mut opj_tcd_tilecomp_t,
) -> OPJ_BOOL {
  return opj_dwt_encode_procedure(
    (*p_tcd).thread_pool,
    tilec,
    Some(
      opj_dwt_encode_and_deinterleave_v_real
        as unsafe extern "C" fn(
          _: *mut libc::c_void,
          _: *mut libc::c_void,
          _: OPJ_UINT32,
          _: OPJ_BOOL,
          _: OPJ_UINT32,
          _: OPJ_UINT32,
        ) -> (),
    ),
    Some(
      opj_dwt_encode_and_deinterleave_h_one_row_real
        as unsafe extern "C" fn(
          _: *mut libc::c_void,
          _: *mut libc::c_void,
          _: OPJ_UINT32,
          _: OPJ_BOOL,
        ) -> (),
    ),
  );
}

/* <summary>                */
/* Get norm of 9-7 wavelet. */
/* </summary>               */
#[no_mangle]
pub fn opj_dwt_getnorm_real(
  mut level: OPJ_UINT32,
  orient: OPJ_UINT32,
) -> OPJ_FLOAT64 {
  /* FIXME ! This is just a band-aid to avoid a buffer overflow */
  /* but the array should really be extended up to 33 resolution levels */
  /* See https://github.com/uclouvain/openjpeg/issues/493 */
  if orient == 0 && level >= 10 {
    level = 9
  } else if orient > 0 && level >= 9 {
    level = 8
  }
  return opj_dwt_norms_real[orient as usize][level as usize];
}

#[no_mangle]
pub(crate) unsafe fn opj_dwt_calc_explicit_stepsizes(
  mut tccp: *mut opj_tccp_t,
  mut prec: OPJ_UINT32,
) {
  let mut numbands: OPJ_UINT32 = 0;
  let mut bandno: OPJ_UINT32 = 0;
  numbands = (3u32)
    .wrapping_mul((*tccp).numresolutions)
    .wrapping_sub(2u32);
  bandno = 0 as OPJ_UINT32;
  while bandno < numbands {
    let mut stepsize: OPJ_FLOAT64 = 0.;
    let mut resno: OPJ_UINT32 = 0;
    let mut level: OPJ_UINT32 = 0;
    let mut orient: OPJ_UINT32 = 0;
    let mut gain: OPJ_UINT32 = 0;
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
    level = (*tccp)
      .numresolutions
      .wrapping_sub(1u32)
      .wrapping_sub(resno);
    gain = if (*tccp).qmfbid == 0u32 {
      0i32
    } else if orient == 0u32 {
      0i32
    } else if orient == 1u32
      || orient == 2u32
    {
      1i32
    } else {
      2i32
    } as OPJ_UINT32;
    if (*tccp).qntsty == 0u32 {
      stepsize = 1.0f64
    } else {
      let mut norm = opj_dwt_getnorm_real(level, orient);
      stepsize = ((1i32) << gain) as libc::c_double / norm
    }
    opj_dwt_encode_stepsize(
      floor(stepsize * 8192.0f64) as OPJ_INT32,
      prec.wrapping_add(gain) as OPJ_INT32,
      &mut *(*tccp).stepsizes.as_mut_ptr().offset(bandno as isize),
    );
    bandno = bandno.wrapping_add(1)
  }
}
/* <summary>                             */
/* Determine maximum computed resolution level for inverse wavelet transform */
/* </summary>                            */
unsafe fn opj_dwt_max_resolution(
  mut r: *mut opj_tcd_resolution_t,
  mut i: OPJ_UINT32,
) -> OPJ_UINT32 {
  let mut mr = 0 as OPJ_UINT32;
  let mut w: OPJ_UINT32 = 0;
  loop {
    i = i.wrapping_sub(1);
    if !(i != 0) {
      break;
    }
    r = r.offset(1);
    w = ((*r).x1 - (*r).x0) as OPJ_UINT32;
    if mr < w {
      mr = w
    }
    w = ((*r).y1 - (*r).y0) as OPJ_UINT32;
    if mr < w {
      mr = w
    }
  }
  return mr;
}
unsafe extern "C" fn opj_dwt_decode_h_func(
  mut user_data: *mut libc::c_void,
  mut _tls: *mut opj_tls_t,
) {
  let mut j: OPJ_UINT32 = 0;
  let mut job = 0 as *mut opj_dwt_decode_h_job_t;
  job = user_data as *mut opj_dwt_decode_h_job_t;
  j = (*job).min_j;
  while j < (*job).max_j {
    opj_idwt53_h(
      &mut (*job).h,
      &mut *(*job).tiledp.offset(j.wrapping_mul((*job).w) as isize),
    );
    j = j.wrapping_add(1)
  }
  opj_aligned_free((*job).h.mem as *mut libc::c_void);
  opj_free(job as *mut libc::c_void);
}
unsafe extern "C" fn opj_dwt_decode_v_func(
  mut user_data: *mut libc::c_void,
  mut _tls: *mut opj_tls_t,
) {
  let mut j: OPJ_UINT32 = 0;
  let mut job = 0 as *mut opj_dwt_decode_v_job_t;
  job = user_data as *mut opj_dwt_decode_v_job_t;
  j = (*job).min_j;
  while j.wrapping_add((2i32 * 4i32) as libc::c_uint) <= (*job).max_j {
    opj_idwt53_v(
      &mut (*job).v,
      &mut *(*job).tiledp.offset(j as isize),
      (*job).w as OPJ_SIZE_T,
      2i32 * 4i32,
    );
    j = (j as libc::c_uint).wrapping_add((2i32 * 4i32) as libc::c_uint)
      as OPJ_UINT32
  }
  if j < (*job).max_j {
    opj_idwt53_v(
      &mut (*job).v,
      &mut *(*job).tiledp.offset(j as isize),
      (*job).w as OPJ_SIZE_T,
      (*job).max_j.wrapping_sub(j) as OPJ_INT32,
    );
  }
  opj_aligned_free((*job).v.mem as *mut libc::c_void);
  opj_free(job as *mut libc::c_void);
}
/* *
Inverse wavelet transform in 2-D.
*/
/* <summary>                            */
/* Inverse wavelet transform in 2-D.    */
/* </summary>                           */
unsafe fn opj_dwt_decode_tile(
  mut tp: *mut opj_thread_pool_t,
  mut tilec: *mut opj_tcd_tilecomp_t,
  mut numres: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut h = opj_dwt_t {
    mem: 0 as *mut OPJ_INT32,
    dn: 0,
    sn: 0,
    cas: 0,
  }; /* width of the resolution level computed */
  let mut v = opj_dwt_t {
    mem: 0 as *mut OPJ_INT32,
    dn: 0,
    sn: 0,
    cas: 0,
  }; /* height of the resolution level computed */
  let mut tr = (*tilec).resolutions;
  let mut rw = ((*tr).x1 - (*tr).x0) as OPJ_UINT32;
  let mut rh = ((*tr).y1 - (*tr).y0) as OPJ_UINT32;
  let mut w = ((*(*tilec).resolutions.offset(
    (*tilec)
      .minimum_num_resolutions
      .wrapping_sub(1u32) as isize,
  ))
  .x1
    - (*(*tilec).resolutions.offset(
      (*tilec)
        .minimum_num_resolutions
        .wrapping_sub(1u32) as isize,
    ))
    .x0) as OPJ_UINT32;
  let mut h_mem_size: OPJ_SIZE_T = 0;
  let mut num_threads: libc::c_int = 0;
  if numres == 1u32 {
    return 1i32;
  }
  num_threads = opj_thread_pool_get_thread_count(tp);
  h_mem_size = opj_dwt_max_resolution(tr, numres) as OPJ_SIZE_T;
  /* overflow check */
  if h_mem_size
    > (18446744073709551615u64)
      .wrapping_div((2i32 * 4i32) as libc::c_ulong)
      .wrapping_div(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong)
  {
    /* FIXME event manager error callback */
    return 0i32;
  }
  /* We need PARALLEL_COLS_53 times the height of the array, */
  /* since for the vertical pass */
  /* we process PARALLEL_COLS_53 columns at a time */
  h_mem_size = (h_mem_size as libc::c_ulong).wrapping_mul(
    ((2i32 * 4i32) as libc::c_ulong)
      .wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
  ) as OPJ_SIZE_T as OPJ_SIZE_T;
  h.mem = opj_aligned_32_malloc(h_mem_size) as *mut OPJ_INT32;
  if h.mem.is_null() {
    /* FIXME event manager error callback */
    return 0i32;
  }
  v.mem = h.mem;
  loop {
    numres = numres.wrapping_sub(1);
    if !(numres != 0) {
      break;
    }
    let mut tiledp = (*tilec).data;
    let mut j: OPJ_UINT32 = 0;
    tr = tr.offset(1);
    h.sn = rw as OPJ_INT32;
    v.sn = rh as OPJ_INT32;
    rw = ((*tr).x1 - (*tr).x0) as OPJ_UINT32;
    rh = ((*tr).y1 - (*tr).y0) as OPJ_UINT32;
    h.dn = rw.wrapping_sub(h.sn as OPJ_UINT32) as OPJ_INT32;
    h.cas = (*tr).x0 % 2i32;
    if num_threads <= 1i32 || rh <= 1u32 {
      j = 0 as OPJ_UINT32;
      while j < rh {
        opj_idwt53_h(
          &mut h,
          &mut *tiledp.offset((j as OPJ_SIZE_T).wrapping_mul(w as libc::c_ulong) as isize),
        );
        j = j.wrapping_add(1)
      }
    } else {
      let mut num_jobs = num_threads as OPJ_UINT32;
      let mut step_j: OPJ_UINT32 = 0;
      if rh < num_jobs {
        num_jobs = rh
      }
      step_j = rh.wrapping_div(num_jobs);
      j = 0 as OPJ_UINT32;
      while j < num_jobs {
        let mut job = 0 as *mut opj_dwt_decode_h_job_t;
        job = opj_malloc(::std::mem::size_of::<opj_dwt_decode_h_job_t>() as libc::c_ulong)
          as *mut opj_dwt_decode_h_job_t;
        if job.is_null() {
          /* It would be nice to fallback to single thread case, but */
          /* unfortunately some jobs may be launched and have modified */
          /* tiledp, so it is not practical to recover from that error */
          /* FIXME event manager error callback */
          opj_thread_pool_wait_completion(tp, 0i32); /* this can overflow */
          opj_aligned_free(h.mem as *mut libc::c_void);
          return 0i32;
        }
        (*job).h = h;
        (*job).rw = rw;
        (*job).w = w;
        (*job).tiledp = tiledp;
        (*job).min_j = j.wrapping_mul(step_j);
        (*job).max_j = j.wrapping_add(1u32).wrapping_mul(step_j);
        if j == num_jobs.wrapping_sub(1u32) {
          /* this will take care of the overflow */
          (*job).max_j = rh
        }
        (*job).h.mem = opj_aligned_32_malloc(h_mem_size) as *mut OPJ_INT32;
        if (*job).h.mem.is_null() {
          /* FIXME event manager error callback */
          opj_thread_pool_wait_completion(tp, 0i32);
          opj_free(job as *mut libc::c_void);
          opj_aligned_free(h.mem as *mut libc::c_void);
          return 0i32;
        }
        opj_thread_pool_submit_job(
          tp,
          Some(
            opj_dwt_decode_h_func
              as unsafe extern "C" fn(_: *mut libc::c_void, _: *mut opj_tls_t) -> (),
          ),
          job as *mut libc::c_void,
        );
        j = j.wrapping_add(1)
      }
      opj_thread_pool_wait_completion(tp, 0i32);
    }
    v.dn = rh.wrapping_sub(v.sn as OPJ_UINT32) as OPJ_INT32;
    v.cas = (*tr).y0 % 2i32;
    if num_threads <= 1i32 || rw <= 1u32 {
      j = 0 as OPJ_UINT32;
      while j.wrapping_add((2i32 * 4i32) as libc::c_uint) <= rw {
        opj_idwt53_v(
          &mut v,
          &mut *tiledp.offset(j as isize),
          w as OPJ_SIZE_T,
          2i32 * 4i32,
        );
        j = (j as libc::c_uint).wrapping_add((2i32 * 4i32) as libc::c_uint)
          as OPJ_UINT32
      }
      if j < rw {
        opj_idwt53_v(
          &mut v,
          &mut *tiledp.offset(j as isize),
          w as OPJ_SIZE_T,
          rw.wrapping_sub(j) as OPJ_INT32,
        );
      }
    } else {
      let mut num_jobs_0 = num_threads as OPJ_UINT32;
      let mut step_j_0: OPJ_UINT32 = 0;
      if rw < num_jobs_0 {
        num_jobs_0 = rw
      }
      step_j_0 = rw.wrapping_div(num_jobs_0);
      j = 0 as OPJ_UINT32;
      while j < num_jobs_0 {
        let mut job_0 = 0 as *mut opj_dwt_decode_v_job_t;
        job_0 = opj_malloc(::std::mem::size_of::<opj_dwt_decode_v_job_t>() as libc::c_ulong)
          as *mut opj_dwt_decode_v_job_t;
        if job_0.is_null() {
          /* It would be nice to fallback to single thread case, but */
          /* unfortunately some jobs may be launched and have modified */
          /* tiledp, so it is not practical to recover from that error */
          /* FIXME event manager error callback */
          opj_thread_pool_wait_completion(tp, 0i32); /* this can overflow */
          opj_aligned_free(v.mem as *mut libc::c_void);
          return 0i32;
        }
        (*job_0).v = v;
        (*job_0).rh = rh;
        (*job_0).w = w;
        (*job_0).tiledp = tiledp;
        (*job_0).min_j = j.wrapping_mul(step_j_0);
        (*job_0).max_j = j.wrapping_add(1u32).wrapping_mul(step_j_0);
        if j == num_jobs_0.wrapping_sub(1u32) {
          /* this will take care of the overflow */
          (*job_0).max_j = rw
        }
        (*job_0).v.mem = opj_aligned_32_malloc(h_mem_size) as *mut OPJ_INT32;
        if (*job_0).v.mem.is_null() {
          /* FIXME event manager error callback */
          opj_thread_pool_wait_completion(tp, 0i32);
          opj_free(job_0 as *mut libc::c_void);
          opj_aligned_free(v.mem as *mut libc::c_void);
          return 0i32;
        }
        opj_thread_pool_submit_job(
          tp,
          Some(
            opj_dwt_decode_v_func
              as unsafe extern "C" fn(_: *mut libc::c_void, _: *mut opj_tls_t) -> (),
          ),
          job_0 as *mut libc::c_void,
        );
        j = j.wrapping_add(1)
      }
      opj_thread_pool_wait_completion(tp, 0i32);
    }
  }
  opj_aligned_free(h.mem as *mut libc::c_void);
  return 1i32;
}
unsafe fn opj_dwt_interleave_partial_h(
  mut dest: *mut OPJ_INT32,
  mut cas: OPJ_INT32,
  mut sa: *mut opj_sparse_array_int32_t,
  mut sa_line: OPJ_UINT32,
  mut sn: OPJ_UINT32,
  mut win_l_x0: OPJ_UINT32,
  mut win_l_x1: OPJ_UINT32,
  mut win_h_x0: OPJ_UINT32,
  mut win_h_x1: OPJ_UINT32,
) {
  let mut ret: OPJ_BOOL = 0;
  ret = opj_sparse_array_int32_read(
    sa,
    win_l_x0,
    sa_line,
    win_l_x1,
    sa_line.wrapping_add(1u32),
    dest
      .offset(cas as isize)
      .offset((2u32).wrapping_mul(win_l_x0) as isize),
    2 as OPJ_UINT32,
    0 as OPJ_UINT32,
    1i32,
  );
  assert!(ret != 0);
  ret = opj_sparse_array_int32_read(
    sa,
    sn.wrapping_add(win_h_x0),
    sa_line,
    sn.wrapping_add(win_h_x1),
    sa_line.wrapping_add(1u32),
    dest
      .offset(1)
      .offset(-(cas as isize))
      .offset((2u32).wrapping_mul(win_h_x0) as isize),
    2 as OPJ_UINT32,
    0 as OPJ_UINT32,
    1i32,
  );
  assert!(ret != 0);
}
unsafe fn opj_dwt_interleave_partial_v(
  mut dest: *mut OPJ_INT32,
  mut cas: OPJ_INT32,
  mut sa: *mut opj_sparse_array_int32_t,
  mut sa_col: OPJ_UINT32,
  mut nb_cols: OPJ_UINT32,
  mut sn: OPJ_UINT32,
  mut win_l_y0: OPJ_UINT32,
  mut win_l_y1: OPJ_UINT32,
  mut win_h_y0: OPJ_UINT32,
  mut win_h_y1: OPJ_UINT32,
) {
  let mut ret: OPJ_BOOL = 0;
  ret = opj_sparse_array_int32_read(
    sa,
    sa_col,
    win_l_y0,
    sa_col.wrapping_add(nb_cols),
    win_l_y1,
    dest.offset((cas * 4i32) as isize).offset(
      ((2i32 * 4i32) as libc::c_uint).wrapping_mul(win_l_y0) as isize,
    ),
    1 as OPJ_UINT32,
    (2i32 * 4i32) as OPJ_UINT32,
    1i32,
  );
  assert!(ret != 0);
  ret = opj_sparse_array_int32_read(
    sa,
    sa_col,
    sn.wrapping_add(win_h_y0),
    sa_col.wrapping_add(nb_cols),
    sn.wrapping_add(win_h_y1),
    dest
      .offset(((1i32 - cas) * 4i32) as isize)
      .offset(
        ((2i32 * 4i32) as libc::c_uint).wrapping_mul(win_h_y0) as isize,
      ),
    1 as OPJ_UINT32,
    (2i32 * 4i32) as OPJ_UINT32,
    1i32,
  );
  assert!(ret != 0);
}
unsafe fn opj_dwt_decode_partial_1(
  mut a: *mut OPJ_INT32,
  mut dn: OPJ_INT32,
  mut sn: OPJ_INT32,
  mut cas: OPJ_INT32,
  mut win_l_x0: OPJ_INT32,
  mut win_l_x1: OPJ_INT32,
  mut win_h_x0: OPJ_INT32,
  mut win_h_x1: OPJ_INT32,
) {
  let mut i: OPJ_INT32 = 0;
  if cas == 0 {
    if dn > 0i32 || sn > 1i32 {
      /* NEW :  CASE ONE ELEMENT */
      /* Naive version is :
      for (i = win_l_x0; i < i_max; i++) {
          OPJ_S(i) -= (OPJ_D_(i - 1) + OPJ_D_(i) + 2) >> 2;
      }
      for (i = win_h_x0; i < win_h_x1; i++) {
          OPJ_D(i) += (OPJ_S_(i) + OPJ_S_(i + 1)) >> 1;
      }
      but the compiler doesn't manage to unroll it to avoid bound
      checking in OPJ_S_ and OPJ_D_ macros
      */
      i = win_l_x0;
      if i < win_l_x1 {
        let mut i_max: OPJ_INT32 = 0;
        /* Left-most case */
        let ref mut fresh51 = *a.offset((i * 2i32) as isize);
        *fresh51 -= (if (i - 1i32) < 0i32 {
          *a.offset((1i32 + 0i32 * 2i32) as isize)
        } else {
          (if i - 1i32 >= dn {
            *a.offset((1i32 + (dn - 1i32) * 2i32) as isize)
          } else {
            *a.offset((1i32 + (i - 1i32) * 2i32) as isize)
          })
        }) + (if i < 0i32 {
          *a.offset((1i32 + 0i32 * 2i32) as isize)
        } else {
          (if i >= dn {
            *a.offset((1i32 + (dn - 1i32) * 2i32) as isize)
          } else {
            *a.offset((1i32 + i * 2i32) as isize)
          })
        }) + 2i32
          >> 2i32;
        i += 1;
        i_max = win_l_x1;
        if i_max > dn {
          i_max = dn
        }
        while i < i_max {
          /* No bound checking */
          let ref mut fresh52 = *a.offset((i * 2i32) as isize);
          *fresh52 -= *a
            .offset((1i32 + (i - 1i32) * 2i32) as isize)
            + *a.offset((1i32 + i * 2i32) as isize)
            + 2i32
            >> 2i32;
          i += 1
        }
        while i < win_l_x1 {
          /* Right-most case */
          let ref mut fresh53 = *a.offset((i * 2i32) as isize);
          *fresh53 -= (if (i - 1i32) < 0i32 {
            *a.offset((1i32 + 0i32 * 2i32) as isize)
          } else {
            (if i - 1i32 >= dn {
              *a.offset((1i32 + (dn - 1i32) * 2i32) as isize)
            } else {
              *a.offset((1i32 + (i - 1i32) * 2i32) as isize)
            })
          }) + (if i < 0i32 {
            *a.offset((1i32 + 0i32 * 2i32) as isize)
          } else {
            (if i >= dn {
              *a.offset((1i32 + (dn - 1i32) * 2i32) as isize)
            } else {
              *a.offset((1i32 + i * 2i32) as isize)
            })
          }) + 2i32
            >> 2i32;
          i += 1
        }
      }
      i = win_h_x0;
      if i < win_h_x1 {
        let mut i_max_0 = win_h_x1;
        if i_max_0 >= sn {
          i_max_0 = sn - 1i32
        }
        while i < i_max_0 {
          /* No bound checking */
          let ref mut fresh54 = *a.offset((1i32 + i * 2i32) as isize);
          *fresh54 += *a.offset((i * 2i32) as isize)
            + *a.offset(((i + 1i32) * 2i32) as isize)
            >> 1i32;
          i += 1
        }
        while i < win_h_x1 {
          /* Right-most case */
          let ref mut fresh55 = *a.offset((1i32 + i * 2i32) as isize);
          *fresh55 += (if i < 0i32 {
            *a.offset((0i32 * 2i32) as isize)
          } else {
            (if i >= sn {
              *a.offset(((sn - 1i32) * 2i32) as isize)
            } else {
              *a.offset((i * 2i32) as isize)
            })
          }) + (if (i + 1i32) < 0i32 {
            *a.offset((0i32 * 2i32) as isize)
          } else {
            (if i + 1i32 >= sn {
              *a.offset(((sn - 1i32) * 2i32) as isize)
            } else {
              *a.offset(((i + 1i32) * 2i32) as isize)
            })
          }) >> 1i32;
          i += 1
        }
      }
    }
  } else if sn == 0 && dn == 1i32 {
    /* NEW :  CASE ONE ELEMENT */
    let ref mut fresh56 = *a.offset((0i32 * 2i32) as isize);
    *fresh56 /= 2i32
  } else {
    i = win_l_x0;
    while i < win_l_x1 {
      *a.offset((1i32 + i * 2i32) as isize) = opj_int_sub_no_overflow(
        *a.offset((1i32 + i * 2i32) as isize),
        opj_int_add_no_overflow(
          opj_int_add_no_overflow(
            if i < 0i32 {
              *a.offset((0i32 * 2i32) as isize)
            } else {
              (if i >= dn {
                *a.offset(((dn - 1i32) * 2i32) as isize)
              } else {
                *a.offset((i * 2i32) as isize)
              })
            },
            if (i + 1i32) < 0i32 {
              *a.offset((0i32 * 2i32) as isize)
            } else {
              (if i + 1i32 >= dn {
                *a.offset(((dn - 1i32) * 2i32) as isize)
              } else {
                *a.offset(((i + 1i32) * 2i32) as isize)
              })
            },
          ),
          2i32,
        ) >> 2i32,
      );
      i += 1
    }
    i = win_h_x0;
    while i < win_h_x1 {
      *a.offset((i * 2i32) as isize) = opj_int_add_no_overflow(
        *a.offset((i * 2i32) as isize),
        opj_int_add_no_overflow(
          if i < 0i32 {
            *a.offset((1i32 + 0i32 * 2i32) as isize)
          } else {
            (if i >= sn {
              *a.offset((1i32 + (sn - 1i32) * 2i32) as isize)
            } else {
              *a.offset((1i32 + i * 2i32) as isize)
            })
          },
          if (i - 1i32) < 0i32 {
            *a.offset((1i32 + 0i32 * 2i32) as isize)
          } else {
            (if i - 1i32 >= sn {
              *a.offset((1i32 + (sn - 1i32) * 2i32) as isize)
            } else {
              *a.offset((1i32 + (i - 1i32) * 2i32) as isize)
            })
          },
        ) >> 1i32,
      );
      i += 1
    }
  };
}
unsafe fn opj_dwt_decode_partial_1_parallel(
  mut a: *mut OPJ_INT32,
  mut _nb_cols: OPJ_UINT32,
  mut dn: OPJ_INT32,
  mut sn: OPJ_INT32,
  mut cas: OPJ_INT32,
  mut win_l_x0: OPJ_INT32,
  mut win_l_x1: OPJ_INT32,
  mut win_h_x0: OPJ_INT32,
  mut win_h_x1: OPJ_INT32,
) {
  let mut i: OPJ_INT32 = 0;
  let mut off: OPJ_UINT32 = 0;
  if cas == 0 {
    if dn > 0i32 || sn > 1i32 {
      /* NEW :  CASE ONE ELEMENT */
      /* Naive version is :
      for (i = win_l_x0; i < i_max; i++) {
          OPJ_S(i) -= (OPJ_D_(i - 1) + OPJ_D_(i) + 2) >> 2;
      }
      for (i = win_h_x0; i < win_h_x1; i++) {
          OPJ_D(i) += (OPJ_S_(i) + OPJ_S_(i + 1)) >> 1;
      }
      but the compiler doesn't manage to unroll it to avoid bound
      checking in OPJ_S_ and OPJ_D_ macros
      */
      i = win_l_x0;
      if i < win_l_x1 {
        let mut i_max: OPJ_INT32 = 0;
        /* Left-most case */
        off = 0 as OPJ_UINT32;
        while off < 4u32 {
          let ref mut fresh57 = *a.offset(
            (i as OPJ_UINT32)
              .wrapping_mul(2u32)
              .wrapping_mul(4u32)
              .wrapping_add(off) as isize,
          );
          *fresh57 -= (if (i - 1i32) < 0i32 {
            *a.offset(
              (1u32)
                .wrapping_add(
                  (0 as OPJ_UINT32).wrapping_mul(2u32),
                )
                .wrapping_mul(4u32)
                .wrapping_add(off) as isize,
            )
          } else {
            (if i - 1i32 >= dn {
              *a.offset(
                (1u32)
                  .wrapping_add(
                    ((dn - 1i32) as OPJ_UINT32)
                      .wrapping_mul(2u32),
                  )
                  .wrapping_mul(4u32)
                  .wrapping_add(off) as isize,
              )
            } else {
              *a.offset(
                (1u32)
                  .wrapping_add(
                    ((i - 1i32) as OPJ_UINT32)
                      .wrapping_mul(2u32),
                  )
                  .wrapping_mul(4u32)
                  .wrapping_add(off) as isize,
              )
            })
          }) + (if i < 0i32 {
            *a.offset(
              (1u32)
                .wrapping_add(
                  (0 as OPJ_UINT32).wrapping_mul(2u32),
                )
                .wrapping_mul(4u32)
                .wrapping_add(off) as isize,
            )
          } else {
            (if i >= dn {
              *a.offset(
                (1u32)
                  .wrapping_add(
                    ((dn - 1i32) as OPJ_UINT32)
                      .wrapping_mul(2u32),
                  )
                  .wrapping_mul(4u32)
                  .wrapping_add(off) as isize,
              )
            } else {
              *a.offset(
                (1u32)
                  .wrapping_add((i as OPJ_UINT32).wrapping_mul(2u32))
                  .wrapping_mul(4u32)
                  .wrapping_add(off) as isize,
              )
            })
          }) + 2i32
            >> 2i32;
          off = off.wrapping_add(1)
        }
        i += 1;
        i_max = win_l_x1;
        if i_max > dn {
          i_max = dn
        }
        while i < i_max {
          /* No bound checking */
          off = 0 as OPJ_UINT32;
          while off < 4u32 {
            let ref mut fresh58 = *a.offset(
              (i as OPJ_UINT32)
                .wrapping_mul(2u32)
                .wrapping_mul(4u32)
                .wrapping_add(off) as isize,
            );
            *fresh58 -= *a.offset(
              (1u32)
                .wrapping_add(
                  ((i - 1i32) as OPJ_UINT32)
                    .wrapping_mul(2u32),
                )
                .wrapping_mul(4u32)
                .wrapping_add(off) as isize,
            ) + *a.offset(
              (1u32)
                .wrapping_add((i as OPJ_UINT32).wrapping_mul(2u32))
                .wrapping_mul(4u32)
                .wrapping_add(off) as isize,
            ) + 2i32
              >> 2i32;
            off = off.wrapping_add(1)
          }
          i += 1
        }
        while i < win_l_x1 {
          /* Right-most case */
          off = 0 as OPJ_UINT32;
          while off < 4u32 {
            let ref mut fresh59 = *a.offset(
              (i as OPJ_UINT32)
                .wrapping_mul(2u32)
                .wrapping_mul(4u32)
                .wrapping_add(off) as isize,
            );
            *fresh59 -= (if (i - 1i32) < 0i32 {
              *a.offset(
                (1u32)
                  .wrapping_add(
                    (0 as OPJ_UINT32).wrapping_mul(2u32),
                  )
                  .wrapping_mul(4u32)
                  .wrapping_add(off) as isize,
              )
            } else {
              (if i - 1i32 >= dn {
                *a.offset(
                  (1u32)
                    .wrapping_add(
                      ((dn - 1i32) as OPJ_UINT32)
                        .wrapping_mul(2u32),
                    )
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              } else {
                *a.offset(
                  (1u32)
                    .wrapping_add(
                      ((i - 1i32) as OPJ_UINT32)
                        .wrapping_mul(2u32),
                    )
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              })
            }) + (if i < 0i32 {
              *a.offset(
                (1u32)
                  .wrapping_add(
                    (0 as OPJ_UINT32).wrapping_mul(2u32),
                  )
                  .wrapping_mul(4u32)
                  .wrapping_add(off) as isize,
              )
            } else {
              (if i >= dn {
                *a.offset(
                  (1u32)
                    .wrapping_add(
                      ((dn - 1i32) as OPJ_UINT32)
                        .wrapping_mul(2u32),
                    )
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              } else {
                *a.offset(
                  (1u32)
                    .wrapping_add((i as OPJ_UINT32).wrapping_mul(2u32))
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              })
            }) + 2i32
              >> 2i32;
            off = off.wrapping_add(1)
          }
          i += 1
        }
      }
      i = win_h_x0;
      if i < win_h_x1 {
        let mut i_max_0 = win_h_x1;
        if i_max_0 >= sn {
          i_max_0 = sn - 1i32
        }
        while i < i_max_0 {
          /* No bound checking */
          off = 0 as OPJ_UINT32;
          while off < 4u32 {
            let ref mut fresh60 = *a.offset(
              (1u32)
                .wrapping_add((i as OPJ_UINT32).wrapping_mul(2u32))
                .wrapping_mul(4u32)
                .wrapping_add(off) as isize,
            );
            *fresh60 += *a.offset(
              (i as OPJ_UINT32)
                .wrapping_mul(2u32)
                .wrapping_mul(4u32)
                .wrapping_add(off) as isize,
            ) + *a.offset(
              ((i + 1i32) as OPJ_UINT32)
                .wrapping_mul(2u32)
                .wrapping_mul(4u32)
                .wrapping_add(off) as isize,
            ) >> 1i32;
            off = off.wrapping_add(1)
          }
          i += 1
        }
        while i < win_h_x1 {
          /* Right-most case */
          off = 0 as OPJ_UINT32;
          while off < 4u32 {
            let ref mut fresh61 = *a.offset(
              (1u32)
                .wrapping_add((i as OPJ_UINT32).wrapping_mul(2u32))
                .wrapping_mul(4u32)
                .wrapping_add(off) as isize,
            );
            *fresh61 += (if i < 0i32 {
              *a.offset(
                (0 as OPJ_UINT32)
                  .wrapping_mul(2u32)
                  .wrapping_mul(4u32)
                  .wrapping_add(off) as isize,
              )
            } else {
              (if i >= sn {
                *a.offset(
                  ((sn - 1i32) as OPJ_UINT32)
                    .wrapping_mul(2u32)
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              } else {
                *a.offset(
                  (i as OPJ_UINT32)
                    .wrapping_mul(2u32)
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              })
            }) + (if (i + 1i32) < 0i32 {
              *a.offset(
                (0 as OPJ_UINT32)
                  .wrapping_mul(2u32)
                  .wrapping_mul(4u32)
                  .wrapping_add(off) as isize,
              )
            } else {
              (if i + 1i32 >= sn {
                *a.offset(
                  ((sn - 1i32) as OPJ_UINT32)
                    .wrapping_mul(2u32)
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              } else {
                *a.offset(
                  ((i + 1i32) as OPJ_UINT32)
                    .wrapping_mul(2u32)
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              })
            }) >> 1i32;
            off = off.wrapping_add(1)
          }
          i += 1
        }
      }
    }
  } else if sn == 0 && dn == 1i32 {
    /* NEW :  CASE ONE ELEMENT */
    off = 0 as OPJ_UINT32;
    while off < 4u32 {
      let ref mut fresh62 = *a.offset(
        (0 as OPJ_UINT32)
          .wrapping_mul(2u32)
          .wrapping_mul(4u32)
          .wrapping_add(off) as isize,
      );
      *fresh62 /= 2i32;
      off = off.wrapping_add(1)
    }
  } else {
    i = win_l_x0;
    while i < win_l_x1 {
      off = 0 as OPJ_UINT32;
      while off < 4u32 {
        *a.offset(
          (1u32)
            .wrapping_add((i as OPJ_UINT32).wrapping_mul(2u32))
            .wrapping_mul(4u32)
            .wrapping_add(off) as isize,
        ) = opj_int_sub_no_overflow(
          *a.offset(
            (1u32)
              .wrapping_add((i as OPJ_UINT32).wrapping_mul(2u32))
              .wrapping_mul(4u32)
              .wrapping_add(off) as isize,
          ),
          opj_int_add_no_overflow(
            opj_int_add_no_overflow(
              if i < 0i32 {
                *a.offset(
                  (0 as OPJ_UINT32)
                    .wrapping_mul(2u32)
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              } else {
                (if i >= dn {
                  *a.offset(
                    ((dn - 1i32) as OPJ_UINT32)
                      .wrapping_mul(2u32)
                      .wrapping_mul(4u32)
                      .wrapping_add(off) as isize,
                  )
                } else {
                  *a.offset(
                    (i as OPJ_UINT32)
                      .wrapping_mul(2u32)
                      .wrapping_mul(4u32)
                      .wrapping_add(off) as isize,
                  )
                })
              },
              if (i + 1i32) < 0i32 {
                *a.offset(
                  (0 as OPJ_UINT32)
                    .wrapping_mul(2u32)
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              } else {
                (if i + 1i32 >= dn {
                  *a.offset(
                    ((dn - 1i32) as OPJ_UINT32)
                      .wrapping_mul(2u32)
                      .wrapping_mul(4u32)
                      .wrapping_add(off) as isize,
                  )
                } else {
                  *a.offset(
                    ((i + 1i32) as OPJ_UINT32)
                      .wrapping_mul(2u32)
                      .wrapping_mul(4u32)
                      .wrapping_add(off) as isize,
                  )
                })
              },
            ),
            2i32,
          ) >> 2i32,
        );
        off = off.wrapping_add(1)
      }
      i += 1
    }
    i = win_h_x0;
    while i < win_h_x1 {
      off = 0 as OPJ_UINT32;
      while off < 4u32 {
        *a.offset(
          (i as OPJ_UINT32)
            .wrapping_mul(2u32)
            .wrapping_mul(4u32)
            .wrapping_add(off) as isize,
        ) = opj_int_add_no_overflow(
          *a.offset(
            (i as OPJ_UINT32)
              .wrapping_mul(2u32)
              .wrapping_mul(4u32)
              .wrapping_add(off) as isize,
          ),
          opj_int_add_no_overflow(
            if i < 0i32 {
              *a.offset(
                (1u32)
                  .wrapping_add(
                    (0 as OPJ_UINT32).wrapping_mul(2u32),
                  )
                  .wrapping_mul(4u32)
                  .wrapping_add(off) as isize,
              )
            } else {
              (if i >= sn {
                *a.offset(
                  (1u32)
                    .wrapping_add(
                      ((sn - 1i32) as OPJ_UINT32)
                        .wrapping_mul(2u32),
                    )
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              } else {
                *a.offset(
                  (1u32)
                    .wrapping_add((i as OPJ_UINT32).wrapping_mul(2u32))
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              })
            },
            if (i - 1i32) < 0i32 {
              *a.offset(
                (1u32)
                  .wrapping_add(
                    (0 as OPJ_UINT32).wrapping_mul(2u32),
                  )
                  .wrapping_mul(4u32)
                  .wrapping_add(off) as isize,
              )
            } else {
              (if i - 1i32 >= sn {
                *a.offset(
                  (1u32)
                    .wrapping_add(
                      ((sn - 1i32) as OPJ_UINT32)
                        .wrapping_mul(2u32),
                    )
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              } else {
                *a.offset(
                  (1u32)
                    .wrapping_add(
                      ((i - 1i32) as OPJ_UINT32)
                        .wrapping_mul(2u32),
                    )
                    .wrapping_mul(4u32)
                    .wrapping_add(off) as isize,
                )
              })
            },
          ) >> 1i32,
        );
        off = off.wrapping_add(1)
      }
      i += 1
    }
  };
}
unsafe fn opj_dwt_get_band_coordinates(
  mut tilec: *mut opj_tcd_tilecomp_t,
  mut resno: OPJ_UINT32,
  mut bandno: OPJ_UINT32,
  mut tcx0: OPJ_UINT32,
  mut tcy0: OPJ_UINT32,
  mut tcx1: OPJ_UINT32,
  mut tcy1: OPJ_UINT32,
  mut tbx0: *mut OPJ_UINT32,
  mut tby0: *mut OPJ_UINT32,
  mut tbx1: *mut OPJ_UINT32,
  mut tby1: *mut OPJ_UINT32,
) {
  /* Compute number of decomposition for this band. See table F-1 */
  let mut nb = if resno == 0u32 {
    (*tilec)
      .numresolutions
      .wrapping_sub(1u32)
  } else {
    (*tilec).numresolutions.wrapping_sub(resno)
  };
  /* Map above tile-based coordinates to sub-band-based coordinates per */
  /* equation B-15 of the standard */
  let mut x0b = bandno & 1u32;
  let mut y0b = bandno >> 1i32;
  if !tbx0.is_null() {
    *tbx0 = if nb == 0u32 {
      tcx0
    } else if tcx0
      <= ((1u32) << nb.wrapping_sub(1u32))
        .wrapping_mul(x0b)
    {
      0u32
    } else {
      opj_uint_ceildivpow2(
        tcx0.wrapping_sub(
          ((1u32) << nb.wrapping_sub(1u32))
            .wrapping_mul(x0b),
        ),
        nb,
      )
    }
  }
  if !tby0.is_null() {
    *tby0 = if nb == 0u32 {
      tcy0
    } else if tcy0
      <= ((1u32) << nb.wrapping_sub(1u32))
        .wrapping_mul(y0b)
    {
      0u32
    } else {
      opj_uint_ceildivpow2(
        tcy0.wrapping_sub(
          ((1u32) << nb.wrapping_sub(1u32))
            .wrapping_mul(y0b),
        ),
        nb,
      )
    }
  }
  if !tbx1.is_null() {
    *tbx1 = if nb == 0u32 {
      tcx1
    } else if tcx1
      <= ((1u32) << nb.wrapping_sub(1u32))
        .wrapping_mul(x0b)
    {
      0u32
    } else {
      opj_uint_ceildivpow2(
        tcx1.wrapping_sub(
          ((1u32) << nb.wrapping_sub(1u32))
            .wrapping_mul(x0b),
        ),
        nb,
      )
    }
  }
  if !tby1.is_null() {
    *tby1 = if nb == 0u32 {
      tcy1
    } else if tcy1
      <= ((1u32) << nb.wrapping_sub(1u32))
        .wrapping_mul(y0b)
    {
      0u32
    } else {
      opj_uint_ceildivpow2(
        tcy1.wrapping_sub(
          ((1u32) << nb.wrapping_sub(1u32))
            .wrapping_mul(y0b),
        ),
        nb,
      )
    }
  };
}
unsafe fn opj_dwt_segment_grow(
  mut filter_width: OPJ_UINT32,
  mut max_size: OPJ_UINT32,
  mut start: *mut OPJ_UINT32,
  mut end: *mut OPJ_UINT32,
) {
  *start = opj_uint_subs(*start, filter_width);
  *end = opj_uint_adds(*end, filter_width);
  *end = opj_uint_min(*end, max_size);
}
unsafe fn opj_dwt_init_sparse_array(
  mut tilec: *mut opj_tcd_tilecomp_t,
  mut numres: OPJ_UINT32,
) -> *mut opj_sparse_array_int32_t {
  let mut tr_max: *mut opj_tcd_resolution_t = &mut *(*tilec)
    .resolutions
    .offset(numres.wrapping_sub(1u32) as isize)
    as *mut opj_tcd_resolution_t;
  let mut w = ((*tr_max).x1 - (*tr_max).x0) as OPJ_UINT32;
  let mut h = ((*tr_max).y1 - (*tr_max).y0) as OPJ_UINT32;
  let mut resno: OPJ_UINT32 = 0;
  let mut bandno: OPJ_UINT32 = 0;
  let mut precno: OPJ_UINT32 = 0;
  let mut cblkno: OPJ_UINT32 = 0;
  let mut sa = opj_sparse_array_int32_create(
    w,
    h,
    opj_uint_min(w, 64 as OPJ_UINT32),
    opj_uint_min(h, 64 as OPJ_UINT32),
  );
  if sa.is_null() {
    return 0 as *mut opj_sparse_array_int32_t;
  }
  resno = 0 as OPJ_UINT32;
  while resno < numres {
    let mut res: *mut opj_tcd_resolution_t =
      &mut *(*tilec).resolutions.offset(resno as isize) as *mut opj_tcd_resolution_t;
    bandno = 0 as OPJ_UINT32;
    while bandno < (*res).numbands {
      let mut band: *mut opj_tcd_band_t =
        &mut *(*res).bands.as_mut_ptr().offset(bandno as isize) as *mut opj_tcd_band_t;
      precno = 0 as OPJ_UINT32;
      while precno < (*res).pw.wrapping_mul((*res).ph) {
        let mut precinct: *mut opj_tcd_precinct_t =
          &mut *(*band).precincts.offset(precno as isize) as *mut opj_tcd_precinct_t;
        cblkno = 0 as OPJ_UINT32;
        while cblkno < (*precinct).cw.wrapping_mul((*precinct).ch) {
          let mut cblk: *mut opj_tcd_cblk_dec_t =
            &mut *(*precinct).cblks.dec.offset(cblkno as isize) as *mut opj_tcd_cblk_dec_t;
          if !(*cblk).decoded_data.is_null() {
            let mut x = ((*cblk).x0 - (*band).x0) as OPJ_UINT32;
            let mut y = ((*cblk).y0 - (*band).y0) as OPJ_UINT32;
            let mut cblk_w = ((*cblk).x1 - (*cblk).x0) as OPJ_UINT32;
            let mut cblk_h = ((*cblk).y1 - (*cblk).y0) as OPJ_UINT32;
            if (*band).bandno & 1u32 != 0 {
              let mut pres: *mut opj_tcd_resolution_t = &mut *(*tilec)
                .resolutions
                .offset(resno.wrapping_sub(1u32) as isize)
                as *mut opj_tcd_resolution_t;
              x = x.wrapping_add(((*pres).x1 - (*pres).x0) as OPJ_UINT32)
                as OPJ_UINT32
            }
            if (*band).bandno & 2u32 != 0 {
              let mut pres_0: *mut opj_tcd_resolution_t = &mut *(*tilec)
                .resolutions
                .offset(resno.wrapping_sub(1u32) as isize)
                as *mut opj_tcd_resolution_t;
              y = (y as libc::c_uint).wrapping_add(((*pres_0).y1 - (*pres_0).y0) as OPJ_UINT32)
                as OPJ_UINT32
            }
            if opj_sparse_array_int32_write(
              sa,
              x,
              y,
              x.wrapping_add(cblk_w),
              y.wrapping_add(cblk_h),
              (*cblk).decoded_data,
              1 as OPJ_UINT32,
              cblk_w,
              1i32,
            ) == 0
            {
              opj_sparse_array_int32_free(sa);
              return 0 as *mut opj_sparse_array_int32_t;
            }
          }
          cblkno = cblkno.wrapping_add(1)
        }
        precno = precno.wrapping_add(1)
      }
      bandno = bandno.wrapping_add(1)
    }
    resno = resno.wrapping_add(1)
  }
  return sa;
}
unsafe fn opj_dwt_decode_partial_tile(
  mut tilec: *mut opj_tcd_tilecomp_t,
  mut numres: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut sa = 0 as *mut opj_sparse_array_int32_t;
  let mut h = opj_dwt_t {
    mem: 0 as *mut OPJ_INT32,
    dn: 0,
    sn: 0,
    cas: 0,
  };
  let mut v = opj_dwt_t {
    mem: 0 as *mut OPJ_INT32,
    dn: 0,
    sn: 0,
    cas: 0,
  };
  let mut resno: OPJ_UINT32 = 0;
  /* This value matches the maximum left/right extension given in tables */
  /* F.2 and F.3 of the standard. */
  let filter_width = 2u32; /* width of the resolution level computed */
  let mut tr = (*tilec).resolutions; /* height of the resolution level computed */
  let mut tr_max: *mut opj_tcd_resolution_t = &mut *(*tilec)
    .resolutions
    .offset(numres.wrapping_sub(1u32) as isize)
    as *mut opj_tcd_resolution_t;
  let mut rw = ((*tr).x1 - (*tr).x0) as OPJ_UINT32;
  let mut rh = ((*tr).y1 - (*tr).y0) as OPJ_UINT32;
  let mut h_mem_size: OPJ_SIZE_T = 0;
  /* Compute the intersection of the area of interest, expressed in tile coordinates */
  /* with the tile coordinates */
  let mut win_tcx0 = (*tilec).win_x0;
  let mut win_tcy0 = (*tilec).win_y0;
  let mut win_tcx1 = (*tilec).win_x1;
  let mut win_tcy1 = (*tilec).win_y1;
  if (*tr_max).x0 == (*tr_max).x1 || (*tr_max).y0 == (*tr_max).y1 {
    return 1i32;
  }
  sa = opj_dwt_init_sparse_array(tilec, numres);
  if sa.is_null() {
    return 0i32;
  }
  if numres == 1u32 {
    let mut ret = opj_sparse_array_int32_read(
      sa,
      (*tr_max).win_x0.wrapping_sub((*tr_max).x0 as OPJ_UINT32),
      (*tr_max).win_y0.wrapping_sub((*tr_max).y0 as OPJ_UINT32),
      (*tr_max).win_x1.wrapping_sub((*tr_max).x0 as OPJ_UINT32),
      (*tr_max).win_y1.wrapping_sub((*tr_max).y0 as OPJ_UINT32),
      (*tilec).data_win,
      1 as OPJ_UINT32,
      (*tr_max).win_x1.wrapping_sub((*tr_max).win_x0),
      1i32,
    );
    assert!(ret != 0);
    opj_sparse_array_int32_free(sa);
    return 1i32;
  }
  h_mem_size = opj_dwt_max_resolution(tr, numres) as OPJ_SIZE_T;
  /* overflow check */
  /* in vertical pass, we process 4 columns at a time */
  if h_mem_size
    > (18446744073709551615u64).wrapping_div(
      (4u64)
        .wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
    )
  {
    /* FIXME event manager error callback */
    opj_sparse_array_int32_free(sa);
    return 0i32;
  }
  h_mem_size = (h_mem_size as libc::c_ulong).wrapping_mul(
    (4u64)
      .wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
  ) as OPJ_SIZE_T as OPJ_SIZE_T;
  h.mem = opj_aligned_32_malloc(h_mem_size) as *mut OPJ_INT32;
  if h.mem.is_null() {
    /* FIXME event manager error callback */
    opj_sparse_array_int32_free(sa);
    return 0i32;
  }
  v.mem = h.mem;
  resno = 1 as OPJ_UINT32;
  while resno < numres {
    let mut i: OPJ_UINT32 = 0;
    let mut j: OPJ_UINT32 = 0;
    /* Window of interest subband-based coordinates */
    let mut win_ll_x0: OPJ_UINT32 = 0;
    let mut win_ll_y0: OPJ_UINT32 = 0;
    let mut win_ll_x1: OPJ_UINT32 = 0;
    let mut win_ll_y1: OPJ_UINT32 = 0;
    let mut win_hl_x0: OPJ_UINT32 = 0;
    let mut win_hl_x1: OPJ_UINT32 = 0;
    let mut win_lh_y0: OPJ_UINT32 = 0;
    let mut win_lh_y1: OPJ_UINT32 = 0;
    /* Window of interest tile-resolution-based coordinates */
    let mut win_tr_x0: OPJ_UINT32 = 0;
    let mut win_tr_x1: OPJ_UINT32 = 0;
    let mut win_tr_y0: OPJ_UINT32 = 0;
    let mut win_tr_y1: OPJ_UINT32 = 0;
    /* Tile-resolution subband-based coordinates */
    let mut tr_ll_x0: OPJ_UINT32 = 0;
    let mut tr_ll_y0: OPJ_UINT32 = 0;
    let mut tr_hl_x0: OPJ_UINT32 = 0;
    let mut tr_lh_y0: OPJ_UINT32 = 0;
    tr = tr.offset(1);
    h.sn = rw as OPJ_INT32;
    v.sn = rh as OPJ_INT32;
    rw = ((*tr).x1 - (*tr).x0) as OPJ_UINT32;
    rh = ((*tr).y1 - (*tr).y0) as OPJ_UINT32;
    h.dn = rw.wrapping_sub(h.sn as OPJ_UINT32) as OPJ_INT32;
    h.cas = (*tr).x0 % 2i32;
    v.dn = rh.wrapping_sub(v.sn as OPJ_UINT32) as OPJ_INT32;
    v.cas = (*tr).y0 % 2i32;
    /* Get the subband coordinates for the window of interest */
    /* LL band */
    opj_dwt_get_band_coordinates(
      tilec,
      resno,
      0 as OPJ_UINT32,
      win_tcx0,
      win_tcy0,
      win_tcx1,
      win_tcy1,
      &mut win_ll_x0,
      &mut win_ll_y0,
      &mut win_ll_x1,
      &mut win_ll_y1,
    );
    /* HL band */
    opj_dwt_get_band_coordinates(
      tilec,
      resno,
      1 as OPJ_UINT32,
      win_tcx0,
      win_tcy0,
      win_tcx1,
      win_tcy1,
      &mut win_hl_x0,
      0 as *mut OPJ_UINT32,
      &mut win_hl_x1,
      0 as *mut OPJ_UINT32,
    );
    /* LH band */
    opj_dwt_get_band_coordinates(
      tilec,
      resno,
      2 as OPJ_UINT32,
      win_tcx0,
      win_tcy0,
      win_tcx1,
      win_tcy1,
      0 as *mut OPJ_UINT32,
      &mut win_lh_y0,
      0 as *mut OPJ_UINT32,
      &mut win_lh_y1,
    );
    /* Beware: band index for non-LL0 resolution are 0=HL, 1=LH and 2=HH */
    tr_ll_x0 = (*tr).bands[1 as usize].x0 as OPJ_UINT32;
    tr_ll_y0 = (*tr).bands[0 as usize].y0 as OPJ_UINT32;
    tr_hl_x0 = (*tr).bands[0 as usize].x0 as OPJ_UINT32;
    tr_lh_y0 = (*tr).bands[1 as usize].y0 as OPJ_UINT32;
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
      h.sn as OPJ_UINT32,
      &mut win_ll_x0,
      &mut win_ll_x1,
    );
    opj_dwt_segment_grow(
      filter_width,
      h.dn as OPJ_UINT32,
      &mut win_hl_x0,
      &mut win_hl_x1,
    );
    opj_dwt_segment_grow(
      filter_width,
      v.sn as OPJ_UINT32,
      &mut win_ll_y0,
      &mut win_ll_y1,
    );
    opj_dwt_segment_grow(
      filter_width,
      v.dn as OPJ_UINT32,
      &mut win_lh_y0,
      &mut win_lh_y1,
    );
    /* Compute the tile-resolution-based coordinates for the window of interest */
    if h.cas == 0i32 {
      win_tr_x0 = opj_uint_min(
        (2u32).wrapping_mul(win_ll_x0),
        (2u32)
          .wrapping_mul(win_hl_x0)
          .wrapping_add(1u32),
      );
      win_tr_x1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_ll_x1),
          (2u32)
            .wrapping_mul(win_hl_x1)
            .wrapping_add(1u32),
        ),
        rw,
      )
    } else {
      win_tr_x0 = opj_uint_min(
        (2u32).wrapping_mul(win_hl_x0),
        (2u32)
          .wrapping_mul(win_ll_x0)
          .wrapping_add(1u32),
      );
      win_tr_x1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_hl_x1),
          (2u32)
            .wrapping_mul(win_ll_x1)
            .wrapping_add(1u32),
        ),
        rw,
      )
    }
    if v.cas == 0i32 {
      win_tr_y0 = opj_uint_min(
        (2u32).wrapping_mul(win_ll_y0),
        (2u32)
          .wrapping_mul(win_lh_y0)
          .wrapping_add(1u32),
      );
      win_tr_y1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_ll_y1),
          (2u32)
            .wrapping_mul(win_lh_y1)
            .wrapping_add(1u32),
        ),
        rh,
      )
    } else {
      win_tr_y0 = opj_uint_min(
        (2u32).wrapping_mul(win_lh_y0),
        (2u32)
          .wrapping_mul(win_ll_y0)
          .wrapping_add(1u32),
      );
      win_tr_y1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_lh_y1),
          (2u32)
            .wrapping_mul(win_ll_y1)
            .wrapping_add(1u32),
        ),
        rh,
      )
    }
    j = 0 as OPJ_UINT32;
    while j < rh {
      if j >= win_ll_y0 && j < win_ll_y1
        || j >= win_lh_y0.wrapping_add(v.sn as OPJ_UINT32)
          && j < win_lh_y1.wrapping_add(v.sn as OPJ_UINT32)
      {
        /* Avoids dwt.c:1584:44 (in opj_dwt_decode_partial_1): runtime error: */
        /* signed integer overflow: -1094795586 + -1094795586 cannot be represented in type 'int' */
        /* on opj_decompress -i  ../../openjpeg/MAPA.jp2 -o out.tif -d 0,0,256,256 */
        /* This is less extreme than memsetting the whole buffer to 0 */
        /* although we could potentially do better with better handling of edge conditions */
        if win_tr_x1 >= 1u32 && win_tr_x1 < rw {
          *h.mem
            .offset(win_tr_x1.wrapping_sub(1u32) as isize) =
            0i32
        }
        if win_tr_x1 < rw {
          *h.mem.offset(win_tr_x1 as isize) = 0i32
        }
        opj_dwt_interleave_partial_h(
          h.mem,
          h.cas,
          sa,
          j,
          h.sn as OPJ_UINT32,
          win_ll_x0,
          win_ll_x1,
          win_hl_x0,
          win_hl_x1,
        );
        opj_dwt_decode_partial_1(
          h.mem,
          h.dn,
          h.sn,
          h.cas,
          win_ll_x0 as OPJ_INT32,
          win_ll_x1 as OPJ_INT32,
          win_hl_x0 as OPJ_INT32,
          win_hl_x1 as OPJ_INT32,
        );
        if opj_sparse_array_int32_write(
          sa,
          win_tr_x0,
          j,
          win_tr_x1,
          j.wrapping_add(1u32),
          h.mem.offset(win_tr_x0 as isize),
          1 as OPJ_UINT32,
          0 as OPJ_UINT32,
          1i32,
        ) == 0
        {
          /* FIXME event manager error callback */
          opj_sparse_array_int32_free(sa);
          opj_aligned_free(h.mem as *mut libc::c_void);
          return 0i32;
        }
      }
      j = j.wrapping_add(1)
    }
    i = win_tr_x0;
    while i < win_tr_x1 {
      let mut nb_cols = opj_uint_min(4u32, win_tr_x1.wrapping_sub(i));
      opj_dwt_interleave_partial_v(
        v.mem,
        v.cas,
        sa,
        i,
        nb_cols,
        v.sn as OPJ_UINT32,
        win_ll_y0,
        win_ll_y1,
        win_lh_y0,
        win_lh_y1,
      );
      opj_dwt_decode_partial_1_parallel(
        v.mem,
        nb_cols,
        v.dn,
        v.sn,
        v.cas,
        win_ll_y0 as OPJ_INT32,
        win_ll_y1 as OPJ_INT32,
        win_lh_y0 as OPJ_INT32,
        win_lh_y1 as OPJ_INT32,
      );
      if opj_sparse_array_int32_write(
        sa,
        i,
        win_tr_y0,
        i.wrapping_add(nb_cols),
        win_tr_y1,
        v.mem
          .offset((4u32).wrapping_mul(win_tr_y0) as isize),
        1 as OPJ_UINT32,
        4 as OPJ_UINT32,
        1i32,
      ) == 0
      {
        /* FIXME event manager error callback */
        opj_sparse_array_int32_free(sa);
        opj_aligned_free(h.mem as *mut libc::c_void);
        return 0i32;
      }
      i = (i as libc::c_uint).wrapping_add(nb_cols) as OPJ_UINT32
    }
    resno = resno.wrapping_add(1)
  }
  opj_aligned_free(h.mem as *mut libc::c_void);
  let mut ret_0 = opj_sparse_array_int32_read(
    sa,
    (*tr_max).win_x0.wrapping_sub((*tr_max).x0 as OPJ_UINT32),
    (*tr_max).win_y0.wrapping_sub((*tr_max).y0 as OPJ_UINT32),
    (*tr_max).win_x1.wrapping_sub((*tr_max).x0 as OPJ_UINT32),
    (*tr_max).win_y1.wrapping_sub((*tr_max).y0 as OPJ_UINT32),
    (*tilec).data_win,
    1 as OPJ_UINT32,
    (*tr_max).win_x1.wrapping_sub((*tr_max).win_x0),
    1i32,
  );
  assert!(ret_0 != 0);
  opj_sparse_array_int32_free(sa);
  return 1i32;
}
unsafe fn opj_v8dwt_interleave_h(
  mut dwt: *mut opj_v8dwt_t,
  mut a: *mut OPJ_FLOAT32,
  mut width: OPJ_UINT32,
  mut remaining_height: OPJ_UINT32,
) {
  let mut bi = (*dwt).wavelet.offset((*dwt).cas as isize) as *mut OPJ_FLOAT32;
  let mut i: OPJ_UINT32 = 0;
  let mut k: OPJ_UINT32 = 0;
  let mut x0 = (*dwt).win_l_x0;
  let mut x1 = (*dwt).win_l_x1;
  k = 0 as OPJ_UINT32;
  while k < 2u32 {
    if remaining_height >= 8u32
      && a as OPJ_SIZE_T & 0xfu64 == 0u64
      && bi as OPJ_SIZE_T & 0xfu64 == 0u64
    {
      /* Fast code path */
      i = x0;
      while i < x1 {
        let mut j = i;
        let mut dst = bi.offset(
          i.wrapping_mul(2u32)
            .wrapping_mul(8u32) as isize,
        );
        *dst.offset(0) = *a.offset(j as isize);
        j = (j as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
        *dst.offset(1) = *a.offset(j as isize);
        j = (j as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
        *dst.offset(2) = *a.offset(j as isize);
        j = (j as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
        *dst.offset(3) = *a.offset(j as isize);
        j = (j as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
        *dst.offset(4) = *a.offset(j as isize);
        j = (j as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
        *dst.offset(5) = *a.offset(j as isize);
        j = (j as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
        *dst.offset(6) = *a.offset(j as isize);
        j = (j as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
        *dst.offset(7) = *a.offset(j as isize);
        i = i.wrapping_add(1)
      }
    } else {
      /* Slow code path */
      i = x0;
      while i < x1 {
        let mut j_0 = i;
        let mut dst_0 = bi.offset(
          i.wrapping_mul(2u32)
            .wrapping_mul(8u32) as isize,
        );
        *dst_0.offset(0) = *a.offset(j_0 as isize);
        j_0 = (j_0 as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
        if !(remaining_height == 1u32) {
          *dst_0.offset(1) = *a.offset(j_0 as isize);
          j_0 = (j_0 as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
          if !(remaining_height == 2u32) {
            *dst_0.offset(2) = *a.offset(j_0 as isize);
            j_0 = (j_0 as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
            if !(remaining_height == 3u32) {
              *dst_0.offset(3) = *a.offset(j_0 as isize);
              j_0 = (j_0 as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
              if !(remaining_height == 4u32) {
                *dst_0.offset(4) = *a.offset(j_0 as isize);
                j_0 = (j_0 as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
                if !(remaining_height == 5u32) {
                  *dst_0.offset(5) = *a.offset(j_0 as isize);
                  j_0 = (j_0 as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
                  if !(remaining_height == 6u32) {
                    *dst_0.offset(6) = *a.offset(j_0 as isize);
                    j_0 = (j_0 as libc::c_uint).wrapping_add(width) as OPJ_UINT32;
                    if !(remaining_height == 7u32) {
                      *dst_0.offset(7) = *a.offset(j_0 as isize)
                    }
                  }
                }
              }
            }
          }
        }
        i = i.wrapping_add(1)
      }
    }
    bi = (*dwt)
      .wavelet
      .offset(1)
      .offset(-((*dwt).cas as isize)) as *mut OPJ_FLOAT32;
    a = a.offset((*dwt).sn as isize);
    x0 = (*dwt).win_h_x0;
    x1 = (*dwt).win_h_x1;
    k = k.wrapping_add(1)
  }
}
unsafe fn opj_v8dwt_interleave_partial_h(
  mut dwt: *mut opj_v8dwt_t,
  mut sa: *mut opj_sparse_array_int32_t,
  mut sa_line: OPJ_UINT32,
  mut remaining_height: OPJ_UINT32,
) {
  let mut i: OPJ_UINT32 = 0;
  i = 0 as OPJ_UINT32;
  while i < remaining_height {
    let mut ret: OPJ_BOOL = 0;
    ret = opj_sparse_array_int32_read(
      sa,
      (*dwt).win_l_x0,
      sa_line.wrapping_add(i),
      (*dwt).win_l_x1,
      sa_line
        .wrapping_add(i)
        .wrapping_add(1u32),
      ((*dwt)
        .wavelet
        .offset((*dwt).cas as isize)
        .offset((2u32).wrapping_mul((*dwt).win_l_x0) as isize)
        as *mut OPJ_INT32)
        .offset(i as isize),
      (2i32 * 8i32) as OPJ_UINT32,
      0 as OPJ_UINT32,
      1i32,
    );
    assert!(ret != 0);
    ret = opj_sparse_array_int32_read(
      sa,
      ((*dwt).sn as OPJ_UINT32).wrapping_add((*dwt).win_h_x0),
      sa_line.wrapping_add(i),
      ((*dwt).sn as OPJ_UINT32).wrapping_add((*dwt).win_h_x1),
      sa_line
        .wrapping_add(i)
        .wrapping_add(1u32),
      ((*dwt)
        .wavelet
        .offset(1)
        .offset(-((*dwt).cas as isize))
        .offset((2u32).wrapping_mul((*dwt).win_h_x0) as isize)
        as *mut OPJ_INT32)
        .offset(i as isize),
      (2i32 * 8i32) as OPJ_UINT32,
      0 as OPJ_UINT32,
      1i32,
    );
    assert!(ret != 0);
    i = i.wrapping_add(1)
  }
}
#[inline]
unsafe fn opj_v8dwt_interleave_v(
  mut dwt: *mut opj_v8dwt_t,
  mut a: *mut OPJ_FLOAT32,
  mut width: OPJ_UINT32,
  mut nb_elts_read: OPJ_UINT32,
) {
  let mut bi = (*dwt).wavelet.offset((*dwt).cas as isize);
  let mut i: OPJ_UINT32 = 0;
  i = (*dwt).win_l_x0;
  while i < (*dwt).win_l_x1 {
    memcpy(
      &mut *bi.offset(i.wrapping_mul(2u32) as isize) as *mut opj_v8_t
        as *mut libc::c_void,
      &mut *a.offset((i as libc::c_ulong).wrapping_mul(width as OPJ_SIZE_T) as isize)
        as *mut OPJ_FLOAT32 as *const libc::c_void,
      (nb_elts_read as OPJ_SIZE_T)
        .wrapping_mul(::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong),
    );
    i = i.wrapping_add(1)
  }
  a =
    a.offset(((*dwt).sn as OPJ_UINT32 as libc::c_ulong).wrapping_mul(width as OPJ_SIZE_T) as isize);
  bi = (*dwt)
    .wavelet
    .offset(1)
    .offset(-((*dwt).cas as isize));
  i = (*dwt).win_h_x0;
  while i < (*dwt).win_h_x1 {
    memcpy(
      &mut *bi.offset(i.wrapping_mul(2u32) as isize) as *mut opj_v8_t
        as *mut libc::c_void,
      &mut *a.offset((i as libc::c_ulong).wrapping_mul(width as OPJ_SIZE_T) as isize)
        as *mut OPJ_FLOAT32 as *const libc::c_void,
      (nb_elts_read as OPJ_SIZE_T)
        .wrapping_mul(::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong),
    );
    i = i.wrapping_add(1)
  }
}
unsafe fn opj_v8dwt_interleave_partial_v(
  mut dwt: *mut opj_v8dwt_t,
  mut sa: *mut opj_sparse_array_int32_t,
  mut sa_col: OPJ_UINT32,
  mut nb_elts_read: OPJ_UINT32,
) {
  let mut ret: OPJ_BOOL = 0;
  ret = opj_sparse_array_int32_read(
    sa,
    sa_col,
    (*dwt).win_l_x0,
    sa_col.wrapping_add(nb_elts_read),
    (*dwt).win_l_x1,
    (*dwt)
      .wavelet
      .offset((*dwt).cas as isize)
      .offset((2u32).wrapping_mul((*dwt).win_l_x0) as isize)
      as *mut OPJ_INT32,
    1 as OPJ_UINT32,
    (2i32 * 8i32) as OPJ_UINT32,
    1i32,
  );
  assert!(ret != 0);
  ret = opj_sparse_array_int32_read(
    sa,
    sa_col,
    ((*dwt).sn as OPJ_UINT32).wrapping_add((*dwt).win_h_x0),
    sa_col.wrapping_add(nb_elts_read),
    ((*dwt).sn as OPJ_UINT32).wrapping_add((*dwt).win_h_x1),
    (*dwt)
      .wavelet
      .offset(1)
      .offset(-((*dwt).cas as isize))
      .offset((2u32).wrapping_mul((*dwt).win_h_x0) as isize)
      as *mut OPJ_INT32,
    1 as OPJ_UINT32,
    (2i32 * 8i32) as OPJ_UINT32,
    1i32,
  );
  assert!(ret != 0);
}
unsafe fn opj_v8dwt_decode_step1(
  mut w: *mut opj_v8_t,
  mut start: OPJ_UINT32,
  mut end: OPJ_UINT32,
  c: OPJ_FLOAT32,
) {
  let mut fw = w as *mut OPJ_FLOAT32;
  let mut i: OPJ_UINT32 = 0;
  /* To be adapted if NB_ELTS_V8 changes */
  i = start;
  while i < end {
    *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32) as isize,
    ) = *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32) as isize,
    ) * c;
    *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(1u32) as isize,
    ) = *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(1u32) as isize,
    ) * c;
    *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(2u32) as isize,
    ) = *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(2u32) as isize,
    ) * c;
    *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(3u32) as isize,
    ) = *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(3u32) as isize,
    ) * c;
    *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(4u32) as isize,
    ) = *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(4u32) as isize,
    ) * c;
    *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(5u32) as isize,
    ) = *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(5u32) as isize,
    ) * c;
    *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(6u32) as isize,
    ) = *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(6u32) as isize,
    ) * c;
    *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(7u32) as isize,
    ) = *fw.offset(
      i.wrapping_mul(2u32)
        .wrapping_mul(8u32)
        .wrapping_add(7u32) as isize,
    ) * c;
    i = i.wrapping_add(1)
  }
}
unsafe fn opj_v8dwt_decode_step2(
  mut l: *mut opj_v8_t,
  mut w: *mut opj_v8_t,
  mut start: OPJ_UINT32,
  mut end: OPJ_UINT32,
  mut m: OPJ_UINT32,
  mut c: OPJ_FLOAT32,
) {
  let mut fl = l as *mut OPJ_FLOAT32;
  let mut fw = w as *mut OPJ_FLOAT32;
  let mut i: OPJ_UINT32 = 0;
  let mut imax = opj_uint_min(end, m);
  if start > 0u32 {
    fw = fw
      .offset(((2i32 * 8i32) as libc::c_uint).wrapping_mul(start) as isize);
    fl = fw.offset(-((2i32 * 8i32) as isize))
  }
  /* To be adapted if NB_ELTS_V8 changes */
  i = start;
  while i < imax {
    *fw.offset(-(8i32) as isize) = *fw.offset(-(8i32) as isize)
      + (*fl.offset(0) + *fw.offset(0)) * c;
    *fw.offset(-(7i32) as isize) = *fw.offset(-(7i32) as isize)
      + (*fl.offset(1) + *fw.offset(1)) * c;
    *fw.offset(-(6i32) as isize) = *fw.offset(-(6i32) as isize)
      + (*fl.offset(2) + *fw.offset(2)) * c;
    *fw.offset(-(5i32) as isize) = *fw.offset(-(5i32) as isize)
      + (*fl.offset(3) + *fw.offset(3)) * c;
    *fw.offset(-(4i32) as isize) = *fw.offset(-(4i32) as isize)
      + (*fl.offset(4) + *fw.offset(4)) * c;
    *fw.offset(-(3i32) as isize) = *fw.offset(-(3i32) as isize)
      + (*fl.offset(5) + *fw.offset(5)) * c;
    *fw.offset(-(2i32) as isize) = *fw.offset(-(2i32) as isize)
      + (*fl.offset(6) + *fw.offset(6)) * c;
    *fw.offset(-(1i32) as isize) = *fw.offset(-(1i32) as isize)
      + (*fl.offset(7) + *fw.offset(7)) * c;
    fl = fw;
    fw = fw.offset((2i32 * 8i32) as isize);
    i = i.wrapping_add(1)
  }
  if m < end {
    assert!(m.wrapping_add(1u32) == end);
    c += c;
    *fw.offset(-(8i32) as isize) =
      *fw.offset(-(8i32) as isize) + *fl.offset(0) * c;
    *fw.offset(-(7i32) as isize) =
      *fw.offset(-(7i32) as isize) + *fl.offset(1) * c;
    *fw.offset(-(6i32) as isize) =
      *fw.offset(-(6i32) as isize) + *fl.offset(2) * c;
    *fw.offset(-(5i32) as isize) =
      *fw.offset(-(5i32) as isize) + *fl.offset(3) * c;
    *fw.offset(-(4i32) as isize) =
      *fw.offset(-(4i32) as isize) + *fl.offset(4) * c;
    *fw.offset(-(3i32) as isize) =
      *fw.offset(-(3i32) as isize) + *fl.offset(5) * c;
    *fw.offset(-(2i32) as isize) =
      *fw.offset(-(2i32) as isize) + *fl.offset(6) * c;
    *fw.offset(-(1i32) as isize) =
      *fw.offset(-(1i32) as isize) + *fl.offset(7) * c
  };
}
/* <summary>                             */
/* Inverse 9-7 wavelet transform in 1-D. */
/* </summary>                            */
unsafe fn opj_v8dwt_decode(mut dwt: *mut opj_v8dwt_t) {
  let mut a: OPJ_INT32 = 0;
  let mut b: OPJ_INT32 = 0;
  /* BUG_WEIRD_TWO_INVK (look for this identifier in tcd.c) */
  /* Historic value for 2 / opj_invK */
  /* Normally, we should use invK, but if we do so, we have failures in the */
  /* conformance test, due to MSE and peak errors significantly higher than */
  /* accepted value */
  /* Due to using two_invK instead of invK, we have to compensate in tcd.c */
  /* the computation of the stepsize for the non LL subbands */
  let two_invK = 1.625732422f32;
  if (*dwt).cas == 0i32 {
    if !((*dwt).dn > 0i32 || (*dwt).sn > 1i32) {
      return;
    }
    a = 0i32;
    b = 1i32
  } else {
    if !((*dwt).sn > 0i32 || (*dwt).dn > 1i32) {
      return;
    }
    a = 1i32;
    b = 0i32
  }
  opj_v8dwt_decode_step1(
    (*dwt).wavelet.offset(a as isize),
    (*dwt).win_l_x0,
    (*dwt).win_l_x1,
    opj_K,
  );
  opj_v8dwt_decode_step1(
    (*dwt).wavelet.offset(b as isize),
    (*dwt).win_h_x0,
    (*dwt).win_h_x1,
    two_invK,
  );
  opj_v8dwt_decode_step2(
    (*dwt).wavelet.offset(b as isize),
    (*dwt)
      .wavelet
      .offset(a as isize)
      .offset(1),
    (*dwt).win_l_x0,
    (*dwt).win_l_x1,
    opj_int_min((*dwt).sn, (*dwt).dn - a) as OPJ_UINT32,
    -opj_dwt_delta,
  );
  opj_v8dwt_decode_step2(
    (*dwt).wavelet.offset(a as isize),
    (*dwt)
      .wavelet
      .offset(b as isize)
      .offset(1),
    (*dwt).win_h_x0,
    (*dwt).win_h_x1,
    opj_int_min((*dwt).dn, (*dwt).sn - b) as OPJ_UINT32,
    -opj_dwt_gamma,
  );
  opj_v8dwt_decode_step2(
    (*dwt).wavelet.offset(b as isize),
    (*dwt)
      .wavelet
      .offset(a as isize)
      .offset(1),
    (*dwt).win_l_x0,
    (*dwt).win_l_x1,
    opj_int_min((*dwt).sn, (*dwt).dn - a) as OPJ_UINT32,
    -opj_dwt_beta,
  );
  opj_v8dwt_decode_step2(
    (*dwt).wavelet.offset(a as isize),
    (*dwt)
      .wavelet
      .offset(b as isize)
      .offset(1),
    (*dwt).win_h_x0,
    (*dwt).win_h_x1,
    opj_int_min((*dwt).dn, (*dwt).sn - b) as OPJ_UINT32,
    -opj_dwt_alpha,
  );
}
unsafe extern "C" fn opj_dwt97_decode_h_func(
  mut user_data: *mut libc::c_void,
  mut _tls: *mut opj_tls_t,
) {
  let mut j: OPJ_UINT32 = 0;
  let mut job = 0 as *mut opj_dwt97_decode_h_job_t;
  let mut aj = 0 as *mut OPJ_FLOAT32;
  let mut w: OPJ_UINT32 = 0;
  job = user_data as *mut opj_dwt97_decode_h_job_t;
  w = (*job).w;
  assert!(
    (*job)
      .nb_rows
      .wrapping_rem(8u32)
      == 0u32
  );
  aj = (*job).aj;
  j = 0 as OPJ_UINT32;
  while j.wrapping_add(8u32) <= (*job).nb_rows {
    let mut k: OPJ_UINT32 = 0;
    opj_v8dwt_interleave_h(&mut (*job).h, aj, (*job).w, 8 as OPJ_UINT32);
    opj_v8dwt_decode(&mut (*job).h);
    /* To be adapted if NB_ELTS_V8 changes */
    k = 0 as OPJ_UINT32;
    while k < (*job).rw {
      *aj.offset(k as isize) = (*(*job).h.wavelet.offset(k as isize)).f[0 as usize];
      *aj.offset((k as libc::c_ulong).wrapping_add(w as OPJ_SIZE_T) as isize) =
        (*(*job).h.wavelet.offset(k as isize)).f[1 as usize];
      *aj.offset(
        (k as libc::c_ulong)
          .wrapping_add((w as OPJ_SIZE_T).wrapping_mul(2u64))
          as isize,
      ) = (*(*job).h.wavelet.offset(k as isize)).f[2 as usize];
      *aj.offset(
        (k as libc::c_ulong)
          .wrapping_add((w as OPJ_SIZE_T).wrapping_mul(3u64))
          as isize,
      ) = (*(*job).h.wavelet.offset(k as isize)).f[3 as usize];
      k = k.wrapping_add(1)
    }
    k = 0 as OPJ_UINT32;
    while k < (*job).rw {
      *aj.offset(
        (k as libc::c_ulong)
          .wrapping_add((w as OPJ_SIZE_T).wrapping_mul(4u64))
          as isize,
      ) = (*(*job).h.wavelet.offset(k as isize)).f[4 as usize];
      *aj.offset(
        (k as libc::c_ulong)
          .wrapping_add((w as OPJ_SIZE_T).wrapping_mul(5u64))
          as isize,
      ) = (*(*job).h.wavelet.offset(k as isize)).f[5 as usize];
      *aj.offset(
        (k as libc::c_ulong)
          .wrapping_add((w as OPJ_SIZE_T).wrapping_mul(6u64))
          as isize,
      ) = (*(*job).h.wavelet.offset(k as isize)).f[6 as usize];
      *aj.offset(
        (k as libc::c_ulong)
          .wrapping_add((w as OPJ_SIZE_T).wrapping_mul(7u64))
          as isize,
      ) = (*(*job).h.wavelet.offset(k as isize)).f[7 as usize];
      k = k.wrapping_add(1)
    }
    aj = aj.offset(w.wrapping_mul(8u32) as isize);
    j =
      (j as libc::c_uint).wrapping_add(8u32) as OPJ_UINT32
  }
  opj_aligned_free((*job).h.wavelet as *mut libc::c_void);
  opj_free(job as *mut libc::c_void);
}
unsafe extern "C" fn opj_dwt97_decode_v_func(
  mut user_data: *mut libc::c_void,
  mut _tls: *mut opj_tls_t,
) {
  let mut j: OPJ_UINT32 = 0;
  let mut job = 0 as *mut opj_dwt97_decode_v_job_t;
  let mut aj = 0 as *mut OPJ_FLOAT32;
  job = user_data as *mut opj_dwt97_decode_v_job_t;
  assert!(
    (*job)
      .nb_columns
      .wrapping_rem(8u32)
      == 0u32
  );
  aj = (*job).aj;
  j = 0 as OPJ_UINT32;
  while j.wrapping_add(8u32) <= (*job).nb_columns {
    let mut k: OPJ_UINT32 = 0;
    opj_v8dwt_interleave_v(&mut (*job).v, aj, (*job).w, 8 as OPJ_UINT32);
    opj_v8dwt_decode(&mut (*job).v);
    k = 0 as OPJ_UINT32;
    while k < (*job).rh {
      memcpy(
        &mut *aj.offset((k as libc::c_ulong).wrapping_mul((*job).w as OPJ_SIZE_T) as isize)
          as *mut OPJ_FLOAT32 as *mut libc::c_void,
        &mut *(*job).v.wavelet.offset(k as isize) as *mut opj_v8_t as *const libc::c_void,
        (8u64)
          .wrapping_mul(::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong),
      );
      k = k.wrapping_add(1)
    }
    aj = aj.offset(8);
    j =
      (j as libc::c_uint).wrapping_add(8u32) as OPJ_UINT32
  }
  opj_aligned_free((*job).v.wavelet as *mut libc::c_void);
  opj_free(job as *mut libc::c_void);
}
/* <summary>                             */
/* Inverse 9-7 wavelet transform in 2-D. */
/* </summary>                            */
unsafe fn opj_dwt_decode_tile_97(
  mut tp: *mut opj_thread_pool_t,
  mut tilec: *mut opj_tcd_tilecomp_t,
  mut numres: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut h = opj_v8dwt_t {
    wavelet: 0 as *mut opj_v8_t,
    dn: 0,
    sn: 0,
    cas: 0,
    win_l_x0: 0,
    win_l_x1: 0,
    win_h_x0: 0,
    win_h_x1: 0,
  }; /* width of the resolution level computed */
  let mut v = opj_v8dwt_t {
    wavelet: 0 as *mut opj_v8_t,
    dn: 0,
    sn: 0,
    cas: 0,
    win_l_x0: 0,
    win_l_x1: 0,
    win_h_x0: 0,
    win_h_x1: 0,
  }; /* height of the resolution level computed */
  let mut res = (*tilec).resolutions;
  let mut rw = ((*res).x1 - (*res).x0) as OPJ_UINT32;
  let mut rh = ((*res).y1 - (*res).y0) as OPJ_UINT32;
  let mut w = ((*(*tilec).resolutions.offset(
    (*tilec)
      .minimum_num_resolutions
      .wrapping_sub(1u32) as isize,
  ))
  .x1
    - (*(*tilec).resolutions.offset(
      (*tilec)
        .minimum_num_resolutions
        .wrapping_sub(1u32) as isize,
    ))
    .x0) as OPJ_UINT32;
  let mut l_data_size: OPJ_SIZE_T = 0;
  let num_threads = opj_thread_pool_get_thread_count(tp);
  if numres == 1u32 {
    return 1i32;
  }
  l_data_size = opj_dwt_max_resolution(res, numres) as OPJ_SIZE_T;
  /* overflow check */
  if l_data_size
    > (18446744073709551615u64)
      .wrapping_div(::std::mem::size_of::<opj_v8_t>() as libc::c_ulong)
  {
    /* FIXME event manager error callback */
    return 0i32;
  }
  h.wavelet = opj_aligned_malloc(
    l_data_size.wrapping_mul(::std::mem::size_of::<opj_v8_t>() as libc::c_ulong),
  ) as *mut opj_v8_t;
  if h.wavelet.is_null() {
    /* FIXME event manager error callback */
    return 0i32;
  } /* width of the resolution level computed */
  v.wavelet = h.wavelet; /* height of the resolution level computed */
  loop {
    numres = numres.wrapping_sub(1);
    if !(numres != 0) {
      break;
    }
    let mut aj = (*tilec).data as *mut OPJ_FLOAT32;
    let mut j: OPJ_UINT32 = 0;
    h.sn = rw as OPJ_INT32;
    v.sn = rh as OPJ_INT32;
    res = res.offset(1);
    rw = ((*res).x1 - (*res).x0) as OPJ_UINT32;
    rh = ((*res).y1 - (*res).y0) as OPJ_UINT32;
    h.dn = rw.wrapping_sub(h.sn as OPJ_UINT32) as OPJ_INT32;
    h.cas = (*res).x0 % 2i32;
    h.win_l_x0 = 0 as OPJ_UINT32;
    h.win_l_x1 = h.sn as OPJ_UINT32;
    h.win_h_x0 = 0 as OPJ_UINT32;
    h.win_h_x1 = h.dn as OPJ_UINT32;
    if num_threads <= 1i32 || rh < (2i32 * 8i32) as libc::c_uint
    {
      j = 0 as OPJ_UINT32;
      while j.wrapping_add((8i32 - 1i32) as libc::c_uint) < rh {
        let mut k: OPJ_UINT32 = 0;
        opj_v8dwt_interleave_h(&mut h, aj, w, 8 as OPJ_UINT32);
        opj_v8dwt_decode(&mut h);
        /* To be adapted if NB_ELTS_V8 changes */
        k = 0 as OPJ_UINT32;
        while k < rw {
          *aj.offset(k as isize) = (*h.wavelet.offset(k as isize)).f[0 as usize];
          *aj.offset((k as libc::c_ulong).wrapping_add(w as OPJ_SIZE_T) as isize) =
            (*h.wavelet.offset(k as isize)).f[1 as usize];
          *aj.offset(
            (k as libc::c_ulong)
              .wrapping_add((w as OPJ_SIZE_T).wrapping_mul(2u64))
              as isize,
          ) = (*h.wavelet.offset(k as isize)).f[2 as usize];
          *aj.offset(
            (k as libc::c_ulong)
              .wrapping_add((w as OPJ_SIZE_T).wrapping_mul(3u64))
              as isize,
          ) = (*h.wavelet.offset(k as isize)).f[3 as usize];
          k = k.wrapping_add(1)
        }
        k = 0 as OPJ_UINT32;
        while k < rw {
          *aj.offset(
            (k as libc::c_ulong)
              .wrapping_add((w as OPJ_SIZE_T).wrapping_mul(4u64))
              as isize,
          ) = (*h.wavelet.offset(k as isize)).f[4 as usize];
          *aj.offset(
            (k as libc::c_ulong)
              .wrapping_add((w as OPJ_SIZE_T).wrapping_mul(5u64))
              as isize,
          ) = (*h.wavelet.offset(k as isize)).f[5 as usize];
          *aj.offset(
            (k as libc::c_ulong)
              .wrapping_add((w as OPJ_SIZE_T).wrapping_mul(6u64))
              as isize,
          ) = (*h.wavelet.offset(k as isize)).f[6 as usize];
          *aj.offset(
            (k as libc::c_ulong)
              .wrapping_add((w as OPJ_SIZE_T).wrapping_mul(7u64))
              as isize,
          ) = (*h.wavelet.offset(k as isize)).f[7 as usize];
          k = k.wrapping_add(1)
        }
        aj = aj.offset(w.wrapping_mul(8u32) as isize);
        j = (j as libc::c_uint).wrapping_add(8u32) as OPJ_UINT32
          as OPJ_UINT32
      }
    } else {
      let mut num_jobs = num_threads as OPJ_UINT32;
      let mut step_j: OPJ_UINT32 = 0;
      if rh.wrapping_div(8u32) < num_jobs {
        num_jobs = rh.wrapping_div(8u32)
      }
      step_j = rh
        .wrapping_div(num_jobs)
        .wrapping_div(8u32)
        .wrapping_mul(8u32);
      j = 0 as OPJ_UINT32;
      while j < num_jobs {
        let mut job = 0 as *mut opj_dwt97_decode_h_job_t;
        job = opj_malloc(::std::mem::size_of::<opj_dwt97_decode_h_job_t>() as libc::c_ulong)
          as *mut opj_dwt97_decode_h_job_t;
        if job.is_null() {
          opj_thread_pool_wait_completion(tp, 0i32);
          opj_aligned_free(h.wavelet as *mut libc::c_void);
          return 0i32;
        }
        (*job).h.wavelet = opj_aligned_malloc(
          l_data_size.wrapping_mul(::std::mem::size_of::<opj_v8_t>() as libc::c_ulong),
        ) as *mut opj_v8_t;
        if (*job).h.wavelet.is_null() {
          opj_thread_pool_wait_completion(tp, 0i32);
          opj_free(job as *mut libc::c_void);
          opj_aligned_free(h.wavelet as *mut libc::c_void);
          return 0i32;
        }
        (*job).h.dn = h.dn;
        (*job).h.sn = h.sn;
        (*job).h.cas = h.cas;
        (*job).h.win_l_x0 = h.win_l_x0;
        (*job).h.win_l_x1 = h.win_l_x1;
        (*job).h.win_h_x0 = h.win_h_x0;
        (*job).h.win_h_x1 = h.win_h_x1;
        (*job).rw = rw;
        (*job).w = w;
        (*job).aj = aj;
        (*job).nb_rows = if j.wrapping_add(1u32) == num_jobs {
          (rh & !(8i32 - 1i32) as OPJ_UINT32)
            .wrapping_sub(j.wrapping_mul(step_j))
        } else {
          step_j
        };
        aj = aj.offset(w.wrapping_mul((*job).nb_rows) as isize);
        opj_thread_pool_submit_job(
          tp,
          Some(
            opj_dwt97_decode_h_func
              as unsafe extern "C" fn(_: *mut libc::c_void, _: *mut opj_tls_t) -> (),
          ),
          job as *mut libc::c_void,
        );
        j = j.wrapping_add(1)
      }
      opj_thread_pool_wait_completion(tp, 0i32);
      j = rh & !(8i32 - 1i32) as OPJ_UINT32
    }
    if j < rh {
      let mut k_0: OPJ_UINT32 = 0;
      opj_v8dwt_interleave_h(&mut h, aj, w, rh.wrapping_sub(j));
      opj_v8dwt_decode(&mut h);
      k_0 = 0 as OPJ_UINT32;
      while k_0 < rw {
        let mut l: OPJ_UINT32 = 0;
        l = 0 as OPJ_UINT32;
        while l < rh.wrapping_sub(j) {
          *aj.offset(
            (k_0 as libc::c_ulong).wrapping_add((w as OPJ_SIZE_T).wrapping_mul(l as libc::c_ulong))
              as isize,
          ) = (*h.wavelet.offset(k_0 as isize)).f[l as usize];
          l = l.wrapping_add(1)
        }
        k_0 = k_0.wrapping_add(1)
      }
    }
    v.dn = rh.wrapping_sub(v.sn as OPJ_UINT32) as OPJ_INT32;
    v.cas = (*res).y0 % 2i32;
    v.win_l_x0 = 0 as OPJ_UINT32;
    v.win_l_x1 = v.sn as OPJ_UINT32;
    v.win_h_x0 = 0 as OPJ_UINT32;
    v.win_h_x1 = v.dn as OPJ_UINT32;
    aj = (*tilec).data as *mut OPJ_FLOAT32;
    if num_threads <= 1i32 || rw < (2i32 * 8i32) as libc::c_uint
    {
      j = rw;
      while j > (8i32 - 1i32) as libc::c_uint {
        let mut k_1: OPJ_UINT32 = 0;
        opj_v8dwt_interleave_v(&mut v, aj, w, 8 as OPJ_UINT32);
        opj_v8dwt_decode(&mut v);
        k_1 = 0 as OPJ_UINT32;
        while k_1 < rh {
          memcpy(
            &mut *aj.offset((k_1 as libc::c_ulong).wrapping_mul(w as OPJ_SIZE_T) as isize)
              as *mut OPJ_FLOAT32 as *mut libc::c_void,
            &mut *v.wavelet.offset(k_1 as isize) as *mut opj_v8_t as *const libc::c_void,
            (8u64)
              .wrapping_mul(::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong),
          );
          k_1 = k_1.wrapping_add(1)
        }
        aj = aj.offset(8);
        j = (j as libc::c_uint).wrapping_sub(8u32) as OPJ_UINT32
          as OPJ_UINT32
      }
    } else {
      /* "bench_dwt -I" shows that scaling is poor, likely due to RAM
         transfer being the limiting factor. So limit the number of
         threads.
      */
      let mut num_jobs_0 = opj_uint_max(
        (num_threads as OPJ_UINT32).wrapping_div(2u32),
        2u32,
      );
      let mut step_j_0: OPJ_UINT32 = 0;
      if rw.wrapping_div(8u32) < num_jobs_0 {
        num_jobs_0 = rw.wrapping_div(8u32)
      }
      step_j_0 = rw
        .wrapping_div(num_jobs_0)
        .wrapping_div(8u32)
        .wrapping_mul(8u32);
      j = 0 as OPJ_UINT32;
      while j < num_jobs_0 {
        let mut job_0 = 0 as *mut opj_dwt97_decode_v_job_t;
        job_0 = opj_malloc(::std::mem::size_of::<opj_dwt97_decode_v_job_t>() as libc::c_ulong)
          as *mut opj_dwt97_decode_v_job_t;
        if job_0.is_null() {
          opj_thread_pool_wait_completion(tp, 0i32);
          opj_aligned_free(h.wavelet as *mut libc::c_void);
          return 0i32;
        }
        (*job_0).v.wavelet = opj_aligned_malloc(
          l_data_size.wrapping_mul(::std::mem::size_of::<opj_v8_t>() as libc::c_ulong),
        ) as *mut opj_v8_t;
        if (*job_0).v.wavelet.is_null() {
          opj_thread_pool_wait_completion(tp, 0i32);
          opj_free(job_0 as *mut libc::c_void);
          opj_aligned_free(h.wavelet as *mut libc::c_void);
          return 0i32;
        }
        (*job_0).v.dn = v.dn;
        (*job_0).v.sn = v.sn;
        (*job_0).v.cas = v.cas;
        (*job_0).v.win_l_x0 = v.win_l_x0;
        (*job_0).v.win_l_x1 = v.win_l_x1;
        (*job_0).v.win_h_x0 = v.win_h_x0;
        (*job_0).v.win_h_x1 = v.win_h_x1;
        (*job_0).rh = rh;
        (*job_0).w = w;
        (*job_0).aj = aj;
        (*job_0).nb_columns = if j.wrapping_add(1u32) == num_jobs_0 {
          (rw & !(8i32 - 1i32) as OPJ_UINT32)
            .wrapping_sub(j.wrapping_mul(step_j_0))
        } else {
          step_j_0
        };
        aj = aj.offset((*job_0).nb_columns as isize);
        opj_thread_pool_submit_job(
          tp,
          Some(
            opj_dwt97_decode_v_func
              as unsafe extern "C" fn(_: *mut libc::c_void, _: *mut opj_tls_t) -> (),
          ),
          job_0 as *mut libc::c_void,
        );
        j = j.wrapping_add(1)
      }
      opj_thread_pool_wait_completion(tp, 0i32);
    }
    if rw & (8i32 - 1i32) as libc::c_uint != 0 {
      let mut k_2: OPJ_UINT32 = 0;
      j = rw & (8i32 - 1i32) as libc::c_uint;
      opj_v8dwt_interleave_v(&mut v, aj, w, j);
      opj_v8dwt_decode(&mut v);
      k_2 = 0 as OPJ_UINT32;
      while k_2 < rh {
        memcpy(
          &mut *aj.offset((k_2 as libc::c_ulong).wrapping_mul(w as OPJ_SIZE_T) as isize)
            as *mut OPJ_FLOAT32 as *mut libc::c_void,
          &mut *v.wavelet.offset(k_2 as isize) as *mut opj_v8_t as *const libc::c_void,
          (j as OPJ_SIZE_T).wrapping_mul(::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong),
        );
        k_2 = k_2.wrapping_add(1)
      }
    }
  }
  opj_aligned_free(h.wavelet as *mut libc::c_void);
  return 1i32;
}
unsafe fn opj_dwt_decode_partial_97(
  mut tilec: *mut opj_tcd_tilecomp_t,
  mut numres: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut sa = 0 as *mut opj_sparse_array_int32_t;
  let mut h = opj_v8dwt_t {
    wavelet: 0 as *mut opj_v8_t,
    dn: 0,
    sn: 0,
    cas: 0,
    win_l_x0: 0,
    win_l_x1: 0,
    win_h_x0: 0,
    win_h_x1: 0,
  };
  let mut v = opj_v8dwt_t {
    wavelet: 0 as *mut opj_v8_t,
    dn: 0,
    sn: 0,
    cas: 0,
    win_l_x0: 0,
    win_l_x1: 0,
    win_h_x0: 0,
    win_h_x1: 0,
  };
  let mut resno: OPJ_UINT32 = 0;
  /* This value matches the maximum left/right extension given in tables */
  /* F.2 and F.3 of the standard. Note: in opj_tcd_is_subband_area_of_interest() */
  /* we currently use 3. */
  let filter_width = 4u32; /* width of the resolution level computed */
  let mut tr = (*tilec).resolutions; /* height of the resolution level computed */
  let mut tr_max: *mut opj_tcd_resolution_t = &mut *(*tilec)
    .resolutions
    .offset(numres.wrapping_sub(1u32) as isize)
    as *mut opj_tcd_resolution_t;
  let mut rw = ((*tr).x1 - (*tr).x0) as OPJ_UINT32;
  let mut rh = ((*tr).y1 - (*tr).y0) as OPJ_UINT32;
  let mut l_data_size: OPJ_SIZE_T = 0;
  /* Compute the intersection of the area of interest, expressed in tile coordinates */
  /* with the tile coordinates */
  let mut win_tcx0 = (*tilec).win_x0;
  let mut win_tcy0 = (*tilec).win_y0;
  let mut win_tcx1 = (*tilec).win_x1;
  let mut win_tcy1 = (*tilec).win_y1;
  if (*tr_max).x0 == (*tr_max).x1 || (*tr_max).y0 == (*tr_max).y1 {
    return 1i32;
  }
  sa = opj_dwt_init_sparse_array(tilec, numres);
  if sa.is_null() {
    return 0i32;
  }
  if numres == 1u32 {
    let mut ret = opj_sparse_array_int32_read(
      sa,
      (*tr_max).win_x0.wrapping_sub((*tr_max).x0 as OPJ_UINT32),
      (*tr_max).win_y0.wrapping_sub((*tr_max).y0 as OPJ_UINT32),
      (*tr_max).win_x1.wrapping_sub((*tr_max).x0 as OPJ_UINT32),
      (*tr_max).win_y1.wrapping_sub((*tr_max).y0 as OPJ_UINT32),
      (*tilec).data_win,
      1 as OPJ_UINT32,
      (*tr_max).win_x1.wrapping_sub((*tr_max).win_x0),
      1i32,
    );
    assert!(ret != 0);
    opj_sparse_array_int32_free(sa);
    return 1i32;
  }
  l_data_size = opj_dwt_max_resolution(tr, numres) as OPJ_SIZE_T;
  /* overflow check */
  if l_data_size
    > (18446744073709551615u64)
      .wrapping_div(::std::mem::size_of::<opj_v8_t>() as libc::c_ulong)
  {
    /* FIXME event manager error callback */
    opj_sparse_array_int32_free(sa);
    return 0i32;
  }
  h.wavelet = opj_aligned_malloc(
    l_data_size.wrapping_mul(::std::mem::size_of::<opj_v8_t>() as libc::c_ulong),
  ) as *mut opj_v8_t;
  if h.wavelet.is_null() {
    /* FIXME event manager error callback */
    opj_sparse_array_int32_free(sa);
    return 0i32;
  }
  v.wavelet = h.wavelet;
  resno = 1 as OPJ_UINT32;
  while resno < numres {
    let mut j: OPJ_UINT32 = 0;
    /* Window of interest subband-based coordinates */
    let mut win_ll_x0: OPJ_UINT32 = 0;
    let mut win_ll_y0: OPJ_UINT32 = 0;
    let mut win_ll_x1: OPJ_UINT32 = 0;
    let mut win_ll_y1: OPJ_UINT32 = 0;
    let mut win_hl_x0: OPJ_UINT32 = 0;
    let mut win_hl_x1: OPJ_UINT32 = 0;
    let mut win_lh_y0: OPJ_UINT32 = 0;
    let mut win_lh_y1: OPJ_UINT32 = 0;
    /* Window of interest tile-resolution-based coordinates */
    let mut win_tr_x0: OPJ_UINT32 = 0;
    let mut win_tr_x1: OPJ_UINT32 = 0;
    let mut win_tr_y0: OPJ_UINT32 = 0;
    let mut win_tr_y1: OPJ_UINT32 = 0;
    /* Tile-resolution subband-based coordinates */
    let mut tr_ll_x0: OPJ_UINT32 = 0;
    let mut tr_ll_y0: OPJ_UINT32 = 0;
    let mut tr_hl_x0: OPJ_UINT32 = 0;
    let mut tr_lh_y0: OPJ_UINT32 = 0;
    tr = tr.offset(1);
    h.sn = rw as OPJ_INT32;
    v.sn = rh as OPJ_INT32;
    rw = ((*tr).x1 - (*tr).x0) as OPJ_UINT32;
    rh = ((*tr).y1 - (*tr).y0) as OPJ_UINT32;
    h.dn = rw.wrapping_sub(h.sn as OPJ_UINT32) as OPJ_INT32;
    h.cas = (*tr).x0 % 2i32;
    v.dn = rh.wrapping_sub(v.sn as OPJ_UINT32) as OPJ_INT32;
    v.cas = (*tr).y0 % 2i32;
    /* Get the subband coordinates for the window of interest */
    /* LL band */
    opj_dwt_get_band_coordinates(
      tilec,
      resno,
      0 as OPJ_UINT32,
      win_tcx0,
      win_tcy0,
      win_tcx1,
      win_tcy1,
      &mut win_ll_x0,
      &mut win_ll_y0,
      &mut win_ll_x1,
      &mut win_ll_y1,
    );
    /* HL band */
    opj_dwt_get_band_coordinates(
      tilec,
      resno,
      1 as OPJ_UINT32,
      win_tcx0,
      win_tcy0,
      win_tcx1,
      win_tcy1,
      &mut win_hl_x0,
      0 as *mut OPJ_UINT32,
      &mut win_hl_x1,
      0 as *mut OPJ_UINT32,
    );
    /* LH band */
    opj_dwt_get_band_coordinates(
      tilec,
      resno,
      2 as OPJ_UINT32,
      win_tcx0,
      win_tcy0,
      win_tcx1,
      win_tcy1,
      0 as *mut OPJ_UINT32,
      &mut win_lh_y0,
      0 as *mut OPJ_UINT32,
      &mut win_lh_y1,
    );
    /* Beware: band index for non-LL0 resolution are 0=HL, 1=LH and 2=HH */
    tr_ll_x0 = (*tr).bands[1 as usize].x0 as OPJ_UINT32;
    tr_ll_y0 = (*tr).bands[0 as usize].y0 as OPJ_UINT32;
    tr_hl_x0 = (*tr).bands[0 as usize].x0 as OPJ_UINT32;
    tr_lh_y0 = (*tr).bands[1 as usize].y0 as OPJ_UINT32;
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
      h.sn as OPJ_UINT32,
      &mut win_ll_x0,
      &mut win_ll_x1,
    );
    opj_dwt_segment_grow(
      filter_width,
      h.dn as OPJ_UINT32,
      &mut win_hl_x0,
      &mut win_hl_x1,
    );
    opj_dwt_segment_grow(
      filter_width,
      v.sn as OPJ_UINT32,
      &mut win_ll_y0,
      &mut win_ll_y1,
    );
    opj_dwt_segment_grow(
      filter_width,
      v.dn as OPJ_UINT32,
      &mut win_lh_y0,
      &mut win_lh_y1,
    );
    /* Compute the tile-resolution-based coordinates for the window of interest */
    if h.cas == 0i32 {
      win_tr_x0 = opj_uint_min(
        (2u32).wrapping_mul(win_ll_x0),
        (2u32)
          .wrapping_mul(win_hl_x0)
          .wrapping_add(1u32),
      );
      win_tr_x1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_ll_x1),
          (2u32)
            .wrapping_mul(win_hl_x1)
            .wrapping_add(1u32),
        ),
        rw,
      )
    } else {
      win_tr_x0 = opj_uint_min(
        (2u32).wrapping_mul(win_hl_x0),
        (2u32)
          .wrapping_mul(win_ll_x0)
          .wrapping_add(1u32),
      );
      win_tr_x1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_hl_x1),
          (2u32)
            .wrapping_mul(win_ll_x1)
            .wrapping_add(1u32),
        ),
        rw,
      )
    }
    if v.cas == 0i32 {
      win_tr_y0 = opj_uint_min(
        (2u32).wrapping_mul(win_ll_y0),
        (2u32)
          .wrapping_mul(win_lh_y0)
          .wrapping_add(1u32),
      );
      win_tr_y1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_ll_y1),
          (2u32)
            .wrapping_mul(win_lh_y1)
            .wrapping_add(1u32),
        ),
        rh,
      )
    } else {
      win_tr_y0 = opj_uint_min(
        (2u32).wrapping_mul(win_lh_y0),
        (2u32)
          .wrapping_mul(win_ll_y0)
          .wrapping_add(1u32),
      );
      win_tr_y1 = opj_uint_min(
        opj_uint_max(
          (2u32).wrapping_mul(win_lh_y1),
          (2u32)
            .wrapping_mul(win_ll_y1)
            .wrapping_add(1u32),
        ),
        rh,
      )
    }
    h.win_l_x0 = win_ll_x0;
    h.win_l_x1 = win_ll_x1;
    h.win_h_x0 = win_hl_x0;
    h.win_h_x1 = win_hl_x1;
    j = 0 as OPJ_UINT32;
    while j.wrapping_add((8i32 - 1i32) as libc::c_uint) < rh {
      if j.wrapping_add((8i32 - 1i32) as libc::c_uint) >= win_ll_y0
        && j < win_ll_y1
        || j.wrapping_add((8i32 - 1i32) as libc::c_uint)
          >= win_lh_y0.wrapping_add(v.sn as OPJ_UINT32)
          && j < win_lh_y1.wrapping_add(v.sn as OPJ_UINT32)
      {
        opj_v8dwt_interleave_partial_h(
          &mut h,
          sa,
          j,
          opj_uint_min(8 as OPJ_UINT32, rh.wrapping_sub(j)),
        );
        opj_v8dwt_decode(&mut h);
        if opj_sparse_array_int32_write(
          sa,
          win_tr_x0,
          j,
          win_tr_x1,
          j.wrapping_add(8u32),
          &mut *(*h.wavelet.offset(win_tr_x0 as isize))
            .f
            .as_mut_ptr()
            .offset(0) as *mut OPJ_FLOAT32 as *mut OPJ_INT32,
          8 as OPJ_UINT32,
          1 as OPJ_UINT32,
          1i32,
        ) == 0
        {
          /* FIXME event manager error callback */
          opj_sparse_array_int32_free(sa);
          opj_aligned_free(h.wavelet as *mut libc::c_void);
          return 0i32;
        }
      }
      j = (j as libc::c_uint).wrapping_add(8u32) as OPJ_UINT32
        as OPJ_UINT32
    }
    if j < rh
      && (j.wrapping_add((8i32 - 1i32) as libc::c_uint) >= win_ll_y0
        && j < win_ll_y1
        || j.wrapping_add((8i32 - 1i32) as libc::c_uint)
          >= win_lh_y0.wrapping_add(v.sn as OPJ_UINT32)
          && j < win_lh_y1.wrapping_add(v.sn as OPJ_UINT32))
    {
      opj_v8dwt_interleave_partial_h(&mut h, sa, j, rh.wrapping_sub(j));
      opj_v8dwt_decode(&mut h);
      if opj_sparse_array_int32_write(
        sa,
        win_tr_x0,
        j,
        win_tr_x1,
        rh,
        &mut *(*h.wavelet.offset(win_tr_x0 as isize))
          .f
          .as_mut_ptr()
          .offset(0) as *mut OPJ_FLOAT32 as *mut OPJ_INT32,
        8 as OPJ_UINT32,
        1 as OPJ_UINT32,
        1i32,
      ) == 0
      {
        /* FIXME event manager error callback */
        opj_sparse_array_int32_free(sa);
        opj_aligned_free(h.wavelet as *mut libc::c_void);
        return 0i32;
      }
    }
    v.win_l_x0 = win_ll_y0;
    v.win_l_x1 = win_ll_y1;
    v.win_h_x0 = win_lh_y0;
    v.win_h_x1 = win_lh_y1;
    j = win_tr_x0;
    while j < win_tr_x1 {
      let mut nb_elts = opj_uint_min(8 as OPJ_UINT32, win_tr_x1.wrapping_sub(j));
      opj_v8dwt_interleave_partial_v(&mut v, sa, j, nb_elts);
      opj_v8dwt_decode(&mut v);
      if opj_sparse_array_int32_write(
        sa,
        j,
        win_tr_y0,
        j.wrapping_add(nb_elts),
        win_tr_y1,
        &mut *(*h.wavelet.offset(win_tr_y0 as isize))
          .f
          .as_mut_ptr()
          .offset(0) as *mut OPJ_FLOAT32 as *mut OPJ_INT32,
        1 as OPJ_UINT32,
        8 as OPJ_UINT32,
        1i32,
      ) == 0
      {
        /* FIXME event manager error callback */
        opj_sparse_array_int32_free(sa);
        opj_aligned_free(h.wavelet as *mut libc::c_void);
        return 0i32;
      }
      j = (j as libc::c_uint).wrapping_add(8u32) as OPJ_UINT32
        as OPJ_UINT32
    }
    resno = resno.wrapping_add(1)
  }
  let mut ret_0 = opj_sparse_array_int32_read(
    sa,
    (*tr_max).win_x0.wrapping_sub((*tr_max).x0 as OPJ_UINT32),
    (*tr_max).win_y0.wrapping_sub((*tr_max).y0 as OPJ_UINT32),
    (*tr_max).win_x1.wrapping_sub((*tr_max).x0 as OPJ_UINT32),
    (*tr_max).win_y1.wrapping_sub((*tr_max).y0 as OPJ_UINT32),
    (*tilec).data_win,
    1 as OPJ_UINT32,
    (*tr_max).win_x1.wrapping_sub((*tr_max).win_x0),
    1i32,
  );
  assert!(ret_0 != 0);
  opj_sparse_array_int32_free(sa);
  opj_aligned_free(h.wavelet as *mut libc::c_void);
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_dwt_decode_real(
  mut p_tcd: *mut opj_tcd_t,
  mut tilec: *mut opj_tcd_tilecomp_t,
  mut numres: OPJ_UINT32,
) -> OPJ_BOOL {
  if (*p_tcd).whole_tile_decoding != 0 {
    return opj_dwt_decode_tile_97((*p_tcd).thread_pool, tilec, numres);
  } else {
    return opj_dwt_decode_partial_97(tilec, numres);
  };
}
