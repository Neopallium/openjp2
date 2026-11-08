use super::consts::*;
use super::dwt::*;
use super::event::*;
use super::math::*;
use super::mct::*;
use super::openjpeg::*;
use super::par;

// TEMPORARY profiling (OPJ_PROFILE=1): stage timings for the rayon plan.
macro_rules! tprof {
  ($name:expr, $body:expr) => {{
    let _t = std::time::Instant::now();
    let r = $body;
    if std::env::var_os("OPJ_PROFILE").is_some() {
      eprintln!("PROF {} {:.2}ms", $name, _t.elapsed().as_secs_f64() * 1e3);
    }
    r
  }};
}
use super::pi::*;
use super::t1::*;
use super::t2::*;
use super::tgt::*;
use crate::image::{CoreImage, CoreImageComp};

/* ----------------------------------------------------------------------- */
impl Tcd {
  pub fn new(m_is_decoder: bool) -> Self {
    let mut tcd = Self::default();
    tcd.m_is_decoder = m_is_decoder;
    tcd
  }

  pub fn used_component(&self, compno: u32) -> bool {
    if self.used_component.is_empty() {
      return true;
    }
    self.used_component[compno as usize]
  }
}

impl Drop for Tcd {
  fn drop(&mut self) {
    opj_tcd_free_tile(self);
  }
}

/* ----------------------------------------------------------------------- */
fn opj_tcd_rateallocate_fixed(tcd: &mut Tcd, image: &mut CoreImage, cp: &mut Cp) {
  let mut layno: u32 = 0; /* fixed_quality */
  // SAFETY: `tcp` is set by opj_tcd_init_tile and valid while the tcd is in use.
  let numlayers = cp.tcps[tcd.tcd_tileno as usize].numlayers;
  layno = 0_u32;
  while layno < numlayers {
    opj_tcd_makelayer_fixed(tcd, layno, 1_u32, image, cp);
    layno += 1;
  }
}

/** Returns OPJ_TRUE if the layer allocation is unchanged w.r.t to the previous
 * invokation with a different threshold */
fn opj_tcd_makelayer(tcd: &mut Tcd, layno: u32, thresh: f64, final_0: u32) -> bool {
  let mut compno: u32 = 0;
  let mut resno: u32 = 0;
  let mut bandno: u32 = 0;
  let mut precno: u32 = 0;
  let mut cblkno: u32 = 0;
  let mut passno: u32 = 0;
  let mut layer_allocation_is_same = true;
  tcd.tcd_image.tiles.distolayer[layno as usize] = 0 as f64;
  compno = 0_u32;
  while compno < tcd.tcd_image.tiles.numcomps {
    let tilec = &mut tcd.tcd_image.tiles.comps[compno as usize];
    resno = 0_u32;
    while resno < tilec.numresolutions {
      let res = &mut tilec.resolutions[resno as usize];
      bandno = 0_u32;
      while bandno < res.numbands {
        let band = &mut res.bands[bandno as usize];
        /* Skip empty bands */
        if opj_tcd_is_band_empty(band) == 0 {
          precno = 0_u32;
          while precno < res.pw.wrapping_mul(res.ph) {
            let prc = &mut band.precincts[precno as usize];
            cblkno = 0_u32;
            while cblkno < prc.cw.wrapping_mul(prc.ch) {
              let cblk = prc.cblks.enc_mut(cblkno as usize);
              let mut n: u32 = 0;
              if layno == 0u32 {
                cblk.numpassesinlayers = 0_u32
              }
              n = cblk.numpassesinlayers;
              if thresh < 0 as core::ffi::c_double {
                /* Special value to indicate to use all passes */
                n = cblk.totalpasses
              } else {
                passno = cblk.numpassesinlayers;
                while passno < cblk.totalpasses {
                  let mut dr: u32 = 0;
                  let mut dd: f64 = 0.;
                  if n == 0u32 {
                    dr = cblk.passes[passno as usize].rate;
                    dd = cblk.passes[passno as usize].distortiondec
                  } else {
                    dr = cblk.passes[passno as usize]
                      .rate
                      .wrapping_sub(cblk.passes[n.wrapping_sub(1u32) as usize].rate);
                    dd = cblk.passes[passno as usize].distortiondec
                      - cblk.passes[n.wrapping_sub(1u32) as usize].distortiondec
                  }
                  if dr == 0 {
                    if dd != 0 as core::ffi::c_double {
                      n = passno.wrapping_add(1u32)
                    }
                  } else if (thresh - dd / dr as core::ffi::c_double)
                    < 2.220_446_049_250_313e-16_f64
                  {
                    /* do not rely on float equality, check with DBL_EPSILON margin */
                    n = passno.wrapping_add(1u32)
                  }
                  passno += 1;
                }
              } /*, matrice[tcd_tcp->numlayers][tcd_tile->comps[0].numresolutions][3]; */

              if cblk.layers[layno as usize].numpasses != n - cblk.numpassesinlayers {
                layer_allocation_is_same = false;
                cblk.layers[layno as usize].numpasses = n - cblk.numpassesinlayers;
              }
              if cblk.layers[layno as usize].numpasses == 0 {
                cblk.layers[layno as usize].disto = 0 as f64
              } else {
                if cblk.numpassesinlayers == 0u32 {
                  cblk.layers[layno as usize].len = cblk.passes[n.wrapping_sub(1u32) as usize].rate;
                  cblk.layers[layno as usize].data_offset = 1;
                  cblk.layers[layno as usize].disto =
                    cblk.passes[n.wrapping_sub(1u32) as usize].distortiondec
                } else {
                  cblk.layers[layno as usize].len = cblk.passes[n.wrapping_sub(1u32) as usize]
                    .rate
                    .wrapping_sub(
                      cblk.passes[cblk.numpassesinlayers.wrapping_sub(1u32) as usize].rate,
                    );
                  // The layer payload aliases the codeblock buffer at the
                  // pass offset, exactly like the C pointer arithmetic.
                  cblk.layers[layno as usize].data_offset = 1usize.wrapping_add(
                    cblk.passes[cblk.numpassesinlayers.wrapping_sub(1u32) as usize].rate as usize,
                  );
                  cblk.layers[layno as usize].disto = cblk.passes[n.wrapping_sub(1u32) as usize]
                    .distortiondec
                    - cblk.passes[cblk.numpassesinlayers.wrapping_sub(1u32) as usize].distortiondec
                }
                tcd.tcd_image.tiles.distolayer[layno as usize] += cblk.layers[layno as usize].disto;
                if final_0 != 0 {
                  cblk.numpassesinlayers = n
                }
              }
              cblkno += 1;
            }
            precno += 1;
          }
        }
        bandno += 1;
      }
      resno += 1;
    }
    compno += 1;
  }
  layer_allocation_is_same
}
fn opj_tcd_makelayer_fixed(
  tcd: &mut Tcd,
  layno: u32,
  final_0: u32,
  image: &mut CoreImage,
  cp: &mut Cp,
) {
  let mut compno: u32 = 0;
  let mut resno: u32 = 0;
  let mut bandno: u32 = 0;
  let mut precno: u32 = 0;
  let mut cblkno: u32 = 0;
  let mut value: i32 = 0;
  let mut matrice: [[[i32; 3]; j2k::J2K_TCD_MATRIX_MAX_RESOLUTION_COUNT as usize];
    j2k::J2K_TCD_MATRIX_MAX_LAYER_COUNT as usize] = [[[0; 3];
    j2k::J2K_TCD_MATRIX_MAX_RESOLUTION_COUNT as usize];
    j2k::J2K_TCD_MATRIX_MAX_LAYER_COUNT as usize];
  let mut i: u32 = 0;
  let mut j: u32 = 0;
  let mut k: u32 = 0;
  // SAFETY: `cp`/`tcp`/`image` are set by opj_tcd_init(_tile) and valid here.
  let tcd_tcp = &cp.tcps[tcd.tcd_tileno as usize];
  compno = 0_u32;
  while compno < tcd.tcd_image.tiles.numcomps {
    let tilec = &mut tcd.tcd_image.tiles.comps[compno as usize];
    i = 0_u32;
    while i < tcd_tcp.numlayers {
      j = 0_u32;
      while j < tilec.numresolutions {
        k = 0_u32;
        while k < 3u32 {
          matrice[i as usize][j as usize][k as usize] = (cp.m_specific_param.m_enc.m_matrice[i
            .wrapping_mul(tilec.numresolutions)
            .wrapping_mul(3u32)
            .wrapping_add(j.wrapping_mul(3u32))
            .wrapping_add(k)
            as usize] as f32
            * (image.comps[compno as usize].prec as core::ffi::c_double / 16.0f64) as f32)
            as i32;
          k += 1;
        }
        j += 1;
      }
      i += 1;
    }
    resno = 0_u32;
    while resno < tilec.numresolutions {
      let res = &mut tilec.resolutions[resno as usize];
      bandno = 0_u32;
      while bandno < res.numbands {
        let band = &mut res.bands[bandno as usize];
        /* Skip empty bands */
        if opj_tcd_is_band_empty(band) == 0 {
          precno = 0_u32; /* number of bit-plan equal to zero */
          while precno < res.pw.wrapping_mul(res.ph) {
            let prc = &mut band.precincts[precno as usize];
            cblkno = 0_u32;
            while cblkno < prc.cw.wrapping_mul(prc.ch) {
              let cblk = prc.cblks.enc_mut(cblkno as usize);
              let mut n: u32 = 0;
              let mut imsb = image.comps[compno as usize].prec.wrapping_sub(cblk.numbps) as i32;
              /* Correction of the matrix of coefficient to include the IMSB information */
              if layno == 0u32 {
                value = matrice[layno as usize][resno as usize][bandno as usize]; /* fixed_quality */
                if imsb >= value {
                  value = 0i32
                } else {
                  value -= imsb
                }
              } else {
                value = matrice[layno as usize][resno as usize][bandno as usize]
                  - matrice[layno.wrapping_sub(1u32) as usize][resno as usize][bandno as usize]; /* 1.1; fixed_quality */
                if imsb
                  >= matrice[layno.wrapping_sub(1u32) as usize][resno as usize][bandno as usize]
                {
                  value -= imsb
                    - matrice[layno.wrapping_sub(1u32) as usize][resno as usize][bandno as usize]; /* fixed_quality */
                  if value < 0i32 {
                    value = 0i32
                  }
                }
              } /* compno */
              if layno == 0u32 {
                cblk.numpassesinlayers = 0_u32
              } /* resno */
              n = cblk.numpassesinlayers;
              if cblk.numpassesinlayers == 0u32 {
                if value != 0i32 {
                  n = (3u32)
                    .wrapping_mul(value as u32)
                    .wrapping_sub(2u32)
                    .wrapping_add(cblk.numpassesinlayers)
                } else {
                  n = cblk.numpassesinlayers
                }
              } else {
                n = (3u32)
                  .wrapping_mul(value as u32)
                  .wrapping_add(cblk.numpassesinlayers)
              }
              cblk.layers[layno as usize].numpasses = n.wrapping_sub(cblk.numpassesinlayers);
              if cblk.layers[layno as usize].numpasses != 0 {
                if cblk.numpassesinlayers == 0u32 {
                  cblk.layers[layno as usize].len = cblk.passes[n.wrapping_sub(1u32) as usize].rate;
                  cblk.layers[layno as usize].data_offset = 1;
                } else {
                  cblk.layers[layno as usize].len = cblk.passes[n.wrapping_sub(1u32) as usize]
                    .rate
                    .wrapping_sub(
                      cblk.passes[cblk.numpassesinlayers.wrapping_sub(1u32) as usize].rate,
                    );
                  // The layer payload aliases the codeblock buffer at the
                  // pass offset, exactly like the C pointer arithmetic.
                  cblk.layers[layno as usize].data_offset = 1usize.wrapping_add(
                    cblk.passes[cblk.numpassesinlayers.wrapping_sub(1u32) as usize].rate as usize,
                  );
                }
                if final_0 != 0 {
                  cblk.numpassesinlayers = n
                }
              }
              cblkno += 1;
            }
            precno += 1;
          }
        }
        bandno += 1;
      }
      resno += 1;
    }
    compno += 1;
  }
}
fn opj_tcd_rateallocate(
  tcd: &mut Tcd,
  dest: &mut [u8],
  p_data_written: &mut u32,
  len: u32,
  p_manager: &mut EventMgr,
  image: &mut CoreImage,
  cp: &mut Cp,
) -> OPJ_BOOL {
  {
    let mut compno: u32 = 0;
    let mut resno: u32 = 0;
    let mut bandno: u32 = 0;
    let mut precno: u32 = 0;
    let mut cblkno: u32 = 0;
    let mut layno: u32 = 0;
    let mut passno: u32 = 0;
    let mut min: f64 = 0.;
    let mut max: f64 = 0.;
    let mut cumdisto: [f64; 100] = [0.; 100];
    let k = 1_f64;
    let mut max_se = 0 as f64;
    let tcd_tcp_numlayers = cp.tcps[tcd.tcd_tileno as usize].numlayers;
    let tcd_tcp_rates = cp.tcps[tcd.tcd_tileno as usize].rates;
    let tcd_tcp_distoratio = cp.tcps[tcd.tcd_tileno as usize].distoratio;
    min = 1.797_693_134_862_315_7e308_f64;
    max = 0 as f64;
    tcd.tcd_image.tiles.numpix = 0i32;
    compno = 0_u32;
    while compno < tcd.tcd_image.tiles.numcomps {
      let tilec = &mut tcd.tcd_image.tiles.comps[compno as usize];
      tilec.numpix = 0i32;
      resno = 0_u32;
      while resno < tilec.numresolutions {
        let res = &mut tilec.resolutions[resno as usize];
        bandno = 0_u32;
        while bandno < res.numbands {
          let band = &mut res.bands[bandno as usize];
          /* bandno */
          /* precno */
          /* Skip empty bands */
          if opj_tcd_is_band_empty(band) == 0 {
            precno = 0_u32;
            while precno < res.pw.wrapping_mul(res.ph) {
              let prc = &mut band.precincts[precno as usize];
              cblkno = 0_u32;
              while cblkno < prc.cw.wrapping_mul(prc.ch) {
                let cblk = prc.cblks.enc_mut(cblkno as usize);
                /* cbklno */
                passno = 0_u32; /* passno */
                while passno < cblk.totalpasses {
                  let mut dr: i32 = 0;
                  let mut dd: f64 = 0.;
                  let mut rdslope: f64 = 0.;
                  if passno == 0u32 {
                    dr = cblk.passes[passno as usize].rate as i32;
                    dd = cblk.passes[passno as usize].distortiondec
                  } else {
                    dr = cblk.passes[passno as usize]
                      .rate
                      .wrapping_sub(cblk.passes[passno.wrapping_sub(1u32) as usize].rate)
                      as i32;
                    dd = cblk.passes[passno as usize].distortiondec
                      - cblk.passes[passno.wrapping_sub(1u32) as usize].distortiondec
                  }
                  if dr != 0i32 {
                    rdslope = dd / dr as core::ffi::c_double;
                    if rdslope < min {
                      min = rdslope
                    }
                    if rdslope > max {
                      max = rdslope
                    }
                  }
                  passno += 1;
                }

                {
                  let cblk_pix_count = (cblk.x1 - cblk.x0) * (cblk.y1 - cblk.y0);
                  tcd.tcd_image.tiles.numpix += cblk_pix_count;
                  tilec.numpix += cblk_pix_count;
                }
                cblkno += 1;
              }
              precno += 1;
            }
          }
          bandno += 1;
        }
        resno += 1;
      }
      let prec = image.comps[compno as usize].prec;
      max_se += (((1i32) << prec) as f64 - 1.0f64)
        * (((1i32) << prec) as f64 - 1.0f64)
        * tilec.numpix as f64;
      compno += 1;
    }

    /* index file */

    layno = 0_u32;
    while layno < tcd_tcp_numlayers {
      let mut lo = min;
      let mut hi = max;
      let mut maxlen = if tcd_tcp_rates[layno as usize] > 0.0f32 {
        opj_uint_min(
          (tcd_tcp_rates[layno as usize] as core::ffi::c_double).ceil() as u32,
          len,
        )
      } else {
        len
      };
      let mut goodthresh = 0 as f64;
      let mut stable_thresh = 0 as f64;
      let mut i: u32 = 0;
      let mut distotarget: f64 = 0.;

      distotarget = tcd.tcd_image.tiles.distotile
        - k * max_se
          / (10_f32 as core::ffi::c_double).powf(
            (tcd_tcp_distoratio[layno as usize] / 10 as core::ffi::c_float) as core::ffi::c_double,
          );

      /* Don't try to find an optimal threshold but rather take everything not included yet, if
      -r xx,yy,zz,0   (m_quality_layer_alloc_strategy == RATE_DISTORTION_RATIO and rates == NULL)
      -q xx,yy,zz,0   (m_quality_layer_alloc_strategy == FIXED_DISTORTION_RATIO and distoratio == NULL)
      ==> possible to have some lossy layers and the last layer for sure lossless */
      if cp.m_specific_param.m_enc.m_quality_layer_alloc_strategy
        == J2K_QUALITY_LAYER_ALLOCATION_STRATEGY::RATE_DISTORTION_RATIO
        && tcd_tcp_rates[layno as usize] > 0.0f32
        || cp.m_specific_param.m_enc.m_quality_layer_alloc_strategy
          == J2K_QUALITY_LAYER_ALLOCATION_STRATEGY::FIXED_DISTORTION_RATIO
          && tcd_tcp_distoratio[layno as usize] as core::ffi::c_double > 0.0f64
      {
        /* fixed_quality */
        let mut thresh = 0 as f64;
        let mut last_layer_allocation_ok = false;

        i = 0_u32;
        while i < 128u32 {
          let mut distoachieved = 0 as f64;
          let new_thresh = (lo + hi) / 2.0;
          /* Stop iterating when the threshold has stabilized enough */
          /* 0.5 * 1e-5 is somewhat arbitrary, but has been selected */
          /* so that this doesn't change the results of the regression */
          /* test suite. */
          if (new_thresh - thresh).abs() <= 0.5 * 1e-5 * thresh {
            break;
          }
          thresh = new_thresh;

          let layer_allocation_is_same = opj_tcd_makelayer(tcd, layno, thresh, 0_u32) && i != 0;
          if cp.m_specific_param.m_enc.m_quality_layer_alloc_strategy
            == J2K_QUALITY_LAYER_ALLOCATION_STRATEGY::FIXED_DISTORTION_RATIO
          {
            if cp.rsiz as core::ffi::c_int >= 0x3i32 && cp.rsiz as core::ffi::c_int <= 0x6i32
              || cp.rsiz as core::ffi::c_int >= 0x400i32
                && cp.rsiz as core::ffi::c_int <= 0x900i32 | 0x9bi32
            {
              if opj_t2_encode_packets(
                &mut opj_t2_create(image, cp),
                tcd.tcd_tileno,
                &mut tcd.tcd_image.tiles,
                layno.wrapping_add(1u32),
                &mut dest[..maxlen as usize],
                p_data_written,
                None,
                tcd.cur_tp_num,
                tcd.tp_pos,
                tcd.cur_pino,
                THRESH_CALC,
                p_manager,
              ) == 0
              {
                lo = thresh
              } else {
                distoachieved = if layno == 0u32 {
                  tcd.tcd_image.tiles.distolayer[0_usize]
                } else {
                  (cumdisto[layno.wrapping_sub(1u32) as usize])
                    + tcd.tcd_image.tiles.distolayer[layno as usize]
                };
                if distoachieved < distotarget {
                  hi = thresh;
                  stable_thresh = thresh
                } else {
                  lo = thresh
                }
              }
            } else {
              distoachieved = if layno == 0u32 {
                tcd.tcd_image.tiles.distolayer[0_usize]
              } else {
                (cumdisto[layno.wrapping_sub(1u32) as usize])
                  + tcd.tcd_image.tiles.distolayer[layno as usize]
              };
              if distoachieved < distotarget {
                hi = thresh;
                stable_thresh = thresh
              } else {
                lo = thresh
              }
            }
          } else {
            /* Disto/rate based optimization */
            /* Check if the layer allocation done by opj_tcd_makelayer()
             * is compatible of the maximum rate allocation. If not,
             * retry with a higher threshold.
             * If OK, try with a lower threshold.
             * Call opj_t2_encode_packets() only if opj_tcd_makelayer()
             * has resulted in different truncation points since its last
             * call. */
            if (layer_allocation_is_same && !last_layer_allocation_ok)
              || (!layer_allocation_is_same
                && opj_t2_encode_packets(
                  &mut opj_t2_create(image, cp),
                  tcd.tcd_tileno,
                  &mut tcd.tcd_image.tiles,
                  layno.wrapping_add(1u32),
                  &mut dest[..maxlen as usize],
                  p_data_written,
                  None,
                  tcd.cur_tp_num,
                  tcd.tp_pos,
                  tcd.cur_pino,
                  THRESH_CALC,
                  p_manager,
                ) == 0)
            {
              last_layer_allocation_ok = false;
              lo = thresh;
            } else {
              last_layer_allocation_ok = true;
              hi = thresh;
              stable_thresh = thresh
            }
          }
          i += 1;
        }
        goodthresh = if stable_thresh == 0 as core::ffi::c_double {
          thresh
        } else {
          stable_thresh
        };
      } else {
        /* Special value to indicate to use all passes */
        goodthresh = -(1i32) as f64
      }
      /* Threshold for Marcela Index */

      opj_tcd_makelayer(tcd, layno, goodthresh, 1_u32);

      cumdisto[layno as usize] = if layno == 0u32 {
        tcd.tcd_image.tiles.distolayer[0_usize]
      } else {
        (cumdisto[layno.wrapping_sub(1u32) as usize])
          + tcd.tcd_image.tiles.distolayer[layno as usize]
      };
      layno += 1;
    }
    1i32
  }
}

