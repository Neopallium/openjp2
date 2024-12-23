use crate::getopt::{GetOpts, OptDef, ParsedOpt};
use crate::params::*;
use openjp2::openjpeg::*;
use std::path::PathBuf;

// Parameters struct similar to opj_decompress_parameters
#[derive(Clone, Debug)]
pub struct DecompressParameters {
  // Core parameters
  pub num_threads: i32,
  pub tile_index: Option<u32>,
  pub nb_tile_to_decode: u32,
  pub core: CoreParameters,

  // Input/output files
  pub input_file: Option<PathBuf>,
  pub output_file: Option<PathBuf>,
  pub codec_format: Option<CodecFormat>,
  pub decode_format: Option<ImageFileFormat>,

  // Decoding area parameters
  pub da_x0: u32,
  pub da_y0: u32,
  pub da_x1: u32,
  pub da_y1: u32,

  // Misc options
  pub force_rgb: bool,
  pub upsample: bool,
  pub split_pnm: bool,
  pub quiet: bool,
  pub allow_partial: bool,

  // Component parameters
  pub numcomps: u32,
  pub comps_indices: Vec<u32>,

  // Precision parameters
  pub precision: Vec<PrecisionParameter>,
}

// Core parameters
#[derive(Clone, Debug)]
pub struct CoreParameters {
  pub cp_reduce: u32,
  pub cp_layer: u32,
  pub indexfilename: Option<String>,
}

// Precision mode enum
#[derive(Clone, Debug)]
pub enum PrecisionMode {
  Clip,
  Scale,
}

// Precision parameter
#[derive(Clone, Debug)]
pub struct PrecisionParameter {
  pub prec: u32,
  pub mode: PrecisionMode,
}

// CLI options enum
#[derive(Debug, Clone, PartialEq)]
enum DecompressOpt {
  Input,
  Output,
  Help,
  ImgDir,
  OutFormat,
  Reduce,
  Layer,
  Threads,
  DecodingArea,
  TileIndex,
  IndexFile,
  Precision,
  Components,
  ForceRGB,
  Upsample,
  SplitPNM,
  Quiet,
  AllowPartial,
}

impl Default for DecompressParameters {
  fn default() -> Self {
    Self {
      num_threads: 1,
      tile_index: None,
      nb_tile_to_decode: 0,
      core: CoreParameters {
        cp_reduce: 0,
        cp_layer: 0,
        indexfilename: None,
      },
      input_file: None,
      output_file: None,
      codec_format: None,
      decode_format: None,
      da_x0: 0,
      da_y0: 0,
      da_x1: 0,
      da_y1: 0,
      force_rgb: false,
      upsample: false,
      split_pnm: false,
      quiet: false,
      allow_partial: false,
      numcomps: 0,
      comps_indices: Vec::new(),
      precision: Vec::new(),
    }
  }
}

// Implement option parsing
impl DecompressParameters {
  fn parse_precision(&mut self, arg: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = arg.split(',').collect();
    for part in parts {
      let mut mode = PrecisionMode::Clip;
      let mut prec_str = part;

      if let Some(c) = part.chars().last() {
        match c {
          'C' => {
            prec_str = &part[..part.len() - 1];
            mode = PrecisionMode::Clip;
          }
          'S' => {
            prec_str = &part[..part.len() - 1];
            mode = PrecisionMode::Scale;
          }
          _ => {}
        }
      }

      let prec: u32 = prec_str.parse()?;
      if prec < 1 || prec > 32 {
        return Err("Precision must be between 1 and 32".into());
      }

      self.precision.push(PrecisionParameter { prec, mode });
    }
    Ok(())
  }

  fn parse_components(&mut self, arg: &str) -> Result<(), Box<dyn std::error::Error>> {
    for comp_str in arg.split(',') {
      let comp: u32 = comp_str.parse()?;
      self.comps_indices.push(comp);
      self.numcomps += 1;
    }
    Ok(())
  }

  fn parse_decoding_area(&mut self, arg: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = arg.split(',').collect();
    if parts.len() != 4 {
      return Err("Decoding area requires 4 values: x0,y0,x1,y1".into());
    }

    self.da_x0 = parts[0].parse()?;
    self.da_y0 = parts[1].parse()?;
    self.da_x1 = parts[2].parse()?;
    self.da_y1 = parts[3].parse()?;

    Ok(())
  }

