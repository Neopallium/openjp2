use openjp2_tools::cli::run_compare_raw_files;

fn main() -> Result<(), String> {
  let success = run_compare_raw_files(std::env::args().collect())?;
  std::process::exit(if success { 0 } else { 1 });
}
