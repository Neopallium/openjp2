//! Writes small valid codestreams (J2K and JP2, reversible and
//! irreversible, gray and RGB) into `fuzz/corpus/fuzz_decode/` so the
//! fuzzer starts from structurally valid inputs instead of noise.
//!
//! Run: `cargo run --bin gen_seeds` from the `fuzz/` directory.

use openjp2::ffi::{OPJ_CLRSPC_GRAY, OPJ_CLRSPC_SRGB};
use openjp2::{CoreImage, CoreImageComp, EncodeParameters, Image};

fn xorshift(r: &mut u64) -> u64 {
  *r ^= *r << 13;
  *r ^= *r >> 7;
  *r ^= *r << 17;
  *r
}

fn main() {
  let mut r: u64 = 0x243f6a8885a308d3;
  let out = std::path::Path::new("corpus/fuzz_decode");
  std::fs::create_dir_all(out).unwrap();
  let mut n = 0usize;
  for &(w, h, comps, prec, rev) in &[
    (32u32, 32u32, 1u32, 8u32, true),
    (48, 24, 3, 8, true),
    (64, 64, 3, 12, false),
    (16, 48, 1, 16, true),
    (80, 16, 3, 8, false),
  ] {
    let mut core = CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: comps,
      color_space: if comps == 1 {
        OPJ_CLRSPC_GRAY
      } else {
        OPJ_CLRSPC_SRGB
      },
      comps: Vec::new(),
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    for c in 0..comps {
      let dx = if c == 0 { 1 } else { 2 };
      let (cw, ch) = (w.div_ceil(dx), h.div_ceil(dx));
      let mut data = Vec::with_capacity((cw * ch) as usize);
      for _ in 0..cw * ch {
        data.push((xorshift(&mut r) % (1u64 << prec)) as i32);
      }
      core.comps.push(CoreImageComp {
        dx,
        dy: dx,
        w: cw,
        h: ch,
        x0: 0,
        y0: 0,
        prec,
        bpp: prec,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data,
      });
    }
    let image = Image::from_core_image(core);
    let params = EncodeParameters::new()
      .num_resolutions(2)
      .num_layers(2)
      .irreversible(!rev);
    for (fmt, cs) in [
      ("j2k", image.clone().encode(params.clone())),
      ("jp2", image.encode_jp2(params.clone())),
    ] {
      if let Ok(cs) = cs {
        let name = format!("seed{:02}_{}x{}_{}c_{}b_{}.{}", n, w, h, comps, prec, rev, fmt);
        println!("{} ({} bytes)", name, cs.len());
        std::fs::write(out.join(&name), &cs).unwrap();
        n += 1;
      }
    }
  }
}
