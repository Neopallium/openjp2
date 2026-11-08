use openjp2_tools::cli::run_compress;
use openjp2_tools::configure_thread_pool;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  configure_thread_pool();
    run_compress(std::env::args().collect())
}
