use openjp2_tools::cli::run_test_decode_area;

fn main() -> Result<(), String> {
  run_test_decode_area(std::env::args().collect())
}
