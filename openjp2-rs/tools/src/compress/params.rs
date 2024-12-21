use crate::convert::*;
use crate::getopt::{GetOpts, OptDef, ParsedOpt};
use crate::params::*;
use openjp2::openjpeg::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;

// New struct to hold parsed CLI options
#[derive(Clone, Debug)]
pub struct CLIOptions {
  pub compression_params: CompressionParameters,
  pub img_folder: ImageFolder,
}

#[derive(Debug, Clone, PartialEq)]
enum CompressOpt {
  Input,
  Output,
  Help,
  ImgDir,
  OutFormat,
  Threads,
  NumResolutions,
  CompressionRatio,
  ProgressionOrder,
  TileSize,
  Irreversible,
  GuardBits,
  TargetBitDepth,
  MCT,
  MCTData,
  ROI,
  Quality,
  SOP,
  EPH,
  PLT,
  TLM,
  ModeSwitch,
  POC,
  Cinema2K,
  Cinema4K,
  IMF,
  JPIP,
  CodeBlockSize,
  PrecinctSize,
  TileParts,
  RawFormat,
  Comment,
  SubsamplingFactor,
  IndexFile,
  ImageOffset,
  TileOffset,
  FixedLayer,
}

fn encode_help_display() {
  println!("\nThis is the opj_compress utility from the OpenJPEG project.");
  println!("It compresses various image formats with the JPEG 2000 algorithm.");
  println!(
    "It has been compiled against openjp2 library v{}.\n",
    OPJ_VERSION,
  );

  println!("Default encoding options:");
  println!("-------------------------");
  println!("");
  println!(" * Lossless");
  println!(" * 1 tile");
  println!(" * RGB->YCC conversion if at least 3 components");
  println!(" * Size of precinct : 2^15 x 2^15 (means 1 precinct)");
  println!(" * Size of code-block : 64 x 64");
  println!(" * Number of resolutions: 6");
  println!(" * No SOP marker in the codestream");
  println!(" * No EPH marker in the codestream");
  println!(" * No sub-sampling in x or y direction");
  println!(" * No mode switch activated");
  println!(" * Progression order: LRCP");
  println!(" * No ROI upshifted");
  println!(" * No offset of the origin of the image");
  println!(" * No offset of the origin of the tiles");
  println!(" * Reversible DWT 5-3");
  println!("");

  println!("Note:");
  println!("-----");
  println!("");
  println!("The markers written to the main_header are : SOC SIZ COD QCD COM.");
  println!("COD and QCD never appear in the tile_header.");
  println!("");

  println!("Parameters:");
  println!("-----------");
  println!("");
  println!("Required Parameters (except with -h):");
  println!("One of the two options -ImgDir or -i must be used");
  println!("");
  println!("-i <file>");
  println!("    Input file");
  println!("    Known extensions are <PBM|PGM|PPM|PNM|PAM|PGX|PNG|BMP|TIF|TIFF|RAW|YUV|RAWL|TGA>");
  println!("    If used, '-o <file>' must be provided");
  println!("-o <compressed file>");
  println!("    Output file (accepted extensions are j2k or jp2).");
  println!("-ImgDir <dir>");
  println!("    Image file Directory path (example ../Images) ");
  println!("    When using this option -OutFor must be used");
  println!("-OutFor <J2K|J2C|JP2>");
  println!("    Output format for compressed files.");
  println!("    Required only if -ImgDir is used");
  println!("-F <width>,<height>,<ncomp>,<bitdepth>,{{s,u}}@<dx1>x<dy1>:...:<dxn>x<dyn>");
  println!("    Characteristics of the raw or yuv input image");
  println!("    If subsampling is omitted, 1x1 is assumed for all components");
  println!("     Example: -F 512,512,3,8,u@1x1:2x2:2x2");
  println!("              for raw or yuv 512x512 size with 4:2:0 subsampling");
  println!("    Required only if RAW or RAWL input file is provided.");
  println!("");
  println!("Optional Parameters:");
  println!("");
  println!("-h");
  println!("    Display the help information.");
  println!("-r <compression ratio>,<compression ratio>,...");
  println!("    Different compression ratios for successive layers.");
  println!("    The rate specified for each quality level is the desired");
  println!("    compression factor (use 1 for lossless)");
  println!("    Decreasing ratios required.");
  println!("      Example: -r 20,10,1 means ");
  println!("            quality layer 1: compress 20x, ");
  println!("            quality layer 2: compress 10x ");
  println!("            quality layer 3: compress lossless");
  println!("    Options -r and -q cannot be used together.");
  println!("-q <psnr value>,<psnr value>,<psnr value>,...");
  println!("    Different psnr for successive layers (-q 30,40,50).");
  println!("    Increasing PSNR values required, except 0 which can");
  println!("    be used for the last layer to indicate it is lossless.");
  println!("    Options -r and -q cannot be used together.");
  println!("-n <number of resolutions>");
  println!("    Number of resolutions.");
  println!("    It corresponds to the number of DWT decompositions +1. ");
  println!("    Default: 6.");
  println!("-TargetBitDepth <target bit depth>");
  println!("    Target bit depth.");
  println!("    Number of bits per component to use from input image");
  println!("    if all bits are unwanted.");
  println!("    (Currently only implemented for TIF.)");
  println!("-b <cblk width>,<cblk height>");
  println!("    Code-block size. The dimension must respect the constraint ");
  println!("    defined in the JPEG-2000 standard (no dimension smaller than 4 ");
  println!("    or greater than 1024, no code-block with more than 4096 coefficients).");
  println!("    The maximum value authorized is 64x64. ");
  println!("    Default: 64x64.");
  println!("-c [<prec width>,<prec height>],[<prec width>,<prec height>],...");
  println!("    Precinct size. Values specified must be power of 2. ");
  println!("    Multiple records may be supplied, in which case the first record refers");
  println!("    to the highest resolution level and subsequent records to lower ");
  println!("    resolution levels. The last specified record is halved successively for each ");
  println!("    remaining lower resolution levels.");
  println!("    Default: 2^15x2^15 at each resolution.");
  println!("-t <tile width>,<tile height>");
  println!("    Tile size.");
  println!("    Default: the dimension of the whole image, thus only one tile.");
  println!("-p <LRCP|RLCP|RPCL|PCRL|CPRL>");
  println!("    Progression order.");
  println!("    Default: LRCP.");
  println!("-s  <subX,subY>");
  println!("    Subsampling factor.");
  println!("    Subsampling bigger than 2 can produce error");
  println!("    Default: no subsampling.");
  println!("-POC <progression order change>/<progression order change>/...");
  println!("    Progression order change.");
  println!("    The syntax of a progression order change is the following:");
  println!("    T<tile>=<resStart>,<compStart>,<layerEnd>,<resEnd>,<compEnd>,<progOrder>");
  println!("      Example: -POC T1=0,0,1,5,3,CPRL/T1=5,0,1,6,3,CPRL");
  println!("-SOP");
  println!("    Write SOP marker before each packet.");
  println!("-EPH");
  println!("    Write EPH marker after each header packet.");
  println!("-PLT");
  println!("    Write PLT marker in tile-part header.");
  println!("-TLM");
  println!("    Write TLM marker in main header.");
  println!("-M <key value>");
  println!("    Mode switch.");
  println!("    [1=BYPASS(LAZY) 2=RESET 4=RESTART(TERMALL)");
  println!("    8=VSC 16=ERTERM(SEGTERM) 32=SEGMARK(SEGSYM)]");
  println!("    Indicate multiple modes by adding their values.");
  println!("      Example: RESTART(4) + RESET(2) + SEGMARK(32) => -M 38");
  println!("-TP <R|L|C>");
  println!("    Divide packets of every tile into tile-parts.");
  println!("    Division is made by grouping Resolutions (R), Layers (L)");
  println!("    or Components (C).");
  println!("-ROI c=<component index>,U=<upshifting value>");
  println!("    Quantization indices upshifted for a component. ");
  println!("    Warning: This option does not implement the usual ROI (Region of Interest).");
  println!("    It should be understood as a 'Component of Interest'. It offers the ");
  println!("    possibility to upshift the value of a component during quantization step.");
  println!("    The value after c= is the component number [0, 1, 2, ...] and the value ");
  println!("    after U= is the value of upshifting. U must be in the range [0, 37].");
  println!("-d <image offset X,image offset Y>");
  println!("    Offset of the origin of the image.");
  println!("-T <tile offset X,tile offset Y>");
  println!("    Offset of the origin of the tiles.");
  println!("-I");
  println!("    Use the irreversible DWT 9-7.");
  println!("-mct <0|1|2>");
  println!("    Explicitly specifies if a Multiple Component Transform has to be used.");
  println!("    0: no MCT ; 1: RGB->YCC conversion ; 2: custom MCT.");
  println!("    If custom MCT, \"-m\" option has to be used (see hereunder).");
  println!("    By default, RGB->YCC conversion is used if there are 3 components or more,");
  println!("    no conversion otherwise.");
  println!("-m <file>");
  println!("    Use array-based MCT, values are coma separated, line by line");
  println!("    No specific separators between lines, no space allowed between values.");
  println!("    If this option is used, it automatically sets \"-mct\" option to 2.");
  println!("-cinema2K <24|48>");
  println!("    Digital Cinema 2K profile compliant codestream.");
  println!("	Need to specify the frames per second for a 2K resolution.");
  println!("    Only 24 or 48 fps are currently allowed.");
  println!("-cinema4K");
  println!("    Digital Cinema 4K profile compliant codestream.");
  println!("	Frames per second not required. Default value is 24fps.");
  println!("-IMF <PROFILE>[,mainlevel=X][,sublevel=Y][,framerate=FPS]");
  println!("    Interoperable Master Format compliant codestream.");
  println!("    <PROFILE>=2K, 4K, 8K, 2K_R, 4K_R or 8K_R.");
  println!("    X >= 0 and X <= 11.");
  println!("    Y >= 0 and Y <= 9.");
  println!(
    "    framerate > 0 may be specified to enhance checks and set maximum bit rate when Y > 0."
  );
  println!("-GuardBits value");
  println!("    Number of guard bits in [0,7] range. Usually 1 or 2 (default value).");
  println!("-jpip");
  println!("    Write jpip codestream index box in JP2 output file.");
  println!("    Currently supports only RPCL order.");
  println!("-C <comment>");
  println!("    Add <comment> in the comment marker segment.");
  /*
  if (opj_has_thread_support()) {
    println!("-threads <num_threads|ALL_CPUS>");
    println!("    Number of threads to use for encoding or ALL_CPUS for all available cores.");
  }
  */
  println!("");
}

