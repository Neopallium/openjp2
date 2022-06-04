use super::openjpeg::*;
use super::event::*;
use super::t1::*;
use super::t1_ht_luts::*;
use super::thread::*;
use ::libc;

use super::malloc::*;

extern "C" {
  fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;

  fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct dec_mel {
  pub data: *mut OPJ_UINT8,
  pub tmp: OPJ_UINT64,
  pub bits: libc::c_int,
  pub size: libc::c_int,
  pub unstuff: OPJ_BOOL,
  pub k: libc::c_int,
  pub num_runs: libc::c_int,
  pub runs: OPJ_UINT64,
}
//* ***********************************************************************/
/* * @brief MEL state structure for reading and decoding the MEL bitstream
 *
 *  A number of events is decoded from the MEL bitstream ahead of time
 *  and stored in run/num_runs.
 *  Each run represents the number of zero events before a one event.
 */
pub type dec_mel_t = dec_mel;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct rev_struct {
  pub data: *mut OPJ_UINT8,
  pub tmp: OPJ_UINT64,
  pub bits: OPJ_UINT32,
  pub size: libc::c_int,
  pub unstuff: OPJ_BOOL,
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
pub type rev_struct_t = rev_struct;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct frwd_struct {
  pub data: *const OPJ_UINT8,
  pub tmp: OPJ_UINT64,
  pub bits: OPJ_UINT32,
  pub unstuff: OPJ_BOOL,
  pub size: libc::c_int,
  pub X: OPJ_UINT32,
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
pub type frwd_struct_t = frwd_struct;
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
static mut only_cleanup_pass_is_decoded: OPJ_BOOL = 0 as libc::c_int;
//* ***********************************************************************/
/* * @brief Generates population count (i.e., the number of set bits)
 *
 *   @param [in]  val is the value for which population count is sought
 */
#[inline]
fn population_count(mut val: OPJ_UINT32) -> OPJ_UINT32 {
  return val.count_ones() as i32 as OPJ_UINT32;
}
//* ***********************************************************************/
/* * @brief Counts the number of leading zeros
 *
 *   @param [in]  val is the value for which leading zero count is sought
 */
#[inline]
fn count_leading_zeros(mut val: OPJ_UINT32) -> OPJ_UINT32 {
  return val.leading_zeros() as i32 as OPJ_UINT32;
}
//* ***********************************************************************/
/* * @brief Read a little-endian serialized UINT32.
 *
 *   @param [in]  dataIn pointer to byte stream to read from
 */
#[inline]
unsafe fn read_le_uint32(mut dataIn: *const libc::c_void) -> OPJ_UINT32 {
  return *(dataIn as *mut OPJ_UINT32);
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
#[inline]
unsafe fn mel_read(mut melp: *mut dec_mel_t) {
  let mut val: OPJ_UINT32 = 0;
  let mut bits: libc::c_int = 0;
  let mut t: OPJ_UINT32 = 0;
  let mut unstuff: OPJ_BOOL = 0;
  if (*melp).bits > 32 as libc::c_int {
    //there are enough bits in the tmp variable
    return;
    // return without reading new data
  } // feed in 0xFF if buffer is exhausted
  val = 0xffffffff as libc::c_uint;
  if (*melp).size > 4 as libc::c_int {
    // if there is more than 4 bytes the MEL segment
    val = read_le_uint32((*melp).data as *const libc::c_void); // read 32 bits from MEL data
                                                               // reduce counter
    (*melp).data = (*melp).data.offset(4 as libc::c_int as isize); // advance pointer
    (*melp).size -= 4 as libc::c_int
  } else if (*melp).size > 0 as libc::c_int {
    // 4 or less
    let mut m: OPJ_UINT32 = 0; // read one byte at a time
    let mut v: OPJ_UINT32 = 0; // mask of location
    let mut i = 0 as libc::c_int; // put byte in its correct location
    while (*melp).size > 1 as libc::c_int {
      let fresh0 = (*melp).data;
      (*melp).data = (*melp).data.offset(1);
      let mut v_0 = *fresh0 as OPJ_UINT32;
      let mut m_0 = !((0xff as libc::c_uint) << i);
      val = val & m_0 | v_0 << i;
      (*melp).size -= 1;
      i += 8 as libc::c_int
    }
    // size equal to 1
    let fresh1 = (*melp).data; // the one before the last is different
    (*melp).data = (*melp).data.offset(1); // MEL and VLC segments can overlap
    v = *fresh1 as OPJ_UINT32;
    v |= 0xf as libc::c_int as libc::c_uint;
    m = !((0xff as libc::c_uint) << i);
    val = val & m | v << i;
    (*melp).size -= 1
  }
  // next we unstuff them before adding them to the buffer
  bits = 32 as libc::c_int - (*melp).unstuff; // number of bits in val, subtract 1 if
                                              // the previously read byte requires
                                              // unstuffing
                                              // data is unstuffed and accumulated in t
                                              // bits has the number of bits in t
  t = val & 0xff as libc::c_int as libc::c_uint; // true if the byte needs unstuffing
  unstuff = (val & 0xff as libc::c_int as libc::c_uint == 0xff as libc::c_int as libc::c_uint)
    as libc::c_int; // there is one less bit in t if unstuffing is needed
  bits -= unstuff; // move up to make room for the next byte
  t = t << 8 as libc::c_int - unstuff;
  //this is a repeat of the above
  t |= val >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint;
  unstuff = (val >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint
    == 0xff as libc::c_int as libc::c_uint) as libc::c_int;
  bits -= unstuff;
  t = t << 8 as libc::c_int - unstuff;
  t |= val >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint;
  unstuff = (val >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint
    == 0xff as libc::c_int as libc::c_uint) as libc::c_int;
  bits -= unstuff;
  t = t << 8 as libc::c_int - unstuff;
  t |= val >> 24 as libc::c_int & 0xff as libc::c_int as libc::c_uint;
  (*melp).unstuff = (val >> 24 as libc::c_int & 0xff as libc::c_int as libc::c_uint
    == 0xff as libc::c_int as libc::c_uint) as libc::c_int;
  // move t to tmp, and push the result all the way up, so we read from
  // the MSB
  (*melp).tmp |= (t as OPJ_UINT64) << 64 as libc::c_int - bits - (*melp).bits;
  (*melp).bits += bits;
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
#[inline]
unsafe fn mel_decode(mut melp: *mut dec_mel_t) {
  static mut mel_exp: [libc::c_int; 13] = [
    0 as libc::c_int,
    0 as libc::c_int,
    0 as libc::c_int,
    1 as libc::c_int,
    1 as libc::c_int,
    1 as libc::c_int,
    2 as libc::c_int,
    2 as libc::c_int,
    2 as libc::c_int,
    3 as libc::c_int,
    3 as libc::c_int,
    4 as libc::c_int,
    5 as libc::c_int,
  ];
  if (*melp).bits < 6 as libc::c_int {
    // if there are less than 6 bits in tmp
    mel_read(melp);
    // then read from the MEL bitstream
  }
  // 6 bits is the largest decodable MEL cwd
  //repeat so long that there is enough decodable bits in tmp,
  // and the runs store is not full (num_runs < 8)
  while (*melp).bits >= 6 as libc::c_int && (*melp).num_runs < 8 as libc::c_int {
    let mut eval = mel_exp[(*melp).k as usize]; // number of bits associated with state
    let mut run = 0 as libc::c_int;
    if (*melp).tmp as libc::c_ulonglong & (1 as libc::c_ulonglong) << 63 as libc::c_int != 0 {
      //The next bit to decode (stored in MSB)
      //one is found
      run = (1 as libc::c_int) << eval;
      // a stretch of zeros not terminating in one
      run -= 1; // consecutive runs of 0 events - 1
      (*melp).k = if ((*melp).k + 1 as libc::c_int) < 12 as libc::c_int {
        ((*melp).k) + 1 as libc::c_int
      } else {
        12 as libc::c_int
      }; //increment, max is 12
      (*melp).tmp <<= 1 as libc::c_int; // consume one bit from tmp
      (*melp).bits -= 1 as libc::c_int;
      run = run << 1 as libc::c_int
    } else {
      //0 is found
      run = ((*melp).tmp >> 63 as libc::c_int - eval) as libc::c_int
        & ((1 as libc::c_int) << eval) - 1 as libc::c_int;
      // a stretch of zeros terminating with one
      (*melp).k = if (*melp).k - 1 as libc::c_int > 0 as libc::c_int {
        ((*melp).k) - 1 as libc::c_int
      } else {
        0 as libc::c_int
      }; //decrement, min is 0
      (*melp).tmp <<= eval + 1 as libc::c_int; //consume eval + 1 bits (max is 6)
      (*melp).bits -= eval + 1 as libc::c_int; // 7 bits per run
      run = (run << 1 as libc::c_int) + 1 as libc::c_int
    } // 6 bits are sufficient
    eval = (*melp).num_runs * 7 as libc::c_int; // store the value in runs
    (*melp).runs &= !((0x3f as libc::c_int as OPJ_UINT64) << eval);
    (*melp).runs |= (run as OPJ_UINT64) << eval;
    (*melp).num_runs += 1
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
#[inline]
unsafe fn mel_init(
  mut melp: *mut dec_mel_t,
  mut bbuf: *mut OPJ_UINT8,
  mut lcup: libc::c_int,
  mut scup: libc::c_int,
) {
  let mut num: libc::c_int = 0; // move the pointer to the start of MEL
  let mut i: libc::c_int = 0; // 0 bits in tmp
  (*melp).data = bbuf.offset(lcup as isize).offset(-(scup as isize)); //
  (*melp).bits = 0 as libc::c_int; // no unstuffing
  (*melp).tmp = 0 as libc::c_int as OPJ_UINT64; // size is the length of MEL+VLC-1
  (*melp).unstuff = 0 as libc::c_int; // 0 for state
  (*melp).size = scup - 1 as libc::c_int; // num_runs is 0
  (*melp).k = 0 as libc::c_int; //
  (*melp).num_runs = 0 as libc::c_int;
  (*melp).runs = 0 as libc::c_int as OPJ_UINT64;
  //This code is borrowed; original is for a different architecture
  //These few lines take care of the case where data is not at a multiple
  // of 4 boundary.  It reads 1,2,3 up to 4 bytes from the MEL segment
  num = 4 as libc::c_int
    - ((*melp).data as intptr_t & 0x3 as libc::c_int as libc::c_long) as libc::c_int;
  i = 0 as libc::c_int;
  while i < num {
    // this code is similar to mel_read
    let mut d: OPJ_UINT64 = 0; // if buffer is consumed
    let mut d_bits: libc::c_int = 0;
    assert!(
      (*melp).unstuff == 0 as libc::c_int
        || *(*melp).data.offset(0 as libc::c_int as isize) as libc::c_int <= 0x8f as libc::c_int
    );
    d = if (*melp).size > 0 as libc::c_int {
      *(*melp).data as libc::c_int
    } else {
      0xff as libc::c_int
    } as OPJ_UINT64;
    // set data to 0xFF
    if (*melp).size == 1 as libc::c_int {
      d |= 0xf as libc::c_int as libc::c_ulong
      //if this is MEL+VLC-1, set LSBs to 0xF
    }
    // see the standard
    let fresh2 = (*melp).size; //increment if the end is not reached
    (*melp).size = (*melp).size - 1; //if unstuffing is needed, reduce by 1
    (*melp).data = (*melp)
      .data
      .offset((fresh2 > 0 as libc::c_int) as libc::c_int as isize); //store bits in tmp
    d_bits = 8 as libc::c_int - (*melp).unstuff; //increment tmp by number of bits
    (*melp).tmp = (*melp).tmp << d_bits | d;
    (*melp).bits += d_bits;
    (*melp).unstuff = (d & 0xff as libc::c_int as libc::c_ulong
      == 0xff as libc::c_int as libc::c_ulong) as libc::c_int;
    i += 1
  }
  (*melp).tmp <<= 64 as libc::c_int - (*melp).bits;
  //push all the way up so the first bit
  // is the MSB
}
//* ***********************************************************************/
/* * @brief Retrieves one run from dec_mel_t; if there are no runs stored
 *         MEL segment is decoded
 *
 * @param [in]  melp is a pointer to dec_mel_t structure
 */
#[inline]
unsafe fn mel_get_run(mut melp: *mut dec_mel_t) -> libc::c_int {
  let mut t: libc::c_int = 0;
  if (*melp).num_runs == 0 as libc::c_int {
    //if no runs, decode more bit from MEL segment
    mel_decode(melp); //retrieve one run
  } // remove the retrieved run
  t = ((*melp).runs & 0x7f as libc::c_int as libc::c_ulong) as libc::c_int;
  (*melp).runs >>= 7 as libc::c_int;
  (*melp).num_runs -= 1;
  return t;
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
#[inline]
unsafe fn rev_read(mut vlcp: *mut rev_struct_t) {
  let mut val: OPJ_UINT32 = 0;
  let mut tmp: OPJ_UINT32 = 0;
  let mut bits: OPJ_UINT32 = 0;
  let mut unstuff: OPJ_BOOL = 0;
  //process 4 bytes at a time
  if (*vlcp).bits > 32 as libc::c_int as libc::c_uint {
    // if there are more than 32 bits in tmp, then
    return;
    // reading 32 bits can overflow vlcp->tmp
  }
  val = 0 as libc::c_int as OPJ_UINT32;
  //the next line (the if statement) needs to be tested first
  if (*vlcp).size > 3 as libc::c_int {
    // if there are more than 3 bytes left in VLC
    // (vlcp->data - 3) move pointer back to read 32 bits at once
    val = read_le_uint32((*vlcp).data.offset(-(3 as libc::c_int as isize)) as *const libc::c_void); // then read 32 bits
                                                                                                    // reduce available byte by 4
    (*vlcp).data = (*vlcp).data.offset(-(4 as libc::c_int as isize)); // move data pointer back by 4
    (*vlcp).size -= 4 as libc::c_int
  } else if (*vlcp).size > 0 as libc::c_int {
    // 4 or less
    let mut i = 24 as libc::c_int; // read one byte at a time
    while (*vlcp).size > 0 as libc::c_int {
      let fresh3 = (*vlcp).data; // put byte in its correct location
      (*vlcp).data = (*vlcp).data.offset(-1);
      let mut v = *fresh3 as OPJ_UINT32;
      val |= v << i;
      (*vlcp).size -= 1;
      i -= 8 as libc::c_int
    }
  }
  //accumulate in tmp, number of bits in tmp are stored in bits
  tmp = val >> 24 as libc::c_int; //start with the MSB byte
                                  // test unstuff (previous byte is >0x8F), and this byte is 0x7F
  bits = (8 as libc::c_uint).wrapping_sub(
    if (*vlcp).unstuff != 0
      && val >> 24 as libc::c_int & 0x7f as libc::c_int as libc::c_uint
        == 0x7f as libc::c_int as libc::c_uint
    {
      1 as libc::c_uint
    } else {
      0 as libc::c_uint
    },
  ); //this is for the next byte
  unstuff = (val >> 24 as libc::c_int > 0x8f as libc::c_int as libc::c_uint) as libc::c_int; //process the next byte
  tmp |= (val >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint) << bits;
  bits = (bits as libc::c_uint).wrapping_add((8 as libc::c_uint).wrapping_sub(
    if unstuff != 0
      && val >> 16 as libc::c_int & 0x7f as libc::c_int as libc::c_uint
        == 0x7f as libc::c_int as libc::c_uint
    {
      1 as libc::c_uint
    } else {
      0 as libc::c_uint
    },
  )) as OPJ_UINT32 as OPJ_UINT32;
  unstuff = (val >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint
    > 0x8f as libc::c_int as libc::c_uint) as libc::c_int;
  tmp |= (val >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint) << bits;
  bits = (bits as libc::c_uint).wrapping_add((8 as libc::c_uint).wrapping_sub(
    if unstuff != 0
      && val >> 8 as libc::c_int & 0x7f as libc::c_int as libc::c_uint
        == 0x7f as libc::c_int as libc::c_uint
    {
      1 as libc::c_uint
    } else {
      0 as libc::c_uint
    },
  )) as OPJ_UINT32 as OPJ_UINT32;
  unstuff = (val >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint
    > 0x8f as libc::c_int as libc::c_uint) as libc::c_int;
  tmp |= (val & 0xff as libc::c_int as libc::c_uint) << bits;
  bits = (bits as libc::c_uint).wrapping_add((8 as libc::c_uint).wrapping_sub(
    if unstuff != 0
      && val & 0x7f as libc::c_int as libc::c_uint == 0x7f as libc::c_int as libc::c_uint
    {
      1 as libc::c_uint
    } else {
      0 as libc::c_uint
    },
  )) as OPJ_UINT32 as OPJ_UINT32;
  unstuff = (val & 0xff as libc::c_int as libc::c_uint > 0x8f as libc::c_int as libc::c_uint)
    as libc::c_int;
  // now move the read and unstuffed bits into vlcp->tmp
  (*vlcp).tmp |= (tmp as OPJ_UINT64) << (*vlcp).bits;
  (*vlcp).bits = ((*vlcp).bits as libc::c_uint).wrapping_add(bits) as OPJ_UINT32 as OPJ_UINT32;
  (*vlcp).unstuff = unstuff;
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
#[inline]
unsafe fn rev_init(
  mut vlcp: *mut rev_struct_t,
  mut data: *mut OPJ_UINT8,
  mut lcup: libc::c_int,
  mut scup: libc::c_int,
) {
  let mut d: OPJ_UINT32 = 0;
  let mut num: libc::c_int = 0;
  let mut tnum: libc::c_int = 0;
  let mut i: libc::c_int = 0;
  //first byte has only the upper 4 bits
  (*vlcp).data = data
    .offset(lcup as isize)
    .offset(-(2 as libc::c_int as isize));
  //size can not be larger than this, in fact it should be smaller
  (*vlcp).size = scup - 2 as libc::c_int; // read one byte (this is a half byte)
  let fresh4 = (*vlcp).data; // both initialize and set
  (*vlcp).data = (*vlcp).data.offset(-1); //check standard
  d = *fresh4 as OPJ_UINT32; //this is useful for the next byte
  (*vlcp).tmp = (d >> 4 as libc::c_int) as OPJ_UINT64;
  (*vlcp).bits = (4 as libc::c_int
    - ((*vlcp).tmp & 7 as libc::c_int as libc::c_ulong == 7 as libc::c_int as libc::c_ulong)
      as libc::c_int) as OPJ_UINT32;
  (*vlcp).unstuff =
    (d | 0xf as libc::c_int as libc::c_uint > 0x8f as libc::c_int as libc::c_uint) as libc::c_int;
  //This code is designed for an architecture that read address should
  // align to the read size (address multiple of 4 if read size is 4)
  //These few lines take care of the case where data is not at a multiple
  // of 4 boundary. It reads 1,2,3 up to 4 bytes from the VLC bitstream.
  // To read 32 bits, read from (vlcp->data - 3)
  num = 1 as libc::c_int
    + ((*vlcp).data as intptr_t & 0x3 as libc::c_int as libc::c_long) as libc::c_int;
  tnum = if num < (*vlcp).size {
    num
  } else {
    (*vlcp).size
  };
  i = 0 as libc::c_int;
  while i < tnum {
    let mut d_0: OPJ_UINT64 = 0;
    let mut d_bits: OPJ_UINT32 = 0;
    // for next byte
    let fresh5 = (*vlcp).data; // read one byte and move read pointer
    (*vlcp).data = (*vlcp).data.offset(-1);
    d_0 = *fresh5 as OPJ_UINT64;
    d_bits = (8 as libc::c_uint).wrapping_sub(
      if (*vlcp).unstuff != 0
        && d_0 & 0x7f as libc::c_int as libc::c_ulong == 0x7f as libc::c_int as libc::c_ulong
      {
        1 as libc::c_uint
      } else {
        0 as libc::c_uint
      },
    );
    (*vlcp).tmp |= d_0 << (*vlcp).bits;
    (*vlcp).bits = ((*vlcp).bits as libc::c_uint).wrapping_add(d_bits) as OPJ_UINT32 as OPJ_UINT32;
    (*vlcp).unstuff = (d_0 > 0x8f as libc::c_int as libc::c_ulong) as libc::c_int;
    i += 1
  }
  (*vlcp).size -= tnum;
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
#[inline]
unsafe fn rev_fetch(mut vlcp: *mut rev_struct_t) -> OPJ_UINT32 {
  if (*vlcp).bits < 32 as libc::c_int as libc::c_uint {
    // if there are less then 32 bits, read more
    rev_read(vlcp); // read 32 bits, but unstuffing might reduce this
    if (*vlcp).bits < 32 as libc::c_int as libc::c_uint {
      // if there is still space in vlcp->tmp for 32 bits
      rev_read(vlcp);
      // read another 32
    }
  }
  return (*vlcp).tmp as OPJ_UINT32;
  // return the head (bottom-most) of vlcp->tmp
}
//* ***********************************************************************/
/* * @brief Consumes num_bits from a rev_struct structure
 *
 *  @param [in]  vlcp is a pointer to rev_struct structure
 *  @param [in]  num_bits is the number of bits to be removed
 */
#[inline]
unsafe fn rev_advance(
  mut vlcp: *mut rev_struct_t,
  mut num_bits: OPJ_UINT32,
) -> OPJ_UINT32 {
  assert!(num_bits <= (*vlcp).bits); // remove bits
  (*vlcp).tmp >>= num_bits; // decrement the number of bits
  (*vlcp).bits = ((*vlcp).bits as libc::c_uint).wrapping_sub(num_bits) as OPJ_UINT32 as OPJ_UINT32;
  return (*vlcp).tmp as OPJ_UINT32;
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
#[inline]
unsafe fn rev_read_mrp(mut mrp: *mut rev_struct_t) {
  let mut val: OPJ_UINT32 = 0;
  let mut tmp: OPJ_UINT32 = 0;
  let mut bits: OPJ_UINT32 = 0;
  let mut unstuff: OPJ_BOOL = 0;
  //process 4 bytes at a time
  if (*mrp).bits > 32 as libc::c_int as libc::c_uint {
    return;
  }
  val = 0 as libc::c_int as OPJ_UINT32;
  if (*mrp).size > 3 as libc::c_int {
    // If there are 3 byte or more
    // (mrp->data - 3) move pointer back to read 32 bits at once
    val = read_le_uint32((*mrp).data.offset(-(3 as libc::c_int as isize)) as *const libc::c_void); // read 32 bits
                                                                                                   // reduce count
    (*mrp).data = (*mrp).data.offset(-(4 as libc::c_int as isize)); // move back pointer
    (*mrp).size -= 4 as libc::c_int
  } else if (*mrp).size > 0 as libc::c_int {
    let mut i = 24 as libc::c_int; // read one byte at a time
    while (*mrp).size > 0 as libc::c_int {
      let fresh6 = (*mrp).data; // put byte in its correct location
      (*mrp).data = (*mrp).data.offset(-1);
      let mut v = *fresh6 as OPJ_UINT32;
      val |= v << i;
      (*mrp).size -= 1;
      i -= 8 as libc::c_int
    }
  }
  //accumulate in tmp, and keep count in bits
  tmp = val >> 24 as libc::c_int;
  //test if the last byte > 0x8F (unstuff must be true) and this is 0x7F
  bits = (8 as libc::c_uint).wrapping_sub(
    if (*mrp).unstuff != 0
      && val >> 24 as libc::c_int & 0x7f as libc::c_int as libc::c_uint
        == 0x7f as libc::c_int as libc::c_uint
    {
      1 as libc::c_uint
    } else {
      0 as libc::c_uint
    },
  );
  unstuff = (val >> 24 as libc::c_int > 0x8f as libc::c_int as libc::c_uint) as libc::c_int;
  //process the next byte
  tmp |= (val >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint) << bits; // move data to mrp pointer
  bits = (bits as libc::c_uint).wrapping_add((8 as libc::c_uint).wrapping_sub(
    if unstuff != 0
      && val >> 16 as libc::c_int & 0x7f as libc::c_int as libc::c_uint
        == 0x7f as libc::c_int as libc::c_uint
    {
      1 as libc::c_uint
    } else {
      0 as libc::c_uint
    },
  )) as OPJ_UINT32 as OPJ_UINT32;
  unstuff = (val >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint
    > 0x8f as libc::c_int as libc::c_uint) as libc::c_int;
  tmp |= (val >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint) << bits;
  bits = (bits as libc::c_uint).wrapping_add((8 as libc::c_uint).wrapping_sub(
    if unstuff != 0
      && val >> 8 as libc::c_int & 0x7f as libc::c_int as libc::c_uint
        == 0x7f as libc::c_int as libc::c_uint
    {
      1 as libc::c_uint
    } else {
      0 as libc::c_uint
    },
  )) as OPJ_UINT32 as OPJ_UINT32;
  unstuff = (val >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint
    > 0x8f as libc::c_int as libc::c_uint) as libc::c_int;
  tmp |= (val & 0xff as libc::c_int as libc::c_uint) << bits;
  bits = (bits as libc::c_uint).wrapping_add((8 as libc::c_uint).wrapping_sub(
    if unstuff != 0
      && val & 0x7f as libc::c_int as libc::c_uint == 0x7f as libc::c_int as libc::c_uint
    {
      1 as libc::c_uint
    } else {
      0 as libc::c_uint
    },
  )) as OPJ_UINT32 as OPJ_UINT32;
  unstuff = (val & 0xff as libc::c_int as libc::c_uint > 0x8f as libc::c_int as libc::c_uint)
    as libc::c_int;
  (*mrp).tmp |= (tmp as OPJ_UINT64) << (*mrp).bits;
  (*mrp).bits = ((*mrp).bits as libc::c_uint).wrapping_add(bits) as OPJ_UINT32 as OPJ_UINT32;
  (*mrp).unstuff = unstuff;
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
#[inline]
unsafe fn rev_init_mrp(
  mut mrp: *mut rev_struct_t,
  mut data: *mut OPJ_UINT8,
  mut lcup: libc::c_int,
  mut len2: libc::c_int,
) {
  let mut num: libc::c_int = 0;
  let mut i: libc::c_int = 0;
  (*mrp).data = data
    .offset(lcup as isize)
    .offset(len2 as isize)
    .offset(-(1 as libc::c_int as isize));
  (*mrp).size = len2;
  (*mrp).unstuff = 1 as libc::c_int;
  (*mrp).bits = 0 as libc::c_int as OPJ_UINT32;
  (*mrp).tmp = 0 as libc::c_int as OPJ_UINT64;
  //This code is designed for an architecture that read address should
  // align to the read size (address multiple of 4 if read size is 4)
  //These few lines take care of the case where data is not at a multiple
  // of 4 boundary.  It reads 1,2,3 up to 4 bytes from the MRP stream
  num = 1 as libc::c_int
    + ((*mrp).data as intptr_t & 0x3 as libc::c_int as libc::c_long) as libc::c_int;
  i = 0 as libc::c_int;
  while i < num {
    let mut d: OPJ_UINT64 = 0;
    let mut d_bits: OPJ_UINT32 = 0;
    // for next byte
    let fresh7 = (*mrp).size;
    (*mrp).size = (*mrp).size - 1;
    d = if fresh7 > 0 as libc::c_int {
      let fresh8 = (*mrp).data;
      (*mrp).data = (*mrp).data.offset(-1);
      *fresh8 as libc::c_int
    } else {
      0 as libc::c_int
    } as OPJ_UINT64;
    d_bits = (8 as libc::c_uint).wrapping_sub(
      if (*mrp).unstuff != 0
        && d & 0x7f as libc::c_int as libc::c_ulong == 0x7f as libc::c_int as libc::c_ulong
      {
        1 as libc::c_uint
      } else {
        0 as libc::c_uint
      },
    );
    (*mrp).tmp |= d << (*mrp).bits;
    (*mrp).bits = ((*mrp).bits as libc::c_uint).wrapping_add(d_bits) as OPJ_UINT32 as OPJ_UINT32;
    (*mrp).unstuff = (d > 0x8f as libc::c_int as libc::c_ulong) as libc::c_int;
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
#[inline]
unsafe fn rev_fetch_mrp(mut mrp: *mut rev_struct_t) -> OPJ_UINT32 {
  if (*mrp).bits < 32 as libc::c_int as libc::c_uint {
    // if there are less than 32 bits in mrp->tmp
    rev_read_mrp(mrp); // read 30-32 bits from mrp
    if (*mrp).bits < 32 as libc::c_int as libc::c_uint {
      // if there is a space of 32 bits
      rev_read_mrp(mrp);
      // read more
    }
  }
  return (*mrp).tmp as OPJ_UINT32;
  // return the head of mrp->tmp
}
//* ***********************************************************************/
/* * @brief Consumes num_bits from a rev_struct structure
 *
 *  @param [in]  mrp is a pointer to rev_struct structure
 *  @param [in]  num_bits is the number of bits to be removed
 */
#[inline]
unsafe fn rev_advance_mrp(
  mut mrp: *mut rev_struct_t,
  mut num_bits: OPJ_UINT32,
) -> OPJ_UINT32 {
  assert!(num_bits <= (*mrp).bits); // discard the lowest num_bits bits
  (*mrp).tmp >>= num_bits;
  (*mrp).bits = ((*mrp).bits as libc::c_uint).wrapping_sub(num_bits) as OPJ_UINT32 as OPJ_UINT32;
  return (*mrp).tmp as OPJ_UINT32;
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
#[inline]
unsafe fn decode_init_uvlc(
  mut vlc: OPJ_UINT32,
  mut mode: OPJ_UINT32,
  mut u: *mut OPJ_UINT32,
) -> OPJ_UINT32 {
  //table stores possible decoding three bits from vlc
  // there are 8 entries for xx1, x10, 100, 000, where x means do not care
  // table value is made up of
  // 2 bits in the LSB for prefix length
  // 3 bits for suffix length
  // 3 bits in the MSB for prefix value (u_pfx in Table 3 of ITU T.814)
  static mut dec: [OPJ_UINT8; 8] = [
    (3 as libc::c_int
      | (5 as libc::c_int) << 2 as libc::c_int
      | (5 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (1 as libc::c_int
      | (0 as libc::c_int) << 2 as libc::c_int
      | (1 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (2 as libc::c_int
      | (0 as libc::c_int) << 2 as libc::c_int
      | (2 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (1 as libc::c_int
      | (0 as libc::c_int) << 2 as libc::c_int
      | (1 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (3 as libc::c_int
      | (1 as libc::c_int) << 2 as libc::c_int
      | (3 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (1 as libc::c_int
      | (0 as libc::c_int) << 2 as libc::c_int
      | (1 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (2 as libc::c_int
      | (0 as libc::c_int) << 2 as libc::c_int
      | (2 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (1 as libc::c_int
      | (0 as libc::c_int) << 2 as libc::c_int
      | (1 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
  ];
  let mut consumed_bits = 0 as libc::c_int as OPJ_UINT32;
  if mode == 0 as libc::c_int as libc::c_uint {
    // both u_off are 0
    let ref mut fresh9 = *u.offset(1 as libc::c_int as isize);
    *fresh9 = 1 as libc::c_int as OPJ_UINT32;
    *u.offset(0 as libc::c_int as isize) = *fresh9
  //Kappa is 1 for initial line
  } else if mode <= 2 as libc::c_int as libc::c_uint {
    // u_off are either 01 or 10
    let mut d: OPJ_UINT32 = 0; //look at the least significant 3 bits
    let mut suffix_len: OPJ_UINT32 = 0; //prefix length
    d = dec[(vlc & 0x7 as libc::c_int as libc::c_uint) as usize] as OPJ_UINT32; // u value
    vlc >>= d & 0x3 as libc::c_int as libc::c_uint; // kappa is 1 for initial line
    consumed_bits = (consumed_bits as libc::c_uint)
      .wrapping_add(d & 0x3 as libc::c_int as libc::c_uint) as OPJ_UINT32
      as OPJ_UINT32;
    suffix_len = d >> 2 as libc::c_int & 0x7 as libc::c_int as libc::c_uint;
    consumed_bits =
      (consumed_bits as libc::c_uint).wrapping_add(suffix_len) as OPJ_UINT32 as OPJ_UINT32;
    d = (d >> 5 as libc::c_int).wrapping_add(
      vlc & ((1 as libc::c_uint) << suffix_len).wrapping_sub(1 as libc::c_int as libc::c_uint),
    );
    *u.offset(0 as libc::c_int as isize) = if mode == 1 as libc::c_int as libc::c_uint {
      d.wrapping_add(1 as libc::c_int as libc::c_uint)
    } else {
      1 as libc::c_int as libc::c_uint
    };
    *u.offset(1 as libc::c_int as isize) = if mode == 1 as libc::c_int as libc::c_uint {
      1 as libc::c_int as libc::c_uint
    } else {
      d.wrapping_add(1 as libc::c_int as libc::c_uint)
    }
  } else if mode == 3 as libc::c_int as libc::c_uint {
    // both u_off are 1, and MEL event is 0
    let mut d1 = dec[(vlc & 0x7 as libc::c_int as libc::c_uint) as usize] as OPJ_UINT32; // LSBs of VLC are prefix codeword
    vlc >>= d1 & 0x3 as libc::c_int as libc::c_uint; // Consume bits
    consumed_bits = (consumed_bits as libc::c_uint)
      .wrapping_add(d1 & 0x3 as libc::c_int as libc::c_uint) as OPJ_UINT32
      as OPJ_UINT32;
    if d1 & 0x3 as libc::c_int as libc::c_uint > 2 as libc::c_int as libc::c_uint {
      let mut suffix_len_0: OPJ_UINT32 = 0;
      //Kappa is 1 for initial line
      *u.offset(1 as libc::c_int as isize) = (vlc & 1 as libc::c_int as libc::c_uint)
        .wrapping_add(1 as libc::c_int as libc::c_uint)
        .wrapping_add(1 as libc::c_int as libc::c_uint);
      consumed_bits = consumed_bits.wrapping_add(1);
      vlc >>= 1 as libc::c_int;
      suffix_len_0 = d1 >> 2 as libc::c_int & 0x7 as libc::c_int as libc::c_uint;
      consumed_bits =
        (consumed_bits as libc::c_uint).wrapping_add(suffix_len_0) as OPJ_UINT32 as OPJ_UINT32;
      d1 = (d1 >> 5 as libc::c_int).wrapping_add(
        vlc & ((1 as libc::c_uint) << suffix_len_0).wrapping_sub(1 as libc::c_int as libc::c_uint),
      );
      *u.offset(0 as libc::c_int as isize) = d1.wrapping_add(1 as libc::c_int as libc::c_uint)
    } else {
      let mut d2: OPJ_UINT32 = 0;
      let mut suffix_len_1: OPJ_UINT32 = 0;
      //u_{q_2} prefix
      //Kappa is 1 for initial line
      // u value
      //Kappa is 1 for initial line
      d2 = dec[(vlc & 0x7 as libc::c_int as libc::c_uint) as usize] as OPJ_UINT32; // LSBs of VLC are prefix codeword
      vlc >>= d2 & 0x3 as libc::c_int as libc::c_uint; // Consume bits
      consumed_bits = (consumed_bits as libc::c_uint)
        .wrapping_add(d2 & 0x3 as libc::c_int as libc::c_uint) as OPJ_UINT32
        as OPJ_UINT32; // u value
      suffix_len_1 = d1 >> 2 as libc::c_int & 0x7 as libc::c_int as libc::c_uint; //Kappa is 1 for initial line
      consumed_bits =
        (consumed_bits as libc::c_uint).wrapping_add(suffix_len_1) as OPJ_UINT32 as OPJ_UINT32; // u value
      d1 = (d1 >> 5 as libc::c_int).wrapping_add(
        vlc & ((1 as libc::c_uint) << suffix_len_1).wrapping_sub(1 as libc::c_int as libc::c_uint),
      );
      *u.offset(0 as libc::c_int as isize) = d1.wrapping_add(1 as libc::c_int as libc::c_uint);
      vlc >>= suffix_len_1;
      suffix_len_1 = d2 >> 2 as libc::c_int & 0x7 as libc::c_int as libc::c_uint;
      consumed_bits =
        (consumed_bits as libc::c_uint).wrapping_add(suffix_len_1) as OPJ_UINT32 as OPJ_UINT32;
      d2 = (d2 >> 5 as libc::c_int).wrapping_add(
        vlc & ((1 as libc::c_uint) << suffix_len_1).wrapping_sub(1 as libc::c_int as libc::c_uint),
      );
      *u.offset(1 as libc::c_int as isize) = d2.wrapping_add(1 as libc::c_int as libc::c_uint)
    }
  } else if mode == 4 as libc::c_int as libc::c_uint {
    // both u_off are 1, and MEL event is 1
    let mut d1_0: OPJ_UINT32 = 0; // LSBs of VLC are prefix codeword
    let mut d2_0: OPJ_UINT32 = 0; // Consume bits
    let mut suffix_len_2: OPJ_UINT32 = 0; // LSBs of VLC are prefix codeword
    d1_0 = dec[(vlc & 0x7 as libc::c_int as libc::c_uint) as usize] as OPJ_UINT32; // Consume bits
    vlc >>= d1_0 & 0x3 as libc::c_int as libc::c_uint; // u value
    consumed_bits = (consumed_bits as libc::c_uint)
      .wrapping_add(d1_0 & 0x3 as libc::c_int as libc::c_uint) as OPJ_UINT32
      as OPJ_UINT32; // add 2+kappa
    d2_0 = dec[(vlc & 0x7 as libc::c_int as libc::c_uint) as usize] as OPJ_UINT32; // u value
    vlc >>= d2_0 & 0x3 as libc::c_int as libc::c_uint;
    consumed_bits = (consumed_bits as libc::c_uint)
      .wrapping_add(d2_0 & 0x3 as libc::c_int as libc::c_uint) as OPJ_UINT32
      as OPJ_UINT32;
    suffix_len_2 = d1_0 >> 2 as libc::c_int & 0x7 as libc::c_int as libc::c_uint;
    consumed_bits =
      (consumed_bits as libc::c_uint).wrapping_add(suffix_len_2) as OPJ_UINT32 as OPJ_UINT32;
    d1_0 = (d1_0 >> 5 as libc::c_int).wrapping_add(
      vlc & ((1 as libc::c_uint) << suffix_len_2).wrapping_sub(1 as libc::c_int as libc::c_uint),
    );
    *u.offset(0 as libc::c_int as isize) = d1_0.wrapping_add(3 as libc::c_int as libc::c_uint);
    vlc >>= suffix_len_2;
    suffix_len_2 = d2_0 >> 2 as libc::c_int & 0x7 as libc::c_int as libc::c_uint;
    consumed_bits =
      (consumed_bits as libc::c_uint).wrapping_add(suffix_len_2) as OPJ_UINT32 as OPJ_UINT32;
    d2_0 = (d2_0 >> 5 as libc::c_int).wrapping_add(
      vlc & ((1 as libc::c_uint) << suffix_len_2).wrapping_sub(1 as libc::c_int as libc::c_uint),
    );
    *u.offset(1 as libc::c_int as isize) = d2_0.wrapping_add(3 as libc::c_int as libc::c_uint)
  }
  return consumed_bits;
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
#[inline]
unsafe fn decode_noninit_uvlc(
  mut vlc: OPJ_UINT32,
  mut mode: OPJ_UINT32,
  mut u: *mut OPJ_UINT32,
) -> OPJ_UINT32 {
  //table stores possible decoding three bits from vlc
  // there are 8 entries for xx1, x10, 100, 000, where x means do not care
  // table value is made up of
  // 2 bits in the LSB for prefix length
  // 3 bits for suffix length
  // 3 bits in the MSB for prefix value (u_pfx in Table 3 of ITU T.814)
  static mut dec: [OPJ_UINT8; 8] = [
    (3 as libc::c_int
      | (5 as libc::c_int) << 2 as libc::c_int
      | (5 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (1 as libc::c_int
      | (0 as libc::c_int) << 2 as libc::c_int
      | (1 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (2 as libc::c_int
      | (0 as libc::c_int) << 2 as libc::c_int
      | (2 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (1 as libc::c_int
      | (0 as libc::c_int) << 2 as libc::c_int
      | (1 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (3 as libc::c_int
      | (1 as libc::c_int) << 2 as libc::c_int
      | (3 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (1 as libc::c_int
      | (0 as libc::c_int) << 2 as libc::c_int
      | (1 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (2 as libc::c_int
      | (0 as libc::c_int) << 2 as libc::c_int
      | (2 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
    (1 as libc::c_int
      | (0 as libc::c_int) << 2 as libc::c_int
      | (1 as libc::c_int) << 5 as libc::c_int) as OPJ_UINT8,
  ];
  let mut consumed_bits = 0 as libc::c_int as OPJ_UINT32;
  if mode == 0 as libc::c_int as libc::c_uint {
    let ref mut fresh10 = *u.offset(1 as libc::c_int as isize);
    *fresh10 = 1 as libc::c_int as OPJ_UINT32;
    *u.offset(0 as libc::c_int as isize) = *fresh10
  //for kappa
  } else if mode <= 2 as libc::c_int as libc::c_uint {
    //u_off are either 01 or 10
    let mut d: OPJ_UINT32 = 0; //look at the least significant 3 bits
    let mut suffix_len: OPJ_UINT32 = 0; //prefix length
    d = dec[(vlc & 0x7 as libc::c_int as libc::c_uint) as usize] as OPJ_UINT32; // u value
    vlc >>= d & 0x3 as libc::c_int as libc::c_uint; //for kappa
    consumed_bits = (consumed_bits as libc::c_uint)
      .wrapping_add(d & 0x3 as libc::c_int as libc::c_uint) as OPJ_UINT32
      as OPJ_UINT32;
    suffix_len = d >> 2 as libc::c_int & 0x7 as libc::c_int as libc::c_uint;
    consumed_bits =
      (consumed_bits as libc::c_uint).wrapping_add(suffix_len) as OPJ_UINT32 as OPJ_UINT32;
    d = (d >> 5 as libc::c_int).wrapping_add(
      vlc & ((1 as libc::c_uint) << suffix_len).wrapping_sub(1 as libc::c_int as libc::c_uint),
    );
    *u.offset(0 as libc::c_int as isize) = if mode == 1 as libc::c_int as libc::c_uint {
      d.wrapping_add(1 as libc::c_int as libc::c_uint)
    } else {
      1 as libc::c_int as libc::c_uint
    };
    *u.offset(1 as libc::c_int as isize) = if mode == 1 as libc::c_int as libc::c_uint {
      1 as libc::c_int as libc::c_uint
    } else {
      d.wrapping_add(1 as libc::c_int as libc::c_uint)
    }
  } else if mode == 3 as libc::c_int as libc::c_uint {
    // both u_off are 1
    let mut d1: OPJ_UINT32 = 0; // LSBs of VLC are prefix codeword
    let mut d2: OPJ_UINT32 = 0; // Consume bits
    let mut suffix_len_0: OPJ_UINT32 = 0; // LSBs of VLC are prefix codeword
    d1 = dec[(vlc & 0x7 as libc::c_int as libc::c_uint) as usize] as OPJ_UINT32; // Consume bits
    vlc >>= d1 & 0x3 as libc::c_int as libc::c_uint; // u value
    consumed_bits = (consumed_bits as libc::c_uint)
      .wrapping_add(d1 & 0x3 as libc::c_int as libc::c_uint) as OPJ_UINT32
      as OPJ_UINT32; //1 for kappa
    d2 = dec[(vlc & 0x7 as libc::c_int as libc::c_uint) as usize] as OPJ_UINT32; // u value
    vlc >>= d2 & 0x3 as libc::c_int as libc::c_uint;
    consumed_bits = (consumed_bits as libc::c_uint)
      .wrapping_add(d2 & 0x3 as libc::c_int as libc::c_uint) as OPJ_UINT32
      as OPJ_UINT32;
    suffix_len_0 = d1 >> 2 as libc::c_int & 0x7 as libc::c_int as libc::c_uint;
    consumed_bits =
      (consumed_bits as libc::c_uint).wrapping_add(suffix_len_0) as OPJ_UINT32 as OPJ_UINT32;
    d1 = (d1 >> 5 as libc::c_int).wrapping_add(
      vlc & ((1 as libc::c_uint) << suffix_len_0).wrapping_sub(1 as libc::c_int as libc::c_uint),
    );
    *u.offset(0 as libc::c_int as isize) = d1.wrapping_add(1 as libc::c_int as libc::c_uint);
    vlc >>= suffix_len_0;
    suffix_len_0 = d2 >> 2 as libc::c_int & 0x7 as libc::c_int as libc::c_uint;
    consumed_bits =
      (consumed_bits as libc::c_uint).wrapping_add(suffix_len_0) as OPJ_UINT32 as OPJ_UINT32;
    d2 = (d2 >> 5 as libc::c_int).wrapping_add(
      vlc & ((1 as libc::c_uint) << suffix_len_0).wrapping_sub(1 as libc::c_int as libc::c_uint),
    );
    *u.offset(1 as libc::c_int as isize) = d2.wrapping_add(1 as libc::c_int as libc::c_uint)
  }
  return consumed_bits;
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
#[inline]
unsafe fn frwd_read(mut msp: *mut frwd_struct_t) {
  let mut val: OPJ_UINT32 = 0; // assert that there is a space for 32 bits
  let mut bits: OPJ_UINT32 = 0; // read 32 bits
  let mut t: OPJ_UINT32 = 0;
  let mut unstuff: OPJ_BOOL = 0;
  assert!((*msp).bits <= 32 as libc::c_int as libc::c_uint);
  val = 0 as libc::c_uint;
  if (*msp).size > 3 as libc::c_int {
    val = read_le_uint32((*msp).data as *const libc::c_void);
    // reduce size
    (*msp).data = (*msp).data.offset(4 as libc::c_int as isize); // increment pointer
    (*msp).size -= 4 as libc::c_int
  } else if (*msp).size > 0 as libc::c_int {
    let mut i = 0 as libc::c_int; // read one byte at a time
    val = if (*msp).X != 0 as libc::c_int as libc::c_uint {
      0xffffffff as libc::c_uint
    } else {
      0 as libc::c_int as libc::c_uint
    }; // mask of location
    while (*msp).size > 0 as libc::c_int {
      let fresh11 = (*msp).data; // put one byte in its correct location
      (*msp).data = (*msp).data.offset(1);
      let mut v = *fresh11 as OPJ_UINT32;
      let mut m = !((0xff as libc::c_uint) << i);
      val = val & m | v << i;
      (*msp).size -= 1;
      i += 8 as libc::c_int
    }
  } else {
    val = if (*msp).X != 0 as libc::c_int as libc::c_uint {
      0xffffffff as libc::c_uint
    } else {
      0 as libc::c_int as libc::c_uint
    }
  }
  // we accumulate in t and keep a count of the number of bits in bits
  bits = (8 as libc::c_uint).wrapping_sub(if (*msp).unstuff != 0 {
    1 as libc::c_uint
  } else {
    0 as libc::c_uint
  }); // Do we need unstuffing next?
  t = val & 0xff as libc::c_int as libc::c_uint; // for next byte
  unstuff = (val & 0xff as libc::c_int as libc::c_uint == 0xff as libc::c_int as libc::c_uint)
    as libc::c_int; // move data to msp->tmp
  t |= (val >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint) << bits;
  bits = (bits as libc::c_uint).wrapping_add((8 as libc::c_uint).wrapping_sub(if unstuff != 0 {
    1 as libc::c_uint
  } else {
    0 as libc::c_uint
  })) as OPJ_UINT32 as OPJ_UINT32;
  unstuff = (val >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint
    == 0xff as libc::c_int as libc::c_uint) as libc::c_int;
  t |= (val >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint) << bits;
  bits = (bits as libc::c_uint).wrapping_add((8 as libc::c_uint).wrapping_sub(if unstuff != 0 {
    1 as libc::c_uint
  } else {
    0 as libc::c_uint
  })) as OPJ_UINT32 as OPJ_UINT32;
  unstuff = (val >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint
    == 0xff as libc::c_int as libc::c_uint) as libc::c_int;
  t |= (val >> 24 as libc::c_int & 0xff as libc::c_int as libc::c_uint) << bits;
  bits = (bits as libc::c_uint).wrapping_add((8 as libc::c_uint).wrapping_sub(if unstuff != 0 {
    1 as libc::c_uint
  } else {
    0 as libc::c_uint
  })) as OPJ_UINT32 as OPJ_UINT32;
  (*msp).unstuff = (val >> 24 as libc::c_int & 0xff as libc::c_int as libc::c_uint
    == 0xff as libc::c_int as libc::c_uint) as libc::c_int;
  (*msp).tmp |= (t as OPJ_UINT64) << (*msp).bits;
  (*msp).bits = ((*msp).bits as libc::c_uint).wrapping_add(bits) as OPJ_UINT32 as OPJ_UINT32;
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
#[inline]
unsafe fn frwd_init(
  mut msp: *mut frwd_struct_t,
  mut data: *const OPJ_UINT8,
  mut size: libc::c_int,
  mut X: OPJ_UINT32,
) {
  let mut num: libc::c_int = 0;
  let mut i: libc::c_int = 0;
  (*msp).data = data;
  (*msp).tmp = 0 as libc::c_int as OPJ_UINT64;
  (*msp).bits = 0 as libc::c_int as OPJ_UINT32;
  (*msp).unstuff = 0 as libc::c_int;
  (*msp).size = size;
  (*msp).X = X;
  assert!(
    (*msp).X == 0 as libc::c_int as libc::c_uint || (*msp).X == 0xff as libc::c_int as libc::c_uint
  );
  //This code is designed for an architecture that read address should
  // align to the read size (address multiple of 4 if read size is 4)
  //These few lines take care of the case where data is not at a multiple
  // of 4 boundary.  It reads 1,2,3 up to 4 bytes from the bitstream
  num = 4 as libc::c_int
    - ((*msp).data as intptr_t & 0x3 as libc::c_int as libc::c_long) as libc::c_int;
  i = 0 as libc::c_int;
  while i < num {
    let mut d: OPJ_UINT64 = 0;
    // unstuffing for next byte
    let fresh12 = (*msp).size;
    (*msp).size = (*msp).size - 1;
    d = if fresh12 > 0 as libc::c_int {
      let fresh13 = (*msp).data;
      (*msp).data = (*msp).data.offset(1);
      *fresh13 as libc::c_uint
    } else {
      (*msp).X
    } as OPJ_UINT64;
    (*msp).tmp |= d << (*msp).bits;
    (*msp).bits = ((*msp).bits as libc::c_uint).wrapping_add((8 as libc::c_uint).wrapping_sub(
      if (*msp).unstuff != 0 {
        1 as libc::c_uint
      } else {
        0 as libc::c_uint
      },
    )) as OPJ_UINT32 as OPJ_UINT32;
    (*msp).unstuff = (d & 0xff as libc::c_int as libc::c_ulong
      == 0xff as libc::c_int as libc::c_ulong) as libc::c_int;
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
#[inline]
unsafe fn frwd_advance(mut msp: *mut frwd_struct_t, mut num_bits: OPJ_UINT32) {
  assert!(num_bits <= (*msp).bits);
  (*msp).tmp >>= num_bits;
  (*msp).bits = ((*msp).bits as libc::c_uint).wrapping_sub(num_bits) as OPJ_UINT32 as OPJ_UINT32;
}
//* ***********************************************************************/
/* * @brief Fetches 32 bits from the frwd_struct_t bitstream
 *
 *  @param [in]  msp is a pointer to frwd_struct_t
 */
#[inline]
unsafe fn frwd_fetch(mut msp: *mut frwd_struct_t) -> OPJ_UINT32 {
  if (*msp).bits < 32 as libc::c_int as libc::c_uint {
    frwd_read(msp);
    if (*msp).bits < 32 as libc::c_int as libc::c_uint {
      //need to test
      frwd_read(msp);
    }
  }
  return (*msp).tmp as OPJ_UINT32;
}
//* ***********************************************************************/
/* * @brief Allocates T1 buffers
 *
 *  @param [in, out]  t1 is codeblock cofficients storage
 *  @param [in]       w is codeblock width
 *  @param [in]       h is codeblock height
 */
unsafe fn opj_t1_allocate_buffers(
  mut t1: &mut opj_t1_t,
  mut w: OPJ_UINT32,
  mut h: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut flagssize: OPJ_UINT32 = 0;
  /* No risk of overflow. Prior checks ensure those assert are met */
  /* They are per the specification */

  assert!(w <= 1024 as libc::c_int as libc::c_uint);
  assert!(h <= 1024 as libc::c_int as libc::c_uint);
  assert!(w.wrapping_mul(h) <= 4096 as libc::c_int as libc::c_uint);
  /* encoder uses tile buffer, so no need to allocate */
  let mut datasize = w.wrapping_mul(h);
  if datasize > t1.datasize {
    opj_aligned_free(t1.data as *mut libc::c_void);
    t1.data = opj_aligned_malloc(
      (datasize as libc::c_ulong).wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
    ) as *mut OPJ_INT32;
    if t1.data.is_null() {
      /* FIXME event manager error callback */
      return 0 as libc::c_int;
    }
    t1.datasize = datasize
  }
  /* memset first arg is declared to never be null by gcc */
  if !t1.data.is_null() {
    memset(
      t1.data as *mut libc::c_void,
      0 as libc::c_int,
      (datasize as libc::c_ulong).wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
    );
  }
  // We expand these buffers to multiples of 16 bytes.
  // We need 4 buffers of 129 integers each, expanded to 132 integers each
  // We also need 514 bytes of buffer, expanded to 528 bytes
  flagssize = (132 as libc::c_uint as libc::c_ulong)
    .wrapping_mul(::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong)
    .wrapping_mul(4 as libc::c_uint as libc::c_ulong) as OPJ_UINT32; // expanded to multiple of 16
  flagssize =
    (flagssize as libc::c_uint).wrapping_add(528 as libc::c_uint) as OPJ_UINT32 as OPJ_UINT32; // 514 expanded to multiples of 16
  if flagssize > t1.flagssize {
    opj_aligned_free(t1.flags as *mut libc::c_void);
    t1.flags = opj_aligned_malloc(flagssize as size_t) as *mut opj_flag_t;
    if t1.flags.is_null() {
      /* FIXME event manager error callback */
      return 0 as libc::c_int;
    }
  }
  t1.flagssize = flagssize;
  memset(
    t1.flags as *mut libc::c_void,
    0 as libc::c_int,
    flagssize as libc::c_ulong,
  );
  t1.w = w;
  t1.h = h;
  return 1 as libc::c_int;
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
 *  @param [in]       p_manager_mutex a mutex to control access to p_manager
 *  @param [in]       check_pterm: check termination (not used)
 */
pub(crate) unsafe fn opj_t1_ht_decode_cblk(
  mut t1: &mut opj_t1_t,
  mut cblk: *mut opj_tcd_cblk_dec_t,
  mut _orient: OPJ_UINT32,
  mut roishift: OPJ_UINT32,
  mut cblksty: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
  mut p_manager_mutex: *mut opj_mutex_t,
  mut _check_pterm: OPJ_BOOL,
) -> OPJ_BOOL {
  let mut cblkdata = 0 as *mut OPJ_BYTE; // fetched data from VLC bitstream
  let mut coded_data = 0 as *mut OPJ_UINT8; // loop indices
  let mut decoded_data = 0 as *mut OPJ_UINT32;
  let mut zero_bplanes: OPJ_UINT32 = 0;
  let mut num_passes: OPJ_UINT32 = 0;
  let mut lengths1: OPJ_UINT32 = 0;
  let mut lengths2: OPJ_UINT32 = 0;
  let mut width: OPJ_INT32 = 0;
  let mut height: OPJ_INT32 = 0;
  let mut stride: OPJ_INT32 = 0;
  let mut pflags = 0 as *mut OPJ_UINT32;
  let mut sigma1 = 0 as *mut OPJ_UINT32;
  let mut sigma2 = 0 as *mut OPJ_UINT32;
  let mut mbr1 = 0 as *mut OPJ_UINT32;
  let mut mbr2 = 0 as *mut OPJ_UINT32;
  let mut sip = 0 as *mut OPJ_UINT32;
  let mut sip_shift: OPJ_UINT32 = 0;
  let mut p: OPJ_UINT32 = 0;
  let mut zero_bplanes_p1: OPJ_UINT32 = 0;
  let mut lcup: libc::c_int = 0;
  let mut scup: libc::c_int = 0;
  let mut mel = dec_mel_t {
    data: 0 as *mut OPJ_UINT8,
    tmp: 0,
    bits: 0,
    size: 0,
    unstuff: 0,
    k: 0,
    num_runs: 0,
    runs: 0,
  };
  let mut vlc = rev_struct_t {
    data: 0 as *mut OPJ_UINT8,
    tmp: 0,
    bits: 0,
    size: 0,
    unstuff: 0,
  };
  let mut magsgn = frwd_struct_t {
    data: 0 as *const OPJ_UINT8,
    tmp: 0,
    bits: 0,
    unstuff: 0,
    size: 0,
    X: 0,
  };
  let mut sigprop = frwd_struct_t {
    data: 0 as *const OPJ_UINT8,
    tmp: 0,
    bits: 0,
    unstuff: 0,
    size: 0,
    X: 0,
  };
  let mut magref = rev_struct_t {
    data: 0 as *mut OPJ_UINT8,
    tmp: 0,
    bits: 0,
    size: 0,
    unstuff: 0,
  };
  let mut lsp = 0 as *mut OPJ_UINT8;
  let mut line_state = 0 as *mut OPJ_UINT8;
  let mut run: libc::c_int = 0;
  let mut vlc_val: OPJ_UINT32 = 0;
  let mut qinf: [OPJ_UINT32; 2] = [0; 2];
  let mut c_q: OPJ_UINT32 = 0;
  let mut sp = 0 as *mut OPJ_UINT32;
  let mut x: OPJ_INT32 = 0;
  let mut y: OPJ_INT32 = 0;
  let mut stripe_causal = (cblksty & 0x8 as libc::c_int as libc::c_uint
    != 0 as libc::c_int as libc::c_uint) as libc::c_int;
  let mut cblk_len = 0 as libc::c_int as OPJ_UINT32;
  // stops unused parameter message
  // stops unused parameter message
  // We ignor orient, because the same decoder is used for all subbands
  // We also ignore check_pterm, because I am not sure how it applies
  if roishift != 0 as libc::c_int as libc::c_uint {
    if !p_manager_mutex.is_null() {
      opj_mutex_lock(p_manager_mutex);
    }
    opj_event_msg(
      p_manager,
      1 as libc::c_int,
      b"We do not support ROI in decoding HT codeblocks\n\x00" as *const u8 as *const libc::c_char,
    );
    if !p_manager_mutex.is_null() {
      opj_mutex_unlock(p_manager_mutex);
    }
    return 0 as libc::c_int;
  }
  if opj_t1_allocate_buffers(
    t1,
    ((*cblk).x1 - (*cblk).x0) as OPJ_UINT32,
    ((*cblk).y1 - (*cblk).y0) as OPJ_UINT32,
  ) == 0
  {
    return 0 as libc::c_int;
  }
  if (*cblk).Mb == 0 as libc::c_int as libc::c_uint {
    return 1 as libc::c_int;
  }
  /* numbps = Mb + 1 - zero_bplanes, Mb = Kmax, zero_bplanes = missing_msbs */
  zero_bplanes = (*cblk)
    .Mb
    .wrapping_add(1 as libc::c_int as libc::c_uint)
    .wrapping_sub((*cblk).numbps);
  /* Compute whole codeblock length from chunk lengths */
  cblk_len = 0 as libc::c_int as OPJ_UINT32;
  let mut i: OPJ_UINT32 = 0;
  i = 0 as libc::c_int as OPJ_UINT32;
  while i < (*cblk).numchunks {
    cblk_len = (cblk_len as libc::c_uint).wrapping_add((*(*cblk).chunks.offset(i as isize)).len)
      as OPJ_UINT32 as OPJ_UINT32;
    i = i.wrapping_add(1)
  }
  if (*cblk).numchunks > 1 as libc::c_int as libc::c_uint || t1.mustuse_cblkdatabuffer != 0 {
    let mut i_0: OPJ_UINT32 = 0;
    /* Allocate temporary memory if needed */
    if cblk_len > t1.cblkdatabuffersize {
      cblkdata = opj_realloc(
        t1.cblkdatabuffer as *mut libc::c_void,
        cblk_len as size_t,
      ) as *mut OPJ_BYTE;
      if cblkdata.is_null() {
        return 0 as libc::c_int;
      }
      t1.cblkdatabuffer = cblkdata;
      t1.cblkdatabuffersize = cblk_len
    }
    /* Concatenate all chunks */
    cblkdata = t1.cblkdatabuffer;
    cblk_len = 0 as libc::c_int as OPJ_UINT32;
    i_0 = 0 as libc::c_int as OPJ_UINT32;
    while i_0 < (*cblk).numchunks {
      memcpy(
        cblkdata.offset(cblk_len as isize) as *mut libc::c_void,
        (*(*cblk).chunks.offset(i_0 as isize)).data as *const libc::c_void,
        (*(*cblk).chunks.offset(i_0 as isize)).len as libc::c_ulong,
      );
      cblk_len = (cblk_len as libc::c_uint).wrapping_add((*(*cblk).chunks.offset(i_0 as isize)).len)
        as OPJ_UINT32 as OPJ_UINT32;
      i_0 = i_0.wrapping_add(1)
    }
  } else if (*cblk).numchunks == 1 as libc::c_int as libc::c_uint {
    cblkdata = (*(*cblk).chunks.offset(0 as libc::c_int as isize)).data
  } else {
    /* Not sure if that can happen in practice, but avoid Coverity to */
    /* think we will dereference a null cblkdta pointer */
    return 1 as libc::c_int;
  }
  // OPJ_BYTE* coded_data is a pointer to bitstream
  coded_data = cblkdata;
  // OPJ_UINT32* decoded_data is a pointer to decoded codeblock data buf.
  decoded_data = t1.data as *mut OPJ_UINT32;
  // OPJ_UINT32 num_passes is the number of passes: 1 if CUP only, 2 for
  // CUP+SPP, and 3 for CUP+SPP+MRP
  num_passes = if (*cblk).numsegs > 0 as libc::c_int as libc::c_uint {
    (*(*cblk).segs.offset(0 as libc::c_int as isize)).real_num_passes
  } else {
    0 as libc::c_int as libc::c_uint
  };
  num_passes = (num_passes as libc::c_uint).wrapping_add(
    if (*cblk).numsegs > 1 as libc::c_int as libc::c_uint {
      (*(*cblk).segs.offset(1 as libc::c_int as isize)).real_num_passes
    } else {
      0 as libc::c_int as libc::c_uint
    },
  ) as OPJ_UINT32 as OPJ_UINT32;
  // OPJ_UINT32 lengths1 is the length of cleanup pass
  lengths1 = if num_passes > 0 as libc::c_int as libc::c_uint {
    (*(*cblk).segs.offset(0 as libc::c_int as isize)).len
  } else {
    0 as libc::c_int as libc::c_uint
  };
  // OPJ_UINT32 lengths2 is the length of refinement passes (either SPP only or SPP+MRP)
  lengths2 = if num_passes > 1 as libc::c_int as libc::c_uint {
    (*(*cblk).segs.offset(1 as libc::c_int as isize)).len
  } else {
    0 as libc::c_int as libc::c_uint
  };
  // OPJ_INT32 width is the decoded codeblock width
  width = (*cblk).x1 - (*cblk).x0;
  // OPJ_INT32 height is the decoded codeblock height
  height = (*cblk).y1 - (*cblk).y0;
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
  pflags = t1.flags as *mut OPJ_UINT32;
  sigma1 = pflags;
  sigma2 = sigma1.offset(132 as libc::c_int as isize);
  // mbr arrangement is similar to sigma; mbr contains locations
  // that become significant during significance propagation pass
  mbr1 = sigma2.offset(132 as libc::c_int as isize);
  mbr2 = mbr1.offset(132 as libc::c_int as isize);
  //a pointer to sigma
  sip = sigma1; //pointers to arrays to be used interchangeably
  sip_shift = 0 as libc::c_int as OPJ_UINT32; //the amount of shift needed for sigma
  if num_passes > 1 as libc::c_int as libc::c_uint && lengths2 == 0 as libc::c_int as libc::c_uint {
    if !p_manager_mutex.is_null() {
      opj_mutex_lock(p_manager_mutex);
    }
    opj_event_msg(p_manager, 2 as libc::c_int,
                      b"A malformed codeblock that has more than one coding pass, but zero length for 2nd and potentially the 3rd pass in an HT codeblock.\n\x00"
                          as *const u8 as *const libc::c_char);
    if !p_manager_mutex.is_null() {
      opj_mutex_unlock(p_manager_mutex);
    }
    num_passes = 1 as libc::c_int as OPJ_UINT32
  }
  if num_passes > 3 as libc::c_int as libc::c_uint {
    if !p_manager_mutex.is_null() {
      opj_mutex_lock(p_manager_mutex);
    }
    opj_event_msg(p_manager, 1 as libc::c_int,
                      b"We do not support more than 3 coding passes in an HT codeblock; This codeblocks has %d passes.\n\x00"
                          as *const u8 as *const libc::c_char, num_passes);
    if !p_manager_mutex.is_null() {
      opj_mutex_unlock(p_manager_mutex);
    }
    return 0 as libc::c_int;
  }
  if (*cblk).Mb > 30 as libc::c_int as libc::c_uint {
    /* This check is better moved to opj_t2_read_packet_header() in t2.c
      We do not have enough precision to decode any passes
      The design of openjpeg assumes that the bits of a 32-bit integer are
      assigned as follows:
      bit 31 is for sign
      bits 30-1 are for magnitude
      bit 0 is for the center of the quantization bin
      Therefore we can only do values of cblk->Mb <= 30
    */
    if !p_manager_mutex.is_null() {
      opj_mutex_lock(p_manager_mutex);
    }
    opj_event_msg(p_manager, 1 as libc::c_int,
                      b"32 bits are not enough to decode this codeblock, since the number of bitplane, %d, is larger than 30.\n\x00"
                          as *const u8 as *const libc::c_char, (*cblk).Mb);
    if !p_manager_mutex.is_null() {
      opj_mutex_unlock(p_manager_mutex);
    }
    return 0 as libc::c_int;
  }
  if zero_bplanes > (*cblk).Mb {
    /* This check is better moved to opj_t2_read_packet_header() in t2.c,
    in the line "l_cblk->numbps = (OPJ_UINT32)l_band->numbps + 1 - i;"
    where i is the zero bitplanes, and should be no larger than cblk->Mb
    We cannot have more zero bitplanes than there are planes. */
    if !p_manager_mutex.is_null() {
      opj_mutex_lock(p_manager_mutex);
    }
    opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Malformed HT codeblock. Decoding this codeblock is stopped. There are %d zero bitplanes in %d bitplanes.\n\x00"
                          as *const u8 as *const libc::c_char, zero_bplanes,
                      (*cblk).Mb);
    if !p_manager_mutex.is_null() {
      opj_mutex_unlock(p_manager_mutex);
    }
    return 0 as libc::c_int;
  } else {
    if zero_bplanes == (*cblk).Mb && num_passes > 1 as libc::c_int as libc::c_uint {
      /* When the number of zero bitplanes is equal to the number of bitplanes,
      only the cleanup pass makes sense*/
      if only_cleanup_pass_is_decoded == 0 as libc::c_int {
        if !p_manager_mutex.is_null() {
          opj_mutex_lock(p_manager_mutex);
        }
        /* We have a second check to prevent the possibility of an overrun condition,
        in the very unlikely event of a second thread discovering that
        only_cleanup_pass_is_decoded is false before the first thread changing
        the condition. */
        if only_cleanup_pass_is_decoded == 0 as libc::c_int {
          only_cleanup_pass_is_decoded = 1 as libc::c_int;
          opj_event_msg(p_manager, 2 as libc::c_int,
                                  b"Malformed HT codeblock. When the number of zero planes bitplanes is equal to the number of bitplanes, only the cleanup pass makes sense, but we have %d passes in this codeblock. Therefore, only the cleanup pass will be decoded. This message will not be displayed again.\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  num_passes);
        }
        if !p_manager_mutex.is_null() {
          opj_mutex_unlock(p_manager_mutex);
        }
      }
      num_passes = 1 as libc::c_int as OPJ_UINT32
    }
  }
  /* OPJ_UINT32 */
  p = (*cblk).numbps;
  // OPJ_UINT32 zero planes plus 1
  zero_bplanes_p1 = zero_bplanes.wrapping_add(1 as libc::c_int as libc::c_uint);
  if lengths1 < 2 as libc::c_int as libc::c_uint
    || lengths1 > cblk_len
    || lengths1.wrapping_add(lengths2) > cblk_len
  {
    if !p_manager_mutex.is_null() {
      opj_mutex_lock(p_manager_mutex);
    }
    opj_event_msg(
      p_manager,
      1 as libc::c_int,
      b"Malformed HT codeblock. Invalid codeblock length values.\n\x00" as *const u8
        as *const libc::c_char,
    );
    if !p_manager_mutex.is_null() {
      opj_mutex_unlock(p_manager_mutex);
    }
    return 0 as libc::c_int;
  }
  // read scup and fix the bytes there
  lcup = lengths1 as libc::c_int; // length of CUP
                                  //scup is the length of MEL + VLC
  scup = ((*coded_data.offset((lcup - 1 as libc::c_int) as isize) as libc::c_int)
    << 4 as libc::c_int)
    + (*coded_data.offset((lcup - 2 as libc::c_int) as isize) as libc::c_int & 0xf as libc::c_int);
  if scup < 2 as libc::c_int || scup > lcup || scup > 4079 as libc::c_int {
    //something is wrong
    /* The standard stipulates 2 <= Scup <= min(Lcup, 4079) */
    if !p_manager_mutex.is_null() {
      opj_mutex_lock(p_manager_mutex);
    }
    opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Malformed HT codeblock. One of the following condition is not met: 2 <= Scup <= min(Lcup, 4079)\n\x00"
                          as *const u8 as *const libc::c_char);
    if !p_manager_mutex.is_null() {
      opj_mutex_unlock(p_manager_mutex);
    }
    return 0 as libc::c_int;
  }
  // init structures
  mel_init(&mut mel, coded_data, lcup, scup);
  rev_init(&mut vlc, coded_data, lcup, scup);
  frwd_init(
    &mut magsgn,
    coded_data,
    lcup - scup,
    0xff as libc::c_int as OPJ_UINT32,
  );
  if num_passes > 1 as libc::c_int as libc::c_uint {
    // needs to be tested
    frwd_init(
      &mut sigprop,
      coded_data.offset(lengths1 as isize),
      lengths2 as libc::c_int,
      0 as libc::c_int as OPJ_UINT32,
    );
  }
  if num_passes > 2 as libc::c_int as libc::c_uint {
    rev_init_mrp(
      &mut magref,
      coded_data,
      lengths1 as libc::c_int,
      lengths2 as libc::c_int,
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
  line_state = mbr2.offset(132 as libc::c_int as isize) as *mut OPJ_UINT8;
  //initial 2 lines
  // ///////////////
  lsp = line_state; // point to line state
  *lsp.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_UINT8; // for initial row of quad, we set to 0
  run = mel_get_run(&mut mel); // decode runs of events from MEL bitstrm
                               // data represented as runs of 0 events
                               // See mel_decode description
  qinf[1 as libc::c_int as usize] = 0 as libc::c_int as OPJ_UINT32; // quad info decoded from VLC bitstream
  qinf[0 as libc::c_int as usize] = qinf[1 as libc::c_int as usize]; // context for quad q
  c_q = 0 as libc::c_int as OPJ_UINT32; // decoded codeblock samples
  sp = decoded_data;
  // vlc_val;                 // fetched data from VLC bitstream
  x = 0 as libc::c_int;
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
    qinf[0 as libc::c_int as usize] = vlc_tbl0
      [(c_q << 7 as libc::c_int | vlc_val & 0x7f as libc::c_int as libc::c_uint) as usize]
      as OPJ_UINT32;
    if c_q == 0 as libc::c_int as libc::c_uint {
      // if zero context, we need to use one MEL event
      run -= 2 as libc::c_int; //the number of 0 events is multiplied by 2, so subtract 2
                               // Is the run terminated in 1? if so, use decoded VLC code,
                               // otherwise, discard decoded data, since we will decoded again
                               // using a different context
      qinf[0 as libc::c_int as usize] = if run == -(1 as libc::c_int) {
        qinf[0 as libc::c_int as usize]
      } else {
        0 as libc::c_int as libc::c_uint
      };
      // is run -1 or -2? this means a run has been consumed
      if run < 0 as libc::c_int {
        run = mel_get_run(&mut mel)
        // get another run
      }
    }
    // prepare context for the next quad; eqn. 1 in ITU T.814
    c_q = (qinf[0 as libc::c_int as usize] & 0x10 as libc::c_int as libc::c_uint)
      >> 4 as libc::c_int
      | (qinf[0 as libc::c_int as usize] & 0xe0 as libc::c_int as libc::c_uint) >> 5 as libc::c_int;
    //remove data from vlc stream (0 bits are removed if qinf is not used)
    vlc_val = rev_advance(
      &mut vlc,
      qinf[0 as libc::c_int as usize] & 0x7 as libc::c_int as libc::c_uint,
    );
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
    *sip |= ((qinf[0 as libc::c_int as usize] & 0x30 as libc::c_int as libc::c_uint)
      >> 4 as libc::c_int
      | (qinf[0 as libc::c_int as usize] & 0xc0 as libc::c_int as libc::c_uint)
        >> 2 as libc::c_int)
      << sip_shift;
    //second quad
    qinf[1 as libc::c_int as usize] = 0 as libc::c_int as OPJ_UINT32;
    if (x + 2 as libc::c_int) < width {
      // do not run if codeblock is narrower
      //decode VLC using the context c_q and the head of the VLC bitstream
      qinf[1 as libc::c_int as usize] = vlc_tbl0
        [(c_q << 7 as libc::c_int | vlc_val & 0x7f as libc::c_int as libc::c_uint) as usize]
        as OPJ_UINT32;
      // if context is zero, use one MEL event
      if c_q == 0 as libc::c_int as libc::c_uint {
        //zero context
        run -= 2 as libc::c_int; //subtract 2, since events number if multiplied by 2
                                 // if event is 0, discard decoded qinf
        qinf[1 as libc::c_int as usize] = if run == -(1 as libc::c_int) {
          qinf[1 as libc::c_int as usize]
        } else {
          0 as libc::c_int as libc::c_uint
        };
        if run < 0 as libc::c_int {
          // have we consumed all events in a run
          run = mel_get_run(&mut mel)
          // if yes, then get another run
        }
      }
      //prepare context for the next quad, eqn. 1 in ITU T.814
      c_q = (qinf[1 as libc::c_int as usize] & 0x10 as libc::c_int as libc::c_uint)
        >> 4 as libc::c_int
        | (qinf[1 as libc::c_int as usize] & 0xe0 as libc::c_int as libc::c_uint)
          >> 5 as libc::c_int;
      //remove data from vlc stream, if qinf is not used, cwdlen is 0
      vlc_val = rev_advance(
        &mut vlc,
        qinf[1 as libc::c_int as usize] & 0x7 as libc::c_int as libc::c_uint,
      )
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
    *sip |= (qinf[1 as libc::c_int as usize] & 0x30 as libc::c_int as libc::c_uint
      | (qinf[1 as libc::c_int as usize] & 0xc0 as libc::c_int as libc::c_uint)
        << 2 as libc::c_int)
      << (4 as libc::c_int as libc::c_uint).wrapping_add(sip_shift); // move sigma pointer to next entry
    sip = sip.offset(if x & 0x7 as libc::c_int != 0 {
      1 as libc::c_int
    } else {
      0 as libc::c_int
    } as isize); // increment/decrement sip_shift by 16
    sip_shift ^= 0x10 as libc::c_int as libc::c_uint;
    // retrieve u
    // ///////////
    // uvlc_mode is made up of u_offset bits from the quad pair
    uvlc_mode = (qinf[0 as libc::c_int as usize] & 0x8 as libc::c_int as libc::c_uint)
      >> 3 as libc::c_int
      | (qinf[1 as libc::c_int as usize] & 0x8 as libc::c_int as libc::c_uint) >> 2 as libc::c_int;
    if uvlc_mode == 3 as libc::c_int as libc::c_uint {
      // if both u_offset are set, get an event from
      // the MEL run of events
      run -= 2 as libc::c_int; //subtract 2, since events number if multiplied by 2
      uvlc_mode = (uvlc_mode as libc::c_uint).wrapping_add(if run == -(1 as libc::c_int) {
        1 as libc::c_int
      } else {
        0 as libc::c_int
      } as libc::c_uint) as OPJ_UINT32 as OPJ_UINT32; //increment uvlc_mode if event is 1
      if run < 0 as libc::c_int {
        // if run is consumed (run is -1 or -2), get another run
        run = mel_get_run(&mut mel)
      }
    }
    //decode uvlc_mode to get u for both quads
    consumed_bits = decode_init_uvlc(vlc_val, uvlc_mode, U_q.as_mut_ptr());
    if U_q[0 as libc::c_int as usize] > zero_bplanes_p1
      || U_q[1 as libc::c_int as usize] > zero_bplanes_p1
    {
      if !p_manager_mutex.is_null() {
        opj_mutex_lock(p_manager_mutex);
      }
      opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Malformed HT codeblock. Decoding this codeblock is stopped. U_q is larger than zero bitplanes + 1 \n\x00"
                              as *const u8 as *const libc::c_char);
      if !p_manager_mutex.is_null() {
        opj_mutex_unlock(p_manager_mutex);
      }
      return 0 as libc::c_int;
    }
    //consume u bits in the VLC code
    vlc_val = rev_advance(&mut vlc, consumed_bits);
    //decode magsgn and update line_state
    // ///////////////////////////////////
    //We obtain a mask for the samples locations that needs evaluation
    locs = 0xff as libc::c_int as OPJ_UINT32;
    if x + 4 as libc::c_int > width {
      locs >>= x + 4 as libc::c_int - width << 1 as libc::c_int
      // limits width
    } // limits height
    locs = if height > 1 as libc::c_int {
      locs
    } else {
      (locs) & 0x55 as libc::c_int as libc::c_uint
    };
    if ((qinf[0 as libc::c_int as usize] & 0xf0 as libc::c_int as libc::c_uint) >> 4 as libc::c_int
      | qinf[1 as libc::c_int as usize] & 0xf0 as libc::c_int as libc::c_uint)
      & !locs
      != 0
    {
      if !p_manager_mutex.is_null() {
        opj_mutex_lock(p_manager_mutex);
      }
      opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Malformed HT codeblock. VLC code produces significant samples outside the codeblock area.\n\x00"
                              as *const u8 as *const libc::c_char);
      if !p_manager_mutex.is_null() {
        opj_mutex_unlock(p_manager_mutex);
      }
      return 0 as libc::c_int;
    }
    //first quad, starting at first sample in quad and moving on
    if qinf[0 as libc::c_int as usize] & 0x10 as libc::c_int as libc::c_uint != 0 {
      //is it significant? (sigma_n)
      let mut val: OPJ_UINT32 = 0; //get 32 bits of magsgn data
      ms_val = frwd_fetch(&mut magsgn); //evaluate m_n (number of bits
      m_n = U_q[0 as libc::c_int as usize].wrapping_sub(
        qinf[0 as libc::c_int as usize] >> 12 as libc::c_int & 1 as libc::c_int as libc::c_uint,
      );
      // to read from bitstream), using EMB e_k
      frwd_advance(&mut magsgn, m_n); //consume m_n
      val = ms_val << 31 as libc::c_int; //get sign bit
      v_n = ms_val & ((1 as libc::c_uint) << m_n).wrapping_sub(1 as libc::c_int as libc::c_uint); //keep only m_n bits
      v_n |= ((qinf[0 as libc::c_int as usize] & 0x100 as libc::c_int as libc::c_uint)
        >> 8 as libc::c_int)
        << m_n; //add EMB e_1 as MSB
      v_n |= 1 as libc::c_int as libc::c_uint; //add center of bin
                                               //v_n now has 2 * (\mu - 1) + 0.5 with correct sign bit
                                               //add 2 to make it 2*\mu+0.5, shift it up to missing MSBs
      *sp.offset(0 as libc::c_int as isize) = val
        | v_n.wrapping_add(2 as libc::c_int as libc::c_uint)
          << p.wrapping_sub(1 as libc::c_int as libc::c_uint)
    } else if locs & 0x1 as libc::c_int as libc::c_uint != 0 {
      // if this is inside the codeblock, set the
      *sp.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_UINT32
      // sample to zero
    }
    if qinf[0 as libc::c_int as usize] & 0x20 as libc::c_int as libc::c_uint != 0 {
      //sigma_n
      let mut val_0: OPJ_UINT32 = 0; //get 32 bits
      let mut t: OPJ_UINT32 = 0; //m_n, uses EMB e_k
      ms_val = frwd_fetch(&mut magsgn); //consume m_n
      m_n = U_q[0 as libc::c_int as usize].wrapping_sub(
        qinf[0 as libc::c_int as usize] >> 13 as libc::c_int & 1 as libc::c_int as libc::c_uint,
      ); //get sign bit
      frwd_advance(&mut magsgn, m_n); //keep only m_n bits
      val_0 = ms_val << 31 as libc::c_int; //add EMB e_1
      v_n = ms_val & ((1 as libc::c_uint) << m_n).wrapping_sub(1 as libc::c_int as libc::c_uint); //bin center
      v_n |= ((qinf[0 as libc::c_int as usize] & 0x200 as libc::c_int as libc::c_uint)
        >> 9 as libc::c_int)
        << m_n;
      v_n |= 1 as libc::c_int as libc::c_uint;
      //v_n now has 2 * (\mu - 1) + 0.5 with correct sign bit
      //add 2 to make it 2*\mu+0.5, shift it up to missing MSBs
      *sp.offset(stride as isize) = val_0
        | v_n.wrapping_add(2 as libc::c_int as libc::c_uint)
          << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
      //update line_state: bit 7 (\sigma^N), and E^N
      t =
        (*lsp.offset(0 as libc::c_int as isize) as libc::c_int & 0x7f as libc::c_int) as OPJ_UINT32; // keep E^NW
      v_n = (32 as libc::c_int as libc::c_uint).wrapping_sub(count_leading_zeros(v_n));
      *lsp.offset(0 as libc::c_int as isize) =
        (0x80 as libc::c_int as libc::c_uint | (if t > v_n { t } else { v_n })) as OPJ_UINT8
    } else if locs & 0x2 as libc::c_int as libc::c_uint != 0 {
      // if this is inside the codeblock, set the
      *sp.offset(stride as isize) = 0 as libc::c_int as OPJ_UINT32
      // sample to zero
    } // move to next quad information
    lsp = lsp.offset(1); // move to next column of samples
    sp = sp.offset(1);
    //this is similar to the above two samples
    if qinf[0 as libc::c_int as usize] & 0x40 as libc::c_int as libc::c_uint != 0 {
      let mut val_1: OPJ_UINT32 = 0; //m_n
      ms_val = frwd_fetch(&mut magsgn); //center of bin
      m_n = U_q[0 as libc::c_int as usize].wrapping_sub(
        qinf[0 as libc::c_int as usize] >> 14 as libc::c_int & 1 as libc::c_int as libc::c_uint,
      );
      frwd_advance(&mut magsgn, m_n);
      val_1 = ms_val << 31 as libc::c_int;
      v_n = ms_val & ((1 as libc::c_uint) << m_n).wrapping_sub(1 as libc::c_int as libc::c_uint);
      v_n |= ((qinf[0 as libc::c_int as usize] & 0x400 as libc::c_int as libc::c_uint)
        >> 10 as libc::c_int)
        << m_n;
      v_n |= 1 as libc::c_int as libc::c_uint;
      *sp.offset(0 as libc::c_int as isize) = val_1
        | v_n.wrapping_add(2 as libc::c_int as libc::c_uint)
          << p.wrapping_sub(1 as libc::c_int as libc::c_uint)
    } else if locs & 0x4 as libc::c_int as libc::c_uint != 0 {
      *sp.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_UINT32
    }
    *lsp.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_UINT8;
    if qinf[0 as libc::c_int as usize] & 0x80 as libc::c_int as libc::c_uint != 0 {
      let mut val_2: OPJ_UINT32 = 0;
      ms_val = frwd_fetch(&mut magsgn);
      m_n = U_q[0 as libc::c_int as usize].wrapping_sub(
        qinf[0 as libc::c_int as usize] >> 15 as libc::c_int & 1 as libc::c_int as libc::c_uint,
      );
      frwd_advance(&mut magsgn, m_n);
      val_2 = ms_val << 31 as libc::c_int;
      v_n = ms_val & ((1 as libc::c_uint) << m_n).wrapping_sub(1 as libc::c_int as libc::c_uint);
      v_n |= ((qinf[0 as libc::c_int as usize] & 0x800 as libc::c_int as libc::c_uint)
        >> 11 as libc::c_int)
        << m_n;
      v_n |= 1 as libc::c_int as libc::c_uint;
      *sp.offset(stride as isize) = val_2
        | v_n.wrapping_add(2 as libc::c_int as libc::c_uint)
          << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
      //line_state: bit 7 (\sigma^NW), and E^NW for next quad
      *lsp.offset(0 as libc::c_int as isize) = (0x80 as libc::c_int as libc::c_uint
        | (32 as libc::c_int as libc::c_uint).wrapping_sub(count_leading_zeros(v_n)))
        as OPJ_UINT8
    } else if locs & 0x8 as libc::c_int as libc::c_uint != 0 {
      //if outside set to 0
      *sp.offset(stride as isize) = 0 as libc::c_int as OPJ_UINT32
    } //move to next column
    sp = sp.offset(1);
    //second quad
    if qinf[1 as libc::c_int as usize] & 0x10 as libc::c_int as libc::c_uint != 0 {
      let mut val_3: OPJ_UINT32 = 0; //m_n
      ms_val = frwd_fetch(&mut magsgn);
      m_n = U_q[1 as libc::c_int as usize].wrapping_sub(
        qinf[1 as libc::c_int as usize] >> 12 as libc::c_int & 1 as libc::c_int as libc::c_uint,
      );
      frwd_advance(&mut magsgn, m_n);
      val_3 = ms_val << 31 as libc::c_int;
      v_n = ms_val & ((1 as libc::c_uint) << m_n).wrapping_sub(1 as libc::c_int as libc::c_uint);
      v_n |= ((qinf[1 as libc::c_int as usize] & 0x100 as libc::c_int as libc::c_uint)
        >> 8 as libc::c_int)
        << m_n;
      v_n |= 1 as libc::c_int as libc::c_uint;
      *sp.offset(0 as libc::c_int as isize) = val_3
        | v_n.wrapping_add(2 as libc::c_int as libc::c_uint)
          << p.wrapping_sub(1 as libc::c_int as libc::c_uint)
    } else if locs & 0x10 as libc::c_int as libc::c_uint != 0 {
      *sp.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_UINT32
    }
    if qinf[1 as libc::c_int as usize] & 0x20 as libc::c_int as libc::c_uint != 0 {
      let mut val_4: OPJ_UINT32 = 0;
      let mut t_0: OPJ_UINT32 = 0;
      ms_val = frwd_fetch(&mut magsgn);
      //max(E^NW, E^N) | s
      m_n = U_q[1 as libc::c_int as usize].wrapping_sub(
        qinf[1 as libc::c_int as usize] >> 13 as libc::c_int & 1 as libc::c_int as libc::c_uint,
      ); //m_n
      frwd_advance(&mut magsgn, m_n);
      val_4 = ms_val << 31 as libc::c_int;
      v_n = ms_val & ((1 as libc::c_uint) << m_n).wrapping_sub(1 as libc::c_int as libc::c_uint);
      v_n |= ((qinf[1 as libc::c_int as usize] & 0x200 as libc::c_int as libc::c_uint)
        >> 9 as libc::c_int)
        << m_n;
      v_n |= 1 as libc::c_int as libc::c_uint;
      *sp.offset(stride as isize) = val_4
        | v_n.wrapping_add(2 as libc::c_int as libc::c_uint)
          << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
      t_0 =
        (*lsp.offset(0 as libc::c_int as isize) as libc::c_int & 0x7f as libc::c_int) as OPJ_UINT32;
      v_n = (32 as libc::c_int as libc::c_uint).wrapping_sub(count_leading_zeros(v_n));
      *lsp.offset(0 as libc::c_int as isize) =
        (0x80 as libc::c_int as libc::c_uint | (if t_0 > v_n { t_0 } else { v_n })) as OPJ_UINT8
    } else if locs & 0x20 as libc::c_int as libc::c_uint != 0 {
      *sp.offset(stride as isize) = 0 as libc::c_int as OPJ_UINT32
      //update line_state: bit 7 (\sigma^N), and E^N
      //E^NW
      //E^N
      //no need to update line_state
    } //move line state to next quad
    lsp = lsp.offset(1); //move to next sample
    sp = sp.offset(1); //m_n
    if qinf[1 as libc::c_int as usize] & 0x40 as libc::c_int as libc::c_uint != 0 {
      let mut val_5: OPJ_UINT32 = 0; //m_n
      ms_val = frwd_fetch(&mut magsgn); //center of bin
      m_n = U_q[1 as libc::c_int as usize].wrapping_sub(
        qinf[1 as libc::c_int as usize] >> 14 as libc::c_int & 1 as libc::c_int as libc::c_uint,
      );
      frwd_advance(&mut magsgn, m_n);
      val_5 = ms_val << 31 as libc::c_int;
      v_n = ms_val & ((1 as libc::c_uint) << m_n).wrapping_sub(1 as libc::c_int as libc::c_uint);
      v_n |= ((qinf[1 as libc::c_int as usize] & 0x400 as libc::c_int as libc::c_uint)
        >> 10 as libc::c_int)
        << m_n;
      v_n |= 1 as libc::c_int as libc::c_uint;
      *sp.offset(0 as libc::c_int as isize) = val_5
        | v_n.wrapping_add(2 as libc::c_int as libc::c_uint)
          << p.wrapping_sub(1 as libc::c_int as libc::c_uint)
    } else if locs & 0x40 as libc::c_int as libc::c_uint != 0 {
      *sp.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_UINT32
    }
    *lsp.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_UINT8;
    if qinf[1 as libc::c_int as usize] & 0x80 as libc::c_int as libc::c_uint != 0 {
      let mut val_6: OPJ_UINT32 = 0;
      ms_val = frwd_fetch(&mut magsgn);
      m_n = U_q[1 as libc::c_int as usize].wrapping_sub(
        qinf[1 as libc::c_int as usize] >> 15 as libc::c_int & 1 as libc::c_int as libc::c_uint,
      );
      frwd_advance(&mut magsgn, m_n);
      val_6 = ms_val << 31 as libc::c_int;
      v_n = ms_val & ((1 as libc::c_uint) << m_n).wrapping_sub(1 as libc::c_int as libc::c_uint);
      v_n |= ((qinf[1 as libc::c_int as usize] & 0x800 as libc::c_int as libc::c_uint)
        >> 11 as libc::c_int)
        << m_n;
      v_n |= 1 as libc::c_int as libc::c_uint;
      *sp.offset(stride as isize) = val_6
        | v_n.wrapping_add(2 as libc::c_int as libc::c_uint)
          << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
      //line_state: bit 7 (\sigma^NW), and E^NW for next quad
      *lsp.offset(0 as libc::c_int as isize) = (0x80 as libc::c_int as libc::c_uint
        | (32 as libc::c_int as libc::c_uint).wrapping_sub(count_leading_zeros(v_n)))
        as OPJ_UINT8
    } else if locs & 0x80 as libc::c_int as libc::c_uint != 0 {
      *sp.offset(stride as isize) = 0 as libc::c_int as OPJ_UINT32
    }
    sp = sp.offset(1);
    x += 4 as libc::c_int
  }
  //non-initial lines
  // ////////////////////////
  y = 2 as libc::c_int;
  while y < height {
    /*done at the end of loop*/
    let mut sip_0 = 0 as *mut OPJ_UINT32; // shift sigma to the upper half od the nibble
    let mut ls0: OPJ_UINT8 = 0; //move back to 0 (it might have been at 0x10)
    let mut x_0: OPJ_INT32 = 0; //choose sigma array
    sip_shift ^= 0x2 as libc::c_int as libc::c_uint; // read the line state value
    sip_shift &= 0xffffffef as libc::c_uint; // and set it to zero
    sip_0 = if y & 0x4 as libc::c_int != 0 {
      sigma2
    } else {
      sigma1
    }; // generated samples
    lsp = line_state; // context
    ls0 = *lsp.offset(0 as libc::c_int as isize);
    *lsp.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_UINT8;
    sp = decoded_data.offset((y * stride) as isize);
    c_q = 0 as libc::c_int as OPJ_UINT32;
    x_0 = 0 as libc::c_int;
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
      c_q |= (ls0 as libc::c_int >> 7 as libc::c_int) as libc::c_uint; //\sigma^NW | \sigma^N
      c_q |= (*lsp.offset(1 as libc::c_int as isize) as libc::c_int >> 5 as libc::c_int
        & 0x4 as libc::c_int) as libc::c_uint; //\sigma^NE | \sigma^NF
                                               //the following is very similar to previous code, so please refer to
                                               // that
      vlc_val = rev_fetch(&mut vlc);
      qinf[0 as libc::c_int as usize] = vlc_tbl1
        [(c_q << 7 as libc::c_int | vlc_val & 0x7f as libc::c_int as libc::c_uint) as usize]
        as OPJ_UINT32;
      if c_q == 0 as libc::c_int as libc::c_uint {
        //zero context
        run -= 2 as libc::c_int;
        qinf[0 as libc::c_int as usize] = if run == -(1 as libc::c_int) {
          qinf[0 as libc::c_int as usize]
        } else {
          0 as libc::c_int as libc::c_uint
        };
        if run < 0 as libc::c_int {
          run = mel_get_run(&mut mel)
        }
      }
      //prepare context for the next quad, \sigma^W | \sigma^SW
      c_q = (qinf[0 as libc::c_int as usize] & 0x40 as libc::c_int as libc::c_uint)
        >> 5 as libc::c_int
        | (qinf[0 as libc::c_int as usize] & 0x80 as libc::c_int as libc::c_uint)
          >> 6 as libc::c_int;
      //remove data from vlc stream
      vlc_val = rev_advance(
        &mut vlc,
        qinf[0 as libc::c_int as usize] & 0x7 as libc::c_int as libc::c_uint,
      );
      //update sigma
      // The update depends on the value of x and y; consider one OPJ_UINT32
      // if x is 0, 8, 16 and so on, and y is 2, 6, etc., then this
      // line update c locations
      //      nibble (4 bits) number   0 1 2 3 4 5 6 7
      //                         LSB   0 0 0 0 0 0 0 0
      //                               0 0 0 0 0 0 0 0
      //                               c c 0 0 0 0 0 0
      //                               c c 0 0 0 0 0 0
      *sip_0 |= ((qinf[0 as libc::c_int as usize] & 0x30 as libc::c_int as libc::c_uint)
        >> 4 as libc::c_int
        | (qinf[0 as libc::c_int as usize] & 0xc0 as libc::c_int as libc::c_uint)
          >> 2 as libc::c_int)
        << sip_shift;
      //second quad
      qinf[1 as libc::c_int as usize] = 0 as libc::c_int as OPJ_UINT32;
      if (x_0 + 2 as libc::c_int) < width {
        c_q |= (*lsp.offset(1 as libc::c_int as isize) as libc::c_int >> 7 as libc::c_int)
          as libc::c_uint;
        c_q |= (*lsp.offset(2 as libc::c_int as isize) as libc::c_int >> 5 as libc::c_int
          & 0x4 as libc::c_int) as libc::c_uint;
        qinf[1 as libc::c_int as usize] = vlc_tbl1
          [(c_q << 7 as libc::c_int | vlc_val & 0x7f as libc::c_int as libc::c_uint) as usize]
          as OPJ_UINT32;
        if c_q == 0 as libc::c_int as libc::c_uint {
          //zero context
          run -= 2 as libc::c_int;
          qinf[1 as libc::c_int as usize] = if run == -(1 as libc::c_int) {
            qinf[1 as libc::c_int as usize]
          } else {
            0 as libc::c_int as libc::c_uint
          };
          if run < 0 as libc::c_int {
            run = mel_get_run(&mut mel)
          }
        }
        //prepare context for the next quad
        c_q = (qinf[1 as libc::c_int as usize] & 0x40 as libc::c_int as libc::c_uint)
          >> 5 as libc::c_int
          | (qinf[1 as libc::c_int as usize] & 0x80 as libc::c_int as libc::c_uint)
            >> 6 as libc::c_int;
        //remove data from vlc stream
        vlc_val = rev_advance(
          &mut vlc,
          qinf[1 as libc::c_int as usize] & 0x7 as libc::c_int as libc::c_uint,
        )
      }
      //update sigma
      *sip_0 |= (qinf[1 as libc::c_int as usize] & 0x30 as libc::c_int as libc::c_uint
        | (qinf[1 as libc::c_int as usize] & 0xc0 as libc::c_int as libc::c_uint)
          << 2 as libc::c_int)
        << (4 as libc::c_int as libc::c_uint).wrapping_add(sip_shift);
      sip_0 = sip_0.offset(if x_0 & 0x7 as libc::c_int != 0 {
        1 as libc::c_int
      } else {
        0 as libc::c_int
      } as isize);
      sip_shift ^= 0x10 as libc::c_int as libc::c_uint;
      //retrieve u
      // //////////
      uvlc_mode_0 = (qinf[0 as libc::c_int as usize] & 0x8 as libc::c_int as libc::c_uint)
        >> 3 as libc::c_int
        | (qinf[1 as libc::c_int as usize] & 0x8 as libc::c_int as libc::c_uint)
          >> 2 as libc::c_int;
      consumed_bits_0 = decode_noninit_uvlc(vlc_val, uvlc_mode_0, U_q_0.as_mut_ptr());
      vlc_val = rev_advance(&mut vlc, consumed_bits_0);
      //calculate E^max and add it to U_q, eqns 5 and 6 in ITU T.814
      if qinf[0 as libc::c_int as usize]
        & 0xf0 as libc::c_int as libc::c_uint
        & (qinf[0 as libc::c_int as usize] & 0xf0 as libc::c_int as libc::c_uint)
          .wrapping_sub(1 as libc::c_int as libc::c_uint)
        != 0
      {
        // is \gamma_q 1?
        let mut E = ls0 as libc::c_uint & 0x7f as libc::c_uint; //max(E, E^NE, E^NF)
        E = if E > *lsp.offset(1 as libc::c_int as isize) as libc::c_uint & 0x7f as libc::c_uint {
          E
        } else {
          (*lsp.offset(1 as libc::c_int as isize) as libc::c_uint) & 0x7f as libc::c_uint
        };
        //since U_q already has u_q + 1, we subtract 2 instead of 1
        U_q_0[0 as libc::c_int as usize] = (U_q_0[0 as libc::c_int as usize] as libc::c_uint)
          .wrapping_add(if E > 2 as libc::c_int as libc::c_uint {
            E.wrapping_sub(2 as libc::c_int as libc::c_uint)
          } else {
            0 as libc::c_int as libc::c_uint
          }) as OPJ_UINT32 as OPJ_UINT32
      }
      if qinf[1 as libc::c_int as usize]
        & 0xf0 as libc::c_int as libc::c_uint
        & (qinf[1 as libc::c_int as usize] & 0xf0 as libc::c_int as libc::c_uint)
          .wrapping_sub(1 as libc::c_int as libc::c_uint)
        != 0
      {
        //is \gamma_q 1?
        let mut E_0 = *lsp.offset(1 as libc::c_int as isize) as libc::c_uint & 0x7f as libc::c_uint; //max(E, E^NE, E^NF)
        E_0 = if E_0 > *lsp.offset(2 as libc::c_int as isize) as libc::c_uint & 0x7f as libc::c_uint
        {
          E_0
        } else {
          (*lsp.offset(2 as libc::c_int as isize) as libc::c_uint) & 0x7f as libc::c_uint
        };
        //since U_q already has u_q + 1, we subtract 2 instead of 1
        U_q_0[1 as libc::c_int as usize] = (U_q_0[1 as libc::c_int as usize] as libc::c_uint)
          .wrapping_add(if E_0 > 2 as libc::c_int as libc::c_uint {
            E_0.wrapping_sub(2 as libc::c_int as libc::c_uint)
          } else {
            0 as libc::c_int as libc::c_uint
          }) as OPJ_UINT32 as OPJ_UINT32
      } //for next double quad
      if U_q_0[0 as libc::c_int as usize] > zero_bplanes_p1
        || U_q_0[1 as libc::c_int as usize] > zero_bplanes_p1
      {
        if !p_manager_mutex.is_null() {
          opj_mutex_lock(p_manager_mutex);
        }
        opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Malformed HT codeblock. Decoding this codeblock is stopped. U_q islarger than bitplanes + 1 \n\x00"
                                  as *const u8 as *const libc::c_char);
        if !p_manager_mutex.is_null() {
          opj_mutex_unlock(p_manager_mutex);
        }
        return 0 as libc::c_int;
      }
      ls0 = *lsp.offset(2 as libc::c_int as isize);
      let ref mut fresh14 = *lsp.offset(2 as libc::c_int as isize);
      *fresh14 = 0 as libc::c_int as OPJ_UINT8;
      *lsp.offset(1 as libc::c_int as isize) = *fresh14;
      //decode magsgn and update line_state
      // ///////////////////////////////////
      //locations where samples need update
      locs_0 = 0xff as libc::c_int as OPJ_UINT32;
      if x_0 + 4 as libc::c_int > width {
        locs_0 >>= x_0 + 4 as libc::c_int - width << 1 as libc::c_int
      }
      locs_0 = if y + 2 as libc::c_int <= height {
        locs_0
      } else {
        (locs_0) & 0x55 as libc::c_int as libc::c_uint
      };
      if ((qinf[0 as libc::c_int as usize] & 0xf0 as libc::c_int as libc::c_uint)
        >> 4 as libc::c_int
        | qinf[1 as libc::c_int as usize] & 0xf0 as libc::c_int as libc::c_uint)
        & !locs_0
        != 0
      {
        if !p_manager_mutex.is_null() {
          opj_mutex_lock(p_manager_mutex);
        }
        opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Malformed HT codeblock. VLC code produces significant samples outside the codeblock area.\n\x00"
                                  as *const u8 as *const libc::c_char);
        if !p_manager_mutex.is_null() {
          opj_mutex_unlock(p_manager_mutex);
        }
        return 0 as libc::c_int;
      }
      if qinf[0 as libc::c_int as usize] & 0x10 as libc::c_int as libc::c_uint != 0 {
        //sigma_n
        let mut val_7: OPJ_UINT32 = 0; //m_n
        ms_val_0 = frwd_fetch(&mut magsgn); //center of bin
        m_n_0 = U_q_0[0 as libc::c_int as usize].wrapping_sub(
          qinf[0 as libc::c_int as usize] >> 12 as libc::c_int & 1 as libc::c_int as libc::c_uint,
        );
        frwd_advance(&mut magsgn, m_n_0);
        val_7 = ms_val_0 << 31 as libc::c_int;
        v_n_0 =
          ms_val_0 & ((1 as libc::c_uint) << m_n_0).wrapping_sub(1 as libc::c_int as libc::c_uint);
        v_n_0 |= ((qinf[0 as libc::c_int as usize] & 0x100 as libc::c_int as libc::c_uint)
          >> 8 as libc::c_int)
          << m_n_0;
        v_n_0 |= 1 as libc::c_int as libc::c_uint;
        *sp.offset(0 as libc::c_int as isize) = val_7
          | v_n_0.wrapping_add(2 as libc::c_int as libc::c_uint)
            << p.wrapping_sub(1 as libc::c_int as libc::c_uint)
      } else if locs_0 & 0x1 as libc::c_int as libc::c_uint != 0 {
        *sp.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_UINT32
      }
      if qinf[0 as libc::c_int as usize] & 0x20 as libc::c_int as libc::c_uint != 0 {
        //sigma_n
        let mut val_8: OPJ_UINT32 = 0; //m_n
        let mut t_1: OPJ_UINT32 = 0; //center of bin
        ms_val_0 = frwd_fetch(&mut magsgn);
        m_n_0 = U_q_0[0 as libc::c_int as usize].wrapping_sub(
          qinf[0 as libc::c_int as usize] >> 13 as libc::c_int & 1 as libc::c_int as libc::c_uint,
        );
        frwd_advance(&mut magsgn, m_n_0);
        val_8 = ms_val_0 << 31 as libc::c_int;
        v_n_0 =
          ms_val_0 & ((1 as libc::c_uint) << m_n_0).wrapping_sub(1 as libc::c_int as libc::c_uint);
        v_n_0 |= ((qinf[0 as libc::c_int as usize] & 0x200 as libc::c_int as libc::c_uint)
          >> 9 as libc::c_int)
          << m_n_0;
        v_n_0 |= 1 as libc::c_int as libc::c_uint;
        *sp.offset(stride as isize) = val_8
          | v_n_0.wrapping_add(2 as libc::c_int as libc::c_uint)
            << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
        //update line_state: bit 7 (\sigma^N), and E^N
        t_1 = (*lsp.offset(0 as libc::c_int as isize) as libc::c_int & 0x7f as libc::c_int)
          as OPJ_UINT32; //E^NW
        v_n_0 = (32 as libc::c_int as libc::c_uint).wrapping_sub(count_leading_zeros(v_n_0));
        *lsp.offset(0 as libc::c_int as isize) = (0x80 as libc::c_int as libc::c_uint
          | (if t_1 > v_n_0 { t_1 } else { v_n_0 }))
          as OPJ_UINT8
      } else if locs_0 & 0x2 as libc::c_int as libc::c_uint != 0 {
        *sp.offset(stride as isize) = 0 as libc::c_int as OPJ_UINT32
        //no need to update line_state
      }
      lsp = lsp.offset(1);
      sp = sp.offset(1);
      if qinf[0 as libc::c_int as usize] & 0x40 as libc::c_int as libc::c_uint != 0 {
        //sigma_n
        let mut val_9: OPJ_UINT32 = 0; //m_n
        ms_val_0 = frwd_fetch(&mut magsgn); //center of bin
        m_n_0 = U_q_0[0 as libc::c_int as usize].wrapping_sub(
          qinf[0 as libc::c_int as usize] >> 14 as libc::c_int & 1 as libc::c_int as libc::c_uint,
        );
        frwd_advance(&mut magsgn, m_n_0);
        val_9 = ms_val_0 << 31 as libc::c_int;
        v_n_0 =
          ms_val_0 & ((1 as libc::c_uint) << m_n_0).wrapping_sub(1 as libc::c_int as libc::c_uint);
        v_n_0 |= ((qinf[0 as libc::c_int as usize] & 0x400 as libc::c_int as libc::c_uint)
          >> 10 as libc::c_int)
          << m_n_0;
        v_n_0 |= 1 as libc::c_int as libc::c_uint;
        *sp.offset(0 as libc::c_int as isize) = val_9
          | v_n_0.wrapping_add(2 as libc::c_int as libc::c_uint)
            << p.wrapping_sub(1 as libc::c_int as libc::c_uint)
      } else if locs_0 & 0x4 as libc::c_int as libc::c_uint != 0 {
        *sp.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_UINT32
      }
      if qinf[0 as libc::c_int as usize] & 0x80 as libc::c_int as libc::c_uint != 0 {
        //sigma_n
        let mut val_10: OPJ_UINT32 = 0; //m_n
        ms_val_0 = frwd_fetch(&mut magsgn); //center of bin
        m_n_0 = U_q_0[0 as libc::c_int as usize].wrapping_sub(
          qinf[0 as libc::c_int as usize] >> 15 as libc::c_int & 1 as libc::c_int as libc::c_uint,
        );
        frwd_advance(&mut magsgn, m_n_0);
        val_10 = ms_val_0 << 31 as libc::c_int;
        v_n_0 =
          ms_val_0 & ((1 as libc::c_uint) << m_n_0).wrapping_sub(1 as libc::c_int as libc::c_uint);
        v_n_0 |= ((qinf[0 as libc::c_int as usize] & 0x800 as libc::c_int as libc::c_uint)
          >> 11 as libc::c_int)
          << m_n_0;
        v_n_0 |= 1 as libc::c_int as libc::c_uint;
        *sp.offset(stride as isize) = val_10
          | v_n_0.wrapping_add(2 as libc::c_int as libc::c_uint)
            << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
        //update line_state: bit 7 (\sigma^NW), and E^NW for next quad
        *lsp.offset(0 as libc::c_int as isize) = (0x80 as libc::c_int as libc::c_uint
          | (32 as libc::c_int as libc::c_uint).wrapping_sub(count_leading_zeros(v_n_0)))
          as OPJ_UINT8
      } else if locs_0 & 0x8 as libc::c_int as libc::c_uint != 0 {
        *sp.offset(stride as isize) = 0 as libc::c_int as OPJ_UINT32
      }
      sp = sp.offset(1);
      if qinf[1 as libc::c_int as usize] & 0x10 as libc::c_int as libc::c_uint != 0 {
        //sigma_n
        let mut val_11: OPJ_UINT32 = 0; //m_n
        ms_val_0 = frwd_fetch(&mut magsgn); //center of bin
        m_n_0 = U_q_0[1 as libc::c_int as usize].wrapping_sub(
          qinf[1 as libc::c_int as usize] >> 12 as libc::c_int & 1 as libc::c_int as libc::c_uint,
        );
        frwd_advance(&mut magsgn, m_n_0);
        val_11 = ms_val_0 << 31 as libc::c_int;
        v_n_0 =
          ms_val_0 & ((1 as libc::c_uint) << m_n_0).wrapping_sub(1 as libc::c_int as libc::c_uint);
        v_n_0 |= ((qinf[1 as libc::c_int as usize] & 0x100 as libc::c_int as libc::c_uint)
          >> 8 as libc::c_int)
          << m_n_0;
        v_n_0 |= 1 as libc::c_int as libc::c_uint;
        *sp.offset(0 as libc::c_int as isize) = val_11
          | v_n_0.wrapping_add(2 as libc::c_int as libc::c_uint)
            << p.wrapping_sub(1 as libc::c_int as libc::c_uint)
      } else if locs_0 & 0x10 as libc::c_int as libc::c_uint != 0 {
        *sp.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_UINT32
      }
      if qinf[1 as libc::c_int as usize] & 0x20 as libc::c_int as libc::c_uint != 0 {
        //sigma_n
        let mut val_12: OPJ_UINT32 = 0; //m_n
        let mut t_2: OPJ_UINT32 = 0; //center of bin
        ms_val_0 = frwd_fetch(&mut magsgn);
        m_n_0 = U_q_0[1 as libc::c_int as usize].wrapping_sub(
          qinf[1 as libc::c_int as usize] >> 13 as libc::c_int & 1 as libc::c_int as libc::c_uint,
        );
        frwd_advance(&mut magsgn, m_n_0);
        val_12 = ms_val_0 << 31 as libc::c_int;
        v_n_0 =
          ms_val_0 & ((1 as libc::c_uint) << m_n_0).wrapping_sub(1 as libc::c_int as libc::c_uint);
        v_n_0 |= ((qinf[1 as libc::c_int as usize] & 0x200 as libc::c_int as libc::c_uint)
          >> 9 as libc::c_int)
          << m_n_0;
        v_n_0 |= 1 as libc::c_int as libc::c_uint;
        *sp.offset(stride as isize) = val_12
          | v_n_0.wrapping_add(2 as libc::c_int as libc::c_uint)
            << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
        //update line_state: bit 7 (\sigma^N), and E^N
        t_2 = (*lsp.offset(0 as libc::c_int as isize) as libc::c_int & 0x7f as libc::c_int)
          as OPJ_UINT32; //E^NW
        v_n_0 = (32 as libc::c_int as libc::c_uint).wrapping_sub(count_leading_zeros(v_n_0));
        *lsp.offset(0 as libc::c_int as isize) = (0x80 as libc::c_int as libc::c_uint
          | (if t_2 > v_n_0 { t_2 } else { v_n_0 }))
          as OPJ_UINT8
      } else if locs_0 & 0x20 as libc::c_int as libc::c_uint != 0 {
        *sp.offset(stride as isize) = 0 as libc::c_int as OPJ_UINT32
        //no need to update line_state
      }
      lsp = lsp.offset(1);
      sp = sp.offset(1);
      if qinf[1 as libc::c_int as usize] & 0x40 as libc::c_int as libc::c_uint != 0 {
        //sigma_n
        let mut val_13: OPJ_UINT32 = 0; //m_n
        ms_val_0 = frwd_fetch(&mut magsgn); //center of bin
        m_n_0 = U_q_0[1 as libc::c_int as usize].wrapping_sub(
          qinf[1 as libc::c_int as usize] >> 14 as libc::c_int & 1 as libc::c_int as libc::c_uint,
        );
        frwd_advance(&mut magsgn, m_n_0);
        val_13 = ms_val_0 << 31 as libc::c_int;
        v_n_0 =
          ms_val_0 & ((1 as libc::c_uint) << m_n_0).wrapping_sub(1 as libc::c_int as libc::c_uint);
        v_n_0 |= ((qinf[1 as libc::c_int as usize] & 0x400 as libc::c_int as libc::c_uint)
          >> 10 as libc::c_int)
          << m_n_0;
        v_n_0 |= 1 as libc::c_int as libc::c_uint;
        *sp.offset(0 as libc::c_int as isize) = val_13
          | v_n_0.wrapping_add(2 as libc::c_int as libc::c_uint)
            << p.wrapping_sub(1 as libc::c_int as libc::c_uint)
      } else if locs_0 & 0x40 as libc::c_int as libc::c_uint != 0 {
        *sp.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_UINT32
      }
      if qinf[1 as libc::c_int as usize] & 0x80 as libc::c_int as libc::c_uint != 0 {
        //sigma_n
        let mut val_14: OPJ_UINT32 = 0; //m_n
        ms_val_0 = frwd_fetch(&mut magsgn); //center of bin
        m_n_0 = U_q_0[1 as libc::c_int as usize].wrapping_sub(
          qinf[1 as libc::c_int as usize] >> 15 as libc::c_int & 1 as libc::c_int as libc::c_uint,
        );
        frwd_advance(&mut magsgn, m_n_0);
        val_14 = ms_val_0 << 31 as libc::c_int;
        v_n_0 =
          ms_val_0 & ((1 as libc::c_uint) << m_n_0).wrapping_sub(1 as libc::c_int as libc::c_uint);
        v_n_0 |= ((qinf[1 as libc::c_int as usize] & 0x800 as libc::c_int as libc::c_uint)
          >> 11 as libc::c_int)
          << m_n_0;
        v_n_0 |= 1 as libc::c_int as libc::c_uint;
        *sp.offset(stride as isize) = val_14
          | v_n_0.wrapping_add(2 as libc::c_int as libc::c_uint)
            << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
        //update line_state: bit 7 (\sigma^NW), and E^NW for next quad
        *lsp.offset(0 as libc::c_int as isize) = (0x80 as libc::c_int as libc::c_uint
          | (32 as libc::c_int as libc::c_uint).wrapping_sub(count_leading_zeros(v_n_0)))
          as OPJ_UINT8
      } else if locs_0 & 0x80 as libc::c_int as libc::c_uint != 0 {
        *sp.offset(stride as isize) = 0 as libc::c_int as OPJ_UINT32
      }
      sp = sp.offset(1);
      x_0 += 4 as libc::c_int
    }
    y += 2 as libc::c_int;
    if num_passes > 1 as libc::c_int as libc::c_uint && y & 3 as libc::c_int == 0 as libc::c_int {
      //executed at multiples of 4
      // This is for SPP and potentially MRP
      if num_passes > 2 as libc::c_int as libc::c_uint {
        //do MRP
        // select the current stripe
        let mut cur_sig = if y & 0x4 as libc::c_int != 0 {
          sigma1
        } else {
          sigma2
        };
        // the address of the data that needs updating
        let mut dpp = decoded_data.offset(((y - 4 as libc::c_int) * stride) as isize); // half the center of the bin
        let mut half = (1 as libc::c_uint) << p.wrapping_sub(2 as libc::c_int as libc::c_uint);
        let mut i_1: OPJ_INT32 = 0;
        i_1 = 0 as libc::c_int;
        while i_1 < width {
          //Process one entry from sigma array at a time
          // Each nibble (4 bits) in the sigma array represents 4 rows,
          // and the 32 bits contain 8 columns
          let mut cwd = rev_fetch_mrp(&mut magref); // get 32 bit data
          let fresh15 = cur_sig; // 32 bit that will be processed now
          cur_sig = cur_sig.offset(1); // a mask for a column in sig
          let mut sig = *fresh15; // next column in decode samples
          let mut col_mask = 0xf as libc::c_uint;
          let mut dp = dpp.offset(i_1 as isize);
          if sig != 0 {
            // if any of the 32 bits are set
            let mut j: libc::c_int = 0;
            j = 0 as libc::c_int;
            while j < 8 as libc::c_int {
              //one column at a time
              if sig & col_mask != 0 {
                // lowest nibble
                let mut sample_mask = 0x11111111 as libc::c_uint & col_mask; //LSB
                if sig & sample_mask != 0 {
                  //if LSB is set
                  let mut sym: OPJ_UINT32 = 0; // decoded value cannot be zero
                  assert!(
                    *dp.offset(0 as libc::c_int as isize) != 0 as libc::c_int as libc::c_uint
                  );
                  sym = cwd & 1 as libc::c_int as libc::c_uint;
                  // remove center of bin if sym is 0
                  let ref mut fresh16 = *dp.offset(0 as libc::c_int as isize); // put half the center of bin
                  *fresh16 ^= (1 as libc::c_int as libc::c_uint).wrapping_sub(sym)
                    << p.wrapping_sub(1 as libc::c_int as libc::c_uint); //next row
                  let ref mut fresh17 = *dp.offset(0 as libc::c_int as isize);
                  *fresh17 |= half;
                  cwd >>= 1 as libc::c_int
                }
                sample_mask = (sample_mask as libc::c_uint).wrapping_add(sample_mask) as OPJ_UINT32
                  as OPJ_UINT32;
                if sig & sample_mask != 0 {
                  let mut sym_0: OPJ_UINT32 = 0;
                  assert!(*dp.offset(stride as isize) != 0 as libc::c_int as libc::c_uint);
                  sym_0 = cwd & 1 as libc::c_int as libc::c_uint;
                  let ref mut fresh18 = *dp.offset(stride as isize);
                  *fresh18 ^= (1 as libc::c_int as libc::c_uint).wrapping_sub(sym_0)
                    << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
                  let ref mut fresh19 = *dp.offset(stride as isize);
                  *fresh19 |= half;
                  cwd >>= 1 as libc::c_int
                }
                sample_mask = (sample_mask as libc::c_uint).wrapping_add(sample_mask) as OPJ_UINT32
                  as OPJ_UINT32;
                if sig & sample_mask != 0 {
                  let mut sym_1: OPJ_UINT32 = 0;
                  assert!(
                    *dp.offset((2 as libc::c_int * stride) as isize)
                      != 0 as libc::c_int as libc::c_uint
                  );
                  sym_1 = cwd & 1 as libc::c_int as libc::c_uint;
                  let ref mut fresh20 = *dp.offset((2 as libc::c_int * stride) as isize);
                  *fresh20 ^= (1 as libc::c_int as libc::c_uint).wrapping_sub(sym_1)
                    << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
                  let ref mut fresh21 = *dp.offset((2 as libc::c_int * stride) as isize);
                  *fresh21 |= half;
                  cwd >>= 1 as libc::c_int
                }
                sample_mask = (sample_mask as libc::c_uint).wrapping_add(sample_mask) as OPJ_UINT32
                  as OPJ_UINT32;
                if sig & sample_mask != 0 {
                  let mut sym_2: OPJ_UINT32 = 0;
                  assert!(
                    *dp.offset((3 as libc::c_int * stride) as isize)
                      != 0 as libc::c_int as libc::c_uint
                  );
                  sym_2 = cwd & 1 as libc::c_int as libc::c_uint;
                  let ref mut fresh22 = *dp.offset((3 as libc::c_int * stride) as isize);
                  *fresh22 ^= (1 as libc::c_int as libc::c_uint).wrapping_sub(sym_2)
                    << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
                  let ref mut fresh23 = *dp.offset((3 as libc::c_int * stride) as isize);
                  *fresh23 |= half;
                  cwd >>= 1 as libc::c_int
                }
                sample_mask = (sample_mask as libc::c_uint).wrapping_add(sample_mask) as OPJ_UINT32
                  as OPJ_UINT32
              }
              col_mask <<= 4 as libc::c_int;
              j += 1;
              dp = dp.offset(1)
              //next column
            }
          }
          // consume data according to the number of bits set
          rev_advance_mrp(&mut magref, population_count(sig));
          i_1 += 8 as libc::c_int
        }
      }
      if y >= 4 as libc::c_int {
        // update mbr array at the end of each stripe
        //generate mbr corresponding to a stripe
        let mut sig_0 = if y & 0x4 as libc::c_int != 0 {
          sigma1
        } else {
          sigma2
        };
        let mut mbr = if y & 0x4 as libc::c_int != 0 {
          mbr1
        } else {
          mbr2
        };
        //data is processed in patches of 8 columns, each
        // each 32 bits in sigma1 or mbr1 represent 4 rows
        //integrate horizontally
        let mut prev = 0 as libc::c_int as OPJ_UINT32; // previous columns
        let mut i_2: OPJ_INT32 = 0;
        i_2 = 0 as libc::c_int;
        while i_2 < width {
          let mut t_3: OPJ_UINT32 = 0;
          let mut z: OPJ_UINT32 = 0;
          //remove already significance samples
          *mbr.offset(0 as libc::c_int as isize) = *sig_0.offset(0 as libc::c_int as isize); //start with significant samples
          let ref mut fresh24 = *mbr.offset(0 as libc::c_int as isize); //for first column, left neighbors
          *fresh24 |= prev >> 28 as libc::c_int; //left neighbors
          let ref mut fresh25 = *mbr.offset(0 as libc::c_int as isize); //right neighbors
          *fresh25 |= *sig_0.offset(0 as libc::c_int as isize) << 4 as libc::c_int; //for last column, right neighbors
          let ref mut fresh26 = *mbr.offset(0 as libc::c_int as isize); // for next group of columns
          *fresh26 |= *sig_0.offset(0 as libc::c_int as isize) >> 4 as libc::c_int;
          let ref mut fresh27 = *mbr.offset(0 as libc::c_int as isize);
          *fresh27 |= *sig_0.offset(1 as libc::c_int as isize) << 28 as libc::c_int;
          prev = *sig_0.offset(0 as libc::c_int as isize);
          t_3 = *mbr.offset(0 as libc::c_int as isize);
          z = *mbr.offset(0 as libc::c_int as isize);
          z |= (t_3 & 0x77777777 as libc::c_int as libc::c_uint) << 1 as libc::c_int;
          z |= (t_3 & 0xeeeeeeee as libc::c_uint) >> 1 as libc::c_int;
          *mbr.offset(0 as libc::c_int as isize) = z & !*sig_0.offset(0 as libc::c_int as isize);
          i_2 += 8 as libc::c_int;
          mbr = mbr.offset(1);
          sig_0 = sig_0.offset(1)
        }
      }
      if y >= 8 as libc::c_int {
        //integrate vertically
        //above neighbors
        //below neighbors
        //wait until 8 rows has been processed
        let mut cur_sig_0 = 0 as *mut OPJ_UINT32;
        let mut cur_mbr = 0 as *mut OPJ_UINT32;
        let mut nxt_sig = 0 as *mut OPJ_UINT32;
        let mut nxt_mbr = 0 as *mut OPJ_UINT32;
        let mut prev_0: OPJ_UINT32 = 0;
        let mut val_15: OPJ_UINT32 = 0;
        let mut i_3: OPJ_INT32 = 0;
        // add membership from the next stripe, obtained above
        cur_sig_0 = if y & 0x4 as libc::c_int != 0 {
          sigma2
        } else {
          sigma1
        }; //future samples
        cur_mbr = if y & 0x4 as libc::c_int != 0 {
          mbr2
        } else {
          mbr1
        }; // the columns before these group of 8 columns
        nxt_sig = if y & 0x4 as libc::c_int != 0 {
          sigma1
        } else {
          sigma2
        };
        prev_0 = 0 as libc::c_int as OPJ_UINT32;
        i_3 = 0 as libc::c_int;
        while i_3 < width {
          let mut t_4 = *nxt_sig.offset(0 as libc::c_int as isize);
          //remove already significance samples
          t_4 |= prev_0 >> 28 as libc::c_int; //for first column, left neighbors
          t_4 |= *nxt_sig.offset(0 as libc::c_int as isize) << 4 as libc::c_int; //left neighbors
          t_4 |= *nxt_sig.offset(0 as libc::c_int as isize) >> 4 as libc::c_int; //right neighbors
          t_4 |= *nxt_sig.offset(1 as libc::c_int as isize) << 28 as libc::c_int; //for last column, right neighbors
          prev_0 = *nxt_sig.offset(0 as libc::c_int as isize); // for next group of columns
          if stripe_causal == 0 {
            let ref mut fresh28 = *cur_mbr.offset(0 as libc::c_int as isize);
            *fresh28 |= (t_4 & 0x11111111 as libc::c_uint) << 3 as libc::c_int
            //propagate up to cur_mbr
          }
          let ref mut fresh29 = *cur_mbr.offset(0 as libc::c_int as isize);
          *fresh29 &= !*cur_sig_0.offset(0 as libc::c_int as isize);
          i_3 += 8 as libc::c_int;
          cur_mbr = cur_mbr.offset(1);
          cur_sig_0 = cur_sig_0.offset(1);
          nxt_sig = nxt_sig.offset(1)
        }
        //find new locations and get signs
        cur_sig_0 = if y & 0x4 as libc::c_int != 0 {
          sigma2
        } else {
          sigma1
        }; //future samples
        cur_mbr = if y & 0x4 as libc::c_int != 0 {
          mbr2
        } else {
          mbr1
        }; //future samples
        nxt_sig = if y & 0x4 as libc::c_int != 0 {
          sigma1
        } else {
          sigma2
        }; // sample values for newly discovered
        nxt_mbr = if y & 0x4 as libc::c_int != 0 {
          mbr1
        } else {
          mbr2
        };
        val_15 = (3 as libc::c_uint) << p.wrapping_sub(2 as libc::c_int as libc::c_uint);
        // significant samples including the bin center
        i_3 = 0 as libc::c_int;
        while i_3 < width {
          let mut ux: OPJ_UINT32 = 0;
          let mut tx: OPJ_UINT32 = 0;
          let mut mbr_0 = *cur_mbr;
          let mut new_sig = 0 as libc::c_int as OPJ_UINT32;
          if mbr_0 != 0 {
            //are there any samples that might be significant
            let mut n: OPJ_INT32 = 0; //get 32 bits
            n = 0 as libc::c_int; //address for decoded samples
            while n < 8 as libc::c_int {
              let mut col_mask_0: OPJ_UINT32 = 0; //a mask to select a column
              let mut inv_sig: OPJ_UINT32 = 0; // insignificant samples
              let mut end: OPJ_INT32 = 0;
              let mut j_0: OPJ_INT32 = 0;
              let mut cwd_0 = frwd_fetch(&mut sigprop);
              let mut cnt = 0 as libc::c_int as OPJ_UINT32;
              let mut dp_0 = decoded_data.offset(((y - 8 as libc::c_int) * stride) as isize);
              dp_0 = dp_0.offset((i_3 + n) as isize);
              col_mask_0 = (0xf as libc::c_uint) << 4 as libc::c_int * n;
              inv_sig = !*cur_sig_0.offset(0 as libc::c_int as isize);
              //find the last sample we operate on
              end = if n + 4 as libc::c_int + i_3 < width {
                (n) + 4 as libc::c_int
              } else {
                (width) - i_3
              };
              j_0 = n;
              while j_0 < end {
                let mut sample_mask_0: OPJ_UINT32 = 0;
                if !(col_mask_0 & mbr_0 == 0 as libc::c_int as libc::c_uint) {
                  //scan mbr to find a new significant sample
                  sample_mask_0 = 0x11111111 as libc::c_uint & col_mask_0; // LSB
                  if mbr_0 & sample_mask_0 != 0 {
                    assert!(
                      *dp_0.offset(0 as libc::c_int as isize) == 0 as libc::c_int as libc::c_uint
                    );
                    if cwd_0 & 1 as libc::c_int as libc::c_uint != 0 {
                      //consume bit and increment number of
                      //consumed bits
                      //if this sample has become significant
                      // must propagate it to nearby samples
                      let mut t_5: OPJ_UINT32 = 0; // new significant samples
                      new_sig |= sample_mask_0; // propagation to neighbors
                      t_5 = (0x32 as libc::c_uint) << j_0 * 4 as libc::c_int; // next row
                      mbr_0 |= t_5 & inv_sig
                    }
                    cwd_0 >>= 1 as libc::c_int;
                    cnt = cnt.wrapping_add(1)
                  }
                  sample_mask_0 = (sample_mask_0 as libc::c_uint).wrapping_add(sample_mask_0)
                    as OPJ_UINT32 as OPJ_UINT32;
                  if mbr_0 & sample_mask_0 != 0 {
                    assert!(*dp_0.offset(stride as isize) == 0 as libc::c_int as libc::c_uint);
                    if cwd_0 & 1 as libc::c_int as libc::c_uint != 0 {
                      let mut t_6: OPJ_UINT32 = 0;
                      new_sig |= sample_mask_0;
                      t_6 = (0x74 as libc::c_uint) << j_0 * 4 as libc::c_int;
                      mbr_0 |= t_6 & inv_sig
                    }
                    cwd_0 >>= 1 as libc::c_int;
                    cnt = cnt.wrapping_add(1)
                  }
                  sample_mask_0 = (sample_mask_0 as libc::c_uint).wrapping_add(sample_mask_0)
                    as OPJ_UINT32 as OPJ_UINT32;
                  if mbr_0 & sample_mask_0 != 0 {
                    assert!(
                      *dp_0.offset((2 as libc::c_int * stride) as isize)
                        == 0 as libc::c_int as libc::c_uint
                    );
                    if cwd_0 & 1 as libc::c_int as libc::c_uint != 0 {
                      let mut t_7: OPJ_UINT32 = 0;
                      new_sig |= sample_mask_0;
                      t_7 = (0xe8 as libc::c_uint) << j_0 * 4 as libc::c_int;
                      mbr_0 |= t_7 & inv_sig
                    }
                    cwd_0 >>= 1 as libc::c_int;
                    cnt = cnt.wrapping_add(1)
                  }
                  sample_mask_0 = (sample_mask_0 as libc::c_uint).wrapping_add(sample_mask_0)
                    as OPJ_UINT32 as OPJ_UINT32;
                  if mbr_0 & sample_mask_0 != 0 {
                    assert!(
                      *dp_0.offset((3 as libc::c_int * stride) as isize)
                        == 0 as libc::c_int as libc::c_uint
                    );
                    if cwd_0 & 1 as libc::c_int as libc::c_uint != 0 {
                      let mut t_8: OPJ_UINT32 = 0;
                      new_sig |= sample_mask_0;
                      t_8 = (0xc0 as libc::c_uint) << j_0 * 4 as libc::c_int;
                      mbr_0 |= t_8 & inv_sig
                    }
                    cwd_0 >>= 1 as libc::c_int;
                    cnt = cnt.wrapping_add(1)
                  }
                }
                //no samples need checking
                j_0 += 1;
                dp_0 = dp_0.offset(1);
                col_mask_0 <<= 4 as libc::c_int
              }
              //obtain signs here
              if new_sig & (0xffff as libc::c_uint) << 4 as libc::c_int * n != 0 {
                //if any
                let mut col_mask_1: OPJ_UINT32 = 0; // decoded samples address
                let mut j_1: OPJ_INT32 = 0; //mask to select a column
                let mut dp_1 = decoded_data.offset(((y - 8 as libc::c_int) * stride) as isize);
                dp_1 = dp_1.offset((i_3 + n) as isize);
                col_mask_1 = (0xf as libc::c_uint) << 4 as libc::c_int * n;
                j_1 = n;
                while j_1 < end {
                  let mut sample_mask_1: OPJ_UINT32 = 0;
                  if !(col_mask_1 & new_sig == 0 as libc::c_int as libc::c_uint) {
                    //scan 4 signs
                    sample_mask_1 = 0x11111111 as libc::c_uint & col_mask_1;
                    if new_sig & sample_mask_1 != 0 {
                      assert!(
                        *dp_1.offset(0 as libc::c_int as isize) == 0 as libc::c_int as libc::c_uint
                      );
                      //consume bit and increment number
                      //of consumed bits
                      let ref mut fresh30 = *dp_1.offset(0 as libc::c_int as isize); //put value and sign
                      *fresh30 |=
                        (cwd_0 & 1 as libc::c_int as libc::c_uint) << 31 as libc::c_int | val_15;
                      cwd_0 >>= 1 as libc::c_int;
                      cnt = cnt.wrapping_add(1)
                    }
                    sample_mask_1 = (sample_mask_1 as libc::c_uint).wrapping_add(sample_mask_1)
                      as OPJ_UINT32 as OPJ_UINT32;
                    if new_sig & sample_mask_1 != 0 {
                      assert!(*dp_1.offset(stride as isize) == 0 as libc::c_int as libc::c_uint);
                      let ref mut fresh31 = *dp_1.offset(stride as isize);
                      *fresh31 |=
                        (cwd_0 & 1 as libc::c_int as libc::c_uint) << 31 as libc::c_int | val_15;
                      cwd_0 >>= 1 as libc::c_int;
                      cnt = cnt.wrapping_add(1)
                    }
                    sample_mask_1 = (sample_mask_1 as libc::c_uint).wrapping_add(sample_mask_1)
                      as OPJ_UINT32 as OPJ_UINT32;
                    if new_sig & sample_mask_1 != 0 {
                      assert!(
                        *dp_1.offset((2 as libc::c_int * stride) as isize)
                          == 0 as libc::c_int as libc::c_uint
                      );
                      let ref mut fresh32 = *dp_1.offset((2 as libc::c_int * stride) as isize);
                      *fresh32 |=
                        (cwd_0 & 1 as libc::c_int as libc::c_uint) << 31 as libc::c_int | val_15;
                      cwd_0 >>= 1 as libc::c_int;
                      cnt = cnt.wrapping_add(1)
                    }
                    sample_mask_1 = (sample_mask_1 as libc::c_uint).wrapping_add(sample_mask_1)
                      as OPJ_UINT32 as OPJ_UINT32;
                    if new_sig & sample_mask_1 != 0 {
                      assert!(
                        *dp_1.offset((3 as libc::c_int * stride) as isize)
                          == 0 as libc::c_int as libc::c_uint
                      );
                      let ref mut fresh33 = *dp_1.offset((3 as libc::c_int * stride) as isize);
                      *fresh33 |=
                        (cwd_0 & 1 as libc::c_int as libc::c_uint) << 31 as libc::c_int | val_15;
                      cwd_0 >>= 1 as libc::c_int;
                      cnt = cnt.wrapping_add(1)
                    }
                  }
                  //if non is significant
                  j_1 += 1; //consume the bits from bitstrm
                  dp_1 = dp_1.offset(1);
                  col_mask_1 <<= 4 as libc::c_int
                }
              }
              frwd_advance(&mut sigprop, cnt);
              cnt = 0 as libc::c_int as OPJ_UINT32;
              //update the next 8 columns
              if n == 4 as libc::c_int {
                //horizontally
                let mut t_9 = new_sig >> 28 as libc::c_int;
                t_9 |= (t_9 & 0xe as libc::c_int as libc::c_uint) >> 1 as libc::c_int
                  | (t_9 & 7 as libc::c_int as libc::c_uint) << 1 as libc::c_int;
                let ref mut fresh34 = *cur_mbr.offset(1 as libc::c_int as isize);
                *fresh34 |= t_9 & !*cur_sig_0.offset(1 as libc::c_int as isize)
              }
              n += 4 as libc::c_int
            }
          }
          //update the next stripe (vertically propagation)
          new_sig |= *cur_sig_0.offset(0 as libc::c_int as isize); //left and right neighbors
          ux = (new_sig & 0x88888888 as libc::c_uint) >> 3 as libc::c_int;
          tx = ux | ux << 4 as libc::c_int | ux >> 4 as libc::c_int;
          if i_3 > 0 as libc::c_int {
            let ref mut fresh35 = *nxt_mbr.offset(-(1 as libc::c_int) as isize);
            *fresh35 |= ux << 28 as libc::c_int & !*nxt_sig.offset(-(1 as libc::c_int) as isize)
          }
          let ref mut fresh36 = *nxt_mbr.offset(0 as libc::c_int as isize);
          *fresh36 |= tx & !*nxt_sig.offset(0 as libc::c_int as isize);
          let ref mut fresh37 = *nxt_mbr.offset(1 as libc::c_int as isize);
          *fresh37 |= ux >> 28 as libc::c_int & !*nxt_sig.offset(1 as libc::c_int as isize);
          i_3 += 8 as libc::c_int;
          cur_sig_0 = cur_sig_0.offset(1);
          cur_mbr = cur_mbr.offset(1);
          nxt_sig = nxt_sig.offset(1);
          nxt_mbr = nxt_mbr.offset(1)
        }
        //clear current sigma
        //mbr need not be cleared because it is overwritten
        cur_sig_0 = if y & 0x4 as libc::c_int != 0 {
          sigma2
        } else {
          sigma1
        };
        memset(
          cur_sig_0 as *mut libc::c_void,
          0 as libc::c_int,
          (((width as OPJ_UINT32).wrapping_add(7 as libc::c_uint) >> 3 as libc::c_int)
            .wrapping_add(1 as libc::c_uint)
            << 2 as libc::c_int) as libc::c_ulong,
        );
      }
    }
  }
  //terminating
  if num_passes > 1 as libc::c_int as libc::c_uint {
    let mut st: OPJ_INT32 = 0;
    let mut y_0: OPJ_INT32 = 0;
    if num_passes > 2 as libc::c_int as libc::c_uint
      && (height & 3 as libc::c_int == 1 as libc::c_int
        || height & 3 as libc::c_int == 2 as libc::c_int)
    {
      //do magref
      let mut cur_sig_1 = if height & 0x4 as libc::c_int != 0 {
        sigma2
      } else {
        sigma1
      }; //reversed
      let mut dpp_0 = decoded_data.offset(((height & 0xfffffc as libc::c_int) * stride) as isize);
      let mut half_0 = (1 as libc::c_uint) << p.wrapping_sub(2 as libc::c_int as libc::c_uint);
      let mut i_4: OPJ_INT32 = 0;
      i_4 = 0 as libc::c_int;
      while i_4 < width {
        let mut cwd_1 = rev_fetch_mrp(&mut magref);
        let fresh38 = cur_sig_1;
        cur_sig_1 = cur_sig_1.offset(1);
        let mut sig_1 = *fresh38;
        let mut col_mask_2 = 0xf as libc::c_int as OPJ_UINT32;
        let mut dp_2 = dpp_0.offset(i_4 as isize);
        if sig_1 != 0 {
          let mut j_2: libc::c_int = 0;
          j_2 = 0 as libc::c_int;
          while j_2 < 8 as libc::c_int {
            if sig_1 & col_mask_2 != 0 {
              let mut sample_mask_2 = 0x11111111 as libc::c_int as libc::c_uint & col_mask_2;
              if sig_1 & sample_mask_2 != 0 {
                let mut sym_3: OPJ_UINT32 = 0;
                assert!(
                  *dp_2.offset(0 as libc::c_int as isize) != 0 as libc::c_int as libc::c_uint
                );
                sym_3 = cwd_1 & 1 as libc::c_int as libc::c_uint;
                let ref mut fresh39 = *dp_2.offset(0 as libc::c_int as isize);
                *fresh39 ^= (1 as libc::c_int as libc::c_uint).wrapping_sub(sym_3)
                  << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
                let ref mut fresh40 = *dp_2.offset(0 as libc::c_int as isize);
                *fresh40 |= half_0;
                cwd_1 >>= 1 as libc::c_int
              }
              sample_mask_2 = (sample_mask_2 as libc::c_uint).wrapping_add(sample_mask_2)
                as OPJ_UINT32 as OPJ_UINT32;
              if sig_1 & sample_mask_2 != 0 {
                let mut sym_4: OPJ_UINT32 = 0;
                assert!(*dp_2.offset(stride as isize) != 0 as libc::c_int as libc::c_uint);
                sym_4 = cwd_1 & 1 as libc::c_int as libc::c_uint;
                let ref mut fresh41 = *dp_2.offset(stride as isize);
                *fresh41 ^= (1 as libc::c_int as libc::c_uint).wrapping_sub(sym_4)
                  << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
                let ref mut fresh42 = *dp_2.offset(stride as isize);
                *fresh42 |= half_0;
                cwd_1 >>= 1 as libc::c_int
              }
              sample_mask_2 = (sample_mask_2 as libc::c_uint).wrapping_add(sample_mask_2)
                as OPJ_UINT32 as OPJ_UINT32;
              if sig_1 & sample_mask_2 != 0 {
                let mut sym_5: OPJ_UINT32 = 0;
                assert!(
                  *dp_2.offset((2 as libc::c_int * stride) as isize)
                    != 0 as libc::c_int as libc::c_uint
                );
                sym_5 = cwd_1 & 1 as libc::c_int as libc::c_uint;
                let ref mut fresh43 = *dp_2.offset((2 as libc::c_int * stride) as isize);
                *fresh43 ^= (1 as libc::c_int as libc::c_uint).wrapping_sub(sym_5)
                  << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
                let ref mut fresh44 = *dp_2.offset((2 as libc::c_int * stride) as isize);
                *fresh44 |= half_0;
                cwd_1 >>= 1 as libc::c_int
              }
              sample_mask_2 = (sample_mask_2 as libc::c_uint).wrapping_add(sample_mask_2)
                as OPJ_UINT32 as OPJ_UINT32;
              if sig_1 & sample_mask_2 != 0 {
                let mut sym_6: OPJ_UINT32 = 0;
                assert!(
                  *dp_2.offset((3 as libc::c_int * stride) as isize)
                    != 0 as libc::c_int as libc::c_uint
                );
                sym_6 = cwd_1 & 1 as libc::c_int as libc::c_uint;
                let ref mut fresh45 = *dp_2.offset((3 as libc::c_int * stride) as isize);
                *fresh45 ^= (1 as libc::c_int as libc::c_uint).wrapping_sub(sym_6)
                  << p.wrapping_sub(1 as libc::c_int as libc::c_uint);
                let ref mut fresh46 = *dp_2.offset((3 as libc::c_int * stride) as isize);
                *fresh46 |= half_0;
                cwd_1 >>= 1 as libc::c_int
              }
              sample_mask_2 = (sample_mask_2 as libc::c_uint).wrapping_add(sample_mask_2)
                as OPJ_UINT32 as OPJ_UINT32
            }
            col_mask_2 <<= 4 as libc::c_int;
            j_2 += 1;
            dp_2 = dp_2.offset(1)
          }
        }
        rev_advance_mrp(&mut magref, population_count(sig_1));
        i_4 += 8 as libc::c_int
      }
    }
    //do the last incomplete stripe
    // for cases of (height & 3) == 0 and 3
    // the should have been processed previously
    if height & 3 as libc::c_int == 1 as libc::c_int
      || height & 3 as libc::c_int == 2 as libc::c_int
    {
      //generate mbr of first stripe
      let mut sig_2 = if height & 0x4 as libc::c_int != 0 {
        sigma2
      } else {
        sigma1
      };
      let mut mbr_1 = if height & 0x4 as libc::c_int != 0 {
        mbr2
      } else {
        mbr1
      };
      //integrate horizontally
      let mut prev_1 = 0 as libc::c_int as OPJ_UINT32;
      let mut i_5: OPJ_INT32 = 0;
      i_5 = 0 as libc::c_int;
      while i_5 < width {
        let mut t_10: OPJ_UINT32 = 0;
        let mut z_0: OPJ_UINT32 = 0;
        *mbr_1.offset(0 as libc::c_int as isize) = *sig_2.offset(0 as libc::c_int as isize);
        //remove already significance samples
        let ref mut fresh47 = *mbr_1.offset(0 as libc::c_int as isize); //for first column, left neighbors
        *fresh47 |= prev_1 >> 28 as libc::c_int; //left neighbors
        let ref mut fresh48 = *mbr_1.offset(0 as libc::c_int as isize); //left neighbors
        *fresh48 |= *sig_2.offset(0 as libc::c_int as isize) << 4 as libc::c_int; //for last column, right neighbors
        let ref mut fresh49 = *mbr_1.offset(0 as libc::c_int as isize);
        *fresh49 |= *sig_2.offset(0 as libc::c_int as isize) >> 4 as libc::c_int;
        let ref mut fresh50 = *mbr_1.offset(0 as libc::c_int as isize);
        *fresh50 |= *sig_2.offset(1 as libc::c_int as isize) << 28 as libc::c_int;
        prev_1 = *sig_2.offset(0 as libc::c_int as isize);
        t_10 = *mbr_1.offset(0 as libc::c_int as isize);
        z_0 = *mbr_1.offset(0 as libc::c_int as isize);
        z_0 |= (t_10 & 0x77777777 as libc::c_int as libc::c_uint) << 1 as libc::c_int;
        z_0 |= (t_10 & 0xeeeeeeee as libc::c_uint) >> 1 as libc::c_int;
        *mbr_1.offset(0 as libc::c_int as isize) = z_0 & !*sig_2.offset(0 as libc::c_int as isize);
        i_5 += 8 as libc::c_int;
        mbr_1 = mbr_1.offset(1);
        sig_2 = sig_2.offset(1)
      }
    }
    st = height;
    st -= if height > 6 as libc::c_int {
      (height + 1 as libc::c_int & 3 as libc::c_int) + 3 as libc::c_int
    } else {
      height
    };
    y_0 = st;
    while y_0 < height {
      let mut cur_sig_2 = 0 as *mut OPJ_UINT32;
      let mut cur_mbr_0 = 0 as *mut OPJ_UINT32;
      let mut nxt_sig_0 = 0 as *mut OPJ_UINT32;
      let mut nxt_mbr_0 = 0 as *mut OPJ_UINT32;
      let mut val_16: OPJ_UINT32 = 0;
      let mut i_6: OPJ_INT32 = 0;
      //integrate vertically
      //above neighbors
      //below neighbors
      let mut pattern = 0xffffffff as libc::c_uint; // a pattern needed samples
      if height - y_0 == 3 as libc::c_int {
        pattern = 0x77777777 as libc::c_uint
      } else if height - y_0 == 2 as libc::c_int {
        pattern = 0x33333333 as libc::c_uint
      } else if height - y_0 == 1 as libc::c_int {
        pattern = 0x11111111 as libc::c_uint
      }
      //add membership from the next stripe, obtained above
      if height - y_0 > 4 as libc::c_int {
        let mut prev_2 = 0 as libc::c_int as OPJ_UINT32; //for first column, left neighbors
        let mut i_7: OPJ_INT32 = 0; //left neighbors
        cur_sig_2 = if y_0 & 0x4 as libc::c_int != 0 {
          sigma2
        } else {
          sigma1
        }; //left neighbors
        cur_mbr_0 = if y_0 & 0x4 as libc::c_int != 0 {
          mbr2
        } else {
          mbr1
        }; //for last column, right neighbors
        nxt_sig_0 = if y_0 & 0x4 as libc::c_int != 0 {
          sigma1
        } else {
          sigma2
        };
        i_7 = 0 as libc::c_int;
        while i_7 < width {
          let mut t_11 = *nxt_sig_0.offset(0 as libc::c_int as isize);
          t_11 |= prev_2 >> 28 as libc::c_int;
          t_11 |= *nxt_sig_0.offset(0 as libc::c_int as isize) << 4 as libc::c_int;
          t_11 |= *nxt_sig_0.offset(0 as libc::c_int as isize) >> 4 as libc::c_int;
          t_11 |= *nxt_sig_0.offset(1 as libc::c_int as isize) << 28 as libc::c_int;
          prev_2 = *nxt_sig_0.offset(0 as libc::c_int as isize);
          if stripe_causal == 0 {
            let ref mut fresh51 = *cur_mbr_0.offset(0 as libc::c_int as isize);
            *fresh51 |= (t_11 & 0x11111111 as libc::c_uint) << 3 as libc::c_int
          }
          //remove already significance samples
          let ref mut fresh52 = *cur_mbr_0.offset(0 as libc::c_int as isize);
          *fresh52 &= !*cur_sig_2.offset(0 as libc::c_int as isize);
          i_7 += 8 as libc::c_int;
          cur_mbr_0 = cur_mbr_0.offset(1);
          cur_sig_2 = cur_sig_2.offset(1);
          nxt_sig_0 = nxt_sig_0.offset(1)
        }
      }
      //find new locations and get signs
      cur_sig_2 = if y_0 & 0x4 as libc::c_int != 0 {
        sigma2
      } else {
        sigma1
      }; //skip unneeded samples
      cur_mbr_0 = if y_0 & 0x4 as libc::c_int != 0 {
        mbr2
      } else {
        mbr1
      };
      nxt_sig_0 = if y_0 & 0x4 as libc::c_int != 0 {
        sigma1
      } else {
        sigma2
      };
      nxt_mbr_0 = if y_0 & 0x4 as libc::c_int != 0 {
        mbr1
      } else {
        mbr2
      };
      val_16 = (3 as libc::c_uint) << p.wrapping_sub(2 as libc::c_int as libc::c_uint);
      i_6 = 0 as libc::c_int;
      while i_6 < width {
        let mut mbr_2 = *cur_mbr_0 & pattern;
        let mut new_sig_0 = 0 as libc::c_int as OPJ_UINT32;
        let mut ux_0: OPJ_UINT32 = 0;
        let mut tx_0: OPJ_UINT32 = 0;
        if mbr_2 != 0 {
          let mut n_0: OPJ_INT32 = 0;
          n_0 = 0 as libc::c_int;
          while n_0 < 8 as libc::c_int {
            let mut col_mask_3: OPJ_UINT32 = 0;
            let mut inv_sig_0: OPJ_UINT32 = 0;
            let mut end_0: OPJ_INT32 = 0;
            let mut j_3: OPJ_INT32 = 0;
            let mut cwd_2 = frwd_fetch(&mut sigprop);
            let mut cnt_0 = 0 as libc::c_int as OPJ_UINT32;
            let mut dp_3 = decoded_data.offset((y_0 * stride) as isize);
            dp_3 = dp_3.offset((i_6 + n_0) as isize);
            col_mask_3 = (0xf as libc::c_uint) << 4 as libc::c_int * n_0;
            inv_sig_0 = !*cur_sig_2.offset(0 as libc::c_int as isize) & pattern;
            end_0 = if n_0 + 4 as libc::c_int + i_6 < width {
              (n_0) + 4 as libc::c_int
            } else {
              (width) - i_6
            };
            j_3 = n_0;
            while j_3 < end_0 {
              let mut sample_mask_3: OPJ_UINT32 = 0;
              if !(col_mask_3 & mbr_2 == 0 as libc::c_int as libc::c_uint) {
                //scan 4 mbr
                sample_mask_3 = 0x11111111 as libc::c_uint & col_mask_3;
                if mbr_2 & sample_mask_3 != 0 {
                  assert!(
                    *dp_3.offset(0 as libc::c_int as isize) == 0 as libc::c_int as libc::c_uint
                  );
                  if cwd_2 & 1 as libc::c_int as libc::c_uint != 0 {
                    let mut t_12: OPJ_UINT32 = 0;
                    new_sig_0 |= sample_mask_3;
                    t_12 = (0x32 as libc::c_uint) << j_3 * 4 as libc::c_int;
                    mbr_2 |= t_12 & inv_sig_0
                  }
                  cwd_2 >>= 1 as libc::c_int;
                  cnt_0 = cnt_0.wrapping_add(1)
                }
                sample_mask_3 = (sample_mask_3 as libc::c_uint).wrapping_add(sample_mask_3)
                  as OPJ_UINT32 as OPJ_UINT32;
                if mbr_2 & sample_mask_3 != 0 {
                  assert!(*dp_3.offset(stride as isize) == 0 as libc::c_int as libc::c_uint);
                  if cwd_2 & 1 as libc::c_int as libc::c_uint != 0 {
                    let mut t_13: OPJ_UINT32 = 0;
                    new_sig_0 |= sample_mask_3;
                    t_13 = (0x74 as libc::c_uint) << j_3 * 4 as libc::c_int;
                    mbr_2 |= t_13 & inv_sig_0
                  }
                  cwd_2 >>= 1 as libc::c_int;
                  cnt_0 = cnt_0.wrapping_add(1)
                }
                sample_mask_3 = (sample_mask_3 as libc::c_uint).wrapping_add(sample_mask_3)
                  as OPJ_UINT32 as OPJ_UINT32;
                if mbr_2 & sample_mask_3 != 0 {
                  assert!(
                    *dp_3.offset((2 as libc::c_int * stride) as isize)
                      == 0 as libc::c_int as libc::c_uint
                  );
                  if cwd_2 & 1 as libc::c_int as libc::c_uint != 0 {
                    let mut t_14: OPJ_UINT32 = 0;
                    new_sig_0 |= sample_mask_3;
                    t_14 = (0xe8 as libc::c_uint) << j_3 * 4 as libc::c_int;
                    mbr_2 |= t_14 & inv_sig_0
                  }
                  cwd_2 >>= 1 as libc::c_int;
                  cnt_0 = cnt_0.wrapping_add(1)
                }
                sample_mask_3 = (sample_mask_3 as libc::c_uint).wrapping_add(sample_mask_3)
                  as OPJ_UINT32 as OPJ_UINT32;
                if mbr_2 & sample_mask_3 != 0 {
                  assert!(
                    *dp_3.offset((3 as libc::c_int * stride) as isize)
                      == 0 as libc::c_int as libc::c_uint
                  );
                  if cwd_2 & 1 as libc::c_int as libc::c_uint != 0 {
                    let mut t_15: OPJ_UINT32 = 0;
                    new_sig_0 |= sample_mask_3;
                    t_15 = (0xc0 as libc::c_uint) << j_3 * 4 as libc::c_int;
                    mbr_2 |= t_15 & inv_sig_0
                  }
                  cwd_2 >>= 1 as libc::c_int;
                  cnt_0 = cnt_0.wrapping_add(1)
                }
              }
              j_3 += 1;
              dp_3 = dp_3.offset(1);
              col_mask_3 <<= 4 as libc::c_int
            }
            //signs here
            if new_sig_0 & (0xffff as libc::c_uint) << 4 as libc::c_int * n_0 != 0 {
              let mut col_mask_4: OPJ_UINT32 = 0;
              let mut j_4: OPJ_INT32 = 0;
              let mut dp_4 = decoded_data.offset((y_0 * stride) as isize);
              dp_4 = dp_4.offset((i_6 + n_0) as isize);
              col_mask_4 = (0xf as libc::c_uint) << 4 as libc::c_int * n_0;
              j_4 = n_0;
              while j_4 < end_0 {
                let mut sample_mask_4: OPJ_UINT32 = 0;
                if !(col_mask_4 & new_sig_0 == 0 as libc::c_int as libc::c_uint) {
                  //scan 4 signs
                  sample_mask_4 = 0x11111111 as libc::c_uint & col_mask_4;
                  if new_sig_0 & sample_mask_4 != 0 {
                    assert!(
                      *dp_4.offset(0 as libc::c_int as isize) == 0 as libc::c_int as libc::c_uint
                    );
                    let ref mut fresh53 = *dp_4.offset(0 as libc::c_int as isize);
                    *fresh53 |=
                      (cwd_2 & 1 as libc::c_int as libc::c_uint) << 31 as libc::c_int | val_16;
                    cwd_2 >>= 1 as libc::c_int;
                    cnt_0 = cnt_0.wrapping_add(1)
                  }
                  sample_mask_4 = (sample_mask_4 as libc::c_uint).wrapping_add(sample_mask_4)
                    as OPJ_UINT32 as OPJ_UINT32;
                  if new_sig_0 & sample_mask_4 != 0 {
                    assert!(*dp_4.offset(stride as isize) == 0 as libc::c_int as libc::c_uint);
                    let ref mut fresh54 = *dp_4.offset(stride as isize);
                    *fresh54 |=
                      (cwd_2 & 1 as libc::c_int as libc::c_uint) << 31 as libc::c_int | val_16;
                    cwd_2 >>= 1 as libc::c_int;
                    cnt_0 = cnt_0.wrapping_add(1)
                  }
                  sample_mask_4 = (sample_mask_4 as libc::c_uint).wrapping_add(sample_mask_4)
                    as OPJ_UINT32 as OPJ_UINT32;
                  if new_sig_0 & sample_mask_4 != 0 {
                    assert!(
                      *dp_4.offset((2 as libc::c_int * stride) as isize)
                        == 0 as libc::c_int as libc::c_uint
                    );
                    let ref mut fresh55 = *dp_4.offset((2 as libc::c_int * stride) as isize);
                    *fresh55 |=
                      (cwd_2 & 1 as libc::c_int as libc::c_uint) << 31 as libc::c_int | val_16;
                    cwd_2 >>= 1 as libc::c_int;
                    cnt_0 = cnt_0.wrapping_add(1)
                  }
                  sample_mask_4 = (sample_mask_4 as libc::c_uint).wrapping_add(sample_mask_4)
                    as OPJ_UINT32 as OPJ_UINT32;
                  if new_sig_0 & sample_mask_4 != 0 {
                    assert!(
                      *dp_4.offset((3 as libc::c_int * stride) as isize)
                        == 0 as libc::c_int as libc::c_uint
                    );
                    let ref mut fresh56 = *dp_4.offset((3 as libc::c_int * stride) as isize);
                    *fresh56 |=
                      (cwd_2 & 1 as libc::c_int as libc::c_uint) << 31 as libc::c_int | val_16;
                    cwd_2 >>= 1 as libc::c_int;
                    cnt_0 = cnt_0.wrapping_add(1)
                  }
                }
                j_4 += 1;
                dp_4 = dp_4.offset(1);
                col_mask_4 <<= 4 as libc::c_int
              }
            }
            frwd_advance(&mut sigprop, cnt_0);
            cnt_0 = 0 as libc::c_int as OPJ_UINT32;
            //update next columns
            if n_0 == 4 as libc::c_int {
              //horizontally
              let mut t_16 = new_sig_0 >> 28 as libc::c_int;
              t_16 |= (t_16 & 0xe as libc::c_int as libc::c_uint) >> 1 as libc::c_int
                | (t_16 & 7 as libc::c_int as libc::c_uint) << 1 as libc::c_int;
              let ref mut fresh57 = *cur_mbr_0.offset(1 as libc::c_int as isize);
              *fresh57 |= t_16 & !*cur_sig_2.offset(1 as libc::c_int as isize)
            }
            n_0 += 4 as libc::c_int
          }
        }
        //propagate down (vertically propagation)
        new_sig_0 |= *cur_sig_2.offset(0 as libc::c_int as isize);
        ux_0 = (new_sig_0 & 0x88888888 as libc::c_uint) >> 3 as libc::c_int;
        tx_0 = ux_0 | ux_0 << 4 as libc::c_int | ux_0 >> 4 as libc::c_int;
        if i_6 > 0 as libc::c_int {
          let ref mut fresh58 = *nxt_mbr_0.offset(-(1 as libc::c_int) as isize);
          *fresh58 |= ux_0 << 28 as libc::c_int & !*nxt_sig_0.offset(-(1 as libc::c_int) as isize)
        }
        let ref mut fresh59 = *nxt_mbr_0.offset(0 as libc::c_int as isize);
        *fresh59 |= tx_0 & !*nxt_sig_0.offset(0 as libc::c_int as isize);
        let ref mut fresh60 = *nxt_mbr_0.offset(1 as libc::c_int as isize);
        *fresh60 |= ux_0 >> 28 as libc::c_int & !*nxt_sig_0.offset(1 as libc::c_int as isize);
        i_6 += 8 as libc::c_int;
        cur_sig_2 = cur_sig_2.offset(1);
        cur_mbr_0 = cur_mbr_0.offset(1);
        nxt_sig_0 = nxt_sig_0.offset(1);
        nxt_mbr_0 = nxt_mbr_0.offset(1)
      }
      y_0 += 4 as libc::c_int
    }
  }
  let mut x_1: OPJ_INT32 = 0;
  let mut y_1: OPJ_INT32 = 0;
  y_1 = 0 as libc::c_int;
  while y_1 < height {
    let mut sp_0 = (decoded_data as *mut OPJ_INT32).offset((y_1 * stride) as isize);
    x_1 = 0 as libc::c_int;
    while x_1 < width {
      let mut val_17 = *sp_0 & 0x7fffffff as libc::c_int;
      *sp_0 = if *sp_0 as OPJ_UINT32 & 0x80000000 as libc::c_uint != 0 {
        -val_17
      } else {
        val_17
      };
      x_1 += 1;
      sp_0 = sp_0.offset(1)
    }
    y_1 += 1
  }
  return 1 as libc::c_int;
}
