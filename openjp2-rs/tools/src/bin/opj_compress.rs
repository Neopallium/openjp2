use clap::Parser;
use image::{self, DynamicImage};
use openjp2::{detect_format_from_file, image::opj_image, openjpeg::*};
use std::ffi::CString;
use std::io;
use std::path::{Path, PathBuf};

// Equivalent to img_fol_t
struct ImageFolder {
  img_dir_path: Option<PathBuf>,
  out_format: Option<String>,
  set_img_dir: bool,
  set_out_format: bool,
}

// Basic compression parameters (subset of opj_cparameters_t)
#[derive(Default)]
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

#[derive(Debug, Default, PartialEq)]
enum CodecFormat {
  #[default]
  Unknown,
  J2K,
  JP2,
}

#[derive(Debug, Default, PartialEq)]
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

#[derive(Debug, Default, PartialEq)]
enum ProgressionOrder {
  #[default]
  LRCP,
  RLCP,
  RPCL,
  PCRL,
  CPRL,
}

// For raw image parameters
#[derive(Default)]
pub struct RawParameters {
  width: u32,
  height: u32,
  num_comps: u32,
  bit_depth: u32,
  signed: bool,
  components: Vec<RawComponentParameters>,
}

#[derive(Default, Clone)]
pub struct RawComponentParameters {
  dx: u32,
  dy: u32,
}

#[derive(Parser)]
#[command(name = "opj_compress")]
#[command(version = "3.0.0")]
#[command(about = "JPEG 2000 compression utility")]
struct Args {
  /// Input file
  #[arg(short = 'i', long = "input")]
  input: Option<PathBuf>,

  /// Output file (.j2k, .jp2)
  #[arg(short = 'o', long = "output")]
  output: Option<PathBuf>,

  /// Image directory path
  #[arg(long = "ImgDir")]
  img_dir: Option<PathBuf>,

  /// Output format (J2K, JP2)
  #[arg(long = "OutFor")]
  out_format: Option<String>,

  /// Number of threads (or ALL_CPUS)
  #[arg(short = 'B', long = "threads")]
  threads: Option<String>,

  /// Number of resolutions
  #[arg(short = 'n')]
  resolutions: Option<u32>,

  /// Compression ratios
  #[arg(short = 'r')]
  compression: Option<String>,

  /// Progression order (LRCP, RLCP, RPCL, PCRL, CPRL)
  #[arg(short = 'p')]
  progression: Option<String>,

  /// Tile size (width,height)
  #[arg(short = 't')]
  tile_size: Option<String>,

  /// Use irreversible DWT 9-7
  #[arg(short = 'I')]
  irreversible: bool,

  /// Guard bits (0-7)
  #[arg(long = "GuardBits")]
  guard_bits: Option<u32>,

  /// Color transform: 0=none, 1=RGB->YCC, 2=custom
  #[arg(long = "mct")]
  mct_mode: Option<u32>,

  /// Custom MCT transform file
  #[arg(short = 'm')]
  mct_file: Option<PathBuf>,

  /// Raw image parameters - width,height,ncomp,bitdepth,[s|u],dx1,dy1:...:dxn,dyn
  #[arg(short = 'F')]
  raw_params: Option<String>,

  /// Subsampling factors
  #[arg(short = 's')]
  subsampling: Option<String>,

  /// Code-block size
  #[arg(short = 'b')]
  codeblock_size: Option<String>,

  /// Precinct size
  #[arg(short = 'c')]
  precinct_size: Option<String>,

  /// ROI: c=component,U=shift
  #[arg(long = "ROI")]
  roi: Option<String>,

  /// Quality layers (PSNR/rates)
  #[arg(short = 'q')]
  quality_layers: Option<String>,

  /// Enable SOP marker
  #[arg(long = "SOP")]
  sop: bool,

  /// Enable EPH marker
  #[arg(long = "EPH")]
  eph: bool,

  /// Enable PLT marker
  #[arg(long = "PLT")]
  plt: bool,

  /// Enable TLM marker
  #[arg(long = "TLM")]
  tlm: bool,

  /// Mode switches [1=BYPASS, 2=RESET, 4=RESTART, 8=VSC, 16=ERTERM, 32=SEGMARK]
  #[arg(short = 'M')]
  mode: Option<u32>,

  /// Progression order change.
  /// The syntax of a progression order change is the following:
  /// T<tile>=<resStart>,<compStart>,<layerEnd>,<resEnd>,<compEnd>,<progOrder>
  /// Example: -POC T1=0,0,1,5,3,CPRL/T1=5,0,1,6,3,CPRL
  #[arg(long = "POC")]
  poc: Option<String>,

  /// Digital Cinema 2K profile (24/48 fps)
  #[arg(long = "cinema2K")]
  cinema2k: Option<u32>,

