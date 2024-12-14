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
//! use openjp2_tools::getopt::{GetOpts, OptDef, ParsedOpt};
//!
//! // Define our custom option types
//! #[derive(Debug, Clone, PartialEq)]
//! enum Opt {
//!     Verbose,
//!     Output,
//! }
//!
//! // Define options: -v (verbose), -o <file> (output file)
//! let opts = vec![
//!     OptDef::short('v', Opt::Verbose, false),     // no argument
//!     OptDef::short('o', Opt::Output, true),      // requires argument
//! ];
//!
//! let parser = GetOpts::new(&opts);
//! let args = vec!["myapp", "-v", "-o", "output.txt"];
//!
//! for opt in parser.parse_args(args) {
//!     match opt {
//!         ParsedOpt::Program(name) => println!("Program: {}", name),
//!         ParsedOpt::Opt(Opt::Verbose, None) => println!("Verbose mode enabled"),
//!         ParsedOpt::Opt(Opt::Output, Some(file)) => println!("Output file: {}", file),
//!         ParsedOpt::InvalidOpt(opt) => println!("Invalid option: {}", opt),
//!         _ => {}
//!     }
//! }
//! ```
//!
//! Using both short and long options:
//! ```rust
//! use openjp2_tools::getopt::{GetOpts, OptDef};
//!
//! #[derive(Debug, Clone, PartialEq)]
//! enum Opt {
//!     Help,
//!     Output,
//!     Verbose,
//! }
//!
//! let opts = vec![
//!     OptDef::both('h', "help", Opt::Help, false),
//!     OptDef::both('o', "output", Opt::Output, true),
//!     OptDef::long("verbose", Opt::Verbose, false),
//! ];
//!
//! // Will match both "-h" and "-help" returning Opt::Help
//! // Will match both "-o file.txt" and "-output file.txt" returning Opt::Output
//! // Will match "-verbose" returning Opt::Verbose
//! ```
//!
//! # Error Handling
//!
//! Invalid options are returned as `ParsedOpt::InvalidOpt` variants:
//! - Unknown options
//! - Missing required arguments
//!
//! ```rust
//! use openjp2_tools::getopt::{GetOpts, OptDef, ParsedOpt};
//!
//! #[derive(Debug, Clone, PartialEq)]
//! enum Opt {
//!     RequiredArg,
//! }
//!
//! let opts = vec![OptDef::short('a', Opt::RequiredArg, true)];
//! let parser = GetOpts::new(&opts);
//!
//! // Missing argument for -a
//! let args = vec!["prog", "-a"];
//! let parsed: Vec<_> = parser.parse_args(args).collect();
//! assert!(matches!(parsed[1], ParsedOpt::MissingArgument(_, _)));
//! ```

use std::{collections::HashMap, iter::Peekable};

/// Represents a command line option definition that can have a short form (-h),
/// long form (-help), and optionally take an argument.
///
/// # Type Parameters
///
/// * `V` - The type of value associated with this option, typically an enum variant
///
/// # Examples
///
/// ```rust
/// use openjp2_tools::getopt::OptDef;
///
/// #[derive(Debug, Clone)]
/// enum MyOpt {
///     Help,
///     Output,
/// }
///
/// let help_opt = OptDef::short('h', MyOpt::Help, false);
/// let output_opt = OptDef::both('o', "output", MyOpt::Output, true);
/// ```
#[derive(Debug, Clone)]
pub struct OptDef<V> {
  pub short: Option<char>,
  pub long: Option<String>,
  pub has_arg: bool,
  pub val: V,
}

impl<V: Clone> OptDef<V> {
  /// Creates a new option definition with only a short form (e.g., -h).
  ///
  /// # Arguments
  ///
  /// * `name` - The single character used for the short option
  /// * `val` - The value to associate with this option when matched
  /// * `has_arg` - Whether this option expects an argument
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use openjp2_tools::getopt::OptDef;
  /// #[derive(Debug, Clone)]
  /// enum Opt { Verbose }
  ///
  /// let opt = OptDef::short('v', Opt::Verbose, false);
  /// ```
  pub fn short(name: char, val: V, has_arg: bool) -> Self {
    Self {
      short: Some(name),
      long: None,
      has_arg,
      val,
    }
  }

  /// Creates a new option definition with only a long form (e.g., -help).
  ///
  /// # Arguments
  ///
  /// * `long` - The string used for the long option
  /// * `val` - The value to associate with this option when matched
  /// * `has_arg` - Whether this option expects an argument
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use openjp2_tools::getopt::OptDef;
  /// #[derive(Debug, Clone)]
  /// enum Opt { Help }
  ///
  /// let opt = OptDef::long("help", Opt::Help, false);
  /// ```
  pub fn long(long: &str, val: V, has_arg: bool) -> Self {
    Self {
      short: None,
      long: Some(long.to_string()),
      has_arg,
      val,
    }
  }

