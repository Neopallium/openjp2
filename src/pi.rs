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
// Copyright (c) 2006-2007, Parvatha Elangovan
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
//! Packet iteration: the progression-order walks (LRCP/RLCP/RPCL/
//! PCRL/CPRL) that drive tier-2. See `doc/jpeg2000-design-spec.md` §3.8.
use super::consts::*;
use super::event::*;
use super::j2k::*;
use super::math::*;
use super::openjpeg::*;
use crate::image::CoreImage;
use core::cell::Cell;
use std::rc::Rc;

pub type T2_MODE = core::ffi::c_uint;
pub const FINAL_PASS: T2_MODE = 1;
pub const THRESH_CALC: T2_MODE = 0;
pub type J2K_T2_MODE = T2_MODE;

#[derive(Default, Clone)]
pub struct PiResolution {
  pub pdx: u32,
  pub pdy: u32,
  pub pw: u32,
  pub ph: u32,
}
#[derive(Default, Clone)]
pub struct PiComp {
  pub dx: u32,
  pub dy: u32,
  pub numresolutions: u32,
  pub resolutions: Vec<PiResolution>,
}
#[derive(Clone, Default)]
pub struct PiIterator {
  pub tp_on: u8,
  /* Every pi element for a tile shares this one allocation
  (pi_create_decode chains `include = (prev)->include.clone()`); the Rc
  refcount keeps it alive until the last element is dropped. All accesses
  bounds-check against include_size. */
  pub include: Rc<[Cell<i16>]>,
  pub include_size: u32,
  pub step_l: u32,
  pub step_r: u32,
  pub step_c: u32,
  pub step_p: u32,
  pub compno: u32,
  pub resno: u32,
  pub precno: u32,
  pub layno: u32,
  pub first: bool,
  pub poc: crate::types::Poc,
  pub numcomps: u32,
  pub comps: Vec<PiComp>,
  pub tx0: u32,
  pub ty0: u32,
  pub tx1: u32,
  pub ty1: u32,
  pub x: u32,
  pub y: u32,
  pub dx: u32,
  pub dy: u32,
}
#[inline]
fn uint_ceildivpow2(mut a: u32, mut b: u32) -> u32 {
  ((a as u64).wrapping_add(1_u64 << b).wrapping_sub(1u64) >> b) as u32
}

//
//
// PI - Implementation of a packet iterator
//@{
// Local static functions
//@{
//
// Get next packet in layer-resolution-component-precinct order.
// * `pi` — packet iterator to modify
// returns false if pi pointed to the last packet or else returns true
//
//@}
//@}
/*
==========================================================
   local functions
==========================================================
*/
fn pi_next_lrcp(pi: &mut PiIterator, manager: &mut EventMgr) -> OpResult {
  let mut current_block: u64;
  let mut index = 0_u32;
  if pi.poc.compno0 >= pi.numcomps || pi.poc.compno1 >= pi.numcomps.wrapping_add(1u32) {
    event_msg!(
      manager,
      EVT_ERROR,
      "pi_next_lrcp(): invalid compno0/compno1\n",
    );
    return Err(());
  }
  if !pi.first {
    current_block = 5634871135123216486;
  } else {
    pi.first = false;
    pi.layno = pi.poc.layno0;
    current_block = 2868539653012386629;
  }
  loop {
    match current_block {
      5634871135123216486 => {
        pi.precno = pi.precno.wrapping_add(1);
        current_block = 1109700713171191020;
      }
      _ => {
        if pi.layno >= pi.poc.layno1 {
          break;
        }
        pi.resno = pi.poc.resno0;
        current_block = 1856101646708284338;
      }
    }
    loop {
      match current_block {
        1856101646708284338 => {
          if pi.resno < pi.poc.resno1 {
            pi.compno = pi.poc.compno0
          } else {
            pi.layno = pi.layno.wrapping_add(1);
            current_block = 2868539653012386629;
            break;
          }
          current_block = 10048703153582371463;
        }
        _ => {
          if pi.precno < pi.poc.precno1 {
            index = pi
              .layno
              .wrapping_mul(pi.step_l)
              .wrapping_add(pi.resno.wrapping_mul(pi.step_r))
              .wrapping_add(pi.compno.wrapping_mul(pi.step_c))
              .wrapping_add(pi.precno.wrapping_mul(pi.step_p));
            /* Avoids index out of bounds access with */
            /* id_000098,sig_11,src_005411,op_havoc,rep_2 of */
            /* https://github.com/uclouvain/openjpeg/issues/938 */
            /* Not sure if this is the most clever fix. Perhaps */
            /* include should be resized when a POC arises, or */
            /* the POC should be rejected */
            if index >= pi.include_size {
              event_msg!(manager, EVT_ERROR, "Invalid access to pi->include",);
              return Err(());
            }
            if pi.include[index as usize].get() == 0 {
              pi.include[index as usize].set(1_i16);
              return Ok(());
            }
            current_block = 5634871135123216486;
            break;
          } else {
            current_block = 17860125682698302841;
          }
        }
      }
      loop {
        match current_block {
          17860125682698302841 => {
            pi.compno = pi.compno.wrapping_add(1);
            current_block = 10048703153582371463;
          }
          _ => {
            if pi.compno < pi.poc.compno1 {
              if pi.resno >= pi.comps[pi.compno as usize].numresolutions {
                current_block = 17860125682698302841;
                continue;
              }
              if pi.tp_on == 0 {
                pi.poc.precno1 = pi.comps[pi.compno as usize].resolutions[pi.resno as usize]
                  .pw
                  .wrapping_mul(pi.comps[pi.compno as usize].resolutions[pi.resno as usize].ph)
              }
              pi.precno = pi.poc.precno0;
              current_block = 1109700713171191020;
              break;
            } else {
              pi.resno = pi.resno.wrapping_add(1);
              current_block = 1856101646708284338;
              break;
            }
          }
        }
      }
    }
  }
  Err(())
}

/// Get next packet in resolution-layer-component-precinct order.
/// `pi` — packet iterator to modify
/// returns false if pi pointed to the last packet or else returns true
fn pi_next_rlcp(pi: &mut PiIterator, manager: &mut EventMgr) -> OpResult {
  let mut current_block: u64;
  let mut index = 0_u32;
  if pi.poc.compno0 >= pi.numcomps || pi.poc.compno1 >= pi.numcomps.wrapping_add(1u32) {
    event_msg!(
      manager,
      EVT_ERROR,
      "pi_next_rlcp(): invalid compno0/compno1\n",
    );
    return Err(());
  }
  if !pi.first {
    current_block = 5634871135123216486;
  } else {
    pi.first = false;
    pi.resno = pi.poc.resno0;
    current_block = 2868539653012386629;
  }
  loop {
    match current_block {
      5634871135123216486 => {
        pi.precno = pi.precno.wrapping_add(1);
        current_block = 1109700713171191020;
      }
      _ => {
        if pi.resno >= pi.poc.resno1 {
          break;
        }
        pi.layno = pi.poc.layno0;
        current_block = 1856101646708284338;
      }
    }
    loop {
      match current_block {
        1856101646708284338 => {
          if pi.layno < pi.poc.layno1 {
            pi.compno = pi.poc.compno0
          } else {
            pi.resno = pi.resno.wrapping_add(1);
            current_block = 2868539653012386629;
            break;
          }
          current_block = 10048703153582371463;
        }
        _ => {
          if pi.precno < pi.poc.precno1 {
            index = pi
              .layno
              .wrapping_mul(pi.step_l)
              .wrapping_add(pi.resno.wrapping_mul(pi.step_r))
              .wrapping_add(pi.compno.wrapping_mul(pi.step_c))
              .wrapping_add(pi.precno.wrapping_mul(pi.step_p));
            if index >= pi.include_size {
              event_msg!(manager, EVT_ERROR, "Invalid access to pi->include",);
              return Err(());
            }
            if pi.include[index as usize].get() == 0 {
              pi.include[index as usize].set(1_i16);
              return Ok(());
            }
            current_block = 5634871135123216486;
            break;
          } else {
            current_block = 17860125682698302841;
          }
        }
      }
      loop {
        match current_block {
          17860125682698302841 => {
            pi.compno = pi.compno.wrapping_add(1);
            current_block = 10048703153582371463;
          }
          _ => {
            if pi.compno < pi.poc.compno1 {
              if pi.resno >= pi.comps[pi.compno as usize].numresolutions {
                current_block = 17860125682698302841;
                continue;
              }
              if pi.tp_on == 0 {
                pi.poc.precno1 = pi.comps[pi.compno as usize].resolutions[pi.resno as usize]
                  .pw
                  .wrapping_mul(pi.comps[pi.compno as usize].resolutions[pi.resno as usize].ph)
              }
              pi.precno = pi.poc.precno0;
              current_block = 1109700713171191020;
              break;
            } else {
              pi.layno = pi.layno.wrapping_add(1);
              current_block = 1856101646708284338;
              break;
            }
          }
        }
      }
    }
  }
  Err(())
}

