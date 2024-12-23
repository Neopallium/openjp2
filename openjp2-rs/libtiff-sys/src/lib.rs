#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
  use super::*;
  use std::ffi::CString;

  #[test]
  fn test_tiff_functions() {
    unsafe {
      // Test TIFFOpen
      let filename = CString::new("nonexistent.tiff").unwrap();
      let mode = CString::new("r").unwrap();
      let tiff = TIFFOpen(filename.as_ptr(), mode.as_ptr());
      assert!(tiff.is_null());

      // Test TIFFGetVersion
      let version = TIFFGetVersion();
      assert!(!version.is_null());

      // Test a few more function pointers exist
      assert!(TIFFClose as usize != 0);
      assert!(TIFFReadScanline as usize != 0);
      assert!(TIFFWriteScanline as usize != 0);
    }
  }
}
