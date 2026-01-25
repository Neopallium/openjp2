use std::collections::BTreeMap;
use std::env;
/// Common test utilities for integration tests
use std::path::{Path, PathBuf};
use std::sync::Arc;

use openjp2_tools::cli::{
  run_compare_dump_files, run_compare_images, run_compress, run_decompress, run_dump,
};

use crate::{args, common};

const OPJ_TEST_CMD_LIST: &str = include_str!("../../../../tests/nonregression/test_suite.ctest.in");
const OPJ_TEST_MD5_LIST: &str = include_str!("../../../../tests/nonregression/md5refs.txt");
const LIBTIFF_4_1: &str = "libtiff_4_1:";

pub struct MD5References {
  pub refs: BTreeMap<String, String>,
  pub libtiff_41_refs: BTreeMap<String, String>,
}

impl MD5References {
  pub fn load_md5_references() -> Arc<MD5References> {
    let mut refs = BTreeMap::new();
    let mut libtiff_41_refs = BTreeMap::new();
    for line in OPJ_TEST_MD5_LIST.lines() {
      let line = line.trim();
      if line.is_empty() || line.starts_with('#') {
        continue;
      }
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() == 2 {
        let hash = parts[0];
        let filename = parts[1];
        if hash.starts_with(LIBTIFF_4_1) {
          let clean_hash = hash.trim_start_matches(LIBTIFF_4_1);
          libtiff_41_refs.insert(filename.to_string(), clean_hash.to_string());
        } else {
          refs.insert(filename.to_string(), hash.to_string());
        }
      }
    }
    Arc::new(MD5References {
      refs,
      libtiff_41_refs,
    })
  }

  pub fn check_output_files_md5(&self, output: PathBuf) -> Result<(), String> {
    if output.exists() {
      self.check_output_md5(output)?;
    } else {
      // Handle cases where multiple output files are expected (e.g., multi-component images)
      let parent_dir = output
        .parent()
        .expect("Output file has no parent directory");
      let file_stem = output
        .file_stem()
        .expect("Output file has no stem")
        .to_string_lossy();
      let extension = output
        .extension()
        .expect("Output file has no extension")
        .to_string_lossy();

      let mut found = false;
      for comp_idx in 0..10 {
        let comp_file = parent_dir.join(format!("{}_{}.{}", file_stem, comp_idx, extension));
        if comp_file.exists() {
          found = true;
          self.check_output_md5(comp_file)?;
        } else {
          break;
        }
      }
      if !found {
        return Err(format!("No output files found for {:?}", output));
      }
    }

    Ok(())
  }

  pub fn check_output_md5(&self, output: PathBuf) -> Result<(), String> {
    let output_name = output.file_name().unwrap().to_string_lossy().to_string();
    // Calculate MD5 of output file.
    let file_md5 = common::md5_file(&output).expect("Failed to compute MD5 of output file");

    // Get expected MD5 from references.
    let expected_md5 = self.refs.get(&output_name);
    let expected_alt_md5 = self.libtiff_41_refs.get(&output_name);
    let md5_and_file = format!("{} {}", file_md5, output_name);
    if Some(&file_md5) == expected_md5 {
      let expected_md5 = expected_md5.unwrap();
      println!(
        "equal: [{} {}] vs [{}]",
        expected_md5, output_name, md5_and_file
      );
    } else if Some(&file_md5) == expected_alt_md5 {
      let expected_md5 = expected_alt_md5.unwrap();
      println!(
        "equal: [libtiff_4_1:{} {}] vs [libtiff_4_1:{}]",
        expected_md5, output_name, md5_and_file
      );
    } else {
      let expected_md5 = expected_md5.or(expected_alt_md5);
      if let Some(expected_md5) = expected_md5 {
        return Err(format!(
          "not equal: [{} {}] vs [{}]",
          expected_md5, output_name, md5_and_file
        ));
      } else {
        return Err(format!(
          "not equal: [no reference md5] vs [{}]",
          md5_and_file
        ));
      }
    }

    Ok(())
  }
}

#[derive(Debug)]
pub struct TestCommand {
  pub command: Vec<String>,
  pub is_encode: bool,
  pub index: usize,
  pub input_file: String,
  pub output_file: String,
  pub should_fail: bool,
  pub lossy_check: Option<Vec<String>>,
}

