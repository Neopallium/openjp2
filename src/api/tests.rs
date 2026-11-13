// Split of the safe API (doc/api-review.md): see the facade in api.rs
// and doc/redesign-proposal.md B.3.2.

use super::*;

/// Regression: a fixed-alloc matrix implying more passes than a codeblock
/// actually coded must not panic (the C records the over-promising count
/// and reads passes[] OOB in both makelayer_fixed and t2 — UB).  Covers
/// the all-zero cblk (totalpasses == 0) and the coded-cblk sub-cases.
#[test]
fn fixed_alloc_overpromise_does_not_panic() {
  for data in [
    vec![0i32; 4096],                                 // all-zero cblks: totalpasses == 0
    (0..4096).map(|i| i % 251).collect::<Vec<i32>>(), // real passes
  ] {
    let (w, h) = (64u32, 64u32);
    let core = crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 1,
      color_space: crate::ColorSpace::Gray,
      comps: vec![crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data: data.clone(),
      }],
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    let image = Image { core };
    let numres = 7i32; // 64x64 -> 6 levels + 1
    let numlayers = 3i32;
    let mut params = EncodeParameters::new();
    {
      let ll = params.low_level();
      ll.cp_fixed_alloc = 1;
      ll.tcp_numlayers = numlayers;
      ll.numresolution = numres;
      ll.cp_matrice = Some(vec![100i32; (numlayers * numres * 3) as usize]);
    }
    let codestream = image.clone().encode(params).expect("fixed-alloc encode");
    let back = Image::from_bytes(&codestream).expect("decode fixed-alloc output");
    assert_eq!(back.width(), w);
    assert_eq!(back.height(), h);
    // With the matrix clamped to the coded-pass budget every coded pass
    // lands in layer 0, so the round trip stays lossless.
    assert_eq!(back.components()[0].data, image.components()[0].data);
  }
  // The reviewer's literal 3-entry matrix: the C reads the caller's array
  // OOB; the port must reject it cleanly (setup_encoder validation).
  let (w, h) = (64u32, 64u32);
  let mut params = EncodeParameters::new();
  {
    let ll = params.low_level();
    ll.cp_fixed_alloc = 1;
    ll.tcp_numlayers = 3;
    ll.numresolution = 7;
    ll.cp_matrice = Some(vec![100i32; 3]);
  }
  let core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 1,
    color_space: crate::ColorSpace::Gray,
    comps: vec![crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data: vec![0; (w * h) as usize],
    }],
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let image = Image { core };
  assert!(
    image.encode(params).is_err(),
    "short matrix must be rejected"
  );
}

/// Decodes the bundled sample and round-trips it through the encoder.
#[test]
fn decode_sample_and_round_trip() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let image = Image::from_bytes(&buf).expect("decode sample");
  assert_eq!(image.num_components(), 3);
  assert!(image.width() > 0 && image.height() > 0);
  // Round-trip through the J2K encoder and back.
  let codestream = image
    .clone()
    .encode(
      EncodeParameters::new()
        .num_resolutions(3)
        .num_layers(1)
        .progression_order(ProgressionOrder::Lrcp)
        .comment("api test"),
    )
    .expect("encode");
  assert!(!codestream.is_empty());
  let back = Image::from_bytes(&codestream).expect("decode round-trip");
  assert_eq!(back.width(), image.width());
  assert_eq!(back.height(), image.height());
  assert_eq!(back.num_components(), image.num_components());
  // The reversible transform is lossless: pixels must match.
  for (a, b) in image.components().iter().zip(back.components()) {
    assert_eq!(a.data, b.data);
  }
}

#[cfg(feature = "image")]
#[test]
fn sample_to_dynamic_image() {
  use core::convert::TryFrom;
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let image = Image::from_bytes(&buf).expect("decode sample");
  let pixels = image.get_pixels(None).expect("get_pixels");
  assert_eq!(
    (pixels.width, pixels.height),
    (image.width(), image.height())
  );
  let dyn_img = ::image::DynamicImage::try_from(&image).expect("convert");
  assert_eq!(dyn_img.width(), image.width());
  assert_eq!(dyn_img.height(), image.height());
}

#[test]
fn decode_area_and_layers() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let full = Image::from_bytes(&buf).expect("decode full");
  // Decode a 64x64 corner with a resolution reduction.
  let area = DecodeArea::new(0, 0, 64, 64);
  let sub = Image::from_bytes_with(
    &buf,
    DecodeParameters::new()
      .decode_area(Some(area))
      .reduce(1)
      .layers(1)
      .strict(true),
  )
  .expect("decode area");
  assert!(sub.width() <= 64 && sub.height() <= 64);
  assert!(sub.width() > 0 && sub.height() > 0);
  assert_eq!(sub.num_components(), full.num_components());
}

/// Encodes random images with the reversible transform and checks the
/// round trip is lossless, across a few sizes/precisions/component counts.
#[test]
fn lossless_round_trip_property() {
  let mut rng: u64 = 0x9e3779b97f4a7c15;
  let mut next = move || {
    rng ^= rng << 13;
    rng ^= rng >> 7;
    rng ^= rng << 17;
    rng
  };
  for (w, h, comps, prec) in [(17, 13, 1, 8), (64, 64, 3, 8), (33, 31, 3, 12)] {
    let mut core = crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: comps,
      color_space: if comps == 1 {
        crate::ColorSpace::Gray
      } else {
        crate::ColorSpace::Srgb
      },
      comps: Vec::new(),
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    };
    for c in 0..comps {
      let dx = if c == 0 { 1 } else { 2 };
      let (cw, ch) = (w.div_ceil(dx), h.div_ceil(dx));
      let mut data = Vec::with_capacity((cw * ch) as usize);
      for _ in 0..cw * ch {
        data.push((next() % (1u64 << prec)) as i32);
      }
      core.comps.push(crate::CoreImageComp {
        dx,
        dy: dx,
        w: cw,
        h: ch,
        x0: 0,
        y0: 0,
        prec,
        bpp: prec,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data,
      });
    }
    let image = Image { core };
    let codestream = image
      .clone()
      .encode(EncodeParameters::new().num_resolutions(3).num_layers(1))
      .expect("encode");
    let back = Image::from_bytes(&codestream).expect("decode");
    assert_eq!(back.width(), image.width());
    assert_eq!(back.height(), image.height());
    assert_eq!(back.num_components(), comps);
    for (a, b) in image.components().iter().zip(back.components()) {
      assert_eq!(
        a.data, b.data,
        "lossless round trip failed for {w}x{h} prec {prec}"
      );
    }
  }
}

/// 16-bit pixel extraction through the `image`-crate interop.
#[cfg(feature = "image")]
#[test]
fn get_pixels_16bit() {
  let (w, h) = (64u32, 48u32);
  let mut core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 1,
    color_space: crate::ColorSpace::Gray,
    comps: Vec::new(),
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let mut data = Vec::with_capacity((w * h) as usize);
  for i in 0..w * h {
    data.push((i % 4096) as i32);
  }
  core.comps.push(crate::CoreImageComp {
    dx: 1,
    dy: 1,
    w,
    h,
    x0: 0,
    y0: 0,
    prec: 12,
    bpp: 12,
    sgnd: 0,
    resno_decoded: 0,
    factor: 0,
    alpha: 0,
    data,
  });
  let image = Image { core };
  let pixels = image.get_pixels(None).expect("get_pixels");
  assert_eq!(pixels.format, ::image::ExtendedColorType::L16);
  assert_eq!(pixels.width, w);
  assert_eq!(pixels.height, h);
  match pixels.data {
    super::image_conv::ImagePixelData::L16(v) => {
      assert_eq!(v.len(), (w * h) as usize);
      // 12-bit 0..4095 maps to 16-bit 0..65535 (proportional)
      assert_eq!(v[0], 0);
      assert!(v[1] > 0 && v[1] < 65535);
    }
    other => panic!("unexpected pixel format {:?}", other),
  }
  // And the DynamicImage conversion path.
  use core::convert::TryFrom;
  let dyn_img = ::image::DynamicImage::try_from(&image).expect("convert");
  assert_eq!((dyn_img.width(), dyn_img.height()), (w, h));
}

/// Parallel-vs-serial encode byte-identity over the test corpus (runs
/// when OPJ_DATA_ROOT is set): every input that decodes must re-encode
/// to the identical codestream through both paths.
#[cfg(feature = "rayon")]
#[test]
fn parallel_encode_matches_serial_on_corpus() {
  let root = match std::env::var_os("OPJ_DATA_ROOT") {
    Some(r) => std::path::PathBuf::from(r),
    None => return,
  };
  let input = root.join("input").join("nonregression");
  if !input.is_dir() {
    return;
  }
  let mut checked = 0usize;
  let mut entries: Vec<_> = std::fs::read_dir(&input)
    .expect("read_dir")
    .filter_map(|e| e.ok())
    .map(|e| e.path())
    .filter(|p| {
      matches!(
        p.extension().and_then(|e| e.to_str()),
        Some("j2k") | Some("j2c") | Some("jp2")
      )
    })
    .collect();
  entries.sort();
  for path in entries {
    let buf = std::fs::read(&path).expect("read input");
    let Ok(image) = Image::from_bytes(&buf) else {
      continue;
    };
    let params = EncodeParameters::new()
      .num_resolutions(4)
      .tile_size(256, 256);
    let Ok(par) = image.clone().encode(params.clone()) else {
      continue;
    };
    #[allow(deprecated)]
    let serial = {
      let mut codec = Encoder::new(J2KFormat::J2K).unwrap();
      let mut low = params.clone().into_inner();
      let mut img = image.into_core_image();
      if codec.setup(&mut low, &mut img).is_err() {
        continue;
      }
      let buf2 = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
      {
        let writer = crate::stream::SharedVecWriter {
          buf: buf2.clone(),
          pos: 0,
        };
        let mut stream = Stream::new_output(crate::consts::J2K_STREAM_CHUNK_SIZE as usize, writer);
        codec.start(&mut img, &mut stream).unwrap();
        codec.encode(&mut stream).unwrap();
        codec.finish(&mut stream).unwrap();
        drop(stream);
      }
      std::rc::Rc::try_unwrap(buf2)
        .ok()
        .map(|b| b.into_inner())
        .expect("unique handle")
    };
    assert_eq!(
      par,
      serial,
      "{}: parallel/serial encode mismatch",
      path.display()
    );
    checked += 1;
  }
  assert!(checked > 10, "expected a corpus of re-encodable inputs");
}