/// Get next packet in resolution-precinct-component-layer order.
/// `pi` — packet iterator to modify
/// returns false if pi pointed to the last packet or else returns true
fn pi_next_rpcl(pi: &mut PiIterator, manager: &mut EventMgr) -> OpResult {
  let mut levelno: u32 = 0;
  let mut trx0: u32 = 0;
  let mut try0: u32 = 0;
  let mut trx1: u32 = 0;
  let mut try1: u32 = 0;
  let mut rpx: u32 = 0;
  let mut rpy: u32 = 0;
  let mut prci: u32 = 0;
  let mut prcj: u32 = 0;
  let mut current_block: u64;
  let mut index = 0_u32;
  if pi.poc.compno0 >= pi.numcomps || pi.poc.compno1 >= pi.numcomps.wrapping_add(1u32) {
    event_msg!(
      manager,
      EVT_ERROR,
      "pi_next_rpcl(): invalid compno0/compno1\n",
    );
    return Err(());
  }
  if !pi.first {
    current_block = 129780949503461575;
  } else {
    let mut compno: u32 = 0;
    let mut resno: u32 = 0;
    pi.first = false;
    pi.dx = 0_u32;
    pi.dy = 0_u32;
    compno = 0_u32;
    while compno < pi.numcomps {
      resno = 0_u32;
      while resno < pi.comps[compno as usize].numresolutions {
        let mut dx: u32 = 0;
        let mut dy: u32 = 0;
        if pi.comps[compno as usize].resolutions[resno as usize]
          .pdx
          .wrapping_add(pi.comps[compno as usize].numresolutions)
          .wrapping_sub(1u32)
          .wrapping_sub(resno)
          < 32u32
          && pi.comps[compno as usize].dx
            <= (2147483647u32)
              .wrapping_mul(2u32)
              .wrapping_add(1u32)
              .wrapping_div(
                (1u32)
                  << pi.comps[compno as usize].resolutions[resno as usize]
                    .pdx
                    .wrapping_add(pi.comps[compno as usize].numresolutions)
                    .wrapping_sub(1u32)
                    .wrapping_sub(resno),
              )
        {
          dx = pi.comps[compno as usize].dx.wrapping_mul(
            (1u32)
              << pi.comps[compno as usize].resolutions[resno as usize]
                .pdx
                .wrapping_add(pi.comps[compno as usize].numresolutions)
                .wrapping_sub(1u32)
                .wrapping_sub(resno),
          );
          pi.dx = if pi.dx == 0 { dx } else { uint_min(pi.dx, dx) }
        }
        if pi.comps[compno as usize].resolutions[resno as usize]
          .pdy
          .wrapping_add(pi.comps[compno as usize].numresolutions)
          .wrapping_sub(1u32)
          .wrapping_sub(resno)
          < 32u32
          && pi.comps[compno as usize].dy
            <= (2147483647u32)
              .wrapping_mul(2u32)
              .wrapping_add(1u32)
              .wrapping_div(
                (1u32)
                  << pi.comps[compno as usize].resolutions[resno as usize]
                    .pdy
                    .wrapping_add(pi.comps[compno as usize].numresolutions)
                    .wrapping_sub(1u32)
                    .wrapping_sub(resno),
              )
        {
          dy = pi.comps[compno as usize].dy.wrapping_mul(
            (1u32)
              << pi.comps[compno as usize].resolutions[resno as usize]
                .pdy
                .wrapping_add(pi.comps[compno as usize].numresolutions)
                .wrapping_sub(1u32)
                .wrapping_sub(resno),
          );
          pi.dy = if pi.dy == 0 { dy } else { uint_min(pi.dy, dy) }
        }
        resno += 1;
      }
      compno += 1;
    }
    if pi.dx == 0u32 || pi.dy == 0u32 {
      return Err(());
    }
    if pi.tp_on == 0 {
      pi.poc.ty0 = pi.ty0;
      pi.poc.tx0 = pi.tx0;
      pi.poc.ty1 = pi.ty1;
      pi.poc.tx1 = pi.tx1
    }
    pi.resno = pi.poc.resno0;
    current_block = 11385396242402735691;
  }
  loop {
    match current_block {
      129780949503461575 => {
        pi.layno = pi.layno.wrapping_add(1);
        current_block = 2606304779496145856;
      }
      _ => {
        if pi.resno >= pi.poc.resno1 {
          break;
        }
        pi.y = pi.poc.ty0;
        current_block = 6450636197030046351;
      }
    }
    loop {
      match current_block {
        6450636197030046351 => {
          if pi.y < pi.poc.ty1 {
            pi.x = pi.poc.tx0
          } else {
            pi.resno = pi.resno.wrapping_add(1);
            current_block = 11385396242402735691;
            break;
          }
          current_block = 3123434771885419771;
        }
        _ => {
          if pi.layno < pi.poc.layno1 {
            index = pi
              .layno
              .wrapping_mul(pi.step_l)
              .wrapping_add(pi.resno.wrapping_mul(pi.step_r))
              .wrapping_add(pi.compno.wrapping_mul(pi.step_c))
              .wrapping_add(pi.precno.wrapping_mul(pi.step_p));
            if index >= pi.include_size {
              event_msg!(manager, EVT_ERROR, "Invalid access to pi->include",);
              return Err(());
            }
            if pi.include[index as usize].get() == 0 {
              pi.include[index as usize].set(1_i16);
              return Ok(());
            }
            current_block = 129780949503461575;
            break;
          } else {
            current_block = 10891380440665537214;
          }
        }
      }
      loop {
        match current_block {
          10891380440665537214 => pi.compno = pi.compno.wrapping_add(1),
          _ => {
            if pi.x < pi.poc.tx1 {
              pi.compno = pi.poc.compno0
            } else {
              pi.y = (pi.y as core::ffi::c_uint)
                .wrapping_add(pi.dy.wrapping_sub(pi.y.wrapping_rem(pi.dy)));
              current_block = 6450636197030046351;
              break;
            }
          }
        }
        if pi.compno < pi.poc.compno1 {
          levelno = 0;
          trx0 = 0;
          try0 = 0;
          trx1 = 0;
          try1 = 0;
          rpx = 0;
          rpy = 0;
          prci = 0;
          prcj = 0;
          if pi.resno >= pi.comps[pi.compno as usize].numresolutions {
            current_block = 10891380440665537214;
            continue;
          }
          levelno = pi.comps[pi.compno as usize].numresolutions - 1 - pi.resno;
          let dx = pi.comps[pi.compno as usize].dx as u64;
          let dy = pi.comps[pi.compno as usize].dy as u64;

          if ((dx << levelno as u64) >> levelno) != dx || ((dy << levelno as u64) >> levelno) != dy
          {
            current_block = 10891380440665537214;
            continue;
          }

          trx0 = uint64_ceildiv_res_uint32(pi.tx0 as u64, dx << levelno);
          try0 = uint64_ceildiv_res_uint32(pi.ty0 as u64, dy << levelno);
          trx1 = uint64_ceildiv_res_uint32(pi.tx1 as u64, dx << levelno);
          try1 = uint64_ceildiv_res_uint32(pi.ty1 as u64, dy << levelno);
          rpx = pi.comps[pi.compno as usize].resolutions[pi.resno as usize]
            .pdx
            .wrapping_add(levelno);
          rpy = pi.comps[pi.compno as usize].resolutions[pi.resno as usize]
            .pdy
            .wrapping_add(levelno);

          if ((dx << rpx) >> rpx) != dx || ((dy << rpy) >> rpy) != dy {
            current_block = 10891380440665537214;
            continue;
          }

          /* See ISO-15441. B.12.1.3 Resolution level-position-component-layer progression */
          if !(pi.y as u64).is_multiple_of(dy << rpy)
            && !(pi.y == pi.ty0 && !((try0 as u64) << levelno).is_multiple_of(1u64 << rpy))
          {
            current_block = 10891380440665537214;
            continue;
          }
          if !(pi.x as u64).is_multiple_of(dx << rpx)
            && !(pi.x == pi.tx0 && !((trx0 as u64) << levelno).is_multiple_of(1u64 << rpx))
          {
            current_block = 10891380440665537214;
            continue;
          }
          if pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pw == 0u32
            || pi.comps[pi.compno as usize].resolutions[pi.resno as usize].ph == 0u32
          {
            current_block = 10891380440665537214;
            continue;
          }
          if trx0 == trx1 || try0 == try1 {
            current_block = 10891380440665537214;
            continue;
          }
          prci = uint_floordivpow2(
            uint64_ceildiv_res_uint32(pi.x as u64, dx << levelno),
            pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pdx,
          ) - uint_floordivpow2(
            trx0,
            pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pdx,
          );
          prcj = uint_floordivpow2(
            uint64_ceildiv_res_uint32(pi.y as u64, dy << levelno),
            pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pdy,
          ) - uint_floordivpow2(
            try0,
            pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pdy,
          );
          pi.precno = prci.wrapping_add(
            prcj.wrapping_mul(pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pw),
          );
          pi.layno = pi.poc.layno0;
          current_block = 2606304779496145856;
          break;
        } else {
          pi.x =
            (pi.x as core::ffi::c_uint).wrapping_add(pi.dx.wrapping_sub(pi.x.wrapping_rem(pi.dx)));
          current_block = 3123434771885419771;
        }
      }
    }
  }
  Err(())
}

/// Get next packet in precinct-component-resolution-layer order.
/// `pi` — packet iterator to modify
/// returns false if pi pointed to the last packet or else returns true
fn pi_next_pcrl(pi: &mut PiIterator, manager: &mut EventMgr) -> OpResult {
  let mut levelno: u32 = 0;
  let mut trx0: u32 = 0;
  let mut try0: u32 = 0;
  let mut trx1: u32 = 0;
  let mut try1: u32 = 0;
  let mut rpx: u32 = 0;
  let mut rpy: u32 = 0;
  let mut prci: u32 = 0;
  let mut prcj: u32 = 0;
  let mut current_block: u64;
  let mut index = 0_u32;
  if pi.poc.compno0 >= pi.numcomps || pi.poc.compno1 >= pi.numcomps.wrapping_add(1u32) {
    event_msg!(
      manager,
      EVT_ERROR,
      "pi_next_pcrl(): invalid compno0/compno1\n",
    );
    return Err(());
  }
  if !pi.first {
    current_block = 10853015579903106591;
  } else {
    let mut compno: u32 = 0;
    let mut resno: u32 = 0;
    pi.first = false;
    pi.dx = 0_u32;
    pi.dy = 0_u32;
    compno = 0_u32;
    while compno < pi.numcomps {
      resno = 0_u32;
      while resno < pi.comps[compno as usize].numresolutions {
        let mut dx: u32 = 0;
        let mut dy: u32 = 0;
        if pi.comps[compno as usize].resolutions[resno as usize]
          .pdx
          .wrapping_add(pi.comps[compno as usize].numresolutions)
          .wrapping_sub(1u32)
          .wrapping_sub(resno)
          < 32u32
          && pi.comps[compno as usize].dx
            <= (2147483647u32)
              .wrapping_mul(2u32)
              .wrapping_add(1u32)
              .wrapping_div(
                (1u32)
                  << pi.comps[compno as usize].resolutions[resno as usize]
                    .pdx
                    .wrapping_add(pi.comps[compno as usize].numresolutions)
                    .wrapping_sub(1u32)
                    .wrapping_sub(resno),
              )
        {
          dx = pi.comps[compno as usize].dx.wrapping_mul(
            (1u32)
              << pi.comps[compno as usize].resolutions[resno as usize]
                .pdx
                .wrapping_add(pi.comps[compno as usize].numresolutions)
                .wrapping_sub(1u32)
                .wrapping_sub(resno),
          );
          pi.dx = if pi.dx == 0 { dx } else { uint_min(pi.dx, dx) }
        }
        if pi.comps[compno as usize].resolutions[resno as usize]
          .pdy
          .wrapping_add(pi.comps[compno as usize].numresolutions)
          .wrapping_sub(1u32)
          .wrapping_sub(resno)
          < 32u32
          && pi.comps[compno as usize].dy
            <= (2147483647u32)
              .wrapping_mul(2u32)
              .wrapping_add(1u32)
              .wrapping_div(
                (1u32)
                  << pi.comps[compno as usize].resolutions[resno as usize]
                    .pdy
                    .wrapping_add(pi.comps[compno as usize].numresolutions)
                    .wrapping_sub(1u32)
                    .wrapping_sub(resno),
              )
        {
          dy = pi.comps[compno as usize].dy.wrapping_mul(
            (1u32)
              << pi.comps[compno as usize].resolutions[resno as usize]
                .pdy
                .wrapping_add(pi.comps[compno as usize].numresolutions)
                .wrapping_sub(1u32)
                .wrapping_sub(resno),
          );
          pi.dy = if pi.dy == 0 { dy } else { uint_min(pi.dy, dy) }
        }
        resno += 1;
      }
      compno += 1;
    }
    if pi.dx == 0u32 || pi.dy == 0u32 {
      return Err(());
    }
    if pi.tp_on == 0 {
      pi.poc.ty0 = pi.ty0;
      pi.poc.tx0 = pi.tx0;
      pi.poc.ty1 = pi.ty1;
      pi.poc.tx1 = pi.tx1
    }
    pi.y = pi.poc.ty0;
    current_block = 7245201122033322888;
  }
  loop {
    match current_block {
      10853015579903106591 => {
        pi.layno = pi.layno.wrapping_add(1);
        current_block = 6281126495347172768;
      }
      _ => {
        if pi.y >= pi.poc.ty1 {
          break;
        }
        pi.x = pi.poc.tx0;
        current_block = 8845338526596852646;
      }
    }
    loop {
      match current_block {
        8845338526596852646 => {
          if pi.x < pi.poc.tx1 {
            pi.compno = pi.poc.compno0
          } else {
            pi.y = (pi.y as core::ffi::c_uint)
              .wrapping_add(pi.dy.wrapping_sub(pi.y.wrapping_rem(pi.dy)));
            current_block = 7245201122033322888;
            break;
          }
          current_block = 980989089337379490;
        }
        _ => {
          if pi.layno < pi.poc.layno1 {
            index = pi
              .layno
              .wrapping_mul(pi.step_l)
              .wrapping_add(pi.resno.wrapping_mul(pi.step_r))
              .wrapping_add(pi.compno.wrapping_mul(pi.step_c))
              .wrapping_add(pi.precno.wrapping_mul(pi.step_p));
            if index >= pi.include_size {
              event_msg!(manager, EVT_ERROR, "Invalid access to pi->include",);
              return Err(());
            }
            if pi.include[index as usize].get() == 0 {
              pi.include[index as usize].set(1_i16);
              return Ok(());
            }
            current_block = 10853015579903106591;
            break;
          } else {
            current_block = 15512526488502093901;
          }
        }
      }
      loop {
        match current_block {
          15512526488502093901 => pi.resno = pi.resno.wrapping_add(1),
          _ => {
            if pi.compno < pi.poc.compno1 {
              pi.resno = pi.poc.resno0
            } else {
              pi.x = (pi.x as core::ffi::c_uint)
                .wrapping_add(pi.dx.wrapping_sub(pi.x.wrapping_rem(pi.dx)));
              current_block = 8845338526596852646;
              break;
            }
          }
        }
        if pi.resno < uint_min(pi.poc.resno1, pi.comps[pi.compno as usize].numresolutions) {
          levelno = 0;
          trx0 = 0;
          try0 = 0;
          trx1 = 0;
          try1 = 0;
          rpx = 0;
          rpy = 0;
          prci = 0;
          prcj = 0;
          levelno = pi.comps[pi.compno as usize].numresolutions - 1 - pi.resno;

          if ((pi.comps[pi.compno as usize].dx as u64) << levelno >> levelno)
            != pi.comps[pi.compno as usize].dx as u64
            || ((pi.comps[pi.compno as usize].dy as u64) << levelno >> levelno)
              != pi.comps[pi.compno as usize].dy as u64
          {
            current_block = 15512526488502093901;
            continue;
          }

          trx0 = uint64_ceildiv_res_uint32(
            pi.tx0 as u64,
            (pi.comps[pi.compno as usize].dx as u64) << levelno,
          );
          try0 = uint64_ceildiv_res_uint32(
            pi.ty0 as u64,
            (pi.comps[pi.compno as usize].dy as u64) << levelno,
          );
          trx1 = uint64_ceildiv_res_uint32(
            pi.tx1 as u64,
            (pi.comps[pi.compno as usize].dx as u64) << levelno,
          );
          try1 = uint64_ceildiv_res_uint32(
            pi.ty1 as u64,
            (pi.comps[pi.compno as usize].dy as u64) << levelno,
          );
          rpx = pi.comps[pi.compno as usize].resolutions[pi.resno as usize]
            .pdx
            .wrapping_add(levelno);
          rpy = pi.comps[pi.compno as usize].resolutions[pi.resno as usize]
            .pdy
            .wrapping_add(levelno);

          if (((pi.comps[pi.compno as usize].dx as u64) << rpx) >> rpx) as u32
            != pi.comps[pi.compno as usize].dx
            || (((pi.comps[pi.compno as usize].dy as u64) << rpy) >> rpy) as u32
              != pi.comps[pi.compno as usize].dy
          {
            current_block = 15512526488502093901;
            continue;
          }
          /* See ISO-15441. B.12.1.4 Position-component-resolution level-layer progression */
          if !(pi.y as u64).is_multiple_of((pi.comps[pi.compno as usize].dy as u64) << rpy)
            && !(pi.y == pi.ty0 && !((try0 as u64) << levelno).is_multiple_of(1u64 << rpy))
          {
            current_block = 15512526488502093901;
            continue;
          }
          if !(pi.x as u64).is_multiple_of((pi.comps[pi.compno as usize].dx as u64) << rpx)
            && !(pi.x == pi.tx0 && !((trx0 as u64) << levelno).is_multiple_of(1u64 << rpx))
          {
            current_block = 15512526488502093901;
            continue;
          }
          if pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pw == 0u32
            || pi.comps[pi.compno as usize].resolutions[pi.resno as usize].ph == 0u32
          {
            current_block = 15512526488502093901;
            continue;
          }
          if trx0 == trx1 || try0 == try1 {
            current_block = 15512526488502093901;
            continue;
          }
          prci = uint_floordivpow2(
            uint64_ceildiv_res_uint32(
              pi.x as u64,
              (pi.comps[pi.compno as usize].dx as u64) << levelno,
            ),
            pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pdx,
          ) - uint_floordivpow2(
            trx0,
            pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pdx,
          );
          prcj = uint_floordivpow2(
            uint64_ceildiv_res_uint32(
              pi.y as u64,
              (pi.comps[pi.compno as usize].dy as u64) << levelno,
            ),
            pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pdy,
          ) - uint_floordivpow2(
            try0,
            pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pdy,
          );
          pi.precno = prci.wrapping_add(
            prcj.wrapping_mul(pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pw),
          );
          pi.layno = pi.poc.layno0;
          current_block = 6281126495347172768;
          break;
        } else {
          pi.compno = pi.compno.wrapping_add(1);
          current_block = 980989089337379490;
        }
      }
    }
  }
  Err(())
}

