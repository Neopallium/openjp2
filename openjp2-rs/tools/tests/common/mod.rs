use std::env;
/// Common test utilities for integration tests
use std::path::{Path, PathBuf};

const OPJ_TEST_CMD_LIST: &str = include_str!("../../../../tests/nonregression/test_suite.ctest.in");

#[derive(Debug)]
pub struct TestCommand {
  pub command: Vec<String>,
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

  /// Get the input file name without path or extension.
  pub fn input_file_name(&self) -> String {
    self
      .input_file()
      .file_prefix()
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
pub fn parse_and_filter_test_commands(is_encode: bool) -> Vec<TestCommand> {
  let mut tests = Vec::new();
  let lines = OPJ_TEST_CMD_LIST.lines();

  let input_nr_path = get_input_nr_dir().to_string_lossy().to_string();
  let temp_path = get_temp_dir().to_string_lossy().to_string();

  let command_prefix = if is_encode {
    "opj_compress"
  } else {
    "opj_decompress"
  };

  let mut lines_iter = lines.peekable();
  while let Some(line) = lines_iter.next() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }

    // Replace placeholders
    let line = line
      .replace("@INPUT_NR_PATH@", &input_nr_path)
      .replace("@TEMP_PATH@", &temp_path);

    let mut should_fail = false;
    let mut command_line = line.to_string();

    if command_line.starts_with('!') {
      should_fail = true;
      command_line = command_line[1..].trim().to_string();
    }

    if !command_line.starts_with(command_prefix) {
      continue;
    }

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
      input_file: input_file.expect("Input file not specified").to_string(),
      output_file: output_file.expect("Output file not specified").to_string(),
      should_fail,
      lossy_check,
    });
  }

  tests
}

/// Get the test data root directory
pub fn get_test_data_root() -> PathBuf {
  // Check environment variable first
  if let Ok(data_root) = env::var("OPJ_DATA_ROOT") {
    return PathBuf::from(data_root);
  }

  // Default to the data directory in the repository
  let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
  PathBuf::from(manifest_dir)
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

/// Get baseline test data directory
pub fn get_baseline_dir() -> PathBuf {
  get_test_data_root().join("baseline")
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
    let n = file.read(&mut buffer)?;
    if n == 0 {
      break;
    }
    context.consume(&buffer[..n]);
  }

  Ok(format!("{:x}", context.compute()))
}

/// Compare two files by MD5 hash
pub fn files_match_md5<P: AsRef<Path>>(file1: P, file2: P) -> Result<bool, std::io::Error> {
  let hash1 = md5_file(file1)?;
  let hash2 = md5_file(file2)?;
  Ok(hash1 == hash2)
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
