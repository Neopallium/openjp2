// Split of the safe API (doc/api-review.md): see the facade in api.rs
// and doc/redesign-proposal.md B.3.2.

use super::*;
use crate::consts::J2K_STREAM_CHUNK_SIZE;

pub(crate) fn encode_image_to_writer<W: std::io::Write + std::io::Seek + 'static>(
  image: CoreImage,
  params: crate::EncoderParameters,
  format: J2KFormat,
  writer: W,
) -> Result<()> {
  catch_codec_panic(false, || {
    encode_image_to_writer_inner(image, params, format, writer)
  })
}

pub(crate) fn encode_image_to_writer_inner<W: std::io::Write + std::io::Seek + 'static>(
  mut image: CoreImage,
  params: crate::EncoderParameters,
  format: J2KFormat,
  writer: W,
) -> Result<()> {
  let mut codec = Encoder::new(format)?;
  let mut params = params;
  let mut image = image.clone();
  codec.setup(&mut params, &mut image)?;
  let mut stream = Stream::new_output(J2K_STREAM_CHUNK_SIZE as usize, writer);
  codec.start(&mut image, &mut stream)?;
  /* Multi-tile images encode concurrently (byte-identical fallback). */
  #[cfg(feature = "rayon")]
  let parallel_ok = codec.num_tiles() > 1 && codec.encode_parallel(&mut stream) == Some(true);
  #[cfg(feature = "rayon")]
  let encode_ok = if parallel_ok {
    true
  } else {
    codec.encode(&mut stream).is_ok()
  };
  #[cfg(not(feature = "rayon"))]
  let encode_ok = codec.encode(&mut stream).is_ok();
  if !encode_ok {
    return Err(Error::Encode("encode failed".into()));
  }
  codec.finish(&mut stream)?;
  Ok(())
}

/// `Write + Seek` adapter collecting into a shared `Vec<u8>`; the j2k
/// encoder seeks back to patch marker lengths, so the position must be
/// honoured.
/// Internal driver over the low-level Encoder API.
pub(crate) fn encode_image(
  image: CoreImage,
  params: crate::EncoderParameters,
  format: J2KFormat,
) -> Result<Vec<u8>> {
  catch_codec_panic(false, || encode_image_inner(image, params, format))
}

pub(crate) fn encode_image_inner(
  mut image: CoreImage,
  params: crate::EncoderParameters,
  format: J2KFormat,
) -> Result<Vec<u8>> {
  let mut codec = Encoder::new(format)?;
  let mut params = params;
  codec.setup(&mut params, &mut image)?;
  let writer = crate::stream::SharedVecWriter {
    buf: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
    pos: 0,
  };
  let out_buf = writer.buf.clone();
  let mut stream = Stream::new_output(J2K_STREAM_CHUNK_SIZE as usize, writer);
  codec.start(&mut image, &mut stream)?;
  /* Concurrent tile encoding (rayon): only multi-tile images; any failure
  falls back to the sequential encode below, which yields identical bytes. */
  #[cfg(feature = "rayon")]
  let parallel_ok = codec.num_tiles() > 1 && codec.encode_parallel(&mut stream) == Some(true);
  #[cfg(feature = "rayon")]
  let encode_ok = if parallel_ok {
    true
  } else {
    codec.encode(&mut stream).is_ok()
  };
  #[cfg(not(feature = "rayon"))]
  let encode_ok = codec.encode(&mut stream).is_ok();
  if !encode_ok {
    return Err(Error::Encode("encode failed".into()));
  }
  codec.finish(&mut stream)?;
  let out = out_buf.borrow().clone();
  Ok(out)
}
