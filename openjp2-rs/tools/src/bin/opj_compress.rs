use openjp2::{detect_format_from_file, image::opj_image, openjpeg::*};
use openjp2_tools::{compress::*, convert::*, params::*};
use std::ffi::CString;
use std::path::{Path, PathBuf};

fn compress_image(
  mut image: Box<opj_image>,
  params: &CompressionParameters,
  output: &Path,
) -> Result<(), ImageError> {
  let mut params = params.clone();

  // If the image was a TIFF handle scaling the components.
  if params.decode_format == Some(ImageFileFormat::TIF) {
    let scaled = if params.is_cinema() {
      // If Cinematic mode was set and the image is RGB(A) rescale
      // to 12 bits per component to comply with cinema profiles.
      let comps = image
        .comps_mut()
        .ok_or_else(|| ImageError::EncodeError("Failed to get image components".into()))?;
      if comps[0].prec != 12 {
        for comp in comps {
          comp.scale(12);
        }
        true
      } else {
        // It was already 12 bits per component.
        false
      }
    } else {
      false
    };
    // If it wasn't scale for Cinematic mode, check if the CLI requested a target bit depth.
    if !scaled {
      if let Some(target_bit_depth) = params.target_bit_depth {
        let comps = image
          .comps_mut()
          .ok_or_else(|| ImageError::EncodeError("Failed to get image components".into()))?;
        for comp in comps {
          comp.scale(target_bit_depth);
        }
      }
    }
  }

  // If MCT mode wasn't set in CLI and image has 3+ components, enable MCT
  match (&params.mct_mode, image.numcomps) {
    (None, ncomps) if ncomps >= 3 => {
      // Default to RGB->YCC if image has at least 3 components.
      params.mct_mode = Some(MCTMode::RGB2YCC);
    }
    (Some(MCTMode::RGB2YCC), ncomps) if ncomps < 3 => {
      return Err(ImageError::EncodeError(
        "RGB->YCC conversion requires at least 3 components".into(),
      ));
    }
    (Some(MCTMode::Custom), _) if params.mct_data.is_none() => {
      return Err(ImageError::EncodeError(
        "Custom MCT selected but no MCT data provided".into(),
      ));
    }
    _ => {}
  }
  eprintln!("{params:#?}");

  // Create encoder based on codec format
  let codec = unsafe {
    match params.codec_format {
      Some(CodecFormat::J2K) => opj_create_compress(OPJ_CODEC_J2K),
      Some(CodecFormat::JP2) => opj_create_compress(OPJ_CODEC_JP2),
      None => return Err(ImageError::InvalidFormat("No codec format".into())),
      _ => return Err(ImageError::InvalidFormat("Unknown output format".into())),
    }
  };

  if codec.is_null() {
    return Err(ImageError::EncodeError("Failed to create codec".into()));
  }

  // Set compression parameters
  let status = unsafe {
    // Set parameters from CompressionParameters
    let mut c_params = params.to_c_params();

    opj_setup_encoder(codec, &mut c_params, image.as_mut())
  };

  if status == 0 {
    return Err(ImageError::EncodeError("Failed to setup encoder".into()));
  }

  // Create output stream
  let stream = unsafe {
    let path_str = CString::new(output.to_str().unwrap()).unwrap();
    opj_stream_create_default_file_stream(path_str.as_ptr(), 0)
  };

  if stream.is_null() {
    return Err(ImageError::EncodeError(
      "Failed to create output stream".into(),
    ));
  }

  // Compress image
  let result = unsafe {
    let success = opj_start_compress(codec, image.as_mut(), stream) != 0
      && opj_encode(codec, stream) != 0
      && opj_end_compress(codec, stream) != 0;

    opj_stream_destroy(stream);
    opj_destroy_codec(codec);

    success
  };

  if !result {
    return Err(ImageError::EncodeError("Compression failed".into()));
  }

  Ok(())
}

fn generate_output_path(input: &Path, img_folder: &ImageFolder) -> Result<PathBuf, ImageError> {
  let stem = input
    .file_stem()
    .ok_or_else(|| ImageError::InvalidFormat("No filename".into()))?;

  let ext = match img_folder.out_format.as_deref() {
    Some("J2K") => "j2k",
    Some("JP2") => "jp2",
    _ => return Err(ImageError::InvalidFormat("Invalid output format".into())),
  };

  let mut output = PathBuf::from(stem);
  output.set_extension(ext);

  // If output directory specified, put file there
  if let Some(dir) = img_folder.img_dir_path.as_ref() {
    output = dir.join(output);
  }

  Ok(output)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Parse command line options
  let cli_opts = match parse_cli_options(std::env::args().collect())? {
    Some(opts) => opts,
    None => {
      // Show help
      return Ok(());
    }
  };

  // Process files
  let start_time = std::time::Instant::now();
  let mut num_compressed = 0;

  if let Some(dir) = cli_opts.img_folder.img_dir_path.as_ref() {
    // Process directory
    let dir_contents = DirContents::new(dir)?;

    for file in dir_contents.files {
      if let Ok(_format) = detect_format_from_file(&file) {
        println!("\nProcessing: {}", file.display());

        // Update parameters for this file
        let mut params = cli_opts.compression_params.clone();
        params.input_file = Some(file.clone());
        params.decode_format =
          ImageFileFormat::get_file_format(file.to_str().ok_or("Invalid path")?).ok();

        // Generate output filename
        let output = generate_output_path(&file, &cli_opts.img_folder)?;
        params.output_file = Some(output.clone());

        // Process file
        let image = load_image(&file, &params)?;
        compress_image(image, &params, &output)?;

        num_compressed += 1;
      }
    }
  } else if let Some(input) = cli_opts.compression_params.input_file.as_ref() {
    // Process single file
    let image = load_image(input, &cli_opts.compression_params)?;
    let output = cli_opts
      .compression_params
      .output_file
      .as_ref()
      .ok_or("No output file specified")?;
    compress_image(image, &cli_opts.compression_params, output)?;
    num_compressed += 1;
  }

  let elapsed = start_time.elapsed();
  if num_compressed > 0 {
    println!(
      "Compressed {} files in {:.2} seconds",
      num_compressed,
      elapsed.as_secs_f64()
    );
  }

  Ok(())
}
