//! Minimal encoder: loads an image the `image` crate can read (PNG, JPEG,
//! TIFF, ...) and writes a lossless or lossy J2K/JP2 file.
//!
//! ```sh
//! cargo run --example simple_encode -- input.png [output.j2k] [--lossy]
//! ```

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = std::env::args().skip(1).collect();
  if args.is_empty() {
    eprintln!("usage: simple_encode <input.png|jpg|...> [output.j2k|output.jp2] [--lossy]");
    std::process::exit(1);
  }
  let input = &args[0];
  let output = args.get(1).cloned().unwrap_or_else(|| "output.j2k".into());
  let lossy = args.iter().any(|a| a == "--lossy");

  let dynamic = image::open(input)?;
  let image = openjp2::Image::from_dynamic_image(&dynamic);

  let mut params = openjp2::EncodeParameters::new();
  if lossy {
    params = params.irreversible(true).num_layers(4);
  }

  let codestream = if output.ends_with(".jp2") {
    image.encode_jp2(params)?
  } else {
    image.encode(params)?
  };
  std::fs::write(&output, codestream)?;
  println!("wrote {output} ({} bytes, {})", output.len(), if lossy { "lossy" } else { "lossless" });
  Ok(())
}
