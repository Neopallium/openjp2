use openjp2_tools::cli::run_test_tile_decoder;

fn main() -> Result<(), String> {
  run_test_tile_decoder(std::env::args().collect())
}
