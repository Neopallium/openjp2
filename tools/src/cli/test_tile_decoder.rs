use openjp2::{detect_format_from_file, openjpeg::*, Decoder, Stream};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
struct DecodeArea {
  x0: i32,
  y0: i32,
  x1: i32,
  y1: i32,
}

fn create_codec_and_stream<P: AsRef<Path>>(input: P) -> Result<(Decoder, Stream), String> {
  let input = input.as_ref();
  let params = opj_dparameters_t {
    // Do not use layer decoding limitations
    cp_layer: 0,
    // Do not use resolutions reductions
    cp_reduce: 0,
    ..Default::default()
  };

  let codec_format = detect_format_from_file(input)?;
  let stream = Stream::new_file(input, 1_000_000, true).map_err(|e| format!("{e:?}"))?;

  let mut codec = Decoder::new(codec_format).map_err(|_| "Failed to create codec".to_string())?;

  /* catch events using our callbacks and give a local context */
  codec.set_info_handler(|msg: &str| print!("[INFO] {}", msg));
  codec.set_warning_handler(|msg: &str| print!("[WARNING] {}", msg));
  codec.set_error_handler(|msg: &str| print!("[ERROR] {}", msg));

  codec
    .apply_settings(&openjp2::DecoderSettings {
      layers: params.cp_layer,
      reduce: params.cp_reduce,
      ..Default::default()
    })
    .map_err(|_| "Failed to setup decoder".to_string())?;

  Ok((codec, stream))
}

pub fn run_test_tile_decoder(args: Vec<String>, default_file: &str) -> Result<(), String> {
  let mut args = args.into_iter();
  // Skip program name.
  args.next();

  let area = DecodeArea {
    x0: args.next().and_then(|s| s.parse().ok()).unwrap_or(0),
    y0: args.next().and_then(|s| s.parse().ok()).unwrap_or(0),
    x1: args.next().and_then(|s| s.parse().ok()).unwrap_or(1_000),
    y1: args.next().and_then(|s| s.parse().ok()).unwrap_or(1_000),
  };

  let input_file = args.next().unwrap_or_else(|| default_file.to_string());

  // Create code and stream
  let (mut codec, mut stream) = create_codec_and_stream(input_file)?;

  // Decode image header and create image.
  let mut image = codec
    .read_header(&mut stream)
    .map(|h| h.into_core_image())
    .map_err(|_| "Failed to read header")?;

  // Set decode area
  codec
    .set_decode_area(
      &mut image,
      Some(openjp2::DecodeArea::new(
        area.x0 as u32,
        area.y0 as u32,
        area.x1 as u32,
        area.y1 as u32,
      )),
    )
    .map_err(|_| "Failed to set decode area")?;

  let mut data = vec![0; 1000];
  let mut go_on = true;
  while go_on {
    // Decode tile
    let tile_info = codec
      .read_tile_header(&mut stream)
      .map_err(|_| "Failed to read tile header")?;
    go_on = tile_info.go_on;

    if go_on {
      let data_size = tile_info.data_size.unwrap_or_default() as usize;
      if data_size > data.len() {
        data.resize(data_size, 0);
      }
      // Decode tile
      codec
        .decode_tile_data(&mut stream, tile_info.index, Some(data.as_mut_slice()))
        .map_err(|_| "Failed to decode tile")?;
    }
  }

  // End decompression
  codec
    .end_decompress(&mut stream)
    .map_err(|_| "Failed to end decompress")?;

  Ok(())
}
