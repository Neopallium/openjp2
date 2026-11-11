use openjp2::ffi::opj_image_cmptparm_t;
use openjp2::CoreImage;
use openjp2::{detect_format_from_extension_os_str, openjpeg::*, Encoder, Stream};
use rand::prelude::*;
use std::path::PathBuf;

pub fn run_test_tile_encoder(args: Vec<String>, default_file: &str) -> Result<(), String> {
  let mut params = EncoderParameters::default();
  let mut args = args.into_iter();

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
  let output_file = PathBuf::from(args.next().unwrap_or_else(|| default_file.to_string()));
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
    let num: u32 = is_random.parse().expect("Invalid random flag");
    num == 1
  } else {
    false
  };

  let nb_tiles_width = (offsetx + image_width).div_ceil(tile_width);
  let nb_tiles_height = (offsety + image_height).div_ceil(tile_height);
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
    let comp_param = opj_image_cmptparm_t {
      dx: 1,
      dy: 1,
      w: image_width,
      h: image_height,
      prec: comp_prec,
      sgnd: 0,
      x0: offsetx,
      y0: offsety,
      ..Default::default()
    };
    image_components.push(comp_param);
  }

  // Create compression codec
  let codec_format = detect_format_from_extension_os_str(output_ext)?;
  let mut codec = Encoder::new(codec_format).map_err(|_| "Failed to create codec".to_string())?;

  /* catch events using our callbacks and give a local context */
  codec.set_info_handler(|msg: &str| print!("[INFO] {}", msg));
  codec.set_warning_handler(|msg: &str| print!("[WARNING] {}", msg));
  codec.set_error_handler(|msg: &str| print!("[ERROR] {}", msg));

  // Create tile image
  let mut image =
    CoreImage::tile_create(&image_components, OPJ_CLRSPC_SRGB).ok_or("Failed to create image")?;

  image.x0 = offsetx;
  image.y0 = offsety;
  image.x1 = offsetx + image_width;
  image.y1 = offsety + image_height;

  if codec.setup(&mut params, &mut image).is_err() {
    return Err("Failed to setup encoder".into());
  }

  // Create stream
  let mut stream = Stream::new_file(output_file, 1_000_000, false).map_err(|e| format!("{e:?}"))?;

  // Start compression
  if codec.start(&mut image, &mut stream).is_err() {
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

    if codec.write_tile(i, data, &mut stream).is_err() {
      Err("Failed to write tile")?;
    }
  }

  // End compression
  if codec.finish(&mut stream).is_err() {
    Err("Failed to end compress")?;
  }

  Ok(())
}
