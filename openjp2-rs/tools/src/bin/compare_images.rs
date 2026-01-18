use openjp2_tools::cli::run_compare_images;

fn main() -> Result<(), String> {
  run_compare_images(std::env::args().collect())
}
