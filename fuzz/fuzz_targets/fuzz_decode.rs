//! libFuzzer target for the J2K/JP2 decoders.
//!
//! Every input is decoded in three modes - lenient (progressive-friendly),
//! strict, and limit-constrained (`DecodeLimits`).  None may panic or hang:
//! the decoders must reject malformed, truncated or malicious data
//! gracefully (the limit checks run before any allocation sized from the
//! codestream).
//!
//! Run: `cargo fuzz run fuzz_decode -- -max_len=131072`
#![no_main]
use libfuzzer_sys::fuzz_target;

use openjp2::{DecodeLimits, DecodeParameters, Image};

fuzz_target!(|data: &[u8]| {
  let _ = Image::from_bytes_with(data, DecodeParameters::new().strict(false));
  let _ = Image::from_bytes_with(data, DecodeParameters::new().strict(true));
  let limits = DecodeLimits::new()
    .max_image_size(2048, 2048)
    .max_tile_pixels(2048 * 2048)
    .max_mem_alloc(32 * 1024 * 1024)
    .max_metadata_len(1 << 20);
  let _ = Image::from_bytes_with(data, DecodeParameters::new().strict(false).limits(limits));
});
