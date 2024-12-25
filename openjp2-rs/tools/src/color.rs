use log;
use std::cmp::{max, min};
use std::convert::TryInto;

use openjp2::openjpeg::*;

#[cfg(feature = "lcms2")]
use lcms2::*;

// ...existing code...

// --------------------------------------------------------
// Matrix for sYCC, Amendment 1 to IEC 61966-2-1
//
// Y :   0.299   0.587    0.114   :R
// Cb:  -0.1687 -0.3312   0.5     :G
// Cr:   0.5    -0.4187  -0.0812  :B
//
// Inverse:
//
// R: 1        -3.68213e-05    1.40199      :Y
// G: 1.00003  -0.344125      -0.714128     :Cb - 2^(prec - 1)
// B: 0.999823  1.77204       -8.04142e-06  :Cr - 2^(prec - 1)
//
// -----------------------------------------------------------
fn sycc_to_rgb(offset: i32, upb: i32, y: i32, cb: i32, cr: i32) -> (i32, i32, i32) {
  let cb = cb - offset;
  let cr = cr - offset;

  let mut r = y + (1.402 * cr as f32) as i32;
  let mut g = y - (0.34413 * cb as f32 + 0.71414 * cr as f32) as i32;
  let mut b = y + (1.772 * cb as f32) as i32;

  r = min(max(r, 0), upb);
  g = min(max(g, 0), upb);
  b = min(max(b, 0), upb);

  (r, g, b)
}

fn sycc444_to_rgb(image: &mut opj_image_t) {
  let (maxw, maxh, prec) = image.comp0_dims_prec();
  let comps = match image.comps_mut() {
    Some(c) => c,
    None => return,
  };

  if comps.len() < 3 {
    return;
  }

  let max = maxw * maxh;

  let upb = (1 << prec) - 1;
  let offset = 1 << (prec - 1);

  let (y_data, cb_data, cr_data) = match (comps[0].data(), comps[1].data(), comps[2].data()) {
    (Some(y), Some(cb), Some(cr)) => (y, cb, cr),
    _ => return,
  };

  let mut r = Vec::with_capacity(max);
  let mut g = Vec::with_capacity(max);
  let mut b = Vec::with_capacity(max);

  for i in 0..max {
    let (rd, gd, bd) = sycc_to_rgb(offset, upb, y_data[i], cb_data[i], cr_data[i]);
    r.push(rd);
    g.push(gd);
    b.push(bd);
  }

  image.set_rgb(maxw, maxh, &r, &g, &b);
}

fn sycc422_to_rgb(image: &mut opj_image_t) {
  let (maxw, maxh, prec) = image.comp0_dims_prec();

  let comps = match image.comps() {
    Some(c) => c,
    None => return,
  };

  let max = maxw * maxh;

  let upb = (1 << prec) - 1;
  let offset = 1 << (prec - 1);

  let (y_data, cb_data, cr_data) = match (comps[0].data(), comps[1].data(), comps[2].data()) {
    (Some(y), Some(cb), Some(cr)) => (y, cb, cr),
    _ => return,
  };

  let mut r = Vec::with_capacity(max);
  let mut g = Vec::with_capacity(max);
  let mut b = Vec::with_capacity(max);

  // if img->x0 is odd, then first column shall use Cb/Cr = 0
  let offx = image.x0 & 1;
  let loopmaxw = maxw - (offx as usize);

  let mut y_off = 0;
  let mut cb_off = 0;
  let mut cr_off = 0;
  for _ in 0..maxh {
    // Handle first pixel if offset
    if offx > 0 {
      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y_data[y_off], 0, 0);
      r.push(rd);
      g.push(gd);
      b.push(bd);
      y_off += 1;
    }

    // Handle pairs of pixels
    let mut j = 0;
    while j < (loopmaxw & !1) {
      let y = y_data[y_off];
      let cb = cb_data[cb_off];
      let cr = cr_data[cr_off];

      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y, cb, cr);
      r.push(rd);
      g.push(gd);
      b.push(bd);

      y_off += 1;
      let y = y_data[y_off];
      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y, cb, cr);
      r.push(rd);
      g.push(gd);
      b.push(bd);

      y_off += 1;
      cb_off += 1;
      cr_off += 1;
      j += 2;
    }

    // Handle last pixel if needed
    if j < loopmaxw {
      let y1 = y_data[y_off];
      let cb = cb_data[cb_off];
      let cr = cr_data[cr_off];

      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y1, cb, cr);
      r.push(rd);
      g.push(gd);
      b.push(bd);

      y_off += 1;
      cb_off += 1;
      cr_off += 1;
    }
  }

  // Update image data
  image.set_rgb(maxw, maxh, &r, &g, &b);
}

