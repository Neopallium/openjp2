//! libFuzzer target for the J2K/JP2 decoders.
//!
//! Every input is decoded in three modes - lenient (progressive-friendly),
//! strict, and limit-constrained (`DecodeLimits`).  None may panic or hang:
//! the decoders must reject malformed, truncated or malicious data
//! gracefully (the limit checks run before any allocation sized from the
//! codestream).
//!
//! Structure awareness (review follow-up): a custom mutator
//! (`LLVMFuzzerCustomMutator`, see `fuzz_mutator!` below) parses the J2K
//! marker / JP2 box layout and rewrites header fields (SIZ geometry, Csiz,
//! COD counts, Psot, IHDR dims) with extreme values while keeping the
//! containing length field consistent, so mutations reach the deep handlers
//! instead of being rejected at the framing checks.  Roughly half the
//! mutations fall back to libFuzzer's own byte-level mutator (fed by
//! `fuzz_decode.dict`).
//!
//! Run: `cargo fuzz run fuzz_decode -- -max_len=131072 -dict=fuzz_decode.dict`
//! (ALWAYS cap the time with `-max_total_time`.)
#![no_main]
use libfuzzer_sys::{fuzz_mutator, fuzz_target};

use openjp2::{DecodeLimits, DecodeParameters, Image};

fuzz_target!(|data: &[u8]| {
  let _ = Image::from_bytes_with(data, DecodeParameters::new().strict(false));
  let _ = Image::from_bytes_with(data, DecodeParameters::new().strict(true));
  let limits = DecodeLimits::new()
    .max_image_size(2048, 2048)
    .max_tile_pixels(2048 * 2048)
    .max_mem_alloc(32 * 1024 * 1024)
    .max_metadata_len(1 << 20);
  let _ = Image::from_bytes_with(data, DecodeParameters::new().strict(false).limits(limits));
});

/// Extreme values biased toward length/count/geometry fields.
const EXTREMES: [u32; 12] = [
  0,
  1,
  2,
  3,
  7,
  0x7f,
  0x80,
  0x7fff,
  0x8000,
  0x7fffffff,
  0x80000000,
  0xffffffff,
];

fn put_be(buf: &mut [u8], value: u32, bytes: usize) {
  let be = value.to_be_bytes();
  buf[..bytes].copy_from_slice(&be[4 - bytes..]);
}

fn pick(seed: u32) -> u32 {
  EXTREMES[(seed as usize) % EXTREMES.len()]
}

/// J2K marker fields the mutator rewrites.  `off` is the field offset within
/// the marker BODY; `len` its byte length.
struct Field {
  #[allow(dead_code)]
  name: &'static str,
  off: usize,
  len: usize,
}

const SIZ_FIELDS: &[Field] = &[
  Field { name: "Xsiz", off: 2, len: 4 },
  Field { name: "Ysiz", off: 6, len: 4 },
  Field { name: "XOsiz", off: 10, len: 4 },
  Field { name: "YOsiz", off: 14, len: 4 },
  Field { name: "XTsiz", off: 18, len: 4 },
  Field { name: "YTsiz", off: 22, len: 4 },
  Field { name: "XTOsiz", off: 26, len: 4 },
  Field { name: "YTOsiz", off: 30, len: 4 },
  Field { name: "Ssiz", off: 36, len: 1 }, // first component's precision
  Field { name: "XRsiz", off: 37, len: 1 },
  Field { name: "YRsiz", off: 38, len: 1 },
];

const COD_FIELDS: &[Field] = &[
  Field { name: "numlayers", off: 2, len: 2 },
  Field { name: "numdecomp", off: 5, len: 1 },
  Field { name: "cblkw", off: 6, len: 1 },
  Field { name: "cblkh", off: 7, len: 1 },
  Field { name: "cblksty", off: 8, len: 1 },
];

const SOT_FIELDS: &[Field] = &[Field { name: "Psot", off: 2, len: 4 }];
const QCD_FIELDS: &[Field] = &[Field { name: "Sqcd", off: 0, len: 1 }];

