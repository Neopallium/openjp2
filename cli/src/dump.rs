//! `dump`: print the codestream structure (markers, boxes, tile parts).

use std::path::PathBuf;

use clap::Args;
use openjp2::{DumpImage, ALL};

/// Print the codestream structure (markers, boxes, tile parts).
#[derive(Args)]
pub struct DumpArgs {
  /// Input JP2/J2K codestream.
  pub input: PathBuf,

  /// Also decode the tile data so the tile-part index is populated.
  #[arg(long)]
  pub tiles: bool,
}

pub fn run(args: DumpArgs) -> openjp2::Result<()> {
  let mut dump = DumpImage::from_file(&args.input).map_err(|e| {
    openjp2::Error::Decode(format!("failed to read `{}`: {e}", args.input.display()).into())
  })?;
  if args.tiles {
    dump.decode().map_err(|e| {
      openjp2::Error::Decode(format!("failed to decode `{}`: {e}", args.input.display()).into())
    })?;
  }
  print!("{}", dump.dump(ALL));
  Ok(())
}