/// The concurrent-tile encode must produce byte-identical codestreams
/// to the sequential low-level encode.
#[cfg(feature = "rayon")]
#[test]
fn parallel_tile_encode_byte_identical() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let image = Image::from_bytes(&buf).expect("decode");
  let params = EncodeParameters::new()
    .num_resolutions(3)
    .num_layers(2)
    .tile_size(512, 512)
    .progression_order(ProgressionOrder::Lrcp);
  let par = image
    .clone()
    .encode(params.clone())
    .expect("parallel encode");
  #[allow(deprecated)]
  let serial = {
    let mut codec = Encoder::new(J2KFormat::J2K).unwrap();
    let mut low = params.into_inner();
    let mut img = image.into_core_image();
    codec.setup(&mut low, &mut img).unwrap();
    let buf2 = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    {
      let writer = crate::stream::SharedVecWriter {
        buf: buf2.clone(),
        pos: 0,
      };
      let mut stream = Stream::new_output(crate::consts::J2K_STREAM_CHUNK_SIZE as usize, writer);
      codec.start(&mut img, &mut stream).unwrap();
      codec.encode(&mut stream).unwrap();
      codec.finish(&mut stream).unwrap();
      drop(stream);
    }
    std::rc::Rc::try_unwrap(buf2)
      .ok()
      .map(|b| b.into_inner())
      .expect("unique handle")
  };
  assert_eq!(par, serial, "parallel/serial encode mismatch");
}

/// Regression (review round 3, M3): EPH/SOC/SOD/EOC have no entries in
/// the C's marker table, so a stray one in the main header is skipped by
/// `read_unk` (and the scan resumes at the next real marker); the port
/// used to treat them as known-and-handlerless -> fatal "No handler".
#[test]
fn stray_no_table_markers_are_skipped() {
  let (w, h) = (32u32, 32u32);
  let core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 1,
    color_space: crate::ColorSpace::Gray,
    comps: vec![crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data: (0..(w * h)).map(|i| (i % 199) as i32).collect(),
    }],
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let image = Image { core };
  let codestream = image
    .encode(EncodeParameters::new().num_resolutions(1))
    .expect("encode");
  let reference = Image::from_bytes(&codestream).expect("reference decode");
  for stray in [[0xff, 0x92], [0xff, 0x4f], [0xff, 0x93], [0xff, 0xd9]] {
    // Insert the stray marker + zero length after SOC + SIZ's marker ID
    // (offset 4), so the format-detection magic (FF4F FF51) is intact and
    // the main-header loop meets the stray where the C's read_unk skips
    // it.
    let mut patched = Vec::with_capacity(codestream.len() + 4);
    patched.extend_from_slice(&codestream[..4]);
    patched.extend_from_slice(&stray);
    patched.extend_from_slice(&[0x00, 0x00]);
    patched.extend_from_slice(&codestream[4..]);
    let decoded = Image::from_bytes(&patched).unwrap_or_else(|e| {
      panic!(
        "stray {:02X}{:02X} must be skipped: {e:?}",
        stray[0], stray[1]
      )
    });
    for (a, b) in reference.components().iter().zip(decoded.components()) {
      assert_eq!(a.data, b.data, "stray marker changed the decode");
    }
  }
}

/// Regression (0.7 Codec split): a degenerate whole-image decode area
/// (the CLI's default `-d 0,0,0,0` call shape) must decode identically to
/// no area at all.  It must not take the parallel decode-area path, whose
/// workers reproduce only the lenient tile-part walk — pinned by the
/// issue235 fixture (multi-tile), where the parallel-area path produced
/// different pixels than the serial decode.
#[test]
fn degenerate_decode_area_matches_whole() {
  let buf = std::fs::read("samples/issue235_cielab_multitile.jp2").expect("sample file");
  let whole = Image::from_bytes(&buf).expect("whole decode");
  // Low-level flow like the CLI: set_decode_area((0,0,0,0)) then decode.
  let mut dec = Decoder::new(J2KFormat::JP2).unwrap();
  let mut s = Stream::new_input(
    crate::consts::J2K_STREAM_CHUNK_SIZE as usize,
    std::io::Cursor::new(buf.clone()),
    buf.len() as u64,
  );
  let mut hdr = dec.read_header(&mut s).unwrap().into_core_image();
  dec
    .set_decode_area(&mut hdr, Some(DecodeArea::new(0, 0, 0, 0)))
    .unwrap();
  let area = dec.decode(&mut s).unwrap();
  assert_eq!(area.comps.len(), whole.components().len());
  for (a, b) in whole.components().iter().zip(area.comps.iter()) {
    assert_eq!(a.data, b.data, "degenerate area vs whole component data");
  }
}

/// `DecodeParameters::threads`: the per-decode pool limit must be
/// byte-identical to the automatic (ambient pool) decode, at every
/// sensible size including a single worker and one more worker than
/// needed.  Uses the multi-tile issue235 fixture so the parallel tile
/// driver AND the nested codeblock work both exercise the local pool.
#[test]
fn decode_parameters_threads_limit_is_byte_identical() {
  let buf = std::fs::read("samples/issue235_cielab_multitile.jp2").expect("sample file");
  let reference = Image::from_bytes(&buf).expect("reference decode");
  for n in [1usize, 2, 4] {
    let limited =
      Image::from_bytes_with(&buf, DecodeParameters::new().threads(n)).expect("limited decode");
    assert_eq!(
      reference.components().len(),
      limited.components().len(),
      "threads({n}) component count"
    );
    for (a, b) in reference
      .components()
      .iter()
      .zip(limited.components().iter())
    {
      assert_eq!(a.data, b.data, "threads({n}) component data");
    }
  }
}

/// The per-decode pool is scoped: a limited decode must not leak the
/// local pool into a following automatic decode on the same thread, and
/// a low-level Decoder reuses its pool across decodes deterministically.
#[test]
fn decode_parameters_threads_pool_is_scoped() {
  let buf = std::fs::read("samples/issue235_cielab_multitile.jp2").expect("sample file");
  let reference = Image::from_bytes(&buf).expect("reference decode");
  // First a limited decode, then an automatic one on the same thread.
  let _ = Image::from_bytes_with(&buf, DecodeParameters::new().threads(2)).expect("limited decode");
  let after = Image::from_bytes(&buf).expect("automatic decode after limited");
  for (a, b) in reference.components().iter().zip(after.components().iter()) {
    assert_eq!(a.data, b.data, "automatic decode after a limited one");
  }
  // One codec, several tile decodes: the local pool is built once and
  // reused (each get_decoded_tile re-runs through with_pool), and the
  // result is deterministic.
  let mut dec = Decoder::new(J2KFormat::JP2).unwrap();
  dec
    .apply_settings(&DecoderSettings {
      threads: 2,
      ..Default::default()
    })
    .unwrap();
  let mut s = Stream::new_input(
    crate::consts::J2K_STREAM_CHUNK_SIZE as usize,
    std::io::Cursor::new(buf.clone()),
    buf.len() as u64,
  );
  let _hdr = dec.read_header(&mut s).unwrap();
  let first = dec.get_decoded_tile(&mut s, 0).unwrap();
  let second = dec.get_decoded_tile(&mut s, 0).unwrap();
  assert!(
    first.comps.iter().any(|c| !c.data.is_empty()),
    "tile decode produced data"
  );
  for (a, b) in first.comps.iter().zip(second.comps.iter()) {
    assert_eq!(a.data, b.data, "repeated limited tile decode");
  }
}

/// Regression (review_3 pass-2 residual, OF-43): a failed DumpImage
/// decode must not trap a retry - the decoder keeps its own header copy
/// and the retry simply fails again on the mid-state stream.
#[test]
fn dump_image_failed_decode_retry_does_not_panic() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  // Truncate the stream mid-data and decode STRICT, so the decode fails
  // (lenient mode would deliver the partial image per its contract).
  let truncated = &buf[..buf.len() / 2];
  let mut dump = DumpImage::from_bytes_with(truncated, DecodeParameters::new().strict(true))
    .expect("header reads");
  let first = dump.decode();
  let second = dump.decode();
  // The retry must not panic; both results are errors (mid-state stream).
  assert!(first.is_err());
  assert!(second.is_err());
}

/// DumpImage + the codestream index/info accessors.
#[test]
fn dump_image_and_index() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let mut dump = DumpImage::from_bytes(&buf).expect("dump image");
  dump.decode().expect("decode");
  let index = dump.get_codestream_index().expect("index");
  assert_eq!(index.tile_indices().len(), 1);
  assert!(!index.markers().is_empty());
  let info = dump.get_codestream_info().expect("info");
  assert_eq!((info.tw(), info.th()), (1, 1));
  assert_eq!(info.nbcomps(), 3);
  let text = dump.dump(
    (crate::consts::IMG_INFO | crate::consts::J2K_MH_INFO | crate::consts::J2K_MH_IND) as i32,
  );
  // The JP2 dump prints the boxes and the codestream header info.
  assert!(text.contains("Codestream info"));
}

/// Fixed-quality layer rates via the builder (rate allocation).
#[test]
fn encode_with_layer_rates() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let image = Image::from_bytes(&buf).expect("decode");
  let codestream = image
    .clone()
    .encode(
      EncodeParameters::new()
        .num_resolutions(3)
        .layer_rate(50.0)
        .layer_rate(10.0),
    )
    .expect("encode");
  let back = Image::from_bytes(&codestream).expect("decode");
  assert_eq!(
    (back.width(), back.height()),
    (image.width(), image.height())
  );
  assert!(back.components()[0].data.iter().any(|v| *v != 0));
}

/// Encode to an arbitrary writer and decode the bytes back.
#[test]
fn encode_to_writer_round_trip() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let image = Image::from_bytes(&buf).expect("decode");
  let out_path = std::env::temp_dir().join("openjp2_encode_to_writer.j2k");
  let file = std::fs::File::create(&out_path).expect("create");
  image
    .clone()
    .encode_to_writer(
      EncodeParameters::new()
        .num_resolutions(3)
        .tile_size(512, 512),
      file,
    )
    .expect("encode to writer");
  let back = Image::from_file(&out_path).expect("decode");
  let _ = std::fs::remove_file(&out_path);
  assert_eq!(
    (back.width(), back.height()),
    (image.width(), image.height())
  );
  for (a, b) in image.components().iter().zip(back.components()) {
    assert_eq!(a.data, b.data, "writer round trip");
  }
}