  pub fn to_c_params(&self) -> opj_dparameters_t {
    let mut params = opj_dparameters_t::default();

    // Core parameters
    params.cp_reduce = self.core.cp_reduce;
    params.cp_layer = self.core.cp_layer;

    // Set decoding parameters
    if let Some(ref fmt) = self.decode_format {
      params.decod_format = match fmt {
        ImageFileFormat::PGX => 0,
        ImageFileFormat::PXM => 1,
        ImageFileFormat::BMP => 2,
        ImageFileFormat::TIF => 3,
        ImageFileFormat::RAW => 4,
        ImageFileFormat::RAWL => 5,
        ImageFileFormat::TGA => 6,
        ImageFileFormat::PNG => 7,
        _ => -1,
      };
    }

    // Set codec format
    if let Some(ref fmt) = self.codec_format {
      params.cod_format = match fmt {
        CodecFormat::J2K => OPJ_CODEC_J2K as i32,
        CodecFormat::JP2 => OPJ_CODEC_JP2 as i32,
        CodecFormat::JPT => OPJ_CODEC_JPT as i32,
        CodecFormat::JPP => OPJ_CODEC_JPP as i32,
        CodecFormat::JPX => OPJ_CODEC_JPX as i32,
      };
    }

    // Set decoding area
    params.DA_x0 = self.da_x0;
    params.DA_y0 = self.da_y0;
    params.DA_x1 = self.da_x1;
    params.DA_y1 = self.da_y1;

    // Set tile parameters
    if let Some(idx) = self.tile_index {
      params.tile_index = idx;
      params.nb_tile_to_decode = self.nb_tile_to_decode;
    }

    // Set other flags
    params.m_verbose = (!self.quiet) as i32;
    params.flags = if self.force_rgb { 1 } else { 0 }
      | if self.upsample { 2 } else { 0 }
      | if self.split_pnm { 4 } else { 0 }
      | if self.allow_partial { 8 } else { 0 };

    params
  }
}

fn validate_args(args: Vec<String>) -> Option<Vec<(DecompressOpt, Option<String>)>> {
  let parser = GetOpts::new(&[
    OptDef::short('i', DecompressOpt::Input, true),
    OptDef::short('o', DecompressOpt::Output, true),
    OptDef::short('r', DecompressOpt::Reduce, true),
    OptDef::short('l', DecompressOpt::Layer, true),
    OptDef::short('x', DecompressOpt::IndexFile, true),
    OptDef::short('d', DecompressOpt::DecodingArea, true),
    OptDef::short('t', DecompressOpt::TileIndex, true),
    OptDef::short('p', DecompressOpt::Precision, true),
    OptDef::short('c', DecompressOpt::Components, true),
    OptDef::short('h', DecompressOpt::Help, false),
    OptDef::long("ImgDir", DecompressOpt::ImgDir, true),
    OptDef::long("OutFor", DecompressOpt::OutFormat, true),
    OptDef::long("force-rgb", DecompressOpt::ForceRGB, false),
    OptDef::long("upsample", DecompressOpt::Upsample, false),
    OptDef::long("split-pnm", DecompressOpt::SplitPNM, false),
    OptDef::long("threads", DecompressOpt::Threads, true),
    OptDef::long("quiet", DecompressOpt::Quiet, false),
    OptDef::long("allow-partial", DecompressOpt::AllowPartial, false),
  ]);

  let args = parser.parse_args(args);
  let mut valid_args = Vec::new();
  let mut show_help = false;
  for arg in args {
    match arg {
      ParsedOpt::Program(_) => (),
      ParsedOpt::Opt(DecompressOpt::Help, _) => show_help = true,
      ParsedOpt::Opt(opt, arg) => valid_args.push((opt, arg)),
      ParsedOpt::InvalidOpt(invalid) => {
        println!("Invalid option: {}", invalid);
        show_help = true;
      }
      ParsedOpt::MissingArgument(opt, _) => {
        println!("Missing argument for option: {:?}", opt);
        show_help = true;
      }
    }
  }
  if show_help {
    decode_help_display();
    return None;
  }
  Some(valid_args)
}

