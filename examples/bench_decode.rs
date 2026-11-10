//! Quick decode/encode benchmark through the safe API.
//!
//! Usage: `cargo run --release --example bench_decode -- <file> [iterations]`
use openjp2::*;
use std::time::Instant;

fn main() -> Result<()> {
  let args: Vec<String> = std::env::args().collect();
  let path = args.get(1).map(String::as_str).unwrap_or("samples/Hadley_Crater.jp2");
  let n = args
    .get(2)
    .and_then(|s| s.parse::<u32>().ok())
    .unwrap_or(10);
  let buf = std::fs::read(path)?;
  let image = Image::from_bytes(&buf)?;
  println!("{}: {}x{} {:?}, {} components", path, image.width(), image.height(), image.color_space(), image.num_components());

  let t0 = Instant::now();
  for _ in 0..n {
    let _ = Image::from_bytes(&buf)?;
  }
  println!("decode: {:.3} ms/image", t0.elapsed().as_secs_f64() * 1e3 / n as f64);

  let params = EncodeParameters::new().num_resolutions(4).tile_size(512, 512);
  let t1 = Instant::now();
  for _ in 0..n {
    let _ = image.clone().encode(params.clone())?;
  }
  println!("encode (multi-tile): {:.3} ms/image", t1.elapsed().as_secs_f64() * 1e3 / n as f64);
  Ok(())
}
