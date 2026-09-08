//! Integration tests using the safe Rust API (no C FFI).
//!
//! These tests exercise the pure-Rust allocator path when run with:
//!   cargo test --no-default-features --test rust_api
//!
//! They also work with default features (c_api) for comparison:
//!   cargo test --test rust_api

use std::io::Cursor;

use openjp2::*;

/// Helper: decode Hadley_Crater.jp2 via the safe Rust API and return the image.
fn decode_hadley_crater() -> Box<opj_image> {
    let bytes = std::fs::read("samples/Hadley_Crater.jp2").expect("read sample file");
    let len = bytes.len() as u64;
    let cursor = Cursor::new(bytes);
    let mut stream = Stream::from_reader(cursor, len);

    let mut codec = Codec::new_decoder(OPJ_CODEC_JP2).expect("create decoder");
    let mut params = opj_dparameters_t::default();
    assert_eq!(codec.setup_decoder(&mut params), 1, "setup_decoder failed");

    let mut image = codec.read_header(&mut stream).expect("read_header failed");
    assert_eq!(codec.decode(&mut stream, &mut image), 1, "decode failed");
    assert_eq!(
        codec.end_decompress(&mut stream),
        1,
        "end_decompress failed"
    );
    image
}

#[test]
fn decode_header_metadata() {
    let image = decode_hadley_crater();

    assert_eq!(image.x0, 0);
    assert_eq!(image.y0, 0);
    assert_eq!(image.x1, 1920);
    assert_eq!(image.y1, 1088);
    assert_eq!(image.numcomps, 3);
    assert_eq!(image.color_space, OPJ_CLRSPC_SRGB);
}

#[test]
fn decode_component_properties() {
    let image = decode_hadley_crater();
    let comps = image.comps().expect("components should be present");
    assert_eq!(comps.len(), 3);

    for (i, comp) in comps.iter().enumerate() {
        assert_eq!(comp.w, 1920, "comp[{i}] width");
        assert_eq!(comp.h, 1088, "comp[{i}] height");
        assert_eq!(comp.prec, 8, "comp[{i}] precision");
        assert_eq!(comp.sgnd, 0, "comp[{i}] signed");
        assert_eq!(comp.dx, 1, "comp[{i}] dx");
        assert_eq!(comp.dy, 1, "comp[{i}] dy");
    }
}

#[test]
fn decode_pixel_values() {
    let image = decode_hadley_crater();
    let comps = image.comps().expect("components");

    // Reference pixel values verified against both c_api and pure-Rust paths.
    // Format: [comp][position] where positions are:
    //   top-left, top-right, bottom-left, bottom-right, center
    let expected: [[i32; 5]; 3] = [
        [142, 160, 89, 83, 72],  // R
        [77, 123, 44, 44, 20],   // G
        [28, 74, 8, 29, 0],      // B
    ];

    for (i, comp) in comps.iter().enumerate() {
        let data = comp.data().expect("pixel data");
        let w = comp.w as usize;
        let h = comp.h as usize;

        assert_eq!(data[0], expected[i][0], "comp[{i}] top-left");
        assert_eq!(data[w - 1], expected[i][1], "comp[{i}] top-right");
        assert_eq!(data[(h - 1) * w], expected[i][2], "comp[{i}] bottom-left");
        assert_eq!(data[h * w - 1], expected[i][3], "comp[{i}] bottom-right");
        assert_eq!(data[(h / 2) * w + w / 2], expected[i][4], "comp[{i}] center");
    }
}

#[test]
fn decode_from_bytes_matches_format_detection() {
    let bytes = std::fs::read("samples/Hadley_Crater.jp2").expect("read sample file");
    let format = openjp2::detect_format(&bytes).expect("detect format");
    assert_eq!(format, J2KFormat::JP2);
}

#[test]
fn decode_pixel_data_nonzero() {
    // Verify the decoded image contains meaningful data (not all zeros).
    let image = decode_hadley_crater();
    let comps = image.comps().expect("components");

    for (i, comp) in comps.iter().enumerate() {
        let data = comp.data().expect("pixel data");
        let nonzero_count = data.iter().filter(|&&v| v != 0).count();
        let total = data.len();
        // A real image should have a significant fraction of non-zero pixels
        assert!(
            nonzero_count > total / 4,
            "comp[{}]: only {}/{} non-zero pixels — data may be corrupt",
            i, nonzero_count, total
        );
    }
}

#[test]
fn decode_reduced_resolution() {
    let bytes = std::fs::read("samples/Hadley_Crater.jp2").expect("read sample file");
    let len = bytes.len() as u64;
    let cursor = Cursor::new(bytes);
    let mut stream = Stream::from_reader(cursor, len);

    let mut codec = Codec::new_decoder(OPJ_CODEC_JP2).expect("create decoder");
    let mut params = opj_dparameters_t::default();
    params.cp_reduce = 2; // Decode at 1/4 resolution
    assert_eq!(codec.setup_decoder(&mut params), 1);

    let mut image = codec.read_header(&mut stream).expect("read header");
    assert_eq!(codec.decode(&mut stream, &mut image), 1);
    assert_eq!(codec.end_decompress(&mut stream), 1);

    let comps = image.comps().expect("components");
    // At reduce=2, dimensions should be roughly 1/4 of original
    for comp in comps.iter() {
        assert!(comp.w > 0 && comp.w <= 1920 / 3, "reduced width={}", comp.w);
        assert!(comp.h > 0 && comp.h <= 1088 / 3, "reduced height={}", comp.h);
        assert!(comp.data().is_some(), "reduced resolution should have data");
    }
}
