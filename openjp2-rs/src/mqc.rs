use super::openjpeg::*;
use ::libc;

extern "C" {

  fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_mqc_state {
  pub qeval: OPJ_UINT32,
  pub mps: OPJ_UINT32,
  pub nmps: *const opj_mqc_state,
  pub nlps: *const opj_mqc_state,
}
pub type opj_mqc_state_t = opj_mqc_state;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_mqc {
  pub c: OPJ_UINT32,
  pub a: OPJ_UINT32,
  pub ct: OPJ_UINT32,
  pub end_of_byte_stream_counter: OPJ_UINT32,
  pub bp: *mut OPJ_BYTE,
  pub start: *mut OPJ_BYTE,
  pub end: *mut OPJ_BYTE,
  pub ctxs: [*const opj_mqc_state_t; 19],
  pub curctx: *mut *const opj_mqc_state_t,
  pub lut_ctxno_zc_orient: *const OPJ_BYTE,
  pub backup: [OPJ_BYTE; 2],
}
pub type opj_mqc_t = opj_mqc;

impl opj_mqc {
  pub fn set_curctx(&mut self, ctxno: usize) {
    self.curctx = (&mut self.ctxs[ctxno]) as *mut *const opj_mqc_state_t;
  }

  pub fn set_curctx_nlps(&mut self) {
    unsafe {
      *self.curctx = (**self.curctx).nlps;
    }
  }

  pub fn set_curctx_nmps(&mut self) {
    unsafe {
      *self.curctx = (**self.curctx).nmps;
    }
  }

  pub fn curctx(&self) -> &'static opj_mqc_state_t {
    unsafe { &**(self.curctx) }
  }

  pub fn write_byte(&mut self, b: OPJ_BYTE) {
    unsafe {
      *self.bp = b;
    }
  }

  pub fn bp_mut(&mut self) -> &mut OPJ_BYTE {
    unsafe { &mut *self.bp }
  }

  pub fn check_start(&self, offset: isize) -> bool {
    self.bp >= unsafe { self.start.offset(offset) }
  }

  pub fn bp(&self) -> OPJ_BYTE {
    unsafe { *self.bp }
  }

  pub fn bp_peek_offset(&self, offset: isize) -> OPJ_BYTE {
    unsafe { *self.bp.offset(offset) }
  }

  pub fn bp_peek(&self) -> OPJ_BYTE {
    self.bp_peek_offset(1)
  }

  pub fn inc_bp(&mut self) {
    self.bp_offset(1)
  }

  pub fn bp_offset(&mut self, offset: isize) {
    unsafe {
      self.bp = self.bp.offset(offset)
    }
  }
}

#[inline]
pub fn opj_mqc_setcurctx(mqc: &mut opj_mqc_t, ctxno: usize) {
  mqc.set_curctx(ctxno);
}

/* For internal use of opj_mqc_decode_macro() */
fn opj_mqc_mpsexchange_macro(d: &mut OPJ_UINT32, mqc: &mut opj_mqc_t) {
  let curctx = mqc.curctx();
  if mqc.a < curctx.qeval {
    *d = (curctx.mps == 0) as u32;
    mqc.set_curctx_nlps();
  } else {
    *d = curctx.mps;
    mqc.set_curctx_nmps();
  }
}

/* For internal use of opj_mqc_decode_macro() */
fn opj_mqc_lpsexchange_macro(d: &mut OPJ_UINT32, mqc: &mut opj_mqc_t) {
  let curctx = mqc.curctx();
  if mqc.a < curctx.qeval {
    mqc.a = curctx.qeval;
    *d = curctx.mps;
    mqc.set_curctx_nmps();
  } else {
    mqc.a = curctx.qeval;
    *d = (curctx.mps == 0) as u32;
    mqc.set_curctx_nlps();
  }
}

