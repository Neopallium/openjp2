/// Tests for OpenJPEG conformance
use openjp2_tools::{
  args,
  cli::{run_test_decode_area, run_test_tile_decoder, run_test_tile_encoder},
  testing::get_temp_dir,
};

fn test_tile_encode_decode(
  num: usize,
  test_file: &str,
  encoder_args: Vec<String>,
  decoder_args: Option<Vec<String>>,
) -> Result<(), String> {
  let temp_dir = get_temp_dir().to_string_lossy().to_string();
  let tile_file = format!("{temp_dir}/{test_file}");
  println!("tte{num}");
  run_test_tile_encoder(encoder_args, &tile_file)?;
  if let Some(decoder_args) = decoder_args {
    println!("ttd{num}");
    run_test_tile_decoder(decoder_args, &tile_file)?;
    return Ok(());
  }
  Ok(())
}

#[test]
fn test_tile_encoder() -> Result<(), String> {
  test_tile_encode_decode(0, "test.j2k", args![], Some(args![]))?;

  test_tile_encode_decode(
    1,
    "tte1.j2k",
    args!["3", "2048", "2048", "1024", "1024", "8", "1"],
    Some(args!["0", "0", "1024", "1024"]),
  )?;

  test_tile_encode_decode(
    2,
    "tte2.jp2",
    args!["3", "2048", "2048", "1024", "1024", "8", "1"],
    Some(args!["0", "0", "1024", "1024"]),
  )?;

  test_tile_encode_decode(
    3,
    "tte3.j2k",
    args!["1", "2048", "2048", "1024", "1024", "8", "1"],
    None,
  )?;

  test_tile_encode_decode(
    4,
    "tte4.j2k",
    args!["1", "256", "256", "128", "128", "8", "0"],
    None,
  )?;

  test_tile_encode_decode(
    5,
    "tte5.j2k",
    args!["1", "512", "512", "256", "256", "8", "0"],
    None,
  )?;

  Ok(())
}

fn run_test_tda(
  name: &str,
  mut tile_encoder_args: Vec<String>,
  tile_encoder_after_file: &[&str],
  mut decode_area_args: Vec<String>,
) -> Result<(), String> {
  let temp_dir = get_temp_dir().to_string_lossy().to_string();
  let test_file = format!("{}/{}.j2k", temp_dir, name);
  println!("tda_prep_{name}");
  tile_encoder_args.push(test_file.clone());
  tile_encoder_args.extend(tile_encoder_after_file.iter().map(|s| s.to_string()));
  run_test_tile_encoder(tile_encoder_args, "test.j2k")?;
  println!("tda_{name}");
  decode_area_args.push(test_file);
  run_test_decode_area(decode_area_args)?;
  Ok(())
}

#[test]
fn tda_reversible_no_precinct() -> Result<(), String> {
  // add_test(NAME tda_prep_reversible_no_precinct COMMAND test_tile_encoder 1 256 256 32 32 8 0 reversible_no_precinct.j2k 4 4 3 0 0 1)
  // add_test(NAME tda_reversible_no_precinct COMMAND test_decode_area -q reversible_no_precinct.j2k)
  run_test_tda(
    "reversible_no_precinct",
    args!["1", "256", "256", "32", "32", "8", "0",],
    &["4", "4", "3", "0", "0", "1"],
    args!["-q"],
  )
}

#[test]
fn tda_reversible_203_201_17_19_no_precinct() -> Result<(), String> {
  // add_test(NAME tda_prep_reversible_203_201_17_19_no_precinct COMMAND test_tile_encoder 1 203 201 17 19 8 0 reversible_203_201_17_19_no_precinct.j2k 4 4 3 0 0 1)
  // add_test(NAME tda_reversible_203_201_17_19_no_precinct COMMAND test_decode_area -q reversible_203_201_17_19_no_precinct.j2k)
  run_test_tda(
    "reversible_203_201_17_19_no_precinct",
    args!["1", "203", "201", "17", "19", "8", "0",],
    &["4", "4", "3", "0", "0", "1"],
    args!["-q"],
  )
}

#[test]
fn tda_reversible_with_precinct() -> Result<(), String> {
  // add_test(NAME tda_prep_reversible_with_precinct COMMAND test_tile_encoder 1 256 256 32 32 8 0 reversible_with_precinct.j2k 4 4 3 0 0 1 16 16)
  // add_test(NAME tda_reversible_with_precinct COMMAND test_decode_area -q reversible_with_precinct.j2k)
  run_test_tda(
    "reversible_with_precinct",
    args!["1", "256", "256", "32", "32", "8", "0",],
    &["4", "4", "3", "0", "0", "1", "16", "16"],
    args!["-q"],
  )
}

#[test]
fn tda_irreversible_no_precinct() -> Result<(), String> {
  // add_test(NAME tda_prep_irreversible_no_precinct COMMAND test_tile_encoder 1 256 256 32 32 8 1 irreversible_no_precinct.j2k 4 4 3 0 0 1)
  // add_test(NAME tda_irreversible_no_precinct COMMAND test_decode_area -q irreversible_no_precinct.j2k)
  run_test_tda(
    "irreversible_no_precinct",
    args!["1", "256", "256", "32", "32", "8", "1",],
    &["4", "4", "3", "0", "0", "1"],
    args!["-q"],
  )
}

#[test]
fn tda_irreversible_203_201_17_19_no_precinct() -> Result<(), String> {
  // add_test(NAME tda_prep_irreversible_203_201_17_19_no_precinct COMMAND test_tile_encoder 1 203 201 17 19 8 1 irreversible_203_201_17_19_no_precinct.j2k 4 4 3 0 0 1)
  // add_test(NAME tda_irreversible_203_201_17_19_no_precinct COMMAND test_decode_area -q irreversible_203_201_17_19_no_precinct.j2k)
  run_test_tda(
    "irreversible_203_201_17_19_no_precinct",
    args!["1", "203", "201", "17", "19", "8", "1",],
    &["4", "4", "3", "0", "0", "1"],
    args!["-q"],
  )
}

#[test]
fn tda_strip() -> Result<(), String> {
  // add_test(NAME tda_prep_strip COMMAND test_tile_encoder 1 256 256 256 256 8 0 tda_single_tile.j2k)
  // add_test(NAME tda_strip COMMAND test_decode_area -q -strip_height 3 -strip_check tda_single_tile.j2k)
  run_test_tda(
    "strip",
    args!["1", "256", "256", "256", "256", "8", "0",],
    &[],
    args!["-q", "-strip_height", "3"],
  )
}
