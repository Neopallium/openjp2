use openjp2_tools::getopt::{GetOpts, OptDef, ParsedOpt};
use std::fs::File;
use std::io::{Read, Result as IoResult};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
enum Opt {
  Base,
  Test,
  Help,
}

struct Args {
  base_filename: Option<PathBuf>,
  test_filename: Option<PathBuf>,
}

impl Default for Args {
  fn default() -> Self {
    Self {
      base_filename: None,
      test_filename: None,
    }
  }
}

fn parse_args() -> Result<Args, String> {
  let opts = vec![
    OptDef::short('b', Opt::Base, true),
    OptDef::short('t', Opt::Test, true),
    OptDef::short('h', Opt::Help, false),
  ];

  let parser = GetOpts::new(&opts);
  let mut args = Args::default();

  for opt in parser.parse_args(std::env::args()) {
    match opt {
      ParsedOpt::Program(_) => {}
      ParsedOpt::Opt(opt, arg) => match opt {
        Opt::Base => {
          args.base_filename = Some(PathBuf::from(arg.ok_or("Missing base filename")?));
        }
        Opt::Test => {
          args.test_filename = Some(PathBuf::from(arg.ok_or("Missing test filename")?));
        }
        Opt::Help => {
          print_help();
          std::process::exit(0);
        }
      },
      ParsedOpt::Positional(_, _) => {
        return Err("Positional arguments are not supported".into());
      }
      ParsedOpt::ParseError(err) => {
        return Err(err);
      }
    }
  }

  if args.base_filename.is_none() || args.test_filename.is_none() {
    return Err("Both -b and -t options are required".into());
  }

  Ok(args)
}

fn print_help() {
  println!("\nList of parameters for the compare_raw_files function");
  println!();
  println!("  -b \t REQUIRED \t filename to the reference/baseline RAW image");
  println!("  -t \t REQUIRED \t filename to the test RAW image");
  println!();
}

fn compare_files(base_path: &PathBuf, test_path: &PathBuf) -> IoResult<bool> {
  let mut file_base = File::open(base_path)?;
  let mut file_test = File::open(test_path)?;

  let mut pos = 0;
  let mut base_buf = [0u8; 1];
  let mut test_buf = [0u8; 1];

  loop {
    let base_read = file_base.read(&mut base_buf)?;
    let test_read = file_test.read(&mut test_buf)?;

    // Check if we've reached the end of both files
    if base_read == 0 && test_read == 0 {
      println!("---- TEST SUCCEED: Files are equal ----");
      return Ok(true);
    }

    // Check for files of different sizes
    if base_read != test_read {
      println!("Files have different sizes.");
      return Ok(false);
    }

    // Compare bytes
    if base_buf[0] != test_buf[0] {
      println!(
        "Binary values read in the file are different {:x} vs {:x} at position {}.",
        base_buf[0], test_buf[0], pos
      );
      return Ok(false);
    }

    pos += 1;
  }
}

fn main() -> Result<(), String> {
  let args = parse_args()?;

  let success = compare_files(
    args.base_filename.as_ref().unwrap(),
    args.test_filename.as_ref().unwrap(),
  )
  .map_err(|e| format!("Error comparing files: {}", e))?;

  std::process::exit(if success { 0 } else { 1 });
}
