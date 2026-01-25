/// Tests for OpenJPEG conformance
use openjp2_tools::{
  args,
  cli::{run_compare_dump_files, run_compare_images, run_decompress, run_dump},
  skip_without_test_data,
  testing::*,
};
#[cfg(feature = "parallel-tests")]
use rayon::prelude::*;

const CP0_NBC_LIST: &[&str] = &[
  "not_used", "1", "1", "1", "3", "4", "4", "3", "3", "1", "3", "1", "1", "4", "3", "1", "1",
];
const CP1_NBC_LIST: &[&str] = &[
  "not_used", "1", "3", "4", "1", "3", "3", "2", "2", "1", "2", "1", "1", "3", "2", "1", "1",
];

// --------------------------------------------------------------------------
// Tests about class 1 profile 0
// try to decode
// compare to ref file
// non regression comparison

// Parameters and tolerances given by Table C.6
const C1P0_RESFACTOR_LIST: &[&str] = &[
  "not_used", "0", "0", "0", "0", "0", "0", "0", "1", "0", "0", "0", "0", "0", "0", "0", "0",
];
const C1P0_PEAK_LIST: &[&str] = &[
  "not_used",
  "0",
  "0",
  "0",
  "5:4:6",
  "2:2:2:0",
  "635:403:378:0",
  "0:0:0",
  "0:0:0",
  "0",
  "0:0:0",
  "0",
  "0",
  "0:0:0:0",
  "0:0:0",
  "0",
  "0",
];
const C1P0_MSE_LIST: &[&str] = &[
  "not_used",
  "0",
  "0",
  "0",
  "0.776:0.626:1.070",
  "0.302:0.307:0.269:0",
  "11287:6124:3968:0",
  "0:0:0",
  "0:0:0",
  "0",
  "0:0:0",
  "0",
  "0",
  "0:0:0:0",
  "0:0:0",
  "0",
  "0",
];

#[test]
fn test_conformance_c1p0() {
  skip_without_test_data!();

  let input_conf_dir = get_input_conf_dir().to_string_lossy().to_string();
  let baseline_conf_dir = get_baseline_conf_dir().to_string_lossy().to_string();
  let baseline_nr_dir = get_baseline_nr_dir().to_string_lossy().to_string();
  let temp_dir = get_temp_dir().to_string_lossy().to_string();

  for num in 1..=16 {
    let input_file = format!("p0_{:02}.j2k", num);
    let ref_file = format!("c1p0_{:02}.pgx", num);
    let input = format!("{input_conf_dir}/{input_file}");
    let output = format!("{temp_dir}/c1{input_file}.pgx");

    // # Get corresponding tests parameters
    let nbc = CP0_NBC_LIST[num];
    let resfactor = C1P0_RESFACTOR_LIST[num];
    let peak = C1P0_PEAK_LIST[num];
    let mse = C1P0_MSE_LIST[num];

    println!("ETS-C1P0-{input_file}-decode");
    run_decompress(args!["-i", input, "-o", &output, "-r", resfactor,])
      .expect("Decompression failed");

    println!("ETS-C1P0-{input_file}-compare2ref");
    run_compare_images(args![
      "-b",
      format!("{baseline_conf_dir}/{ref_file}"),
      "-t",
      &output,
      "-n",
      nbc,
      "-p",
      peak,
      "-m",
      mse,
      "-s",
      "b_t_",
    ])
    .expect("Image comparison failed");

    println!("NR-C1P0-{input_file}-compare2base");
    run_compare_images(args![
      "-b",
      format!("{baseline_nr_dir}/opj_{ref_file}"),
      "-t",
      &output,
      "-n",
      nbc,
      "-d",
      "-s",
      "b_t_",
    ])
    .expect("Image comparison to baseline failed");
  }
}

// --------------------------------------------------------------------------
// Tests about class 1 profile 1
// try to decode
// compare to ref file
// non regression comparison

