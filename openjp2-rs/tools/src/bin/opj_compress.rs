use openjp2_tools::cli::run_compress;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  run_compress(std::env::args().collect())
}