  /// Digital Cinema 4K profile
  #[arg(long = "cinema4K")]
  cinema4k: bool,

  /// IMF profile
  #[arg(long = "IMF")]
  imf: Option<String>,

  /// JPIP indexing
  #[arg(long = "jpip")]
  jpip: bool,

  /// Comment to add
  #[arg(short = 'C')]
  comment: Option<String>,

  /// Image/tile origin offset
  #[arg(short = 'd')]
  offset: Option<String>,

  /// Tile offset
  #[arg(short = 'T')]
  tile_offset: Option<String>,

  /// Tile parts: R=resolution, L=layer, C=component
  #[arg(long = "TP")]
  tile_parts: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = Args::parse();

  // Initialize parameters
  let mut compression_params = CompressionParameters::default();
  let mut img_folder = ImageFolder {
    img_dir_path: None,
    out_format: None,
    set_img_dir: false,
    set_out_format: false,
  };

  // Parse input file/directory
  if let Some(input) = args.input.clone() {
    compression_params.decode_format =
      get_file_format(input.to_str().ok_or("Invalid input path")?)?;
    compression_params.input_file = Some(input);
  }

  if let Some(imgdir) = args.img_dir.clone() {
    img_folder.img_dir_path = Some(imgdir);
    img_folder.set_img_dir = true;
  }

  // Parse output file/format
  if let Some(output) = args.output.clone() {
    compression_params.codec_format =
      get_codec_format(output.to_str().ok_or("Invalid output path")?)?;
    compression_params.output_file = Some(output);
  }

  if let Some(format) = args.out_format.clone() {
    img_folder.out_format = Some(format);
    img_folder.set_out_format = true;
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

  // Update compression parameters from args
  compression_params.update_from_args(&args)?;

  // Process files
  let start_time = std::time::Instant::now();
  let mut num_compressed = 0;

  if let Some(dir) = args.img_dir.as_ref() {
    // Process directory
    let dir_contents = DirContents::new(dir)?;

    for file in dir_contents.files {
      if let Ok(_format) = detect_format_from_file(&file) {
        println!("\nProcessing: {}", file.display());

        // Update parameters for this file
        compression_params.input_file = Some(file.clone());
        compression_params.decode_format = get_file_format(file.to_str().ok_or("Invalid path")?)?;

        // Generate output filename
        let output = generate_output_path(&file, &img_folder)?;
        compression_params.output_file = Some(output.clone());

        // Process file
        let image = load_image(&file, &compression_params)?;
        compress_image(image, &compression_params, &output)?;

        num_compressed += 1;
      }
    }
  } else if let Some(input) = compression_params.input_file.as_ref() {
    // Process single file
    let image = load_image(input, &compression_params)?;
    let output = compression_params
      .output_file
      .as_ref()
      .ok_or("No output file specified")?;
    compress_image(image, &compression_params, output)?;
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
struct POCMarker {
  tile: u32,
  resolution: u32,
  component: u32,
  layer: u32,
  prog_order: ProgressionOrder,
}

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

  fn update_from_args(&mut self, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    // Handle resolutions
    if let Some(res) = args.resolutions {
      self.num_resolutions = res;
    }

    // Handle compression ratios
    if let Some(ref comp) = args.compression {
      self.rates = Self::parse_quality_layers(comp)?;
    }

    // Handle progression order
    if let Some(ref prog) = args.progression {
      self.prog_order = parse_progression_order(prog)?;
    }

    // Handle tile size
    if let Some(ref tile_size) = args.tile_size {
      let (w, h) = parse_dimensions(tile_size)?;
      self.tile_size = (w, h);
      self.tile_size_on = true;
    }

    // Handle raw parameters
    if let Some(ref raw) = args.raw_params {
      let _raw_params = Self::parse_raw_params(raw)?;
      // TODO: Update compression params with raw params
    }

    // Handle code block size
    if let Some(ref block_size) = args.codeblock_size {
      let (w, h) = parse_dimensions(block_size)?;
      self.codeblock_width = w;
      self.codeblock_height = h;
    }

    // Handle markers
    self.csty |= if args.sop { 0x02 } else { 0 };
    self.csty |= if args.eph { 0x04 } else { 0 };

    // Handle mode switches
    if let Some(mode) = args.mode {
      self.mode_switch = mode;
    }

    // Handle ROI
    if let Some(ref roi) = args.roi {
      let (comp, shift) = parse_roi(roi)?;
      self.roi_comp = comp;
      self.roi_shift = shift;
    }

    // Handle other parameters
    self.guard_bits = args.guard_bits.unwrap_or(2);
    self.mct_mode = args.mct_mode.unwrap_or(1); // Default to RGB->YCC
    self.jpip_on = args.jpip;
    self.comment = args.comment.clone();

    Ok(())
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
