//! Sparse arrays: the block-cached per-band coefficient storage.
/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2017, IntoPix SA <contact@intopix.com>
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

use core::cell::Cell;

use crate::math::*;

/// Sample types a `SparseArray` region can be transferred to/from.
///
/// The array always stores raw 32-bit words, exactly like the C code, which memcpy's tile data
/// through it regardless of whether the codestream is reversible (`i32` coefficients) or
/// irreversible (`f32` coefficients). This trait makes that reinterpretation explicit and safe
/// instead of casting pointers.
pub(crate) trait SparseSample: Copy {
  fn from_word(word: i32) -> Self;
  fn to_word(self) -> i32;
}

impl SparseSample for i32 {
  #[inline]
  fn from_word(word: i32) -> Self {
    word
  }
  #[inline]
  fn to_word(self) -> i32 {
    self
  }
}

impl SparseSample for f32 {
  #[inline]
  fn from_word(word: i32) -> Self {
    f32::from_bits(word as u32)
  }
  #[inline]
  fn to_word(self) -> i32 {
    self.to_bits() as i32
  }
}

#[derive(Clone)]
pub struct SparseArray {
  pub width: u32,
  pub height: u32,
  pub block_width: u32,
  pub block_height: u32,
  pub block_count_hor: u32,
  pub block_count_ver: u32,
  pub blocks: Vec<Option<Vec<i32>>>,
}

impl SparseArray {
  pub fn new(width: u32, height: u32, block_width: u32, block_height: u32) -> Option<Self> {
    if width == 0 || height == 0 || block_width == 0 || block_height == 0 {
      return None;
    }

    /* The C's create-time per-block size contract (open-findings OF-30):
    block_width * block_height samples must fit the element count. */
    if block_width > u32::MAX / block_height / core::mem::size_of::<i32>() as u32 {
      return None;
    }

    let block_count_hor = uint_ceildiv(width, block_width);
    let block_count_ver = uint_ceildiv(height, block_height);

    if block_count_hor > u32::MAX / block_count_ver {
      return None;
    }

    let total_blocks = (block_count_hor * block_count_ver) as usize;
    Some(Self {
      width,
      height,
      block_width,
      block_height,
      block_count_hor,
      block_count_ver,
      blocks: vec![None; total_blocks],
    })
  }

  fn block_index(&self, x: u32, y: u32) -> usize {
    (y * self.block_count_hor + x) as usize
  }

  pub fn data_block(&self, x: u32, y: u32) -> Option<&[i32]> {
    self.blocks.get(self.block_index(x, y))?.as_deref()
  }

  pub fn data_block_mut(&mut self, x: u32, y: u32) -> Option<&mut [i32]> {
    let index = self.block_index(x, y);
    self.blocks.get_mut(index)?.as_deref_mut()
  }

  pub fn set_data_block(&mut self, x: u32, y: u32, value: Vec<i32>) {
    let index = self.block_index(x, y);
    self.blocks[index] = Some(value);
  }
}

pub(crate) fn is_region_valid(sa: &SparseArray, x0: u32, y0: u32, x1: u32, y1: u32) -> bool {
  !(x0 >= sa.width || x1 <= x0 || x1 > sa.width || y0 >= sa.height || y1 <= y0 || y1 > sa.height)
}

/// Number of `i32` slots a caller buffer must expose for a `(x1-x0) x (y1-y0)` region
/// laid out with the given strides. This is the index of the last touched element plus one.
fn region_extent(x0: u32, y0: u32, x1: u32, y1: u32, col_stride: u32, line_stride: u32) -> usize {
  if x1 <= x0 || y1 <= y0 {
    return 0;
  }
  let w = (x1 - x0) as usize;
  let h = (y1 - y0) as usize;
  (h - 1) * line_stride as usize + (w - 1) * col_stride as usize + 1
}

