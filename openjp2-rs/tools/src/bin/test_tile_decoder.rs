use openjp2::{detect_format_from_file, openjpeg::*, Codec, J2KFormat, Stream, TileInfo};
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::path::Path;
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

#[derive(Debug, Clone, Copy)]
struct DecodeArea {
  x0: i32,
  y0: i32,
  x1: i32,
  y1: i32,
}

fn create_codec_and_stream<P: AsRef<Path>>(input: P) -> Result<(Codec, Stream), String> {
  let input = input.as_ref();
  let mut params = opj_dparameters_t::default();

  // Do not use layer decoding limitations
  params.cp_layer = 0;

  // Do not use resolutions reductions
  params.cp_reduce = 0;

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

fn main() -> Result<(), String> {
  let mut tile_info = TileInfo::default();
  let mut args = std::env::args();
  // Skip program name.
  args.next();

  let area = DecodeArea {
    x0: args.next().and_then(|s| s.parse().ok()).unwrap_or(0),
    y0: args.next().and_then(|s| s.parse().ok()).unwrap_or(0),
    x1: args.next().and_then(|s| s.parse().ok()).unwrap_or(1_000),
    y1: args.next().and_then(|s| s.parse().ok()).unwrap_or(1_000),
  };

  let input_file = args.next().unwrap_or_else(|| "test.j2k".to_string());

  // Create code and stream
  let (mut codec, mut stream) = create_codec_and_stream(input_file)?;

  // Decode image header and create image.
  let mut image = codec
    .read_header(&mut stream)
    .ok_or_else(|| "Failed to read header")?;

  // Set decode area
  if codec.set_decode_area(&mut image, area.x0, area.y0, area.x1, area.y1) != 1 {
    Err("Failed to set decode area")?;
  }

  let mut data = vec![0; 1000];
  let mut go_on = true;
  while go_on {
    // Decode tile
    if !codec.read_tile_header(&mut stream, &mut tile_info) {
      Err("Failed to read tile header")?;
    }
    go_on = tile_info.go_on;

    if go_on {
      let data_size = tile_info.data_size.unwrap_or_default() as usize;
      if data_size > data.len() {
        data.resize(data_size, 0);
      }
      // Decode tile
      if codec.decode_tile_data(&mut stream, tile_info.index, Some(data.as_mut_slice())) != 1 {
        Err("Failed to decode tile")?;
      }
    }
  }

  // Decode image
  if codec.decode(&mut stream, &mut image) != 1 {
    Err("Failed to decode image")?;
  }

  // End decompression
  if codec.end_decompress(&mut stream) != 1 {
    Err("Failed to end decompress")?;
  }

  Ok(())
}