// Parameters and tolerances given by Table C.7
const C1P1_PEAK_LIST: &[&str] = &[
  "not_used", "0", "5:4:6", "2:2:1:0", "624", "40:40:40", "2:2:2", "0:0",
];
const C1P1_MSE_LIST: &[&str] = &[
  "not_used",
  "0",
  "0.765:0.616:1.051",
  "0.3:0.210:0.200:0",
  "3080",
  "8.458:9.816:10.154",
  "0.6:0.6:0.6",
  "0:0",
];

#[test]
fn test_conformance_c1p1() {
  skip_without_test_data!();

  let input_conf_dir = get_input_conf_dir().to_string_lossy().to_string();
  let baseline_conf_dir = get_baseline_conf_dir().to_string_lossy().to_string();
  let baseline_nr_dir = get_baseline_nr_dir().to_string_lossy().to_string();
  let temp_dir = get_temp_dir().to_string_lossy().to_string();

  for num in 1..=7 {
    let input_file = format!("p1_{:02}.j2k", num);
    let ref_file = format!("c1p1_{:02}.pgx", num);
    let input = format!("{input_conf_dir}/{input_file}");
    let output = format!("{temp_dir}/c1{input_file}.pgx");

    // Get corresponding tests parameters
    let nbc = CP1_NBC_LIST[num];
    let peak = C1P1_PEAK_LIST[num];
    let mse = C1P1_MSE_LIST[num];

    println!("ETS-C1P1-{input_file}-decode");
    run_decompress(args!["-i", input, "-o", &output, "-r", "0",]).expect("Decompression failed");

    println!("ETS-C1P1-{input_file}-compare2ref");
    run_compare_images(args![
      "-b",
      format!("{baseline_conf_dir}/{ref_file}"),
      "-t",
      &output,
      "-n",
      nbc,
      "-p",
      peak,
      "-m",
      mse,
      "-s",
      "b_t_",
    ])
    .expect("Image comparison failed");

    println!("NR-C1P1-{input_file}-compare2base");
    run_compare_images(args![
      "-b",
      format!("{baseline_nr_dir}/opj_{ref_file}"),
      "-t",
      &output,
      "-n",
      nbc,
      "-d",
      "-s",
      "b_t_",
    ])
    .expect("Image comparison to baseline failed");
  }
}

// --------------------------------------------------------------------------
// Tests about JP2 file
// try to decode
// compare to ref file
// non regression comparison
//
// Tolerances given by Part 4 - Table G.1
// Peak is set to 4 only

#[test]
fn test_conformance_jp2() {
  skip_without_test_data!();

  let input_conf_dir = get_input_conf_dir().to_string_lossy().to_string();
  let baseline_conf_dir = get_baseline_conf_dir().to_string_lossy().to_string();
  let baseline_nr_dir = get_baseline_nr_dir().to_string_lossy().to_string();
  let temp_dir = get_temp_dir().to_string_lossy().to_string();

  for num in 1..=9 {
    let input_file = format!("file{}.jp2", num);
    let ref_file = format!("jp2_{}.tif", num);
    let input = format!("{input_conf_dir}/{input_file}");
    let output = format!("{temp_dir}/{input_file}.tif");

    println!("ETS-JP2-{input_file}-decode");
    run_decompress(args!["-i", input, "-o", &output, "-p", "8S", "-force-rgb",])
      .expect("Decompression failed");

    println!("ETS-JP2-{input_file}-compare2ref");
    run_compare_images(args![
      "-b",
      format!("{baseline_conf_dir}/{ref_file}"),
      "-t",
      &output,
      "-n",
      "3",
      "-p",
      "4:4:4",
      "-m",
      "1:1:1",
      "-s",
      "b_t_",
    ])
    .expect("Image comparison failed");

    println!("NR-JP2-{input_file}-compare2base");
    run_compare_images(args![
      "-b",
      format!("{baseline_nr_dir}/opj_{ref_file}"),
      "-t",
      &output,
      "-n",
      "3",
      "-d",
      "-s",
      "b_t_",
    ])
    .expect("Image comparison to baseline failed");
  }
}

