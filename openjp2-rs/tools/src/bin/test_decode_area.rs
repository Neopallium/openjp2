use openjp2::{detect_format_from_file, openjpeg::*, Codec, J2KFormat, Stream};
use openjp2_tools::getopt::{GetOpts, OptDef, ParsedOpt, PositionalArg};
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

#[derive(Debug, Clone, PartialEq)]
enum Opt {
  Quiet,
  Steps,
  StripHeight,
  StripCheck,
  Help,
}

#[derive(Debug, Clone, PartialEq)]
enum PosArg {
  InputFile,
  DecodeArea,
}

#[derive(Debug, Clone, Copy)]
struct DecodeArea {
  x0: i32,
  y0: i32,
  x1: i32,
  y1: i32,
}

struct Args {
  input_file: Option<PathBuf>,
  quiet: bool,
  nsteps: u32,
  strip_height: u32,
  strip_check: bool,
  decode_area: Option<DecodeArea>,
}

impl Default for Args {
  fn default() -> Self {
    Self {
      input_file: None,
      quiet: false,
      nsteps: 100,
      strip_height: 0,
      strip_check: false,
      decode_area: None,
    }
  }
}

fn parse_args() -> Result<Args, String> {
  let opts = vec![
    OptDef::short('q', Opt::Quiet, false),
    OptDef::long("steps", Opt::Steps, true),
    OptDef::long("strip_height", Opt::StripHeight, true),
    OptDef::long("strip_check", Opt::StripCheck, false),
    OptDef::short('h', Opt::Help, false),
  ];

  let parser = GetOpts::new_with_positionals(
    &opts,
    &[
      PositionalArg::new("input_file", PosArg::InputFile),
      PositionalArg::new_multi("x0 y0 x1 y1", 4, PosArg::DecodeArea),
    ],
  );
  let mut args = Args::default();

  for opt in parser.parse_args(std::env::args()) {
    match opt {
      ParsedOpt::Program(_) => {}
      ParsedOpt::Opt(Opt::Quiet, _) => args.quiet = true,
      ParsedOpt::Opt(Opt::Steps, Some(arg)) => {
        args.nsteps = arg.parse().map_err(|_| "Invalid steps value")?;
      }
      ParsedOpt::Opt(Opt::StripHeight, Some(arg)) => {
        args.strip_height = arg.parse().map_err(|_| "Invalid strip height value")?;
      }
      ParsedOpt::Opt(Opt::StripCheck, _) => args.strip_check = true,
      ParsedOpt::Opt(Opt::Help, _) => {
        print_help();
        std::process::exit(0);
      }
      ParsedOpt::Positional(PosArg::InputFile, arg) => {
        args.input_file = Some(PathBuf::from(&arg[0]));
      }
      ParsedOpt::Positional(PosArg::DecodeArea, arg) => {
        let coords: Result<Vec<i32>, _> = arg[0..4].iter().map(|s| s.parse::<i32>()).collect();
        match coords {
          Ok(v) => {
            args.decode_area = Some(DecodeArea {
              x0: v[0],
              y0: v[1],
              x1: v[2],
              y1: v[3],
            })
          }
          Err(_) => return Err("Invalid decode area coordinates".into()),
        }
      }
      ParsedOpt::Opt(opt, None) => {
        return Err(format!("Missing argument for option: {opt:?}").into())
      }
      ParsedOpt::ParseError(err) => {
        return Err(err);
      }
    }
  }

  Ok(args)
}

fn print_help() {
  println!("\nUsage: test_decode_area [-q] [--steps n] [--strip_height h] [--strip_check] input_file [x0 y0 x1 y1]");
  println!("\nOptions:");
  println!("  -q                  Quiet mode");
  println!("  -steps n            Number of test steps (default: 100)");
  println!("  -strip_height h     Strip height for strip-based decoding");
  println!("  -strip_check        Enable strip checking");
  println!("  -h                  Display this help message");
  println!("\nArguments:");
  println!("  input_file          Input JPEG2000 file");
  println!("  x0 y0 x1 y1         Optional decode area coordinates");
}

