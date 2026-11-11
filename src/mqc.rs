// The copyright in this software is being made available under the 2-clauses
// BSD License, included below. This software may be subject to other third
// party and contributor rights, including patent rights, and no such rights
// are granted under this license.
//
// Copyright (c) 2002-2014, Universite catholique de Louvain (UCL), Belgium
// Copyright (c) 2002-2014, Professor Benoit Macq
// Copyright (c) 2001-2003, David Janssens
// Copyright (c) 2002-2003, Yannick Verschueren
// Copyright (c) 2003-2007, Francois-Olivier Devaux
// Copyright (c) 2003-2014, Antonin Descampe
// Copyright (c) 2005, Herve Drolon, FreeImage Team
// Copyright (c) 2008, Jerome Fimes, Communications & Systemes <jerome.fimes@c-s.fr>
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS `AS IS'
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED.  IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
//! MQ arithmetic coder: the context/state machine, byte-in and
//! byte-out with the 0xFF stuffing rules, bypass and termination.
//! See `doc/jpeg2000-design-spec.md` §3.7 (MQ arithmetic coder).
use super::consts::*;
use super::t1_luts::*;
use core::cell::Cell;

#[derive(Copy, Clone)]
pub struct MqcState {
  pub qeval: u32,
  // 0/1 bit values (u8 for parity with the C code)
  pub mps: u8,
  pub nmps: u8,
  pub nlps: u8,
}
pub struct Mqc {
  pub c: u32,
  pub a: u32,
  pub ct: u32,
  pub end_of_byte_stream_counter: u32,
  /// Owned byte buffer this coder reads from / writes to. The code-block data
  /// sits at `start..end`; `Cell` lets the shared-borrow byte-out path write.
  buf: Vec<Cell<u8>>,
  /// Index of the code-block's first byte within `base`. The encoder's buffer carries one
  /// sentinel byte in front of it (see `tcd_code_block_enc_allocate_data`), so `start` is
  /// 1 there and the cursor may legally sit one byte before the code-block data.
  start: usize,
  /// Index one past the code-block's last byte within `base` (decoder only). The
  /// `COMMON_CBLK_DATA_EXTRA` scratch bytes live at `end..base_len`.
  end: usize,
  /// Read/write cursor, as an index into `base`.
  cursor: usize,
  /// Value array of the 19 contexts, padded to 256 slots so a `u8` context
  /// number indexes it in-bounds without a branch.
  pub ctxs: [MqcState; 256],
  pub curctx: u8,
  pub lut_ctxno_zc_orient: &'static [u8; 512],
  pub backup: [u8; 2],
}
impl Default for Mqc {
  fn default() -> Self {
    Self {
      c: 0,
      a: 0,
      ct: 0,
      end_of_byte_stream_counter: 0,
      buf: Vec::new(),
      start: 0,
      end: 0,
      cursor: 0,
      ctxs: [MQC_STATES[0]; 256],
      curctx: 0,
      lut_ctxno_zc_orient: &LUT_CTXNO_ZC[0],
      backup: [0; 2],
    }
  }
}

impl Mqc {
  pub(crate) fn set_curctx(&mut self, ctxno: u8) {
    self.curctx = ctxno;
  }

  pub(crate) fn set_curctx_nlps(&mut self) {
    let curctx = self.curctx();
    self.ctxs[self.curctx as usize] = MQC_STATES[curctx.nlps as usize];
  }

  pub(crate) fn set_curctx_nmps(&mut self) {
    let curctx = self.curctx();
    self.ctxs[self.curctx as usize] = MQC_STATES[curctx.nmps as usize];
  }

  pub(crate) fn curctx(&self) -> MqcState {
    self.ctxs[self.curctx as usize]
  }

  /// Byte view over the owned buffer. `Cell` lets a shared borrow still write,
  /// which keeps the byte-out path free of `&mut` juggling.
  #[inline]
  fn buf(&self) -> &[Cell<u8>] {
    &self.buf[..]
  }

  /// Grows the owned buffer (if needed) to `len` zero bytes.
  pub(crate) fn grow_buf(&mut self, len: usize) {
    if self.buf.len() < len {
      self.buf.resize(len, Cell::new(0));
    }
  }

  /// Public read view of the owned buffer.
  pub(crate) fn buf_ref(&self) -> &[Cell<u8>] {
    &self.buf[..]
  }

  /// Public mutable view of the owned buffer.
  pub(crate) fn buf_mut(&mut self) -> &mut [Cell<u8>] {
    &mut self.buf[..]
  }

  /// Points the coder at the code-block data running from index `start` to
  /// index `end` (within the owned buffer), with the cursor at `start`.
  fn attach(&mut self, start: usize, end: usize) {
    self.start = start;
    self.end = end;
    self.cursor = start;
  }

  pub(crate) fn write_byte(&mut self, b: u8) {
    self.buf()[self.cursor].set(b);
  }

  pub(crate) fn check_start(&self, offset: isize) -> bool {
    self.cursor as isize >= self.start as isize + offset
  }

  pub(crate) fn bp(&self) -> u8 {
    self.buf()[self.cursor].get()
  }

  pub(crate) fn bp_peek_offset(&self, offset: isize) -> u8 {
    self.buf()[self.cursor.wrapping_add_signed(offset)].get()
  }

  pub(crate) fn bp_peek(&self) -> u8 {
    self.bp_peek_offset(1)
  }

  pub(crate) fn inc_bp(&mut self) {
    self.bp_offset(1)
  }

  pub(crate) fn bp_offset(&mut self, offset: isize) {
    self.cursor = self.cursor.wrapping_add_signed(offset);
  }

  /// Bytes of the code-block still unread by the decoder, i.e. `end - cursor`.
  pub(crate) fn remaining(&self) -> isize {
    self.end as isize - self.cursor as isize
  }

  /// Bytes of the code-block consumed by the decoder so far.
  pub(crate) fn consumed(&self) -> isize {
    self.cursor as isize - self.start as isize
  }

  /// Total length of the code-block segment the decoder was handed.
  pub(crate) fn seg_len(&self) -> isize {
    self.end as isize - self.start as isize
  }

  pub(crate) fn backup_extra(&mut self) {
    let mut backup = [0_u8; 2];
    let buf = self.buf();
    // Backup extra bytes and replace with `0xff`
    for (idx, byte) in backup.iter_mut().enumerate() {
      let cell = &buf[self.end + idx];
      *byte = cell.get();
      cell.set(0xff);
    }
    self.backup = backup;
  }

  pub(crate) fn restore_extra(&mut self) {
    let backup = self.backup;
    let buf = self.buf();
    for (idx, byte) in backup.iter().enumerate() {
      buf[self.end + idx].set(*byte);
    }
  }
}

#[inline]
pub(crate) fn mqc_setcurctx(mqc: &mut Mqc, ctxno: u8) {
  mqc.set_curctx(ctxno);
}

/* For internal use of mqc_decode_macro() */
#[inline(always)]
fn mqc_mpsexchange_macro(d: &mut u32, mqc: &mut Mqc) {
  let curctx = mqc.curctx();
  if mqc.a < curctx.qeval {
    *d = (curctx.mps == 0) as u32;
    mqc.set_curctx_nlps();
  } else {
    *d = curctx.mps as u32;
    mqc.set_curctx_nmps();
  }
}

/* For internal use of mqc_decode_macro() */
#[inline(always)]
fn mqc_lpsexchange_macro(d: &mut u32, mqc: &mut Mqc) {
  let curctx = mqc.curctx();
  if mqc.a < curctx.qeval {
    mqc.a = curctx.qeval;
    *d = curctx.mps as u32;
    mqc.set_curctx_nmps();
  } else {
    mqc.a = curctx.qeval;
    *d = (curctx.mps == 0) as u32;
    mqc.set_curctx_nlps();
  }
}

/// Decode a symbol using raw-decoder. Cfr p.506 TAUBMAN
/// `mqc` — MQC handle
/// Returns the decoded symbol (0 or 1)
#[inline]
pub(crate) fn mqc_raw_decode(mut mqc: &mut Mqc) -> u32 {
  if mqc.ct == 0 {
    /* Given mqc_raw_init_dec() we know that at some point we will */
    /* have a 0xFF 0xFF artificial marker */
    if mqc.c == 0xff {
      if mqc.bp() > 0x8f {
        mqc.c = 0xff;
        mqc.ct = 8;
      } else {
        mqc.c = mqc.bp() as u32;
        mqc.inc_bp();
        mqc.ct = 7;
      }
    } else {
      mqc.c = mqc.bp() as u32;
      mqc.inc_bp();
      mqc.ct = 8;
    }
  }
  mqc.ct = mqc.ct.wrapping_sub(1);

  mqc.c >> mqc.ct & 0x1
}

#[inline(always)]
fn mqc_bytein_macro(mqc: &mut Mqc) {
  let mut l_c: u32 = 0;
  /* Given mqc_init_dec() we know that at some point we will */
  /* have a 0xFF 0xFF artificial marker */
  l_c = mqc.bp_peek() as u32;
  if mqc.bp() == 0xff {
    if l_c > 0x8f {
      mqc.c = mqc.c.wrapping_add(0xff00);
      mqc.ct = 8;
      mqc.end_of_byte_stream_counter = mqc.end_of_byte_stream_counter.wrapping_add(1);
    } else {
      mqc.inc_bp();
      mqc.c = mqc.c.wrapping_add(l_c << 9);
      mqc.ct = 7;
    }
  } else {
    mqc.inc_bp();
    mqc.c = mqc.c.wrapping_add(l_c << 8);
    mqc.ct = 8;
  }
}

#[inline]
pub(crate) fn mqc_bytein(mqc: &mut Mqc) {
  mqc_bytein_macro(mqc);
}

/* For internal use of mqc_decode_macro() */
#[inline(always)]
fn mqc_renormd_macro(mqc: &mut Mqc) {
  loop {
    if mqc.ct == 0 {
      mqc_bytein_macro(mqc);
    }
    mqc.a <<= 1;
    mqc.c <<= 1;
    mqc.ct = mqc.ct.wrapping_sub(1);
    if mqc.a >= 0x8000 {
      break;
    }
  }
}

#[inline(always)]
pub(crate) fn mqc_decode_macro(d: &mut u32, mqc: &mut Mqc) {
  /* Implements ISO 15444-1 C.3.2 Decoding a decision (DECODE) */
  /* Note: alternate "J.2 - Decoding an MPS or an LPS in the */
  /* software-conventions decoder" has been tried, but does not bring any */
  /* improvement. See https://github.com/uclouvain/openjpeg/issues/921 */
  mqc.a = mqc.a.wrapping_sub(mqc.curctx().qeval);
  if (mqc.c >> 16) < mqc.curctx().qeval {
    mqc_lpsexchange_macro(d, mqc);
    mqc_renormd_macro(mqc);
  } else {
    mqc.c = mqc.c.wrapping_sub(mqc.curctx().qeval << 16);
    if mqc.a & 0x8000 == 0 {
      mqc_mpsexchange_macro(d, mqc);
      mqc_renormd_macro(mqc);
    } else {
      *d = mqc.curctx().mps as u32
    }
  }
}

/// Renormalize mqc->a and mqc->c while encoding, so that mqc->a stays between 0x8000 and 0x10000
/// `mqc` — MQC handle
/// `a_` — value of mqc->a
/// `c_` — value of mqc->c_
/// `ct_` — value of mqc->ct_
#[inline(always)]
fn mqc_renorme_macro(mqc: &mut Mqc) {
  loop {
    mqc.a <<= 1;
    mqc.c <<= 1;
    mqc.ct = mqc.ct.wrapping_sub(1);
    if mqc.ct == 0 {
      mqc_byteout(mqc);
    }
    if mqc.a & 0x8000 != 0 {
      break;
    }
  }
}

#[inline(always)]
fn mqc_codemps_macro(mqc: &mut Mqc) {
  mqc.a = mqc.a.wrapping_sub(mqc.curctx().qeval);
  if mqc.a & 0x8000 == 0 {
    if mqc.a < mqc.curctx().qeval {
      mqc.a = mqc.curctx().qeval
    } else {
      mqc.c = mqc.c.wrapping_add(mqc.curctx().qeval);
    }
    mqc.set_curctx_nmps();
    mqc_renorme_macro(mqc);
  } else {
    mqc.c = mqc.c.wrapping_add(mqc.curctx().qeval);
  }
}

#[inline(always)]
fn mqc_codelps_macro(mqc: &mut Mqc) {
  mqc.a = mqc.a.wrapping_sub(mqc.curctx().qeval);
  if mqc.a < mqc.curctx().qeval {
    mqc.c = mqc.c.wrapping_add(mqc.curctx().qeval);
  } else {
    mqc.a = mqc.curctx().qeval
  }
  mqc.set_curctx_nlps();
  mqc_renorme_macro(mqc);
}

#[inline(always)]
pub(crate) fn mqc_encode_macro(mqc: &mut Mqc, d: u32) {
  if mqc.curctx().mps as u32 == d {
    mqc_codemps(mqc);
  } else {
    mqc_codelps(mqc);
  }
}

#[inline]
pub(crate) fn mqc_bypass_enc_macro(mqc: &mut Mqc, d: u32) {
  if mqc.ct == BYPASS_CT_INIT {
    mqc.ct = 8;
  }
  mqc.ct = mqc.ct.wrapping_sub(1);
  mqc.c = mqc.c.wrapping_add(d << mqc.ct);
  if mqc.ct == 0 {
    mqc.write_byte(mqc.c as u8);
    mqc.ct = 8;
    /* If the previous byte was 0xff, make sure that the next msb is 0 */
    if mqc.bp() == 0xff {
      mqc.ct = 7;
    }
    mqc.inc_bp();
    mqc.c = 0;
  }
}

//@}
//@}
/* <summary> */
/* This array defines all the possible states for a context. */
/* </summary> */
const MQC_STATES: [MqcState; 47 * 2] = [
  MqcState {
    qeval: 0x5601,
    mps: 0,
    nmps: 2,
    nlps: 3,
  },
  MqcState {
    qeval: 0x5601,
    mps: 1,
    nmps: 3,
    nlps: 2,
  },
  MqcState {
    qeval: 0x3401,
    mps: 0,
    nmps: 4,
    nlps: 12,
  },
  MqcState {
    qeval: 0x3401,
    mps: 1,
    nmps: 5,
    nlps: 13,
  },
  MqcState {
    qeval: 0x1801,
    mps: 0,
    nmps: 6,
    nlps: 18,
  },
  MqcState {
    qeval: 0x1801,
    mps: 1,
    nmps: 7,
    nlps: 19,
  },
  MqcState {
    qeval: 0x0ac1,
    mps: 0,
    nmps: 8,
    nlps: 24,
  },
  MqcState {
    qeval: 0x0ac1,
    mps: 1,
    nmps: 9,
    nlps: 25,
  },
  MqcState {
    qeval: 0x0521,
    mps: 0,
    nmps: 10,
    nlps: 58,
  },
  MqcState {
    qeval: 0x0521,
    mps: 1,
    nmps: 11,
    nlps: 59,
  },
  MqcState {
    qeval: 0x0221,
    mps: 0,
    nmps: 76,
    nlps: 66,
  },
  MqcState {
    qeval: 0x0221,
    mps: 1,
    nmps: 77,
    nlps: 67,
  },
  MqcState {
    qeval: 0x5601,
    mps: 0,
    nmps: 14,
    nlps: 13,
  },
  MqcState {
    qeval: 0x5601,
    mps: 1,
    nmps: 15,
    nlps: 12,
  },
  MqcState {
    qeval: 0x5401,
    mps: 0,
    nmps: 16,
    nlps: 28,
  },
  MqcState {
    qeval: 0x5401,
    mps: 1,
    nmps: 17,
    nlps: 29,
  },
  MqcState {
    qeval: 0x4801,
    mps: 0,
    nmps: 18,
    nlps: 28,
  },
  MqcState {
    qeval: 0x4801,
    mps: 1,
    nmps: 19,
    nlps: 29,
  },
  MqcState {
    qeval: 0x3801,
    mps: 0,
    nmps: 20,
    nlps: 28,
  },
  MqcState {
    qeval: 0x3801,
    mps: 1,
    nmps: 21,
    nlps: 29,
  },
  MqcState {
    qeval: 0x3001,
    mps: 0,
    nmps: 22,
    nlps: 34,
  },
  MqcState {
    qeval: 0x3001,
    mps: 1,
    nmps: 23,
    nlps: 35,
  },
  MqcState {
    qeval: 0x2401,
    mps: 0,
    nmps: 24,
    nlps: 36,
  },
  MqcState {
    qeval: 0x2401,
    mps: 1,
    nmps: 25,
    nlps: 37,
  },
  MqcState {
    qeval: 0x1c01,
    mps: 0,
    nmps: 26,
    nlps: 40,
  },
  MqcState {
    qeval: 0x1c01,
    mps: 1,
    nmps: 27,
    nlps: 41,
  },
  MqcState {
    qeval: 0x1601,
    mps: 0,
    nmps: 58,
    nlps: 42,
  },
  MqcState {
    qeval: 0x1601,
    mps: 1,
    nmps: 59,
    nlps: 43,
  },
  MqcState {
    qeval: 0x5601,
    mps: 0,
    nmps: 30,
    nlps: 29,
  },
  MqcState {
    qeval: 0x5601,
    mps: 1,
    nmps: 31,
    nlps: 28,
  },
  MqcState {
    qeval: 0x5401,
    mps: 0,
    nmps: 32,
    nlps: 28,
  },
  MqcState {
    qeval: 0x5401,
    mps: 1,
    nmps: 33,
    nlps: 29,
  },
  MqcState {
    qeval: 0x5101,
    mps: 0,
    nmps: 34,
    nlps: 30,
  },
  MqcState {
    qeval: 0x5101,
    mps: 1,
    nmps: 35,
    nlps: 31,
  },
  MqcState {
    qeval: 0x4801,
    mps: 0,
    nmps: 36,
    nlps: 32,
  },
  MqcState {
    qeval: 0x4801,
    mps: 1,
    nmps: 37,
    nlps: 33,
  },
  MqcState {
    qeval: 0x3801,
    mps: 0,
    nmps: 38,
    nlps: 34,
  },
  MqcState {
    qeval: 0x3801,
    mps: 1,
    nmps: 39,
    nlps: 35,
  },
  MqcState {
    qeval: 0x3401,
    mps: 0,
    nmps: 40,
    nlps: 36,
  },
  MqcState {
    qeval: 0x3401,
    mps: 1,
    nmps: 41,
    nlps: 37,
  },
  MqcState {
    qeval: 0x3001,
    mps: 0,
    nmps: 42,
    nlps: 38,
  },
  MqcState {
    qeval: 0x3001,
    mps: 1,
    nmps: 43,
    nlps: 39,
  },
  MqcState {
    qeval: 0x2801,
    mps: 0,
    nmps: 44,
    nlps: 38,
  },
  MqcState {
    qeval: 0x2801,
    mps: 1,
    nmps: 45,
    nlps: 39,
  },
  MqcState {
    qeval: 0x2401,
    mps: 0,
    nmps: 46,
    nlps: 40,
  },
  MqcState {
    qeval: 0x2401,
    mps: 1,
    nmps: 47,
    nlps: 41,
  },
  MqcState {
    qeval: 0x2201,
    mps: 0,
    nmps: 48,
    nlps: 42,
  },
  MqcState {
    qeval: 0x2201,
    mps: 1,
    nmps: 49,
    nlps: 43,
  },
  MqcState {
    qeval: 0x1c01,
    mps: 0,
    nmps: 50,
    nlps: 44,
  },
  MqcState {
    qeval: 0x1c01,
    mps: 1,
    nmps: 51,
    nlps: 45,
  },
  MqcState {
    qeval: 0x1801,
    mps: 0,
    nmps: 52,
    nlps: 46,
  },
  MqcState {
    qeval: 0x1801,
    mps: 1,
    nmps: 53,
    nlps: 47,
  },
  MqcState {
    qeval: 0x1601,
    mps: 0,
    nmps: 54,
    nlps: 48,
  },
  MqcState {
    qeval: 0x1601,
    mps: 1,
    nmps: 55,
    nlps: 49,
  },
  MqcState {
    qeval: 0x1401,
    mps: 0,
    nmps: 56,
    nlps: 50,
  },
  MqcState {
    qeval: 0x1401,
    mps: 1,
    nmps: 57,
    nlps: 51,
  },
  MqcState {
    qeval: 0x1201,
    mps: 0,
    nmps: 58,
    nlps: 52,
  },
  MqcState {
    qeval: 0x1201,
    mps: 1,
    nmps: 59,
    nlps: 53,
  },
  MqcState {
    qeval: 0x1101,
    mps: 0,
    nmps: 60,
    nlps: 54,
  },
  MqcState {
    qeval: 0x1101,
    mps: 1,
    nmps: 61,
    nlps: 55,
  },
  MqcState {
    qeval: 0x0ac1,
    mps: 0,
    nmps: 62,
    nlps: 56,
  },
  MqcState {
    qeval: 0x0ac1,
    mps: 1,
    nmps: 63,
    nlps: 57,
  },
  MqcState {
    qeval: 0x09c1,
    mps: 0,
    nmps: 64,
    nlps: 58,
  },
  MqcState {
    qeval: 0x09c1,
    mps: 1,
    nmps: 65,
    nlps: 59,
  },
  MqcState {
    qeval: 0x08a1,
    mps: 0,
    nmps: 66,
    nlps: 60,
  },
  MqcState {
    qeval: 0x08a1,
    mps: 1,
    nmps: 67,
    nlps: 61,
  },
  MqcState {
    qeval: 0x0521,
    mps: 0,
    nmps: 68,
    nlps: 62,
  },
  MqcState {
    qeval: 0x0521,
    mps: 1,
    nmps: 69,
    nlps: 63,
  },
  MqcState {
    qeval: 0x0441,
    mps: 0,
    nmps: 70,
    nlps: 64,
  },
  MqcState {
    qeval: 0x0441,
    mps: 1,
    nmps: 71,
    nlps: 65,
  },
  MqcState {
    qeval: 0x02a1,
    mps: 0,
    nmps: 72,
    nlps: 66,
  },
  MqcState {
    qeval: 0x02a1,
    mps: 1,
    nmps: 73,
    nlps: 67,
  },
  MqcState {
    qeval: 0x0221,
    mps: 0,
    nmps: 74,
    nlps: 68,
  },
  MqcState {
    qeval: 0x0221,
    mps: 1,
    nmps: 75,
    nlps: 69,
  },
  MqcState {
    qeval: 0x0141,
    mps: 0,
    nmps: 76,
    nlps: 70,
  },
  MqcState {
    qeval: 0x0141,
    mps: 1,
    nmps: 77,
    nlps: 71,
  },
  MqcState {
    qeval: 0x0111,
    mps: 0,
    nmps: 78,
    nlps: 72,
  },
  MqcState {
    qeval: 0x0111,
    mps: 1,
    nmps: 79,
    nlps: 73,
  },
  MqcState {
    qeval: 0x0085,
    mps: 0,
    nmps: 80,
    nlps: 74,
  },
  MqcState {
    qeval: 0x0085,
    mps: 1,
    nmps: 81,
    nlps: 75,
  },
  MqcState {
    qeval: 0x0049,
    mps: 0,
    nmps: 82,
    nlps: 76,
  },
  MqcState {
    qeval: 0x0049,
    mps: 1,
    nmps: 83,
    nlps: 77,
  },
  MqcState {
    qeval: 0x0025,
    mps: 0,
    nmps: 84,
    nlps: 78,
  },
  MqcState {
    qeval: 0x0025,
    mps: 1,
    nmps: 85,
    nlps: 79,
  },
  MqcState {
    qeval: 0x0015,
    mps: 0,
    nmps: 86,
    nlps: 80,
  },
  MqcState {
    qeval: 0x0015,
    mps: 1,
    nmps: 87,
    nlps: 81,
  },
  MqcState {
    qeval: 0x0009,
    mps: 0,
    nmps: 88,
    nlps: 82,
  },
  MqcState {
    qeval: 0x0009,
    mps: 1,
    nmps: 89,
    nlps: 83,
  },
  MqcState {
    qeval: 0x0005,
    mps: 0,
    nmps: 90,
    nlps: 84,
  },
  MqcState {
    qeval: 0x0005,
    mps: 1,
    nmps: 91,
    nlps: 85,
  },
  MqcState {
    qeval: 0x0001,
    mps: 0,
    nmps: 90,
    nlps: 86,
  },
  MqcState {
    qeval: 0x0001,
    mps: 1,
    nmps: 91,
    nlps: 87,
  },
  MqcState {
    qeval: 0x5601,
    mps: 0,
    nmps: 92,
    nlps: 92,
  },
  MqcState {
    qeval: 0x5601,
    mps: 1,
    nmps: 93,
    nlps: 93,
  },
];

//
//

// MQC - Implementation of an MQ-Coder
//@{

// Local static functions
//@{

/*
==========================================================
   local functions
==========================================================
*/

/// Fill mqc->c with 1's for flushing
/// `mqc` — MQC handle
fn mqc_setbits(mut mqc: &mut Mqc) {
  let mut tempc = mqc.c.wrapping_add(mqc.a);
  mqc.c |= 0xffff;
  if mqc.c >= tempc {
    mqc.c = mqc.c.wrapping_sub(0x8000);
  }
}

/*
==========================================================
   MQ-Coder interface
==========================================================
*/

pub(crate) fn mqc_numbytes(mut mqc: &mut Mqc) -> u32 {
  mqc.consumed() as u32
}

/// `bp` is the code-block data buffer and `len` its usable length. The buffer is allocated with
/// one extra sentinel byte in front of `bp` (see `tcd_code_block_enc_allocate_data`), which
/// is what lets the cursor start one byte before the code-block data.
pub(crate) fn mqc_init_enc(mut mqc: &mut Mqc, len: u32) {
  /* To avoid the curctx pointer to be dangling, but not strictly */
  /* required as the current context is always set before encoding */
  mqc_setcurctx(mqc, 0);

  /* As specified in Figure C.10 - Initialization of the encoder */
  /* (C.2.8 Initialization of the encoder (INITENC)) */
  mqc.a = 0x8000;
  mqc.c = 0;
  let base_len = (len as usize).wrapping_add(1);
  mqc.grow_buf(base_len);
  /* Zero the sentinel byte the cursor reads through at bp - 1: the C's
  comment ("our initial fake byte is set at 0") relies on the codeblock
  allocation being zeroed, but the port's buffer is shared with the t1
  DECODE path (which leaves the last decoded chunk's bytes, including a
  possible 0xFF, in buf[0]) - re-establish the invariant here. */
  mqc.buf_mut()[0].set(0);
  mqc.attach(1, base_len);
  /* Yes, we point before the start of the buffer, but this is safe */
  /* given tcd_code_block_enc_allocate_data() */
  mqc.bp_offset(-1);
  mqc.ct = 12;
  /* At this point we should test *(mqc->bp) against 0xFF, but this is not */
  /* necessary, as this is only used at the beginning of the code block */
  /* and our initial fake byte is set at 0 */
  debug_assert!(mqc.bp() != 0xff);

  mqc.end_of_byte_stream_counter = 0;
}

pub(crate) fn mqc_flush(mut mqc: &mut Mqc) {
  /* C.2.9 Termination of coding (FLUSH) */
  /* Figure C.11 – FLUSH procedure */
  mqc_setbits(mqc);
  mqc.c <<= mqc.ct;
  mqc_byteout(mqc);
  mqc.c <<= mqc.ct;
  mqc_byteout(mqc);

  /* It is forbidden that a coding pass ends with 0xff */
  if mqc.bp() != 0xff {
    /* Advance pointer so that mqc_numbytes() returns a valid value */
    mqc.inc_bp()
  }
}

pub(crate) fn mqc_bypass_init_enc(mut mqc: &mut Mqc) {
  /* This function is normally called after at least one mqc_flush() */
  /* which will have advance mqc->bp by at least 2 bytes beyond its */
  /* initial position */
  debug_assert!(mqc.check_start(0));
  mqc.c = 0;
  /* in theory we should initialize to 8, but use this special value */
  /* as a hint that mqc_bypass_enc() has never been called, so */
  /* as to avoid the 0xff 0x7f elimination trick in mqc_bypass_flush_enc() */
  /* to trigger when we don't have output any bit during this bypass sequence */
  /* Any value > 8 will do */
  mqc.ct = BYPASS_CT_INIT;
  /* Given that we are called after mqc_flush(), the previous byte */
  /* cannot be 0xff. */
  debug_assert!(mqc.bp_peek_offset(-1) != 0xff);
}

pub(crate) fn mqc_bypass_enc(mut mqc: &mut Mqc, d: u32) {
  mqc_bypass_enc_macro(mqc, d)
}

pub(crate) fn mqc_bypass_get_extra_bytes(mut mqc: &mut Mqc, mut erterm: bool) -> u32 {
  if mqc.ct < 7 || mqc.ct == 7 && (erterm || mqc.bp_peek_offset(-1) != 0xff) {
    1
  } else {
    0
  }
}

pub(crate) fn mqc_bypass_flush_enc(mut mqc: &mut Mqc, mut erterm: bool) {
  /* Is there any bit remaining to be flushed ? */
  /* If the last output byte is 0xff, we can discard it, unless */
  /* erterm is required (I'm not completely sure why in erterm */
  /* we must output 0xff 0x2a if the last byte was 0xff instead of */
  /* discarding it, but Kakadu requires it when decoding */
  /* in -fussy mode) */
  if mqc.ct < 7 || mqc.ct == 7 && (erterm || mqc.bp_peek_offset(-1) != 0xff) {
    let mut bit_value = 0i32;
    /* If so, fill the remaining lsbs with an alternating sequence of */
    /* 0,1,... */
    /* Note: it seems the standard only requires that for a ERTERM flush */
    /* and doesn't specify what to do for a regular BYPASS flush */
    while mqc.ct > 0 {
      mqc.ct = mqc.ct.wrapping_sub(1);
      mqc.c = mqc.c.wrapping_add((bit_value << mqc.ct) as u32);
      bit_value = 1i32.wrapping_sub(bit_value);
    }
    mqc.write_byte(mqc.c as u8);
    /* Advance pointer so that mqc_numbytes() returns a valid value */
    mqc.inc_bp()
  } else if mqc.ct == 7 && mqc.bp_peek_offset(-1) == 0xff {
    /* Discard last 0xff */
    debug_assert!(!erterm);
    mqc.bp_offset(-1)
  } else if mqc.ct == 8
    && !erterm
    && mqc.bp_peek_offset(-1) == 0x7f
    && mqc.bp_peek_offset(-2) == 0xff
  {
    /* Tiny optimization: discard terminating 0xff 0x7f since it is */
    /* interpreted as 0xff 0x7f [0xff 0xff] by the decoder, and given */
    /* the bit stuffing, in fact as 0xff 0xff [0xff ..] */
    /* Happens once on opj_compress -i ../MAPA.tif -o MAPA.j2k  -M 1 */
    mqc.bp_offset(-2)
  }

  debug_assert!(mqc.bp_peek_offset(-1) != 0xff);
}

pub(crate) fn mqc_reset_enc(mut mqc: &mut Mqc) {
  mqc_resetstates(mqc);
  mqc_setstate(mqc, T1_CTXNO_UNI, 0, 46);
  mqc_setstate(mqc, T1_CTXNO_AGG, 0, 3);
  mqc_setstate(mqc, T1_CTXNO_ZC, 0, 4);
}

pub(crate) fn mqc_restart_init_enc(mut mqc: &mut Mqc) {
  /* <Re-init part> */
  /* As specified in Figure C.10 - Initialization of the encoder */
  /* (C.2.8 Initialization of the encoder (INITENC)) */
  mqc.a = 0x8000;
  mqc.c = 0;
  mqc.ct = 12;
  /* This function is normally called after at least one mqc_flush() */
  /* which will have advance mqc->bp by at least 2 bytes beyond its */
  /* initial position */
  mqc.bp_offset(-1);

  debug_assert!(mqc.check_start(-1));
  debug_assert!(mqc.bp() != 0xff);
  if mqc.bp() == 0xff {
    mqc.ct = 13;
  }
}

pub(crate) fn mqc_erterm_enc(mut mqc: &mut Mqc) {
  let mut k = (11_i32).wrapping_sub(mqc.ct as i32).wrapping_add(1);
  while k > 0 {
    mqc.c <<= mqc.ct;
    mqc.ct = 0;
    mqc_byteout(mqc);
    k -= mqc.ct as i32
  }
  if mqc.bp() != 0xff {
    mqc_byteout(mqc);
  }
}

/// Encode the most probable symbol
/// `mqc` — MQC handle
#[inline]
fn mqc_codemps(mut mqc: &mut Mqc) {
  mqc_codemps_macro(mqc)
}

/// Encode the most least symbol
/// `mqc` — MQC handle
#[inline]
fn mqc_codelps(mut mqc: &mut Mqc) {
  mqc_codelps_macro(mqc)
}

/// Encode a symbol using the MQ-coder
/// `mqc` — MQC handle
/// `d` — The symbol to be encoded (0 or 1)
#[inline]
pub(crate) fn mqc_encode(mut mqc: &mut Mqc, mut d: u32) {
  mqc_encode_macro(mqc, d)
}

pub(crate) fn mqc_segmark_enc(mut mqc: &mut Mqc) {
  mqc_setcurctx(mqc, 18);

  for i in 1u32..5 {
    mqc_encode(mqc, i.wrapping_rem(2));
  }
}

fn mqc_init_dec_common(
  mut mqc: &mut Mqc,
  mut offset: usize,
  mut len: u32,
  mut extra_writable_bytes: u32,
) {
  debug_assert!(extra_writable_bytes >= COMMON_CBLK_DATA_EXTRA);
  let len = len as usize;
  debug_assert!(
    mqc.buf.len()
      >= offset
        .wrapping_add(len)
        .wrapping_add(extra_writable_bytes as usize)
  );
  mqc.attach(offset, offset + len);
  /* Insert an artificial 0xFF 0xFF marker at end of the code block */
  /* data so that the bytein routines stop on it. This saves us comparing */
  /* the bp and end pointers */
  /* But before inserting it, backup th bytes we will overwrite */
  mqc.backup_extra();
}

pub(crate) fn mqc_init_dec(
  mut mqc: &mut Mqc,
  mut offset: usize,
  mut len: u32,
  mut extra_writable_bytes: u32,
) {
  /* Implements ISO 15444-1 C.3.5 Initialization of the decoder (INITDEC) */
  /* Note: alternate "J.1 - Initialization of the software-conventions */
  /* decoder" has been tried, but does */
  /* not bring any improvement. */
  /* See https://github.com/uclouvain/openjpeg/issues/921 */
  mqc_init_dec_common(mqc, offset, len, extra_writable_bytes);
  mqc_setcurctx(mqc, 0);
  mqc.end_of_byte_stream_counter = 0;
  if len == 0 {
    mqc.c = 0xff << 16;
  } else {
    mqc.c = (mqc.bp() as u32) << 16;
  }
  mqc_bytein(mqc);
  mqc.c <<= 7;
  mqc.ct = mqc.ct.wrapping_sub(7);
  mqc.a = 0x8000;
}

pub(crate) fn mqc_raw_init_dec(mqc: &mut Mqc, offset: usize, len: u32, extra_writable_bytes: u32) {
  mqc_init_dec_common(mqc, offset, len, extra_writable_bytes);
  mqc.c = 0;
  mqc.ct = 0;
}

pub(crate) fn opq_mqc_finish_dec(mqc: &mut Mqc) {
  /* Restore the bytes overwritten by mqc_init_dec_common() */
  mqc.restore_extra();
}

pub(crate) fn mqc_resetstates(mqc: &mut Mqc) {
  for i in 0..19 {
    mqc.ctxs[i] = MQC_STATES[0];
  }
}

pub(crate) fn mqc_setstate(mqc: &mut Mqc, ctxno: u8, msb: u32, prob: i32) {
  mqc.ctxs[ctxno as usize] = MQC_STATES[msb.wrapping_add((prob << 1) as u32) as usize];
}

#[inline(always)]
pub(crate) fn mqc_byteout(mut mqc: &mut Mqc) {
  /* bp is initialized to start - 1 in mqc_init_enc() */
  /* but this is safe, see tcd_code_block_enc_allocate_data() */
  debug_assert!(mqc.check_start(-1));
  if mqc.bp() == 0xff {
    mqc.inc_bp();
    mqc.write_byte((mqc.c >> 20) as u8);
    mqc.c &= 0xfffff;
    mqc.ct = 7
  } else if mqc.c & 0x8000000 == 0 {
    mqc.inc_bp();
    mqc.write_byte((mqc.c >> 19) as u8);
    mqc.c &= 0x7ffff;
    mqc.ct = 8
  } else {
    mqc.write_byte(mqc.bp().wrapping_add(1));
    if mqc.bp() == 0xff {
      mqc.c &= 0x7ffffff;
      mqc.inc_bp();
      mqc.write_byte((mqc.c >> 20) as u8);
      mqc.c &= 0xfffff;
      mqc.ct = 7
    } else {
      mqc.inc_bp();
      mqc.write_byte((mqc.c >> 19) as u8);
      mqc.c &= 0x7ffff;
      mqc.ct = 8
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_coder() -> Mqc {
    let mut mqc = Mqc::default();
    mqc_resetstates(&mut mqc);
    mqc_setcurctx(&mut mqc, 0);
    mqc
  }

  /// Encodes a bit sequence into a fresh coder and decodes it back.
  #[test]
  fn encode_decode_round_trip() {
    let mut enc = make_coder();
    mqc_init_enc(&mut enc, 4096);
    let bits: Vec<u32> = (0..2000u32).map(|i| (i * 7919 + 13) % 2).collect();
    for &b in &bits {
      mqc_setcurctx(&mut enc, (b as u8) % 19);
      mqc_encode_macro(&mut enc, b);
    }
    mqc_flush(&mut enc);
    /* The coded bytes are buf[start..cursor]; buf[0] is the sentinel the
    first byteout clobbers (the C writes at data - 1). */
    let nbytes = enc.cursor.saturating_sub(enc.start);

    let mut dec = make_coder();
    dec.buf = enc.buf.clone();
    dec.buf.resize(
      enc.start + nbytes + COMMON_CBLK_DATA_EXTRA as usize,
      Cell::new(0),
    );
    mqc_init_dec(&mut dec, enc.start, nbytes as u32, COMMON_CBLK_DATA_EXTRA);
    let mut decoded = Vec::with_capacity(bits.len());
    for &b in &bits {
      mqc_setcurctx(&mut dec, (b as u8) % 19);
      let mut d = 0u32;
      mqc_decode_macro(&mut d, &mut dec);
      decoded.push(d);
    }
    assert_eq!(bits, decoded);
  }

  /// The encoder's sentinel byte (buf[0], the byte the cursor reads through
  /// at bp - 1) must be re-zeroed by mqc_init_enc: the buffer is shared
  /// with the t1 DECODE path, which leaves the last decoded chunk's bytes
  /// in it (a stale 0xFF used to trip the always-on assert below).
  #[test]
  fn init_enc_zeroes_sentinel() {
    let mut mqc = Mqc::default();
    mqc_resetstates(&mut mqc);
    // Simulate a decode that left a chunk starting with 0xFF in the buffer.
    mqc.grow_buf(64);
    mqc.buf_mut()[0].set(0xff);
    mqc.buf_mut()[1].set(0x7f);
    mqc_init_enc(&mut mqc, 63);
    assert_eq!(mqc.buf_ref()[0].get(), 0, "sentinel must be zeroed");
  }
}
