//! WASM bindings for the pure-Rust OpenJPEG port (`openjp2`).
//!
//! This crate wraps the safe Rust API (NOT the C ABI) for use from
//! JavaScript:
//!
//! - [`decode_to_rgba`] decodes a JP2/J2K file's bytes into tightly packed
//!   RGBA8 pixels, ready for an [`ImageData`]/canvas upload.
//! - [`decode_info`] returns the geometry and component layout of a
//!   codestream without decoding the pixels.
//! - [`encode_rgba_to_j2k`] / [`encode_rgba_to_jp2`] encode RGBA8 pixels to
//!   a codestream / JP2 file.
//!
//! Build with `wasm-pack build wasm` (or
//! `cargo build --target wasm32-unknown-unknown --release` plus
//! `wasm-bindgen`), then from JavaScript:
//!
//! ```js
//! import init, { decode_to_rgba, decode_info } from "./openjp2_wasm.js";
//! await init();
//!
//! const bytes = new Uint8Array(await file.arrayBuffer());
//! const info = decode_info(bytes); // { width, height, components }
//! const rgba = decode_to_rgba(bytes); // Uint8Array of w*h*4 RGBA8
//!
//! const ctx = canvas.getContext("2d");
//! const img = new ImageData(
//!   new Uint8ClampedArray(rgba.buffer, rgba.byteOffset, rgba.length),
//!   info.width,
//!   info.height,
//! );
//! ctx.putImageData(img, 0, 0);
//! ```
//!
//! See `examples/` for a complete HTML page.
//!
//! # Pixel contract
//!
//! - Every decode returns tightly packed RGBA8 (`width * height * 4`
//!   bytes, red first), regardless of the codestream's colour space
//!   (`ColorSpace` / ICC profiles are not colour-managed).
//! - Components must not be subsampled (`dx == dy == 1`); subsampled
//!   codestreams are rejected with an error.
//! - Samples outside 8 bits are shifted (down for `prec > 8`, up for
//!   `prec < 8`) and clamped to `0..=255`; unsigned samples have their
//!   mid-point offset removed first.
//! - Encode always writes 8-bit, 3-component (RGB) images with the
//!   reversible (lossless) or irreversible transform as requested.

use wasm_bindgen::prelude::*;

use openjp2::{DecodeParameters, EncodeParameters, Image};

/// Installs a panic hook that logs panics to the browser console instead
/// of aborting.  Call this once at startup.
#[wasm_bindgen]
pub fn init() {
  console_error_panic_hook::set_once();
}

/// Codestream geometry and component layout (no pixel decoding).
#[wasm_bindgen]
pub struct ImageInfo {
  /// Image width in pixels.
  pub width: u32,
  /// Image height in pixels.
  pub height: u32,
  /// Number of components.
  pub components: u32,
  /// Bits per component.
  pub prec: u32,
  /// Whether the component samples are signed.
  pub sgnd: bool,
}

/// Reads the codestream header and returns the geometry.
///
/// Returns an error `JsValue` describing the problem when the data is
/// not a decodable JP2/J2K codestream.
#[wasm_bindgen]
pub fn decode_info(data: &[u8]) -> Result<ImageInfo, JsValue> {
  let image = Image::from_bytes(data).map_err(js_err)?;
  let comp = &image.components()[0];
  Ok(ImageInfo {
    width: image.width(),
    height: image.height(),
    components: image.num_components(),
    prec: comp.prec,
    sgnd: comp.sgnd != 0,
  })
}

/// Decodes a JP2/J2K codestream to tightly packed RGBA8 pixels.
///
/// `strict` enables strict codestream validation; the default (lenient)
/// matches the safe API's builder default and tolerates truncated
/// codestreams per the documented partial-decode contract.
#[wasm_bindgen]
pub fn decode_to_rgba(data: &[u8], strict: Option<bool>) -> Result<Vec<u8>, JsValue> {
  let mut params = DecodeParameters::new();
  if strict.unwrap_or(false) {
    params = params.strict(true);
  }
  let image = Image::from_bytes_with(data, params).map_err(js_err)?;
  to_rgba(&image).map_err(js_err)
}

/// Decodes with the given width/height forced (unused; kept for API
/// stability of the examples).
#[wasm_bindgen]
pub fn decode_to_rgba_full(data: &[u8]) -> Result<Vec<u8>, JsValue> {
  decode_to_rgba(data, Some(false))
}

