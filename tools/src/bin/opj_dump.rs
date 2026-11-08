use openjp2_tools::cli::run_dump;
use openjp2_tools::configure_thread_pool;

fn main() -> Result<(), String> {
  configure_thread_pool();
    run_dump(std::env::args().collect())
}
