#![allow(deprecated)] // the CLI tools drive the low-level Codec API (tile decode etc.)
pub mod cli;

/// Installs the global rayon pool sized from `OPJ_NUM_THREADS` (the C tools'
/// variable); `RAYON_NUM_THREADS`, when set, takes precedence and is read by
/// rayon itself.
/// Sizes the global rayon pool: the explicit `threads` argument (the
/// `-threads` CLI flag) wins; otherwise `OPJ_NUM_THREADS` (the C tools'
/// variable); `RAYON_NUM_THREADS`, when set, takes precedence over both
/// and is read by rayon itself.
pub fn configure_thread_pool(threads: usize) {
  if threads > 0 {
    let _ = rayon::ThreadPoolBuilder::new().num_threads(threads).build_global();
    return;
  }
  if std::env::var("RAYON_NUM_THREADS").is_ok() {
    return;
  }
  let Ok(env_threads) = std::env::var("OPJ_NUM_THREADS") else {
    return;
  };
  let Ok(n) = env_threads.parse::<usize>() else {
    return;
  };
  if n > 0 {
    // build_global only succeeds once per process; that is all we need.
    let _ = rayon::ThreadPoolBuilder::new().num_threads(n).build_global();
  }
}
// Common code shared between the openjp2 CLI tools.
pub mod color;
pub mod convert;
pub mod getopt;
pub mod params;

// Compression formats
pub const J2K_CFMT: u32 = 0;
pub const JP2_CFMT: u32 = 1;
pub const JPT_CFMT: u32 = 2;

// Decompression formats
pub const PXM_DFMT: i32 = 10;
pub const PGX_DFMT: i32 = 11;
pub const BMP_DFMT: i32 = 12;
pub const YUV_DFMT: i32 = 13;
pub const TIF_DFMT: i32 = 14;
pub const RAW_DFMT: i32 = 15; // MSB / Big Endian
pub const TGA_DFMT: i32 = 16;
pub const PNG_DFMT: i32 = 17;
pub const RAWL_DFMT: i32 = 18; // LSB / Little Endian

#[cfg(feature = "testing")]
#[macro_use]
pub mod testing;