/// Encodes tightly packed RGBA8 pixels to a J2K codestream.
///
/// Only the RGB channels are used (the alpha channel is dropped).
/// `lossless` selects the reversible 5-3 transform; `false` uses the
/// irreversible 9-7 transform with one quality layer.
#[wasm_bindgen]
pub fn encode_rgba_to_j2k(width: u32, height: u32, rgba: &[u8], lossless: bool) -> Result<Vec<u8>, JsValue> {
  encode_rgba(width, height, rgba, lossless, false).map_err(js_err)
}

/// Encodes tightly packed RGBA8 pixels to a JP2 file (like
/// [`encode_rgba_to_j2k`], but wrapped in the JP2 container).
#[wasm_bindgen]
pub fn encode_rgba_to_jp2(width: u32, height: u32, rgba: &[u8], lossless: bool) -> Result<Vec<u8>, JsValue> {
  encode_rgba(width, height, rgba, lossless, true).map_err(js_err)
}

/// Converts the decoded component planes to tightly packed RGBA8.
///
/// Handles 1 (gray), 2 (gray+alpha), 3 (RGB) and 4 (RGBA) components;
/// rejects subsampled components (`dx != 1 || dy != 1`).
fn to_rgba(image: &Image) -> Result<Vec<u8>, String> {
  let w = image.width() as usize;
  let h = image.height() as usize;
  let comps = image.components();
  if comps.is_empty() {
    return Err("image has no components".into());
  }
  for c in comps {
    if c.dx != 1 || c.dy != 1 || c.w as usize != w || c.h as usize != h {
      return Err(format!(
        "subsampled component (dx={} dy={} {}x{}) is not supported by the WASM wrapper",
        c.dx, c.dy, c.w, c.h
      ));
    }
  }
  let n = comps.len();
  let mut out = vec![0u8; w * h * 4];
  let shift = |v: i32, prec: u32, sgnd: bool| -> u8 {
    let mut s = v;
    if !sgnd {
      s -= 1 << (prec - 1);
    }
    if prec > 8 {
      s >>= prec - 8;
    } else if prec < 8 {
      s <<= 8 - prec;
    }
    s.clamp(0, 255) as u8
  };
  let get = |ci: usize, i: usize| shift(comps[ci].data[i], comps[ci].prec, comps[ci].sgnd != 0);
  match n {
    1 => {
      for (i, px) in out.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let g = get(0, i);
        px[0] = g;
        px[1] = g;
        px[2] = g;
        px[3] = 255;
      }
    }
    2 => {
      for (i, px) in out.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let g = get(0, i);
        px[0] = g;
        px[1] = g;
        px[2] = g;
        px[3] = get(1, i);
      }
    }
    3 | 4 => {
      for (i, px) in out.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        px[0] = get(0, i);
        px[1] = get(1, i);
        px[2] = get(2, i);
        px[3] = if n == 4 { get(3, i) } else { 255 };
      }
    }
    _ => {
      return Err(format!(
        "unsupported component count {n} (1, 2, 3 or 4 supported)"
      ))
    }
  }
  Ok(out)
}

/// Builds a 3-component 8-bit image from RGBA8 and encodes it.
fn encode_rgba(
  width: u32,
  height: u32,
  rgba: &[u8],
  lossless: bool,
  jp2: bool,
) -> Result<Vec<u8>, String> {
  let n = width as usize * height as usize;
  if rgba.len() < n * 4 {
    return Err(format!(
      "rgba buffer is {} bytes, expected {}",
      rgba.len(),
      n * 4
    ));
  }
  let mut r = Vec::with_capacity(n);
  let mut g = Vec::with_capacity(n);
  let mut b = Vec::with_capacity(n);
  for px in rgba[..n * 4].as_chunks::<4>().0 {
    r.push(px[0] as i32);
    g.push(px[1] as i32);
    b.push(px[2] as i32);
  }
  let comp = |data: Vec<i32>| openjp2::CoreImageComp {
    dx: 1,
    dy: 1,
    w: width,
    h: height,
    x0: 0,
    y0: 0,
    prec: 8,
    bpp: 8,
    sgnd: 0,
    resno_decoded: 0,
    factor: 0,
    alpha: 0,
    data,
  };
  let core = openjp2::CoreImage {
    x0: 0,
    y0: 0,
    x1: width,
    y1: height,
    numcomps: 3,
    color_space: openjp2::ColorSpace::Srgb,
    comps: vec![comp(r), comp(g), comp(b)],
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let image = Image::from_core_image(core);
  let mut params = EncodeParameters::new().num_resolutions(6).num_layers(1);
  if !lossless {
    params = params.irreversible(true);
  }
  if jp2 {
    image.encode_jp2(params)
  } else {
    image.encode(params)
  }
  .map_err(|e| e.to_string())
}

fn js_err(e: impl std::fmt::Display) -> JsValue {
  JsValue::from_str(&e.to_string())
}
