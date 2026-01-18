use openjp2_tools::cli::run_decompress;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  run_decompress(std::env::args().collect())
}
