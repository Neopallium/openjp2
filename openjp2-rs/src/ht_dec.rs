use super::event::*;
use super::openjpeg::*;
use super::t1::*;
use super::t1_ht_luts::*;

use core::cell::Cell;
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Copy, Clone)]
pub struct dec_mel<'a> {
  pub data: &'a [OPJ_UINT8],
  /// Index of the next byte to read; the MEL segment grows forward.
  pub pos: usize,
  pub tmp: OPJ_UINT64,
  pub bits: core::ffi::c_int,
  pub size: core::ffi::c_int,
  pub unstuff: OPJ_BOOL,
  pub k: core::ffi::c_int,
  pub num_runs: core::ffi::c_int,
  pub runs: OPJ_UINT64,
}

impl dec_mel<'_> {
  /// Address of the read cursor. The C code aligns the 32-bit reads to a 4-byte boundary, and
  /// how many bytes it consumes one at a time up front depends on it.
  fn addr(&self) -> usize {
    self.data.as_ptr() as usize + self.pos
  }
}
//* ***********************************************************************/
/* * @brief MEL state structure for reading and decoding the MEL bitstream
 *
 *  A number of events is decoded from the MEL bitstream ahead of time
 *  and stored in run/num_runs.
 *  Each run represents the number of zero events before a one event.
 */
pub type dec_mel_t<'a> = dec_mel<'a>;

#[derive(Copy, Clone)]
pub struct rev_struct<'a> {
  pub data: &'a [OPJ_UINT8],
  /// Index of the next byte to read; VLC and MRP segments grow backward, so this walks down and
  /// may end one slot below the segment. Signed to keep that final decrement from underflowing.
  pub pos: isize,
  pub tmp: OPJ_UINT64,
  pub bits: OPJ_UINT32,
  pub size: core::ffi::c_int,
  pub unstuff: OPJ_BOOL,
}

impl rev_struct<'_> {
  fn addr(&self) -> usize {
    (self.data.as_ptr() as usize).wrapping_add_signed(self.pos)
  }

  fn byte(&self) -> OPJ_UINT8 {
    self.data[self.pos as usize]
  }
}
// data decoding machinery
// !<the address of data (or bitstream)
// !<temporary buffer for read data
// !<number of bits stored in tmp
// !<number of bytes in MEL code
// !<true if the next bit needs to be unstuffed
// !<state of MEL decoder
// queue of decoded runs
// !<number of decoded runs left in runs (maximum 8)
// !<runs of decoded MEL codewords (7 bits/run)
//* ***********************************************************************/
/* * @brief A structure for reading and unstuffing a segment that grows
 *         backward, such as VLC and MRP
 */
pub type rev_struct_t<'a> = rev_struct<'a>;

#[derive(Copy, Clone)]
pub struct frwd_struct<'a> {
  pub data: &'a [OPJ_UINT8],
  /// Index of the next byte to read; MagSgn and SPP segments grow forward.
  pub pos: usize,
  pub tmp: OPJ_UINT64,
  pub bits: OPJ_UINT32,
  pub unstuff: OPJ_BOOL,
  pub size: core::ffi::c_int,
  pub X: OPJ_UINT32,
}

impl frwd_struct<'_> {
  fn addr(&self) -> usize {
    self.data.as_ptr() as usize + self.pos
  }
}
//storage
// !<pointer to where to read data
// !<temporary buffer of read data
// !<number of bits stored in tmp
// !<number of bytes left
// !<true if the last byte is more than 0x8F
// !<then the current byte is unstuffed if it is 0x7F
//* ***********************************************************************/
/* * @brief State structure for reading and unstuffing of forward-growing
 *         bitstreams; these are: MagSgn and SPP bitstreams
 */
pub type frwd_struct_t<'a> = frwd_struct<'a>;
// !<pointer to bitstream
// !<temporary buffer of read data
// !<number of bits stored in tmp
// !<true if a bit needs to be unstuffed from next byte
// !<size of data
// !<0 or 0xFF, X's are inserted at end of bitstream
//* **************************************************************************/
// This software is released under the 2-Clause BSD license, included
// below.
//
// Copyright (c) 2021, Aous Naman
// Copyright (c) 2021, Kakadu Software Pty Ltd, Australia
// Copyright (c) 2021, The University of New South Wales, Australia
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//* **************************************************************************/
// This file is part of the OpenJpeg software implementation.
// File: ht_dec.c
// Author: Aous Naman
// Date: 01 September 2021
//* **************************************************************************/
//* **************************************************************************/
/* * @file ht_dec.c
 *  @brief implements HTJ2K block decoder
 */
// ///////////////////////////////////////////////////////////////////////////
// compiler detection
// ///////////////////////////////////////////////////////////////////////////
//* ***********************************************************************/
/* * @brief Displays the error message for disabling the decoding of SPP and
 * MRP passes
 */
/// Process-wide "message shown once" flag (the C code used a `static` local). An atomic keeps the
/// double-checked claim logic without mutable-static UB.
static ONLY_CLEANUP_PASS_DECODED: AtomicBool = AtomicBool::new(false);
//* ***********************************************************************/
/* * @brief Generates population count (i.e., the number of set bits)
 *
 *   @param [in]  val is the value for which population count is sought
 */
fn population_count(mut val: OPJ_UINT32) -> OPJ_UINT32 {
  val.count_ones()
}
//* ***********************************************************************/
/* * @brief Counts the number of leading zeros
 *
 *   @param [in]  val is the value for which leading zero count is sought
 */
fn count_leading_zeros(mut val: OPJ_UINT32) -> OPJ_UINT32 {
  val.leading_zeros()
}
//* ***********************************************************************/
/* * @brief Read a little-endian serialized UINT32.
 *
 *   @param [in]  data is the byte stream to read from
 *   @param [in]  pos is the index of the first of the four bytes
 */
fn read_le_uint32(data: &[OPJ_UINT8], pos: usize) -> OPJ_UINT32 {
  OPJ_UINT32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
}

//* ***********************************************************************/
/* * @brief Reads and unstuffs the MEL bitstream
 *
 *  This design needs more bytes in the codeblock buffer than the length
 *  of the cleanup pass by up to 2 bytes.
 *
 *  Unstuffing removes the MSB of the byte following a byte whose
 *  value is 0xFF; this prevents sequences larger than 0xFF7F in value
 *  from appearing the bitstream.
 *
 *  @param [in]  melp is a pointer to dec_mel_t structure
 */
fn mel_read(melp: &mut dec_mel_t) {
  let mut val: OPJ_UINT32 = 0;
  let mut bits: core::ffi::c_int = 0;
  let mut t: OPJ_UINT32 = 0;
  let mut unstuff: OPJ_BOOL = 0;
  if melp.bits > 32i32 {
    //there are enough bits in the tmp variable
    return;
    // return without reading new data
  } // feed in 0xFF if buffer is exhausted
  val = 0xffffffffu32;
  if melp.size > 4i32 {
    // if there is more than 4 bytes the MEL segment
    val = read_le_uint32(melp.data, melp.pos); // read 32 bits from MEL data
    melp.pos += 4; // advance pointer
    melp.size -= 4i32 // reduce counter
  } else if melp.size > 0i32 {
    // 4 or less
    let mut m: OPJ_UINT32 = 0; // read one byte at a time
    let mut v: OPJ_UINT32 = 0; // mask of location
    let mut i = 0i32; // put byte in its correct location
    while melp.size > 1i32 {
      let mut v_0 = melp.data[melp.pos] as OPJ_UINT32;
      melp.pos += 1;
      let mut m_0 = !((0xffu32) << i);
      val = val & m_0 | v_0 << i;
      melp.size -= 1;
      i += 8i32
    }
    // size equal to 1
    // the one before the last is different
    // MEL and VLC segments can overlap
    v = melp.data[melp.pos] as OPJ_UINT32;
    melp.pos += 1;
    v |= 0xfu32;
    m = !((0xffu32) << i);
    val = val & m | v << i;
    melp.size -= 1
  }
  // next we unstuff them before adding them to the buffer
  bits = 32i32 - melp.unstuff; // number of bits in val, subtract 1 if
                               // the previously read byte requires
                               // unstuffing
                               // data is unstuffed and accumulated in t
                               // bits has the number of bits in t
  t = val & 0xffu32; // true if the byte needs unstuffing
  unstuff = (val & 0xffu32 == 0xffu32) as core::ffi::c_int; // there is one less bit in t if unstuffing is needed
  bits -= unstuff; // move up to make room for the next byte
  t <<= 8i32 - unstuff;
  //this is a repeat of the above
  t |= val >> 8i32 & 0xffu32;
  unstuff = (val >> 8i32 & 0xffu32 == 0xffu32) as core::ffi::c_int;
  bits -= unstuff;
  t <<= 8i32 - unstuff;
  t |= val >> 16i32 & 0xffu32;
  unstuff = (val >> 16i32 & 0xffu32 == 0xffu32) as core::ffi::c_int;
  bits -= unstuff;
  t <<= 8i32 - unstuff;
  t |= val >> 24i32 & 0xffu32;
  melp.unstuff = (val >> 24i32 & 0xffu32 == 0xffu32) as core::ffi::c_int;
  // move t to tmp, and push the result all the way up, so we read from
  // the MSB
  melp.tmp |= (t as OPJ_UINT64) << (64i32 - bits - melp.bits);
  melp.bits += bits;
  //increment the number of bits in tmp
}
//* ***********************************************************************/
/* * @brief Decodes unstuffed MEL segment bits stored in tmp to runs
 *
 *  Runs are stored in "runs" and the number of runs in "num_runs".
 *  Each run represents a number of zero events that may or may not
 *  terminate in a 1 event.
 *  Each run is stored in 7 bits.  The LSB is 1 if the run terminates in
 *  a 1 event, 0 otherwise.  The next 6 bits, for the case terminating
 *  with 1, contain the number of consecutive 0 zero events * 2; for the
 *  case terminating with 0, they store (number of consecutive 0 zero
 *  events - 1) * 2.
 *  A total of 6 bits (made up of 1 + 5) should have been enough.
 *
 *  @param [in]  melp is a pointer to dec_mel_t structure
 */
fn mel_decode(melp: &mut dec_mel_t) {
  const mel_exp: [core::ffi::c_int; 13] = [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 5];
  if melp.bits < 6i32 {
    // if there are less than 6 bits in tmp
    mel_read(melp);
    // then read from the MEL bitstream
  }
  // 6 bits is the largest decodable MEL cwd
  //repeat so long that there is enough decodable bits in tmp,
  // and the runs store is not full (num_runs < 8)
  while melp.bits >= 6i32 && melp.num_runs < 8i32 {
    let mut eval = mel_exp[melp.k as usize]; // number of bits associated with state
    let mut run = 0i32;
    if melp.tmp & (1u64) << 63i32 != 0 {
      //The next bit to decode (stored in MSB)
      //one is found
      run = (1i32) << eval;
      // a stretch of zeros not terminating in one
      run -= 1; // consecutive runs of 0 events - 1
      melp.k = if (melp.k + 1i32) < 12i32 {
        (melp.k) + 1i32
      } else {
        12i32
      }; //increment, max is 12
      melp.tmp <<= 1i32; // consume one bit from tmp
      melp.bits -= 1i32;
      run <<= 1i32
    } else {
      //0 is found
      run = (melp.tmp >> (63i32 - eval)) as core::ffi::c_int & (((1i32) << eval) - 1i32);
      // a stretch of zeros terminating with one
      melp.k = if melp.k - 1i32 > 0i32 {
        (melp.k) - 1i32
      } else {
        0i32
      }; //decrement, min is 0
      melp.tmp <<= eval + 1i32; //consume eval + 1 bits (max is 6)
      melp.bits -= eval + 1i32; // 7 bits per run
      run = (run << 1i32) + 1i32
    } // 6 bits are sufficient
    eval = melp.num_runs * 7i32; // store the value in runs
    melp.runs &= !((0x3f as OPJ_UINT64) << eval);
    melp.runs |= (run as OPJ_UINT64) << eval;
    melp.num_runs += 1
  }
}
//* ***********************************************************************/
/* * @brief Initiates a dec_mel_t structure for MEL decoding and reads
 *         some bytes in order to get the read address to a multiple
 *         of 4
 *
 *  @param [in]  melp is a pointer to dec_mel_t structure
 *  @param [in]  bbuf is a pointer to byte buffer
 *  @param [in]  lcup is the length of MagSgn+MEL+VLC segments
 *  @param [in]  scup is the length of MEL+VLC segments
 */
fn mel_init<'a>(
  melp: &mut dec_mel_t<'a>,
  bbuf: &'a [OPJ_UINT8],
  mut lcup: core::ffi::c_int,
  mut scup: core::ffi::c_int,
) -> bool {
  let mut num: core::ffi::c_int = 0; // move the pointer to the start of MEL
  let mut i: core::ffi::c_int = 0; // 0 bits in tmp
  melp.data = bbuf;
  melp.pos = (lcup - scup) as usize; //
  melp.bits = 0i32; // no unstuffing
  melp.tmp = 0 as OPJ_UINT64; // size is the length of MEL+VLC-1
  melp.unstuff = 0i32; // 0 for state
  melp.size = scup - 1i32; // num_runs is 0
  melp.k = 0i32; //
  melp.num_runs = 0i32;
  melp.runs = 0 as OPJ_UINT64;
  //This code is borrowed; original is for a different architecture
  //These few lines take care of the case where data is not at a multiple
  // of 4 boundary.  It reads 1,2,3 up to 4 bytes from the MEL segment
  num = 4i32 - (melp.addr() & 0x3) as core::ffi::c_int;
  i = 0i32;
  while i < num {
    // this code is similar to mel_read
    let mut d: OPJ_UINT64 = 0; // if buffer is consumed
    let mut d_bits: core::ffi::c_int = 0;
    if melp.unstuff != 0i32 && melp.data[melp.pos] as core::ffi::c_int > 0x8fi32 {
      return false;
    }
    d = if melp.size > 0i32 {
      melp.data[melp.pos] as core::ffi::c_int
    } else {
      0xffi32
    } as OPJ_UINT64;
    // set data to 0xFF
    if melp.size == 1i32 {
      d |= 0xfu64
      //if this is MEL+VLC-1, set LSBs to 0xF
    }
    // see the standard
    let fresh2 = melp.size; //increment if the end is not reached
    melp.size -= 1; //if unstuffing is needed, reduce by 1
    melp.pos += (fresh2 > 0i32) as usize; //store bits in tmp
    d_bits = 8i32 - melp.unstuff; //increment tmp by number of bits
    melp.tmp = melp.tmp << d_bits | d;
    melp.bits += d_bits;
    melp.unstuff = (d & 0xffu64 == 0xffu64) as core::ffi::c_int;
    i += 1
  }
  melp.tmp <<= 64i32 - melp.bits;
  //push all the way up so the first bit
  // is the MSB
  true
}
//* ***********************************************************************/
/* * @brief Retrieves one run from dec_mel_t; if there are no runs stored
 *         MEL segment is decoded
 *
 * @param [in]  melp is a pointer to dec_mel_t structure
 */
fn mel_get_run(melp: &mut dec_mel_t) -> core::ffi::c_int {
  let mut t: core::ffi::c_int = 0;
  if melp.num_runs == 0i32 {
    //if no runs, decode more bit from MEL segment
    mel_decode(melp); //retrieve one run
  } // remove the retrieved run
  t = (melp.runs & 0x7fu64) as core::ffi::c_int;
  melp.runs >>= 7i32;
  melp.num_runs -= 1;
  t
  // return run
}
//* ***********************************************************************/
/* * @brief Read and unstuff data from a backwardly-growing segment
 *
 *  This reader can read up to 8 bytes from before the VLC segment.
 *  Care must be taken not read from unreadable memory, causing a
 *  segmentation fault.
 *
 *  Note that there is another subroutine rev_read_mrp that is slightly
 *  different.  The other one fills zeros when the buffer is exhausted.
 *  This one basically does not care if the bytes are consumed, because
 *  any extra data should not be used in the actual decoding.
 *
 *  Unstuffing is needed to prevent sequences more than 0xFF8F from
 *  appearing in the bits stream; since we are reading backward, we keep
 *  watch when a value larger than 0x8F appears in the bitstream.
 *  If the byte following this is 0x7F, we unstuff this byte (ignore the
 *  MSB of that byte, which should be 0).
 *
 *  @param [in]  vlcp is a pointer to rev_struct_t structure
 */