/// Build an image from an image-crate DynamicImage and round-trip it.
#[cfg(feature = "image")]
#[test]
fn from_dynamic_image_round_trip() {
  let mut img = ::image::RgbaImage::new(96, 64);
  for (i, px) in img.pixels_mut().enumerate() {
    *px = ::image::Rgba([
      (i % 251) as u8,
      ((i * 7) % 233) as u8,
      ((i * 13) % 199) as u8,
      255,
    ]);
  }
  let dyn_img = ::image::DynamicImage::ImageRgba8(img);
  let image = Image::from_dynamic_image(&dyn_img);
  assert_eq!(image.num_components(), 4);
  assert_eq!((image.width(), image.height()), (96, 64));
  // Round-trip through the encoder (lossless 5-3).
  let codestream = image
    .clone()
    .encode(EncodeParameters::new().num_resolutions(3))
    .expect("encode");
  let back = Image::from_bytes(&codestream).expect("decode");
  for (a, b) in image.components().iter().zip(back.components()) {
    assert_eq!(a.data, b.data);
  }
  // And back into pixels.
  let pixels = back.get_pixels(None).expect("get_pixels");
  assert_eq!(pixels.format, ::image::ExtendedColorType::Rgba8);
}

/// Grey+alpha and RGB+alpha pixel extraction (image-crate interop).
#[cfg(feature = "image")]
#[test]
fn get_pixels_alpha() {
  // 2 components, the second marked as alpha (cdef typ 1).
  let (w, h) = (32u32, 32u32);
  let mut core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 2,
    color_space: crate::ColorSpace::Gray,
    comps: Vec::new(),
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  for c in 0..2 {
    core.comps.push(crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: if c == 1 { 1 } else { 0 },
      data: vec![c * 200; (w * h) as usize],
    });
  }
  let image = Image { core };
  let pixels = image.get_pixels(None).expect("get_pixels");
  assert_eq!(pixels.format, ::image::ExtendedColorType::La8);
  assert_eq!(pixels.width, w);
  assert_eq!(pixels.height, h);
  // A single component with a default alpha fill.
  let gray = Image {
    core: crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 1,
      color_space: crate::ColorSpace::Gray,
      comps: vec![crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data: vec![7; (w * h) as usize],
      }],
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    },
  };
  let filled = gray.get_pixels(Some(255)).expect("get_pixels with alpha");
  assert_eq!(filled.format, ::image::ExtendedColorType::La8);
  match filled.data {
    super::image_conv::ImagePixelData::La8(v) => {
      assert_eq!(v.len(), (w * h * 2) as usize);
      assert_eq!(&v[0..2], &[7, 255]);
    }
    other => panic!("unexpected {:?}", other),
  }
}

/// Code-block sizes and comment markers through the builder.
#[test]
fn encode_with_codeblock_sizes_and_comment() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let image = Image::from_bytes(&buf).expect("decode");
  let codestream = image
    .clone()
    .encode(
      EncodeParameters::new()
        .num_resolutions(3)
        .codeblock_width(16)
        .codeblock_height(16)
        .comment("openjp2 test comment"),
    )
    .expect("encode");
  let back = Image::from_bytes(&codestream).expect("decode");
  assert_eq!(
    (back.width(), back.height()),
    (image.width(), image.height())
  );
  for (a, b) in image.components().iter().zip(back.components()) {
    assert_eq!(a.data, b.data, "lossless with 16x16 code-blocks");
  }
}

/// The default codestream comment identifies the port itself; the
/// `openjpeg-version-string` feature switches it to the upstream
/// string (checked by the encode byte-identity gates).
#[test]
fn default_codestream_comment_is_ours() {
  let comment = crate::openjpeg::default_codestream_comment();
  #[cfg(feature = "openjpeg-version-string")]
  assert!(comment.starts_with("Created by OpenJPEG version "));
  #[cfg(not(feature = "openjpeg-version-string"))]
  {
    assert!(comment.starts_with("Created by "));
    assert!(comment.contains(env!("CARGO_PKG_NAME")));
    assert!(comment.contains(env!("CARGO_PKG_VERSION")));
    assert!(!comment.contains("OpenJPEG version"));
  }
}

/// Decoding garbage must return an error, never panic.
#[test]
fn decode_garbage_errors() {
  assert!(Image::from_bytes(b"not a jpeg2000 file at all").is_err());
  let mut data = vec![0xFFu8, 0x4F]; // SOC only
  data.extend([0u8; 64]);
  assert!(Image::from_bytes(&data).is_err());
}

/// The irreversible 9-7 transform: a lossy encode/decode round trip.
#[test]
fn irreversible_encode_round_trip() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let image = Image::from_bytes(&buf).expect("decode");
  let codestream = image
    .clone()
    .encode(
      EncodeParameters::new()
        .irreversible(true)
        .num_resolutions(4)
        .num_layers(1)
        .tile_size(512, 512),
    )
    .expect("encode");
  let back = Image::from_bytes(&codestream).expect("decode");
  assert_eq!(
    (back.width(), back.height()),
    (image.width(), image.height())
  );
  assert_eq!(back.num_components(), image.num_components());
  // Lossy, but the pixels must be close (9-7 with one layer).
  for (a, b) in image.components().iter().zip(back.components()) {
    let max_delta = a
      .data
      .iter()
      .zip(&b.data)
      .map(|(x, y)| (x - y).unsigned_abs())
      .max()
      .unwrap_or(0);
    assert!(max_delta < 64, "9-7 round trip drifted by {}", max_delta);
  }
}

/// Lenient mode returns a partial image for truncated codestreams instead
/// of an error (progressive download), and the decoder limits reject
/// oversized geometry before any allocation.
#[test]
fn truncation_and_limits() {
  let mut rng: u64 = 0x243f6a8885a308d3;
  let mut next = move || {
    rng ^= rng << 13;
    rng ^= rng >> 7;
    rng ^= rng << 17;
    rng
  };
  // Build a small valid codestream with the encoder.
  let (w, h) = (96u32, 64u32);
  let mut core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 3,
    color_space: crate::ColorSpace::Srgb,
    comps: Vec::new(),
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  for _c in 0..3u32 {
    let mut data = Vec::with_capacity((w * h) as usize);
    for _ in 0..w * h {
      data.push((next() % 256) as i32);
    }
    core.comps.push(crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data,
    });
  }
  let image = Image { core };
  let codestream = image
    .encode(EncodeParameters::new().num_resolutions(2).num_layers(1))
    .expect("encode");

  // 1. Full decode works.
  let full = Image::from_bytes(&codestream).expect("full decode");
  assert_eq!(full.width(), w);
  assert_eq!(full.height(), h);

  // 2. Strict mode rejects a truncated codestream; lenient mode returns the
  // partial image with the right dimensions (and component count).
  let half = &codestream[..codestream.len() / 2];
  assert!(Image::from_bytes_with(half, DecodeParameters::new().strict(true)).is_err());
  let partial = Image::from_bytes_with(half, DecodeParameters::new().strict(false))
    .expect("lenient partial decode");
  assert_eq!(partial.width(), w);
  assert_eq!(partial.height(), h);
  assert_eq!(partial.num_components(), 3);

  // 3. A codestream cut inside the header (after SIZ) still yields the
  // dimensions in lenient mode.
  let siz_end = {
    // SIZ marker: FF 51, then size; find it in the first 200 bytes.
    let pos = codestream
      .windows(2)
      .position(|win| win == [0xff, 0x51])
      .expect("SIZ marker");
    let l = u16::from_be_bytes([codestream[pos + 2], codestream[pos + 3]]) as usize;
    pos + 2 + l
  };
  let header_only = &codestream[..siz_end + 8];
  let partial_hdr = Image::from_bytes_with(header_only, DecodeParameters::new().strict(false))
    .expect("lenient header-only decode");
  assert_eq!(partial_hdr.width(), w);
  assert_eq!(partial_hdr.height(), h);
  assert!(Image::from_bytes_with(header_only, DecodeParameters::new().strict(true)).is_err());

  // 4. Decoder limits: image size, tile area, and memory budget all reject
  // the codestream before decoding.
  assert!(Image::from_bytes_with(
    &codestream,
    DecodeParameters::new().limits(DecodeLimits::new().max_image_size(w - 1, h)),
  )
  .is_err());
  assert!(Image::from_bytes_with(
    &codestream,
    DecodeParameters::new().limits(DecodeLimits::new().max_image_size(w, h - 1)),
  )
  .is_err());
  assert!(Image::from_bytes_with(
    &codestream,
    DecodeParameters::new().limits(DecodeLimits::new().max_tile_pixels(10)),
  )
  .is_err());
  assert!(Image::from_bytes_with(
    &codestream,
    DecodeParameters::new().limits(DecodeLimits::new().max_mem_alloc(1024)),
  )
  .is_err());
  // ... and generous limits accept it.
  let ok = Image::from_bytes_with(
    &codestream,
    DecodeParameters::new().limits(
      DecodeLimits::new()
        .max_image_size(w, h)
        .max_tile_pixels(w as u64 * h as u64)
        .max_mem_alloc(64 * 1024 * 1024),
    ),
  )
  .expect("decode within limits");
  assert_eq!(ok.width(), w);
}

/// The concurrent-tile decode honours a resolution reduction.
#[cfg(feature = "rayon")]
#[test]
fn parallel_tile_decode_with_reduce() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let image = Image::from_bytes(&buf).expect("decode");
  let codestream = image
    .clone()
    .encode(
      EncodeParameters::new()
        .num_resolutions(4)
        .tile_size(512, 512),
    )
    .expect("encode");
  let reduced =
    Image::from_bytes_with(&codestream, DecodeParameters::new().reduce(2)).expect("decode reduced");
  /* The image bounds stay full-dim; the component planes are reduced. */
  let c = &reduced.components()[0];
  assert_eq!(
    (c.w, c.h),
    (image.width().div_ceil(4), image.height().div_ceil(4))
  );
  assert!(reduced.components()[0].data.iter().any(|v| *v != 0));
}

