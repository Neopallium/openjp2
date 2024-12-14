use image::{self, DynamicImage};
use openjp2::{detect_format_from_file, image::opj_image, openjpeg::*};
use openjp2_tools::getopt::{GetOpts, OptDef, ParsedOpt};
use std::ffi::CString;
use std::io;
use std::path::{Path, PathBuf};

// New struct to hold parsed CLI options
struct CLIOptions {
  compression_params: CompressionParameters,
  img_folder: ImageFolder,
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

fn parse_cli_options(args: Vec<String>) -> Result<Option<CLIOptions>, Box<dyn std::error::Error>> {
  let mut compression_params = CompressionParameters::default();
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
        compression_params.decode_format =
          get_file_format(input.to_str().ok_or("Invalid input path")?)?;
        compression_params.input_file = Some(input);
      }
      (CompressOpt::Output, Some(arg)) => {
        let output = PathBuf::from(arg);
        compression_params.codec_format =
          get_codec_format(output.to_str().ok_or("Invalid output path")?)?;
        compression_params.output_file = Some(output);
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
        compression_params.num_resolutions = arg.parse()?
      }
      (CompressOpt::CompressionRatio, Some(arg)) => {
        compression_params.rates = CompressionParameters::parse_quality_layers(&arg)?
      }
      (CompressOpt::ProgressionOrder, Some(arg)) => {
        compression_params.prog_order = parse_progression_order(&arg)?
      }
      (CompressOpt::TileSize, Some(arg)) => {
        let (w, h) = parse_dimensions(&arg)?;
        compression_params.tile_size = (w, h);
        compression_params.tile_size_on = true;
      }
      (CompressOpt::Irreversible, None) => compression_params.irreversible = true,
      (CompressOpt::GuardBits, Some(arg)) => compression_params.guard_bits = arg.parse()?,
      (CompressOpt::MCT, Some(arg)) => compression_params.mct_mode = arg.parse()?,
      (CompressOpt::SOP, None) => compression_params.csty |= 0x02,
      (CompressOpt::EPH, None) => compression_params.csty |= 0x04,
      (CompressOpt::ModeSwitch, Some(arg)) => compression_params.mode_switch = arg.parse()?,
      (opt, arg) => return Err(format!("TODO: Unhandled option: {:?}, arg={:?}", opt, arg).into()),
    }
  }

  // Validate parameters
  if img_folder.set_img_dir {
    if compression_params.input_file.is_some() {
      return Err("Cannot use -ImgDir with -i".into());
    }
    if !img_folder.set_out_format {
      return Err("Must specify -OutFor when using -ImgDir".into());
    }
  } else if compression_params.input_file.is_none() || compression_params.output_file.is_none() {
    return Err("Must specify input (-i) and output (-o) files".into());
  }

  Ok(Some(CLIOptions {
    compression_params,
    img_folder,
  }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Parse command line options
  let cli_opts = match parse_cli_options(std::env::args().collect())? {
    Some(opts) => opts,
    None => {
      // Show help
      return Ok(());
    }
  };

  // Process files
  let start_time = std::time::Instant::now();
  let mut num_compressed = 0;

  if let Some(dir) = cli_opts.img_folder.img_dir_path.as_ref() {
    // Process directory
    let dir_contents = DirContents::new(dir)?;

    for file in dir_contents.files {
      if let Ok(_format) = detect_format_from_file(&file) {
        println!("\nProcessing: {}", file.display());

        // Update parameters for this file
        let mut params = cli_opts.compression_params.clone();
        params.input_file = Some(file.clone());
        params.decode_format = get_file_format(file.to_str().ok_or("Invalid path")?)?;

        // Generate output filename
        let output = generate_output_path(&file, &cli_opts.img_folder)?;
        params.output_file = Some(output.clone());

        // Process file
        let image = load_image(&file, &params)?;
        compress_image(image, &params, &output)?;

        num_compressed += 1;
      }
    }
  } else if let Some(input) = cli_opts.compression_params.input_file.as_ref() {
    // Process single file
    let image = load_image(input, &cli_opts.compression_params)?;
    let output = cli_opts
      .compression_params
      .output_file
      .as_ref()
      .ok_or("No output file specified")?;
    compress_image(image, &cli_opts.compression_params, output)?;
    num_compressed += 1;
  }

  let elapsed = start_time.elapsed();
  if num_compressed > 0 {
    println!(
      "Compressed {} files in {:.2} seconds",
      num_compressed,
      elapsed.as_secs_f64()
    );
  }

  Ok(())
}

// Equivalent to img_fol_t
struct ImageFolder {
  img_dir_path: Option<PathBuf>,
  out_format: Option<String>,
  set_img_dir: bool,
  set_out_format: bool,
}

// Basic compression parameters (subset of opj_cparameters_t)
#[derive(Clone, Debug, Default)]
struct CompressionParameters {
  input_file: Option<PathBuf>,
  output_file: Option<PathBuf>,
  codec_format: CodecFormat,
  decode_format: DecodeFormat,
  num_threads: i32,
  num_resolutions: u32,
  prog_order: ProgressionOrder,
  irreversible: bool,
  tile_size_on: bool,
  tile_size: (u32, u32),
  // New fields
  guard_bits: u32,
  mct_mode: u32,
  poc_markers: Vec<POCMarker>,
  csty: u32,        // Coding style
  mode_switch: u32, // Mode switches
  num_layers: u32,  // Number of quality layers
  rates: Vec<f32>,  // Target compression ratios
  comment: Option<String>,
  roi_comp: i32,        // ROI component number
  roi_shift: u32,       // ROI upshift value
  codeblock_width: u32, // Code-block dimensions
  codeblock_height: u32,
  precinct_width: Vec<u32>, // Precinct dimensions per resolution
  precinct_height: Vec<u32>,
  image_offset: (i32, i32), // Image origin offset
  tile_offset: (i32, i32),  // Tile origin offset
  tile_parts: Option<char>, // Tile parts division mode
  jp2_mode: bool,           // JP2 file format
  jpip_on: bool,            // JPIP indexing
  cinema_mode: u32,         // Digital Cinema profile
  imf_profile: Option<IMFProfile>,
  // ... add more parameters as needed
}

#[derive(Clone, Debug, Default, PartialEq)]
enum CodecFormat {
  #[default]
  Unknown,
  J2K,
  JP2,
}

#[derive(Clone, Debug, Default, PartialEq)]
enum DecodeFormat {
  #[default]
  Unknown,
  PGX,
  PXM,
  BMP,
  TIF,
  RAW,
  RAWL,
  TGA,
  PNG,
}

#[derive(Clone, Debug, Default, PartialEq)]
enum ProgressionOrder {
  #[default]
  LRCP,
  RLCP,
  RPCL,
  PCRL,
  CPRL,
}

// For raw image parameters
#[derive(Clone, Debug, Default)]
pub struct RawParameters {
  width: u32,
  height: u32,
  num_comps: u32,
  bit_depth: u32,
  signed: bool,
  components: Vec<RawComponentParameters>,
}

#[derive(Clone, Debug, Default)]
pub struct RawComponentParameters {
  dx: u32,
  dy: u32,
}

fn get_file_format(filename: &str) -> Result<DecodeFormat, Box<dyn std::error::Error>> {
  match filename.rsplit('.').next().map(|s| s.to_lowercase()) {
    Some(ext) => match ext.as_str() {
      "pgx" => Ok(DecodeFormat::PGX),
      "pnm" | "pgm" | "ppm" => Ok(DecodeFormat::PXM),
      "bmp" => Ok(DecodeFormat::BMP),
      "tif" | "tiff" => Ok(DecodeFormat::TIF),
      "raw" | "yuv" => Ok(DecodeFormat::RAW),
      "rawl" => Ok(DecodeFormat::RAWL),
      "tga" => Ok(DecodeFormat::TGA),
      "png" => Ok(DecodeFormat::PNG),
      _ => Err("Unknown input format".into()),
    },
    None => Err("Missing file extension".into()),
  }
}

fn get_codec_format(filename: &str) -> Result<CodecFormat, Box<dyn std::error::Error>> {
  match filename.rsplit('.').next().map(|s| s.to_lowercase()) {
    Some(ext) => match ext.as_str() {
      "j2k" | "j2c" => Ok(CodecFormat::J2K),
      "jp2" => Ok(CodecFormat::JP2),
      _ => Err("Unknown output format - must be .j2k, .j2c or .jp2".into()),
    },
    None => Err("Missing file extension".into()),
  }
}

// Helper structs for parameter parsing
#[derive(Clone, Debug, Default)]
struct POCMarker {
  tile: u32,
  resolution: u32,
  component: u32,
  layer: u32,
  prog_order: ProgressionOrder,
}

#[derive(Clone, Debug, Default)]
struct IMFProfile {
  profile: u32,
  mainlevel: u32,
  sublevel: u32,
  framerate: Option<u32>,
}

// Add parameter parsing functions
impl CompressionParameters {
  fn parse_raw_params(raw_str: &str) -> Result<RawParameters, ParameterError> {
    let parts: Vec<&str> = raw_str.split(',').collect();
    if parts.len() < 5 {
      return Err(ParameterError::InvalidFormat(
        "Raw params format: width,height,ncomp,bitdepth,[s|u],dx1,dy1:...:dxn,dyn".into(),
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
        let factors: Vec<&str> = comp.split('x').collect();
        if factors.len() != 2 {
          return Err(ParameterError::InvalidFormat(
            "Subsampling format: dx1xdy1:dx2xdy2...".into(),
          ));
        }
        let dx = factors[0]
          .parse()
          .map_err(|_| ParameterError::ParseError("Invalid dx".into()))?;
        let dy = factors[1]
          .parse()
          .map_err(|_| ParameterError::ParseError("Invalid dy".into()))?;
        components.push(RawComponentParameters { dx, dy });
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

  fn parse_quality_layers(layers_str: &str) -> Result<Vec<f32>, ParameterError> {
    layers_str
      .split(',')
      .map(|s| {
        s.parse::<f32>()
          .map_err(|_| ParameterError::ParseError("Invalid quality value".into()))
      })
      .collect()
  }

  fn parse_poc_markers(poc_str: &str) -> Result<Vec<POCMarker>, ParameterError> {
    poc_str
      .split('/')
      .map(|prog| {
        let mut parts = prog.split('=');
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
          prog_order: parse_progression_order(params[4])?,
        })
      })
      .collect()
  }
}

fn parse_dimensions(dim_str: &str) -> Result<(u32, u32), ParameterError> {
  let parts: Vec<&str> = dim_str.split(',').collect();
  if parts.len() != 2 {
    return Err(ParameterError::InvalidFormat(
      "Format should be: width,height".into(),
    ));
  }

  Ok((
    parts[0]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid width".into()))?,
    parts[1]
      .parse()
      .map_err(|_| ParameterError::ParseError("Invalid height".into()))?,
  ))
}

fn parse_progression_order(order: &str) -> Result<ProgressionOrder, ParameterError> {
  match order {
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

fn parse_roi(roi_str: &str) -> Result<(i32, u32), ParameterError> {
  let parts: Vec<&str> = roi_str.split(',').collect();
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

  Ok((comp, shift))
}

#[derive(Debug)]
enum ParameterError {
  InvalidFormat(String),
  InvalidValue(String),
  ParseError(String),
}

impl std::fmt::Display for ParameterError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
      Self::InvalidValue(s) => write!(f, "Invalid value: {}", s),
      Self::ParseError(s) => write!(f, "Parse error: {}", s),
    }
  }
}

impl std::error::Error for ParameterError {}

// Add Directory handling
struct DirContents {
  files: Vec<PathBuf>,
}

impl DirContents {
  fn new(dir_path: &Path) -> io::Result<Self> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir_path)? {
      let entry = entry?;
      let path = entry.path();
      if path.is_file() {
        files.push(path);
      }
    }
    Ok(Self { files })
  }
}

