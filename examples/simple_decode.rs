//! Minimal decoder: loads a JP2/J2K file, prints its geometry, and (with
//! the `image` feature) writes a PNG.
//!
//! ```sh
//! cargo run --example simple_decode -- input.jp2 [output.png]
//! ```

fn main() -> Result<(), Box<dyn std::error::Error>> {
  use std::convert::TryInto;
  let mut args = std::env::args().skip(1);
  let input = args
    .next()
    .ok_or("usage: simple_decode <input.jp2|input.j2k> [output.png]")?;
  let output = args.next();

  let image = openjp2::Image::from_file(&input)?;
  println!(
    "{}: {}x{} {:?}, {} components",
    input,
    image.width(),
    image.height(),
    image.color_space(),
    image.num_components()
  );

  #[cfg(feature = "image")]
  if let Some(output) = output {
    let dynamic: image::DynamicImage = (&image).try_into()?;
    dynamic.save(&output)?;
    println!("wrote {output}");
  }
  #[cfg(not(feature = "image"))]
  if output.is_some() {
    eprintln!("PNG output needs the `image` feature");
  }
  Ok(())
}
