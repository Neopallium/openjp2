use ::libc;
use super::openjpeg::*;

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

#[inline]
unsafe extern "C" fn opj_mqc_bytein(mqc: *mut opj_mqc_t) {
  let mut l_c: OPJ_UINT32 = 0;
  l_c = *(*mqc).bp.offset(1 as libc::c_int as isize) as OPJ_UINT32;
  if *(*mqc).bp as libc::c_int == 0xff as libc::c_int {
    if l_c > 0x8f as libc::c_int as libc::c_uint {
      (*mqc).c = ((*mqc).c as libc::c_uint).wrapping_add(0xff00 as libc::c_int as libc::c_uint)
        as OPJ_UINT32 as OPJ_UINT32;
      (*mqc).ct = 8 as libc::c_int as OPJ_UINT32;
      (*mqc).end_of_byte_stream_counter = (*mqc).end_of_byte_stream_counter.wrapping_add(1)
    } else {
      (*mqc).bp = (*mqc).bp.offset(1);
      (*mqc).c = ((*mqc).c as libc::c_uint).wrapping_add(l_c << 9 as libc::c_int) as OPJ_UINT32
        as OPJ_UINT32;
      (*mqc).ct = 7 as libc::c_int as OPJ_UINT32
    }
  } else {
    (*mqc).bp = (*mqc).bp.offset(1);
    (*mqc).c =
      ((*mqc).c as libc::c_uint).wrapping_add(l_c << 8 as libc::c_int) as OPJ_UINT32 as OPJ_UINT32;
    (*mqc).ct = 8 as libc::c_int as OPJ_UINT32
  };
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
/* *
Fill mqc->c with 1's for flushing
@param mqc MQC handle
*/
/*
==========================================================
   local functions
==========================================================
*/
unsafe extern "C" fn opj_mqc_setbits(mut mqc: *mut opj_mqc_t) {
  let mut tempc = (*mqc).c.wrapping_add((*mqc).a);
  (*mqc).c |= 0xffff as libc::c_int as libc::c_uint;
  if (*mqc).c >= tempc {
    (*mqc).c = ((*mqc).c as libc::c_uint).wrapping_sub(0x8000 as libc::c_int as libc::c_uint)
      as OPJ_UINT32 as OPJ_UINT32
  };
}
/*
==========================================================
   MQ-Coder interface
==========================================================
*/
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_numbytes(mut mqc: *mut opj_mqc_t) -> OPJ_UINT32 {
  let diff = (*mqc).bp.wrapping_offset_from((*mqc).start) as libc::c_long;
  return diff as OPJ_UINT32;
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_init_enc(mut mqc: *mut opj_mqc_t, mut bp: *mut OPJ_BYTE) {
  /* To avoid the curctx pointer to be dangling, but not strictly */
  /* required as the current context is always set before encoding */
  (*mqc).curctx = &mut *(*mqc)
    .ctxs
    .as_mut_ptr()
    .offset(0 as libc::c_int as OPJ_UINT32 as isize)
    as *mut *const opj_mqc_state_t;
  /* As specified in Figure C.10 - Initialization of the encoder */
  /* (C.2.8 Initialization of the encoder (INITENC)) */
  (*mqc).a = 0x8000 as libc::c_int as OPJ_UINT32;
  (*mqc).c = 0 as libc::c_int as OPJ_UINT32;
  /* Yes, we point before the start of the buffer, but this is safe */
  /* given opj_tcd_code_block_enc_allocate_data() */
  (*mqc).bp = bp.offset(-(1 as libc::c_int as isize));
  (*mqc).ct = 12 as libc::c_int as OPJ_UINT32;
  /* At this point we should test *(mqc->bp) against 0xFF, but this is not */
  /* necessary, as this is only used at the beginning of the code block */
  /* and our initial fake byte is set at 0 */
  assert!(*(*mqc).bp as libc::c_int != 0xff as libc::c_int);
  (*mqc).start = bp;
  (*mqc).end_of_byte_stream_counter = 0 as libc::c_int as OPJ_UINT32;
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_flush(mut mqc: *mut opj_mqc_t) {
  /* C.2.9 Termination of coding (FLUSH) */
  /* Figure C.11 – FLUSH procedure */
  opj_mqc_setbits(mqc);
  (*mqc).c <<= (*mqc).ct;
  opj_mqc_byteout(mqc);
  (*mqc).c <<= (*mqc).ct;
  opj_mqc_byteout(mqc);
  /* It is forbidden that a coding pass ends with 0xff */
  if *(*mqc).bp as libc::c_int != 0xff as libc::c_int {
    /* Advance pointer so that opj_mqc_numbytes() returns a valid value */
    (*mqc).bp = (*mqc).bp.offset(1)
  };
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_bypass_init_enc(mut mqc: *mut opj_mqc_t) {
  /* This function is normally called after at least one opj_mqc_flush() */
  /* which will have advance mqc->bp by at least 2 bytes beyond its */
  /* initial position */
  assert!((*mqc).bp >= (*mqc).start);
  (*mqc).c = 0 as libc::c_int as OPJ_UINT32;
  /* in theory we should initialize to 8, but use this special value */
  /* as a hint that opj_mqc_bypass_enc() has never been called, so */
  /* as to avoid the 0xff 0x7f elimination trick in opj_mqc_bypass_flush_enc() */
  /* to trigger when we don't have output any bit during this bypass sequence */
  /* Any value > 8 will do */
  (*mqc).ct = 0xdeadbeef as libc::c_uint;
  /* Given that we are called after opj_mqc_flush(), the previous byte */
  /* cannot be 0xff. */
  assert!(*(*mqc).bp.offset(-(1 as libc::c_int) as isize) as libc::c_int != 0xff as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_bypass_enc(mut mqc: *mut opj_mqc_t, mut d: OPJ_UINT32) {
  if (*mqc).ct == 0xdeadbeef as libc::c_uint {
    (*mqc).ct = 8 as libc::c_int as OPJ_UINT32
  }
  (*mqc).ct = (*mqc).ct.wrapping_sub(1);
  (*mqc).c = (*mqc).c.wrapping_add(d << (*mqc).ct);
  if (*mqc).ct == 0 as libc::c_int as libc::c_uint {
    *(*mqc).bp = (*mqc).c as OPJ_BYTE;
    (*mqc).ct = 8 as libc::c_int as OPJ_UINT32;
    /* If the previous byte was 0xff, make sure that the next msb is 0 */
    if *(*mqc).bp as libc::c_int == 0xff as libc::c_int {
      (*mqc).ct = 7 as libc::c_int as OPJ_UINT32
    }
    (*mqc).bp = (*mqc).bp.offset(1);
    (*mqc).c = 0 as libc::c_int as OPJ_UINT32
  };
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_bypass_get_extra_bytes(
  mut mqc: *mut opj_mqc_t,
  mut erterm: OPJ_BOOL,
) -> OPJ_UINT32 {
  return if (*mqc).ct < 7 as libc::c_int as libc::c_uint
    || (*mqc).ct == 7 as libc::c_int as libc::c_uint
      && (erterm != 0
        || *(*mqc).bp.offset(-(1 as libc::c_int) as isize) as libc::c_int != 0xff as libc::c_int)
  {
    1 as libc::c_int
  } else {
    0 as libc::c_int
  } as OPJ_UINT32;
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_bypass_flush_enc(mut mqc: *mut opj_mqc_t, mut erterm: OPJ_BOOL) {
  /* Is there any bit remaining to be flushed ? */
  /* If the last output byte is 0xff, we can discard it, unless */
  /* erterm is required (I'm not completely sure why in erterm */
  /* we must output 0xff 0x2a if the last byte was 0xff instead of */
  /* discarding it, but Kakadu requires it when decoding */
  /* in -fussy mode) */
  if (*mqc).ct < 7 as libc::c_int as libc::c_uint
    || (*mqc).ct == 7 as libc::c_int as libc::c_uint
      && (erterm != 0
        || *(*mqc).bp.offset(-(1 as libc::c_int) as isize) as libc::c_int != 0xff as libc::c_int)
  {
    let mut bit_value = 0 as libc::c_int as OPJ_BYTE;
    /* If so, fill the remaining lsbs with an alternating sequence of */
    /* 0,1,... */
    /* Note: it seems the standard only requires that for a ERTERM flush */
    /* and doesn't specify what to do for a regular BYPASS flush */
    while (*mqc).ct > 0 as libc::c_int as libc::c_uint {
      (*mqc).ct = (*mqc).ct.wrapping_sub(1);
      (*mqc).c = ((*mqc).c as libc::c_uint)
        .wrapping_add(((bit_value as libc::c_int) << (*mqc).ct) as OPJ_UINT32)
        as OPJ_UINT32 as OPJ_UINT32;
      bit_value = (1 as libc::c_uint).wrapping_sub(bit_value as libc::c_uint) as OPJ_BYTE
    }
    *(*mqc).bp = (*mqc).c as OPJ_BYTE;
    /* Advance pointer so that opj_mqc_numbytes() returns a valid value */
    (*mqc).bp = (*mqc).bp.offset(1)
  } else if (*mqc).ct == 7 as libc::c_int as libc::c_uint
    && *(*mqc).bp.offset(-(1 as libc::c_int) as isize) as libc::c_int == 0xff as libc::c_int
  {
    /* Discard last 0xff */
    assert!(erterm == 0);
    (*mqc).bp = (*mqc).bp.offset(-1)
  } else if (*mqc).ct == 8 as libc::c_int as libc::c_uint
    && erterm == 0
    && *(*mqc).bp.offset(-(1 as libc::c_int) as isize) as libc::c_int == 0x7f as libc::c_int
    && *(*mqc).bp.offset(-(2 as libc::c_int) as isize) as libc::c_int == 0xff as libc::c_int
  {
    /* Tiny optimization: discard terminating 0xff 0x7f since it is */
    /* interpreted as 0xff 0x7f [0xff 0xff] by the decoder, and given */
    /* the bit stuffing, in fact as 0xff 0xff [0xff ..] */
    /* Happens once on opj_compress -i ../MAPA.tif -o MAPA.j2k  -M 1 */
    (*mqc).bp = (*mqc).bp.offset(-(2 as libc::c_int as isize))
  }
  assert!(*(*mqc).bp.offset(-(1 as libc::c_int) as isize) as libc::c_int != 0xff as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_reset_enc(mut mqc: *mut opj_mqc_t) {
  opj_mqc_resetstates(mqc);
  opj_mqc_setstate(
    mqc,
    (0 as libc::c_int + 9 as libc::c_int + 5 as libc::c_int + 3 as libc::c_int + 1 as libc::c_int)
      as OPJ_UINT32,
    0 as libc::c_int as OPJ_UINT32,
    46 as libc::c_int,
  );
  opj_mqc_setstate(
    mqc,
    (0 as libc::c_int + 9 as libc::c_int + 5 as libc::c_int + 3 as libc::c_int) as OPJ_UINT32,
    0 as libc::c_int as OPJ_UINT32,
    3 as libc::c_int,
  );
  opj_mqc_setstate(
    mqc,
    0 as libc::c_int as OPJ_UINT32,
    0 as libc::c_int as OPJ_UINT32,
    4 as libc::c_int,
  );
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_restart_init_enc(mut mqc: *mut opj_mqc_t) {
  /* <Re-init part> */
  /* As specified in Figure C.10 - Initialization of the encoder */
  /* (C.2.8 Initialization of the encoder (INITENC)) */
  (*mqc).a = 0x8000 as libc::c_int as OPJ_UINT32;
  (*mqc).c = 0 as libc::c_int as OPJ_UINT32;
  (*mqc).ct = 12 as libc::c_int as OPJ_UINT32;
  /* This function is normally called after at least one opj_mqc_flush() */
  /* which will have advance mqc->bp by at least 2 bytes beyond its */
  /* initial position */
  (*mqc).bp = (*mqc).bp.offset(-1);

  assert!((*mqc).bp >= (*mqc).start.offset(-(1 as libc::c_int as isize)));
  assert!(*(*mqc).bp as libc::c_int != 0xff as libc::c_int);
  if *(*mqc).bp as libc::c_int == 0xff as libc::c_int {
    (*mqc).ct = 13 as libc::c_int as OPJ_UINT32
  };
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_erterm_enc(mut mqc: *mut opj_mqc_t) {
  let mut k = (11 as libc::c_int as libc::c_uint)
    .wrapping_sub((*mqc).ct)
    .wrapping_add(1 as libc::c_int as libc::c_uint) as OPJ_INT32;
  while k > 0 as libc::c_int {
    (*mqc).c <<= (*mqc).ct;
    (*mqc).ct = 0 as libc::c_int as OPJ_UINT32;
    opj_mqc_byteout(mqc);
    k -= (*mqc).ct as OPJ_INT32
  }
  if *(*mqc).bp as libc::c_int != 0xff as libc::c_int {
    opj_mqc_byteout(mqc);
  };
}
/* *
Encode the most probable symbol
@param mqc MQC handle
*/
#[inline]
unsafe extern "C" fn opj_mqc_codemps(mut mqc: *mut opj_mqc_t) {
  (*mqc).a =
    ((*mqc).a as libc::c_uint).wrapping_sub((**(*mqc).curctx).qeval) as OPJ_UINT32 as OPJ_UINT32;
  if (*mqc).a & 0x8000 as libc::c_int as libc::c_uint == 0 as libc::c_int as libc::c_uint {
    if (*mqc).a < (**(*mqc).curctx).qeval {
      (*mqc).a = (**(*mqc).curctx).qeval
    } else {
      (*mqc).c =
        ((*mqc).c as libc::c_uint).wrapping_add((**(*mqc).curctx).qeval) as OPJ_UINT32 as OPJ_UINT32
    }
    *(*mqc).curctx = (**(*mqc).curctx).nmps;
    loop {
      (*mqc).a <<= 1 as libc::c_int;
      (*mqc).c <<= 1 as libc::c_int;
      (*mqc).ct = (*mqc).ct.wrapping_sub(1);
      if (*mqc).ct == 0 as libc::c_int as libc::c_uint {
        (*mqc).c = (*mqc).c;
        opj_mqc_byteout(mqc);
        (*mqc).c = (*mqc).c;
        (*mqc).ct = (*mqc).ct
      }
      if !((*mqc).a & 0x8000 as libc::c_int as libc::c_uint == 0 as libc::c_int as libc::c_uint) {
        break;
      }
    }
  } else {
    (*mqc).c =
      ((*mqc).c as libc::c_uint).wrapping_add((**(*mqc).curctx).qeval) as OPJ_UINT32 as OPJ_UINT32
  };
}
/* *
Encode the most least symbol
@param mqc MQC handle
*/
#[inline]
unsafe extern "C" fn opj_mqc_codelps(mut mqc: *mut opj_mqc_t) {
  (*mqc).a =
    ((*mqc).a as libc::c_uint).wrapping_sub((**(*mqc).curctx).qeval) as OPJ_UINT32 as OPJ_UINT32;
  if (*mqc).a < (**(*mqc).curctx).qeval {
    (*mqc).c =
      ((*mqc).c as libc::c_uint).wrapping_add((**(*mqc).curctx).qeval) as OPJ_UINT32 as OPJ_UINT32
  } else {
    (*mqc).a = (**(*mqc).curctx).qeval
  }
  *(*mqc).curctx = (**(*mqc).curctx).nlps;
  loop {
    (*mqc).a <<= 1 as libc::c_int;
    (*mqc).c <<= 1 as libc::c_int;
    (*mqc).ct = (*mqc).ct.wrapping_sub(1);
    if (*mqc).ct == 0 as libc::c_int as libc::c_uint {
      (*mqc).c = (*mqc).c;
      opj_mqc_byteout(mqc);
      (*mqc).c = (*mqc).c;
      (*mqc).ct = (*mqc).ct
    }
    if !((*mqc).a & 0x8000 as libc::c_int as libc::c_uint == 0 as libc::c_int as libc::c_uint) {
      break;
    }
  }
}
/* *
Encode a symbol using the MQ-coder
@param mqc MQC handle
@param d The symbol to be encoded (0 or 1)
*/
#[inline]
unsafe extern "C" fn opj_mqc_encode(mut mqc: *mut opj_mqc_t, mut d: OPJ_UINT32) {
  if (**(*mqc).curctx).mps == d {
    opj_mqc_codemps(mqc);
  } else {
    opj_mqc_codelps(mqc);
  };
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_segmark_enc(mut mqc: *mut opj_mqc_t) {
  let mut i: OPJ_UINT32 = 0;
  (*mqc).curctx = &mut *(*mqc)
    .ctxs
    .as_mut_ptr()
    .offset(18 as libc::c_int as OPJ_UINT32 as isize)
    as *mut *const opj_mqc_state_t;
  i = 1 as libc::c_int as OPJ_UINT32;
  while i < 5 as libc::c_int as libc::c_uint {
    opj_mqc_encode(mqc, i.wrapping_rem(2 as libc::c_int as libc::c_uint));
    i = i.wrapping_add(1)
  }
}
unsafe extern "C" fn opj_mqc_init_dec_common(
  mut mqc: *mut opj_mqc_t,
  mut bp: *mut OPJ_BYTE,
  mut len: OPJ_UINT32,
  mut extra_writable_bytes: OPJ_UINT32,
) {
  assert!(extra_writable_bytes >= 2 as libc::c_int as libc::c_uint);
  (*mqc).start = bp;
  (*mqc).end = bp.offset(len as isize);
  /* Insert an artificial 0xFF 0xFF marker at end of the code block */
  /* data so that the bytein routines stop on it. This saves us comparing */
  /* the bp and end pointers */
  /* But before inserting it, backup th bytes we will overwrite */
  memcpy(
    (*mqc).backup.as_mut_ptr() as *mut libc::c_void,
    (*mqc).end as *const libc::c_void,
    2 as libc::c_int as libc::c_ulong,
  );
  *(*mqc).end.offset(0 as libc::c_int as isize) = 0xff as libc::c_int as OPJ_BYTE;
  *(*mqc).end.offset(1 as libc::c_int as isize) = 0xff as libc::c_int as OPJ_BYTE;
  (*mqc).bp = bp;
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_init_dec(
  mut mqc: *mut opj_mqc_t,
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
  (*mqc).curctx = &mut *(*mqc)
    .ctxs
    .as_mut_ptr()
    .offset(0 as libc::c_int as OPJ_UINT32 as isize)
    as *mut *const opj_mqc_state_t;
  (*mqc).end_of_byte_stream_counter = 0 as libc::c_int as OPJ_UINT32;
  if len == 0 as libc::c_int as libc::c_uint {
    (*mqc).c = ((0xff as libc::c_int) << 16 as libc::c_int) as OPJ_UINT32
  } else {
    (*mqc).c = ((*(*mqc).bp as libc::c_int) << 16 as libc::c_int) as OPJ_UINT32
  }
  opj_mqc_bytein(mqc);
  (*mqc).c <<= 7 as libc::c_int;
  (*mqc).ct = ((*mqc).ct as libc::c_uint).wrapping_sub(7 as libc::c_int as libc::c_uint)
    as OPJ_UINT32 as OPJ_UINT32;
  (*mqc).a = 0x8000 as libc::c_int as OPJ_UINT32;
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_raw_init_dec(
  mut mqc: *mut opj_mqc_t,
  mut bp: *mut OPJ_BYTE,
  mut len: OPJ_UINT32,
  mut extra_writable_bytes: OPJ_UINT32,
) {
  opj_mqc_init_dec_common(mqc, bp, len, extra_writable_bytes);
  (*mqc).c = 0 as libc::c_int as OPJ_UINT32;
  (*mqc).ct = 0 as libc::c_int as OPJ_UINT32;
}
#[no_mangle]
pub unsafe extern "C" fn opq_mqc_finish_dec(mut mqc: *mut opj_mqc_t) {
  /* Restore the bytes overwritten by opj_mqc_init_dec_common() */
  memcpy(
    (*mqc).end as *mut libc::c_void,
    (*mqc).backup.as_mut_ptr() as *const libc::c_void,
    2 as libc::c_int as libc::c_ulong,
  );
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_resetstates(mut mqc: *mut opj_mqc_t) {
  let mut i: OPJ_UINT32 = 0;
  i = 0 as libc::c_int as OPJ_UINT32;
  while i < 19 as libc::c_int as libc::c_uint {
    (*mqc).ctxs[i as usize] = mqc_states.as_ptr();
    i = i.wrapping_add(1)
  }
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_setstate(
  mut mqc: *mut opj_mqc_t,
  mut ctxno: OPJ_UINT32,
  mut msb: OPJ_UINT32,
  mut prob: OPJ_INT32,
) {
  (*mqc).ctxs[ctxno as usize] = &*mqc_states
    .as_ptr()
    .offset(msb.wrapping_add((prob << 1 as libc::c_int) as OPJ_UINT32) as isize)
    as *const opj_mqc_state_t;
}
#[no_mangle]
pub unsafe extern "C" fn opj_mqc_byteout(mut mqc: *mut opj_mqc_t) {
  /* bp is initialized to start - 1 in opj_mqc_init_enc() */
  /* but this is safe, see opj_tcd_code_block_enc_allocate_data() */
  assert!((*mqc).bp >= (*mqc).start.offset(-(1 as libc::c_int as isize)));
  if *(*mqc).bp as libc::c_int == 0xff as libc::c_int {
    (*mqc).bp = (*mqc).bp.offset(1);
    *(*mqc).bp = ((*mqc).c >> 20 as libc::c_int) as OPJ_BYTE;
    (*mqc).c &= 0xfffff as libc::c_int as libc::c_uint;
    (*mqc).ct = 7 as libc::c_int as OPJ_UINT32
  } else if (*mqc).c & 0x8000000 as libc::c_int as libc::c_uint == 0 as libc::c_int as libc::c_uint
  {
    (*mqc).bp = (*mqc).bp.offset(1);
    *(*mqc).bp = ((*mqc).c >> 19 as libc::c_int) as OPJ_BYTE;
    (*mqc).c &= 0x7ffff as libc::c_int as libc::c_uint;
    (*mqc).ct = 8 as libc::c_int as OPJ_UINT32
  } else {
    *(*mqc).bp = (*(*mqc).bp).wrapping_add(1);
    if *(*mqc).bp as libc::c_int == 0xff as libc::c_int {
      (*mqc).c &= 0x7ffffff as libc::c_int as libc::c_uint;
      (*mqc).bp = (*mqc).bp.offset(1);
      *(*mqc).bp = ((*mqc).c >> 20 as libc::c_int) as OPJ_BYTE;
      (*mqc).c &= 0xfffff as libc::c_int as libc::c_uint;
      (*mqc).ct = 7 as libc::c_int as OPJ_UINT32
    } else {
      (*mqc).bp = (*mqc).bp.offset(1);
      *(*mqc).bp = ((*mqc).c >> 19 as libc::c_int) as OPJ_BYTE;
      (*mqc).c &= 0x7ffff as libc::c_int as libc::c_uint;
      (*mqc).ct = 8 as libc::c_int as OPJ_UINT32
    }
  };
}
unsafe extern "C" fn run_static_initializers() {
  mqc_states = [
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(2 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(3 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(3 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(2 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3401 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(4 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(12 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3401 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(5 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(13 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1801 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(6 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(18 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1801 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(7 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(19 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0xac1 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(8 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(24 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0xac1 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(9 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(25 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x521 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(10 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(58 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x521 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(11 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(59 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x221 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(76 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(66 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x221 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(77 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(67 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(14 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(13 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(15 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(12 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5401 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(16 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(28 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5401 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(17 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(29 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x4801 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(18 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(28 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x4801 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(19 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(29 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3801 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(20 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(28 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3801 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(21 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(29 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3001 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(22 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(34 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3001 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(23 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(35 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2401 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(24 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(36 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2401 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(25 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(37 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1c01 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(26 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(40 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1c01 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(27 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(41 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1601 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(58 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(42 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1601 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(59 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(43 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(30 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(29 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(31 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(28 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5401 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(32 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(28 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5401 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(33 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(29 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5101 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(34 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(30 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5101 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(35 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(31 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x4801 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(36 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(32 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x4801 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(37 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(33 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3801 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(38 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(34 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3801 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(39 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(35 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3401 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(40 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(36 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3401 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(41 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(37 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3001 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(42 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(38 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x3001 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(43 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(39 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2801 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(44 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(38 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2801 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(45 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(39 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2401 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(46 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(40 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2401 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(47 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(41 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2201 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(48 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(42 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2201 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(49 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(43 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1c01 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(50 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(44 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1c01 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(51 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(45 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1801 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(52 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(46 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1801 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(53 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(47 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1601 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(54 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(48 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1601 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(55 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(49 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1401 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(56 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(50 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1401 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(57 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(51 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1201 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(58 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(52 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1201 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(59 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(53 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1101 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(60 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(54 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1101 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(61 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(55 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0xac1 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(62 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(56 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0xac1 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(63 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(57 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x9c1 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(64 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(58 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x9c1 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(65 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(59 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x8a1 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(66 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(60 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x8a1 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(67 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(61 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x521 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(68 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(62 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x521 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(69 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(63 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x441 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(70 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(64 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x441 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(71 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(65 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2a1 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(72 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(66 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x2a1 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(73 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(67 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x221 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(74 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(68 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x221 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(75 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(69 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x141 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(76 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(70 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x141 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(77 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(71 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x111 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(78 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(72 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x111 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(79 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(73 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x85 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(80 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(74 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x85 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(81 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(75 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x49 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(82 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(76 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x49 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(83 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(77 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x25 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(84 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(78 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x25 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(85 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(79 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x15 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(86 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(80 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x15 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(87 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(81 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x9 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(88 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(82 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x9 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(89 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(83 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(90 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(84 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(91 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(85 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(90 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(86 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x1 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(91 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(87 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601 as libc::c_int as OPJ_UINT32,
        mps: 0 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(92 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(92 as libc::c_int as isize) as *const opj_mqc_state_t,
      };
      init
    },
    {
      let mut init = opj_mqc_state {
        qeval: 0x5601 as libc::c_int as OPJ_UINT32,
        mps: 1 as libc::c_int as OPJ_UINT32,
        nmps: &*mqc_states.as_ptr().offset(93 as libc::c_int as isize) as *const opj_mqc_state_t,
        nlps: &*mqc_states.as_ptr().offset(93 as libc::c_int as isize) as *const opj_mqc_state_t,
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