impl TestCommand {
  /// Get the input file path.
  pub fn input_file(&self) -> PathBuf {
    PathBuf::from(&self.input_file)
  }

  /// Get the input file name without path.
  pub fn input_file_name(&self) -> String {
    self
      .input_file()
      .file_name()
      .unwrap()
      .to_string_lossy()
      .to_string()
  }

  /// Get the output file path.
  pub fn output_file(&self) -> PathBuf {
    PathBuf::from(&self.output_file)
  }

  /// Get the output file name without path or extension.
  pub fn output_file_name(&self) -> String {
    self
      .output_file()
      .file_prefix()
      .unwrap()
      .to_string_lossy()
      .to_string()
  }

  pub fn run_nonreg(self, md5_refs: &Arc<MD5References>) {
    let index = self.index;
    if self.is_encode {
      self.run_nonreg_encode(index);
    } else {
      self.run_nonreg_decode(index, md5_refs);
    }
  }

  pub fn run_nonreg_encode(self, test_index: usize) {
    // Get the input filename without path or extension.
    let input_filename = self.input_file_name();
    // Get the output filename without path or extension.
    let output_name_we = self.output_file_name();
    let output_filename = self.output_file.clone();
    eprintln!("Running test: {:?}", self);
    let is_lossless = self.command[0].contains("_lossless");
    let command_arg_n = self.command.len() - 1;

    // Encode an image into the jpeg2000 format
    println!("NR-ENC-{}-{}-encode", input_filename, test_index);
    let result = run_compress(self.command.clone());

    if self.should_fail {
      assert!(result.is_err(), "Test was expected to fail but succeeded");
      return;
    } else {
      result.expect("Compression failed unexpectedly");
    }

    // Dump the encoding file
    println!("NR-ENC-{}-{}-dump", input_filename, test_index);
    let dump_file = format!("{}-ENC-{}.txt", &output_filename, test_index);
    run_dump(args!["-i", &self.output_file, "-o", &dump_file])
      .expect("Dumping failed unexpectedly");

    // Compare the dump file with the baseline
    println!("NR-ENC-{}-{}-compare_dump2base", input_filename, test_index);

    run_compare_dump_files(args![
      "-b",
      format!(
        "{}/opj_v2_{}-ENC-{}.txt",
        common::get_baseline_nr_dir().to_string_lossy(),
        output_name_we,
        test_index
      ),
      "-t",
      &dump_file
    ])
    .expect("Dump file comparison with baseline failed");

    // Do lossy check by decoding the encoded file and comparing with the original
    if let Some(lossy_check) = self.lossy_check {
      // Decode the encoded file
      println!("NR-ENC-{}-{}-decode-ref", input_filename, test_index);
      let decoded_file = format!("{}.tif", &output_filename);
      run_decompress(args!["-i", &self.output_file, "-o", &decoded_file])
        .expect("Decompression failed unexpectedly");

      // Compare the decoded file with the original input file
      println!(
        "NR-ENC-{}-{}-compare_dec-ref-out2base",
        input_filename, test_index
      );
      let mut args = args!["-b", &self.input_file, "-t", &decoded_file, "-s", "bXtY"];
      args.extend(lossy_check.into_iter());
      run_compare_images(args).expect("Image comparison failed unexpectedly");
    }

    // If lossless compression (simple test is 4 arguments), decompress & compare
    if command_arg_n == 4 || is_lossless {
      // can we compare with the input image ?
      if self.input_file.ends_with(".tif") {
        // Lossless: decode and compare with original
        println!("NR-ENC-{}-{}-lossless-decode", input_filename, test_index);
        let output = format!("{}-lossless.tif", &output_filename);
        run_decompress(args!["-i", &self.output_file, "-o", &output])
          .expect("Decompression failed unexpectedly");
        println!("NR-ENC-{}-{}-lossless-compare", input_filename, test_index);
        run_compare_images(args![
          "-b",
          &self.input_file,
          "-t",
          &output,
          "-n",
          "1",
          "-d"
        ])
        .expect("Image comparison failed unexpectedly");
      }
    }
  }

