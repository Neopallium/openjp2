use openjp2_tools::getopt::{GetOpts, OptDef, ParsedOpt};
use std::fs::File;
use std::io::{BufRead, BufReader};
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

  // Validate required arguments
  if args.base_filename.is_none() || args.test_filename.is_none() {
    return Err("Both -b and -t options are required".into());
  }

  Ok(args)
}

fn print_help() {
  println!("\nList of parameters for the compare_dump_files function");
  println!();
  println!("  -b \t REQUIRED \t filename to the reference/baseline dump file");
  println!("  -t \t REQUIRED \t filename to the test dump file image");
  println!();
}

fn compare_files(base_path: &PathBuf, test_path: &PathBuf) -> Result<bool, String> {
  // Display parameters
  println!("******Parameters*********");
  println!(" base_filename = {}", base_path.display());
  println!(" test_filename = {}", test_path.display());
  println!("*************************");

  // Open base file
  print!("Try to open: {} for reading ... ", base_path.display());
  let base_file = File::open(base_path).map_err(|e| format!("Failed to open base file: {}", e))?;
  println!("Ok.");

  // Open test file
  print!("Try to open: {} for reading ... ", test_path.display());
  let test_file = File::open(test_path).map_err(|e| format!("Failed to open test file: {}", e))?;
  println!("Ok.");

  let base_reader = BufReader::new(base_file);
  let test_reader = BufReader::new(test_file);

  // Compare files line by line
  for (base_line, test_line) in base_reader.lines().zip(test_reader.lines()) {
    let base_str = base_line.map_err(|e| format!("Failed to read base file: {}", e))?;
    let test_str = test_line.map_err(|e| format!("Failed to read test file: {}", e))?;

    if base_str != test_str {
      eprintln!("<{}> vs. <{}>", base_str, test_str);
      return Ok(false);
    }
  }

  println!("\n***** TEST SUCCEED: Files are the same. *****");
  Ok(true)
}

fn main() -> Result<(), String> {
  let args = parse_args()?;

  let base_path = args.base_filename.unwrap();
  let test_path = args.test_filename.unwrap();

  let success = compare_files(&base_path, &test_path)?;

  std::process::exit(if success { 0 } else { 1 });
}