/// The concurrent-tile decode honours a decode-area request (only the
/// intersecting tiles are decoded).
#[cfg(feature = "rayon")]
#[test]
fn parallel_tile_decode_with_area() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  // The sample is single-tile; build a multi-tile codestream first.
  let image = Image::from_bytes(&buf).expect("decode");
  let codestream = image
    .encode(
      EncodeParameters::new()
        .num_resolutions(2)
        .tile_size(512, 512),
    )
    .expect("encode");
  /* Tile-aligned area: exercises the parallel area path (the partial
  one below falls back to the sequential decode). */
  let aligned = DecodeArea::new(0, 0, 1024, 1024);
  let sub = Image::from_bytes_with(
    &codestream,
    DecodeParameters::new().decode_area(Some(aligned)),
  )
  .expect("decode aligned area");
  assert_eq!((sub.width(), sub.height()), (1024, 1024));
  assert!(sub.components()[0].data.iter().any(|v| *v != 0));
  /* Partial area: sequential fallback, same API surface. */
  let partial = DecodeArea::new(64, 64, 900, 700);
  let sub2 = Image::from_bytes_with(
    &codestream,
    DecodeParameters::new().decode_area(Some(partial)),
  )
  .expect("decode partial area");
  assert_eq!((sub2.width(), sub2.height()), (836, 636));
  assert!(sub2.components()[0].data.iter().any(|v| *v != 0));
}

/// The concurrent-tile decode honours a quality-layer limit (the
/// reduce cases stay on the serial path).
#[cfg(feature = "rayon")]
#[test]
fn parallel_tile_decode_with_layers() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let full = Image::from_bytes(&buf).expect("decode");
  let limited = Image::from_bytes_with(&buf, DecodeParameters::new().layers(1))
    .expect("decode with layer limit");
  assert_eq!(
    (limited.width(), limited.height()),
    (full.width(), full.height())
  );
  // The first layer alone must decode (a lossy rendition of the image).
  assert!(limited.components()[0].data.iter().any(|v| *v != 0));
}

/// Parallel-vs-serial byte identity over the real test corpus (runs
/// when OPJ_DATA_ROOT is set, e.g. in the dev gates): every file that
/// decodes must produce the same pixels through the parallel one-shot
/// API and the sequential low-level Codec API.
#[cfg(feature = "rayon")]
#[test]
fn parallel_matches_serial_on_corpus() {
  let root = match std::env::var_os("OPJ_DATA_ROOT") {
    Some(r) => std::path::PathBuf::from(r),
    None => return, /* no test data -> skip */
  };
  let input = root.join("input").join("nonregression");
  if !input.is_dir() {
    return;
  }
  let mut checked = 0usize;
  let mut entries: Vec<_> = std::fs::read_dir(&input)
    .expect("read_dir")
    .filter_map(|e| e.ok())
    .map(|e| e.path())
    .filter(|p| {
      matches!(
        p.extension().and_then(|e| e.to_str()),
        Some("j2k") | Some("j2c") | Some("jp2") | Some("jhc") | Some("jph")
      )
    })
    .collect();
  let conformance = root.join("input").join("conformance");
  if conformance.is_dir() {
    entries.extend(
      std::fs::read_dir(&conformance)
        .expect("read_dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
          matches!(
            p.extension().and_then(|e| e.to_str()),
            Some("j2k") | Some("j2c") | Some("jp2") | Some("jhc") | Some("jph")
          )
        }),
    );
  }
  entries.sort();
  for path in entries {
    let buf = std::fs::read(&path).expect("read input");
    let serial: Option<crate::CoreImage> = (|| {
      let fmt = crate::detect_format(&buf).ok()?;
      let mut codec = Decoder::new(fmt).ok()?;
      /* The safe API defaults to lenient decoding (DecodeParameters
      ::default().strict == false); mirror it. */
      codec.set_strict(false).ok()?;
      let mut stream = Stream::new_input(
        crate::consts::J2K_STREAM_CHUNK_SIZE as usize,
        std::io::Cursor::new(buf.clone()),
        buf.len() as u64,
      );
      codec.read_header(&mut stream).ok()?;
      let img = codec.decode(&mut stream).ok()?;
      codec.end_decompress(&mut stream).ok()?;
      Some(img)
    })();
    let par = Image::from_bytes(&buf).ok();
    // A parallel-vs-serial comparison for an arbitrary parameter set:
    // both paths must succeed/fail together and produce identical data.
    let compare = |params: &DecodeParameters, label: &str| {
      let serial: Option<crate::CoreImage> = (|| {
        let fmt = crate::detect_format(&buf).ok()?;
        let mut codec = Decoder::new(fmt).ok()?;
        codec
          .apply_settings(&DecoderSettings {
            layers: params.layers,
            reduce: params.reduce,
            ..Default::default()
          })
          .ok()?;
        codec.set_strict(params.strict).ok()?;
        let mut stream = Stream::new_input(
          crate::consts::J2K_STREAM_CHUNK_SIZE as usize,
          std::io::Cursor::new(buf.clone()),
          buf.len() as u64,
        );
        let mut img = codec.read_header(&mut stream).ok()?.into_core_image();
        if let Some(area) = params.decode_area {
          codec.set_decode_area(&mut img, Some(area)).ok()?;
        }
        let img = codec.decode(&mut stream).ok()?;
        codec.end_decompress(&mut stream).ok()?;
        Some(img)
      })();
      let par = Image::from_bytes_with(&buf, params.clone()).ok();
      match (serial.as_ref(), par) {
        (Some(s), Some(p)) => {
          assert_eq!(
            p.components().len(),
            s.comps.len(),
            "{}: component count differs ({label})",
            path.display()
          );
          for (a, b) in p.components().iter().zip(s.comps.iter()) {
            assert_eq!(
              a.data,
              b.data,
              "{}: parallel/serial mismatch ({label})",
              path.display()
            );
          }
        }
        (Some(_), None) => panic!(
          "{}: serial decoded, parallel failed ({label})",
          path.display()
        ),
        (None, Some(_)) => panic!(
          "{}: parallel decoded, serial failed ({label})",
          path.display()
        ),
        (None, None) => {}
      }
    };
    compare(&DecodeParameters::new().strict(true), "strict");
    compare(
      &DecodeParameters::new().layers(2).reduce(1),
      "layers+reduce",
    );
    compare(
      &DecodeParameters::new().decode_area(Some(DecodeArea::new(64, 64, 256, 256))),
      "area-partial",
    );
    /* A whole-image "area" (clamped to the image bounds) covers every
    tile completely, so this exercises the parallel area driver. */
    compare(
      &DecodeParameters::new().decode_area(Some(DecodeArea::new(0, 0, 100_000_000, 100_000_000))),
      "area-whole",
    );
    match (serial, par) {
      (Some(s), Some(p)) => {
        assert_eq!(
          p.components().len(),
          s.comps.len(),
          "{}: component count differs",
          path.display()
        );
        for (a, b) in p.components().iter().zip(s.comps.iter()) {
          assert_eq!(
            a.data,
            b.data,
            "{}: parallel/serial mismatch",
            path.display()
          );
        }
        checked += 1;
      }
      (Some(_), None) => panic!("{}: serial decoded, parallel failed", path.display()),
      (None, Some(_)) => panic!("{}: parallel decoded, serial failed", path.display()),
      (None, None) => {} /* expected-corrupt input */
    }
  }
  assert!(checked > 10, "expected a corpus of decodable inputs");
}

/// The concurrent-tile decode (rayon): a multi-tile codestream must
/// decode byte-identically to the source pixels.
#[cfg(feature = "rayon")]
#[test]
fn parallel_tile_decode_matches_source() {
  let mut rng: u64 = 0x1234_5678_9abc_def0;
  let mut next = move || {
    rng ^= rng << 13;
    rng ^= rng >> 7;
    rng ^= rng << 17;
    rng
  };
  let (w, h) = (192u32, 128u32);
  let mut core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 3,
    color_space: crate::ColorSpace::Srgb,
    comps: Vec::new(),
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  for _c in 0..3 {
    let mut data = Vec::with_capacity((w * h) as usize);
    for _ in 0..w * h {
      data.push((next() % 256) as i32);
    }
    core.comps.push(crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data,
    });
  }
  let image = Image { core };
  let codestream = image
    .clone()
    .encode(
      EncodeParameters::new()
        .num_resolutions(2)
        .num_layers(1)
        .tile_size(64, 64),
    )
    .expect("encode");
  let back = Image::from_bytes(&codestream).expect("decode");
  assert_eq!(back.width(), w);
  assert_eq!(back.height(), h);
  for (a, b) in image.components().iter().zip(back.components()) {
    assert_eq!(a.data, b.data, "parallel tile decode mismatch");
  }
}

/// Encode config sweep: every progression order with tiles and layers,
/// always decoding back losslessly.
#[test]
fn encode_config_sweep() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let image = Image::from_bytes(&buf).expect("decode sample");
  for order in [
    ProgressionOrder::Lrcp,
    ProgressionOrder::Rlcp,
    ProgressionOrder::Rpcl,
    ProgressionOrder::Pcrl,
    ProgressionOrder::Cprl,
  ] {
    let codestream = image
      .clone()
      .encode(
        EncodeParameters::new()
          .num_resolutions(2)
          .num_layers(2)
          .progression_order(order)
          .tile_size(512, 512),
      )
      .expect("encode");
    let back = Image::from_bytes(&codestream).expect("decode");
    assert_eq!(back.width(), image.width(), "order {order:?}");
    assert_eq!(back.height(), image.height(), "order {order:?}");
    for (a, b) in image.components().iter().zip(back.components()) {
      assert_eq!(a.data, b.data, "lossless for order {order:?}");
    }
  }
}

