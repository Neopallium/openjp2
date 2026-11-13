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
// Copyright (c) 2008, 2011-2012, Centre National d'Etudes Spatiales (CNES), FR
// Copyright (c) 2012, CS Systemes d'Information, France
// Copyright (c) 2017, IntoPIX SA <support@intopix.com>
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
//! Tier-2 codec: packet header parsing/writing (inclusion, zero
//! bit-planes, coding passes, segment lengths) and the packet data I/O.
//! See `doc/jpeg2000-design-spec.md` §3.8.
use super::bio::*;
use super::consts::*;
use super::event::*;
use super::j2k::PacketInfo;
use super::math::*;
use super::openjpeg::*;
use super::pi::*;
use super::tcd::*;
use super::tgt::*;
use crate::image::CoreImage;

#[repr(C)]
pub(crate) struct T2<'a> {
  pub image: &'a mut CoreImage,
  pub cp: &'a mut Cp,
}
//
//

// T2 - Implementation of a tier-2 coding

// -----------------------------------------------------------------------
fn t2_putcommacode(bio: &mut Bio, mut n: i32) {
  for _ in 0..n {
    bio_putbit(bio, 1);
  }
  bio_putbit(bio, 0);
}

fn t2_getcommacode(bio: &mut Bio) -> u32 {
  let mut n = 0_u32;
  while bio_read(bio, 1_u32) != 0 {
    /* The C's u32 counter wraps here (t2.c:175-182); match it exactly. */
    n = n.wrapping_add(1);
  }
  n
}

/// Variable length code for signalling delta Zil (truncation point)
/// `bio` -- Bit Input/Output component
/// `n` -- delta Zil
fn t2_putnumpasses(bio: &mut Bio, mut n: u32) {
  if n == 1u32 {
    bio_putbit(bio, 0);
  } else if n == 2u32 {
    bio_write(bio, 2_u32, 2_u32);
  } else if n <= 5u32 {
    bio_write(bio, 0xcu32 | n.wrapping_sub(3u32), 4_u32);
  } else if n <= 36u32 {
    bio_write(bio, 0x1e0u32 | n.wrapping_sub(6u32), 9_u32);
  } else if n <= 164u32 {
    bio_write(bio, 0xff80u32 | n.wrapping_sub(37u32), 16_u32);
  };
}

fn t2_getnumpasses(bio: &mut Bio) -> u32 {
  let mut n: u32 = 0;
  if bio_read(bio, 1_u32) == 0 {
    return 1_u32;
  }
  if bio_read(bio, 1_u32) == 0 {
    return 2_u32;
  }
  n = bio_read(bio, 2_u32);
  if n != 3u32 {
    return (3u32).wrapping_add(n);
  }
  n = bio_read(bio, 5_u32);
  if n != 31u32 {
    return (6u32).wrapping_add(n);
  }
  (37u32).wrapping_add(bio_read(bio, 7_u32))
}

