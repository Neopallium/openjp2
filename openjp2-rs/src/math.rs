use super::openjpeg::*;
use ::libc;

extern "C" {
  fn lrintf(_: libc::c_float) -> libc::c_long;
}

#[inline]
pub fn opj_int_max(mut a: OPJ_INT32, mut b: OPJ_INT32) -> OPJ_INT32 {
  return if a > b { a } else { b };
}
#[inline]
pub fn opj_int_min(mut a: OPJ_INT32, mut b: OPJ_INT32) -> OPJ_INT32 {
  return if a < b { a } else { b };
}
#[inline]
pub fn opj_uint_min(mut a: OPJ_UINT32, mut b: OPJ_UINT32) -> OPJ_UINT32 {
  return if a < b { a } else { b };
}
#[inline]
pub fn opj_uint_max(mut a: OPJ_UINT32, mut b: OPJ_UINT32) -> OPJ_UINT32 {
  return if a > b { a } else { b };
}

#[inline]
pub fn opj_int_floordivpow2(mut a: OPJ_INT32, mut b: OPJ_INT32) -> OPJ_INT32 {
  return a >> b;
}
#[inline]
pub fn opj_int_floorlog2(mut a: OPJ_INT32) -> OPJ_INT32 {
  let mut l: OPJ_INT32 = 0;
  l = 0 as libc::c_int;
  while a > 1 as libc::c_int {
    a >>= 1 as libc::c_int;
    l += 1
  }
  return l;
}
#[inline]
pub fn opj_uint_floordivpow2(mut a: OPJ_UINT32, mut b: OPJ_UINT32) -> OPJ_UINT32 {
  return a >> b;
}
#[inline]
pub fn opj_uint_floorlog2(mut a: OPJ_UINT32) -> OPJ_UINT32 {
  let mut l: OPJ_UINT32 = 0;
  l = 0 as libc::c_int as OPJ_UINT32;
  while a > 1 as libc::c_int as libc::c_uint {
    a >>= 1 as libc::c_int;
    l = l.wrapping_add(1)
  }
  return l;
}

#[inline]
pub fn opj_uint_ceildivpow2(mut a: OPJ_UINT32, mut b: OPJ_UINT32) -> OPJ_UINT32 {
  return ((a as libc::c_ulong)
    .wrapping_add((1 as libc::c_uint as OPJ_UINT64) << b)
    .wrapping_sub(1 as libc::c_uint as libc::c_ulong)
    >> b) as OPJ_UINT32;
}
#[inline]
pub fn opj_uint_ceildiv(mut a: OPJ_UINT32, mut b: OPJ_UINT32) -> OPJ_UINT32 {
  assert!(b != 0);
  return (a as OPJ_UINT64)
    .wrapping_add(b as libc::c_ulong)
    .wrapping_sub(1 as libc::c_int as libc::c_ulong)
    .wrapping_div(b as libc::c_ulong) as OPJ_UINT32;
}
#[inline]
pub fn opj_int64_ceildivpow2(mut a: OPJ_INT64, mut b: OPJ_INT32) -> OPJ_INT32 {
  return (a + ((1 as libc::c_int as OPJ_INT64) << b) - 1 as libc::c_int as libc::c_long >> b)
    as OPJ_INT32;
}
#[inline]
pub fn opj_int_ceildiv(mut a: OPJ_INT32, mut b: OPJ_INT32) -> OPJ_INT32 {
  assert!(b != 0);
  return ((a as OPJ_INT64 + b as libc::c_long - 1 as libc::c_int as libc::c_long)
    / b as libc::c_long) as OPJ_INT32;
}

#[inline]
pub fn opj_int_ceildivpow2(mut a: OPJ_INT32, mut b: OPJ_INT32) -> OPJ_INT32 {
  return (a as libc::c_long + ((1 as libc::c_int as OPJ_INT64) << b)
    - 1 as libc::c_int as libc::c_long
    >> b) as OPJ_INT32;
}

#[inline]
pub fn opj_uint_adds(mut a: OPJ_UINT32, mut b: OPJ_UINT32) -> OPJ_UINT32 {
  let mut sum = (a as OPJ_UINT64).wrapping_add(b as OPJ_UINT64);
  return -((sum >> 32 as libc::c_int) as OPJ_INT32) as OPJ_UINT32 | sum as OPJ_UINT32;
}
#[inline]
pub fn opj_int_abs(mut a: OPJ_INT32) -> OPJ_INT32 {
  return if a < 0 as libc::c_int { -a } else { a };
}
#[inline]
pub fn opj_uint_subs(mut a: OPJ_UINT32, mut b: OPJ_UINT32) -> OPJ_UINT32 {
  return if a >= b {
    a.wrapping_sub(b)
  } else {
    0 as libc::c_int as libc::c_uint
  };
}

#[inline]
pub unsafe fn opj_lrintf(mut f: libc::c_float) -> libc::c_long {
  return lrintf(f);
}

#[inline]
pub fn opj_int64_clamp(mut a: OPJ_INT64, mut min: OPJ_INT64, mut max: OPJ_INT64) -> OPJ_INT64 {
  if a < min {
    return min;
  }
  if a > max {
    return max;
  }
  return a;
}
#[inline]
pub fn opj_int_clamp(mut a: OPJ_INT32, mut min: OPJ_INT32, mut max: OPJ_INT32) -> OPJ_INT32 {
  if a < min {
    return min;
  }
  if a > max {
    return max;
  }
  return a;
}

#[inline]
pub unsafe fn opj_int_sub_no_overflow(mut a: OPJ_INT32, mut b: OPJ_INT32) -> OPJ_INT32 {
  let mut pa = &mut a as *mut OPJ_INT32 as *mut libc::c_void;
  let mut pb = &mut b as *mut OPJ_INT32 as *mut libc::c_void;
  let mut upa = pa as *mut OPJ_UINT32;
  let mut upb = pb as *mut OPJ_UINT32;
  let mut ures = (*upa).wrapping_sub(*upb);
  let mut pures = &mut ures as *mut OPJ_UINT32 as *mut libc::c_void;
  let mut ipres = pures as *mut OPJ_INT32;
  return *ipres;
}
#[inline]
pub unsafe fn opj_int_add_no_overflow(mut a: OPJ_INT32, mut b: OPJ_INT32) -> OPJ_INT32 {
  let mut pa = &mut a as *mut OPJ_INT32 as *mut libc::c_void;
  let mut pb = &mut b as *mut OPJ_INT32 as *mut libc::c_void;
  let mut upa = pa as *mut OPJ_UINT32;
  let mut upb = pb as *mut OPJ_UINT32;
  let mut ures = (*upa).wrapping_add(*upb);
  let mut pures = &mut ures as *mut OPJ_UINT32 as *mut libc::c_void;
  let mut ipres = pures as *mut OPJ_INT32;
  return *ipres;
}