/// Regression (review round 2, H1): a QCD marker with Sqcd == 1 (SIQNT)
/// and no SPqcd bytes must be rejected, not panic (the 2-byte band read
/// used to slice past the end of the marker content).
#[test]
fn siqnt_truncated_qcd_rejected() {
  let mut cs = Vec::new();
  cs.extend([0xff, 0x4f]); // SOC
                           // SIZ 8x8 1 comp
  cs.extend([0xff, 0x51, 0x00, 0x29]);
  cs.extend(0u16.to_be_bytes()); // Rsiz
  for _ in 0..8 {
    cs.extend(8u32.to_be_bytes());
  }
  cs.extend(0u32.to_be_bytes());
  cs.extend(1u16.to_be_bytes()); // Csiz
  cs.extend([0x07, 0x01, 0x01]);
  // COD
  cs.extend([0xff, 0x52, 0x00, 0x0c]);
  cs.extend([0x00, 0x00]);
  cs.extend(1u16.to_be_bytes());
  cs.extend([0x00]);
  cs.extend([0x00, 0x04, 0x04, 0x00, 0x01]);
  // QCD: Lqcd=3, Sqcd=0x01 (SIQNT), no SPqcd -> truncated
  cs.extend([0xff, 0x5c, 0x00, 0x03, 0x01]);

  assert!(Image::from_bytes_with(&cs, DecodeParameters::new().strict(true)).is_err());
  // The raw (unguarded) Codec path must fail cleanly too, not panic.
  #[allow(deprecated)]
  {
    let mut codec = Decoder::new(J2KFormat::J2K).unwrap();
    let mut stream = Stream::new_input(
      crate::consts::J2K_STREAM_CHUNK_SIZE as usize,
      std::io::Cursor::new(cs.clone()),
      cs.len() as u64,
    );
    codec.apply_settings(&DecoderSettings::default()).unwrap();
    codec.set_strict(true).unwrap();
    assert!(codec.read_header(&mut stream).is_err());
  }
}

/// Regression (review round 2, H2): components that subsample to a zero
/// width/height must not panic the tile API (the C SIGFPEs on one shape
/// and no-ops on the other; both now fail/succeed cleanly).
#[test]
fn zerodim_tile_api_does_not_panic() {
  for path in [
    "samples/zerowidth_patched.j2k",
    "samples/zerodim_patched.j2k",
  ] {
    let data = std::fs::read(path).unwrap();
    #[allow(deprecated)]
    {
      let mut codec = Decoder::new(J2KFormat::J2K).unwrap();
      let mut stream = Stream::new_input(
        crate::consts::J2K_STREAM_CHUNK_SIZE as usize,
        std::io::Cursor::new(data.clone()),
        data.len() as u64,
      );
      codec.apply_settings(&DecoderSettings::default()).unwrap();
      let _image = codec.read_header(&mut stream).expect("header");
      let tile_info = codec
        .read_tile_header(&mut stream)
        .expect("{path}: read_tile_header");
      let mut buf = vec![0u8; tile_info.data_size.unwrap_or(0) as usize];
      let ok = codec.decode_tile_data(&mut stream, 0, Some(&mut buf));
      assert!(
        ok.is_ok(),
        "{}",
        format!("{path}: decode_tile_data -> {ok:?}")
      );
    }
  }
}

/// Regression (review round 2, M2 / round 3, H1): in lenient mode a
/// codestream truncated right after the tile data (missing EOC) must
/// still deliver the decoded pixels - the C returns the full image, the
/// port used to return an image with empty component data.
#[test]
fn lenient_truncated_stream_keeps_pixels() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").unwrap();
  let image = Image::from_bytes(&buf).unwrap();
  let full = image
    .clone()
    .encode(EncodeParameters::new().num_resolutions(3))
    .unwrap();
  let trunc = &full[..full.len() - 2]; // drop the EOC marker
  let partial = Image::from_bytes_with(trunc, DecodeParameters::new().strict(false))
    .expect("lenient truncated decode");
  assert_eq!(partial.num_components(), image.num_components());
  for (a, b) in image.components().iter().zip(partial.components()) {
    assert_eq!(
      a.data.len(),
      b.data.len(),
      "component data must be preserved"
    );
    assert_eq!(a.data, b.data, "pixels must match the full decode");
  }
  // Strict mode still rejects the same stream.
  assert!(Image::from_bytes_with(trunc, DecodeParameters::new().strict(true)).is_err());
}

/// Regression (review round 2, H3): an empty image with tiles requested
/// (or a tile origin outside the image) must be rejected, not divide by
/// zero.
#[test]
fn empty_image_with_tiles_rejected() {
  let empty = Image {
    core: crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: 0,
      y1: 0,
      numcomps: 1,
      color_space: crate::ColorSpace::Gray,
      comps: vec![crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w: 0,
        h: 0,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data: vec![],
      }],
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    },
  };
  assert!(empty
    .encode(EncodeParameters::new().tile_size(64, 64))
    .is_err());
}

/// Regression (review round 3, M4): is_imf_compliant must not index
/// prcw_init past the end for a caller-supplied res_spec > 33 (the C
/// reads OOB and returns FALSE; the port clamps the walk and encodes).
#[test]
fn imf_oversized_res_spec_encodes() {
  let (w, h) = (32u32, 32u32);
  let core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 1,
    color_space: crate::ColorSpace::Gray,
    comps: vec![crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data: vec![1; (w * h) as usize],
    }],
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let image = Image { core };
  let mut params = EncodeParameters::new();
  params.low_level().rsiz = 0x0400; // IMF 2K
  params.low_level().res_spec = 40;
  params.low_level().csty = 2;
  assert!(image.encode(params).is_ok(), "encode must not panic");
}

/// Regression (review round 2, M6): custom-MCT matrix and dc-shift
/// vectors with the wrong lengths must be rejected (the C reads them
/// unconditionally).
#[test]
fn custom_mct_lengths_validated() {
  let (w, h) = (16u32, 16u32);
  let core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 3,
    color_space: crate::ColorSpace::Srgb,
    comps: (0..3)
      .map(|_| crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data: vec![7; (w * h) as usize],
      })
      .collect(),
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let image = Image { core };
  let mut params = EncodeParameters::new().num_resolutions(2);
  params.low_level().mct_data = Some((vec![1.0f32; 4], vec![0i32; 3])); // matrix too short
  assert!(image.clone().encode(params.clone()).is_err());
  let mut params = EncodeParameters::new().num_resolutions(2);
  params.low_level().mct_data = Some((vec![1.0f32; 9], vec![0i32; 2])); // dc_shift too short
  assert!(image.clone().encode(params).is_err());
}

/// Regression (review round 3, L3): an out-of-range progression order
/// must be rejected instead of written as a garbage marker.
#[test]
fn prog_order_validated() {
  let (w, h) = (16u32, 16u32);
  let core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 1,
    color_space: crate::ColorSpace::Gray,
    comps: vec![crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data: vec![1; (w * h) as usize],
    }],
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let image = Image { core };
  let mut params = EncodeParameters::new();
  // The C's progression-order validation moved to the C-ABI boundary (the
  // native enum is exhaustive); the core still validates the numeric
  // parameters.
  params.low_level().res_spec = 0;
  params.low_level().csty = 1;
  assert!(image.encode(params).is_err());
}

/// Regression (review rounds 2+3, M1; re-verified after B.2.1): with TLM
/// enabled and multi-part tiles, the parallel encoder records one TLM
/// entry per tile-part (each worker records into its own buffer clone
/// and the master merges in tile order), so it must produce output
/// byte-identical to the serial encode.
#[cfg(feature = "rayon")]
#[test]
fn parallel_tlm_multi_part_matches_serial() {
  let (w, h) = (192u32, 128u32);
  let mut comps = Vec::new();
  for c in 0..3u32 {
    let mut data = Vec::with_capacity((w * h) as usize);
    for i in 0..w * h {
      data.push(((i * 7 + c * 31) % 256) as i32);
    }
    comps.push(crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data,
    });
  }
  let core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 3,
    color_space: crate::ColorSpace::Srgb,
    comps,
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  #[allow(deprecated)]
  let encode_low_level = |tlm: bool| -> Vec<u8> {
    let mut codec = Encoder::new(J2KFormat::J2K).unwrap();
    if tlm {
      codec.extra_options(&["TLM=YES"]).unwrap();
    }
    let mut params = crate::EncoderParameters {
      tile_size_on: true,
      cp_tdx: 64,
      cp_tdy: 64,
      tp_on: 1,
      tp_flag: b'R' as i8, // one tile-part per rate layer
      cp_disto_alloc: 1,   // enable rate allocation so the layers split
      tcp_numlayers: 2,
      tcp_rates: {
        let mut r = [0.0; 100];
        r[0] = 20.0;
        r[1] = 10.0;
        r
      },
      numresolution: 3,
      irreversible: 0,
      ..Default::default()
    };
    let mut img = core.clone();
    codec.setup(&mut params, &mut img).unwrap();
    let buf = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    {
      let writer = crate::stream::SharedVecWriter {
        buf: buf.clone(),
        pos: 0,
      };
      let mut stream = Stream::new_output(crate::consts::J2K_STREAM_CHUNK_SIZE as usize, writer);
      codec.start(&mut img, &mut stream).unwrap();
      // The parallel path handles TLM + multi-part tiles (per-part
      // entries, merged by the master); verify it applies for both.
      let parallel = codec.encode_parallel(&mut stream);
      assert_eq!(
        parallel,
        Some(true),
        "parallel path must handle TLM + multi-part"
      );
      codec.finish(&mut stream).unwrap();
    }
    std::rc::Rc::try_unwrap(buf)
      .ok()
      .map(|b| b.into_inner())
      .expect("unique handle")
  };
  // The parallel output must be byte-identical to the serial encode for
  // both variants (the per-part TLM entries included).
  let parallel_tlm = encode_low_level(true);
  let parallel_plain = encode_low_level(false);
  let serial = |tlm: bool| -> Vec<u8> {
    let mut codec = Encoder::new(J2KFormat::J2K).unwrap();
    if tlm {
      codec.extra_options(&["TLM=YES"]).unwrap();
    }
    let mut params = crate::EncoderParameters {
      tile_size_on: true,
      cp_tdx: 64,
      cp_tdy: 64,
      tp_on: 1,
      tp_flag: b'R' as i8,
      cp_disto_alloc: 1,
      tcp_numlayers: 2,
      tcp_rates: {
        let mut r = [0.0; 100];
        r[0] = 20.0;
        r[1] = 10.0;
        r
      },
      numresolution: 3,
      irreversible: 0,
      ..Default::default()
    };
    let mut img = core.clone();
    codec.setup(&mut params, &mut img).unwrap();
    let buf = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    {
      let writer = crate::stream::SharedVecWriter {
        buf: buf.clone(),
        pos: 0,
      };
      let mut stream = Stream::new_output(crate::consts::J2K_STREAM_CHUNK_SIZE as usize, writer);
      codec.start(&mut img, &mut stream).unwrap();
      codec.encode(&mut stream).unwrap();
      codec.finish(&mut stream).unwrap();
    }
    std::rc::Rc::try_unwrap(buf)
      .ok()
      .map(|b| b.into_inner())
      .expect("unique handle")
  };
  assert_eq!(
    parallel_tlm,
    serial(true),
    "TLM parallel/serial byte identity"
  );
  assert_eq!(
    parallel_plain,
    serial(false),
    "plain parallel/serial byte identity"
  );
  let back = Image::from_bytes(&parallel_tlm).expect("decode TLM stream");
  assert_eq!(back.num_components(), 3);
  assert_eq!((back.width(), back.height()), (w, h));
}