pub(crate) fn opj_tcd_init(p_tcd: &mut Tcd, image: &CoreImage, cp: &mut Cp) -> OPJ_BOOL {
  p_tcd.tcd_image.tiles.comps = (0..image.numcomps as usize)
    .map(|_| Default::default())
    .collect();
  p_tcd.tcd_image.tiles.numcomps = image.numcomps;
  p_tcd.tp_pos = cp.m_specific_param.m_enc.m_tp_pos;
  1i32
}

pub(crate) fn opj_alloc_tile_component_data(l_tilec: &mut TcdTileComp) -> OPJ_BOOL {
  let needed = l_tilec.data_size_needed;
  if l_tilec.data.ensure(needed) {
    1i32
  } else {
    l_tilec.data_size_needed = 0i32 as usize;
    0i32
  }
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
 * Copyright (c) 2006-2007, Parvatha Elangovan
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
/* ----------------------------------------------------------------------- */
/* *
 * Initializes tile coding/decoding
 */
/* ----------------------------------------------------------------------- */
pub(crate) fn opj_tcd_init_tile(
  mut p_tcd: &mut Tcd,
  mut p_tile_no: u32,
  mut is_encoder: OPJ_BOOL,
  mut manager: &mut EventMgr,
  image: &mut CoreImage,
  cp: &mut Cp,
) -> OPJ_BOOL {
  let sizeof_block = if is_encoder != 0 {
    core::mem::size_of::<TcdCblkEnc>()
  } else {
    core::mem::size_of::<TcdCblkDec>()
  };
  let mut compno: u32 = 0;
  let mut resno: u32 = 0;
  let mut bandno: u32 = 0;
  let mut precno: u32 = 0;
  let mut cblkno: u32 = 0;
  let mut p: u32 = 0;
  let mut q: u32 = 0;
  let mut l_level_no: u32 = 0;
  let mut l_pdx: u32 = 0;
  let mut l_pdy: u32 = 0;
  let mut l_x0b: i32 = 0;
  let mut l_y0b: i32 = 0;
  let mut l_tx0: u32 = 0;
  let mut l_ty0: u32 = 0;
  /* extent of precincts , top left, bottom right**/
  let mut l_tl_prc_x_start: i32 = 0;
  let mut l_tl_prc_y_start: i32 = 0;
  let mut l_br_prc_x_end: i32 = 0;
  let mut l_br_prc_y_end: i32 = 0;
  /* number of precinct for a resolution */
  let mut l_nb_precincts: u32 = 0;
  /* number of code blocks for a precinct*/
  let mut l_nb_code_blocks: u32 = 0;
  /* size of data for a tile */
  /* tile coordinates */
  let tcp = &cp.tcps[p_tile_no as usize];
  let tw = cp.tw;
  let tx0 = cp.tx0;
  let tdx = cp.tdx;
  let ty0 = cp.ty0;
  let tdy = cp.tdy;
  let m_reduce = cp.m_specific_param.m_dec.m_reduce;
  p = p_tile_no.wrapping_rem(tw);
  q = p_tile_no.wrapping_div(tw);
  /*fprintf(stderr, "Tile coordinate = %d,%d\n", p, q);*/
  /* 4 borders of the tile rescale on the image if necessary */
  l_tx0 = tx0.wrapping_add(p.wrapping_mul(tdx)); /* can't be greater than l_image->x1 so won't overflow */
  p_tcd.tcd_image.tiles.x0 = opj_uint_max(l_tx0, image.x0) as i32;
  p_tcd.tcd_image.tiles.x1 = opj_uint_min(opj_uint_adds(l_tx0, tdx), image.x1) as i32;
  /* all those u32 are casted to i32, let's do some sanity check */
  if p_tcd.tcd_image.tiles.x0 < 0i32 || p_tcd.tcd_image.tiles.x1 <= p_tcd.tcd_image.tiles.x0 {
    event_msg!(manager, EVT_ERROR, "Tile X coordinates are not supported\n",); /* can't be greater than l_image->y1 so won't overflow */
    return 0i32;
  }
  l_ty0 = ty0.wrapping_add(q.wrapping_mul(tdy));
  p_tcd.tcd_image.tiles.y0 = opj_uint_max(l_ty0, image.y0) as i32;
  p_tcd.tcd_image.tiles.y1 = opj_uint_min(opj_uint_adds(l_ty0, tdy), image.y1) as i32;
  /* all those u32 are casted to i32, let's do some sanity check */
  if p_tcd.tcd_image.tiles.y0 < 0i32 || p_tcd.tcd_image.tiles.y1 <= p_tcd.tcd_image.tiles.y0 {
    event_msg!(manager, EVT_ERROR, "Tile Y coordinates are not supported\n",);
    return 0i32;
  }
  /* testcase 1888.pdf.asan.35.988 */
  if tcp.tccps[0].numresolutions == 0u32 {
    event_msg!(
      manager,
      EVT_ERROR,
      "tiles require at least one resolution\n",
    );
    return 0i32;
  }
  /*fprintf(stderr, "Tile border = %d,%d,%d,%d\n", l_tile->x0, l_tile->y0,l_tile->x1,l_tile->y1);*/
  /*tile->numcomps = image->numcomps; */
  compno = 0_u32; /* compno */
  while compno < p_tcd.tcd_image.tiles.numcomps {
    /*fprintf(stderr, "compno = %d/%d\n", compno, l_tile->numcomps);*/
    let image_comp = &mut image.comps[compno as usize];
    let tccp = &tcp.tccps[compno as usize];
    let tilec = &mut p_tcd.tcd_image.tiles.comps[compno as usize];
    image_comp.resno_decoded = 0_u32;
    /* border of each l_tile component (global) */
    tilec.x0 = opj_int_ceildiv(p_tcd.tcd_image.tiles.x0, image_comp.dx as i32);
    tilec.y0 = opj_int_ceildiv(p_tcd.tcd_image.tiles.y0, image_comp.dy as i32);
    tilec.x1 = opj_int_ceildiv(p_tcd.tcd_image.tiles.x1, image_comp.dx as i32);
    tilec.y1 = opj_int_ceildiv(p_tcd.tcd_image.tiles.y1, image_comp.dy as i32);
    tilec.compno = compno;
    /*fprintf(stderr, "\tTile compo border = %d,%d,%d,%d\n", l_tilec->x0, l_tilec->y0,l_tilec->x1,l_tilec->y1);*/
    tilec.numresolutions = tccp.numresolutions;
    if tccp.numresolutions < m_reduce {
      tilec.minimum_num_resolutions = 1_u32
    } else {
      tilec.minimum_num_resolutions = tccp.numresolutions.wrapping_sub(m_reduce)
    }
    if is_encoder != 0 {
      let mut l_tile_data_size: usize = 0;
      /* compute l_data_size with overflow check */
      let mut w = (tilec.x1 - tilec.x0) as usize;
      let mut h = (tilec.y1 - tilec.y0) as usize;
      /* issue 733, l_data_size == 0U, probably something wrong should be checked before getting here */
      if h > 0 && w > (usize::MAX).wrapping_div(h) {
        event_msg!(
          manager,
          EVT_ERROR,
          "Size of tile data exceeds system limits\n",
        );
        return 0i32;
      }
      l_tile_data_size = w.wrapping_mul(h);
      if (usize::MAX).wrapping_div(core::mem::size_of::<u32>()) < l_tile_data_size {
        event_msg!(
          manager,
          EVT_ERROR,
          "Size of tile data exceeds system limits\n",
        );
        return 0i32;
      }
      l_tile_data_size = l_tile_data_size.wrapping_mul(core::mem::size_of::<u32>());
      tilec.data_size_needed = l_tile_data_size
    }
    tilec.data_win.release();
    tilec.win_x0 = 0_u32;
    tilec.win_y0 = 0_u32;
    tilec.win_x1 = 0_u32;
    tilec.win_y1 = 0_u32;
    /* Grow-only, like the C realloc path: existing entries (and the precinct
     * buffers they own) are preserved, new entries are zeroed. */
    {
      let resolutions = &mut tilec.resolutions;
      if resolutions.len() < tilec.numresolutions as usize {
        resolutions.resize_with(tilec.numresolutions as usize, Default::default);
      }
    }
    l_level_no = tilec.numresolutions;
    /*fprintf(stderr, "\tlevel_no=%d\n",l_level_no);*/
    let mut l_step_size_idx: usize = 0;
    resno = 0_u32; /* resno */
    while resno < tilec.numresolutions {
      /*fprintf(stderr, "\t\tresno = %d/%d\n", resno, l_tilec->numresolutions);*/
      let mut tlcbgxstart: i32 = 0;
      let mut tlcbgystart: i32 = 0;
      let mut cbgwidthexpn: u32 = 0;
      let mut cbgheightexpn: u32 = 0;
      let mut cblkwidthexpn: u32 = 0;
      let mut cblkheightexpn: u32 = 0;
      l_level_no = l_level_no.wrapping_sub(1);
      /*, brcbgxend, brcbgyend*/
      let res = &mut tilec.resolutions[resno as usize];
      res.x0 = opj_int_ceildivpow2(tilec.x0, l_level_no as i32);
      res.y0 = opj_int_ceildivpow2(tilec.y0, l_level_no as i32);
      res.x1 = opj_int_ceildivpow2(tilec.x1, l_level_no as i32);
      res.y1 = opj_int_ceildivpow2(tilec.y1, l_level_no as i32);
      l_pdx = tccp.prcw[resno as usize];
      l_pdy = tccp.prch[resno as usize];
      l_tl_prc_x_start = opj_int_floordivpow2(res.x0, l_pdx as i32) << l_pdx;
      l_tl_prc_y_start = opj_int_floordivpow2(res.y0, l_pdy as i32) << l_pdy;
      let mut tmp = (opj_int_ceildivpow2(res.x1, l_pdx as i32) as u32) << l_pdx;
      if tmp > 2147483647_u32 {
        event_msg!(manager, EVT_ERROR, "Integer overflow\n",);
        return 0i32;
      }
      l_br_prc_x_end = tmp as i32;
      let mut tmp_0 = (opj_int_ceildivpow2(res.y1, l_pdy as i32) as u32) << l_pdy;
      if tmp_0 > 2147483647_u32 {
        event_msg!(manager, EVT_ERROR, "Integer overflow\n",);
        return 0i32;
      }
      l_br_prc_y_end = tmp_0 as i32;
      res.pw = if res.x0 == res.x1 {
        0u32
      } else {
        ((l_br_prc_x_end - l_tl_prc_x_start) >> l_pdx) as u32
      };
      res.ph = if res.y0 == res.y1 {
        0u32
      } else {
        ((l_br_prc_y_end - l_tl_prc_y_start) >> l_pdy) as u32
      };
      if res.pw != 0u32 && (-(1i32) as u32).wrapping_div(res.pw) < res.ph {
        event_msg!(
          manager,
          EVT_ERROR,
          "Size of tile data exceeds system limits\n",
        );
        return 0i32;
      }
      l_nb_precincts = res.pw.wrapping_mul(res.ph);
      if (-(1i32) as u32).wrapping_div(core::mem::size_of::<TcdPrecinct>() as u32) < l_nb_precincts
      {
        event_msg!(
          manager,
          EVT_ERROR,
          "Size of tile data exceeds system limits\n",
        );
        return 0i32;
      }
      if resno == 0u32 {
        tlcbgxstart = l_tl_prc_x_start;
        tlcbgystart = l_tl_prc_y_start;
        /* border for each resolution level (global) */
        /*fprintf(stderr, "\t\t\tres_x0= %d, res_y0 =%d, res_x1=%d, res_y1=%d\n", l_res->x0, l_res->y0, l_res->x1, l_res->y1);*/
        /* p. 35, table A-23, ISO/IEC FDIS154444-1 : 2000 (18 august 2000) */
        /*fprintf(stderr, "\t\t\tpdx=%d, pdy=%d\n", l_pdx, l_pdy);*/
        /* p. 64, B.6, ISO/IEC FDIS15444-1 : 2000 (18 august 2000)  */
        /*fprintf(stderr, "\t\t\tprc_x_start=%d, prc_y_start=%d, br_prc_x_end=%d, br_prc_y_end=%d \n", l_tl_prc_x_start, l_tl_prc_y_start, l_br_prc_x_end ,l_br_prc_y_end );*/
        /*fprintf(stderr, "\t\t\tres_pw=%d, res_ph=%d\n", l_res->pw, l_res->ph );*/
        /*brcbgxend = l_br_prc_x_end;*/
        /* brcbgyend = l_br_prc_y_end;*/
        cbgwidthexpn = l_pdx;
        cbgheightexpn = l_pdy;
        res.numbands = 1_u32
      } else {
        tlcbgxstart = opj_int_ceildivpow2(l_tl_prc_x_start, 1i32);
        tlcbgystart = opj_int_ceildivpow2(l_tl_prc_y_start, 1i32);
        /*brcbgxend = opj_int_ceildivpow2(l_br_prc_x_end, 1);*/
        /*brcbgyend = opj_int_ceildivpow2(l_br_prc_y_end, 1);*/
        cbgwidthexpn = l_pdx.wrapping_sub(1u32); /* bandno */
        cbgheightexpn = l_pdy.wrapping_sub(1u32);
        res.numbands = 3_u32
      }
      cblkwidthexpn = opj_uint_min(tccp.cblkw, cbgwidthexpn);
      cblkheightexpn = opj_uint_min(tccp.cblkh, cbgheightexpn);
      let res_pw = res.pw;
      bandno = 0_u32;
      while bandno < res.numbands {
        /*fprintf(stderr, "\t\t\tband_no=%d/%d\n", bandno, l_res->numbands );*/
        let band = &mut res.bands[bandno as usize];
        if resno == 0u32 {
          band.bandno = 0_u32;
          band.x0 = opj_int_ceildivpow2(tilec.x0, l_level_no as i32);
          band.y0 = opj_int_ceildivpow2(tilec.y0, l_level_no as i32);
          band.x1 = opj_int_ceildivpow2(tilec.x1, l_level_no as i32);
          band.y1 = opj_int_ceildivpow2(tilec.y1, l_level_no as i32)
        } else {
          band.bandno = bandno.wrapping_add(1u32);
          /* x0b = 1 if bandno = 1 or 3 */
          l_x0b = (band.bandno & 1u32) as i32;
          /* y0b = 1 if bandno = 2 or 3 */
          l_y0b = (band.bandno >> 1i32) as i32;
          /* l_band border (global) */
          band.x0 = opj_int64_ceildivpow2(
            tilec.x0 as i64 - ((l_x0b as i64) << l_level_no),
            l_level_no.wrapping_add(1u32) as i32,
          );
          band.y0 = opj_int64_ceildivpow2(
            tilec.y0 as i64 - ((l_y0b as i64) << l_level_no),
            l_level_no.wrapping_add(1u32) as i32,
          );
          band.x1 = opj_int64_ceildivpow2(
            tilec.x1 as i64 - ((l_x0b as i64) << l_level_no),
            l_level_no.wrapping_add(1u32) as i32,
          );
          band.y1 = opj_int64_ceildivpow2(
            tilec.y1 as i64 - ((l_y0b as i64) << l_level_no),
            l_level_no.wrapping_add(1u32) as i32,
          )
        }
        if is_encoder == 0 || opj_tcd_is_band_empty(band) == 0 {
          let step_size = &tccp.stepsizes[l_step_size_idx];
          /* Table E-1 - Sub-band gains */
          /* BUG_WEIRD_TWO_INVK (look for this identifier in dwt.c): */
          /* the test (!is_encoder && l_tccp->qmfbid == 0) is strongly */
          /* linked to the use of two_invK instead of invk */
          let log2_gain = if (is_encoder == 0 && tccp.qmfbid == 0u32) || band.bandno == 0u32 {
            0i32
          } else if band.bandno == 3u32 {
            2i32
          } else {
            1i32
          };
          /* Nominal dynamic range. Equation E-4 */
          let r_b = image_comp.prec as i32 + log2_gain;
          /* Delta_b value of Equation E-3 in "E.1 Inverse quantization
           * procedure" of the standard */
          band.stepsize = ((1.0f64 + step_size.mant as core::ffi::c_double / 2048.0f64)
            * 2.0f64.powf((r_b - step_size.expn) as core::ffi::c_double))
            as f32;
          /* Mb value of Equation E-2 in "E.1 Inverse quantization
           * procedure" of the standard */
          band.numbps = step_size.expn + tccp.numgbits as i32 - 1i32;
          /* Grow-only, like the C realloc path: existing precincts (and the
           * tag trees / code blocks they own) are preserved, new ones are zeroed. */
          {
            let precincts = &mut band.precincts;
            if precincts.len() < l_nb_precincts as usize {
              precincts.resize_with(l_nb_precincts as usize, Default::default);
            }
          }
          let band_x0 = band.x0;
          let band_y0 = band.y0;
          let band_x1 = band.x1;
          let band_y1 = band.y1;
          precno = 0_u32;
          while precno < l_nb_precincts {
            let mut tlcblkxstart: i32 = 0;
            let mut tlcblkystart: i32 = 0;
            let mut brcblkxend: i32 = 0;
            let mut brcblkyend: i32 = 0;
            let mut cbgxstart =
              tlcbgxstart + precno.wrapping_rem(res_pw) as i32 * ((1i32) << cbgwidthexpn);
            let mut cbgystart =
              tlcbgystart + precno.wrapping_div(res_pw) as i32 * ((1i32) << cbgheightexpn);
            let mut cbgxend = cbgxstart + ((1i32) << cbgwidthexpn);
            let mut cbgyend = cbgystart + ((1i32) << cbgheightexpn);
            /*fprintf(stderr, "\t precno=%d; bandno=%d, resno=%d; compno=%d\n", precno, bandno , resno, compno);*/
            /*fprintf(stderr, "\t tlcbgxstart(=%d) + (precno(=%d) percent res->pw(=%d)) * (1 << cbgwidthexpn(=%d)) \n",tlcbgxstart,precno,l_res->pw,cbgwidthexpn);*/
            /* precinct size (global) */
            /*fprintf(stderr, "\t cbgxstart=%d, l_band->x0 = %d \n",cbgxstart, l_band->x0);*/
            let precinct = &mut band.precincts[precno as usize];
            precinct.x0 = opj_int_max(cbgxstart, band_x0);
            precinct.y0 = opj_int_max(cbgystart, band_y0);
            precinct.x1 = opj_int_min(cbgxend, band_x1);
            precinct.y1 = opj_int_min(cbgyend, band_y1);
            /*fprintf(stderr, "\t prc_x0=%d; prc_y0=%d, prc_x1=%d; prc_y1=%d\n",l_current_precinct->x0, l_current_precinct->y0 ,l_current_precinct->x1, l_current_precinct->y1);*/
            tlcblkxstart = opj_int_floordivpow2(precinct.x0, cblkwidthexpn as i32) << cblkwidthexpn;
            /*fprintf(stderr, "\t tlcblkxstart =%d\n",tlcblkxstart );*/
            tlcblkystart =
              opj_int_floordivpow2(precinct.y0, cblkheightexpn as i32) << cblkheightexpn;
            /*fprintf(stderr, "\t tlcblkystart =%d\n",tlcblkystart );*/
            brcblkxend = opj_int_ceildivpow2(precinct.x1, cblkwidthexpn as i32) << cblkwidthexpn;
            /*fprintf(stderr, "\t brcblkxend =%d\n",brcblkxend );*/
            brcblkyend = opj_int_ceildivpow2(precinct.y1, cblkheightexpn as i32) << cblkheightexpn;
            /*fprintf(stderr, "\t brcblkyend =%d\n",brcblkyend );*/
            precinct.cw = ((brcblkxend - tlcblkxstart) >> cblkwidthexpn) as u32;
            precinct.ch = ((brcblkyend - tlcblkystart) >> cblkheightexpn) as u32;
            l_nb_code_blocks = precinct.cw.wrapping_mul(precinct.ch);
            /*fprintf(stderr, "\t\t\t\t precinct_cw = %d x recinct_ch = %d\n",l_current_precinct->cw, l_current_precinct->ch);      */
            if (-(1i32) as u32).wrapping_div(sizeof_block as u32) < l_nb_code_blocks {
              event_msg!(
                manager,
                EVT_ERROR,
                "Size of code block data exceeds system limits\n",
              );
              return 0i32;
            }
            let precinct_cw = precinct.cw;
            let precinct_ch = precinct.ch;
            let precinct_x0 = precinct.x0;
            let precinct_y0 = precinct.y0;
            let precinct_x1 = precinct.x1;
            let precinct_y1 = precinct.y1;
            if is_encoder != 0 {
              precinct.cblks.grow_enc(l_nb_code_blocks as usize);
            } else {
              precinct.cblks.grow_dec(l_nb_code_blocks as usize);
            }
            precinct.incltree = match precinct.incltree.take() {
              Some(tree) => opj_tgt_init(tree, precinct_cw, precinct_ch, manager),
              None => opj_tgt_create(precinct_cw, precinct_ch, manager),
            };
            precinct.imsbtree = match precinct.imsbtree.take() {
              Some(tree) => opj_tgt_init(tree, precinct_cw, precinct_ch, manager),
              None => opj_tgt_create(precinct_cw, precinct_ch, manager),
            };
            cblkno = 0_u32;
            while cblkno < l_nb_code_blocks {
              let mut cblkxstart =
                tlcblkxstart + cblkno.wrapping_rem(precinct_cw) as i32 * ((1i32) << cblkwidthexpn);
              let mut cblkystart =
                tlcblkystart + cblkno.wrapping_div(precinct_cw) as i32 * ((1i32) << cblkheightexpn);
              let mut cblkxend = cblkxstart + ((1i32) << cblkwidthexpn);
              let mut cblkyend = cblkystart + ((1i32) << cblkheightexpn);
              if is_encoder != 0 {
                let code_block = precinct.cblks.enc_mut(cblkno as usize);
                opj_tcd_code_block_enc_allocate(code_block);
                /* code-block size (global) */
                code_block.x0 = opj_int_max(cblkxstart, precinct_x0);
                code_block.y0 = opj_int_max(cblkystart, precinct_y0);
                code_block.x1 = opj_int_min(cblkxend, precinct_x1);
                code_block.y1 = opj_int_min(cblkyend, precinct_y1);
                opj_tcd_code_block_enc_allocate_data(code_block);
              } else {
                let code_block = precinct.cblks.dec_mut(cblkno as usize);
                opj_tcd_code_block_dec_allocate(code_block);
                /* code-block size (global) */
                code_block.x0 = opj_int_max(cblkxstart, precinct_x0);
                code_block.y0 = opj_int_max(cblkystart, precinct_y0);
                code_block.x1 = opj_int_min(cblkxend, precinct_x1);
                code_block.y1 = opj_int_min(cblkyend, precinct_y1)
              }
              cblkno += 1;
            }
            precno += 1;
          }
        }
        /* Do not zero l_band->precints to avoid leaks */
        /* but make sure we don't use it later, since */
        /* it will point to precincts of previous bands... */
        bandno = bandno.wrapping_add(1);
        l_step_size_idx += 1;
      }
      resno += 1;
    }
    compno += 1;
  }
  1i32
}
pub(crate) fn opj_tcd_init_encode_tile(
  mut p_tcd: &mut Tcd,
  mut p_tile_no: u32,
  mut p_manager: &mut EventMgr,
  image: &mut CoreImage,
  cp: &mut Cp,
) -> OPJ_BOOL {
  opj_tcd_init_tile(p_tcd, p_tile_no, 1i32, p_manager, image, cp)
}

pub(crate) fn opj_tcd_init_decode_tile(
  mut p_tcd: &mut Tcd,
  mut p_tile_no: u32,
  mut p_manager: &mut EventMgr,
  image: &mut CoreImage,
  cp: &mut Cp,
) -> OPJ_BOOL {
  opj_tcd_init_tile(p_tcd, p_tile_no, 0i32, p_manager, image, cp)
}
/* *
 * Allocates memory for an encoding code block (but not data).
 */
/* *
 * Allocates memory for an encoding code block (but not data memory).
 */
fn opj_tcd_code_block_enc_allocate(p_code_block: &mut TcdCblkEnc) {
  if p_code_block.layers.is_empty() {
    p_code_block.layers.resize(100, Default::default());
  }
  if p_code_block.passes.is_empty() {
    p_code_block.passes.resize(100, Default::default());
  }
}
/* *
 * Allocates data for an encoding code block
 */
/* *
 * Allocates data memory for an encoding code block.
 */
fn opj_tcd_code_block_enc_allocate_data(p_code_block: &mut TcdCblkEnc) {
  let mut l_data_size: u32 = 0;
  /* +1 is needed for https://github.com/uclouvain/openjpeg/issues/835 */
  /* and actually +2 required for https://github.com/uclouvain/openjpeg/issues/982 */
  /* and +7 for https://github.com/uclouvain/openjpeg/issues/1283 (-M 3) */
  /* and +26 for https://github.com/uclouvain/openjpeg/issues/1283 (-M 7) */
  /* and +28 for https://github.com/uclouvain/openjpeg/issues/1283 (-M 44) */
  /* and +33 for https://github.com/uclouvain/openjpeg/issues/1283 (-M 4) */
  /* and +63 for https://github.com/uclouvain/openjpeg/issues/1283 (-M 4 -IMF 2K) */
  /* and +74 for https://github.com/uclouvain/openjpeg/issues/1283 (-M 4 -n 8 -s 7,7 -I) */
  /* Note: no known theoretical upper bound for the compressed code */
  /* block size ? */
  l_data_size = (74u32).wrapping_add(
    ((p_code_block.x1 - p_code_block.x0)
      * (p_code_block.y1 - p_code_block.y0)
      * core::mem::size_of::<u32>() as i32) as u32,
  );
  if l_data_size > p_code_block.data_size {
    p_code_block.data.clear();
    /* +1 for the leading sentinel byte the MQ coder reads through `data[-1]` */
    p_code_block.data.resize(l_data_size as usize + 1, 0);
    p_code_block.data_size = l_data_size;
  }
}

pub(crate) fn opj_tcd_reinit_segment(seg: &mut TcdSeg) {
  *seg = TcdSeg::default();
}
/* We reserve the initial byte as a fake byte to a non-FF value */
/* and increment the data pointer, so that opj_mqc_init_enc() */
/* can do bp = data - 1, and opj_mqc_byteout() can safely dereference */
/* it. */
/* *
* Allocates memory for a decoding code block.
*/
/* *
 * Allocates memory for a decoding code block.
 */
fn opj_tcd_code_block_dec_allocate(p_code_block: &mut TcdCblkDec) {
  if p_code_block.segs.is_empty() {
    p_code_block.segs.resize(10, Default::default());
  } else {
    /* Reset every scalar but keep the seg/chunk buffers for reuse. */
    let segs = core::mem::take(&mut p_code_block.segs);
    let chunks = core::mem::take(&mut p_code_block.chunks);
    *p_code_block = Default::default();
    p_code_block.segs = segs;
    p_code_block.chunks = chunks;
    for seg in p_code_block.segs.iter_mut() {
      opj_tcd_reinit_segment(seg);
    }
  }
}

pub(crate) fn opj_tcd_get_decoded_tile_size(
  p_tcd: &mut Tcd,
  take_into_account_partial_decoding: OPJ_BOOL,
  image: &mut CoreImage,
) -> u32 {
  let mut i: u32 = 0;
  let mut l_data_size = 0_u32;
  let mut l_size_comp: u32 = 0;
  let mut l_remaining: u32 = 0;
  let mut l_temp: u32 = 0;
  i = 0_u32;
  while i < image.numcomps {
    let l_tile_comp = &p_tcd.tcd_image.tiles.comps[i as usize];
    let l_img_comp = &image.comps[i as usize];
    let mut w: u32 = 0;
    let mut h: u32 = 0;
    l_size_comp = l_img_comp.prec >> 3i32;
    l_remaining = l_img_comp.prec & 7u32;
    if l_remaining != 0 {
      l_size_comp += 1;
    }
    if l_size_comp == 3u32 {
      l_size_comp = 4_u32
    }
    let l_res =
      &l_tile_comp.resolutions[(l_tile_comp.minimum_num_resolutions.wrapping_sub(1u32)) as usize];
    if take_into_account_partial_decoding != 0 && p_tcd.whole_tile_decoding == 0 {
      w = l_res.win_x1.wrapping_sub(l_res.win_x0);
      h = l_res.win_y1.wrapping_sub(l_res.win_y0)
    } else {
      w = (l_res.x1 - l_res.x0) as u32;
      h = (l_res.y1 - l_res.y0) as u32
    }
    if h > 0u32
      && (2147483647u32)
        .wrapping_mul(2u32)
        .wrapping_add(1u32)
        .wrapping_div(w)
        < h
    {
      return (2147483647u32).wrapping_mul(2u32).wrapping_add(1u32);
    }
    l_temp = w.wrapping_mul(h);
    if l_size_comp != 0
      && (2147483647u32)
        .wrapping_mul(2u32)
        .wrapping_add(1u32)
        .wrapping_div(l_size_comp)
        < l_temp
    {
      return (2147483647u32).wrapping_mul(2u32).wrapping_add(1u32);
    }
    l_temp = (l_temp as core::ffi::c_uint).wrapping_mul(l_size_comp);
    if l_temp
      > (2147483647u32)
        .wrapping_mul(2u32)
        .wrapping_add(1u32)
        .wrapping_sub(l_data_size)
    {
      return (2147483647u32).wrapping_mul(2u32).wrapping_add(1u32);
    }
    l_data_size = (l_data_size as core::ffi::c_uint).wrapping_add(l_temp);
    i += 1;
  }
  l_data_size
}

pub(crate) fn opj_tcd_encode_tile(
  p_tcd: &mut Tcd,
  p_tile_no: u32,
  p_dest: &mut [u8],
  p_data_written: &mut u32,
  p_max_length: u32,
  p_marker_info: Option<&mut TcdMarkerInfo>,
  p_manager: &mut EventMgr,
  image: &mut CoreImage,
  cp: &mut Cp,
) -> OPJ_BOOL {
  if p_tcd.cur_tp_num == 0u32 {
    p_tcd.tcd_tileno = p_tile_no;
    if tprof!("dc_shift_encode", opj_tcd_dc_level_shift_encode(p_tcd, cp)) == 0 {
      return 0i32;
    }
    if tprof!("mct_encode", opj_tcd_mct_encode(p_tcd, image, cp)) == 0 {
      return 0i32;
    }
    if tprof!("dwt_encode", opj_tcd_dwt_encode(p_tcd, cp)) == 0 {
      return 0i32;
    }
    if tprof!("t1_encode", opj_tcd_t1_encode(p_tcd, image, cp)) == 0 {
      return 0i32;
    }
    if opj_tcd_rate_allocate_encode(
      p_tcd,
      &mut p_dest[..p_max_length as usize],
      p_max_length,
      p_manager,
      image,
      cp,
    ) == 0
    {
      return 0i32;
    }
  }
  /* << INDEX */
  /*---------------TILE-------------------*/
  /*--------------TIER2------------------*/
  /* INDEX */
  if opj_tcd_t2_encode(
    p_tcd,
    &mut p_dest[..p_max_length as usize],
    p_data_written,
    p_max_length,
    p_marker_info,
    p_manager,
    image,
    cp,
  ) == 0
  {
    return 0i32;
  }
  1i32
}
pub(crate) fn opj_tcd_decode_tile(
  p_tcd: &mut Tcd,
  win_x0: u32,
  win_y0: u32,
  win_x1: u32,
  win_y1: u32,
  numcomps_to_decode: u32,
  comps_indices: &[u32],
  p_src: &mut [u8],
  p_tile_no: u32,
  p_manager: &mut EventMgr,
  image: &mut CoreImage,
  cp: &mut Cp,
) -> OPJ_BOOL {
  let mut l_data_read: u32 = 0;
  let mut compno: u32 = 0;
  p_tcd.tcd_tileno = p_tile_no;
  p_tcd.win_x0 = win_x0;
  p_tcd.win_y0 = win_y0;
  p_tcd.win_x1 = win_x1;
  p_tcd.win_y1 = win_y1;
  p_tcd.whole_tile_decoding = 1i32;
  if numcomps_to_decode != 0 {
    let mut used_component = vec![false; image.numcomps as usize];
    compno = 0_u32;
    // SAFETY: the caller provides numcomps_to_decode valid component indices.
    let indices = comps_indices;
    for &idx in indices {
      used_component[idx as usize] = true;
    }
    p_tcd.used_component = used_component
  } else {
    p_tcd.used_component = vec![]
  }
  compno = 0_u32;
  while compno < image.numcomps {
    if p_tcd.used_component(compno) && opj_tcd_is_whole_tilecomp_decoding(p_tcd, compno, image) == 0
    {
      p_tcd.whole_tile_decoding = 0i32;
      break;
    }
    compno += 1;
  }
  if p_tcd.whole_tile_decoding != 0 {
    compno = 0_u32;
    while compno < image.numcomps {
      if p_tcd.used_component(compno) {
        let tilec = &mut p_tcd.tcd_image.tiles.comps[compno as usize];
        let res_idx = (tilec.minimum_num_resolutions.wrapping_sub(1u32)) as usize;
        let mut l_data_size: usize = 0;
        /* compute l_data_size with overflow check */
        let mut res_w = (tilec.resolutions[res_idx].x1 - tilec.resolutions[res_idx].x0) as usize;
        let mut res_h = (tilec.resolutions[res_idx].y1 - tilec.resolutions[res_idx].y0) as usize;
        /* issue 733, l_data_size == 0U, probably something wrong should be checked before getting here */
        if res_h > 0 && res_w > (usize::MAX).wrapping_div(res_h) {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Size of tile data exceeds system limits\n",
          );
          return 0i32;
        }
        l_data_size = res_w.wrapping_mul(res_h);
        if (usize::MAX).wrapping_div(core::mem::size_of::<u32>()) < l_data_size {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Size of tile data exceeds system limits\n",
          );
          return 0i32;
        }
        l_data_size = l_data_size.wrapping_mul(core::mem::size_of::<u32>()) as usize as usize;
        tilec.data_size_needed = l_data_size;
        if opj_alloc_tile_component_data(tilec) == 0 {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Size of tile data exceeds system limits\n",
          );
          return 0i32;
        }
      }
      compno += 1;
    }
  } else {
    /* Compute restricted tile-component and tile-resolution coordinates */
    /* of the window of interest, but defer the memory allocation until */
    /* we know the resno_decoded */
    compno = 0_u32;
    while compno < image.numcomps {
      let mut resno: u32 = 0;
      if p_tcd.used_component(compno) {
        let tilec_0 = &mut p_tcd.tcd_image.tiles.comps[compno as usize];
        let image_comp = &image.comps[compno as usize];
        /* Compute the intersection of the area of interest, expressed in tile coordinates */
        /* with the tile coordinates */
        tilec_0.win_x0 = opj_uint_max(
          tilec_0.x0 as u32,
          opj_uint_ceildiv(p_tcd.win_x0, image_comp.dx),
        );
        tilec_0.win_y0 = opj_uint_max(
          tilec_0.y0 as u32,
          opj_uint_ceildiv(p_tcd.win_y0, image_comp.dy),
        );
        tilec_0.win_x1 = opj_uint_min(
          tilec_0.x1 as u32,
          opj_uint_ceildiv(p_tcd.win_x1, image_comp.dx),
        );
        tilec_0.win_y1 = opj_uint_min(
          tilec_0.y1 as u32,
          opj_uint_ceildiv(p_tcd.win_y1, image_comp.dy),
        );
        if tilec_0.win_x1 < tilec_0.win_x0 || tilec_0.win_y1 < tilec_0.win_y0 {
          /* We should not normally go there. The circumstance is when */
          /* the tile coordinates do not intersect the area of interest */
          /* Upper level logic should not even try to decode that tile */
          event_msg!(p_manager, EVT_ERROR, "Invalid tilec->win_xxx values\n",);
          return 0i32;
        }
        resno = 0_u32;
        while resno < tilec_0.numresolutions {
          let res = &mut tilec_0.resolutions[resno as usize];
          res.win_x0 = opj_uint_ceildivpow2(
            tilec_0.win_x0,
            tilec_0
              .numresolutions
              .wrapping_sub(1u32)
              .wrapping_sub(resno),
          );
          res.win_y0 = opj_uint_ceildivpow2(
            tilec_0.win_y0,
            tilec_0
              .numresolutions
              .wrapping_sub(1u32)
              .wrapping_sub(resno),
          );
          res.win_x1 = opj_uint_ceildivpow2(
            tilec_0.win_x1,
            tilec_0
              .numresolutions
              .wrapping_sub(1u32)
              .wrapping_sub(resno),
          );
          res.win_y1 = opj_uint_ceildivpow2(
            tilec_0.win_y1,
            tilec_0
              .numresolutions
              .wrapping_sub(1u32)
              .wrapping_sub(resno),
          );
          resno += 1;
        }
      }
      compno += 1;
    }
  }
  /*--------------TIER2------------------*/
  l_data_read = 0_u32;
  if tprof!("t2_decode", opj_tcd_t2_decode(p_tcd, p_src, &mut l_data_read, p_manager, image, cp)) == 0 {
    return 0i32;
  }
  /*------------------TIER1-----------------*/
  if tprof!("t1_decode", opj_tcd_t1_decode(p_tcd, p_manager, image, cp, p_src)) == 0 {
    return 0i32;
  }
  /* For subtile decoding, now we know the resno_decoded, we can allocate */
  /* the tile data buffer */
  if p_tcd.whole_tile_decoding == 0 {
    compno = 0_u32;
    while compno < image.numcomps {
      if p_tcd.used_component(compno) {
        let tilec_1 = &mut p_tcd.tcd_image.tiles.comps[compno as usize];
        let image_comp_0 = &image.comps[compno as usize];
        let res_0 = &tilec_1.resolutions[image_comp_0.resno_decoded as usize];
        let w = res_0.win_x1.wrapping_sub(res_0.win_x0) as usize;
        let h = res_0.win_y1.wrapping_sub(res_0.win_y0) as usize;
        let mut l_data_size_0: usize = 0;
        tilec_1.data_win.release();
        if w > 0 && h > 0 {
          if w > (usize::MAX).wrapping_div(h) {
            event_msg!(
              p_manager,
              EVT_ERROR,
              "Size of tile data exceeds system limits\n",
            );
            return 0i32;
          }
          l_data_size_0 = w.wrapping_mul(h);
          if l_data_size_0 > (usize::MAX).wrapping_div(core::mem::size_of::<i32>()) {
            event_msg!(
              p_manager,
              EVT_ERROR,
              "Size of tile data exceeds system limits\n",
            );
            return 0i32;
          }
          l_data_size_0 = l_data_size_0.wrapping_mul(core::mem::size_of::<i32>()) as usize as usize;
          if !tilec_1.data_win.alloc(l_data_size_0) {
            event_msg!(
              p_manager,
              EVT_ERROR,
              "Size of tile data exceeds system limits\n",
            );
            return 0i32;
          }
        }
      }
      compno += 1;
    }
  }
  /*----------------DWT---------------------*/
  if tprof!("dwt_decode", opj_tcd_dwt_decode(p_tcd, image, cp)) == 0 {
    return 0i32;
  }
  /*----------------MCT-------------------*/
  if tprof!("mct_decode", opj_tcd_mct_decode(p_tcd, p_manager, image, cp)) == 0 {
    return 0i32;
  }
  if tprof!("dc_shift_decode", opj_tcd_dc_level_shift_decode(p_tcd, image, cp)) == 0 {
    return 0i32;
  }
  /*---------------TILE-------------------*/
  1i32 /*(/ 8)*/
}
pub(crate) fn opj_tcd_update_tile_data(
  p_tcd: &mut Tcd,
  mut p_dest: &mut [u8],
  image: &mut CoreImage,
) -> OPJ_BOOL {
  {
    let mut p_dest_length = p_dest.len() as u32;
    let mut l_data_size = 0_u32;
    let mut l_stride = 0usize;
    let mut l_width = 0usize;
    let mut l_height = 0usize;
    l_data_size = opj_tcd_get_decoded_tile_size(p_tcd, 1i32, image);
    if l_data_size == (2147483647u32).wrapping_mul(2u32).wrapping_add(1u32)
      || l_data_size > p_dest_length
    {
      return 0i32;
    }
    let numcomps = image.numcomps as usize;
    let l_tilec = &mut p_tcd.tcd_image.tiles.comps[..numcomps];
    for (l_tilec, l_img_comp) in l_tilec.iter_mut().zip(image.comps.iter()) {
      let mut l_size_comp = l_img_comp.prec >> 3i32;
      let l_remaining = l_img_comp.prec & 7u32;
      let l_res = &l_tilec.resolutions[l_img_comp.resno_decoded as usize];
      let l_src_data = if p_tcd.whole_tile_decoding != 0 {
        l_width = (l_res.x1 - l_res.x0) as usize;
        l_height = (l_res.y1 - l_res.y0) as usize;
        l_stride =
          ((l_tilec.resolutions[(l_tilec.minimum_num_resolutions.wrapping_sub(1u32)) as usize].x1
            - l_tilec.resolutions[(l_tilec.minimum_num_resolutions.wrapping_sub(1u32)) as usize].x0)
            as usize)
            .wrapping_sub(l_width);
        l_tilec
          .data
          .as_mut_slice((l_width.wrapping_add(l_stride)).wrapping_mul(l_height))
      } else {
        l_width = l_res.win_x1.wrapping_sub(l_res.win_x0) as usize;
        l_height = l_res.win_y1.wrapping_sub(l_res.win_y0) as usize;
        l_stride = 0usize;
        l_tilec
          .data_win
          .as_mut_slice(l_width.wrapping_mul(l_height))
      };
      if l_remaining != 0 {
        l_size_comp += 1;
      }
      if l_size_comp == 3u32 {
        l_size_comp = 4_u32
      }
      let l_nb_elem = l_height * l_width;
      let mut l_src = l_src_data;
      match l_size_comp {
        1 => {
          let (dest, remain) = p_dest.split_at_mut(l_nb_elem);
          p_dest = remain;
          if l_img_comp.sgnd != 0 {
            for (src, dest) in l_src
              .chunks_exact(l_width + l_stride)
              .zip(dest.chunks_exact_mut(l_width))
            {
              let src = &src[0..l_width];
              for (src, dest) in src.iter().zip(dest.iter_mut()) {
                *dest = *src as i8 as u8;
              }
            }
          } else {
            for (src, dest) in l_src
              .chunks_exact(l_width + l_stride)
              .zip(dest.chunks_exact_mut(l_width))
            {
              let src = &src[0..l_width];
              for (src, dest) in src.iter().zip(dest.iter_mut()) {
                *dest = (*src & 0xffi32) as u8;
              }
            }
          }
        }
        2 => {
          let (dest, remain) = p_dest.split_at_mut(l_nb_elem * 2);
          p_dest = remain;
          if l_img_comp.sgnd != 0 {
            for (src, dest) in l_src
              .chunks_exact(l_width + l_stride)
              .zip(dest.chunks_exact_mut(l_width * 2))
            {
              let src = &src[0..l_width];
              let (chunks, _) = dest.as_chunks_mut::<2>();
              for (src, dest) in src.iter().zip(chunks) {
                let val = *src as i16;
                *dest = val.to_ne_bytes();
              }
            }
          } else {
            for (src, dest) in l_src
              .chunks_exact(l_width + l_stride)
              .zip(dest.chunks_exact_mut(l_width * 2))
            {
              let src = &src[0..l_width];
              let (chunks, _) = dest.as_chunks_mut::<2>();
              for (src, dest) in src.iter().zip(chunks) {
                let val = (*src & 0xffffi32) as i16;
                *dest = val.to_ne_bytes();
              }
            }
          }
        }
        4 => {
          let (dest, remain) = p_dest.split_at_mut(l_nb_elem * 4);
          p_dest = remain;
          for (src, dest) in l_src
            .chunks_exact(l_width + l_stride)
            .zip(dest.chunks_exact_mut(l_width * 4))
          {
            let src = &src[0..l_width];
            let (chunks, _) = dest.as_chunks_mut::<4>();
            for (src, dest) in src.iter().zip(chunks) {
              *dest = src.to_ne_bytes();
            }
          }
        }
        _ => {}
      }
    }
    1i32
  }
}

