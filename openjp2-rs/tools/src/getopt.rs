//! A simple command line argument parser inspired by the classic getopt interface.
//!
//! This module provides functionality to parse command line arguments in a way that's
//! familiar to users of the traditional Unix getopt library, while providing a more
//! Rust-friendly interface. It supports both options (with short/long forms) and
//! positional arguments.
//!
//! # Features
//!
//! - Short (-h) and long (-help) option forms
//! - Required option arguments (-o file)
//! - Positional arguments with fixed counts
//! - Comprehensive error reporting
//!
//! # Examples
//!
//! Basic usage with options and positional arguments:
//! ```rust
//! use openjp2_tools::getopt::{GetOpts, OptDef, PositionalArg, ParsedOpt};
//!
//! // Define our option types
//! #[derive(Debug, Clone, PartialEq)]
//! enum Opt {
//!     Verbose,
//!     Help,
//! }
//!
//! // Define our positional argument types
//! #[derive(Debug, Clone, PartialEq)]
//! enum PosArg {
//!     Input,
//!     Output,
//! }
//!
//! // Define options and positional args
//! let opts = vec![
//!     OptDef::short('v', Opt::Verbose, false),
//!     OptDef::short('h', Opt::Help, false),
//! ];
//!
//! let pos_args = vec![
//!     PositionalArg::new("input", PosArg::Input),
//!     PositionalArg::new("output", PosArg::Output),
//! ];
//!
//! let parser = GetOpts::new_with_positionals(&opts, &pos_args);
//! let args = vec!["myapp", "-v", "input.txt", "output.txt"];
//!
//! for opt in parser.parse_args(args) {
//!     match opt {
//!         ParsedOpt::Program(name) => println!("Program: {}", name),
//!         ParsedOpt::Opt(Opt::Verbose, None) => println!("Verbose mode enabled"),
//!         ParsedOpt::Positional(PosArg::Input, args) => println!("Input file: {}", args[0]),
//!         ParsedOpt::Positional(PosArg::Output, args) => println!("Output file: {}", args[0]),
//!         ParsedOpt::ParseError(err) => eprintln!("Error: {}", err),
//!         _ => {}
//!     }
//! }
//! ```
//!
//! # Error Handling
//!
//! All errors are returned as `ParsedOpt::ParseError` variants including:
//! - Unknown options
//! - Missing required arguments
//! - Insufficient positional arguments
//!
//! ```rust
//! # use openjp2_tools::getopt::{GetOpts, OptDef, ParsedOpt};
//! # #[derive(Debug, Clone, PartialEq)]
//! # enum Opt { Output }
//! let opts = vec![OptDef::short('o', Opt::Output, true)];
//! let parser = GetOpts::new(&opts);
//!
//! // Missing argument for -o
//! let args = vec!["prog", "-o"];
//! let parsed: Vec<_> = parser.parse_args(args).collect();
//! assert!(matches!(parsed[1], ParsedOpt::ParseError(_)));
//! ```

use std::fmt;
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

impl<V: Clone + fmt::Debug> OptDef<V> {
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

/// A positional argument that doesn't have an associated option.
///
/// Positional arguments are processed in the order they are defined after
/// all options have been parsed. Each positional argument can require one
/// or more values.
///
/// # Examples
///
/// ```rust
/// # use openjp2_tools::getopt::PositionalArg;
/// #[derive(Debug, Clone, PartialEq)]
/// enum PosArg {
///     Files,
///     Output,
/// }
///
/// // Single argument
/// let input = PositionalArg::new("input", PosArg::Files);
///
/// // Multiple arguments
/// let files = PositionalArg::new_multi("files", 3, PosArg::Files);
/// ```
#[derive(Debug, Clone)]
pub struct PositionalArg<V> {
  /// The name of the positional argument
  pub name: String,
  /// The number of arguments this positional argument expects
  pub arg_count: usize,
  /// The value associated with this positional argument
  pub val: V,
  /// Whether this positional argument is optional (default: false)
  pub optional: bool,
}

impl<V> PositionalArg<V> {
  /// Creates a new positional argument definition.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the positional argument
  /// * `val` - The value to associate with this positional argument
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use openjp2_tools::getopt::PositionalArg;
  /// #[derive(Debug, Clone)]
  /// enum Opt { Input }
  ///
  /// let arg = PositionalArg::new("input", Opt::Input);
  /// ```
  pub fn new(name: &str, val: V) -> Self {
    Self {
      name: name.to_string(),
      arg_count: 1,
      val,
      optional: false,
    }
  }

