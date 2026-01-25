use openjp2_tools::cli::run_test_tile_encoder;

fn main() -> Result<(), String> {
  run_test_tile_encoder(std::env::args().collect())
}
