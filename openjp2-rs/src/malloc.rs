#[cfg(not(any(feature = "c_api", feature = "test_malloc")))]
use alloc::alloc::{alloc, alloc_zeroed, dealloc, Layout};

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

pub(crate) fn opj_malloc(mut size: usize) -> *mut core::ffi::c_void {
  if size == 0 {
    /* prevent implementation defined behavior of malloc */
    return core::ptr::null_mut::<core::ffi::c_void>();
  }
  unsafe { malloc(size) }
}

#[cfg(feature = "test_malloc")]
#[repr(C)]
pub struct TestAllocHeader {
  magic: u32,
  size: usize,
}
#[cfg(feature = "test_malloc")]
pub const TEST_ALLOC_MAGIC: u32 = 0xDEADBEEF;

#[cfg(feature = "test_malloc")]
impl TestAllocHeader {
  fn size() -> usize {
    core::mem::size_of::<TestAllocHeader>()
  }

  fn write_head<T>(ptr: *mut core::ffi::c_void, size: usize) -> *mut T {
    if ptr.is_null() {
      return core::ptr::null_mut::<T>();
    }
    let header = ptr as *mut TestAllocHeader;
    unsafe {
      (*header).magic = TEST_ALLOC_MAGIC;
      (*header).size = size;
    }
    (ptr as *mut u8).wrapping_add(Self::size()) as *mut T
  }

  fn get_head<T>(ptr: *mut T) -> *mut TestAllocHeader {
    if ptr.is_null() {
      return core::ptr::null_mut::<TestAllocHeader>();
    }
    let header_ptr = (ptr as *mut u8).wrapping_sub(Self::size());
    let header = header_ptr as *mut TestAllocHeader;
    unsafe {
      if !(*header).is_valid() {
        panic!("Invalid magic number in TestAllocHeader");
      }
    }
    header
  }

  fn malloc<T>(size: usize) -> *mut T {
    let total_size = size + Self::size();
    Self::write_head(opj_malloc(total_size), size)
  }

  fn calloc<T>(num: usize, size: usize) -> *mut T {
    let total_size = num
      .checked_mul(size)
      .and_then(|s| s.checked_add(Self::size()))
      .unwrap_or(0);
    if total_size == 0 {
      return core::ptr::null_mut::<T>();
    }
    Self::write_head(opj_calloc(1, total_size), num * size)
  }

  fn realloc<T>(ptr: *mut T, new_size: usize) -> *mut T {
    if ptr.is_null() {
      return Self::malloc(new_size);
    }
    let header = Self::get_head(ptr);
    let total_size = new_size + Self::size();
    Self::write_head(
      opj_realloc(header as *mut core::ffi::c_void, total_size),
      new_size,
    )
  }

  fn free<T>(ptr: *mut T) {
    if ptr.is_null() {
      return;
    }
    let header = Self::get_head(ptr);
    unsafe {
      (*header).magic = 0; // Invalidate the magic number
      (*header).size = 0;
      opj_free(header as *mut core::ffi::c_void);
    }
  }

  fn is_valid(&self) -> bool {
    self.magic == TEST_ALLOC_MAGIC
  }
}

pub(crate) fn opj_alloc_type<T>() -> *mut T {
  let size = core::mem::size_of::<T>();
  if size == 0 {
    return core::ptr::null_mut::<T>();
  }
  #[cfg(feature = "c_api")]
  {
    opj_malloc(size) as *mut T
  }
  #[cfg(not(feature = "c_api"))]
  {
    #[cfg(feature = "test_malloc")]
    {
      TestAllocHeader::malloc::<T>(size)
    }

    #[cfg(not(feature = "test_malloc"))]
    {
      let layout = Layout::new::<T>();
      unsafe { alloc(layout) as *mut T }
    }
  }
}

pub(crate) fn opj_calloc(mut num: usize, mut size: usize) -> *mut core::ffi::c_void {
  if num == 0 || size == 0 {
    /* prevent implementation defined behavior of calloc */
    return core::ptr::null_mut::<core::ffi::c_void>();
  }
  unsafe { calloc(num, size) }
}

pub(crate) fn opj_calloc_type<T>() -> *mut T {
  let size = core::mem::size_of::<T>();
  if size == 0 {
    /* prevent implementation defined behavior of calloc */
    return core::ptr::null_mut::<T>();
  }
  #[cfg(feature = "c_api")]
  {
    opj_calloc(1, size) as *mut T
  }
  #[cfg(not(feature = "c_api"))]
  {
    #[cfg(feature = "test_malloc")]
    {
      TestAllocHeader::calloc::<T>(1, size)
    }

    #[cfg(not(feature = "test_malloc"))]
    {
      let layout = Layout::new::<T>();
      unsafe { alloc_zeroed(layout) as *mut T }
    }
  }
}

