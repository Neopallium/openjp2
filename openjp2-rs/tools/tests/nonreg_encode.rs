/// Integration tests for non-regression encode tests
/// These tests verify compression functionality against known good baselines
#[macro_use]
mod common;

use openjp2_tools::cli::{
  run_compare_dump_files, run_compare_images, run_compress, run_decompress, run_dump,
};

#[test]
fn test_nonreg_encode() {
  skip_without_test_data!();

  let tests = common::parse_and_filter_test_commands(true);

  for (index, test) in tests.iter().enumerate() {
    let test_indx = index + 1;
    // Get the input filename without path or extension.
    let input_filename = test.input_file_name();
    // Get the output filename without path or extension.
    let output_filename = test.output_file_name();

    eprintln!("Running test: {:?}", test);

    // Encode an image into the jpeg2000 format
    println!("NR-ENC-{}-{}-encode", input_filename, test_indx);
    let result = run_compress(test.command.clone());

    if test.should_fail {
      assert!(result.is_err(), "Test was expected to fail but succeeded");
      continue;
    } else {
      result.expect("Compression failed unexpectedly");
    }

    // Dump the encoding file
    println!("NR-ENC-{}-{}-dump", input_filename, test_indx);
    let dump_file = format!("{}-ENC-{}.txt", &output_filename, test_indx);
    run_dump(args!["-i", &test.output_file, "-o", &dump_file])
      .expect("Dumping failed unexpectedly");

    // Compare the dump file with the baseline
    println!("NR-ENC-{}-{}-compare_dump2base", input_filename, test_indx);

    run_compare_dump_files(args![
      "-b",
      format!(
        "{}/opj_v2_{}-ENC-{}.txt",
        common::get_test_data_root()
          .join("baseline/nonregression")
          .to_string_lossy(),
        output_filename,
        test_indx
      ),
      "-t",
      &dump_file
    ])
    .expect("Dump file comparison with baseline failed");
  }
}