// --------------------------------------------------------------------------
// Tests about Kakadu/J2K file
// try to decode
// compare to ref file
// non regression comparison

#[test]
fn test_conformance_kakadu_j2k() {
  skip_without_test_data!();

  let input_conf_dir = get_input_conf_dir().to_string_lossy().to_string();
  let baseline_conf_dir = get_baseline_conf_dir().to_string_lossy().to_string();
  let baseline_nr_dir = get_baseline_nr_dir().to_string_lossy().to_string();
  let temp_dir = get_temp_dir().to_string_lossy().to_string();

  let kdu_j2k_conf_files = vec![
    "a1_mono",
    "a2_colr",
    "a3_mono",
    "a4_colr",
    "a5_mono",
    "a6_mono_colr",
    "b1_mono",
    "b2_mono",
    "b3_mono",
    "c1_mono",
    "c2_mono",
    "d1_colr",
    "d2_colr",
    "e1_colr",
    "e2_colr",
    "f1_mono",
    "f2_mono",
    "g1_colr",
    "g2_colr",
    "g3_colr",
    "g4_colr",
  ];

  for kdu_file in kdu_j2k_conf_files {
    let input_file = format!("{kdu_file}.j2c");
    let ref_file = format!("{kdu_file}.ppm");
    let input = format!("{input_conf_dir}/{input_file}");
    let output = format!("{temp_dir}/{input_file}.ppm");

    let is_a6_mono_colr = kdu_file == "a6_mono_colr";

    println!("ETS-KDU-{input_file}-decode");
    if is_a6_mono_colr {
      run_decompress(args!["-i", input, "-o", &output, "-upsample", "-split-pnm",])
        .expect("Decompression failed");
    } else {
      run_decompress(args!["-i", input, "-o", &output, "-upsample",])
        .expect("Decompression failed");
    }

    println!("ETS-KDU-{input_file}-compare2ref");
    if is_a6_mono_colr {
      run_compare_images(args![
        "-b",
        format!("{baseline_conf_dir}/{ref_file}"),
        "-t",
        &output,
        "-n",
        "4",
        "-p",
        "4:4:4:4",
        "-m",
        "1:1:1:1",
        "-s",
        "b_t_",
      ])
      .expect("Image comparison failed");
    } else {
      run_compare_images(args![
        "-b",
        format!("{baseline_conf_dir}/{ref_file}"),
        "-t",
        &output,
        "-n",
        "1",
        "-p",
        "4:4:4",
        "-m",
        "1:1:1",
      ])
      .expect("Image comparison failed");
    }

    println!("NR-KDU-{input_file}-compare2base");
    if is_a6_mono_colr {
      run_compare_images(args![
        "-b",
        format!("{baseline_nr_dir}/opj_{ref_file}"),
        "-t",
        &output,
        "-n",
        "4",
        "-d",
        "-s",
        "b_t_",
      ])
      .expect("Image comparison to baseline failed");
    } else {
      run_compare_images(args![
        "-b",
        format!("{baseline_nr_dir}/opj_{ref_file}"),
        "-t",
        &output,
        "-n",
        "1",
        "-d",
      ])
      .expect("Image comparison to baseline failed");
    }
  }
}

// --------------------------------------------------------------------------
// Tests about Richter/J2K file
// try to decode
// compare to ref file
// non regression comparison