/* *
Free the memory allocated for encoding
@param tcd TCD handle
*/
fn opj_tcd_free_tile(mut p_tcd: &mut Tcd) {
  for l_tile_comp in p_tcd.tcd_image.tiles.comps.iter_mut() {
    for l_res in l_tile_comp.resolutions.iter_mut() {
      for l_band in l_res.bands.iter_mut() {
        for l_precinct in l_band.precincts.iter_mut() {
          l_precinct.incltree = None;
          l_precinct.imsbtree = None;
          l_precinct.cblks = CodeBlocks::Empty;
        }
        l_band.precincts.clear();
      }
    }
    l_tile_comp.resolutions.clear();
    l_tile_comp.data.release();
    l_tile_comp.data_size_needed = 0i32 as usize;
    l_tile_comp.data_win.release();
  }
  p_tcd.tcd_image.tiles.comps.clear();
}
fn opj_tcd_t2_decode(
  mut p_tcd: &mut Tcd,
  p_src_data: &mut [u8],
  p_data_read: &mut u32,
  mut p_manager: &mut EventMgr,
  image: &mut CoreImage,
  cp: &mut Cp,
) -> OPJ_BOOL {
  let mut t2 = opj_t2_create(image, cp);
  if opj_t2_decode_packets(
    p_tcd,
    &mut t2,
    p_tcd.tcd_tileno,
    p_src_data,
    p_data_read,
    p_manager,
  ) == 0
  {
    return 0i32;
  }

  /*---------------CLEAN-------------------*/
  1i32
}
fn opj_tcd_t1_decode(
  p_tcd: &mut Tcd,
  p_manager: &mut EventMgr,
  image: &mut CoreImage,
  cp: &mut Cp,
  src: &mut [u8],
) -> OPJ_BOOL {
  let tcp = &cp.tcps[p_tcd.tcd_tileno as usize];
  let mut compno: u32 = 0;
  let mut ret = 1i32;
  let mut check_pterm = 0i32;
  /* Only enable PTERM check if we decode all layers */
  if tcp.num_layers_to_decode == tcp.numlayers && tcp.tccps[0].cblksty & 0x10u32 != 0u32 {
    check_pterm = 1i32
  }
  compno = 0_u32;
  while compno < p_tcd.tcd_image.tiles.numcomps {
    if p_tcd.used_component(compno) {
      let l_tilec = &mut p_tcd.tcd_image.tiles.comps[compno as usize];
      let view = TcdTilecView {
        win: [p_tcd.win_x0, p_tcd.win_y0, p_tcd.win_x1, p_tcd.win_y1],
        whole_tile_decoding: p_tcd.whole_tile_decoding,
        x0: l_tilec.x0,
        y0: l_tilec.y0,
        x1: l_tilec.x1,
        y1: l_tilec.y1,
        numresolutions: l_tilec.numresolutions,
        minimum_num_resolutions: l_tilec.minimum_num_resolutions,
        compno: l_tilec.compno,
      };
      opj_t1_decode_cblks(
        &view,
        &tcp.tccps,
        &image.comps,
        &mut ret,
        l_tilec,
        &tcp.tccps[compno as usize],
        p_manager,
        check_pterm,
        src,
      );
      if ret == 0 {
        break;
      }
    }
    compno = compno.wrapping_add(1);
  }
  ret
}
fn opj_tcd_dwt_decode(p_tcd: &mut Tcd, image: &mut CoreImage, cp: &mut Cp) -> OPJ_BOOL {
  // SAFETY: `tcp`/`image` are set by opj_tcd_init and valid here.
  let tccps = &cp.tcps[p_tcd.tcd_tileno as usize].tccps;
  let img_comps = &image.comps;
  /* Parallel across components: each tile component's inverse transform is
  independent (the t1 pass already wrote disjoint buffers per component). */
  let whole_tile = p_tcd.whole_tile_decoding;
  let used: Vec<bool> = (0..p_tcd.tcd_image.tiles.numcomps)
    .map(|c| p_tcd.used_component(c))
    .collect();
  let results: Vec<OPJ_BOOL> = par::map_collect(
    &mut p_tcd.tcd_image.tiles.comps,
    1,
    |compno, l_tile_comp| {
      if !used[compno] {
        return 1i32;
      }
      let resno_decoded = img_comps[compno].resno_decoded.wrapping_add(1u32);
      if tccps[compno].qmfbid == 1u32 {
        opj_dwt_decode(whole_tile, l_tile_comp, resno_decoded)
      } else {
        opj_dwt_decode_real(whole_tile, l_tile_comp, resno_decoded)
      }
    },
  );
  if results.contains(&0) {
    return 0i32;
  }
  1i32
}
/// Runs `f` on a bit-exact `&mut [f32]` view of the i32 buffer: the float MCT
/// stores f32 bit patterns in the i32 component data (the C code reinterprets
/// in place; here we convert per element).
fn with_f32_view<R>(data: &mut [i32], f: impl FnOnce(&mut [f32]) -> R) -> R {
  let mut fdata: Vec<f32> = data.iter().map(|&x| f32::from_bits(x as u32)).collect();
  let r = f(&mut fdata);
  for (d, &v) in data.iter_mut().zip(fdata.iter()) {
    *d = v.to_bits() as i32;
  }
  r
}

