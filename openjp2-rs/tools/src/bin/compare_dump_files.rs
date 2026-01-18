use openjp2_tools::cli::run_compare_dump_files;

fn main() -> Result<(), String> {
  run_compare_dump_files(std::env::args().collect())
}
