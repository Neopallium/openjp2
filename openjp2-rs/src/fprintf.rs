/// A trait for writing formatted strings without requiring std
pub trait StringWriter {
  fn write_string(&mut self, s: &str) -> usize;
}

#[cfg(feature = "file-io")]
impl StringWriter for std::fs::File {
  fn write_string(&mut self, s: &str) -> usize {
    use std::io::Write;
    self.write(s.as_bytes()).unwrap_or_default()
  }
}

#[cfg(feature = "file-io")]
impl StringWriter for std::io::Stdout {
  fn write_string(&mut self, s: &str) -> usize {
    use std::io::Write;
    self.write(s.as_bytes()).unwrap_or_default()
  }
}

#[cfg(feature = "file-io")]
impl StringWriter for std::io::Stderr {
  fn write_string(&mut self, s: &str) -> usize {
    use std::io::Write;
    self.write(s.as_bytes()).unwrap_or_default()
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