fn rev_read(vlcp: &mut rev_struct_t) {
  let mut val: OPJ_UINT32 = 0;
  let mut tmp: OPJ_UINT32 = 0;
  let mut bits: OPJ_UINT32 = 0;
  let mut unstuff: OPJ_BOOL = 0;
  //process 4 bytes at a time
  if vlcp.bits > 32u32 {
    // if there are more than 32 bits in tmp, then
    return;
    // reading 32 bits can overflow vlcp->tmp
  }
  val = 0 as OPJ_UINT32;
  //the next line (the if statement) needs to be tested first
  if vlcp.size > 3i32 {
    // if there are more than 3 bytes left in VLC
    // (pos - 3) moves back to read 32 bits at once
    val = read_le_uint32(vlcp.data, (vlcp.pos - 3) as usize); // then read 32 bits
    vlcp.pos -= 4; // move data pointer back by 4
    vlcp.size -= 4i32 // reduce available byte by 4
  } else if vlcp.size > 0i32 {
    // 4 or less
    let mut i = 24i32; // read one byte at a time
    while vlcp.size > 0i32 {
      let mut v = vlcp.byte() as OPJ_UINT32; // put byte in its correct location
      vlcp.pos -= 1;
      val |= v << i;
      vlcp.size -= 1;
      i -= 8i32
    }
  }
  //accumulate in tmp, number of bits in tmp are stored in bits
  tmp = val >> 24i32; //start with the MSB byte
                      // test unstuff (previous byte is >0x8F), and this byte is 0x7F
  bits = (8u32).wrapping_sub(if vlcp.unstuff != 0 && val >> 24i32 & 0x7fu32 == 0x7fu32 {
    1u32
  } else {
    0u32
  }); //this is for the next byte
  unstuff = (val >> 24i32 > 0x8fu32) as core::ffi::c_int; //process the next byte
  tmp |= (val >> 16i32 & 0xffu32) << bits;
  bits = (bits as core::ffi::c_uint).wrapping_add((8u32).wrapping_sub(
    if unstuff != 0 && val >> 16i32 & 0x7fu32 == 0x7fu32 {
      1u32
    } else {
      0u32
    },
  )) as OPJ_UINT32;
  unstuff = (val >> 16i32 & 0xffu32 > 0x8fu32) as core::ffi::c_int;
  tmp |= (val >> 8i32 & 0xffu32) << bits;
  bits = (bits as core::ffi::c_uint).wrapping_add((8u32).wrapping_sub(
    if unstuff != 0 && val >> 8i32 & 0x7fu32 == 0x7fu32 {
      1u32
    } else {
      0u32
    },
  )) as OPJ_UINT32;
  unstuff = (val >> 8i32 & 0xffu32 > 0x8fu32) as core::ffi::c_int;
  tmp |= (val & 0xffu32) << bits;
  bits = (bits as core::ffi::c_uint).wrapping_add((8u32).wrapping_sub(
    if unstuff != 0 && val & 0x7fu32 == 0x7fu32 {
      1u32
    } else {
      0u32
    },
  )) as OPJ_UINT32;
  unstuff = (val & 0xffu32 > 0x8fu32) as core::ffi::c_int;
  // now move the read and unstuffed bits into vlcp->tmp
  vlcp.tmp |= (tmp as OPJ_UINT64) << vlcp.bits;
  vlcp.bits = (vlcp.bits as core::ffi::c_uint).wrapping_add(bits) as OPJ_UINT32;
  vlcp.unstuff = unstuff;
  // this for the next read
}
//* ***********************************************************************/
/* * @brief Initiates the rev_struct_t structure and reads a few bytes to
 *         move the read address to multiple of 4
 *
 *  There is another similar rev_init_mrp subroutine.  The difference is
 *  that this one, rev_init, discards the first 12 bits (they have the
 *  sum of the lengths of VLC and MEL segments), and first unstuff depends
 *  on first 4 bits.
 *
 *  @param [in]  vlcp is a pointer to rev_struct_t structure
 *  @param [in]  data is a pointer to byte at the start of the cleanup pass
 *  @param [in]  lcup is the length of MagSgn+MEL+VLC segments
 *  @param [in]  scup is the length of MEL+VLC segments
 */
fn rev_init<'a>(
  vlcp: &mut rev_struct_t<'a>,
  data: &'a [OPJ_UINT8],
  mut lcup: core::ffi::c_int,
  mut scup: core::ffi::c_int,
) {
  let mut d: OPJ_UINT32 = 0;
  let mut num: core::ffi::c_int = 0;
  let mut tnum: core::ffi::c_int = 0;
  let mut i: core::ffi::c_int = 0;
  //first byte has only the upper 4 bits
  vlcp.data = data;
  vlcp.pos = (lcup - 2) as isize;
  //size can not be larger than this, in fact it should be smaller
  vlcp.size = scup - 2i32; // read one byte (this is a half byte)
  d = vlcp.byte() as OPJ_UINT32; // both initialize and set
  vlcp.pos -= 1; //check standard
  vlcp.tmp = (d >> 4i32) as OPJ_UINT64; //this is useful for the next byte
  vlcp.bits = (4i32 - (vlcp.tmp & 7u64 == 7u64) as core::ffi::c_int) as OPJ_UINT32;
  vlcp.unstuff = (d | 0xfu32 > 0x8fu32) as core::ffi::c_int;
  //This code is designed for an architecture that read address should
  // align to the read size (address multiple of 4 if read size is 4)
  //These few lines take care of the case where data is not at a multiple
  // of 4 boundary. It reads 1,2,3 up to 4 bytes from the VLC bitstream.
  // To read 32 bits, read from (vlcp->data - 3)
  num = 1i32 + (vlcp.addr() & 0x3) as core::ffi::c_int;
  tnum = if num < vlcp.size { num } else { vlcp.size };
  i = 0i32;
  while i < tnum {
    let mut d_0: OPJ_UINT64 = 0;
    let mut d_bits: OPJ_UINT32 = 0;
    // for next byte
    d_0 = vlcp.byte() as OPJ_UINT64; // read one byte and move read pointer
    vlcp.pos -= 1;
    d_bits = (8u32).wrapping_sub(if vlcp.unstuff != 0 && d_0 & 0x7fu64 == 0x7fu64 {
      1u32
    } else {
      0u32
    });
    vlcp.tmp |= d_0 << vlcp.bits;
    vlcp.bits = (vlcp.bits as core::ffi::c_uint).wrapping_add(d_bits) as OPJ_UINT32;
    vlcp.unstuff = (d_0 > 0x8fu64) as core::ffi::c_int;
    i += 1
  }
  vlcp.size -= tnum;
  rev_read(vlcp);
  //check if the last byte was >0x8F (unstuff == true) and this is 0x7F
  // move data to vlcp->tmp
  // read another 32 buts
}
//* ***********************************************************************/
/* * @brief Retrieves 32 bits from the head of a rev_struct structure
 *
 *  By the end of this call, vlcp->tmp must have no less than 33 bits
 *
 *  @param [in]  vlcp is a pointer to rev_struct structure
 */
fn rev_fetch(vlcp: &mut rev_struct_t) -> OPJ_UINT32 {
  if vlcp.bits < 32u32 {
    // if there are less then 32 bits, read more
    rev_read(vlcp); // read 32 bits, but unstuffing might reduce this
    if vlcp.bits < 32u32 {
      // if there is still space in vlcp->tmp for 32 bits
      rev_read(vlcp);
      // read another 32
    }
  }
  vlcp.tmp as OPJ_UINT32
  // return the head (bottom-most) of vlcp->tmp
}
//* ***********************************************************************/
/* * @brief Consumes num_bits from a rev_struct structure
 *
 *  @param [in]  vlcp is a pointer to rev_struct structure
 *  @param [in]  num_bits is the number of bits to be removed
 */
fn rev_advance(vlcp: &mut rev_struct_t, mut num_bits: OPJ_UINT32) -> OPJ_UINT32 {
  assert!(num_bits <= vlcp.bits); // remove bits
  vlcp.tmp >>= num_bits; // decrement the number of bits
  vlcp.bits = (vlcp.bits as core::ffi::c_uint).wrapping_sub(num_bits) as OPJ_UINT32;
  vlcp.tmp as OPJ_UINT32
}
//* ***********************************************************************/
/* * @brief Reads and unstuffs from rev_struct
 *
 *  This is different than rev_read in that this fills in zeros when the
 *  the available data is consumed.  The other does not care about the
 *  values when all data is consumed.
 *
 *  See rev_read for more information about unstuffing
 *
 *  @param [in]  mrp is a pointer to rev_struct structure
 */
fn rev_read_mrp(mrp: &mut rev_struct_t) {
  let mut val: OPJ_UINT32 = 0;
  let mut tmp: OPJ_UINT32 = 0;
  let mut bits: OPJ_UINT32 = 0;
  let mut unstuff: OPJ_BOOL = 0;
  //process 4 bytes at a time
  if mrp.bits > 32u32 {
    return;
  }
  val = 0 as OPJ_UINT32;
  if mrp.size > 3i32 {
    // If there are 3 byte or more
    // (pos - 3) moves back to read 32 bits at once
    val = read_le_uint32(mrp.data, (mrp.pos - 3) as usize); // read 32 bits
    mrp.pos -= 4; // move back pointer
    mrp.size -= 4i32 // reduce count
  } else if mrp.size > 0i32 {
    let mut i = 24i32; // read one byte at a time
    while mrp.size > 0i32 {
      let mut v = mrp.byte() as OPJ_UINT32; // put byte in its correct location
      mrp.pos -= 1;
      val |= v << i;
      mrp.size -= 1;
      i -= 8i32
    }
  }
  //accumulate in tmp, and keep count in bits
  tmp = val >> 24i32;
  //test if the last byte > 0x8F (unstuff must be true) and this is 0x7F
  bits = (8u32).wrapping_sub(if mrp.unstuff != 0 && val >> 24i32 & 0x7fu32 == 0x7fu32 {
    1u32
  } else {
    0u32
  });
  unstuff = (val >> 24i32 > 0x8fu32) as core::ffi::c_int;
  //process the next byte
  tmp |= (val >> 16i32 & 0xffu32) << bits; // move data to mrp pointer
  bits = (bits as core::ffi::c_uint).wrapping_add((8u32).wrapping_sub(
    if unstuff != 0 && val >> 16i32 & 0x7fu32 == 0x7fu32 {
      1u32
    } else {
      0u32
    },
  )) as OPJ_UINT32;
  unstuff = (val >> 16i32 & 0xffu32 > 0x8fu32) as core::ffi::c_int;
  tmp |= (val >> 8i32 & 0xffu32) << bits;
  bits = (bits as core::ffi::c_uint).wrapping_add((8u32).wrapping_sub(
    if unstuff != 0 && val >> 8i32 & 0x7fu32 == 0x7fu32 {
      1u32
    } else {
      0u32
    },
  )) as OPJ_UINT32;
  unstuff = (val >> 8i32 & 0xffu32 > 0x8fu32) as core::ffi::c_int;
  tmp |= (val & 0xffu32) << bits;
  bits = (bits as core::ffi::c_uint).wrapping_add((8u32).wrapping_sub(
    if unstuff != 0 && val & 0x7fu32 == 0x7fu32 {
      1u32
    } else {
      0u32
    },
  )) as OPJ_UINT32;
  unstuff = (val & 0xffu32 > 0x8fu32) as core::ffi::c_int;
  mrp.tmp |= (tmp as OPJ_UINT64) << mrp.bits;
  mrp.bits = (mrp.bits as core::ffi::c_uint).wrapping_add(bits) as OPJ_UINT32;
  mrp.unstuff = unstuff;
  // next byte
}
//* ***********************************************************************/
/* * @brief Initialized rev_struct structure for MRP segment, and reads
 *         a number of bytes such that the next 32 bits read are from
 *         an address that is a multiple of 4. Note this is designed for
 *         an architecture that read size must be compatible with the
 *         alignment of the read address
 *
 *  There is another similar subroutine rev_init.  This subroutine does
 *  NOT skip the first 12 bits, and starts with unstuff set to true.
 *
 *  @param [in]  mrp is a pointer to rev_struct structure
 *  @param [in]  data is a pointer to byte at the start of the cleanup pass
 *  @param [in]  lcup is the length of MagSgn+MEL+VLC segments
 *  @param [in]  len2 is the length of SPP+MRP segments
 */
fn rev_init_mrp<'a>(
  mrp: &mut rev_struct_t<'a>,
  data: &'a [OPJ_UINT8],
  mut lcup: core::ffi::c_int,
  mut len2: core::ffi::c_int,
) {
  let mut num: core::ffi::c_int = 0;
  let mut i: core::ffi::c_int = 0;
  mrp.data = data;
  mrp.pos = (lcup + len2 - 1) as isize;
  mrp.size = len2;
  mrp.unstuff = 1i32;
  mrp.bits = 0 as OPJ_UINT32;
  mrp.tmp = 0 as OPJ_UINT64;
  //This code is designed for an architecture that read address should
  // align to the read size (address multiple of 4 if read size is 4)
  //These few lines take care of the case where data is not at a multiple
  // of 4 boundary.  It reads 1,2,3 up to 4 bytes from the MRP stream
  num = 1i32 + (mrp.addr() & 0x3) as core::ffi::c_int;
  i = 0i32;
  while i < num {
    let mut d: OPJ_UINT64 = 0;
    let mut d_bits: OPJ_UINT32 = 0;
    // for next byte
    let fresh7 = mrp.size;
    mrp.size -= 1;
    d = if fresh7 > 0i32 {
      let b = mrp.byte();
      mrp.pos -= 1;
      b as core::ffi::c_int
    } else {
      0i32
    } as OPJ_UINT64;
    d_bits = (8u32).wrapping_sub(if mrp.unstuff != 0 && d & 0x7fu64 == 0x7fu64 {
      1u32
    } else {
      0u32
    });
    mrp.tmp |= d << mrp.bits;
    mrp.bits = (mrp.bits as core::ffi::c_uint).wrapping_add(d_bits) as OPJ_UINT32;
    mrp.unstuff = (d > 0x8fu64) as core::ffi::c_int;
    i += 1
  }
  rev_read_mrp(mrp);
}
//read a byte, 0 if no more data
//check if unstuffing is needed
// move data to vlcp->tmp
//* ***********************************************************************/
/* * @brief Retrieves 32 bits from the head of a rev_struct structure
 *
 *  By the end of this call, mrp->tmp must have no less than 33 bits
 *
 *  @param [in]  mrp is a pointer to rev_struct structure
 */
fn rev_fetch_mrp(mrp: &mut rev_struct_t) -> OPJ_UINT32 {
  if mrp.bits < 32u32 {
    // if there are less than 32 bits in mrp->tmp
    rev_read_mrp(mrp); // read 30-32 bits from mrp
    if mrp.bits < 32u32 {
      // if there is a space of 32 bits
      rev_read_mrp(mrp);
      // read more
    }
  }
  mrp.tmp as OPJ_UINT32
  // return the head of mrp->tmp
}
//* ***********************************************************************/
/* * @brief Consumes num_bits from a rev_struct structure
 *
 *  @param [in]  mrp is a pointer to rev_struct structure
 *  @param [in]  num_bits is the number of bits to be removed
 */
fn rev_advance_mrp(mrp: &mut rev_struct_t, mut num_bits: OPJ_UINT32) -> OPJ_UINT32 {
  assert!(num_bits <= mrp.bits); // discard the lowest num_bits bits
  mrp.tmp >>= num_bits;
  mrp.bits = (mrp.bits as core::ffi::c_uint).wrapping_sub(num_bits) as OPJ_UINT32;
  mrp.tmp as OPJ_UINT32
  // return data after consumption
}
//* ***********************************************************************/
/* * @brief Decode initial UVLC to get the u value (or u_q)
 *
 *  @param [in]  vlc is the head of the VLC bitstream
 *  @param [in]  mode is 0, 1, 2, 3, or 4. Values in 0 to 3 are composed of
 *               u_off of 1st quad and 2nd quad of a quad pair.  The value
 *               4 occurs when both bits are 1, and the event decoded
 *               from MEL bitstream is also 1.
 *  @param [out] u is the u value (or u_q) + 1.  Note: we produce u + 1;
 *               this value is a partial calculation of u + kappa.
 */
