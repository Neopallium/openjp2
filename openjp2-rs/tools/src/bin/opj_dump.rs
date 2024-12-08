use clap::Parser;
use openjp2::{detect_format_from_file, openjpeg::*, J2KFormat};
use std::ffi::CString;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "opj_dump")]
#[command(about = "Dump JPEG 2000 codestream info to stdout or a given file")]
struct Args {
  /// Input image directory path
  #[arg(long = "ImgDir")]
  img_dir: Option<PathBuf>,

  /// Input image file (required if ImgDir not specified)
  #[arg(short = 'i')]
  input: Option<PathBuf>,

  /// Output file (optional, defaults to stdout)
  #[arg(short = 'o')]
  output: Option<PathBuf>,

  /// Enable informative messages
  #[arg(short = 'v')]
  verbose: bool,

  /// Flag for output options
  #[arg(short = 'f')]
  flag: Option<u32>,
}

#[derive(Debug)]
struct DirContents {
  files: Vec<PathBuf>,
}

impl DirContents {
  fn new(dir_path: &Path) -> std::io::Result<Self> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir_path)? {
      let entry = entry?;
      let path = entry.path();
      if path.is_file() {
        files.push(path);
      }
    }
    Ok(Self { files })
  }
}

fn process_file(
  input: &Path,
  output: Option<&Path>,
  verbose: bool,
  flag: u32,
) -> Result<(), String> {
  unsafe {
    // Set up decoder parameters
    let mut parameters = std::mem::zeroed::<opj_dparameters>();
    opj_set_default_decoder_parameters(&mut parameters);
    parameters.m_verbose = verbose as i32;

    // Convert input path to C string
    let input_c = CString::new(input.to_str().ok_or("Invalid input path")?)
      .map_err(|_| "Invalid input path")?;

    // Determine input format
    let format = detect_format_from_file(input)?;
    eprintln!("Format: {:?}", format);

    // Copy input path
    let infile = parameters.infile.as_mut_ptr();
    std::ptr::copy_nonoverlapping(
      input_c.as_ptr(),
      infile,
      input_c.as_bytes().len().min(OPJ_PATH_LEN as usize - 1),
    );

    // Set up output
    if let Some(output_path) = output {
      let output_c = CString::new(output_path.to_str().ok_or("Invalid output path")?)
        .map_err(|_| "Invalid output path")?;
      let outfile = parameters.outfile.as_mut_ptr();
      std::ptr::copy_nonoverlapping(
        output_c.as_ptr(),
        outfile,
        output_c.as_bytes().len().min(OPJ_PATH_LEN as usize - 1),
      );
    }

    // Create stream
    let stream = opj_stream_create_default_file_stream(input_c.as_ptr(), 1);
    if stream.is_null() {
      return Err("Failed to create stream".into());
    }

    // Create codec
    let codec = match format {
      J2KFormat::J2K => opj_create_decompress(OPJ_CODEC_J2K),
      J2KFormat::JP2 => opj_create_decompress(OPJ_CODEC_JP2),
      J2KFormat::JPT => opj_create_decompress(OPJ_CODEC_JPT),
    };
    if codec.is_null() {
      opj_stream_destroy(stream);
      return Err("Failed to create codec".into());
    }

    // Open output file.
    let output_file = if output.is_some() {
      libc::fopen(parameters.outfile.as_ptr(), b"w\0".as_ptr() as *const i8)
    } else {
      libc::fdopen(libc::STDOUT_FILENO, b"w\0".as_ptr() as *const i8)
    };

    parameters.flags |= OPJ_DPARAMETERS_DUMP_FLAG;

    if opj_setup_decoder(codec, &mut parameters) == 0 {
      opj_stream_destroy(stream);
      opj_destroy_codec(codec);
      return Err("Failed to setup decoder".into());
    }

    let mut image = std::ptr::null_mut();
    if opj_read_header(stream, codec, &mut image) == 0 {
      opj_stream_destroy(stream);
      opj_destroy_codec(codec);
      return Err("Failed to read header".into());
    }

    opj_dump_codec(codec, flag as i32, output_file);
    libc::fflush(output_file);

    // Clean up
    opj_stream_destroy(stream);
    opj_destroy_codec(codec);
    if !image.is_null() {
      opj_image_destroy(image);
    }

    Ok(())
  }
}

fn main() -> Result<(), String> {
  // Setup rust logging.
  env_logger::init();

  let args = Args::parse();

  // Set default flag if not specified
  let flag = args
    .flag
    .unwrap_or(OPJ_IMG_INFO | OPJ_J2K_MH_INFO | OPJ_J2K_MH_IND);

  if let Some(img_dir) = args.img_dir {
    // Process directory
    let dir_contents =
      DirContents::new(&img_dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for file in dir_contents.files {
      if let Ok(_format) = detect_format_from_file(&file) {
        println!("\nProcessing: {}", file.display());
        process_file(&file, args.output.as_deref(), args.verbose, flag)?;
      }
    }
  } else if let Some(input) = args.input {
    // Process single file
    process_file(&input, args.output.as_deref(), args.verbose, flag)?;
  } else {
    return Err("Either -i <file> or --ImgDir <directory> must be specified".into());
  }

  Ok(())
}