// -----------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn t2_encode_packets(
  t2: &mut T2,
  mut p_tile_no: u32,
  tile: &mut TcdTile,
  mut p_maxlayers: u32,
  mut dest: &mut [u8],
  mut p_data_written: &mut u32,
  mut p_marker_info: Option<&mut TcdMarkerInfo>,
  mut p_tp_num: u32,
  mut p_tp_pos: i32,
  mut p_pino: u32,
  mut p_t2_mode: J2K_T2_MODE,
  mut p_manager: &mut EventMgr,
) -> OpResult {
  let mut dest = &mut dest[..];
  let image = &mut *t2.image;
  let cp = &mut *t2.cp;
  let mut l_nb_bytes = 0_u32;
  let mut compno: u32 = 0;
  let mut poc: u32 = 0;
  let pocno = if cp.rsiz as core::ffi::c_int == 0x4i32 {
    2i32
  } else {
    1i32
  } as u32;
  let l_max_comp = if cp.m_specific_param.m_enc.m_max_comp_size > 0u32 {
    image.numcomps
  } else {
    1u32
  };
  let mut pi_vec = pi_initialise_encode(image, cp, p_tile_no, p_t2_mode);
  if pi_vec.is_empty() {
    return Err(());
  }
  *p_data_written = 0_u32;
  if p_t2_mode as core::ffi::c_uint == THRESH_CALC as core::ffi::c_uint {
    /* Calculating threshold */
    compno = 0_u32;
    while compno < l_max_comp {
      let mut l_comp_len = 0_u32;
      poc = 0_u32;
      while poc < pocno {
        let mut l_tp_num = compno;
        /* Note: in v2 this cannot fail (allocations are infallible); the
        return value is kept for parity with v1 */
        pi_create_encode(
          pi_vec.as_mut_slice(),
          cp,
          p_tile_no,
          poc,
          l_tp_num,
          p_tp_pos,
          p_t2_mode,
        );
        /* (Invalid progression orders are rejected when the POC marker is
        read; the native enum cannot hold them.) */
        loop {
          let pi = &mut pi_vec[poc as usize];
          if pi_next(pi, p_manager).is_err() {
            break;
          }
          if pi.layno < p_maxlayers {
            l_nb_bytes = 0_u32;
            if t2_encode_packet(
              p_tile_no,
              tile,
              &mut cp.tcps[p_tile_no as usize],
              pi,
              &mut dest[..],
              &mut l_nb_bytes,
              p_t2_mode,
              p_manager,
            )
            .is_err()
            {
              pi_destroy(&mut pi_vec);
              return Err(());
            }
            l_comp_len = (l_comp_len as core::ffi::c_uint).wrapping_add(l_nb_bytes);
            let (_, rest) = dest.split_at_mut(l_nb_bytes as usize);
            dest = rest;
            *p_data_written = (*p_data_written as core::ffi::c_uint).wrapping_add(l_nb_bytes)
          }
        }
        if cp.m_specific_param.m_enc.m_max_comp_size != 0
          && l_comp_len > cp.m_specific_param.m_enc.m_max_comp_size
        {
          pi_destroy(&mut pi_vec);
          return Err(());
        }
        poc += 1;
      }
      compno += 1;
    }
  } else {
    pi_create_encode(
      pi_vec.as_mut_slice(),
      cp,
      p_tile_no,
      p_pino,
      p_tp_num,
      p_tp_pos,
      p_t2_mode,
    );
    /* (Invalid progression orders are rejected when the POC marker is
    read; the native enum cannot hold them.) */
    if let Some(marker_info) = p_marker_info.as_deref_mut() {
      if marker_info.need_plt {
        /* One time use intended */
        debug_assert!(marker_info.packet_count == 0u32);
        debug_assert!(marker_info.p_packet_size.is_empty());
        let count = get_encoding_packet_count(image, cp, p_tile_no) as usize;
        /* The C's malloc is checked; a crafted geometry can ask for up to
        2^32 entries here. */
        let mut sizes = Vec::new();
        if sizes.try_reserve_exact(count).is_err() {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Not enough memory for the PLT packet sizes\n",
          );
          return Err(());
        }
        sizes.resize(count, 0);
        marker_info.p_packet_size = sizes;
      }
    }
    loop {
      let pi = &mut pi_vec[p_pino as usize];
      if pi_next(pi, p_manager).is_err() {
        break;
      }
      if pi.layno < p_maxlayers {
        l_nb_bytes = 0_u32;
        if t2_encode_packet(
          p_tile_no,
          tile,
          &mut cp.tcps[p_tile_no as usize],
          pi,
          &mut dest[..],
          &mut l_nb_bytes,
          p_t2_mode,
          p_manager,
        )
        .is_err()
        {
          pi_destroy(&mut pi_vec);
          return Err(());
        }
        let (_, rest) = dest.split_at_mut(l_nb_bytes as usize);
        dest = rest;
        *p_data_written = (*p_data_written as core::ffi::c_uint).wrapping_add(l_nb_bytes);
        if let Some(marker_info) = p_marker_info.as_deref_mut() {
          if marker_info.need_plt {
            marker_info.p_packet_size[marker_info.packet_count as usize] = l_nb_bytes;
            marker_info.packet_count = marker_info.packet_count.wrapping_add(1)
          }
        }
        /* INDEX >> */
        /* << INDEX */
        tile.packno = tile.packno.wrapping_add(1)
      }
    }
  }
  pi_destroy(&mut pi_vec);
  Ok(())
}
pub(crate) fn t2_decode_packets(
  mut tcd: &mut Tcd,
  t2: &mut T2,
  mut p_tile_no: u32,
  mut src: &mut [u8],
  mut p_data_read: &mut u32,
  mut p_manager: &mut EventMgr,
  p_data_start: i64,
) -> Result<Vec<PacketInfo>, ()> {
  let mut packet_index: Vec<PacketInfo> = Vec::new();
  let p_data_read_initial = src.len();
  let image = &mut *t2.image;
  let cp = &mut *t2.cp;
  let mut src_base: usize = 0;
  let mut pino: u32 = 0;
  let mut l_nb_bytes_read: u32 = 0;
  /* create a packet iterator */
  let mut pi_vec = pi_create_decode(image, cp, p_tile_no);
  if pi_vec.is_empty() {
    return Err(());
  }
  let (num_layers_to_decode, numpocs) = {
    let tcp = &cp.tcps[p_tile_no as usize];
    (tcp.num_layers_to_decode, tcp.numpocs)
  };
  pino = 0_u32;
  while pino <= numpocs {
    /* if the resolution needed is too low, one dim of the tilec could be equal to zero
     * and no packets are used to decode this resolution and
     * l_current_pi->resno is always >= p_tile->comps[l_current_pi->compno].minimum_num_resolutions
     * and no l_img_comp->resno_decoded are computed
     */
    /* C t2.c: an invalid COD progression order fails the whole tile here.
    An invalid POC marker byte must NOT: its segment walks zero packets
    (pi_next returns Err -> break) and the other segments decode. */
    if pi_vec[pino as usize].prg_unknown_fatal {
      return Err(());
    }
    let mut first_pass_failed: Vec<bool> = vec![true; image.numcomps as usize];
    loop {
      let pi = &mut pi_vec[pino as usize];
      if pi_next(pi, p_manager).is_err() {
        break;
      }
      /* Lenient truncation short-circuit: when the tile data is exhausted
      every remaining packet reads zero bytes, so the C's loop burns the
      whole include-mask space without changing the output (minutes on
      crafted headers).  Breaking here produces the identical image.
      Strict mode keeps walking - its read path reports the truncation. */
      if src.is_empty() && !cp.strict {
        break;
      }
      let mut skip_packet = 0i32;
      log::debug!(
        "packet offset=00000166 prg={} cmptno={:02} rlvlno={:02} prcno={:03} lyrno={:02}",
        pi.poc.prg1 as core::ffi::c_int,
        pi.compno,
        pi.resno,
        pi.precno,
        pi.layno,
      );
      /* INDEX >> */
      /* << INDEX */
      if pi.layno >= num_layers_to_decode
        || pi.resno >= tcd.tcd_image.tiles.comps[pi.compno as usize].minimum_num_resolutions
      {
        skip_packet = 1i32
      } else {
        /* If the packet layer is greater or equal than the maximum */
        /* number of layers, skip the packet */
        /* If the packet resolution number is greater than the minimum */
        /* number of resolution allowed, skip the packet */
        /* If no precincts of any band intersects the area of interest, */
        /* skip the packet */
        let mut bandno: u32 = 0;
        let tilec = &tcd.tcd_image.tiles.comps[pi.compno as usize];
        let res = &tilec.resolutions[pi.resno as usize];
        skip_packet = 1i32;
        bandno = 0_u32;
        while bandno < res.numbands {
          let band = &res.bands[bandno as usize];
          if (pi.precno as usize) >= band.precincts.len() {
            event_msg!(p_manager, EVT_ERROR, "Invalid precinct\n",);
            pi_destroy(&mut pi_vec);
            return Err(());
          }
          let prec = &band.precincts[pi.precno as usize];
          if tcd_is_subband_area_of_interest(
            &TcdTilecView {
              win: [tcd.win_x0, tcd.win_y0, tcd.win_x1, tcd.win_y1],
              whole_tile_decoding: tcd.whole_tile_decoding,
              x0: tilec.x0,
              y0: tilec.y0,
              x1: tilec.x1,
              y1: tilec.y1,
              numresolutions: tilec.numresolutions,
              minimum_num_resolutions: tilec.minimum_num_resolutions,
              compno: pi.compno,
            },
            &cp.tcps[p_tile_no as usize].tccps[pi.compno as usize],
            &image.comps,
            pi.resno,
            band.bandno,
            prec.x0 as u32,
            prec.y0 as u32,
            prec.x1 as u32,
            prec.y1 as u32,
          ) {
            skip_packet = 0i32;
            break;
          } else {
            bandno += 1;
          }
        }
      }
      let mut l_header_len: u32 = 0;
      if skip_packet == 0 {
        l_nb_bytes_read = 0_u32;
        first_pass_failed[pi.compno as usize] = false;
        if t2_decode_packet(
          cp,
          p_tile_no,
          &mut tcd.tcd_image.tiles,
          pi,
          &mut src[..],
          src_base,
          &mut l_nb_bytes_read,
          &mut l_header_len,
          p_manager,
        )
        .is_err()
        {
          pi_destroy(&mut pi_vec);

          return Err(());
        }
        image.comps[pi.compno as usize].resno_decoded =
          uint_max(pi.resno, image.comps[pi.compno as usize].resno_decoded)
      } else {
        l_nb_bytes_read = 0_u32;
        if t2_skip_packet(
          cp,
          p_tile_no,
          &mut tcd.tcd_image.tiles,
          pi,
          &mut src[..],
          &mut l_nb_bytes_read,
          &mut l_header_len,
          p_manager,
        )
        .is_err()
        {
          pi_destroy(&mut pi_vec);

          return Err(());
        }
      }
      if first_pass_failed[pi.compno as usize]
        && image.comps[pi.compno as usize].resno_decoded == 0u32
      {
        image.comps[pi.compno as usize].resno_decoded = tcd.tcd_image.tiles.comps
          [pi.compno as usize]
          .minimum_num_resolutions
          .wrapping_sub(1u32)
      }
      let packet_start = p_data_start + src_base as i64;
      /* This index has no live C counterpart (the C's packet recording is
      #ifdef TODO_MSD).  Cap it so a truncated stream that yields millions
      of zero-byte packets cannot abort the process in an infallible Vec
      growth; the decode itself continues. */
      const MAX_RECORDED_PACKETS: usize = 1 << 20;
      if packet_index.len() < MAX_RECORDED_PACKETS {
        packet_index.push(PacketInfo {
          start_pos: packet_start,
          end_ph_pos: packet_start + l_header_len as i64,
          end_pos: packet_start + l_nb_bytes_read as i64,
          disto: 0.0,
          layno: pi.layno,
          resno: pi.resno,
          compno: pi.compno,
          precno: pi.precno,
        });
      }
      let (_, rest) = src.split_at_mut(l_nb_bytes_read as usize);
      src = rest;
      src_base += l_nb_bytes_read as usize;
    }
    pino += 1;
  }
  /* don't forget to release pi */
  pi_destroy(&mut pi_vec);
  *p_data_read = (p_data_read_initial - src.len()) as u32;
  Ok(packet_index)
}
pub(crate) fn t2_create<'a>(image: &'a mut CoreImage, cp: &'a mut Cp) -> Box<T2<'a>> {
  /* create the t2 structure */
  Box::new(T2 { image, cp })
}

