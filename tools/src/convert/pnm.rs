use super::ImageError;
use crate::params::CompressionParameters;
use openjp2::{ColorSpace, CoreImage, CoreImageComp};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Whitespace-delimited token with `#`-comment skipping (the PNM header
/// grammar: tokens separated by any whitespace, comments to end-of-line).
fn next_token(reader: &mut BufReader<std::fs::File>) -> Result<Option<String>, ImageError> {
  let mut out: Vec<u8> = Vec::new();
  loop {
    let mut b = [0u8; 1];
    match reader
      .read(&mut b)
      .map_err(|e| ImageError::InvalidFormat(format!("PNM header read failed: {e}")))?
    {
      0 => {
        return if out.is_empty() {
          Ok(None)
        } else {
          Ok(Some(String::from_utf8_lossy(&out).into_owned()))
        }
      }
      _ => match b[0] {
        b'#' => {
          let mut line = Vec::new();
          let _ = reader.read_until(b'\n', &mut line);
          if !out.is_empty() {
            return Ok(Some(String::from_utf8_lossy(&out).into_owned()));
          }
        }
        b if b.is_ascii_whitespace() => {
          if !out.is_empty() {
            return Ok(Some(String::from_utf8_lossy(&out).into_owned()));
          }
        }
        b => out.push(b),
      },
    }
  }
}

/// Reads a binary or ASCII PPM/PGM (P1..P6) directly into a `CoreImage`
/// without the image-crate intermediate (the C tools' `ppmtoimage`
/// equivalent): one component for PGM, three for PPM.  The raster is
/// written straight into the component planes.
pub fn load_pnm_image(
  path: &Path,
  params: &CompressionParameters,
) -> Result<CoreImage, ImageError> {
  let file = std::fs::File::open(path).map_err(|e| ImageError::ReadError(e.to_string()))?;
  let mut reader = BufReader::new(file);
  let read_num = |reader: &mut BufReader<std::fs::File>| -> Result<u32, ImageError> {
    let t = next_token(reader)?
      .ok_or_else(|| ImageError::InvalidFormat("PNM header truncated".into()))?;
    t.parse::<u32>()
      .map_err(|_| ImageError::InvalidFormat(format!("Invalid PNM value: {t}")))
  };
  let magic =
    next_token(&mut reader)?.ok_or_else(|| ImageError::InvalidFormat("Empty PNM file".into()))?;
  /* P7 (PAM): named key/value header fields terminated by ENDHDR. */
  let (width, height, maxval, num_comps, is_ascii) = if magic == "P7" {
    let mut width = 0u32;
    let mut height = 0u32;
    let mut depth = 0u32;
    let mut maxval = 0u32;
    loop {
      let key = next_token(&mut reader)?
        .ok_or_else(|| ImageError::InvalidFormat("PAM header truncated".into()))?;
      match key.as_str() {
        "ENDHDR" => break,
        "WIDTH" => width = read_num(&mut reader)?,
        "HEIGHT" => height = read_num(&mut reader)?,
        "DEPTH" => depth = read_num(&mut reader)?,
        "MAXVAL" => maxval = read_num(&mut reader)?,
        _ => {
          /* TUPLTYPE and unknown fields: skip the value token */
          next_token(&mut reader)?;
        }
      }
    }
    if width == 0 || height == 0 || depth == 0 || maxval == 0 {
      return Err(ImageError::InvalidFormat("Invalid PAM header".into()));
    }
    (width, height, maxval, depth, false)
  } else {
    let width = read_num(&mut reader)?;
    let height = read_num(&mut reader)?;
    let maxval = if matches!(magic.as_str(), "P1" | "P4") {
      1
    } else {
      read_num(&mut reader)?
    };
    let num_comps = match magic.as_str() {
      "P1" | "P2" | "P4" | "P5" => 1,
      "P3" | "P6" => 3,
      other => {
        return Err(ImageError::InvalidFormat(format!(
          "Unknown PNM magic {other}"
        )));
      }
    };
    (
      width,
      height,
      maxval,
      num_comps,
      matches!(magic.as_str(), "P1" | "P2" | "P3"),
    )
  };
  let bytes_per_sample = if maxval > 255 { 2 } else { 1 };
  let num_samples = (width as usize) * (height as usize);

  /* Match the C tools' ppmtoimage: the image origin and the component
  subsampling factors come from -d and -s (the PNM raster itself is always
  full resolution). */
  let offset = params.image_offset();
  let subsampling = params.subsampling();
  let mut comps: Vec<CoreImageComp> = (0..num_comps)
    .map(|_| CoreImageComp {
      dx: subsampling.width,
      dy: subsampling.height,
      w: width,
      h: height,
      x0: 0,
      y0: 0,
      prec: if maxval > 255 { 16 } else { 8 },
      bpp: if maxval > 255 { 16 } else { 8 },
      sgnd: 0,
      resno_decoded: 0,
      factor: 0,
      alpha: 0,
      data: Vec::with_capacity(num_samples),
    })
    .collect();

  if is_ascii {
    let mut text = String::new();
    reader
      .read_to_string(&mut text)
      .map_err(|e| ImageError::InvalidFormat(format!("PNM data read failed: {e}")))?;
    let mut vals = text.split_whitespace();
    for _ in 0..num_samples {
      for comp in comps.iter_mut() {
        let v: i32 = vals
          .next()
          .ok_or_else(|| ImageError::InvalidFormat("PNM data too short".into()))?
          .parse()
          .map_err(|_| ImageError::InvalidFormat("Invalid PNM sample".into()))?;
        comp.data.push(v);
      }
    }
  } else {
    /* Stream the raster straight into the component planes (no separate
    raster buffer - the C tools' pnmtoimage reads the file directly). */
    let mut reader = reader;
    let mut sample_iter = std::iter::from_fn(move || {
      let mut b = [0u8; 1];
      if bytes_per_sample == 1 {
        match reader.read(&mut b) {
          Ok(0) | Err(_) => None,
          Ok(_) => Some(u32::from(b[0])),
        }
      } else {
        match reader.read(&mut b) {
          Ok(0) | Err(_) => None,
          Ok(_) => {
            let hi = b[0];
            match reader.read(&mut b) {
              Ok(0) | Err(_) => None,
              Ok(_) => Some((u32::from(hi) << 8) | u32::from(b[0])),
            }
          }
        }
      }
    });
    for _ in 0..num_samples {
      for comp in comps.iter_mut() {
        let v = sample_iter
          .next()
          .ok_or_else(|| ImageError::InvalidFormat("PNM data too short".into()))?
          as i32;
        comp.data.push(v);
      }
    }
  }
  Ok(CoreImage {
    x0: offset.x,
    y0: offset.y,
    x1: offset.x + (width - 1) * subsampling.width + 1,
    y1: offset.y + (height - 1) * subsampling.height + 1,
    numcomps: num_comps,
    color_space: if num_comps == 1 {
      ColorSpace::Gray
    } else {
      ColorSpace::Srgb
    },
    comps,
    icc_profile_buf: Vec::new(),
    icc_profile_is_cielab: false,
  })
}
