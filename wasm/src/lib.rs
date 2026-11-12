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

/// The openjp2-wasm crate version.
#[wasm_bindgen]
pub fn version() -> String {
  env!("CARGO_PKG_VERSION").to_string()
}

/// Codestream geometry, layout and coding defaults (header only, no
/// pixel decoding).
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
  /// Number of resolutions in the codestream (DWT levels + 1);
  /// `reduce` values `0..numresolutions-1` are decodable.
  pub numresolutions: u32,
  /// Declared colour space (`Unknown` / `Srgb` / `Gray` / `Sycc` /
  /// `Eycc` / `Cmyk`).
  #[wasm_bindgen(getter_with_clone)]
  pub color_space: String,
  /// Whether the codestream carries an ICC colour profile.
  pub has_icc_profile: bool,
  /// Number of quality layers in the default tile.
  pub num_layers: u32,
  /// Progression order (`0` = LRCP, `1` = RLCP, `2` = RPCL, `3` = PCRL,
  /// `4` = CPRL).
  pub progression: u32,
  /// Multiple-component transform mode (`0`/`1`/`2`).
  pub mct: u32,
  /// Tiles horizontally.
  pub tiles_h: u32,
  /// Tiles vertically.
  pub tiles_v: u32,
  /// Tile width.
  pub tile_w: u32,
  /// Tile height.
  pub tile_h: u32,
  /// Code-block width exponent (`2^codeblock_w` pixels).
  pub codeblock_w: u32,
  /// Code-block height exponent (`2^codeblock_h` pixels).
  pub codeblock_h: u32,
  /// Horizontal component subsampling (XRsiz).
  pub dx: u32,
  /// Vertical component subsampling (YRsiz).
  pub dy: u32,
}