  /// Creates a new option definition with both short and long forms.
  ///
  /// # Arguments
  ///
  /// * `short` - The single character used for the short option
  /// * `long` - The string used for the long option
  /// * `val` - The value to associate with this option when matched
  /// * `has_arg` - Whether this option expects an argument
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use openjp2_tools::getopt::OptDef;
  /// #[derive(Debug, Clone)]
  /// enum Opt { Output }
  ///
  /// let opt = OptDef::both('o', "output", Opt::Output, true);
  /// ```
  pub fn both(short: char, long: &str, val: V, has_arg: bool) -> Self {
    Self {
      short: Some(short),
      long: Some(long.to_string()),
      has_arg,
      val,
    }
  }
}

/// The main parser struct that holds option definitions and creates iterators
/// over command line arguments.
///
/// # Type Parameters
///
/// * `V` - The type of value associated with options, typically an enum
///
/// # Examples
///
/// ```rust
/// # use openjp2_tools::getopt::{GetOpts, OptDef};
/// #[derive(Debug, Clone)]
/// enum Opt { Help }
///
/// let opts = vec![OptDef::short('h', Opt::Help, false)];
/// let parser = GetOpts::new(&opts);
/// ```
#[derive(Debug)]
pub struct GetOpts<V> {
  opt_map: HashMap<String, OptDef<V>>,
}

/// An iterator that processes command line arguments and yields parsed options.
///
/// Created by [`GetOpts::parse_args`].
///
/// # Type Parameters
///
/// * `V` - The type of value associated with options
#[derive(Debug)]
pub struct GetOptsIterator<V> {
  program: Option<String>,
  args: Peekable<std::vec::IntoIter<String>>,
  opt_map: HashMap<String, OptDef<V>>,
}

/// Represents a parsed command line option or related value.
///
/// # Type Parameters
///
/// * `V` - The type of value associated with options
///
/// # Variants
///
/// * `Program` - The program name (first argument)
/// * `Opt` - A matched option with its optional argument
/// * `InvalidOpt` - An unrecognized option
/// * `MissingArgument` - An option that requires an argument but didn't receive one
#[derive(Debug, Clone)]
pub enum ParsedOpt<V> {
  Program(String),
  Opt(V, Option<String>),
  InvalidOpt(String),
  MissingArgument(V, Option<String>),
}

impl<V: Clone> GetOpts<V> {
  /// Creates a new option parser with the given option definitions.
  ///
  /// # Arguments
  ///
  /// * `opts` - Slice of option definitions that will be recognized by the parser
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use openjp2_tools::getopt::{GetOpts, OptDef};
  /// #[derive(Debug, Clone)]
  /// enum Opt { Help, Version }
  ///
  /// let opts = vec![
  ///     OptDef::short('h', Opt::Help, false),
  ///     OptDef::short('v', Opt::Version, false),
  /// ];
  /// let parser = GetOpts::new(&opts);
  /// ```
  pub fn new(opts: &[OptDef<V>]) -> Self {
    let mut opt_map = HashMap::new();
    for opt in opts {
      if let Some(ref long) = opt.long {
        opt_map.insert(format!("-{long}"), opt.clone());
      }
      if let Some(short) = opt.short {
        opt_map.insert(format!("-{short}"), opt.clone());
      }
    }
    Self { opt_map }
  }

  /// Creates an iterator that will parse the given command line arguments.
  ///
  /// The iterator yields:
  /// 1. The program name as a `Program` variant
  /// 2. Each recognized option as an `Opt` variant
  /// 3. Any unrecognized options as `InvalidOpt` variants
  /// 4. Options missing required arguments as `MissingArgument` variants
  ///
  /// # Arguments
  ///
  /// * `args` - Iterator of command line arguments, typically `std::env::args()`
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use openjp2_tools::getopt::{GetOpts, OptDef, ParsedOpt};
  /// #[derive(Debug, Clone, PartialEq)]
  /// enum Opt { Help }
  ///
  /// let opts = vec![OptDef::short('h', Opt::Help, false)];
  /// let parser = GetOpts::new(&opts);
  ///
  /// for opt in parser.parse_args(vec!["prog", "-h"]) {
  ///     match opt {
  ///         ParsedOpt::Program(name) => println!("Program: {}", name),
  ///         ParsedOpt::Opt(Opt::Help, None) => println!("Help requested"),
  ///         _ => {}
  ///     }
  /// }
  /// ```
  pub fn parse_args<T, I>(&self, args: I) -> GetOptsIterator<V>
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
      args: args.peekable(),
      opt_map: self.opt_map.clone(),
    }
  }
}