fn decode_init_uvlc(
  mut vlc: OPJ_UINT32,
  mut mode: OPJ_UINT32,
  u: &mut [OPJ_UINT32; 2],
) -> OPJ_UINT32 {
  //table stores possible decoding three bits from vlc
  // there are 8 entries for xx1, x10, 100, 000, where x means do not care
  // table value is made up of
  // 2 bits in the LSB for prefix length
  // 3 bits for suffix length
  // 3 bits in the MSB for prefix value (u_pfx in Table 3 of ITU T.814)
  static dec: [OPJ_UINT8; 8] = [
    (3i32 | (5i32) << 2i32 | (5i32) << 5i32) as OPJ_UINT8,
    (1i32 | (1i32) << 5i32) as OPJ_UINT8,
    (2i32 | (2i32) << 5i32) as OPJ_UINT8,
    (1i32 | (1i32) << 5i32) as OPJ_UINT8,
    (3i32 | (1i32) << 2i32 | (3i32) << 5i32) as OPJ_UINT8,
    (1i32 | (1i32) << 5i32) as OPJ_UINT8,
    (2i32 | (2i32) << 5i32) as OPJ_UINT8,
    (1i32 | (1i32) << 5i32) as OPJ_UINT8,
  ];
  let mut consumed_bits = 0 as OPJ_UINT32;
  if mode == 0u32 {
    // both u_off are 0
    u[1] = 1 as OPJ_UINT32;
    u[0] = u[1]
    //Kappa is 1 for initial line
  } else if mode <= 2u32 {
    // u_off are either 01 or 10
    let mut d: OPJ_UINT32 = 0; //look at the least significant 3 bits
    let mut suffix_len: OPJ_UINT32 = 0; //prefix length
    d = dec[(vlc & 0x7u32) as usize] as OPJ_UINT32; // u value
    vlc >>= d & 0x3u32; // kappa is 1 for initial line
    consumed_bits =
      (consumed_bits as core::ffi::c_uint).wrapping_add(d & 0x3u32) as OPJ_UINT32 as OPJ_UINT32;
    suffix_len = d >> 2i32 & 0x7u32;
    consumed_bits = (consumed_bits as core::ffi::c_uint).wrapping_add(suffix_len) as OPJ_UINT32;
    d = (d >> 5i32).wrapping_add(vlc & ((1u32) << suffix_len).wrapping_sub(1u32));
    u[0] = if mode == 1u32 {
      d.wrapping_add(1u32)
    } else {
      1u32
    };
    u[1] = if mode == 1u32 {
      1u32
    } else {
      d.wrapping_add(1u32)
    }
  } else if mode == 3u32 {
    // both u_off are 1, and MEL event is 0
    let mut d1 = dec[(vlc & 0x7u32) as usize] as OPJ_UINT32; // LSBs of VLC are prefix codeword
    vlc >>= d1 & 0x3u32; // Consume bits
    consumed_bits =
      (consumed_bits as core::ffi::c_uint).wrapping_add(d1 & 0x3u32) as OPJ_UINT32 as OPJ_UINT32;
    if d1 & 0x3u32 > 2u32 {
      let mut suffix_len_0: OPJ_UINT32 = 0;
      //Kappa is 1 for initial line
      u[1] = (vlc & 1u32).wrapping_add(1u32).wrapping_add(1u32);
      consumed_bits = consumed_bits.wrapping_add(1);
      vlc >>= 1i32;
      suffix_len_0 = d1 >> 2i32 & 0x7u32;
      consumed_bits = (consumed_bits as core::ffi::c_uint).wrapping_add(suffix_len_0) as OPJ_UINT32;
      d1 = (d1 >> 5i32).wrapping_add(vlc & ((1u32) << suffix_len_0).wrapping_sub(1u32));
      u[0] = d1.wrapping_add(1u32)
    } else {
      let mut d2: OPJ_UINT32 = 0;
      let mut suffix_len_1: OPJ_UINT32 = 0;
      //u_{q_2} prefix
      //Kappa is 1 for initial line
      // u value
      //Kappa is 1 for initial line
      d2 = dec[(vlc & 0x7u32) as usize] as OPJ_UINT32; // LSBs of VLC are prefix codeword
      vlc >>= d2 & 0x3u32; // Consume bits
      consumed_bits =
        (consumed_bits as core::ffi::c_uint).wrapping_add(d2 & 0x3u32) as OPJ_UINT32 as OPJ_UINT32; // u value
      suffix_len_1 = d1 >> 2i32 & 0x7u32; //Kappa is 1 for initial line
      consumed_bits = (consumed_bits as core::ffi::c_uint).wrapping_add(suffix_len_1) as OPJ_UINT32; // u value
      d1 = (d1 >> 5i32).wrapping_add(vlc & ((1u32) << suffix_len_1).wrapping_sub(1u32));
      u[0] = d1.wrapping_add(1u32);
      vlc >>= suffix_len_1;
      suffix_len_1 = d2 >> 2i32 & 0x7u32;
      consumed_bits = (consumed_bits as core::ffi::c_uint).wrapping_add(suffix_len_1) as OPJ_UINT32;
      d2 = (d2 >> 5i32).wrapping_add(vlc & ((1u32) << suffix_len_1).wrapping_sub(1u32));
      u[1] = d2.wrapping_add(1u32)
    }
  } else if mode == 4u32 {
    // both u_off are 1, and MEL event is 1
    let mut d1_0: OPJ_UINT32 = 0; // LSBs of VLC are prefix codeword
    let mut d2_0: OPJ_UINT32 = 0; // Consume bits
    let mut suffix_len_2: OPJ_UINT32 = 0; // LSBs of VLC are prefix codeword
    d1_0 = dec[(vlc & 0x7u32) as usize] as OPJ_UINT32; // Consume bits
    vlc >>= d1_0 & 0x3u32; // u value
    consumed_bits =
      (consumed_bits as core::ffi::c_uint).wrapping_add(d1_0 & 0x3u32) as OPJ_UINT32 as OPJ_UINT32; // add 2+kappa
    d2_0 = dec[(vlc & 0x7u32) as usize] as OPJ_UINT32; // u value
    vlc >>= d2_0 & 0x3u32;
    consumed_bits =
      (consumed_bits as core::ffi::c_uint).wrapping_add(d2_0 & 0x3u32) as OPJ_UINT32 as OPJ_UINT32;
    suffix_len_2 = d1_0 >> 2i32 & 0x7u32;
    consumed_bits = (consumed_bits as core::ffi::c_uint).wrapping_add(suffix_len_2) as OPJ_UINT32;
    d1_0 = (d1_0 >> 5i32).wrapping_add(vlc & ((1u32) << suffix_len_2).wrapping_sub(1u32));
    u[0] = d1_0.wrapping_add(3u32);
    vlc >>= suffix_len_2;
    suffix_len_2 = d2_0 >> 2i32 & 0x7u32;
    consumed_bits = (consumed_bits as core::ffi::c_uint).wrapping_add(suffix_len_2) as OPJ_UINT32;
    d2_0 = (d2_0 >> 5i32).wrapping_add(vlc & ((1u32) << suffix_len_2).wrapping_sub(1u32));
    u[1] = d2_0.wrapping_add(3u32)
  }
  consumed_bits
}
//* ***********************************************************************/
/* * @brief Decode non-initial UVLC to get the u value (or u_q)
 *
 *  @param [in]  vlc is the head of the VLC bitstream
 *  @param [in]  mode is 0, 1, 2, or 3. The 1st bit is u_off of 1st quad
 *               and 2nd for 2nd quad of a quad pair
 *  @param [out] u is the u value (or u_q) + 1.  Note: we produce u + 1;
 *               this value is a partial calculation of u + kappa.
 */
fn decode_noninit_uvlc(
  mut vlc: OPJ_UINT32,
  mut mode: OPJ_UINT32,
  u: &mut [OPJ_UINT32; 2],
) -> OPJ_UINT32 {
  //table stores possible decoding three bits from vlc
  // there are 8 entries for xx1, x10, 100, 000, where x means do not care
  // table value is made up of
  // 2 bits in the LSB for prefix length
  // 3 bits for suffix length
  // 3 bits in the MSB for prefix value (u_pfx in Table 3 of ITU T.814)
  static dec: [OPJ_UINT8; 8] = [
    (3i32 | (5i32) << 2i32 | (5i32) << 5i32) as OPJ_UINT8,
    (1i32 | (1i32) << 5i32) as OPJ_UINT8,
    (2i32 | (2i32) << 5i32) as OPJ_UINT8,
    (1i32 | (1i32) << 5i32) as OPJ_UINT8,
    (3i32 | (1i32) << 2i32 | (3i32) << 5i32) as OPJ_UINT8,
    (1i32 | (1i32) << 5i32) as OPJ_UINT8,
    (2i32 | (2i32) << 5i32) as OPJ_UINT8,
    (1i32 | (1i32) << 5i32) as OPJ_UINT8,
  ];
  let mut consumed_bits = 0 as OPJ_UINT32;
  if mode == 0u32 {
    u[1] = 1 as OPJ_UINT32;
    u[0] = u[1]
    //for kappa
  } else if mode <= 2u32 {
    //u_off are either 01 or 10
    let mut d: OPJ_UINT32 = 0; //look at the least significant 3 bits
    let mut suffix_len: OPJ_UINT32 = 0; //prefix length
    d = dec[(vlc & 0x7u32) as usize] as OPJ_UINT32; // u value
    vlc >>= d & 0x3u32; //for kappa
    consumed_bits =
      (consumed_bits as core::ffi::c_uint).wrapping_add(d & 0x3u32) as OPJ_UINT32 as OPJ_UINT32;
    suffix_len = d >> 2i32 & 0x7u32;
    consumed_bits = (consumed_bits as core::ffi::c_uint).wrapping_add(suffix_len) as OPJ_UINT32;
    d = (d >> 5i32).wrapping_add(vlc & ((1u32) << suffix_len).wrapping_sub(1u32));
    u[0] = if mode == 1u32 {
      d.wrapping_add(1u32)
    } else {
      1u32
    };
    u[1] = if mode == 1u32 {
      1u32
    } else {
      d.wrapping_add(1u32)
    }
  } else if mode == 3u32 {
    // both u_off are 1
    let mut d1: OPJ_UINT32 = 0; // LSBs of VLC are prefix codeword
    let mut d2: OPJ_UINT32 = 0; // Consume bits
    let mut suffix_len_0: OPJ_UINT32 = 0; // LSBs of VLC are prefix codeword
    d1 = dec[(vlc & 0x7u32) as usize] as OPJ_UINT32; // Consume bits
    vlc >>= d1 & 0x3u32; // u value
    consumed_bits =
      (consumed_bits as core::ffi::c_uint).wrapping_add(d1 & 0x3u32) as OPJ_UINT32 as OPJ_UINT32; //1 for kappa
    d2 = dec[(vlc & 0x7u32) as usize] as OPJ_UINT32; // u value
    vlc >>= d2 & 0x3u32;
    consumed_bits =
      (consumed_bits as core::ffi::c_uint).wrapping_add(d2 & 0x3u32) as OPJ_UINT32 as OPJ_UINT32;
    suffix_len_0 = d1 >> 2i32 & 0x7u32;
    consumed_bits = (consumed_bits as core::ffi::c_uint).wrapping_add(suffix_len_0) as OPJ_UINT32;
    d1 = (d1 >> 5i32).wrapping_add(vlc & ((1u32) << suffix_len_0).wrapping_sub(1u32));
    u[0] = d1.wrapping_add(1u32);
    vlc >>= suffix_len_0;
    suffix_len_0 = d2 >> 2i32 & 0x7u32;
    consumed_bits = (consumed_bits as core::ffi::c_uint).wrapping_add(suffix_len_0) as OPJ_UINT32;
    d2 = (d2 >> 5i32).wrapping_add(vlc & ((1u32) << suffix_len_0).wrapping_sub(1u32));
    u[1] = d2.wrapping_add(1u32)
  }
  consumed_bits
}
//* ***********************************************************************/
/* * @brief Read and unstuffs 32 bits from forward-growing bitstream
 *
 *  A subroutine to read from both the MagSgn or SPP bitstreams;
 *  in particular, when MagSgn bitstream is consumed, 0xFF's are fed,
 *  while when SPP is exhausted 0's are fed in.
 *  X controls this value.
 *
 *  Unstuffing prevent sequences that are more than 0xFF7F from appearing
 *  in the conpressed sequence.  So whenever a value of 0xFF is coded, the
 *  MSB of the next byte is set 0 and must be ignored during decoding.
 *
 *  Reading can go beyond the end of buffer by up to 3 bytes.
 *
 *  @param  [in]  msp is a pointer to frwd_struct_t structure
 *
 */
fn frwd_read(msp: &mut frwd_struct_t) {
  let mut val: OPJ_UINT32 = 0; // assert that there is a space for 32 bits
  let mut bits: OPJ_UINT32 = 0; // read 32 bits
  let mut t: OPJ_UINT32 = 0;
  let mut unstuff: OPJ_BOOL = 0;
  assert!(msp.bits <= 32u32);
  val = 0u32;
  if msp.size > 3i32 {
    val = read_le_uint32(msp.data, msp.pos);
    msp.pos += 4; // increment pointer
    msp.size -= 4i32 // reduce size
  } else if msp.size > 0i32 {
    let mut i = 0i32; // read one byte at a time
    val = if msp.X != 0u32 { 0xffffffffu32 } else { 0u32 }; // mask of location
    while msp.size > 0i32 {
      let mut v = msp.data[msp.pos] as OPJ_UINT32; // put one byte in its correct location
      msp.pos += 1;
      let mut m = !((0xffu32) << i);
      val = val & m | v << i;
      msp.size -= 1;
      i += 8i32
    }
  } else {
    val = if msp.X != 0u32 { 0xffffffffu32 } else { 0u32 }
  }
  // we accumulate in t and keep a count of the number of bits in bits
  bits = (8u32).wrapping_sub(if msp.unstuff != 0 { 1u32 } else { 0u32 }); // Do we need unstuffing next?
  t = val & 0xffu32; // for next byte
  unstuff = (val & 0xffu32 == 0xffu32) as core::ffi::c_int; // move data to msp->tmp
  t |= (val >> 8i32 & 0xffu32) << bits;
  bits = (bits as core::ffi::c_uint).wrapping_add((8u32).wrapping_sub(if unstuff != 0 {
    1u32
  } else {
    0u32
  })) as OPJ_UINT32;
  unstuff = (val >> 8i32 & 0xffu32 == 0xffu32) as core::ffi::c_int;
  t |= (val >> 16i32 & 0xffu32) << bits;
  bits = (bits as core::ffi::c_uint).wrapping_add((8u32).wrapping_sub(if unstuff != 0 {
    1u32
  } else {
    0u32
  })) as OPJ_UINT32;
  unstuff = (val >> 16i32 & 0xffu32 == 0xffu32) as core::ffi::c_int;
  t |= (val >> 24i32 & 0xffu32) << bits;
  bits = (bits as core::ffi::c_uint).wrapping_add((8u32).wrapping_sub(if unstuff != 0 {
    1u32
  } else {
    0u32
  })) as OPJ_UINT32;
  msp.unstuff = (val >> 24i32 & 0xffu32 == 0xffu32) as core::ffi::c_int;
  msp.tmp |= (t as OPJ_UINT64) << msp.bits;
  msp.bits = (msp.bits as core::ffi::c_uint).wrapping_add(bits) as OPJ_UINT32;
}
//* ***********************************************************************/
/* * @brief Initialize frwd_struct_t struct and reads some bytes
 *
 *  @param [in]  msp is a pointer to frwd_struct_t
 *  @param [in]  data is a pointer to the start of data
 *  @param [in]  size is the number of byte in the bitstream
 *  @param [in]  X is the value fed in when the bitstream is exhausted.
 *               See frwd_read.
 */
fn frwd_init<'a>(
  msp: &mut frwd_struct_t<'a>,
  data: &'a [OPJ_UINT8],
  mut size: core::ffi::c_int,
  mut X: OPJ_UINT32,
) {
  let mut num: core::ffi::c_int = 0;
  let mut i: core::ffi::c_int = 0;
  msp.data = data;
  msp.pos = 0;
  msp.tmp = 0 as OPJ_UINT64;
  msp.bits = 0 as OPJ_UINT32;
  msp.unstuff = 0i32;
  msp.size = size;
  msp.X = X;
  assert!(msp.X == 0u32 || msp.X == 0xffu32);
  //This code is designed for an architecture that read address should
  // align to the read size (address multiple of 4 if read size is 4)
  //These few lines take care of the case where data is not at a multiple
  // of 4 boundary.  It reads 1,2,3 up to 4 bytes from the bitstream
  num = 4i32 - (msp.addr() & 0x3) as core::ffi::c_int;
  i = 0i32;
  while i < num {
    let mut d: OPJ_UINT64 = 0;
    // unstuffing for next byte
    let fresh12 = msp.size;
    msp.size -= 1;
    d = if fresh12 > 0i32 {
      let b = msp.data[msp.pos];
      msp.pos += 1;
      b as core::ffi::c_uint
    } else {
      msp.X
    } as OPJ_UINT64;
    msp.tmp |= d << msp.bits;
    msp.bits = (msp.bits as core::ffi::c_uint)
      .wrapping_add((8u32).wrapping_sub(if msp.unstuff != 0 { 1u32 } else { 0u32 }))
      as OPJ_UINT32;
    msp.unstuff = (d & 0xffu64 == 0xffu64) as core::ffi::c_int;
    i += 1
  }
  frwd_read(msp);
  //read a byte if the buffer is not exhausted, otherwise set it to X
  // store data in msp->tmp
  // number of bits added to msp->tmp
  // read 32 bits more
}
//* ***********************************************************************/
/* * @brief Consume num_bits bits from the bitstream of frwd_struct_t
 *
 *  @param [in]  msp is a pointer to frwd_struct_t
 *  @param [in]  num_bits is the number of bit to consume
 */
fn frwd_advance(msp: &mut frwd_struct_t, mut num_bits: OPJ_UINT32) {
  assert!(num_bits <= msp.bits);
  msp.tmp >>= num_bits;
  msp.bits = (msp.bits as core::ffi::c_uint).wrapping_sub(num_bits) as OPJ_UINT32;
}
//* ***********************************************************************/
/* * @brief Fetches 32 bits from the frwd_struct_t bitstream
 *
 *  @param [in]  msp is a pointer to frwd_struct_t
 */
fn frwd_fetch(msp: &mut frwd_struct_t) -> OPJ_UINT32 {
  if msp.bits < 32u32 {
    frwd_read(msp);
    if msp.bits < 32u32 {
      //need to test
      frwd_read(msp);
    }
  }
  msp.tmp as OPJ_UINT32
}
//* ***********************************************************************/
/* * @brief Allocates T1 buffers
 *
 *  @param [in, out]  t1 is codeblock cofficients storage
 *  @param [in]       w is codeblock width
 *  @param [in]       h is codeblock height
 */
