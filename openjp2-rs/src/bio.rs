use super::openjpeg::*;

/// Individual bit input-output stream over a borrowed byte buffer.
pub(crate) struct opj_bio<'a> {
  data: &'a mut [u8],
  cursor: usize,
  buf: OPJ_UINT32,
  ct: OPJ_UINT32,
}
pub(crate) type opj_bio_t<'a> = opj_bio<'a>;

impl<'a> opj_bio<'a> {
  pub(crate) fn new_enc(data: &'a mut [u8]) -> Self {
    Self {
      data,
      cursor: 0,
      buf: 0,
      ct: 8,
    }
  }

  pub(crate) fn new_dec(data: &'a mut [u8]) -> Self {
    Self {
      data,
      cursor: 0,
      buf: 0,
      ct: 0,
    }
  }

  /// Number of bytes consumed from / written to the buffer so far.
  pub(crate) fn numbytes(&self) -> usize {
    self.cursor
  }

  fn byteout(&mut self) -> OPJ_BOOL {
    self.buf = self.buf << 8i32 & 0xffffu32;
    self.ct = if self.buf == 0xff00u32 { 7 } else { 8 };
    match self.data.get_mut(self.cursor) {
      Some(slot) => {
        *slot = (self.buf >> 8i32) as OPJ_BYTE;
        self.cursor += 1;
        1i32
      }
      None => 0i32,
    }
  }

  fn bytein(&mut self) -> OPJ_BOOL {
    self.buf = self.buf << 8i32 & 0xffffu32;
    self.ct = if self.buf == 0xff00u32 { 7 } else { 8 };
    match self.data.get(self.cursor) {
      Some(&byte) => {
        self.buf |= byte as OPJ_UINT32;
        self.cursor += 1;
        1i32
      }
      None => 0i32,
    }
  }

  fn putbit(&mut self, b: OPJ_UINT32) {
    if self.ct == 0 {
      /* MSD: why not check the return value of this function ? */
      self.byteout();
    }
    self.ct -= 1;
    self.buf |= b << self.ct;
  }

  fn getbit(&mut self) -> OPJ_UINT32 {
    if self.ct == 0 {
      /* MSD: why not check the return value of this function ? */
      self.bytein();
    }
    self.ct -= 1;
    self.buf >> self.ct & 1u32
  }
}

/*
==========================================================
   Bit Input/Output interface
==========================================================
*/

pub(crate) fn opj_bio_numbytes(bio: &opj_bio) -> isize {
  bio.numbytes() as isize
}

/* *
Write a bit
@param bio BIO handle
@param b Bit to write (0 or 1)
*/
pub(crate) fn opj_bio_putbit(bio: &mut opj_bio, b: OPJ_UINT32) {
  bio.putbit(b)
}

pub(crate) fn opj_bio_write(bio: &mut opj_bio, v: OPJ_UINT32, n: OPJ_UINT32) {
  assert!(n > 0u32 && n <= 32u32);
  let mut i = n as OPJ_INT32 - 1i32;
  while i >= 0i32 {
    bio.putbit(v >> i & 1u32);
    i -= 1
  }
}

pub(crate) fn opj_bio_read(bio: &mut opj_bio, n: OPJ_UINT32) -> OPJ_UINT32 {
  assert!(n > 0u32);
  let mut v = 0u32;
  let mut i = n as OPJ_INT32 - 1i32;
  while i >= 0i32 {
    /* can't overflow, getbit returns 0 or 1 */
    v |= bio.getbit() << i;
    i -= 1
  }
  v
}

pub(crate) fn opj_bio_flush(bio: &mut opj_bio) -> OPJ_BOOL {
  if bio.byteout() == 0 {
    return 0i32;
  }
  if bio.ct == 7u32 && bio.byteout() == 0 {
    return 0i32;
  }
  1i32
}

pub(crate) fn opj_bio_inalign(bio: &mut opj_bio) -> OPJ_BOOL {
  if bio.buf & 0xffu32 == 0xffu32 && bio.bytein() == 0 {
    return 0i32;
  }
  bio.ct = 0;
  1i32
}

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
/* * @defgroup BIO BIO - Individual bit input-output stream */