fn opj_tcd_mct_decode(
  p_tcd: &mut Tcd,
  p_manager: &mut EventMgr,
  image: &mut CoreImage,
  cp: &mut Cp,
) -> OPJ_BOOL {
  let tcp = &cp.tcps[p_tcd.tcd_tileno as usize];
  let mut l_samples: usize = 0;
  if tcp.mct == 0u32 || !p_tcd.used_component.is_empty() {
    return 1i32;
  }
  let min_num_res = p_tcd.tcd_image.tiles.comps[0].minimum_num_resolutions;
  if p_tcd.whole_tile_decoding != 0 {
    let res_comp0 =
      &p_tcd.tcd_image.tiles.comps[0].resolutions[(min_num_res.wrapping_sub(1u32)) as usize];
    /* A bit inefficient: we process more data than needed if */
    /* resno_decoded < l_tile_comp->minimum_num_resolutions-1, */
    /* but we would need to take into account a stride then */
    l_samples =
      ((res_comp0.x1 - res_comp0.x0) as usize).wrapping_mul((res_comp0.y1 - res_comp0.y0) as usize);
    if p_tcd.tcd_image.tiles.numcomps >= 3u32
      && (min_num_res != p_tcd.tcd_image.tiles.comps[1].minimum_num_resolutions
        || min_num_res != p_tcd.tcd_image.tiles.comps[2].minimum_num_resolutions)
    {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Tiles don't all have the same dimension. Skip the MCT step.\n",
      );
      return 0i32;
    }
    if p_tcd.tcd_image.tiles.numcomps >= 3u32 {
      let res_comp1 =
        &p_tcd.tcd_image.tiles.comps[1].resolutions[(min_num_res.wrapping_sub(1u32)) as usize];
      let res_comp2 =
        &p_tcd.tcd_image.tiles.comps[2].resolutions[(min_num_res.wrapping_sub(1u32)) as usize];
      /* testcase 1336.pdf.asan.47.376 */
      if image.comps[0].resno_decoded != image.comps[1].resno_decoded
        || image.comps[0].resno_decoded != image.comps[2].resno_decoded
        || ((res_comp1.x1 - res_comp1.x0) as usize)
          .wrapping_mul((res_comp1.y1 - res_comp1.y0) as usize)
          != l_samples
        || ((res_comp2.x1 - res_comp2.x0) as usize)
          .wrapping_mul((res_comp2.y1 - res_comp2.y0) as usize)
          != l_samples
      {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Tiles don't all have the same dimension. Skip the MCT step.\n",
        );
        return 0i32;
      }
    }
  } else {
    let res_comp0_0 =
      &p_tcd.tcd_image.tiles.comps[0].resolutions[image.comps[0].resno_decoded as usize];
    l_samples = (res_comp0_0.win_x1.wrapping_sub(res_comp0_0.win_x0) as usize)
      .wrapping_mul(res_comp0_0.win_y1.wrapping_sub(res_comp0_0.win_y0) as usize);
    if p_tcd.tcd_image.tiles.numcomps >= 3u32 {
      let res_comp1_0 =
        &p_tcd.tcd_image.tiles.comps[1].resolutions[image.comps[1].resno_decoded as usize];
      let res_comp2_0 =
        &p_tcd.tcd_image.tiles.comps[2].resolutions[image.comps[2].resno_decoded as usize];
      /* testcase 1336.pdf.asan.47.376 */
      if image.comps[0].resno_decoded != image.comps[1].resno_decoded
        || image.comps[0].resno_decoded != image.comps[2].resno_decoded
        || (res_comp1_0.win_x1.wrapping_sub(res_comp1_0.win_x0) as usize)
          .wrapping_mul(res_comp1_0.win_y1.wrapping_sub(res_comp1_0.win_y0) as usize)
          != l_samples
        || (res_comp2_0.win_x1.wrapping_sub(res_comp2_0.win_x0) as usize)
          .wrapping_mul(res_comp2_0.win_y1.wrapping_sub(res_comp2_0.win_y0) as usize)
          != l_samples
      {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Tiles don't all have the same dimension. Skip the MCT step.\n",
        );
        return 0i32;
      }
    }
  }
  if p_tcd.tcd_image.tiles.numcomps >= 3u32 {
    let comps = &mut p_tcd.tcd_image.tiles.comps[..];
    if comps[0].data.is_null() && comps[0].data_win.is_null() {
      /* The tile component data was never allocated (e.g. a zero-sized
      decode window); there is nothing to transform. The C code no-ops here
      because `l_samples` happens to be 0, but constructing a slice from the
      null pointer is UB in Rust. */
      return 1i32;
    }
    /* The whole-tile path targets `data`, the window path `data_win`. */
    let whole = p_tcd.whole_tile_decoding != 0;
    let numcomps = comps.len();
    if tcp.mct == 2u32 {
      if tcp.m_mct_decoding_matrix.is_empty() {
        return 1i32;
      }
      let mut l_data: Vec<Vec<f32>> = Vec::with_capacity(numcomps);
      {
        let mut rest = &mut *comps;
        for _ in 0..numcomps {
          let (first, r) = rest.split_at_mut(1);
          let c = &mut first[0];
          let s = if whole {
            c.data.as_mut_slice(l_samples)
          } else {
            c.data_win.as_mut_slice(l_samples)
          };
          l_data.push(s.iter().map(|&x| f32::from_bits(x as u32)).collect());
          rest = r;
        }
      }
      let matrix = &tcp.m_mct_decoding_matrix[..];
      {
        let mut l_views: Vec<&mut [f32]> = l_data.iter_mut().map(|v| &mut v[..]).collect();
        if opj_mct_decode_custom(matrix, l_samples, &mut l_views, image.comps[0].sgnd) == 0 {
          return 0i32;
        }
      }
      {
        let mut rest = &mut *comps;
        for fdata in &l_data {
          let (first, r) = rest.split_at_mut(1);
          let c = &mut first[0];
          let s = if whole {
            c.data.as_mut_slice(l_samples)
          } else {
            c.data_win.as_mut_slice(l_samples)
          };
          for (d, &v) in s.iter_mut().zip(fdata.iter()) {
            *d = v.to_bits() as i32;
          }
          rest = r;
        }
      }
    } else if tcp.tccps[0].qmfbid == 1u32 {
      let (c0, rest) = comps.split_at_mut(1);
      let (c1, rest) = rest.split_at_mut(1);
      let (c2, _) = rest.split_at_mut(1);
      let s0 = if whole {
        c0[0].data.as_mut_slice(l_samples)
      } else {
        c0[0].data_win.as_mut_slice(l_samples)
      };
      let s1 = if whole {
        c1[0].data.as_mut_slice(l_samples)
      } else {
        c1[0].data_win.as_mut_slice(l_samples)
      };
      let s2 = if whole {
        c2[0].data.as_mut_slice(l_samples)
      } else {
        c2[0].data_win.as_mut_slice(l_samples)
      };
      opj_mct_decode(s0, s1, s2);
    } else {
      let (c0, rest) = comps.split_at_mut(1);
      let (c1, rest) = rest.split_at_mut(1);
      let (c2, _) = rest.split_at_mut(1);
      let s0 = if whole {
        c0[0].data.as_mut_slice(l_samples)
      } else {
        c0[0].data_win.as_mut_slice(l_samples)
      };
      let s1 = if whole {
        c1[0].data.as_mut_slice(l_samples)
      } else {
        c1[0].data_win.as_mut_slice(l_samples)
      };
      let s2 = if whole {
        c2[0].data.as_mut_slice(l_samples)
      } else {
        c2[0].data_win.as_mut_slice(l_samples)
      };
      with_f32_view(s0, |f0| {
        with_f32_view(s1, |f1| {
          with_f32_view(s2, |f2| opj_mct_decode_real(f0, f1, f2))
        })
      });
    }
  } else {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Number of components ({}) is inconsistent with a MCT. Skip the MCT step.\n",
      p_tcd.tcd_image.tiles.numcomps,
    );
  }
  1i32
}
fn opj_tcd_dc_level_shift_decode(p_tcd: &mut Tcd, image: &mut CoreImage, cp: &mut Cp) -> OPJ_BOOL {
  // SAFETY: `tcp`/`image` are set by opj_tcd_init and valid here.
  let tccps = &cp.tcps[p_tcd.tcd_tileno as usize].tccps;
  let img_comps = &image.comps;
  let whole_tile = p_tcd.whole_tile_decoding;
  let used: Vec<bool> = (0..p_tcd.tcd_image.tiles.numcomps)
    .map(|c| p_tcd.used_component(c))
    .collect();
  /* Parallel across components (each shifts its own buffer). */
  par::for_each(&mut p_tcd.tcd_image.tiles.comps, |compno, l_tile_comp| {
    if !used[compno] {
      return;
    }
    let l_img_comp = &img_comps[compno];
    let l_tccp = &tccps[compno];
    let l_res = &l_tile_comp.resolutions[l_img_comp.resno_decoded as usize];
    let (l_width, l_height, l_stride) = if whole_tile == 0 {
      (
        l_res.win_x1.wrapping_sub(l_res.win_x0),
        l_res.win_y1.wrapping_sub(l_res.win_y0),
        0_u32,
      )
    } else {
      let l_width = (l_res.x1 - l_res.x0) as u32;
      let l_height = (l_res.y1 - l_res.y0) as u32;
      let l_stride = ((l_tile_comp.resolutions
        [(l_tile_comp.minimum_num_resolutions.wrapping_sub(1u32)) as usize]
        .x1
        - l_tile_comp.resolutions
          [(l_tile_comp.minimum_num_resolutions.wrapping_sub(1u32)) as usize]
          .x0) as u32)
        .wrapping_sub(l_width);
      assert!(
        l_height == 0u32
          || l_width.wrapping_add(l_stride) as usize
            <= l_tile_comp.data.bytes().wrapping_div(l_height as usize)
      );
      (l_width, l_height, l_stride)
    };

    if l_width != 0 && l_height != 0 {
      let row_stride = (l_width.wrapping_add(l_stride)) as usize;
      let data = if whole_tile == 0 {
        l_tile_comp
          .data_win
          .as_mut_slice(row_stride * (l_height as usize))
      } else {
        l_tile_comp
          .data
          .as_mut_slice(row_stride * (l_height as usize))
      };
      let (l_min, l_max) = if l_img_comp.sgnd != 0 {
        (
          -((1i32) << l_img_comp.prec.wrapping_sub(1u32)),
          ((1i32) << l_img_comp.prec.wrapping_sub(1u32)) - 1i32,
        )
      } else {
        (0i32, ((1u32) << l_img_comp.prec).wrapping_sub(1u32) as i32)
      };

      if l_tccp.qmfbid == 1u32 {
        for j in 0..l_height as usize {
          for i in 0..l_width as usize {
            /* Note: could do the addition in i64 */
            let idx = j * row_stride + i;
            data[idx] = opj_int_clamp(data[idx] + l_tccp.m_dc_level_shift, l_min, l_max);
          }
        }
      } else {
        for j in 0..l_height as usize {
          for i in 0..l_width as usize {
            let idx = j * row_stride + i;
            let l_value = f32::from_bits(data[idx] as u32);
            if l_value > 2147483647 as core::ffi::c_float {
              data[idx] = l_max
            } else if l_value < (-(2147483647i32) - 1i32) as core::ffi::c_float {
              data[idx] = l_min
            } else {
              /* Do addition on int64 to avoid overflows */
              let l_value_int = opj_lrintf(l_value);
              data[idx] = opj_int64_clamp(
                l_value_int + l_tccp.m_dc_level_shift as i64,
                l_min as i64,
                l_max as i64,
              ) as i32
            }
          }
        }
      }
    }
  });
  1i32
}