fn j2k_fields(marker: u16) -> &'static [Field] {
  match marker {
    0xff51 => SIZ_FIELDS,
    0xff52 | 0xff53 => COD_FIELDS,
    0xff5c | 0xff5d => QCD_FIELDS,
    0xff90 => SOT_FIELDS,
    _ => &[],
  }
}

/// Rewrites one header field of a J2K codestream in place; returns the new
/// size (== `size`).  Only fixed-size fields are touched, so the marker
/// lengths stay valid and the mutation reaches the field's handler.
fn mutate_j2k_field(data: &mut [u8], size: usize, seed: u32) -> usize {
  if size < 6 || data[0] != 0xff || data[1] != 0x4f {
    return size; // not a J2K codestream
  }
  // Collect (marker, body offset, body len) for the main header + tile parts.
  let mut markers: Vec<(u16, usize, usize)> = Vec::new();
  let mut pos = 2usize;
  let mut guard = 0usize;
  while pos + 4 <= size && guard < 64 {
    if data[pos] != 0xff {
      break;
    }
    let marker = u16::from_be_bytes([data[pos], data[pos + 1]]);
    let len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
    if len < 2 {
      break;
    }
    let body = pos + 4;
    if body + len - 2 > size {
      break;
    }
    markers.push((marker, body, len - 2));
    if marker == 0xff93 {
      break; // SOD: tile data follows
    }
    pos = body + len - 2;
    guard += 1;
  }
  if markers.is_empty() {
    return size;
  }
  // Choose a marker with mutable fields, then one of its fields.
  let mut candidates: Vec<(usize, &Field)> = Vec::new();
  for (i, (marker, _body, body_len)) in markers.iter().enumerate() {
    for f in j2k_fields(*marker) {
      if f.off + f.len <= *body_len {
        candidates.push((i, f));
      }
    }
  }
  if candidates.is_empty() {
    return size;
  }
  let (marker_idx, field) = candidates[(seed as usize) % candidates.len()];
  let (marker, body, _body_len) = markers[marker_idx];
  let value = pick(seed >> 3);
  // For the 1-byte count/exp fields, use the low byte of the extreme value.
  let bytes = field.len.min(4);
  put_be(&mut data[body + field.off..], value, bytes);
  let _ = marker;
  size
}

/// Rewrites one IHDR field of a JP2 file in place; returns the new size.
fn mutate_jp2_field(data: &mut [u8], size: usize, seed: u32) -> usize {
  // Look for the IHDR box: 8-byte header (len, 'ihdr') then a 14-byte body.
  let mut pos = 0usize;
  while pos + 22 <= size {
    let len = u32::from_be_bytes([
      data[pos],
      data[pos + 1],
      data[pos + 2],
      data[pos + 3],
    ]) as usize;
    let ty = &data[pos + 4..pos + 8];
    if len < 8 || pos + len > size {
      return size;
    }
    if ty == b"ihdr" && len >= 22 {
      let body = pos + 8;
      let fields: [(usize, usize); 4] = [(0, 4), (4, 4), (8, 2), (10, 1)]; // h, w, nc, bpc
      let (off, len_field) = fields[(seed as usize) % 4];
      let value = pick(seed >> 2);
      let bytes = len_field.min(4);
      put_be(&mut data[body + off..], value, bytes);
      return size;
    }
    pos += len;
  }
  size
}

fuzz_mutator!(|data: &mut [u8], size: usize, max_size: usize, seed: u32| {
  // Half of the mutations use libFuzzer's own byte-level strategy (which the
  // dictionary below feeds); the other half are structure-aware field
  // rewrites that stay inside the marker/box framing.
  if seed & 1 == 0 {
    return libfuzzer_sys::fuzzer_mutate(data, size, max_size);
  }
  let size = if seed & 2 == 0 {
    mutate_j2k_field(data, size, seed)
  } else {
    mutate_jp2_field(data, size, seed)
  };
  size
});