/// Get next packet in component-precinct-resolution-layer order.
/// `pi` — packet iterator to modify
/// returns false if pi pointed to the last packet or else returns true
fn pi_next_cprl(pi: &mut PiIterator, manager: &mut EventMgr) -> OpResult {
  let mut resno: u32 = 0;
  let mut levelno: u32 = 0;
  let mut trx0: u32 = 0;
  let mut try0: u32 = 0;
  let mut trx1: u32 = 0;
  let mut try1: u32 = 0;
  let mut rpx: u32 = 0;
  let mut rpy: u32 = 0;
  let mut prci: u32 = 0;
  let mut prcj: u32 = 0;
  let mut current_block: u64;
  let mut index = 0_u32;
  if pi.poc.compno0 >= pi.numcomps || pi.poc.compno1 >= pi.numcomps.wrapping_add(1u32) {
    event_msg!(
      manager,
      EVT_ERROR,
      "pi_next_cprl(): invalid compno0/compno1\n",
    );
    return Err(());
  }
  if !pi.first {
    current_block = 13707613154239713890;
  } else {
    pi.first = false;
    pi.compno = pi.poc.compno0;
    current_block = 3640593987805443782;
  }
  loop {
    match current_block {
      13707613154239713890 => {
        pi.layno = pi.layno.wrapping_add(1);
        current_block = 15594839951440953787;
      }
      _ => {
        if pi.compno >= pi.poc.compno1 {
          break;
        }
        resno = 0;
        pi.dx = 0_u32;
        pi.dy = 0_u32;
        resno = 0_u32;
        while resno < pi.comps[pi.compno as usize].numresolutions {
          let mut dx: u32 = 0;
          let mut dy: u32 = 0;
          if pi.comps[pi.compno as usize].resolutions[resno as usize]
            .pdx
            .wrapping_add(pi.comps[pi.compno as usize].numresolutions)
            .wrapping_sub(1u32)
            .wrapping_sub(resno)
            < 32u32
            && pi.comps[pi.compno as usize].dx
              <= (2147483647u32)
                .wrapping_mul(2u32)
                .wrapping_add(1u32)
                .wrapping_div(
                  (1u32)
                    << pi.comps[pi.compno as usize].resolutions[resno as usize]
                      .pdx
                      .wrapping_add(pi.comps[pi.compno as usize].numresolutions)
                      .wrapping_sub(1u32)
                      .wrapping_sub(resno),
                )
          {
            dx = pi.comps[pi.compno as usize].dx.wrapping_mul(
              (1u32)
                << pi.comps[pi.compno as usize].resolutions[resno as usize]
                  .pdx
                  .wrapping_add(pi.comps[pi.compno as usize].numresolutions)
                  .wrapping_sub(1u32)
                  .wrapping_sub(resno),
            );
            pi.dx = if pi.dx == 0 { dx } else { uint_min(pi.dx, dx) }
          }
          if pi.comps[pi.compno as usize].resolutions[resno as usize]
            .pdy
            .wrapping_add(pi.comps[pi.compno as usize].numresolutions)
            .wrapping_sub(1u32)
            .wrapping_sub(resno)
            < 32u32
            && pi.comps[pi.compno as usize].dy
              <= (2147483647u32)
                .wrapping_mul(2u32)
                .wrapping_add(1u32)
                .wrapping_div(
                  (1u32)
                    << pi.comps[pi.compno as usize].resolutions[resno as usize]
                      .pdy
                      .wrapping_add(pi.comps[pi.compno as usize].numresolutions)
                      .wrapping_sub(1u32)
                      .wrapping_sub(resno),
                )
          {
            dy = pi.comps[pi.compno as usize].dy.wrapping_mul(
              (1u32)
                << pi.comps[pi.compno as usize].resolutions[resno as usize]
                  .pdy
                  .wrapping_add(pi.comps[pi.compno as usize].numresolutions)
                  .wrapping_sub(1u32)
                  .wrapping_sub(resno),
            );
            pi.dy = if pi.dy == 0 { dy } else { uint_min(pi.dy, dy) }
          }
          resno += 1;
        }
        if pi.dx == 0u32 || pi.dy == 0u32 {
          return Err(());
        }
        if pi.tp_on == 0 {
          pi.poc.ty0 = pi.ty0;
          pi.poc.tx0 = pi.tx0;
          pi.poc.ty1 = pi.ty1;
          pi.poc.tx1 = pi.tx1
        }
        pi.y = pi.poc.ty0;
        current_block = 18153031941552419006;
      }
    }
    loop {
      match current_block {
        18153031941552419006 => {
          if pi.y < pi.poc.ty1 {
            pi.x = pi.poc.tx0
          } else {
            pi.compno = pi.compno.wrapping_add(1);
            current_block = 3640593987805443782;
            break;
          }
          current_block = 10692455896603418738;
        }
        _ => {
          if pi.layno < pi.poc.layno1 {
            index = pi
              .layno
              .wrapping_mul(pi.step_l)
              .wrapping_add(pi.resno.wrapping_mul(pi.step_r))
              .wrapping_add(pi.compno.wrapping_mul(pi.step_c))
              .wrapping_add(pi.precno.wrapping_mul(pi.step_p));
            if index >= pi.include_size {
              event_msg!(manager, EVT_ERROR, "Invalid access to pi->include",);
              return Err(());
            }
            if pi.include[index as usize].get() == 0 {
              pi.include[index as usize].set(1_i16);
              return Ok(());
            }
            current_block = 13707613154239713890;
            break;
          } else {
            current_block = 3123434771885419771;
          }
        }
      }
      loop {
        match current_block {
          3123434771885419771 => pi.resno = pi.resno.wrapping_add(1),
          _ => {
            if pi.x < pi.poc.tx1 {
              pi.resno = pi.poc.resno0
            } else {
              pi.y = (pi.y as core::ffi::c_uint)
                .wrapping_add(pi.dy.wrapping_sub(pi.y.wrapping_rem(pi.dy)));
              current_block = 18153031941552419006;
              break;
            }
          }
        }
        if pi.resno < uint_min(pi.poc.resno1, pi.comps[pi.compno as usize].numresolutions) {
          levelno = 0;
          trx0 = 0;
          try0 = 0;
          trx1 = 0;
          try1 = 0;
          rpx = 0;
          rpy = 0;
          prci = 0;
          prcj = 0;
          levelno = pi.comps[pi.compno as usize].numresolutions - 1 - pi.resno;

          if ((pi.comps[pi.compno as usize].dx as u64) << levelno >> levelno)
            != pi.comps[pi.compno as usize].dx as u64
            || ((pi.comps[pi.compno as usize].dy as u64) << levelno >> levelno)
              != pi.comps[pi.compno as usize].dy as u64
          {
            current_block = 3123434771885419771;
            continue;
          }

          trx0 = uint64_ceildiv_res_uint32(
            pi.tx0 as u64,
            (pi.comps[pi.compno as usize].dx as u64) << levelno,
          );
          try0 = uint64_ceildiv_res_uint32(
            pi.ty0 as u64,
            (pi.comps[pi.compno as usize].dy as u64) << levelno,
          );
          trx1 = uint64_ceildiv_res_uint32(
            pi.tx1 as u64,
            (pi.comps[pi.compno as usize].dx as u64) << levelno,
          );
          try1 = uint64_ceildiv_res_uint32(
            pi.ty1 as u64,
            (pi.comps[pi.compno as usize].dy as u64) << levelno,
          );
          rpx = pi.comps[pi.compno as usize].resolutions[pi.resno as usize]
            .pdx
            .wrapping_add(levelno);
          rpy = pi.comps[pi.compno as usize].resolutions[pi.resno as usize]
            .pdy
            .wrapping_add(levelno);

          if (((pi.comps[pi.compno as usize].dx as u64) << rpx) >> rpx) as u32
            != pi.comps[pi.compno as usize].dx
            || (((pi.comps[pi.compno as usize].dy as u64) << rpy) >> rpy) as u32
              != pi.comps[pi.compno as usize].dy
          {
            current_block = 3123434771885419771;
            continue;
          }
          /* See ISO-15441. B.12.1.5 Component-position-resolution level-layer progression */
          if !(pi.y as u64).is_multiple_of((pi.comps[pi.compno as usize].dy as u64) << rpy)
            && !(pi.y == pi.ty0 && !((try0 as u64) << levelno).is_multiple_of(1u64 << rpy))
          {
            current_block = 3123434771885419771;
            continue;
          }
          if !(pi.x as u64).is_multiple_of((pi.comps[pi.compno as usize].dx as u64) << rpx)
            && !(pi.x == pi.tx0 && !((trx0 as u64) << levelno).is_multiple_of(1u64 << rpx))
          {
            current_block = 3123434771885419771;
            continue;
          }
          if pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pw == 0u32
            || pi.comps[pi.compno as usize].resolutions[pi.resno as usize].ph == 0u32
          {
            current_block = 3123434771885419771;
            continue;
          }
          if trx0 == trx1 || try0 == try1 {
            current_block = 3123434771885419771;
            continue;
          }
          prci = uint_floordivpow2(
            uint64_ceildiv_res_uint32(
              pi.x as u64,
              (pi.comps[pi.compno as usize].dx as u64) << levelno,
            ),
            pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pdx,
          ) - uint_floordivpow2(
            trx0,
            pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pdx,
          );
          prcj = uint_floordivpow2(
            uint64_ceildiv_res_uint32(
              pi.y as u64,
              (pi.comps[pi.compno as usize].dy as u64) << levelno,
            ),
            pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pdy,
          ) - uint_floordivpow2(
            try0,
            pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pdy,
          );
          pi.precno = prci.wrapping_add(
            prcj.wrapping_mul(pi.comps[pi.compno as usize].resolutions[pi.resno as usize].pw),
          );
          pi.layno = pi.poc.layno0;
          current_block = 15594839951440953787;
          break;
        } else {
          pi.x =
            (pi.x as core::ffi::c_uint).wrapping_add(pi.dx.wrapping_sub(pi.x.wrapping_rem(pi.dx)));
          current_block = 10692455896603418738;
        }
      }
    }
  }
  Err(())
}

