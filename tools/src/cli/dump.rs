use crate::getopt::{GetOpts, OptDef, ParsedOpt};
use openjp2::{detect_format_from_file, openjpeg::*, Decoder, Stream};
use std::ffi::CString;
use std::path::{Path, PathBuf};

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
  println!();
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

fn process_file(
  input: &Path,
  output: Option<&Path>,
  verbose: bool,
  flag: u32,
) -> Result<(), String> {
  unsafe {
    // Set up decoder parameters
    let mut parameters = opj_dparameters {
      m_verbose: verbose as i32,
      ..Default::default()
    };

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
    let mut codec = Decoder::new(format).map_err(|_| "Failed to create codec")?;

    /* catch events using our callbacks and give a local context */
    codec.set_info_handler(|msg: &str| print!("[INFO] {}", msg));
    codec.set_warning_handler(|msg: &str| print!("[WARNING] {}", msg));
    codec.set_error_handler(|msg: &str| print!("[ERROR] {}", msg));

    parameters.flags |= OPJ_DPARAMETERS_DUMP_FLAG;

    codec
      .apply_settings(&openjp2::DecoderSettings {
        layers: parameters.cp_layer,
        reduce: parameters.cp_reduce,
        ignore_pclr_cmap_cdef: parameters.flags & OPJ_DPARAMETERS_IGNORE_PCLR_CMAP_CDEF_FLAG != 0,
        dump_state: parameters.flags & OPJ_DPARAMETERS_DUMP_FLAG != 0,
        threads: 0,
      })
      .map_err(|_| "Failed to setup decoder")?;

    // Decode image header and create image.
    codec
      .read_header(&mut stream)
      .map_err(|_| "Failed to read header")?;

    // Dump codec info
    if let Some(output) = output {
      // Dump to file
      let mut file = std::fs::File::create(output)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
      codec.dump_codec(flag as i32, &mut file);
    } else {
      // Dump to stdout
      let mut stdout = std::io::stdout();
      codec.dump_codec(flag as i32, &mut stdout);
    }

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
      if detect_format_from_file(file).is_ok() {
        process_file(file, args.output.as_deref(), args.verbose, args.flag)?;
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