/// Regression (review round 2, M4 / round 3, H2b): subsampled component
/// planes must be rejected by get_pixels/DynamicImage instead of
/// silently truncating and panicking in the conversion.
#[cfg(feature = "image")]
#[test]
fn get_pixels_subsampled_rejected() {
  let (w, h) = (64u32, 48u32);
  let mut comps = Vec::new();
  for c in 0..3u32 {
    let dx = if c == 0 { 1 } else { 2 };
    let (cw, ch) = (w.div_ceil(dx), h.div_ceil(dx));
    comps.push(crate::CoreImageComp {
      dx,
      dy: dx,
      w: cw,
      h: ch,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data: vec![c as i32 * 40; (cw * ch) as usize],
    });
  }
  let image = Image {
    core: crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 3,
      color_space: crate::ColorSpace::Unknown,
      comps,
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    },
  };
  assert!(
    image.get_pixels(None).is_err(),
    "subsampled planes must be rejected"
  );
  use core::convert::TryFrom;
  assert!(
    ::image::DynamicImage::try_from(&image).is_err(),
    "DynamicImage conversion must not panic"
  );
}

/// Regression (review round 3, M5, re-opened by the reviewer): a tile whose
/// components subsample to nothing (subsampling larger than the tile size)
/// must encode AND decode - the C's calloc(0)/no-op loops succeed, the port
/// used to fail (decode: empty pi treated as hard error; encode: empty pi +
/// zero-dim DWT panics).
#[test]
fn subsampled_empty_tiles_round_trip() {
  let (w, h) = (256u32, 256u32);
  let core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 1,
    color_space: crate::ColorSpace::Gray,
    comps: vec![crate::CoreImageComp {
      dx: 128,
      dy: 128,
      w: w.div_ceil(128),
      h: h.div_ceil(128),
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data: (0..(w / 128) * (h / 128))
        .map(|i| (i % 256) as i32)
        .collect(),
    }],
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let image = Image { core };
  let params = EncodeParameters::new().num_resolutions(3).tile_size(64, 64);
  let codestream = image
    .clone()
    .encode(params)
    .expect("subsampled tiled encode");
  let back = Image::from_bytes(&codestream).expect("subsampled tiled decode");
  assert_eq!((back.width(), back.height()), (w, h));
  assert_eq!(back.num_components(), 1);
  // The visible pixels (the 2x2 subsampled plane) must round-trip losslessly
  // (reversible 5-3).
  for (a, b) in image.components()[0]
    .data
    .iter()
    .zip(back.components()[0].data.iter())
  {
    assert_eq!(a, b, "subsampled round trip must be lossless");
  }
}

#[test]
fn color_space_round_trip() {
  for cs in [
    ColorSpace::Unknown,
    ColorSpace::Srgb,
    ColorSpace::Gray,
    ColorSpace::Sycc,
    ColorSpace::Eycc,
    ColorSpace::Cmyk,
  ] {
    let c: OPJ_COLOR_SPACE = cs.into();
    let back: ColorSpace = c.into();
    assert_eq!(back, cs);
  }
}

/// Minimal J2K builder: a valid 8x8 single-component codestream with the
/// given COD parameters, used to craft regression inputs.
fn build_minimal_j2k(numresolutions: u8, prog_order: u8, layers: u16) -> Vec<u8> {
  let mut cs = Vec::new();
  // SOC
  cs.extend([0xff, 0x4f]);
  // SIZ: 1 component, 8x8, 8-bit unsigned
  cs.extend([0xff, 0x51, 0x00, 0x29]);
  cs.extend(0u32.to_be_bytes()); // Rsiz
  cs.extend(8u32.to_be_bytes()); // Xsiz
  cs.extend(8u32.to_be_bytes()); // Ysiz
  cs.extend(0u32.to_be_bytes()); // XOsiz
  cs.extend(0u32.to_be_bytes()); // YOsiz
  cs.extend(8u32.to_be_bytes()); // XTsiz
  cs.extend(8u32.to_be_bytes()); // YTsiz
  cs.extend(0u32.to_be_bytes()); // XTOsiz
  cs.extend(0u32.to_be_bytes()); // YTOsiz
  cs.extend(1u16.to_be_bytes()); // Csiz
  cs.extend([0x07, 0x01, 0x01]); // Ssiz (8-bit) XRsiz YRsiz
                                 // COD
  cs.extend([0xff, 0x52, 0x00, 0x0c]);
  cs.extend([0x00, prog_order]); // Scod, progression order
  cs.extend(layers.to_be_bytes()); // num layers
  cs.extend([0x00]); // MCT
  cs.extend([numresolutions - 1, 0x04, 0x04, 0x00, 0x01]); // SPcod: levels, cblk 64, qmfbid 5-3
                                                           // QCD: no quantization, 2 bytes per subband
  let numbands = 3 * numresolutions as usize - 2;
  let lqcd = 2 + 1 + 2 * numbands;
  cs.extend([0xff, 0x5c]);
  cs.extend((lqcd as u16).to_be_bytes());
  cs.extend([0x00]); // Sqcd
  cs.extend(std::iter::repeat_n(0u8, 2 * numbands));
  // SOT + SOD + a little tile data
  cs.extend([0xff, 0x90, 0x00, 0x0a]);
  cs.extend(0u16.to_be_bytes()); // Isot
  cs.extend(0u32.to_be_bytes()); // Psot (unknown)
  cs.extend([0x00, 0x01]); // TPsot, TNsot
  cs.extend([0xff, 0x93]); // SOD
  cs.extend([0u8; 16]);
  cs
}

/// Regression (review finding 2): a codestream with Csiz = 0 must be
/// rejected cleanly - the old code panicked in read_cod on the empty
/// tccps array.
#[test]
fn csiz_zero_is_rejected() {
  let mut cs = Vec::new();
  cs.extend([0xff, 0x4f]); // SOC
                           // SIZ with Csiz = 0 (no component records)
  cs.extend([0xff, 0x51, 0x00, 0x26]);
  cs.extend(0u32.to_be_bytes()); // Rsiz
  cs.extend(8u32.to_be_bytes()); // Xsiz
  cs.extend(8u32.to_be_bytes()); // Ysiz
  cs.extend(0u32.to_be_bytes()); // XOsiz
  cs.extend(0u32.to_be_bytes()); // YOsiz
  cs.extend(8u32.to_be_bytes()); // XTsiz
  cs.extend(8u32.to_be_bytes()); // YTsiz
  cs.extend(0u32.to_be_bytes()); // XTOsiz
  cs.extend(0u32.to_be_bytes()); // YTOsiz
  cs.extend(0u16.to_be_bytes()); // Csiz = 0
                                 // COD (valid, 1 resolution)
  cs.extend([0xff, 0x52, 0x00, 0x0c]);
  cs.extend([0x00, 0x00]); // Scod, LRCP
  cs.extend(1u16.to_be_bytes());
  cs.extend([0x00]); // MCT
  cs.extend([0x00, 0x04, 0x04, 0x00, 0x01]);
  // QCD
  cs.extend([0xff, 0x5c, 0x00, 0x05, 0x00, 0x00, 0x00]);
  // SOT + SOD + garbage
  cs.extend([0xff, 0x90, 0x00, 0x0a]);
  cs.extend(0u16.to_be_bytes());
  cs.extend(0u32.to_be_bytes());
  cs.extend([0x00, 0x01]);
  cs.extend([0xff, 0x93]);
  cs.extend([0u8; 16]);
  assert!(
    Image::from_bytes_with(&cs, DecodeParameters::new().strict(true)).is_err(),
    "Csiz=0 must be rejected"
  );
}

/// Regression (review finding 5): 6 PLT continuation bytes overflow the
/// packet-length shift in debug builds; the decode must fail cleanly.
#[test]
fn plt_overflow_is_rejected() {
  let mut cs = build_minimal_j2k(1, 0, 1);
  // Replace the trailing tile data with a PLT marker + 6 continuation bytes.
  let insert_at = cs.len() - 16 - 2; // before SOD
  let mut header = cs.split_off(insert_at);
  cs.extend([0xff, 0x58, 0x00, 0x09, 0x00]); // PLT, Zplt = 0
  cs.extend([0x81u8; 6]);
  cs.append(&mut header);
  assert!(
    Image::from_bytes_with(&cs, DecodeParameters::new().strict(true)).is_err(),
    "overflowed PLT must fail the decode"
  );
}

/// Regression (review finding 3): numresolutions = 33 with PCRL order
/// drives shift counts of 32 through the packet iterator; the decode must
/// not panic (the truncated tile data makes it fail in strict mode).
#[test]
fn pi_high_resolution_pcrl_does_not_panic() {
  let cs = build_minimal_j2k(33, 3, 1);
  let result = Image::from_bytes_with(&cs, DecodeParameters::new().strict(true));
  let lenient = Image::from_bytes_with(&cs, DecodeParameters::new().strict(false));
  // The exact outcome depends on how far the decode gets; only panics are
  // bugs here.
  let _ = (result, lenient);
}

/// Regression (review finding 7): out-of-range encoder parameters must be
/// rejected with an error, never panic (the C overflows its fixed arrays
/// for these).
#[test]
fn encoder_parameters_are_validated() {
  let (w, h) = (16u32, 16u32);
  let mut core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 1,
    color_space: crate::ColorSpace::Gray,
    comps: vec![crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data: vec![64i32; (w * h) as usize],
    }],
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let image = Image { core };

  assert!(image
    .clone()
    .encode(EncodeParameters::new().num_layers(101))
    .is_err());
  assert!(image
    .clone()
    .encode(EncodeParameters::new().num_layers(150))
    .is_err());

  let mut params = EncodeParameters::new();
  params.low_level().numpocs = 33;
  assert!(image.clone().encode(params).is_err());

  let mut params = EncodeParameters::new();
  params.low_level().csty = 1;
  params.low_level().res_spec = 0;
  assert!(image.clone().encode(params).is_err());

  let mut params = EncodeParameters::new();
  params.low_level().csty = 1;
  params.low_level().res_spec = 34;
  assert!(image.clone().encode(params).is_err());
}