pub fn parse_decompress_options(
  args: Vec<String>,
) -> Result<Option<(DecompressParameters, ImageFolder)>, Box<dyn std::error::Error>> {
  let mut params = DecompressParameters::default();

  let args = match validate_args(args) {
    Some(args) => args,
    None => return Ok(None),
  };

  let mut img_folder = ImageFolder {
    img_dir_path: None,
    out_format: None,
    set_img_dir: false,
    set_out_format: false,
  };

  for arg in args {
    match arg {
      (DecompressOpt::Input, Some(arg)) => {
        let input = PathBuf::from(arg);
        params.codec_format = CodecFormat::get_file_format(input.to_str().unwrap()).ok();
        params.input_file = Some(input);
      }
      (DecompressOpt::Output, Some(arg)) => {
        let output = PathBuf::from(arg);
        params.decode_format = ImageFileFormat::get_file_format(&output).ok();
        params.output_file = Some(output);
      }
      (DecompressOpt::ImgDir, Some(arg)) => {
        img_folder.img_dir_path = Some(PathBuf::from(arg));
        img_folder.set_img_dir = true;
      }
      (DecompressOpt::OutFormat, Some(arg)) => {
        img_folder.out_format = Some(arg);
        img_folder.set_out_format = true;
      }
      (DecompressOpt::Reduce, Some(arg)) => params.core.cp_reduce = arg.parse()?,
      (DecompressOpt::Layer, Some(arg)) => params.core.cp_layer = arg.parse()?,
      (DecompressOpt::Threads, Some(arg)) => {
        if arg == "ALL_CPUS" {
          // TODO: Use num_cpus crate
          params.num_threads = 4; //num_cpus::get() as i32;
          if params.num_threads == 1 {
            params.num_threads = 0;
          }
        } else {
          params.num_threads = arg.parse()?;
        }
      }
      (DecompressOpt::DecodingArea, Some(arg)) => params.parse_decoding_area(&arg)?,
      (DecompressOpt::TileIndex, Some(arg)) => {
        params.tile_index = Some(arg.parse()?);
        params.nb_tile_to_decode = 1;
      }
      (DecompressOpt::IndexFile, Some(arg)) => params.core.indexfilename = Some(arg),
      (DecompressOpt::Precision, Some(arg)) => params.parse_precision(&arg)?,
      (DecompressOpt::Components, Some(arg)) => params.parse_components(&arg)?,
      (DecompressOpt::ForceRGB, _) => params.force_rgb = true,
      (DecompressOpt::Upsample, _) => params.upsample = true,
      (DecompressOpt::SplitPNM, _) => params.split_pnm = true,
      (DecompressOpt::Quiet, _) => params.quiet = true,
      (DecompressOpt::AllowPartial, _) => params.allow_partial = true,
      (DecompressOpt::Help, _) => {
        decode_help_display();
        return Ok(None);
      }
      (opt, None) => return Err(format!("Missing argument for option: {:?}", opt).into()),
    }
  }

  // Validate parameters
  if img_folder.set_img_dir {
    if params.input_file.is_some() {
      return Err("Cannot use -ImgDir with -i".into());
    }
    if !img_folder.set_out_format {
      return Err("Must specify -OutFor when using -ImgDir".into());
    }
  } else if params.input_file.is_none() || params.output_file.is_none() {
    return Err("Must specify input (-i) and output (-o) files".into());
  }

  Ok(Some((params, img_folder)))
}

fn decode_help_display() {
  println!("\nThis is the opj_decompress utility from the OpenJPEG project.\n");
  println!("It decompresses JPEG 2000 codestreams to various image formats.\n");
  println!("Parameters:\n");
  println!("  -i <file>");
  println!("    Input file");
  println!("  -o <file>");
  println!("    Output file (PGX, PNM, BMP, TIF, RAW, RAWL, TGA, PNG)");
  println!("  -r <reduce>");
  println!("    Number of highest resolution levels to discard");
  println!("  -l <layers>");
  println!("    Maximum number of quality layers to decode");
  // ... Add more help text as needed ...
}
