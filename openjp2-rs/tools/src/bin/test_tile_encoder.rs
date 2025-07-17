use openjp2::image::opj_image_cmptparm_t;
use openjp2::{detect_format_from_extension_os_str, openjpeg::*, Codec, J2KFormat, Stream};
use rand::prelude::*;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::path::PathBuf;
use std::ptr;

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

fn main() -> Result<(), String> {
  let mut params = opj_cparameters_t::default();
  let mut args = std::env::args();

  // Skip program name.
  args.next();

  // Parse arguments
  let num_comps: u32 = args
    .next()
    .unwrap_or_else(|| "3".to_string())
    .parse()
    .expect("Invalid number of components");
  let image_width: u32 = args
    .next()
    .unwrap_or_else(|| "2000".to_string())
    .parse()
    .expect("Invalid image width");
  let image_height: u32 = args
    .next()
    .unwrap_or_else(|| "2000".to_string())
    .parse()
    .expect("Invalid image height");
  let tile_width: u32 = args
    .next()
    .unwrap_or_else(|| "1000".to_string())
    .parse()
    .expect("Invalid tile width");
  let tile_height: u32 = args
    .next()
    .unwrap_or_else(|| "1000".to_string())
    .parse()
    .expect("Invalid tile height");
  let comp_prec: u32 = args
    .next()
    .unwrap_or_else(|| "8".to_string())
    .parse()
    .expect("Invalid component precision");

  // Use irreversible encoding?
  params.irreversible = args
    .next()
    .unwrap_or_else(|| "1".to_string())
    .parse()
    .expect("Invalid irreversible flag");
  let output_file = PathBuf::from(args.next().unwrap_or_else(|| "test.j2k".to_string()));
  let output_ext = output_file.extension();

  params.cblockw_init = 64;
  params.cblockh_init = 64;
  if let Some(cblockw_init) = args.next() {
    params.cblockw_init = cblockw_init.parse().expect("Invalid codeblock width");
    params.cblockh_init = cblockw_init.parse().expect("Invalid codeblock height");
  } else {
    params.tcp_numlayers = 1;
    params.cp_fixed_quality = 1;
    params.tcp_distoratio[0] = 20.0;
  };

  // number of resolutions
  params.numresolution = 6;
  if let Some(numresolution) = args.next() {
    params.numresolution = numresolution
      .parse()
      .expect("Invalid number of resolutions");
  }

  let (offsetx, offsety) = if let Some(offsetx) = args.next() {
    let offsety = args.next().expect("Missing offset y");
    (
      offsetx.parse().expect("Invalid offset x"),
      offsety.parse().expect("Invalid offset y"),
    )
  } else {
    (0, 0)
  };

  let is_random = if let Some(is_random) = args.next() {
    is_random.parse().expect("Invalid random flag")
  } else {
    false
  };

  let nb_tiles_width = (offsetx + image_width + tile_width - 1) / tile_width;
  let nb_tiles_height = (offsety + image_height + tile_height - 1) / tile_height;
  let nb_tiles = nb_tiles_width * nb_tiles_height;
  let data_size = (tile_width * tile_height * num_comps * comp_prec / 8) as usize;

  let mut data = Vec::with_capacity(data_size);
  if is_random {
    let mut rng = rand::thread_rng();
    for _ in 0..data_size {
      data.push(rng.gen());
    }
  } else {
    for i in 0..data_size {
      data.push(i as u8);
    }
  }

  // Set encoding parameters
  params.cp_tx0 = 0;
  params.cp_ty0 = 0;
  params.tile_size_on = 1;
  params.cp_tdx = tile_width as i32;
  params.cp_tdy = tile_height as i32;

  // Progression order.
  params.prog_order = OPJ_LRCP;

  // Image components
  let mut image_components = Vec::with_capacity(num_comps as usize);
  for _ in 0..num_comps {
    let mut comp_param = opj_image_cmptparm_t::default();
    comp_param.dx = 1;
    comp_param.dy = 1;
    comp_param.w = image_width;
    comp_param.h = image_height;
    comp_param.prec = comp_prec;
    comp_param.sgnd = 0;
    comp_param.x0 = offsetx;
    comp_param.y0 = offsety;
    image_components.push(comp_param);
  }

  // Create compression codec
  let codec_format = detect_format_from_extension_os_str(output_ext)?;
  let cod_format = match codec_format {
    J2KFormat::J2K => OPJ_CODEC_J2K,
    J2KFormat::JP2 => OPJ_CODEC_JP2,
    _ => {
      return Err(format!("Unknown codec format"));
    }
  };
  let mut codec =
    Codec::new_encoder(cod_format).ok_or_else(|| "Failed to create codec".to_string())?;

  /* catch events using our callbacks and give a local context */
  codec.set_info_handler(Some(info_callback), ptr::null_mut());
  codec.set_warning_handler(Some(warning_callback), ptr::null_mut());
  codec.set_error_handler(Some(error_callback), ptr::null_mut());

  // Create tile image
  let mut image = opj_image::tile_create(&image_components, OPJ_CLRSPC_SRGB)
    .ok_or_else(|| "Failed to create image")?;

  image.x0 = offsetx;
  image.y0 = offsety;
  image.x1 = offsetx + image_width;
  image.y1 = offsety + image_height;

  if codec.setup_encoder(&mut params, &mut image) != 1 {
    return Err("Failed to setup encoder".into());
  }

  // Create stream
  let mut stream = Stream::new_file(output_file, 1_000_000, false).map_err(|e| format!("{e:?}"))?;

  // Start compression
  if codec.start_compress(&mut image, &mut stream) != 1 {
    Err("Failed to start compress")?;
  }

  // Encode tiles
  for i in 0..nb_tiles {
    let tile_x = i % nb_tiles_width;
    let tile_y = i / nb_tiles_width;
    let tile_x0 = image.x0.max(tile_x * tile_width);
    let tile_y0 = image.y0.max(tile_y * tile_height);
    let tile_x1 = image.x1.min((tile_x + 1) * tile_width);
    let tile_y1 = image.y1.min((tile_y + 1) * tile_height);
    let tile_size = (tile_x1 - tile_x0) * (tile_y1 - tile_y0) * num_comps * comp_prec / 8;
    let data = &data[..tile_size as usize];

    if codec.write_tile(i, &data, &mut stream) != 1 {
      Err("Failed to write tile")?;
    }
  }

  // End compression
  if codec.end_compress(&mut stream) != 1 {
    Err("Failed to end compress")?;
  }

  Ok(())
}