fn sycc420_to_rgb(image: &mut opj_image_t) {
  let (maxw, maxh, prec) = image.comp0_dims_prec();

  let comps = match image.comps() {
    Some(c) => c,
    None => return,
  };

  let max = maxw * maxh;

  let upb = (1 << prec) - 1;
  let offset = 1 << (prec - 1);

  let (y_d, cb_d, cr_d) = match (comps[0].data(), comps[1].data(), comps[2].data()) {
    (Some(y), Some(cb), Some(cr)) => (y, cb, cr),
    _ => return,
  };

  // TODO: need to fill with zeros to write to next row.
  let mut r = Vec::with_capacity(max);
  let mut g = Vec::with_capacity(max);
  let mut b = Vec::with_capacity(max);

  // if img->x0 is odd, then first column shall use Cb/Cr = 0
  let offx = image.x0 & 1;
  let loopmaxw = maxw - (offx as usize);
  // if img->y0 is odd, then first line shall use Cb/Cr = 0
  let offy = image.y0 & 1;
  let loopmaxh = maxh - (offy as usize);

  // Handle first row if offset
  if offy > 0 {
    for j in 0..maxw {
      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y_d[j], 0, 0);
      r.push(rd);
      g.push(gd);
      b.push(bd);
    }
  }

  for i in (0..(loopmaxh & !1)).step_by(2) {
    let row_off = i * maxw;
    let next_row = row_off + maxw;
    let cbcr_row = (i / 2) * (maxw / 2);

    // Handle first pixel if offset
    if offx > 0 {
      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y_d[row_off], 0, 0);
      r.push(rd);
      g.push(gd);
      b.push(bd);

      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y_d[next_row], cb_d[row_off], cr_d[row_off]);
      // TODO: broken.  need to write to next row.
      r.push(rd);
      g.push(gd);
      b.push(bd);
    }

    // Handle pixel pairs
    let mut j = 0;
    while j < (loopmaxw & !1) {
      let cb = cb_d[cbcr_row + j / 2];
      let cr = cr_d[cbcr_row + j / 2];

      // Current row
      let y1 = y_d[row_off + j + offx as usize];
      let y2 = y_d[row_off + j + 1 + offx as usize];

      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y1, cb, cr);
      r.push(rd);
      g.push(gd);
      b.push(bd);

      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y2, cb, cr);
      r.push(rd);
      g.push(gd);
      b.push(bd);

      // Next row
      let y1 = y_d[next_row + j + offx as usize];
      let y2 = y_d[next_row + j + 1 + offx as usize];

      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y1, cb, cr);
      r.push(rd);
      g.push(gd);
      b.push(bd);

      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y2, cb, cr);
      r.push(rd);
      g.push(gd);
      b.push(bd);

      j += 2;
    }

    // Handle last pixel if needed
    if j < loopmaxw {
      let cb = cb_d[cbcr_row + j / 2];
      let cr = cr_d[cbcr_row + j / 2];

      let y1 = y_d[row_off + j + offx as usize];
      let y2 = y_d[next_row + j + offx as usize];

      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y1, cb, cr);
      r.push(rd);
      g.push(gd);
      b.push(bd);

      let (rd, gd, bd) = sycc_to_rgb(offset, upb, y2, cb, cr);
      r.push(rd);
      g.push(gd);
      b.push(bd);
    }
  }

  // Update image data
  image.set_rgb(maxw, maxh, &r, &g, &b);
}

