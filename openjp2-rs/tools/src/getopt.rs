//! A simple command line argument parser inspired by the classic getopt interface.
//!
//! This module provides functionality to parse command line arguments in a way that's
//! familiar to users of the traditional Unix getopt library, while providing a more
//! Rust-friendly interface.
//!
//! # Examples
//!
//! Basic usage with short options:
//! ```rust
//! use crate::getopt::{GetOpts, OptionDef, ParsedOpt};
//!
//! // Define options: -v (verbose), -o <file> (output file)
//! let opts = vec![
//!     OptionDef::short('v', false),     // no argument
//!     OptionDef::short('o', true),      // requires argument
//! ];
//!
//! let parser = GetOpts::new(&opts);
//! let args = vec!["myapp", "-v", "-o", "output.txt"];
//!
//! for opt in parser.parse_args(args) {
//!     match opt {
//!         ParsedOpt::Program(name) => println!("Program: {}", name),
//!         ParsedOpt::Opt('v', None) => println!("Verbose mode enabled"),
//!         ParsedOpt::Opt('o', Some(file)) => println!("Output file: {}", file),
//!         ParsedOpt::InvalidOpt(opt) => println!("Invalid option: {}", opt),
//!         _ => {}
//!     }
//! }
//! ```
//!
//! Using both short and long options:
//! ```rust
//! use crate::getopt::{GetOpts, OptionDef};
//!
//! let opts = vec![
//!     OptionDef::both('h', "help", false),
//!     OptionDef::both('o', "output", true),
//!     OptionDef::long("verbose", 'v', false),
//! ];
//!
//! // Will match both "-h" and "--help"
//! // Will match both "-o file.txt" and "--output file.txt"
//! // Will match "--verbose"
//! ```
//!
//! # Error Handling
//!
//! Invalid options are returned as `ParsedOpt::InvalidOpt` variants:
//! - Unknown options
//! - Missing required arguments
//!
//! ```rust
//! use crate::getopt::{GetOpts, OptionDef, ParsedOpt};
//!
//! let opts = vec![OptionDef::short('a', true)];
//! let parser = GetOpts::new(&opts);
//!
//! // Missing argument for -a
//! let args = vec!["prog", "-a"];
//! let parsed: Vec<_> = parser.parse_args(args).collect();
//! assert!(matches!(parsed[1], ParsedOpt::InvalidOpt(_)));
//! ```

use std::collections::HashMap;

/// Represents a command line option definition that can have a short form (-h),
/// long form (--help), and optionally take an argument.
#[derive(Debug, Clone)]
pub struct OptionDef {
  pub short: Option<char>,
  pub long: Option<String>,
  pub has_arg: bool,
  pub val: char,
}

impl OptionDef {
  /// Creates a new option definition with only a short form (e.g., -h).
  ///
  /// * `name` - The single character used for the short option
  /// * `has_arg` - Whether this option expects an argument
  pub fn short(name: char, has_arg: bool) -> Self {
    Self {
      short: Some(name),
      long: None,
      has_arg,
      val: name,
    }
  }

  /// Creates a new option definition with only a long form (e.g., --help).
  ///
  /// * `long` - The string used for the long option
  /// * `val` - The character to return when this option is matched
  /// * `has_arg` - Whether this option expects an argument
  pub fn long(long: &str, val: char, has_arg: bool) -> Self {
    Self {
      short: None,
      long: Some(long.to_string()),
      has_arg,
      val,
    }
  }

  /// Creates a new option definition with both short and long forms.
  ///
  /// * `short` - The single character used for the short option
  /// * `long` - The string used for the long option
  /// * `has_arg` - Whether this option expects an argument
  pub fn both(short: char, long: &str, has_arg: bool) -> Self {
    Self {
      short: Some(short),
      long: Some(long.to_string()),
      has_arg,
      val: short,
    }
  }
}

/// The main parser struct that holds option definitions and creates iterators
/// over command line arguments.
#[derive(Debug)]
pub struct GetOpts {
  opt_map: HashMap<String, OptionDef>,
}

/// An iterator that processes command line arguments and yields parsed options.
#[derive(Debug)]
pub struct GetOptsIterator {
  program: Option<String>,
  args: std::vec::IntoIter<String>,
  opt_map: HashMap<String, OptionDef>,
}

/// Represents a parsed command line option or related value.
#[derive(Debug, Clone)]
pub enum ParsedOpt {
  /// The program name (first argument)
  Program(String),
  /// A matched option with its optional argument
  Opt(char, Option<String>),
  /// An unrecognized option
  InvalidOpt(String),
}

