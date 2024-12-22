use super::ImageError;
use crate::compress::{CompressionParameters, MCTMode};
use crate::params::ParameterError;
use openjp2::{image::opj_image, openjpeg::*};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

// For raw image parameters
#[derive(Clone, Debug, Default)]
pub struct RawParameters {
  pub width: u32,
  pub height: u32,
  pub num_comps: u32,
  pub bit_depth: u32,
  pub signed: bool,
  pub components: Vec<RawComponentParameters>,
}

impl FromStr for RawParameters {
  type Err = ParameterError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = s.split(&[',', '@', ':']).collect();
    if parts.len() < 5 {
      return Err(ParameterError::InvalidFormat(
        "Raw params format: width,height,ncomp,bitdepth,[s|u]@dx1,dy1:...:dxn,dyn".into(),
      ));
    }

    let width = parts[0]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid width".into()))?;
    let height = parts[1]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid height".into()))?;
    let num_comps = parts[2]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid component count".into()))?;
    let bit_depth = parts[3]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid bit depth".into()))?;
    let signed = match parts[4] {
      "s" => true,
      "u" => false,
      _ => {
        return Err(ParameterError::InvalidValue(
          "Signed flag must be 's' or 'u'".into(),
        ))
      }
    };

    let mut components = Vec::new();
    if parts.len() > 5 {
      // Parse subsampling factors
      for comp in parts[5..].iter() {
        components.push(comp.parse()?);
      }
    } else {
      // Default 1x1 subsampling for all components
      components = vec![RawComponentParameters { dx: 1, dy: 1 }; num_comps as usize];
    }

    Ok(RawParameters {
      width,
      height,
      num_comps,
      bit_depth,
      signed,
      components,
    })
  }
}

#[derive(Clone, Debug, Default)]
pub struct RawComponentParameters {
  pub dx: u32,
  pub dy: u32,
}

impl FromStr for RawComponentParameters {
  type Err = ParameterError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let factors: Vec<&str> = s.split('x').collect();
    if factors.len() != 2 {
      return Err(ParameterError::InvalidFormat(
        "Subsampling format: dx x dy".into(),
      ));
    }

    let dx = factors[0]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid dx".into()))?;
    let dy = factors[1]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid dy".into()))?;

    Ok(RawComponentParameters { dx, dy })
  }
}

pub fn load_raw_image(
  path: &Path,
  params: &CompressionParameters,
  big_endian: bool,
) -> Result<Box<opj_image>, ImageError> {
  // Get raw parameters from compression parameters
  let raw_params = params.raw_params.as_ref().ok_or_else(|| {
    ImageError::InvalidFormat("Raw parameters required for RAW/RAWL format".into())
  })?;

  // Validate parameters
  if raw_params.width == 0
    || raw_params.height == 0
    || raw_params.num_comps == 0
    || raw_params.bit_depth == 0
  {
    return Err(ImageError::InvalidFormat(
      "Invalid raw image parameters. Use -F option.".into(),
    ));
  }

  // Calculate buffer size needed for one component
  let bytes_per_sample = match raw_params.bit_depth {
    bd if bd <= 8 => 1,
    bd if bd <= 16 => 2,
    bd if bd <= 32 => 4,
    _ => {
      return Err(ImageError::InvalidFormat(
        "Bit depth > 32 not supported".into(),
      ))
    }
  };

  let num_pixels = (raw_params.width * raw_params.height) as usize;
  let buffer_size = num_pixels * bytes_per_sample;

  // Create image and initialize components
  let mut image = opj_image::new();
  image.color_space = if raw_params.num_comps == 1 {
    OPJ_CLRSPC_GRAY
  } else if raw_params.num_comps >= 3 {
    match &params.mct_mode {
      Some(MCTMode::None) => OPJ_CLRSPC_SYCC,
      None | Some(MCTMode::RGB2YCC) => OPJ_CLRSPC_SRGB,
      _ => OPJ_CLRSPC_UNKNOWN,
    }
  } else {
    OPJ_CLRSPC_UNKNOWN
  };

  // Set image parameters
  let offset = params.image_offset();
  let subsampling = params.subsampling();

  image.x0 = offset.x;
  image.y0 = offset.y;
  image.x1 = offset.x + (raw_params.width - 1) * subsampling.width + 1;
  image.y1 = offset.y + (raw_params.height - 1) * subsampling.height + 1;

  // Allocate components
  image.alloc_comps(raw_params.num_comps);
  let comps = image.comps_mut().expect("We just allocated the components");

  // Detect alpha channel.
  let alpha = match raw_params.num_comps {
    2 => Some(1),
    4 => Some(3),
    _ => None,
  };

  // Initialize components
  for (i, comp) in comps.iter_mut().enumerate() {
    let raw_comp = raw_params
      .components
      .get(i)
      .unwrap_or(&RawComponentParameters { dx: 1, dy: 1 });

    comp.dx = subsampling.width * raw_comp.dx;
    comp.dy = subsampling.height * raw_comp.dy;
    comp.w = raw_params.width;
    comp.h = raw_params.height;
    comp.x0 = 0;
    comp.y0 = 0;
    comp.prec = raw_params.bit_depth;
    comp.sgnd = raw_params.signed as u32;
    comp.alpha = (Some(i as u32) == alpha) as u16;

    if !comp.alloc_data() {
      return Err(ImageError::InvalidFormat(
        "Failed to allocate component data".into(),
      ));
    }
  }

  // Allocate single reusable buffer for reading component data
  let mut buffer = vec![0u8; buffer_size];
  let mut file = File::open(path)?;

  // Read and process one component at a time
  for comp in comps.iter_mut() {
    // Read data for this component
    file.read_exact(&mut buffer)?;

    let data = comp.data_mut().expect("We just allocated it");

    match raw_params.bit_depth {
      bd if bd <= 8 => {
        for (dst, &src) in data.iter_mut().zip(buffer.iter()) {
          *dst = if raw_params.signed {
            src as i8 as i32
          } else {
            src as i32
          };
        }
      }
      bd if bd <= 16 => {
        let from_bytes = if big_endian {
          u16::from_be_bytes
        } else {
          u16::from_le_bytes
        };
        for (dst, bytes) in data.iter_mut().zip(buffer.chunks_exact(2)) {
          let value = from_bytes([bytes[0], bytes[1]]);
          *dst = if raw_params.signed {
            value as i16 as i32
          } else {
            value as i32
          };
        }
      }
      bd if bd <= 32 => {
        let from_bytes = if big_endian {
          u32::from_be_bytes
        } else {
          u32::from_le_bytes
        };
        for (dst, bytes) in data.iter_mut().zip(buffer.chunks_exact(4)) {
          let value = from_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
          *dst = value as i32;
        }
      }
      _ => unreachable!(),
    }
  }

  Ok(image)
}
