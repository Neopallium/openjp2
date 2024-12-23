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

    TIFFGetField(tiff, TIFFTAG_IMAGEWIDTH, &mut width);
    TIFFGetField(tiff, TIFFTAG_IMAGELENGTH, &mut height);
    TIFFGetField(tiff, TIFFTAG_BITSPERSAMPLE, &mut bps);
    TIFFGetField(tiff, TIFFTAG_SAMPLESPERPIXEL, &mut spp);

    println!("TIFF File: {}", args[1]);
    println!("Dimensions: {} x {}", width, height);
    println!("Bits per sample: {}", bps);
    println!("Samples per pixel: {}", spp);

    TIFFClose(tiff);
  }
}
