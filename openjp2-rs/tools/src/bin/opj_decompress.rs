use openjp2::{detect_format_from_file, openjpeg::*};
use openjp2_tools::{color::*, convert::*, params::*};
use std::path::Path;

fn decompress_image<P: AsRef<Path>>(
  input: P,
  output: P,
  params: &DecompressParameters,
) -> Result<(), ImageError> {
  let input = input.as_ref();
  let output = output.as_ref();

  // Create decompression codec
  let codec = match params.codec_format {
    Some(CodecFormat::J2K) => opj_create_decompress(OPJ_CODEC_J2K),
    Some(CodecFormat::JP2) => opj_create_decompress(OPJ_CODEC_JP2),
    None => return Err(ImageError::InvalidFormat("No codec format".into())),
    _ => return Err(ImageError::InvalidFormat("Unknown output format".into())),
  };

  if codec.is_null() {
    return Err(ImageError::EncodeError("Failed to create codec".into()));
  }

  // setup the decoder with the provided parameters.
  let status = unsafe {
    let mut d_params = params.to_c_params();
    opj_setup_decoder(codec, &mut d_params)
  };

  if status == 0 {
    unsafe {
      opj_destroy_codec(codec);
    }
    return Err(ImageError::DecodeError("Failed to setup decoder".into()));
  }

  // TODO: set strict mode.
  // TODO: set the number of threads.

  // Create input stream
  let stream = unsafe {
    let path_str = std::ffi::CString::new(input.to_str().unwrap()).unwrap();
    opj_stream_create_default_file_stream(path_str.as_ptr(), 1)
  };

  if stream.is_null() {
    unsafe {
      opj_destroy_codec(codec);
    }
    return Err(ImageError::DecodeError(
      "Failed to create input stream".into(),
    ));
  }

  // Decode image header and create image.
  let mut image = unsafe {
    let mut image: *mut opj_image_t = std::ptr::null_mut();
    let status = opj_read_header(stream, codec, &mut image);
    if status == 0 {
      opj_destroy_codec(codec);
      opj_stream_destroy(stream);
      return Err(ImageError::DecodeError("Failed to read header".into()));
    }
    &mut (*image)
  };

  // TODO: set decoded components.
  // TODO: set decoded resolution factors.

  // TODO: Handle decode area and decode tile.

  // Decode image
  let status = unsafe {
    let status = opj_decode(codec, stream, image) == 1 && opj_end_decompress(codec, stream) == 1;
    opj_destroy_codec(codec);
    opj_stream_destroy(stream);
    status
  };

  if !status {
    opj_image_destroy(image);
    return Err(ImageError::DecodeError("Failed to decode image".into()));
  }

  // Close input stream
  unsafe {
    opj_stream_destroy(stream);
  }

  // Get image components
  let comps = image
    .comps()
    .ok_or_else(|| ImageError::DecodeError("No components".into()))?;
  // Handle color space conversion
  if image.color_space != OPJ_CLRSPC_SYCC
    && image.numcomps == 3
    && comps[0].dx == comps[0].dy
    && comps[1].dx != 1
  {
    image.color_space = OPJ_CLRSPC_SYCC;
  } else if image.numcomps <= 2 {
    image.color_space = OPJ_CLRSPC_GRAY;
  }

  // Handle color conversions
  if image.color_space == OPJ_CLRSPC_SYCC {
    color_sycc_to_rgb(&mut image);
  } else if image.color_space == OPJ_CLRSPC_CMYK {
    color_cmyk_to_rgb(&mut image);
  } else if image.color_space == OPJ_CLRSPC_EYCC {
    color_esycc_to_rgb(&mut image);
  }

  // Apply ICC profile if present
  if let Some(profile) = image.icc_profile() {
    if profile.len() > 0 {
      color_apply_icc_profile(&mut image);
    } else {
      color_cielab_to_rgb(image);
    }
    image.clear_icc_profile();
  }

  // Handle precision parameters
  if !params.precision.is_empty() {
    if let Some(comps) = image.comps_mut() {
      for (i, comp) in comps.iter_mut().enumerate() {
        let prec_idx = std::cmp::min(i, params.precision.len() - 1);
        let param = &params.precision[prec_idx];

        if param.prec != 0 {
          match param.mode {
            PrecisionMode::Clip => comp.clip(param.prec),
            PrecisionMode::Scale => comp.scale(param.prec),
          }
        }
      }
    }
  }

  // Handle upsampling if requested
  if params.upsample {
    todo!("Handle upsampling");
    //image = upsample_components(image)?;
  }

  // Handle forcing RGB output
  if params.force_rgb {
    match image.color_space {
      OPJ_CLRSPC_SRGB => (),
      OPJ_CLRSPC_GRAY => {
        todo!("Handle gray to RGB conversion");
        //image = convert_gray_to_rgb(image)?;
      }
      _ => {
        return Err(ImageError::DecodeError(
          "Don't know how to convert image to RGB colorspace".into(),
        ))
      }
    }
  }

  // Write output file based on format
  save_image(&image, output)?;

  Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Parse command line options
  let (params, img_folder) = match parse_decompress_options(std::env::args().collect())? {
    Some(opts) => opts,
    None => return Ok(()),
  };

  let start_time = std::time::Instant::now();
  let mut num_decompressed = 0;

  if let Some(dir) = img_folder.img_dir_path {
    // Process directory
    let dir_contents = DirContents::new(&dir)?;

    for file in dir_contents.files {
      if let Ok(_format) = detect_format_from_file(&file) {
        println!("\nProcessing: {}", file.display());

        // Update parameters for this file
        let mut file_params = params.clone();
        file_params.input_file = Some(file.clone());
        file_params.decode_format = ImageFileFormat::get_file_format(&file).ok();

        // Generate output filename
        let stem = file.file_stem().ok_or("Invalid filename")?;
        let mut output = dir.join(stem);

        // Set extension based on output format
        let ext = match img_folder.out_format.as_deref() {
          Some("PGX") => "pgx",
          Some("PGM") | Some("PPM") | Some("PNM") => "pnm",
          Some("BMP") => "bmp",
          Some("TIF") | Some("TIFF") => "tif",
          Some("RAW") => "raw",
          Some("RAWL") => "rawl",
          Some("TGA") => "tga",
          Some("PNG") => "png",
          _ => return Err("Invalid output format".into()),
        };
        output.set_extension(ext);
        file_params.output_file = Some(output.clone());

        // Process file
        decompress_image(file, output, &file_params)?;

        num_decompressed += 1;
      }
    }
  } else if let Some(input) = &params.input_file {
    // Process single file
    let output = params.output_file.as_ref().ok_or("No output file")?;
    decompress_image(input, output, &params)?;
    num_decompressed += 1;
  }

  let elapsed = start_time.elapsed();
  if !params.quiet && num_decompressed > 0 {
    println!(
      "Decompressed {} files in {:.2} seconds",
      num_decompressed,
      elapsed.as_secs_f64()
    );
  }

  Ok(())
}
