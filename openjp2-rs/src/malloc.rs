use super::openjpeg::*;

extern "C" {

  fn malloc(_: usize) -> *mut core::ffi::c_void;

  fn calloc(_: usize, _: usize) -> *mut core::ffi::c_void;

  fn realloc(_: *mut core::ffi::c_void, _: usize) -> *mut core::ffi::c_void;

  fn free(_: *mut core::ffi::c_void);

  fn memcpy(
    _: *mut core::ffi::c_void,
    _: *const core::ffi::c_void,
    _: usize,
  ) -> *mut core::ffi::c_void;
}
/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2015, Mathieu Malaterre <mathieu.malaterre@gmail.com>
 * Copyright (c) 2015, Matthieu Darbois
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
#[inline]
unsafe fn opj_aligned_alloc_n(mut alignment: size_t, mut size: size_t) -> *mut core::ffi::c_void {
  /* alignment shall be power of 2 */
  assert!(alignment != 0 && alignment & alignment.wrapping_sub(1) == 0);
  /* alignment shall be at least sizeof(void*) */
  assert!(alignment >= core::mem::size_of::<*mut core::ffi::c_void>());
  if size == 0 {
    /* prevent implementation defined behavior of realloc */
    return std::ptr::null_mut::<core::ffi::c_void>();
  }
  #[cfg(windows)]
  {
    libc::aligned_malloc(size, alignment)
  }
  #[cfg(not(windows))]
  {
    let mut ptr = std::ptr::null_mut::<core::ffi::c_void>();
    if libc::posix_memalign(&mut ptr, alignment, size) != 0 {
      ptr = std::ptr::null_mut::<core::ffi::c_void>()
    }
    ptr
  }
}
#[inline]
unsafe fn opj_aligned_realloc_n(
  mut ptr: *mut core::ffi::c_void,
  mut alignment: size_t,
  mut new_size: size_t,
) -> *mut core::ffi::c_void {
  let mut r_ptr = std::ptr::null_mut::<core::ffi::c_void>();
  /* alignment shall be power of 2 */

  /* alignment shall be at least sizeof(void*) */
  assert!(alignment != 0 && alignment & alignment.wrapping_sub(1) == 0);
  assert!(alignment >= core::mem::size_of::<*mut core::ffi::c_void>());
  if new_size == 0 {
    /* prevent implementation defined behavior of realloc */
    return std::ptr::null_mut::<core::ffi::c_void>();
  }
  /* no portable aligned realloc */
  /* glibc doc states one can mix aligned malloc with realloc */
  r_ptr = realloc(ptr, new_size); /* fast path */
  /* we simply use `size_t` to cast, since we are only interest in binary AND
   * operator */
  if r_ptr as size_t & alignment.wrapping_sub(1) != 0 {
    /* this is non-trivial to implement a portable aligned realloc, so use a
     * simple approach where we do not need a function that return the size of an
     * allocated array (eg. _msize on Windows, malloc_size on MacOS,
     * malloc_usable_size on systems with glibc) */
    let mut a_ptr = opj_aligned_alloc_n(alignment, new_size);
    if !a_ptr.is_null() {
      memcpy(a_ptr, r_ptr, new_size);
    }
    free(r_ptr);
    r_ptr = a_ptr
  }
  /* _MSC_VER */
  r_ptr
}
#[no_mangle]
pub(crate) unsafe fn opj_malloc(mut size: size_t) -> *mut core::ffi::c_void {
  if size == 0 {
    /* prevent implementation defined behavior of realloc */
    return std::ptr::null_mut::<core::ffi::c_void>();
  }
  malloc(size)
}
#[no_mangle]
pub(crate) unsafe fn opj_calloc(mut num: size_t, mut size: size_t) -> *mut core::ffi::c_void {
  if num == 0 || size == 0 {
    /* prevent implementation defined behavior of realloc */
    return std::ptr::null_mut::<core::ffi::c_void>();
  }
  calloc(num, size)
}
#[no_mangle]
pub(crate) unsafe fn opj_aligned_malloc(mut size: size_t) -> *mut core::ffi::c_void {
  opj_aligned_alloc_n(16u32 as size_t, size)
}
#[no_mangle]
pub(crate) unsafe fn opj_aligned_realloc(
  mut ptr: *mut core::ffi::c_void,
  mut size: size_t,
) -> *mut core::ffi::c_void {
  opj_aligned_realloc_n(ptr, 16u32 as size_t, size)
}
#[no_mangle]
pub(crate) unsafe fn opj_aligned_32_malloc(mut size: size_t) -> *mut core::ffi::c_void {
  opj_aligned_alloc_n(32u32 as size_t, size)
}
#[no_mangle]
pub(crate) unsafe fn opj_aligned_32_realloc(
  mut ptr: *mut core::ffi::c_void,
  mut size: size_t,
) -> *mut core::ffi::c_void {
  opj_aligned_realloc_n(ptr, 32u32 as size_t, size)
}
#[no_mangle]
pub(crate) unsafe fn opj_aligned_free(mut ptr: *mut core::ffi::c_void) {
  free(ptr);
}
#[no_mangle]
pub(crate) unsafe fn opj_realloc(
  mut ptr: *mut core::ffi::c_void,
  mut new_size: size_t,
) -> *mut core::ffi::c_void {
  if new_size == 0 {
    /* prevent implementation defined behavior of realloc */
    return std::ptr::null_mut::<core::ffi::c_void>();
  }
  realloc(ptr, new_size)
}
#[no_mangle]
pub(crate) unsafe fn opj_free(mut ptr: *mut core::ffi::c_void) {
  free(ptr);
}
