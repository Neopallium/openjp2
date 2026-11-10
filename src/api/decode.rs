use super::*;
pub(crate) fn decode_stream(stream: &mut Stream, format: J2KFormat, params: &DecodeParameters) -> Result<Image> {
  catch_codec_panic(true, || decode_stream_inner(stream, format, params))
}

/// Internal driver over the (deprecated-for-users) low-level Codec methods.
pub(crate) fn decode_stream_inner(
  stream: &mut Stream,
  format: J2KFormat,
  params: &DecodeParameters,
) -> Result<Image> {
  let mut codec = Decoder::new(format)?;
  codec.apply_settings(&DecoderSettings {
    layers: params.layers,
    reduce: params.reduce,
    ignore_pclr_cmap_cdef: params.ignore_pclr_cmap_cdef,
    ..Default::default()
  })?;
  codec.set_limits(params.limits)?;
  /* The C library default is strict; the builder flag must control the
  mode explicitly so strict(false) actually relaxes it. */
  codec.set_strict(params.strict)?;

  let header = codec.read_header(stream)?;
  let mut image = header.into_core_image();

  if let Some(area) = params.decode_area {
    codec.set_decode_area(&mut image, Some(area))?;
  }

  let core = codec.decode(stream)?;
  if codec.end_decompress(stream).is_err() && params.strict {
    return Err(Error::Decode("end_decompress failed".into()));
  }

  Ok(Image { core })
}

/// Detect the format of an in-memory buffer.
pub fn detect_format(buf: &[u8]) -> Option<J2KFormat> {
  crate::detect_format(buf).ok()
}

