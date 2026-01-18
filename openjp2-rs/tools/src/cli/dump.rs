use crate::getopt::{GetOpts, OptDef, ParsedOpt};
use openjp2::{detect_format_from_file, openjpeg::*, Codec, J2KFormat, Stream};
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::path::{Path, PathBuf};
use std::ptr;

#[derive(Debug, Clone, PartialEq)]
enum Opt {
  ImgDir,
  Input,
  Output,
  Verbose,
  Help,
  Flag,
}

struct Args {
  img_dir: Option<PathBuf>,
  input: Option<PathBuf>,
  output: Option<PathBuf>,
  verbose: bool,
  flag: u32,
}

impl Default for Args {
  fn default() -> Self {
    Self {
      img_dir: None,
      input: None,
      output: None,
      verbose: false,
      flag: OPJ_IMG_INFO | OPJ_J2K_MH_INFO | OPJ_J2K_MH_IND,
    }
  }
}

fn parse_args(arguments: Vec<String>) -> Result<Args, String> {
  let opts = vec![
    OptDef::long("ImgDir", Opt::ImgDir, true),
    OptDef::short('i', Opt::Input, true),
    OptDef::short('o', Opt::Output, true),
    OptDef::short('v', Opt::Verbose, false),
    OptDef::short('h', Opt::Help, false),
    OptDef::short('f', Opt::Flag, true),
  ];

  let parser = GetOpts::new(&opts);
  let mut args = Args::default();

  for opt in parser.parse_args(arguments) {
    match opt {
      ParsedOpt::Program(_) => {}
      ParsedOpt::Opt(opt, arg) => match opt {
        Opt::ImgDir => {
          args.img_dir = Some(PathBuf::from(arg.ok_or("Missing ImgDir path")?));
        }
        Opt::Input => {
          args.input = Some(PathBuf::from(arg.ok_or("Missing input file")?));
        }
        Opt::Output => {
          args.output = Some(PathBuf::from(arg.ok_or("Missing output file")?));
        }
        Opt::Verbose => args.verbose = true,
        Opt::Help => {
          print_help();
          std::process::exit(0);
        }
        Opt::Flag => {
          args.flag = arg
            .ok_or("Missing flag value")?
            .parse()
            .map_err(|_| "Invalid flag value")?;
        }
      },
      ParsedOpt::Positional(_, _) => {
        return Err("Positional arguments are not supported".into());
      }
      ParsedOpt::ParseError(err) => {
        return Err(err);
      }
    }
  }

  // Validate args
  if args.img_dir.is_none() && args.input.is_none() {
    return Err("Either -i <file> or --ImgDir <directory> must be specified".into());
  }

  Ok(args)
}

fn print_help() {
  println!("\nThis is the opj_dump utility from the OpenJPEG project.");
  println!("It dumps JPEG 2000 codestream info to stdout or a given file.");
  println!(
    "It has been compiled against openjp2 library v{}.",
    OPJ_VERSION,
  );

  println!("\nParameters:");
  println!("-----------\n");
  println!("  -ImgDir <directory>");
  println!("\tImage file Directory path");
  println!("  -i <compressed file>");
  println!("    REQUIRED only if an Input image directory not specified");
  println!("    Currently accepts J2K-files, JP2-files and JPT-files. The file type");
  println!("    is identified based on its suffix.");
  println!("  -o <output file>");
  println!("    OPTIONAL");
  println!("    Output file where file info will be dump.");
  println!("    By default it will be in the stdout.");
  println!("  -v");
  println!("    OPTIONAL");
  println!("    Enable informative messages");
  println!("    By default verbose mode is off.");
  println!("");
}

#[derive(Debug)]
struct DirContents {
  files: Vec<PathBuf>,
}

impl DirContents {
  fn new(dir_path: &Path) -> std::io::Result<Self> {
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

extern "C" fn info_callback(msg: *const c_char, _data: *mut c_void) {
  unsafe {
    print!("[INFO] {}", CStr::from_ptr(msg).to_string_lossy());
  }
}

extern "C" fn warning_callback(msg: *const c_char, _data: *mut c_void) {
  unsafe {
    print!("[WARNING] {}", CStr::from_ptr(msg).to_string_lossy());
  }
}

extern "C" fn error_callback(msg: *const c_char, _data: *mut c_void) {
  unsafe {
    print!("[ERROR] {}", CStr::from_ptr(msg).to_string_lossy());
  }
}

fn process_file(
  input: &Path,
  output: Option<&Path>,
  verbose: bool,
  flag: u32,
) -> Result<(), String> {
  unsafe {
    // Set up decoder parameters
    let mut parameters = opj_dparameters::default();
    parameters.m_verbose = verbose as i32;

    // Determine input format
    let format = detect_format_from_file(input)?;
    //eprintln!("Format: {:?}", format);

    // Set up output
    if let Some(output_path) = output {
      let output_c = CString::new(output_path.to_str().ok_or("Invalid output path")?)
        .map_err(|_| "Invalid output path")?;
      let outfile = parameters.outfile.as_mut_ptr();
      std::ptr::copy_nonoverlapping(
        output_c.as_ptr(),
        outfile,
        output_c.as_bytes().len().min(OPJ_PATH_LEN as usize - 1),
      );
    }

    // Create stream
    let mut stream = Stream::new_file(input, 1_000_000, true)
      .map_err(|e| format!("Failed to create stream: {}", e))?;

    // Create decompression codec
    let cod_format = match format {
      J2KFormat::J2K => OPJ_CODEC_J2K,
      J2KFormat::JP2 => OPJ_CODEC_JP2,
      _ => {
        return Err("Unknown codec format".into());
      }
    };
    let mut codec = Codec::new_decoder(cod_format).ok_or("Failed to create codec")?;

    // Open output file.
    let output_file = if output.is_some() {
      libc::fopen(parameters.outfile.as_ptr(), b"w\0".as_ptr() as *const i8)
    } else {
      libc::fdopen(libc::STDOUT_FILENO, b"w\0".as_ptr() as *const i8)
    };

    /* catch events using our callbacks and give a local context */
    codec.set_info_handler(Some(info_callback), ptr::null_mut());
    codec.set_warning_handler(Some(warning_callback), ptr::null_mut());
    codec.set_error_handler(Some(error_callback), ptr::null_mut());

    parameters.flags |= OPJ_DPARAMETERS_DUMP_FLAG;

    let status = codec.setup_decoder(&mut parameters);
    if status == 0 {
      return Err("Failed to setup decoder".into());
    }

    // Decode image header and create image.
    let _image = codec
      .read_header(&mut stream)
      .ok_or("Failed to read header")?;

    // Dump codec info
    codec.dump_codec(flag as i32, output_file);
    libc::fflush(output_file);

    Ok(())
  }
}

pub fn run_dump(args: Vec<String>) -> Result<(), String> {
  let args = parse_args(args)?;

  if let Some(img_dir) = args.img_dir {
    // Process directory
    let dir_contents =
      DirContents::new(&img_dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for (i, file) in dir_contents.files.iter().enumerate() {
      eprintln!();
      eprintln!("File Number {} {:?}", i, file.file_name().unwrap());
      if detect_format_from_file(&file).is_ok() {
        process_file(&file, args.output.as_deref(), args.verbose, args.flag)?;
      } else {
        eprintln!("skipping file...");
      }
    }
  } else if let Some(input) = args.input {
    // Process single file
    process_file(&input, args.output.as_deref(), args.verbose, args.flag)?;
  }

  Ok(())
}