pub fn color_sycc_to_rgb(image: &mut opj_image_t) {
  let comps = match image.comps() {
    Some(c) => c,
    None => return,
  };

  if comps.len() < 3 {
    image.color_space = OPJ_CLRSPC_GRAY;
    return;
  }

  if comps[0].dx == 1
    && comps[1].dx == 2
    && comps[2].dx == 2
    && comps[0].dy == 1
    && comps[1].dy == 2
    && comps[2].dy == 2
  {
    // horizontal and vertical sub-sample
    sycc420_to_rgb(image);
  } else if comps[0].dx == 1
    && comps[1].dx == 2
    && comps[2].dx == 2
    && comps[0].dy == 1
    && comps[1].dy == 1
    && comps[2].dy == 1
  {
    // horizontal sub-sample only
    sycc422_to_rgb(image);
  } else if comps[0].dx == 1
    && comps[1].dx == 1
    && comps[2].dx == 1
    && comps[0].dy == 1
    && comps[1].dy == 1
    && comps[2].dy == 1
  {
    // no sub-sample
    sycc444_to_rgb(image);
  } else {
    eprintln!(
      "{}:{}: color_sycc_to_rgb\n\tCAN NOT CONVERT",
      file!(),
      line!()
    );
  }
}

#[cfg(feature = "lcms2")]
pub fn color_apply_icc_profile(image: &mut opj_image_t) {
  let icc_profile = match image.icc_profile() {
    Some(p) => p,
    None => return,
  };

  let in_profile = match Profile::new_icc(icc_profile) {
    Ok(p) => p,
    Err(_) => return,
  };

  let out_space = in_profile.color_space();
  let out_profile = Profile::new_srgb();
  let intent = in_profile.header_rendering_intent();

  let (maxw, maxh, prec) = image.comp0_dims_prec();

  let (in_type, out_type) = if out_space == ColorSpaceSignature::RgbData {
    // enumCS 16
    let nr_comp = image.numcomps;

    if nr_comp < 3 {
      // GRAY or GRAYA, not RGB or RGBA
      return;
    }

    // Check if components match
    if !image.comps_match() {
      return;
    }

    if prec <= 8 {
      (PixelFormat::RGB_8, PixelFormat::RGB_8)
    } else {
      (PixelFormat::RGB_16, PixelFormat::RGB_16)
    }
  } else if out_space == ColorSpaceSignature::GrayData {
    // enumCS 17
    (PixelFormat::GRAY_8, PixelFormat::RGB_8)
  } else if out_space == ColorSpaceSignature::YCbCrData {
    // enumCS 18
    if image.numcomps < 3 {
      return;
    }
    (PixelFormat::YCbCr_16, PixelFormat::RGB_16)
  } else {
    log::debug!(
      "ICC Profile has unknown output colorspace({:x})({:?})",
      out_space as u32,
      String::from_utf8_lossy(&[
        ((out_space as u32) >> 24) as u8,
        ((out_space as u32) >> 16) as u8,
        ((out_space as u32) >> 8) as u8,
        (out_space as u32) as u8
      ])
    );
    return;
  };

  // Take ownership of the old components.
  let orig = image.take_comps();
  let Some(mut orig_comps) = orig.comps_data_iter() else {
    eprintln!("color_apply_icc_profile: missing components");
    return;
  };
  // Should always have at least one component
  let Some(o_red) = orig_comps.next() else {
    eprintln!("color_apply_icc_profile: missing component 0");
    return;
  };
  // if RGB(A) then we have two more components
  let (o_green, o_blue) = if orig.numcomps >= 3 {
    let o_green = orig_comps.next();
    let o_blue = orig_comps.next();
    (o_green, o_blue)
  } else {
    (None, None)
  };
  // if RGBA or GRAYA then we have one more component
  let o_alpha = orig_comps.next();

  // Allocate new components
  let numcomps = match orig.numcomps {
    1 | 3 => 3,
    2 | 4 => 4,
    _ => {
      eprintln!(
        "color_apply_icc_profile: invalid numcomps {}",
        orig.numcomps
      );
      return;
    }
  };
  if !image.alloc_comps(numcomps) {
    eprintln!("color_apply_icc_profile: failed to allocate components");
    return;
  }
  image.color_space = OPJ_CLRSPC_SRGB;

  // Copy the original components details to the new components
  let mut comps = image
    .comps_mut()
    .expect("We just allocated this")
    .iter_mut();
  // There must be at least 3 components (RGB).  Get and initialize them.
  let red = comps.next().expect("We just allocated this");
  red.copy_props(o_red.comp);
  let green = comps.next().expect("We just allocated this");
  green.copy_props(o_red.comp);
  let blue = comps.next().expect("We just allocated this");
  blue.copy_props(o_red.comp);

  // Allocate data for the new components
  if !red.alloc_data() || !green.alloc_data() || !blue.alloc_data() {
    eprintln!("color_apply_icc_profile: failed to allocate data");
    return;
  }

  // Just copy the alpha channel if it exists
  if let Some(o_alpha) = o_alpha {
    let alpha = comps.next().expect("We just allocated this");
    alpha.copy(o_alpha.comp);
  }

  // Get the component data
  let red = red.data_mut().expect("We just allocated this");
  let green = green.data_mut().expect("We just allocated this");
  let blue = blue.data_mut().expect("We just allocated this");
  let comp_pixels = red.iter_mut().zip(green.iter_mut()).zip(blue.iter_mut());

  let num_pixels = maxw * maxh;
  if prec <= 8 {
    let transform = match Transform::new(&in_profile, in_type, &out_profile, out_type, intent) {
      Ok(t) => t,
      Err(e) => {
        eprintln!("color_apply_icc_profile: {:?}", e);
        return;
      }
    };

    // Copy the original component data to a single buffer
    let mut in_data: Vec<u8> = Vec::with_capacity(num_pixels * 3);
    match (o_red, o_green, o_blue) {
      (red, Some(green), Some(blue)) => {
        for ((r, g), b) in red.data.iter().zip(green.data.iter()).zip(blue.data.iter()) {
          in_data.push(*r as u8);
          in_data.push(*g as u8);
          in_data.push(*b as u8);
        }
      }
      (gray, None, None) => {
        for v in gray.data.iter() {
          in_data.push(*v as u8);
        }
      }
      _ => {
        eprintln!("color_apply_icc_profile: invalid components");
        return;
      }
    }

    // Transform the pixels
    let mut out_data = vec![0u8; num_pixels * 3];
    transform.transform_pixels(&in_data, &mut out_data);

    // Copy the transformed data back to the components
    let src_pixels = out_data.chunks_exact(3);
    for (src, ((r, g), b)) in src_pixels.zip(comp_pixels) {
      *r = src[0] as i32;
      *g = src[1] as i32;
      *b = src[2] as i32;
    }
  } else {
    let mut out_data = vec![[0u16, 0u16, 0u16]; num_pixels];

    // Copy the original component data to a single buffer
    match (o_red, o_green, o_blue) {
      (red, Some(green), Some(blue)) => {
        let mut in_data = vec![[0u16, 0u16, 0u16]; num_pixels];
        for ((r, g), b) in red.data.iter().zip(green.data.iter()).zip(blue.data.iter()) {
          in_data.push([*r as u16, *g as u16, *b as u16]);
        }
        let transform = match Transform::new(&in_profile, in_type, &out_profile, out_type, intent) {
          Ok(t) => t,
          Err(e) => {
            eprintln!("color_apply_icc_profile: {:?}", e);
            return;
          }
        };

        // Transform the pixels
        transform.transform_pixels(&in_data, &mut out_data);
      }
      (gray, None, None) => {
        let mut in_data: Vec<u16> = Vec::with_capacity(num_pixels);
        for v in gray.data.iter() {
          in_data.push(*v as u16);
        }

        let transform = match Transform::new(&in_profile, in_type, &out_profile, out_type, intent) {
          Ok(t) => t,
          Err(e) => {
            eprintln!("color_apply_icc_profile: {:?}", e);
            return;
          }
        };

        // Transform the pixels
        transform.transform_pixels(&in_data, &mut out_data);
      }
      _ => {
        eprintln!("color_apply_icc_profile: invalid components");
        return;
      }
    }

    // Copy the transformed data back to the components
    for (src, ((r, g), b)) in out_data.iter().zip(comp_pixels) {
      *r = src[0] as i32;
      *g = src[1] as i32;
      *b = src[2] as i32;
    }
  }
}