  pub fn run_nonreg_decode(self, test_index: usize, md5_refs: &Arc<MD5References>) {
    // Get the input filename without path.
    let input_filename = self.input_file_name();
    eprintln!("Running test: {:?}", self);
    println!("NR-DEC-{}-{}-decode", input_filename, test_index);
    let result = run_decompress(self.command.clone());

    if self.should_fail {
      assert!(result.is_err(), "Test was expected to fail but succeeded");
      return;
    } else {
      result.expect("Decompression failed unexpectedly");
    }

    // Check MD5 of output file against reference
    println!("NR-DEC-{}-{}-decode-md5", input_filename, test_index);
    let result = md5_refs.check_output_files_md5(self.output_file());

    if let Err(err) = result {
      // Check for expected md5 failure
      match input_filename.as_str() {
        "issue205.jp2" | "issue208.jp2" | "issue226.j2k" => {
          eprintln!("Expected MD5 mismatch for {}", input_filename);
        }
        _ => panic!("MD5 check failed: {}", err),
      }
    }
  }
}

/// Parse and filter test commands from the test commands file
///
/// The test commands file has both encode and decode tests. This function
/// extracts encode or decode tests based on the `is_encode` flag.
///
/// Tests starting with '!' should fail.
///
/// For encode tests also parse out the `lossy-check { ... }` options if present.
///
/// Input and temp paths should be replaced before returning the commands.
///
/// Example test commands file snippet:
/// ```
/// # issue 843 Crash with invalid ppm file
/// !opj_compress -i @INPUT_NR_PATH@/issue843.ppm -o @TEMP_PATH@/issue843.ppm.jp2
///
/// # Test all 6 coding options individually
/// opj_compress -i @INPUT_NR_PATH@/Bretagne2.ppm -o @TEMP_PATH@/Bretagne2_vsc.j2k -M 8
/// # related to issue 62
/// opj_compress -i @INPUT_NR_PATH@/tmp-issue-0062.raw -o @TEMP_PATH@/tmp-issue-0062-u.raw.j2k -F 512,512,1,16,u
/// opj_compress -i @INPUT_NR_PATH@/tmp-issue-0062.raw -o @TEMP_PATH@/tmp-issue-0062-s.raw.j2k -F 512,512,1,16,s
/// opj_compress lossy-check { -n 3 -i prec -m 175:100:212 -p 79:64:92 } -i @INPUT_NR_PATH@/X_4_2K_24_185_CBR_WB_000.tif -o @TEMP_PATH@/X_4_2K_24_185_CBR_WB_000_C2K_24.j2k -cinema2K 24
/// opj_compress lossy-check { -n 3 -i prec -m 298:168:363 -p 122:73:164 } -i @INPUT_NR_PATH@/X_5_2K_24_235_CBR_STEM24_000.tif -o @TEMP_PATH@/X_5_2K_24_235_CBR_STEM24_000_C2K_24.j2k -cinema2K 24
/// ```
pub fn parse_test_commands() -> Vec<TestCommand> {
  let mut tests = Vec::new();
  let mut encode_index = 0;
  let mut decode_index = 0;
  let lines = OPJ_TEST_CMD_LIST.lines();

  let input_nr_path = get_input_nr_dir().to_string_lossy().to_string();
  let input_conformance_path = get_input_conformance_dir().to_string_lossy().to_string();
  let temp_path = get_temp_dir().to_string_lossy().to_string();

  let mut lines_iter = lines.peekable();
  while let Some(line) = lines_iter.next() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }

    // Replace placeholders
    let line = line
      .replace("@INPUT_NR_PATH@", &input_nr_path)
      .replace("@INPUT_CONF_PATH@", &input_conformance_path)
      .replace("@TEMP_PATH@", &temp_path);

    let mut should_fail = false;
    let mut command_line = line.to_string();

    if command_line.starts_with('!') {
      should_fail = true;
      command_line = command_line[1..].trim().to_string();
    }

    let is_encode = command_line.starts_with("opj_compress");

    let index = if is_encode {
      encode_index += 1;
      encode_index
    } else {
      decode_index += 1;
      decode_index
    };

    let mut lossy_check = None;

    if is_encode && command_line.contains("lossy-check") {
      let mut parts = command_line.splitn(2, "lossy-check");
      let before_lossy = parts.next().unwrap().trim();
      let after_lossy = parts.next().unwrap().trim();

      if let Some(start_brace) = after_lossy.find('{') {
        if let Some(end_brace) = after_lossy.find('}') {
          let lossy_options = &after_lossy[start_brace + 1..end_brace].trim();
          lossy_check = Some(lossy_options.split_whitespace().map(String::from).collect());

          command_line = format!("{} {}", before_lossy, after_lossy[end_brace + 1..].trim());
        }
      }
    }

    let command = command_line
      .split_whitespace()
      .map(String::from)
      .collect::<Vec<_>>();

    // Find the input and output file paths if present
    let mut input_file = None;
    let mut output_file = None;
    let mut args_iter = command.iter();
    while let Some(arg) = args_iter.next() {
      if arg == "-i" {
        if let Some(input) = args_iter.next() {
          input_file = Some(input.clone());
        }
      } else if arg == "-o" {
        if let Some(output) = args_iter.next() {
          output_file = Some(output.clone());
        }
      }
    }

    tests.push(TestCommand {
      command,
      is_encode,
      index,
      input_file: input_file.expect("Input file not specified").to_string(),
      output_file: output_file.expect("Output file not specified").to_string(),
      should_fail,
      lossy_check,
    });
  }

  tests
}

