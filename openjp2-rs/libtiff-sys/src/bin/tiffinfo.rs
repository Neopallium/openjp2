#![allow(non_snake_case)]
use libtiff_sys::*;
use std::env;
use std::ffi::CString;

fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() != 2 {
    eprintln!("Usage: {} <tiff_file>", args[0]);
    std::process::exit(1);
  }

  let filename = CString::new(args[1].as_str()).unwrap();
  unsafe {
    let tiff = TIFFOpen(filename.as_ptr(), "r\0".as_ptr() as *const i8);
    if tiff.is_null() {
      eprintln!("Failed to open TIFF file: {}", args[1]);
      std::process::exit(1);
    }

    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut bps: u16 = 0;
    let mut spp: u16 = 0;
    let mut sample_format: u16 = 0;
    let mut photometric: u16 = 0;
    let mut planar_config: u16 = 0;

    TIFFGetField(tiff, TIFFTAG_IMAGEWIDTH, &mut width);
    TIFFGetField(tiff, TIFFTAG_IMAGELENGTH, &mut height);
    TIFFGetField(tiff, TIFFTAG_BITSPERSAMPLE, &mut bps);
    TIFFGetField(tiff, TIFFTAG_SAMPLESPERPIXEL, &mut spp);
    TIFFGetField(tiff, TIFFTAG_SAMPLEFORMAT, &mut sample_format);
    TIFFGetField(tiff, TIFFTAG_PHOTOMETRIC, &mut photometric);
    TIFFGetField(tiff, TIFFTAG_PLANARCONFIG, &mut planar_config);

    println!("TIFF File: {}", args[1]);
    println!("Dimensions: {} x {}", width, height);
    println!("Bits per sample: {}", bps);
    println!("Samples per pixel: {}", spp);
    println!(
      "Sample format: {}",
      match sample_format as u32 {
        SAMPLEFORMAT_UINT => "Unsigned integer",
        SAMPLEFORMAT_INT => "Signed integer",
        SAMPLEFORMAT_IEEEFP => "IEEE floating point",
        SAMPLEFORMAT_VOID => "Unspecified",
        _ => "Unknown",
      }
    );
    println!(
      "Photometric interpretation: {}",
      match photometric as u32 {
        PHOTOMETRIC_MINISWHITE => "Min is white",
        PHOTOMETRIC_MINISBLACK => "Min is black",
        PHOTOMETRIC_RGB => "RGB",
        PHOTOMETRIC_PALETTE => "Palette",
        PHOTOMETRIC_MASK => "Mask",
        PHOTOMETRIC_SEPARATED => "Separated",
        PHOTOMETRIC_YCBCR => "YCbCr",
        PHOTOMETRIC_CIELAB => "CIE L*a*b*",
        PHOTOMETRIC_ICCLAB => "ICC L*a*b*",
        PHOTOMETRIC_ITULAB => "ITU L*a*b*",
        PHOTOMETRIC_LOGL => "LogL",
        PHOTOMETRIC_LOGLUV => "LogLuv",
        _ => "Unknown",
      }
    );
    println!(
      "Planar configuration: {}",
      match planar_config as u32 {
        PLANARCONFIG_CONTIG => "Contiguous",
        PLANARCONFIG_SEPARATE => "Separate",
        _ => "Unknown",
      }
    );

    TIFFClose(tiff);
  }
}
