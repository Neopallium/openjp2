//! The merged codec handle behind the single `opj_codec_t*` C-ABI pointer
//! contract.
//!
//! The core split its merged `Codec` into `openjp2::Decoder`/`Encoder` (0.7)
//! with a C-type-free `Result` surface.  This crate keeps one handle type by
//! wrapping either direction, and is the only place that translates between
//! the C-shaped parameters (`opj_dparameters_t`, `opj_cparameters_t`) and the
//! core API.

use openjp2_core::{
  DecodeArea, DecodeLimits, Decoder, DecoderSettings, Encoder, HeaderImage, J2KFormat,
};
use openjp2_ctypes::OPJ_BOOL;

/// The `opj_codec_t*` handle: either a decompressor or a compressor.
pub enum Codec {
  Dec(Decoder),
  Enc(Encoder),
}

fn map_format(format: openjp2_ctypes::OPJ_CODEC_FORMAT) -> Option<J2KFormat> {
  match format {
    openjp2_ctypes::OPJ_CODEC_J2K => Some(J2KFormat::J2K),
    openjp2_ctypes::OPJ_CODEC_JP2 => Some(J2KFormat::JP2),
    _ => None,
  }
}

impl Codec {
  pub fn new_decoder(format: openjp2_ctypes::OPJ_CODEC_FORMAT) -> Option<Self> {
    Some(Codec::Dec(Decoder::new(map_format(format)?).ok()?))
  }

  pub fn new_encoder(format: openjp2_ctypes::OPJ_CODEC_FORMAT) -> Option<Self> {
    Some(Codec::Enc(Encoder::new(map_format(format)?).ok()?))
  }

