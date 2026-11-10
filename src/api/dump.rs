// Split of the safe API (doc/api-review.md): see the facade in api.rs
// and doc/redesign-proposal.md B.3.2.

use super::*;

/// Header/decode driver for the codestream index/info and the dump text
/// (the jpeg2k `DumpImage` equivalent).
pub struct DumpImage {
  codec: Decoder,
  stream: Stream,
}

impl DumpImage {
  #[allow(deprecated)]
  pub fn from_bytes(buf: &[u8]) -> Result<Self> {
    Self::from_bytes_with(buf, DecodeParameters::default())
  }

  pub fn from_bytes_with(buf: &[u8], params: DecodeParameters) -> Result<Self> {
    let format = detect_format(buf)
      .ok_or_else(|| Error::Unsupported("unknown JPEG 2000 format".into()))?;
    let mut stream = Stream::from_bytes(buf.to_vec());
    let mut codec = Decoder::new(format)?;
    codec.apply_settings(&DecoderSettings {
      layers: params.layers,
      reduce: params.reduce,
      ignore_pclr_cmap_cdef: params.ignore_pclr_cmap_cdef,
      ..Default::default()
    })?;
    /* Mirror decode_stream_inner: the strict flag and the resource limits
    must reach the DumpImage decoder too (they used to be silently
    dropped). */
    codec.set_strict(params.strict)?;
    codec.set_limits(params.limits)?;
    catch_codec_panic(true, || codec.read_header(&mut stream).map(|_| ()))?;
    Ok(Self { codec, stream })
  }

  pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
    let buf = std::fs::read(path)?;
    Self::from_bytes(&buf)
  }

  pub fn from_file_with<P: AsRef<std::path::Path>>(path: P, params: DecodeParameters) -> Result<Self> {
    let buf = std::fs::read(path)?;
    Self::from_bytes_with(&buf, params)
  }

  /// Decode the tile data (needed to populate the codestream index with
  /// the tile markers).
  pub fn decode(&mut self) -> Result<()> {
    let codec = &mut self.codec;
    let stream = &mut self.stream;
    /* The Decoder keeps its own header copy, so a failed decode cannot
    trap a retry (the retry simply fails again on the mid-state stream,
    matching the C). */
    catch_codec_panic(true, || codec.decode(stream).map(|_| ()))
  }

  /// The codestream index (tile-parts, markers, packets).
  pub fn get_codestream_index(&self) -> Option<crate::CodestreamIndex> {
    self.codec.cstr_index()
  }

  /// The codestream header info (v2).
  pub fn get_codestream_info(&self) -> Option<crate::CstrInfoV2> {
    self.codec.cstr_info()
  }

  /// The dump text for the given info flag.
  pub fn dump(&self, info_flag: i32) -> String {
    let mut out = String::new();
    self.codec.dump_codec(info_flag, &mut out);
    out
  }
}



