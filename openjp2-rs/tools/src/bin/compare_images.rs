use openjp2::opj_image;
use openjp2_tools::convert::load_image;
use openjp2_tools::getopt::{GetOpts, OptDef, ParsedOpt};
use openjp2_tools::params::{CompressionParameters, ImageFileFormat};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
enum Opt {
  Base,
  Test,
  NumComp,
  MSE,
  PEAK,
  Separator,
  NonRegression,
  Ignore,
  Help,
}

struct Args {
  base_file: Option<PathBuf>,
  test_file: Option<PathBuf>,
  num_components: usize,
  mse_tolerances: Vec<f64>,
  peak_tolerances: Vec<f64>,
  base_separator: String,
  test_separator: String,
  non_regression: bool,
  ignore_prec: bool,
}

impl Default for Args {
  fn default() -> Self {
    Self {
      base_file: None,
      test_file: None,
      num_components: 0,
      mse_tolerances: vec![],
      peak_tolerances: vec![],
      base_separator: String::new(),
      test_separator: String::new(),
      non_regression: false,
      ignore_prec: false,
    }
  }
}

fn parse_args() -> Result<Args, String> {
  let opts = vec![
    OptDef::short('b', Opt::Base, true),
    OptDef::short('t', Opt::Test, true),
    OptDef::short('n', Opt::NumComp, true),
    OptDef::short('m', Opt::MSE, true),
    OptDef::short('p', Opt::PEAK, true),
    OptDef::short('s', Opt::Separator, true),
    OptDef::short('d', Opt::NonRegression, false),
    OptDef::short('i', Opt::Ignore, true),
    OptDef::short('h', Opt::Help, false),
  ];

  let parser = GetOpts::new(&opts);
  let mut args = Args::default();

  for opt in parser.parse_args(std::env::args()) {
    match opt {
      ParsedOpt::Program(_) => {}
      ParsedOpt::Opt(opt, arg) => match opt {
        Opt::Base => {
          args.base_file = Some(PathBuf::from(arg.ok_or("Missing base file")?));
        }
        Opt::Test => {
          args.test_file = Some(PathBuf::from(arg.ok_or("Missing test file")?));
        }
        Opt::NumComp => {
          args.num_components = arg
            .ok_or("Missing number of components")?
            .parse()
            .map_err(|_| "Invalid number of components")?;
        }
        Opt::MSE => {
          args.mse_tolerances = parse_float_list(&arg.ok_or("Missing MSE values")?)?;
        }
        Opt::PEAK => {
          args.peak_tolerances = parse_float_list(&arg.ok_or("Missing PEAK values")?)?;
        }
        Opt::Separator => {
          let seps = parse_separators(&arg.ok_or("Missing separators")?)?;
          args.base_separator = seps.0;
          args.test_separator = seps.1;
        }
        Opt::NonRegression => args.non_regression = true,
        Opt::Ignore => {
          if arg.ok_or("Missing ignore parameter")? == "prec" {
            args.ignore_prec = true;
          } else {
            return Err("Only 'prec' is supported for -i option".into());
          }
        }
        Opt::Help => {
          print_help();
          std::process::exit(0);
        }
      },
      ParsedOpt::InvalidOpt(opt) => {
        return Err(format!("Invalid option: {}", opt));
      }
      ParsedOpt::MissingArgument(opt, _) => {
        return Err(format!("Missing argument for option: {:?}", opt));
      }
    }
  }

  // Validate required args
  if args.base_file.is_none() {
    return Err("Base file (-b) is required".into());
  }
  if args.test_file.is_none() {
    return Err("Test file (-t) is required".into());
  }
  if args.num_components == 0 {
    return Err("Number of components (-n) is required".into());
  }

  // Validate test mode
  if args.non_regression {
    if !args.mse_tolerances.is_empty() || !args.peak_tolerances.is_empty() {
      return Err("Cannot specify tolerances in non-regression mode".into());
    }
  } else {
    if args.mse_tolerances.is_empty() || args.peak_tolerances.is_empty() {
      return Err("MSE and PEAK tolerances required in comparison mode".into());
    }
  }

  Ok(args)
}