#[cfg(feature = "lcms2")]
pub fn color_cielab_to_rgb(image: &mut opj_image_t) {
  // Check dimensions match
  if !image.comps_same_dims() {
    log::error!("color_cielab_to_rgb: components are not all of the same dimension");
    return;
  }

  let icc_profile = image.icc_profile().map(|icc| icc.to_owned());
  let (profile_data, comps) = match (icc_profile, image.comps_mut()) {
    (Some(p), Some(c)) => (p, c),
    _ => return,
  };

  if comps.len() != 3 {
    eprintln!(
      "{}:{}: color_cielab_to_rgb\n\tnumcomps {} not handled",
      file!(),
      line!(),
      comps.len()
    );
    return;
  }

  // Get color space enum from ICC profile
  let enumcs = match profile_data
    .get(0..4)
    .map(|b| i32::from_ne_bytes(b.try_into().unwrap()))
  {
    Some(cs) => cs,
    None => return,
  };

  if enumcs == 14 {
    // CIELab
    let in_profile = match Profile::new_lab4_context(GlobalContext::new(), &Default::default()) {
      Ok(p) => p,
      Err(_) => return,
    };

    let out_profile = Profile::new_srgb();

    let transform = match Transform::new(
      &in_profile,
      PixelFormat::Lab_DBL,
      &out_profile,
      PixelFormat::RGB_16,
      Intent::Perceptual,
    ) {
      Ok(t) => t,
      Err(_) => return,
    };

    let w = comps[0].w;
    let h = comps[0].h;
    let max = (w * h) as usize;

    // Get range info from ICC profile
    let (rl, ra, rb, ol, oa, ob) = if profile_data.len() >= 8 {
      let default_type = i32::from_ne_bytes(profile_data[4..8].try_into().unwrap());
      if default_type == 0x44454600 {
        // DEF : default
        (
          100.0,
          170.0,
          200.0,
          0.0,
          2f64.powi(comps[1].prec as i32 - 1),
          2f64.powi(comps[2].prec as i32 - 2) + 2f64.powi(comps[2].prec as i32 - 3),
        )
      } else if profile_data.len() >= 32 {
        let values = profile_data[8..32]
          .chunks(4)
          .map(|b| f64::from_bits(u64::from_ne_bytes([0, 0, 0, 0, b[0], b[1], b[2], b[3]])))
          .collect::<Vec<_>>();
        (
          values[0], values[2], values[4], values[1], values[3], values[5],
        )
      } else {
        return;
      }
    } else {
      return;
    };

    let prec0 = comps[0].prec as f64;
    let prec1 = comps[1].prec as f64;
    let prec2 = comps[2].prec as f64;

    let min_l = -(rl * ol) / (2f64.powi(prec0 as i32) - 1.0);
    let max_l = min_l + rl;
    let mina = -(ra * oa) / (2f64.powi(prec1 as i32) - 1.0);
    let maxa = mina + ra;
    let minb = -(rb * ob) / (2f64.powi(prec2 as i32) - 1.0);
    let maxb = minb + rb;

    let (src_l, src_a, src_b) = match (comps[0].data(), comps[1].data(), comps[2].data()) {
      (Some(l), Some(a), Some(b)) => (l, a, b),
      _ => return,
    };

    let mut r = vec![0i32; max];
    let mut g = vec![0i32; max];
    let mut b = vec![0i32; max];

    for i in 0..max {
      let lab = [
        min_l + (src_l[i] as f64) * (max_l - min_l) / (2f64.powi(prec0 as i32) - 1.0),
        mina + (src_a[i] as f64) * (maxa - mina) / (2f64.powi(prec1 as i32) - 1.0),
        minb + (src_b[i] as f64) * (maxb - minb) / (2f64.powi(prec2 as i32) - 1.0),
      ];

      let mut rgb = [0u16; 3];
      transform.transform_pixels(&[lab], &mut rgb);

      r[i] = rgb[0] as i32;
      g[i] = rgb[1] as i32;
      b[i] = rgb[2] as i32;
    }

    // Update image data
    image.set_rgb(w as usize, h as usize, &r, &g, &b);
    if let Some(comps) = image.comps_mut() {
      comps[0].prec = 16;
      comps[1].prec = 16;
      comps[2].prec = 16;
    }
    return;
  }

  eprintln!(
    "{}:{}: color_cielab_to_rgb\n\tenumCS {} not handled",
    file!(),
    line!(),
    enumcs
  );
}