/**
Decode a symbol using raw-decoder. Cfr p.506 TAUBMAN
@param mqc MQC handle
@return Returns the decoded symbol (0 or 1)
*/
#[inline]
pub fn opj_mqc_raw_decode(mut mqc: &mut opj_mqc_t) -> OPJ_UINT32 {
  if mqc.ct == 0 {
    /* Given opj_mqc_raw_init_dec() we know that at some point we will */
    /* have a 0xFF 0xFF artificial marker */
    if mqc.c == 0xff {
      if mqc.bp() > 0x8f {
        mqc.c = 0xff;
        mqc.ct = 8;
      } else {
        mqc.c = mqc.bp() as OPJ_UINT32;
        mqc.inc_bp();
        mqc.ct = 7;
      }
    } else {
      mqc.c = mqc.bp() as OPJ_UINT32;
      mqc.inc_bp();
      mqc.ct = 8;
    }
  }
  mqc.ct = mqc.ct.wrapping_sub(1);
  let d = mqc.c >> mqc.ct & 0x1;
  return d;
}

//#define opj_mqc_bytein_macro(mqc, c, ct) \
#[inline]
fn opj_mqc_bytein_macro(mqc: &mut opj_mqc_t) {
  let mut l_c: OPJ_UINT32 = 0;
  /* Given opj_mqc_init_dec() we know that at some point we will */
  /* have a 0xFF 0xFF artificial marker */
  l_c = mqc.bp_peek() as OPJ_UINT32;
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
fn opj_mqc_bytein(mqc: &mut opj_mqc_t) {
  opj_mqc_bytein_macro(mqc);
}

/* For internal use of opj_mqc_decode_macro() */
#[inline]
fn opj_mqc_renormd_macro(mqc: &mut opj_mqc_t) {
  loop {
    if mqc.ct == 0 {
      opj_mqc_bytein_macro(mqc);
    }
    mqc.a <<= 1;
    mqc.c <<= 1;
    mqc.ct = mqc.ct.wrapping_sub(1);
    if !(mqc.a < 0x8000) {
      break;
    }
  }
}

//#define opj_mqc_decode_macro(d, mqc, curctx, a, c, ct) \
#[inline]
pub fn opj_mqc_decode_macro(d: &mut OPJ_UINT32, mqc: &mut opj_mqc_t) {
  /* Implements ISO 15444-1 C.3.2 Decoding a decision (DECODE) */
  /* Note: alternate "J.2 - Decoding an MPS or an LPS in the */
  /* software-conventions decoder" has been tried, but does not bring any */
  /* improvement. See https://github.com/uclouvain/openjpeg/issues/921 */
  mqc.a = mqc.a.wrapping_sub(mqc.curctx().qeval);
  if (mqc.c >> 16) < mqc.curctx().qeval {
    opj_mqc_lpsexchange_macro(d, mqc);
    opj_mqc_renormd_macro(mqc);
  } else {
    mqc.c = mqc.c.wrapping_sub(mqc.curctx().qeval << 16);
    if mqc.a & 0x8000 == 0 {
      opj_mqc_mpsexchange_macro(d, mqc);
      opj_mqc_renormd_macro(mqc);
    } else {
      *d = mqc.curctx().mps
    }
  }
}

/**
Renormalize mqc->a and mqc->c while encoding, so that mqc->a stays between 0x8000 and 0x10000
@param mqc MQC handle
@param a_ value of mqc->a
@param c_ value of mqc->c_
@param ct_ value of mqc->ct_
*/
//#define opj_mqc_renorme_macro(mqc, a_, c_, ct_) \
#[inline]
fn opj_mqc_renorme_macro(mqc: &mut opj_mqc_t) {
  loop {
    mqc.a <<= 1;
    mqc.c <<= 1;
    mqc.ct = mqc.ct.wrapping_sub(1);
    if mqc.ct == 0 {
      mqc.c = mqc.c;
      opj_mqc_byteout(mqc);
      mqc.c = mqc.c;
      mqc.ct = mqc.ct
    }
    if !(mqc.a & 0x8000 == 0) {
      break;
    }
  }
}

//#define opj_mqc_codemps_macro(mqc, curctx, a, c, ct) \
#[inline]
fn opj_mqc_codemps_macro(mqc: &mut opj_mqc_t) {
  mqc.a = mqc.a.wrapping_sub(mqc.curctx().qeval);
  if mqc.a & 0x8000 == 0 {
    if mqc.a < mqc.curctx().qeval {
      mqc.a = mqc.curctx().qeval
    } else {
      mqc.c = mqc.c.wrapping_add(mqc.curctx().qeval);
    }
    mqc.set_curctx_nmps();
    opj_mqc_renorme_macro(mqc);
  } else {
    mqc.c = mqc.c.wrapping_add(mqc.curctx().qeval);
  };
}

//#define opj_mqc_codelps_macro(mqc, curctx, a, c, ct) \
#[inline]
fn opj_mqc_codelps_macro(mqc: &mut opj_mqc_t) {
  mqc.a = mqc.a.wrapping_sub(mqc.curctx().qeval);
  if mqc.a < mqc.curctx().qeval {
    mqc.c = mqc.c.wrapping_add(mqc.curctx().qeval);
  } else {
    mqc.a = mqc.curctx().qeval
  }
  mqc.set_curctx_nlps();
  opj_mqc_renorme_macro(mqc);
}

//#define opj_mqc_encode_macro(mqc, curctx, a, c, ct, d) \
#[inline]
fn opj_mqc_encode_macro(mqc: &mut opj_mqc_t, d: OPJ_UINT32) {
  if mqc.curctx().mps == d {
    opj_mqc_codemps(mqc);
  } else {
    opj_mqc_codelps(mqc);
  };
}

//#define opj_mqc_bypass_enc_macro(mqc, c, ct, d) \
#[inline]
fn opj_mqc_bypass_enc_macro(mqc: &mut opj_mqc_t, d: OPJ_UINT32) {
  if mqc.ct == 0xdeadbeef {
    mqc.ct = 8;
  }
  mqc.ct = mqc.ct.wrapping_sub(1);
  mqc.c = mqc.c.wrapping_add(d << mqc.ct);
  if mqc.ct == 0 {
    mqc.write_byte(mqc.c as OPJ_BYTE);
    mqc.ct = 8;
    /* If the previous byte was 0xff, make sure that the next msb is 0 */
    if mqc.bp() == 0xff {
      mqc.ct = 7;
    }
    mqc.inc_bp();
    mqc.c = 0;
  }
}

/*@}*/
/*@}*/
/* <summary> */
/* This array defines all the possible states for a context. */
/* </summary> */
// Initialized in run_static_initializers
static mut mqc_states: [opj_mqc_state_t; 94] = [opj_mqc_state_t {
  qeval: 0,
  mps: 0,
  nmps: 0 as *const opj_mqc_state,
  nlps: 0 as *const opj_mqc_state,
}; 94];
/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2002-2014, Universite catholique de Louvain (UCL), Belgium
 * Copyright (c) 2002-2014, Professor Benoit Macq
 * Copyright (c) 2001-2003, David Janssens
 * Copyright (c) 2002-2003, Yannick Verschueren
 * Copyright (c) 2003-2007, Francois-Olivier Devaux
 * Copyright (c) 2003-2014, Antonin Descampe
 * Copyright (c) 2005, Herve Drolon, FreeImage Team
 * Copyright (c) 2008, Jerome Fimes, Communications & Systemes <jerome.fimes@c-s.fr>
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS `AS IS'
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */

/* * @defgroup MQC MQC - Implementation of an MQ-Coder */
/*@{*/

/* * @name Local static functions */
/*@{*/

/*
==========================================================
   local functions
==========================================================
*/

/* *
Fill mqc->c with 1's for flushing
@param mqc MQC handle
*/
fn opj_mqc_setbits(mut mqc: &mut opj_mqc_t) {
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

#[no_mangle]
pub fn opj_mqc_numbytes(mut mqc: &mut opj_mqc_t) -> OPJ_UINT32 {
  let diff = mqc.bp.wrapping_offset_from(mqc.start);
  return diff as OPJ_UINT32;
}

#[no_mangle]
pub fn opj_mqc_init_enc(mut mqc: &mut opj_mqc_t, mut bp: *mut OPJ_BYTE) {
  /* To avoid the curctx pointer to be dangling, but not strictly */
  /* required as the current context is always set before encoding */
  opj_mqc_setcurctx(mqc, 0);

  /* As specified in Figure C.10 - Initialization of the encoder */
  /* (C.2.8 Initialization of the encoder (INITENC)) */
  mqc.a = 0x8000;
  mqc.c = 0;
  /* Yes, we point before the start of the buffer, but this is safe */
  /* given opj_tcd_code_block_enc_allocate_data() */
  mqc.bp = bp;
  mqc.bp_offset(-1);
  mqc.ct = 12;
  /* At this point we should test *(mqc->bp) against 0xFF, but this is not */
  /* necessary, as this is only used at the beginning of the code block */
  /* and our initial fake byte is set at 0 */
  assert!(mqc.bp() != 0xff);

  mqc.start = bp;
  mqc.end_of_byte_stream_counter = 0;
}

#[no_mangle]
pub fn opj_mqc_flush(mut mqc: &mut opj_mqc_t) {
  /* C.2.9 Termination of coding (FLUSH) */
  /* Figure C.11 – FLUSH procedure */
  opj_mqc_setbits(mqc);
  mqc.c <<= mqc.ct;
  opj_mqc_byteout(mqc);
  mqc.c <<= mqc.ct;
  opj_mqc_byteout(mqc);

  /* It is forbidden that a coding pass ends with 0xff */
  if mqc.bp() != 0xff {
    /* Advance pointer so that opj_mqc_numbytes() returns a valid value */
    mqc.inc_bp()
  }
}

#[no_mangle]
pub fn opj_mqc_bypass_init_enc(mut mqc: &mut opj_mqc_t) {
  /* This function is normally called after at least one opj_mqc_flush() */
  /* which will have advance mqc->bp by at least 2 bytes beyond its */
  /* initial position */
  assert!(mqc.check_start(0));
  mqc.c = 0;
  /* in theory we should initialize to 8, but use this special value */
  /* as a hint that opj_mqc_bypass_enc() has never been called, so */
  /* as to avoid the 0xff 0x7f elimination trick in opj_mqc_bypass_flush_enc() */
  /* to trigger when we don't have output any bit during this bypass sequence */
  /* Any value > 8 will do */
  mqc.ct = 0xdeadbeef;
  /* Given that we are called after opj_mqc_flush(), the previous byte */
  /* cannot be 0xff. */
  assert!(mqc.bp_peek_offset(-1) != 0xff);
}

#[no_mangle]
pub fn opj_mqc_bypass_enc(mut mqc: &mut opj_mqc_t, d: OPJ_UINT32) {
  opj_mqc_bypass_enc_macro(mqc, d)
}

#[no_mangle]
pub fn opj_mqc_bypass_get_extra_bytes(
  mut mqc: &mut opj_mqc_t,
  mut erterm: OPJ_BOOL,
) -> OPJ_UINT32 {
  if mqc.ct < 7 ||
     mqc.ct == 7 && (erterm != 0 || mqc.bp_peek_offset(-1) != 0xff) {
    1
  } else {
    0
  }
}

#[no_mangle]
pub fn opj_mqc_bypass_flush_enc(mut mqc: &mut opj_mqc_t, mut erterm: OPJ_BOOL) {
  /* Is there any bit remaining to be flushed ? */
  /* If the last output byte is 0xff, we can discard it, unless */
  /* erterm is required (I'm not completely sure why in erterm */
  /* we must output 0xff 0x2a if the last byte was 0xff instead of */
  /* discarding it, but Kakadu requires it when decoding */
  /* in -fussy mode) */
  if mqc.ct < 7 || mqc.ct == 7 && (erterm != 0 || mqc.bp_peek_offset(-1) != 0xff) {
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
    mqc.write_byte(mqc.c as OPJ_BYTE);
    /* Advance pointer so that opj_mqc_numbytes() returns a valid value */
    mqc.inc_bp()
  } else if mqc.ct == 7 && mqc.bp_peek_offset(-1) == 0xff {
    /* Discard last 0xff */
    assert!(erterm == 0);
    mqc.bp_offset(-1)
  } else if mqc.ct == 8 && erterm == 0 &&
            mqc.bp_peek_offset(-1) == 0x7f && mqc.bp_peek_offset(-2) == 0xff {
    /* Tiny optimization: discard terminating 0xff 0x7f since it is */
    /* interpreted as 0xff 0x7f [0xff 0xff] by the decoder, and given */
    /* the bit stuffing, in fact as 0xff 0xff [0xff ..] */
    /* Happens once on opj_compress -i ../MAPA.tif -o MAPA.j2k  -M 1 */
    mqc.bp_offset(-2)
  }

  assert!(mqc.bp_peek_offset(-1) != 0xff);
}

#[no_mangle]
pub fn opj_mqc_reset_enc(mut mqc: &mut opj_mqc_t) {
  opj_mqc_resetstates(mqc);
  opj_mqc_setstate(
    mqc,
    0 + 9 + 5 + 3 + 1,
    0,
    46,
  );
  opj_mqc_setstate(
    mqc,
    0 + 9 + 5 + 3,
    0,
    3,
  );
  opj_mqc_setstate(
    mqc,
    0,
    0,
    4,
  );
}

#[no_mangle]
pub fn opj_mqc_restart_init_enc(mut mqc: &mut opj_mqc_t) {
  /* <Re-init part> */
  /* As specified in Figure C.10 - Initialization of the encoder */
  /* (C.2.8 Initialization of the encoder (INITENC)) */
  mqc.a = 0x8000;
  mqc.c = 0;
  mqc.ct = 12;
  /* This function is normally called after at least one opj_mqc_flush() */
  /* which will have advance mqc->bp by at least 2 bytes beyond its */
  /* initial position */
  mqc.bp_offset(-1);

  assert!(mqc.check_start(-1));
  assert!(mqc.bp() != 0xff);
  if mqc.bp() == 0xff {
    mqc.ct = 13
  };
}

#[no_mangle]
pub fn opj_mqc_erterm_enc(mut mqc: &mut opj_mqc_t) {
  let mut k = (11_i32)
    .wrapping_sub(mqc.ct as i32)
    .wrapping_add(1);
  while k > 0 {
    mqc.c <<= mqc.ct;
    mqc.ct = 0;
    opj_mqc_byteout(mqc);
    k -= mqc.ct as OPJ_INT32
  }
  if mqc.bp() != 0xff {
    opj_mqc_byteout(mqc);
  };
}

/* *
Encode the most probable symbol
@param mqc MQC handle
*/
#[inline]
fn opj_mqc_codemps(mut mqc: &mut opj_mqc_t) {
  opj_mqc_codemps_macro(mqc)
}

/* *
Encode the most least symbol
@param mqc MQC handle
*/
#[inline]
fn opj_mqc_codelps(mut mqc: &mut opj_mqc_t) {
  opj_mqc_codelps_macro(mqc)
}

/* *
Encode a symbol using the MQ-coder
@param mqc MQC handle
@param d The symbol to be encoded (0 or 1)
*/
#[inline]
fn opj_mqc_encode(mut mqc: &mut opj_mqc_t, mut d: OPJ_UINT32) {
  opj_mqc_encode_macro(mqc, d)
}

#[no_mangle]
pub fn opj_mqc_segmark_enc(mut mqc: &mut opj_mqc_t) {
  opj_mqc_setcurctx(mqc, 18);

  for i in 1u32..5 {
    opj_mqc_encode(mqc, i.wrapping_rem(2));
  }
}

fn opj_mqc_init_dec_common(
  mut mqc: &mut opj_mqc_t,
  mut bp: *mut OPJ_BYTE,
  mut len: OPJ_UINT32,
  mut extra_writable_bytes: OPJ_UINT32,
) {
  assert!(extra_writable_bytes >= 2);
  unsafe {
    mqc.start = bp;
    mqc.end = bp.offset(len as isize);
    /* Insert an artificial 0xFF 0xFF marker at end of the code block */
    /* data so that the bytein routines stop on it. This saves us comparing */
    /* the bp and end pointers */
    /* But before inserting it, backup th bytes we will overwrite */
    memcpy(
      mqc.backup.as_mut_ptr() as *mut libc::c_void,
      mqc.end as *const libc::c_void,
      2,
    );
    *mqc.end.offset(0) = 0xff;
    *mqc.end.offset(1) = 0xff;
    mqc.bp = bp;
  }
}

#[no_mangle]
pub fn opj_mqc_init_dec(
  mut mqc: &mut opj_mqc_t,
  mut bp: *mut OPJ_BYTE,
  mut len: OPJ_UINT32,
  mut extra_writable_bytes: OPJ_UINT32,
) {
  /* Implements ISO 15444-1 C.3.5 Initialization of the decoder (INITDEC) */
  /* Note: alternate "J.1 - Initialization of the software-conventions */
  /* decoder" has been tried, but does */
  /* not bring any improvement. */
  /* See https://github.com/uclouvain/openjpeg/issues/921 */
  opj_mqc_init_dec_common(mqc, bp, len, extra_writable_bytes);
  opj_mqc_setcurctx(mqc, 0);
  mqc.end_of_byte_stream_counter = 0;
  if len == 0 {
    mqc.c = 0xff << 16;
  } else {
    mqc.c = (mqc.bp() as u32) << 16;
  }
  opj_mqc_bytein(mqc);
  mqc.c <<= 7;
  mqc.ct = mqc.ct.wrapping_sub(7);
  mqc.a = 0x8000;
}

#[no_mangle]
pub fn opj_mqc_raw_init_dec(
  mut mqc: &mut opj_mqc_t,
  mut bp: *mut OPJ_BYTE,
  mut len: OPJ_UINT32,
  mut extra_writable_bytes: OPJ_UINT32,
) {
  opj_mqc_init_dec_common(mqc, bp, len, extra_writable_bytes);
  mqc.c = 0;
  mqc.ct = 0;
}

#[no_mangle]
pub fn opq_mqc_finish_dec(mut mqc: &mut opj_mqc_t) {
  unsafe {
    /* Restore the bytes overwritten by opj_mqc_init_dec_common() */
    memcpy(
      mqc.end as *mut libc::c_void,
      mqc.backup.as_mut_ptr() as *const libc::c_void,
      2,
    );
  }
}

#[no_mangle]
pub fn opj_mqc_resetstates(mut mqc: &mut opj_mqc_t) {
  let mut i: OPJ_UINT32 = 0;
  i = 0;
  while i < 19 {
    mqc.ctxs[i as usize] = unsafe { mqc_states.as_ptr() };
    i = i.wrapping_add(1)
  }
}

#[no_mangle]
pub fn opj_mqc_setstate(
  mut mqc: &mut opj_mqc_t,
  mut ctxno: OPJ_UINT32,
  mut msb: OPJ_UINT32,
  mut prob: OPJ_INT32,
) {
  unsafe {
    mqc.ctxs[ctxno as usize] = &*mqc_states
      .as_ptr()
      .offset(msb.wrapping_add((prob << 1) as OPJ_UINT32) as isize)
      as *const opj_mqc_state_t;
  }
}

#[no_mangle]
pub fn opj_mqc_byteout(mut mqc: &mut opj_mqc_t) {
  /* bp is initialized to start - 1 in opj_mqc_init_enc() */
  /* but this is safe, see opj_tcd_code_block_enc_allocate_data() */
  assert!(mqc.check_start(-1));
  if mqc.bp() == 0xff {
    mqc.inc_bp();
    mqc.write_byte((mqc.c >> 20) as OPJ_BYTE);
    mqc.c &= 0xfffff;
    mqc.ct = 7
  } else if mqc.c & 0x8000000 == 0
  {
    mqc.inc_bp();
    mqc.write_byte((mqc.c >> 19) as OPJ_BYTE);
    mqc.c &= 0x7ffff;
    mqc.ct = 8
  } else {
    mqc.write_byte(mqc.bp().wrapping_add(1));
    if mqc.bp() == 0xff {
      mqc.c &= 0x7ffffff;
      mqc.inc_bp();
      mqc.write_byte((mqc.c >> 20) as OPJ_BYTE);
      mqc.c &= 0xfffff;
      mqc.ct = 7
    } else {
      mqc.inc_bp();
      mqc.write_byte((mqc.c >> 19) as OPJ_BYTE);
      mqc.c &= 0x7ffff;
      mqc.ct = 8
    }
  };
}
unsafe extern "C" fn run_static_initializers() {
  mqc_states = [
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(2) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(3) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(3) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(2) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3401,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(4) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(12) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3401,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(5) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(13) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1801,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(6) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(18) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1801,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(7) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(19) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0xac1,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(8) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(24) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0xac1,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(9) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(25) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x521,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(10) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(58) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x521,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(11) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(59) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x221,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(76) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(66) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x221,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(77) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(67) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(14) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(13) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(15) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(12) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5401,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(16) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(28) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5401,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(17) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(29) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x4801,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(18) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(28) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x4801,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(19) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(29) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3801,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(20) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(28) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3801,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(21) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(29) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3001,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(22) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(34) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3001,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(23) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(35) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2401,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(24) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(36) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2401,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(25) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(37) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1c01,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(26) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(40) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1c01,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(27) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(41) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1601,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(58) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(42) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1601,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(59) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(43) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(30) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(29) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(31) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(28) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5401,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(32) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(28) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5401,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(33) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(29) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5101,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(34) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(30) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5101,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(35) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(31) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x4801,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(36) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(32) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x4801,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(37) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(33) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3801,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(38) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(34) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3801,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(39) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(35) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3401,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(40) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(36) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3401,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(41) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(37) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3001,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(42) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(38) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3001,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(43) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(39) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2801,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(44) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(38) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2801,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(45) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(39) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2401,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(46) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(40) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2401,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(47) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(41) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2201,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(48) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(42) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2201,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(49) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(43) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1c01,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(50) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(44) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1c01,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(51) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(45) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1801,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(52) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(46) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1801,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(53) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(47) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1601,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(54) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(48) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1601,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(55) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(49) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1401,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(56) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(50) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1401,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(57) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(51) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1201,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(58) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(52) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1201,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(59) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(53) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1101,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(60) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(54) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1101,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(61) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(55) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0xac1,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(62) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(56) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0xac1,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(63) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(57) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x9c1,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(64) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(58) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x9c1,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(65) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(59) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x8a1,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(66) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(60) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x8a1,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(67) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(61) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x521,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(68) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(62) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x521,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(69) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(63) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x441,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(70) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(64) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x441,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(71) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(65) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2a1,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(72) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(66) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2a1,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(73) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(67) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x221,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(74) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(68) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x221,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(75) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(69) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x141,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(76) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(70) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x141,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(77) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(71) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x111,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(78) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(72) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x111,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(79) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(73) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x85,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(80) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(74) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x85,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(81) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(75) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x49,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(82) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(76) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x49,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(83) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(77) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x25,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(84) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(78) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x25,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(85) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(79) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x15,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(86) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(80) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x15,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(87) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(81) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x9,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(88) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(82) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x9,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(89) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(83) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(90) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(84) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(91) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(85) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(90) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(86) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(91) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(87) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601,
        mps: 0,
        nmps: &*mqc_states.as_ptr().offset(92) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(92) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601,
        mps: 1,
        nmps: &*mqc_states.as_ptr().offset(93) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(93) as *const opj_mqc_state_t,
      };
      init
    },
  ]
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];