// Replace create_option_defs() with:
fn validate_args(args: Vec<String>) -> Option<Vec<(CompressOpt, Option<String>)>> {
  let parser = GetOpts::new(&[
    OptDef::short('i', CompressOpt::Input, true),
    OptDef::short('o', CompressOpt::Output, true),
    OptDef::short('r', CompressOpt::CompressionRatio, true),
    OptDef::short('q', CompressOpt::Quality, true),
    OptDef::short('n', CompressOpt::NumResolutions, true),
    OptDef::short('b', CompressOpt::CodeBlockSize, true),
    OptDef::short('c', CompressOpt::PrecinctSize, true),
    OptDef::short('t', CompressOpt::TileSize, true),
    OptDef::short('p', CompressOpt::ProgressionOrder, true),
    OptDef::short('s', CompressOpt::SubsamplingFactor, true),
    OptDef::short('M', CompressOpt::ModeSwitch, true),
    OptDef::short('m', CompressOpt::MCTData, true),
    OptDef::short('x', CompressOpt::IndexFile, true),
    OptDef::short('d', CompressOpt::ImageOffset, true),
    OptDef::short('T', CompressOpt::TileOffset, true),
    OptDef::short('I', CompressOpt::Irreversible, false),
    OptDef::short('f', CompressOpt::FixedLayer, true),
    OptDef::short('C', CompressOpt::Comment, true),
    OptDef::short('F', CompressOpt::RawFormat, true),
    OptDef::short('h', CompressOpt::Help, false),
    OptDef::both('w', "cinema2K", CompressOpt::Cinema2K, true),
    OptDef::both('y', "cinema4K", CompressOpt::Cinema4K, false),
    OptDef::both('z', "ImgDir", CompressOpt::ImgDir, true),
    OptDef::both('u', "TP", CompressOpt::TileParts, true),
    OptDef::both('S', "SOP", CompressOpt::SOP, false),
    OptDef::both('E', "EPH", CompressOpt::EPH, false),
    OptDef::both('O', "OutFor", CompressOpt::OutFormat, true),
    OptDef::both('P', "POC", CompressOpt::POC, true),
    OptDef::both('R', "ROI", CompressOpt::ROI, true),
    OptDef::both('J', "jpip", CompressOpt::JPIP, false),
    OptDef::both('Y', "mct", CompressOpt::MCT, true),
    OptDef::both('Z', "IMF", CompressOpt::IMF, true),
    OptDef::both('A', "PLT", CompressOpt::PLT, false),
    OptDef::both('B', "threads", CompressOpt::Threads, true),
    OptDef::both('D', "TLM", CompressOpt::TLM, false),
    OptDef::both('X', "TargetBitDepth", CompressOpt::TargetBitDepth, true),
    OptDef::both('G', "GuardBits", CompressOpt::GuardBits, true),
  ]);

  let args = parser.parse_args(args);
  let mut valid_args = Vec::new();
  let mut show_help = false;
  for arg in args {
    match arg {
      ParsedOpt::Program(_) => (),
      ParsedOpt::Opt(CompressOpt::Help, _) => show_help = true,
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
    encode_help_display();
    return None;
  }
  Some(valid_args)
}

pub fn parse_cli_options(
  args: Vec<String>,
) -> Result<Option<CLIOptions>, Box<dyn std::error::Error>> {
  let mut c_params = CompressionParameters::default();
  let mut img_folder = ImageFolder {
    img_dir_path: None,
    out_format: None,
    set_img_dir: false,
    set_out_format: false,
  };

  let args = match validate_args(args) {
    Some(args) => args,
    None => return Ok(None),
  };

  for arg in args {
    match arg {
      (CompressOpt::Input, Some(arg)) => {
        let input = PathBuf::from(arg);
        c_params.decode_format =
          ImageFileFormat::get_file_format(input.to_str().ok_or("Invalid input path")?).ok();
        c_params.input_file = Some(input);
      }
      (CompressOpt::Output, Some(arg)) => {
        let output = PathBuf::from(arg);
        c_params.codec_format =
          CodecFormat::get_file_format(output.to_str().ok_or("Invalid output path")?).ok();
        c_params.output_file = Some(output);
      }
      (CompressOpt::ImgDir, Some(arg)) => {
        img_folder.img_dir_path = Some(PathBuf::from(arg));
        img_folder.set_img_dir = true;
      }
      (CompressOpt::OutFormat, Some(arg)) => {
        img_folder.out_format = Some(arg);
        img_folder.set_out_format = true;
      }
      (CompressOpt::NumResolutions, Some(arg)) => {
        c_params.num_resolutions = arg.parse()?;
      }
      (CompressOpt::CompressionRatio, Some(arg)) => {
        c_params.rates = arg
          .split(',')
          .map(|s| s.parse())
          .collect::<Result<_, _>>()
          .map_err(|e| ParameterError::InvalidValue(format!("Invalid compression ratio: {}", e)))?;
        c_params.cp_disto_alloc = true;
      }
      (CompressOpt::ProgressionOrder, Some(arg)) => {
        c_params.prog_order = arg.parse()?;
      }
      (CompressOpt::TileSize, Some(arg)) => {
        c_params.tile_size = Some(arg.parse()?);
      }
      (CompressOpt::ROI, Some(arg)) => {
        c_params.roi = Some(arg.parse()?);
      }
      (CompressOpt::POC, Some(arg)) => {
        c_params.poc_markers = arg
          .split('/')
          .map(POCMarker::from_str)
          .collect::<Result<_, _>>()?;
      }
      (CompressOpt::RawFormat, Some(arg)) => {
        c_params.raw_params = Some(arg.parse()?);
      }
      (CompressOpt::Irreversible, _) => c_params.irreversible = true,
      (CompressOpt::GuardBits, Some(arg)) => c_params.guard_bits = arg.parse()?,
      (CompressOpt::MCT, Some(arg)) => c_params.mct_mode = Some(arg.parse()?),
      (CompressOpt::SOP, _) => c_params.csty |= 0x02,
      (CompressOpt::EPH, _) => c_params.csty |= 0x04,
      (CompressOpt::ModeSwitch, Some(arg)) => c_params.mode_switch = arg.parse()?,
      // Add missing option handlers:
      (CompressOpt::Threads, Some(arg)) => {
        if arg == "ALL_CPUS" {
          // TODO: Use num_cpus crate
          c_params.num_threads = 4; //num_cpus::get() as i32;
          if c_params.num_threads == 1 {
            c_params.num_threads = 0;
          }
        } else {
          c_params.num_threads = arg.parse()?;
        }
      }
      (CompressOpt::PLT, _) => c_params.write_plt = true,
      (CompressOpt::TLM, _) => c_params.write_tlm = true,
      (CompressOpt::Quality, Some(arg)) => {
        c_params.psnrs = arg
          .split(',')
          .map(|s| s.parse())
          .collect::<Result<_, _>>()?;
        c_params.cp_fixed_quality = true;
      }
      (CompressOpt::FixedLayer, Some(arg)) => {
        // Parse fixed layer parameters
        let layer_params: Vec<&str> = arg.split(',').collect();
        if layer_params.len() < 1 {
          return Err("Invalid fixed layer parameters".into());
        }
        c_params.num_layers = layer_params[0].parse()?;
        c_params.cp_fixed_alloc = true;
        todo!("Parse fixed layer parameters to cp_matrice.");
      }
      (CompressOpt::Comment, Some(arg)) => c_params.comment = Some(arg),
      (CompressOpt::Cinema2K, Some(arg)) => {
        let fps: u32 = arg.parse()?;
        c_params.cinema_mode = match fps {
          24 => Some(CinemaMode::Cinema2K24),
          48 => Some(CinemaMode::Cinema2K48),
          _ => return Err("Cinema 2K fps must be 24 or 48".into()),
        };
      }
      (CompressOpt::Cinema4K, _) => {
        c_params.cinema_mode = Some(CinemaMode::Cinema4K24);
      }
      (CompressOpt::IMF, Some(arg)) => {
        let parts: Vec<&str> = arg.split(',').collect();
        let profile = match parts[0] {
          "2K" => IMFProfile::new(OPJ_PROFILE_IMF_2K),
          "4K" => IMFProfile::new(OPJ_PROFILE_IMF_4K),
          "8K" => IMFProfile::new(OPJ_PROFILE_IMF_8K),
          "2K_R" => IMFProfile::new(OPJ_PROFILE_IMF_2K_R),
          "4K_R" => IMFProfile::new(OPJ_PROFILE_IMF_4K_R),
          "8K_R" => IMFProfile::new(OPJ_PROFILE_IMF_8K_R),
          _ => return Err("Invalid IMF profile".into()),
        };
        c_params.imf_profile = Some(profile);

        // Parse optional parameters
        for param in parts.iter().skip(1) {
          if let Some(val) = param.strip_prefix("mainlevel=") {
            let level: u32 = val.parse()?;
            if level > 11 {
              return Err("IMF mainlevel must be <= 11".into());
            }
            c_params.imf_profile.as_mut().unwrap().mainlevel = level;
          } else if let Some(val) = param.strip_prefix("sublevel=") {
            let level: u32 = val.parse()?;
            if level > 9 {
              return Err("IMF sublevel must be <= 9".into());
            }
            c_params.imf_profile.as_mut().unwrap().sublevel = level;
          } else if let Some(val) = param.strip_prefix("framerate=") {
            let fps: u32 = val.parse()?;
            if fps == 0 {
              return Err("IMF framerate must be > 0".into());
            }
            c_params.imf_profile.as_mut().unwrap().framerate = Some(fps);
          }
        }
      }
      (CompressOpt::TileParts, Some(arg)) => {
        c_params.tp_flag = match arg.as_str() {
          "R" => Some(TPFlag::R),
          "L" => Some(TPFlag::L),
          "C" => Some(TPFlag::C),
          _ => return Err("Invalid tile part flag - must be R, L or C".into()),
        }
      }
      (CompressOpt::IndexFile, Some(arg)) => c_params.indexfile = Some(arg),
      (CompressOpt::ImageOffset, Some(arg)) => c_params.image_offset = Some(arg.parse()?),
      (CompressOpt::TileOffset, Some(arg)) => c_params.tile_offset = Some(arg.parse()?),
      (CompressOpt::SubsamplingFactor, Some(arg)) => c_params.subsampling = Some(arg.parse()?),
      (CompressOpt::JPIP, _) => c_params.jpip_on = true,
      (CompressOpt::MCTData, Some(_arg)) => {
        todo!("Parse MCT data from file");
        // Read MCT data from file
        //let mct_data = read_mct_data(arg)?;
        //compression_params.mct_data = Some(mct_data);
      }
      (CompressOpt::TargetBitDepth, Some(arg)) => {
        c_params.target_bit_depth = Some(arg.parse()?);
      }
      (CompressOpt::CodeBlockSize, Some(arg)) => {
        c_params.codeblock = Some(arg.parse()?);
      }
      (CompressOpt::PrecinctSize, Some(arg)) => {
        c_params.precinct = arg
          .split(',')
          .map(|s| s.parse())
          .collect::<Result<_, _>>()
          .map_err(|e| ParameterError::InvalidValue(format!("Invalid precinct size: {}", e)))?;
        c_params.csty |= 0x01;
      }
      (CompressOpt::Help, _) => {
        encode_help_display();
        return Ok(None);
      }
      (opt, None) => return Err(format!("Missing argument for option: {:?}", opt).into()),
    }
  }

  // Validate parameters
  if img_folder.set_img_dir {
    if c_params.input_file.is_some() {
      return Err("Cannot use -ImgDir with -i".into());
    }
    if !img_folder.set_out_format {
      return Err("Must specify -OutFor when using -ImgDir".into());
    }
  } else if c_params.input_file.is_none() || c_params.output_file.is_none() {
    return Err("Must specify input (-i) and output (-o) files".into());
  }

  // Validate raw format parameters
  if matches!(
    c_params.decode_format,
    Some(ImageFileFormat::RAW | ImageFileFormat::RAWL)
  ) && c_params.raw_params.is_none()
  {
    return Err("Must specify raw format parameters with -F option".into());
  }

  match (
    c_params.cp_disto_alloc,
    c_params.cp_fixed_quality,
    c_params.cp_fixed_alloc,
  ) {
    (true, true, _) => return Err("Options -r and -q cannot be used together".into()),
    (true, _, true) => return Err("Options -r and -f cannot be used together".into()),
    (_, true, true) => return Err("Options -r and -f cannot be used together".into()),
    _ => (),
  }

  // Default to lossless if no rate specified
  if c_params.num_layers == 0 {
    c_params.rates = vec![0.0];
    c_params.num_layers = 1;
    c_params.cp_disto_alloc = true;
  }

  // Validate tile offsets
  if let Some(tile_offset) = &c_params.tile_offset {
    if let Some(image_offset) = &c_params.image_offset {
      if tile_offset.x > image_offset.x || tile_offset.y > image_offset.y {
        return Err("Tile offset must be <= image offset".into());
      }
    }
  }

  // Validate POC markers
  for poc in &c_params.poc_markers {
    if matches!(poc.prog_order, ProgressionOrder::UNKNOWN) {
      return Err("Invalid progression order in POC".into());
    }
  }

  // Validate ROI upshift
  if let Some(roi) = &c_params.roi {
    if roi.shift > 37 {
      return Err("ROI upshift value must be <= 37".into());
    }
  }

  Ok(Some(CLIOptions {
    compression_params: c_params,
    img_folder,
  }))
}

// Equivalent to img_fol_t
#[derive(Clone, Debug)]
pub struct ImageFolder {
  pub img_dir_path: Option<PathBuf>,
  pub out_format: Option<String>,
  pub set_img_dir: bool,
  pub set_out_format: bool,
}

/// MCT Mode
#[derive(Clone, Debug, PartialEq)]
pub enum MCTMode {
  None,
  RGB2YCC,
  Custom,
}

impl FromStr for MCTMode {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "0" => Ok(MCTMode::None),
      "1" => Ok(MCTMode::RGB2YCC),
      "2" => Ok(MCTMode::Custom),
      _ => Err("Invalid MCT mode".into()),
    }
  }
}

