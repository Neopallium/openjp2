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
  codec.setup(&mut params, &mut image)?;
  let mut stream = Stream::new_output(J2K_STREAM_CHUNK_SIZE as usize, writer);
  codec.start(&mut image, &mut stream)?;
  let encode_ok = encode_maybe_parallel(&mut codec, &mut stream);
  if !encode_ok {
    return Err(Error::Encode("encode failed".into()));
  }
  codec.finish(&mut stream)?;
  Ok(())
}

/// Concurrent tile encoding (rayon): only multi-tile images.  A worker
/// failure happens before any tile bytes are written, so the serial retry
/// is safe.  A stream write failure is not: the stream already holds some
/// tile-parts, and retrying serially would append a second codestream.
fn encode_maybe_parallel(codec: &mut Encoder, stream: &mut Stream) -> bool {
  #[cfg(feature = "rayon")]
  {
    if codec.num_tiles() > 1 {
      let start = stream.tell();
      if codec.encode_parallel(stream) == Some(true) {
        return true;
      }
      if stream.tell() != start {
        log::error!("parallel encode failed after writing tile data; not retrying");
        return false;
      }
    }
  }
  codec.encode(stream).is_ok()
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
  let encode_ok = encode_maybe_parallel(&mut codec, &mut stream);
  if !encode_ok {
    return Err(Error::Encode("encode failed".into()));
  }
  codec.finish(&mut stream)?;
  let out = out_buf.borrow().clone();
  Ok(out)
}
