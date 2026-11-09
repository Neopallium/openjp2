/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2005, Herve Drolon, FreeImage Team
 * Copyright (c) 2008, 2011-2012, Centre National d'Etudes Spatiales (CNES), FR
 * Copyright (c) 2012, CS Systemes d'Information, France
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS `AS IS'
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */
use super::jp2::*;
use super::image::*;

use super::c_api_types::*;
use super::consts::*;
use super::types::*;

use super::event::*;
use super::fprintf::StringWriter;
use super::j2k::*;

#[allow(clippy::large_enum_variant)]
pub(crate) enum CodecFormat {
  J2K(J2k),
  JP2(Jp2),
}

pub(crate) enum CodecType {
  Encoder(CodecFormat),
  Decoder(CodecFormat),
}

#[repr(C)]
pub struct Codec {
  pub(crate) m_codec: CodecType,
  pub(crate) m_event_mgr: EventMgr,
}
pub type opj_codec_private_t = Codec;

impl Codec {
  pub fn new_encoder(format: CODEC_FORMAT) -> Option<Self> {
    let m_codec = match format {
      OPJ_CODEC_J2K => CodecType::Encoder(CodecFormat::J2K(opj_j2k_create_compress()?)),
      OPJ_CODEC_JP2 => {
        /* get a JP2 encoder handle */
        CodecType::Encoder(CodecFormat::JP2(opj_jp2_create(0i32)?))
      }
      _ => {
        return None;
      }
    };
    Some(Codec {
      m_codec,
      m_event_mgr: Default::default(),
    })
  }

  pub fn new_decoder(format: CODEC_FORMAT) -> Option<Self> {
    let m_codec = match format {
      OPJ_CODEC_J2K => CodecType::Decoder(CodecFormat::J2K(opj_j2k_create_decompress()?)),
      OPJ_CODEC_JP2 => {
        /* get a JP2 decoder handle */
        CodecType::Decoder(CodecFormat::JP2(opj_jp2_create(1i32)?))
      }
      _ => {
        return None;
      }
    };
    Some(Codec {
      m_codec,
      m_event_mgr: Default::default(),
    })
  }

  pub fn set_info_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) -> OPJ_BOOL {
    self.m_event_mgr.set_info_handler(handler);
    1i32
  }

  pub fn set_warning_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) -> OPJ_BOOL {
    self.m_event_mgr.set_warning_handler(handler);
    1i32
  }

  pub fn set_error_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) -> OPJ_BOOL {
    self.m_event_mgr.set_error_handler(handler);
    1i32
  }

  pub fn set_threads(&mut self, mut num_threads: core::ffi::c_int) -> OPJ_BOOL {
    if num_threads >= 0i32 {
      match &mut self.m_codec {
        CodecType::Encoder(CodecFormat::J2K(j2k)) | CodecType::Decoder(CodecFormat::J2K(j2k)) => {
          opj_j2k_set_threads(j2k, num_threads as _)
        }
        CodecType::Encoder(CodecFormat::JP2(jp2)) | CodecType::Decoder(CodecFormat::JP2(jp2)) => {
          opj_jp2_set_threads(jp2, num_threads as _)
        }
      }
    } else {
      0
    }
  }

  pub fn dump_codec<W: StringWriter>(&self, mut info_flag: i32, mut output_stream: &mut W) {
    match &self.m_codec {
      CodecType::Encoder(CodecFormat::J2K(j2k)) | CodecType::Decoder(CodecFormat::J2K(j2k)) => {
        j2k_dump(j2k, info_flag, output_stream)
      }
      CodecType::Encoder(CodecFormat::JP2(jp2)) | CodecType::Decoder(CodecFormat::JP2(jp2)) => {
        jp2_dump(jp2, info_flag, output_stream)
      }
    }
  }

  /// Safe, owned codestream info v2 (header summary). The capi crate
  /// marshals this into the C-ABI mirror for `opj_get_cstr_info`.
  pub fn get_cstr_info_v2(&self) -> Option<crate::j2k::CstrInfoV2> {
    match &self.m_codec {
      CodecType::Encoder(CodecFormat::J2K(j2k)) | CodecType::Decoder(CodecFormat::J2K(j2k)) => {
        crate::j2k::j2k_get_cstr_info_owned(j2k)
      }
      CodecType::Encoder(CodecFormat::JP2(jp2)) | CodecType::Decoder(CodecFormat::JP2(jp2)) => {
        crate::j2k::j2k_get_cstr_info_owned(&jp2.j2k)
      }
    }
  }

  /// Number of tiles of the configured encoder (safe-API internal).
  #[cfg(feature = "rayon")]
  pub(crate) fn num_tiles(&self) -> u32 {
    match &self.m_codec {
      CodecType::Encoder(CodecFormat::J2K(j2k)) | CodecType::Decoder(CodecFormat::J2K(j2k)) => {
        j2k.m_cp.tw.wrapping_mul(j2k.m_cp.th)
      }
      CodecType::Encoder(CodecFormat::JP2(jp2)) | CodecType::Decoder(CodecFormat::JP2(jp2)) => {
        jp2.j2k.m_cp.tw.wrapping_mul(jp2.j2k.m_cp.th)
      }
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
      CodecType::Encoder(CodecFormat::J2K(j2k)) => j2k,
      CodecType::Encoder(CodecFormat::JP2(jp2)) => &mut jp2.j2k,
      _ => return None,
    };
    let num_tiles = j2k.m_cp.tw.wrapping_mul(j2k.m_cp.th);
    if num_tiles <= 1 {
      return None;
    }
    let mut manager = crate::event::EventMgr::default();
    Some(crate::j2k::j2k_encode_tiles_parallel(j2k, stream, &mut manager) != 0)
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
      CodecType::Decoder(CodecFormat::J2K(j2k)) => {
        if j2k.m_header_partial {
          return None;
        }
        (&j2k.m_cp, j2k.decode_limits)
      }
      CodecType::Decoder(CodecFormat::JP2(jp2)) => {
        if jp2.j2k.m_header_partial {
          return None;
        }
        (&jp2.j2k.m_cp, jp2.j2k.decode_limits)
      }
      _ => return None,
    };
    let info = self.get_cstr_info_v2()?;
    let num_tiles = info.tw.wrapping_mul(info.th);
    let dec = match &self.m_codec {
      CodecType::Decoder(CodecFormat::J2K(j2k)) => &j2k.m_specific_param.m_decoder,
      CodecType::Decoder(CodecFormat::JP2(jp2)) => &jp2.j2k.m_specific_param.m_decoder,
      _ => return None,
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
    let cstr_index = self.get_cstr_index_owned()?;
    let layer = cp.m_specific_param.m_dec.m_layer;
    let strict = cp.strict != 0;
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
    opj_jp2_decode). */
    if let CodecType::Decoder(CodecFormat::JP2(jp2)) = &mut self.m_codec {
      let mut manager = crate::event::EventMgr::default();
      if crate::jp2::opj_jp2_apply_color_postprocessing(jp2, &mut core, &mut manager) == 0 {
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
      CodecType::Decoder(CodecFormat::J2K(j2k)) => {
        if j2k.m_header_partial {
          return None;
        }
        (&j2k.m_cp, j2k.decode_limits)
      }
      CodecType::Decoder(CodecFormat::JP2(jp2)) => {
        if jp2.j2k.m_header_partial {
          return None;
        }
        (&jp2.j2k.m_cp, jp2.j2k.decode_limits)
      }
      _ => return None,
    };
    let info = self.get_cstr_info_v2()?;
    let num_tiles = info.tw.wrapping_mul(info.th);
    if num_tiles <= 1 {
      return None;
    }
    let cstr_index = self.get_cstr_index_owned()?;
    let tiles: Vec<u32> = (0..num_tiles).collect();
    /* The merge target mirrors the serial output image: for a reduced
    decode it carries the factor and the reduced component dimensions. */
    let mut merge = header.clone();
    if reduce > 0 {
      for comp in &mut merge.comps {
        comp.factor = reduce;
      }
      let mut manager = crate::event::EventMgr::default();
      if crate::j2k::opj_j2k_update_image_dimensions(&mut merge, &mut manager) == 0 {
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
    post-processing the serial opj_jp2_decode applies after the codestream
    decode (the palette lookup turns indices into colour values). */
    if let CodecType::Decoder(CodecFormat::JP2(jp2)) = &mut self.m_codec {
      let mut manager = crate::event::EventMgr::default();
      if crate::jp2::opj_jp2_apply_color_postprocessing(jp2, &mut core, &mut manager) == 0 {
        return None;
      }
    }
    Some(core)
  }

  /// Safe, owned codestream index (the capi crate marshals this into the
  /// C-ABI mirror for `opj_get_cstr_index`).
  pub fn get_cstr_index_owned(&self) -> Option<crate::j2k::CodestreamIndex> {
    match &self.m_codec {
      CodecType::Encoder(CodecFormat::J2K(j2k)) | CodecType::Decoder(CodecFormat::J2K(j2k)) => {
        j2k.cstr_index.as_deref().cloned()
      }
      CodecType::Encoder(CodecFormat::JP2(jp2)) | CodecType::Decoder(CodecFormat::JP2(jp2)) => {
        jp2.j2k.cstr_index.as_deref().cloned()
      }
    }
  }
}

// Decoder
impl Codec {
  pub fn setup_decoder(&mut self, mut parameters: &mut opj_dparameters_t) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(_) => {
        event_msg!(
          &mut self.m_event_mgr,
          EVT_ERROR,
          "Codec provided to the opj_setup_decoder function is not a decompressor handler.\n",
        );
      }
      CodecType::Decoder(dec) => {
        match dec {
          CodecFormat::J2K(dec) => {
            opj_j2k_setup_decoder(dec, parameters);
          }
          CodecFormat::JP2(dec) => {
            opj_jp2_setup_decoder(dec, parameters);
          }
        }
        return 1;
      }
    }
    0i32
  }

  pub fn decoder_set_strict_mode(&mut self, mut strict: OPJ_BOOL) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(_) => {
        event_msg!(&mut self.m_event_mgr,
                      EVT_ERROR,
                      "Codec provided to the opj_decoder_set_strict_mode function is not a decompressor handler.\n",);
        0
      }
      CodecType::Decoder(dec) => {
        match dec {
          CodecFormat::J2K(dec) => {
            opj_j2k_decoder_set_strict_mode(dec, strict);
          }
          CodecFormat::JP2(dec) => {
            opj_jp2_decoder_set_strict_mode(dec, strict);
          }
        }
        1
      }
    }
  }

  /// Applies decode resource limits (Rust core API only; all-zero = unlimited).
  /// See `crate::api::DecodeLimits`.
  pub fn set_decode_limits(&mut self, limits: crate::api::DecodeLimits) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(_) => {
        event_msg!(
          &mut self.m_event_mgr,
          EVT_ERROR,
          "Codec provided to the opj_set_decode_limits function is not a decompressor handler.\n",
        );
        0
      }
      CodecType::Decoder(dec) => match dec {
        CodecFormat::J2K(dec) => {
          dec.decode_limits = limits;
          1
        }
        CodecFormat::JP2(dec) => {
          dec.j2k.decode_limits = limits;
          1
        }
      },
    }
  }

  #[deprecated(note = "use `Image::from_bytes`/`from_file` (src/api.rs) for one-shot decoding")]
  pub fn read_header(&mut self, p_stream: &mut Stream) -> Option<CoreImage> {
    match &mut self.m_codec {
      CodecType::Encoder(_) => {
        event_msg!(
          &mut self.m_event_mgr,
          EVT_ERROR,
          "Codec provided to the opj_read_header function is not a decompressor handler.\n",
        );
        None
      }
      CodecType::Decoder(dec) => match dec {
        CodecFormat::J2K(dec) => opj_j2k_read_header(p_stream, dec, &mut self.m_event_mgr),
        CodecFormat::JP2(dec) => opj_jp2_read_header(p_stream, dec, &mut self.m_event_mgr),
      },
    }
  }

  pub fn set_decoded_components(
    &mut self,
    mut components: &[u32],
    mut apply_color_transforms: OPJ_BOOL,
  ) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(_) => {
        event_msg!(&mut self.m_event_mgr,
                      EVT_ERROR,
                      "Codec provided to the opj_set_decoded_components function is not a decompressor handler.\n",
                      );
        0
      }
      CodecType::Decoder(dec) => {
        if apply_color_transforms != 0 {
          event_msg!(
            &mut self.m_event_mgr,
            EVT_ERROR,
            "apply_color_transforms = OPJ_TRUE is not supported.\n",
          );
          return 0i32;
        }
        match dec {
          CodecFormat::J2K(dec) => {
            opj_j2k_set_decoded_components(dec, components, &mut self.m_event_mgr)
          }
          CodecFormat::JP2(dec) => {
            opj_jp2_set_decoded_components(dec, components, &mut self.m_event_mgr)
          }
        }
      }
    }
  }

  #[deprecated(note = "use `Image::from_bytes`/`from_file` (src/api.rs) for one-shot decoding")]
  pub fn decode(&mut self, p_stream: &mut Stream, p_image: &mut CoreImage) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(_) => (),
      CodecType::Decoder(dec) => {
        return match dec {
          CodecFormat::J2K(dec) => opj_j2k_decode(dec, p_stream, p_image, &mut self.m_event_mgr),
          CodecFormat::JP2(dec) => opj_jp2_decode(dec, p_stream, p_image, &mut self.m_event_mgr),
        };
      }
    }
    0i32
  }

  #[deprecated(note = "use `Image::from_bytes`/`from_file` (src/api.rs) for one-shot decoding")]
  pub fn end_decompress(&mut self, mut p_stream: &mut Stream) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(_) => (),
      CodecType::Decoder(dec) => {
        return match dec {
          CodecFormat::J2K(dec) => opj_j2k_end_decompress(dec, p_stream, &mut self.m_event_mgr),
          CodecFormat::JP2(dec) => opj_jp2_end_decompress(dec, p_stream, &mut self.m_event_mgr),
        };
      }
    }
    0
  }

  pub fn set_decode_area(
    &mut self,
    p_image: &mut CoreImage,
    mut p_start_x: i32,
    mut p_start_y: i32,
    mut p_end_x: i32,
    mut p_end_y: i32,
  ) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(_) => (),
      CodecType::Decoder(dec) => {
        return match dec {
          CodecFormat::J2K(dec) => opj_j2k_set_decode_area(
            dec,
            p_image,
            p_start_x,
            p_start_y,
            p_end_x,
            p_end_y,
            &mut self.m_event_mgr,
          ),
          CodecFormat::JP2(dec) => opj_jp2_set_decode_area(
            dec,
            p_image,
            p_start_x,
            p_start_y,
            p_end_x,
            p_end_y,
            &mut self.m_event_mgr,
          ),
        };
      }
    }
    0i32
  }

  pub fn read_tile_header(&mut self, mut p_stream: &mut Stream, tile_info: &mut TileInfo) -> bool {
    match &mut self.m_codec {
      CodecType::Encoder(_) => (),
      CodecType::Decoder(dec) => {
        return match dec {
          CodecFormat::J2K(dec) => {
            opj_j2k_read_tile_header(dec, p_stream, tile_info, &mut self.m_event_mgr)
          }
          CodecFormat::JP2(dec) => {
            opj_jp2_read_tile_header(dec, p_stream, tile_info, &mut self.m_event_mgr)
          }
        };
      }
    }
    false
  }

  pub fn decode_tile_data(
    &mut self,
    mut p_stream: &mut Stream,
    mut p_tile_index: u32,
    p_data: Option<&mut [u8]>,
  ) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(_) => (),
      CodecType::Decoder(dec) => {
        return match dec {
          CodecFormat::J2K(dec) => {
            opj_j2k_decode_tile(dec, p_tile_index, p_data, p_stream, &mut self.m_event_mgr)
          }
          CodecFormat::JP2(dec) => {
            opj_jp2_decode_tile(dec, p_tile_index, p_data, p_stream, &mut self.m_event_mgr)
          }
        };
      }
    }
    0i32
  }

  pub fn get_decoded_tile(
    &mut self,
    p_stream: &mut Stream,
    p_image: &mut CoreImage,
    mut tile_index: u32,
  ) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(_) => (),
      CodecType::Decoder(dec) => {
        return match dec {
          CodecFormat::J2K(dec) => {
            opj_j2k_get_tile(dec, p_stream, p_image, &mut self.m_event_mgr, tile_index)
          }
          CodecFormat::JP2(dec) => {
            opj_jp2_get_tile(dec, p_stream, p_image, &mut self.m_event_mgr, tile_index)
          }
        };
      }
    }
    0
  }

  pub fn set_decoded_resolution_factor(&mut self, mut res_factor: u32) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(_) => 0,
      CodecType::Decoder(CodecFormat::J2K(dec)) => {
        opj_j2k_set_decoded_resolution_factor(dec, res_factor, &mut self.m_event_mgr)
      }
      CodecType::Decoder(CodecFormat::JP2(dec)) => {
        opj_jp2_set_decoded_resolution_factor(dec, res_factor, &mut self.m_event_mgr)
      }
    }
  }
}

// Encoder
impl Codec {
  /// Tile/code-block geometry of the codestream header: a safe subset of the
  /// C-ABI codestream-info mirror (tile width/height and the code-block size
  /// exponents).
  pub fn get_tile_geometry(&self) -> (u32, u32, u32, u32) {
    let j2k = match &self.m_codec {
      CodecType::Encoder(CodecFormat::J2K(j2k)) | CodecType::Decoder(CodecFormat::J2K(j2k)) => j2k,
      CodecType::Encoder(CodecFormat::JP2(jp2)) | CodecType::Decoder(CodecFormat::JP2(jp2)) => {
        &jp2.j2k
      }
    };
    let tcp = j2k.m_specific_param.m_decoder.m_default_tcp.as_deref();
    let (cblkw, cblkh) = match tcp {
      Some(t) => (t.tccps[0].cblkw, t.tccps[0].cblkh),
      None => (0, 0),
    };
    (j2k.m_cp.tdx, j2k.m_cp.tdy, cblkw, cblkh)
  }

  pub fn setup_encoder(
    &mut self,
    parameters: &mut EncoderParameters,
    p_image: &mut CoreImage,
  ) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(enc) => {
        return match enc {
          CodecFormat::J2K(enc) => {
            opj_j2k_setup_encoder(enc, parameters, p_image, &mut self.m_event_mgr)
          }
          CodecFormat::JP2(enc) => {
            opj_jp2_setup_encoder(enc, parameters, p_image, &mut self.m_event_mgr)
          }
        };
      }
      CodecType::Decoder(_) => (),
    }
    0i32
  }

  pub fn encoder_set_extra_options(&mut self, options: &[&str]) -> bool {
    match &mut self.m_codec {
      CodecType::Encoder(enc) => match enc {
        CodecFormat::J2K(enc) => {
          opj_j2k_encoder_set_extra_options(enc, options, &mut self.m_event_mgr)
        }
        CodecFormat::JP2(enc) => {
          opj_jp2_encoder_set_extra_options(enc, options, &mut self.m_event_mgr)
        }
      },
      CodecType::Decoder(_) => false,
    }
  }

  pub fn start_compress(&mut self, p_image: &mut CoreImage, p_stream: &mut Stream) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(enc) => {
        return match enc {
          CodecFormat::J2K(enc) => {
            opj_j2k_start_compress(enc, p_stream, p_image, &mut self.m_event_mgr)
          }
          CodecFormat::JP2(enc) => {
            opj_jp2_start_compress(enc, p_stream, p_image, &mut self.m_event_mgr)
          }
        };
      }
      CodecType::Decoder(_) => (),
    }
    0i32
  }

  pub fn encode(&mut self, mut p_stream: &mut Stream) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(enc) => {
        return match enc {
          CodecFormat::J2K(enc) => opj_j2k_encode(enc, p_stream, &mut self.m_event_mgr),
          CodecFormat::JP2(enc) => opj_jp2_encode(enc, p_stream, &mut self.m_event_mgr),
        };
      }
      CodecType::Decoder(_) => (),
    }
    0i32
  }

  pub fn end_compress(&mut self, mut p_stream: &mut Stream) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(enc) => {
        return match enc {
          CodecFormat::J2K(enc) => opj_j2k_end_compress(enc, p_stream, &mut self.m_event_mgr),
          CodecFormat::JP2(enc) => opj_jp2_end_compress(enc, p_stream, &mut self.m_event_mgr),
        };
      }
      CodecType::Decoder(_) => (),
    }
    0i32
  }

  pub fn write_tile(
    &mut self,
    p_tile_index: u32,
    p_data: &[u8],
    p_stream: &mut Stream,
  ) -> OPJ_BOOL {
    match &mut self.m_codec {
      CodecType::Encoder(enc) => match enc {
        CodecFormat::J2K(enc) => {
          opj_j2k_write_tile(enc, p_tile_index, p_data, p_stream, &mut self.m_event_mgr)
        }
        CodecFormat::JP2(enc) => {
          opj_jp2_write_tile(enc, p_tile_index, p_data, p_stream, &mut self.m_event_mgr)
        }
      },
      CodecType::Decoder(_) => 0,
    }
  }
}
