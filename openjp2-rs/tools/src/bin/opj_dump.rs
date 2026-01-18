use openjp2_tools::cli::run_dump;

fn main() -> Result<(), String> {
  run_dump(std::env::args().collect())
}