/// Gets the encoding parameters needed to update the coding parameters and all the pocs.
///
/// `p_image` — the image being encoded.
/// `p_cp` — the coding parameters.
/// `tileno` — the tile index of the tile being encoded.
/// `p_tx0` — pointer that will hold the X0 parameter for the tile
/// `p_tx1` — pointer that will hold the X1 parameter for the tile
/// `p_ty0` — pointer that will hold the Y0 parameter for the tile
/// `p_ty1` — pointer that will hold the Y1 parameter for the tile
/// `p_max_prec` — pointer that will hold the maximum precision for all the bands of the tile
/// `p_max_res` — pointer that will hold the maximum number of resolutions for all the poc inside the tile.
/// `p_dx_min` — pointer that will hold the minimum dx of all the components of all the resolutions for the tile.
/// `p_dy_min` — pointer that will hold the minimum dy of all the components of all the resolutions for the tile.
pub(crate) fn get_encoding_parameters(
  p_image: &mut CoreImage,
  p_cp: &Cp,
  p_tileno: u32,
  p_tx0: &mut u32,
  p_tx1: &mut u32,
  p_ty0: &mut u32,
  p_ty1: &mut u32,
  p_dx_min: &mut u32,
  p_dy_min: &mut u32,
  p_max_prec: &mut u32,
  p_max_res: &mut u32,
) {
  /* loop */
  let mut compno: u32 = 0;
  let mut resno: u32 = 0;
  /* position in x and y of tile */
  let mut p: u32 = 0;
  let mut q: u32 = 0;
  /* non-corrected (in regard to image offset) tile offset */
  let mut l_tx0: u32 = 0;
  let mut l_ty0: u32 = 0;
  /* preconditions */

  assert!(p_tileno < p_cp.tw.wrapping_mul(p_cp.th));
  /* initializations */
  let tcp = &p_cp.tcps[p_tileno as usize];
  /* here calculation of tx0, tx1, ty0, ty1, maxprec, dx and dy */
  p = p_tileno.wrapping_rem(p_cp.tw);
  q = p_tileno.wrapping_div(p_cp.tw);
  /* find extent of tile */
  l_tx0 = p_cp.tx0.wrapping_add(p.wrapping_mul(p_cp.tdx)); /* can't be greater than p_image->x1 so won't overflow */
  *p_tx0 = uint_max(l_tx0, p_image.x0); /* can't be greater than p_image->y1 so won't overflow */
  *p_tx1 = uint_min(uint_adds(l_tx0, p_cp.tdx), p_image.x1);
  l_ty0 = p_cp.ty0.wrapping_add(q.wrapping_mul(p_cp.tdy));
  *p_ty0 = uint_max(l_ty0, p_image.y0);
  *p_ty1 = uint_min(uint_adds(l_ty0, p_cp.tdy), p_image.y1);
  /* max precision is 0 (can only grow) */
  *p_max_prec = 0_u32;
  *p_max_res = 0_u32;
  /* take the largest value for dx_min and dy_min */
  *p_dx_min = 0x7fffffff_u32;
  *p_dy_min = 0x7fffffff_u32;
  compno = 0_u32;
  while compno < p_image.numcomps {
    /* arithmetic variables to calculate */
    let mut l_level_no: u32 = 0;
    let mut l_rx0: u32 = 0;
    let mut l_ry0: u32 = 0;
    let mut l_rx1: u32 = 0;
    let mut l_ry1: u32 = 0;
    let mut l_px0: u32 = 0;
    let mut l_py0: u32 = 0;
    let mut l_px1: u32 = 0;
    let mut py1: u32 = 0;
    let mut l_pdx: u32 = 0;
    let mut l_pdy: u32 = 0;
    let mut l_pw: u32 = 0;
    let mut l_ph: u32 = 0;
    let mut l_product: u32 = 0;
    let mut l_tcx0: u32 = 0;
    let mut l_tcy0: u32 = 0;
    let mut l_tcx1: u32 = 0;
    let mut l_tcy1: u32 = 0;
    let l_tccp = &tcp.tccps[compno as usize];
    let l_img_comp = &p_image.comps[compno as usize];
    l_tcx0 = uint_ceildiv(*p_tx0, l_img_comp.dx);
    l_tcy0 = uint_ceildiv(*p_ty0, l_img_comp.dy);
    l_tcx1 = uint_ceildiv(*p_tx1, l_img_comp.dx);
    l_tcy1 = uint_ceildiv(*p_ty1, l_img_comp.dy);
    if l_tccp.numresolutions > *p_max_res {
      *p_max_res = l_tccp.numresolutions
    }
    /* use custom size for precincts */
    resno = 0_u32;
    while resno < l_tccp.numresolutions {
      let mut l_dx: u64 = 0;
      let mut l_dy: u64 = 0;

      /* precinct width and height */
      l_pdx = l_tccp.prcw[resno as usize];
      l_pdy = l_tccp.prch[resno as usize];

      l_dx = l_img_comp.dx as u64 * ((1u64) << (l_pdx + l_tccp.numresolutions - 1 - resno));
      l_dy = l_img_comp.dy as u64 * ((1u64) << (l_pdy + l_tccp.numresolutions - 1u32 - resno));
      /* take the minimum size for dx for each comp and resolution */
      if l_dx <= u32::MAX as u64 {
        *p_dx_min = uint_min(*p_dx_min, l_dx as u32);
      }
      if l_dy <= u32::MAX as u64 {
        *p_dy_min = uint_min(*p_dy_min, l_dy as u32);
      }
      /* various calculations of extents */
      l_level_no = l_tccp.numresolutions.wrapping_sub(1u32).wrapping_sub(resno);
      l_rx0 = uint_ceildivpow2(l_tcx0, l_level_no);
      l_ry0 = uint_ceildivpow2(l_tcy0, l_level_no);
      l_rx1 = uint_ceildivpow2(l_tcx1, l_level_no);
      l_ry1 = uint_ceildivpow2(l_tcy1, l_level_no);
      l_px0 = uint_floordivpow2(l_rx0, l_pdx) << l_pdx;
      l_py0 = uint_floordivpow2(l_ry0, l_pdy) << l_pdy;
      l_px1 = uint_ceildivpow2(l_rx1, l_pdx) << l_pdx;
      py1 = uint_ceildivpow2(l_ry1, l_pdy) << l_pdy;
      l_pw = if l_rx0 == l_rx1 {
        0u32
      } else {
        (l_px1.wrapping_sub(l_px0)) >> l_pdx
      };
      l_ph = if l_ry0 == l_ry1 {
        0u32
      } else {
        (py1.wrapping_sub(l_py0)) >> l_pdy
      };
      l_product = l_pw.wrapping_mul(l_ph);
      /* update precision */
      if l_product > *p_max_prec {
        *p_max_prec = l_product
      }
      resno += 1;
    }
    compno += 1;
  }
}

/// Gets the encoding parameters needed to update the coding parameters and all the pocs.
/// The precinct widths, heights, dx and dy for each component at each resolution will be stored as well.
/// the last parameter of the function should be an array of pointers of size nb components, each pointer leading
/// to an area of size 4 * max_res. The data is stored inside this area with the following pattern :
/// dx_compi_res0 , dy_compi_res0 , w_compi_res0, h_compi_res0 , dx_compi_res1 , dy_compi_res1 , w_compi_res1, h_compi_res1 , ...
///
/// `p_image` — the image being encoded.
/// `p_cp` — the coding parameters.
/// `tileno` — the tile index of the tile being encoded.
/// `p_tx0` — pointer that will hold the X0 parameter for the tile
/// `p_tx1` — pointer that will hold the X1 parameter for the tile
/// `p_ty0` — pointer that will hold the Y0 parameter for the tile
/// `p_ty1` — pointer that will hold the Y1 parameter for the tile
/// `p_max_prec` — pointer that will hold the maximum precision for all the bands of the tile
/// `p_max_res` — pointer that will hold the maximum number of resolutions for all the poc inside the tile.
/// `p_dx_min` — pointer that will hold the minimum dx of all the components of all the resolutions for the tile.
/// `p_dy_min` — pointer that will hold the minimum dy of all the components of all the resolutions for the tile.
/// `p_resolutions` — pointer to an area corresponding to the one described above.
pub(crate) fn get_all_encoding_parameters(
  p_image: &mut CoreImage,
  p_cp: &Cp,
  tileno: u32,
  p_tx0: &mut u32,
  p_tx1: &mut u32,
  p_ty0: &mut u32,
  p_ty1: &mut u32,
  p_dx_min: &mut u32,
  p_dy_min: &mut u32,
  p_max_prec: &mut u32,
  p_max_res: &mut u32,
  mut p_resolutions: Option<&mut [u32]>,
) {
  /* loop*/
  let mut compno: u32 = 0;
  let mut resno: u32 = 0;
  /* position in x and y of tile*/
  let mut p: u32 = 0;
  let mut q: u32 = 0;
  /* non-corrected (in regard to image offset) tile offset */
  let mut l_tx0: u32 = 0;
  let mut l_ty0: u32 = 0;
  /* preconditions in debug*/

  assert!(tileno < p_cp.tw.wrapping_mul(p_cp.th));
  /* initializations*/
  let tcp = &p_cp.tcps[tileno as usize];
  /* position in x and y of tile*/
  p = tileno.wrapping_rem(p_cp.tw);
  q = tileno.wrapping_div(p_cp.tw);
  /* here calculation of tx0, tx1, ty0, ty1, maxprec, l_dx and l_dy */
  l_tx0 = p_cp.tx0.wrapping_add(p.wrapping_mul(p_cp.tdx)); /* can't be greater than p_image->x1 so won't overflow */
  *p_tx0 = uint_max(l_tx0, p_image.x0); /* can't be greater than p_image->y1 so won't overflow */
  *p_tx1 = uint_min(uint_adds(l_tx0, p_cp.tdx), p_image.x1);
  l_ty0 = p_cp.ty0.wrapping_add(q.wrapping_mul(p_cp.tdy));
  *p_ty0 = uint_max(l_ty0, p_image.y0);
  *p_ty1 = uint_min(uint_adds(l_ty0, p_cp.tdy), p_image.y1);
  /* max precision and resolution is 0 (can only grow)*/
  *p_max_prec = 0_u32;
  *p_max_res = 0_u32;
  /* take the largest value for dx_min and dy_min*/
  *p_dx_min = 0x7fffffff_u32;
  *p_dy_min = 0x7fffffff_u32;
  compno = 0_u32;
  while compno < p_image.numcomps {
    /* arithmetic variables to calculate*/
    let mut l_level_no: u32 = 0;
    let mut l_rx0: u32 = 0;
    let mut l_ry0: u32 = 0;
    let mut l_rx1: u32 = 0;
    let mut l_ry1: u32 = 0;
    let mut l_px0: u32 = 0;
    let mut l_py0: u32 = 0;
    let mut l_px1: u32 = 0;
    let mut py1: u32 = 0;
    let mut l_product: u32 = 0;
    let mut l_tcx0: u32 = 0;
    let mut l_tcy0: u32 = 0;
    let mut l_tcx1: u32 = 0;
    let mut l_tcy1: u32 = 0;
    let mut l_pdx: u32 = 0;
    let mut l_pdy: u32 = 0;
    let mut l_pw: u32 = 0;
    let mut l_ph: u32 = 0;
    let l_tccp = &tcp.tccps[compno as usize];
    let l_img_comp = &p_image.comps[compno as usize];
    /* Each component owns a stride of 4 u32 per resolution (max 33). */
    let mut l_resolution_offset =
      (compno as usize).wrapping_mul((4i32 * J2K_MAXRLVLS as i32) as usize);
    l_tcx0 = uint_ceildiv(*p_tx0, l_img_comp.dx);
    l_tcy0 = uint_ceildiv(*p_ty0, l_img_comp.dy);
    l_tcx1 = uint_ceildiv(*p_tx1, l_img_comp.dx);
    l_tcy1 = uint_ceildiv(*p_ty1, l_img_comp.dy);
    if l_tccp.numresolutions > *p_max_res {
      *p_max_res = l_tccp.numresolutions
    }
    /* use custom size for precincts*/
    l_level_no = l_tccp.numresolutions;
    resno = 0_u32;
    while resno < l_tccp.numresolutions {
      let mut l_dx: u32 = 0;
      let mut l_dy: u32 = 0;
      l_level_no = l_level_no.wrapping_sub(1);
      /* precinct width and height*/
      l_pdx = l_tccp.prcw[resno as usize];
      l_pdy = l_tccp.prch[resno as usize];
      if let Some(buf) = p_resolutions.as_deref_mut() {
        buf[l_resolution_offset] = l_pdx;
        l_resolution_offset += 1;
        buf[l_resolution_offset] = l_pdy;
        l_resolution_offset += 1;
      }
      if l_pdx.wrapping_add(l_level_no) < 32u32
        && l_img_comp.dx
          <= (2147483647u32)
            .wrapping_mul(2u32)
            .wrapping_add(1u32)
            .wrapping_div((1u32) << l_pdx.wrapping_add(l_level_no))
      {
        l_dx = l_img_comp
          .dx
          .wrapping_mul((1u32) << l_pdx.wrapping_add(l_level_no));
        /* take the minimum size for l_dx for each comp and resolution*/
        *p_dx_min = uint_min(*p_dx_min, l_dx)
      }
      if l_pdy.wrapping_add(l_level_no) < 32u32
        && l_img_comp.dy
          <= (2147483647u32)
            .wrapping_mul(2u32)
            .wrapping_add(1u32)
            .wrapping_div((1u32) << l_pdy.wrapping_add(l_level_no))
      {
        l_dy = l_img_comp
          .dy
          .wrapping_mul((1u32) << l_pdy.wrapping_add(l_level_no));
        *p_dy_min = uint_min(*p_dy_min, l_dy)
      }
      /* various calculations of extents*/
      l_rx0 = uint_ceildivpow2(l_tcx0, l_level_no);
      l_ry0 = uint_ceildivpow2(l_tcy0, l_level_no);
      l_rx1 = uint_ceildivpow2(l_tcx1, l_level_no);
      l_ry1 = uint_ceildivpow2(l_tcy1, l_level_no);
      l_px0 = uint_floordivpow2(l_rx0, l_pdx) << l_pdx;
      l_py0 = uint_floordivpow2(l_ry0, l_pdy) << l_pdy;
      l_px1 = uint_ceildivpow2(l_rx1, l_pdx) << l_pdx;
      py1 = uint_ceildivpow2(l_ry1, l_pdy) << l_pdy;
      l_pw = if l_rx0 == l_rx1 {
        0u32
      } else {
        (l_px1.wrapping_sub(l_px0)) >> l_pdx
      };
      l_ph = if l_ry0 == l_ry1 {
        0u32
      } else {
        (py1.wrapping_sub(l_py0)) >> l_pdy
      };
      if let Some(buf) = p_resolutions.as_deref_mut() {
        buf[l_resolution_offset] = l_pw;
        l_resolution_offset += 1;
        buf[l_resolution_offset] = l_ph;
        l_resolution_offset += 1;
      }
      l_product = l_pw.wrapping_mul(l_ph);
      /* update precision*/
      if l_product > *p_max_prec {
        *p_max_prec = l_product
      }
      resno += 1;
    }
    compno += 1;
  }
}