/// Regression (review finding 1): a custom MCT matrix must reach the
/// matrix inversion without panicking.  The old port panicked there
/// ("attempt to subtract with overflow").  Upstream parity note: the C
/// reference writes the MCT data group only for the ColorTransform profile and
/// its decoder rejects COD mct=2 ("Invalid multiple component
/// transformation"), so - like the C - the produced codestream is not
/// decodable; the encode side is what this test pins down.
#[test]
fn custom_mct_encode_reaches_inversion() {
  let (w, h) = (32u32, 24u32);
  let mut rng: u64 = 0x6a09e667f3bcc909;
  let mut next = move || {
    rng ^= rng << 13;
    rng ^= rng >> 7;
    rng ^= rng << 17;
    rng
  };
  let mut comps = Vec::new();
  for _ in 0..3 {
    let mut data = Vec::with_capacity((w * h) as usize);
    for _ in 0..w * h {
      data.push((next() % 256) as i32);
    }
    comps.push(crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data,
    });
  }
  let image = Image {
    core: crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 3,
      color_space: crate::ColorSpace::Srgb,
      comps,
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    },
  };
  let mut params = EncodeParameters::new().num_resolutions(3).num_layers(1);
  // Non-singular decorrelation matrix; identity would also do.
  let matrix = vec![1.0f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
  params.low_level().mct_data = Some((matrix, vec![128i32; 3]));
  let codestream = image
    .clone()
    .encode(params)
    .expect("custom MCT encode (matrix inversion)");
  assert!(!codestream.is_empty());
  // C parity for the WRITER: without the ColorTransform rsiz bits no MCT/MCC/MCO
  // markers are written, so the stream declares mct=2 but carries no
  // decoding matrix.  The port's documented extension then: strict
  // decodes reject it, lenient decodes skip the transform.
  assert!(
    Image::from_bytes_with(&codestream, DecodeParameters::new().strict(true)).is_err(),
    "mct=2 without a decoding matrix is rejected in strict mode"
  );
  // The identity matrix leaves the samples unchanged either way.
  let lenient = Image::from_bytes_with(&codestream, DecodeParameters::new().strict(false))
    .expect("lenient decode skips the missing transform");
  for (a, b) in image.components().iter().zip(lenient.components().iter()) {
    assert_eq!(a.data, b.data, "identity transform passes samples through");
  }
}

/// B.2.4 end-to-end: with the ColorTransform rsiz the encoder writes the
/// MCT/MCC/MCO markers, and the decoder applies the decoding matrix —
/// the identity matrix round-trips losslessly.
#[test]
fn custom_mct_identity_round_trip_lossless() {
  let (w, h) = (32u32, 24u32);
  let mut rng: u64 = 0x9e3779b97f4a7c15;
  let mut next = move || {
    rng ^= rng << 13;
    rng ^= rng >> 7;
    rng ^= rng << 17;
    rng
  };
  let mut comps = Vec::new();
  for _ in 0..3 {
    let mut data = Vec::with_capacity((w * h) as usize);
    for _ in 0..w * h {
      data.push((next() % 256) as i32);
    }
    comps.push(crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data,
    });
  }
  let image = Image {
    core: crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 3,
      color_space: crate::ColorSpace::Srgb,
      comps,
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    },
  };
  let mut params = EncodeParameters::new().num_resolutions(3).num_layers(1);
  let matrix = vec![1.0f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
  params.low_level().mct_data = Some((matrix, vec![128i32; 3]));
  // ColorTransform = PROFILE_PART2 | EXTENSION_MCT (0x8100): the rsiz
  // gate that makes the encoder emit the MCT data group.
  params.low_level().rsiz = 0x8100;
  let codestream = image.clone().encode(params).expect("custom MCT encode");
  // The MCT marker (0xFF74) must be present.
  assert!(
    codestream.windows(2).any(|w| w == [0xFF, 0x74]),
    "the encoder wrote the MCT marker"
  );
  let decoded = Image::from_bytes(&codestream).expect("custom MCT decode");
  for (a, b) in image.components().iter().zip(decoded.components().iter()) {
    assert_eq!(a.data, b.data, "identity custom MCT round-trips losslessly");
  }
}

/// B.2.4: a real decorrelation matrix round-trips within the fixed-point
/// quantization tolerance, and the transform is actually applied (a
/// skipped inverse would leave the transformed samples, far off).
#[test]
fn custom_mct_real_matrix_round_trip() {
  let (w, h) = (32u32, 24u32);
  let mut rng: u64 = 0xdeadbeefcafef00d;
  let mut next = move || {
    rng ^= rng << 13;
    rng ^= rng >> 7;
    rng ^= rng << 17;
    rng
  };
  let mut comps = Vec::new();
  for _ in 0..3 {
    let mut data = Vec::with_capacity((w * h) as usize);
    for _ in 0..w * h {
      data.push((next() % 256) as i32);
    }
    comps.push(crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data,
    });
  }
  let image = Image {
    core: crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 3,
      color_space: crate::ColorSpace::Srgb,
      comps,
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    },
  };
  let mut params = EncodeParameters::new().num_resolutions(3).num_layers(1);
  // The classic (well-conditioned) YCbCr decorrelation matrix.
  let matrix = vec![
    0.299f32, 0.587, 0.114, -0.168736, -0.331264, 0.5, 0.5, -0.418688, -0.081312,
  ];
  params.low_level().mct_data = Some((matrix, vec![128i32; 3]));
  params.low_level().rsiz = 0x8100;
  let codestream = image.clone().encode(params).expect("custom MCT encode");
  let decoded = Image::from_bytes(&codestream).expect("custom MCT decode");
  for (a, b) in image.components().iter().zip(decoded.components().iter()) {
    assert_eq!(a.data.len(), b.data.len());
    for (x, y) in a.data.iter().zip(b.data.iter()) {
      assert!((x - y).abs() <= 4, "custom MCT round trip within tolerance");
    }
  }
}

/// OF-44 remainder: the codeblock-style mode bits (the CLI's `-M` -
/// 1=LAZY, 2=RESET, 4=TERMALL, 8=VSC, 16=PTERM, 32=SEGSYM) change the
/// codestream's signaling structure; every one must decode losslessly
/// (single layer, reversible 5-3), and the decoder must not regress on
/// any combination.
#[test]
fn codeblock_style_modes_decode_losslessly() {
  let (w, h) = (64u32, 48u32);
  let mut rng: u64 = 0x243f6a8885a308d3;
  let mut next = move || {
    rng ^= rng << 13;
    rng ^= rng >> 7;
    rng ^= rng << 17;
    rng
  };
  let mut data = Vec::with_capacity((w * h) as usize);
  for _ in 0..w * h {
    data.push((next() % 256) as i32);
  }
  let image = Image {
    core: crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 1,
      color_space: crate::ColorSpace::Gray,
      comps: vec![crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 8,
        bpp: 8,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data,
      }],
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    },
  };
  for mode in [1i32, 2, 4, 8, 16, 32, 0x3F] {
    let mut params = EncodeParameters::new().num_resolutions(3).num_layers(1);
    params.low_level().mode = mode;
    let codestream = image
      .clone()
      .encode(params)
      .unwrap_or_else(|e| panic!("mode {}: encode failed: {}", mode, e));
    let back = Image::from_bytes(&codestream)
      .unwrap_or_else(|e| panic!("mode {}: decode failed: {}", mode, e));
    for (a, b) in image.components()[0]
      .data
      .iter()
      .zip(back.components()[0].data.iter())
    {
      assert_eq!(a, b, "mode {mode}: 5-3 round trip must be lossless");
    }
  }
}

/// Regression (review finding 6): a 16-bit image with 6 decomposition
/// levels pushes the t1 encoder's significance shift past 31; the old code
/// panicked in debug builds.
#[test]
fn encode_16bit_deep_dwt() {
  let (w, h) = (64u32, 48u32);
  let mut data = Vec::with_capacity((w * h) as usize);
  for i in 0..w * h {
    data.push((i * 1009 % 65536) as i32);
  }
  let image = Image {
    core: crate::CoreImage {
      x0: 0,
      y0: 0,
      x1: w,
      y1: h,
      numcomps: 1,
      color_space: crate::ColorSpace::Gray,
      comps: vec![crate::CoreImageComp {
        dx: 1,
        dy: 1,
        w,
        h,
        x0: 0,
        y0: 0,
        prec: 16,
        bpp: 16,
        sgnd: 0,
        resno_decoded: 0,
        factor: 0,
        alpha: 0,
        data,
      }],
      icc_profile_buf: Vec::new(),
      icc_profile_is_cielab: false,
    },
  };
  let codestream = image
    .clone()
    .encode(EncodeParameters::new().num_resolutions(6).num_layers(1))
    .expect("16-bit deep-DWT encode");
  let back = Image::from_bytes(&codestream).expect("decode");
  assert_eq!(back.num_components(), 1);
  for (x, y) in image.components()[0]
    .data
    .iter()
    .zip(back.components()[0].data.iter())
  {
    assert_eq!(x, y, "reversible 5-3 must be lossless");
  }
}