// Add error types
#[derive(Debug)]
enum ImageError {
  InvalidFormat(String),
  ReadError(String),
  EncodeError(String),
  IOError(io::Error),
}

impl std::fmt::Display for ImageError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
      Self::ReadError(s) => write!(f, "Read error: {}", s),
      Self::EncodeError(s) => write!(f, "Encode error: {}", s),
      Self::IOError(e) => write!(f, "IO error: {}", e),
    }
  }
}

impl std::error::Error for ImageError {}

impl From<io::Error> for ImageError {
  fn from(error: io::Error) -> Self {
    ImageError::IOError(error)
  }
}

// Add this struct to represent our image data
#[derive(Debug)]
struct ImageComponent {
  data: Vec<i32>,
  width: u32,
  height: u32,
  precision: u32,
  signed: bool,
  dx: u32,
  dy: u32,
}

// Replace existing load_image function
fn load_image(path: &Path, params: &CompressionParameters) -> Result<Box<opj_image>, ImageError> {
  let img = match params.decode_format {
    // TODO: handle raw
    //DecodeFormat::RAW | DecodeFormat::RAWL => load_raw_image(path, params)?,
    _ => load_regular_image(path)?,
  };

  // Convert the loaded image to OpenJPEG format
  convert_to_opj_image(img, params)
}