  pub fn set_info_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_info_handler(handler),
      Codec::Enc(enc) => enc.set_info_handler(handler),
    }
    1i32
  }

  pub fn set_warning_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_warning_handler(handler),
      Codec::Enc(enc) => enc.set_warning_handler(handler),
    }
    1i32
  }

  pub fn set_error_handler(&mut self, handler: impl FnMut(&str) + Send + 'static) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_error_handler(handler),
      Codec::Enc(enc) => enc.set_error_handler(handler),
    }
    1i32
  }

  pub fn set_threads(&mut self, num_threads: core::ffi::c_int) -> OPJ_BOOL {
    let ok = match self {
      Codec::Dec(dec) => dec.set_threads(num_threads),
      Codec::Enc(enc) => enc.set_threads(num_threads),
    };
    ok.is_ok() as OPJ_BOOL
  }

  pub fn dump_codec(&mut self, info_flag: i32, output_stream: &mut impl openjp2_core::StringWriter) {
    match self {
      Codec::Dec(dec) => dec.dump_codec(info_flag, output_stream),
      Codec::Enc(enc) => enc.dump_codec(info_flag, output_stream),
    }
  }

  pub fn get_cstr_info_v2(&self) -> Option<openjp2_core::CstrInfoV2> {
    match self {
      Codec::Dec(dec) => dec.cstr_info(),
      Codec::Enc(enc) => enc.cstr_info(),
    }
  }

  pub fn get_cstr_index_owned(&self) -> Option<openjp2_core::CodestreamIndex> {
    match self {
      Codec::Dec(dec) => dec.cstr_index(),
      Codec::Enc(enc) => enc.cstr_index(),
    }
  }

  // ---- decompressor surface (wrong-direction calls reproduce the C's
  //      error events through the arm's event manager) ----

  pub fn setup_decoder(&mut self, parameters: &mut openjp2_ctypes::opj_dparameters_t) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => {
        let settings = DecoderSettings {
          layers: parameters.cp_layer,
          reduce: parameters.cp_reduce,
          ignore_pclr_cmap_cdef:
            parameters.flags & openjp2_ctypes::consts::opj::OPJ_DPARAMETERS_IGNORE_PCLR_CMAP_CDEF_FLAG
              != 0,
          dump_state: parameters.flags & openjp2_ctypes::consts::opj::OPJ_DPARAMETERS_DUMP_FLAG
            != 0,
          /* The C-ABI thread control stays the process-wide pool
          (opj_codec_set_threads); the native per-decode limit is unused
          here. */
          threads: 0,
        };
        dec.apply_settings(&settings).is_ok() as OPJ_BOOL
      }
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
      Codec::Dec(dec) => dec.set_strict(strict != 0).is_ok() as OPJ_BOOL,
      Codec::Enc(enc) => {
        enc.emit_error_event(
          "Codec provided to the opj_decoder_set_strict_mode function is not a decompressor handler.\n",
        );
        0
      }
    }
  }

  pub fn set_decode_limits(&mut self, limits: DecodeLimits) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_limits(limits).is_ok() as OPJ_BOOL,
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
      Codec::Dec(dec) => dec.read_header(p_stream).ok().map(HeaderImage::into_core_image),
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
      Codec::Dec(dec) => {
        if apply_color_transforms != 0 {
          dec.emit_error_event("apply_color_transforms = OPJ_TRUE is not supported.\n");
          return 0;
        }
        dec.set_components(components).is_ok() as OPJ_BOOL
      }
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
  ) -> Option<openjp2_core::CoreImage> {
    match self {
      Codec::Dec(dec) => dec.decode(p_stream).ok(),
      Codec::Enc(_) => None,
    }
  }

  pub fn end_decompress(&mut self, p_stream: &mut openjp2_core::Stream) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.end_decompress(p_stream).is_ok() as OPJ_BOOL,
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
      Codec::Dec(dec) => dec
        .set_decode_area(
          p_image,
          Some(DecodeArea::new(
            p_start_x as u32,
            p_start_y as u32,
            p_end_x as u32,
            p_end_y as u32,
          )),
        )
        .is_ok() as OPJ_BOOL,
      Codec::Enc(_) => 0,
    }
  }

  pub fn read_tile_header(
    &mut self,
    p_stream: &mut openjp2_core::Stream,
  ) -> Option<openjp2_core::TileInfo> {
    match self {
      Codec::Dec(dec) => dec.read_tile_header(p_stream).ok(),
      Codec::Enc(_) => None,
    }
  }

  pub fn decode_tile_data(
    &mut self,
    p_stream: &mut openjp2_core::Stream,
    p_tile_index: u32,
    p_data: Option<&mut [u8]>,
  ) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.decode_tile_data(p_stream, p_tile_index, p_data).is_ok() as OPJ_BOOL,
      Codec::Enc(_) => 0,
    }
  }

  pub fn get_decoded_tile(
    &mut self,
    p_stream: &mut openjp2_core::Stream,
    tile_index: u32,
  ) -> Option<openjp2_core::CoreImage> {
    match self {
      Codec::Dec(dec) => dec.get_decoded_tile(p_stream, tile_index).ok(),
      Codec::Enc(_) => None,
    }
  }

  pub fn set_decoded_resolution_factor(&mut self, res_factor: u32) -> OPJ_BOOL {
    match self {
      Codec::Dec(dec) => dec.set_resolution_factor(res_factor).is_ok() as OPJ_BOOL,
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
      Codec::Enc(enc) => enc.setup(parameters, p_image).is_ok() as OPJ_BOOL,
      Codec::Dec(_) => 0,
    }
  }

  pub fn encoder_set_extra_options(&mut self, options: &[&str]) -> bool {
    match self {
      Codec::Enc(enc) => enc.extra_options(options).is_ok(),
      Codec::Dec(_) => false,
    }
  }

  pub fn start_compress(
    &mut self,
    p_image: &mut openjp2_core::CoreImage,
    p_stream: &mut openjp2_core::Stream,
  ) -> OPJ_BOOL {
    match self {
      Codec::Enc(enc) => enc.start(p_image, p_stream).is_ok() as OPJ_BOOL,
      Codec::Dec(_) => 0,
    }
  }

  pub fn encode(&mut self, p_stream: &mut openjp2_core::Stream) -> OPJ_BOOL {
    match self {
      Codec::Enc(enc) => enc.encode(p_stream).is_ok() as OPJ_BOOL,
      Codec::Dec(_) => 0,
    }
  }

  pub fn end_compress(&mut self, p_stream: &mut openjp2_core::Stream) -> OPJ_BOOL {
    match self {
      Codec::Enc(enc) => enc.finish(p_stream).is_ok() as OPJ_BOOL,
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
      Codec::Enc(enc) => enc.write_tile(p_tile_index, p_data, p_stream).is_ok() as OPJ_BOOL,
      Codec::Dec(_) => 0,
    }
  }
}
