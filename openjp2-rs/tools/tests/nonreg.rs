/// Integration tests for non-regression tests
/// These tests verify compression and decompression functionality against known good baselines
#[macro_use]
mod common;

#[cfg(feature = "parallel-tests")]
use rayon::prelude::*;

#[test]
fn test_nonreg() {
  skip_without_test_data!();

  let tests = common::parse_test_commands();
  let md5_refs = common::MD5References::load_md5_references();

  #[cfg(feature = "parallel-tests")]
  {
    tests.into_par_iter().for_each(|test| {
      test.run_nonreg(&md5_refs);
    });
  }

  #[cfg(not(feature = "parallel-tests"))]
  {
    for test in tests {
      test.run_nonreg(&md5_refs);
    }
  }
}