impl<V: Clone> Iterator for GetOptsIterator<V> {
  type Item = ParsedOpt<V>;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(program) = self.program.take() {
      return Some(ParsedOpt::Program(program));
    }
    while let Some(opt_str) = self.args.next() {
      if !opt_str.starts_with('-') || opt_str == "-" {
        continue;
      }

      if let Some(opt) = self.opt_map.get(&opt_str) {
        if opt.has_arg {
          let val = opt.val.clone();
          return match self.args.peek() {
            Some(arg) => {
              // Detect if the argument is another option.
              if self.opt_map.contains_key(arg) {
                Some(ParsedOpt::MissingArgument(val, Some(arg.clone())))
              } else {
                let arg = self.args.next().unwrap();
                Some(ParsedOpt::Opt(val, Some(arg)))
              }
            }
            None => Some(ParsedOpt::MissingArgument(val, None)),
          };
        } else {
          return Some(ParsedOpt::Opt(opt.val.clone(), None));
        }
      } else {
        return Some(ParsedOpt::InvalidOpt(opt_str));
      }
    }
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Example enum for testing generic value type
  #[derive(Debug, Clone, PartialEq)]
  enum Opt {
    Help,
    Verbose,
    Output,
  }

  #[test]
  fn test_with_enum() {
    let args = vec!["prog", "-h", "-verbose", "-o", "file.txt"];
    let opts = vec![
      OptDef::both('h', "help", Opt::Help, false),
      OptDef::long("verbose", Opt::Verbose, false),
      OptDef::both('o', "output", Opt::Output, true),
    ];

    let parser = GetOpts::new(&opts);
    let parsed: Vec<_> = parser.parse_args(args).collect();

    assert_eq!(parsed.len(), 4);
    assert!(matches!(parsed[0], ParsedOpt::Program(ref s) if s == "prog"));
    assert!(matches!(parsed[1], ParsedOpt::Opt(Opt::Help, None)));
    assert!(matches!(parsed[2], ParsedOpt::Opt(Opt::Verbose, None)));
    assert!(matches!(parsed[3], ParsedOpt::Opt(Opt::Output, Some(ref s)) if s == "file.txt"));
  }

  // Legacy test using char values
  #[test]
  fn test_with_char() {
    let args = vec!["prog", "-a", "-b", "val"];
    let opts = vec![
      OptDef::short('a', 'a', false),
      OptDef::short('b', 'b', true),
    ];

    let parser = GetOpts::new(&opts);
    let parsed: Vec<_> = parser.parse_args(args).collect();

    assert_eq!(parsed.len(), 3);
    assert!(matches!(parsed[0], ParsedOpt::Program(ref s) if s == "prog"));
    assert!(matches!(parsed[1], ParsedOpt::Opt('a', None)));
    assert!(matches!(parsed[2], ParsedOpt::Opt('b', Some(ref s)) if s == "val"));
  }

  #[test]
  fn test_unknown_option() {
    let args = vec!["prog", "-x", "-y", "--unknown"];
    let opts = vec![
      OptDef::short('a', Opt::Help, false),
      OptDef::both('b', "beta", Opt::Output, true),
    ];

    let parser = GetOpts::new(&opts);
    let parsed: Vec<_> = parser.parse_args(args).collect();

    assert_eq!(parsed.len(), 4);
    assert!(matches!(parsed[0], ParsedOpt::Program(ref s) if s == "prog"));
    assert!(matches!(parsed[1], ParsedOpt::InvalidOpt(ref s) if s == "-x"));
    assert!(matches!(parsed[2], ParsedOpt::InvalidOpt(ref s) if s == "-y"));
    assert!(matches!(parsed[3], ParsedOpt::InvalidOpt(ref s) if s == "--unknown"));
  }

  #[test]
  fn test_missing_argument() {
    let args = vec!["prog", "-o", "-v"];
    let opts = vec![
      OptDef::short('o', Opt::Output, true), // requires argument
      OptDef::short('v', Opt::Verbose, false),
    ];

    let parser = GetOpts::new(&opts);
    let parsed: Vec<_> = parser.parse_args(args).collect();

    assert_eq!(parsed.len(), 3);
    eprintln!("{:?}", parsed);
    assert!(matches!(parsed[0], ParsedOpt::Program(ref s) if s == "prog"));
    assert!(matches!(parsed[1], ParsedOpt::MissingArgument(Opt::Output, Some(ref s)) if s == "-v"));
    assert!(matches!(parsed[2], ParsedOpt::Opt(Opt::Verbose, None)));
  }

  #[test]
  fn test_empty_args() {
    let args: Vec<String> = vec![];
    let opts = vec![OptDef::short('a', Opt::Help, false)];

    let parser = GetOpts::new(&opts);
    let parsed: Vec<_> = parser.parse_args(args).collect();

    assert_eq!(parsed.len(), 0);
  }
}