/// Allocates memory for a packet iterator. Data and data sizes are set by this operation.
/// No other data is set. The include section of the packet  iterator is not allocated.
///
/// `p_image` — the image used to initialize the packet iterator (in fact only the number of components is relevant.
/// `p_cp` — the coding parameters.
/// `tileno` — the index of the tile from which creating the packet iterator.
/// `manager` — Event manager
pub(crate) fn pi_create(image: &CoreImage, cp: &Cp, tileno: u32) -> Vec<PiIterator> {
  /* loop*/
  let mut pino: u32 = 0;
  let mut compno: u32 = 0;
  /* number of poc in the p_pi*/
  let mut l_poc_bound: u32 = 0;
  /* preconditions in debug*/

  assert!(tileno < cp.tw.wrapping_mul(cp.th));
  /* initializations*/
  let tcp = &cp.tcps[tileno as usize];
  l_poc_bound = tcp.numpocs.wrapping_add(1u32);
  /* memory allocations*/
  let mut l_pi: Vec<PiIterator> = Vec::with_capacity(l_poc_bound as usize);
  pino = 0_u32;
  while pino < l_poc_bound {
    let mut current = PiIterator {
      numcomps: image.numcomps,
      comps: vec![Default::default(); image.numcomps as usize],
      ..Default::default()
    };
    compno = 0_u32;
    while compno < image.numcomps {
      let comp = &mut current.comps[compno as usize];
      let tccp = &tcp.tccps[compno as usize];
      comp.resolutions = vec![Default::default(); tccp.numresolutions as usize];
      comp.numresolutions = tccp.numresolutions;
      compno += 1;
    }
    l_pi.push(current);
    pino += 1;
  }
  l_pi
}
pub(crate) fn pi_update_encode_poc_and_final(
  p_cp: &mut Cp,
  p_tileno: u32,
  p_tx0: u32,
  p_tx1: u32,
  p_ty0: u32,
  p_ty1: u32,
  p_max_prec: u32,
  _p_max_res: u32,
  p_dx_min: u32,
  p_dy_min: u32,
) {
  /* loop*/
  let mut pino: u32 = 0;
  /* number of pocs*/
  let mut l_poc_bound: u32 = 0;
  /* preconditions in debug*/

  assert!(p_tileno < p_cp.tw.wrapping_mul(p_cp.th));
  /* initializations*/
  let l_tcp = &mut p_cp.tcps[p_tileno as usize];
  /* number of iterations in the loop */
  l_poc_bound = l_tcp.numpocs.wrapping_add(1u32);
  l_tcp.pocs[0].comp_s = l_tcp.pocs[0].compno0;
  l_tcp.pocs[0].comp_e = l_tcp.pocs[0].compno1;
  l_tcp.pocs[0].res_s = l_tcp.pocs[0].resno0;
  l_tcp.pocs[0].res_e = l_tcp.pocs[0].resno1;
  l_tcp.pocs[0].lay_e = l_tcp.pocs[0].layno1;
  /* special treatment for the first element*/
  l_tcp.pocs[0].lay_s = 0_u32;
  l_tcp.pocs[0].prg = l_tcp.pocs[0].prg1;
  l_tcp.pocs[0].prc_s = 0_u32;
  l_tcp.pocs[0].prc_e = p_max_prec;
  l_tcp.pocs[0].tx_s = p_tx0;
  l_tcp.pocs[0].tx_e = p_tx1;
  l_tcp.pocs[0].ty_s = p_ty0;
  l_tcp.pocs[0].ty_e = p_ty1;
  l_tcp.pocs[0].dx = p_dx_min;
  l_tcp.pocs[0].dy = p_dy_min;

  pino = 1_u32;
  while pino < l_poc_bound {
    l_tcp.pocs[pino as usize].comp_s = l_tcp.pocs[pino as usize].compno0;
    l_tcp.pocs[pino as usize].comp_e = l_tcp.pocs[pino as usize].compno1;
    l_tcp.pocs[pino as usize].res_s = l_tcp.pocs[pino as usize].resno0;
    l_tcp.pocs[pino as usize].res_e = l_tcp.pocs[pino as usize].resno1;
    l_tcp.pocs[pino as usize].lay_e = l_tcp.pocs[pino as usize].layno1;
    l_tcp.pocs[pino as usize].prg = l_tcp.pocs[pino as usize].prg1;
    l_tcp.pocs[pino as usize].prc_s = 0_u32;
    /* special treatment here different from the first element*/
    l_tcp.pocs[pino as usize].lay_s =
      if l_tcp.pocs[pino as usize].lay_e > l_tcp.pocs[(pino - 1) as usize].lay_e {
        l_tcp.pocs[pino as usize].lay_e
      } else {
        0u32
      };
    l_tcp.pocs[pino as usize].prc_e = p_max_prec;
    l_tcp.pocs[pino as usize].tx_s = p_tx0;
    l_tcp.pocs[pino as usize].tx_e = p_tx1;
    l_tcp.pocs[pino as usize].ty_s = p_ty0;
    l_tcp.pocs[pino as usize].ty_e = p_ty1;
    l_tcp.pocs[pino as usize].dx = p_dx_min;
    l_tcp.pocs[pino as usize].dy = p_dy_min;
    pino += 1;
  }
}

/// Updates the coding parameters if the encoding is not used with Progression order changes and final (and cinema parameters are used).
///
/// `p_cp` — the coding parameters to modify
/// `p_num_comps` — the number of components
/// `p_tileno` — the tile index being concerned.
/// `p_tx0` — X0 parameter for the tile
/// `p_tx1` — X1 parameter for the tile
/// `p_ty0` — Y0 parameter for the tile
/// `p_ty1` — Y1 parameter for the tile
/// `p_max_prec` — the maximum precision for all the bands of the tile
/// `p_max_res` — the maximum number of resolutions for all the poc inside the tile.
/// `p_dx_min` — the minimum dx of all the components of all the resolutions for the tile.
/// `p_dy_min` — the minimum dy of all the components of all the resolutions for the tile.
pub(crate) fn pi_update_encode_not_poc(
  p_cp: &mut Cp,
  p_num_comps: u32,
  p_tileno: u32,
  p_tx0: u32,
  p_tx1: u32,
  p_ty0: u32,
  p_ty1: u32,
  p_max_prec: u32,
  p_max_res: u32,
  p_dx_min: u32,
  p_dy_min: u32,
) {
  /* loop*/
  let mut pino: u32 = 0;
  /* number of pocs*/
  let mut l_poc_bound: u32 = 0;
  /* preconditions in debug*/

  assert!(p_tileno < p_cp.tw.wrapping_mul(p_cp.th));
  /* initializations*/
  let l_tcp = &mut p_cp.tcps[p_tileno as usize];
  /* number of iterations in the loop */
  l_poc_bound = l_tcp.numpocs.wrapping_add(1u32);
  pino = 0_u32;
  while pino < l_poc_bound {
    l_tcp.pocs[pino as usize].comp_s = 0_u32;
    l_tcp.pocs[pino as usize].comp_e = p_num_comps;
    l_tcp.pocs[pino as usize].res_s = 0_u32;
    l_tcp.pocs[pino as usize].res_e = p_max_res;
    l_tcp.pocs[pino as usize].lay_s = 0_u32;
    l_tcp.pocs[pino as usize].lay_e = l_tcp.numlayers;
    l_tcp.pocs[pino as usize].prg = l_tcp.prg.into();
    l_tcp.pocs[pino as usize].prc_s = 0_u32;
    l_tcp.pocs[pino as usize].prc_e = p_max_prec;
    l_tcp.pocs[pino as usize].tx_s = p_tx0;
    l_tcp.pocs[pino as usize].tx_e = p_tx1;
    l_tcp.pocs[pino as usize].ty_s = p_ty0;
    l_tcp.pocs[pino as usize].ty_e = p_ty1;
    l_tcp.pocs[pino as usize].dx = p_dx_min;
    l_tcp.pocs[pino as usize].dy = p_dy_min;
    pino += 1;
  }
}

