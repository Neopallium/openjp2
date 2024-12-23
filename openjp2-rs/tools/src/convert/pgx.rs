use super::ImageError;
use crate::params::CompressionParameters;
use openjp2::image::opj_image;
use openjp2::openjpeg::*;
use std::fs::File;
use std::io::{BufRead, BufWriter, Read, Write};
use std::path::Path;

pub fn load_pgx_image(
  path: &Path,
  params: &CompressionParameters,
) -> Result<Box<opj_image>, ImageError> {
  let mut file = File::open(path)?;
  let mut header = String::new();
  std::io::BufReader::new(&file)
    .read_line(&mut header)
    .map_err(|e| ImageError::InvalidFormat(format!("Failed to read PGX header: {}", e)))?;

  // Parse PGX header format: "PG <endian> <+/-> <precision> <width> <height>"
  let parts: Vec<&str> = header.trim().split_whitespace().collect();
  if parts.len() != 6 || parts[0] != "PG" || parts[1] != "ML" {
    return Err(ImageError::InvalidFormat(
      "Invalid PGX header format".into(),
    ));
  }

  let signed = parts[2] == "-";
  let precision: u32 = parts[3]
    .parse()
    .map_err(|_| ImageError::InvalidFormat("Invalid precision value".into()))?;
  let width: u32 = parts[4]
    .parse()
    .map_err(|_| ImageError::InvalidFormat("Invalid width value".into()))?;
  let height: u32 = parts[5]
    .parse()
    .map_err(|_| ImageError::InvalidFormat("Invalid height value".into()))?;

  if width < 1 || height < 1 || precision < 1 || precision > 31 {
    return Err(ImageError::InvalidFormat(
      "Invalid PGX dimensions or precision".into(),
    ));
  }

  // Validate file size
  let expected_data_size = if precision <= 8 {
    width * height
  } else if precision <= 16 {
    width * height * 2
  } else {
    width * height * 4
  };

  let file_size = file
    .metadata()
    .map_err(|e| ImageError::InvalidFormat(format!("Failed to get file size: {}", e)))?
    .len();

  if file_size < expected_data_size as u64 + header.len() as u64 {
    return Err(ImageError::InvalidFormat("File too small".into()));
  }

  // Create image
  let mut image = opj_image::new();
  image.color_space = OPJ_CLRSPC_GRAY;

  // Set image parameters
  let offset = params.image_offset();
  let subsampling = params.subsampling();

  // Set dimensions
  image.x0 = offset.x;
  image.y0 = offset.y;
  let x1 = offset.x + (width - 1) * subsampling.width + 1;
  let y1 = offset.y + (height - 1) * subsampling.height + 1;
  image.x1 = x1;
  image.y1 = y1;

  // Initialize single component
  let data = {
    image.alloc_comps(1);
    let comp = &mut (image.comps_mut().expect("Component allocation failed")[0]);

    comp.dx = subsampling.width;
    comp.dy = subsampling.height;
    comp.w = x1;
    comp.h = y1;
    comp.x0 = offset.x;
    comp.y0 = offset.y;
    comp.prec = precision;
    comp.sgnd = signed as u32;

    if !comp.alloc_data() {
      return Err(ImageError::InvalidFormat(
        "Failed to allocate component data".into(),
      ));
    }
    comp.data_mut().expect("Data allocation failed")
  };

  // Read pixel data based on precision
  if precision <= 8 {
    let mut buffer = vec![0u8; (width * height) as usize];
    file.read_exact(&mut buffer)?;

    for (i, &value) in buffer.iter().enumerate() {
      data[i] = if signed {
        value as i8 as i32
      } else {
        value as i32
      };
    }
  } else if precision <= 16 {
    let mut buffer = vec![0u8; (width * height * 2) as usize];
    file.read_exact(&mut buffer)?;

    for (i, chunk) in buffer.chunks_exact(2).enumerate() {
      let value = u16::from_be_bytes([chunk[0], chunk[1]]);
      data[i] = if signed {
        value as i16 as i32
      } else {
        value as i32
      };
    }
  } else {
    let mut buffer = vec![0u8; (width * height * 4) as usize];
    file.read_exact(&mut buffer)?;

    for (i, chunk) in buffer.chunks_exact(4).enumerate() {
      let value = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
      data[i] = value as i32;
    }
  }

  Ok(image)
}

pub fn save_pgx_image(image: &opj_image, path: &Path) -> Result<(), ImageError> {
  // For multi-component images we need to save each component as a separate file
  let need_suffix = image.numcomps > 1;
  let stem = path
    .file_stem()
    .and_then(|s| s.to_str())
    .ok_or_else(|| ImageError::InvalidFormat("Invalid path".into()))?;
  let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("pgx");

  for (comp_idx, comp_data) in image
    .comps_data_iter()
    .ok_or_else(|| ImageError::InvalidFormat("Missing components".into()))?
    .enumerate()
  {
    // Create filename with component suffix if needed
    let comp_path = if need_suffix {
      path.with_file_name(format!("{}_{}.{}", stem, comp_idx, ext))
    } else {
      path.to_path_buf()
    };

    let file = std::fs::File::create(&comp_path)?;
    let mut writer = BufWriter::new(file);

    // Write PGX header
    let sign_char = if comp_data.comp.sgnd != 0 { '-' } else { '+' };
    write!(
      writer,
      "PG ML {} {} {} {}\n",
      sign_char, comp_data.comp.prec, comp_data.comp.w, comp_data.comp.h
    )?;

    let precision = comp_data.comp.prec;
    let signed = comp_data.comp.sgnd != 0;
    let adjust = comp_data.adjust;

    if precision <= 8 {
      // Write 8-bit values
      let (min, max) = if signed { (-128, 127) } else { (0, 255) };

      for &value in comp_data.data.iter() {
        let value = (value + adjust).clamp(min, max) as u8;
        writer.write_all(&[value])?;
      }
    } else if precision <= 16 {
      // Write 16-bit values in big-endian order
      let (min, max) = if signed { (-32768, 32767) } else { (0, 65535) };

      for &value in comp_data.data.iter() {
        let value = (value + adjust).clamp(min, max) as u16;
        writer.write_all(&value.to_be_bytes())?;
      }
    } else {
      return Err(ImageError::InvalidFormat(
        "PGX format only supports up to 16 bits per component".into(),
      ));
    }

    // Ensure all data is written
    writer.flush()?;
  }

  Ok(())
}
