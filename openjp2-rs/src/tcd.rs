use ::libc;
use super::openjpeg::*;
use super::pi::*;
use super::thread::*;
use super::t2::*;
use super::math::*;

extern "C" {
  fn pow(_: libc::c_double, _: libc::c_double) -> libc::c_double;

  fn ceil(_: libc::c_double) -> libc::c_double;

  fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;

  fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;

  fn opj_image_data_alloc(size: OPJ_SIZE_T) -> *mut libc::c_void;

  fn opj_image_data_free(ptr: *mut libc::c_void);

  fn opj_malloc(size: size_t) -> *mut libc::c_void;

  fn opj_calloc(numOfElements: size_t, sizeOfElements: size_t) -> *mut libc::c_void;

  fn opj_aligned_free(ptr: *mut libc::c_void);

  fn opj_realloc(m: *mut libc::c_void, s: size_t) -> *mut libc::c_void;

  fn opj_free(m: *mut libc::c_void);

  fn opj_event_msg(
    event_mgr: *mut opj_event_mgr_t,
    event_type: OPJ_INT32,
    fmt: *const libc::c_char,
    _: ...
  ) -> OPJ_BOOL;

  fn opj_mutex_create() -> *mut opj_mutex_t;

  fn opj_mutex_destroy(mutex: *mut opj_mutex_t);

  fn opj_thread_pool_wait_completion(tp: *mut opj_thread_pool_t, max_remaining_jobs: libc::c_int);

  fn opj_tgt_create(
    numleafsh: OPJ_UINT32,
    numleafsv: OPJ_UINT32,
    p_manager: *mut opj_event_mgr_t,
  ) -> *mut opj_tgt_tree_t;

  fn opj_tgt_init(
    p_tree: *mut opj_tgt_tree_t,
    p_num_leafs_h: OPJ_UINT32,
    p_num_leafs_v: OPJ_UINT32,
    p_manager: *mut opj_event_mgr_t,
  ) -> *mut opj_tgt_tree_t;

  fn opj_tgt_destroy(tree: *mut opj_tgt_tree_t);

  fn opj_t2_create(p_image: *mut opj_image_t, p_cp: *mut opj_cp_t) -> *mut opj_t2_t;

  fn opj_t2_destroy(t2: *mut opj_t2_t);

  fn opj_t2_encode_packets(
    t2: *mut opj_t2_t,
    tileno: OPJ_UINT32,
    tile: *mut opj_tcd_tile_t,
    maxlayers: OPJ_UINT32,
    dest: *mut OPJ_BYTE,
    p_data_written: *mut OPJ_UINT32,
    len: OPJ_UINT32,
    cstr_info: *mut opj_codestream_info_t,
    p_marker_info: *mut opj_tcd_marker_info_t,
    tpnum: OPJ_UINT32,
    tppos: OPJ_INT32,
    pino: OPJ_UINT32,
    t2_mode: J2K_T2_MODE,
    p_manager: *mut opj_event_mgr_t,
  ) -> OPJ_BOOL;

  fn opj_t1_encode_cblks(
    tcd: *mut opj_tcd_t,
    tile: *mut opj_tcd_tile_t,
    tcp: *mut opj_tcp_t,
    mct_norms: *const OPJ_FLOAT64,
    mct_numcomps: OPJ_UINT32,
  ) -> OPJ_BOOL;

  fn opj_mct_get_mct_norms() -> *const OPJ_FLOAT64;

  fn opj_mct_get_mct_norms_real() -> *const OPJ_FLOAT64;

  fn opj_dwt_encode_real(p_tcd: *mut opj_tcd_t, tilec: *mut opj_tcd_tilecomp_t) -> OPJ_BOOL;

  fn opj_dwt_encode(p_tcd: *mut opj_tcd_t, tilec: *mut opj_tcd_tilecomp_t) -> OPJ_BOOL;

  fn opj_mct_encode(c0: *mut OPJ_INT32, c1: *mut OPJ_INT32, c2: *mut OPJ_INT32, n: OPJ_SIZE_T);

  fn opj_mct_encode_real(
    c0: *mut OPJ_FLOAT32,
    c1: *mut OPJ_FLOAT32,
    c2: *mut OPJ_FLOAT32,
    n: OPJ_SIZE_T,
  );

  fn opj_mct_encode_custom(
    p_coding_data: *mut OPJ_BYTE,
    n: OPJ_SIZE_T,
    p_data: *mut *mut OPJ_BYTE,
    p_nb_comp: OPJ_UINT32,
    is_signed: OPJ_UINT32,
  ) -> OPJ_BOOL;

  fn opj_mct_decode_real(
    c0: *mut OPJ_FLOAT32,
    c1: *mut OPJ_FLOAT32,
    c2: *mut OPJ_FLOAT32,
    n: OPJ_SIZE_T,
  );

  fn opj_mct_decode(c0: *mut OPJ_INT32, c1: *mut OPJ_INT32, c2: *mut OPJ_INT32, n: OPJ_SIZE_T);

  fn opj_mct_decode_custom(
    pDecodingData: *mut OPJ_BYTE,
    n: OPJ_SIZE_T,
    pData: *mut *mut OPJ_BYTE,
    pNbComp: OPJ_UINT32,
    isSigned: OPJ_UINT32,
  ) -> OPJ_BOOL;

  fn opj_dwt_decode_real(
    p_tcd: *mut opj_tcd_t,
    tilec: *mut opj_tcd_tilecomp_t,
    numres: OPJ_UINT32,
  ) -> OPJ_BOOL;

  fn opj_dwt_decode(
    p_tcd: *mut opj_tcd_t,
    tilec: *mut opj_tcd_tilecomp_t,
    numres: OPJ_UINT32,
  ) -> OPJ_BOOL;

  fn opj_t1_decode_cblks(
    tcd: *mut opj_tcd_t,
    pret: *mut OPJ_BOOL,
    tilec: *mut opj_tcd_tilecomp_t,
    tccp: *mut opj_tccp_t,
    p_manager: *mut opj_event_mgr_t,
    p_manager_mutex: *mut opj_mutex_t,
    check_pterm: OPJ_BOOL,
  );

  fn opj_t2_decode_packets(
    tcd: *mut opj_tcd_t,
    t2: *mut opj_t2_t,
    tileno: OPJ_UINT32,
    tile: *mut opj_tcd_tile_t,
    src: *mut OPJ_BYTE,
    p_data_read: *mut OPJ_UINT32,
    len: OPJ_UINT32,
    cstr_info: *mut opj_codestream_index_t,
    p_manager: *mut opj_event_mgr_t,
  ) -> OPJ_BOOL;
}