pub(crate) fn pi_update_decode_poc(
  p_pi: &mut [PiIterator],
  p_tcp: &Tcp,
  p_max_precision: u32,
  _p_max_res: u32,
) {
  /* loop*/
  let mut pino: u32 = 0;
  /* encoding parameters to set*/
  let mut l_bound: u32 = 0;
  /* preconditions in debug*/

  /* initializations*/
  l_bound = p_tcp.numpocs.wrapping_add(1u32); /* Progression Order #0 */
  pino = 0_u32; /* Resolution Level Index #0 (End) */
  while pino < l_bound {
    p_pi[pino as usize].poc.prg = p_tcp.pocs[pino as usize].prg; /* Component Index #0 (End) */
    p_pi[pino as usize].first = true; /* Layer Index #0 (End) */
    p_pi[pino as usize].poc.resno0 = p_tcp.pocs[pino as usize].resno0;
    p_pi[pino as usize].poc.compno0 = p_tcp.pocs[pino as usize].compno0;
    p_pi[pino as usize].poc.layno0 = 0_u32;
    p_pi[pino as usize].poc.precno0 = 0_u32;
    p_pi[pino as usize].poc.resno1 = p_tcp.pocs[pino as usize].resno1;
    p_pi[pino as usize].poc.compno1 = p_tcp.pocs[pino as usize].compno1;
    p_pi[pino as usize].poc.layno1 = uint_min(p_tcp.pocs[pino as usize].layno1, p_tcp.numlayers);
    p_pi[pino as usize].poc.precno1 = p_max_precision;
    pino += 1;
  }
}

pub(crate) fn pi_update_decode_not_poc(
  p_pi: &mut [PiIterator],
  p_tcp: &Tcp,
  p_max_precision: u32,
  p_max_res: u32,
) {
  /* loop*/
  let mut pino: u32 = 0;
  /* encoding parameters to set*/
  let mut l_bound: u32 = 0;
  /* preconditions in debug*/

  /* initializations*/
  l_bound = p_tcp.numpocs.wrapping_add(1u32);
  pino = 0_u32;
  while pino < l_bound {
    p_pi[pino as usize].poc.prg = p_tcp.prg.into();
    p_pi[pino as usize].first = true;
    p_pi[pino as usize].poc.resno0 = 0_u32;
    p_pi[pino as usize].poc.compno0 = 0_u32;
    p_pi[pino as usize].poc.layno0 = 0_u32;
    p_pi[pino as usize].poc.precno0 = 0_u32;
    p_pi[pino as usize].poc.resno1 = p_max_res;
    p_pi[pino as usize].poc.compno1 = p_pi[pino as usize].numcomps;
    p_pi[pino as usize].poc.layno1 = p_tcp.numlayers;
    p_pi[pino as usize].poc.precno1 = p_max_precision;
    pino += 1;
  }
}

fn pi_check_next_level(pos: i32, tcp: &crate::types::Poc, prog: ProgressionOrder) -> OpResult {
  let mut i: i32 = 0; /*end if*/
  if pos >= 0i32 {
    i = pos;
    while i >= 0i32 {
      match prog.get_step(i) {
        ProgressionStep::Resolution => {
          if tcp.res_t == tcp.res_e {
            if pi_check_next_level(pos - 1i32, tcp, prog).is_ok() {
              return Ok(());
            } else {
              return Err(());
            }
          } else {
            return Ok(());
          }
          /*end case P*/
        }
        ProgressionStep::Component => {
          if tcp.comp_t == tcp.comp_e {
            if pi_check_next_level(pos - 1i32, tcp, prog).is_ok() {
              return Ok(());
            } else {
              return Err(());
            }
          } else {
            return Ok(());
          }
        }
        ProgressionStep::Layer => {
          if tcp.lay_t == tcp.lay_e {
            if pi_check_next_level(pos - 1i32, tcp, prog).is_ok() {
              return Ok(());
            } else {
              return Err(());
            }
          } else {
            return Ok(());
          }
        }
        ProgressionStep::Precinct => {
          match tcp.prg as core::ffi::c_int {
            0 | 1 => {
              /* fall through */
              if tcp.prc_t == tcp.prc_e {
                if pi_check_next_level(i - 1i32, tcp, prog).is_ok() {
                  return Ok(());
                } else {
                  return Err(());
                }
              } else {
                return Ok(());
              }
            }
            _ => {
              if tcp.tx0_t == tcp.tx_e {
                /*TY*/
                if tcp.ty0_t == tcp.ty_e {
                  if pi_check_next_level(i - 1i32, tcp, prog).is_ok() {
                    return Ok(());
                  } else {
                    return Err(());
                  }
                } else {
                  return Ok(());
                }
              /*TY*/
              } else {
                return Ok(());
              }
            }
          }
        }
        _ => {}
      }
      i -= 1
      /*end switch*/
    }
    /*end for*/
  }
  Err(())
}
/*
==========================================================
   Packet iterator interface
==========================================================
*/

pub(crate) fn pi_create_decode(
  p_image: &mut CoreImage,
  p_cp: &mut Cp,
  p_tile_no: u32,
) -> Vec<PiIterator> {
  let numcomps = p_image.numcomps;
  /* loop */
  let mut pino: u32 = 0;
  let mut compno: u32 = 0;
  let mut resno: u32 = 0;
  /* encoding parameters to set */
  let mut l_max_res: u32 = 0;
  let mut l_max_prec: u32 = 0;
  let mut l_tx0: u32 = 0;
  let mut l_tx1: u32 = 0;
  let mut l_ty0: u32 = 0;
  let mut l_ty1: u32 = 0;
  let mut l_dx_min: u32 = 0;
  let mut l_dy_min: u32 = 0;
  let mut l_bound: u32 = 0;
  let mut l_step_p: u32 = 0;
  let mut l_step_c: u32 = 0;
  let mut l_step_r: u32 = 0;
  let mut l_step_l: u32 = 0;
  /* preconditions in debug */

  assert!(p_tile_no < p_cp.tw.wrapping_mul(p_cp.th));
  /* initializations */
  l_bound = p_cp.tcps[p_tile_no as usize].numpocs.wrapping_add(1u32);
  let l_data_stride = (4i32 * J2K_MAXRLVLS as i32) as usize;
  let mut l_tmp_data: Vec<u32> = vec![0; l_data_stride.wrapping_mul(numcomps as usize)];
  /* memory allocation for pi */
  let mut l_pi = pi_create(p_image, p_cp, p_tile_no);
  if l_pi.is_empty() {
    return l_pi;
  }
  /* get encoding parameters */
  get_all_encoding_parameters(
    p_image,
    p_cp,
    p_tile_no,
    &mut l_tx0,
    &mut l_tx1,
    &mut l_ty0,
    &mut l_ty1,
    &mut l_dx_min,
    &mut l_dy_min,
    &mut l_max_prec,
    &mut l_max_res,
    Some(&mut l_tmp_data),
  );
  /* step calculations */
  l_step_p = 1_u32;
  l_step_c = l_max_prec.wrapping_mul(l_step_p);
  l_step_r = numcomps.wrapping_mul(l_step_c);
  l_step_l = l_max_res.wrapping_mul(l_step_r);
  /* memory allocation for include */
  /* prevent an integer overflow issue */
  /* 0 < l_tcp->numlayers < 65536 c.f. j2k_read_cod in j2k.c */
  let include_size = if l_step_l
    <= u32::MAX.wrapping_div(p_cp.tcps[p_tile_no as usize].numlayers.wrapping_add(1u32))
  {
    p_cp.tcps[p_tile_no as usize]
      .numlayers
      .wrapping_add(1u32)
      .wrapping_mul(l_step_l)
  } else {
    /* The step product overflows u32: the C wraps it (UB, wrong-sized
    include buffer); the port fails cleanly here instead. */
    return Vec::new();
  };
  /* An all-components-empty tile (subsampling larger than the tile size)
  legitimately yields include_size == 0: the C's calloc(0) succeeds and the
  iterator emits zero packets (precno1 = pw*ph = 0 keeps the precinct loops
  from ever touching the include mask).  Create the iterator with an empty
  mask so the tile decodes as empty like the C instead of failing. */
  {
    // The include bitmask is shared across the pi elements of this tile:
    // elements 1.. clone element 0's Rc, exactly like the C version.
    // The C's calloc fails gracefully on OOM; a vec! would abort the
    // process on a crafted header driving a large include mask.
    let mut include_vec: Vec<Cell<i16>> = Vec::new();
    if include_vec
      .try_reserve_exact(include_size as usize)
      .is_err()
    {
      return Vec::new();
    }
    include_vec.resize(include_size as usize, Cell::new(0i16));
    let include: Rc<[Cell<i16>]> = include_vec.into();
    l_pi[0].include = include;
  }
  l_pi[0].include_size = include_size;
  /* special treatment for the first packet iterator */
  l_pi[0].tx0 = l_tx0;
  l_pi[0].ty0 = l_ty0;
  l_pi[0].tx1 = l_tx1;
  l_pi[0].ty1 = l_ty1;

  l_pi[0].step_p = l_step_p;
  l_pi[0].step_c = l_step_c;
  l_pi[0].step_r = l_step_r;
  l_pi[0].step_l = l_step_l;
  /* allocation for components and number of components has already been calculated by pi_create */
  compno = 0_u32;
  while compno < numcomps {
    let l_current_comp = &mut l_pi[0].comps[compno as usize];
    l_current_comp.dx = p_image.comps[compno as usize].dx;
    l_current_comp.dy = p_image.comps[compno as usize].dy;
    /* resolutions have already been initialized */
    let mut ev_offset = (compno as usize).wrapping_mul(l_data_stride);
    resno = 0_u32;
    while resno < l_current_comp.numresolutions {
      let l_res = &mut l_current_comp.resolutions[resno as usize];
      l_res.pdx = l_tmp_data[ev_offset];
      ev_offset += 1;
      l_res.pdy = l_tmp_data[ev_offset];
      ev_offset += 1;
      l_res.pw = l_tmp_data[ev_offset];
      ev_offset += 1;
      l_res.ph = l_tmp_data[ev_offset];
      ev_offset += 1;
      resno += 1;
    }
    compno += 1;
  }
  pino = 1_u32;
  while pino < l_bound {
    l_pi[pino as usize].tx0 = l_tx0;
    l_pi[pino as usize].ty0 = l_ty0;
    l_pi[pino as usize].tx1 = l_tx1;
    l_pi[pino as usize].ty1 = l_ty1;
    /*l_pi[pino as usize].dx = l_dx_min;*/
    /*l_pi[pino as usize].dy = l_dy_min;*/
    l_pi[pino as usize].step_p = l_step_p;
    l_pi[pino as usize].step_c = l_step_c;
    l_pi[pino as usize].step_r = l_step_r;
    l_pi[pino as usize].step_l = l_step_l;
    /* allocation for components and number of components has already been calculated by pi_create */
    compno = 0_u32;
    while compno < numcomps {
      let l_current_comp = &mut l_pi[pino as usize].comps[compno as usize];
      l_current_comp.dx = p_image.comps[compno as usize].dx;
      l_current_comp.dy = p_image.comps[compno as usize].dy;
      /* resolutions have already been initialized */
      let mut ev_offset = (compno as usize).wrapping_mul(l_data_stride);
      resno = 0_u32;
      while resno < l_current_comp.numresolutions {
        let l_res_0 = &mut l_current_comp.resolutions[resno as usize];
        l_res_0.pdx = l_tmp_data[ev_offset];
        ev_offset += 1;
        l_res_0.pdy = l_tmp_data[ev_offset];
        ev_offset += 1;
        l_res_0.pw = l_tmp_data[ev_offset];
        ev_offset += 1;
        l_res_0.ph = l_tmp_data[ev_offset];
        ev_offset += 1;
        resno += 1;
      }
      compno += 1;
    }
    /* special treatment */
    l_pi[pino as usize].include = l_pi[(pino - 1) as usize].include.clone();
    l_pi[pino as usize].include_size = l_pi[(pino - 1) as usize].include_size;
    pino += 1;
  }
  if p_cp.tcps[p_tile_no as usize].poc {
    pi_update_decode_poc(
      &mut l_pi,
      &p_cp.tcps[p_tile_no as usize],
      l_max_prec,
      l_max_res,
    );
  } else {
    pi_update_decode_not_poc(
      &mut l_pi,
      &p_cp.tcps[p_tile_no as usize],
      l_max_prec,
      l_max_res,
    );
  }
  l_pi
}
pub(crate) fn get_encoding_packet_count(p_image: &mut CoreImage, p_cp: &Cp, p_tile_no: u32) -> u32 {
  let mut l_max_res: u32 = 0;
  let mut l_max_prec: u32 = 0;
  let mut l_tx0: u32 = 0;
  let mut l_tx1: u32 = 0;
  let mut l_ty0: u32 = 0;
  let mut l_ty1: u32 = 0;
  let mut l_dx_min: u32 = 0;
  let mut l_dy_min: u32 = 0;
  /* preconditions in debug*/

  assert!(p_tile_no < p_cp.tw.wrapping_mul(p_cp.th));
  /* get encoding parameters*/
  get_all_encoding_parameters(
    p_image,
    p_cp,
    p_tile_no,
    &mut l_tx0,
    &mut l_tx1,
    &mut l_ty0,
    &mut l_ty1,
    &mut l_dx_min,
    &mut l_dy_min,
    &mut l_max_prec,
    &mut l_max_res,
    None,
  );
  p_cp.tcps[p_tile_no as usize]
    .numlayers
    .wrapping_mul(l_max_prec)
    .wrapping_mul(p_image.numcomps)
    .wrapping_mul(l_max_res)
}

