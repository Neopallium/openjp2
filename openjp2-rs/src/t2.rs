use super::bio::*;
use super::event::*;
use super::math::*;
use super::openjpeg::*;
use crate::image::CoreImage;
use super::pi::*;
use super::tcd::*;
use super::tgt::*;


#[repr(C)]
pub(crate) struct opj_t2<'a> {
  pub image: &'a mut CoreImage,
  pub cp: &'a mut opj_cp_t,
}
pub(crate) type opj_t2_t<'a> = opj_t2<'a>;
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
 * Copyright (c) 2008, 2011-2012, Centre National d'Etudes Spatiales (CNES), FR
 * Copyright (c) 2012, CS Systemes d'Information, France
 * Copyright (c) 2017, IntoPIX SA <support@intopix.com>
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

/* * @defgroup T2 T2 - Implementation of a tier-2 coding */

/* ----------------------------------------------------------------------- */
fn opj_t2_putcommacode(bio: &mut opj_bio, mut n: OPJ_INT32) {
  for _ in 0..n {
    opj_bio_putbit(bio, 1);
  }
  opj_bio_putbit(bio, 0);
}

fn opj_t2_getcommacode(bio: &mut opj_bio) -> OPJ_UINT32 {
  let mut n = 0 as OPJ_UINT32;
  while opj_bio_read(bio, 1 as OPJ_UINT32) != 0 {
    n += 1;
  }
  n
}

/* *
Variable length code for signalling delta Zil (truncation point)
@param bio  Bit Input/Output component
@param n    delta Zil
*/
fn opj_t2_putnumpasses(bio: &mut opj_bio, mut n: OPJ_UINT32) {
  if n == 1u32 {
    opj_bio_putbit(bio, 0);
  } else if n == 2u32 {
    opj_bio_write(bio, 2 as OPJ_UINT32, 2 as OPJ_UINT32);
  } else if n <= 5u32 {
    opj_bio_write(bio, 0xcu32 | n.wrapping_sub(3u32), 4 as OPJ_UINT32);
  } else if n <= 36u32 {
    opj_bio_write(bio, 0x1e0u32 | n.wrapping_sub(6u32), 9 as OPJ_UINT32);
  } else if n <= 164u32 {
    opj_bio_write(bio, 0xff80u32 | n.wrapping_sub(37u32), 16 as OPJ_UINT32);
  };
}

fn opj_t2_getnumpasses(bio: &mut opj_bio) -> OPJ_UINT32 {
  let mut n: OPJ_UINT32 = 0;
  if opj_bio_read(bio, 1 as OPJ_UINT32) == 0 {
    return 1 as OPJ_UINT32;
  }
  if opj_bio_read(bio, 1 as OPJ_UINT32) == 0 {
    return 2 as OPJ_UINT32;
  }
  n = opj_bio_read(bio, 2 as OPJ_UINT32);
  if n != 3u32 {
    return (3u32).wrapping_add(n);
  }
  n = opj_bio_read(bio, 5 as OPJ_UINT32);
  if n != 31u32 {
    return (6u32).wrapping_add(n);
  }
  (37u32).wrapping_add(opj_bio_read(bio, 7 as OPJ_UINT32))
}

/* ----------------------------------------------------------------------- */