/// Decode a packet of a tile from a source buffer
/// `t2` -- T2 handle
/// `tile` -- Tile for which to write the packets
/// `tcp` -- Tile coding parameters
/// `pi` -- Packet identity
/// `src` -- Source buffer
/// `data_read`
/// `max_length`
/// `pack_info` -- Packet information
/// `p_manager` -- the user event manager
fn t2_decode_packet(
  cp: &mut Cp,
  p_tile_no: u32,
  tile: &mut TcdTile,
  pi: &mut PiIterator,
  src: &mut [u8],
  src_base: usize,
  p_data_read: &mut u32,
  p_header_len: &mut u32,
  p_manager: &mut EventMgr,
) -> OpResult {
  let mut l_read_data: bool = false;
  let mut l_nb_bytes_read = 0_u32;
  let mut l_nb_total_bytes_read = 0_u32;
  *p_data_read = 0_u32;
  if t2_read_packet_header(
    cp,
    p_tile_no,
    tile,
    pi,
    &mut l_read_data,
    src,
    &mut l_nb_bytes_read,
    p_manager,
  )
  .is_err()
  {
    return Err(());
  }
  *p_header_len = l_nb_bytes_read;
  let src = &mut src[l_nb_bytes_read as usize..];
  let src_base = src_base + l_nb_bytes_read as usize;
  l_nb_total_bytes_read =
    (l_nb_total_bytes_read as core::ffi::c_uint).wrapping_add(l_nb_bytes_read);
  /* we should read data for the packet */
  if l_read_data {
    l_nb_bytes_read = 0_u32;
    if t2_read_packet_data(cp, tile, pi, src, src_base, &mut l_nb_bytes_read, p_manager).is_err() {
      return Err(());
    }
    l_nb_total_bytes_read =
      (l_nb_total_bytes_read as core::ffi::c_uint).wrapping_add(l_nb_bytes_read)
  }
  *p_data_read = l_nb_total_bytes_read;
  Ok(())
}

