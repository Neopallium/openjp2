use openjp2_tools::cli::run_compare_raw_files;

fn main() -> Result<(), String> {
  run_compare_raw_files(std::env::args().collect())
}