pub(crate) fn opj_tcd_get_encoder_input_buffer_size(
  mut p_tcd: &mut Tcd,
  image: &CoreImage,
) -> usize {
  let mut l_data_size = 0_usize;
  let mut l_size_comp: u32 = 0;
  let mut l_remaining: u32 = 0;
  let numcomps = image.numcomps as usize;
  let l_tilec = &p_tcd.tcd_image.tiles.comps[..numcomps];
  for (l_tilec, l_img_comp) in l_tilec.iter().zip(image.comps.iter()) {
    l_size_comp = l_img_comp.prec >> 3i32;
    l_remaining = l_img_comp.prec & 7u32;
    if l_remaining != 0 {
      l_size_comp += 1;
    }
    if l_size_comp == 3u32 {
      l_size_comp = 4_u32
    }
    l_data_size = l_data_size.wrapping_add((l_size_comp as usize).wrapping_mul(
      ((l_tilec.x1 - l_tilec.x0) as usize).wrapping_mul((l_tilec.y1 - l_tilec.y0) as usize),
    ));
  }
  l_data_size
}

fn opj_tcd_dc_level_shift_encode(p_tcd: &mut Tcd, cp: &mut Cp) -> OPJ_BOOL {
  // SAFETY: `tcp` is set by opj_tcd_init and valid here.
  let tccps = &cp.tcps[p_tcd.tcd_tileno as usize].tccps;
  /* Parallel across components (each component shifts its own buffer). */
  par::for_each(&mut p_tcd.tcd_image.tiles.comps, |compno, l_tile_comp| {
    let l_tccp = &tccps[compno];
    let l_nb_elem = ((l_tile_comp.x1 - l_tile_comp.x0) as usize)
      .wrapping_mul((l_tile_comp.y1 - l_tile_comp.y0) as usize);
    let data = l_tile_comp.data.as_mut_slice(l_nb_elem);
    if l_tccp.qmfbid == 1u32 {
      for v in data.iter_mut() {
        *v -= l_tccp.m_dc_level_shift;
      }
    } else {
      for v in data.iter_mut() {
        // The C code reads the sample as an i32, shifts it, then stores the
        // result as f32 bits for the 9x7 transform to consume.
        let val = *v;
        *v = ((val - l_tccp.m_dc_level_shift) as f32).to_bits() as i32;
      }
    }
  });
  1i32
}
fn opj_tcd_mct_encode(p_tcd: &mut Tcd, image: &mut CoreImage, cp: &mut Cp) -> OPJ_BOOL {
  let tcp = &cp.tcps[p_tcd.tcd_tileno as usize];
  let samples = ((p_tcd.tcd_image.tiles.comps[0].x1 - p_tcd.tcd_image.tiles.comps[0].x0) as usize)
    .wrapping_mul((p_tcd.tcd_image.tiles.comps[0].y1 - p_tcd.tcd_image.tiles.comps[0].y0) as usize);
  if tcp.mct == 0 {
    return 1i32;
  }
  let comps = &mut p_tcd.tcd_image.tiles.comps[..];
  if tcp.mct == 2u32 {
    if tcp.m_mct_coding_matrix.is_empty() {
      return 1i32;
    }
    let numcomps = comps.len();
    let mut l_data: Vec<&mut [i32]> = Vec::with_capacity(numcomps);
    let mut rest = comps;
    for _ in 0..numcomps {
      let (first, r) = rest.split_at_mut(1);
      l_data.push(first[0].data.as_mut_slice(samples));
      rest = r;
    }
    let matrix = &tcp.m_mct_coding_matrix[..];
    if opj_mct_encode_custom(matrix, samples, &mut l_data, image.comps[0].sgnd) == 0 {
      return 0i32;
    }
  } else if tcp.tccps[0].qmfbid == 0u32 {
    let (c0, rest) = comps.split_at_mut(1);
    let (c1, rest) = rest.split_at_mut(1);
    let (c2, _) = rest.split_at_mut(1);
    with_f32_view(c0[0].data.as_mut_slice(samples), |f0| {
      with_f32_view(c1[0].data.as_mut_slice(samples), |f1| {
        with_f32_view(c2[0].data.as_mut_slice(samples), |f2| {
          opj_mct_encode_real(f0, f1, f2)
        })
      })
    });
  } else {
    let (c0, rest) = comps.split_at_mut(1);
    let (c1, rest) = rest.split_at_mut(1);
    let (c2, _) = rest.split_at_mut(1);
    opj_mct_encode(
      c0[0].data.as_mut_slice(samples),
      c1[0].data.as_mut_slice(samples),
      c2[0].data.as_mut_slice(samples),
    );
  }
  1i32
}
fn opj_tcd_dwt_encode(p_tcd: &mut Tcd, cp: &mut Cp) -> OPJ_BOOL {
  // SAFETY: `tcp` is set by opj_tcd_init and valid here.
  let tccps = &cp.tcps[p_tcd.tcd_tileno as usize].tccps;
  /* Parallel across components (each forward transform is independent). */
  let results: Vec<OPJ_BOOL> = par::map_collect(
    &mut p_tcd.tcd_image.tiles.comps,
    1,
    |compno, l_tile_comp| {
      let qmfbid = tccps[compno].qmfbid;
      if qmfbid == 1u32 {
        opj_dwt_encode(l_tile_comp)
      } else if qmfbid == 0u32 {
        opj_dwt_encode_real(l_tile_comp)
      } else {
        1i32
      }
    },
  );
  if results.contains(&0) {
    return 0i32;
  }
  1i32
}
fn opj_tcd_t1_encode(p_tcd: &mut Tcd, image: &mut CoreImage, cp: &mut Cp) -> OPJ_BOOL {
  // SAFETY: `tcp`/`image` are set by opj_tcd_init and valid here.
  let l_tcp = &cp.tcps[p_tcd.tcd_tileno as usize];
  let mut l_mct_numcomps = 0u32;
  let l_mct_norms: Option<&[f64]> = if l_tcp.mct == 1u32 {
    l_mct_numcomps = 3u32;
    /* irreversible encoding */
    if l_tcp.tccps[0].qmfbid == 0u32 {
      Some(opj_mct_get_mct_norms_real())
    } else {
      Some(opj_mct_get_mct_norms())
    }
  } else {
    l_mct_numcomps = image.numcomps;
    /* An empty norms Vec means no per-component norms apply. */
    if l_tcp.mct_norms.is_empty() {
      None
    } else {
      Some(&l_tcp.mct_norms[..])
    }
  };
  opj_t1_encode_cblks(
    &mut p_tcd.tcd_image.tiles,
    l_tcp,
    l_mct_norms,
    l_mct_numcomps,
  )
}
fn opj_tcd_t2_encode(
  mut p_tcd: &mut Tcd,
  mut dest_data: &mut [u8],
  mut p_data_written: &mut u32,
  mut p_max_dest_size: u32,
  mut p_marker_info: Option<&mut TcdMarkerInfo>,
  mut p_manager: &mut EventMgr,
  image: &mut CoreImage,
  cp: &mut Cp,
) -> OPJ_BOOL {
  let numlayers = cp.tcps[p_tcd.tcd_tileno as usize].numlayers;
  let mut t2 = opj_t2_create(image, cp);
  if opj_t2_encode_packets(
    &mut t2,
    p_tcd.tcd_tileno,
    &mut p_tcd.tcd_image.tiles,
    numlayers,
    &mut dest_data[..p_max_dest_size as usize],
    p_data_written,
    p_marker_info,
    p_tcd.tp_num,
    p_tcd.tp_pos,
    p_tcd.cur_pino,
    FINAL_PASS,
    p_manager,
  ) == 0
  {
    return 0i32;
  }

  /*---------------CLEAN-------------------*/
  1i32
}
fn opj_tcd_rate_allocate_encode(
  mut p_tcd: &mut Tcd,
  mut dest_data: &mut [u8],
  mut p_max_dest_size: u32,
  mut p_manager: &mut EventMgr,
  image: &mut CoreImage,
  cp: &mut Cp,
) -> OPJ_BOOL {
  {
    let mut l_nb_written = 0_u32;

    if cp.m_specific_param.m_enc.m_quality_layer_alloc_strategy
      == J2K_QUALITY_LAYER_ALLOCATION_STRATEGY::RATE_DISTORTION_RATIO
      || cp.m_specific_param.m_enc.m_quality_layer_alloc_strategy
        == J2K_QUALITY_LAYER_ALLOCATION_STRATEGY::FIXED_DISTORTION_RATIO
    {
      if opj_tcd_rateallocate(
        p_tcd,
        &mut dest_data[..p_max_dest_size as usize],
        &mut l_nb_written,
        p_max_dest_size,
        p_manager,
        image,
        cp,
      ) == 0
      {
        return 0i32;
      }
    } else {
      /* Fixed layer allocation */
      opj_tcd_rateallocate_fixed(p_tcd, image, cp); /*(/ 8)*/
    } /* (%8) */
    1i32
  }
}