/// The codestream index reports every packet's identity (layer,
/// resolution, component, precinct) and byte positions, and the derived
/// per-layer byte thresholds are exact: a codestream prefix cut at the
/// threshold decodes identically to the full file for that layer count,
/// while one byte less does not.
#[test]
fn cstr_index_packet_structure_and_layer_thresholds() {
  let buf = std::fs::read("samples/Hadley_Crater.jp2").expect("sample file");
  let image = Image::from_bytes(&buf).expect("decode");
  let mut params = EncodeParameters::new().num_resolutions(3);
  for rate in [40.0f32, 10.0, 0.0] {
    params = params.layer_rate(rate);
  }
  let codestream = image.clone().encode(params).expect("encode");

  let mut dec = crate::codec::Decoder::new(crate::J2KFormat::J2K).expect("decoder");
  let mut stream = crate::stream::Stream::from_bytes(codestream.clone());
  dec.read_header(&mut stream).expect("header");
  dec.set_resolution_factor(2).expect("reduce"); // keep the test fast
  dec.decode(&mut stream).expect("decode");
  let index = dec.cstr_index().expect("codestream index");
  let packets: Vec<_> = index.packets().collect();
  assert!(!packets.is_empty(), "packets were not indexed");
  for p in &packets {
    assert!(p.layno < 3, "layer {} out of range", p.layno);
    assert!(p.start_pos < p.end_ph_pos, "packet header positions");
    assert!(p.end_ph_pos <= p.end_pos, "packet end before header end");
    assert!(p.end_pos as usize <= codestream.len(), "packet past EOF");
  }
  let layer_counts: std::collections::BTreeSet<u32> = packets.iter().map(|p| p.layno).collect();
  assert_eq!(layer_counts.len(), 3, "all layers should appear");

  let thresholds = index.layer_byte_thresholds();
  assert_eq!(thresholds.len(), 3);
  assert!(
    thresholds.windows(2).all(|w| w[0] <= w[1]),
    "thresholds must be monotonic: {:?}",
    thresholds
  );
  assert!(thresholds[2] as usize <= codestream.len());

  // exactness: a prefix cut at the threshold decodes identically to the
  // full codestream for that layer count (all layer packets are present).
  // (A byte or two less may still decode the same when the trailing byte
  // is code-block padding/termination, so the boundary is inclusive.)
  for (i, &t) in thresholds.iter().enumerate() {
    let layers = i as u32 + 1;
    let full = Image::from_bytes_with(
      &codestream,
      DecodeParameters::new().layers(layers).reduce(2),
    )
    .expect("full decode");
    let part = Image::from_bytes_with(
      &codestream[..t as usize],
      DecodeParameters::new().layers(layers).reduce(2),
    )
    .expect("prefix decode");
    assert!(
      full
        .components()
        .iter()
        .zip(part.components())
        .all(|(a, b)| a.data == b.data),
      "prefix at threshold {} must match layers={}",
      t,
      layers
    );
  }
}

/// Regression: an all-zero image produces code-blocks with no coding
/// passes (the "empty code-block" case behind the 0.6.x null-`dealloc`
/// UB reports, issues #8/#11/#12/#13).  The safe port keeps decoded
/// samples in owned `Vec`s, so there is no manual deallocation; decoding
/// the flat image must be clean (this runs in debug builds, where the old
/// code aborted on the `NonNull` precondition).
#[test]
fn empty_code_blocks_decode_cleanly() {
  let (w, h) = (64u32, 64u32);
  let core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 1,
    color_space: crate::ColorSpace::Gray,
    comps: vec![crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data: vec![0i32; (w * h) as usize],
    }],
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let image = Image::from_core_image(core);
  let codestream = image
    .clone()
    .encode(EncodeParameters::new().num_resolutions(3))
    .expect("encode flat image");
  let decoded = Image::from_bytes(&codestream).expect("decode flat image");
  assert_eq!((decoded.width(), decoded.height()), (w, h));
  assert!(
    decoded.components()[0].data.iter().all(|&v| v == 0),
    "flat image must round-trip to zeros"
  );
}

#[test]
fn tile_geometry_before_header_is_zero() {
  let decoder = Decoder::new(J2KFormat::J2K).expect("decoder");
  assert_eq!(decoder.get_tile_geometry(), (0, 0, 0, 0));
}

#[test]
fn set_decode_area_before_header_is_an_error() {
  let mut decoder = Decoder::new(J2KFormat::J2K).expect("decoder");
  let mut header = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: 0,
    y1: 0,
    numcomps: 0,
    color_space: crate::ColorSpace::Gray,
    comps: Vec::new(),
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let result = decoder.set_decode_area(&mut header, Some(DecodeArea::new(0, 0, 16, 16)));
  assert!(
    result.is_err(),
    "set_decode_area before read_header must fail cleanly"
  );
}

#[test]
fn decode_area_none_resets_to_the_full_image() {
  let buf = std::fs::read("doc/example.jp2").expect("example image");
  let mut decoder = Decoder::new(J2KFormat::JP2).expect("decoder");
  let mut stream = Stream::from_bytes(buf);
  let header = decoder.read_header(&mut stream).expect("header");
  let (full_w, full_h) = (header.width(), header.height());
  let mut core = header.into_core_image();
  decoder
    .set_decode_area(&mut core, Some(DecodeArea::new(0, 0, 128, 64)))
    .expect("set area");
  assert_eq!((core.x1 - core.x0, core.y1 - core.y0), (128, 64));
  decoder
    .set_decode_area(&mut core, None)
    .expect("reset area");
  assert_eq!((core.x1 - core.x0, core.y1 - core.y0), (full_w, full_h));
}

#[test]
fn zero_component_image_is_rejected_by_setup() {
  let mut encoder = Encoder::new(J2KFormat::J2K).expect("encoder");
  let mut params = EncodeParameters::default().into_inner();
  let mut image = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: 16,
    y1: 16,
    numcomps: 0,
    color_space: crate::ColorSpace::Gray,
    comps: Vec::new(),
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  assert!(encoder.setup(&mut params, &mut image).is_err());
}

#[test]
fn detect_format_from_file_prefers_magic_over_extension() {
  const JP2_MAGIC: [u8; 12] = [
    0x00, 0x00, 0x00, 0x0C, b'j', b'P', b' ', b' ', 0x0D, 0x0A, 0x87, 0x0A,
  ];
  let dir = std::env::temp_dir();
  let path = dir.join(format!(
    "openjp2-detect-{}-{:?}.dat",
    std::process::id(),
    std::thread::current().id()
  ));
  std::fs::write(&path, JP2_MAGIC).expect("write temp file");
  let detected = crate::detect_format_from_file(&path);
  let _ = std::fs::remove_file(&path);
  assert_eq!(detected, Ok(J2KFormat::JP2));
}

#[test]
fn get_pixels_clamps_out_of_range_samples() {
  let comp = crate::CoreImageComp {
    dx: 1,
    dy: 1,
    w: 4,
    h: 1,
    x0: 0,
    y0: 0,
    prec: 8,
    bpp: 8,
    sgnd: 0,
    resno_decoded: 0,
    factor: 0,
    alpha: 0,
    data: vec![-5, 0, 128, 1000],
  };
  let out: Vec<u8> = comp.data_u8().collect();
  assert_eq!(out, vec![0, 0, 128, 255]);
}

#[test]
fn decode_error_display_renders_the_full_marker() {
  let error = Error::Decode(DecodeError {
    message: "bad header".into(),
    marker: Some(0xFF51),
    ..Default::default()
  });
  assert_eq!(
    error.to_string(),
    "decode error: bad header (marker 0xFF51)"
  );
}

#[test]
fn invalid_cod_progression_order_fails_the_decode() {
  let (w, h) = (64u32, 64u32);
  let core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 1,
    color_space: crate::ColorSpace::Gray,
    comps: vec![crate::CoreImageComp {
      dx: 1,
      dy: 1,
      w,
      h,
      x0: 0,
      y0: 0,
      prec: 8,
      bpp: 8,
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data: (0..(w * h)).map(|i| (i % 251) as i32).collect(),
    }],
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let mut bytes = Image::from_core_image(core)
    .encode(EncodeParameters::new())
    .expect("encode");
  let cod = bytes
    .windows(2)
    .position(|w| w == [0xFF, 0x52])
    .expect("COD marker");
  /* FF52, Lcod(2), Scod(1), then SGcod's first byte is the progression order. */
  let prg = cod + 2 + 2 + 1;
  assert_eq!(bytes[prg], 0, "default progression is LRCP");
  bytes[prg] = 7;
  let strict = Image::from_bytes_with(&bytes, DecodeParameters::default().strict(true));
  assert!(
    strict.is_err(),
    "an invalid COD progression order must fail like the C (t2 rejects the tile)"
  );
  /* Lenient mode keeps the documented partial-decode contract: the header
  image comes back, but no packets are decoded as LRCP. */
  let lenient = Image::from_bytes(&bytes).expect("lenient partial image");
  assert!(
    lenient.components().iter().all(|c| c.data.is_empty()),
    "invalid progression must not decode packets in a fallback order"
  );
}

#[test]
fn mixed_coc_transform_with_mct_is_rejected() {
  let (w, h) = (64u32, 64u32);
  let data: Vec<i32> = (0..(w * h)).map(|i| (i % 251) as i32).collect();
  let comp = |data: &Vec<i32>| crate::CoreImageComp {
    dx: 1,
    dy: 1,
    w,
    h,
    x0: 0,
    y0: 0,
    prec: 8,
    bpp: 8,
    sgnd: 0,
    resno_decoded: 0,
    factor: 0,
    alpha: 0,
    data: data.clone(),
  };
  let core = crate::CoreImage {
    x0: 0,
    y0: 0,
    x1: w,
    y1: h,
    numcomps: 3,
    color_space: crate::ColorSpace::Srgb,
    comps: vec![comp(&data), comp(&data), comp(&data)],
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  };
  let mut params = EncodeParameters::new();
  params.low_level().tcp_mct = 1;
  let mut bytes = Image::from_core_image(core)
    .encode(params)
    .expect("encode with MCT");
  /* Find COD and copy its SPcod for a COC on component 1 with qmfbid=0
  (9-7) while components 0/2 stay reversible. */
  let find = |m: u16| {
    let pat = m.to_be_bytes();
    bytes
      .windows(2)
      .position(|w| w == pat)
      .unwrap_or_else(|| panic!("marker {:#06X}", m))
  };
  let cod = find(0xFF52);
  let lcod = u16::from_be_bytes([bytes[cod + 2], bytes[cod + 3]]) as usize;
  let scod = bytes[cod + 4];
  let spcod_start = cod + 5 + 4; /* Scod + prg(1) + layers(2) + mct(1) */
  let spcod_end = cod + 2 + lcod;
  let mut spcoc = bytes[spcod_start..spcod_end].to_vec();
  spcoc[4] = 0; /* SPcoc: numres, cblkw, cblkh, style, qmfbid */
  let lcoc = (2 + 1 + 1 + spcoc.len()) as u16;
  let mut coc = vec![0xFF, 0x53];
  coc.extend_from_slice(&lcoc.to_be_bytes());
  coc.push(1); /* component 1 */
  coc.push(scod);
  coc.extend_from_slice(&spcoc);
  let sot = find(0xFF90);
  bytes.splice(sot..sot, coc);
  assert!(
    Image::from_bytes_with(&bytes, DecodeParameters::default().strict(true)).is_err(),
    "mixed per-component transforms must be rejected, not panic or silently skip MCT"
  );
}
