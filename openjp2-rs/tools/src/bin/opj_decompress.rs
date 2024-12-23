use openjp2::{detect_format_from_file, openjpeg::*, Codec, Stream};
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
  let cod_format = match params.codec_format {
    Some(CodecFormat::J2K) => OPJ_CODEC_J2K,
    Some(CodecFormat::JP2) => OPJ_CODEC_JP2,
    None => {
      return Err(ImageError::InvalidFormat(
        "No codec format specified".into(),
      ));
    }
    _ => {
      return Err(ImageError::InvalidFormat("Unknown codec format".into()));
    }
  };
  let mut codec = Codec::new_decoder(cod_format)
    .ok_or_else(|| ImageError::EncodeError("Failed to create codec".into()))?;

  // setup the decoder with the provided parameters.
  let mut d_params = params.to_c_params();
  let status = codec.setup_decoder(&mut d_params);
  if status == 0 {
    return Err(ImageError::EncodeError("Failed to setup decoder".into()));
  }

  // TODO: set strict mode.
  // TODO: set the number of threads.

  // Create input stream
  let mut stream = Stream::new_file(input, 1_000_000, true)?;

  // Decode image header and create image.
  let mut image = codec
    .read_header(&mut stream)
    .ok_or_else(|| ImageError::DecodeError("Failed to read header".into()))?;

  // TODO: set decoded components.
  // TODO: set decoded resolution factors.

  // TODO: Handle decode area and decode tile.

  // Decode image
  let status = codec.decode(&mut stream, &mut image) == 1 && codec.end_decompress(&mut stream) == 1;

  if !status {
    return Err(ImageError::DecodeError("Failed to decode image".into()));
  }

  // Close input stream
  drop(stream);

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
      color_cielab_to_rgb(&mut image);
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
    match convert_gray_to_rgb(&image)? {
      Some(new_image) => image = new_image,
      None => {
        if !params.quiet {
          println!("Image is already in RGB colorspace");
        }
      }
    }
  }

  // Write output file based on format
  save_image(&image, output)?;

  Ok(())
}

fn convert_gray_to_rgb(orig: &opj_image) -> Result<Option<Box<opj_image>>, ImageError> {
  // Check if image needs to be converted.
  match orig.color_space {
    OPJ_CLRSPC_SRGB => {
      return Ok(None);
    }
    OPJ_CLRSPC_GRAY => (),
    _ => {
      return Err(ImageError::DecodeError(
        "Don't know how to convert image to RGB colorspace".into(),
      ))
    }
  }

  // Create new image.
  let mut image = opj_image::new();
  image.x0 = orig.x0;
  image.y0 = orig.y0;
  image.x1 = orig.x1;
  image.y1 = orig.y1;
  image.color_space = OPJ_CLRSPC_SRGB;

  // Allocate new components.
  let num_new_comp = orig.numcomps + 2;
  if !image.alloc_comps(num_new_comp) {
    return Err(ImageError::DecodeError(
      "Failed to allocate components".into(),
    ));
  }

  // Get the original and new components.
  let orig_comps = orig
    .comps()
    .ok_or_else(|| ImageError::DecodeError("No components".into()))?;
  let new_comps = image
    .comps_mut()
    .ok_or_else(|| ImageError::DecodeError("No components".into()))?;

  // Split the components into gray, RGB, and remaining.
  let (gray, old_remain) = orig_comps
    .split_first()
    .ok_or_else(|| ImageError::DecodeError("No components".into()))?;
  let (rgb, new_remain) = new_comps.split_at_mut(3);

  // Copy the gray component to the RGB components.
  for comp in rgb.iter_mut() {
    comp.copy(gray);
  }
  // Copy the remaining components.
  for (old, new) in old_remain.iter().zip(new_remain.iter_mut()) {
    new.copy(old);
  }

  Ok(Some(image))
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
