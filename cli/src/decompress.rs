//! `decompress`: decode a JP2/J2K codestream to a common format.

use std::path::PathBuf;

use clap::Args;
use openjp2::{DecodeArea, DecodeParameters, Image};

/// Decode a JP2/J2K image to a common format (PNG by default).
#[derive(Args)]
pub struct DecompressArgs {
  /// Input JP2/J2K codestream.
  pub input: PathBuf,

  /// Output image; the format follows the extension (`.png` by
  /// default).  Defaults to the input path with a `.png` extension.
  pub output: Option<PathBuf>,

  /// Strict codestream validation (rejects lenient-mode tolerances).
  #[arg(short, long)]
  pub strict: bool,

  /// Decode only the first N quality layers.
  #[arg(long)]
  pub layers: Option<u32>,

  /// Decode at a reduced resolution factor (0 = full, 1 = half, ...).
  #[arg(long)]
  pub reduce: Option<u32>,

  /// Decode only the given window `X0,Y0,X1,Y1` (end exclusive).
  #[arg(long, value_name = "X0,Y0,X1,Y1", value_parser = parse_area)]
  pub area: Option<(u32, u32, u32, u32)>,

  /// Number of decode threads (0 = auto).
  #[arg(long, default_value_t = 0)]
  pub threads: usize,
}

fn parse_area(s: &str) -> Result<(u32, u32, u32, u32), String> {
  let parts: Vec<&str> = s.split(',').collect();
  if parts.len() != 4 {
    return Err(format!("area must be X0,Y0,X1,Y1, got `{s}`"));
  }
  let mut nums = [0u32; 4];
  for (i, p) in parts.iter().enumerate() {
    nums[i] = p
      .parse::<u32>()
      .map_err(|_| format!("`{p}` is not a number"))?;
  }
  Ok((nums[0], nums[1], nums[2], nums[3]))
}

pub fn run(args: DecompressArgs) -> openjp2::Result<()> {
  let mut params = DecodeParameters::new();
  if args.strict {
    params = params.strict(true);
  }
  if let Some(layers) = args.layers {
    params = params.layers(layers);
  }
  if let Some(reduce) = args.reduce {
    params = params.reduce(reduce);
  }
  if let Some((x0, y0, x1, y1)) = args.area {
    params = params.decode_area(Some(DecodeArea::new(x0, y0, x1, y1)));
  }
  if args.threads > 0 {
    params = params.threads(args.threads);
  }

  let image = Image::from_file_with(&args.input, params).map_err(|e| {
    openjp2::Error::Decode(format!("failed to decode `{}`: {e}", args.input.display()).into())
  })?;
  let dynamic = ::image::DynamicImage::try_from(&image).map_err(|e| {
    openjp2::Error::Unsupported(format!(
      "cannot convert `{}` to a common pixel format: {e}",
      args.input.display()
    ))
  })?;

  let output = args
    .output
    .clone()
    .unwrap_or_else(|| args.input.with_extension("png"));
  dynamic.save(&output).map_err(|e| {
    openjp2::Error::Io(std::io::Error::other(format!(
      "cannot write `{}`: {e}",
      output.display()
    )))
  })?;
  Ok(())
}