pub(crate) fn opj_alloc_type_array<T>(num: usize) -> *mut T {
  let size = core::mem::size_of::<T>();
  if num == 0 || size == 0 {
    return core::ptr::null_mut::<T>();
  }
  #[cfg(feature = "c_api")]
  {
    opj_malloc(num * size) as *mut T
  }
  #[cfg(not(feature = "c_api"))]
  {
    #[cfg(feature = "test_malloc")]
    {
      TestAllocHeader::malloc::<T>(num * size)
    }

    #[cfg(not(feature = "test_malloc"))]
    {
      let layout = Layout::array::<T>(num).expect("Failed to create layout for array");
      unsafe { alloc(layout) as *mut T }
    }
  }
}

pub(crate) fn opj_calloc_type_array<T>(num: usize) -> *mut T {
  let size = core::mem::size_of::<T>();
  if num == 0 || size == 0 {
    /* prevent implementation defined behavior of calloc */
    return core::ptr::null_mut::<T>();
  }
  #[cfg(feature = "c_api")]
  {
    opj_calloc(num, size) as *mut T
  }
  #[cfg(not(feature = "c_api"))]
  {
    #[cfg(feature = "test_malloc")]
    {
      TestAllocHeader::calloc::<T>(num, size)
    }

    #[cfg(not(feature = "test_malloc"))]
    {
      let layout = Layout::array::<T>(num).expect("Failed to create layout for array");
      unsafe { alloc_zeroed(layout) as *mut T }
    }
  }
}

pub(crate) fn opj_realloc(
  mut ptr: *mut core::ffi::c_void,
  mut new_size: usize,
) -> *mut core::ffi::c_void {
  if new_size == 0 {
    opj_free(ptr);
    /* prevent implementation defined behavior of realloc */
    return core::ptr::null_mut::<core::ffi::c_void>();
  }
  unsafe { realloc(ptr, new_size) }
}

pub(crate) fn opj_realloc_type_array<T>(mut ptr: *mut T, old_num: usize, mut num: usize) -> *mut T {
  let size = core::mem::size_of::<T>();
  if num == 0 || size == 0 {
    opj_free_type_array(ptr, old_num);
    /* prevent implementation defined behavior of realloc */
    return core::ptr::null_mut::<T>();
  }
  let new_size = num * size;
  #[cfg(feature = "c_api")]
  {
    opj_realloc(ptr as *mut core::ffi::c_void, new_size) as *mut T
  }
  #[cfg(not(feature = "c_api"))]
  {
    #[cfg(feature = "test_malloc")]
    {
      TestAllocHeader::realloc::<T>(ptr, new_size)
    }

    #[cfg(not(feature = "test_malloc"))]
    {
      if old_num != 0 {
        let old_size = old_num * size;
        let layout = Layout::array::<T>(old_size).expect("Failed to create layout for array");
        unsafe { alloc::alloc::realloc(ptr as *mut u8, layout, new_size) as *mut T }
      } else {
        let layout = Layout::array::<T>(new_size).expect("Failed to create layout for array");
        unsafe { alloc(layout) as *mut T }
      }
    }
  }
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
    #[cfg(feature = "c_api")]
    {
      opj_free(ptr as *mut core::ffi::c_void);
    }
    #[cfg(not(feature = "c_api"))]
    {
      #[cfg(feature = "test_malloc")]
      {
        TestAllocHeader::free(ptr);
      }

      #[cfg(not(feature = "test_malloc"))]
      {
        let layout = Layout::new::<T>();
        unsafe { dealloc(ptr as *mut u8, layout) }
      }
    }
  }
}

pub(crate) fn opj_free_type_array<T>(mut ptr: *mut T, _num: usize) {
  if !ptr.is_null() {
    #[cfg(feature = "c_api")]
    {
      opj_free(ptr as *mut core::ffi::c_void);
    }
    #[cfg(not(feature = "c_api"))]
    {
      #[cfg(feature = "test_malloc")]
      {
        TestAllocHeader::free(ptr);
      }

      #[cfg(not(feature = "test_malloc"))]
      {
        let layout = Layout::array::<T>(_num).expect("Failed to create layout for array");
        unsafe { dealloc(ptr as *mut u8, layout) }
      }
    }
  }
}