fn load_regular_image(path: &Path) -> Result<Vec<ImageComponent>, ImageError> {
  let img = image::open(path).map_err(|e| ImageError::ReadError(e.to_string()))?;

  match img {
    DynamicImage::ImageRgb8(img) => {
      let (width, height) = img.dimensions();
      let mut components = Vec::new();

      // Extract R, G, B components
      for c in 0..3 {
        let mut data = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
          for x in 0..width {
            let pixel = img.get_pixel(x, y);
            data.push(pixel[c] as i32);
          }
        }

        components.push(ImageComponent {
          data,
          width,
          height,
          precision: 8,
          signed: false,
          dx: 1,
          dy: 1,
        });
      }

      Ok(components)
    }
    DynamicImage::ImageLuma8(img) => {
      let (width, height) = img.dimensions();
      let mut data = Vec::with_capacity((width * height) as usize);

      for y in 0..height {
        for x in 0..width {
          let pixel = img.get_pixel(x, y);
          data.push(pixel[0] as i32);
        }
      }

      Ok(vec![ImageComponent {
        data,
        width,
        height,
        precision: 8,
        signed: false,
        dx: 1,
        dy: 1,
      }])
    }
    _ => Err(ImageError::InvalidFormat(
      "Unsupported image format - convert to RGB8 or Luma8 first".into(),
    )),
  }
}

