/// A trait for writing formatted strings without requiring std
pub trait StringWriter {
  /// Writes the formatted string, returning the number of bytes written.
  fn write_string(&mut self, s: &str) -> usize;
}

fn write_all(writer: &mut impl std::io::Write, mut buf: &[u8]) -> usize {
  /* `Write::write` may write partially; loop like the C's fwrite-until-done
  (open-findings OF-28: the single write truncated large dumps). */
  let total = buf.len();
  while !buf.is_empty() {
    match writer.write(buf) {
      Ok(0) => break, /* would loop forever; treat as an error */
      Ok(n) => buf = &buf[n..],
      Err(_) => break,
    }
  }
  total - buf.len()
}

impl StringWriter for std::fs::File {
  fn write_string(&mut self, s: &str) -> usize {
    write_all(self, s.as_bytes())
  }
}

impl StringWriter for std::io::Stdout {
  fn write_string(&mut self, s: &str) -> usize {
    write_all(self, s.as_bytes())
  }
}

impl StringWriter for String {
  fn write_string(&mut self, s: &str) -> usize {
    self.push_str(s);
    s.len()
  }
}

impl StringWriter for std::io::Stderr {
  fn write_string(&mut self, s: &str) -> usize {
    write_all(self, s.as_bytes())
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