/// Reads the codestream header and returns the geometry and coding
/// defaults.
///
/// Returns an error `JsValue` describing the problem when the data is
/// not a decodable JP2/J2K codestream.
#[wasm_bindgen]
pub fn decode_info(data: &[u8]) -> Result<ImageInfo, JsValue> {
  let format = openjp2::detect_format(data).map_err(js_err)?;
  let mut stream = openjp2::Stream::from_bytes(data.to_vec());
  let mut decoder = openjp2::Decoder::new(format).map_err(js_err)?;
  let header = decoder.read_header(&mut stream).map_err(js_err)?;
  let width = header.width();
  let height = header.height();
  let core = header.into_core_image();
  let comp = core
    .comps
    .first()
    .ok_or_else(|| js_err("image has no components"))?;
  let info = decoder.cstr_info();
  let default_tile = info.as_ref().map(|i| &i.default_tile);
  let tccp = default_tile.and_then(|t| t.tccps.first());
  let progression = match default_tile.map(|t| t.prg) {
    Some(openjp2::ProgressionOrder::Lrcp) => 0,
    Some(openjp2::ProgressionOrder::Rlcp) => 1,
    Some(openjp2::ProgressionOrder::Rpcl) => 2,
    Some(openjp2::ProgressionOrder::Pcrl) => 3,
    Some(openjp2::ProgressionOrder::Cprl) => 4,
    _ => 0,
  };
  Ok(ImageInfo {
    width,
    height,
    components: core.numcomps,
    prec: comp.prec,
    sgnd: comp.sgnd != 0,
    numresolutions: tccp.map(|t| t.numresolutions).unwrap_or(1).max(1),
    color_space: format!("{:?}", core.color_space),
    has_icc_profile: !core.icc_profile_buf.is_empty(),
    num_layers: default_tile.map(|t| t.numlayers).unwrap_or(1),
    progression,
    mct: default_tile.map(|t| t.mct).unwrap_or(0),
    tiles_h: info.as_ref().map(|i| i.tw).unwrap_or(1),
    tiles_v: info.as_ref().map(|i| i.th).unwrap_or(1),
    tile_w: info.as_ref().map(|i| i.tdx).unwrap_or(width),
    tile_h: info.as_ref().map(|i| i.tdy).unwrap_or(height),
    codeblock_w: tccp.map(|t| t.cblkw).unwrap_or(6),
    codeblock_h: tccp.map(|t| t.cblkh).unwrap_or(6),
    dx: comp.dx,
    dy: comp.dy,
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

/// Decode outcome with a completeness flag (the explorer's probe).
///
/// `complete` reflects whether every component buffer reached the
/// declared geometry.  Note: the lenient decoder zero-fills undecoded
/// regions, so a truncated codestream can still report full-length
/// buffers; for a coverage verdict, check whether the codestream's
/// markers reach EOC (the playground does this on the JS side).
#[wasm_bindgen]
pub struct DecodeProbe {
  /// Declared image width (component plane).
  pub width: u32,
  /// Declared image height (component plane).
  pub height: u32,
  /// Whether every pixel of the declared geometry was decoded.
  pub complete: bool,
  /// Number of components in the image.
  pub components: u32,
  /// Declared colour space.
  #[wasm_bindgen(getter_with_clone)]
  pub color_space: String,
  /// Tightly packed RGBA8 (width * height * 4); empty when only the
  /// header survived.
  #[wasm_bindgen(getter_with_clone)]
  pub pixels: Vec<u8>,
}

/// Decodes leniently (or strictly) and reports how much of the image
/// survived.
///
/// This is the truncation-slider backend: it never fails on a truncated
/// codestream that has at least a readable header.
#[wasm_bindgen]
pub fn decode_probe(data: &[u8], strict: Option<bool>) -> Result<DecodeProbe, JsValue> {
  let opts = DecodeOptions {
    strict: strict.unwrap_or(false),
    ..Default::default()
  };
  decode_probe_opt(data, &opts)
}

/// Decoder settings for [`decode_probe_opt`] (the safe-API builder
/// mirrors: `DecodeParameters`).
#[wasm_bindgen]
#[derive(Default)]
pub struct DecodeOptions {
  /// Strict codestream validation (default: lenient).
  pub strict: bool,
  /// Resolution reduction factor (`0` = full, `1` = half, ...; must be
  /// below the codestream's resolution count).
  pub reduce: u32,
  /// Number of quality layers to decode (`0` = all layers).
  pub layers: u32,
  /// Decode-area window `x0,y0,x1,y1` (end exclusive; all zeros = the
  /// whole image).
  pub x0: u32,
  /// Decode-area window top.
  pub y0: u32,
  /// Decode-area window right.
  pub x1: u32,
  /// Decode-area window bottom.
  pub y1: u32,
  /// Ignore the pclr/cmap/cdef boxes (the C's
  /// `OPJ_DPARAMETERS_IGNORE_PCLR_CMAP_CDEF_FLAG`).
  pub ignore_pclr_cmap_cdef: bool,
}

#[wasm_bindgen]
impl DecodeOptions {
  /// Default settings: lenient, full resolution, all layers, whole
  /// image.
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self::default()
  }
}

/// Decodes with the given settings and reports how much of the image
/// survived (see [`decode_probe`]).
#[wasm_bindgen]
pub fn decode_probe_opt(data: &[u8], opts: &DecodeOptions) -> Result<DecodeProbe, JsValue> {
  let mut params = DecodeParameters::new();
  if opts.strict {
    params = params.strict(true);
  }
  if opts.reduce > 0 {
    params = params.reduce(opts.reduce);
  }
  if opts.layers > 0 {
    params = params.layers(opts.layers);
  }
  if opts.x1 > opts.x0 && opts.y1 > opts.y0 {
    params = params.decode_area(Some(openjp2::DecodeArea::new(
      opts.x0, opts.y0, opts.x1, opts.y1,
    )));
  }
  if opts.ignore_pclr_cmap_cdef {
    params = params.ignore_pclr_cmap_cdef();
  }
  let image = Image::from_bytes_with(data, params).map_err(js_err)?;
  let comps = image.components();
  /* The image bounds stay full-size; the component planes carry the
  reduction (and subsampling).  Use the first component's plane dims,
  like the image-crate conversion does. */
  let (w, h) = comps
    .first()
    .map(|c| (c.w as usize, c.h as usize))
    .unwrap_or((0, 0));
  let total = w * h;
  let mut pixels = vec![0u8; total * 4];
  let mut complete = !comps.is_empty();
  for (ci, c) in comps.iter().enumerate().take(4) {
    if c.dx != 1 || c.dy != 1 {
      continue;
    }
    let shift = |v: i32| -> u8 {
      if c.sgnd != 0 {
        let old_max = 1i64 << (c.prec - 1);
        const NEW_MAX: i64 = 1 << 7;
        ((((v as i64) * NEW_MAX) / old_max) + NEW_MAX) as u8
      } else {
        let old_max = (1u64 << c.prec) - 1;
        const NEW_MAX: u64 = (1 << 8) - 1;
        (((v as u64) * NEW_MAX) / old_max) as u8
      }
    };
    if c.data.len() < total {
      complete = false;
    }
    let n = c.data.len().min(total);
    match (comps.len(), ci) {
      (1, 0) => {
        for (i, px) in pixels.as_chunks_mut::<4>().0[..n].iter_mut().enumerate() {
          let g = shift(c.data[i]);
          px[0] = g;
          px[1] = g;
          px[2] = g;
          px[3] = 255;
        }
      }
      (2, 0) => {
        for (i, px) in pixels.as_chunks_mut::<4>().0[..n].iter_mut().enumerate() {
          let g = shift(c.data[i]);
          px[0] = g;
          px[1] = g;
          px[2] = g;
          px[3] = 255;
        }
      }
      (2, 1) => {
        for (i, px) in pixels.as_chunks_mut::<4>().0[..n].iter_mut().enumerate() {
          px[3] = shift(c.data[i]);
        }
      }
      (_, 0..=3) => {
        for (i, px) in pixels.as_chunks_mut::<4>().0[..n].iter_mut().enumerate() {
          px[ci] = shift(c.data[i]);
          if ci == 2 {
            px[3] = 255;
          }
        }
      }
      _ => {}
    }
  }
  Ok(DecodeProbe {
    width: w as u32,
    height: h as u32,
    complete,
    components: image.num_components(),
    color_space: format!("{:?}", image.color_space()),
    pixels,
  })
}

/// Encodes tightly packed RGBA8 pixels to a J2K codestream.
///
/// Only the RGB channels are used (the alpha channel is dropped).
/// `lossless` selects the reversible 5-3 transform; `false` uses the
/// irreversible 9-7 transform with one quality layer.
#[wasm_bindgen]
pub fn encode_rgba_to_j2k(
  width: u32,
  height: u32,
  rgba: &[u8],
  lossless: bool,
) -> Result<Vec<u8>, JsValue> {
  let opts = EncodeOptions {
    lossless,
    jp2: false,
    ..Default::default()
  };
  encode_rgba_opt(width, height, rgba, &opts)
}

/// Encodes tightly packed RGBA8 pixels to a JP2 file (like
/// [`encode_rgba_to_j2k`], but wrapped in the JP2 container).
#[wasm_bindgen]
pub fn encode_rgba_to_jp2(
  width: u32,
  height: u32,
  rgba: &[u8],
  lossless: bool,
) -> Result<Vec<u8>, JsValue> {
  let opts = EncodeOptions {
    lossless,
    jp2: true,
    ..Default::default()
  };
  encode_rgba_opt(width, height, rgba, &opts)
}

/// Encoder settings for [`encode_rgba_opt`] (the safe-API builder
/// mirrors: `EncodeParameters`).
#[wasm_bindgen]
pub struct EncodeOptions {
  /// Reversible 5-3 (lossless) or irreversible 9-7 transform.
  pub lossless: bool,
  /// Number of resolutions (`0` = the encoder default of 6).
  pub num_resolutions: u32,
  /// Number of quality layers (`0` = one layer).
  pub num_layers: u32,
  /// Packet progression order (`0` = LRCP, `1` = RLCP, `2` = RPCL,
  /// `3` = PCRL, `4` = CPRL).
  pub progression: u32,
  /// Tile width (`0` = single tile).
  pub tile_w: u32,
  /// Tile height (`0` = single tile).
  pub tile_h: u32,
  /// Compression ratio for rate-distortion allocation (`0` = off,
  /// fixed layers instead).
  pub rate: f32,
  /// Explicit per-layer rate targets as a comma-separated list (e.g.
  /// `"100,20,5,0"`; `0` = lossless final layer).  When set, it takes
  /// precedence over `rate` and the automatic ladder.
  #[wasm_bindgen(getter_with_clone)]
  pub rates: Option<String>,
  /// COM comment marker embedded in the codestream.
  #[wasm_bindgen(getter_with_clone)]
  pub comment: Option<String>,
  /// Wrap the codestream in the JP2 container.
  pub jp2: bool,
}

#[wasm_bindgen]
impl EncodeOptions {
  /// Default settings: lossless 5-3, 6 resolutions, one layer, LRCP,
  /// single tile, no ratio allocation, JP2 container.
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self::default()
  }
}

impl Default for EncodeOptions {
  fn default() -> Self {
    Self {
      lossless: true,
      num_resolutions: 6,
      num_layers: 1,
      progression: 0,
      tile_w: 0,
      tile_h: 0,
      rate: 0.0,
      rates: None,
      comment: None,
      jp2: true,
    }
  }
}

/// Encodes tightly packed RGBA8 pixels with the given settings (see
/// [`EncodeOptions`]).
#[wasm_bindgen]
pub fn encode_rgba_opt(
  width: u32,
  height: u32,
  rgba: &[u8],
  opts: &EncodeOptions,
) -> Result<Vec<u8>, JsValue> {
  let n = width as usize * height as usize;
  if rgba.len() < n * 4 {
    return Err(js_err(format!(
      "rgba buffer is {} bytes, expected {}",
      rgba.len(),
      n * 4
    )));
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
  let mut params = EncodeParameters::new();
  if opts.num_resolutions > 0 {
    params = params.num_resolutions(opts.num_resolutions);
  }
  if !opts.lossless {
    params = params.irreversible(true);
  }
  /* Quality layers.  Rate targets are what actually split the code-block
  passes across layers: a bare layer count leaves everything in the last
  layer, so decoding N layers would look identical to all of them.
  Priority: explicit `rates`, then a single `rate`, then an automatic
  geometric ladder for `num_layers > 1`. */
  let explicit_rates: Vec<f32> = opts
    .rates
    .as_deref()
    .unwrap_or("")
    .split(',')
    .filter_map(|s| s.trim().parse::<f32>().ok())
    .collect();
  if !explicit_rates.is_empty() {
    for rate in explicit_rates {
      params = params.layer_rate(rate);
    }
  } else if opts.rate > 0.0 {
    params = params.layer_rate(opts.rate);
  } else if opts.num_layers > 1 {
    let n = opts.num_layers.min(100);
    for i in 0..n {
      // geometric ladder from 100:1 down to the lossless final layer
      let ratio = if i == n - 1 {
        0.0
      } else {
        100.0f32.powf(1.0 - i as f32 / (n - 1) as f32)
      };
      params = params.layer_rate(ratio);
    }
  } else if opts.num_layers > 0 {
    params = params.num_layers(opts.num_layers);
  }
  if opts.tile_w > 0 && opts.tile_h > 0 {
    params = params.tile_size(opts.tile_w, opts.tile_h);
  }
  params = params.progression_order(match opts.progression {
    1 => openjp2::ProgressionOrder::Rlcp,
    2 => openjp2::ProgressionOrder::Rpcl,
    3 => openjp2::ProgressionOrder::Pcrl,
    4 => openjp2::ProgressionOrder::Cprl,
    _ => openjp2::ProgressionOrder::Lrcp,
  });
  if let Some(comment) = &opts.comment {
    params = params.comment(comment);
  }
  if opts.jp2 {
    image.encode_jp2(params)
  } else {
    image.encode(params)
  }
  .map_err(js_err)
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
  // Scale to 8 bits exactly like the image-crate conversion (comp_u8):
  // the decoder has already applied the DC level shift, so unsigned
  // samples span 0..2^prec-1 and signed ones -2^(prec-1)..2^(prec-1)-1.
  let shift = |v: i32, prec: u32, sgnd: bool| -> u8 {
    if sgnd {
      let old_max = 1i64 << (prec - 1);
      const NEW_MAX: i64 = 1 << 7;
      ((((v as i64) * NEW_MAX) / old_max) + NEW_MAX) as u8
    } else {
      let old_max = (1u64 << prec) - 1;
      const NEW_MAX: u64 = (1 << 8) - 1;
      (((v as u64) * NEW_MAX) / old_max) as u8
    }
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

fn js_err(e: impl std::fmt::Display) -> JsValue {
  JsValue::from_str(&e.to_string())
}