/// Encode a packet of a tile to a destination buffer
/// `tileno` -- Number of the tile encoded
/// `tile` -- Tile for which to write the packets
/// `tcp` -- Tile coding parameters
/// `pi` -- Packet identity
/// `dest` -- Destination buffer
/// `p_data_written`
/// `len` -- Length of the destination buffer
/// `cstr_info` -- Codestream information structure
/// `p_t2_mode` -- If == THRESH_CALC In threshold calculation ,If == FINAL_PASS Final pass
/// `p_manager` -- the user event manager
fn t2_skip_packet(
  cp: &mut Cp,
  p_tile_no: u32,
  tile: &mut TcdTile,
  pi: &mut PiIterator,
  src: &mut [u8],
  p_data_read: &mut u32,
  p_header_len: &mut u32,
  p_manager: &mut EventMgr,
) -> OpResult {
  let mut l_read_data: bool = false;
  let mut l_nb_bytes_read = 0_u32;
  let mut l_nb_total_bytes_read = 0_u32;
  *p_data_read = 0_u32;
  if t2_read_packet_header(
    cp,
    p_tile_no,
    tile,
    pi,
    &mut l_read_data,
    src,
    &mut l_nb_bytes_read,
    p_manager,
  )
  .is_err()
  {
    return Err(());
  }
  *p_header_len = l_nb_bytes_read;
  let (_, src_rest) = src.split_at_mut(l_nb_bytes_read as usize);
  l_nb_total_bytes_read =
    (l_nb_total_bytes_read as core::ffi::c_uint).wrapping_add(l_nb_bytes_read);
  let p_max_length = src_rest.len() as u32;
  /* we should read data for the packet */
  if l_read_data {
    l_nb_bytes_read = 0_u32;
    if t2_skip_packet_data(cp, tile, pi, &mut l_nb_bytes_read, p_max_length, p_manager).is_err() {
      return Err(());
    }
    l_nb_total_bytes_read =
      (l_nb_total_bytes_read as core::ffi::c_uint).wrapping_add(l_nb_bytes_read)
  }
  *p_data_read = l_nb_total_bytes_read;
  Ok(())
}
fn t2_encode_packet(
  _tileno: u32,
  tile: &mut TcdTile,
  tcp: &mut Tcp,
  pi: &mut PiIterator,
  mut dest: &mut [u8],
  p_data_written: &mut u32,
  mut p_t2_mode: J2K_T2_MODE,
  mut p_manager: &mut EventMgr,
) -> OpResult {
  let mut bandno: u32 = 0; /* component value */
  let mut cblkno: u32 = 0; /* resolution level value */
  let mut pos = 0usize; /* precinct value */
  let mut length = dest.len() as u32;
  let mut l_nb_bytes: u32 = 0; /* quality layer value */
  let compno = pi.compno; /* BIO component */
  let resno = pi.resno;
  let precno = pi.precno;
  let layno = pi.layno;
  let mut l_nb_blocks: u32 = 0;
  let mut packet_empty = 0i32;
  let packno = tile.packno;
  /* <SOP 0xff91> */
  if tcp.csty & J2K_CP_CSTY_SOP != 0 {
    if length < 6u32 {
      if p_t2_mode as core::ffi::c_uint == FINAL_PASS as core::ffi::c_uint {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "t2_encode_packet(): only {} bytes remaining in output buffer. {} needed.\n",
          length,
          6i32,
        ); /* packno is uint32_t */
      }
      return Err(());
    }
    dest[0] = 255_u8;
    dest[1] = 145_u8;
    dest[2] = 0_u8;
    dest[3] = 4_u8;
    dest[4] = (packno >> 8i32 & 0xffu32) as u8;
    dest[5] = (packno & 0xffu32) as u8;
    pos += 6;
    length = (length as core::ffi::c_uint).wrapping_sub(6u32)
  }
  /* </SOP> */
  let tilec = &mut tile.comps[compno as usize];
  let res = &mut tilec.resolutions[resno as usize];
  if layno == 0 {
    bandno = 0_u32;
    while bandno < res.numbands {
      let band = &mut res.bands[bandno as usize];
      /* Skip empty bands */
      if !tcd_is_band_empty(band) {
        /* Avoid out of bounds access of https://github.com/uclouvain/openjpeg/issues/1294 */
        /* but likely not a proper fix. */
        if precno >= res.pw.wrapping_mul(res.ph) {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "t2_encode_packet(): accessing precno={} >= {}\n",
            precno,
            res.pw.wrapping_mul(res.ph),
          );
          return Err(());
        }
        let prc = &mut band.precincts[precno as usize];
        /* A degenerate precinct (cw*ch == 0) has no tag trees; the C code no-ops here. */
        if let Some(tree) = prc.incltree.as_mut() {
          tgt_reset(tree);
        }
        if let Some(tree) = prc.imsbtree.as_mut() {
          tgt_reset(tree);
        }
        l_nb_blocks = prc.cw.wrapping_mul(prc.ch);
        cblkno = 0_u32;
        while cblkno < l_nb_blocks {
          let cblk = prc.cblks.enc_mut(cblkno as usize);
          cblk.numpasses = 0_u32;
          // SAFETY: imsbtree is the precinct's tag tree, owned by the precinct.
          tgt_setvalue(
            prc.imsbtree.as_mut().expect("imsbtree"),
            cblkno,
            band.numbps - cblk.numbps as i32,
          );
          cblkno += 1;
        }
      }
      bandno = bandno.wrapping_add(1);
    }
  }
  let mut bio = Bio::new_enc(&mut dest[pos..]);
  let bio = &mut bio;
  /* Empty header bit */
  bio_putbit(bio, if packet_empty != 0 { 0 } else { 1 });

  /* Writing Packet header */
  bandno = 0_u32;
  while packet_empty == 0 && bandno < res.numbands {
    let band = &mut res.bands[bandno as usize];
    /* Skip empty bands */
    if !tcd_is_band_empty(band) {
      /* Avoid out of bounds access of https://github.com/uclouvain/openjpeg/issues/1297 */
      /* but likely not a proper fix. */
      if precno >= res.pw.wrapping_mul(res.ph) {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "t2_encode_packet(): accessing precno={} >= {}\n",
          precno,
          res.pw.wrapping_mul(res.ph),
        );
        return Err(());
      }
      let prc = &mut band.precincts[precno as usize];
      l_nb_blocks = prc.cw.wrapping_mul(prc.ch);
      cblkno = 0_u32;
      while cblkno < l_nb_blocks {
        let cblk = prc.cblks.enc_mut(cblkno as usize);
        let layer = &mut cblk.layers[layno as usize];
        if cblk.numpasses == 0 && layer.numpasses != 0 {
          // SAFETY: incltree is the precinct's tag tree, owned by the precinct.
          tgt_setvalue(
            prc.incltree.as_mut().expect("incltree"),
            cblkno,
            layno as i32,
          );
        }
        cblkno += 1;
      }
      cblkno = 0_u32;
      while cblkno < l_nb_blocks {
        let cblk = prc.cblks.enc_mut(cblkno as usize);
        let mut increment = 0_u32;
        let mut nump = 0_u32;
        let mut len = 0_u32;
        let mut passno: u32 = 0;
        let mut l_nb_passes: u32 = 0;
        /* cblk inclusion bits */
        if cblk.numpasses == 0 {
          tgt_encode(
            bio,
            prc.incltree.as_mut().expect("incltree"),
            cblkno,
            layno.wrapping_add(1u32) as i32,
          );
        } else {
          bio_putbit(bio, (cblk.layers[layno as usize].numpasses != 0u32) as u32);
        }
        /* if cblk not included, go to the next cblk  */
        if cblk.layers[layno as usize].numpasses == 0 {
        } else {
          /* if first instance of cblk --> zero bit-planes information */
          if cblk.numpasses == 0 {
            cblk.numlenbits = 3_u32;
            tgt_encode(
              bio,
              prc.imsbtree.as_mut().expect("imsbtree"),
              cblkno,
              999i32,
            );
          }
          /* number of coding passes included */
          t2_putnumpasses(bio, cblk.layers[layno as usize].numpasses);
          l_nb_passes = cblk
            .numpasses
            .wrapping_add(cblk.layers[layno as usize].numpasses);
          /* computation of the increase of the length indicator and insertion in the header     */
          passno = cblk.numpasses;
          while passno < l_nb_passes {
            nump = nump.wrapping_add(1);
            len = (len as core::ffi::c_uint).wrapping_add(cblk.passes[passno as usize].len);
            if cblk.passes[passno as usize].term
              || passno
                == cblk
                  .numpasses
                  .wrapping_add(cblk.layers[layno as usize].numpasses)
                  .wrapping_sub(1u32)
            {
              increment = int_max(
                increment as i32,
                int_floorlog2(len as i32) + 1i32
                  - (cblk.numlenbits as i32 + int_floorlog2(nump as i32)),
              ) as u32;
              len = 0_u32;
              nump = 0_u32
            }
            passno += 1;
          }
          t2_putcommacode(bio, increment as i32);
          /* computation of the new Length indicator */
          cblk.numlenbits = (cblk.numlenbits as core::ffi::c_uint).wrapping_add(increment);
          /* insertion of the codeword segment length */
          passno = cblk.numpasses;
          while passno < l_nb_passes {
            nump = nump.wrapping_add(1);
            len = (len as core::ffi::c_uint).wrapping_add(cblk.passes[passno as usize].len);
            if cblk.passes[passno as usize].term
              || passno
                == cblk
                  .numpasses
                  .wrapping_add(cblk.layers[layno as usize].numpasses)
                  .wrapping_sub(1u32)
            {
              bio_write(
                bio,
                len,
                cblk
                  .numlenbits
                  .wrapping_add(int_floorlog2(nump as i32) as u32),
              );
              len = 0_u32;
              nump = 0_u32
            }
            passno += 1;
          }
        }
        cblkno += 1;
      }
    }
    bandno = bandno.wrapping_add(1);
  }
  if bio_flush(bio).is_err() {
    return Err(());
    /* modified to eliminate longjmp !! */
  }
  l_nb_bytes = bio_numbytes(bio) as u32;
  pos += l_nb_bytes as usize;
  length = (length as core::ffi::c_uint).wrapping_sub(l_nb_bytes);
  /* <EPH 0xff92> */
  if tcp.csty & J2K_CP_CSTY_EPH != 0 {
    if length < 2u32 {
      if p_t2_mode as core::ffi::c_uint == FINAL_PASS as core::ffi::c_uint {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "t2_encode_packet(): only {} bytes remaining in output buffer. {} needed.\n",
          length,
          2i32,
        );
      }
      return Err(());
    }
    dest[pos] = 255_u8;
    dest[pos + 1] = 146_u8;
    pos += 2;
    length = (length as core::ffi::c_uint).wrapping_sub(2u32)
  }
  /* </EPH> */
  /* << INDEX */
  /* End of packet header position. Currently only represents the distance to start of packet
  Will be updated later by incrementing with packet start value*/
  /* INDEX >> */
  /* Writing the packet body */
  bandno = 0_u32;
  while packet_empty == 0 && bandno < res.numbands {
    let band = &mut res.bands[bandno as usize];
    /* Skip empty bands */
    if !tcd_is_band_empty(band) {
      let prc = &mut band.precincts[precno as usize];
      l_nb_blocks = prc.cw.wrapping_mul(prc.ch);
      cblkno = 0_u32;
      while cblkno < l_nb_blocks {
        let cblk = prc.cblks.enc_mut(cblkno as usize);
        if cblk.layers[layno as usize].numpasses == 0 {
        } else {
          if cblk.layers[layno as usize].len > length {
            if p_t2_mode as core::ffi::c_uint == FINAL_PASS as core::ffi::c_uint {
              event_msg!(
                p_manager,
                EVT_ERROR,
                "t2_encode_packet(): only {} bytes remaining in output buffer. {} needed.\n",
                length,
                cblk.layers[layno as usize].len
              );
            }
            return Err(());
          }

          if p_t2_mode == FINAL_PASS {
            let len = cblk.layers[layno as usize].len as usize;
            let off = cblk.layers[layno as usize].data_offset;
            dest[pos..pos + len].copy_from_slice(&cblk.data[off..off + len]);
          }
          cblk.numpasses = (cblk.numpasses as core::ffi::c_uint)
            .wrapping_add(cblk.layers[layno as usize].numpasses);
          pos += cblk.layers[layno as usize].len as usize;
          length = (length as core::ffi::c_uint).wrapping_sub(cblk.layers[layno as usize].len);
          /* INDEX >> */
          /* << INDEX */
        }
        cblkno += 1;
      }
    }
    bandno += 1;
  }
  *p_data_written = pos as u32;
  Ok(())
}
fn t2_read_packet_header(
  cp: &mut Cp,
  p_tile_no: u32,
  tile: &mut TcdTile,
  pi: &mut PiIterator,
  p_is_data_present: &mut bool,
  p_src_data: &mut [u8],
  p_data_read: &mut u32,
  p_manager: &mut EventMgr,
) -> OpResult {
  /* loop */
  let mut bandno: u32 = 0; /* BIO component */
  let mut cblkno: u32 = 0;
  let mut l_nb_code_blocks: u32 = 0;
  let mut l_header_length: u32 = 0;
  let p_max_length = p_src_data.len() as u32;
  let mut l_pos = 0usize;
  let l_cp = cp;
  let csty = l_cp.tcps[p_tile_no as usize].csty;
  let tccp_cblksty = l_cp.tcps[p_tile_no as usize].tccps[pi.compno as usize].cblksty;
  let res = &mut tile.comps[pi.compno as usize].resolutions[pi.resno as usize];
  let mut l_present: u32 = 0;
  if pi.layno == 0u32 {
    /* reset tagtrees */
    bandno = 0_u32;
    while bandno < res.numbands {
      let band = &mut res.bands[bandno as usize];
      if !tcd_is_band_empty(band) {
        if (pi.precno as usize) >= band.precincts.len() {
          event_msg!(p_manager, EVT_ERROR, "Invalid precinct\n",);
          return Err(());
        }
        let prc = &mut band.precincts[pi.precno as usize];
        /* A degenerate precinct (cw*ch == 0) has no tag trees; the C code no-ops here. */
        if let Some(tree) = prc.incltree.as_mut() {
          tgt_reset(tree);
        }
        if let Some(tree) = prc.imsbtree.as_mut() {
          tgt_reset(tree);
        }
        l_nb_code_blocks = prc.cw.wrapping_mul(prc.ch);
        cblkno = 0_u32;
        while cblkno < l_nb_code_blocks {
          let cblk = prc.cblks.dec_mut(cblkno as usize);
          cblk.numsegs = 0_u32;
          cblk.real_num_segs = 0_u32;
          cblkno += 1;
        }
      }
      bandno += 1;
    }
  }

  /* SOP markers */
  if csty & J2K_CP_CSTY_SOP != 0 {
    if p_max_length < 6u32 {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "Not enough space for expected SOP marker\n",
      );
    } else if p_src_data[l_pos] as core::ffi::c_int != 0xffi32
      || p_src_data[l_pos + 1] as core::ffi::c_int != 0x91i32
    {
      event_msg!(p_manager, EVT_WARNING, "Expected SOP marker\n",);
    } else {
      l_pos += 6
    }
    /* Note (work plan): the Nsop value is not validated */
  }

  /*
  When the marker PPT/PPM is used the packet header are store in PPT/PPM marker
  This part deal with this characteristic
  step 1: Read packet header in the saved structure
  step 2: Return to codestream for decoding
  */
  /* The header lives either in the PPM buffer, the PPT buffer, or inline in the codestream. */
  let l_header: &mut [u8] = if l_cp.ppm {
    &mut l_cp.ppm_buffer[l_cp.ppm_data_pos..][..l_cp.ppm_len as usize]
  } else if l_cp.tcps[p_tile_no as usize].ppt {
    let tcp = &mut l_cp.tcps[p_tile_no as usize];
    &mut tcp.ppt_buffer[tcp.ppt_data_pos..][..tcp.ppt_len as usize]
  } else {
    &mut p_src_data[l_pos..]
  };
  let mut l_hdr_pos = 0usize;

  let mut l_bio = Bio::new_dec(&mut l_header[..]);
  let l_bio = &mut l_bio;

  l_present = bio_read(l_bio, 1_u32);
  log::debug!("present={}", l_present);
  if l_present == 0 {
    /* Note: no output check here (parity with upstream) */
    let _ = bio_inalign(l_bio);
    l_hdr_pos += bio_numbytes(l_bio) as usize;
    /* EPH markers */
    if csty & J2K_CP_CSTY_EPH != 0 {
      /* EPH markers are required when specified (upstream #1547) */
      if l_header.len() - l_hdr_pos < 2usize {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Not enough space for required EPH marker\n",
        );
        return Err(());
      } else if l_header[l_hdr_pos] as core::ffi::c_int != 0xffi32
        || l_header[l_hdr_pos + 1] as core::ffi::c_int != 0x92i32
      {
        event_msg!(p_manager, EVT_ERROR, "Expected EPH marker\n",);
        return Err(());
      } else {
        l_hdr_pos += 2
      }
    }
    l_header_length = l_hdr_pos as u32;
    if l_cp.ppm {
      l_cp.ppm_data_pos += l_header_length as usize;
      l_cp.ppm_len -= l_header_length;
    } else if l_cp.tcps[p_tile_no as usize].ppt {
      let tcp = &mut l_cp.tcps[p_tile_no as usize];
      tcp.ppt_data_pos += l_header_length as usize;
      tcp.ppt_len -= l_header_length;
    } else {
      l_pos += l_header_length as usize;
    }
    /* << INDEX */
    /* End of packet header position. Currently only represents the distance to start of packet
    Will be updated later by incrementing with packet start value */
    /* INDEX >> */
    *p_is_data_present = false;
    *p_data_read = l_pos as u32;
    return Ok(());
  }

  bandno = 0_u32;
  while bandno < res.numbands {
    let band = &mut res.bands[bandno as usize];
    if !tcd_is_band_empty(band) {
      if (pi.precno as usize) >= band.precincts.len() {
        event_msg!(p_manager, EVT_ERROR, "Invalid precinct\n",);
        return Err(());
      }
      let prc = &mut band.precincts[pi.precno as usize];
      l_nb_code_blocks = prc.cw.wrapping_mul(prc.ch);
      cblkno = 0_u32;
      while cblkno < l_nb_code_blocks {
        let mut l_included: u32 = 0;
        let mut l_increment: u32 = 0;
        let mut l_segno: u32 = 0;
        let mut n: i32 = 0;
        let cblk = prc.cblks.dec_mut(cblkno as usize);

        /* if cblk not yet included before --> inclusion tagtree */
        if cblk.numsegs == 0 {
          l_included = tgt_decode(
            l_bio,
            prc.incltree.as_mut().expect("incltree"),
            cblkno,
            pi.layno.wrapping_add(1u32) as i32,
          )
        /* else one bit */
        } else {
          l_included = bio_read(l_bio, 1_u32)
        }

        /* if cblk not included */
        if l_included == 0 {
          cblk.numnewpasses = 0_u32;
          log::debug!("included={}", l_included);
        } else {
          /* if cblk not yet included --> zero-bitplane tagtree */
          if cblk.numsegs == 0 {
            let mut i = 0_u32;
            while tgt_decode(
              l_bio,
              // SAFETY: imsbtree is the precinct's tag tree, owned by the precinct.
              prc.imsbtree.as_mut().expect("imsbtree"),
              cblkno,
              i as i32,
            ) == 0
            {
              i += 1;
            }
            cblk.mb = band.numbps as u32;
            /* Both C branches compute the same wrapping arithmetic; the
             * branch exists upstream only to document the PR #1488 integer
             * overflow while keeping the regression suite happy. */
            cblk.numbps = band.numbps.wrapping_add(1).wrapping_sub(i as i32) as u32;
            cblk.numlenbits = 3_u32
          }
          /* number of coding passes */
          cblk.numnewpasses = t2_getnumpasses(l_bio);
          l_increment = t2_getcommacode(l_bio);
          /* length indicator increment */
          cblk.numlenbits = (cblk.numlenbits as core::ffi::c_uint).wrapping_add(l_increment);
          l_segno = 0_u32;
          if cblk.numsegs == 0 {
            if t2_init_seg(cblk, l_segno, tccp_cblksty, 1_u32).is_err() {
              return Err(());
            }
          } else {
            l_segno = cblk.numsegs.wrapping_sub(1u32);
            if cblk.segs[l_segno as usize].numpasses == cblk.segs[l_segno as usize].maxpasses {
              l_segno = l_segno.wrapping_add(1);
              if t2_init_seg(cblk, l_segno, tccp_cblksty, 0_u32).is_err() {
                return Err(());
              }
            }
          }
          n = cblk.numnewpasses as i32;
          if tccp_cblksty & 0x40u32 != 0u32 {
            loop {
              let mut bit_number: u32 = 0;
              cblk.segs[l_segno as usize].numnewpasses =
                if l_segno == 0u32 { 1u32 } else { n as u32 };
              bit_number = cblk
                .numlenbits
                .wrapping_add(uint_floorlog2(cblk.segs[l_segno as usize].numnewpasses));
              if bit_number > 32u32 {
                event_msg!(
                  p_manager,
                  EVT_ERROR,
                  "Invalid bit number {} in t2_read_packet_header()\n",
                  bit_number,
                );
                return Err(());
              }
              cblk.segs[l_segno as usize].newlen = bio_read(l_bio, bit_number);
              log::debug!(
                "included={} numnewpasses={} increment={} len={}",
                l_included,
                cblk.segs[l_segno as usize].numnewpasses,
                l_increment,
                cblk.segs[l_segno as usize].newlen,
              );
              n -= cblk.segs[l_segno as usize].numnewpasses as i32;
              if n > 0i32 {
                l_segno = l_segno.wrapping_add(1);
                if t2_init_seg(cblk, l_segno, tccp_cblksty, 0_u32).is_err() {
                  return Err(());
                }
              }
              if n <= 0i32 {
                break;
              }
            }
          } else {
            loop {
              let mut bit_number_0: u32 = 0;
              cblk.segs[l_segno as usize].numnewpasses = int_min(
                cblk.segs[l_segno as usize]
                  .maxpasses
                  .wrapping_sub(cblk.segs[l_segno as usize].numpasses) as i32,
                n,
              ) as u32;
              bit_number_0 = cblk
                .numlenbits
                .wrapping_add(uint_floorlog2(cblk.segs[l_segno as usize].numnewpasses));
              if bit_number_0 > 32u32 {
                event_msg!(
                  p_manager,
                  EVT_ERROR,
                  "Invalid bit number {} in t2_read_packet_header()\n",
                  bit_number_0,
                );
                return Err(());
              }
              cblk.segs[l_segno as usize].newlen = bio_read(l_bio, bit_number_0);
              log::debug!(
                "included={} numnewpasses={} increment={} len={}",
                l_included,
                cblk.segs[l_segno as usize].numnewpasses,
                l_increment,
                cblk.segs[l_segno as usize].newlen,
              );
              n -= cblk.segs[l_segno as usize].numnewpasses as i32;
              if n > 0i32 {
                l_segno = l_segno.wrapping_add(1);
                if t2_init_seg(cblk, l_segno, tccp_cblksty, 0_u32).is_err() {
                  return Err(());
                }
              }
              if n <= 0i32 {
                break;
              }
            }
          }
        }
        cblkno += 1;
      }
    }
    bandno = bandno.wrapping_add(1);
  }
  if bio_inalign(l_bio).is_err() {
    return Err(());
  }
  l_hdr_pos += bio_numbytes(l_bio) as usize;
  /* EPH markers */
  if csty & J2K_CP_CSTY_EPH != 0 {
    /* EPH markers are required when specified (upstream #1547) */
    if l_header.len() - l_hdr_pos < 2usize {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Not enough space for required EPH marker\n",
      );
      return Err(());
    } else if l_header[l_hdr_pos] as core::ffi::c_int != 0xffi32
      || l_header[l_hdr_pos + 1] as core::ffi::c_int != 0x92i32
    {
      event_msg!(p_manager, EVT_ERROR, "Expected EPH marker\n",);
      return Err(());
    } else {
      l_hdr_pos += 2
    }
  }
  l_header_length = l_hdr_pos as u32;
  /* Do not allow a zero-length header in a non-zero-length packet
  (upstream #1526) */
  if l_header_length == 0 {
    return Err(());
  }
  log::debug!("hdrlen={}", l_header_length);
  log::debug!("packet body");
  if l_cp.ppm {
    l_cp.ppm_data_pos += l_header_length as usize;
    l_cp.ppm_len -= l_header_length;
  } else if l_cp.tcps[p_tile_no as usize].ppt {
    let tcp = &mut l_cp.tcps[p_tile_no as usize];
    tcp.ppt_data_pos += l_header_length as usize;
    tcp.ppt_len -= l_header_length;
  } else {
    l_pos += l_header_length as usize;
  }
  /* << INDEX */
  /* End of packet header position. Currently only represents the distance to start of packet
  Will be updated later by incrementing with packet start value */
  /* INDEX >> */
  *p_is_data_present = true; /* next code_block */
  *p_data_read = l_pos as u32;
  Ok(())
}
fn t2_read_packet_data(
  cp: &mut Cp,
  tile: &mut TcdTile,
  pi: &mut PiIterator,
  p_src_data: &[u8],
  src_base: usize,
  p_data_read: &mut u32,
  p_manager: &mut EventMgr,
) -> OpResult {
  let mut bandno: u32 = 0;
  let mut cblkno: u32 = 0;
  let mut l_nb_code_blocks: u32 = 0;
  let p_max_length = p_src_data.len() as u32;
  let mut l_pos = 0usize;
  let res = &mut tile.comps[pi.compno as usize].resolutions[pi.resno as usize];
  let mut partial_buffer = 0i32;
  bandno = 0_u32;
  while bandno < res.numbands {
    let band = &mut res.bands[bandno as usize];
    if band.x1 - band.x0 == 0i32 || band.y1 - band.y0 == 0i32 {
      bandno += 1;
      continue;
    }
    if (pi.precno as usize) >= band.precincts.len() {
      event_msg!(p_manager, EVT_ERROR, "Invalid precinct\n",);
      return Err(());
    }
    let prc = &mut band.precincts[pi.precno as usize];
    l_nb_code_blocks = prc.cw.wrapping_mul(prc.ch);
    cblkno = 0_u32;
    while cblkno < l_nb_code_blocks {
      let cblk = prc.cblks.dec_mut(cblkno as usize);
      if cblk.numnewpasses == 0 {
        /* nothing to do */
        cblkno += 1;
        continue;
      }
      // if a previous segment in this packet couldn't be decoded, or if this
      // code block was corrupted in a previous layer, then mark it corrupted
      // since we have no data to actually decode (upstream #1536).
      if partial_buffer != 0 || cblk.corrupted {
        cblk.numchunks = 0_u32;
        cblk.corrupted = true;
        cblkno += 1;
        continue;
      }
      {
        let mut l_segno: usize;
        if cblk.numsegs == 0 {
          l_segno = 0;
          cblk.numsegs = cblk.numsegs.wrapping_add(1)
        } else {
          l_segno = cblk.numsegs.wrapping_sub(1u32) as usize;
          if cblk.segs[l_segno].numpasses == cblk.segs[l_segno].maxpasses {
            l_segno += 1;
            cblk.numsegs = cblk.numsegs.wrapping_add(1)
          }
        }
        loop
        /* Check the remaining size of the source buffer */
        {
          let seg = &mut cblk.segs[l_segno];
          if seg.newlen as usize > p_src_data.len() - l_pos || partial_buffer != 0 {
            if cp.strict {
              event_msg!(p_manager, EVT_ERROR,
                                            "read: segment too long ({}) with max ({}) for codeblock {} (p={}, b={}, r={}, c={})\n",
                                            seg.newlen, p_max_length,
                                            cblkno, pi.precno, bandno,
                                            pi.resno, pi.compno);
              return Err(());
            } else {
              event_msg!(p_manager, EVT_WARNING,
                                            "read: segment too long ({}) with max ({}) for codeblock {} (p={}, b={}, r={}, c={})\n",
                                            seg.newlen, p_max_length,
                                            cblkno, pi.precno, bandno,
                                            pi.resno, pi.compno);
              /* skip this codeblock (and the following ones in this packet)
              since it is a partial read (upstream #1533) */
              partial_buffer = 1i32;
              cblk.corrupted = true;
              cblk.numchunks = 0_u32;
              break;
            }
          } else {
            if cblk.numchunks as usize == cblk.chunks.len() {
              let new_len = cblk.chunks.len() * 2 + 1;
              cblk.chunks.resize(
                new_len,
                TcdSegDataChunk {
                  data_offset: 0,
                  len: 0,
                },
              );
            }
            let chunk = &mut cblk.chunks[cblk.numchunks as usize];
            chunk.data_offset = src_base + l_pos;
            chunk.len = seg.newlen;
            cblk.numchunks = cblk.numchunks.wrapping_add(1);
            l_pos += seg.newlen as usize;
            seg.len = (seg.len as core::ffi::c_uint).wrapping_add(seg.newlen);
            seg.numpasses = (seg.numpasses as core::ffi::c_uint).wrapping_add(seg.numnewpasses);
            cblk.numnewpasses =
              (cblk.numnewpasses as core::ffi::c_uint).wrapping_sub(seg.numnewpasses);
            seg.real_num_passes = seg.numpasses;
            if cblk.numnewpasses > 0u32 {
              l_segno += 1;
              cblk.numsegs = cblk.numsegs.wrapping_add(1)
            }
          }
          if cblk.numnewpasses == 0u32 {
            break;
          }
        }
        cblk.real_num_segs = cblk.numsegs;
      }
      cblkno += 1;
    }
    bandno += 1;
  }
  // return the number of bytes read
  if partial_buffer != 0 {
    *p_data_read = p_max_length
  } else {
    *p_data_read = l_pos as u32
  }
  Ok(())
}