/* ----------------------------------------------------------------------- */
/* *
Create a new TCD handle
*/
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_create(mut p_is_decoder: OPJ_BOOL) -> *mut opj_tcd_t {
  let mut l_tcd = 0 as *mut opj_tcd_t;
  /* create the tcd structure */
  l_tcd = opj_calloc(
    1 as libc::c_int as size_t,
    ::std::mem::size_of::<opj_tcd_t>() as libc::c_ulong,
  ) as *mut opj_tcd_t;
  if l_tcd.is_null() {
    return 0 as *mut opj_tcd_t;
  }
  (*l_tcd).set_m_is_decoder(if p_is_decoder != 0 {
    1 as libc::c_int
  } else {
    0 as libc::c_int
  } as OPJ_BITFIELD);
  (*l_tcd).tcd_image = opj_calloc(
    1 as libc::c_int as size_t,
    ::std::mem::size_of::<opj_tcd_image_t>() as libc::c_ulong,
  ) as *mut opj_tcd_image_t;
  if (*l_tcd).tcd_image.is_null() {
    opj_free(l_tcd as *mut libc::c_void);
    return 0 as *mut opj_tcd_t;
  }
  return l_tcd;
}
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_rateallocate_fixed(mut tcd: *mut opj_tcd_t) {
  let mut layno: OPJ_UINT32 = 0; /* fixed_quality */
  layno = 0 as libc::c_int as OPJ_UINT32;
  while layno < (*(*tcd).tcp).numlayers {
    opj_tcd_makelayer_fixed(tcd, layno, 1 as libc::c_int as OPJ_UINT32);
    layno = layno.wrapping_add(1)
  }
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_makelayer(
  mut tcd: *mut opj_tcd_t,
  mut layno: OPJ_UINT32,
  mut thresh: OPJ_FLOAT64,
  mut final_0: OPJ_UINT32,
) {
  let mut compno: OPJ_UINT32 = 0;
  let mut resno: OPJ_UINT32 = 0;
  let mut bandno: OPJ_UINT32 = 0;
  let mut precno: OPJ_UINT32 = 0;
  let mut cblkno: OPJ_UINT32 = 0;
  let mut passno: OPJ_UINT32 = 0;
  let mut tcd_tile = (*(*tcd).tcd_image).tiles;
  (*tcd_tile).distolayer[layno as usize] = 0 as libc::c_int as OPJ_FLOAT64;
  compno = 0 as libc::c_int as OPJ_UINT32;
  while compno < (*tcd_tile).numcomps {
    let mut tilec: *mut opj_tcd_tilecomp_t =
      &mut *(*tcd_tile).comps.offset(compno as isize) as *mut opj_tcd_tilecomp_t;
    resno = 0 as libc::c_int as OPJ_UINT32;
    while resno < (*tilec).numresolutions {
      let mut res: *mut opj_tcd_resolution_t =
        &mut *(*tilec).resolutions.offset(resno as isize) as *mut opj_tcd_resolution_t;
      bandno = 0 as libc::c_int as OPJ_UINT32;
      while bandno < (*res).numbands {
        let mut band: *mut opj_tcd_band_t =
          &mut *(*res).bands.as_mut_ptr().offset(bandno as isize) as *mut opj_tcd_band_t;
        /* Skip empty bands */
        if !(opj_tcd_is_band_empty(band) != 0) {
          precno = 0 as libc::c_int as OPJ_UINT32;
          while precno < (*res).pw.wrapping_mul((*res).ph) {
            let mut prc: *mut opj_tcd_precinct_t =
              &mut *(*band).precincts.offset(precno as isize) as *mut opj_tcd_precinct_t;
            cblkno = 0 as libc::c_int as OPJ_UINT32;
            while cblkno < (*prc).cw.wrapping_mul((*prc).ch) {
              let mut cblk: *mut opj_tcd_cblk_enc_t =
                &mut *(*prc).cblks.enc.offset(cblkno as isize) as *mut opj_tcd_cblk_enc_t;
              let mut layer: *mut opj_tcd_layer_t =
                &mut *(*cblk).layers.offset(layno as isize) as *mut opj_tcd_layer_t;
              let mut n: OPJ_UINT32 = 0;
              if layno == 0 as libc::c_int as libc::c_uint {
                (*cblk).numpassesinlayers = 0 as libc::c_int as OPJ_UINT32
              }
              n = (*cblk).numpassesinlayers;
              if thresh < 0 as libc::c_int as libc::c_double {
                /* Special value to indicate to use all passes */
                n = (*cblk).totalpasses
              } else {
                passno = (*cblk).numpassesinlayers;
                while passno < (*cblk).totalpasses {
                  let mut dr: OPJ_UINT32 = 0;
                  let mut dd: OPJ_FLOAT64 = 0.;
                  let mut pass: *mut opj_tcd_pass_t =
                    &mut *(*cblk).passes.offset(passno as isize) as *mut opj_tcd_pass_t;
                  if n == 0 as libc::c_int as libc::c_uint {
                    dr = (*pass).rate;
                    dd = (*pass).distortiondec
                  } else {
                    dr = (*pass).rate.wrapping_sub(
                      (*(*cblk)
                        .passes
                        .offset(n.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
                      .rate,
                    );
                    dd = (*pass).distortiondec
                      - (*(*cblk)
                        .passes
                        .offset(n.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
                      .distortiondec
                  }
                  if dr == 0 {
                    if dd != 0 as libc::c_int as libc::c_double {
                      n = passno.wrapping_add(1 as libc::c_int as libc::c_uint)
                    }
                  } else if (thresh - dd / dr as libc::c_double) < 2.2204460492503131e-16f64 {
                    /* do not rely on float equality, check with DBL_EPSILON margin */
                    n = passno.wrapping_add(1 as libc::c_int as libc::c_uint)
                  } /* fixed_quality */
                  passno = passno.wrapping_add(1)
                }
              } /*, matrice[tcd_tcp->numlayers][tcd_tile->comps[0].numresolutions][3]; */
              (*layer).numpasses = n.wrapping_sub((*cblk).numpassesinlayers);
              if (*layer).numpasses == 0 {
                (*layer).disto = 0 as libc::c_int as OPJ_FLOAT64
              } else {
                if (*cblk).numpassesinlayers == 0 as libc::c_int as libc::c_uint {
                  (*layer).len = (*(*cblk)
                    .passes
                    .offset(n.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
                  .rate;
                  (*layer).data = (*cblk).data;
                  (*layer).disto = (*(*cblk)
                    .passes
                    .offset(n.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
                  .distortiondec
                } else {
                  (*layer).len = (*(*cblk)
                    .passes
                    .offset(n.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
                  .rate
                  .wrapping_sub(
                    (*(*cblk).passes.offset(
                      (*cblk)
                        .numpassesinlayers
                        .wrapping_sub(1 as libc::c_int as libc::c_uint)
                        as isize,
                    ))
                    .rate,
                  );
                  (*layer).data = (*cblk).data.offset(
                    (*(*cblk).passes.offset(
                      (*cblk)
                        .numpassesinlayers
                        .wrapping_sub(1 as libc::c_int as libc::c_uint)
                        as isize,
                    ))
                    .rate as isize,
                  );
                  (*layer).disto = (*(*cblk)
                    .passes
                    .offset(n.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
                  .distortiondec
                    - (*(*cblk).passes.offset(
                      (*cblk)
                        .numpassesinlayers
                        .wrapping_sub(1 as libc::c_int as libc::c_uint)
                        as isize,
                    ))
                    .distortiondec
                }
                (*tcd_tile).distolayer[layno as usize] += (*layer).disto;
                if final_0 != 0 {
                  (*cblk).numpassesinlayers = n
                }
              }
              cblkno = cblkno.wrapping_add(1)
            }
            precno = precno.wrapping_add(1)
          }
        }
        bandno = bandno.wrapping_add(1)
      }
      resno = resno.wrapping_add(1)
    }
    compno = compno.wrapping_add(1)
  }
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_makelayer_fixed(
  mut tcd: *mut opj_tcd_t,
  mut layno: OPJ_UINT32,
  mut final_0: OPJ_UINT32,
) {
  let mut compno: OPJ_UINT32 = 0;
  let mut resno: OPJ_UINT32 = 0;
  let mut bandno: OPJ_UINT32 = 0;
  let mut precno: OPJ_UINT32 = 0;
  let mut cblkno: OPJ_UINT32 = 0;
  let mut value: OPJ_INT32 = 0;
  let mut matrice: [[[OPJ_INT32; 3]; 10]; 10] = [[[0; 3]; 10]; 10];
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut k: OPJ_UINT32 = 0;
  let mut cp = (*tcd).cp;
  let mut tcd_tile = (*(*tcd).tcd_image).tiles;
  let mut tcd_tcp = (*tcd).tcp;
  compno = 0 as libc::c_int as OPJ_UINT32;
  while compno < (*tcd_tile).numcomps {
    let mut tilec: *mut opj_tcd_tilecomp_t =
      &mut *(*tcd_tile).comps.offset(compno as isize) as *mut opj_tcd_tilecomp_t;
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < (*tcd_tcp).numlayers {
      j = 0 as libc::c_int as OPJ_UINT32;
      while j < (*tilec).numresolutions {
        k = 0 as libc::c_int as OPJ_UINT32;
        while k < 3 as libc::c_int as libc::c_uint {
          matrice[i as usize][j as usize][k as usize] =
            (*(*cp).m_specific_param.m_enc.m_matrice.offset(
              i.wrapping_mul((*tilec).numresolutions)
                .wrapping_mul(3 as libc::c_int as libc::c_uint)
                .wrapping_add(j.wrapping_mul(3 as libc::c_int as libc::c_uint))
                .wrapping_add(k) as isize,
            ) as OPJ_FLOAT32
              * ((*(*(*tcd).image).comps.offset(compno as isize)).prec as libc::c_double / 16.0f64)
                as OPJ_FLOAT32) as OPJ_INT32;
          k = k.wrapping_add(1)
        }
        j = j.wrapping_add(1)
      }
      i = i.wrapping_add(1)
    }
    resno = 0 as libc::c_int as OPJ_UINT32;
    while resno < (*tilec).numresolutions {
      let mut res: *mut opj_tcd_resolution_t =
        &mut *(*tilec).resolutions.offset(resno as isize) as *mut opj_tcd_resolution_t;
      bandno = 0 as libc::c_int as OPJ_UINT32;
      while bandno < (*res).numbands {
        let mut band: *mut opj_tcd_band_t =
          &mut *(*res).bands.as_mut_ptr().offset(bandno as isize) as *mut opj_tcd_band_t;
        /* Skip empty bands */
        if !(opj_tcd_is_band_empty(band) != 0) {
          precno = 0 as libc::c_int as OPJ_UINT32; /* number of bit-plan equal to zero */
          while precno < (*res).pw.wrapping_mul((*res).ph) {
            let mut prc: *mut opj_tcd_precinct_t =
              &mut *(*band).precincts.offset(precno as isize) as *mut opj_tcd_precinct_t;
            cblkno = 0 as libc::c_int as OPJ_UINT32;
            while cblkno < (*prc).cw.wrapping_mul((*prc).ch) {
              let mut cblk: *mut opj_tcd_cblk_enc_t =
                &mut *(*prc).cblks.enc.offset(cblkno as isize) as *mut opj_tcd_cblk_enc_t;
              let mut layer: *mut opj_tcd_layer_t =
                &mut *(*cblk).layers.offset(layno as isize) as *mut opj_tcd_layer_t;
              let mut n: OPJ_UINT32 = 0;
              let mut imsb = (*(*(*tcd).image).comps.offset(compno as isize))
                .prec
                .wrapping_sub((*cblk).numbps) as OPJ_INT32;
              /* Correction of the matrix of coefficient to include the IMSB information */
              if layno == 0 as libc::c_int as libc::c_uint {
                value = matrice[layno as usize][resno as usize][bandno as usize]; /* fixed_quality */
                if imsb >= value {
                  value = 0 as libc::c_int
                } else {
                  value -= imsb
                }
              } else {
                value = matrice[layno as usize][resno as usize][bandno as usize]
                  - matrice[layno.wrapping_sub(1 as libc::c_int as libc::c_uint) as usize]
                    [resno as usize][bandno as usize]; /* 1.1; fixed_quality */
                if imsb
                  >= matrice[layno.wrapping_sub(1 as libc::c_int as libc::c_uint) as usize]
                    [resno as usize][bandno as usize]
                {
                  value -= imsb
                    - matrice[layno.wrapping_sub(1 as libc::c_int as libc::c_uint) as usize]
                      [resno as usize][bandno as usize]; /* fixed_quality */
                  if value < 0 as libc::c_int {
                    value = 0 as libc::c_int
                  }
                }
              } /* compno */
              if layno == 0 as libc::c_int as libc::c_uint {
                (*cblk).numpassesinlayers = 0 as libc::c_int as OPJ_UINT32
              } /* resno */
              n = (*cblk).numpassesinlayers;
              if (*cblk).numpassesinlayers == 0 as libc::c_int as libc::c_uint {
                if value != 0 as libc::c_int {
                  n = (3 as libc::c_int as libc::c_uint)
                    .wrapping_mul(value as OPJ_UINT32)
                    .wrapping_sub(2 as libc::c_int as libc::c_uint)
                    .wrapping_add((*cblk).numpassesinlayers)
                } else {
                  n = (*cblk).numpassesinlayers
                }
              } else {
                n = (3 as libc::c_int as libc::c_uint)
                  .wrapping_mul(value as OPJ_UINT32)
                  .wrapping_add((*cblk).numpassesinlayers)
              }
              (*layer).numpasses = n.wrapping_sub((*cblk).numpassesinlayers);
              if !((*layer).numpasses == 0) {
                if (*cblk).numpassesinlayers == 0 as libc::c_int as libc::c_uint {
                  (*layer).len = (*(*cblk)
                    .passes
                    .offset(n.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
                  .rate;
                  (*layer).data = (*cblk).data
                } else {
                  (*layer).len = (*(*cblk)
                    .passes
                    .offset(n.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
                  .rate
                  .wrapping_sub(
                    (*(*cblk).passes.offset(
                      (*cblk)
                        .numpassesinlayers
                        .wrapping_sub(1 as libc::c_int as libc::c_uint)
                        as isize,
                    ))
                    .rate,
                  );
                  (*layer).data = (*cblk).data.offset(
                    (*(*cblk).passes.offset(
                      (*cblk)
                        .numpassesinlayers
                        .wrapping_sub(1 as libc::c_int as libc::c_uint)
                        as isize,
                    ))
                    .rate as isize,
                  )
                }
                if final_0 != 0 {
                  (*cblk).numpassesinlayers = n
                }
              }
              cblkno = cblkno.wrapping_add(1)
            }
            precno = precno.wrapping_add(1)
          }
        }
        bandno = bandno.wrapping_add(1)
      }
      resno = resno.wrapping_add(1)
    }
    compno = compno.wrapping_add(1)
  }
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_rateallocate(
  mut tcd: *mut opj_tcd_t,
  mut dest: *mut OPJ_BYTE,
  mut p_data_written: *mut OPJ_UINT32,
  mut len: OPJ_UINT32,
  mut cstr_info: *mut opj_codestream_info_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut compno: OPJ_UINT32 = 0;
  let mut resno: OPJ_UINT32 = 0;
  let mut bandno: OPJ_UINT32 = 0;
  let mut precno: OPJ_UINT32 = 0;
  let mut cblkno: OPJ_UINT32 = 0;
  let mut layno: OPJ_UINT32 = 0;
  let mut passno: OPJ_UINT32 = 0;
  let mut min: OPJ_FLOAT64 = 0.;
  let mut max: OPJ_FLOAT64 = 0.;
  let mut cumdisto: [OPJ_FLOAT64; 100] = [0.; 100];
  let K = 1 as libc::c_int as OPJ_FLOAT64;
  let mut maxSE = 0 as libc::c_int as OPJ_FLOAT64;
  let mut cp = (*tcd).cp;
  let mut tcd_tile = (*(*tcd).tcd_image).tiles;
  let mut tcd_tcp = (*tcd).tcp;
  min = 1.7976931348623157e+308f64;
  max = 0 as libc::c_int as OPJ_FLOAT64;
  (*tcd_tile).numpix = 0 as libc::c_int;
  compno = 0 as libc::c_int as OPJ_UINT32;
  while compno < (*tcd_tile).numcomps {
    let mut tilec: *mut opj_tcd_tilecomp_t =
      &mut *(*tcd_tile).comps.offset(compno as isize) as *mut opj_tcd_tilecomp_t;
    (*tilec).numpix = 0 as libc::c_int;
    resno = 0 as libc::c_int as OPJ_UINT32;
    while resno < (*tilec).numresolutions {
      let mut res: *mut opj_tcd_resolution_t =
        &mut *(*tilec).resolutions.offset(resno as isize) as *mut opj_tcd_resolution_t;
      bandno = 0 as libc::c_int as OPJ_UINT32;
      while bandno < (*res).numbands {
        let mut band: *mut opj_tcd_band_t =
          &mut *(*res).bands.as_mut_ptr().offset(bandno as isize) as *mut opj_tcd_band_t;
        /* bandno */
        /* precno */
        /* Skip empty bands */
        if !(opj_tcd_is_band_empty(band) != 0) {
          precno = 0 as libc::c_int as OPJ_UINT32;
          while precno < (*res).pw.wrapping_mul((*res).ph) {
            let mut prc: *mut opj_tcd_precinct_t =
              &mut *(*band).precincts.offset(precno as isize) as *mut opj_tcd_precinct_t;
            cblkno = 0 as libc::c_int as OPJ_UINT32;
            while cblkno < (*prc).cw.wrapping_mul((*prc).ch) {
              let mut cblk: *mut opj_tcd_cblk_enc_t =
                &mut *(*prc).cblks.enc.offset(cblkno as isize) as *mut opj_tcd_cblk_enc_t;
              /* cbklno */
              passno = 0 as libc::c_int as OPJ_UINT32; /* passno */
              while passno < (*cblk).totalpasses {
                let mut pass: *mut opj_tcd_pass_t =
                  &mut *(*cblk).passes.offset(passno as isize) as *mut opj_tcd_pass_t;
                let mut dr: OPJ_INT32 = 0;
                let mut dd: OPJ_FLOAT64 = 0.;
                let mut rdslope: OPJ_FLOAT64 = 0.;
                if passno == 0 as libc::c_int as libc::c_uint {
                  dr = (*pass).rate as OPJ_INT32;
                  dd = (*pass).distortiondec
                } else {
                  dr = (*pass).rate.wrapping_sub(
                    (*(*cblk)
                      .passes
                      .offset(passno.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
                    .rate,
                  ) as OPJ_INT32;
                  dd = (*pass).distortiondec
                    - (*(*cblk)
                      .passes
                      .offset(passno.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
                    .distortiondec
                }
                if !(dr == 0 as libc::c_int) {
                  rdslope = dd / dr as libc::c_double;
                  if rdslope < min {
                    min = rdslope
                  }
                  if rdslope > max {
                    max = rdslope
                  }
                }
                passno = passno.wrapping_add(1)
              }
              /* fixed_quality */
              (*tcd_tile).numpix += ((*cblk).x1 - (*cblk).x0) * ((*cblk).y1 - (*cblk).y0);
              (*tilec).numpix += ((*cblk).x1 - (*cblk).x0) * ((*cblk).y1 - (*cblk).y0);
              cblkno = cblkno.wrapping_add(1)
            }
            precno = precno.wrapping_add(1)
          }
        }
        bandno = bandno.wrapping_add(1)
      }
      resno = resno.wrapping_add(1)
    }
    maxSE += (((1 as libc::c_int) << (*(*(*tcd).image).comps.offset(compno as isize)).prec)
      as OPJ_FLOAT64
      - 1.0f64)
      * (((1 as libc::c_int) << (*(*(*tcd).image).comps.offset(compno as isize)).prec)
        as OPJ_FLOAT64
        - 1.0f64)
      * (*tilec).numpix as OPJ_FLOAT64;
    compno = compno.wrapping_add(1)
  }
  /* index file */
  if !cstr_info.is_null() {
    let mut tile_info: *mut opj_tile_info_t =
      &mut *(*cstr_info).tile.offset((*tcd).tcd_tileno as isize) as *mut opj_tile_info_t;
    (*tile_info).numpix = (*tcd_tile).numpix;
    (*tile_info).distotile = (*tcd_tile).distotile;
    (*tile_info).thresh = opj_malloc(
      ((*tcd_tcp).numlayers as libc::c_ulong)
        .wrapping_mul(::std::mem::size_of::<OPJ_FLOAT64>() as libc::c_ulong),
    ) as *mut OPJ_FLOAT64;
    if (*tile_info).thresh.is_null() {
      /* FIXME event manager error callback */
      return 0 as libc::c_int;
    }
  } /* fixed_quality */
  layno = 0 as libc::c_int as OPJ_UINT32;
  while layno < (*tcd_tcp).numlayers {
    let mut lo = min;
    let mut hi = max;
    let mut maxlen = if (*tcd_tcp).rates[layno as usize] > 0.0f32 {
      opj_uint_min(
        ceil((*tcd_tcp).rates[layno as usize] as libc::c_double) as OPJ_UINT32,
        len,
      )
    } else {
      len
    };
    let mut goodthresh = 0 as libc::c_int as OPJ_FLOAT64;
    let mut stable_thresh = 0 as libc::c_int as OPJ_FLOAT64;
    let mut i: OPJ_UINT32 = 0;
    let mut distotarget: OPJ_FLOAT64 = 0.;
    /* fixed_quality */
    distotarget = (*tcd_tile).distotile
      - K * maxSE
        / pow(
          10 as libc::c_int as OPJ_FLOAT32 as libc::c_double,
          ((*tcd_tcp).distoratio[layno as usize] / 10 as libc::c_int as libc::c_float)
            as libc::c_double,
        );
    /* Don't try to find an optimal threshold but rather take everything not included yet, if
    -r xx,yy,zz,0   (disto_alloc == 1 and rates == 0)
    -q xx,yy,zz,0   (fixed_quality == 1 and distoratio == 0)
    ==> possible to have some lossy layers and the last layer for sure lossless */
    if (*cp).m_specific_param.m_enc.m_disto_alloc() as libc::c_int == 1 as libc::c_int
      && (*tcd_tcp).rates[layno as usize] > 0.0f32
      || (*cp).m_specific_param.m_enc.m_fixed_quality() as libc::c_int == 1 as libc::c_int
        && (*tcd_tcp).distoratio[layno as usize] as libc::c_double > 0.0f64
    {
      let mut t2 = opj_t2_create((*tcd).image, cp); /* fixed_quality */
      let mut thresh = 0 as libc::c_int as OPJ_FLOAT64;
      if t2.is_null() {
        return 0 as libc::c_int;
      }
      i = 0 as libc::c_int as OPJ_UINT32;
      while i < 128 as libc::c_int as libc::c_uint {
        let mut distoachieved = 0 as libc::c_int as OPJ_FLOAT64;
        thresh = (lo + hi) / 2 as libc::c_int as libc::c_double;
        opj_tcd_makelayer(tcd, layno, thresh, 0 as libc::c_int as OPJ_UINT32);
        if (*cp).m_specific_param.m_enc.m_fixed_quality() != 0 {
          /* fixed_quality */
          if (*cp).rsiz as libc::c_int >= 0x3 as libc::c_int
            && (*cp).rsiz as libc::c_int <= 0x6 as libc::c_int
            || (*cp).rsiz as libc::c_int >= 0x400 as libc::c_int
              && (*cp).rsiz as libc::c_int <= 0x900 as libc::c_int | 0x9b as libc::c_int
          {
            if opj_t2_encode_packets(
              t2,
              (*tcd).tcd_tileno,
              tcd_tile,
              layno.wrapping_add(1 as libc::c_int as libc::c_uint),
              dest,
              p_data_written,
              maxlen,
              cstr_info,
              0 as *mut opj_tcd_marker_info_t,
              (*tcd).cur_tp_num,
              (*tcd).tp_pos,
              (*tcd).cur_pino,
              THRESH_CALC,
              p_manager,
            ) == 0
            {
              lo = thresh
            } else {
              distoachieved = if layno == 0 as libc::c_int as libc::c_uint {
                (*tcd_tile).distolayer[0 as libc::c_int as usize]
              } else {
                (cumdisto[layno.wrapping_sub(1 as libc::c_int as libc::c_uint) as usize])
                  + (*tcd_tile).distolayer[layno as usize]
              };
              if distoachieved < distotarget {
                hi = thresh;
                stable_thresh = thresh
              } else {
                lo = thresh
              }
            }
          } else {
            distoachieved = if layno == 0 as libc::c_int as libc::c_uint {
              (*tcd_tile).distolayer[0 as libc::c_int as usize]
            } else {
              (cumdisto[layno.wrapping_sub(1 as libc::c_int as libc::c_uint) as usize])
                + (*tcd_tile).distolayer[layno as usize]
            };
            if distoachieved < distotarget {
              hi = thresh;
              stable_thresh = thresh
            } else {
              lo = thresh
            }
          }
        } else if opj_t2_encode_packets(
          t2,
          (*tcd).tcd_tileno,
          tcd_tile,
          layno.wrapping_add(1 as libc::c_int as libc::c_uint),
          dest,
          p_data_written,
          maxlen,
          cstr_info,
          0 as *mut opj_tcd_marker_info_t,
          (*tcd).cur_tp_num,
          (*tcd).tp_pos,
          (*tcd).cur_pino,
          THRESH_CALC,
          p_manager,
        ) == 0
        {
          /* TODO: what to do with l ??? seek / tell ??? */
          /* opj_event_msg(tcd->cinfo, EVT_INFO, "rate alloc: len=%d, max=%d\n", l, maxlen); */
          lo = thresh
        } else {
          hi = thresh;
          stable_thresh = thresh
        }
        i = i.wrapping_add(1)
      }
      goodthresh = if stable_thresh == 0 as libc::c_int as libc::c_double {
        thresh
      } else {
        stable_thresh
      };
      opj_t2_destroy(t2);
    } else {
      /* Special value to indicate to use all passes */
      goodthresh = -(1 as libc::c_int) as OPJ_FLOAT64
    }
    if !cstr_info.is_null() {
      /* Threshold for Marcela Index */
      *(*(*cstr_info).tile.offset((*tcd).tcd_tileno as isize))
        .thresh
        .offset(layno as isize) = goodthresh
    }
    opj_tcd_makelayer(tcd, layno, goodthresh, 1 as libc::c_int as OPJ_UINT32);
    /* fixed_quality */
    cumdisto[layno as usize] = if layno == 0 as libc::c_int as libc::c_uint {
      (*tcd_tile).distolayer[0 as libc::c_int as usize]
    } else {
      (cumdisto[layno.wrapping_sub(1 as libc::c_int as libc::c_uint) as usize])
        + (*tcd_tile).distolayer[layno as usize]
    };
    layno = layno.wrapping_add(1)
  }
  return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_init(
  mut p_tcd: *mut opj_tcd_t,
  mut p_image: *mut opj_image_t,
  mut p_cp: *mut opj_cp_t,
  mut p_tp: *mut opj_thread_pool_t,
) -> OPJ_BOOL {
  (*p_tcd).image = p_image;
  (*p_tcd).cp = p_cp;
  (*(*p_tcd).tcd_image).tiles = opj_calloc(
    1 as libc::c_int as size_t,
    ::std::mem::size_of::<opj_tcd_tile_t>() as libc::c_ulong,
  ) as *mut opj_tcd_tile_t;
  if (*(*p_tcd).tcd_image).tiles.is_null() {
    return 0 as libc::c_int;
  }
  (*(*(*p_tcd).tcd_image).tiles).comps = opj_calloc(
    (*p_image).numcomps as size_t,
    ::std::mem::size_of::<opj_tcd_tilecomp_t>() as libc::c_ulong,
  ) as *mut opj_tcd_tilecomp_t;
  if (*(*(*p_tcd).tcd_image).tiles).comps.is_null() {
    return 0 as libc::c_int;
  }
  (*(*(*p_tcd).tcd_image).tiles).numcomps = (*p_image).numcomps;
  (*p_tcd).tp_pos = (*p_cp).m_specific_param.m_enc.m_tp_pos;
  (*p_tcd).thread_pool = p_tp;
  return 1 as libc::c_int;
}
/* *
Destroy a previously created TCD handle
*/
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_destroy(mut tcd: *mut opj_tcd_t) {
  if !tcd.is_null() {
    opj_tcd_free_tile(tcd);
    if !(*tcd).tcd_image.is_null() {
      opj_free((*tcd).tcd_image as *mut libc::c_void);
      (*tcd).tcd_image = 0 as *mut opj_tcd_image_t
    }
    opj_free((*tcd).used_component as *mut libc::c_void);
    opj_free(tcd as *mut libc::c_void);
  };
}
#[no_mangle]
pub unsafe extern "C" fn opj_alloc_tile_component_data(
  mut l_tilec: *mut opj_tcd_tilecomp_t,
) -> OPJ_BOOL {
  if (*l_tilec).data.is_null()
    || (*l_tilec).data_size_needed > (*l_tilec).data_size && (*l_tilec).ownsData == 0 as libc::c_int
  {
    (*l_tilec).data = opj_image_data_alloc((*l_tilec).data_size_needed) as *mut OPJ_INT32;
    if (*l_tilec).data.is_null() && (*l_tilec).data_size_needed != 0 as libc::c_int as libc::c_ulong
    {
      return 0 as libc::c_int;
    }
    /*fprintf(stderr, "tAllocate data of tilec (int): %d x OPJ_UINT32n",l_data_size);*/
    (*l_tilec).data_size = (*l_tilec).data_size_needed;
    (*l_tilec).ownsData = 1 as libc::c_int
  } else if (*l_tilec).data_size_needed > (*l_tilec).data_size {
    /* We don't need to keep old data */
    opj_image_data_free((*l_tilec).data as *mut libc::c_void);
    (*l_tilec).data = opj_image_data_alloc((*l_tilec).data_size_needed) as *mut OPJ_INT32;
    if (*l_tilec).data.is_null() {
      (*l_tilec).data_size = 0 as libc::c_int as size_t;
      (*l_tilec).data_size_needed = 0 as libc::c_int as size_t;
      (*l_tilec).ownsData = 0 as libc::c_int;
      return 0 as libc::c_int;
    }
    /*fprintf(stderr, "tReallocate data of tilec (int): from %d to %d x OPJ_UINT32n", l_tilec->data_size, l_data_size);*/
    (*l_tilec).data_size = (*l_tilec).data_size_needed;
    (*l_tilec).ownsData = 1 as libc::c_int
  }
  return 1 as libc::c_int;
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
/* TODO MSD: */
/* *
 * Initializes tile coding/decoding
 */
/* ----------------------------------------------------------------------- */
#[inline]
unsafe extern "C" fn opj_tcd_init_tile(
  mut p_tcd: *mut opj_tcd_t,
  mut p_tile_no: OPJ_UINT32,
  mut isEncoder: OPJ_BOOL,
  mut sizeof_block: OPJ_SIZE_T,
  mut manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut compno: OPJ_UINT32 = 0;
  let mut resno: OPJ_UINT32 = 0;
  let mut bandno: OPJ_UINT32 = 0;
  let mut precno: OPJ_UINT32 = 0;
  let mut cblkno: OPJ_UINT32 = 0;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tile = 0 as *mut opj_tcd_tile_t;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  let mut l_tilec = 0 as *mut opj_tcd_tilecomp_t;
  let mut l_image_comp = 0 as *mut opj_image_comp_t;
  let mut l_res = 0 as *mut opj_tcd_resolution_t;
  let mut l_band = 0 as *mut opj_tcd_band_t;
  let mut l_step_size = 0 as *mut opj_stepsize_t;
  let mut l_current_precinct = 0 as *mut opj_tcd_precinct_t;
  let mut l_image = 0 as *mut opj_image_t;
  let mut p: OPJ_UINT32 = 0;
  let mut q: OPJ_UINT32 = 0;
  let mut l_level_no: OPJ_UINT32 = 0;
  let mut l_pdx: OPJ_UINT32 = 0;
  let mut l_pdy: OPJ_UINT32 = 0;
  let mut l_x0b: OPJ_INT32 = 0;
  let mut l_y0b: OPJ_INT32 = 0;
  let mut l_tx0: OPJ_UINT32 = 0;
  let mut l_ty0: OPJ_UINT32 = 0;
  /* extent of precincts , top left, bottom right**/
  let mut l_tl_prc_x_start: OPJ_INT32 = 0;
  let mut l_tl_prc_y_start: OPJ_INT32 = 0;
  let mut l_br_prc_x_end: OPJ_INT32 = 0;
  let mut l_br_prc_y_end: OPJ_INT32 = 0;
  /* number of precinct for a resolution */
  let mut l_nb_precincts: OPJ_UINT32 = 0;
  /* room needed to store l_nb_precinct precinct for a resolution */
  let mut l_nb_precinct_size: OPJ_UINT32 = 0;
  /* number of code blocks for a precinct*/
  let mut l_nb_code_blocks: OPJ_UINT32 = 0;
  /* room needed to store l_nb_code_blocks code blocks for a precinct*/
  let mut l_nb_code_blocks_size: OPJ_UINT32 = 0;
  /* size of data for a tile */
  let mut l_data_size: OPJ_UINT32 = 0; /* tile coordinates */
  l_cp = (*p_tcd).cp;
  l_tcp = &mut *(*l_cp).tcps.offset(p_tile_no as isize) as *mut opj_tcp_t;
  l_tile = (*(*p_tcd).tcd_image).tiles;
  l_tccp = (*l_tcp).tccps;
  l_tilec = (*l_tile).comps;
  l_image = (*p_tcd).image;
  l_image_comp = (*(*p_tcd).image).comps;
  p = p_tile_no.wrapping_rem((*l_cp).tw);
  q = p_tile_no.wrapping_div((*l_cp).tw);
  /*fprintf(stderr, "Tile coordinate = %d,%d\n", p, q);*/
  /* 4 borders of the tile rescale on the image if necessary */
  l_tx0 = (*l_cp).tx0.wrapping_add(p.wrapping_mul((*l_cp).tdx)); /* can't be greater than l_image->x1 so won't overflow */
  (*l_tile).x0 = opj_uint_max(l_tx0, (*l_image).x0) as OPJ_INT32;
  (*l_tile).x1 = opj_uint_min(opj_uint_adds(l_tx0, (*l_cp).tdx), (*l_image).x1) as OPJ_INT32;
  /* all those OPJ_UINT32 are casted to OPJ_INT32, let's do some sanity check */
  if (*l_tile).x0 < 0 as libc::c_int || (*l_tile).x1 <= (*l_tile).x0 {
    opj_event_msg(
      manager,
      1 as libc::c_int,
      b"Tile X coordinates are not supported\n\x00" as *const u8 as *const libc::c_char,
    ); /* can't be greater than l_image->y1 so won't overflow */
    return 0 as libc::c_int;
  }
  l_ty0 = (*l_cp).ty0.wrapping_add(q.wrapping_mul((*l_cp).tdy));
  (*l_tile).y0 = opj_uint_max(l_ty0, (*l_image).y0) as OPJ_INT32;
  (*l_tile).y1 = opj_uint_min(opj_uint_adds(l_ty0, (*l_cp).tdy), (*l_image).y1) as OPJ_INT32;
  /* all those OPJ_UINT32 are casted to OPJ_INT32, let's do some sanity check */
  if (*l_tile).y0 < 0 as libc::c_int || (*l_tile).y1 <= (*l_tile).y0 {
    opj_event_msg(
      manager,
      1 as libc::c_int,
      b"Tile Y coordinates are not supported\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0 as libc::c_int;
  }
  /* testcase 1888.pdf.asan.35.988 */
  if (*l_tccp).numresolutions == 0 as libc::c_int as libc::c_uint {
    opj_event_msg(
      manager,
      1 as libc::c_int,
      b"tiles require at least one resolution\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0 as libc::c_int;
  }
  /*fprintf(stderr, "Tile border = %d,%d,%d,%d\n", l_tile->x0, l_tile->y0,l_tile->x1,l_tile->y1);*/
  /*tile->numcomps = image->numcomps; */
  compno = 0 as libc::c_int as OPJ_UINT32; /* compno */
  while compno < (*l_tile).numcomps {
    /*fprintf(stderr, "compno = %d/%d\n", compno, l_tile->numcomps);*/
    (*l_image_comp).resno_decoded = 0 as libc::c_int as OPJ_UINT32;
    /* border of each l_tile component (global) */
    (*l_tilec).x0 = opj_int_ceildiv((*l_tile).x0, (*l_image_comp).dx as OPJ_INT32);
    (*l_tilec).y0 = opj_int_ceildiv((*l_tile).y0, (*l_image_comp).dy as OPJ_INT32);
    (*l_tilec).x1 = opj_int_ceildiv((*l_tile).x1, (*l_image_comp).dx as OPJ_INT32);
    (*l_tilec).y1 = opj_int_ceildiv((*l_tile).y1, (*l_image_comp).dy as OPJ_INT32);
    (*l_tilec).compno = compno;
    /*fprintf(stderr, "\tTile compo border = %d,%d,%d,%d\n", l_tilec->x0, l_tilec->y0,l_tilec->x1,l_tilec->y1);*/
    (*l_tilec).numresolutions = (*l_tccp).numresolutions;
    if (*l_tccp).numresolutions < (*l_cp).m_specific_param.m_dec.m_reduce {
      (*l_tilec).minimum_num_resolutions = 1 as libc::c_int as OPJ_UINT32
    } else {
      (*l_tilec).minimum_num_resolutions = (*l_tccp)
        .numresolutions
        .wrapping_sub((*l_cp).m_specific_param.m_dec.m_reduce)
    }
    if isEncoder != 0 {
      let mut l_tile_data_size: OPJ_SIZE_T = 0;
      /* compute l_data_size with overflow check */
      let mut w = ((*l_tilec).x1 - (*l_tilec).x0) as OPJ_SIZE_T;
      let mut h = ((*l_tilec).y1 - (*l_tilec).y0) as OPJ_SIZE_T;
      /* issue 733, l_data_size == 0U, probably something wrong should be checked before getting here */
      if h > 0 as libc::c_int as libc::c_ulong
        && w > (18446744073709551615 as libc::c_ulong).wrapping_div(h)
      {
        opj_event_msg(
          manager,
          1 as libc::c_int,
          b"Size of tile data exceeds system limits\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0 as libc::c_int;
      }
      l_tile_data_size = w.wrapping_mul(h);
      if (18446744073709551615 as libc::c_ulong)
        .wrapping_div(::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong)
        < l_tile_data_size
      {
        opj_event_msg(
          manager,
          1 as libc::c_int,
          b"Size of tile data exceeds system limits\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0 as libc::c_int;
      }
      l_tile_data_size =
        l_tile_data_size.wrapping_mul(::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong);
      (*l_tilec).data_size_needed = l_tile_data_size
    }
    l_data_size = (*l_tilec)
      .numresolutions
      .wrapping_mul(::std::mem::size_of::<opj_tcd_resolution_t>() as libc::c_ulong as OPJ_UINT32);
    opj_image_data_free((*l_tilec).data_win as *mut libc::c_void);
    (*l_tilec).data_win = 0 as *mut OPJ_INT32;
    (*l_tilec).win_x0 = 0 as libc::c_int as OPJ_UINT32;
    (*l_tilec).win_y0 = 0 as libc::c_int as OPJ_UINT32;
    (*l_tilec).win_x1 = 0 as libc::c_int as OPJ_UINT32;
    (*l_tilec).win_y1 = 0 as libc::c_int as OPJ_UINT32;
    if (*l_tilec).resolutions.is_null() {
      (*l_tilec).resolutions = opj_malloc(l_data_size as size_t) as *mut opj_tcd_resolution_t;
      if (*l_tilec).resolutions.is_null() {
        return 0 as libc::c_int;
      }
      /*fprintf(stderr, "\tAllocate resolutions of tilec (opj_tcd_resolution_t): %d\n",l_data_size);*/
      (*l_tilec).resolutions_size = l_data_size;
      memset(
        (*l_tilec).resolutions as *mut libc::c_void,
        0 as libc::c_int,
        l_data_size as libc::c_ulong,
      );
    } else if l_data_size > (*l_tilec).resolutions_size {
      let mut new_resolutions = opj_realloc(
        (*l_tilec).resolutions as *mut libc::c_void,
        l_data_size as size_t,
      ) as *mut opj_tcd_resolution_t;
      if new_resolutions.is_null() {
        opj_event_msg(
          manager,
          1 as libc::c_int,
          b"Not enough memory for tile resolutions\n\x00" as *const u8 as *const libc::c_char,
        );
        opj_free((*l_tilec).resolutions as *mut libc::c_void);
        (*l_tilec).resolutions = 0 as *mut opj_tcd_resolution_t;
        (*l_tilec).resolutions_size = 0 as libc::c_int as OPJ_UINT32;
        return 0 as libc::c_int;
      }
      (*l_tilec).resolutions = new_resolutions;
      /*fprintf(stderr, "\tReallocate data of tilec (int): from %d to %d x OPJ_UINT32\n", l_tilec->resolutions_size, l_data_size);*/
      memset(
        ((*l_tilec).resolutions as *mut OPJ_BYTE).offset((*l_tilec).resolutions_size as isize)
          as *mut libc::c_void,
        0 as libc::c_int,
        l_data_size.wrapping_sub((*l_tilec).resolutions_size) as libc::c_ulong,
      );
      (*l_tilec).resolutions_size = l_data_size
    }
    l_level_no = (*l_tilec).numresolutions;
    l_res = (*l_tilec).resolutions;
    l_step_size = (*l_tccp).stepsizes.as_mut_ptr();
    /*fprintf(stderr, "\tlevel_no=%d\n",l_level_no);*/
    resno = 0 as libc::c_int as OPJ_UINT32; /* resno */
    while resno < (*l_tilec).numresolutions {
      /*fprintf(stderr, "\t\tresno = %d/%d\n", resno, l_tilec->numresolutions);*/
      let mut tlcbgxstart: OPJ_INT32 = 0;
      let mut tlcbgystart: OPJ_INT32 = 0;
      let mut cbgwidthexpn: OPJ_UINT32 = 0;
      let mut cbgheightexpn: OPJ_UINT32 = 0;
      let mut cblkwidthexpn: OPJ_UINT32 = 0;
      let mut cblkheightexpn: OPJ_UINT32 = 0;
      l_level_no = l_level_no.wrapping_sub(1);
      /*, brcbgxend, brcbgyend*/
      (*l_res).x0 = opj_int_ceildivpow2((*l_tilec).x0, l_level_no as OPJ_INT32);
      (*l_res).y0 = opj_int_ceildivpow2((*l_tilec).y0, l_level_no as OPJ_INT32);
      (*l_res).x1 = opj_int_ceildivpow2((*l_tilec).x1, l_level_no as OPJ_INT32);
      (*l_res).y1 = opj_int_ceildivpow2((*l_tilec).y1, l_level_no as OPJ_INT32);
      l_pdx = (*l_tccp).prcw[resno as usize];
      l_pdy = (*l_tccp).prch[resno as usize];
      l_tl_prc_x_start = opj_int_floordivpow2((*l_res).x0, l_pdx as OPJ_INT32) << l_pdx;
      l_tl_prc_y_start = opj_int_floordivpow2((*l_res).y0, l_pdy as OPJ_INT32) << l_pdy;
      let mut tmp = (opj_int_ceildivpow2((*l_res).x1, l_pdx as OPJ_INT32) as OPJ_UINT32) << l_pdx;
      if tmp > 2147483647 as libc::c_int as OPJ_UINT32 {
        opj_event_msg(
          manager,
          1 as libc::c_int,
          b"Integer overflow\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0 as libc::c_int;
      }
      l_br_prc_x_end = tmp as OPJ_INT32;
      let mut tmp_0 = (opj_int_ceildivpow2((*l_res).y1, l_pdy as OPJ_INT32) as OPJ_UINT32) << l_pdy;
      if tmp_0 > 2147483647 as libc::c_int as OPJ_UINT32 {
        opj_event_msg(
          manager,
          1 as libc::c_int,
          b"Integer overflow\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0 as libc::c_int;
      }
      l_br_prc_y_end = tmp_0 as OPJ_INT32;
      (*l_res).pw = if (*l_res).x0 == (*l_res).x1 {
        0 as libc::c_uint
      } else {
        (l_br_prc_x_end - l_tl_prc_x_start >> l_pdx) as OPJ_UINT32
      };
      (*l_res).ph = if (*l_res).y0 == (*l_res).y1 {
        0 as libc::c_uint
      } else {
        (l_br_prc_y_end - l_tl_prc_y_start >> l_pdy) as OPJ_UINT32
      };
      if (*l_res).pw != 0 as libc::c_uint
        && (-(1 as libc::c_int) as OPJ_UINT32).wrapping_div((*l_res).pw) < (*l_res).ph
      {
        opj_event_msg(
          manager,
          1 as libc::c_int,
          b"Size of tile data exceeds system limits\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0 as libc::c_int;
      }
      l_nb_precincts = (*l_res).pw.wrapping_mul((*l_res).ph);
      if (-(1 as libc::c_int) as OPJ_UINT32)
        .wrapping_div(::std::mem::size_of::<opj_tcd_precinct_t>() as libc::c_ulong as OPJ_UINT32)
        < l_nb_precincts
      {
        opj_event_msg(
          manager,
          1 as libc::c_int,
          b"Size of tile data exceeds system limits\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0 as libc::c_int;
      }
      l_nb_precinct_size = l_nb_precincts
        .wrapping_mul(::std::mem::size_of::<opj_tcd_precinct_t>() as libc::c_ulong as OPJ_UINT32);
      if resno == 0 as libc::c_int as libc::c_uint {
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
        (*l_res).numbands = 1 as libc::c_int as OPJ_UINT32
      } else {
        tlcbgxstart = opj_int_ceildivpow2(l_tl_prc_x_start, 1 as libc::c_int);
        tlcbgystart = opj_int_ceildivpow2(l_tl_prc_y_start, 1 as libc::c_int);
        /*brcbgxend = opj_int_ceildivpow2(l_br_prc_x_end, 1);*/
        /*brcbgyend = opj_int_ceildivpow2(l_br_prc_y_end, 1);*/
        cbgwidthexpn = l_pdx.wrapping_sub(1 as libc::c_int as libc::c_uint); /* bandno */
        cbgheightexpn = l_pdy.wrapping_sub(1 as libc::c_int as libc::c_uint);
        (*l_res).numbands = 3 as libc::c_int as OPJ_UINT32
      }
      cblkwidthexpn = opj_uint_min((*l_tccp).cblkw, cbgwidthexpn);
      cblkheightexpn = opj_uint_min((*l_tccp).cblkh, cbgheightexpn);
      l_band = (*l_res).bands.as_mut_ptr();
      let mut current_block_246: u64;
      bandno = 0 as libc::c_int as OPJ_UINT32;
      while bandno < (*l_res).numbands {
        /*fprintf(stderr, "\t\t\tband_no=%d/%d\n", bandno, l_res->numbands );*/
        if resno == 0 as libc::c_int as libc::c_uint {
          (*l_band).bandno = 0 as libc::c_int as OPJ_UINT32;
          (*l_band).x0 = opj_int_ceildivpow2((*l_tilec).x0, l_level_no as OPJ_INT32);
          (*l_band).y0 = opj_int_ceildivpow2((*l_tilec).y0, l_level_no as OPJ_INT32);
          (*l_band).x1 = opj_int_ceildivpow2((*l_tilec).x1, l_level_no as OPJ_INT32);
          (*l_band).y1 = opj_int_ceildivpow2((*l_tilec).y1, l_level_no as OPJ_INT32)
        } else {
          (*l_band).bandno = bandno.wrapping_add(1 as libc::c_int as libc::c_uint);
          /* x0b = 1 if bandno = 1 or 3 */
          l_x0b = ((*l_band).bandno & 1 as libc::c_int as libc::c_uint) as OPJ_INT32;
          /* y0b = 1 if bandno = 2 or 3 */
          l_y0b = ((*l_band).bandno >> 1 as libc::c_int) as OPJ_INT32;
          /* l_band border (global) */
          (*l_band).x0 = opj_int64_ceildivpow2(
            (*l_tilec).x0 as libc::c_long - ((l_x0b as OPJ_INT64) << l_level_no),
            l_level_no.wrapping_add(1 as libc::c_int as libc::c_uint) as OPJ_INT32,
          );
          (*l_band).y0 = opj_int64_ceildivpow2(
            (*l_tilec).y0 as libc::c_long - ((l_y0b as OPJ_INT64) << l_level_no),
            l_level_no.wrapping_add(1 as libc::c_int as libc::c_uint) as OPJ_INT32,
          );
          (*l_band).x1 = opj_int64_ceildivpow2(
            (*l_tilec).x1 as libc::c_long - ((l_x0b as OPJ_INT64) << l_level_no),
            l_level_no.wrapping_add(1 as libc::c_int as libc::c_uint) as OPJ_INT32,
          );
          (*l_band).y1 = opj_int64_ceildivpow2(
            (*l_tilec).y1 as libc::c_long - ((l_y0b as OPJ_INT64) << l_level_no),
            l_level_no.wrapping_add(1 as libc::c_int as libc::c_uint) as OPJ_INT32,
          )
        }
        if isEncoder != 0 {
          /* precno */
          /* Skip empty bands */
          if opj_tcd_is_band_empty(l_band) != 0 {
            current_block_246 = 10357520176418200368;
          } else {
            current_block_246 = 13895078145312174667;
          }
        } else {
          current_block_246 = 13895078145312174667;
        }
        match current_block_246 {
          13895078145312174667 => {
            /* Table E-1 - Sub-band gains */
            /* BUG_WEIRD_TWO_INVK (look for this identifier in dwt.c): */
            /* the test (!isEncoder && l_tccp->qmfbid == 0) is strongly */
            /* linked to the use of two_invK instead of invK */
            let log2_gain =
              if isEncoder == 0 && (*l_tccp).qmfbid == 0 as libc::c_int as libc::c_uint {
                0 as libc::c_int
              } else if (*l_band).bandno == 0 as libc::c_int as libc::c_uint {
                0 as libc::c_int
              } else if (*l_band).bandno == 3 as libc::c_int as libc::c_uint {
                2 as libc::c_int
              } else {
                1 as libc::c_int
              };
            /* Nominal dynamic range. Equation E-4 */
            let Rb = (*l_image_comp).prec as OPJ_INT32 + log2_gain;
            /* Delta_b value of Equation E-3 in "E.1 Inverse quantization
             * procedure" of the standard */
            (*l_band).stepsize = ((1.0f64 + (*l_step_size).mant as libc::c_double / 2048.0f64)
              * pow(2.0f64, (Rb - (*l_step_size).expn) as libc::c_double))
              as OPJ_FLOAT32;
            /* Mb value of Equation E-2 in "E.1 Inverse quantization
             * procedure" of the standard */
            (*l_band).numbps =
              (*l_step_size).expn + (*l_tccp).numgbits as OPJ_INT32 - 1 as libc::c_int;
            if (*l_band).precincts.is_null() && l_nb_precincts > 0 as libc::c_uint {
              (*l_band).precincts =
                opj_malloc(l_nb_precinct_size as size_t) as *mut opj_tcd_precinct_t;
              if (*l_band).precincts.is_null() {
                opj_event_msg(
                  manager,
                  1 as libc::c_int,
                  b"Not enough memory to handle band precints\n\x00" as *const u8
                    as *const libc::c_char,
                );
                return 0 as libc::c_int;
              }
              /*fprintf(stderr, "\t\t\t\tAllocate precincts of a band (opj_tcd_precinct_t): %d\n",l_nb_precinct_size);     */
              memset(
                (*l_band).precincts as *mut libc::c_void,
                0 as libc::c_int,
                l_nb_precinct_size as libc::c_ulong,
              );
              (*l_band).precincts_data_size = l_nb_precinct_size
            } else if (*l_band).precincts_data_size < l_nb_precinct_size {
              let mut new_precincts = opj_realloc(
                (*l_band).precincts as *mut libc::c_void,
                l_nb_precinct_size as size_t,
              ) as *mut opj_tcd_precinct_t;
              if new_precincts.is_null() {
                opj_event_msg(
                  manager,
                  1 as libc::c_int,
                  b"Not enough memory to handle band precints\n\x00" as *const u8
                    as *const libc::c_char,
                );
                opj_free((*l_band).precincts as *mut libc::c_void);
                (*l_band).precincts = 0 as *mut opj_tcd_precinct_t;
                (*l_band).precincts_data_size = 0 as libc::c_int as OPJ_UINT32;
                return 0 as libc::c_int;
              }
              (*l_band).precincts = new_precincts;
              /*fprintf(stderr, "\t\t\t\tReallocate precincts of a band (opj_tcd_precinct_t): from %d to %d\n",l_band->precincts_data_size, l_nb_precinct_size);*/
              memset(
                ((*l_band).precincts as *mut OPJ_BYTE)
                  .offset((*l_band).precincts_data_size as isize)
                  as *mut libc::c_void,
                0 as libc::c_int,
                l_nb_precinct_size.wrapping_sub((*l_band).precincts_data_size) as libc::c_ulong,
              );
              (*l_band).precincts_data_size = l_nb_precinct_size
            }
            l_current_precinct = (*l_band).precincts;
            precno = 0 as libc::c_int as OPJ_UINT32;
            while precno < l_nb_precincts {
              let mut tlcblkxstart: OPJ_INT32 = 0;
              let mut tlcblkystart: OPJ_INT32 = 0;
              let mut brcblkxend: OPJ_INT32 = 0;
              let mut brcblkyend: OPJ_INT32 = 0;
              let mut cbgxstart = tlcbgxstart
                + precno.wrapping_rem((*l_res).pw) as OPJ_INT32
                  * ((1 as libc::c_int) << cbgwidthexpn);
              let mut cbgystart = tlcbgystart
                + precno.wrapping_div((*l_res).pw) as OPJ_INT32
                  * ((1 as libc::c_int) << cbgheightexpn);
              let mut cbgxend = cbgxstart + ((1 as libc::c_int) << cbgwidthexpn);
              let mut cbgyend = cbgystart + ((1 as libc::c_int) << cbgheightexpn);
              /*fprintf(stderr, "\t precno=%d; bandno=%d, resno=%d; compno=%d\n", precno, bandno , resno, compno);*/
              /*fprintf(stderr, "\t tlcbgxstart(=%d) + (precno(=%d) percent res->pw(=%d)) * (1 << cbgwidthexpn(=%d)) \n",tlcbgxstart,precno,l_res->pw,cbgwidthexpn);*/
              /* precinct size (global) */
              /*fprintf(stderr, "\t cbgxstart=%d, l_band->x0 = %d \n",cbgxstart, l_band->x0);*/
              (*l_current_precinct).x0 = opj_int_max(cbgxstart, (*l_band).x0);
              (*l_current_precinct).y0 = opj_int_max(cbgystart, (*l_band).y0);
              (*l_current_precinct).x1 = opj_int_min(cbgxend, (*l_band).x1);
              (*l_current_precinct).y1 = opj_int_min(cbgyend, (*l_band).y1);
              /*fprintf(stderr, "\t prc_x0=%d; prc_y0=%d, prc_x1=%d; prc_y1=%d\n",l_current_precinct->x0, l_current_precinct->y0 ,l_current_precinct->x1, l_current_precinct->y1);*/
              tlcblkxstart =
                opj_int_floordivpow2((*l_current_precinct).x0, cblkwidthexpn as OPJ_INT32)
                  << cblkwidthexpn;
              /*fprintf(stderr, "\t tlcblkxstart =%d\n",tlcblkxstart );*/
              tlcblkystart =
                opj_int_floordivpow2((*l_current_precinct).y0, cblkheightexpn as OPJ_INT32)
                  << cblkheightexpn;
              /*fprintf(stderr, "\t tlcblkystart =%d\n",tlcblkystart );*/
              brcblkxend =
                opj_int_ceildivpow2((*l_current_precinct).x1, cblkwidthexpn as OPJ_INT32)
                  << cblkwidthexpn;
              /*fprintf(stderr, "\t brcblkxend =%d\n",brcblkxend );*/
              brcblkyend =
                opj_int_ceildivpow2((*l_current_precinct).y1, cblkheightexpn as OPJ_INT32)
                  << cblkheightexpn;
              /*fprintf(stderr, "\t brcblkyend =%d\n",brcblkyend );*/
              (*l_current_precinct).cw = (brcblkxend - tlcblkxstart >> cblkwidthexpn) as OPJ_UINT32;
              (*l_current_precinct).ch =
                (brcblkyend - tlcblkystart >> cblkheightexpn) as OPJ_UINT32;
              l_nb_code_blocks = (*l_current_precinct)
                .cw
                .wrapping_mul((*l_current_precinct).ch);
              /*fprintf(stderr, "\t\t\t\t precinct_cw = %d x recinct_ch = %d\n",l_current_precinct->cw, l_current_precinct->ch);      */
              if (-(1 as libc::c_int) as OPJ_UINT32).wrapping_div(sizeof_block as OPJ_UINT32)
                < l_nb_code_blocks
              {
                opj_event_msg(
                  manager,
                  1 as libc::c_int,
                  b"Size of code block data exceeds system limits\n\x00" as *const u8
                    as *const libc::c_char,
                );
                return 0 as libc::c_int;
              }
              l_nb_code_blocks_size = l_nb_code_blocks.wrapping_mul(sizeof_block as OPJ_UINT32);
              if (*l_current_precinct).cblks.blocks.is_null()
                && l_nb_code_blocks > 0 as libc::c_uint
              {
                (*l_current_precinct).cblks.blocks = opj_malloc(l_nb_code_blocks_size as size_t);
                if (*l_current_precinct).cblks.blocks.is_null() {
                  return 0 as libc::c_int;
                }
                /*fprintf(stderr, "\t\t\t\tAllocate cblks of a precinct (opj_tcd_cblk_dec_t): %d\n",l_nb_code_blocks_size);*/
                memset(
                  (*l_current_precinct).cblks.blocks,
                  0 as libc::c_int,
                  l_nb_code_blocks_size as libc::c_ulong,
                );
                (*l_current_precinct).block_size = l_nb_code_blocks_size
              } else if l_nb_code_blocks_size > (*l_current_precinct).block_size {
                let mut new_blocks = opj_realloc(
                  (*l_current_precinct).cblks.blocks,
                  l_nb_code_blocks_size as size_t,
                );
                if new_blocks.is_null() {
                  opj_free((*l_current_precinct).cblks.blocks);
                  (*l_current_precinct).cblks.blocks = 0 as *mut libc::c_void;
                  (*l_current_precinct).block_size = 0 as libc::c_int as OPJ_UINT32;
                  opj_event_msg(
                    manager,
                    1 as libc::c_int,
                    b"Not enough memory for current precinct codeblock element\n\x00" as *const u8
                      as *const libc::c_char,
                  );
                  return 0 as libc::c_int;
                }
                (*l_current_precinct).cblks.blocks = new_blocks;
                /*fprintf(stderr, "\t\t\t\tReallocate cblks of a precinct (opj_tcd_cblk_dec_t): from %d to %d\n",l_current_precinct->block_size, l_nb_code_blocks_size);     */
                memset(
                  ((*l_current_precinct).cblks.blocks as *mut OPJ_BYTE)
                    .offset((*l_current_precinct).block_size as isize)
                    as *mut libc::c_void,
                  0 as libc::c_int,
                  l_nb_code_blocks_size.wrapping_sub((*l_current_precinct).block_size)
                    as libc::c_ulong,
                );
                (*l_current_precinct).block_size = l_nb_code_blocks_size
              }
              if (*l_current_precinct).incltree.is_null() {
                (*l_current_precinct).incltree =
                  opj_tgt_create((*l_current_precinct).cw, (*l_current_precinct).ch, manager)
              } else {
                (*l_current_precinct).incltree = opj_tgt_init(
                  (*l_current_precinct).incltree,
                  (*l_current_precinct).cw,
                  (*l_current_precinct).ch,
                  manager,
                )
              }
              if (*l_current_precinct).imsbtree.is_null() {
                (*l_current_precinct).imsbtree =
                  opj_tgt_create((*l_current_precinct).cw, (*l_current_precinct).ch, manager)
              } else {
                (*l_current_precinct).imsbtree = opj_tgt_init(
                  (*l_current_precinct).imsbtree,
                  (*l_current_precinct).cw,
                  (*l_current_precinct).ch,
                  manager,
                )
              }
              cblkno = 0 as libc::c_int as OPJ_UINT32;
              while cblkno < l_nb_code_blocks {
                let mut cblkxstart = tlcblkxstart
                  + cblkno.wrapping_rem((*l_current_precinct).cw) as OPJ_INT32
                    * ((1 as libc::c_int) << cblkwidthexpn);
                let mut cblkystart = tlcblkystart
                  + cblkno.wrapping_div((*l_current_precinct).cw) as OPJ_INT32
                    * ((1 as libc::c_int) << cblkheightexpn);
                let mut cblkxend = cblkxstart + ((1 as libc::c_int) << cblkwidthexpn);
                let mut cblkyend = cblkystart + ((1 as libc::c_int) << cblkheightexpn);
                if isEncoder != 0 {
                  let mut l_code_block = (*l_current_precinct).cblks.enc.offset(cblkno as isize);
                  if opj_tcd_code_block_enc_allocate(l_code_block) == 0 {
                    return 0 as libc::c_int;
                  }
                  /* code-block size (global) */
                  (*l_code_block).x0 = opj_int_max(cblkxstart, (*l_current_precinct).x0);
                  (*l_code_block).y0 = opj_int_max(cblkystart, (*l_current_precinct).y0);
                  (*l_code_block).x1 = opj_int_min(cblkxend, (*l_current_precinct).x1);
                  (*l_code_block).y1 = opj_int_min(cblkyend, (*l_current_precinct).y1);
                  if opj_tcd_code_block_enc_allocate_data(l_code_block) == 0 {
                    return 0 as libc::c_int;
                  }
                } else {
                  let mut l_code_block_0 = (*l_current_precinct).cblks.dec.offset(cblkno as isize);
                  if opj_tcd_code_block_dec_allocate(l_code_block_0) == 0 {
                    return 0 as libc::c_int;
                  }
                  /* code-block size (global) */
                  (*l_code_block_0).x0 = opj_int_max(cblkxstart, (*l_current_precinct).x0);
                  (*l_code_block_0).y0 = opj_int_max(cblkystart, (*l_current_precinct).y0);
                  (*l_code_block_0).x1 = opj_int_min(cblkxend, (*l_current_precinct).x1);
                  (*l_code_block_0).y1 = opj_int_min(cblkyend, (*l_current_precinct).y1)
                }
                cblkno = cblkno.wrapping_add(1)
              }
              l_current_precinct = l_current_precinct.offset(1);
              precno = precno.wrapping_add(1)
            }
          }
          _ => {}
        }
        /* Do not zero l_band->precints to avoid leaks */
        /* but make sure we don't use it later, since */
        /* it will point to precincts of previous bands... */
        bandno = bandno.wrapping_add(1);
        l_band = l_band.offset(1);
        l_step_size = l_step_size.offset(1)
      }
      l_res = l_res.offset(1);
      resno = resno.wrapping_add(1)
    }
    l_tccp = l_tccp.offset(1);
    l_tilec = l_tilec.offset(1);
    l_image_comp = l_image_comp.offset(1);
    compno = compno.wrapping_add(1)
  }
  return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_init_encode_tile(
  mut p_tcd: *mut opj_tcd_t,
  mut p_tile_no: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  return opj_tcd_init_tile(
    p_tcd,
    p_tile_no,
    1 as libc::c_int,
    ::std::mem::size_of::<opj_tcd_cblk_enc_t>() as libc::c_ulong,
    p_manager,
  );
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_init_decode_tile(
  mut p_tcd: *mut opj_tcd_t,
  mut p_tile_no: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  return opj_tcd_init_tile(
    p_tcd,
    p_tile_no,
    0 as libc::c_int,
    ::std::mem::size_of::<opj_tcd_cblk_dec_t>() as libc::c_ulong,
    p_manager,
  );
}
/* *
 * Allocates memory for an encoding code block (but not data).
 */
/* *
 * Allocates memory for an encoding code block (but not data memory).
 */
unsafe extern "C" fn opj_tcd_code_block_enc_allocate(
  mut p_code_block: *mut opj_tcd_cblk_enc_t,
) -> OPJ_BOOL {
  if (*p_code_block).layers.is_null() {
    /* no memset since data */
    (*p_code_block).layers = opj_calloc(
      100 as libc::c_int as size_t,
      ::std::mem::size_of::<opj_tcd_layer_t>() as libc::c_ulong,
    ) as *mut opj_tcd_layer_t;
    if (*p_code_block).layers.is_null() {
      return 0 as libc::c_int;
    }
  }
  if (*p_code_block).passes.is_null() {
    (*p_code_block).passes = opj_calloc(
      100 as libc::c_int as size_t,
      ::std::mem::size_of::<opj_tcd_pass_t>() as libc::c_ulong,
    ) as *mut opj_tcd_pass_t;
    if (*p_code_block).passes.is_null() {
      return 0 as libc::c_int;
    }
  }
  return 1 as libc::c_int;
}
/* *
 * Allocates data for an encoding code block
 */
/* *
 * Allocates data memory for an encoding code block.
 */
unsafe extern "C" fn opj_tcd_code_block_enc_allocate_data(
  mut p_code_block: *mut opj_tcd_cblk_enc_t,
) -> OPJ_BOOL {
  let mut l_data_size: OPJ_UINT32 = 0;
  /* +1 is needed for https://github.com/uclouvain/openjpeg/issues/835 */
  /* and actually +2 required for https://github.com/uclouvain/openjpeg/issues/982 */
  /* and +7 for https://github.com/uclouvain/openjpeg/issues/1283 (-M 3) */
  /* and +26 for https://github.com/uclouvain/openjpeg/issues/1283 (-M 7) */
  /* and +28 for https://github.com/uclouvain/openjpeg/issues/1283 (-M 44) */
  /* and +33 for https://github.com/uclouvain/openjpeg/issues/1283 (-M 4) */
  /* and +63 for https://github.com/uclouvain/openjpeg/issues/1283 (-M 4 -IMF 2K) */
  /* and +74 for https://github.com/uclouvain/openjpeg/issues/1283 (-M 4 -n 8 -s 7,7 -I) */
  /* TODO: is there a theoretical upper-bound for the compressed code */
  /* block size ? */
  l_data_size = (74 as libc::c_int as libc::c_uint).wrapping_add(
    (((*p_code_block).x1 - (*p_code_block).x0)
      * ((*p_code_block).y1 - (*p_code_block).y0)
      * ::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong as OPJ_INT32) as OPJ_UINT32,
  );
  if l_data_size > (*p_code_block).data_size {
    if !(*p_code_block).data.is_null() {
      /* We refer to data - 1 since below we incremented it */
      opj_free((*p_code_block).data.offset(-(1 as libc::c_int as isize)) as *mut libc::c_void);
    }
    (*p_code_block).data =
      opj_malloc(l_data_size.wrapping_add(1 as libc::c_int as libc::c_uint) as size_t)
        as *mut OPJ_BYTE;
    if (*p_code_block).data.is_null() {
      (*p_code_block).data_size = 0 as libc::c_uint;
      return 0 as libc::c_int;
    }
    (*p_code_block).data_size = l_data_size;
    /*why +1 ?*/
    *(*p_code_block).data.offset(0 as libc::c_int as isize) = 0 as libc::c_int as OPJ_BYTE;
    (*p_code_block).data = (*p_code_block).data.offset(1 as libc::c_int as isize)
  }
  return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_reinit_segment(mut seg: *mut opj_tcd_seg_t) {
  memset(
    seg as *mut libc::c_void,
    0 as libc::c_int,
    ::std::mem::size_of::<opj_tcd_seg_t>() as libc::c_ulong,
  );
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
unsafe extern "C" fn opj_tcd_code_block_dec_allocate(
  mut p_code_block: *mut opj_tcd_cblk_dec_t,
) -> OPJ_BOOL {
  if (*p_code_block).segs.is_null() {
    (*p_code_block).segs = opj_calloc(
      10 as libc::c_int as size_t,
      ::std::mem::size_of::<opj_tcd_seg_t>() as libc::c_ulong,
    ) as *mut opj_tcd_seg_t;
    if (*p_code_block).segs.is_null() {
      return 0 as libc::c_int;
    }
    /*fprintf(stderr, "m_current_max_segs of code_block->data = %d\n", p_code_block->m_current_max_segs);*/
    (*p_code_block).m_current_max_segs = 10 as libc::c_int as OPJ_UINT32
  } else {
    /*fprintf(stderr, "Allocate %d elements of code_block->data\n", OPJ_J2K_DEFAULT_NB_SEGS * sizeof(opj_tcd_seg_t));*/
    /* sanitize */
    let mut l_segs = (*p_code_block).segs; /*(/ 8)*/
    let mut l_current_max_segs = (*p_code_block).m_current_max_segs; /* (%8) */
    let mut l_chunks = (*p_code_block).chunks;
    let mut l_numchunksalloc = (*p_code_block).numchunksalloc;
    let mut i: OPJ_UINT32 = 0;
    opj_aligned_free((*p_code_block).decoded_data as *mut libc::c_void);
    (*p_code_block).decoded_data = 0 as *mut OPJ_INT32;
    memset(
      p_code_block as *mut libc::c_void,
      0 as libc::c_int,
      ::std::mem::size_of::<opj_tcd_cblk_dec_t>() as libc::c_ulong,
    );
    (*p_code_block).segs = l_segs;
    (*p_code_block).m_current_max_segs = l_current_max_segs;
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < l_current_max_segs {
      opj_tcd_reinit_segment(&mut *l_segs.offset(i as isize));
      i = i.wrapping_add(1)
    }
    (*p_code_block).chunks = l_chunks;
    (*p_code_block).numchunksalloc = l_numchunksalloc
  }
  return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_get_decoded_tile_size(
  mut p_tcd: *mut opj_tcd_t,
  mut take_into_account_partial_decoding: OPJ_BOOL,
) -> OPJ_UINT32 {
  let mut i: OPJ_UINT32 = 0;
  let mut l_data_size = 0 as libc::c_int as OPJ_UINT32;
  let mut l_img_comp = 0 as *mut opj_image_comp_t;
  let mut l_tile_comp = 0 as *mut opj_tcd_tilecomp_t;
  let mut l_res = 0 as *mut opj_tcd_resolution_t;
  let mut l_size_comp: OPJ_UINT32 = 0;
  let mut l_remaining: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_UINT32 = 0;
  l_tile_comp = (*(*(*p_tcd).tcd_image).tiles).comps;
  l_img_comp = (*(*p_tcd).image).comps;
  i = 0 as libc::c_int as OPJ_UINT32;
  while i < (*(*p_tcd).image).numcomps {
    let mut w: OPJ_UINT32 = 0;
    let mut h: OPJ_UINT32 = 0;
    l_size_comp = (*l_img_comp).prec >> 3 as libc::c_int;
    l_remaining = (*l_img_comp).prec & 7 as libc::c_int as libc::c_uint;
    if l_remaining != 0 {
      l_size_comp = l_size_comp.wrapping_add(1)
    }
    if l_size_comp == 3 as libc::c_int as libc::c_uint {
      l_size_comp = 4 as libc::c_int as OPJ_UINT32
    }
    l_res = (*l_tile_comp)
      .resolutions
      .offset((*l_tile_comp).minimum_num_resolutions as isize)
      .offset(-(1 as libc::c_int as isize));
    if take_into_account_partial_decoding != 0 && (*p_tcd).whole_tile_decoding == 0 {
      w = (*l_res).win_x1.wrapping_sub((*l_res).win_x0);
      h = (*l_res).win_y1.wrapping_sub((*l_res).win_y0)
    } else {
      w = ((*l_res).x1 - (*l_res).x0) as OPJ_UINT32;
      h = ((*l_res).y1 - (*l_res).y0) as OPJ_UINT32
    }
    if h > 0 as libc::c_int as libc::c_uint
      && (2147483647 as libc::c_int as libc::c_uint)
        .wrapping_mul(2 as libc::c_uint)
        .wrapping_add(1 as libc::c_uint)
        .wrapping_div(w)
        < h
    {
      return (2147483647 as libc::c_int as libc::c_uint)
        .wrapping_mul(2 as libc::c_uint)
        .wrapping_add(1 as libc::c_uint);
    }
    l_temp = w.wrapping_mul(h);
    if l_size_comp != 0
      && (2147483647 as libc::c_int as libc::c_uint)
        .wrapping_mul(2 as libc::c_uint)
        .wrapping_add(1 as libc::c_uint)
        .wrapping_div(l_size_comp)
        < l_temp
    {
      return (2147483647 as libc::c_int as libc::c_uint)
        .wrapping_mul(2 as libc::c_uint)
        .wrapping_add(1 as libc::c_uint);
    }
    l_temp = (l_temp as libc::c_uint).wrapping_mul(l_size_comp) as OPJ_UINT32 as OPJ_UINT32;
    if l_temp
      > (2147483647 as libc::c_int as libc::c_uint)
        .wrapping_mul(2 as libc::c_uint)
        .wrapping_add(1 as libc::c_uint)
        .wrapping_sub(l_data_size)
    {
      return (2147483647 as libc::c_int as libc::c_uint)
        .wrapping_mul(2 as libc::c_uint)
        .wrapping_add(1 as libc::c_uint);
    }
    l_data_size = (l_data_size as libc::c_uint).wrapping_add(l_temp) as OPJ_UINT32 as OPJ_UINT32;
    l_img_comp = l_img_comp.offset(1);
    l_tile_comp = l_tile_comp.offset(1);
    i = i.wrapping_add(1)
  }
  return l_data_size;
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_encode_tile(
  mut p_tcd: *mut opj_tcd_t,
  mut p_tile_no: OPJ_UINT32,
  mut p_dest: *mut OPJ_BYTE,
  mut p_data_written: *mut OPJ_UINT32,
  mut p_max_length: OPJ_UINT32,
  mut p_cstr_info: *mut opj_codestream_info_t,
  mut p_marker_info: *mut opj_tcd_marker_info_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  if (*p_tcd).cur_tp_num == 0 as libc::c_int as libc::c_uint {
    (*p_tcd).tcd_tileno = p_tile_no;
    (*p_tcd).tcp = &mut *(*(*p_tcd).cp).tcps.offset(p_tile_no as isize) as *mut opj_tcp_t;
    /* FIXME _ProfStop(PGROUP_RATE); */
    if !p_cstr_info.is_null() {
      let mut l_num_packs = 0 as libc::c_int as OPJ_UINT32;
      let mut i: OPJ_UINT32 = 0;
      /* INDEX >> "Precinct_nb_X et Precinct_nb_Y" */
      let mut l_tilec_idx: *mut opj_tcd_tilecomp_t = &mut *(*(*(*p_tcd).tcd_image).tiles)
        .comps
        .offset(0 as libc::c_int as isize)
        as *mut opj_tcd_tilecomp_t; /* based on component 0 */
      let mut l_tccp = (*(*p_tcd).tcp).tccps; /* based on component 0 */
      i = 0 as libc::c_int as OPJ_UINT32;
      while i < (*l_tilec_idx).numresolutions {
        let mut l_res_idx: *mut opj_tcd_resolution_t =
          &mut *(*l_tilec_idx).resolutions.offset(i as isize) as *mut opj_tcd_resolution_t;
        (*(*p_cstr_info).tile.offset(p_tile_no as isize)).pw[i as usize] =
          (*l_res_idx).pw as libc::c_int;
        (*(*p_cstr_info).tile.offset(p_tile_no as isize)).ph[i as usize] =
          (*l_res_idx).ph as libc::c_int;
        l_num_packs = (l_num_packs as libc::c_uint)
          .wrapping_add((*l_res_idx).pw.wrapping_mul((*l_res_idx).ph))
          as OPJ_UINT32 as OPJ_UINT32;
        (*(*p_cstr_info).tile.offset(p_tile_no as isize)).pdx[i as usize] =
          (*l_tccp).prcw[i as usize] as libc::c_int;
        (*(*p_cstr_info).tile.offset(p_tile_no as isize)).pdy[i as usize] =
          (*l_tccp).prch[i as usize] as libc::c_int;
        i = i.wrapping_add(1)
      }
      let ref mut fresh0 = (*(*p_cstr_info).tile.offset(p_tile_no as isize)).packet;
      *fresh0 = opj_calloc(
        ((*p_cstr_info).numcomps as OPJ_SIZE_T)
          .wrapping_mul((*p_cstr_info).numlayers as OPJ_SIZE_T)
          .wrapping_mul(l_num_packs as libc::c_ulong),
        ::std::mem::size_of::<opj_packet_info_t>() as libc::c_ulong,
      ) as *mut opj_packet_info_t;
      if (*(*p_cstr_info).tile.offset(p_tile_no as isize))
        .packet
        .is_null()
      {
        /* FIXME event manager error callback */
        return 0 as libc::c_int;
      }
    }
    if opj_tcd_dc_level_shift_encode(p_tcd) == 0 {
      return 0 as libc::c_int;
    }
    if opj_tcd_mct_encode(p_tcd) == 0 {
      return 0 as libc::c_int;
    }
    if opj_tcd_dwt_encode(p_tcd) == 0 {
      return 0 as libc::c_int;
    }
    if opj_tcd_t1_encode(p_tcd) == 0 {
      return 0 as libc::c_int;
    }
    if opj_tcd_rate_allocate_encode(p_tcd, p_dest, p_max_length, p_cstr_info, p_manager) == 0 {
      return 0 as libc::c_int;
    }
  }
  /* << INDEX */
  /* FIXME _ProfStart(PGROUP_DC_SHIFT); */
  /*---------------TILE-------------------*/
  /* FIXME _ProfStop(PGROUP_DC_SHIFT); */
  /* FIXME _ProfStart(PGROUP_MCT); */
  /* FIXME _ProfStop(PGROUP_MCT); */
  /* FIXME _ProfStart(PGROUP_DWT); */
  /* FIXME  _ProfStop(PGROUP_DWT); */
  /* FIXME  _ProfStart(PGROUP_T1); */
  /* FIXME _ProfStop(PGROUP_T1); */
  /* FIXME _ProfStart(PGROUP_RATE); */
  /*--------------TIER2------------------*/
  /* INDEX */
  if !p_cstr_info.is_null() {
    (*p_cstr_info).index_write = 1 as libc::c_int
  }
  /* FIXME _ProfStart(PGROUP_T2); */
  if opj_tcd_t2_encode(
    p_tcd,
    p_dest,
    p_data_written,
    p_max_length,
    p_cstr_info,
    p_marker_info,
    p_manager,
  ) == 0
  {
    return 0 as libc::c_int;
  }
  /* FIXME _ProfStop(PGROUP_T2); */
  /*---------------CLEAN-------------------*/
  return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_decode_tile(
  mut p_tcd: *mut opj_tcd_t,
  mut win_x0: OPJ_UINT32,
  mut win_y0: OPJ_UINT32,
  mut win_x1: OPJ_UINT32,
  mut win_y1: OPJ_UINT32,
  mut numcomps_to_decode: OPJ_UINT32,
  mut comps_indices: *const OPJ_UINT32,
  mut p_src: *mut OPJ_BYTE,
  mut p_max_length: OPJ_UINT32,
  mut p_tile_no: OPJ_UINT32,
  mut p_cstr_index: *mut opj_codestream_index_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_data_read: OPJ_UINT32 = 0;
  let mut compno: OPJ_UINT32 = 0;
  (*p_tcd).tcd_tileno = p_tile_no;
  (*p_tcd).tcp = &mut *(*(*p_tcd).cp).tcps.offset(p_tile_no as isize) as *mut opj_tcp_t;
  (*p_tcd).win_x0 = win_x0;
  (*p_tcd).win_y0 = win_y0;
  (*p_tcd).win_x1 = win_x1;
  (*p_tcd).win_y1 = win_y1;
  (*p_tcd).whole_tile_decoding = 1 as libc::c_int;
  opj_free((*p_tcd).used_component as *mut libc::c_void);
  (*p_tcd).used_component = 0 as *mut OPJ_BOOL;
  if numcomps_to_decode != 0 {
    let mut used_component = opj_calloc(
      ::std::mem::size_of::<OPJ_BOOL>() as libc::c_ulong,
      (*(*p_tcd).image).numcomps as size_t,
    ) as *mut OPJ_BOOL;
    if used_component.is_null() {
      return 0 as libc::c_int;
    }
    compno = 0 as libc::c_int as OPJ_UINT32;
    while compno < numcomps_to_decode {
      *used_component.offset(*comps_indices.offset(compno as isize) as isize) = 1 as libc::c_int;
      compno = compno.wrapping_add(1)
    }
    (*p_tcd).used_component = used_component
  }
  compno = 0 as libc::c_int as OPJ_UINT32;
  while compno < (*(*p_tcd).image).numcomps {
    if !(!(*p_tcd).used_component.is_null()
      && *(*p_tcd).used_component.offset(compno as isize) == 0)
    {
      if opj_tcd_is_whole_tilecomp_decoding(p_tcd, compno) == 0 {
        (*p_tcd).whole_tile_decoding = 0 as libc::c_int;
        break;
      }
    }
    compno = compno.wrapping_add(1)
  }
  if (*p_tcd).whole_tile_decoding != 0 {
    compno = 0 as libc::c_int as OPJ_UINT32;
    while compno < (*(*p_tcd).image).numcomps {
      let mut tilec: *mut opj_tcd_tilecomp_t =
        &mut *(*(*(*p_tcd).tcd_image).tiles).comps.offset(compno as isize)
          as *mut opj_tcd_tilecomp_t;
      let mut l_res: *mut opj_tcd_resolution_t = &mut *(*tilec).resolutions.offset(
        (*tilec)
          .minimum_num_resolutions
          .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
      ) as *mut opj_tcd_resolution_t;
      let mut l_data_size: OPJ_SIZE_T = 0;
      /* compute l_data_size with overflow check */
      let mut res_w = ((*l_res).x1 - (*l_res).x0) as OPJ_SIZE_T;
      let mut res_h = ((*l_res).y1 - (*l_res).y0) as OPJ_SIZE_T;
      if !(!(*p_tcd).used_component.is_null()
        && *(*p_tcd).used_component.offset(compno as isize) == 0)
      {
        /* issue 733, l_data_size == 0U, probably something wrong should be checked before getting here */
        if res_h > 0 as libc::c_int as libc::c_ulong
          && res_w > (18446744073709551615 as libc::c_ulong).wrapping_div(res_h)
        {
          opj_event_msg(
            p_manager,
            1 as libc::c_int,
            b"Size of tile data exceeds system limits\n\x00" as *const u8 as *const libc::c_char,
          );
          return 0 as libc::c_int;
        }
        l_data_size = res_w.wrapping_mul(res_h);
        if (18446744073709551615 as libc::c_ulong)
          .wrapping_div(::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong)
          < l_data_size
        {
          opj_event_msg(
            p_manager,
            1 as libc::c_int,
            b"Size of tile data exceeds system limits\n\x00" as *const u8 as *const libc::c_char,
          );
          return 0 as libc::c_int;
        }
        l_data_size = (l_data_size as libc::c_ulong)
          .wrapping_mul(::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong)
          as OPJ_SIZE_T as OPJ_SIZE_T;
        (*tilec).data_size_needed = l_data_size;
        if opj_alloc_tile_component_data(tilec) == 0 {
          opj_event_msg(
            p_manager,
            1 as libc::c_int,
            b"Size of tile data exceeds system limits\n\x00" as *const u8 as *const libc::c_char,
          );
          return 0 as libc::c_int;
        }
      }
      compno = compno.wrapping_add(1)
    }
  } else {
    /* Compute restricted tile-component and tile-resolution coordinates */
    /* of the window of interest, but defer the memory allocation until */
    /* we know the resno_decoded */
    compno = 0 as libc::c_int as OPJ_UINT32;
    while compno < (*(*p_tcd).image).numcomps {
      let mut resno: OPJ_UINT32 = 0;
      let mut tilec_0: *mut opj_tcd_tilecomp_t =
        &mut *(*(*(*p_tcd).tcd_image).tiles).comps.offset(compno as isize)
          as *mut opj_tcd_tilecomp_t;
      let mut image_comp: *mut opj_image_comp_t =
        &mut *(*(*p_tcd).image).comps.offset(compno as isize) as *mut opj_image_comp_t;
      if !(!(*p_tcd).used_component.is_null()
        && *(*p_tcd).used_component.offset(compno as isize) == 0)
      {
        /* Compute the intersection of the area of interest, expressed in tile coordinates */
        /* with the tile coordinates */
        (*tilec_0).win_x0 = opj_uint_max(
          (*tilec_0).x0 as OPJ_UINT32,
          opj_uint_ceildiv((*p_tcd).win_x0, (*image_comp).dx),
        );
        (*tilec_0).win_y0 = opj_uint_max(
          (*tilec_0).y0 as OPJ_UINT32,
          opj_uint_ceildiv((*p_tcd).win_y0, (*image_comp).dy),
        );
        (*tilec_0).win_x1 = opj_uint_min(
          (*tilec_0).x1 as OPJ_UINT32,
          opj_uint_ceildiv((*p_tcd).win_x1, (*image_comp).dx),
        );
        (*tilec_0).win_y1 = opj_uint_min(
          (*tilec_0).y1 as OPJ_UINT32,
          opj_uint_ceildiv((*p_tcd).win_y1, (*image_comp).dy),
        );
        if (*tilec_0).win_x1 < (*tilec_0).win_x0 || (*tilec_0).win_y1 < (*tilec_0).win_y0 {
          /* We should not normally go there. The circumstance is when */
          /* the tile coordinates do not intersect the area of interest */
          /* Upper level logic should not even try to decode that tile */
          opj_event_msg(
            p_manager,
            1 as libc::c_int,
            b"Invalid tilec->win_xxx values\n\x00" as *const u8 as *const libc::c_char,
          );
          return 0 as libc::c_int;
        }
        resno = 0 as libc::c_int as OPJ_UINT32;
        while resno < (*tilec_0).numresolutions {
          let mut res = (*tilec_0).resolutions.offset(resno as isize);
          (*res).win_x0 = opj_uint_ceildivpow2(
            (*tilec_0).win_x0,
            (*tilec_0)
              .numresolutions
              .wrapping_sub(1 as libc::c_int as libc::c_uint)
              .wrapping_sub(resno),
          );
          (*res).win_y0 = opj_uint_ceildivpow2(
            (*tilec_0).win_y0,
            (*tilec_0)
              .numresolutions
              .wrapping_sub(1 as libc::c_int as libc::c_uint)
              .wrapping_sub(resno),
          );
          (*res).win_x1 = opj_uint_ceildivpow2(
            (*tilec_0).win_x1,
            (*tilec_0)
              .numresolutions
              .wrapping_sub(1 as libc::c_int as libc::c_uint)
              .wrapping_sub(resno),
          );
          (*res).win_y1 = opj_uint_ceildivpow2(
            (*tilec_0).win_y1,
            (*tilec_0)
              .numresolutions
              .wrapping_sub(1 as libc::c_int as libc::c_uint)
              .wrapping_sub(resno),
          );
          resno = resno.wrapping_add(1)
        }
      }
      compno = compno.wrapping_add(1)
    }
  }
  /* FIXME */
  /*--------------TIER2------------------*/
  /* FIXME _ProfStart(PGROUP_T2); */
  l_data_read = 0 as libc::c_int as OPJ_UINT32;
  if opj_tcd_t2_decode(
    p_tcd,
    p_src,
    &mut l_data_read,
    p_max_length,
    p_cstr_index,
    p_manager,
  ) == 0
  {
    return 0 as libc::c_int;
  }
  /* FIXME _ProfStop(PGROUP_T2); */
  /*------------------TIER1-----------------*/
  /* FIXME _ProfStart(PGROUP_T1); */
  if opj_tcd_t1_decode(p_tcd, p_manager) == 0 {
    return 0 as libc::c_int;
  }
  /* FIXME _ProfStop(PGROUP_T1); */
  /* For subtile decoding, now we know the resno_decoded, we can allocate */
  /* the tile data buffer */
  if (*p_tcd).whole_tile_decoding == 0 {
    compno = 0 as libc::c_int as OPJ_UINT32;
    while compno < (*(*p_tcd).image).numcomps {
      let mut tilec_1: *mut opj_tcd_tilecomp_t =
        &mut *(*(*(*p_tcd).tcd_image).tiles).comps.offset(compno as isize)
          as *mut opj_tcd_tilecomp_t;
      let mut image_comp_0: *mut opj_image_comp_t =
        &mut *(*(*p_tcd).image).comps.offset(compno as isize) as *mut opj_image_comp_t;
      let mut res_0 = (*tilec_1)
        .resolutions
        .offset((*image_comp_0).resno_decoded as isize);
      let mut w = (*res_0).win_x1.wrapping_sub((*res_0).win_x0) as OPJ_SIZE_T;
      let mut h = (*res_0).win_y1.wrapping_sub((*res_0).win_y0) as OPJ_SIZE_T;
      let mut l_data_size_0: OPJ_SIZE_T = 0;
      opj_image_data_free((*tilec_1).data_win as *mut libc::c_void);
      (*tilec_1).data_win = 0 as *mut OPJ_INT32;
      if !(!(*p_tcd).used_component.is_null()
        && *(*p_tcd).used_component.offset(compno as isize) == 0)
      {
        if w > 0 as libc::c_int as libc::c_ulong && h > 0 as libc::c_int as libc::c_ulong {
          if w > (18446744073709551615 as libc::c_ulong).wrapping_div(h) {
            opj_event_msg(
              p_manager,
              1 as libc::c_int,
              b"Size of tile data exceeds system limits\n\x00" as *const u8 as *const libc::c_char,
            );
            return 0 as libc::c_int;
          }
          l_data_size_0 = w.wrapping_mul(h);
          if l_data_size_0
            > (18446744073709551615 as libc::c_ulong)
              .wrapping_div(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong)
          {
            opj_event_msg(
              p_manager,
              1 as libc::c_int,
              b"Size of tile data exceeds system limits\n\x00" as *const u8 as *const libc::c_char,
            );
            return 0 as libc::c_int;
          }
          l_data_size_0 = (l_data_size_0 as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong)
            as OPJ_SIZE_T as OPJ_SIZE_T;
          (*tilec_1).data_win = opj_image_data_alloc(l_data_size_0) as *mut OPJ_INT32;
          if (*tilec_1).data_win.is_null() {
            opj_event_msg(
              p_manager,
              1 as libc::c_int,
              b"Size of tile data exceeds system limits\n\x00" as *const u8 as *const libc::c_char,
            );
            return 0 as libc::c_int;
          }
        }
      }
      compno = compno.wrapping_add(1)
    }
  }
  /*----------------DWT---------------------*/
  /* FIXME _ProfStart(PGROUP_DWT); */
  if opj_tcd_dwt_decode(p_tcd) == 0 {
    return 0 as libc::c_int;
  }
  /* FIXME _ProfStop(PGROUP_DWT); */
  /*----------------MCT-------------------*/
  /* FIXME _ProfStart(PGROUP_MCT); */
  if opj_tcd_mct_decode(p_tcd, p_manager) == 0 {
    return 0 as libc::c_int;
  }
  /* FIXME _ProfStop(PGROUP_MCT); */
  /* FIXME _ProfStart(PGROUP_DC_SHIFT); */
  if opj_tcd_dc_level_shift_decode(p_tcd) == 0 {
    return 0 as libc::c_int;
  }
  /* FIXME _ProfStop(PGROUP_DC_SHIFT); */
  /*---------------TILE-------------------*/
  return 1 as libc::c_int; /*(/ 8)*/
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_update_tile_data(
  mut p_tcd: *mut opj_tcd_t,
  mut p_dest: *mut OPJ_BYTE,
  mut p_dest_length: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0; /* (%8) */
  let mut j: OPJ_UINT32 = 0;
  let mut k: OPJ_UINT32 = 0;
  let mut l_data_size = 0 as libc::c_int as OPJ_UINT32;
  let mut l_img_comp = 0 as *mut opj_image_comp_t;
  let mut l_tilec = 0 as *mut opj_tcd_tilecomp_t;
  let mut l_res = 0 as *mut opj_tcd_resolution_t;
  let mut l_size_comp: OPJ_UINT32 = 0;
  let mut l_remaining: OPJ_UINT32 = 0;
  let mut l_stride: OPJ_UINT32 = 0;
  let mut l_width: OPJ_UINT32 = 0;
  let mut l_height: OPJ_UINT32 = 0;
  l_data_size = opj_tcd_get_decoded_tile_size(p_tcd, 1 as libc::c_int);
  if l_data_size
    == (2147483647 as libc::c_int as libc::c_uint)
      .wrapping_mul(2 as libc::c_uint)
      .wrapping_add(1 as libc::c_uint)
    || l_data_size > p_dest_length
  {
    return 0 as libc::c_int;
  }
  l_tilec = (*(*(*p_tcd).tcd_image).tiles).comps;
  l_img_comp = (*(*p_tcd).image).comps;
  i = 0 as libc::c_int as OPJ_UINT32;
  while i < (*(*p_tcd).image).numcomps {
    let mut l_src_data = 0 as *const OPJ_INT32;
    l_size_comp = (*l_img_comp).prec >> 3 as libc::c_int;
    l_remaining = (*l_img_comp).prec & 7 as libc::c_int as libc::c_uint;
    l_res = (*l_tilec)
      .resolutions
      .offset((*l_img_comp).resno_decoded as isize);
    if (*p_tcd).whole_tile_decoding != 0 {
      l_width = ((*l_res).x1 - (*l_res).x0) as OPJ_UINT32;
      l_height = ((*l_res).y1 - (*l_res).y0) as OPJ_UINT32;
      l_stride = (((*(*l_tilec).resolutions.offset(
        (*l_tilec)
          .minimum_num_resolutions
          .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
      ))
      .x1
        - (*(*l_tilec).resolutions.offset(
          (*l_tilec)
            .minimum_num_resolutions
            .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
        ))
        .x0) as OPJ_UINT32)
        .wrapping_sub(l_width);
      l_src_data = (*l_tilec).data
    } else {
      l_width = (*l_res).win_x1.wrapping_sub((*l_res).win_x0);
      l_height = (*l_res).win_y1.wrapping_sub((*l_res).win_y0);
      l_stride = 0 as libc::c_int as OPJ_UINT32;
      l_src_data = (*l_tilec).data_win
    }
    if l_remaining != 0 {
      l_size_comp = l_size_comp.wrapping_add(1)
    }
    if l_size_comp == 3 as libc::c_int as libc::c_uint {
      l_size_comp = 4 as libc::c_int as OPJ_UINT32
    }
    match l_size_comp {
      1 => {
        let mut l_dest_ptr = p_dest as *mut OPJ_CHAR;
        let mut l_src_ptr = l_src_data;
        if (*l_img_comp).sgnd != 0 {
          j = 0 as libc::c_int as OPJ_UINT32;
          while j < l_height {
            k = 0 as libc::c_int as OPJ_UINT32;
            while k < l_width {
              let fresh1 = l_src_ptr;
              l_src_ptr = l_src_ptr.offset(1);
              let fresh2 = l_dest_ptr;
              l_dest_ptr = l_dest_ptr.offset(1);
              *fresh2 = *fresh1 as OPJ_CHAR;
              k = k.wrapping_add(1)
            }
            l_src_ptr = l_src_ptr.offset(l_stride as isize);
            j = j.wrapping_add(1)
          }
        } else {
          j = 0 as libc::c_int as OPJ_UINT32;
          while j < l_height {
            k = 0 as libc::c_int as OPJ_UINT32;
            while k < l_width {
              let fresh3 = l_src_ptr;
              l_src_ptr = l_src_ptr.offset(1);
              let fresh4 = l_dest_ptr;
              l_dest_ptr = l_dest_ptr.offset(1);
              *fresh4 = (*fresh3 & 0xff as libc::c_int) as OPJ_CHAR;
              k = k.wrapping_add(1)
            }
            l_src_ptr = l_src_ptr.offset(l_stride as isize);
            j = j.wrapping_add(1)
          }
        }
        p_dest = l_dest_ptr as *mut OPJ_BYTE
      }
      2 => {
        let mut l_src_ptr_0 = l_src_data;
        let mut l_dest_ptr_0 = p_dest as *mut OPJ_INT16;
        if (*l_img_comp).sgnd != 0 {
          j = 0 as libc::c_int as OPJ_UINT32;
          while j < l_height {
            k = 0 as libc::c_int as OPJ_UINT32;
            while k < l_width {
              let fresh5 = l_src_ptr_0;
              l_src_ptr_0 = l_src_ptr_0.offset(1);
              let mut val = *fresh5 as OPJ_INT16;
              memcpy(
                l_dest_ptr_0 as *mut libc::c_void,
                &mut val as *mut OPJ_INT16 as *const libc::c_void,
                ::std::mem::size_of::<OPJ_INT16>() as libc::c_ulong,
              );
              l_dest_ptr_0 = l_dest_ptr_0.offset(1);
              k = k.wrapping_add(1)
            }
            l_src_ptr_0 = l_src_ptr_0.offset(l_stride as isize);
            j = j.wrapping_add(1)
          }
        } else {
          j = 0 as libc::c_int as OPJ_UINT32;
          while j < l_height {
            k = 0 as libc::c_int as OPJ_UINT32;
            while k < l_width {
              let fresh6 = l_src_ptr_0;
              l_src_ptr_0 = l_src_ptr_0.offset(1);
              let mut val_0 = (*fresh6 & 0xffff as libc::c_int) as OPJ_INT16;
              memcpy(
                l_dest_ptr_0 as *mut libc::c_void,
                &mut val_0 as *mut OPJ_INT16 as *const libc::c_void,
                ::std::mem::size_of::<OPJ_INT16>() as libc::c_ulong,
              );
              l_dest_ptr_0 = l_dest_ptr_0.offset(1);
              k = k.wrapping_add(1)
            }
            l_src_ptr_0 = l_src_ptr_0.offset(l_stride as isize);
            j = j.wrapping_add(1)
          }
        }
        p_dest = l_dest_ptr_0 as *mut OPJ_BYTE
      }
      4 => {
        let mut l_dest_ptr_1 = p_dest as *mut OPJ_INT32;
        let mut l_src_ptr_1 = l_src_data;
        j = 0 as libc::c_int as OPJ_UINT32;
        while j < l_height {
          memcpy(
            l_dest_ptr_1 as *mut libc::c_void,
            l_src_ptr_1 as *const libc::c_void,
            (l_width as libc::c_ulong)
              .wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
          );
          l_dest_ptr_1 = l_dest_ptr_1.offset(l_width as isize);
          l_src_ptr_1 = l_src_ptr_1.offset(l_width.wrapping_add(l_stride) as isize);
          j = j.wrapping_add(1)
        }
        p_dest = l_dest_ptr_1 as *mut OPJ_BYTE
      }
      _ => {}
    }
    l_img_comp = l_img_comp.offset(1);
    l_tilec = l_tilec.offset(1);
    i = i.wrapping_add(1)
  }
  return 1 as libc::c_int;
}
/* *
Free the memory allocated for encoding
@param tcd TCD handle
*/
unsafe extern "C" fn opj_tcd_free_tile(mut p_tcd: *mut opj_tcd_t) {
  let mut compno: OPJ_UINT32 = 0; /* for (resno */
  let mut resno: OPJ_UINT32 = 0;
  let mut bandno: OPJ_UINT32 = 0;
  let mut precno: OPJ_UINT32 = 0;
  let mut l_tile = 0 as *mut opj_tcd_tile_t;
  let mut l_tile_comp = 0 as *mut opj_tcd_tilecomp_t;
  let mut l_res = 0 as *mut opj_tcd_resolution_t;
  let mut l_band = 0 as *mut opj_tcd_band_t;
  let mut l_precinct = 0 as *mut opj_tcd_precinct_t;
  let mut l_nb_resolutions: OPJ_UINT32 = 0;
  let mut l_nb_precincts: OPJ_UINT32 = 0;
  let mut l_tcd_code_block_deallocate: Option<
    unsafe extern "C" fn(_: *mut opj_tcd_precinct_t) -> (),
  > = None;
  if p_tcd.is_null() {
    return;
  }
  if (*p_tcd).tcd_image.is_null() {
    return;
  }
  if (*p_tcd).m_is_decoder() != 0 {
    l_tcd_code_block_deallocate = Some(
      opj_tcd_code_block_dec_deallocate as unsafe extern "C" fn(_: *mut opj_tcd_precinct_t) -> (),
    )
  } else {
    l_tcd_code_block_deallocate = Some(
      opj_tcd_code_block_enc_deallocate as unsafe extern "C" fn(_: *mut opj_tcd_precinct_t) -> (),
    )
  }
  l_tile = (*(*p_tcd).tcd_image).tiles;
  if l_tile.is_null() {
    return;
  }
  l_tile_comp = (*l_tile).comps;
  compno = 0 as libc::c_int as OPJ_UINT32;
  while compno < (*l_tile).numcomps {
    l_res = (*l_tile_comp).resolutions;
    if !l_res.is_null() {
      l_nb_resolutions = (*l_tile_comp)
        .resolutions_size
        .wrapping_div(::std::mem::size_of::<opj_tcd_resolution_t>() as libc::c_ulong as OPJ_UINT32);
      resno = 0 as libc::c_int as OPJ_UINT32;
      while resno < l_nb_resolutions {
        l_band = (*l_res).bands.as_mut_ptr();
        bandno = 0 as libc::c_int as OPJ_UINT32;
        while bandno < 3 as libc::c_int as libc::c_uint {
          l_precinct = (*l_band).precincts;
          if !l_precinct.is_null() {
            l_nb_precincts =
              (*l_band).precincts_data_size.wrapping_div(
                ::std::mem::size_of::<opj_tcd_precinct_t>() as libc::c_ulong as OPJ_UINT32,
              );
            precno = 0 as libc::c_int as OPJ_UINT32;
            while precno < l_nb_precincts {
              opj_tgt_destroy((*l_precinct).incltree);
              (*l_precinct).incltree = 0 as *mut opj_tgt_tree_t;
              opj_tgt_destroy((*l_precinct).imsbtree);
              (*l_precinct).imsbtree = 0 as *mut opj_tgt_tree_t;
              Some(l_tcd_code_block_deallocate.expect("non-null function pointer"))
                .expect("non-null function pointer")(l_precinct);
              l_precinct = l_precinct.offset(1);
              precno = precno.wrapping_add(1)
            }
            opj_free((*l_band).precincts as *mut libc::c_void);
            (*l_band).precincts = 0 as *mut opj_tcd_precinct_t
          }
          l_band = l_band.offset(1);
          bandno = bandno.wrapping_add(1)
        }
        l_res = l_res.offset(1);
        resno = resno.wrapping_add(1)
      }
      opj_free((*l_tile_comp).resolutions as *mut libc::c_void);
      (*l_tile_comp).resolutions = 0 as *mut opj_tcd_resolution_t
    }
    if (*l_tile_comp).ownsData != 0 && !(*l_tile_comp).data.is_null() {
      opj_image_data_free((*l_tile_comp).data as *mut libc::c_void);
      (*l_tile_comp).data = 0 as *mut OPJ_INT32;
      (*l_tile_comp).ownsData = 0 as libc::c_int;
      (*l_tile_comp).data_size = 0 as libc::c_int as size_t;
      (*l_tile_comp).data_size_needed = 0 as libc::c_int as size_t
    }
    opj_image_data_free((*l_tile_comp).data_win as *mut libc::c_void);
    l_tile_comp = l_tile_comp.offset(1);
    compno = compno.wrapping_add(1)
  }
  opj_free((*l_tile).comps as *mut libc::c_void);
  (*l_tile).comps = 0 as *mut opj_tcd_tilecomp_t;
  opj_free((*(*p_tcd).tcd_image).tiles as *mut libc::c_void);
  (*(*p_tcd).tcd_image).tiles = 0 as *mut opj_tcd_tile_t;
}
unsafe extern "C" fn opj_tcd_t2_decode(
  mut p_tcd: *mut opj_tcd_t,
  mut p_src_data: *mut OPJ_BYTE,
  mut p_data_read: *mut OPJ_UINT32,
  mut p_max_src_size: OPJ_UINT32,
  mut p_cstr_index: *mut opj_codestream_index_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_t2 = 0 as *mut opj_t2_t;
  l_t2 = opj_t2_create((*p_tcd).image, (*p_tcd).cp);
  if l_t2.is_null() {
    return 0 as libc::c_int;
  }
  if opj_t2_decode_packets(
    p_tcd,
    l_t2,
    (*p_tcd).tcd_tileno,
    (*(*p_tcd).tcd_image).tiles,
    p_src_data,
    p_data_read,
    p_max_src_size,
    p_cstr_index,
    p_manager,
  ) == 0
  {
    opj_t2_destroy(l_t2);
    return 0 as libc::c_int;
  }
  opj_t2_destroy(l_t2);
  /*---------------CLEAN-------------------*/
  return 1 as libc::c_int;
}
unsafe extern "C" fn opj_tcd_t1_decode(
  mut p_tcd: *mut opj_tcd_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut compno: OPJ_UINT32 = 0;
  let mut l_tile = (*(*p_tcd).tcd_image).tiles;
  let mut l_tile_comp = (*l_tile).comps;
  let mut l_tccp = (*(*p_tcd).tcp).tccps;
  let mut ret = 1 as libc::c_int;
  let mut check_pterm = 0 as libc::c_int;
  let mut p_manager_mutex = 0 as *mut opj_mutex_t;
  p_manager_mutex = opj_mutex_create();
  /* Only enable PTERM check if we decode all layers */
  if (*(*p_tcd).tcp).num_layers_to_decode == (*(*p_tcd).tcp).numlayers
    && (*l_tccp).cblksty & 0x10 as libc::c_int as libc::c_uint != 0 as libc::c_int as libc::c_uint
  {
    check_pterm = 1 as libc::c_int
  }
  compno = 0 as libc::c_int as OPJ_UINT32;
  while compno < (*l_tile).numcomps {
    if !(!(*p_tcd).used_component.is_null()
      && *(*p_tcd).used_component.offset(compno as isize) == 0)
    {
      opj_t1_decode_cblks(
        p_tcd,
        &mut ret,
        l_tile_comp,
        l_tccp,
        p_manager,
        p_manager_mutex,
        check_pterm,
      );
      if ret == 0 {
        break;
      }
    }
    compno = compno.wrapping_add(1);
    l_tile_comp = l_tile_comp.offset(1);
    l_tccp = l_tccp.offset(1)
  }
  opj_thread_pool_wait_completion((*p_tcd).thread_pool, 0 as libc::c_int);
  if !p_manager_mutex.is_null() {
    opj_mutex_destroy(p_manager_mutex);
  }
  return ret;
}
unsafe extern "C" fn opj_tcd_dwt_decode(mut p_tcd: *mut opj_tcd_t) -> OPJ_BOOL {
  let mut compno: OPJ_UINT32 = 0;
  let mut l_tile = (*(*p_tcd).tcd_image).tiles;
  let mut l_tile_comp = (*l_tile).comps;
  let mut l_tccp = (*(*p_tcd).tcp).tccps;
  let mut l_img_comp = (*(*p_tcd).image).comps;
  compno = 0 as libc::c_int as OPJ_UINT32;
  while compno < (*l_tile).numcomps {
    if !(!(*p_tcd).used_component.is_null()
      && *(*p_tcd).used_component.offset(compno as isize) == 0)
    {
      if (*l_tccp).qmfbid == 1 as libc::c_int as libc::c_uint {
        if opj_dwt_decode(
          p_tcd,
          l_tile_comp,
          (*l_img_comp)
            .resno_decoded
            .wrapping_add(1 as libc::c_int as libc::c_uint),
        ) == 0
        {
          return 0 as libc::c_int;
        }
      } else if opj_dwt_decode_real(
        p_tcd,
        l_tile_comp,
        (*l_img_comp)
          .resno_decoded
          .wrapping_add(1 as libc::c_int as libc::c_uint),
      ) == 0
      {
        return 0 as libc::c_int;
      }
    }
    compno = compno.wrapping_add(1);
    l_tile_comp = l_tile_comp.offset(1);
    l_img_comp = l_img_comp.offset(1);
    l_tccp = l_tccp.offset(1)
  }
  return 1 as libc::c_int;
}
unsafe extern "C" fn opj_tcd_mct_decode(
  mut p_tcd: *mut opj_tcd_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_tile = (*(*p_tcd).tcd_image).tiles;
  let mut l_tcp = (*p_tcd).tcp;
  let mut l_tile_comp = (*l_tile).comps;
  let mut l_samples: OPJ_SIZE_T = 0;
  let mut i: OPJ_UINT32 = 0;
  if (*l_tcp).mct == 0 as libc::c_int as libc::c_uint || !(*p_tcd).used_component.is_null() {
    return 1 as libc::c_int;
  }
  if (*p_tcd).whole_tile_decoding != 0 {
    let mut res_comp0 = (*(*l_tile).comps.offset(0 as libc::c_int as isize))
      .resolutions
      .offset((*l_tile_comp).minimum_num_resolutions as isize)
      .offset(-(1 as libc::c_int as isize));
    /* A bit inefficient: we process more data than needed if */
    /* resno_decoded < l_tile_comp->minimum_num_resolutions-1, */
    /* but we would need to take into account a stride then */
    l_samples = (((*res_comp0).x1 - (*res_comp0).x0) as OPJ_SIZE_T)
      .wrapping_mul(((*res_comp0).y1 - (*res_comp0).y0) as OPJ_SIZE_T);
    if (*l_tile).numcomps >= 3 as libc::c_int as libc::c_uint {
      if (*l_tile_comp).minimum_num_resolutions
        != (*(*l_tile).comps.offset(1 as libc::c_int as isize)).minimum_num_resolutions
        || (*l_tile_comp).minimum_num_resolutions
          != (*(*l_tile).comps.offset(2 as libc::c_int as isize)).minimum_num_resolutions
      {
        opj_event_msg(
          p_manager,
          1 as libc::c_int,
          b"Tiles don\'t all have the same dimension. Skip the MCT step.\n\x00" as *const u8
            as *const libc::c_char,
        );
        return 0 as libc::c_int;
      }
    }
    if (*l_tile).numcomps >= 3 as libc::c_int as libc::c_uint {
      let mut res_comp1 = (*(*l_tile).comps.offset(1 as libc::c_int as isize))
        .resolutions
        .offset((*l_tile_comp).minimum_num_resolutions as isize)
        .offset(-(1 as libc::c_int as isize));
      let mut res_comp2 = (*(*l_tile).comps.offset(2 as libc::c_int as isize))
        .resolutions
        .offset((*l_tile_comp).minimum_num_resolutions as isize)
        .offset(-(1 as libc::c_int as isize));
      /* testcase 1336.pdf.asan.47.376 */
      if (*(*(*p_tcd).image).comps.offset(0 as libc::c_int as isize)).resno_decoded
        != (*(*(*p_tcd).image).comps.offset(1 as libc::c_int as isize)).resno_decoded
        || (*(*(*p_tcd).image).comps.offset(0 as libc::c_int as isize)).resno_decoded
          != (*(*(*p_tcd).image).comps.offset(2 as libc::c_int as isize)).resno_decoded
        || (((*res_comp1).x1 - (*res_comp1).x0) as OPJ_SIZE_T)
          .wrapping_mul(((*res_comp1).y1 - (*res_comp1).y0) as OPJ_SIZE_T)
          != l_samples
        || (((*res_comp2).x1 - (*res_comp2).x0) as OPJ_SIZE_T)
          .wrapping_mul(((*res_comp2).y1 - (*res_comp2).y0) as OPJ_SIZE_T)
          != l_samples
      {
        opj_event_msg(
          p_manager,
          1 as libc::c_int,
          b"Tiles don\'t all have the same dimension. Skip the MCT step.\n\x00" as *const u8
            as *const libc::c_char,
        );
        return 0 as libc::c_int;
      }
    }
  } else {
    let mut res_comp0_0 = (*(*l_tile).comps.offset(0 as libc::c_int as isize))
      .resolutions
      .offset((*(*(*p_tcd).image).comps.offset(0 as libc::c_int as isize)).resno_decoded as isize);
    l_samples = ((*res_comp0_0).win_x1.wrapping_sub((*res_comp0_0).win_x0) as OPJ_SIZE_T)
      .wrapping_mul((*res_comp0_0).win_y1.wrapping_sub((*res_comp0_0).win_y0) as OPJ_SIZE_T);
    if (*l_tile).numcomps >= 3 as libc::c_int as libc::c_uint {
      let mut res_comp1_0 = (*(*l_tile).comps.offset(1 as libc::c_int as isize))
        .resolutions
        .offset(
          (*(*(*p_tcd).image).comps.offset(1 as libc::c_int as isize)).resno_decoded as isize,
        );
      let mut res_comp2_0 = (*(*l_tile).comps.offset(2 as libc::c_int as isize))
        .resolutions
        .offset(
          (*(*(*p_tcd).image).comps.offset(2 as libc::c_int as isize)).resno_decoded as isize,
        );
      /* testcase 1336.pdf.asan.47.376 */
      if (*(*(*p_tcd).image).comps.offset(0 as libc::c_int as isize)).resno_decoded
        != (*(*(*p_tcd).image).comps.offset(1 as libc::c_int as isize)).resno_decoded
        || (*(*(*p_tcd).image).comps.offset(0 as libc::c_int as isize)).resno_decoded
          != (*(*(*p_tcd).image).comps.offset(2 as libc::c_int as isize)).resno_decoded
        || ((*res_comp1_0).win_x1.wrapping_sub((*res_comp1_0).win_x0) as OPJ_SIZE_T)
          .wrapping_mul((*res_comp1_0).win_y1.wrapping_sub((*res_comp1_0).win_y0) as OPJ_SIZE_T)
          != l_samples
        || ((*res_comp2_0).win_x1.wrapping_sub((*res_comp2_0).win_x0) as OPJ_SIZE_T)
          .wrapping_mul((*res_comp2_0).win_y1.wrapping_sub((*res_comp2_0).win_y0) as OPJ_SIZE_T)
          != l_samples
      {
        opj_event_msg(
          p_manager,
          1 as libc::c_int,
          b"Tiles don\'t all have the same dimension. Skip the MCT step.\n\x00" as *const u8
            as *const libc::c_char,
        );
        return 0 as libc::c_int;
      }
    }
  }
  if (*l_tile).numcomps >= 3 as libc::c_int as libc::c_uint {
    if (*l_tcp).mct == 2 as libc::c_int as libc::c_uint {
      let mut l_data = 0 as *mut *mut OPJ_BYTE;
      if (*l_tcp).m_mct_decoding_matrix.is_null() {
        return 1 as libc::c_int;
      }
      l_data = opj_malloc(
        ((*l_tile).numcomps as libc::c_ulong)
          .wrapping_mul(::std::mem::size_of::<*mut OPJ_BYTE>() as libc::c_ulong),
      ) as *mut *mut OPJ_BYTE;
      if l_data.is_null() {
        return 0 as libc::c_int;
      }
      i = 0 as libc::c_int as OPJ_UINT32;
      while i < (*l_tile).numcomps {
        if (*p_tcd).whole_tile_decoding != 0 {
          let ref mut fresh7 = *l_data.offset(i as isize);
          *fresh7 = (*l_tile_comp).data as *mut OPJ_BYTE
        } else {
          let ref mut fresh8 = *l_data.offset(i as isize);
          *fresh8 = (*l_tile_comp).data_win as *mut OPJ_BYTE
        }
        l_tile_comp = l_tile_comp.offset(1);
        i = i.wrapping_add(1)
      }
      if opj_mct_decode_custom(
        (*l_tcp).m_mct_decoding_matrix as *mut OPJ_BYTE,
        l_samples,
        l_data,
        (*l_tile).numcomps,
        (*(*(*p_tcd).image).comps).sgnd,
      ) == 0
      {
        opj_free(l_data as *mut libc::c_void);
        return 0 as libc::c_int;
      }
      opj_free(l_data as *mut libc::c_void);
    } else if (*(*l_tcp).tccps).qmfbid == 1 as libc::c_int as libc::c_uint {
      if (*p_tcd).whole_tile_decoding != 0 {
        opj_mct_decode(
          (*(*l_tile).comps.offset(0 as libc::c_int as isize)).data,
          (*(*l_tile).comps.offset(1 as libc::c_int as isize)).data,
          (*(*l_tile).comps.offset(2 as libc::c_int as isize)).data,
          l_samples,
        );
      } else {
        opj_mct_decode(
          (*(*l_tile).comps.offset(0 as libc::c_int as isize)).data_win,
          (*(*l_tile).comps.offset(1 as libc::c_int as isize)).data_win,
          (*(*l_tile).comps.offset(2 as libc::c_int as isize)).data_win,
          l_samples,
        );
      }
    } else if (*p_tcd).whole_tile_decoding != 0 {
      opj_mct_decode_real(
        (*(*l_tile).comps.offset(0 as libc::c_int as isize)).data as *mut OPJ_FLOAT32,
        (*(*l_tile).comps.offset(1 as libc::c_int as isize)).data as *mut OPJ_FLOAT32,
        (*(*l_tile).comps.offset(2 as libc::c_int as isize)).data as *mut OPJ_FLOAT32,
        l_samples,
      );
    } else {
      opj_mct_decode_real(
        (*(*l_tile).comps.offset(0 as libc::c_int as isize)).data_win as *mut OPJ_FLOAT32,
        (*(*l_tile).comps.offset(1 as libc::c_int as isize)).data_win as *mut OPJ_FLOAT32,
        (*(*l_tile).comps.offset(2 as libc::c_int as isize)).data_win as *mut OPJ_FLOAT32,
        l_samples,
      );
    }
  } else {
    opj_event_msg(
      p_manager,
      1 as libc::c_int,
      b"Number of components (%d) is inconsistent with a MCT. Skip the MCT step.\n\x00" as *const u8
        as *const libc::c_char,
      (*l_tile).numcomps,
    );
  }
  return 1 as libc::c_int;
}
unsafe extern "C" fn opj_tcd_dc_level_shift_decode(mut p_tcd: *mut opj_tcd_t) -> OPJ_BOOL {
  let mut compno: OPJ_UINT32 = 0;
  let mut l_tile_comp = 0 as *mut opj_tcd_tilecomp_t;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  let mut l_img_comp = 0 as *mut opj_image_comp_t;
  let mut l_res = 0 as *mut opj_tcd_resolution_t;
  let mut l_tile = 0 as *mut opj_tcd_tile_t;
  let mut l_width: OPJ_UINT32 = 0;
  let mut l_height: OPJ_UINT32 = 0;
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut l_current_ptr = 0 as *mut OPJ_INT32;
  let mut l_min: OPJ_INT32 = 0;
  let mut l_max: OPJ_INT32 = 0;
  let mut l_stride: OPJ_UINT32 = 0;
  l_tile = (*(*p_tcd).tcd_image).tiles;
  l_tile_comp = (*l_tile).comps;
  l_tccp = (*(*p_tcd).tcp).tccps;
  l_img_comp = (*(*p_tcd).image).comps;
  compno = 0 as libc::c_int as OPJ_UINT32;
  while compno < (*l_tile).numcomps {
    if !(!(*p_tcd).used_component.is_null()
      && *(*p_tcd).used_component.offset(compno as isize) == 0)
    {
      l_res = (*l_tile_comp)
        .resolutions
        .offset((*l_img_comp).resno_decoded as isize);
      if (*p_tcd).whole_tile_decoding == 0 {
        l_width = (*l_res).win_x1.wrapping_sub((*l_res).win_x0);
        l_height = (*l_res).win_y1.wrapping_sub((*l_res).win_y0);
        l_stride = 0 as libc::c_int as OPJ_UINT32;
        l_current_ptr = (*l_tile_comp).data_win
      } else {
        l_width = ((*l_res).x1 - (*l_res).x0) as OPJ_UINT32;
        l_height = ((*l_res).y1 - (*l_res).y0) as OPJ_UINT32;
        l_stride = (((*(*l_tile_comp).resolutions.offset(
          (*l_tile_comp)
            .minimum_num_resolutions
            .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
        ))
        .x1
          - (*(*l_tile_comp).resolutions.offset(
            (*l_tile_comp)
              .minimum_num_resolutions
              .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
          ))
          .x0) as OPJ_UINT32)
          .wrapping_sub(l_width);
        l_current_ptr = (*l_tile_comp).data;
        assert!(
          l_height == 0 as libc::c_int as libc::c_uint
            || l_width.wrapping_add(l_stride) as libc::c_ulong
              <= (*l_tile_comp)
                .data_size
                .wrapping_div(l_height as libc::c_ulong)
        );
        /*MUPDF*/
      }
      if (*l_img_comp).sgnd != 0 {
        l_min = -((1 as libc::c_int)
          << (*l_img_comp)
            .prec
            .wrapping_sub(1 as libc::c_int as libc::c_uint));
        l_max = ((1 as libc::c_int)
          << (*l_img_comp)
            .prec
            .wrapping_sub(1 as libc::c_int as libc::c_uint))
          - 1 as libc::c_int
      } else {
        l_min = 0 as libc::c_int;
        l_max = ((1 as libc::c_uint) << (*l_img_comp).prec)
          .wrapping_sub(1 as libc::c_int as libc::c_uint) as OPJ_INT32
      }
      if (*l_tccp).qmfbid == 1 as libc::c_int as libc::c_uint {
        j = 0 as libc::c_int as OPJ_UINT32;
        while j < l_height {
          i = 0 as libc::c_int as OPJ_UINT32;
          while i < l_width {
            /* TODO: do addition on int64 ? */
            *l_current_ptr =
              opj_int_clamp(*l_current_ptr + (*l_tccp).m_dc_level_shift, l_min, l_max);
            l_current_ptr = l_current_ptr.offset(1);
            i = i.wrapping_add(1)
          }
          l_current_ptr = l_current_ptr.offset(l_stride as isize);
          j = j.wrapping_add(1)
        }
      } else {
        j = 0 as libc::c_int as OPJ_UINT32;
        while j < l_height {
          i = 0 as libc::c_int as OPJ_UINT32;
          while i < l_width {
            let mut l_value = *(l_current_ptr as *mut OPJ_FLOAT32);
            if l_value > 2147483647 as libc::c_int as libc::c_float {
              *l_current_ptr = l_max
            } else if l_value < (-(2147483647 as libc::c_int) - 1 as libc::c_int) as libc::c_float {
              *l_current_ptr = l_min
            } else {
              /* Do addition on int64 to avoid overflows */
              let mut l_value_int = opj_lrintf(l_value);
              *l_current_ptr = opj_int64_clamp(
                l_value_int + (*l_tccp).m_dc_level_shift as libc::c_long,
                l_min as OPJ_INT64,
                l_max as OPJ_INT64,
              ) as OPJ_INT32
            }
            l_current_ptr = l_current_ptr.offset(1);
            i = i.wrapping_add(1)
          }
          l_current_ptr = l_current_ptr.offset(l_stride as isize);
          j = j.wrapping_add(1)
        }
      }
    }
    compno = compno.wrapping_add(1);
    l_img_comp = l_img_comp.offset(1);
    l_tccp = l_tccp.offset(1);
    l_tile_comp = l_tile_comp.offset(1)
  }
  return 1 as libc::c_int;
}
/* *
 * Deallocates the decoding data of the given precinct.
 */
/* *
 * Deallocates the encoding data of the given precinct.
 */
unsafe extern "C" fn opj_tcd_code_block_dec_deallocate(mut p_precinct: *mut opj_tcd_precinct_t) {
  let mut cblkno: OPJ_UINT32 = 0;
  let mut l_nb_code_blocks: OPJ_UINT32 = 0;
  let mut l_code_block = (*p_precinct).cblks.dec;
  if !l_code_block.is_null() {
    /*fprintf(stderr,"deallocate codeblock:{\n");*/
    /*fprintf(stderr,"\t x0=%d, y0=%d, x1=%d, y1=%d\n",l_code_block->x0, l_code_block->y0, l_code_block->x1, l_code_block->y1);*/
    /*fprintf(stderr,"\t numbps=%d, numlenbits=%d, len=%d, numnewpasses=%d, real_num_segs=%d, m_current_max_segs=%d\n ",
    l_code_block->numbps, l_code_block->numlenbits, l_code_block->len, l_code_block->numnewpasses, l_code_block->real_num_segs, l_code_block->m_current_max_segs );*/
    l_nb_code_blocks = (*p_precinct)
      .block_size
      .wrapping_div(::std::mem::size_of::<opj_tcd_cblk_dec_t>() as libc::c_ulong as OPJ_UINT32);
    /*fprintf(stderr,"nb_code_blocks =%d\t}\n", l_nb_code_blocks);*/
    cblkno = 0 as libc::c_int as OPJ_UINT32;
    while cblkno < l_nb_code_blocks {
      if !(*l_code_block).segs.is_null() {
        opj_free((*l_code_block).segs as *mut libc::c_void);
        (*l_code_block).segs = 0 as *mut opj_tcd_seg_t
      }
      if !(*l_code_block).chunks.is_null() {
        opj_free((*l_code_block).chunks as *mut libc::c_void);
        (*l_code_block).chunks = 0 as *mut opj_tcd_seg_data_chunk_t
      }
      opj_aligned_free((*l_code_block).decoded_data as *mut libc::c_void);
      (*l_code_block).decoded_data = 0 as *mut OPJ_INT32;
      l_code_block = l_code_block.offset(1);
      cblkno = cblkno.wrapping_add(1)
    }
    opj_free((*p_precinct).cblks.dec as *mut libc::c_void);
    (*p_precinct).cblks.dec = 0 as *mut opj_tcd_cblk_dec_t
  };
}
/* *
 * Deallocates the encoding data of the given precinct.
 */
/* *
 * Deallocates the encoding data of the given precinct.
 */
unsafe extern "C" fn opj_tcd_code_block_enc_deallocate(mut p_precinct: *mut opj_tcd_precinct_t) {
  let mut cblkno: OPJ_UINT32 = 0;
  let mut l_nb_code_blocks: OPJ_UINT32 = 0;
  let mut l_code_block = (*p_precinct).cblks.enc;
  if !l_code_block.is_null() {
    l_nb_code_blocks = (*p_precinct)
      .block_size
      .wrapping_div(::std::mem::size_of::<opj_tcd_cblk_enc_t>() as libc::c_ulong as OPJ_UINT32);
    cblkno = 0 as libc::c_int as OPJ_UINT32;
    while cblkno < l_nb_code_blocks {
      if !(*l_code_block).data.is_null() {
        /* We refer to data - 1 since below we incremented it */
        /* in opj_tcd_code_block_enc_allocate_data() */
        opj_free((*l_code_block).data.offset(-(1 as libc::c_int as isize)) as *mut libc::c_void); /*(/ 8)*/
        (*l_code_block).data = 0 as *mut OPJ_BYTE
      } /* (%8) */
      if !(*l_code_block).layers.is_null() {
        opj_free((*l_code_block).layers as *mut libc::c_void);
        (*l_code_block).layers = 0 as *mut opj_tcd_layer_t
      }
      if !(*l_code_block).passes.is_null() {
        opj_free((*l_code_block).passes as *mut libc::c_void);
        (*l_code_block).passes = 0 as *mut opj_tcd_pass_t
      }
      l_code_block = l_code_block.offset(1);
      cblkno = cblkno.wrapping_add(1)
    }
    opj_free((*p_precinct).cblks.enc as *mut libc::c_void);
    (*p_precinct).cblks.enc = 0 as *mut opj_tcd_cblk_enc_t
  };
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_get_encoder_input_buffer_size(
  mut p_tcd: *mut opj_tcd_t,
) -> OPJ_SIZE_T {
  let mut i: OPJ_UINT32 = 0;
  let mut l_data_size = 0 as libc::c_int as OPJ_SIZE_T;
  let mut l_img_comp = 0 as *mut opj_image_comp_t;
  let mut l_tilec = 0 as *mut opj_tcd_tilecomp_t;
  let mut l_size_comp: OPJ_UINT32 = 0;
  let mut l_remaining: OPJ_UINT32 = 0;
  l_tilec = (*(*(*p_tcd).tcd_image).tiles).comps;
  l_img_comp = (*(*p_tcd).image).comps;
  i = 0 as libc::c_int as OPJ_UINT32;
  while i < (*(*p_tcd).image).numcomps {
    l_size_comp = (*l_img_comp).prec >> 3 as libc::c_int;
    l_remaining = (*l_img_comp).prec & 7 as libc::c_int as libc::c_uint;
    if l_remaining != 0 {
      l_size_comp = l_size_comp.wrapping_add(1)
    }
    if l_size_comp == 3 as libc::c_int as libc::c_uint {
      l_size_comp = 4 as libc::c_int as OPJ_UINT32
    }
    l_data_size = (l_data_size as libc::c_ulong).wrapping_add(
      (l_size_comp as libc::c_ulong).wrapping_mul(
        (((*l_tilec).x1 - (*l_tilec).x0) as OPJ_SIZE_T)
          .wrapping_mul(((*l_tilec).y1 - (*l_tilec).y0) as OPJ_SIZE_T),
      ),
    ) as OPJ_SIZE_T as OPJ_SIZE_T;
    l_img_comp = l_img_comp.offset(1);
    l_tilec = l_tilec.offset(1);
    i = i.wrapping_add(1)
  }
  return l_data_size;
}
unsafe extern "C" fn opj_tcd_dc_level_shift_encode(mut p_tcd: *mut opj_tcd_t) -> OPJ_BOOL {
  let mut compno: OPJ_UINT32 = 0;
  let mut l_tile_comp = 0 as *mut opj_tcd_tilecomp_t;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  let mut l_img_comp = 0 as *mut opj_image_comp_t;
  let mut l_tile = 0 as *mut opj_tcd_tile_t;
  let mut l_nb_elem: OPJ_SIZE_T = 0;
  let mut i: OPJ_SIZE_T = 0;
  let mut l_current_ptr = 0 as *mut OPJ_INT32;
  l_tile = (*(*p_tcd).tcd_image).tiles;
  l_tile_comp = (*l_tile).comps;
  l_tccp = (*(*p_tcd).tcp).tccps;
  l_img_comp = (*(*p_tcd).image).comps;
  compno = 0 as libc::c_int as OPJ_UINT32;
  while compno < (*l_tile).numcomps {
    l_current_ptr = (*l_tile_comp).data;
    l_nb_elem = (((*l_tile_comp).x1 - (*l_tile_comp).x0) as OPJ_SIZE_T)
      .wrapping_mul(((*l_tile_comp).y1 - (*l_tile_comp).y0) as OPJ_SIZE_T);
    if (*l_tccp).qmfbid == 1 as libc::c_int as libc::c_uint {
      i = 0 as libc::c_int as OPJ_SIZE_T;
      while i < l_nb_elem {
        *l_current_ptr -= (*l_tccp).m_dc_level_shift;
        l_current_ptr = l_current_ptr.offset(1);
        i = i.wrapping_add(1)
      }
    } else {
      i = 0 as libc::c_int as OPJ_SIZE_T;
      while i < l_nb_elem {
        *(l_current_ptr as *mut OPJ_FLOAT32) =
          (*l_current_ptr - (*l_tccp).m_dc_level_shift) as OPJ_FLOAT32;
        l_current_ptr = l_current_ptr.offset(1);
        i = i.wrapping_add(1)
      }
    }
    l_img_comp = l_img_comp.offset(1);
    l_tccp = l_tccp.offset(1);
    l_tile_comp = l_tile_comp.offset(1);
    compno = compno.wrapping_add(1)
  }
  return 1 as libc::c_int;
}
unsafe extern "C" fn opj_tcd_mct_encode(mut p_tcd: *mut opj_tcd_t) -> OPJ_BOOL {
  let mut l_tile = (*(*p_tcd).tcd_image).tiles;
  let mut l_tile_comp = (*(*(*p_tcd).tcd_image).tiles).comps;
  let mut samples = (((*l_tile_comp).x1 - (*l_tile_comp).x0) as OPJ_SIZE_T)
    .wrapping_mul(((*l_tile_comp).y1 - (*l_tile_comp).y0) as OPJ_SIZE_T);
  let mut i: OPJ_UINT32 = 0;
  let mut l_data = 0 as *mut *mut OPJ_BYTE;
  let mut l_tcp = (*p_tcd).tcp;
  if (*(*p_tcd).tcp).mct == 0 {
    return 1 as libc::c_int;
  }
  if (*(*p_tcd).tcp).mct == 2 as libc::c_int as libc::c_uint {
    if (*(*p_tcd).tcp).m_mct_coding_matrix.is_null() {
      return 1 as libc::c_int;
    }
    l_data = opj_malloc(
      ((*l_tile).numcomps as libc::c_ulong)
        .wrapping_mul(::std::mem::size_of::<*mut OPJ_BYTE>() as libc::c_ulong),
    ) as *mut *mut OPJ_BYTE;
    if l_data.is_null() {
      return 0 as libc::c_int;
    }
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < (*l_tile).numcomps {
      let ref mut fresh9 = *l_data.offset(i as isize);
      *fresh9 = (*l_tile_comp).data as *mut OPJ_BYTE;
      l_tile_comp = l_tile_comp.offset(1);
      i = i.wrapping_add(1)
    }
    if opj_mct_encode_custom(
      (*(*p_tcd).tcp).m_mct_coding_matrix as *mut OPJ_BYTE,
      samples,
      l_data,
      (*l_tile).numcomps,
      (*(*(*p_tcd).image).comps).sgnd,
    ) == 0
    {
      opj_free(l_data as *mut libc::c_void);
      return 0 as libc::c_int;
    }
    opj_free(l_data as *mut libc::c_void);
  } else if (*(*l_tcp).tccps).qmfbid == 0 as libc::c_int as libc::c_uint {
    opj_mct_encode_real(
      (*(*l_tile).comps.offset(0 as libc::c_int as isize)).data as *mut OPJ_FLOAT32,
      (*(*l_tile).comps.offset(1 as libc::c_int as isize)).data as *mut OPJ_FLOAT32,
      (*(*l_tile).comps.offset(2 as libc::c_int as isize)).data as *mut OPJ_FLOAT32,
      samples,
    );
  } else {
    opj_mct_encode(
      (*(*l_tile).comps.offset(0 as libc::c_int as isize)).data,
      (*(*l_tile).comps.offset(1 as libc::c_int as isize)).data,
      (*(*l_tile).comps.offset(2 as libc::c_int as isize)).data,
      samples,
    );
  }
  return 1 as libc::c_int;
}
unsafe extern "C" fn opj_tcd_dwt_encode(mut p_tcd: *mut opj_tcd_t) -> OPJ_BOOL {
  let mut l_tile = (*(*p_tcd).tcd_image).tiles;
  let mut l_tile_comp = (*(*(*p_tcd).tcd_image).tiles).comps;
  let mut l_tccp = (*(*p_tcd).tcp).tccps;
  let mut compno: OPJ_UINT32 = 0;
  compno = 0 as libc::c_int as OPJ_UINT32;
  while compno < (*l_tile).numcomps {
    if (*l_tccp).qmfbid == 1 as libc::c_int as libc::c_uint {
      if opj_dwt_encode(p_tcd, l_tile_comp) == 0 {
        return 0 as libc::c_int;
      }
    } else if (*l_tccp).qmfbid == 0 as libc::c_int as libc::c_uint {
      if opj_dwt_encode_real(p_tcd, l_tile_comp) == 0 {
        return 0 as libc::c_int;
      }
    }
    l_tile_comp = l_tile_comp.offset(1);
    l_tccp = l_tccp.offset(1);
    compno = compno.wrapping_add(1)
  }
  return 1 as libc::c_int;
}
unsafe extern "C" fn opj_tcd_t1_encode(mut p_tcd: *mut opj_tcd_t) -> OPJ_BOOL {
  let mut l_mct_norms = 0 as *const OPJ_FLOAT64;
  let mut l_mct_numcomps = 0 as libc::c_uint;
  let mut l_tcp = (*p_tcd).tcp;
  if (*l_tcp).mct == 1 as libc::c_int as libc::c_uint {
    l_mct_numcomps = 3 as libc::c_uint;
    /* irreversible encoding */
    if (*(*l_tcp).tccps).qmfbid == 0 as libc::c_int as libc::c_uint {
      l_mct_norms = opj_mct_get_mct_norms_real()
    } else {
      l_mct_norms = opj_mct_get_mct_norms()
    }
  } else {
    l_mct_numcomps = (*(*p_tcd).image).numcomps;
    l_mct_norms = (*l_tcp).mct_norms as *const OPJ_FLOAT64
  }
  return opj_t1_encode_cblks(
    p_tcd,
    (*(*p_tcd).tcd_image).tiles,
    l_tcp,
    l_mct_norms,
    l_mct_numcomps,
  );
}
unsafe extern "C" fn opj_tcd_t2_encode(
  mut p_tcd: *mut opj_tcd_t,
  mut p_dest_data: *mut OPJ_BYTE,
  mut p_data_written: *mut OPJ_UINT32,
  mut p_max_dest_size: OPJ_UINT32,
  mut p_cstr_info: *mut opj_codestream_info_t,
  mut p_marker_info: *mut opj_tcd_marker_info_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_t2 = 0 as *mut opj_t2_t;
  l_t2 = opj_t2_create((*p_tcd).image, (*p_tcd).cp);
  if l_t2.is_null() {
    return 0 as libc::c_int;
  }
  if opj_t2_encode_packets(
    l_t2,
    (*p_tcd).tcd_tileno,
    (*(*p_tcd).tcd_image).tiles,
    (*(*p_tcd).tcp).numlayers,
    p_dest_data,
    p_data_written,
    p_max_dest_size,
    p_cstr_info,
    p_marker_info,
    (*p_tcd).tp_num,
    (*p_tcd).tp_pos,
    (*p_tcd).cur_pino,
    FINAL_PASS,
    p_manager,
  ) == 0
  {
    opj_t2_destroy(l_t2);
    return 0 as libc::c_int;
  }
  opj_t2_destroy(l_t2);
  /*---------------CLEAN-------------------*/
  return 1 as libc::c_int;
}
unsafe extern "C" fn opj_tcd_rate_allocate_encode(
  mut p_tcd: *mut opj_tcd_t,
  mut p_dest_data: *mut OPJ_BYTE,
  mut p_max_dest_size: OPJ_UINT32,
  mut p_cstr_info: *mut opj_codestream_info_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_cp = (*p_tcd).cp;
  let mut l_nb_written = 0 as libc::c_int as OPJ_UINT32;
  if !p_cstr_info.is_null() {
    (*p_cstr_info).index_write = 0 as libc::c_int
  }
  if (*l_cp).m_specific_param.m_enc.m_disto_alloc() as libc::c_int != 0
    || (*l_cp).m_specific_param.m_enc.m_fixed_quality() as libc::c_int != 0
  {
    /* fixed_quality */
    /* Normal Rate/distortion allocation */
    if opj_tcd_rateallocate(
      p_tcd,
      p_dest_data,
      &mut l_nb_written,
      p_max_dest_size,
      p_cstr_info,
      p_manager,
    ) == 0
    {
      return 0 as libc::c_int;
    }
  } else {
    /* Fixed layer allocation */
    opj_tcd_rateallocate_fixed(p_tcd); /*(/ 8)*/
  } /* (%8) */
  return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_copy_tile_data(
  mut p_tcd: *mut opj_tcd_t,
  mut p_src: *mut OPJ_BYTE,
  mut p_src_length: OPJ_SIZE_T,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_SIZE_T = 0;
  let mut l_data_size = 0 as libc::c_int as OPJ_SIZE_T;
  let mut l_img_comp = 0 as *mut opj_image_comp_t;
  let mut l_tilec = 0 as *mut opj_tcd_tilecomp_t;
  let mut l_size_comp: OPJ_UINT32 = 0;
  let mut l_remaining: OPJ_UINT32 = 0;
  let mut l_nb_elem: OPJ_SIZE_T = 0;
  l_data_size = opj_tcd_get_encoder_input_buffer_size(p_tcd);
  if l_data_size != p_src_length {
    return 0 as libc::c_int;
  }
  l_tilec = (*(*(*p_tcd).tcd_image).tiles).comps;
  l_img_comp = (*(*p_tcd).image).comps;
  i = 0 as libc::c_int as OPJ_UINT32;
  while i < (*(*p_tcd).image).numcomps {
    l_size_comp = (*l_img_comp).prec >> 3 as libc::c_int;
    l_remaining = (*l_img_comp).prec & 7 as libc::c_int as libc::c_uint;
    l_nb_elem = (((*l_tilec).x1 - (*l_tilec).x0) as OPJ_SIZE_T)
      .wrapping_mul(((*l_tilec).y1 - (*l_tilec).y0) as OPJ_SIZE_T);
    if l_remaining != 0 {
      l_size_comp = l_size_comp.wrapping_add(1)
    }
    if l_size_comp == 3 as libc::c_int as libc::c_uint {
      l_size_comp = 4 as libc::c_int as OPJ_UINT32
    }
    match l_size_comp {
      1 => {
        let mut l_src_ptr = p_src as *mut OPJ_CHAR;
        let mut l_dest_ptr = (*l_tilec).data;
        if (*l_img_comp).sgnd != 0 {
          j = 0 as libc::c_int as OPJ_SIZE_T;
          while j < l_nb_elem {
            let fresh10 = l_src_ptr;
            l_src_ptr = l_src_ptr.offset(1);
            let fresh11 = l_dest_ptr;
            l_dest_ptr = l_dest_ptr.offset(1);
            *fresh11 = *fresh10 as OPJ_INT32;
            j = j.wrapping_add(1)
          }
        } else {
          j = 0 as libc::c_int as OPJ_SIZE_T;
          while j < l_nb_elem {
            let fresh12 = l_src_ptr;
            l_src_ptr = l_src_ptr.offset(1);
            let fresh13 = l_dest_ptr;
            l_dest_ptr = l_dest_ptr.offset(1);
            *fresh13 = *fresh12 as libc::c_int & 0xff as libc::c_int;
            j = j.wrapping_add(1)
          }
        }
        p_src = l_src_ptr as *mut OPJ_BYTE
      }
      2 => {
        let mut l_dest_ptr_0 = (*l_tilec).data;
        let mut l_src_ptr_0 = p_src as *mut OPJ_INT16;
        if (*l_img_comp).sgnd != 0 {
          j = 0 as libc::c_int as OPJ_SIZE_T;
          while j < l_nb_elem {
            let fresh14 = l_src_ptr_0;
            l_src_ptr_0 = l_src_ptr_0.offset(1);
            let fresh15 = l_dest_ptr_0;
            l_dest_ptr_0 = l_dest_ptr_0.offset(1);
            *fresh15 = *fresh14 as OPJ_INT32;
            j = j.wrapping_add(1)
          }
        } else {
          j = 0 as libc::c_int as OPJ_SIZE_T;
          while j < l_nb_elem {
            let fresh16 = l_src_ptr_0;
            l_src_ptr_0 = l_src_ptr_0.offset(1);
            let fresh17 = l_dest_ptr_0;
            l_dest_ptr_0 = l_dest_ptr_0.offset(1);
            *fresh17 = *fresh16 as libc::c_int & 0xffff as libc::c_int;
            j = j.wrapping_add(1)
          }
        }
        p_src = l_src_ptr_0 as *mut OPJ_BYTE
      }
      4 => {
        let mut l_src_ptr_1 = p_src as *mut OPJ_INT32;
        let mut l_dest_ptr_1 = (*l_tilec).data;
        j = 0 as libc::c_int as OPJ_SIZE_T;
        while j < l_nb_elem {
          let fresh18 = l_src_ptr_1;
          l_src_ptr_1 = l_src_ptr_1.offset(1);
          let fresh19 = l_dest_ptr_1;
          l_dest_ptr_1 = l_dest_ptr_1.offset(1);
          *fresh19 = *fresh18;
          j = j.wrapping_add(1)
        }
        p_src = l_src_ptr_1 as *mut OPJ_BYTE
      }
      _ => {}
    }
    l_img_comp = l_img_comp.offset(1);
    l_tilec = l_tilec.offset(1);
    i = i.wrapping_add(1)
  }
  return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_is_band_empty(mut band: *mut opj_tcd_band_t) -> OPJ_BOOL {
  return ((*band).x1 - (*band).x0 == 0 as libc::c_int
    || (*band).y1 - (*band).y0 == 0 as libc::c_int) as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_is_subband_area_of_interest(
  mut tcd: *mut opj_tcd_t,
  mut compno: OPJ_UINT32,
  mut resno: OPJ_UINT32,
  mut bandno: OPJ_UINT32,
  mut band_x0: OPJ_UINT32,
  mut band_y0: OPJ_UINT32,
  mut band_x1: OPJ_UINT32,
  mut band_y1: OPJ_UINT32,
) -> OPJ_BOOL {
  /* Note: those values for filter_margin are in part the result of */
  /* experimentation. The value 2 for QMFBID=1 (5x3 filter) can be linked */
  /* to the maximum left/right extension given in tables F.2 and F.3 of the */
  /* standard. The value 3 for QMFBID=0 (9x7 filter) is more suspicious, */
  /* since F.2 and F.3 would lead to 4 instead, so the current 3 might be */
  /* needed to be bumped to 4, in case inconsistencies are found while */
  /* decoding parts of irreversible coded images. */
  /* See opj_dwt_decode_partial_53 and opj_dwt_decode_partial_97 as well */
  let mut filter_margin =
    if (*(*(*tcd).tcp).tccps.offset(compno as isize)).qmfbid == 1 as libc::c_int as libc::c_uint {
      2 as libc::c_int
    } else {
      3 as libc::c_int
    } as OPJ_UINT32;
  let mut tilec: *mut opj_tcd_tilecomp_t =
    &mut *(*(*(*tcd).tcd_image).tiles).comps.offset(compno as isize) as *mut opj_tcd_tilecomp_t;
  let mut image_comp: *mut opj_image_comp_t =
    &mut *(*(*tcd).image).comps.offset(compno as isize) as *mut opj_image_comp_t;
  /* Compute the intersection of the area of interest, expressed in tile coordinates */
  /* with the tile coordinates */
  let mut tcx0 = opj_uint_max(
    (*tilec).x0 as OPJ_UINT32,
    opj_uint_ceildiv((*tcd).win_x0, (*image_comp).dx),
  );
  let mut tcy0 = opj_uint_max(
    (*tilec).y0 as OPJ_UINT32,
    opj_uint_ceildiv((*tcd).win_y0, (*image_comp).dy),
  );
  let mut tcx1 = opj_uint_min(
    (*tilec).x1 as OPJ_UINT32,
    opj_uint_ceildiv((*tcd).win_x1, (*image_comp).dx),
  );
  let mut tcy1 = opj_uint_min(
    (*tilec).y1 as OPJ_UINT32,
    opj_uint_ceildiv((*tcd).win_y1, (*image_comp).dy),
  );
  /* Compute number of decomposition for this band. See table F-1 */
  let mut nb = if resno == 0 as libc::c_int as libc::c_uint {
    (*tilec)
      .numresolutions
      .wrapping_sub(1 as libc::c_int as libc::c_uint)
  } else {
    (*tilec).numresolutions.wrapping_sub(resno)
  };
  /* Map above tile-based coordinates to sub-band-based coordinates per */
  /* equation B-15 of the standard */
  let mut x0b = bandno & 1 as libc::c_int as libc::c_uint;
  let mut y0b = bandno >> 1 as libc::c_int;
  let mut tbx0 = if nb == 0 as libc::c_int as libc::c_uint {
    tcx0
  } else if tcx0
    <= ((1 as libc::c_uint) << nb.wrapping_sub(1 as libc::c_int as libc::c_uint)).wrapping_mul(x0b)
  {
    0 as libc::c_int as libc::c_uint
  } else {
    opj_uint_ceildivpow2(
      tcx0.wrapping_sub(
        ((1 as libc::c_uint) << nb.wrapping_sub(1 as libc::c_int as libc::c_uint))
          .wrapping_mul(x0b),
      ),
      nb,
    )
  };
  let mut tby0 = if nb == 0 as libc::c_int as libc::c_uint {
    tcy0
  } else if tcy0
    <= ((1 as libc::c_uint) << nb.wrapping_sub(1 as libc::c_int as libc::c_uint)).wrapping_mul(y0b)
  {
    0 as libc::c_int as libc::c_uint
  } else {
    opj_uint_ceildivpow2(
      tcy0.wrapping_sub(
        ((1 as libc::c_uint) << nb.wrapping_sub(1 as libc::c_int as libc::c_uint))
          .wrapping_mul(y0b),
      ),
      nb,
    )
  };
  let mut tbx1 = if nb == 0 as libc::c_int as libc::c_uint {
    tcx1
  } else if tcx1
    <= ((1 as libc::c_uint) << nb.wrapping_sub(1 as libc::c_int as libc::c_uint)).wrapping_mul(x0b)
  {
    0 as libc::c_int as libc::c_uint
  } else {
    opj_uint_ceildivpow2(
      tcx1.wrapping_sub(
        ((1 as libc::c_uint) << nb.wrapping_sub(1 as libc::c_int as libc::c_uint))
          .wrapping_mul(x0b),
      ),
      nb,
    )
  };
  let mut tby1 = if nb == 0 as libc::c_int as libc::c_uint {
    tcy1
  } else if tcy1
    <= ((1 as libc::c_uint) << nb.wrapping_sub(1 as libc::c_int as libc::c_uint)).wrapping_mul(y0b)
  {
    0 as libc::c_int as libc::c_uint
  } else {
    opj_uint_ceildivpow2(
      tcy1.wrapping_sub(
        ((1 as libc::c_uint) << nb.wrapping_sub(1 as libc::c_int as libc::c_uint))
          .wrapping_mul(y0b),
      ),
      nb,
    )
  };
  let mut intersects: OPJ_BOOL = 0;
  if tbx0 < filter_margin {
    tbx0 = 0 as libc::c_int as OPJ_UINT32
  } else {
    tbx0 = (tbx0 as libc::c_uint).wrapping_sub(filter_margin) as OPJ_UINT32 as OPJ_UINT32
  }
  if tby0 < filter_margin {
    tby0 = 0 as libc::c_int as OPJ_UINT32
  } else {
    tby0 = (tby0 as libc::c_uint).wrapping_sub(filter_margin) as OPJ_UINT32 as OPJ_UINT32
  }
  tbx1 = opj_uint_adds(tbx1, filter_margin);
  tby1 = opj_uint_adds(tby1, filter_margin);
  intersects =
    (band_x0 < tbx1 && band_y0 < tby1 && band_x1 > tbx0 && band_y1 > tby0) as libc::c_int;
  return intersects;
}
/* * Returns whether a tile componenent is fully decoded, taking into account
 * p_tcd->win_* members.
 *
 * @param p_tcd    TCD handle.
 * @param compno Component number
 * @return OPJ_TRUE whether the tile componenent is fully decoded
 */
unsafe extern "C" fn opj_tcd_is_whole_tilecomp_decoding(
  mut p_tcd: *mut opj_tcd_t,
  mut compno: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut tilec: *mut opj_tcd_tilecomp_t =
    &mut *(*(*(*p_tcd).tcd_image).tiles).comps.offset(compno as isize) as *mut opj_tcd_tilecomp_t;
  let mut image_comp: *mut opj_image_comp_t =
    &mut *(*(*p_tcd).image).comps.offset(compno as isize) as *mut opj_image_comp_t;
  /* Compute the intersection of the area of interest, expressed in tile coordinates */
  /* with the tile coordinates */
  let mut tcx0 = opj_uint_max(
    (*tilec).x0 as OPJ_UINT32,
    opj_uint_ceildiv((*p_tcd).win_x0, (*image_comp).dx),
  );
  let mut tcy0 = opj_uint_max(
    (*tilec).y0 as OPJ_UINT32,
    opj_uint_ceildiv((*p_tcd).win_y0, (*image_comp).dy),
  );
  let mut tcx1 = opj_uint_min(
    (*tilec).x1 as OPJ_UINT32,
    opj_uint_ceildiv((*p_tcd).win_x1, (*image_comp).dx),
  );
  let mut tcy1 = opj_uint_min(
    (*tilec).y1 as OPJ_UINT32,
    opj_uint_ceildiv((*p_tcd).win_y1, (*image_comp).dy),
  );
  let mut shift = (*tilec)
    .numresolutions
    .wrapping_sub((*tilec).minimum_num_resolutions);
  /* Tolerate small margin within the reduced resolution factor to consider if */
  /* the whole tile path must be taken */
  return (tcx0 >= (*tilec).x0 as OPJ_UINT32
    && tcy0 >= (*tilec).y0 as OPJ_UINT32
    && tcx1 <= (*tilec).x1 as OPJ_UINT32
    && tcy1 <= (*tilec).y1 as OPJ_UINT32
    && (shift >= 32 as libc::c_int as libc::c_uint
      || tcx0.wrapping_sub((*tilec).x0 as OPJ_UINT32) >> shift == 0 as libc::c_int as libc::c_uint
        && tcy0.wrapping_sub((*tilec).y0 as OPJ_UINT32) >> shift
          == 0 as libc::c_int as libc::c_uint
        && ((*tilec).x1 as OPJ_UINT32).wrapping_sub(tcx1) >> shift
          == 0 as libc::c_int as libc::c_uint
        && ((*tilec).y1 as OPJ_UINT32).wrapping_sub(tcy1) >> shift
          == 0 as libc::c_int as libc::c_uint)) as libc::c_int;
}
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_marker_info_create(
  mut need_PLT: OPJ_BOOL,
) -> *mut opj_tcd_marker_info_t {
  let mut l_tcd_marker_info = opj_calloc(
    1 as libc::c_int as size_t,
    ::std::mem::size_of::<opj_tcd_marker_info_t>() as libc::c_ulong,
  ) as *mut opj_tcd_marker_info_t;
  if l_tcd_marker_info.is_null() {
    return 0 as *mut opj_tcd_marker_info_t;
  }
  (*l_tcd_marker_info).need_PLT = need_PLT;
  return l_tcd_marker_info;
}
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe extern "C" fn opj_tcd_marker_info_destroy(
  mut p_tcd_marker_info: *mut opj_tcd_marker_info_t,
) {
  if !p_tcd_marker_info.is_null() {
    opj_free((*p_tcd_marker_info).p_packet_size as *mut libc::c_void);
    opj_free(p_tcd_marker_info as *mut libc::c_void);
  };
}
/* ----------------------------------------------------------------------- */