  /// Create a new positional argument definition that expects multiple arguments.
  ///
  /// # Arguments
  ///
  /// * `name` - The name of the positional argument
  /// * `arg_count` - The number of arguments this positional argument expects
  /// * `val` - The value to associate with this positional argument
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use openjp2_tools::getopt::PositionalArg;
  /// #[derive(Debug, Clone)]
  /// enum Opt { Input }
  ///
  /// let arg = PositionalArg::new_multi("input", 2, Opt::Input);
  /// ```
  pub fn new_multi(name: &str, arg_count: usize, val: V) -> Self {
    Self {
      name: name.to_string(),
      arg_count,
      val,
      optional: false,
    }
  }

  /// Marks this positional argument as optional.
  ///
  /// By default, positional arguments are required. Calling this method
  /// will make the argument optional.
  pub fn optional(mut self) -> Self {
    self.optional = true;
    self
  }
}

/// The main parser struct that holds option and positional argument definitions.
///
/// # Type Parameters
///
/// * `V` - The type of value associated with options, typically an enum
/// * `P` - The type of value associated with positional arguments, typically an enum
///
/// # Examples
///
/// ```rust
/// # use openjp2_tools::getopt::{GetOpts, OptDef, PositionalArg};
/// #[derive(Debug, Clone, PartialEq)]
/// enum Opt { Help }
///
/// #[derive(Debug, Clone, PartialEq)]
/// enum PosArg { Input }
///
/// // Parser with only options
/// let opts = vec![OptDef::short('h', Opt::Help, false)];
/// let parser = GetOpts::new(&opts);
///
/// // Parser with options and positional arguments
/// let pos_args = vec![PositionalArg::new("input", PosArg::Input)];
/// let parser = GetOpts::new_with_positionals(&opts, &pos_args);
/// ```
#[derive(Clone, Debug)]
pub struct GetOpts<V, P = ()> {
  opt_map: HashMap<String, OptDef<V>>,
  positionals: Vec<PositionalArg<P>>,
}

impl<V: Clone + fmt::Debug> GetOpts<V, ()> {
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
    Self {
      opt_map,
      positionals: vec![],
    }
  }
}