fn opj_t1_allocate_buffers(
  mut t1: &mut opj_t1_t,
  mut w: OPJ_UINT32,
  mut h: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut flagssize: usize = 0;
  /* No risk of overflow. Prior checks ensure those assert are met */
  /* They are per the specification */

  assert!(w <= 1024u32);
  assert!(h <= 1024u32);
  assert!(w.wrapping_mul(h) <= 4096u32);
  /* encoder uses tile buffer, so no need to allocate */
  let datasize = w * h;
  t1.data.resize(datasize as usize);
  // We expand these buffers to multiples of 16 bytes.
  // We need 4 buffers of 129 integers each, expanded to 132 integers each
  // We also need 514 bytes of buffer, expanded to 528 bytes
  flagssize = 132usize * core::mem::size_of::<OPJ_UINT32>() * 4; // expanded to multiple of 16
  flagssize += 528; // 514 expanded to multiples of 16
  t1.flags.resize(flagssize);
  t1.w = w;
  t1.h = h;
  1i32
}
//* ***********************************************************************/
/* * @brief Decodes one codeblock, processing the cleanup, siginificance
 *         propagation, and magnitude refinement pass
 *
 *  @param [in, out]  t1 is codeblock cofficients storage
 *  @param [in]       cblk is codeblock properties
 *  @param [in]       orient is the subband to which the codeblock belongs (not needed)
 *  @param [in]       roishift is region of interest shift
 *  @param [in]       cblksty is codeblock style
 *  @param [in]       p_manager is events print manager
 *  @param [in]       check_pterm: check termination (not used)
 */