#[allow(clippy::too_many_arguments)]
pub(crate) fn opj_t2_encode_packets(
  t2: &mut opj_t2_t,
  mut p_tile_no: OPJ_UINT32,
  tile: &mut opj_tcd_tile_t,
  mut p_maxlayers: OPJ_UINT32,
  mut dest: &mut [u8],
  mut p_data_written: &mut OPJ_UINT32,
  mut cstr_info: *mut opj_codestream_info_t,
  mut p_marker_info: Option<&mut opj_tcd_marker_info_t>,
  mut p_tp_num: OPJ_UINT32,
  mut p_tp_pos: OPJ_INT32,
  mut p_pino: OPJ_UINT32,
  mut p_t2_mode: J2K_T2_MODE,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut dest = &mut dest[..];
  let image = &mut *t2.image;
  let cp = &mut *t2.cp;
  let mut l_nb_bytes = 0 as OPJ_UINT32;
  let mut compno: OPJ_UINT32 = 0;
  let mut poc: OPJ_UINT32 = 0;
  let pocno = if cp.rsiz as core::ffi::c_int == 0x4i32 {
    2i32
  } else {
    1i32
  } as OPJ_UINT32;
  let l_max_comp = if cp.m_specific_param.m_enc.m_max_comp_size > 0u32 {
    image.numcomps
  } else {
    1u32
  };
  let mut pi_vec = opj_pi_initialise_encode(image, cp, p_tile_no, p_t2_mode);
  if pi_vec.is_empty() {
    return 0i32;
  }
  *p_data_written = 0 as OPJ_UINT32;
  if p_t2_mode as core::ffi::c_uint == THRESH_CALC as core::ffi::c_uint {
    /* Calculating threshold */
    compno = 0 as OPJ_UINT32;
    while compno < l_max_comp {
      let mut l_comp_len = 0 as OPJ_UINT32;
      poc = 0 as OPJ_UINT32;
      while poc < pocno {
        let mut l_tp_num = compno;
        /* TODO MSD : check why this function cannot fail (cf. v1) */
        opj_pi_create_encode(
          pi_vec.as_mut_slice(),
          cp,
          p_tile_no,
          poc,
          l_tp_num,
          p_tp_pos,
          p_t2_mode,
        );
        if pi_vec[poc as usize].poc.prg as core::ffi::c_int == OPJ_PROG_UNKNOWN as core::ffi::c_int {
          /* TODO ADE : add an error */
          opj_pi_destroy(&mut pi_vec);
          return 0i32;
        }
        loop {
          let pi = &mut pi_vec[poc as usize];
          if opj_pi_next(pi, p_manager) == 0 {
            break;
          }
          if pi.layno < p_maxlayers {
            l_nb_bytes = 0 as OPJ_UINT32;
            if opj_t2_encode_packet(
              p_tile_no,
              tile,
              &mut cp.tcps[p_tile_no as usize],
              pi,
              &mut dest[..],
              &mut l_nb_bytes,
              cstr_info,
              p_t2_mode,
              p_manager,
            ) == 0
            {
              opj_pi_destroy(&mut pi_vec);
              return 0i32;
            }
            l_comp_len = (l_comp_len as core::ffi::c_uint).wrapping_add(l_nb_bytes) as OPJ_UINT32;
            let (_, rest) = dest.split_at_mut(l_nb_bytes as usize);
            dest = rest;
            *p_data_written =
              (*p_data_written as core::ffi::c_uint).wrapping_add(l_nb_bytes) as OPJ_UINT32
          }
        }
        if cp.m_specific_param.m_enc.m_max_comp_size != 0
          && l_comp_len > cp.m_specific_param.m_enc.m_max_comp_size
        {
          opj_pi_destroy(&mut pi_vec);
          return 0i32;
        }
        poc += 1;
      }
      compno += 1;
    }
  } else {
    opj_pi_create_encode(
      pi_vec.as_mut_slice(),
      cp,
      p_tile_no,
      p_pino,
      p_tp_num,
      p_tp_pos,
      p_t2_mode,
    );
    if pi_vec[p_pino as usize].poc.prg as core::ffi::c_int == OPJ_PROG_UNKNOWN as core::ffi::c_int {
      /* TODO ADE : add an error */
      opj_pi_destroy(&mut pi_vec);
      return 0i32;
    }
    if let Some(marker_info) = p_marker_info.as_deref_mut() {
      if marker_info.need_PLT != 0 {
        /* One time use intended */
        assert!(marker_info.packet_count == 0u32);
        assert!(marker_info.p_packet_size.is_empty());
        let count = opj_get_encoding_packet_count(image, cp, p_tile_no) as usize;
        marker_info.p_packet_size = vec![0; count];
      }
    }
    loop {
      let pi = &mut pi_vec[p_pino as usize];
      if opj_pi_next(pi, p_manager) == 0 {
        break;
      }
      if pi.layno < p_maxlayers {
        l_nb_bytes = 0 as OPJ_UINT32;
        if opj_t2_encode_packet(
          p_tile_no,
          tile,
          &mut cp.tcps[p_tile_no as usize],
          pi,
          &mut dest[..],
          &mut l_nb_bytes,
          cstr_info,
          p_t2_mode,
          p_manager,
        ) == 0
        {
          opj_pi_destroy(&mut pi_vec);
          return 0i32;
        }
        let (_, rest) = dest.split_at_mut(l_nb_bytes as usize);
        dest = rest;
        *p_data_written =
          (*p_data_written as core::ffi::c_uint).wrapping_add(l_nb_bytes) as OPJ_UINT32;
        if let Some(marker_info) = p_marker_info.as_deref_mut() {
          if marker_info.need_PLT != 0 {
            marker_info.p_packet_size[marker_info.packet_count as usize] = l_nb_bytes;
            marker_info.packet_count = marker_info.packet_count.wrapping_add(1)
          }
        }
        /* INDEX >> */
        #[cfg(feature = "c_api")]
        #[allow(unsafe_code)]
        if !cstr_info.is_null() {
          // SAFETY: cstr_info is the caller's codestream-info structure.
          unsafe {
            if (*cstr_info).index_write != 0 {
              let mut info_TL: *mut opj_tile_info_t =
                &mut *(*cstr_info).tile.offset(p_tile_no as isize) as *mut opj_tile_info_t;
              let mut info_PK: *mut opj_packet_info_t =
                &mut *(*info_TL).packet.offset((*cstr_info).packno as isize)
                  as *mut opj_packet_info_t;
              if (*cstr_info).packno == 0 {
                (*info_PK).start_pos = ((*info_TL).end_header + 1i32) as OPJ_OFF_T
              } else {
                (*info_PK).start_pos =
                  if (cp.m_specific_param.m_enc.m_tp_on || cp.tcps[p_tile_no as usize].POC)
                    && (*info_PK).start_pos != 0
                  {
                    (*info_PK).start_pos
                  } else {
                    ((*(*info_TL)
                      .packet
                      .offset(((*cstr_info).packno - 1i32) as isize))
                    .end_pos)
                      + 1i64
                  }
              }
              (*info_PK).end_pos = (*info_PK).start_pos + l_nb_bytes as i64 - 1;
              (*info_PK).end_ph_pos += (*info_PK).start_pos - 1i64
              /* End of packet header which now only represents the distance
              to start of packet is incremented by value of start of packet*/
            }
            (*cstr_info).packno += 1
          }
        }
        /* << INDEX */
        tile.packno = tile.packno.wrapping_add(1)
      }
    }
  }
  opj_pi_destroy(&mut pi_vec);
  1i32
}
pub(crate) fn opj_t2_decode_packets(
  mut tcd: &mut opj_tcd,
  t2: &mut opj_t2_t,
  mut p_tile_no: OPJ_UINT32,
  mut src: &mut [u8],
  mut p_data_read: &mut OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let p_data_read_initial = src.len();
  let image = &mut *t2.image;
  let cp = &mut *t2.cp;
  let mut src_base: usize = 0;
  let mut pino: OPJ_UINT32 = 0;
  let mut l_nb_bytes_read: OPJ_UINT32 = 0;
  /* create a packet iterator */
  let mut pi_vec = opj_pi_create_decode(image, cp, p_tile_no);
  if pi_vec.is_empty() {
    return 0i32;
  }
  let (num_layers_to_decode, numpocs) = {
    let tcp = &cp.tcps[p_tile_no as usize];
    (tcp.num_layers_to_decode, tcp.numpocs)
  };
  pino = 0 as OPJ_UINT32;
  while pino <= numpocs {
    /* if the resolution needed is too low, one dim of the tilec could be equal to zero
     * and no packets are used to decode this resolution and
     * l_current_pi->resno is always >= p_tile->comps[l_current_pi->compno].minimum_num_resolutions
     * and no l_img_comp->resno_decoded are computed
     */
    let mut first_pass_failed: Vec<OPJ_BOOL> = Vec::new();
    if pi_vec[pino as usize].poc.prg as core::ffi::c_int == OPJ_PROG_UNKNOWN as core::ffi::c_int {
      /* TODO ADE : add an error */
      opj_pi_destroy(&mut pi_vec);
      return 0i32;
    }
    first_pass_failed = vec![1i32; image.numcomps as usize];
    loop {
      let pi = &mut pi_vec[pino as usize];
      if opj_pi_next(pi, p_manager) == 0 {
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
        let mut bandno: OPJ_UINT32 = 0;
        let tilec = &tcd.tcd_image.tiles.comps[pi.compno as usize];
        let res = &tilec.resolutions[pi.resno as usize];
        skip_packet = 1i32;
        bandno = 0 as OPJ_UINT32;
        while bandno < res.numbands {
          let band = &res.bands[bandno as usize];
          if (pi.precno as usize) >= band.precincts.len() {
            event_msg!(p_manager, EVT_ERROR, "Invalid precinct\n",);
            opj_pi_destroy(&mut pi_vec);
            return 0i32;
          }
          let prec = &band.precincts[pi.precno as usize];
          if opj_tcd_is_subband_area_of_interest(
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
            prec.x0 as OPJ_UINT32,
            prec.y0 as OPJ_UINT32,
            prec.x1 as OPJ_UINT32,
            prec.y1 as OPJ_UINT32,
          ) != 0
          {
            skip_packet = 0i32;
            break;
          } else {
            bandno += 1;
          }
        }
      }
      if skip_packet == 0 {
        l_nb_bytes_read = 0 as OPJ_UINT32;
        first_pass_failed[pi.compno as usize] = 0i32;
        if opj_t2_decode_packet(
          cp,
          p_tile_no,
          &mut tcd.tcd_image.tiles,
          pi,
          &mut src[..],
          src_base,
          &mut l_nb_bytes_read,
          core::ptr::null_mut::<opj_packet_info_t>(),
          p_manager,
        ) == 0
        {
          opj_pi_destroy(&mut pi_vec);

          return 0i32;
        }
        image.comps[pi.compno as usize].resno_decoded =
          opj_uint_max(pi.resno, image.comps[pi.compno as usize].resno_decoded)
      } else {
        l_nb_bytes_read = 0 as OPJ_UINT32;
        if opj_t2_skip_packet(
          cp,
          p_tile_no,
          &mut tcd.tcd_image.tiles,
          pi,
          &mut src[..],
          &mut l_nb_bytes_read,
          core::ptr::null_mut::<opj_packet_info_t>(),
          p_manager,
        ) == 0
        {
          opj_pi_destroy(&mut pi_vec);

          return 0i32;
        }
      }
      if first_pass_failed[pi.compno as usize] != 0
        && image.comps[pi.compno as usize].resno_decoded == 0u32
      {
        image.comps[pi.compno as usize].resno_decoded = tcd.tcd_image.tiles.comps
          [pi.compno as usize]
          .minimum_num_resolutions
          .wrapping_sub(1u32)
      }
      let (_, rest) = src.split_at_mut(l_nb_bytes_read as usize);
      src = rest;
      src_base += l_nb_bytes_read as usize;
    }
    pino += 1;
  }
  /* INDEX >> */
  /* << INDEX */
  /* don't forget to release pi */
  opj_pi_destroy(&mut pi_vec);
  *p_data_read = (p_data_read_initial - src.len()) as OPJ_UINT32;
  1i32
}
pub(crate) fn opj_t2_create<'a>(image: &'a mut CoreImage, cp: &'a mut opj_cp_t) -> Box<opj_t2_t<'a>> {
  /* create the t2 structure */
  Box::new(opj_t2 { image, cp })
}
/* *
Decode a packet of a tile from a source buffer
@param t2 T2 handle
@param tile Tile for which to write the packets
@param tcp Tile coding parameters
@param pi Packet identity
@param src Source buffer
@param data_read   FIXME DOC
@param max_length  FIXME DOC
@param pack_info Packet information
@param p_manager the user event manager

@return  FIXME DOC
*/
fn opj_t2_decode_packet(
  cp: &mut opj_cp_t,
  p_tile_no: OPJ_UINT32,
  tile: &mut opj_tcd_tile_t,
  pi: &mut opj_pi_iterator_t,
  src: &mut [u8],
  src_base: usize,
  p_data_read: &mut OPJ_UINT32,
  pack_info: *mut opj_packet_info_t,
  p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut l_read_data: OPJ_BOOL = 0;
  let mut l_nb_bytes_read = 0 as OPJ_UINT32;
  let mut l_nb_total_bytes_read = 0 as OPJ_UINT32;
  *p_data_read = 0 as OPJ_UINT32;
  if opj_t2_read_packet_header(
    cp,
    p_tile_no,
    tile,
    pi,
    &mut l_read_data,
    src,
    &mut l_nb_bytes_read,
    pack_info,
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  let src = &mut src[l_nb_bytes_read as usize..];
  let src_base = src_base + l_nb_bytes_read as usize;
  l_nb_total_bytes_read =
    (l_nb_total_bytes_read as core::ffi::c_uint).wrapping_add(l_nb_bytes_read) as OPJ_UINT32;
  /* we should read data for the packet */
  if l_read_data != 0 {
    l_nb_bytes_read = 0 as OPJ_UINT32;
    if opj_t2_read_packet_data(
      cp,
      tile,
      pi,
      src,
      src_base,
      &mut l_nb_bytes_read,
      pack_info,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    l_nb_total_bytes_read =
      (l_nb_total_bytes_read as core::ffi::c_uint).wrapping_add(l_nb_bytes_read) as OPJ_UINT32
  }
  *p_data_read = l_nb_total_bytes_read;
  1i32
}
/* *
Encode a packet of a tile to a destination buffer
@param tileno Number of the tile encoded
@param tile Tile for which to write the packets
@param tcp Tile coding parameters
@param pi Packet identity
@param dest Destination buffer
@param p_data_written   FIXME DOC
@param len Length of the destination buffer
@param cstr_info Codestream information structure
@param p_t2_mode If == THRESH_CALC In threshold calculation ,If == FINAL_PASS Final pass
@param p_manager the user event manager
@return
*/
fn opj_t2_skip_packet(
  cp: &mut opj_cp_t,
  p_tile_no: OPJ_UINT32,
  tile: &mut opj_tcd_tile_t,
  pi: &mut opj_pi_iterator_t,
  src: &mut [u8],
  p_data_read: &mut OPJ_UINT32,
  pack_info: *mut opj_packet_info_t,
  p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut l_read_data: OPJ_BOOL = 0;
  let mut l_nb_bytes_read = 0 as OPJ_UINT32;
  let mut l_nb_total_bytes_read = 0 as OPJ_UINT32;
  *p_data_read = 0 as OPJ_UINT32;
  if opj_t2_read_packet_header(
    cp,
    p_tile_no,
    tile,
    pi,
    &mut l_read_data,
    src,
    &mut l_nb_bytes_read,
    pack_info,
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  let (_, src_rest) = src.split_at_mut(l_nb_bytes_read as usize);
  l_nb_total_bytes_read =
    (l_nb_total_bytes_read as core::ffi::c_uint).wrapping_add(l_nb_bytes_read) as OPJ_UINT32;
  let p_max_length = src_rest.len() as OPJ_UINT32;
  /* we should read data for the packet */
  if l_read_data != 0 {
    l_nb_bytes_read = 0 as OPJ_UINT32;
    if opj_t2_skip_packet_data(
      cp,
      tile,
      pi,
      &mut l_nb_bytes_read,
      p_max_length,
      pack_info,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    l_nb_total_bytes_read =
      (l_nb_total_bytes_read as core::ffi::c_uint).wrapping_add(l_nb_bytes_read) as OPJ_UINT32
  }
  *p_data_read = l_nb_total_bytes_read;
  1i32
}
fn opj_t2_encode_packet(
  #[cfg_attr(not(feature = "c_api"), allow(unused_variables))]
  tileno: OPJ_UINT32,
  tile: &mut opj_tcd_tile_t,
  tcp: &mut opj_tcp_t,
  pi: &mut opj_pi_iterator_t,
  mut dest: &mut [u8],
  p_data_written: &mut OPJ_UINT32,
  #[cfg_attr(not(feature = "c_api"), allow(unused_variables))]
  cstr_info: *mut opj_codestream_info_t,
  mut p_t2_mode: J2K_T2_MODE,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut bandno: OPJ_UINT32 = 0; /* component value */
  let mut cblkno: OPJ_UINT32 = 0; /* resolution level value */
  let mut pos = 0usize; /* precinct value */
  let mut length = dest.len() as OPJ_UINT32;
  let mut l_nb_bytes: OPJ_UINT32 = 0; /* quality layer value */
  let compno = pi.compno; /* BIO component */
  let resno = pi.resno;
  let precno = pi.precno;
  let layno = pi.layno;
  let mut l_nb_blocks: OPJ_UINT32 = 0;
  let mut packet_empty = 0i32;
  let packno = tile.packno;
  /* <SOP 0xff91> */
  if tcp.csty & 0x2u32 != 0 {
    if length < 6u32 {
      if p_t2_mode as core::ffi::c_uint == FINAL_PASS as core::ffi::c_uint {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "opj_t2_encode_packet(): only {} bytes remaining in output buffer. {} needed.\n",
          length,
          6i32,
        ); /* packno is uint32_t */
      }
      return 0i32;
    }
    dest[0] = 255 as OPJ_BYTE;
    dest[1] = 145 as OPJ_BYTE;
    dest[2] = 0 as OPJ_BYTE;
    dest[3] = 4 as OPJ_BYTE;
    dest[4] = (packno >> 8i32 & 0xffu32) as OPJ_BYTE;
    dest[5] = (packno & 0xffu32) as OPJ_BYTE;
    pos += 6;
    length = (length as core::ffi::c_uint).wrapping_sub(6u32) as OPJ_UINT32 as OPJ_UINT32
  }
  /* </SOP> */
  let tilec = &mut tile.comps[compno as usize];
  let res = &mut tilec.resolutions[resno as usize];
  if layno == 0 {
    bandno = 0 as OPJ_UINT32;
    while bandno < res.numbands {
      let band = &mut res.bands[bandno as usize];
      /* Skip empty bands */
      if opj_tcd_is_band_empty(band) == 0 {
        /* Avoid out of bounds access of https://github.com/uclouvain/openjpeg/issues/1294 */
        /* but likely not a proper fix. */
        if precno >= res.pw.wrapping_mul(res.ph) {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "opj_t2_encode_packet(): accessing precno={} >= {}\n",
            precno,
            res.pw.wrapping_mul(res.ph),
          );
          return 0i32;
        }
        let prc = &mut band.precincts[precno as usize];
        /* A degenerate precinct (cw*ch == 0) has no tag trees; the C code no-ops here. */
        if let Some(tree) = prc.incltree.as_mut() {
          opj_tgt_reset(tree);
        }
        if let Some(tree) = prc.imsbtree.as_mut() {
          opj_tgt_reset(tree);
        }
        l_nb_blocks = prc.cw.wrapping_mul(prc.ch);
        cblkno = 0 as OPJ_UINT32;
        while cblkno < l_nb_blocks {
          let cblk = prc.cblks.enc_mut(cblkno as usize);
          cblk.numpasses = 0 as OPJ_UINT32;
          // SAFETY: imsbtree is the precinct's tag tree, owned by the precinct.
          opj_tgt_setvalue(
            prc.imsbtree.as_mut().expect("imsbtree"),
            cblkno,
            band.numbps - cblk.numbps as OPJ_INT32,
          );
          cblkno += 1;
        }
      }
      bandno = bandno.wrapping_add(1);
    }
  }
  let mut bio = opj_bio::new_enc(&mut dest[pos..]);
  let bio = &mut bio;
  /* Empty header bit */
  opj_bio_putbit(bio, if packet_empty != 0 { 0 } else { 1 });

  /* Writing Packet header */
  bandno = 0 as OPJ_UINT32;
  while packet_empty == 0 && bandno < res.numbands {
    let band = &mut res.bands[bandno as usize];
    /* Skip empty bands */
    if opj_tcd_is_band_empty(band) == 0 {
      /* Avoid out of bounds access of https://github.com/uclouvain/openjpeg/issues/1297 */
      /* but likely not a proper fix. */
      if precno >= res.pw.wrapping_mul(res.ph) {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "opj_t2_encode_packet(): accessing precno={} >= {}\n",
          precno,
          res.pw.wrapping_mul(res.ph),
        );
        return 0i32;
      }
      let prc = &mut band.precincts[precno as usize];
      l_nb_blocks = prc.cw.wrapping_mul(prc.ch);
      cblkno = 0 as OPJ_UINT32;
      while cblkno < l_nb_blocks {
        let cblk = prc.cblks.enc_mut(cblkno as usize);
        let layer = &mut cblk.layers[layno as usize];
        if cblk.numpasses == 0 && layer.numpasses != 0 {
          // SAFETY: incltree is the precinct's tag tree, owned by the precinct.
          opj_tgt_setvalue(prc.incltree.as_mut().expect("incltree"), cblkno, layno as OPJ_INT32);
        }
        cblkno += 1;
      }
      cblkno = 0 as OPJ_UINT32;
      while cblkno < l_nb_blocks {
        let cblk = prc.cblks.enc_mut(cblkno as usize);
        let mut increment = 0 as OPJ_UINT32;
        let mut nump = 0 as OPJ_UINT32;
        let mut len = 0 as OPJ_UINT32;
        let mut passno: OPJ_UINT32 = 0;
        let mut l_nb_passes: OPJ_UINT32 = 0;
        /* cblk inclusion bits */
        if cblk.numpasses == 0 {
          opj_tgt_encode(
            bio,
            prc.incltree.as_mut().expect("incltree"),
            cblkno,
            layno.wrapping_add(1u32) as OPJ_INT32,
          );
        } else {
          opj_bio_putbit(bio, (cblk.layers[layno as usize].numpasses != 0u32) as u32);
        }
        /* if cblk not included, go to the next cblk  */
        if cblk.layers[layno as usize].numpasses == 0 {
        } else {
          /* if first instance of cblk --> zero bit-planes information */
          if cblk.numpasses == 0 {
            cblk.numlenbits = 3 as OPJ_UINT32;
            opj_tgt_encode(bio, prc.imsbtree.as_mut().expect("imsbtree"), cblkno, 999i32);
          }
          /* number of coding passes included */
          opj_t2_putnumpasses(bio, cblk.layers[layno as usize].numpasses);
          l_nb_passes = cblk.numpasses.wrapping_add(cblk.layers[layno as usize].numpasses);
          /* computation of the increase of the length indicator and insertion in the header     */
          passno = cblk.numpasses;
          while passno < l_nb_passes {
            nump = nump.wrapping_add(1);
            len = (len as core::ffi::c_uint).wrapping_add(cblk.passes[passno as usize].len)
              as OPJ_UINT32;
            if cblk.passes[passno as usize].term
              || passno
                == cblk
                  .numpasses
                  .wrapping_add(cblk.layers[layno as usize].numpasses)
                  .wrapping_sub(1u32)
            {
              increment = opj_int_max(
                increment as OPJ_INT32,
                opj_int_floorlog2(len as OPJ_INT32) + 1i32
                  - (cblk.numlenbits as OPJ_INT32 + opj_int_floorlog2(nump as OPJ_INT32)),
              ) as OPJ_UINT32;
              len = 0 as OPJ_UINT32;
              nump = 0 as OPJ_UINT32
            }
            passno += 1;
          }
          opj_t2_putcommacode(bio, increment as OPJ_INT32);
          /* computation of the new Length indicator */
          cblk.numlenbits =
            (cblk.numlenbits as core::ffi::c_uint).wrapping_add(increment) as OPJ_UINT32;
          /* insertion of the codeword segment length */
          passno = cblk.numpasses;
          while passno < l_nb_passes {
            nump = nump.wrapping_add(1);
            len = (len as core::ffi::c_uint).wrapping_add(cblk.passes[passno as usize].len)
              as OPJ_UINT32;
            if cblk.passes[passno as usize].term
              || passno
                == cblk
                  .numpasses
                  .wrapping_add(cblk.layers[layno as usize].numpasses)
                  .wrapping_sub(1u32)
            {
              opj_bio_write(
                bio,
                len,
                cblk
                  .numlenbits
                  .wrapping_add(opj_int_floorlog2(nump as OPJ_INT32) as OPJ_UINT32),
              );
              len = 0 as OPJ_UINT32;
              nump = 0 as OPJ_UINT32
            }
            passno += 1;
          }
        }
        cblkno += 1;
      }
    }
    bandno = bandno.wrapping_add(1);
  }
  if opj_bio_flush(bio) == 0 {
    return 0i32;
    /* modified to eliminate longjmp !! */
  }
  l_nb_bytes = opj_bio_numbytes(bio) as OPJ_UINT32;
  pos += l_nb_bytes as usize;
  length = (length as core::ffi::c_uint).wrapping_sub(l_nb_bytes) as OPJ_UINT32;
  /* <EPH 0xff92> */
  if tcp.csty & 0x4u32 != 0 {
    if length < 2u32 {
      if p_t2_mode as core::ffi::c_uint == FINAL_PASS as core::ffi::c_uint {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "opj_t2_encode_packet(): only {} bytes remaining in output buffer. {} needed.\n",
          length,
          2i32,
        );
      }
      return 0i32;
    }
    dest[pos] = 255 as OPJ_BYTE;
    dest[pos + 1] = 146 as OPJ_BYTE;
    pos += 2;
    length = (length as core::ffi::c_uint).wrapping_sub(2u32) as OPJ_UINT32 as OPJ_UINT32
  }
  /* </EPH> */
  /* << INDEX */
  /* End of packet header position. Currently only represents the distance to start of packet
  Will be updated later by incrementing with packet start value*/
  #[cfg(feature = "c_api")]
  #[allow(unsafe_code)]
  #[allow(unsafe_code)]
  if !cstr_info.is_null() && unsafe { (*cstr_info).index_write } != 0 {
    // SAFETY: cstr_info is the caller's codestream-info structure.
    unsafe {
      let mut info_PK: *mut opj_packet_info_t = &mut *(*(*cstr_info).tile.offset(tileno as isize))
        .packet
        .offset((*cstr_info).packno as isize)
        as *mut opj_packet_info_t;
      (*info_PK).end_ph_pos = pos as OPJ_OFF_T
    }
  }
  /* INDEX >> */
  /* Writing the packet body */
  bandno = 0 as OPJ_UINT32;
  while packet_empty == 0 && bandno < res.numbands {
    let band = &mut res.bands[bandno as usize];
    /* Skip empty bands */
    if opj_tcd_is_band_empty(band) == 0 {
      let prc = &mut band.precincts[precno as usize];
      l_nb_blocks = prc.cw.wrapping_mul(prc.ch);
      cblkno = 0 as OPJ_UINT32;
      while cblkno < l_nb_blocks {
        let cblk = prc.cblks.enc_mut(cblkno as usize);
        if cblk.layers[layno as usize].numpasses == 0 {
        } else {
          if cblk.layers[layno as usize].len > length {
            if p_t2_mode as core::ffi::c_uint == FINAL_PASS as core::ffi::c_uint {
              event_msg!(
                p_manager,
                EVT_ERROR,
                "opj_t2_encode_packet(): only {} bytes remaining in output buffer. {} needed.\n",
                length,
                cblk.layers[layno as usize].len
              );
            }
            return 0i32;
          }

          if p_t2_mode == FINAL_PASS {
            let len = cblk.layers[layno as usize].len as usize;
            let off = cblk.layers[layno as usize].data_offset;
            dest[pos..pos + len].copy_from_slice(&cblk.data[off..off + len]);
          }
          cblk.numpasses = (cblk.numpasses as core::ffi::c_uint)
            .wrapping_add(cblk.layers[layno as usize].numpasses) as OPJ_UINT32;
          pos += cblk.layers[layno as usize].len as usize;
          length = (length as core::ffi::c_uint)
            .wrapping_sub(cblk.layers[layno as usize].len) as OPJ_UINT32;
          /* INDEX >> */
          /* << INDEX */
          #[cfg(feature = "c_api")]
          #[allow(unsafe_code)]
          #[allow(unsafe_code)]
          if !cstr_info.is_null() && unsafe { (*cstr_info).index_write } != 0 {
            // SAFETY: cstr_info is the caller's codestream-info structure.
            unsafe {
              let mut info_PK_0: *mut opj_packet_info_t =
                &mut *(*(*cstr_info).tile.offset(tileno as isize))
                  .packet
                  .offset((*cstr_info).packno as isize) as *mut opj_packet_info_t;
              (*info_PK_0).disto += cblk.layers[layno as usize].disto;
              if (*cstr_info).D_max < (*info_PK_0).disto {
                (*cstr_info).D_max = (*info_PK_0).disto
              }
            }
          }
        }
        cblkno += 1;
      }
    }
    bandno += 1;
  }
  *p_data_written = pos as OPJ_UINT32;
  1i32
}
fn opj_t2_read_packet_header(
  cp: &mut opj_cp_t,
  p_tile_no: OPJ_UINT32,
  tile: &mut opj_tcd_tile_t,
  pi: &mut opj_pi_iterator_t,
  p_is_data_present: &mut OPJ_BOOL,
  p_src_data: &mut [u8],
  p_data_read: &mut OPJ_UINT32,
  #[cfg_attr(not(feature = "c_api"), allow(unused_variables))]
  p_pack_info: *mut opj_packet_info_t,
  p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* loop */
  let mut bandno: OPJ_UINT32 = 0; /* BIO component */
  let mut cblkno: OPJ_UINT32 = 0;
  let mut l_nb_code_blocks: OPJ_UINT32 = 0;
  let mut l_header_length: OPJ_UINT32 = 0;
  let p_max_length = p_src_data.len() as OPJ_UINT32;
  let mut l_pos = 0usize;
  let l_cp = cp;
  let csty = l_cp.tcps[p_tile_no as usize].csty;
  let tccp_cblksty = l_cp.tcps[p_tile_no as usize].tccps[pi.compno as usize].cblksty;
  let res = &mut tile.comps[pi.compno as usize].resolutions[pi.resno as usize];
  let mut l_present: OPJ_UINT32 = 0;
  if pi.layno == 0u32 {
    /* reset tagtrees */
    bandno = 0 as OPJ_UINT32;
    while bandno < res.numbands {
      let band = &mut res.bands[bandno as usize];
      if opj_tcd_is_band_empty(band) == 0 {
        if (pi.precno as usize) >= band.precincts.len() {
          event_msg!(p_manager, EVT_ERROR, "Invalid precinct\n",);
          return 0i32;
        }
        let prc = &mut band.precincts[pi.precno as usize];
        /* A degenerate precinct (cw*ch == 0) has no tag trees; the C code no-ops here. */
        if let Some(tree) = prc.incltree.as_mut() {
          opj_tgt_reset(tree);
        }
        if let Some(tree) = prc.imsbtree.as_mut() {
          opj_tgt_reset(tree);
        }
        l_nb_code_blocks = prc.cw.wrapping_mul(prc.ch);
        cblkno = 0 as OPJ_UINT32;
        while cblkno < l_nb_code_blocks {
          let cblk = prc.cblks.dec_mut(cblkno as usize);
          cblk.numsegs = 0 as OPJ_UINT32;
          cblk.real_num_segs = 0 as OPJ_UINT32;
          cblkno += 1;
        }
      }
      bandno += 1;
    }
  }

  /* SOP markers */
  if csty & 0x2u32 != 0 {
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
    /* * TODO : check the Nsop value */
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

  let mut l_bio = opj_bio::new_dec(&mut l_header[..]);
  let l_bio = &mut l_bio;

  l_present = opj_bio_read(l_bio, 1 as OPJ_UINT32);
  log::debug!("present={}", l_present);
  if l_present == 0 {
    /* TODO MSD: no test to control the output of this function*/
    opj_bio_inalign(l_bio);
    l_hdr_pos += opj_bio_numbytes(l_bio) as usize;
    /* EPH markers */
    if csty & 0x4u32 != 0 {
      if l_header.len() - l_hdr_pos < 2usize {
        event_msg!(
          p_manager,
          EVT_WARNING,
          "Not enough space for expected EPH marker\n",
        );
      } else if l_header[l_hdr_pos] as core::ffi::c_int != 0xffi32
        || l_header[l_hdr_pos + 1] as core::ffi::c_int != 0x92i32
      {
        event_msg!(p_manager, EVT_WARNING, "Expected EPH marker\n",);
      } else {
        l_hdr_pos += 2
      }
    }
    l_header_length = l_hdr_pos as OPJ_UINT32;
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
    #[cfg(feature = "c_api")]
    #[allow(unsafe_code)]
    if !p_pack_info.is_null() {
      // SAFETY: p_pack_info is the caller's packet-info slot.
      unsafe { (*p_pack_info).end_ph_pos = l_pos as OPJ_OFF_T }
    }
    /* INDEX >> */
    *p_is_data_present = 0i32;
    *p_data_read = l_pos as OPJ_UINT32;
    return 1i32;
  }

  bandno = 0 as OPJ_UINT32;
  while bandno < res.numbands {
    let band = &mut res.bands[bandno as usize];
    if opj_tcd_is_band_empty(band) == 0 {
      if (pi.precno as usize) >= band.precincts.len() {
        event_msg!(p_manager, EVT_ERROR, "Invalid precinct\n",);
        return 0i32;
      }
      let prc = &mut band.precincts[pi.precno as usize];
      l_nb_code_blocks = prc.cw.wrapping_mul(prc.ch);
      cblkno = 0 as OPJ_UINT32;
      while cblkno < l_nb_code_blocks {
        let mut l_included: OPJ_UINT32 = 0;
        let mut l_increment: OPJ_UINT32 = 0;
        let mut l_segno: OPJ_UINT32 = 0;
        let mut n: OPJ_INT32 = 0;
        let cblk = prc.cblks.dec_mut(cblkno as usize);

        /* if cblk not yet included before --> inclusion tagtree */
        if cblk.numsegs == 0 {
          l_included = opj_tgt_decode(
            l_bio,
            prc.incltree.as_mut().expect("incltree"),
            cblkno,
            pi.layno.wrapping_add(1u32) as OPJ_INT32,
          )
        /* else one bit */
        } else {
          l_included = opj_bio_read(l_bio, 1 as OPJ_UINT32)
        }

        /* if cblk not included */
        if l_included == 0 {
          cblk.numnewpasses = 0 as OPJ_UINT32;
          log::debug!("included={}", l_included);
        } else {
          /* if cblk not yet included --> zero-bitplane tagtree */
          if cblk.numsegs == 0 {
            let mut i = 0 as OPJ_UINT32;
            while opj_tgt_decode(
              l_bio,
              // SAFETY: imsbtree is the precinct's tag tree, owned by the precinct.
              prc.imsbtree.as_mut().expect("imsbtree"),
              cblkno,
              i as OPJ_INT32,
            ) == 0
            {
              i += 1;
            }
            cblk.Mb = band.numbps as OPJ_UINT32;
            if band.numbps as u32 + 1 < i {
              /* Not totally sure what we should do in that situation,
               * but that avoids the integer overflow of
               * https://github.com/uclouvain/openjpeg/pull/1488
               * while keeping the regression test suite happy.
               */
              cblk.numbps = (band.numbps + 1 - i as i32) as u32;
            } else {
              cblk.numbps = band.numbps as u32 + 1 - i;
            }
            cblk.numlenbits = 3 as OPJ_UINT32
          }
          /* number of coding passes */
          cblk.numnewpasses = opj_t2_getnumpasses(l_bio);
          l_increment = opj_t2_getcommacode(l_bio);
          /* length indicator increment */
          cblk.numlenbits =
            (cblk.numlenbits as core::ffi::c_uint).wrapping_add(l_increment) as OPJ_UINT32;
          l_segno = 0 as OPJ_UINT32;
          if cblk.numsegs == 0 {
            if opj_t2_init_seg(
              cblk,
              l_segno,
              tccp_cblksty,
              1 as OPJ_UINT32,
            ) == 0
            {
              return 0i32;
            }
          } else {
            l_segno = cblk.numsegs.wrapping_sub(1u32);
            if cblk.segs[l_segno as usize].numpasses == cblk.segs[l_segno as usize].maxpasses {
              l_segno = l_segno.wrapping_add(1);
              if opj_t2_init_seg(
                cblk,
                l_segno,
                tccp_cblksty,
                0 as OPJ_UINT32,
              ) == 0
              {
                return 0i32;
              }
            }
          }
          n = cblk.numnewpasses as OPJ_INT32;
          if tccp_cblksty & 0x40u32 != 0u32 {
            loop {
              let mut bit_number: OPJ_UINT32 = 0;
              cblk.segs[l_segno as usize].numnewpasses = if l_segno == 0u32 {
                1u32
              } else {
                n as OPJ_UINT32
              };
              bit_number = cblk.numlenbits.wrapping_add(opj_uint_floorlog2(
                cblk.segs[l_segno as usize].numnewpasses,
              ));
              if bit_number > 32u32 {
                event_msg!(
                  p_manager,
                  EVT_ERROR,
                  "Invalid bit number {} in opj_t2_read_packet_header()\n",
                  bit_number,
                );
                return 0i32;
              }
              cblk.segs[l_segno as usize].newlen = opj_bio_read(l_bio, bit_number);
              log::debug!(
                "included={} numnewpasses={} increment={} len={}",
                l_included,
                cblk.segs[l_segno as usize].numnewpasses,
                l_increment,
                cblk.segs[l_segno as usize].newlen,
              );
              n -= cblk.segs[l_segno as usize].numnewpasses as OPJ_INT32;
              if n > 0i32 {
                l_segno = l_segno.wrapping_add(1);
                if opj_t2_init_seg(
                  cblk,
                  l_segno,
                  tccp_cblksty,
                  0 as OPJ_UINT32,
                ) == 0
                {
                  return 0i32;
                }
              }
              if n <= 0i32 {
                break;
              }
            }
          } else {
            loop {
              let mut bit_number_0: OPJ_UINT32 = 0;
              cblk.segs[l_segno as usize].numnewpasses = opj_int_min(
                cblk.segs[l_segno as usize]
                  .maxpasses
                  .wrapping_sub(cblk.segs[l_segno as usize].numpasses) as OPJ_INT32,
                n,
              )
                as OPJ_UINT32;
              bit_number_0 = cblk.numlenbits.wrapping_add(opj_uint_floorlog2(
                cblk.segs[l_segno as usize].numnewpasses,
              ));
              if bit_number_0 > 32u32 {
                event_msg!(
                  p_manager,
                  EVT_ERROR,
                  "Invalid bit number {} in opj_t2_read_packet_header()\n",
                  bit_number_0,
                );
                return 0i32;
              }
              cblk.segs[l_segno as usize].newlen = opj_bio_read(l_bio, bit_number_0);
              log::debug!(
                "included={} numnewpasses={} increment={} len={}",
                l_included,
                cblk.segs[l_segno as usize].numnewpasses,
                l_increment,
                cblk.segs[l_segno as usize].newlen,
              );
              n -= cblk.segs[l_segno as usize].numnewpasses as OPJ_INT32;
              if n > 0i32 {
                l_segno = l_segno.wrapping_add(1);
                if opj_t2_init_seg(
                  cblk,
                  l_segno,
                  tccp_cblksty,
                  0 as OPJ_UINT32,
                ) == 0
                {
                  return 0i32;
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
  if opj_bio_inalign(l_bio) == 0 {
    return 0i32;
  }
  l_hdr_pos += opj_bio_numbytes(l_bio) as usize;
  /* EPH markers */
  if csty & 0x4u32 != 0 {
    if l_header.len() - l_hdr_pos < 2usize {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "Not enough space for expected EPH marker\n",
      );
    } else if l_header[l_hdr_pos] as core::ffi::c_int != 0xffi32
      || l_header[l_hdr_pos + 1] as core::ffi::c_int != 0x92i32
    {
      event_msg!(p_manager, EVT_WARNING, "Expected EPH marker\n",);
    } else {
      l_hdr_pos += 2
    }
  }
  l_header_length = l_hdr_pos as OPJ_UINT32;
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
  #[cfg(feature = "c_api")]
  #[allow(unsafe_code)]
  if !p_pack_info.is_null() {
    // SAFETY: p_pack_info is the caller's packet-info slot.
    unsafe { (*p_pack_info).end_ph_pos = l_pos as OPJ_OFF_T }
  }
  /* INDEX >> */
  *p_is_data_present = 1i32; /* next code_block */
  *p_data_read = l_pos as OPJ_UINT32;
  1i32
}
fn opj_t2_read_packet_data(
  cp: &mut opj_cp_t,
  tile: &mut opj_tcd_tile_t,
  pi: &mut opj_pi_iterator_t,
  p_src_data: &[u8],
  src_base: usize,
  p_data_read: &mut OPJ_UINT32,
  _pack_info: *mut opj_packet_info_t,
  p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut bandno: OPJ_UINT32 = 0;
  let mut cblkno: OPJ_UINT32 = 0;
  let mut l_nb_code_blocks: OPJ_UINT32 = 0;
  let p_max_length = p_src_data.len() as OPJ_UINT32;
  let mut l_pos = 0usize;
  let res = &mut tile.comps[pi.compno as usize].resolutions[pi.resno as usize];
  let mut partial_buffer = 0i32;
  bandno = 0 as OPJ_UINT32;
  while bandno < res.numbands {
    let band = &mut res.bands[bandno as usize];
    if band.x1 - band.x0 == 0i32 || band.y1 - band.y0 == 0i32 {
      bandno += 1;
      continue;
    }
    if (pi.precno as usize) >= band.precincts.len() {
      event_msg!(p_manager, EVT_ERROR, "Invalid precinct\n",);
      return 0i32;
    }
    let prc = &mut band.precincts[pi.precno as usize];
    l_nb_code_blocks = prc.cw.wrapping_mul(prc.ch);
    cblkno = 0 as OPJ_UINT32;
    while cblkno < l_nb_code_blocks {
      let cblk = prc.cblks.dec_mut(cblkno as usize);
      // if we have a partial data stream, set numchunks to zero
      // since we have no data to actually decode.
      if partial_buffer != 0 {
        cblk.numchunks = 0 as OPJ_UINT32
      }
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
        loop
        /* Check the remaining size of the source buffer */
        {
          let seg = &mut cblk.segs[l_segno];
          if seg.newlen as usize > p_src_data.len() - l_pos || partial_buffer != 0 {
            if cp.strict != 0 {
              event_msg!(p_manager, EVT_ERROR,
                                            "read: segment too long ({}) with max ({}) for codeblock {} (p={}, b={}, r={}, c={})\n",
                                            seg.newlen, p_max_length,
                                            cblkno, pi.precno, bandno,
                                            pi.resno, pi.compno);
              return 0i32;
            } else {
              event_msg!(p_manager, EVT_WARNING,
                                            "read: segment too long ({}) with max ({}) for codeblock {} (p={}, b={}, r={}, c={})\n",
                                            seg.newlen, p_max_length,
                                            cblkno, pi.precno, bandno,
                                            pi.resno, pi.compno);
              // skip this codeblock since it is a partial read
              partial_buffer = 1i32;
              cblk.numchunks = 0 as OPJ_UINT32;
              seg.numpasses = (seg.numpasses as core::ffi::c_uint)
                .wrapping_add(seg.numnewpasses)
                as OPJ_UINT32;
              cblk.numnewpasses = (cblk.numnewpasses as core::ffi::c_uint)
                .wrapping_sub(seg.numnewpasses)
                as OPJ_UINT32;
              if cblk.numnewpasses > 0u32 {
                l_segno += 1;
                cblk.numsegs = cblk.numsegs.wrapping_add(1);
                break;
              }
            }
          } else {
            /* USE_JPWL */
            if cblk.numchunks as usize == cblk.chunks.len() {
              let new_len = cblk.chunks.len() * 2 + 1;
              cblk.chunks.resize(
                new_len,
                opj_tcd_seg_data_chunk {
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
            seg.len =
              (seg.len as core::ffi::c_uint).wrapping_add(seg.newlen) as OPJ_UINT32;
            seg.numpasses = (seg.numpasses as core::ffi::c_uint)
              .wrapping_add(seg.numnewpasses)
              as OPJ_UINT32;
            cblk.numnewpasses = (cblk.numnewpasses as core::ffi::c_uint)
              .wrapping_sub(seg.numnewpasses)
              as OPJ_UINT32;
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
    *p_data_read = l_pos as OPJ_UINT32
  }
  1i32
}

fn opj_t2_skip_packet_data(
  cp: &mut opj_cp_t,
  tile: &mut opj_tcd_tile_t,
  pi: &mut opj_pi_iterator_t,
  p_data_read: &mut OPJ_UINT32,
  p_max_length: OPJ_UINT32,
  _pack_info: *mut opj_packet_info_t,
  p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut bandno: OPJ_UINT32 = 0;
  let mut cblkno: OPJ_UINT32 = 0;
  let mut l_nb_code_blocks: OPJ_UINT32 = 0;
  let res = &mut tile.comps[pi.compno as usize].resolutions[pi.resno as usize];
  *p_data_read = 0 as OPJ_UINT32;
  bandno = 0 as OPJ_UINT32;
  while bandno < res.numbands {
    let band = &mut res.bands[bandno as usize];
    if band.x1 - band.x0 == 0i32 || band.y1 - band.y0 == 0i32 {
      bandno += 1;
      continue;
    }
    if (pi.precno as usize) >= band.precincts.len() {
      event_msg!(p_manager, EVT_ERROR, "Invalid precinct\n",);
      return 0i32;
    }
    let prc = &mut band.precincts[pi.precno as usize];
    l_nb_code_blocks = prc.cw.wrapping_mul(prc.ch);
    cblkno = 0 as OPJ_UINT32;
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
            if cp.strict != 0 {
              event_msg!(p_manager, EVT_ERROR,
                                            "skip: segment too long ({}) with max ({}) for codeblock {} (p={}, b={}, r={}, c={})\n",
                                            cblk.segs[l_segno].newlen, p_max_length,
                                            cblkno, pi.precno, bandno,
                                            pi.resno, pi.compno);
              return 0i32;
            } else {
              event_msg!(p_manager, EVT_WARNING,
                                            "skip: segment too long ({}) with max ({}) for codeblock {} (p={}, b={}, r={}, c={})\n",
                                            cblk.segs[l_segno].newlen, p_max_length,
                                            cblkno, pi.precno, bandno,
                                            pi.resno, pi.compno);
              return 1i32;
            }
          }
          /* USE_JPWL */
          log::debug!(
            "p_data_read ({}) newlen ({})",
            *p_data_read,
            cblk.segs[l_segno].newlen
          );
          *p_data_read =
            (*p_data_read as core::ffi::c_uint).wrapping_add(cblk.segs[l_segno].newlen)
              as OPJ_UINT32;
          {
            let seg = &mut cblk.segs[l_segno];
            seg.numpasses = (seg.numpasses as core::ffi::c_uint)
              .wrapping_add(seg.numnewpasses)
              as OPJ_UINT32;
          }
          cblk.numnewpasses = (cblk.numnewpasses as core::ffi::c_uint)
            .wrapping_sub(cblk.segs[l_segno].numnewpasses)
            as OPJ_UINT32;
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
  1i32
}
fn opj_t2_init_seg(
  cblk: &mut opj_tcd_cblk_dec_t,
  index: OPJ_UINT32,
  cblksty: OPJ_UINT32,
  first: OPJ_UINT32,
) -> OPJ_BOOL {
  let l_nb_segs = index.wrapping_add(1u32);
  if l_nb_segs as usize > cblk.segs.len() {
    let new_len = cblk.segs.len() + 10;
    cblk.segs.resize(new_len, Default::default());
  }
  if cblksty & 0x4u32 != 0 {
    let seg = &mut cblk.segs[index as usize];
    opj_tcd_reinit_segment(seg);
    seg.maxpasses = 1 as OPJ_UINT32
  } else if cblksty & 0x1u32 != 0 {
    if first != 0 {
      let seg = &mut cblk.segs[index as usize];
      opj_tcd_reinit_segment(seg);
      seg.maxpasses = 10 as OPJ_UINT32
    } else {
      let prev_maxpasses = cblk.segs[index as usize - 1].maxpasses;
      let seg = &mut cblk.segs[index as usize];
      opj_tcd_reinit_segment(seg);
      seg.maxpasses =
        if prev_maxpasses == 1u32 || prev_maxpasses == 10u32 {
          2i32
        } else {
          1i32
        } as OPJ_UINT32
    }
  } else {
    /* See paragraph "B.10.6 Number of coding passes" of the standard.
     * Probably that 109 must be interpreted a (Mb-1)*3 + 1 with Mb=37,
     * Mb being the maximum number of bit-planes available for the
     * representation of coefficients in the sub-band */
    let seg = &mut cblk.segs[index as usize];
    opj_tcd_reinit_segment(seg);
    seg.maxpasses = 109 as OPJ_UINT32
  }
  1i32
}