pub fn color_cmyk_to_rgb(image: &mut opj_image_t) {
  let mut comps = match image.comps_mut() {
    Some(&mut [c, m, y, k]) => (c, m, y, k),
    _ => return,
  };

  if comps.0.dx != comps.1.dx
    || comps.0.dx != comps.2.dx
    || comps.0.dx != comps.3.dx
    || comps.0.dy != comps.1.dy
    || comps.0.dy != comps.2.dy
    || comps.0.dy != comps.3.dy
  {
    eprintln!(
      "{}:{}: color_cmyk_to_rgb\n\tCAN NOT CONVERT",
      file!(),
      line!()
    );
    return;
  }

  let w = comps.0.w;
  let h = comps.0.h;
  let max = (w * h) as usize;

  let sc = 1.0 / ((1 << comps.0.prec) - 1) as f32;
  let sm = 1.0 / ((1 << comps.1.prec) - 1) as f32;
  let sy = 1.0 / ((1 << comps.2.prec) - 1) as f32;
  let sk = 1.0 / ((1 << comps.3.prec) - 1) as f32;

  let (c_data, m_data, y_data, k_data) = match (
    comps.0.data_mut(),
    comps.1.data_mut(),
    comps.2.data_mut(),
    comps.3.data(),
  ) {
    (Some(c), Some(m), Some(y), Some(k)) => (c, m, y, k),
    _ => return,
  };

  for i in 0..max {
    // CMYK values from 0 to 1
    let c = 1.0 - (c_data[i] as f32 * sc);
    let m = 1.0 - (m_data[i] as f32 * sm);
    let y = 1.0 - (y_data[i] as f32 * sy);
    let k = 1.0 - (k_data[i] as f32 * sk);

    // CMYK -> RGB
    c_data[i] = (255.0 * c * k) as i32; // R
    m_data[i] = (255.0 * m * k) as i32; // G
    y_data[i] = (255.0 * y * k) as i32; // B
  }

  // Update component properties
  comps.0.prec = 8;
  comps.1.prec = 8;
  comps.2.prec = 8;

  image.numcomps -= 1;
  image.color_space = OPJ_CLRSPC_SRGB;
}