/// Reads a region into `dest`.
///
/// `dest` is a slice of `Cell`s because DWT scratch buffers are shared between the horizontal and
/// vertical views of a resolution level, which both hold them by shared reference.
pub(crate) fn sparse_array_read<T: SparseSample>(
  sa: &SparseArray,
  x0: u32,
  y0: u32,
  x1: u32,
  y1: u32,
  dest: &[Cell<T>],
  dest_col_stride: u32,
  dest_line_stride: u32,
  forgiving: bool,
) -> bool {
  if !is_region_valid(sa, x0, y0, x1, y1) {
    return forgiving;
  }

  let col_stride = dest_col_stride as usize;
  let line_stride = dest_line_stride as usize;
  let mut y = y0;
  let mut block_y = y0 / sa.block_height;
  while y < y1 {
    let y_incr = if y == y0 {
      sa.block_height
        .wrapping_sub(y0.wrapping_rem(sa.block_height))
    } else {
      sa.block_height
    };
    let block_y_offset = sa.block_height.wrapping_sub(y_incr);
    let y_incr = y_incr.min(y1.wrapping_sub(y));
    let mut block_x = x0.wrapping_div(sa.block_width);
    let mut x = x0;
    while x < x1 {
      let mut x_incr = if x == x0 {
        sa.block_width.wrapping_sub(x0.wrapping_rem(sa.block_width))
      } else {
        sa.block_width
      };
      let block_x_offset = sa.block_width.wrapping_sub(x_incr);
      x_incr = x_incr.min(x1.wrapping_sub(x));
      let mut dest_base = (y.wrapping_sub(y0) as usize)
        .wrapping_mul(line_stride)
        .wrapping_add((x.wrapping_sub(x0) as usize).wrapping_mul(col_stride));
      match sa.data_block(block_x, block_y) {
        Some(src_block) => {
          let mut src = src_block[((block_y_offset * sa.block_width) + block_x_offset) as usize..]
            .chunks(sa.block_width as usize);
          for _ in 0..y_incr {
            let src_row = src.next().unwrap();
            for k in 0..x_incr as usize {
              dest[dest_base + k * col_stride].set(T::from_word(src_row[k]));
            }
            dest_base += line_stride;
          }
        }
        None => {
          for _ in 0..y_incr {
            for k in 0..x_incr as usize {
              dest[dest_base + k * col_stride].set(T::from_word(0i32));
            }
            dest_base += line_stride;
          }
        }
      }
      block_x = block_x.wrapping_add(1);
      x = x.wrapping_add(x_incr)
    }
    block_y = block_y.wrapping_add(1);
    y = y.wrapping_add(y_incr)
  }
  true
}

/// Writes a region from `src`. See [`sparse_array_read`] for why `src` is a slice of `Cell`s.
pub(crate) fn sparse_array_write<T: SparseSample>(
  sa: &mut SparseArray,
  x0: u32,
  y0: u32,
  x1: u32,
  y1: u32,
  src: &[Cell<T>],
  src_col_stride: u32,
  src_line_stride: u32,
  forgiving: bool,
) -> bool {
  if !is_region_valid(sa, x0, y0, x1, y1) {
    return forgiving;
  }

  let col_stride = src_col_stride as usize;
  let line_stride = src_line_stride as usize;
  let mut y = y0;
  let mut block_y = y0 / sa.block_height;
  while y < y1 {
    let y_incr = if y == y0 {
      sa.block_height
        .wrapping_sub(y0.wrapping_rem(sa.block_height))
    } else {
      sa.block_height
    };
    let block_y_offset = sa.block_height.wrapping_sub(y_incr);
    let y_incr = y_incr.min(y1.wrapping_sub(y));
    let mut block_x = x0.wrapping_div(sa.block_width);
    let mut x = x0;
    while x < x1 {
      let mut x_incr = if x == x0 {
        sa.block_width.wrapping_sub(x0.wrapping_rem(sa.block_width))
      } else {
        sa.block_width
      };
      let block_x_offset = sa.block_width.wrapping_sub(x_incr);
      x_incr = x_incr.min(x1.wrapping_sub(x));
      let index = sa.block_index(block_x, block_y);
      if sa.blocks[index].is_none() {
        sa.blocks[index] = Some(vec![0i32; (sa.block_width * sa.block_height) as usize]);
      }
      if let Some(ref mut dest_block) = sa.blocks[index] {
        let mut dest = dest_block[((block_y_offset * sa.block_width) + block_x_offset) as usize..]
          .chunks_mut(sa.block_width as usize);
        let mut src_base = (y.wrapping_sub(y0) as usize)
          .wrapping_mul(line_stride)
          .wrapping_add((x.wrapping_sub(x0) as usize).wrapping_mul(col_stride));
        for _ in 0..y_incr {
          let dest_row = dest.next().unwrap();
          for k in 0..x_incr as usize {
            dest_row[k] = src[src_base + k * col_stride].get().to_word();
          }
          src_base += line_stride;
        }
      }
      block_x = block_x.wrapping_add(1);
      x = x.wrapping_add(x_incr)
    }
    block_y = block_y.wrapping_add(1);
    y = y.wrapping_add(y_incr)
  }
  true
}
