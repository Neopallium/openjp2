#[inline]
pub(crate) fn opj_int_max(mut a: i32, mut b: i32) -> i32 {
  a.max(b)
}

#[inline]
pub(crate) fn opj_int_min(mut a: i32, mut b: i32) -> i32 {
  a.min(b)
}

#[inline]
pub(crate) fn opj_uint_min(mut a: u32, mut b: u32) -> u32 {
  a.min(b)
}

#[inline]
pub(crate) fn opj_uint_max(mut a: u32, mut b: u32) -> u32 {
  a.max(b)
}

#[inline]
pub(crate) fn opj_int_floordivpow2(mut a: i32, mut b: i32) -> i32 {
  return a >> b;
}

#[inline]
pub(crate) fn opj_int_floorlog2(mut a: i32) -> i32 {
  let mut l: i32 = 0;
  l = 0;
  while a > 1 {
    a >>= 1;
    l += 1
  }
  return l;
}

#[inline]
pub(crate) fn opj_uint_floordivpow2(mut a: u32, mut b: u32) -> u32 {
  return a >> b;
}

#[inline]
pub(crate) fn opj_uint_floorlog2(mut a: u32) -> u32 {
  let mut l: u32 = 0;
  l = 0 as u32;
  while a > 1 {
    a >>= 1;
    l += 1;
  }
  return l;
}

#[inline]
pub(crate) fn opj_uint_ceildivpow2(mut a: u32, mut b: u32) -> u32 {
  return ((a as u64)
    .wrapping_add((1 as u64) << b)
    .wrapping_sub(1 as u64)
    >> b) as u32;
}

#[inline]
pub(crate) fn opj_uint_ceildiv(mut a: u32, mut b: u32) -> u32 {
  assert!(b != 0);
  return (a as u64)
    .wrapping_add(b as u64)
    .wrapping_sub(1 as u64)
    .wrapping_div(b as u64) as u32;
}

#[inline]
pub(crate) fn opj_int64_ceildivpow2(mut a: i64, mut b: i32) -> i32 {
  return (a + ((1 as i64) << b) - 1 as i64 >> b)
    as i32;
}

#[inline]
pub(crate) fn opj_int_ceildiv(mut a: i32, mut b: i32) -> i32 {
  assert!(b != 0);
  return ((a as i64 + b as i64 - 1 as i64)
    / b as i64) as i32;
}

#[inline]
pub(crate) fn opj_int_ceildivpow2(mut a: i32, mut b: i32) -> i32 {
  return (a as i64 + ((1 as i64) << b)
    - 1 as i64
    >> b) as i32;
}

#[inline]
pub(crate) fn opj_uint_adds(mut a: u32, mut b: u32) -> u32 {
  a.saturating_add(b)
}

#[inline]
pub(crate) fn opj_int_abs(mut a: i32) -> i32 {
  a.abs()
}

#[inline]
pub(crate) fn opj_uint_subs(mut a: u32, mut b: u32) -> u32 {
  a.saturating_sub(b)
}

#[inline]
pub(crate) fn opj_lrintf(mut f: f32) -> i64 {
  // NOTE: `f32::round()` in Rust doesn't produce the same results:
  // C lrintf:
  // (39.5000) = 40.0
  // (38.5000) = 38.0
  // Rust `round`:
  // (39.5000) = 40.0
  // (38.5000) = 39.0
  extern "C" {
    fn lrintf(_: ::core::ffi::c_float) -> ::core::ffi::c_long;
  }

  unsafe {
    lrintf(f as f32) as i64
  }
}

#[inline]
pub(crate) fn opj_int64_clamp(mut a: i64, mut min: i64, mut max: i64) -> i64 {
  a.clamp(min, max)
}

#[inline]
pub(crate) fn opj_int_clamp(mut a: i32, mut min: i32, mut max: i32) -> i32 {
  a.clamp(min, max)
}

#[inline]
pub(crate) fn opj_int_sub_no_overflow(mut a: i32, mut b: i32) -> i32 {
  a.wrapping_sub(b)
}

#[inline]
pub(crate) fn opj_int_add_no_overflow(mut a: i32, mut b: i32) -> i32 {
  a.wrapping_add(b)
}
