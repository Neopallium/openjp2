macro_rules! fprintf {
  ($file:expr, $($fmt:expr),* $(,)?) => {
    {
      let s = format!($($fmt),*);
      let bytes = s.as_bytes();
      let len = bytes.len();
      let nb = libc::fwrite(bytes.as_ptr() as *const libc::c_void, 1, len, $file);
      nb
    }
  };
}
