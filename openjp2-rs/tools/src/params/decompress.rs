use crate::getopt::{GetOpts, OptDef, ParsedOpt};
use crate::params::*;
use openjp2::{detect_format_from_file, openjpeg::*, J2KFormat};
use std::path::PathBuf;

// Parameters struct similar to opj_decompress_parameters
#[derive(Clone, Debug)]
pub struct DecompressParameters {
  // Core parameters
  pub core: opj_dparameters_t,
  pub index_file: Option<PathBuf>,

  pub num_threads: i32,
  pub tile_index: Option<u32>,

  // Input/output files
  pub input_file: Option<PathBuf>,
  pub output_file: Option<PathBuf>,
  pub codec_format: Option<J2KFormat>,
  pub output_format: Option<ImageFileFormat>,

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
      core: opj_dparameters_t::default(),
      num_threads: 1,
      tile_index: None,
      input_file: None,
      output_file: None,
      index_file: None,
      codec_format: None,
      output_format: None,
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
    self.core.clone()
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
      ParsedOpt::Positional(_, _) => {
        println!("Positional arguments are not supported");
        show_help = true;
      }
      ParsedOpt::ParseError(err) => {
        println!("{err}");
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
        params.codec_format = detect_format_from_file(input.to_str().unwrap()).ok();
        params.input_file = Some(input);
      }
      (DecompressOpt::Output, Some(arg)) => {
        let output = PathBuf::from(arg);
        params.output_format = ImageFileFormat::get_file_format(&output).ok();
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
      }
      (DecompressOpt::IndexFile, Some(arg)) => {
        params.index_file = Some(PathBuf::from(arg));
      }
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
  println!("\nThis is the opj_decompress utility from the OpenJPEG project.");
  println!("It decompresses JPEG 2000 codestreams to various image formats.");
  println!(
    "It has been compiled against openjp2 library v{}.\n",
    OPJ_VERSION,
  );

  println!("Parameters:");
  println!("-----------\n");
  println!("  -ImgDir <directory>");
  println!("\tImage file Directory path");
  println!("  -OutFor <PBM|PGM|PPM|PNM|PAM|PGX|PNG|BMP|TIF|TIFF|RAW|YUV|RAWL|TGA>");
  println!("    REQUIRED only if -ImgDir is used");
  println!("\tOutput format for decompressed images.");
  println!("  -i <compressed file>");
  println!("    REQUIRED only if an Input image directory is not specified");
  println!("    Currently accepts J2K-files, JP2-files and JPT-files. The file type");
  println!("    is identified based on its suffix.");
  println!("  -o <decompressed file>");
  println!("    REQUIRED");
  println!("    Currently accepts formats specified above (see OutFor option)");
  println!("    Binary data is written to the file (not ascii). If a PGX");
  println!("    filename is given, there will be as many output files as there are");
  println!("    components: an indice starting from 0 will then be appended to the");
  println!("    output filename, just before the \"pgx\" extension. If a PGM filename");
  println!("    is given and there are more than one component, only the first component");
  println!("    will be written to the file.");
  println!("  -r <reduce factor>");
  println!("    Set the number of highest resolution levels to be discarded. The");
  println!("    image resolution is effectively divided by 2 to the power of the");
  println!("    number of discarded levels. The reduce factor is limited by the");
  println!("    smallest total number of decomposition levels among tiles.");
  println!("  -l <number of quality layers to decode>");
  println!("    Set the maximum number of quality layers to decode. If there are");
  println!("    less quality layers than the specified number, all the quality layers");
  println!("    are decoded.");
  println!("  -x");
  println!("    Create an index file *.Idx (-x index_name.Idx)");
  println!("  -d <x0,y0,x1,y1>");
  println!("    OPTIONAL");
  println!("    Decoding area");
  println!("    By default all the image is decoded.");
  println!("  -t <tile_number>");
  println!("    OPTIONAL");
  println!("    Set the tile number of the decoded tile. Follow the JPEG2000 convention from left-up to bottom-up");
  println!("    By default all tiles are decoded.");
  println!("  -p <comp 0 precision>[C|S][,<comp 1 precision>[C|S][,...]]");
  println!("    OPTIONAL");
  println!("    Force the precision (bit depth) of components.");
  println!("    There shall be at least 1 value. There is no limit on the number of values (comma separated, last values ignored if too much values).");
  println!("    If there are less values than components, the last value is used for remaining components.");
  println!("    If 'C' is specified (default), values are clipped.");
  println!("    If 'S' is specified, values are scaled.");
  println!("    A 0 value can be specified (meaning original bit depth).");
  println!("  -c first_comp_index[,second_comp_index][,...]");
  println!("    OPTIONAL");
  println!("    To limit the number of components to decode.");
  println!("    Component indices are numbered starting at 0.");
  println!("  -force-rgb");
  println!("    Force output image colorspace to RGB");
  println!("  -upsample");
  println!("    Downsampled components will be upsampled to image size");
  println!("  -split-pnm");
  println!("    Split output components to different files when writing to PNM");
  println!("  -threads <num_threads|ALL_CPUS>");
  println!("    Number of threads to use for decoding or ALL_CPUS for all available cores.");
  println!("  -allow-partial");
  println!("    Disable strict mode to allow decoding partial codestreams.");
  println!("  -quiet");
  println!("    Disable output from the library and other output.");
  println!();
}
