//! The `openjp2` command-line tool: modern front-end for the pure-Rust
//! OpenJPEG port (the `openjp2-cli` package installs the `openjp2` binary).
//! Encode any readable image to JPEG 2000, decode JP2/J2K back to a common
//! format, and inspect codestream structure.
//!
//! The C-parity tools (byte-compatible with the upstream `opj_compress` /
//! `opj_decompress` / `opj_dump`) live in the `openjp2-tools` crate; this
//! binary is the simplified, modern interface.

mod compress;
mod decompress;
mod dump;

use clap::{Parser, Subcommand};

/// JPEG 2000 encode/decode/inspect — pure-Rust OpenJPEG.
#[derive(Parser)]
#[command(
  name = "openjp2",
  version,
  about = "JPEG 2000 encode/decode/inspect - pure-Rust OpenJPEG",
  long_about = None,
  propagate_version = true
)]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  /// Encode any readable image (PNG/JPEG/TIFF/...) to JP2/J2K.
  Compress(compress::CompressArgs),
  /// Decode a JP2/J2K image to a common format (PNG by default).
  Decompress(decompress::DecompressArgs),
  /// Print the codestream structure (markers, boxes, tile parts).
  Dump(dump::DumpArgs),
}

fn main() {
  /* The codec routes diagnostics through `log`; keep the CLI quiet
  (warnings only) unless the RUST_LOG variable says otherwise. */
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
  let cli = Cli::parse();
  let result = match cli.command {
    Command::Compress(args) => compress::run(args),
    Command::Decompress(args) => decompress::run(args),
    Command::Dump(args) => dump::run(args),
  };
  if let Err(e) = result {
    eprintln!("error: {e}");
    std::process::exit(1);
  }
}