pub(crate) fn opj_tcd_copy_tile_data(
  p_tcd: &mut Tcd,
  mut p_src: &[u8],
  image: &CoreImage,
) -> OPJ_BOOL {
  let mut l_data_size = 0_usize;
  let mut l_size_comp: u32 = 0;
  let mut l_remaining: u32 = 0;
  let mut l_nb_elem: usize = 0;
  l_data_size = opj_tcd_get_encoder_input_buffer_size(p_tcd, image);
  if l_data_size != p_src.len() {
    return 0i32;
  }
  let numcomps = image.numcomps as usize;
  let l_tilec = &mut p_tcd.tcd_image.tiles.comps[..numcomps];
  for (l_tilec, l_img_comp) in l_tilec.iter_mut().zip(image.comps.iter()) {
    l_size_comp = l_img_comp.prec >> 3i32;
    l_remaining = l_img_comp.prec & 7u32;
    l_nb_elem =
      ((l_tilec.x1 - l_tilec.x0) as usize).wrapping_mul((l_tilec.y1 - l_tilec.y0) as usize);
    if l_remaining != 0 {
      l_size_comp += 1;
    }
    if l_size_comp == 3u32 {
      l_size_comp = 4_u32
    }
    let l_dest = l_tilec.data.as_mut_slice(l_nb_elem);
    match l_size_comp {
      1 => {
        let (src, remain) = p_src.split_at(l_nb_elem);
        if l_img_comp.sgnd != 0 {
          for (src, dest) in src.iter().zip(l_dest.iter_mut()) {
            *dest = *src as i32;
          }
        } else {
          for (src, dest) in src.iter().zip(l_dest.iter_mut()) {
            *dest = (*src as core::ffi::c_int) & 0xffi32;
          }
        }
        p_src = remain;
      }
      2 => {
        let (src, remain) = p_src.split_at(l_nb_elem * 2);
        if l_img_comp.sgnd != 0 {
          let (chunks, _) = src.as_chunks::<2>();
          for (src, dest) in chunks.iter().zip(l_dest.iter_mut()) {
            *dest = u16::from_ne_bytes(*src) as i32;
          }
        } else {
          let (chunks, _) = src.as_chunks::<2>();
          for (src, dest) in chunks.iter().zip(l_dest.iter_mut()) {
            *dest = u16::from_ne_bytes(*src) as i32 & 0xffffi32;
          }
        }
        p_src = remain;
      }
      4 => {
        let (src, remain) = p_src.split_at(l_nb_elem * 4);
        let (chunks, _) = src.as_chunks::<4>();
        for (src, dest) in chunks.iter().zip(l_dest.iter_mut()) {
          *dest = u32::from_ne_bytes(*src) as i32;
        }
        p_src = remain;
      }
      _ => (),
    }
  }
  1i32
}