pub(crate) fn pi_initialise_encode(
  p_image: &mut CoreImage,
  p_cp: &mut Cp,
  p_tile_no: u32,
  p_t2_mode: J2K_T2_MODE,
) -> Vec<PiIterator> {
  let numcomps = p_image.numcomps;
  /* loop*/
  let mut pino: u32 = 0;
  let mut compno: u32 = 0;
  let mut resno: u32 = 0;
  /* encoding parameters to set*/
  let mut l_max_res: u32 = 0;
  let mut l_max_prec: u32 = 0;
  let mut l_tx0: u32 = 0;
  let mut l_tx1: u32 = 0;
  let mut l_ty0: u32 = 0;
  let mut l_ty1: u32 = 0;
  let mut l_dx_min: u32 = 0;
  let mut l_dy_min: u32 = 0;
  let mut l_bound: u32 = 0;
  let mut l_step_p: u32 = 0;
  let mut l_step_c: u32 = 0;
  let mut l_step_r: u32 = 0;
  let mut l_step_l: u32 = 0;
  /* preconditions in debug*/

  assert!(p_tile_no < p_cp.tw.wrapping_mul(p_cp.th));
  /* initializations*/
  l_bound = p_cp.tcps[p_tile_no as usize].numpocs.wrapping_add(1u32);
  let l_data_stride = (4i32 * J2K_MAXRLVLS as i32) as usize;
  let mut l_tmp_data: Vec<u32> = vec![0; l_data_stride.wrapping_mul(numcomps as usize)];
  /* memory allocation for pi*/
  let mut l_pi = pi_create(p_image, p_cp, p_tile_no);
  if l_pi.is_empty() {
    return l_pi;
  }
  /* get encoding parameters*/
  get_all_encoding_parameters(
    p_image,
    p_cp,
    p_tile_no,
    &mut l_tx0,
    &mut l_tx1,
    &mut l_ty0,
    &mut l_ty1,
    &mut l_dx_min,
    &mut l_dy_min,
    &mut l_max_prec,
    &mut l_max_res,
    Some(&mut l_tmp_data),
  );
  /* step calculations*/
  l_step_p = 1_u32;
  l_step_c = l_max_prec.wrapping_mul(l_step_p);
  l_step_r = numcomps.wrapping_mul(l_step_c);
  l_step_l = l_max_res.wrapping_mul(l_step_r);
  /* set values for first packet iterator*/
  l_pi[0].tp_on = p_cp.m_specific_param.m_enc.m_tp_on as u8;
  /* memory allocation for include*/
  /* Guard against integer overflow (upstream 839936aa): the include array
  covers numlayers * l_step_l entries. */
  let l_numlayers = p_cp.tcps[p_tile_no as usize].numlayers;
  let include_size = if l_numlayers != 0 && l_step_l <= u32::MAX / l_numlayers {
    l_numlayers.wrapping_mul(l_step_l)
  } else {
    /* The step product overflows u32: the C wraps it (UB, wrong-sized
    include buffer); the port fails cleanly here instead. */
    return Vec::new();
  };
  /* An all-components-empty tile (subsampling larger than the tile size)
  legitimately yields include_size == 0: the C's calloc(0) succeeds and the
  iterator emits zero packets (the precinct loops never touch the include
  mask).  Create the iterator with an empty mask so the tile encodes as
  empty like the C instead of failing. */
  {
    // The include bitmask is shared across the pi elements of this tile.
    // The C's calloc fails gracefully on OOM; a vec! would abort the
    // process on a crafted header driving a large include mask.
    let mut include_vec: Vec<Cell<i16>> = Vec::new();
    if include_vec
      .try_reserve_exact(include_size as usize)
      .is_err()
    {
      return Vec::new();
    }
    include_vec.resize(include_size as usize, Cell::new(0i16));
    let include: Rc<[Cell<i16>]> = include_vec.into();
    l_pi[0].include = include;
  }
  l_pi[0].include_size = include_size;
  /* special treatment for the first packet iterator*/
  l_pi[0].tx0 = l_tx0;
  l_pi[0].ty0 = l_ty0;
  l_pi[0].tx1 = l_tx1;
  l_pi[0].ty1 = l_ty1;
  l_pi[0].dx = l_dx_min;
  l_pi[0].dy = l_dy_min;
  l_pi[0].step_p = l_step_p;
  l_pi[0].step_c = l_step_c;
  l_pi[0].step_r = l_step_r;
  l_pi[0].step_l = l_step_l;
  /* allocation for components and number of components has already been calculated by pi_create */
  compno = 0_u32;
  while compno < numcomps {
    let l_current_comp = &mut l_pi[0].comps[compno as usize];
    l_current_comp.dx = p_image.comps[compno as usize].dx;
    l_current_comp.dy = p_image.comps[compno as usize].dy;
    /* resolutions have already been initialized */
    let mut ev_offset = (compno as usize).wrapping_mul(l_data_stride);
    resno = 0_u32;
    while resno < l_current_comp.numresolutions {
      let l_res = &mut l_current_comp.resolutions[resno as usize];
      l_res.pdx = l_tmp_data[ev_offset];
      ev_offset += 1;
      l_res.pdy = l_tmp_data[ev_offset];
      ev_offset += 1;
      l_res.pw = l_tmp_data[ev_offset];
      ev_offset += 1;
      l_res.ph = l_tmp_data[ev_offset];
      ev_offset += 1;
      resno += 1;
    }
    compno += 1;
  }
  pino = 1_u32;
  while pino < l_bound {
    l_pi[pino as usize].tx0 = l_tx0;
    l_pi[pino as usize].ty0 = l_ty0;
    l_pi[pino as usize].tx1 = l_tx1;
    l_pi[pino as usize].ty1 = l_ty1;
    l_pi[pino as usize].dx = l_dx_min;
    l_pi[pino as usize].dy = l_dy_min;
    l_pi[pino as usize].step_p = l_step_p;
    l_pi[pino as usize].step_c = l_step_c;
    l_pi[pino as usize].step_r = l_step_r;
    l_pi[pino as usize].step_l = l_step_l;
    /* allocation for components and number of components has already been calculated by pi_create */
    compno = 0_u32;
    while compno < numcomps {
      let l_current_comp = &mut l_pi[pino as usize].comps[compno as usize];
      l_current_comp.dx = p_image.comps[compno as usize].dx;
      l_current_comp.dy = p_image.comps[compno as usize].dy;
      /* resolutions have already been initialized */
      let mut ev_offset = (compno as usize).wrapping_mul(l_data_stride);
      resno = 0_u32;
      while resno < l_current_comp.numresolutions {
        let l_res_0 = &mut l_current_comp.resolutions[resno as usize];
        l_res_0.pdx = l_tmp_data[ev_offset];
        ev_offset += 1;
        l_res_0.pdy = l_tmp_data[ev_offset];
        ev_offset += 1;
        l_res_0.pw = l_tmp_data[ev_offset];
        ev_offset += 1;
        l_res_0.ph = l_tmp_data[ev_offset];
        ev_offset += 1;
        resno += 1;
      }
      compno += 1;
    }
    /* special treatment*/
    l_pi[pino as usize].include = l_pi[(pino - 1) as usize].include.clone();
    l_pi[pino as usize].include_size = l_pi[(pino - 1) as usize].include_size;
    pino += 1;
  }
  if p_cp.tcps[p_tile_no as usize].poc
    && ((p_cp.rsiz as core::ffi::c_int >= 0x3i32 && p_cp.rsiz as core::ffi::c_int <= 0x6i32)
      || p_t2_mode as core::ffi::c_uint == FINAL_PASS as core::ffi::c_uint)
  {
    pi_update_encode_poc_and_final(
      p_cp, p_tile_no, l_tx0, l_tx1, l_ty0, l_ty1, l_max_prec, l_max_res, l_dx_min, l_dy_min,
    );
  } else {
    pi_update_encode_not_poc(
      p_cp, numcomps, p_tile_no, l_tx0, l_tx1, l_ty0, l_ty1, l_max_prec, l_max_res, l_dx_min,
      l_dy_min,
    );
  }
  l_pi
}
pub(crate) fn pi_create_encode(
  pi: &mut [PiIterator],
  cp: &mut Cp,
  tileno: u32,
  pino: u32,
  tpnum: u32,
  tppos: i32,
  t2_mode: J2K_T2_MODE,
) {
  let mut i: i32 = 0;
  let mut incr_top = 1_u32;
  let mut reset_x = 0_u32;
  let rsiz = cp.rsiz;
  let tp_on = cp.m_specific_param.m_enc.m_tp_on;
  let tcp = &mut cp.tcps[tileno as usize].pocs[pino as usize];
  let prog = j2k_convert_progression_order(tcp.prg.into());
  pi[pino as usize].first = true;
  pi[pino as usize].poc.prg = tcp.prg;
  if !(tp_on
    && (!(rsiz as core::ffi::c_int >= 0x3i32 && rsiz as core::ffi::c_int <= 0x6i32)
      && !(rsiz as core::ffi::c_int >= 0x400i32 && rsiz as core::ffi::c_int <= 0x900i32 | 0x9bi32)
      && t2_mode as core::ffi::c_uint == FINAL_PASS as core::ffi::c_uint
      || rsiz as core::ffi::c_int >= 0x3i32 && rsiz as core::ffi::c_int <= 0x6i32
      || rsiz as core::ffi::c_int >= 0x400i32 && rsiz as core::ffi::c_int <= 0x900i32 | 0x9bi32))
  {
    pi[pino as usize].poc.resno0 = tcp.res_s;
    pi[pino as usize].poc.resno1 = tcp.res_e;
    pi[pino as usize].poc.compno0 = tcp.comp_s;
    pi[pino as usize].poc.compno1 = tcp.comp_e;
    pi[pino as usize].poc.layno0 = tcp.lay_s;
    pi[pino as usize].poc.layno1 = tcp.lay_e;
    pi[pino as usize].poc.precno0 = tcp.prc_s;
    pi[pino as usize].poc.precno1 = tcp.prc_e;
    pi[pino as usize].poc.tx0 = tcp.tx_s;
    pi[pino as usize].poc.ty0 = tcp.ty_s;
    pi[pino as usize].poc.tx1 = tcp.tx_e;
    pi[pino as usize].poc.ty1 = tcp.ty_e
  } else {
    i = tppos + 1i32;
    while i < 4i32 {
      match prog.get_step(i) {
        ProgressionStep::Resolution => {
          pi[pino as usize].poc.resno0 = tcp.res_s;
          pi[pino as usize].poc.resno1 = tcp.res_e
        }
        ProgressionStep::Component => {
          pi[pino as usize].poc.compno0 = tcp.comp_s;
          pi[pino as usize].poc.compno1 = tcp.comp_e
        }
        ProgressionStep::Layer => {
          pi[pino as usize].poc.layno0 = tcp.lay_s;
          pi[pino as usize].poc.layno1 = tcp.lay_e
        }
        ProgressionStep::Precinct => match tcp.prg as core::ffi::c_int {
          0 | 1 => {
            pi[pino as usize].poc.precno0 = tcp.prc_s;
            pi[pino as usize].poc.precno1 = tcp.prc_e
          }
          _ => {
            pi[pino as usize].poc.tx0 = tcp.tx_s;
            pi[pino as usize].poc.ty0 = tcp.ty_s;
            pi[pino as usize].poc.tx1 = tcp.tx_e;
            pi[pino as usize].poc.ty1 = tcp.ty_e
          }
        },
        _ => {}
      }
      i += 1
    }
    if tpnum == 0u32 {
      i = tppos;
      while i >= 0i32 {
        match prog.get_step(i) {
          ProgressionStep::Component => {
            tcp.comp_t = tcp.comp_s;
            pi[pino as usize].poc.compno0 = tcp.comp_t;
            pi[pino as usize].poc.compno1 = tcp.comp_t.wrapping_add(1u32);
            tcp.comp_t = (tcp.comp_t as core::ffi::c_uint).wrapping_add(1u32)
          }
          ProgressionStep::Resolution => {
            tcp.res_t = tcp.res_s;
            pi[pino as usize].poc.resno0 = tcp.res_t;
            pi[pino as usize].poc.resno1 = tcp.res_t.wrapping_add(1u32);
            tcp.res_t = (tcp.res_t as core::ffi::c_uint).wrapping_add(1u32)
          }
          ProgressionStep::Layer => {
            tcp.lay_t = tcp.lay_s;
            pi[pino as usize].poc.layno0 = tcp.lay_t;
            pi[pino as usize].poc.layno1 = tcp.lay_t.wrapping_add(1u32);
            tcp.lay_t = (tcp.lay_t as core::ffi::c_uint).wrapping_add(1u32)
          }
          ProgressionStep::Precinct => match tcp.prg as core::ffi::c_int {
            0 | 1 => {
              tcp.prc_t = tcp.prc_s;
              pi[pino as usize].poc.precno0 = tcp.prc_t;
              pi[pino as usize].poc.precno1 = tcp.prc_t.wrapping_add(1u32);
              tcp.prc_t = (tcp.prc_t as core::ffi::c_uint).wrapping_add(1u32)
            }
            _ => {
              tcp.tx0_t = tcp.tx_s;
              tcp.ty0_t = tcp.ty_s;
              pi[pino as usize].poc.tx0 = tcp.tx0_t;
              pi[pino as usize].poc.tx1 = tcp
                .tx0_t
                .wrapping_add(tcp.dx)
                .wrapping_sub(tcp.tx0_t.wrapping_rem(tcp.dx));
              pi[pino as usize].poc.ty0 = tcp.ty0_t;
              pi[pino as usize].poc.ty1 = tcp
                .ty0_t
                .wrapping_add(tcp.dy)
                .wrapping_sub(tcp.ty0_t.wrapping_rem(tcp.dy));
              tcp.tx0_t = pi[pino as usize].poc.tx1;
              tcp.ty0_t = pi[pino as usize].poc.ty1
            }
          },
          _ => {}
        }
        i -= 1
      }
      incr_top = 1_u32
    } else {
      i = tppos;
      while i >= 0i32 {
        match prog.get_step(i) {
          ProgressionStep::Component => {
            pi[pino as usize].poc.compno0 = tcp.comp_t.wrapping_sub(1u32);
            pi[pino as usize].poc.compno1 = tcp.comp_t
          }
          ProgressionStep::Resolution => {
            pi[pino as usize].poc.resno0 = tcp.res_t.wrapping_sub(1u32);
            pi[pino as usize].poc.resno1 = tcp.res_t
          }
          ProgressionStep::Layer => {
            pi[pino as usize].poc.layno0 = tcp.lay_t.wrapping_sub(1u32);
            pi[pino as usize].poc.layno1 = tcp.lay_t
          }
          ProgressionStep::Precinct => match tcp.prg as core::ffi::c_int {
            0 | 1 => {
              pi[pino as usize].poc.precno0 = tcp.prc_t.wrapping_sub(1u32);
              pi[pino as usize].poc.precno1 = tcp.prc_t
            }
            _ => {
              pi[pino as usize].poc.tx0 = tcp
                .tx0_t
                .wrapping_sub(tcp.dx)
                .wrapping_sub(tcp.tx0_t.wrapping_rem(tcp.dx));
              pi[pino as usize].poc.tx1 = tcp.tx0_t;
              pi[pino as usize].poc.ty0 = tcp
                .ty0_t
                .wrapping_sub(tcp.dy)
                .wrapping_sub(tcp.ty0_t.wrapping_rem(tcp.dy));
              pi[pino as usize].poc.ty1 = tcp.ty0_t
            }
          },
          _ => {}
        }
        if incr_top == 1u32 {
          match prog.get_step(i) {
            ProgressionStep::Resolution => {
              if tcp.res_t == tcp.res_e {
                if pi_check_next_level(i - 1i32, &*tcp, prog).is_ok() {
                  tcp.res_t = tcp.res_s;
                  pi[pino as usize].poc.resno0 = tcp.res_t;
                  pi[pino as usize].poc.resno1 = tcp.res_t.wrapping_add(1u32);
                  tcp.res_t = (tcp.res_t as core::ffi::c_uint).wrapping_add(1u32);
                  incr_top = 1_u32
                } else {
                  incr_top = 0_u32
                }
              } else {
                pi[pino as usize].poc.resno0 = tcp.res_t;
                pi[pino as usize].poc.resno1 = tcp.res_t.wrapping_add(1u32);
                tcp.res_t = (tcp.res_t as core::ffi::c_uint).wrapping_add(1u32);
                incr_top = 0_u32
              }
            }
            ProgressionStep::Component => {
              if tcp.comp_t == tcp.comp_e {
                if pi_check_next_level(i - 1i32, &*tcp, prog).is_ok() {
                  tcp.comp_t = tcp.comp_s;
                  pi[pino as usize].poc.compno0 = tcp.comp_t;
                  pi[pino as usize].poc.compno1 = tcp.comp_t.wrapping_add(1u32);
                  tcp.comp_t = (tcp.comp_t as core::ffi::c_uint).wrapping_add(1u32);
                  incr_top = 1_u32
                } else {
                  incr_top = 0_u32
                }
              } else {
                pi[pino as usize].poc.compno0 = tcp.comp_t;
                pi[pino as usize].poc.compno1 = tcp.comp_t.wrapping_add(1u32);
                tcp.comp_t = (tcp.comp_t as core::ffi::c_uint).wrapping_add(1u32);
                incr_top = 0_u32
              }
            }
            ProgressionStep::Layer => {
              if tcp.lay_t == tcp.lay_e {
                if pi_check_next_level(i - 1i32, &*tcp, prog).is_ok() {
                  tcp.lay_t = tcp.lay_s;
                  pi[pino as usize].poc.layno0 = tcp.lay_t;
                  pi[pino as usize].poc.layno1 = tcp.lay_t.wrapping_add(1u32);
                  tcp.lay_t = (tcp.lay_t as core::ffi::c_uint).wrapping_add(1u32);
                  incr_top = 1_u32
                } else {
                  incr_top = 0_u32
                }
              } else {
                pi[pino as usize].poc.layno0 = tcp.lay_t;
                pi[pino as usize].poc.layno1 = tcp.lay_t.wrapping_add(1u32);
                tcp.lay_t = (tcp.lay_t as core::ffi::c_uint).wrapping_add(1u32);
                incr_top = 0_u32
              }
            }
            ProgressionStep::Precinct => match tcp.prg as core::ffi::c_int {
              0 | 1 => {
                if tcp.prc_t == tcp.prc_e {
                  if pi_check_next_level(i - 1i32, &*tcp, prog).is_ok() {
                    tcp.prc_t = tcp.prc_s;
                    pi[pino as usize].poc.precno0 = tcp.prc_t;
                    pi[pino as usize].poc.precno1 = tcp.prc_t.wrapping_add(1u32);
                    tcp.prc_t = (tcp.prc_t as core::ffi::c_uint).wrapping_add(1u32);
                    incr_top = 1_u32
                  } else {
                    incr_top = 0_u32
                  }
                } else {
                  pi[pino as usize].poc.precno0 = tcp.prc_t;
                  pi[pino as usize].poc.precno1 = tcp.prc_t.wrapping_add(1u32);
                  tcp.prc_t = (tcp.prc_t as core::ffi::c_uint).wrapping_add(1u32);
                  incr_top = 0_u32
                }
              }
              _ => {
                if tcp.tx0_t >= tcp.tx_e {
                  if tcp.ty0_t >= tcp.ty_e {
                    if pi_check_next_level(i - 1i32, &*tcp, prog).is_ok() {
                      tcp.ty0_t = tcp.ty_s;
                      pi[pino as usize].poc.ty0 = tcp.ty0_t;
                      pi[pino as usize].poc.ty1 = tcp
                        .ty0_t
                        .wrapping_add(tcp.dy)
                        .wrapping_sub(tcp.ty0_t.wrapping_rem(tcp.dy));
                      tcp.ty0_t = pi[pino as usize].poc.ty1;
                      incr_top = 1_u32;
                      reset_x = 1_u32
                    } else {
                      incr_top = 0_u32;
                      reset_x = 0_u32
                    }
                  } else {
                    pi[pino as usize].poc.ty0 = tcp.ty0_t;
                    pi[pino as usize].poc.ty1 = tcp
                      .ty0_t
                      .wrapping_add(tcp.dy)
                      .wrapping_sub(tcp.ty0_t.wrapping_rem(tcp.dy));
                    tcp.ty0_t = pi[pino as usize].poc.ty1;
                    incr_top = 0_u32;
                    reset_x = 1_u32
                  }
                  if reset_x == 1u32 {
                    tcp.tx0_t = tcp.tx_s;
                    pi[pino as usize].poc.tx0 = tcp.tx0_t;
                    pi[pino as usize].poc.tx1 = tcp
                      .tx0_t
                      .wrapping_add(tcp.dx)
                      .wrapping_sub(tcp.tx0_t.wrapping_rem(tcp.dx));
                    tcp.tx0_t = pi[pino as usize].poc.tx1
                  }
                } else {
                  pi[pino as usize].poc.tx0 = tcp.tx0_t;
                  pi[pino as usize].poc.tx1 = tcp
                    .tx0_t
                    .wrapping_add(tcp.dx)
                    .wrapping_sub(tcp.tx0_t.wrapping_rem(tcp.dx));
                  tcp.tx0_t = pi[pino as usize].poc.tx1;
                  incr_top = 0_u32
                }
              }
            },
            _ => {}
          }
        }
        i -= 1
      }
    }
  };
}