#[test]
fn test_conformance_richter_jp2() {
  skip_without_test_data!();

  let input_conf_dir = get_input_conf_dir().to_string_lossy().to_string();
  let baseline_conf_dir = get_baseline_conf_dir().to_string_lossy().to_string();
  let baseline_nr_dir = get_baseline_nr_dir().to_string_lossy().to_string();
  let temp_dir = get_temp_dir().to_string_lossy().to_string();

  let richter_jp2_conf_files = vec!["subsampling_1", "subsampling_2", "zoo1", "zoo2"];

  for r_file in richter_jp2_conf_files {
    let input_file = format!("{r_file}.jp2");
    let ref_file = format!("{r_file}.ppm");
    let input = format!("{input_conf_dir}/{input_file}");
    let output = format!("{temp_dir}/{input_file}.ppm");

    println!("ETS-RIC-{input_file}-decode");
    run_decompress(args!["-i", input, "-o", &output,]).expect("Decompression failed");

    println!("ETS-RIC-{input_file}-compare2ref");
    run_compare_images(args![
      "-b",
      format!("{baseline_conf_dir}/{ref_file}"),
      "-t",
      &output,
      "-n",
      "1",
      "-p",
      "4:4:4",
      "-m",
      "2:2:2",
    ])
    .expect("Image comparison failed");

    println!("NR-RIC-{input_file}-compare2base");
    run_compare_images(args![
      "-b",
      format!("{baseline_nr_dir}/opj_{ref_file}"),
      "-t",
      &output,
      "-n",
      "1",
      "-d",
    ])
    .expect("Image comparison to baseline failed");
  }
}

// --------------------------------------------------------------------------
// Tests about dump of profile 0 file
// try to dump image and codestream information into a file
// non regression comparison this file to the baseline
fn image_compare2base(
  input_name_we: &str,
  ext: &str,
  input_dir: &str,
  baseline_nr_dir: &str,
  temp_dir: &str,
) {
  let input_name = format!("{}.{}", input_name_we, ext);
  let output = format!("{}/{}.txt", temp_dir, input_name);
  println!("NR-{}-dump", input_name);
  run_dump(args![
    "-i",
    format!("{}/{}", input_dir, input_name),
    "-o",
    &output,
    "-v"
  ])
  .expect("Dumping failed");

  println!("NR-{}-compare_dump2base", input_name);
  run_compare_dump_files(args![
    "-b",
    format!("{}/opj_v2_{}.txt", baseline_nr_dir, input_name_we),
    "-t",
    &output,
  ])
  .expect("Comparing dump to baseline failed");
}

#[test]
fn test_conformance_dump() {
  skip_without_test_data!();

  let input_conf_dir = get_input_conf_dir().to_string_lossy().to_string();
  let baseline_nr_dir = get_baseline_nr_dir().to_string_lossy().to_string();
  let temp_dir = get_temp_dir().to_string_lossy().to_string();

  let dump_conf_files = vec![
    ("p0_01", "j2k"),
    ("p0_02", "j2k"),
    ("p0_03", "j2k"),
    ("p0_04", "j2k"),
    ("p0_05", "j2k"),
    ("p0_06", "j2k"),
    ("p0_07", "j2k"),
    ("p0_08", "j2k"),
    ("p0_09", "j2k"),
    ("p0_10", "j2k"),
    ("p0_11", "j2k"),
    ("p0_12", "j2k"),
    ("p0_13", "j2k"),
    ("p0_14", "j2k"),
    ("p0_15", "j2k"),
    ("p0_16", "j2k"),
    ("p1_01", "j2k"),
    ("p1_02", "j2k"),
    ("p1_03", "j2k"),
    ("p1_04", "j2k"),
    ("p1_05", "j2k"),
    ("p1_06", "j2k"),
    ("p1_07", "j2k"),
    ("file1", "jp2"),
    ("file2", "jp2"),
    ("file3", "jp2"),
    ("file4", "jp2"),
    ("file5", "jp2"),
    ("file6", "jp2"),
    ("file7", "jp2"),
    ("file8", "jp2"),
    ("file9", "jp2"),
  ];

  for (dump_file, ext) in dump_conf_files {
    image_compare2base(dump_file, ext, &input_conf_dir, &baseline_nr_dir, &temp_dir);
  }
}
