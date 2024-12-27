use openjp2::{detect_format_from_file, openjpeg::*, Codec, ICCProfile, J2KFormat, Stream};
use openjp2_tools::{color::*, convert::*, params::*};
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::{env, path::Path};

extern "C" fn info_callback(msg: *const c_char, _data: *mut c_void) {
  unsafe {
    print!("[INFO] {}", CStr::from_ptr(msg).to_string_lossy());
  }
}

extern "C" fn warning_callback(msg: *const c_char, _data: *mut c_void) {
  unsafe {
    print!("[WARNING] {}", CStr::from_ptr(msg).to_string_lossy());
  }
}

extern "C" fn error_callback(msg: *const c_char, _data: *mut c_void) {
  unsafe {
    print!("[ERROR] {}", CStr::from_ptr(msg).to_string_lossy());
  }
}

fn decompress_image<P: AsRef<Path>>(
  input: P,
  output: P,
  params: &DecompressParameters,
) -> Result<(), ImageError> {
  let input = input.as_ref();
  let output = output.as_ref();

  //eprintln!("params: {:?}", params);

  // Create decompression codec
  let cod_format = match params.codec_format {
    Some(J2KFormat::J2K) => OPJ_CODEC_J2K,
    Some(J2KFormat::JP2) => OPJ_CODEC_JP2,
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

  let set_decoded_resolution_factor =
    env::var("USE_OPJ_SET_DECODED_RESOLUTION_FACTOR")
      .ok()
      .map(|_| {
        let cp_reduce = d_params.cp_reduce;
        d_params.cp_reduce = 0;
        cp_reduce
      });

  /* catch events using our callbacks and give a local context */
  if !params.quiet {
    codec.set_info_handler(Some(info_callback), ptr::null_mut());
    codec.set_warning_handler(Some(warning_callback), ptr::null_mut());
    codec.set_error_handler(Some(error_callback), ptr::null_mut());
  }

  let status = codec.setup_decoder(&mut d_params);
  if status == 0 {
    return Err(ImageError::EncodeError("Failed to setup decoder".into()));
  }

  // Disable strict mode if we want to decode partial codestreams.
  if params.allow_partial {
    if codec.decoder_set_strict_mode(0) == 0 {
      return Err(ImageError::EncodeError("Failed to set strict mode".into()));
    }
  }

  // TODO: set the number of threads.

  // Create input stream
  let mut stream = Stream::new_file(input, 1_000_000, true)?;

  // Decode image header and create image.
  let mut image = codec
    .read_header(&mut stream)
    .ok_or_else(|| ImageError::DecodeError("Failed to read header".into()))?;

  // Set the components to decode.
  if params.numcomps > 0 {
    if codec.set_decoded_components(&params.comps_indices, 0) == 0 {
      return Err(ImageError::DecodeError(
        "Failed to set decoded components".into(),
      ));
    }
  }
  if let Some(cp_reduce) = set_decoded_resolution_factor {
    // For debuging/testing purposes.
    if codec.set_decoded_resolution_factor(cp_reduce) == 0 {
      return Err(ImageError::DecodeError(
        "Failed to set decoded resolution factor".into(),
      ));
    }
  }

  let no_decode_area =
    params.da_x0 == 0 && params.da_y0 == 0 && params.da_x1 == 0 && params.da_y1 == 0;

  if let Some(tile_index) = params.tile_index {
    // Decode a tile.
    if !no_decode_area {
      if !params.quiet {
        eprintln!("WARNING: -d option is ignored when decoding tiles");
      }
    }
    if codec.get_decoded_tile(&mut stream, &mut image, tile_index) == 0 {
      return Err(ImageError::DecodeError(
        "Failed to set decoded tiles".into(),
      ));
    }
  } else {
    if env::var("SKIP_OPJ_SET_DECODE_AREA").is_ok() && no_decode_area {
      // For debuging/testing purposes.
    } else if codec.set_decode_area(
      &mut image,
      params.da_x0 as i32,
      params.da_y0 as i32,
      params.da_x1 as i32,
      params.da_y1 as i32,
    ) == 0
    {
      return Err(ImageError::DecodeError("Failed to set decode area".into()));
    }

    // Decode image
    let status =
      codec.decode(&mut stream, &mut image) == 1 && codec.end_decompress(&mut stream) == 1;
    if !status {
      return Err(ImageError::DecodeError("Failed to decode image".into()));
    }
  }

  // Close input stream
  drop(stream);

  log::debug!(
    "Image: color_space: {:?}, numcomps: {}",
    image.color_space,
    image.numcomps
  );
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

  log::debug!(
    "Image: 1 color_space: {:?}, numcomps: {}",
    image.color_space,
    image.numcomps
  );

  // Handle color conversions
  if image.color_space == OPJ_CLRSPC_SYCC {
    log::debug!("Converting SYCC to RGB");
    color_sycc_to_rgb(&mut image);
  } else if image.color_space == OPJ_CLRSPC_CMYK {
    log::debug!("Converting CMYK to RGB");
    color_cmyk_to_rgb(&mut image);
  } else if image.color_space == OPJ_CLRSPC_EYCC {
    log::debug!("Converting eYCC to RGB");
    color_esycc_to_rgb(&mut image);
  }

  // Apply ICC profile if present
  if let Some(profile) = image.take_icc_profile() {
    match profile {
      ICCProfile::ICC(profile) => {
        log::debug!("Applying ICC profile");
        color_apply_icc_profile(&mut image, &profile);
      }
      ICCProfile::CIELab(profile) => {
        log::debug!("Applying cielab to RGB");
        color_cielab_to_rgb(&mut image, &profile);
      }
    }
  }

  // Handle precision parameters
  if !params.precision.is_empty() {
    if let Some(comps) = image.comps_mut() {
      for (i, comp) in comps.iter_mut().enumerate() {
        let prec_idx = std::cmp::min(i, params.precision.len() - 1);
        let param = &params.precision[prec_idx];

        let prec = if param.prec > 0 {
          param.prec
        } else {
          comp.prec
        };

        match param.mode {
          PrecisionMode::Clip => comp.clip(prec),
          PrecisionMode::Scale => comp.scale(prec),
        }
      }
    }
  }

  // Handle upsampling if requested
  if params.upsample {
    match upsample_image_components(&image)? {
      Some(new_image) => image = new_image,
      None => {
        if !params.quiet {
          println!("Image is already upsampled");
        }
      }
    }
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
  save_image(&mut image, output, params.split_pnm)?;

  Ok(())
}

fn upsample_image_components(orig: &opj_image) -> Result<Option<Box<opj_image>>, ImageError> {
  let mut upsample_needed = false;

  // Check if upsampling is needed
  for comp in orig.comps().unwrap().iter() {
    if comp.dx > 1 || comp.dy > 1 {
      upsample_needed = true;
      break;
    }
  }

  if !upsample_needed {
    return Ok(None);
  }

  // Create new image
  let mut image = opj_image::new();
  image.x0 = orig.x0;
  image.y0 = orig.y0;
  image.x1 = orig.x1;
  image.y1 = orig.y1;
  image.color_space = orig.color_space;

  // Allocate new components.
  if !image.alloc_comps(orig.numcomps) {
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

  // Copy and upsample components
  for (new_comp, org_comp) in new_comps.iter_mut().zip(orig_comps.iter()) {
    // Check if the component doesn't need upsampling.
    if org_comp.dx <= 1 && org_comp.dy <= 1 {
      new_comp.copy(org_comp);
      new_comp.x0 = orig.x0;
      new_comp.y0 = orig.y0;
      continue;
    }
    new_comp.dx = 1;
    new_comp.dy = 1;
    new_comp.w = org_comp.w;
    new_comp.h = org_comp.h;
    new_comp.x0 = orig.x0;
    new_comp.y0 = orig.y0;
    new_comp.prec = org_comp.prec;
    new_comp.bpp = 0;
    new_comp.sgnd = org_comp.sgnd;
    new_comp.factor = org_comp.factor;
    new_comp.alpha = org_comp.alpha;
    new_comp.resno_decoded = org_comp.resno_decoded;

    if org_comp.dx > 1 {
      new_comp.w = orig.x1 - orig.x0;
    }
    if org_comp.dy > 1 {
      new_comp.h = orig.y1 - orig.y0;
    }
    if !new_comp.alloc_data() {
      return Err(ImageError::DecodeError(
        "Failed to allocate component data".into(),
      ));
    }
    let new_w = new_comp.w as usize;
    let new_h = new_comp.h as usize;

    let src = org_comp
      .data()
      .ok_or_else(|| ImageError::DecodeError("No component data".into()))?;
    let dst = new_comp
      .data_mut()
      .ok_or_else(|| ImageError::DecodeError("No component data".into()))?;

    // Need to take into account dx and dy.
    let xoff = (org_comp.dx * org_comp.x0 - orig.x0) as usize;
    let yoff = (org_comp.dy * org_comp.y0 - orig.y0) as usize;
    let orig_dx = org_comp.dx as usize;
    let orig_dy = org_comp.dy as usize;
    if xoff >= orig_dx || yoff >= orig_dy {
      return Err(ImageError::DecodeError(
        "Invalid image/component parameters found when upsampling".into(),
      ));
    }

    let mut src_idx = 0;
    let mut y = yoff;
    let max_y = if new_h > (orig_dy - 1) {
      new_h - (orig_dy - 1)
    } else {
      // For small images, we need to handle the case where max_y is less than dy.
      0
    };
    let max_x = if new_w > (orig_dx - 1) {
      new_w - (orig_dx - 1)
    } else {
      // For small images, we need to handle the case where max_x is less than dx.
      0
    };

    // Zero out initial rows for yoff
    let mut dst_idx = 0;
    for _ in 0..yoff {
      let end = dst_idx + new_w as usize;
      dst[dst_idx..end].fill(0);
      dst_idx += new_w as usize;
    }

    while y < max_y {
      // Handle initial xoff pixels
      for x in 0..xoff {
        dst[dst_idx + x] = 0;
      }

      // Copy and replicate pixels
      let mut x = xoff;
      let mut src_x = 0; // `xorg` in original code
      while x < max_x {
        let val = src[src_idx + src_x];
        for dx in 0..orig_dx {
          dst[dst_idx + (x + dx)] = val;
        }
        x += orig_dx;
        src_x += 1;
      }

      // Handle remaining pixels on the row.
      while x < new_w {
        dst[dst_idx + x] = src[src_idx + src_x];
        x += 1;
      }
      dst_idx += new_w;

      // Copy and replicate rows
      for _ in 1..org_comp.dy {
        dst.copy_within(dst_idx - new_w..dst_idx, dst_idx);
        dst_idx += new_w;
      }

      y += orig_dy;
      src_idx += org_comp.w as usize;
    }

    // Handle remaining rows
    if y < new_h {
      // Handle initial xoff pixels
      for x in 0..xoff {
        dst[dst_idx + x] = 0;
      }

      // Copy and replicate pixels
      let mut x = xoff;
      let mut src_x = 0; // `xorg` in original code
      while x < max_x {
        let val = src[src_idx + src_x];
        for dx in 0..orig_dx {
          dst[dst_idx + x + dx] = val;
        }
        x += orig_dx;
        src_x += 1;
      }

      // Handle remaining pixels on the row.
      while x < new_w {
        dst[dst_idx + x] = src[src_idx + src_x];
        x += 1;
      }
      dst_idx += new_w;
      y += 1;

      // Copy and replicate rows
      for _ in y..new_h {
        dst.copy_within(dst_idx - new_w..dst_idx, dst_idx);
        dst_idx += new_w;
      }
    }
  }

  Ok(Some(image))
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
  env_logger::init();
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