/// TPFlag
#[derive(Clone, Debug, PartialEq)]
pub enum TPFlag {
  R,
  L,
  C,
}

// Basic compression parameters (subset of opj_cparameters_t)
#[derive(Clone, Debug)]
pub struct CompressionParameters {
  pub input_file: Option<PathBuf>,
  pub output_file: Option<PathBuf>,
  pub codec_format: Option<CodecFormat>,
  pub decode_format: Option<ImageFileFormat>,
  pub num_threads: i32,
  pub num_resolutions: u32,
  pub max_comp_size: usize,
  pub max_cs_size: usize,
  pub prog_order: ProgressionOrder,
  pub irreversible: bool,
  // cp_tdx, cp_tdy and tile_sizes_on.
  pub tile_size: Option<Size2D>,
  // cp_tx0, cp_ty0.
  pub tile_offset: Option<Offset2D>, // Tile origin offset
  pub guard_bits: u32,
  pub mct_mode: Option<MCTMode>,
  pub mct_data: Option<Vec<f32>>,
  pub poc_markers: Vec<POCMarker>,
  // tp_on, tp_flag.  Tile parts division mode
  pub tp_flag: Option<TPFlag>,
  pub csty: u32,        // Coding style
  pub mode_switch: u32, // Mode switches
  pub num_layers: u32,  // Number of quality layers
  pub rates: Vec<f32>,  // Target compression ratios
  pub psnrs: Vec<f32>,  // Target PSNR values
  pub comment: Option<String>,
  // roi_comp and roi_shift.
  pub roi: Option<RegionOfInterest>,
  // cblockw_init, cblockh_init.
  pub codeblock: Option<Size2D>,       // Code-block dimensions
  pub precinct: Vec<Size2D>,           // Precinct dimensions per resolution
  pub image_offset: Option<Offset2D>,  // Image origin offset
  pub jpip_on: bool,                   // JPIP indexing
  pub cinema_mode: Option<CinemaMode>, // Digital Cinema profile
  pub imf_profile: Option<IMFProfile>,
  pub raw_params: Option<RawParameters>,
  pub indexfile: Option<String>,
  pub subsampling: Option<Size2D>,
  pub write_plt: bool,
  pub write_tlm: bool,
  pub target_bit_depth: Option<u32>,
  pub cp_disto_alloc: bool,
  pub cp_fixed_alloc: bool,
  pub cp_fixed_quality: bool,
}