pub fn color_esycc_to_rgb(image: &mut opj_image_t) {
  let mut comps = match image.comps_mut() {
    Some(&mut [y, cb, cr]) => (y, cb, cr),
    _ => return,
  };

  if comps.0.dx != comps.1.dx
    || comps.0.dx != comps.2.dx
    || comps.0.dy != comps.1.dy
    || comps.0.dy != comps.2.dy
  {
    eprintln!(
      "{}:{}: color_esycc_to_rgb\n\tCAN NOT CONVERT",
      file!(),
      line!()
    );
    return;
  }

  let max = (comps.0.w * comps.0.h) as usize;
  let flip_value = 1 << (comps.0.prec - 1);
  let max_value = (1 << comps.0.prec) - 1;

  let cb_signed = comps.1.sgnd != 0;
  let cr_signed = comps.2.sgnd != 0;

  let (y_data, cb_data, cr_data) =
    match (comps.0.data_mut(), comps.1.data_mut(), comps.2.data_mut()) {
      (Some(y), Some(cb), Some(cr)) => (y, cb, cr),
      _ => return,
    };

  for i in 0..max {
    let y = y_data[i] as f32;
    let mut cb = cb_data[i] as f32;
    let mut cr = cr_data[i] as f32;

    if !cb_signed {
      cb -= flip_value as f32;
    }
    if !cr_signed {
      cr -= flip_value as f32;
    }

    let mut val = (y - 0.0000368 * cb + 1.40199 * cr + 0.5) as i32;
    y_data[i] = val.max(0).min(max_value);

    val = ((1.0003 * y - 0.344125 * cb - 0.7141128 * cr) + 0.5) as i32;
    cb_data[i] = val.max(0).min(max_value);

    val = ((0.999823 * y + 1.77204 * cb - 0.000008 * cr) + 0.5) as i32;
    cr_data[i] = val.max(0).min(max_value);
  }

  image.color_space = OPJ_CLRSPC_SRGB;
}