fn t2_skip_packet_data(
  cp: &mut Cp,
  tile: &mut TcdTile,
  pi: &mut PiIterator,
  p_data_read: &mut u32,
  p_max_length: u32,
  p_manager: &mut EventMgr,
) -> OpResult {
  let mut bandno: u32 = 0;
  let mut cblkno: u32 = 0;
  let mut l_nb_code_blocks: u32 = 0;
  let res = &mut tile.comps[pi.compno as usize].resolutions[pi.resno as usize];
  *p_data_read = 0_u32;
  bandno = 0_u32;
  while bandno < res.numbands {
    let band = &mut res.bands[bandno as usize];
    if band.x1 - band.x0 == 0i32 || band.y1 - band.y0 == 0i32 {
      bandno += 1;
      continue;
    }
    if (pi.precno as usize) >= band.precincts.len() {
      event_msg!(p_manager, EVT_ERROR, "Invalid precinct\n",);
      return Err(());
    }
    let prc = &mut band.precincts[pi.precno as usize];
    l_nb_code_blocks = prc.cw.wrapping_mul(prc.ch);
    cblkno = 0_u32;
    while cblkno < l_nb_code_blocks {
      let cblk = prc.cblks.dec_mut(cblkno as usize);
      if cblk.numnewpasses == 0 {
        /* nothing to do */
      } else {
        let mut l_segno: usize;
        if cblk.numsegs == 0 {
          l_segno = 0;
          cblk.numsegs = cblk.numsegs.wrapping_add(1)
        } else {
          l_segno = cblk.numsegs.wrapping_sub(1u32) as usize;
          if cblk.segs[l_segno].numpasses == cblk.segs[l_segno].maxpasses {
            l_segno += 1;
            cblk.numsegs = cblk.numsegs.wrapping_add(1)
          }
        }
        loop {
          /* Check possible overflow then size */
          if (*p_data_read).wrapping_add(cblk.segs[l_segno].newlen) < *p_data_read
            || (*p_data_read).wrapping_add(cblk.segs[l_segno].newlen) > p_max_length
          {
            if cp.strict {
              event_msg!(p_manager, EVT_ERROR,
                                            "skip: segment too long ({}) with max ({}) for codeblock {} (p={}, b={}, r={}, c={})\n",
                                            cblk.segs[l_segno].newlen, p_max_length,
                                            cblkno, pi.precno, bandno,
                                            pi.resno, pi.compno);
              return Err(());
            } else {
              event_msg!(p_manager, EVT_WARNING,
                                            "skip: segment too long ({}) with max ({}) for codeblock {} (p={}, b={}, r={}, c={})\n",
                                            cblk.segs[l_segno].newlen, p_max_length,
                                            cblkno, pi.precno, bandno,
                                            pi.resno, pi.compno);
              *p_data_read = p_max_length;
              return Ok(());
            }
          }
          log::debug!(
            "p_data_read ({}) newlen ({})",
            *p_data_read,
            cblk.segs[l_segno].newlen
          );
          *p_data_read =
            (*p_data_read as core::ffi::c_uint).wrapping_add(cblk.segs[l_segno].newlen);
          {
            let seg = &mut cblk.segs[l_segno];
            seg.numpasses = (seg.numpasses as core::ffi::c_uint).wrapping_add(seg.numnewpasses);
          }
          cblk.numnewpasses =
            (cblk.numnewpasses as core::ffi::c_uint).wrapping_sub(cblk.segs[l_segno].numnewpasses);
          if cblk.numnewpasses > 0u32 {
            l_segno += 1;
            cblk.numsegs = cblk.numsegs.wrapping_add(1)
          }
          if cblk.numnewpasses == 0u32 {
            break;
          }
        }
      }
      cblkno += 1;
    }
    bandno += 1;
  }
  Ok(())
}
fn t2_init_seg(cblk: &mut TcdCblkDec, index: u32, cblksty: u32, first: u32) -> OpResult {
  let l_nb_segs = index.wrapping_add(1u32);
  if l_nb_segs as usize > cblk.segs.len() {
    let new_len = cblk.segs.len() + 10;
    cblk.segs.resize(new_len, Default::default());
  }
  if cblksty & 0x4u32 != 0 {
    let seg = &mut cblk.segs[index as usize];
    tcd_reinit_segment(seg);
    seg.maxpasses = 1_u32
  } else if cblksty & 0x1u32 != 0 {
    if first != 0 {
      let seg = &mut cblk.segs[index as usize];
      tcd_reinit_segment(seg);
      seg.maxpasses = 10_u32
    } else {
      let prev_maxpasses = cblk.segs[index as usize - 1].maxpasses;
      let seg = &mut cblk.segs[index as usize];
      tcd_reinit_segment(seg);
      seg.maxpasses = if prev_maxpasses == 1u32 || prev_maxpasses == 10u32 {
        2i32
      } else {
        1i32
      } as u32
    }
  } else {
    /* See paragraph "B.10.6 Number of coding passes" of the standard.
     * Probably that 109 must be interpreted a (Mb-1)*3 + 1 with Mb=37,
     * Mb being the maximum number of bit-planes available for the
     * representation of coefficients in the sub-band */
    let seg = &mut cblk.segs[index as usize];
    tcd_reinit_segment(seg);
    seg.maxpasses = 109_u32
  }
  Ok(())
}