fn parse_float_list(value: &str) -> Result<Vec<f64>, String> {
  value
    .split(':')
    .map(|s| s.parse().map_err(|_| format!("Invalid float value: {}", s)))
    .collect()
}

fn parse_separators(value: &str) -> Result<(String, String), String> {
  let mut parts = value.chars();
  let mut base_sep = "".to_string();
  let mut test_sep = "".to_string();
  while let Some(c) = parts.next() {
    match c {
      't' => {
        test_sep = parts.next().ok_or("Missing test separator")?.to_string();
      }
      'b' => {
        base_sep = parts.next().ok_or("Missing base separator")?.to_string();
      }
      _ => {
        return Err("Invalid separator format".into());
      }
    }
  }
  Ok((base_sep, test_sep))
}

fn print_help() {
  println!("\nThis is the compare_images utility from the OpenJPEG project.");
  println!("It compares two images and outputs the differences.");
  println!("\nParameters:");
  println!("-----------\n");
  println!("  -b <base file>");
  println!("    REQUIRED");
  println!("    Filename of the reference/baseline image.");
  println!("  -t <test file>");
  println!("    REQUIRED");
  println!("    Filename of the test image.");
  println!("  -n <number of components>");
  println!("    REQUIRED");
  println!("    Number of components in the image.");
  println!("  -m <MSE values>");
  println!("    OPTIONAL");
  println!("    List of MSE tolerances, separated by ':'.");
  println!("  -p <PEAK values>");
  println!("    OPTIONAL");
  println!("    List of PEAK tolerances, separated by ':'.");
  println!("  -s <separators>");
  println!("    OPTIONAL");
  println!("    Separators for base and test files, separated by space.");
  println!("  -d");
  println!("    OPTIONAL");
  println!("    Run as non-regression test.");
  println!("  -i <ignore>");
  println!("    OPTIONAL");
  println!("    List of features to ignore. Currently 'prec' only supported.");
  println!("  -h");
  println!("    OPTIONAL");
  println!("    Display this help message.");
  println!("");
}

fn read_image_from_multiple_files(
  file: &PathBuf,
  params: CompressionParameters,
  nb_files: usize,
  separator: &str,
) -> Result<Box<opj_image>, String> {
  let stem = file
    .file_stem()
    .ok_or("Failed to get file stem")?
    .to_string_lossy();
  let ext = match &params.decode_format {
    Some(ImageFileFormat::PGX) => "pgx".to_string(),
    Some(ImageFileFormat::PXM) => "pgm".to_string(),
    _ => file
      .extension()
      .ok_or("Failed to get file extension")?
      .to_string_lossy()
      .to_string(),
  };

  let mut image = opj_image::new();
  if !image.alloc_comps(nb_files as u32) {
    return Err("Failed to allocate memory for components".into());
  }
  let mut comps = image
    .comps_mut()
    .expect("We just allocated the components")
    .into_iter();

  for i in 0..nb_files {
    let file = file.with_file_name(format!("{}{}{}.{}", stem, separator, i, ext));
    let src_image =
      load_image(&file, &params).map_err(|e| format!("Failed to load image from file: {}", e))?;
    let Some(src_comps) = src_image.comps() else {
      return Err("Failed to load image components".into());
    };

    // Copy the first component
    let src_comp = src_comps[0];
    let dst_comp = comps.next().expect("We just allocated the components");
    dst_comp.copy(&src_comp);
    dst_comp.x0 = 0;
    dst_comp.y0 = 0;
    dst_comp.dx = 0;
    dst_comp.dy = 0;
  }

  return Ok(image);
}

