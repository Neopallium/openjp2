use super::openjpeg::*;

extern "C" {

  fn malloc(_: usize) -> *mut core::ffi::c_void;

  fn calloc(_: usize, _: usize) -> *mut core::ffi::c_void;

  fn realloc(_: *mut core::ffi::c_void, _: usize) -> *mut core::ffi::c_void;

  fn free(_: *mut core::ffi::c_void);
}

pub(crate) fn memcpy(
  dest: *mut core::ffi::c_void,
  src: *const core::ffi::c_void,
  n: usize,
) -> *mut core::ffi::c_void {
  unsafe {
    core::ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, n);
  }
  dest
}

pub(crate) fn memmove(
  dest: *mut core::ffi::c_void,
  src: *const core::ffi::c_void,
  n: usize,
) -> *mut core::ffi::c_void {
  unsafe {
    core::ptr::copy(src as *const u8, dest as *mut u8, n);
  }
  dest
}

pub(crate) fn memset(s: *mut core::ffi::c_void, c: i32, n: usize) -> *mut core::ffi::c_void {
  unsafe {
    core::ptr::write_bytes(s as *mut u8, c as u8, n);
  }
  s
}

pub(crate) fn strlen(s: *const i8) -> usize {
  unsafe {
    let mut len = 0;
    while *s.add(len) != 0 {
      len += 1;
    }
    len
  }
}

pub(crate) fn opj_malloc(mut size: size_t) -> *mut core::ffi::c_void {
  if size == 0 {
    /* prevent implementation defined behavior of malloc */
    return core::ptr::null_mut::<core::ffi::c_void>();
  }
  unsafe { malloc(size) }
}

pub(crate) fn opj_alloc_type<T>() -> *mut T {
  let size = core::mem::size_of::<T>();
  if size == 0 {
    return core::ptr::null_mut::<T>();
  }
  opj_malloc(size) as *mut T
}

pub(crate) fn opj_calloc(mut num: size_t, mut size: size_t) -> *mut core::ffi::c_void {
  if num == 0 || size == 0 {
    /* prevent implementation defined behavior of calloc */
    return core::ptr::null_mut::<core::ffi::c_void>();
  }
  unsafe { calloc(num, size) }
}

pub(crate) fn opj_calloc_type<T>(num: size_t) -> *mut T {
  let size = core::mem::size_of::<T>();
  if num == 0 || size == 0 {
    /* prevent implementation defined behavior of calloc */
    return core::ptr::null_mut::<T>();
  }
  opj_calloc(num, size) as *mut T
}

pub(crate) fn opj_alloc_type_array<T>(num: size_t) -> *mut T {
  let size = core::mem::size_of::<T>();
  if num == 0 || size == 0 {
    return core::ptr::null_mut::<T>();
  }
  opj_malloc(size) as *mut T
}

pub(crate) fn opj_calloc_type_array<T>(num: size_t) -> *mut T {
  opj_calloc_type(num)
}

pub(crate) fn opj_realloc(
  mut ptr: *mut core::ffi::c_void,
  mut new_size: size_t,
) -> *mut core::ffi::c_void {
  if new_size == 0 {
    opj_free(ptr);
    /* prevent implementation defined behavior of realloc */
    return core::ptr::null_mut::<core::ffi::c_void>();
  }
  unsafe { realloc(ptr, new_size) }
}

pub(crate) fn opj_free(mut ptr: *mut core::ffi::c_void) {
  unsafe {
    if !ptr.is_null() {
      free(ptr);
    }
  }
}

pub(crate) fn opj_free_type<T>(mut ptr: *mut T) {
  if !ptr.is_null() {
    opj_free(ptr as *mut core::ffi::c_void);
  }
}

pub(crate) fn opj_free_type_array<T>(mut ptr: *mut T, _num: size_t) {
  if !ptr.is_null() {
    opj_free(ptr as *mut core::ffi::c_void);
  }
}