impl GetOpts {
  /// Creates a new option parser with the given option definitions.
  ///
  /// * `opts` - Slice of option definitions that will be recognized by the parser
  pub fn new(opts: &[OptionDef]) -> Self {
    let mut opt_map = HashMap::new();
    for opt in opts {
      if let Some(ref long) = opt.long {
        opt_map.insert(long.clone(), opt.clone());
      }
      if let Some(short) = opt.short {
        opt_map.insert(short.to_string(), opt.clone());
      }
    }
    Self { opt_map }
  }

  /// Creates an iterator that will parse the given command line arguments.
  ///
  /// The iterator will yield:
  /// 1. The program name as a `Program` variant
  /// 2. Each recognized option as an `Opt` variant
  /// 3. Any unrecognized options as `InvalidOpt` variants
  ///
  /// * `args` - Iterator of command line arguments, typically `std::env::args()`
  pub fn parse_args<T, I>(&self, args: I) -> GetOptsIterator
  where
    T: Into<String>,
    I: IntoIterator<Item = T>,
  {
    let mut args = args
      .into_iter()
      .map(|arg| arg.into())
      .collect::<Vec<String>>()
      .into_iter();
    GetOptsIterator {
      program: args.next(),
      args: args.into_iter(),
      opt_map: self.opt_map.clone(),
    }
  }
}

impl Iterator for GetOptsIterator {
  type Item = ParsedOpt;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(program) = self.program.take() {
      return Some(ParsedOpt::Program(program));
    }
    while let Some(arg) = self.args.next() {
      if !arg.starts_with('-') || arg == "-" {
        continue;
      }

      let opt_str = &arg[1..];

      if let Some(opt) = self.opt_map.get(opt_str) {
        if opt.has_arg {
          return match self.args.next() {
            Some(arg) => Some(ParsedOpt::Opt(opt.val, Some(arg))),
            None => Some(ParsedOpt::InvalidOpt(arg)),
          };
        } else {
          return Some(ParsedOpt::Opt(opt.val, None));
        }
      } else {
        return Some(ParsedOpt::InvalidOpt(arg));
      }
    }
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_separate_options() {
    let args = vec!["prog", "-a", "-b", "val", "-c"];
    let opts = vec![
      OptionDef::short('a', false),
      OptionDef::short('b', true),
      OptionDef::short('c', false),
    ];

    let parser = GetOpts::new(&opts);
    let parsed: Vec<_> = parser.parse_args(args).collect();

    assert_eq!(parsed.len(), 4);
    assert!(matches!(parsed[0], ParsedOpt::Program(ref s) if s == "prog"));
    assert!(matches!(parsed[1], ParsedOpt::Opt('a', None)));
    assert!(matches!(parsed[2], ParsedOpt::Opt('b', Some(ref s)) if s == "val"));
    assert!(matches!(parsed[3], ParsedOpt::Opt('c', None)));
  }

  #[test]
  fn test_mixed_options() {
    let args = vec!["prog", "-h", "-verbose", "-o", "file.txt"];
    let opts = vec![
      OptionDef::both('h', "help", false),
      OptionDef::long("verbose", 'v', false),
      OptionDef::both('o', "output", true),
    ];

    let parser = GetOpts::new(&opts);
    let parsed: Vec<_> = parser.parse_args(args).collect();

    assert_eq!(parsed.len(), 4);
    assert!(matches!(parsed[0], ParsedOpt::Program(ref s) if s == "prog"));
    assert!(matches!(parsed[1], ParsedOpt::Opt('h', None)));
    assert!(matches!(parsed[2], ParsedOpt::Opt('v', None)));
    assert!(matches!(parsed[3], ParsedOpt::Opt('o', Some(ref s)) if s == "file.txt"));
  }

  #[test]
  fn test_invalid_options() {
    let args = vec!["prog", "-x", "-y", "val", "--unknown"];
    let opts = vec![OptionDef::short('a', false), OptionDef::short('b', true)];

    let parser = GetOpts::new(&opts);
    let parsed: Vec<_> = parser.parse_args(args).collect();

    assert_eq!(parsed.len(), 4);
    assert!(matches!(parsed[0], ParsedOpt::Program(ref s) if s == "prog"));
    assert!(matches!(parsed[1], ParsedOpt::InvalidOpt(ref s) if s == "-x"));
    assert!(matches!(parsed[2], ParsedOpt::InvalidOpt(ref s) if s == "-y"));
    assert!(matches!(parsed[3], ParsedOpt::InvalidOpt(ref s) if s == "--unknown"));
  }
}