impl Default for CompressionParameters {
  fn default() -> Self {
    CompressionParameters {
      input_file: None,
      output_file: None,
      codec_format: None,
      decode_format: None,
      num_threads: 1,
      num_resolutions: 6,
      max_comp_size: 0,
      max_cs_size: 0,
      prog_order: ProgressionOrder::LRCP,
      irreversible: false,
      tile_size: None,
      tile_offset: None,
      guard_bits: 2,
      mct_mode: None,
      mct_data: None,
      poc_markers: Vec::new(),
      tp_flag: None,
      csty: 0,
      mode_switch: 0,
      num_layers: 0,
      rates: Vec::new(),
      psnrs: Vec::new(),
      comment: None,
      roi: None,
      codeblock: None,
      precinct: Vec::new(),
      image_offset: None,
      jpip_on: false,
      cinema_mode: None,
      imf_profile: None,
      raw_params: None,
      indexfile: None,
      subsampling: None,
      write_plt: false,
      write_tlm: false,
      target_bit_depth: None,
      cp_disto_alloc: false,
      cp_fixed_alloc: false,
      cp_fixed_quality: false,
    }
  }
}

impl CompressionParameters {
  pub fn image_offset(&self) -> Offset2D {
    self.image_offset.clone().unwrap_or_default()
  }