pub(crate) fn opj_tcd_is_band_empty(band: &TcdBand) -> OPJ_BOOL {
  (band.x1 - band.x0 == 0i32 || band.y1 - band.y0 == 0i32) as core::ffi::c_int
}

/// Read-only snapshot of the tile-component fields the area-of-interest
/// predicates and the per-codeblock decoders need, so the &mut walks over
/// the resolutions/bands/precincts never alias the codec-level structures.
pub(crate) struct TcdTilecView {
  pub win: [u32; 4],
  pub whole_tile_decoding: OPJ_BOOL,
  pub x0: i32,
  pub y0: i32,
  pub x1: i32,
  pub y1: i32,
  pub numresolutions: u32,
  pub minimum_num_resolutions: u32,
  pub compno: u32,
}

pub(crate) fn opj_tcd_is_subband_area_of_interest(
  tilec: &TcdTilecView,
  tccp: &Tccp,
  image_comps: &[CoreImageComp],
  resno: u32,
  bandno: u32,
  band_x0: u32,
  band_y0: u32,
  band_x1: u32,
  band_y1: u32,
) -> OPJ_BOOL {
  {
    /* Note: those values for filter_margin are in part the result of */
    /* experimentation. The value 2 for QMFBID=1 (5x3 filter) can be linked */
    /* to the maximum left/right extension given in tables F.2 and F.3 of the */
    /* standard. The value 3 for QMFBID=0 (9x7 filter) is more suspicious, */
    /* since F.2 and F.3 would lead to 4 instead, so the current 3 might be */
    /* needed to be bumped to 4, in case inconsistencies are found while */
    /* decoding parts of irreversible coded images. */
    /* See opj_dwt_decode_partial_53 and opj_dwt_decode_partial_97 as well */
    let compno = tilec.compno as usize;
    let image_comp = &image_comps[compno];
    let mut filter_margin = if tccp.qmfbid == 1u32 { 2i32 } else { 3i32 } as u32;
    /* Compute the intersection of the area of interest, expressed in tile coordinates */
    /* with the tile coordinates */
    let mut tcx0 = opj_uint_max(
      tilec.x0 as u32,
      opj_uint_ceildiv(tilec.win[0], image_comp.dx),
    );
    let mut tcy0 = opj_uint_max(
      tilec.y0 as u32,
      opj_uint_ceildiv(tilec.win[1], image_comp.dy),
    );
    let mut tcx1 = opj_uint_min(
      tilec.x1 as u32,
      opj_uint_ceildiv(tilec.win[2], image_comp.dx),
    );
    let mut tcy1 = opj_uint_min(
      tilec.y1 as u32,
      opj_uint_ceildiv(tilec.win[3], image_comp.dy),
    );
    /* Compute number of decomposition for this band. See table F-1 */
    let mut nb = if resno == 0u32 {
      tilec.numresolutions.wrapping_sub(1u32)
    } else {
      tilec.numresolutions.wrapping_sub(resno)
    };
    /* Map above tile-based coordinates to sub-band-based coordinates per */
    /* equation B-15 of the standard */
    let mut x0b = bandno & 1u32;
    let mut y0b = bandno >> 1i32;
    let mut tbx0 = if nb == 0u32 {
      tcx0
    } else if tcx0 <= ((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(x0b) {
      0u32
    } else {
      opj_uint_ceildivpow2(
        tcx0.wrapping_sub(((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(x0b)),
        nb,
      )
    };
    let mut tby0 = if nb == 0u32 {
      tcy0
    } else if tcy0 <= ((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(y0b) {
      0u32
    } else {
      opj_uint_ceildivpow2(
        tcy0.wrapping_sub(((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(y0b)),
        nb,
      )
    };
    let mut tbx1 = if nb == 0u32 {
      tcx1
    } else if tcx1 <= ((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(x0b) {
      0u32
    } else {
      opj_uint_ceildivpow2(
        tcx1.wrapping_sub(((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(x0b)),
        nb,
      )
    };
    let mut tby1 = if nb == 0u32 {
      tcy1
    } else if tcy1 <= ((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(y0b) {
      0u32
    } else {
      opj_uint_ceildivpow2(
        tcy1.wrapping_sub(((1u32) << nb.wrapping_sub(1u32)).wrapping_mul(y0b)),
        nb,
      )
    };
    let mut intersects: OPJ_BOOL = 0;
    if tbx0 < filter_margin {
      tbx0 = 0_u32
    } else {
      tbx0 = (tbx0 as core::ffi::c_uint).wrapping_sub(filter_margin)
    }
    if tby0 < filter_margin {
      tby0 = 0_u32
    } else {
      tby0 = (tby0 as core::ffi::c_uint).wrapping_sub(filter_margin)
    }
    tbx1 = opj_uint_adds(tbx1, filter_margin);
    tby1 = opj_uint_adds(tby1, filter_margin);
    intersects =
      (band_x0 < tbx1 && band_y0 < tby1 && band_x1 > tbx0 && band_y1 > tby0) as core::ffi::c_int;
    intersects
  }
}
/* * Returns whether a tile componenent is fully decoded, taking into account
 * p_tcd->win_* members.
 *
 * @param p_tcd    TCD handle.
 * @param compno Component number
 * @return OPJ_TRUE whether the tile componenent is fully decoded
 */
fn opj_tcd_is_whole_tilecomp_decoding(
  p_tcd: &mut Tcd,
  compno: u32,
  image: &mut CoreImage,
) -> OPJ_BOOL {
  {
    let tilec = &p_tcd.tcd_image.tiles.comps[compno as usize];
    let image_comp = &image.comps[compno as usize];
    /* Compute the intersection of the area of interest, expressed in tile coordinates */
    /* with the tile coordinates */
    let mut tcx0 = opj_uint_max(
      tilec.x0 as u32,
      opj_uint_ceildiv(p_tcd.win_x0, image_comp.dx),
    );
    let mut tcy0 = opj_uint_max(
      tilec.y0 as u32,
      opj_uint_ceildiv(p_tcd.win_y0, image_comp.dy),
    );
    let mut tcx1 = opj_uint_min(
      tilec.x1 as u32,
      opj_uint_ceildiv(p_tcd.win_x1, image_comp.dx),
    );
    let mut tcy1 = opj_uint_min(
      tilec.y1 as u32,
      opj_uint_ceildiv(p_tcd.win_y1, image_comp.dy),
    );
    let mut shift = tilec
      .numresolutions
      .wrapping_sub(tilec.minimum_num_resolutions);
    /* Tolerate small margin within the reduced resolution factor to consider if */
    /* the whole tile path must be taken */
    (tcx0 >= tilec.x0 as u32
      && tcy0 >= tilec.y0 as u32
      && tcx1 <= tilec.x1 as u32
      && tcy1 <= tilec.y1 as u32
      && (shift >= 32u32
        || tcx0.wrapping_sub(tilec.x0 as u32) >> shift == 0u32
          && tcy0.wrapping_sub(tilec.y0 as u32) >> shift == 0u32
          && (tilec.x1 as u32).wrapping_sub(tcx1) >> shift == 0u32
          && (tilec.y1 as u32).wrapping_sub(tcy1) >> shift == 0u32)) as core::ffi::c_int
  }
}
/* ----------------------------------------------------------------------- */

/* ----------------------------------------------------------------------- */

/* ----------------------------------------------------------------------- */