pub(crate) fn opj_t1_ht_decode_cblk(
  mut t1: &mut opj_t1_t,
  cblk: &mut opj_tcd_cblk_dec_t,
  mut _orient: OPJ_UINT32,
  mut roishift: OPJ_UINT32,
  mut cblksty: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
  mut _check_pterm: OPJ_BOOL,
) -> OPJ_BOOL {
    let cblkdata: &[u8]; // fetched data from VLC bitstream
    let mut zero_bplanes: OPJ_UINT32 = 0;
    let mut num_passes: OPJ_UINT32 = 0;
    let mut lengths1: OPJ_UINT32 = 0;
    let mut lengths2: OPJ_UINT32 = 0;
    let mut width: OPJ_INT32 = 0;
    let mut height: OPJ_INT32 = 0;
    let mut stride: OPJ_INT32 = 0;
    let mut sigma1: FlagsPtr;
    let mut sigma2: FlagsPtr;
    let mut mbr1: FlagsPtr;
    let mut mbr2: FlagsPtr;
    let mut sip: FlagsPtr;
    let mut sip_shift: OPJ_UINT32 = 0;
    let mut p: OPJ_UINT32 = 0;
    let mut zero_bplanes_p1: OPJ_UINT32 = 0;
    let mut lcup: core::ffi::c_int = 0;
    let mut scup: core::ffi::c_int = 0;
    let mut mel = dec_mel_t {
      data: &[],
      pos: 0,
      tmp: 0,
      bits: 0,
      size: 0,
      unstuff: 0,
      k: 0,
      num_runs: 0,
      runs: 0,
    };
    let mut vlc = rev_struct_t {
      data: &[],
      pos: 0,
      tmp: 0,
      bits: 0,
      size: 0,
      unstuff: 0,
    };
    let mut magsgn = frwd_struct_t {
      data: &[],
      pos: 0,
      tmp: 0,
      bits: 0,
      unstuff: 0,
      size: 0,
      X: 0,
    };
    let mut sigprop = frwd_struct_t {
      data: &[],
      pos: 0,
      tmp: 0,
      bits: 0,
      unstuff: 0,
      size: 0,
      X: 0,
    };
    let mut magref = rev_struct_t {
      data: &[],
      pos: 0,
      tmp: 0,
      bits: 0,
      size: 0,
      unstuff: 0,
    };
    // Per-column line state for the cleanup pass: bit 7 is `\sigma`, bits 0-6 are `E`.
    // 514 bytes are enough for a codeblock width of 1024; expanded to 528 like the C code.
    // `lsp` is the cursor into it.
    let line_state: Vec<Cell<OPJ_UINT8>> = vec![Cell::new(0); 528];
    let mut lsp: usize = 0;
    let mut run: core::ffi::c_int = 0;
    let mut vlc_val: OPJ_UINT32 = 0;
    let mut qinf: [OPJ_UINT32; 2] = [0; 2];
    let mut c_q: OPJ_UINT32 = 0;
    let mut sp: usize = 0;
    let mut x: OPJ_INT32 = 0;
    let mut y: OPJ_INT32 = 0;
    let mut stripe_causal = (cblksty & 0x8u32 != 0u32) as core::ffi::c_int;
    let mut cblk_len = 0 as OPJ_UINT32;
    // stops unused parameter message
    // stops unused parameter message
    // We ignor orient, because the same decoder is used for all subbands
    // We also ignore check_pterm, because I am not sure how it applies
    if roishift != 0u32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "We do not support ROI in decoding HT codeblocks\n",
      );
      return 0i32;
    }
    if opj_t1_allocate_buffers(
      t1,
      (cblk.x1 - cblk.x0) as OPJ_UINT32,
      (cblk.y1 - cblk.y0) as OPJ_UINT32,
    ) == 0
    {
      return 0i32;
    }
    if cblk.Mb == 0u32 {
      return 1i32;
    }
    /* numbps = Mb + 1 - zero_bplanes, Mb = Kmax, zero_bplanes = missing_msbs */
    zero_bplanes = cblk.Mb.wrapping_add(1u32).wrapping_sub(cblk.numbps);
    /* Compute whole codeblock length from chunk lengths */
    cblk_len = 0 as OPJ_UINT32;
    let mut i: OPJ_UINT32 = 0;
    i = 0 as OPJ_UINT32;
    while i < cblk.numchunks {
      cblk_len = (cblk_len as core::ffi::c_uint)
        .wrapping_add(cblk.chunks[i as usize].len)
        as OPJ_UINT32;
      i += 1;
    }
    if cblk.numchunks > 1u32 || t1.mustuse_cblkdatabuffer != 0 {
      let mut i_0: OPJ_UINT32 = 0;
      /* Allocate temporary memory if needed */
      t1.grow_cblkdatabuffer(cblk_len as usize);

      /* Concatenate all chunks */
      {
        let buffer = t1.cblkdatabuffer();
        cblk_len = 0 as OPJ_UINT32;
        i_0 = 0 as OPJ_UINT32;
        while i_0 < cblk.numchunks {
          let chunk = cblk.chunks[i_0 as usize].as_slice();
          let off = cblk_len as usize;
          buffer[off..off + chunk.len()].copy_from_slice(chunk);
          cblk_len = (cblk_len as core::ffi::c_uint)
            .wrapping_add(cblk.chunks[i_0 as usize].len)
            as OPJ_UINT32;
          i_0 += 1;
        }
      }
      cblkdata = t1.cblkdatabuffer_ref();
    } else if cblk.numchunks == 1u32 {
      cblkdata = cblk.chunks[0].as_slice();
    } else {
      /* Not sure if that can happen in practice, but avoid Coverity to */
      /* think we will dereference a null cblkdta pointer */
      return 1i32;
    }
    // The C code writes the decoded samples through a `OPJ_UINT32*` alias of the T1 data
    // buffer. Here that alias is one exclusive `&mut [u32]` view, created once at the boundary:
    // the function holds the only `&mut` to `t1` and never touches `t1.data` otherwise.
    let ht_data: &mut [OPJ_UINT32] = unsafe {
      core::slice::from_raw_parts_mut(
        t1.data.as_mut_ptr() as *mut OPJ_UINT32,
        t1.data.as_slice().len(),
      )
    };
    // OPJ_UINT32 num_passes is the number of passes: 1 if CUP only, 2 for
    // CUP+SPP, and 3 for CUP+SPP+MRP
    num_passes = if cblk.numsegs > 0u32 {
      cblk.segs[0].real_num_passes
    } else {
      0u32
    };
    num_passes = (num_passes as core::ffi::c_uint).wrapping_add(if cblk.numsegs > 1u32 {
      cblk.segs[1].real_num_passes
    } else {
      0u32
    }) as OPJ_UINT32;
    // OPJ_UINT32 lengths1 is the length of cleanup pass
    lengths1 = if num_passes > 0u32 {
      cblk.segs[0].len
    } else {
      0u32
    };
    // OPJ_UINT32 lengths2 is the length of refinement passes (either SPP only or SPP+MRP)
    lengths2 = if num_passes > 1u32 {
      cblk.segs[1].len
    } else {
      0u32
    };
    // OPJ_INT32 width is the decoded codeblock width
    width = cblk.x1 - cblk.x0;
    // OPJ_INT32 height is the decoded codeblock height
    height = cblk.y1 - cblk.y0;
    // OPJ_INT32 stride is the decoded codeblock buffer stride
    stride = width;
    /*  sigma1 and sigma2 contains significant (i.e., non-zero) pixel
     *  locations.  The buffers are used interchangeably, because we need
     *  more than 4 rows of significance information at a given time.
     *  Each 32 bits contain significance information for 4 rows of 8
     *  columns each.  If we denote 32 bits by 0xaaaaaaaa, the each "a" is
     *  called a nibble and has significance information for 4 rows.
     *  The least significant nibble has information for the first column,
     *  and so on. The nibble's LSB is for the first row, and so on.
     *  Since, at most, we can have 1024 columns in a quad, we need 128
     *  entries; we added 1 for convenience when propagation of signifcance
     *  goes outside the structure
     *  To work in OpenJPEG these buffers has been expanded to 132.
     */
    // OPJ_UINT32 *pflags, *sigma1, *sigma2, *mbr1, *mbr2, *sip, sip_shift;
    sigma1 = t1.flags.offset(0);
    sigma2 = t1.flags.offset(132);
    // mbr arrangement is similar to sigma; mbr contains locations
    // that become significant during significance propagation pass
    mbr1 = t1.flags.offset(2 * 132);
    mbr2 = t1.flags.offset(3 * 132);
    //a pointer to sigma
    sip = sigma1; //pointers to arrays to be used interchangeably
    sip_shift = 0 as OPJ_UINT32; //the amount of shift needed for sigma
    if num_passes > 1u32 && lengths2 == 0u32 {
      event_msg!(p_manager, EVT_WARNING,
                        "A malformed codeblock that has more than one coding pass, but zero length for 2nd and potentially the 3rd pass in an HT codeblock.\n");
      num_passes = 1 as OPJ_UINT32
    }
    if num_passes > 3u32 {
      event_msg!(p_manager, EVT_ERROR,
                        "We do not support more than 3 coding passes in an HT codeblock; This codeblocks has {} passes.\n", num_passes);
      return 0i32;
    }
    if cblk.Mb > 30u32 {
      /* This check is better moved to opj_t2_read_packet_header() in t2.c
        We do not have enough precision to decode any passes
        The design of openjpeg assumes that the bits of a 32-bit integer are
        assigned as follows:
        bit 31 is for sign
        bits 30-1 are for magnitude
        bit 0 is for the center of the quantization bin
        Therefore we can only do values of cblk->Mb <= 30
      */
      event_msg!(p_manager, EVT_ERROR,
                        "32 bits are not enough to decode this codeblock, since the number of bitplane, {}, is larger than 30.\n", cblk.Mb);
      return 0i32;
    }
    if zero_bplanes > cblk.Mb {
      /* This check is better moved to opj_t2_read_packet_header() in t2.c,
      in the line "l_cblk->numbps = (OPJ_UINT32)l_band->numbps + 1 - i;"
      where i is the zero bitplanes, and should be no larger than cblk->Mb
      We cannot have more zero bitplanes than there are planes. */
      event_msg!(p_manager, EVT_ERROR,
                        "Malformed HT codeblock. Decoding this codeblock is stopped. There are {} zero bitplanes in {} bitplanes.\n", zero_bplanes,
                        cblk.Mb);
      return 0i32;
    } else if zero_bplanes == cblk.Mb && num_passes > 1u32 {
      /* When the number of zero bitplanes is equal to the number of bitplanes,
      only the cleanup pass makes sense*/
      if !ONLY_CLEANUP_PASS_DECODED.load(Ordering::Relaxed) {
        /* We have a second check to prevent the possibility of an overrun condition,
        in the very unlikely event of a second thread discovering that
        ONLY_CLEANUP_PASS_DECODED is false before the first thread changing
        the condition. */
        if !ONLY_CLEANUP_PASS_DECODED.swap(true, Ordering::Relaxed) {
          event_msg!(p_manager, EVT_WARNING,
                              "Malformed HT codeblock. When the number of zero planes bitplanes is equal to the number of bitplanes, only the cleanup pass makes sense, but we have {} passes in this codeblock. Therefore, only the cleanup pass will be decoded. This message will not be displayed again.\n",
                              num_passes);
        }
      }
      num_passes = 1 as OPJ_UINT32
    }
    /* OPJ_UINT32 */
    p = cblk.numbps;
    // OPJ_UINT32 zero planes plus 1
    zero_bplanes_p1 = zero_bplanes.wrapping_add(1u32);
    if lengths1 < 2u32 || lengths1 > cblk_len || lengths1.wrapping_add(lengths2) > cblk_len {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Malformed HT codeblock. Invalid codeblock length values.\n",
      );
      return 0i32;
    }
    // read scup and fix the bytes there
    lcup = lengths1 as core::ffi::c_int; // length of CUP
                                         //scup is the length of MEL + VLC
    scup = ((cblkdata[(lcup - 1i32) as usize] as core::ffi::c_int) << 4i32)
      + (cblkdata[(lcup - 2i32) as usize] as core::ffi::c_int & 0xfi32);
    if scup < 2i32 || scup > lcup || scup > 4079i32 {
      //something is wrong
      /* The standard stipulates 2 <= Scup <= min(Lcup, 4079) */
      event_msg!(p_manager, EVT_ERROR,
                        "Malformed HT codeblock. One of the following condition is not met: 2 <= Scup <= min(Lcup, 4079)\n");
      return 0i32;
    }
    // init structures
    if !mel_init(&mut mel, cblkdata, lcup, scup) {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Malformed HT codeblock.  Incorrect MEL segment sequence.\n"
      );
      return 0i32;
    }
    rev_init(&mut vlc, cblkdata, lcup, scup);
    frwd_init(&mut magsgn, cblkdata, lcup - scup, 0xff as OPJ_UINT32);
    if num_passes > 1u32 {
      // needs to be tested
      frwd_init(
        &mut sigprop,
        &cblkdata[lengths1 as usize..],
        lengths2 as core::ffi::c_int,
        0 as OPJ_UINT32,
      );
    }
    if num_passes > 2u32 {
      rev_init_mrp(
        &mut magref,
        cblkdata,
        lengths1 as core::ffi::c_int,
        lengths2 as core::ffi::c_int,
      );
    }
    /* * State storage
     *  One byte per quad; for 1024 columns, or 512 quads, we need
     *  512 bytes. We are using 2 extra bytes one on the left and one on
     *  the right for convenience.
     *
     *  The MSB bit in each byte is (\sigma^nw | \sigma^n), and the 7 LSBs
     *  contain max(E^nw | E^n)
     */
    // 514 is enough for a block width of 1024, +2 extra
    // here expanded to 528
    //initial 2 lines
    // ///////////////
    lsp = 0; // point to line state
    line_state[lsp].set(0 as OPJ_UINT8); // for initial row of quad, we set to 0
    run = mel_get_run(&mut mel); // decode runs of events from MEL bitstrm
                                 // data represented as runs of 0 events
                                 // See mel_decode description
    qinf[1_usize] = 0 as OPJ_UINT32; // quad info decoded from VLC bitstream
    qinf[0_usize] = qinf[1_usize]; // context for quad q
    c_q = 0 as OPJ_UINT32; // decoded codeblock samples
    sp = 0;
    // vlc_val;                 // fetched data from VLC bitstream
    x = 0i32;
    while x < width {
      // one iteration per quad pair
      let mut U_q: [OPJ_UINT32; 2] = [0; 2]; // u values for the quad pair
      let mut uvlc_mode: OPJ_UINT32 = 0;
      let mut consumed_bits: OPJ_UINT32 = 0;
      let mut m_n: OPJ_UINT32 = 0;
      let mut v_n: OPJ_UINT32 = 0;
      let mut ms_val: OPJ_UINT32 = 0;
      let mut locs: OPJ_UINT32 = 0;
      // decode VLC
      // ///////////
      //first quad
      // Get the head of the VLC bitstream. One fetch is enough for two
      // quads, since the largest VLC code is 7 bits, and maximum number of
      // bits used for u is 8.  Therefore for two quads we need 30 bits
      // (if we include unstuffing, then 32 bits are enough, since we have
      // a maximum of one stuffing per two bytes)
      vlc_val = rev_fetch(&mut vlc);
      //decode VLC using the context c_q and the head of the VLC bitstream
      qinf[0_usize] = vlc_tbl0[(c_q << 7i32 | vlc_val & 0x7fu32) as usize] as OPJ_UINT32;
      if c_q == 0u32 {
        // if zero context, we need to use one MEL event
        run -= 2i32; //the number of 0 events is multiplied by 2, so subtract 2
                     // Is the run terminated in 1? if so, use decoded VLC code,
                     // otherwise, discard decoded data, since we will decoded again
                     // using a different context
        qinf[0_usize] = if run == -(1i32) { qinf[0_usize] } else { 0u32 };
        // is run -1 or -2? this means a run has been consumed
        if run < 0i32 {
          run = mel_get_run(&mut mel)
          // get another run
        }
      }
      // prepare context for the next quad; eqn. 1 in ITU T.814
      c_q = (qinf[0_usize] & 0x10u32) >> 4i32 | (qinf[0_usize] & 0xe0u32) >> 5i32;
      //remove data from vlc stream (0 bits are removed if qinf is not used)
      vlc_val = rev_advance(&mut vlc, qinf[0_usize] & 0x7u32);
      //update sigma
      // The update depends on the value of x; consider one OPJ_UINT32
      // if x is 0, 8, 16 and so on, then this line update c locations
      //      nibble (4 bits) number   0 1 2 3 4 5 6 7
      //                         LSB   c c 0 0 0 0 0 0
      //                               c c 0 0 0 0 0 0
      //                               0 0 0 0 0 0 0 0
      //                               0 0 0 0 0 0 0 0
      // if x is 4, 12, 20, then this line update locations c
      //      nibble (4 bits) number   0 1 2 3 4 5 6 7
      //                         LSB   0 0 0 0 c c 0 0
      //                               0 0 0 0 c c 0 0
      //                               0 0 0 0 0 0 0 0
      //                               0 0 0 0 0 0 0 0
      sip.or(((qinf[0_usize] & 0x30u32) >> 4i32 | (qinf[0_usize] & 0xc0u32) >> 2i32) << sip_shift);
      //second quad
      qinf[1_usize] = 0 as OPJ_UINT32;
      if (x + 2i32) < width {
        // do not run if codeblock is narrower
        //decode VLC using the context c_q and the head of the VLC bitstream
        qinf[1_usize] = vlc_tbl0[(c_q << 7i32 | vlc_val & 0x7fu32) as usize] as OPJ_UINT32;
        // if context is zero, use one MEL event
        if c_q == 0u32 {
          //zero context
          run -= 2i32; //subtract 2, since events number if multiplied by 2
                       // if event is 0, discard decoded qinf
          qinf[1_usize] = if run == -(1i32) { qinf[1_usize] } else { 0u32 };
          if run < 0i32 {
            // have we consumed all events in a run
            run = mel_get_run(&mut mel)
            // if yes, then get another run
          }
        }
        //prepare context for the next quad, eqn. 1 in ITU T.814
        c_q = (qinf[1_usize] & 0x10u32) >> 4i32 | (qinf[1_usize] & 0xe0u32) >> 5i32;
        //remove data from vlc stream, if qinf is not used, cwdlen is 0
        vlc_val = rev_advance(&mut vlc, qinf[1_usize] & 0x7u32)
      }
      //update sigma
      // The update depends on the value of x; consider one OPJ_UINT32
      // if x is 0, 8, 16 and so on, then this line update c locations
      //      nibble (4 bits) number   0 1 2 3 4 5 6 7
      //                         LSB   0 0 c c 0 0 0 0
      //                               0 0 c c 0 0 0 0
      //                               0 0 0 0 0 0 0 0
      //                               0 0 0 0 0 0 0 0
      // if x is 4, 12, 20, then this line update locations c
      //      nibble (4 bits) number   0 1 2 3 4 5 6 7
      //                         LSB   0 0 0 0 0 0 c c
      //                               0 0 0 0 0 0 c c
      //                               0 0 0 0 0 0 0 0
      //                               0 0 0 0 0 0 0 0
      sip.or(
        (qinf[1_usize] & 0x30u32 | (qinf[1_usize] & 0xc0u32) << 2i32)
          << (4u32).wrapping_add(sip_shift),
      ); // move sigma pointer to next entry
      sip = sip.offset(if x & 0x7i32 != 0 { 1i32 } else { 0i32 } as isize); // increment/decrement sip_shift by 16
      sip_shift ^= 0x10u32;
      // retrieve u
      // ///////////
      // uvlc_mode is made up of u_offset bits from the quad pair
      uvlc_mode = (qinf[0_usize] & 0x8u32) >> 3i32 | (qinf[1_usize] & 0x8u32) >> 2i32;
      if uvlc_mode == 3u32 {
        // if both u_offset are set, get an event from
        // the MEL run of events
        run -= 2i32; //subtract 2, since events number if multiplied by 2
        uvlc_mode =
          (uvlc_mode as core::ffi::c_uint)
            .wrapping_add(if run == -(1i32) { 1i32 } else { 0i32 } as core::ffi::c_uint)
            as OPJ_UINT32; //increment uvlc_mode if event is 1
        if run < 0i32 {
          // if run is consumed (run is -1 or -2), get another run
          run = mel_get_run(&mut mel)
        }
      }
      //decode uvlc_mode to get u for both quads
      consumed_bits = decode_init_uvlc(vlc_val, uvlc_mode, &mut U_q);
      if U_q[0_usize] > zero_bplanes_p1 || U_q[1_usize] > zero_bplanes_p1 {
        event_msg!(p_manager, EVT_ERROR,
                            "Malformed HT codeblock. Decoding this codeblock is stopped. U_q is larger than zero bitplanes + 1 \n");
        return 0i32;
      }
      //consume u bits in the VLC code
      vlc_val = rev_advance(&mut vlc, consumed_bits);
      //decode magsgn and update line_state
      // ///////////////////////////////////
      //We obtain a mask for the samples locations that needs evaluation
      locs = 0xff as OPJ_UINT32;
      if x + 4i32 > width {
        locs >>= (x + 4i32 - width) << 1i32
        // limits width
      } // limits height
      locs = if height > 1i32 {
        locs
      } else {
        (locs) & 0x55u32
      };
      if ((qinf[0_usize] & 0xf0u32) >> 4i32 | qinf[1_usize] & 0xf0u32) & !locs != 0 {
        event_msg!(p_manager, EVT_ERROR,
                            "Malformed HT codeblock. VLC code produces significant samples outside the codeblock area.\n");
        return 0i32;
      }
      //first quad, starting at first sample in quad and moving on
      if qinf[0_usize] & 0x10u32 != 0 {
        //is it significant? (sigma_n)
        let mut val: OPJ_UINT32 = 0; //get 32 bits of magsgn data
        ms_val = frwd_fetch(&mut magsgn); //evaluate m_n (number of bits
        m_n = U_q[0_usize].wrapping_sub(qinf[0_usize] >> 12i32 & 1u32);
        // to read from bitstream), using EMB e_k
        frwd_advance(&mut magsgn, m_n); //consume m_n
        val = ms_val << 31i32; //get sign bit
        v_n = ms_val & ((1u32) << m_n).wrapping_sub(1u32); //keep only m_n bits
        v_n |= ((qinf[0_usize] & 0x100u32) >> 8i32) << m_n; //add EMB e_1 as MSB
        v_n |= 1u32; //add center of bin
                     //v_n now has 2 * (\mu - 1) + 0.5 with correct sign bit
                     //add 2 to make it 2*\mu+0.5, shift it up to missing MSBs
        ht_data[sp + (0) as usize] = val | v_n.wrapping_add(2u32) << p.wrapping_sub(1u32)
      } else if locs & 0x1u32 != 0 {
        // if this is inside the codeblock, set the
        ht_data[sp + (0) as usize] = 0 as OPJ_UINT32
        // sample to zero
      }
      if qinf[0_usize] & 0x20u32 != 0 {
        //sigma_n
        let mut val_0: OPJ_UINT32 = 0; //get 32 bits
        let mut t: OPJ_UINT32 = 0; //m_n, uses EMB e_k
        ms_val = frwd_fetch(&mut magsgn); //consume m_n
        m_n = U_q[0_usize].wrapping_sub(qinf[0_usize] >> 13i32 & 1u32); //get sign bit
        frwd_advance(&mut magsgn, m_n); //keep only m_n bits
        val_0 = ms_val << 31i32; //add EMB e_1
        v_n = ms_val & ((1u32) << m_n).wrapping_sub(1u32); //bin center
        v_n |= ((qinf[0_usize] & 0x200u32) >> 9i32) << m_n;
        v_n |= 1u32;
        //v_n now has 2 * (\mu - 1) + 0.5 with correct sign bit
        //add 2 to make it 2*\mu+0.5, shift it up to missing MSBs
        ht_data[sp + (stride) as usize] = val_0 | v_n.wrapping_add(2u32) << p.wrapping_sub(1u32);
        //update line_state: bit 7 (\sigma^N), and E^N
        t = (line_state[lsp].get() as core::ffi::c_int & 0x7fi32) as OPJ_UINT32; // keep E^NW
        v_n = (32u32).wrapping_sub(count_leading_zeros(v_n));
        line_state[lsp].set((0x80u32 | (if t > v_n { t } else { v_n })) as OPJ_UINT8)
      } else if locs & 0x2u32 != 0 {
        // if this is inside the codeblock, set the
        ht_data[sp + (stride) as usize] = 0 as OPJ_UINT32
        // sample to zero
      } // move to next quad information
      lsp += 1; // move to next column of samples
      sp += (1) as usize;
      //this is similar to the above two samples
      if qinf[0_usize] & 0x40u32 != 0 {
        let mut val_1: OPJ_UINT32 = 0; //m_n
        ms_val = frwd_fetch(&mut magsgn); //center of bin
        m_n = U_q[0_usize].wrapping_sub(qinf[0_usize] >> 14i32 & 1u32);
        frwd_advance(&mut magsgn, m_n);
        val_1 = ms_val << 31i32;
        v_n = ms_val & ((1u32) << m_n).wrapping_sub(1u32);
        v_n |= ((qinf[0_usize] & 0x400u32) >> 10i32) << m_n;
        v_n |= 1u32;
        ht_data[sp + (0) as usize] = val_1 | v_n.wrapping_add(2u32) << p.wrapping_sub(1u32)
      } else if locs & 0x4u32 != 0 {
        ht_data[sp + (0) as usize] = 0 as OPJ_UINT32
      }
      line_state[lsp].set(0 as OPJ_UINT8);
      if qinf[0_usize] & 0x80u32 != 0 {
        let mut val_2: OPJ_UINT32 = 0;
        ms_val = frwd_fetch(&mut magsgn);
        m_n = U_q[0_usize].wrapping_sub(qinf[0_usize] >> 15i32 & 1u32);
        frwd_advance(&mut magsgn, m_n);
        val_2 = ms_val << 31i32;
        v_n = ms_val & ((1u32) << m_n).wrapping_sub(1u32);
        v_n |= ((qinf[0_usize] & 0x800u32) >> 11i32) << m_n;
        v_n |= 1u32;
        ht_data[sp + (stride) as usize] = val_2 | v_n.wrapping_add(2u32) << p.wrapping_sub(1u32);
        //line_state: bit 7 (\sigma^NW), and E^NW for next quad
        line_state[lsp].set((0x80u32 | (32u32).wrapping_sub(count_leading_zeros(v_n))) as OPJ_UINT8)
      } else if locs & 0x8u32 != 0 {
        //if outside set to 0
        ht_data[sp + (stride) as usize] = 0 as OPJ_UINT32
      } //move to next column
      sp += (1) as usize;
      //second quad
      if qinf[1_usize] & 0x10u32 != 0 {
        let mut val_3: OPJ_UINT32 = 0; //m_n
        ms_val = frwd_fetch(&mut magsgn);
        m_n = U_q[1_usize].wrapping_sub(qinf[1_usize] >> 12i32 & 1u32);
        frwd_advance(&mut magsgn, m_n);
        val_3 = ms_val << 31i32;
        v_n = ms_val & ((1u32) << m_n).wrapping_sub(1u32);
        v_n |= ((qinf[1_usize] & 0x100u32) >> 8i32) << m_n;
        v_n |= 1u32;
        ht_data[sp + (0) as usize] = val_3 | v_n.wrapping_add(2u32) << p.wrapping_sub(1u32)
      } else if locs & 0x10u32 != 0 {
        ht_data[sp + (0) as usize] = 0 as OPJ_UINT32
      }
      if qinf[1_usize] & 0x20u32 != 0 {
        let mut val_4: OPJ_UINT32 = 0;
        let mut t_0: OPJ_UINT32 = 0;
        ms_val = frwd_fetch(&mut magsgn);
        //max(E^NW, E^N) | s
        m_n = U_q[1_usize].wrapping_sub(qinf[1_usize] >> 13i32 & 1u32); //m_n
        frwd_advance(&mut magsgn, m_n);
        val_4 = ms_val << 31i32;
        v_n = ms_val & ((1u32) << m_n).wrapping_sub(1u32);
        v_n |= ((qinf[1_usize] & 0x200u32) >> 9i32) << m_n;
        v_n |= 1u32;
        ht_data[sp + (stride) as usize] = val_4 | v_n.wrapping_add(2u32) << p.wrapping_sub(1u32);
        t_0 = (line_state[lsp].get() as core::ffi::c_int & 0x7fi32) as OPJ_UINT32;
        v_n = (32u32).wrapping_sub(count_leading_zeros(v_n));
        line_state[lsp].set((0x80u32 | (if t_0 > v_n { t_0 } else { v_n })) as OPJ_UINT8)
      } else if locs & 0x20u32 != 0 {
        ht_data[sp + (stride) as usize] = 0 as OPJ_UINT32
        //update line_state: bit 7 (\sigma^N), and E^N
        //E^NW
        //E^N
        //no need to update line_state
      } //move line state to next quad
      lsp += 1; //move to next sample
      sp += (1) as usize; //m_n
      if qinf[1_usize] & 0x40u32 != 0 {
        let mut val_5: OPJ_UINT32 = 0; //m_n
        ms_val = frwd_fetch(&mut magsgn); //center of bin
        m_n = U_q[1_usize].wrapping_sub(qinf[1_usize] >> 14i32 & 1u32);
        frwd_advance(&mut magsgn, m_n);
        val_5 = ms_val << 31i32;
        v_n = ms_val & ((1u32) << m_n).wrapping_sub(1u32);
        v_n |= ((qinf[1_usize] & 0x400u32) >> 10i32) << m_n;
        v_n |= 1u32;
        ht_data[sp + (0) as usize] = val_5 | v_n.wrapping_add(2u32) << p.wrapping_sub(1u32)
      } else if locs & 0x40u32 != 0 {
        ht_data[sp + (0) as usize] = 0 as OPJ_UINT32
      }
      line_state[lsp].set(0 as OPJ_UINT8);
      if qinf[1_usize] & 0x80u32 != 0 {
        let mut val_6: OPJ_UINT32 = 0;
        ms_val = frwd_fetch(&mut magsgn);
        m_n = U_q[1_usize].wrapping_sub(qinf[1_usize] >> 15i32 & 1u32);
        frwd_advance(&mut magsgn, m_n);
        val_6 = ms_val << 31i32;
        v_n = ms_val & ((1u32) << m_n).wrapping_sub(1u32);
        v_n |= ((qinf[1_usize] & 0x800u32) >> 11i32) << m_n;
        v_n |= 1u32;
        ht_data[sp + (stride) as usize] = val_6 | v_n.wrapping_add(2u32) << p.wrapping_sub(1u32);
        //line_state: bit 7 (\sigma^NW), and E^NW for next quad
        line_state[lsp].set((0x80u32 | (32u32).wrapping_sub(count_leading_zeros(v_n))) as OPJ_UINT8)
      } else if locs & 0x80u32 != 0 {
        ht_data[sp + (stride) as usize] = 0 as OPJ_UINT32
      }
      sp += (1) as usize;
      x += 4i32
    }
    //non-initial lines
    // ////////////////////////
    y = 2i32;
    while y < height {
      /*done at the end of loop*/
      let mut sip_0: FlagsPtr; // shift sigma to the upper half od the nibble
      let mut ls0: OPJ_UINT8 = 0; //move back to 0 (it might have been at 0x10)
      let mut x_0: OPJ_INT32 = 0; //choose sigma array
      sip_shift ^= 0x2u32; // read the line state value
      sip_shift &= 0xffffffefu32; // and set it to zero
      sip_0 = if y & 0x4i32 != 0 { sigma2 } else { sigma1 }; // generated samples
      lsp = 0; // context
      ls0 = line_state[lsp].get();
      line_state[lsp].set(0 as OPJ_UINT8);
      sp = ((y * stride)) as usize;
      c_q = 0 as OPJ_UINT32;
      x_0 = 0i32;
      while x_0 < width {
        let mut U_q_0: [OPJ_UINT32; 2] = [0; 2];
        let mut uvlc_mode_0: OPJ_UINT32 = 0;
        let mut consumed_bits_0: OPJ_UINT32 = 0;
        let mut m_n_0: OPJ_UINT32 = 0;
        let mut v_n_0: OPJ_UINT32 = 0;
        let mut ms_val_0: OPJ_UINT32 = 0;
        let mut locs_0: OPJ_UINT32 = 0;
        // decode vlc
        // ///////////
        //first quad
        // get context, eqn. 2 ITU T.814
        // c_q has \sigma^W | \sigma^SW
        c_q |= (ls0 as core::ffi::c_int >> 7i32) as core::ffi::c_uint; //\sigma^NW | \sigma^N
        c_q |=
          (line_state[lsp + 1].get() as core::ffi::c_int >> 5i32 & 0x4i32) as core::ffi::c_uint; //\sigma^NE | \sigma^NF
                                                                                                 //the following is very similar to previous code, so please refer to
                                                                                                 // that
        vlc_val = rev_fetch(&mut vlc);
        qinf[0_usize] = vlc_tbl1[(c_q << 7i32 | vlc_val & 0x7fu32) as usize] as OPJ_UINT32;
        if c_q == 0u32 {
          //zero context
          run -= 2i32;
          qinf[0_usize] = if run == -(1i32) { qinf[0_usize] } else { 0u32 };
          if run < 0i32 {
            run = mel_get_run(&mut mel)
          }
        }
        //prepare context for the next quad, \sigma^W | \sigma^SW
        c_q = (qinf[0_usize] & 0x40u32) >> 5i32 | (qinf[0_usize] & 0x80u32) >> 6i32;
        //remove data from vlc stream
        vlc_val = rev_advance(&mut vlc, qinf[0_usize] & 0x7u32);
        //update sigma
        // The update depends on the value of x and y; consider one OPJ_UINT32
        // if x is 0, 8, 16 and so on, and y is 2, 6, etc., then this
        // line update c locations
        //      nibble (4 bits) number   0 1 2 3 4 5 6 7
        //                         LSB   0 0 0 0 0 0 0 0
        //                               0 0 0 0 0 0 0 0
        //                               c c 0 0 0 0 0 0
        //                               c c 0 0 0 0 0 0
        sip_0
          .or(((qinf[0_usize] & 0x30u32) >> 4i32 | (qinf[0_usize] & 0xc0u32) >> 2i32) << sip_shift);
        //second quad
        qinf[1_usize] = 0 as OPJ_UINT32;
        if (x_0 + 2i32) < width {
          c_q |= (line_state[lsp + 1].get() as core::ffi::c_int >> 7i32) as core::ffi::c_uint;
          c_q |=
            (line_state[lsp + 2].get() as core::ffi::c_int >> 5i32 & 0x4i32) as core::ffi::c_uint;
          qinf[1_usize] = vlc_tbl1[(c_q << 7i32 | vlc_val & 0x7fu32) as usize] as OPJ_UINT32;
          if c_q == 0u32 {
            //zero context
            run -= 2i32;
            qinf[1_usize] = if run == -(1i32) { qinf[1_usize] } else { 0u32 };
            if run < 0i32 {
              run = mel_get_run(&mut mel)
            }
          }
          //prepare context for the next quad
          c_q = (qinf[1_usize] & 0x40u32) >> 5i32 | (qinf[1_usize] & 0x80u32) >> 6i32;
          //remove data from vlc stream
          vlc_val = rev_advance(&mut vlc, qinf[1_usize] & 0x7u32)
        }
        //update sigma
        sip_0.or(
          (qinf[1_usize] & 0x30u32 | (qinf[1_usize] & 0xc0u32) << 2i32)
            << (4u32).wrapping_add(sip_shift),
        );
        sip_0 = sip_0.offset(if x_0 & 0x7i32 != 0 { 1i32 } else { 0i32 } as isize);
        sip_shift ^= 0x10u32;
        //retrieve u
        // //////////
        uvlc_mode_0 = (qinf[0_usize] & 0x8u32) >> 3i32 | (qinf[1_usize] & 0x8u32) >> 2i32;
        consumed_bits_0 = decode_noninit_uvlc(vlc_val, uvlc_mode_0, &mut U_q_0);
        vlc_val = rev_advance(&mut vlc, consumed_bits_0);
        //calculate E^max and add it to U_q, eqns 5 and 6 in ITU T.814
        if qinf[0_usize] & 0xf0u32 & (qinf[0_usize] & 0xf0u32).wrapping_sub(1u32) != 0 {
          // is \gamma_q 1?
          let mut E = ls0 as core::ffi::c_uint & 0x7fu32; //max(E, E^NE, E^NF)
          E = if E > line_state[lsp + 1].get() as core::ffi::c_uint & 0x7fu32 {
            E
          } else {
            (line_state[lsp + 1].get() as core::ffi::c_uint) & 0x7fu32
          };
          //since U_q already has u_q + 1, we subtract 2 instead of 1
          U_q_0[0_usize] = (U_q_0[0_usize] as core::ffi::c_uint).wrapping_add(if E > 2u32 {
            E.wrapping_sub(2u32)
          } else {
            0u32
          }) as OPJ_UINT32
        }
        if qinf[1_usize] & 0xf0u32 & (qinf[1_usize] & 0xf0u32).wrapping_sub(1u32) != 0 {
          //is \gamma_q 1?
          let mut E_0 = line_state[lsp + 1].get() as core::ffi::c_uint & 0x7fu32; //max(E, E^NE, E^NF)
          E_0 = if E_0 > line_state[lsp + 2].get() as core::ffi::c_uint & 0x7fu32 {
            E_0
          } else {
            (line_state[lsp + 2].get() as core::ffi::c_uint) & 0x7fu32
          };
          //since U_q already has u_q + 1, we subtract 2 instead of 1
          U_q_0[1_usize] = (U_q_0[1_usize] as core::ffi::c_uint).wrapping_add(if E_0 > 2u32 {
            E_0.wrapping_sub(2u32)
          } else {
            0u32
          }) as OPJ_UINT32
        } //for next double quad
        if U_q_0[0_usize] > zero_bplanes_p1 || U_q_0[1_usize] > zero_bplanes_p1 {
          event_msg!(p_manager, EVT_ERROR,
                                "Malformed HT codeblock. Decoding this codeblock is stopped. U_q islarger than bitplanes + 1 \n");
          return 0i32;
        }
        ls0 = line_state[lsp + 2].get();
        line_state[lsp + 2].set(0 as OPJ_UINT8);
        line_state[lsp + 1].set(0 as OPJ_UINT8);
        //decode magsgn and update line_state
        // ///////////////////////////////////
        //locations where samples need update
        locs_0 = 0xff as OPJ_UINT32;
        if x_0 + 4i32 > width {
          locs_0 >>= (x_0 + 4i32 - width) << 1i32
        }
        locs_0 = if y + 2i32 <= height {
          locs_0
        } else {
          (locs_0) & 0x55u32
        };
        if ((qinf[0_usize] & 0xf0u32) >> 4i32 | qinf[1_usize] & 0xf0u32) & !locs_0 != 0 {
          event_msg!(p_manager, EVT_ERROR,
                              "Malformed HT codeblock. VLC code produces significant samples outside the codeblock area.\n");
          return 0i32;
        }
        if qinf[0_usize] & 0x10u32 != 0 {
          //sigma_n
          let mut val_7: OPJ_UINT32 = 0; //m_n
          ms_val_0 = frwd_fetch(&mut magsgn); //center of bin
          m_n_0 = U_q_0[0_usize].wrapping_sub(qinf[0_usize] >> 12i32 & 1u32);
          frwd_advance(&mut magsgn, m_n_0);
          val_7 = ms_val_0 << 31i32;
          v_n_0 = ms_val_0 & ((1u32) << m_n_0).wrapping_sub(1u32);
          v_n_0 |= ((qinf[0_usize] & 0x100u32) >> 8i32) << m_n_0;
          v_n_0 |= 1u32;
          ht_data[sp + (0) as usize] = val_7 | v_n_0.wrapping_add(2u32) << p.wrapping_sub(1u32)
        } else if locs_0 & 0x1u32 != 0 {
          ht_data[sp + (0) as usize] = 0 as OPJ_UINT32
        }
        if qinf[0_usize] & 0x20u32 != 0 {
          //sigma_n
          let mut val_8: OPJ_UINT32 = 0; //m_n
          let mut t_1: OPJ_UINT32 = 0; //center of bin
          ms_val_0 = frwd_fetch(&mut magsgn);
          m_n_0 = U_q_0[0_usize].wrapping_sub(qinf[0_usize] >> 13i32 & 1u32);
          frwd_advance(&mut magsgn, m_n_0);
          val_8 = ms_val_0 << 31i32;
          v_n_0 = ms_val_0 & ((1u32) << m_n_0).wrapping_sub(1u32);
          v_n_0 |= ((qinf[0_usize] & 0x200u32) >> 9i32) << m_n_0;
          v_n_0 |= 1u32;
          ht_data[sp + (stride) as usize] = val_8 | v_n_0.wrapping_add(2u32) << p.wrapping_sub(1u32);
          //update line_state: bit 7 (\sigma^N), and E^N
          t_1 = (line_state[lsp].get() as core::ffi::c_int & 0x7fi32) as OPJ_UINT32; //E^NW
          v_n_0 = (32u32).wrapping_sub(count_leading_zeros(v_n_0));
          line_state[lsp].set((0x80u32 | (if t_1 > v_n_0 { t_1 } else { v_n_0 })) as OPJ_UINT8)
        } else if locs_0 & 0x2u32 != 0 {
          ht_data[sp + (stride) as usize] = 0 as OPJ_UINT32
          //no need to update line_state
        }
        lsp += 1;
        sp += (1) as usize;
        if qinf[0_usize] & 0x40u32 != 0 {
          //sigma_n
          let mut val_9: OPJ_UINT32 = 0; //m_n
          ms_val_0 = frwd_fetch(&mut magsgn); //center of bin
          m_n_0 = U_q_0[0_usize].wrapping_sub(qinf[0_usize] >> 14i32 & 1u32);
          frwd_advance(&mut magsgn, m_n_0);
          val_9 = ms_val_0 << 31i32;
          v_n_0 = ms_val_0 & ((1u32) << m_n_0).wrapping_sub(1u32);
          v_n_0 |= ((qinf[0_usize] & 0x400u32) >> 10i32) << m_n_0;
          v_n_0 |= 1u32;
          ht_data[sp + (0) as usize] = val_9 | v_n_0.wrapping_add(2u32) << p.wrapping_sub(1u32)
        } else if locs_0 & 0x4u32 != 0 {
          ht_data[sp + (0) as usize] = 0 as OPJ_UINT32
        }
        if qinf[0_usize] & 0x80u32 != 0 {
          //sigma_n
          let mut val_10: OPJ_UINT32 = 0; //m_n
          ms_val_0 = frwd_fetch(&mut magsgn); //center of bin
          m_n_0 = U_q_0[0_usize].wrapping_sub(qinf[0_usize] >> 15i32 & 1u32);
          frwd_advance(&mut magsgn, m_n_0);
          val_10 = ms_val_0 << 31i32;
          v_n_0 = ms_val_0 & ((1u32) << m_n_0).wrapping_sub(1u32);
          v_n_0 |= ((qinf[0_usize] & 0x800u32) >> 11i32) << m_n_0;
          v_n_0 |= 1u32;
          ht_data[sp + (stride) as usize] = val_10 | v_n_0.wrapping_add(2u32) << p.wrapping_sub(1u32);
          //update line_state: bit 7 (\sigma^NW), and E^NW for next quad
          line_state[lsp]
            .set((0x80u32 | (32u32).wrapping_sub(count_leading_zeros(v_n_0))) as OPJ_UINT8)
        } else if locs_0 & 0x8u32 != 0 {
          ht_data[sp + (stride) as usize] = 0 as OPJ_UINT32
        }
        sp += (1) as usize;
        if qinf[1_usize] & 0x10u32 != 0 {
          //sigma_n
          let mut val_11: OPJ_UINT32 = 0; //m_n
          ms_val_0 = frwd_fetch(&mut magsgn); //center of bin
          m_n_0 = U_q_0[1_usize].wrapping_sub(qinf[1_usize] >> 12i32 & 1u32);
          frwd_advance(&mut magsgn, m_n_0);
          val_11 = ms_val_0 << 31i32;
          v_n_0 = ms_val_0 & ((1u32) << m_n_0).wrapping_sub(1u32);
          v_n_0 |= ((qinf[1_usize] & 0x100u32) >> 8i32) << m_n_0;
          v_n_0 |= 1u32;
          ht_data[sp + (0) as usize] = val_11 | v_n_0.wrapping_add(2u32) << p.wrapping_sub(1u32)
        } else if locs_0 & 0x10u32 != 0 {
          ht_data[sp + (0) as usize] = 0 as OPJ_UINT32
        }
        if qinf[1_usize] & 0x20u32 != 0 {
          //sigma_n
          let mut val_12: OPJ_UINT32 = 0; //m_n
          let mut t_2: OPJ_UINT32 = 0; //center of bin
          ms_val_0 = frwd_fetch(&mut magsgn);
          m_n_0 = U_q_0[1_usize].wrapping_sub(qinf[1_usize] >> 13i32 & 1u32);
          frwd_advance(&mut magsgn, m_n_0);
          val_12 = ms_val_0 << 31i32;
          v_n_0 = ms_val_0 & ((1u32) << m_n_0).wrapping_sub(1u32);
          v_n_0 |= ((qinf[1_usize] & 0x200u32) >> 9i32) << m_n_0;
          v_n_0 |= 1u32;
          ht_data[sp + (stride) as usize] = val_12 | v_n_0.wrapping_add(2u32) << p.wrapping_sub(1u32);
          //update line_state: bit 7 (\sigma^N), and E^N
          t_2 = (line_state[lsp].get() as core::ffi::c_int & 0x7fi32) as OPJ_UINT32; //E^NW
          v_n_0 = (32u32).wrapping_sub(count_leading_zeros(v_n_0));
          line_state[lsp].set((0x80u32 | (if t_2 > v_n_0 { t_2 } else { v_n_0 })) as OPJ_UINT8)
        } else if locs_0 & 0x20u32 != 0 {
          ht_data[sp + (stride) as usize] = 0 as OPJ_UINT32
          //no need to update line_state
        }
        lsp += 1;
        sp += (1) as usize;
        if qinf[1_usize] & 0x40u32 != 0 {
          //sigma_n
          let mut val_13: OPJ_UINT32 = 0; //m_n
          ms_val_0 = frwd_fetch(&mut magsgn); //center of bin
          m_n_0 = U_q_0[1_usize].wrapping_sub(qinf[1_usize] >> 14i32 & 1u32);
          frwd_advance(&mut magsgn, m_n_0);
          val_13 = ms_val_0 << 31i32;
          v_n_0 = ms_val_0 & ((1u32) << m_n_0).wrapping_sub(1u32);
          v_n_0 |= ((qinf[1_usize] & 0x400u32) >> 10i32) << m_n_0;
          v_n_0 |= 1u32;
          ht_data[sp + (0) as usize] = val_13 | v_n_0.wrapping_add(2u32) << p.wrapping_sub(1u32)
        } else if locs_0 & 0x40u32 != 0 {
          ht_data[sp + (0) as usize] = 0 as OPJ_UINT32
        }
        if qinf[1_usize] & 0x80u32 != 0 {
          //sigma_n
          let mut val_14: OPJ_UINT32 = 0; //m_n
          ms_val_0 = frwd_fetch(&mut magsgn); //center of bin
          m_n_0 = U_q_0[1_usize].wrapping_sub(qinf[1_usize] >> 15i32 & 1u32);
          frwd_advance(&mut magsgn, m_n_0);
          val_14 = ms_val_0 << 31i32;
          v_n_0 = ms_val_0 & ((1u32) << m_n_0).wrapping_sub(1u32);
          v_n_0 |= ((qinf[1_usize] & 0x800u32) >> 11i32) << m_n_0;
          v_n_0 |= 1u32;
          ht_data[sp + (stride) as usize] = val_14 | v_n_0.wrapping_add(2u32) << p.wrapping_sub(1u32);
          //update line_state: bit 7 (\sigma^NW), and E^NW for next quad
          line_state[lsp]
            .set((0x80u32 | (32u32).wrapping_sub(count_leading_zeros(v_n_0))) as OPJ_UINT8)
        } else if locs_0 & 0x80u32 != 0 {
          ht_data[sp + (stride) as usize] = 0 as OPJ_UINT32
        }
        sp += (1) as usize;
        x_0 += 4i32
      }
      y += 2i32;
      if num_passes > 1u32 && y & 3i32 == 0i32 {
        //executed at multiples of 4
        // This is for SPP and potentially MRP
        if num_passes > 2u32 {
          //do MRP
          // select the current stripe
          let mut cur_sig = if y & 0x4i32 != 0 { sigma1 } else { sigma2 };
          // the address of the data that needs updating
          let mut dpp = (((y - 4i32) * stride)) as usize; // half the center of the bin
          let mut half = (1u32) << p.wrapping_sub(2u32);
          let mut i_1: OPJ_INT32 = 0;
          i_1 = 0i32;
          while i_1 < width {
            //Process one entry from sigma array at a time
            // Each nibble (4 bits) in the sigma array represents 4 rows,
            // and the 32 bits contain 8 columns
            let mut cwd = rev_fetch_mrp(&mut magref); // get 32 bit data
            let sig = cur_sig.get(); // 32 bit that will be processed now
            cur_sig = cur_sig.offset(1); // a mask for a column in sig
            let mut col_mask = 0xfu32; // next column in decode samples
            let mut dp = dpp + (i_1) as usize;
            if sig != 0 {
              // if any of the 32 bits are set
              let mut j: core::ffi::c_int = 0;
              j = 0i32;
              while j < 8i32 {
                //one column at a time
                if sig & col_mask != 0 {
                  // lowest nibble
                  let mut sample_mask = 0x11111111u32 & col_mask; //LSB
                  if sig & sample_mask != 0 {
                    //if LSB is set
                    let mut sym: OPJ_UINT32 = 0; // decoded value cannot be zero
                    assert!(ht_data[dp + (0) as usize] != 0u32);
                    sym = cwd & 1u32;
                    // remove center of bin if sym is 0
                    let fresh16 = &mut (ht_data[dp + (0) as usize]); // put half the center of bin
                    *fresh16 ^= (1u32).wrapping_sub(sym) << p.wrapping_sub(1u32); //next row
                    let fresh17 = &mut (ht_data[dp + (0) as usize]);
                    *fresh17 |= half;
                    cwd >>= 1i32
                  }
                  sample_mask = (sample_mask as core::ffi::c_uint).wrapping_add(sample_mask)
                    as OPJ_UINT32 as OPJ_UINT32;
                  if sig & sample_mask != 0 {
                    let mut sym_0: OPJ_UINT32 = 0;
                    assert!(ht_data[dp + (stride) as usize] != 0u32);
                    sym_0 = cwd & 1u32;
                    let fresh18 = &mut (ht_data[dp + (stride) as usize]);
                    *fresh18 ^= (1u32).wrapping_sub(sym_0) << p.wrapping_sub(1u32);
                    let fresh19 = &mut (ht_data[dp + (stride) as usize]);
                    *fresh19 |= half;
                    cwd >>= 1i32
                  }
                  sample_mask = (sample_mask as core::ffi::c_uint).wrapping_add(sample_mask)
                    as OPJ_UINT32 as OPJ_UINT32;
                  if sig & sample_mask != 0 {
                    let mut sym_1: OPJ_UINT32 = 0;
                    assert!(ht_data[dp + ((2i32 * stride)) as usize] != 0u32);
                    sym_1 = cwd & 1u32;
                    let fresh20 = &mut (ht_data[dp + ((2i32 * stride)) as usize]);
                    *fresh20 ^= (1u32).wrapping_sub(sym_1) << p.wrapping_sub(1u32);
                    let fresh21 = &mut (ht_data[dp + ((2i32 * stride)) as usize]);
                    *fresh21 |= half;
                    cwd >>= 1i32
                  }
                  sample_mask = (sample_mask as core::ffi::c_uint).wrapping_add(sample_mask)
                    as OPJ_UINT32 as OPJ_UINT32;
                  if sig & sample_mask != 0 {
                    let mut sym_2: OPJ_UINT32 = 0;
                    assert!(ht_data[dp + ((3i32 * stride)) as usize] != 0u32);
                    sym_2 = cwd & 1u32;
                    let fresh22 = &mut (ht_data[dp + ((3i32 * stride)) as usize]);
                    *fresh22 ^= (1u32).wrapping_sub(sym_2) << p.wrapping_sub(1u32);
                    let fresh23 = &mut (ht_data[dp + ((3i32 * stride)) as usize]);
                    *fresh23 |= half;
                    cwd >>= 1i32
                  }
                  sample_mask = (sample_mask as core::ffi::c_uint).wrapping_add(sample_mask)
                    as OPJ_UINT32 as OPJ_UINT32
                }
                col_mask <<= 4i32;
                j += 1;
                dp += (1) as usize;
                //next column
              }
            }
            // consume data according to the number of bits set
            rev_advance_mrp(&mut magref, population_count(sig));
            i_1 += 8i32
          }
        }
        if y >= 4i32 {
          // update mbr array at the end of each stripe
          //generate mbr corresponding to a stripe
          let mut sig_0 = if y & 0x4i32 != 0 { sigma1 } else { sigma2 };
          let mut mbr = if y & 0x4i32 != 0 { mbr1 } else { mbr2 };
          //data is processed in patches of 8 columns, each
          // each 32 bits in sigma1 or mbr1 represent 4 rows
          //integrate horizontally
          let mut prev = 0 as OPJ_UINT32; // previous columns
          let mut i_2: OPJ_INT32 = 0;
          i_2 = 0i32;
          while i_2 < width {
            let mut t_3: OPJ_UINT32 = 0;
            let mut z: OPJ_UINT32 = 0;
            //remove already significance samples
            mbr.set(sig_0.get()); //start with significant samples
            mbr.or(prev >> 28i32); //for first column, left neighbors
            mbr.or(sig_0.get() << 4i32); //left neighbors
            mbr.or(sig_0.get() >> 4i32); //right neighbors
            mbr.or(sig_0.at(1) << 28i32); //for last column, right neighbors
            prev = sig_0.get(); // for next group of columns
            t_3 = mbr.get();
            z = mbr.get();
            z |= (t_3 & 0x77777777u32) << 1i32;
            z |= (t_3 & 0xeeeeeeeeu32) >> 1i32;
            mbr.set(z & !sig_0.get());
            i_2 += 8i32;
            mbr = mbr.offset(1);
            sig_0 = sig_0.offset(1)
          }
        }
        if y >= 8i32 {
          //integrate vertically
          //above neighbors
          //below neighbors
          //wait until 8 rows has been processed
          let mut cur_sig_0: FlagsPtr;
          let mut cur_mbr: FlagsPtr;
          let mut nxt_sig: FlagsPtr;
          let mut nxt_mbr: FlagsPtr;
          let mut prev_0: OPJ_UINT32 = 0;
          let mut val_15: OPJ_UINT32 = 0;
          let mut i_3: OPJ_INT32 = 0;
          // add membership from the next stripe, obtained above
          cur_sig_0 = if y & 0x4i32 != 0 { sigma2 } else { sigma1 }; //future samples
          cur_mbr = if y & 0x4i32 != 0 { mbr2 } else { mbr1 }; // the columns before these group of 8 columns
          nxt_sig = if y & 0x4i32 != 0 { sigma1 } else { sigma2 };
          prev_0 = 0 as OPJ_UINT32;
          i_3 = 0i32;
          while i_3 < width {
            let mut t_4 = nxt_sig.get();
            //remove already significance samples
            t_4 |= prev_0 >> 28i32; //for first column, left neighbors
            t_4 |= nxt_sig.get() << 4i32; //left neighbors
            t_4 |= nxt_sig.get() >> 4i32; //right neighbors
            t_4 |= nxt_sig.at(1) << 28i32; //for last column, right neighbors
            prev_0 = nxt_sig.get(); // for next group of columns
            if stripe_causal == 0 {
              cur_mbr.or((t_4 & 0x11111111u32) << 3i32)
              //propagate up to cur_mbr
            }
            cur_mbr.clear(cur_sig_0.get());
            i_3 += 8i32;
            cur_mbr = cur_mbr.offset(1);
            cur_sig_0 = cur_sig_0.offset(1);
            nxt_sig = nxt_sig.offset(1)
          }
          //find new locations and get signs
          cur_sig_0 = if y & 0x4i32 != 0 { sigma2 } else { sigma1 }; //future samples
          cur_mbr = if y & 0x4i32 != 0 { mbr2 } else { mbr1 }; //future samples
          nxt_sig = if y & 0x4i32 != 0 { sigma1 } else { sigma2 }; // sample values for newly discovered
          nxt_mbr = if y & 0x4i32 != 0 { mbr1 } else { mbr2 };
          val_15 = (3u32) << p.wrapping_sub(2u32);
          // significant samples including the bin center
          i_3 = 0i32;
          while i_3 < width {
            let mut ux: OPJ_UINT32 = 0;
            let mut tx: OPJ_UINT32 = 0;
            let mut mbr_0 = cur_mbr.get();
            let mut new_sig = 0 as OPJ_UINT32;
            if mbr_0 != 0 {
              //are there any samples that might be significant
              let mut n: OPJ_INT32 = 0; //get 32 bits
              n = 0i32; //address for decoded samples
              while n < 8i32 {
                let mut col_mask_0: OPJ_UINT32 = 0; //a mask to select a column
                let mut inv_sig: OPJ_UINT32 = 0; // insignificant samples
                let mut end: OPJ_INT32 = 0;
                let mut j_0: OPJ_INT32 = 0;
                let mut cwd_0 = frwd_fetch(&mut sigprop);
                let mut cnt = 0 as OPJ_UINT32;
                let mut dp_0 = (((y - 8i32) * stride)) as usize;
                dp_0 += ((i_3 + n)) as usize;
                col_mask_0 = (0xfu32) << (4i32 * n);
                inv_sig = !cur_sig_0.get();
                //find the last sample we operate on
                end = if n + 4i32 + i_3 < width {
                  (n) + 4i32
                } else {
                  (width) - i_3
                };
                j_0 = n;
                while j_0 < end {
                  let mut sample_mask_0: OPJ_UINT32 = 0;
                  if col_mask_0 & mbr_0 != 0u32 {
                    //scan mbr to find a new significant sample
                    sample_mask_0 = 0x11111111u32 & col_mask_0; // LSB
                    if mbr_0 & sample_mask_0 != 0 {
                      assert!(ht_data[dp_0 + (0) as usize] == 0u32);
                      if cwd_0 & 1u32 != 0 {
                        //consume bit and increment number of
                        //consumed bits
                        //if this sample has become significant
                        // must propagate it to nearby samples
                        let mut t_5: OPJ_UINT32 = 0; // new significant samples
                        new_sig |= sample_mask_0; // propagation to neighbors
                        t_5 = (0x32u32) << (j_0 * 4i32); // next row
                        mbr_0 |= t_5 & inv_sig
                      }
                      cwd_0 >>= 1i32;
                      cnt += 1;
                    }
                    sample_mask_0 = (sample_mask_0 as core::ffi::c_uint).wrapping_add(sample_mask_0)
                      as OPJ_UINT32;
                    if mbr_0 & sample_mask_0 != 0 {
                      assert!(ht_data[dp_0 + (stride) as usize] == 0u32);
                      if cwd_0 & 1u32 != 0 {
                        let mut t_6: OPJ_UINT32 = 0;
                        new_sig |= sample_mask_0;
                        t_6 = (0x74u32) << (j_0 * 4i32);
                        mbr_0 |= t_6 & inv_sig
                      }
                      cwd_0 >>= 1i32;
                      cnt += 1;
                    }
                    sample_mask_0 = (sample_mask_0 as core::ffi::c_uint).wrapping_add(sample_mask_0)
                      as OPJ_UINT32;
                    if mbr_0 & sample_mask_0 != 0 {
                      assert!(ht_data[dp_0 + ((2i32 * stride)) as usize] == 0u32);
                      if cwd_0 & 1u32 != 0 {
                        let mut t_7: OPJ_UINT32 = 0;
                        new_sig |= sample_mask_0;
                        t_7 = (0xe8u32) << (j_0 * 4i32);
                        mbr_0 |= t_7 & inv_sig
                      }
                      cwd_0 >>= 1i32;
                      cnt += 1;
                    }
                    sample_mask_0 = (sample_mask_0 as core::ffi::c_uint).wrapping_add(sample_mask_0)
                      as OPJ_UINT32;
                    if mbr_0 & sample_mask_0 != 0 {
                      assert!(ht_data[dp_0 + ((3i32 * stride)) as usize] == 0u32);
                      if cwd_0 & 1u32 != 0 {
                        let mut t_8: OPJ_UINT32 = 0;
                        new_sig |= sample_mask_0;
                        t_8 = (0xc0u32) << (j_0 * 4i32);
                        mbr_0 |= t_8 & inv_sig
                      }
                      cwd_0 >>= 1i32;
                      cnt += 1;
                    }
                  }
                  //no samples need checking
                  j_0 += 1;
                  dp_0 += (1) as usize;
                  col_mask_0 <<= 4i32
                }
                //obtain signs here
                if new_sig & (0xffffu32) << (4i32 * n) != 0 {
                  //if any
                  let mut col_mask_1: OPJ_UINT32 = 0; // decoded samples address
                  let mut j_1: OPJ_INT32 = 0; //mask to select a column
                  let mut dp_1 = (((y - 8i32) * stride)) as usize;
                  dp_1 += ((i_3 + n)) as usize;
                  col_mask_1 = (0xfu32) << (4i32 * n);
                  j_1 = n;
                  while j_1 < end {
                    let mut sample_mask_1: OPJ_UINT32 = 0;
                    if col_mask_1 & new_sig != 0u32 {
                      //scan 4 signs
                      sample_mask_1 = 0x11111111u32 & col_mask_1;
                      if new_sig & sample_mask_1 != 0 {
                        assert!(ht_data[dp_1 + (0) as usize] == 0u32);
                        //consume bit and increment number
                        //of consumed bits
                        let fresh30 = &mut (ht_data[dp_1 + (0) as usize]); //put value and sign
                        *fresh30 |= (cwd_0 & 1u32) << 31i32 | val_15;
                        cwd_0 >>= 1i32;
                        cnt += 1;
                      }
                      sample_mask_1 = (sample_mask_1 as core::ffi::c_uint)
                        .wrapping_add(sample_mask_1)
                        as OPJ_UINT32;
                      if new_sig & sample_mask_1 != 0 {
                        assert!(ht_data[dp_1 + (stride) as usize] == 0u32);
                        let fresh31 = &mut (ht_data[dp_1 + (stride) as usize]);
                        *fresh31 |= (cwd_0 & 1u32) << 31i32 | val_15;
                        cwd_0 >>= 1i32;
                        cnt += 1;
                      }
                      sample_mask_1 = (sample_mask_1 as core::ffi::c_uint)
                        .wrapping_add(sample_mask_1)
                        as OPJ_UINT32;
                      if new_sig & sample_mask_1 != 0 {
                        assert!(ht_data[dp_1 + ((2i32 * stride)) as usize] == 0u32);
                        let fresh32 = &mut (ht_data[dp_1 + ((2i32 * stride)) as usize]);
                        *fresh32 |= (cwd_0 & 1u32) << 31i32 | val_15;
                        cwd_0 >>= 1i32;
                        cnt += 1;
                      }
                      sample_mask_1 = (sample_mask_1 as core::ffi::c_uint)
                        .wrapping_add(sample_mask_1)
                        as OPJ_UINT32;
                      if new_sig & sample_mask_1 != 0 {
                        assert!(ht_data[dp_1 + ((3i32 * stride)) as usize] == 0u32);
                        let fresh33 = &mut (ht_data[dp_1 + ((3i32 * stride)) as usize]);
                        *fresh33 |= (cwd_0 & 1u32) << 31i32 | val_15;
                        cwd_0 >>= 1i32;
                        cnt += 1;
                      }
                    }
                    //if non is significant
                    j_1 += 1; //consume the bits from bitstrm
                    dp_1 += (1) as usize;
                    col_mask_1 <<= 4i32
                  }
                }
                frwd_advance(&mut sigprop, cnt);
                cnt = 0 as OPJ_UINT32;
                //update the next 8 columns
                if n == 4i32 {
                  //horizontally
                  let mut t_9 = new_sig >> 28i32;
                  t_9 |= (t_9 & 0xeu32) >> 1i32 | (t_9 & 7u32) << 1i32;
                  cur_mbr.or_at(1, t_9 & !cur_sig_0.at(1))
                }
                n += 4i32
              }
            }
            //update the next stripe (vertically propagation)
            new_sig |= cur_sig_0.get(); //left and right neighbors
            ux = (new_sig & 0x88888888u32) >> 3i32;
            tx = ux | ux << 4i32 | ux >> 4i32;
            if i_3 > 0i32 {
              nxt_mbr.or_at(-1, ux << 28i32 & !nxt_sig.at(-1))
            }
            nxt_mbr.or_at(0, tx & !nxt_sig.at(0));
            nxt_mbr.or_at(1, ux >> 28i32 & !nxt_sig.at(1));
            i_3 += 8i32;
            cur_sig_0 = cur_sig_0.offset(1);
            cur_mbr = cur_mbr.offset(1);
            nxt_sig = nxt_sig.offset(1);
            nxt_mbr = nxt_mbr.offset(1)
          }
          //clear current sigma
          //mbr need not be cleared because it is overwritten
          cur_sig_0 = if y & 0x4i32 != 0 { sigma2 } else { sigma1 };
          let clear_len = ((width as OPJ_UINT32).wrapping_add(7u32) >> 3i32).wrapping_add(1u32);
          for k in 0..clear_len as isize {
            cur_sig_0.set_at(k, 0);
          }
        }
      }
    }
    //terminating
    if num_passes > 1u32 {
      let mut st: OPJ_INT32 = 0;
      let mut y_0: OPJ_INT32 = 0;
      if num_passes > 2u32 && (height & 3i32 == 1i32 || height & 3i32 == 2i32) {
        //do magref
        let mut cur_sig_1 = if height & 0x4i32 != 0 { sigma2 } else { sigma1 }; //reversed
        let mut dpp_0 = (((height & 0xfffffci32) * stride)) as usize;
        let mut half_0 = (1u32) << p.wrapping_sub(2u32);
        let mut i_4: OPJ_INT32 = 0;
        i_4 = 0i32;
        while i_4 < width {
          let mut cwd_1 = rev_fetch_mrp(&mut magref);
          let sig_1 = cur_sig_1.get();
          cur_sig_1 = cur_sig_1.offset(1);
          let mut col_mask_2 = 0xf as OPJ_UINT32;
          let mut dp_2 = dpp_0 + (i_4) as usize;
          if sig_1 != 0 {
            let mut j_2: core::ffi::c_int = 0;
            j_2 = 0i32;
            while j_2 < 8i32 {
              if sig_1 & col_mask_2 != 0 {
                let mut sample_mask_2 = 0x11111111u32 & col_mask_2;
                if sig_1 & sample_mask_2 != 0 {
                  let mut sym_3: OPJ_UINT32 = 0;
                  assert!(ht_data[dp_2 + (0) as usize] != 0u32);
                  sym_3 = cwd_1 & 1u32;
                  let fresh39 = &mut (ht_data[dp_2 + (0) as usize]);
                  *fresh39 ^= (1u32).wrapping_sub(sym_3) << p.wrapping_sub(1u32);
                  let fresh40 = &mut (ht_data[dp_2 + (0) as usize]);
                  *fresh40 |= half_0;
                  cwd_1 >>= 1i32
                }
                sample_mask_2 =
                  (sample_mask_2 as core::ffi::c_uint).wrapping_add(sample_mask_2) as OPJ_UINT32;
                if sig_1 & sample_mask_2 != 0 {
                  let mut sym_4: OPJ_UINT32 = 0;
                  assert!(ht_data[dp_2 + (stride) as usize] != 0u32);
                  sym_4 = cwd_1 & 1u32;
                  let fresh41 = &mut (ht_data[dp_2 + (stride) as usize]);
                  *fresh41 ^= (1u32).wrapping_sub(sym_4) << p.wrapping_sub(1u32);
                  let fresh42 = &mut (ht_data[dp_2 + (stride) as usize]);
                  *fresh42 |= half_0;
                  cwd_1 >>= 1i32
                }
                sample_mask_2 =
                  (sample_mask_2 as core::ffi::c_uint).wrapping_add(sample_mask_2) as OPJ_UINT32;
                if sig_1 & sample_mask_2 != 0 {
                  let mut sym_5: OPJ_UINT32 = 0;
                  assert!(ht_data[dp_2 + ((2i32 * stride)) as usize] != 0u32);
                  sym_5 = cwd_1 & 1u32;
                  let fresh43 = &mut (ht_data[dp_2 + ((2i32 * stride)) as usize]);
                  *fresh43 ^= (1u32).wrapping_sub(sym_5) << p.wrapping_sub(1u32);
                  let fresh44 = &mut (ht_data[dp_2 + ((2i32 * stride)) as usize]);
                  *fresh44 |= half_0;
                  cwd_1 >>= 1i32
                }
                sample_mask_2 =
                  (sample_mask_2 as core::ffi::c_uint).wrapping_add(sample_mask_2) as OPJ_UINT32;
                if sig_1 & sample_mask_2 != 0 {
                  let mut sym_6: OPJ_UINT32 = 0;
                  assert!(ht_data[dp_2 + ((3i32 * stride)) as usize] != 0u32);
                  sym_6 = cwd_1 & 1u32;
                  let fresh45 = &mut (ht_data[dp_2 + ((3i32 * stride)) as usize]);
                  *fresh45 ^= (1u32).wrapping_sub(sym_6) << p.wrapping_sub(1u32);
                  let fresh46 = &mut (ht_data[dp_2 + ((3i32 * stride)) as usize]);
                  *fresh46 |= half_0;
                  cwd_1 >>= 1i32
                }
                sample_mask_2 =
                  (sample_mask_2 as core::ffi::c_uint).wrapping_add(sample_mask_2) as OPJ_UINT32
              }
              col_mask_2 <<= 4i32;
              j_2 += 1;
              dp_2 += (1) as usize;
            }
          }
          rev_advance_mrp(&mut magref, population_count(sig_1));
          i_4 += 8i32
        }
      }
      //do the last incomplete stripe
      // for cases of (height & 3) == 0 and 3
      // the should have been processed previously
      if height & 3i32 == 1i32 || height & 3i32 == 2i32 {
        //generate mbr of first stripe
        let mut sig_2 = if height & 0x4i32 != 0 { sigma2 } else { sigma1 };
        let mut mbr_1 = if height & 0x4i32 != 0 { mbr2 } else { mbr1 };
        //integrate horizontally
        let mut prev_1 = 0 as OPJ_UINT32;
        let mut i_5: OPJ_INT32 = 0;
        i_5 = 0i32;
        while i_5 < width {
          let mut t_10: OPJ_UINT32 = 0;
          let mut z_0: OPJ_UINT32 = 0;
          mbr_1.set(sig_2.get());
          //remove already significance samples
          mbr_1.or(prev_1 >> 28i32); //for first column, left neighbors
          mbr_1.or(sig_2.get() << 4i32); //left neighbors
          mbr_1.or(sig_2.get() >> 4i32); //left neighbors
          mbr_1.or(sig_2.at(1) << 28i32); //for last column, right neighbors
          prev_1 = sig_2.get();
          t_10 = mbr_1.get();
          z_0 = mbr_1.get();
          z_0 |= (t_10 & 0x77777777u32) << 1i32;
          z_0 |= (t_10 & 0xeeeeeeeeu32) >> 1i32;
          mbr_1.set(z_0 & !sig_2.get());
          i_5 += 8i32;
          mbr_1 = mbr_1.offset(1);
          sig_2 = sig_2.offset(1)
        }
      }
      st = height;
      st -= if height > 6i32 {
        ((height + 1i32) & 3i32) + 3i32
      } else {
        height
      };
      y_0 = st;
      while y_0 < height {
        let mut cur_sig_2: FlagsPtr;
        let mut cur_mbr_0: FlagsPtr;
        let mut nxt_sig_0: FlagsPtr;
        let mut nxt_mbr_0: FlagsPtr;
        let mut val_16: OPJ_UINT32 = 0;
        let mut i_6: OPJ_INT32 = 0;
        //integrate vertically
        //above neighbors
        //below neighbors
        let mut pattern = 0xffffffffu32; // a pattern needed samples
        if height - y_0 == 3i32 {
          pattern = 0x77777777u32
        } else if height - y_0 == 2i32 {
          pattern = 0x33333333u32
        } else if height - y_0 == 1i32 {
          pattern = 0x11111111u32
        }
        //add membership from the next stripe, obtained above
        if height - y_0 > 4i32 {
          let mut prev_2 = 0 as OPJ_UINT32; //for first column, left neighbors
          let mut i_7: OPJ_INT32 = 0; //left neighbors
          cur_sig_2 = if y_0 & 0x4i32 != 0 { sigma2 } else { sigma1 }; //left neighbors
          cur_mbr_0 = if y_0 & 0x4i32 != 0 { mbr2 } else { mbr1 }; //for last column, right neighbors
          nxt_sig_0 = if y_0 & 0x4i32 != 0 { sigma1 } else { sigma2 };
          i_7 = 0i32;
          while i_7 < width {
            let mut t_11 = nxt_sig_0.get();
            t_11 |= prev_2 >> 28i32;
            t_11 |= nxt_sig_0.get() << 4i32;
            t_11 |= nxt_sig_0.get() >> 4i32;
            t_11 |= nxt_sig_0.at(1) << 28i32;
            prev_2 = nxt_sig_0.get();
            if stripe_causal == 0 {
              cur_mbr_0.or((t_11 & 0x11111111u32) << 3i32)
            }
            //remove already significance samples
            cur_mbr_0.clear(cur_sig_2.get());
            i_7 += 8i32;
            cur_mbr_0 = cur_mbr_0.offset(1);
            cur_sig_2 = cur_sig_2.offset(1);
            nxt_sig_0 = nxt_sig_0.offset(1)
          }
        }
        //find new locations and get signs
        cur_sig_2 = if y_0 & 0x4i32 != 0 { sigma2 } else { sigma1 }; //skip unneeded samples
        cur_mbr_0 = if y_0 & 0x4i32 != 0 { mbr2 } else { mbr1 };
        nxt_sig_0 = if y_0 & 0x4i32 != 0 { sigma1 } else { sigma2 };
        nxt_mbr_0 = if y_0 & 0x4i32 != 0 { mbr1 } else { mbr2 };
        val_16 = (3u32) << p.wrapping_sub(2u32);
        i_6 = 0i32;
        while i_6 < width {
          let mut mbr_2 = cur_mbr_0.get() & pattern;
          let mut new_sig_0 = 0 as OPJ_UINT32;
          let mut ux_0: OPJ_UINT32 = 0;
          let mut tx_0: OPJ_UINT32 = 0;
          if mbr_2 != 0 {
            let mut n_0: OPJ_INT32 = 0;
            n_0 = 0i32;
            while n_0 < 8i32 {
              let mut col_mask_3: OPJ_UINT32 = 0;
              let mut inv_sig_0: OPJ_UINT32 = 0;
              let mut end_0: OPJ_INT32 = 0;
              let mut j_3: OPJ_INT32 = 0;
              let mut cwd_2 = frwd_fetch(&mut sigprop);
              let mut cnt_0 = 0 as OPJ_UINT32;
              let mut dp_3 = ((y_0 * stride)) as usize;
              dp_3 += ((i_6 + n_0)) as usize;
              col_mask_3 = (0xfu32) << (4i32 * n_0);
              inv_sig_0 = !cur_sig_2.get() & pattern;
              end_0 = if n_0 + 4i32 + i_6 < width {
                (n_0) + 4i32
              } else {
                (width) - i_6
              };
              j_3 = n_0;
              while j_3 < end_0 {
                let mut sample_mask_3: OPJ_UINT32 = 0;
                if col_mask_3 & mbr_2 != 0u32 {
                  //scan 4 mbr
                  sample_mask_3 = 0x11111111u32 & col_mask_3;
                  if mbr_2 & sample_mask_3 != 0 {
                    assert!(ht_data[dp_3 + (0) as usize] == 0u32);
                    if cwd_2 & 1u32 != 0 {
                      let mut t_12: OPJ_UINT32 = 0;
                      new_sig_0 |= sample_mask_3;
                      t_12 = (0x32u32) << (j_3 * 4i32);
                      mbr_2 |= t_12 & inv_sig_0
                    }
                    cwd_2 >>= 1i32;
                    cnt_0 += 1;
                  }
                  sample_mask_3 =
                    (sample_mask_3 as core::ffi::c_uint).wrapping_add(sample_mask_3) as OPJ_UINT32;
                  if mbr_2 & sample_mask_3 != 0 {
                    assert!(ht_data[dp_3 + (stride) as usize] == 0u32);
                    if cwd_2 & 1u32 != 0 {
                      let mut t_13: OPJ_UINT32 = 0;
                      new_sig_0 |= sample_mask_3;
                      t_13 = (0x74u32) << (j_3 * 4i32);
                      mbr_2 |= t_13 & inv_sig_0
                    }
                    cwd_2 >>= 1i32;
                    cnt_0 += 1;
                  }
                  sample_mask_3 =
                    (sample_mask_3 as core::ffi::c_uint).wrapping_add(sample_mask_3) as OPJ_UINT32;
                  if mbr_2 & sample_mask_3 != 0 {
                    assert!(ht_data[dp_3 + ((2i32 * stride)) as usize] == 0u32);
                    if cwd_2 & 1u32 != 0 {
                      let mut t_14: OPJ_UINT32 = 0;
                      new_sig_0 |= sample_mask_3;
                      t_14 = (0xe8u32) << (j_3 * 4i32);
                      mbr_2 |= t_14 & inv_sig_0
                    }
                    cwd_2 >>= 1i32;
                    cnt_0 += 1;
                  }
                  sample_mask_3 =
                    (sample_mask_3 as core::ffi::c_uint).wrapping_add(sample_mask_3) as OPJ_UINT32;
                  if mbr_2 & sample_mask_3 != 0 {
                    assert!(ht_data[dp_3 + ((3i32 * stride)) as usize] == 0u32);
                    if cwd_2 & 1u32 != 0 {
                      let mut t_15: OPJ_UINT32 = 0;
                      new_sig_0 |= sample_mask_3;
                      t_15 = (0xc0u32) << (j_3 * 4i32);
                      mbr_2 |= t_15 & inv_sig_0
                    }
                    cwd_2 >>= 1i32;
                    cnt_0 += 1;
                  }
                }
                j_3 += 1;
                dp_3 += (1) as usize;
                col_mask_3 <<= 4i32
              }
              //signs here
              if new_sig_0 & (0xffffu32) << (4i32 * n_0) != 0 {
                let mut col_mask_4: OPJ_UINT32 = 0;
                let mut j_4: OPJ_INT32 = 0;
                let mut dp_4 = ((y_0 * stride)) as usize;
                dp_4 += ((i_6 + n_0)) as usize;
                col_mask_4 = (0xfu32) << (4i32 * n_0);
                j_4 = n_0;
                while j_4 < end_0 {
                  let mut sample_mask_4: OPJ_UINT32 = 0;
                  if col_mask_4 & new_sig_0 != 0u32 {
                    //scan 4 signs
                    sample_mask_4 = 0x11111111u32 & col_mask_4;
                    if new_sig_0 & sample_mask_4 != 0 {
                      assert!(ht_data[dp_4 + (0) as usize] == 0u32);
                      let fresh53 = &mut (ht_data[dp_4 + (0) as usize]);
                      *fresh53 |= (cwd_2 & 1u32) << 31i32 | val_16;
                      cwd_2 >>= 1i32;
                      cnt_0 += 1;
                    }
                    sample_mask_4 = (sample_mask_4 as core::ffi::c_uint).wrapping_add(sample_mask_4)
                      as OPJ_UINT32;
                    if new_sig_0 & sample_mask_4 != 0 {
                      assert!(ht_data[dp_4 + (stride) as usize] == 0u32);
                      let fresh54 = &mut (ht_data[dp_4 + (stride) as usize]);
                      *fresh54 |= (cwd_2 & 1u32) << 31i32 | val_16;
                      cwd_2 >>= 1i32;
                      cnt_0 += 1;
                    }
                    sample_mask_4 = (sample_mask_4 as core::ffi::c_uint).wrapping_add(sample_mask_4)
                      as OPJ_UINT32;
                    if new_sig_0 & sample_mask_4 != 0 {
                      assert!(ht_data[dp_4 + ((2i32 * stride)) as usize] == 0u32);
                      let fresh55 = &mut (ht_data[dp_4 + ((2i32 * stride)) as usize]);
                      *fresh55 |= (cwd_2 & 1u32) << 31i32 | val_16;
                      cwd_2 >>= 1i32;
                      cnt_0 += 1;
                    }
                    sample_mask_4 = (sample_mask_4 as core::ffi::c_uint).wrapping_add(sample_mask_4)
                      as OPJ_UINT32;
                    if new_sig_0 & sample_mask_4 != 0 {
                      assert!(ht_data[dp_4 + ((3i32 * stride)) as usize] == 0u32);
                      let fresh56 = &mut (ht_data[dp_4 + ((3i32 * stride)) as usize]);
                      *fresh56 |= (cwd_2 & 1u32) << 31i32 | val_16;
                      cwd_2 >>= 1i32;
                      cnt_0 += 1;
                    }
                  }
                  j_4 += 1;
                  dp_4 += (1) as usize;
                  col_mask_4 <<= 4i32
                }
              }
              frwd_advance(&mut sigprop, cnt_0);
              cnt_0 = 0 as OPJ_UINT32;
              //update next columns
              if n_0 == 4i32 {
                //horizontally
                let mut t_16 = new_sig_0 >> 28i32;
                t_16 |= (t_16 & 0xeu32) >> 1i32 | (t_16 & 7u32) << 1i32;
                cur_mbr_0.or_at(1, t_16 & !cur_sig_2.at(1))
              }
              n_0 += 4i32
            }
          }
          //propagate down (vertically propagation)
          new_sig_0 |= cur_sig_2.get();
          ux_0 = (new_sig_0 & 0x88888888u32) >> 3i32;
          tx_0 = ux_0 | ux_0 << 4i32 | ux_0 >> 4i32;
          if i_6 > 0i32 {
            nxt_mbr_0.or_at(-1, ux_0 << 28i32 & !nxt_sig_0.at(-1))
          }
          nxt_mbr_0.or_at(0, tx_0 & !nxt_sig_0.at(0));
          nxt_mbr_0.or_at(1, ux_0 >> 28i32 & !nxt_sig_0.at(1));
          i_6 += 8i32;
          cur_sig_2 = cur_sig_2.offset(1);
          cur_mbr_0 = cur_mbr_0.offset(1);
          nxt_sig_0 = nxt_sig_0.offset(1);
          nxt_mbr_0 = nxt_mbr_0.offset(1)
        }
        y_0 += 4i32
      }
    }
    let mut x_1: OPJ_INT32 = 0;
    let mut y_1: OPJ_INT32 = 0;
    y_1 = 0i32;
    while y_1 < height {
      let mut sp_0 = ((y_1 * stride)) as usize;
      x_1 = 0i32;
      while x_1 < width {
        let val_17 = ht_data[sp_0] & 0x7fffffffu32;
        ht_data[sp_0] = if ht_data[sp_0] & 0x80000000u32 != 0 {
          0u32.wrapping_sub(val_17)
        } else {
          val_17
        };
        x_1 += 1;
        sp_0 += (1) as usize;
      }
      y_1 += 1
    }

    1i32
}
