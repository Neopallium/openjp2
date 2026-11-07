/// A trait for writing formatted strings without requiring std
pub trait StringWriter {
  fn write_string(&mut self, s: &str) -> usize;
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