fn convert_to_opj_image(
  components: Vec<ImageComponent>,
  _params: &CompressionParameters,
) -> Result<Box<opj_image>, ImageError> {
  if components.is_empty() {
    return Err(ImageError::InvalidFormat("No image components".into()));
  }

  let reference = &components[0];
  let mut image = opj_image::new();

  image.x0 = 0;
  image.y0 = 0;
  image.x1 = reference.width;
  image.y1 = reference.height;
  image.numcomps = components.len() as u32;
  image.color_space = if components.len() >= 3 {
    OPJ_CLRSPC_SRGB
  } else {
    OPJ_CLRSPC_GRAY
  };
  image.alloc_comps(image.numcomps, false);

  let comps = image.comps_mut().expect("We just allocated them");

  for (i, comp) in components.iter().enumerate() {
    let c = &mut comps[i];
    c.dx = comp.dx;
    c.dy = comp.dy;
    c.w = comp.width;
    c.h = comp.height;
    c.x0 = 0;
    c.y0 = 0;
    c.prec = comp.precision;
    c.bpp = comp.precision;
    c.sgnd = comp.signed as u32;

    let data_size = (comp.width * comp.height) as usize;
    let data = unsafe {
      std::slice::from_raw_parts_mut(
        std::alloc::alloc(std::alloc::Layout::array::<i32>(data_size).unwrap()) as *mut i32,
        data_size,
      )
    };
    data.copy_from_slice(&comp.data);
    c.data = data.as_mut_ptr();
  }

  image.comps = comps.as_mut_ptr();

  Ok(image)
}

fn compress_image(
  mut image: Box<opj_image>,
  params: &CompressionParameters,
  output: &Path,
) -> Result<(), ImageError> {
  // Create encoder based on codec format
  let codec = unsafe {
    match params.codec_format {
      CodecFormat::J2K => opj_create_compress(OPJ_CODEC_J2K),
      CodecFormat::JP2 => opj_create_compress(OPJ_CODEC_JP2),
      _ => return Err(ImageError::InvalidFormat("Unknown output format".into())),
    }
  };

  if codec.is_null() {
    return Err(ImageError::EncodeError("Failed to create codec".into()));
  }

  // Set compression parameters
  let status = unsafe {
    let mut c_params = opj_cparameters::default();
    // Set parameters from CompressionParameters

    opj_setup_encoder(codec, &mut c_params, image.as_mut())
  };

  if status == 0 {
    return Err(ImageError::EncodeError("Failed to setup encoder".into()));
  }

  // Create output stream
  let stream = unsafe {
    let path_str = CString::new(output.to_str().unwrap()).unwrap();
    opj_stream_create_default_file_stream(path_str.as_ptr(), 0)
  };

  if stream.is_null() {
    return Err(ImageError::EncodeError(
      "Failed to create output stream".into(),
    ));
  }

  // Compress image
  let result = unsafe {
    let success = opj_start_compress(codec, image.as_mut(), stream) != 0
      && opj_encode(codec, stream) != 0
      && opj_end_compress(codec, stream) != 0;

    opj_stream_destroy(stream);
    opj_destroy_codec(codec);

    success
  };

  if !result {
    return Err(ImageError::EncodeError("Compression failed".into()));
  }

  Ok(())
}

fn generate_output_path(input: &Path, img_folder: &ImageFolder) -> Result<PathBuf, ImageError> {
  let stem = input
    .file_stem()
    .ok_or_else(|| ImageError::InvalidFormat("No filename".into()))?;

  let ext = match img_folder.out_format.as_deref() {
    Some("J2K") => "j2k",
    Some("JP2") => "jp2",
    _ => return Err(ImageError::InvalidFormat("Invalid output format".into())),
  };

  let mut output = PathBuf::from(stem);
  output.set_extension(ext);

  // If output directory specified, put file there
  if let Some(dir) = img_folder.img_dir_path.as_ref() {
    output = dir.join(output);
  }

  Ok(output)
}