  pub fn subsampling(&self) -> Size2D {
    self.subsampling.clone().unwrap_or_else(|| Size2D {
      width: 1,
      height: 1,
    })
  }

  pub fn to_c_params(&self) -> opj_cparameters_t {
    // Start with defaults
    let mut c_params = opj_cparameters_t::default();

    // Input/output files are handled separately

    // Set codec format
    c_params.cod_format = match self.codec_format {
      Some(CodecFormat::J2K) => OPJ_CODEC_FORMAT::OPJ_CODEC_J2K as i32,
      Some(CodecFormat::JP2) => OPJ_CODEC_FORMAT::OPJ_CODEC_JP2 as i32,
      _ => -1,
    };

    // Set decode format
    c_params.decod_format = match self.decode_format {
      Some(ImageFileFormat::PGX) => 0,
      Some(ImageFileFormat::PXM) => 1,
      Some(ImageFileFormat::BMP) => 2,
      Some(ImageFileFormat::TIF) => 3,
      Some(ImageFileFormat::RAW) => 4,
      Some(ImageFileFormat::RAWL) => 5,
      Some(ImageFileFormat::TGA) => 6,
      Some(ImageFileFormat::PNG) => 7,
      _ => -1,
    };

    // Tile parameters
    if let Some(tile_size) = &self.tile_size {
      c_params.tile_size_on = 1;
      c_params.cp_tdx = tile_size.width as i32;
      c_params.cp_tdy = tile_size.height as i32;
    }

    if let Some(offset) = &self.tile_offset {
      c_params.cp_tx0 = offset.x as i32;
      c_params.cp_ty0 = offset.y as i32;
    }

    // Rate allocation
    if self.cp_disto_alloc {
      c_params.cp_disto_alloc = 1;
      for (i, &rate) in self.rates.iter().enumerate() {
        c_params.tcp_rates[i] = rate;
      }
    }
    if self.cp_fixed_alloc {
      c_params.cp_fixed_alloc = 1;
    }
    if self.cp_fixed_quality {
      c_params.cp_fixed_quality = 1;
      for (i, &distortion) in self.psnrs.iter().enumerate() {
        c_params.tcp_distoratio[i] = distortion;
      }
    }

    // Comment
    if let Some(comment) = &self.comment {
      c_params.set_comment(comment);
    }

    // Various parameters
    c_params.csty = self.csty as i32;
    c_params.prog_order = match self.prog_order {
      ProgressionOrder::LRCP => OPJ_LRCP,
      ProgressionOrder::RLCP => OPJ_RLCP,
      ProgressionOrder::RPCL => OPJ_RPCL,
      ProgressionOrder::PCRL => OPJ_PCRL,
      ProgressionOrder::CPRL => OPJ_CPRL,
      _ => OPJ_PROG_UNKNOWN,
    };

    // POC markers
    c_params.numpocs = self.poc_markers.len() as u32;
    for (i, poc) in self.poc_markers.iter().enumerate() {
      if i >= c_params.POC.len() {
        break;
      }
      c_params.POC[i].tile = poc.tile;
      c_params.POC[i].resno0 = poc.resolution;
      c_params.POC[i].compno0 = poc.component;
      c_params.POC[i].layno1 = poc.layer;
      c_params.POC[i].prg = match poc.prog_order {
        ProgressionOrder::LRCP => OPJ_LRCP,
        ProgressionOrder::RLCP => OPJ_RLCP,
        ProgressionOrder::RPCL => OPJ_RPCL,
        ProgressionOrder::PCRL => OPJ_PCRL,
        ProgressionOrder::CPRL => OPJ_CPRL,
        _ => OPJ_PROG_UNKNOWN,
      };
    }

    // Layers and resolutions
    c_params.tcp_numlayers = self.num_layers as i32;
    c_params.numresolution = self.num_resolutions as i32;

    // Code-block size
    if let Some(codeblock) = &self.codeblock {
      c_params.cblockw_init = codeblock.width as i32;
      c_params.cblockh_init = codeblock.height as i32;
    }

    // Mode switches
    c_params.mode = self.mode_switch as i32;
    c_params.irreversible = self.irreversible as i32;

    // ROI
    if let Some(roi) = &self.roi {
      c_params.roi_compno = roi.comp as i32;
      c_params.roi_shift = roi.shift as i32;
    }

    // Precinct sizes
    c_params.res_spec = self.precinct.len() as i32;
    for (i, size) in self.precinct.iter().enumerate() {
      if i >= c_params.prcw_init.len() {
        break;
      }
      c_params.prcw_init[i] = size.width as i32;
      c_params.prch_init[i] = size.height as i32;
    }

    // Image offset
    let offset = self.image_offset();
    c_params.image_offset_x0 = offset.x as i32;
    c_params.image_offset_y0 = offset.y as i32;

    // Subsampling
    let subsampling = self.subsampling();
    c_params.subsampling_dx = subsampling.width as i32;
    c_params.subsampling_dy = subsampling.height as i32;

    // Cinema profiles
    if let Some(mode) = &self.cinema_mode {
      c_params.cp_cinema = match mode {
        CinemaMode::Cinema2K24 => OPJ_CINEMA2K_24,
        CinemaMode::Cinema2K48 => OPJ_CINEMA2K_48,
        CinemaMode::Cinema4K24 => OPJ_CINEMA4K_24,
      };
    }

    // IMF profile
    if let Some(imf) = &self.imf_profile {
      c_params.rsiz = ((imf.profile as u32) | (imf.sublevel << 4) | imf.mainlevel) as u16;

      if imf.sublevel > 0 && imf.sublevel <= 9 {
        if let Some(fps) = imf.framerate {
          let limit_mbits_sec = match imf.sublevel {
            1 => 200,
            2 => 400,
            3 => 800,
            4 => 1600,
            5 => 3200,
            6 => 6400,
            7 => 12800,
            8 => 25600,
            9 => 51200,
            _ => 0,
          };
          c_params.max_cs_size = (limit_mbits_sec * 1000000 / 8 / fps) as i32;
        }
      }
    }

    // MCT mode
    if let Some(mode) = &self.mct_mode {
      c_params.tcp_mct = match mode {
        MCTMode::None => 0,
        MCTMode::RGB2YCC => 1,
        MCTMode::Custom => 2,
      };
    }

    // Various flags
    c_params.jpip_on = self.jpip_on as i32;

    if let Some(flag) = &self.tp_flag {
      c_params.tp_on = 1;
      c_params.tp_flag = match flag {
        TPFlag::R => b'R' as i8,
        TPFlag::L => b'L' as i8,
        TPFlag::C => b'C' as i8,
      };
    }

    c_params
  }
}

