/// A trait for writing formatted strings without requiring std
pub trait StringWriter {
  fn write_string(&mut self, s: &str) -> usize;
}

#[cfg(feature = "file-io")]
impl StringWriter for ::libc::FILE {
  fn write_string(&mut self, s: &str) -> usize {
    unsafe {
      let bytes = s.as_bytes();
      let len = bytes.len();
      let nb = libc::fwrite(bytes.as_ptr() as *const ::libc::c_void, 1, len, self);
      if nb == ::libc::size_t::MAX {
        0
      } else {
        s.len()
      }
    }
  }
}

#[cfg(feature = "file-io")]
impl StringWriter for std::fs::File {
  fn write_string(&mut self, s: &str) -> usize {
    use std::io::Write;
    match self.write(s.as_bytes()) {
      Ok(n) => n,
      Err(_) => 0,
    }
  }
}

#[cfg(feature = "file-io")]
impl StringWriter for std::io::Stdout {
  fn write_string(&mut self, s: &str) -> usize {
    use std::io::Write;
    match self.write(s.as_bytes()) {
      Ok(n) => n,
      Err(_) => 0,
    }
  }
}

#[cfg(feature = "file-io")]
impl StringWriter for std::io::Stderr {
  fn write_string(&mut self, s: &str) -> usize {
    use std::io::Write;
    match self.write(s.as_bytes()) {
      Ok(n) => n,
      Err(_) => 0,
    }
  }
}

macro_rules! fprintf {
  ($writer:expr, $($fmt:expr),* $(,)?) => {
    {
      let s = format!($($fmt),*);
      $writer.write_string(&s)
    }
  };
}