impl<V: Clone + fmt::Debug, P: Clone + fmt::Debug> GetOpts<V, P> {
  /// Creates a new option parser with the given option definitions and positional arguments.
  ///
  /// # Arguments
  ///
  /// * `opts` - Slice of option definitions that will be recognized by the parser
  /// * `positionals` - Slice of positional argument definitions
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use openjp2_tools::getopt::{GetOpts, OptDef, PositionalArg};
  /// #[derive(Debug, Clone, PartialEq)]
  /// enum Opt { Help }
  ///
  /// #[derive(Debug, Clone, PartialEq)]
  /// enum PosArg { Input, Output }
  ///
  /// let opts = vec![OptDef::short('h', Opt::Help, false)];
  /// let parser = GetOpts::new_with_positionals(&opts, &[
  ///   PositionalArg::new("input", PosArg::Input),
  ///   PositionalArg::new("output", PosArg::Output),
  /// ]);
  /// ```
  pub fn new_with_positionals(opts: &[OptDef<V>], positionals: &[PositionalArg<P>]) -> Self {
    let mut opt_map = HashMap::new();
    for opt in opts {
      if let Some(ref long) = opt.long {
        opt_map.insert(format!("-{long}"), opt.clone());
      }
      if let Some(short) = opt.short {
        opt_map.insert(format!("-{short}"), opt.clone());
      }
    }
    Self {
      opt_map,
      positionals: positionals.to_vec(),
    }
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
  pub fn parse_args<T, I>(&self, args: I) -> GetOptsIterator<'_, V, P>
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
      parse_positional: None,
      opts: self,
    }
  }

  fn get_opt(&self, opt: &str) -> Option<&OptDef<V>> {
    self.opt_map.get(opt)
  }

  fn has_opt(&self, opt: &str) -> bool {
    self.opt_map.contains_key(opt)
  }

  fn handle_positional(
    &self,
    idx: usize,
    first: Option<String>,
    args: &mut impl Iterator<Item = String>,
  ) -> Option<ParsedOpt<V, P>> {
    if let Some(pos_arg) = self.positionals.get(idx) {
      let mut values = Vec::with_capacity(pos_arg.arg_count);
      if let Some(first) = first {
        values.push(first);
      }
      for _ in values.len()..pos_arg.arg_count {
        if let Some(arg) = args.next() {
          values.push(arg);
        } else {
          break;
        }
      }
      if values.len() < pos_arg.arg_count {
        if pos_arg.optional && values.is_empty() {
          return None;
        }
        return Some(ParsedOpt::ParseError(format!(
          "Expected {} arguments for {}",
          pos_arg.arg_count, pos_arg.name
        )));
      }
      Some(ParsedOpt::Positional(pos_arg.val.clone(), values))
    } else {
      None
    }
  }
}

/// An iterator that processes command line arguments and yields parsed options.
///
/// Created by [`GetOpts::parse_args`].
///
/// # Type Parameters
///
/// * `V` - The type of value associated with options
#[derive(Debug)]
pub struct GetOptsIterator<'a, V, P> {
  program: Option<String>,
  args: Peekable<std::vec::IntoIter<String>>,
  parse_positional: Option<usize>,
  opts: &'a GetOpts<V, P>,
}

/// Represents the result of parsing a command line argument.
///
/// # Type Parameters
///
/// * `V` - The type of value associated with options
/// * `P` - The type of value associated with positional arguments
///
/// # Variants
///
/// * `Program` - The program name (first argument)
/// * `Opt` - A matched option with its optional argument
/// * `Positional` - A matched positional argument with its values
/// * `ParseError` - Any parsing error with a description
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
/// for result in parser.parse_args(vec!["prog", "-h"]) {
///     match result {
///         ParsedOpt::Program(name) => assert_eq!(name, "prog"),
///         ParsedOpt::Opt(Opt::Help, None) => println!("Help requested"),
///         ParsedOpt::ParseError(err) => eprintln!("Error: {}", err),
///         _ => {}
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum ParsedOpt<V, P> {
  Program(String),
  Opt(V, Option<String>),
  Positional(P, Vec<String>),
  ParseError(String),
}

