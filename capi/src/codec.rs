//! The merged codec handle behind the single `opj_codec_t*` C-ABI pointer
//! contract.
//!
//! The core split its merged `Codec` into `openjp2::Decoder`/`Encoder` (0.7).
//! This crate keeps one handle type by wrapping either direction, and is the
//! only place that translates between the C-shaped parameters
//! (`opj_dparameters_t`, `opj_cparameters_t`) and the core API.

use openjp2_core::{Decoder, Encoder};
use openjp2_ctypes::OPJ_BOOL;

/// The `opj_codec_t*` handle: either a decompressor or a compressor.
pub enum Codec {
  Dec(Decoder),
  Enc(Encoder),
}

impl Codec {
  pub fn new_decoder(format: openjp2_ctypes::OPJ_CODEC_FORMAT) -> Option<Self> {
    Some(Codec::Dec(Decoder::new(format)?))
  }

  pub fn new_encoder(format: openjp2_ctypes::OPJ_CODEC_FORMAT) -> Option<Self> {
    Some(Codec::Enc(Encoder::new(format)?))
  }

  pub fn set_info_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_info_handler(handler),
      Codec::Enc(enc) => enc.set_info_handler(handler),
    }
  }

  pub fn set_warning_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_warning_handler(handler),
      Codec::Enc(enc) => enc.set_warning_handler(handler),
    }
  }

  pub fn set_error_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_error_handler(handler),
      Codec::Enc(enc) => enc.set_error_handler(handler),
    }
  }

  pub fn set_threads(&mut self, num_threads: core::ffi::c_int) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_threads(num_threads),
      Codec::Enc(enc) => enc.set_threads(num_threads),
    }
  }

  pub fn dump_codec(&mut self, info_flag: i32, output_stream: &mut impl openjp2_core::StringWriter) {
    match self {
      Codec::Dec(dec) => dec.dump_codec(info_flag, output_stream),
      Codec::Enc(enc) => enc.dump_codec(info_flag, output_stream),
    }
  }

  pub fn get_cstr_info_v2(&self) -> Option<openjp2_core::CstrInfoV2> {
    match self {
      Codec::Dec(dec) => dec.get_cstr_info_v2(),
      Codec::Enc(enc) => enc.get_cstr_info_v2(),
    }
  }

  pub fn get_cstr_index_owned(&self) -> Option<openjp2_core::CodestreamIndex> {
    match self {
      Codec::Dec(dec) => dec.get_cstr_index_owned(),
      Codec::Enc(enc) => enc.get_cstr_index_owned(),
    }
  }

  // ---- decompressor surface (wrong-direction calls reproduce the C's
  //      error events through the arm's event manager) ----

  pub fn setup_decoder(&mut self, parameters: &mut openjp2_ctypes::opj_dparameters_t) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.setup_decoder(parameters),
      Codec::Enc(enc) => {
        enc.emit_error_event(
          "Codec provided to the opj_setup_decoder function is not a decompressor handler.\n",
        );
        0
      }
    }
  }

  pub fn decoder_set_strict_mode(&mut self, strict: OPJ_BOOL) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.decoder_set_strict_mode(strict),
      Codec::Enc(enc) => {
        enc.emit_error_event(
          "Codec provided to the opj_decoder_set_strict_mode function is not a decompressor handler.\n",
        );
        0
      }
    }
  }

  pub fn set_decode_limits(&mut self, limits: openjp2_core::DecodeLimits) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_decode_limits(limits),
      Codec::Enc(enc) => {
        enc.emit_error_event(
          "Codec provided to the opj_set_decode_limits function is not a decompressor handler.\n",
        );
        0
      }
    }
  }

  pub fn read_header(
    &mut self,
    p_stream: &mut openjp2_core::Stream,
  ) -> Option<openjp2_core::CoreImage> {
    match self {
      Codec::Dec(dec) => dec.read_header(p_stream),
      Codec::Enc(enc) => {
        enc.emit_error_event(
          "Codec provided to the opj_read_header function is not a decompressor handler.\n",
        );
        None
      }
    }
  }

  pub fn set_decoded_components(
    &mut self,
    components: &[u32],
    apply_color_transforms: OPJ_BOOL,
  ) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_decoded_components(components, apply_color_transforms),
      Codec::Enc(enc) => {
        enc.emit_error_event(
          "Codec provided to the opj_set_decoded_components function is not a decompressor handler.\n",
        );
        0
      }
    }
  }

  pub fn decode(
    &mut self,
    p_stream: &mut openjp2_core::Stream,
    p_image: &mut openjp2_core::CoreImage,
  ) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.decode(p_stream, p_image),
      Codec::Enc(_) => 0,
    }
  }

  pub fn end_decompress(&mut self, p_stream: &mut openjp2_core::Stream) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.end_decompress(p_stream),
      Codec::Enc(_) => 0,
    }
  }

  pub fn set_decode_area(
    &mut self,
    p_image: &mut openjp2_core::CoreImage,
    p_start_x: i32,
    p_start_y: i32,
    p_end_x: i32,
    p_end_y: i32,
  ) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_decode_area(p_image, p_start_x, p_start_y, p_end_x, p_end_y),
      Codec::Enc(_) => 0,
    }
  }

  pub fn read_tile_header(
    &mut self,
    p_stream: &mut openjp2_core::Stream,
    tile_info: &mut openjp2_core::TileInfo,
  ) -> bool {
    match self {
      Codec::Dec(dec) => dec.read_tile_header(p_stream, tile_info),
      Codec::Enc(_) => false,
    }
  }

  pub fn decode_tile_data(
    &mut self,
    p_stream: &mut openjp2_core::Stream,
    p_tile_index: u32,
    p_data: Option<&mut [u8]>,
  ) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.decode_tile_data(p_stream, p_tile_index, p_data),
      Codec::Enc(_) => 0,
    }
  }

  pub fn get_decoded_tile(
    &mut self,
    p_stream: &mut openjp2_core::Stream,
    p_image: &mut openjp2_core::CoreImage,
    tile_index: u32,
  ) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.get_decoded_tile(p_stream, p_image, tile_index),
      Codec::Enc(_) => 0,
    }
  }

  pub fn set_decoded_resolution_factor(&mut self, res_factor: u32) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_decoded_resolution_factor(res_factor),
      Codec::Enc(_) => 0,
    }
  }

  // ---- compressor surface ----

  pub fn setup_encoder(
    &mut self,
    parameters: &mut openjp2_core::EncoderParameters,
    p_image: &mut openjp2_core::CoreImage,
  ) -> OPJ_BOOL {
    match self {
      Codec::Enc(enc) => enc.setup_encoder(parameters, p_image),
      Codec::Dec(_) => 0,
    }
  }

  pub fn encoder_set_extra_options(&mut self, options: &[&str]) -> bool {
    match self {
      Codec::Enc(enc) => enc.encoder_set_extra_options(options),
      Codec::Dec(_) => false,
    }
  }

  pub fn start_compress(
    &mut self,
    p_image: &mut openjp2_core::CoreImage,
    p_stream: &mut openjp2_core::Stream,
  ) -> OPJ_BOOL {
    match self {
      Codec::Enc(enc) => enc.start_compress(p_image, p_stream),
      Codec::Dec(_) => 0,
    }
  }

  pub fn encode(&mut self, p_stream: &mut openjp2_core::Stream) -> OPJ_BOOL {
    match self {
      Codec::Enc(enc) => enc.encode(p_stream),
      Codec::Dec(_) => 0,
    }
  }

  pub fn end_compress(&mut self, p_stream: &mut openjp2_core::Stream) -> OPJ_BOOL {
    match self {
      Codec::Enc(enc) => enc.end_compress(p_stream),
      Codec::Dec(_) => 0,
    }
  }

  pub fn write_tile(
    &mut self,
    p_tile_index: u32,
    p_data: &[u8],
    p_stream: &mut openjp2_core::Stream,
  ) -> OPJ_BOOL {
    match self {
      Codec::Enc(enc) => enc.write_tile(p_tile_index, p_data, p_stream),
      Codec::Dec(_) => 0,
    }
  }
}
