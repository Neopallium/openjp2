//! `compress`: encode any `image`-crate-readable picture to JP2/J2K.

use std::path::PathBuf;

use clap::Args;
use openjp2::{EncodeParameters, Image, ProgressionOrder};

/// Encode any readable image (PNG/JPEG/TIFF/...) to JP2/J2K.
#[derive(Args)]
pub struct CompressArgs {
  /// Input image (any format the `image` crate reads).
  pub input: PathBuf,

  /// Output file; the container follows the extension (`.jp2` or
  /// `.j2k`).  Defaults to the input path with a `.jp2` extension.
  pub output: Option<PathBuf>,

  /// Container format when the output has no recognized extension.
  #[arg(short, long, value_parser = clap::value_parser!(Container))]
  pub format: Option<Container>,

  /// Use the irreversible 9-7 transform (default: lossless 5-3).
  #[arg(short, long)]
  pub lossy: bool,

  /// Number of resolutions (DWT decomposition levels + 1).
  #[arg(short, long, default_value_t = 6)]
  pub resolutions: u32,

  /// Number of quality layers (fixed layer count; conflicts with
  /// `--rate`).
  #[arg(long, default_value_t = 1)]
  pub layers: u32,

  /// Compression ratio for rate-distortion allocation (e.g. `20` =
  /// 20:1); implies a single ratio layer.
  #[arg(long)]
  pub rate: Option<f32>,

  /// Tile size as `WxH` (e.g. `512x512`); default is a single tile.
  #[arg(long, value_name = "WxH", value_parser = parse_tile)]
  pub tile: Option<(u32, u32)>,

  /// Packet progression order.
  #[arg(long, value_parser = clap::value_parser!(Order))]
  pub order: Option<Order>,

  /// Embed a COM comment marker in the codestream.
  #[arg(long)]
  pub comment: Option<String>,
}

/// The output container.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Container {
  /// The JP2 file container (`.jp2`).
  Jp2,
  /// The raw codestream (`.j2k`).
  J2k,
}

impl std::str::FromStr for Container {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_ascii_lowercase().as_str() {
      "jp2" => Ok(Self::Jp2),
      "j2k" | "jpc" | "j2c" => Ok(Self::J2k),
      _ => Err(format!("unknown container `{s}` (expected jp2 or j2k)")),
    }
  }
}

/// Packet progression order.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Order {
  /// Layer, resolution, component, precinct.
  Lrcp,
  /// Resolution, layer, component, precinct.
  Rlcp,
  /// Resolution, precinct, component, layer.
  Rpcl,
  /// Precinct, component, resolution, layer.
  Pcrl,
  /// Component, precinct, resolution, layer.
  Cprl,
}

impl std::str::FromStr for Order {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_ascii_lowercase().as_str() {
      "lrcp" => Ok(Self::Lrcp),
      "rlcp" => Ok(Self::Rlcp),
      "rpcl" => Ok(Self::Rpcl),
      "pcrl" => Ok(Self::Pcrl),
      "cprl" => Ok(Self::Cprl),
      _ => Err(format!(
        "unknown progression order `{s}` (expected LRCP, RLCP, RPCL, PCRL or CPRL)"
      )),
    }
  }
}

impl From<Order> for ProgressionOrder {
  fn from(order: Order) -> Self {
    match order {
      Order::Lrcp => ProgressionOrder::Lrcp,
      Order::Rlcp => ProgressionOrder::Rlcp,
      Order::Rpcl => ProgressionOrder::Rpcl,
      Order::Pcrl => ProgressionOrder::Pcrl,
      Order::Cprl => ProgressionOrder::Cprl,
    }
  }
}

fn parse_tile(s: &str) -> Result<(u32, u32), String> {
  let (w, h) = s
    .split_once(['x', 'X'])
    .ok_or_else(|| format!("tile size must be WxH (e.g. 512x512), got `{s}`"))?;
  let w = w
    .parse::<u32>()
    .map_err(|_| format!("tile width `{w}` is not a number"))?;
  let h = h
    .parse::<u32>()
    .map_err(|_| format!("tile height `{h}` is not a number"))?;
  if w == 0 || h == 0 {
    return Err("tile dimensions must be non-zero".into());
  }
  Ok((w, h))
}

pub fn run(args: CompressArgs) -> openjp2::Result<()> {
  let source = ::image::open(&args.input).map_err(|e| {
    openjp2::Error::Io(std::io::Error::new(
      std::io::ErrorKind::InvalidData,
      format!("cannot read `{}`: {e}", args.input.display()),
    ))
  })?;
  let image = Image::from_dynamic_image(&source);

  let mut params = EncodeParameters::new()
    .num_resolutions(args.resolutions)
    .num_layers(args.layers);
  if args.lossy {
    params = params.irreversible(true);
  }
  if let Some(rate) = args.rate {
    /* Ratio allocation: one layer at the requested ratio. */
    params = EncodeParameters::new()
      .num_resolutions(args.resolutions)
      .layer_rate(rate);
  }
  if let Some((w, h)) = args.tile {
    params = params.tile_size(w, h);
  }
  if let Some(order) = args.order {
    params = params.progression_order(order.into());
  }
  if let Some(comment) = &args.comment {
    params = params.comment(comment);
  }

  let (output, container) = destination(&args);
  let bytes = match container {
    Container::Jp2 => image.encode_jp2(params),
    Container::J2k => image.encode(params),
  }
  .map_err(|e| {
    openjp2::Error::Encode(format!("failed to encode `{}`: {e}", args.input.display()))
  })?;
  std::fs::write(&output, bytes).map_err(|e| {
    openjp2::Error::Io(std::io::Error::new(
      e.kind(),
      format!("cannot write `{}`: {e}", output.display()),
    ))
  })?;
  Ok(())
}

/// Picks the output path and container: the `--format` flag wins, then
/// the output extension, then JP2 by default.
fn destination(args: &CompressArgs) -> (PathBuf, Container) {
  let from_ext = |path: &std::path::Path| -> Option<Container> {
    match path
      .extension()
      .and_then(|e| e.to_str())
      .map(|e| e.to_ascii_lowercase())
    {
      Some(ext) if ext == "jp2" => Some(Container::Jp2),
      Some(ext) if ext == "j2k" || ext == "jpc" || ext == "j2c" => Some(Container::J2k),
      _ => None,
    }
  };
  match (&args.output, args.format) {
    (Some(output), Some(format)) => (output.clone(), format),
    (Some(output), None) => (output.clone(), from_ext(output).unwrap_or(Container::Jp2)),
    (None, Some(format)) => {
      let ext = match format {
        Container::Jp2 => "jp2",
        Container::J2k => "j2k",
      };
      let output = args.input.with_extension(ext);
      (output, format)
    }
    (None, None) => (args.input.with_extension("jp2"), Container::Jp2),
  }
}