/// Size2D
#[derive(Clone, Debug, PartialEq)]
pub struct Size2D {
  pub width: u32,
  pub height: u32,
}

impl FromStr for Size2D {
  type Err = ParameterError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
      return Err(ParameterError::InvalidFormat(
        "Size format: width,height".into(),
      ));
    }

    let width = parts[0]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid width".into()))?;
    let height = parts[1]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid height".into()))?;

    Ok(Size2D { width, height })
  }
}

/// Offset2D
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Offset2D {
  pub x: u32,
  pub y: u32,
}

impl FromStr for Offset2D {
  type Err = ParameterError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
      return Err(ParameterError::InvalidFormat("Offset format: x,y".into()));
    }

    let x = parts[0]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid x offset".into()))?;
    let y = parts[1]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid y offset".into()))?;

    Ok(Offset2D { x, y })
  }
}

/// RegionOfInterest
#[derive(Clone, Debug, PartialEq)]
pub struct RegionOfInterest {
  pub comp: u32,
  pub shift: u32,
}

impl FromStr for RegionOfInterest {
  type Err = ParameterError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
      return Err(ParameterError::InvalidFormat(
        "ROI format should be: c=comp,U=shift".into(),
      ));
    }

    let comp = parts[0]
      .trim_start_matches("c=")
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid component number".into()))?;

    let shift = parts[1]
      .trim_start_matches("U=")
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid shift value".into()))?;

    Ok(RegionOfInterest { comp, shift })
  }
}