fn create_codec_and_stream(input: &PathBuf) -> Result<(Codec, Stream), String> {
  let mut params = opj_dparameters_t::default();

  let codec_format = detect_format_from_file(input)?;
  let stream = Stream::new_file(input, 1_000_000, true).map_err(|e| format!("{e:?}"))?;

  // Create decompression codec
  let cod_format = match codec_format {
    J2KFormat::J2K => OPJ_CODEC_J2K,
    J2KFormat::JP2 => OPJ_CODEC_JP2,
    _ => {
      return Err(format!("Unknown codec format"));
    }
  };

  let mut codec =
    Codec::new_decoder(cod_format).ok_or_else(|| "Failed to create codec".to_string())?;

  /* catch events using our callbacks and give a local context */
  codec.set_info_handler(Some(info_callback), ptr::null_mut());
  codec.set_warning_handler(Some(warning_callback), ptr::null_mut());
  codec.set_error_handler(Some(error_callback), ptr::null_mut());

  let status = codec.setup_decoder(&mut params);
  if status == 0 {
    return Err("Failed to setup decoder".into());
  }

  Ok((codec, stream))
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TileInfo {
  pub tile_w: u32,
  pub tile_h: u32,
  pub cblk_w: u32,
  pub cblk_h: u32,
}

fn decode(
  quiet: bool,
  input_file: &PathBuf,
  area: Option<DecodeArea>,
  tile_info: Option<&mut TileInfo>,
) -> Result<Box<opj_image>, String> {
  if !quiet {
    match area {
      Some(area) => println!("Decoding {},{},{},{}", area.x0, area.y0, area.x1, area.y1),
      None => println!("Decoding full image"),
    }
  }

  // Create code and stream
  let (mut codec, mut stream) = create_codec_and_stream(input_file)?;

  // Decode image header and create image.
  let mut image = codec
    .read_header(&mut stream)
    .ok_or_else(|| "Failed to read header")?;

  // Get the codestream info.
  if let Some(tile_info) = tile_info {
    let mut cstr_info = codec.get_cstr_info();
    if cstr_info.is_null() {
      Err("Failed to get codestream info")?;
    }
    unsafe {
      tile_info.tile_w = (*cstr_info).tdx;
      tile_info.tile_h = (*cstr_info).tdy;
      tile_info.cblk_w = (*(*cstr_info).m_default_tile_info.tccp_info).cblkw;
      tile_info.cblk_h = (*(*cstr_info).m_default_tile_info.tccp_info).cblkh;
      opj_destroy_cstr_info(&mut cstr_info);
    }
  };

  // Set decode area if provided
  if let Some(area) = area {
    codec.set_decode_area(&mut image, area.x0, area.y0, area.x1, area.y1);
  }

  // Decode image
  let status = codec.decode(&mut stream, &mut image) == 1 && codec.end_decompress(&mut stream) == 1;
  if !status {
    Err("Failed to decode image")?;
  }

  Ok(image)
}

fn decode_by_strip(
  quiet: bool,
  input_file: &PathBuf,
  strip_height: u32,
  decode_area: Option<DecodeArea>,
  full_image: Option<Box<opj_image>>,
) -> Result<(), String> {
  if !quiet {
    println!("Decoding by strip with height {}", strip_height);
  }

  // Create code and stream
  let (mut codec, mut stream) = create_codec_and_stream(input_file)?;

  // Decode image header and create image.
  let mut image = codec
    .read_header(&mut stream)
    .ok_or_else(|| "Failed to read header")?;

  let full_x0 = image.x0 as i32;
  let full_y0 = image.y0 as i32;
  let full_x1 = image.x1 as i32;
  let full_y1 = image.y1 as i32;

  let area = decode_area.unwrap_or_else(|| DecodeArea {
    x0: full_x0,
    y0: full_y0,
    x1: full_x1,
    y1: full_y1,
  });

  // Decode each strip
  for y in (area.y0..area.y1).step_by(strip_height as usize) {
    let y1 = std::cmp::min(y + strip_height as i32, area.y1);

    if !quiet {
      println!("Decoding {}...{}", y, y1);
    }

    // Set decode area if provided
    if codec.set_decode_area(&mut image, area.x0, y, area.x1, y1) != 1 {
      Err("Failed to set decode area")?;
    }

    // Decode image by strip
    if codec.decode(&mut stream, &mut image) != 1 {
      Err("Failed to decode image")?;
    }

    // Check consistency if full image is provided
    if let Some(full_image) = &full_image {
      if !check_consistency(full_image, &image) {
        Err("Consistency check failed")?;
      }
    }
  }

  // If the image is small enough, try a final decode with the full image
  if (full_x1 - full_x0) < 10_000 && (full_y1 - full_y0) < 10_000 {
    if !quiet {
      println!("Decoding full image");
    }

    // Set decode area if provided
    if codec.set_decode_area(&mut image, full_x0, full_y0, full_x1, full_y1) != 1 {
      Err("Failed to set decode area")?;
    }

    // Decode full image
    if codec.decode(&mut stream, &mut image) != 1 {
      Err("Failed to decode image")?;
    }
  }

  // End decompression
  if codec.end_decompress(&mut stream) != 1 {
    Err("Failed to end decompression")?;
  }

  Ok(())
}

fn check_consistency(image: &opj_image, sub_image: &opj_image) -> bool {
  let image_comps = image.comps().expect("Failed to get image components");
  let sub_image_comps = sub_image
    .comps()
    .expect("Failed to get subimage components");

  for (compno, (img_comp, sub_comp)) in image_comps.iter().zip(sub_image_comps.iter()).enumerate() {
    let shift_y = sub_comp.y0 - img_comp.y0;
    let shift_x = sub_comp.x0 - img_comp.x0;
    let image_w = img_comp.w;
    let sub_image_w = sub_comp.w;

    let img_data = img_comp.data().expect("Failed to get image component data");
    let sub_data = sub_comp
      .data()
      .expect("Failed to get subimage component data");

    for y in 0..sub_comp.h {
      for x in 0..sub_image_w {
        let sub_image_val = sub_data[y as usize * sub_image_w as usize + x as usize];
        let image_val =
          img_data[(y + shift_y) as usize * image_w as usize + (x + shift_x) as usize];

        if sub_image_val != image_val {
          eprintln!(
            "Difference found at subimage pixel ({},{}) of compno={}: got {}, expected {}",
            x, y, compno, sub_image_val, image_val
          );
          return false;
        }
      }
    }
  }
  true
}

fn main() -> Result<(), String> {
  let mut tile_info = TileInfo::default();
  let args = parse_args()?;

  let input_file = args.input_file.as_ref().unwrap();

  // Strip-based decoding
  if args.strip_height > 0 {
    let full_image = if args.strip_check {
      Some(decode(args.quiet, input_file, None, Some(&mut tile_info))?)
    } else {
      None
    };

    // Decode by strip
    return decode_by_strip(
      args.quiet,
      input_file,
      args.strip_height,
      args.decode_area,
      full_image,
    );
  }

  // First decode entire image if needed
  let full_image = decode(args.quiet, input_file, None, Some(&mut tile_info))?;

  // Handle specific decode area if provided
  if let Some(area) = args.decode_area {
    let sub_image = decode(args.quiet, input_file, Some(area), None)?;

    if !check_consistency(&full_image, &sub_image) {
      return Err(format!("Consistency check failed for area {area:?}"));
    }

    return Ok(());
  }

  let w = full_image.x1 - full_image.x0;
  let h = full_image.y1 - full_image.y0;
  let step_x = if w > args.nsteps { w / args.nsteps } else { 1 };
  let step_y = if h > args.nsteps { h / args.nsteps } else { 1 };

  for y in (0..h).step_by(step_y as usize) {
    for x in (0..w).step_by(step_x as usize) {
      let area = DecodeArea {
        x0: (full_image.x0 + x) as i32,
        y0: (full_image.y0 + y) as i32,
        x1: std::cmp::min(full_image.x1, full_image.x0 + x + 1) as i32,
        y1: std::cmp::min(full_image.y1, full_image.y0 + y + 1) as i32,
      };

      // Decode sub image and check consistency
      {
        let sub_image = decode(args.quiet, input_file, Some(area), None)?;

        if !check_consistency(&full_image, &sub_image) {
          return Err(format!("Consistency check failed for area {area:?}"));
        }
      }

      if step_x > 1 || step_y > 1 {
        let mut area = area.clone();
        if step_x > 1 {
          area.x0 = std::cmp::min(full_image.x1, area.x0 as u32 + 1) as i32;
          area.x1 = std::cmp::min(full_image.x1, area.x1 as u32 + 1) as i32;
        }
        if step_y > 1 {
          area.y0 = std::cmp::min(full_image.y1, area.y0 as u32 + 1) as i32;
          area.y1 = std::cmp::min(full_image.y1, area.y1 as u32 + 1) as i32;
        }

        if area.x0 < full_image.x1 as i32 && area.y0 < full_image.y1 as i32 {
          let sub_image = decode(args.quiet, input_file, Some(area), None)?;

          if !check_consistency(&full_image, &sub_image) {
            return Err(format!("Consistency check failed for area {area:?}"));
          }
        }
      }
    }
  }

  Ok(())
}
