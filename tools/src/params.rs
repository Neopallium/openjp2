use std::io;
use std::path::{Path, PathBuf};

mod compress;
pub use compress::*;
mod decompress;
pub use decompress::*;

#[derive(Debug)]
pub enum ParameterError {
  InvalidFormat(String),
  InvalidValue(String),
  ParseError(String),
}

impl std::fmt::Display for ParameterError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
      Self::InvalidValue(s) => write!(f, "Invalid value: {}", s),
      Self::ParseError(s) => write!(f, "Parse error: {}", s),
    }
  }
}

impl std::error::Error for ParameterError {}

// Add Directory handling
pub struct DirContents {
  pub files: Vec<PathBuf>,
}

impl DirContents {
  pub fn new(dir_path: &Path) -> io::Result<Self> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir_path)? {
      let entry = entry?;
      let path = entry.path();
      if path.is_file() {
        files.push(path);
      }
    }
    Ok(Self { files })
  }
}
