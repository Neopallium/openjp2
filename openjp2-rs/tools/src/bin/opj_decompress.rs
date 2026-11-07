use openjp2_tools::cli::run_decompress;
use openjp2_tools::configure_thread_pool;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  configure_thread_pool();
    run_decompress(std::env::args().collect())
}