impl<V: Clone + fmt::Debug, P: Clone + fmt::Debug> Iterator for GetOptsIterator<'_, V, P> {
  type Item = ParsedOpt<V, P>;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(program) = self.program.take() {
      return Some(ParsedOpt::Program(program));
    }
    if let Some(idx) = self.parse_positional {
      self.parse_positional = None;
      let positional = self.opts.handle_positional(idx, None, &mut self.args)?;
      self.parse_positional = Some(idx + 1);
      return Some(positional);
    }
    while let Some(opt_str) = self.args.next() {
      if !opt_str.starts_with('-') || opt_str == "-" {
        if let Some(positional) = self
          .opts
          .handle_positional(0, Some(opt_str), &mut self.args)
        {
          self.parse_positional = Some(1);
          return Some(positional);
        }
        break;
      }

      if let Some(opt) = self.opts.get_opt(&opt_str) {
        if opt.has_arg {
          let val = opt.val.clone();
          return match self.args.peek() {
            Some(arg) => {
              // Detect if the argument is another option.
              if self.opts.has_opt(arg) {
                Some(ParsedOpt::ParseError(format!(
                  "Expected argument for option {val:?}, got another option {arg}"
                )))
              } else {
                let arg = self.args.next().unwrap();
                Some(ParsedOpt::Opt(val, Some(arg)))
              }
            }
            None => Some(ParsedOpt::ParseError(format!(
              "Missing argument for option {val:?}"
            ))),
          };
        } else {
          return Some(ParsedOpt::Opt(opt.val.clone(), None));
        }
      } else if let Some(positional) =
        self
          .opts
          .handle_positional(0, Some(opt_str.clone()), &mut self.args)
      {
        self.parse_positional = Some(1);
        return Some(positional);
      } else {
        return Some(ParsedOpt::ParseError(format!("Invalid option: {opt_str}")));
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

  // Example enum for testing positional arguments
  #[derive(Debug, Clone, PartialEq)]
  enum PosArg {
    Input,
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
    let args = vec!["prog", "-x", "-y", "-unknown"];
    let opts = vec![
      OptDef::short('a', Opt::Help, false),
      OptDef::both('b', "beta", Opt::Output, true),
    ];

    let parser = GetOpts::new(&opts);
    let parsed: Vec<_> = parser.parse_args(args).collect();

    assert_eq!(parsed.len(), 4);
    assert!(matches!(parsed[0], ParsedOpt::Program(ref s) if s == "prog"));
    assert!(matches!(parsed[1], ParsedOpt::ParseError(ref s) if s == "Invalid option: -x"));
    assert!(matches!(parsed[2], ParsedOpt::ParseError(ref s) if s == "Invalid option: -y"));
    assert!(matches!(parsed[3], ParsedOpt::ParseError(ref s) if s == "Invalid option: -unknown"));
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
    assert!(matches!(parsed[0], ParsedOpt::Program(ref s) if s == "prog"));
    assert!(
      matches!(parsed[1], ParsedOpt::ParseError(ref s) if s == "Expected argument for option Output, got another option -v")
    );
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

  #[test]
  fn test_positional_args() {
    let args = vec!["prog", "input.txt", "output.txt"];
    let opts = vec![OptDef::short('h', Opt::Help, false)];
    let positionals = vec![
      PositionalArg::new("input", PosArg::Input),
      PositionalArg::new("output", PosArg::Output),
    ];

    let parser = GetOpts::new_with_positionals(&opts, &positionals);
    let parsed: Vec<_> = parser.parse_args(args).collect();

    assert_eq!(parsed.len(), 3);
    assert!(matches!(parsed[0], ParsedOpt::Program(ref s) if s == "prog"));
    assert!(
      matches!(parsed[1], ParsedOpt::Positional(PosArg::Input, ref v) if v == &["input.txt".to_string()])
    );
    assert!(
      matches!(parsed[2], ParsedOpt::Positional(PosArg::Output, ref v) if v == &["output.txt".to_string()])
    );
  }

  #[test]
  fn test_multi_positional_args() {
    let args = vec!["prog", "input.txt", "output.txt", "extra.txt"];
    let opts = vec![OptDef::short('h', Opt::Help, false)];
    let positionals = vec![
      PositionalArg::new("input", PosArg::Input),
      PositionalArg::new_multi("output", 2, PosArg::Output),
    ];

    let parser = GetOpts::new_with_positionals(&opts, &positionals);
    let parsed: Vec<_> = parser.parse_args(args).collect();

    assert_eq!(parsed.len(), 3);
    assert!(matches!(parsed[0], ParsedOpt::Program(ref s) if s == "prog"));
    assert!(
      matches!(parsed[1], ParsedOpt::Positional(PosArg::Input, ref v) if v == &["input.txt".to_string()])
    );
    assert!(
      matches!(parsed[2], ParsedOpt::Positional(PosArg::Output, ref v) if v == &["output.txt".to_string(), "extra.txt".to_string()])
    );
  }
}