fn read_image_from_file(
  file: &PathBuf,
  nb_files: usize,
  separator: &str,
) -> Result<Box<opj_image>, String> {
  let mut params = CompressionParameters::default();
  let format = ImageFileFormat::get_file_format(file)
    .map_err(|e| format!("Failed to get file format: {}", e))?;
  params.decode_format = Some(format);

  let nb_files = if separator.is_empty() { 1 } else { nb_files };

  if nb_files > 1 || !separator.is_empty() {
    match &params.decode_format {
      Some(ImageFileFormat::PGX | ImageFileFormat::PXM) => {
        return read_image_from_multiple_files(file, params, nb_files, separator);
      }
      _ => (),
    }
  }

  let image =
    load_image(&file, &params).map_err(|e| format!("Failed to load image from file: {}", e))?;
  Ok(image)
}

fn main() -> Result<(), String> {
  env_logger::init();

  let args = match parse_args() {
    Ok(args) => args,
    Err(e) => {
      eprintln!("Error: {}", e);
      print_help();
      std::process::exit(1);
    }
  };

  let base_file = args.base_file.as_ref().expect("Base file not set");
  let test_file = args.test_file.as_ref().expect("Test file not set");

  // Display parameters
  println!("******Parameters********* ");
  println!(" base_filename = {}", base_file.display());
  println!(" test_filename = {}", test_file.display());
  println!(" nb of Components = {}", args.num_components);
  println!(" Non regression test = {}", args.non_regression as u32);
  println!(" separator Base = {}", args.base_separator);
  println!(" separator Test = {}", args.test_separator);

  if !args.mse_tolerances.is_empty() && !args.peak_tolerances.is_empty() {
    println!(" MSE values = {:?}", args.mse_tolerances);
    println!(" PEAK values = {:?}", args.peak_tolerances);
    println!(" Non-regression test = {}", args.non_regression as u32);
  }

  let mut nb_filename_pgxbase = 0;
  let mut nb_filename_pgxtest = 0;
  if !args.base_separator.is_empty() {
    nb_filename_pgxbase = args.num_components;
  }
  if !args.test_separator.is_empty() {
    nb_filename_pgxtest = args.num_components;
  }

  println!(
    " NbFilename to generate from base filename = {}",
    nb_filename_pgxbase
  );
  println!(
    " NbFilename to generate from test filename = {}",
    nb_filename_pgxtest
  );
  println!("************************* ");

  // Load base image
  let base_image = read_image_from_file(base_file, nb_filename_pgxbase, &args.base_separator)?;
  let Some(base_comps) = base_image.comps() else {
    return Err("Failed to load test image components".into());
  };

  // Load test image
  let test_image = read_image_from_file(test_file, nb_filename_pgxtest, &args.test_separator)?;
  let Some(test_comps) = test_image.comps() else {
    return Err("Failed to load test image components".into());
  };

  // Comparison of header parameters
  println!("Step 1 -> Header comparison");

  // Compare images
  if base_image.numcomps != test_image.numcomps {
    return Err("Number of components mismatch".into());
  }

  // Create a new image to store the differences
  let mut diff_image = opj_image::new();
  diff_image.color_space = base_image.color_space;
  diff_image.x0 = base_image.x0;
  diff_image.y0 = base_image.y0;
  diff_image.x1 = base_image.x1;
  diff_image.y1 = base_image.y1;

  // Allocate memory for the components
  if !diff_image.alloc_comps(base_image.numcomps) {
    return Err("Failed to allocate memory for components".into());
  }
  let diff_comps = diff_image
    .comps_mut()
    .expect("We just allocated the components");

  // Compare components
  let comps = base_comps
    .iter()
    .zip(test_comps.iter())
    .zip(diff_comps.iter_mut());
  for (idx, ((base_comp, test_comp), diff_comp)) in comps.enumerate() {
    // Check signedness.
    if base_comp.sgnd != test_comp.sgnd {
      return Err(format!(
        "ERROR: sign mismatch [comp {idx}] ({}><{})",
        base_comp.sgnd, test_comp.sgnd
      ));
    }

    // Check precision.
    if base_comp.prec != test_comp.prec && !args.ignore_prec {
      return Err(format!(
        "ERROR: prec mismatch [comp {idx}] ({}><{})",
        base_comp.prec, test_comp.prec
      ));
    }

    // Check height.
    if base_comp.h != test_comp.h {
      return Err(format!(
        "ERROR: height mismatch [comp {idx}] ({}><{})",
        base_comp.h, test_comp.h
      ));
    }

    // Check width.
    if base_comp.w != test_comp.w {
      return Err(format!(
        "ERROR: width mismatch [comp {idx}] ({}><{})",
        base_comp.w, test_comp.w
      ));
    }

    // Initialize the difference component
    diff_comp.dx = 0;
    diff_comp.dy = 0;
    diff_comp.x0 = 0;
    diff_comp.y0 = 0;
    diff_comp.sgnd = 0;
    diff_comp.prec = 8;
    diff_comp.w = base_comp.w;
    diff_comp.h = base_comp.h;
    if !diff_comp.alloc_data() {
      return Err("Failed to allocate memory for component".into());
    }
  }

  // Measurement computation
  println!("Step 2 -> measurement comparison");

  // Compute pixel differences
  let mut failed = false;
  let mut sum_diff = 0;
  let mut nb_pixel_diff = 0;

  let comps = base_comps
    .iter()
    .zip(test_comps.iter())
    .zip(diff_comps.iter_mut());
  for (idx, ((base_comp, test_comp), diff_comp)) in comps.enumerate() {
    let Some(base_data) = base_comp.data() else {
      return Err("Failed to get base component data".into());
    };
    let Some(test_data) = test_comp.data() else {
      return Err("Failed to get test component data".into());
    };
    let diff_data = diff_comp.data_mut().expect("We just allocated it");
    let mut se = 0.0;
    let mut peak = 0.0;
    let (shift_base, shift_test) = if base_comp.prec > test_comp.prec {
      (base_comp.prec - test_comp.prec, 0)
    } else {
      (0, test_comp.prec - base_comp.prec)
    };

    let pixels = base_data
      .iter()
      .zip(test_data.iter())
      .zip(diff_data.iter_mut());
    for ((base, test), diff) in pixels {
      let value_diff = (base >> shift_base) - (test >> shift_test);
      if value_diff != 0 {
        let diff_abs = value_diff.abs();
        *diff = diff_abs;
        sum_diff += value_diff;
        nb_pixel_diff += 1;

        se += (value_diff * value_diff) as f64;
        if diff_abs as f64 > peak {
          peak = diff_abs as f64;
        }
      } else {
        *diff = 0;
      }
    }

    let mse = se / (base_comp.w * base_comp.h) as f64;

    if !args.non_regression {
      // Conformance test
      println!(
        "<DartMeasurement name=\"PEAK_{idx}\" type=\"numeric/double\"> {peak} </DartMeasurement>"
      );
      println!(
        "<DartMeasurement name=\"MSE_{idx}\" type=\"numeric/double\"> {mse} </DartMeasurement>"
      );

      if mse > args.mse_tolerances[idx] || peak > args.peak_tolerances[idx] {
        eprintln!(
          "ERROR: MSE ({mse}) or PEAK ({peak}) values produced by the decoded file are greater than the allowable error (respectively {} and {})",
          args.mse_tolerances[idx], args.peak_tolerances[idx]
        );
        failed = true;
      }
    } else {
      // Non-regression mode
      if nb_pixel_diff > 0 {
        println!("<DartMeasurement name=\"NumberOfPixelsWithDifferences_{idx}\" type=\"numeric/int\"> {nb_pixel_diff} </DartMeasurement>");
        println!("<DartMeasurement name=\"ComponentError_{idx}\" type=\"numeric/double\"> {sum_diff} </DartMeasurement>");
        println!(
          "<DartMeasurement name=\"MSE_{idx}\" type=\"numeric/double\"> {mse} </DartMeasurement>"
        );
        println!(
          "<DartMeasurement name=\"PEAK_{idx}\" type=\"numeric/double\"> {peak} </DartMeasurement>"
        );
        failed = true;
        break;
      }
    }
  }

  if !failed {
    println!("---- TEST SUCCEED ----");
  }

  std::process::exit(if !failed { 0 } else { 1 });
}