pub(crate) fn pi_destroy(p_pi: &mut Vec<PiIterator>) {
  /* Clearing drops each element's own comps/resolutions Vecs and the
   * shared include Rc. */
  p_pi.clear();
}
pub(crate) fn pi_update_encoding_parameters(
  p_image: &mut CoreImage,
  p_cp: &mut Cp,
  p_tile_no: u32,
) {
  /* encoding parameters to set */
  let mut l_max_res: u32 = 0;
  let mut l_max_prec: u32 = 0;
  let mut l_tx0: u32 = 0;
  let mut l_tx1: u32 = 0;
  let mut l_ty0: u32 = 0;
  let mut l_ty1: u32 = 0;
  let mut l_dx_min: u32 = 0;
  let mut l_dy_min: u32 = 0;
  /* preconditions */

  assert!(p_tile_no < p_cp.tw.wrapping_mul(p_cp.th));
  /* get encoding parameters */
  get_encoding_parameters(
    p_image,
    p_cp,
    p_tile_no,
    &mut l_tx0,
    &mut l_tx1,
    &mut l_ty0,
    &mut l_ty1,
    &mut l_dx_min,
    &mut l_dy_min,
    &mut l_max_prec,
    &mut l_max_res,
  );
  if p_cp.tcps[p_tile_no as usize].poc {
    pi_update_encode_poc_and_final(
      p_cp, p_tile_no, l_tx0, l_tx1, l_ty0, l_ty1, l_max_prec, l_max_res, l_dx_min, l_dy_min,
    );
  } else {
    pi_update_encode_not_poc(
      p_cp,
      p_image.numcomps,
      p_tile_no,
      l_tx0,
      l_tx1,
      l_ty0,
      l_ty1,
      l_max_prec,
      l_max_res,
      l_dx_min,
      l_dy_min,
    );
  };
}

pub(crate) fn pi_next(pi: &mut PiIterator, manager: &mut EventMgr) -> OpResult {
  match pi.poc.prg as core::ffi::c_int {
    0 => return pi_next_lrcp(pi, manager),
    1 => return pi_next_rlcp(pi, manager),
    2 => return pi_next_rpcl(pi, manager),
    3 => return pi_next_pcrl(pi, manager),
    4 => return pi_next_cprl(pi, manager),
    -1 => return Err(()),
    _ => {}
  }
  Err(())
}
