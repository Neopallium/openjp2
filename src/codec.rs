// The copyright in this software is being made available under the 2-clauses
// BSD License, included below. This software may be subject to other third
// party and contributor rights, including patent rights, and no such rights
// are granted under this license.
//
// Copyright (c) 2005, Herve Drolon, FreeImage Team
// Copyright (c) 2008, 2011-2012, Centre National d'Etudes Spatiales (CNES), FR
// Copyright (c) 2012, CS Systemes d'Information, France
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS `AS IS'
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED.  IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
//! The safe `Decoder`/`Encoder` handles over the J2K/JP2 codecs
//! (the `CodecFormat` enum dispatches to the internal state).

use super::image::*;
use super::jp2::*;

#[cfg(feature = "event-handlers")]
use super::consts::*;
use super::stream::Stream;
use super::types::*;

use super::api::{DecodeArea, DecodeError, DecodeLimits, EncoderParameters, Error, Result};
use super::event::*;
use super::fprintf::StringWriter;
use super::j2k::*;
use crate::J2KFormat;

/// The low-level codec split (0.7): the merged `Codec` handle moved to the
/// `openjp2-capi` crate, where it keeps the single `Codec*` C-ABI
/// contract.  The core exposes the two directions directly; each owns the
/// format-specific state and an event manager.  The public signatures are
/// C-type-free (`Result`-returning); the C-shaped parameter translation
/// lives in the capi crate.
#[allow(clippy::large_enum_variant)]
pub(crate) enum CodecFormat {
  J2K(J2k),
  JP2(Jp2),
}

/// Native (C-free) decoder settings: the subset of the C's
/// `DecodeParameters` the core consumes (`cp_layer`, `cp_reduce` and the
/// two `flags` bits).  The file names, formats and JPWL fields of the C
/// struct are CLI-level conveniences consumed nowhere in the core.
#[derive(Clone, Copy, Default, Debug)]
pub struct DecoderSettings {
  /// Quality-layer cap (`0` = all layers; the C's `cp_layer`).
  pub layers: u32,
  /// Resolution reduction (`0` = full resolution; the C's `cp_reduce`).
  pub reduce: u32,
  /// Ignore JP2 palette/channel-definition boxes during colour application
  /// (the C's `ignore_pclr_cmap_cdef`).
  pub ignore_pclr_cmap_cdef: bool,
  /// Dump the codestream info as it is read (the C's
  /// `dump_flag`).
  pub dump_state: bool,
  /// Per-decode thread limit: `0` = the ambient rayon pool (the default),
  /// `n > 0` = a local n-thread pool for this codec's decodes (the native
  /// `DecodeParameters::threads` knob; the C's `set_threads` sizes the
  /// process-wide pool instead).
  pub threads: usize,
}

/// A decoded codestream header: geometry and component metadata with empty
/// pixel data (`Decoder::read_header` output; the pixel data is produced by
/// `Decoder::decode`).  The empty data makes it impossible to mistake a
/// header for decoded pixels.
pub struct HeaderImage(CoreImage);

impl HeaderImage {
  /// Image width in pixels.
  pub fn width(&self) -> u32 {
    self.0.x1 - self.0.x0
  }

  /// Image height in pixels.
  pub fn height(&self) -> u32 {
    self.0.y1 - self.0.y0
  }

  /// Number of components.
  pub fn num_components(&self) -> u32 {
    self.0.numcomps
  }

  /// Consumes the wrapper into the raw `CoreImage` (with empty data).
  pub fn into_core_image(self) -> CoreImage {
    self.0
  }
}

/// Tile-part header summary returned by `Decoder::read_tile_header`.
#[derive(Clone, Default, Debug)]
pub struct TileInfo {
  /// Tile index.
  pub index: u32,
  /// The tile(-part) data size (always requested).
  pub data_size: Option<u32>,
  /// Tile origin x.
  pub x0: i32,
  /// Tile origin y.
  pub y0: i32,
  /// Tile end x.
  pub x1: i32,
  /// Tile end y.
  pub y1: i32,
  /// Number of components.
  pub nb_comps: u32,
  /// Whether more tile-parts follow this header.
  pub go_on: bool,
}

/// Low-level J2K/JP2 decoder (see `Image::from_bytes*` for the one-shot
/// safe API).
pub struct Decoder {
  pub(crate) m_codec: CodecFormat,
  pub(crate) m_event_mgr: EventMgr,
  /// The header read by `read_header` (target image for `decode`,
  /// `get_decoded_tile`, ...).
  header: Option<CoreImage>,
  /// Decode-area request stored by `set_decode_area`; applied by `decode`.
  pending_area: Option<DecodeArea>,
  /// Per-decode thread limit (`DecodeParameters::threads`): `0` = the
  /// ambient rayon pool, `n > 0` = a lazily-built local n-thread pool used
  /// for all parallelism of this codec's decodes.
  threads: usize,
  /// The local pool built for `threads` (shared with the per-thread
  /// `par::CURRENT_POOL` registration during a decode).
  #[cfg(feature = "rayon")]
  pool: Option<std::rc::Rc<rayon::ThreadPool>>,
}

/// Low-level J2K/JP2 encoder (see `Image::encode*` for the one-shot
/// safe API).
pub struct Encoder {
  pub(crate) m_codec: CodecFormat,
  pub(crate) m_event_mgr: EventMgr,
}

/// The methods both directions share: the three handlers, the thread-pool
/// sizing, the dump writer, the owned cstr-info/index accessors and a
/// hidden error emitter for the capi crate's merged handle.
macro_rules! codec_common {
  ($ty:ty) => {
    impl $ty {
      /// C-style event callbacks (the capi crate's `set_*_handler`
      /// surface).  Feature-gated: the stable Rust API routes diagnostics
      /// through the `log` crate instead.
      #[cfg(feature = "event-handlers")]
      pub fn set_info_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) {
        self.m_event_mgr.set_info_handler(handler);
      }

      /// C-style warning callback (see the feature note on
      /// `set_info_handler`).
      #[cfg(feature = "event-handlers")]
      pub fn set_warning_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) {
        self.m_event_mgr.set_warning_handler(handler);
      }

      /// C-style error callback (see the feature note on
      /// `set_info_handler`).
      #[cfg(feature = "event-handlers")]
      pub fn set_error_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) {
        self.m_event_mgr.set_error_handler(handler);
      }

      /// Sets the process-wide thread count (the C's `set_threads`
      /// equivalent; see `DecodeParameters::threads` for the per-decode
      /// limit).
      pub fn set_threads(&mut self, num_threads: i32) -> Result<()> {
        if num_threads < 0i32 {
          return Err(Error::Unsupported("negative thread count".into()));
        }
        let ok = match &mut self.m_codec {
          CodecFormat::J2K(j2k) => j2k_set_threads(j2k, num_threads as u32),
          CodecFormat::JP2(jp2) => jp2_set_threads(jp2, num_threads as u32),
        };
        if ok.is_ok() {
          Ok(())
        } else {
          Err(Error::Unsupported(
            "threading is unavailable or the count is out of range".into(),
          ))
        }
      }

      /// Writes the codestream info/dump text (`dump`'s output).
      pub fn dump_codec<W: StringWriter>(&self, mut info_flag: i32, mut output_stream: &mut W) {
        match &self.m_codec {
          CodecFormat::J2K(j2k) => j2k_dump(j2k, info_flag, output_stream),
          CodecFormat::JP2(jp2) => jp2_dump(jp2, info_flag, output_stream),
        }
      }

      /// Owned codestream info (header summary).  The capi crate marshals
      /// this into the C-ABI mirror for `get_cstr_info`.
      pub fn cstr_info(&self) -> Option<crate::j2k::CstrInfoV2> {
        match &self.m_codec {
          CodecFormat::J2K(j2k) => crate::j2k::j2k_get_cstr_info_owned(j2k),
          CodecFormat::JP2(jp2) => crate::j2k::j2k_get_cstr_info_owned(&jp2.j2k),
        }
      }

      /// Owned codestream index (the capi crate marshals this into the
      /// C-ABI mirror for `get_cstr_index`).
      pub fn cstr_index(&self) -> Option<crate::j2k::CodestreamIndex> {
        match &self.m_codec {
          CodecFormat::J2K(j2k) => j2k.cstr_index.as_deref().cloned(),
          CodecFormat::JP2(jp2) => jp2.j2k.cstr_index.as_deref().cloned(),
        }
      }

      /// Tile/code-block geometry of the codestream header: a safe subset of
      /// the C-ABI codestream-info mirror (tile width/height and the
      /// code-block size exponents).
      pub fn get_tile_geometry(&self) -> (u32, u32, u32, u32) {
        let j2k = match &self.m_codec {
          CodecFormat::J2K(j2k) => j2k,
          CodecFormat::JP2(jp2) => &jp2.j2k,
        };
        let tcp = j2k.m_specific_param.m_decoder.m_default_tcp.as_deref();
        /* `tccps` is empty until the SIZ marker is read (`Tcp::default()`),
        so a call before `read_header` must return zeros, not index it. */
        let (cblkw, cblkh) = tcp
          .and_then(|t| t.tccps.first())
          .map(|t| (t.cblkw, t.cblkh))
          .unwrap_or((0, 0));
        (j2k.m_cp.tdx, j2k.m_cp.tdy, cblkw, cblkh)
      }

      /// Emit a wrong-direction error through the codec's event manager.
      /// Used only by the `openjp2-capi` crate's merged `Codec` handle to
      /// reproduce the C's error events when a decompressor call arrives on
      /// a compressor handle (and vice versa).  Not part of the stable API.
      #[cfg(feature = "event-handlers")]
      #[doc(hidden)]
      pub fn emit_error_event(&mut self, msg: &str) {
        event_msg!(&mut self.m_event_mgr, EVT_ERROR, "{}", msg);
      }
    }
  };
}

codec_common!(Decoder);
codec_common!(Encoder);

impl Decoder {
  /// Creates a decoder for the given format (`J2K` codestream or `JP2`).
  pub fn new(format: J2KFormat) -> Result<Self> {
    let m_codec = match format {
      J2KFormat::J2K => CodecFormat::J2K(
        j2k_create_decompress().ok_or_else(|| Error::Decode("failed to create decoder".into()))?,
      ),
      J2KFormat::JP2 => {
        /* get a JP2 decoder handle */
        CodecFormat::JP2(
          jp2_create(true).ok_or_else(|| Error::Decode("failed to create decoder".into()))?,
        )
      }
      other => {
        return Err(Error::Unsupported(format!(
          "codec format {:?} is not supported for decoding",
          other
        )));
      }
    };
    Ok(Decoder {
      m_codec,
      m_event_mgr: Default::default(),
      header: None,
      pending_area: None,
      threads: 0,
      #[cfg(feature = "rayon")]
      pool: None,
    })
  }

  /// Applies the native decoder settings (layer cap, resolution reduction,
  /// JP2 colour-box handling and the dump flag).  The C's
  /// `Codec::setup` equivalent; the capi crate translates
  /// `DecodeParameters` into this.
  pub fn apply_settings(&mut self, settings: &DecoderSettings) -> Result<()> {
    self.threads = settings.threads;
    match &mut self.m_codec {
      CodecFormat::J2K(dec) => j2k_setup_decoder(dec, settings),
      CodecFormat::JP2(dec) => jp2_setup_decoder(dec, settings),
    }
    Ok(())
  }

  /// Runs `f` with this codec's per-decode pool registered as the current
  /// pool for the parallel helpers (`DecodeParameters::threads`).  `0`
  /// (the default) keeps the ambient pool; `n > 0` builds a local
  /// n-thread pool on first use and reuses it for every decode of this
  /// codec.  Without the `rayon` feature the limit is inert (the helpers
  /// are sequential anyway).
  fn with_pool<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
    #[cfg(feature = "rayon")]
    {
      if self.threads == 0 {
        return f(self);
      }
      if self.pool.is_none() {
        self.pool = rayon::ThreadPoolBuilder::new()
          .num_threads(self.threads)
          .build()
          .ok()
          .map(std::rc::Rc::new);
      }
      match &self.pool {
        Some(pool) => {
          let _guard = crate::par::set_current_pool(pool);
          f(self)
        }
        None => f(self),
      }
    }
    #[cfg(not(feature = "rayon"))]
    {
      let _ = self.threads;
      f(self)
    }
  }

  /// Lenient vs strict decode (the C default is strict; the safe API
  /// builder is lenient by default and calls this explicitly).
  pub fn set_strict(&mut self, strict: bool) -> Result<()> {
    match &mut self.m_codec {
      CodecFormat::J2K(dec) => j2k_decoder_set_strict_mode(dec, strict),
      CodecFormat::JP2(dec) => jp2_decoder_set_strict_mode(dec, strict),
    }
    Ok(())
  }

  /// Applies decode resource limits (`max_mem_alloc == 0` selects the
  /// built-in 1 GiB ceiling).
  pub fn set_limits(&mut self, limits: DecodeLimits) -> Result<()> {
    match &mut self.m_codec {
      CodecFormat::J2K(dec) => {
        dec.decode_limits = limits;
      }
      CodecFormat::JP2(dec) => {
        dec.j2k.decode_limits = limits;
      }
    }
    Ok(())
  }

  /// Restricts decoding to the given component indices.
  pub fn set_components(&mut self, components: &[u32]) -> Result<()> {
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(dec) => j2k_set_decoded_components(dec, components, &mut self.m_event_mgr),
      CodecFormat::JP2(dec) => jp2_set_decoded_components(dec, components, &mut self.m_event_mgr),
    };
    if ok.is_ok() {
      Ok(())
    } else {
      Err(Error::Decode("set_decoded_components failed".into()))
    }
  }

  /// Requests a decode area (a sub-window of the full image).  The header
  /// image's dimensions are updated in place (C contract) and the area is
  /// applied by the next `decode`.  `None` resets to the full image.
  pub fn set_decode_area(
    &mut self,
    header: &mut CoreImage,
    area: Option<DecodeArea>,
  ) -> Result<()> {
    let had_area = self.pending_area.is_some();
    self.pending_area = area;
    /* `None` resets to the full image: the codec-side tile window set by a
    previous `Some(area)` must be cleared too (a bare `pending_area = None`
    would leave the next decode restricted to that window).  The C resets
    with the all-zero area. */
    let Some((start_x, start_y, end_x, end_y)) = area
      .map(|a| {
        (
          a.start_x as i32,
          a.start_y as i32,
          a.end_x as i32,
          a.end_y as i32,
        )
      })
      .or(if had_area { Some((0, 0, 0, 0)) } else { None })
    else {
      return Ok(());
    };
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(dec) => j2k_set_decode_area(
        dec,
        header,
        start_x,
        start_y,
        end_x,
        end_y,
        &mut self.m_event_mgr,
      ),
      CodecFormat::JP2(dec) => jp2_set_decode_area(
        dec,
        header,
        start_x,
        start_y,
        end_x,
        end_y,
        &mut self.m_event_mgr,
      ),
    };
    if ok.is_err() {
      return Err(self.decode_error("set_decode_area failed"));
    }
    Ok(())
  }

  /// Resolution-reduction factor for the decoded image.
  pub fn set_resolution_factor(&mut self, res_factor: u32) -> Result<()> {
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(dec) => {
        j2k_set_decoded_resolution_factor(dec, res_factor, &mut self.m_event_mgr)
      }
      CodecFormat::JP2(dec) => {
        jp2_set_decoded_resolution_factor(dec, res_factor, &mut self.m_event_mgr)
      }
    };
    if ok.is_ok() {
      Ok(())
    } else {
      Err(Error::Decode("set_decoded_resolution_factor failed".into()))
    }
  }

  /// Reads the codestream header.  Must be called before `decode`,
  /// `decode_tile_data` and `get_decoded_tile`.
  pub fn read_header(&mut self, stream: &mut Stream) -> Result<HeaderImage> {
    let image = match &mut self.m_codec {
      CodecFormat::J2K(dec) => j2k_read_header(stream, dec, &mut self.m_event_mgr),
      CodecFormat::JP2(dec) => jp2_read_header(stream, dec, &mut self.m_event_mgr),
    }
    .ok_or_else(|| self.decode_error("failed to read header"))?;
    self.header = Some(image.clone());
    Ok(HeaderImage(image))
  }

  /// Decodes the image (the header must have been read).  In lenient mode a
  /// codestream truncated after the last tile's data delivers the header
  /// image with empty component data instead of an error (C parity; see
  /// `Image::from_bytes_with`'s contract note).
  pub fn decode(&mut self, stream: &mut Stream) -> Result<CoreImage> {
    self.with_pool(|slf| slf.decode_body(stream))
  }

  /// The pool-wrapped body of `decode` (see `with_pool`).
  fn decode_body(&mut self, stream: &mut Stream) -> Result<CoreImage> {
    let mut image = self
      .header
      .clone()
      .ok_or_else(|| self.decode_error("read_header must be called before decode"))?;

    /* Concurrent tile decode (rayon): whole-image, full-quality,
    full-resolution decodes of multi-tile codestreams.  A layer limit or a
    resolution reduction leaves coefficient regions unwritten, which the
    sequential path (like the C reference) reads as the PREVIOUS tile's
    leftovers - a fresh per-tile worker cannot reproduce that, so those
    cases stay on the sequential path.  Any failure also falls back to the
    sequential path below, so the result is always identical. */
    #[cfg(feature = "rayon")]
    if self.pending_area.is_none() {
      let (layer, reduce, strict) = self.state_summary();
      /* The parallel workers reproduce the serial decode only for the
      lenient tile-part walk; strict mode changes the walk semantics
      (the nb-tile-parts correction and the lenient recoveries the
      workers' fresh state cannot replay — see issue235), so strict
      decodes stay on the authoritative sequential path. */
      if !strict && layer == 0 && reduce == 0 {
        let full_header = image.clone();
        if let Some(core) = self.decode_parallel(stream, &full_header, 0, 0, strict) {
          return Ok(core);
        }
      }
    }

    if let Some(area) = self.pending_area {
      #[cfg(feature = "rayon")]
      let full_header = image.clone();
      let ok = match &mut self.m_codec {
        CodecFormat::J2K(dec) => j2k_set_decode_area(
          dec,
          &mut image,
          area.start_x as i32,
          area.start_y as i32,
          area.end_x as i32,
          area.end_y as i32,
          &mut self.m_event_mgr,
        ),
        CodecFormat::JP2(dec) => jp2_set_decode_area(
          dec,
          &mut image,
          area.start_x as i32,
          area.start_y as i32,
          area.end_x as i32,
          area.end_y as i32,
          &mut self.m_event_mgr,
        ),
      };
      if ok.is_err() {
        return Err(Error::Decode("set_decode_area failed".into()));
      }
      /* Concurrent tile decode of the intersecting tiles (rayon); the same
      unwritten-coefficient caveat as above limits it to full-quality
      full-resolution decodes. */
      #[cfg(feature = "rayon")]
      {
        let (layer, reduce, strict) = self.state_summary();
        /* Same strict-mode caveat as the whole-image path above. */
        if !strict && layer == 0 && reduce == 0 {
          if let Some(core) = self.decode_parallel_area(stream, &full_header, &image) {
            return Ok(core);
          }
        }
      }
    }

    /* Progressive-download support: in lenient mode a codestream truncated
    AFTER the last tile's data (missing EOC) delivers the fully decoded image
    instead of an error.  Strict mode keeps the hard failure.  Note the
    scope: truncation MID-TILE (or a used-components failure) may still
    return the header image with empty component data - the C behaves the
    same way, and the safe API documents this partial-decode contract
    rather than silently inventing pixels. */
    let (ok, strict) = match &mut self.m_codec {
      CodecFormat::J2K(dec) => (
        j2k_decode(dec, stream, &mut image, &mut self.m_event_mgr),
        dec.m_cp.strict,
      ),
      CodecFormat::JP2(dec) => (
        jp2_decode(dec, stream, &mut image, &mut self.m_event_mgr),
        dec.j2k.m_cp.strict,
      ),
    };
    if ok.is_err() {
      if strict {
        return Err(self.decode_error("decode failed"));
      }
      return Ok(image);
    }
    Ok(image)
  }

  /// Reads the remaining end-of-codestream markers (the C's
  /// `Codec::finish`).
  pub fn end_decompress(&mut self, stream: &mut Stream) -> Result<()> {
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(dec) => j2k_end_decompress(dec, stream, &mut self.m_event_mgr),
      CodecFormat::JP2(dec) => jp2_end_decompress(dec, stream, &mut self.m_event_mgr),
    };
    if ok.is_ok() {
      Ok(())
    } else {
      Err(self.decode_error("end_decompress failed"))
    }
  }

  /// Reads the next tile-part header (the tile API).  The returned
  /// `data_size` is the tile data size (always requested, matching the C's
  /// `read_tile_header` contract when the caller wants it).
  pub fn read_tile_header(&mut self, stream: &mut Stream) -> Result<TileInfo> {
    let mut tile_info = TileInfo {
      data_size: Some(0), // always request the tile data size
      ..Default::default()
    };
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(dec) => {
        j2k_read_tile_header(dec, stream, &mut tile_info, &mut self.m_event_mgr)
      }
      CodecFormat::JP2(dec) => {
        jp2_read_tile_header(dec, stream, &mut tile_info, &mut self.m_event_mgr)
      }
    };
    if ok {
      Ok(tile_info)
    } else {
      Err(self.decode_error("read_tile_header failed"))
    }
  }

  /// Decodes one tile's data into `out` (or into the codec's internal
  /// buffer when `out` is `None`; the tile API).
  pub fn decode_tile_data(
    &mut self,
    stream: &mut Stream,
    tile: u32,
    out: Option<&mut [u8]>,
  ) -> Result<()> {
    self.with_pool(|slf| slf.decode_tile_data_body(stream, tile, out))
  }

  /// The pool-wrapped body of `decode_tile_data` (see `with_pool`).
  fn decode_tile_data_body(
    &mut self,
    stream: &mut Stream,
    tile: u32,
    out: Option<&mut [u8]>,
  ) -> Result<()> {
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(dec) => j2k_decode_tile(dec, tile, out, stream, &mut self.m_event_mgr),
      CodecFormat::JP2(dec) => jp2_decode_tile(dec, tile, out, stream, &mut self.m_event_mgr),
    };
    if ok.is_ok() {
      Ok(())
    } else {
      Err(self.decode_error("decode_tile_data failed"))
    }
  }

  /// Decodes a whole tile and returns it as an image (the header must have
  /// been read).
  pub fn get_decoded_tile(&mut self, stream: &mut Stream, tile: u32) -> Result<CoreImage> {
    self.with_pool(|slf| slf.get_decoded_tile_body(stream, tile))
  }

  /// The pool-wrapped body of `get_decoded_tile` (see `with_pool`).
  fn get_decoded_tile_body(&mut self, stream: &mut Stream, tile: u32) -> Result<CoreImage> {
    let mut image = self
      .header
      .clone()
      .ok_or_else(|| self.decode_error("read_header must be called before get_decoded_tile"))?;
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(dec) => j2k_get_tile(dec, stream, &mut image, &mut self.m_event_mgr, tile),
      CodecFormat::JP2(dec) => jp2_get_tile(dec, stream, &mut image, &mut self.m_event_mgr, tile),
    };
    if ok.is_ok() {
      Ok(image)
    } else {
      Err(self.decode_error("get_decoded_tile failed"))
    }
  }

  /// Builds the structured decode error from the codec's recorded failure
  /// context (marker/box + stream position; B.1.7).
  fn decode_error(&self, fallback: &str) -> Error {
    match &self.m_codec {
      CodecFormat::J2K(j2k) => Error::Decode(DecodeError {
        message: fallback.into(),
        marker: j2k.last_marker.map(|m| m as u16),
        box_: None,
        pos: j2k.last_pos,
      }),
      CodecFormat::JP2(jp2) => Error::Decode(DecodeError {
        message: fallback.into(),
        marker: jp2.j2k.last_marker.map(|m| m as u16),
        box_: jp2.last_box,
        pos: jp2.last_pos.or(jp2.j2k.last_pos),
      }),
    }
  }

  /// (layer, reduce, strict) of the current decoder state — the parallel
  /// drivers' eligibility summary.
  fn state_summary(&self) -> (u32, u32, bool) {
    match &self.m_codec {
      CodecFormat::J2K(j2k) => (
        j2k.m_cp.m_specific_param.m_dec.m_layer,
        j2k.m_cp.m_specific_param.m_dec.m_reduce,
        j2k.m_cp.strict,
      ),
      CodecFormat::JP2(jp2) => (
        jp2.j2k.m_cp.m_specific_param.m_dec.m_layer,
        jp2.j2k.m_cp.m_specific_param.m_dec.m_reduce,
        jp2.j2k.m_cp.strict,
      ),
    }
  }

  /// Parallel decode of a decode-area request (feature `rayon`, safe-API
  /// internal): only the tiles intersecting the area are decoded, with
  /// the area image as the merge target.  `header` is the full-resolution
  /// header image (the workers' geometry source); `area_image` is the
  /// DA-dim image produced by `set_decode_area`.
  #[cfg(feature = "rayon")]
  pub(crate) fn decode_parallel_area(
    &mut self,
    stream: &mut Stream,
    header: &CoreImage,
    area_image: &CoreImage,
  ) -> Option<CoreImage> {
    let (cp, limits) = match &self.m_codec {
      CodecFormat::J2K(j2k) => {
        if j2k.m_header_partial {
          return None;
        }
        (&j2k.m_cp, j2k.decode_limits)
      }
      CodecFormat::JP2(jp2) => {
        if jp2.j2k.m_header_partial {
          return None;
        }
        (&jp2.j2k.m_cp, jp2.j2k.decode_limits)
      }
    };
    let info = self.cstr_info()?;
    let num_tiles = info.tw.wrapping_mul(info.th);
    let dec = match &self.m_codec {
      CodecFormat::J2K(j2k) => &j2k.m_specific_param.m_decoder,
      CodecFormat::JP2(jp2) => &jp2.j2k.m_specific_param.m_decoder,
    };
    let (start_x, start_y) = (dec.m_start_tile_x, dec.m_start_tile_y);
    let (end_x, end_y) = (dec.m_end_tile_x, dec.m_end_tile_y);
    if end_x <= start_x || end_y <= start_y {
      return None;
    }
    let mut tiles = Vec::new();
    for ty in start_y..end_y {
      for tx in start_x..end_x {
        tiles.push(ty.wrapping_mul(info.tw).wrapping_add(tx));
      }
    }
    if tiles.len() <= 1 {
      return None;
    }
    /* The parallel path decodes each intersecting tile with a fresh tcd:
    it can only reproduce the serial output when every involved tile is
    covered COMPLETELY by the decode area.  A partial window leaves the
    out-of-window filter taps unwritten: the sequential decode (and the
    C reference) reads the PREVIOUS tile's leftover buffer contents
    there, which a fresh worker buffer (zeroed) cannot reproduce. */
    for &tile in &tiles {
      let tx = tile % info.tw;
      let ty = tile / info.tw;
      let tile_x0 = tx.wrapping_mul(info.tdx).wrapping_add(cp.tx0);
      let tile_y0 = ty.wrapping_mul(info.tdy).wrapping_add(cp.ty0);
      let tile_x1 = (tile_x0.wrapping_add(info.tdx)).min(header.x1);
      let tile_y1 = (tile_y0.wrapping_add(info.tdy)).min(header.y1);
      if area_image.x0 > tile_x0
        || area_image.y0 > tile_y0
        || area_image.x1 < tile_x1
        || area_image.y1 < tile_y1
      {
        return None;
      }
    }
    let cstr_index = self.cstr_index()?;
    let layer = cp.m_specific_param.m_dec.m_layer;
    let strict = cp.strict;
    let mut core = crate::j2k::j2k_decode_tiles_parallel(
      cp,
      header,
      num_tiles,
      &tiles,
      &cstr_index,
      layer,
      0,
      strict,
      Some(area_image),
      area_image,
      stream,
      limits.effective_max_mem(),
    )?;
    /* JP2 palette/component-mapping post-processing (same as the serial
    jp2_decode). */
    if let CodecFormat::JP2(jp2) = &mut self.m_codec {
      let mut manager = crate::event::EventMgr::default();
      if crate::jp2::jp2_apply_color_postprocessing(jp2, &mut core, &mut manager).is_err() {
        return None;
      }
    }
    Some(core)
  }

  /// Parallel whole-image decode over the given stream (feature `rayon`,
  /// safe-API internal): decodes every tile with a fresh codec state and
  /// merges the results.  Returns None when the parallel path does not
  /// apply, leaving the stream positioned for the sequential fallback.
  #[cfg(feature = "rayon")]
  pub(crate) fn decode_parallel(
    &mut self,
    stream: &mut Stream,
    header: &CoreImage,
    layer: u32,
    reduce: u32,
    strict: bool,
  ) -> Option<CoreImage> {
    let (cp, limits) = match &self.m_codec {
      CodecFormat::J2K(j2k) => {
        if j2k.m_header_partial {
          return None;
        }
        (&j2k.m_cp, j2k.decode_limits)
      }
      CodecFormat::JP2(jp2) => {
        if jp2.j2k.m_header_partial {
          return None;
        }
        (&jp2.j2k.m_cp, jp2.j2k.decode_limits)
      }
    };
    let info = self.cstr_info()?;
    let num_tiles = info.tw.wrapping_mul(info.th);
    if num_tiles <= 1 {
      return None;
    }
    let cstr_index = self.cstr_index()?;
    let tiles: Vec<u32> = (0..num_tiles).collect();
    /* The merge target mirrors the serial output image: for a reduced
    decode it carries the factor and the reduced component dimensions. */
    let mut merge = header.clone();
    if reduce > 0 {
      for comp in &mut merge.comps {
        comp.factor = reduce;
      }
      let mut manager = crate::event::EventMgr::default();
      if crate::j2k::j2k_update_image_dimensions(&mut merge, &mut manager).is_err() {
        return None;
      }
    }
    let mut core = crate::j2k::j2k_decode_tiles_parallel(
      cp,
      header,
      num_tiles,
      &tiles,
      &cstr_index,
      layer,
      reduce,
      strict,
      None,
      &merge,
      stream,
      limits.effective_max_mem(),
    )?;
    /* JP2 files with a palette/component mapping need the same colour
    post-processing the serial jp2_decode applies after the codestream
    decode (the palette lookup turns indices into colour values). */
    if let CodecFormat::JP2(jp2) = &mut self.m_codec {
      let mut manager = crate::event::EventMgr::default();
      if crate::jp2::jp2_apply_color_postprocessing(jp2, &mut core, &mut manager).is_err() {
        return None;
      }
    }
    Some(core)
  }
}

impl Encoder {
  /// Creates an encoder for the given format (`J2K` codestream or `JP2`).
  pub fn new(format: J2KFormat) -> Result<Self> {
    let m_codec = match format {
      J2KFormat::J2K => CodecFormat::J2K(
        j2k_create_compress().ok_or_else(|| Error::Encode("failed to create encoder".into()))?,
      ),
      J2KFormat::JP2 => {
        /* get a JP2 encoder handle */
        CodecFormat::JP2(
          jp2_create(false).ok_or_else(|| Error::Encode("failed to create encoder".into()))?,
        )
      }
      other => {
        return Err(Error::Unsupported(format!(
          "codec format {:?} is not supported for encoding",
          other
        )));
      }
    };
    Ok(Encoder {
      m_codec,
      m_event_mgr: Default::default(),
    })
  }

  /// Validates the encoder parameters against the image (the C's
  /// `Codec::setup`; must be called before `start`).
  pub fn setup(&mut self, parameters: &mut EncoderParameters, image: &mut CoreImage) -> Result<()> {
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(enc) => j2k_setup_encoder(enc, parameters, image, &mut self.m_event_mgr),
      CodecFormat::JP2(enc) => jp2_setup_encoder(enc, parameters, image, &mut self.m_event_mgr),
    };
    if ok.is_ok() {
      Ok(())
    } else {
      Err(Error::Encode("setup_encoder failed".into()))
    }
  }

  /// Encoder options in the C's `key=value` form ("TLM=YES", "GUARD_BITS=4",
  /// ...).
  pub fn extra_options(&mut self, options: &[&str]) -> Result<()> {
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(enc) => j2k_encoder_set_extra_options(enc, options, &mut self.m_event_mgr),
      CodecFormat::JP2(enc) => jp2_encoder_set_extra_options(enc, options, &mut self.m_event_mgr),
    };
    if ok {
      Ok(())
    } else {
      Err(Error::Encode("extra_options failed".into()))
    }
  }

  /// Writes the codestream header (the C's `Codec::start`).
  pub fn start(&mut self, image: &mut CoreImage, stream: &mut Stream) -> Result<()> {
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(enc) => j2k_start_compress(enc, stream, image, &mut self.m_event_mgr),
      CodecFormat::JP2(enc) => jp2_start_compress(enc, stream, image, &mut self.m_event_mgr),
    };
    if ok.is_ok() {
      Ok(())
    } else {
      Err(Error::Encode("start_compress failed".into()))
    }
  }

  /// Encodes the image (the C's `Codec::encode`).
  pub fn encode(&mut self, stream: &mut Stream) -> Result<()> {
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(enc) => j2k_encode(enc, stream, &mut self.m_event_mgr),
      CodecFormat::JP2(enc) => jp2_encode(enc, stream, &mut self.m_event_mgr),
    };
    if ok.is_ok() {
      Ok(())
    } else {
      Err(Error::Encode("encode failed".into()))
    }
  }

  /// Writes the end-of-codestream markers (the C's `Codec::finish`).
  pub fn finish(&mut self, stream: &mut Stream) -> Result<()> {
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(enc) => j2k_end_compress(enc, stream, &mut self.m_event_mgr),
      CodecFormat::JP2(enc) => jp2_end_compress(enc, stream, &mut self.m_event_mgr),
    };
    if ok.is_ok() {
      Ok(())
    } else {
      Err(Error::Encode("end_compress failed".into()))
    }
  }

  /// Writes one pre-encoded tile (the C's `write_tile`).
  pub fn write_tile(&mut self, tile: u32, data: &[u8], stream: &mut Stream) -> Result<()> {
    let ok = match &mut self.m_codec {
      CodecFormat::J2K(enc) => j2k_write_tile(enc, tile, data, stream, &mut self.m_event_mgr),
      CodecFormat::JP2(enc) => jp2_write_tile(enc, tile, data, stream, &mut self.m_event_mgr),
    };
    if ok.is_ok() {
      Ok(())
    } else {
      Err(Error::Encode("write_tile failed".into()))
    }
  }

  /// Number of tiles of the configured encoder (safe-API internal).
  #[cfg(feature = "rayon")]
  pub(crate) fn num_tiles(&self) -> u32 {
    match &self.m_codec {
      CodecFormat::J2K(j2k) => j2k.m_cp.tw.wrapping_mul(j2k.m_cp.th),
      CodecFormat::JP2(jp2) => jp2.j2k.m_cp.tw.wrapping_mul(jp2.j2k.m_cp.th),
    }
  }

  /// Parallel tile encoding into the given stream (feature `rayon`,
  /// safe-API internal): encodes every tile with a fresh codec state and
  /// writes the tile-parts in order.  Returns None when the parallel path
  /// does not apply, leaving the codec state untouched for the sequential
  /// fallback.
  #[cfg(feature = "rayon")]
  pub(crate) fn encode_parallel(&mut self, stream: &mut Stream) -> Option<bool> {
    let j2k = match &mut self.m_codec {
      CodecFormat::J2K(j2k) => j2k,
      CodecFormat::JP2(jp2) => &mut jp2.j2k,
    };
    let num_tiles = j2k.m_cp.tw.wrapping_mul(j2k.m_cp.th);
    if num_tiles <= 1 {
      return None;
    }
    let mut manager = crate::event::EventMgr::default();
    Some(crate::j2k::j2k_encode_tiles_parallel(j2k, stream, &mut manager).is_ok())
  }
}
