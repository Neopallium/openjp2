use openjp2_tools::cli::run_compare_images;

fn main() -> Result<(), String> {
  let success = run_compare_images(std::env::args().collect())?;

  std::process::exit(if success { 0 } else { 1 });
}
