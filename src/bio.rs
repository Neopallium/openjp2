//! Individual bit input-output (the tier-2 bit-packing layer).
use super::openjpeg::*;

/// Individual bit input-output stream over a borrowed byte buffer.
pub(crate) struct Bio<'a> {
  data: &'a mut [u8],
  cursor: usize,
  buf: u32,
  ct: u32,
}

impl<'a> Bio<'a> {
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

  fn byteout(&mut self) -> OpResult {
    self.buf = self.buf << 8i32 & 0xffffu32;
    self.ct = if self.buf == 0xff00u32 { 7 } else { 8 };
    match self.data.get_mut(self.cursor) {
      Some(slot) => {
        *slot = (self.buf >> 8i32) as u8;
        self.cursor += 1;
        Ok(())
      }
      None => Err(()),
    }
  }

  fn bytein(&mut self) -> OpResult {
    self.buf = self.buf << 8i32 & 0xffffu32;
    self.ct = if self.buf == 0xff00u32 { 7 } else { 8 };
    match self.data.get(self.cursor) {
      Some(&byte) => {
        self.buf |= byte as u32;
        self.cursor += 1;
        Ok(())
      }
      None => Err(()),
    }
  }

  fn putbit(&mut self, b: u32) {
    if self.ct == 0 {
      /* MSD: why not check the return value of this function ? */
      let _ = self.byteout();
    }
    self.ct -= 1;
    self.buf |= b << self.ct;
  }

  fn getbit(&mut self) -> u32 {
    if self.ct == 0 {
      /* MSD: why not check the return value of this function ? */
      let _ = self.bytein();
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

pub(crate) fn bio_numbytes(bio: &Bio) -> isize {
  bio.numbytes() as isize
}

/* *
Write a bit
@param bio BIO handle
@param b Bit to write (0 or 1)
*/
pub(crate) fn bio_putbit(bio: &mut Bio, b: u32) {
  bio.putbit(b)
}

pub(crate) fn bio_write(bio: &mut Bio, v: u32, n: u32) {
  debug_assert!(n > 0u32 && n <= 32u32);
  let mut i = n as i32 - 1i32;
  while i >= 0i32 {
    bio.putbit(v >> i & 1u32);
    i -= 1
  }
}

pub(crate) fn bio_read(bio: &mut Bio, n: u32) -> u32 {
  debug_assert!(n > 0u32);
  let mut v = 0u32;
  let mut i = n as i32 - 1i32;
  while i >= 0i32 {
    /* can't overflow, getbit returns 0 or 1 */
    v |= bio.getbit() << i;
    i -= 1
  }
  v
}

pub(crate) fn bio_flush(bio: &mut Bio) -> OpResult {
  if bio.byteout().is_err() {
    return Err(());
  }
  if bio.ct == 7u32 && bio.byteout().is_err() {
    return Err(());
  }
  Ok(())
}

pub(crate) fn bio_inalign(bio: &mut Bio) -> OpResult {
  if bio.buf & 0xffu32 == 0xffu32 && bio.bytein().is_err() {
    return Err(());
  }
  bio.ct = 0;
  Ok(())
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