#[allow(dead_code)]
pub fn parse_and_filter_test_commands(is_encode: bool) -> Vec<TestCommand> {
  let tests = parse_test_commands();
  tests
    .into_iter()
    .filter(|test| test.is_encode == is_encode)
    .collect()
}

/// Get the test data root directory
pub fn get_test_data_root() -> PathBuf {
  // Check environment variable first
  if let Ok(data_root) = env::var("OPJ_DATA_ROOT") {
    return PathBuf::from(data_root);
  }

  // Default to the data directory in the repository
  let manifest_dir = env::var("CARGO_MANIFEST_DIR")
    .map(PathBuf::from)
    .or_else(|_| env::current_dir())
    .expect("Failed to get cargo manifest dir or current directory");
  manifest_dir
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .join("data")
}

/// Check if test data is available
pub fn has_test_data() -> bool {
  get_test_data_root().exists()
}

/// Skip test if test data is not available
#[macro_export]
macro_rules! skip_without_test_data {
  () => {
    if !common::has_test_data() {
      eprintln!("Skipping test: OPJ_DATA_ROOT not set or data directory not found");
      eprintln!("Set OPJ_DATA_ROOT environment variable to enable these tests");
      return;
    }
  };
}

/// Get input test data directory
pub fn get_input_dir() -> PathBuf {
  get_test_data_root().join("input")
}

/// Get input non-regression test data directory
pub fn get_input_nr_dir() -> PathBuf {
  get_input_dir().join("nonregression")
}

/// Get input conformance test data directory
pub fn get_input_conformance_dir() -> PathBuf {
  get_input_dir().join("conformance")
}

/// Get baseline test data directory
pub fn get_baseline_dir() -> PathBuf {
  get_test_data_root().join("baseline")
}

/// Get baseline non-regression test data directory
pub fn get_baseline_nr_dir() -> PathBuf {
  get_baseline_dir().join("nonregression")
}

/// Get temporary output directory for tests
pub fn get_temp_dir() -> PathBuf {
  let temp = env::temp_dir().join("openjpeg_tests");
  std::fs::create_dir_all(&temp).unwrap();
  temp
}

/// Compute MD5 hash of a file
pub fn md5_file<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
  use std::fs::File;
  use std::io::Read;

  let mut file = File::open(path)?;
  let mut context = md5::Context::new();
  let mut buffer = [0; 8192];

  loop {
    let count = file.read(&mut buffer)?;
    if count == 0 {
      break;
    }
    context.consume(&buffer[..count]);
  }

  Ok(format!("{:x}", context.finalize()))
}

/// Helper to build argument vector for CLI functions
#[macro_export]
macro_rules! args {
    ($($arg:expr),* $(,)?) => {
        vec![
            String::from("test"),  // Program name
            $(String::from($arg)),*
        ]
    };
}