/// CinemaMode
#[derive(Clone, Debug, PartialEq)]
pub enum CinemaMode {
  Cinema2K24,
  Cinema2K48,
  Cinema4K24,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CodecFormat {
  J2K,
  JPT,
  JP2,
  JPP,
  JPX,
}

impl CodecFormat {
  pub fn get_file_format(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
    match filename.rsplit('.').next().map(|s| s.to_lowercase()) {
      Some(ext) => match ext.as_str() {
        "j2k" | "j2c" => Ok(CodecFormat::J2K),
        "jp2" => Ok(CodecFormat::JP2),
        _ => Err("Unknown output format - must be .j2k, .j2c or .jp2".into()),
      },
      None => Err("Missing file extension".into()),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImageFileFormat {
  PGX,
  PXM,
  BMP,
  TIF,
  RAW,
  RAWL,
  TGA,
  PNG,
  J2K,
  JPT,
  JP2,
  JPP,
  JPX,
}

impl ImageFileFormat {
  pub fn get_file_format<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn std::error::Error>> {
    let filename = filename.as_ref();
    let ext = filename
      .extension()
      .and_then(|s| s.to_ascii_lowercase().into_string().ok());
    match ext {
      Some(ext) => match ext.as_str() {
        "pgx" => Ok(ImageFileFormat::PGX),
        "pnm" | "pgm" | "ppm" => Ok(ImageFileFormat::PXM),
        "bmp" => Ok(ImageFileFormat::BMP),
        "tif" | "tiff" => Ok(ImageFileFormat::TIF),
        "raw" | "yuv" => Ok(ImageFileFormat::RAW),
        "rawl" => Ok(ImageFileFormat::RAWL),
        "tga" => Ok(ImageFileFormat::TGA),
        "png" => Ok(ImageFileFormat::PNG),
        "j2k" | "j2c" | "jpc" | "jhc" => Ok(ImageFileFormat::J2K),
        "jp2" | "jph" => Ok(ImageFileFormat::JP2),
        "jpt" => Ok(ImageFileFormat::JPT),
        "jpp" => Ok(ImageFileFormat::JPP),
        "jpx" => Ok(ImageFileFormat::JPX),
        _ => Err("Unknown input format".into()),
      },
      None => Err("Missing file extension".into()),
    }
  }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ProgressionOrder {
  #[default]
  LRCP,
  RLCP,
  RPCL,
  PCRL,
  CPRL,
  UNKNOWN,
}

impl FromStr for ProgressionOrder {
  type Err = ParameterError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "LRCP" => Ok(ProgressionOrder::LRCP),
      "RLCP" => Ok(ProgressionOrder::RLCP),
      "RPCL" => Ok(ProgressionOrder::RPCL),
      "PCRL" => Ok(ProgressionOrder::PCRL),
      "CPRL" => Ok(ProgressionOrder::CPRL),
      _ => Err(ParameterError::InvalidValue(
        "Invalid progression order".into(),
      )),
    }
  }
}

// Helper structs for parameter parsing
#[derive(Clone, Debug, Default)]
pub struct POCMarker {
  pub tile: u32,
  pub resolution: u32,
  pub component: u32,
  pub layer: u32,
  pub prog_order: ProgressionOrder,
}

impl FromStr for POCMarker {
  type Err = ParameterError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut parts = s.split('=');
    let tile_str = parts
      .next()
      .ok_or_else(|| ParameterError::InvalidFormat("Missing tile spec".into()))?;
    let params_str = parts
      .next()
      .ok_or_else(|| ParameterError::InvalidFormat("Missing POC parameters".into()))?;

    let tile = tile_str
      .trim_start_matches('T')
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid tile number".into()))?;

    let params: Vec<&str> = params_str.split(',').collect();
    if params.len() != 5 {
      return Err(ParameterError::InvalidFormat(
        "POC format: T<tile>=<resStart>,<compStart>,<layerEnd>,<resEnd>,<compEnd>,<progOrder>"
          .into(),
      ));
    }

    Ok(POCMarker {
      tile,
      resolution: params[0]
        .parse()
        .map_err(|_| ParameterError::ParseError("Invalid resolution".into()))?,
      component: params[1]
        .parse()
        .map_err(|_| ParameterError::ParseError("Invalid component".into()))?,
      layer: params[2]
        .parse()
        .map_err(|_| ParameterError::ParseError("Invalid layer".into()))?,
      prog_order: params[5].parse()?,
    })
  }
}

#[derive(Clone, Debug, Default)]
pub struct IMFProfile {
  pub profile: u32,
  pub mainlevel: u32,
  pub sublevel: u32,
  pub framerate: Option<u32>,
}

impl IMFProfile {
  pub fn new(profile: u32) -> Self {
    IMFProfile {
      profile,
      mainlevel: 0,
      sublevel: 0,
      framerate: None,
    }
  }
}
