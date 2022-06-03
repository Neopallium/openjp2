use super::math::*;
use super::mqc::*;
use super::openjpeg::*;
use super::t1_luts::*;
use super::thread::*;
use super::consts::*;
use ::libc;

extern "C" {

  fn abs(_: libc::c_int) -> libc::c_int;

  fn lrintf(_: libc::c_float) -> libc::c_long;

  fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;

  fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;

  fn opj_calloc(numOfElements: size_t, sizeOfElements: size_t) -> *mut libc::c_void;

  fn opj_aligned_malloc(size: size_t) -> *mut libc::c_void;

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

  fn opj_mutex_lock(mutex: *mut opj_mutex_t);

  fn opj_mutex_unlock(mutex: *mut opj_mutex_t);

  fn opj_mutex_destroy(mutex: *mut opj_mutex_t);

  fn opj_tls_get(tls: *mut opj_tls_t, key: libc::c_int) -> *mut libc::c_void;

  fn opj_tls_set(
    tls: *mut opj_tls_t,
    key: libc::c_int,
    value: *mut libc::c_void,
    free_func: opj_tls_free_func,
  ) -> OPJ_BOOL;

  fn opj_thread_pool_submit_job(
    tp: *mut opj_thread_pool_t,
    job_fn: opj_job_fn,
    user_data: *mut libc::c_void,
  ) -> OPJ_BOOL;

  fn opj_thread_pool_wait_completion(tp: *mut opj_thread_pool_t, max_remaining_jobs: libc::c_int);

  fn opj_thread_pool_get_thread_count(tp: *mut opj_thread_pool_t) -> libc::c_int;

  fn opj_mqc_byteout(mqc: &mut opj_mqc_t);

  fn opj_mqc_numbytes(mqc: &mut opj_mqc_t) -> OPJ_UINT32;

  fn opj_mqc_resetstates(mqc: &mut opj_mqc_t);

  fn opj_mqc_setstate(mqc: &mut opj_mqc_t, ctxno: OPJ_UINT32, msb: OPJ_UINT32, prob: OPJ_INT32);

  fn opj_mqc_init_enc(mqc: &mut opj_mqc_t, bp: *mut OPJ_BYTE);

  fn opj_mqc_flush(mqc: &mut opj_mqc_t);

  fn opj_mqc_bypass_init_enc(mqc: &mut opj_mqc_t);

  fn opj_mqc_bypass_get_extra_bytes(mqc: &mut opj_mqc_t, erterm: OPJ_BOOL) -> OPJ_UINT32;

  fn opj_mqc_bypass_flush_enc(mqc: &mut opj_mqc_t, erterm: OPJ_BOOL);

  fn opj_mqc_reset_enc(mqc: &mut opj_mqc_t);

  fn opj_mqc_restart_init_enc(mqc: &mut opj_mqc_t);

  fn opj_mqc_erterm_enc(mqc: &mut opj_mqc_t);

  fn opj_mqc_segmark_enc(mqc: &mut opj_mqc_t);

  fn opj_mqc_init_dec(
    mqc: &mut opj_mqc_t,
    bp: *mut OPJ_BYTE,
    len: OPJ_UINT32,
    extra_writable_bytes: OPJ_UINT32,
  );

  fn opj_mqc_raw_init_dec(
    mqc: &mut opj_mqc_t,
    bp: *mut OPJ_BYTE,
    len: OPJ_UINT32,
    extra_writable_bytes: OPJ_UINT32,
  );

  fn opq_mqc_finish_dec(mqc: &mut opj_mqc_t);

  fn opj_tcd_is_band_empty(band: *mut opj_tcd_band_t) -> OPJ_BOOL;

  fn opj_tcd_is_subband_area_of_interest(
    tcd: *mut opj_tcd_t,
    compno: OPJ_UINT32,
    resno: OPJ_UINT32,
    bandno: OPJ_UINT32,
    x0: OPJ_UINT32,
    y0: OPJ_UINT32,
    x1: OPJ_UINT32,
    y1: OPJ_UINT32,
  ) -> OPJ_BOOL;

  fn opj_dwt_getnorm_real(level: OPJ_UINT32, orient: OPJ_UINT32) -> OPJ_FLOAT64;

  fn opj_dwt_getnorm(level: OPJ_UINT32, orient: OPJ_UINT32) -> OPJ_FLOAT64;
  /* *
  Decode 1 HT code-block
  @param t1 T1 handle
  @param cblk Code-block coding parameters
  @param orient
  @param roishift Region of interest shifting value
  @param cblksty Code-block style
  @param p_manager the event manager
  @param p_manager_mutex mutex for the event manager
  @param check_pterm whether PTERM correct termination should be checked
  */

  fn opj_t1_ht_decode_cblk(
    t1: *mut opj_t1_t,
    cblk: *mut opj_tcd_cblk_dec_t,
    orient: OPJ_UINT32,
    roishift: OPJ_UINT32,
    cblksty: OPJ_UINT32,
    p_manager: *mut opj_event_mgr_t,
    p_manager_mutex: *mut opj_mutex_t,
    check_pterm: OPJ_BOOL,
  ) -> OPJ_BOOL;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_t1 {
  pub mqc: opj_mqc_t,
  pub data: *mut OPJ_INT32,
  pub flags: *mut opj_flag_t,
  pub w: OPJ_UINT32,
  pub h: OPJ_UINT32,
  pub datasize: OPJ_UINT32,
  pub flagssize: OPJ_UINT32,
  pub encoder: OPJ_BOOL,
  pub mustuse_cblkdatabuffer: OPJ_BOOL,
  pub cblkdatabuffer: *mut OPJ_BYTE,
  pub cblkdatabuffersize: OPJ_UINT32,
}
pub type opj_t1_t = opj_t1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_t1_cblk_encode_processing_job_t {
  pub compno: OPJ_UINT32,
  pub resno: OPJ_UINT32,
  pub cblk: *mut opj_tcd_cblk_enc_t,
  pub tile: *mut opj_tcd_tile_t,
  pub band: *mut opj_tcd_band_t,
  pub tilec: *mut opj_tcd_tilecomp_t,
  pub tccp: *mut opj_tccp_t,
  pub mct_norms: *const OPJ_FLOAT64,
  pub mct_numcomps: OPJ_UINT32,
  pub pret: *mut OPJ_BOOL,
  pub mutex: *mut opj_mutex_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_t1_cblk_decode_processing_job_t {
  pub whole_tile_decoding: OPJ_BOOL,
  pub resno: OPJ_UINT32,
  pub cblk: *mut opj_tcd_cblk_dec_t,
  pub band: *mut opj_tcd_band_t,
  pub tilec: *mut opj_tcd_tilecomp_t,
  pub tccp: *mut opj_tccp_t,
  pub mustuse_cblkdatabuffer: OPJ_BOOL,
  pub pret: *mut OPJ_BOOL,
  pub p_manager: *mut opj_event_mgr_t,
  pub p_manager_mutex: *mut opj_mutex_t,
  pub check_pterm: OPJ_BOOL,
}

#[inline]
fn opj_t1_setcurctx(mqc: &mut opj_mqc_t, ctxno: usize) {
  mqc.set_curctx(ctxno);
}

/* Macros to deal with signed integer with just MSB bit set for
 * negative values (smr = signed magnitude representation) */
//#define opj_smr_abs(x)  (((OPJ_UINT32)(x)) & 0x7FFFFFFFU)
fn opj_smr_abs(x: i32) -> u32 {
  x as u32 & 0x7FFFFFFFu32
}

//#define opj_smr_sign(x) (((OPJ_UINT32)(x)) >> 31)
fn opj_smr_sign(x: i32) -> u32 {
  x as u32 >> 31
}

//#define opj_to_smr(x)   ((x) >= 0 ? (OPJ_UINT32)(x) : ((OPJ_UINT32)(-x) | 0x80000000U))
fn opj_to_smr(x: i32) -> u32 {
  if x >= 0 {
    x as u32
  } else {
    -x as u32 | 0x80000000
  }
}

/* * @name Local static functions */
/*@{*/
/*@}*/
/*@}*/
/* ----------------------------------------------------------------------- */

#[inline]
fn opj_t1_getctxno_zc(mut mqc: &mut opj_mqc_t, f: OPJ_UINT32) -> OPJ_BYTE {
  unsafe {
    return *mqc.lut_ctxno_zc_orient.offset((f & T1_SIGMA_NEIGHBOURS) as isize);
  }
}

#[inline]
fn opj_t1_getctxtno_sc_or_spb_index(
  mut fX: OPJ_UINT32,
  mut pfX: OPJ_UINT32,
  mut nfX: OPJ_UINT32,
  mut ci: OPJ_UINT32,
) -> OPJ_UINT32 {
  /*
    0 pfX T1_CHI_THIS           T1_LUT_SGN_W
    1 tfX T1_SIGMA_1            T1_LUT_SIG_N
    2 nfX T1_CHI_THIS           T1_LUT_SGN_E
    3 tfX T1_SIGMA_3            T1_LUT_SIG_W
    4  fX T1_CHI_(THIS - 1)     T1_LUT_SGN_N
    5 tfX T1_SIGMA_5            T1_LUT_SIG_E
    6  fX T1_CHI_(THIS + 1)     T1_LUT_SGN_S
    7 tfX T1_SIGMA_7            T1_LUT_SIG_S
  */

  let mut lu = fX >> ci.wrapping_mul(3) & (T1_SIGMA_1 | T1_SIGMA_3 | T1_SIGMA_5 |
                                          T1_SIGMA_7);

  lu |= (pfX >> T1_CHI_THIS_I.wrapping_add(ci.wrapping_mul(3))) & (1 << 0);
  lu |= (nfX >> (T1_CHI_THIS_I - 2).wrapping_add(ci.wrapping_mul(3))) & (1 << 2);
  if ci == 0 {
    lu |= (fX >> (T1_CHI_0_I - 4)) & (1 << 4);
  } else {
    lu |= (fX >> (T1_CHI_1_I - 4).wrapping_add(ci.wrapping_sub(1).wrapping_mul(3))) & (1 << 4);
  }
  lu |= (fX >> (T1_CHI_2_I - 6).wrapping_add(ci.wrapping_mul(3))) & (1 << 6);
  return lu;
}

#[inline]
unsafe extern "C" fn opj_t1_getctxno_sc(mut lu: OPJ_UINT32) -> OPJ_BYTE {
  return lut_ctxno_sc[lu as usize];
}

#[inline]
fn opj_t1_getctxno_mag(mut f: OPJ_UINT32) -> OPJ_UINT32 {
  let tmp = if f & T1_SIGMA_NEIGHBOURS!= 0 { T1_CTXNO_MAG + 1 } else { T1_CTXNO_MAG };
  let tmp2 = if f & T1_MU_0 != 0 { T1_CTXNO_MAG + 2 } else { tmp };
  return tmp2;
}

#[inline]
unsafe extern "C" fn opj_t1_getspb(mut lu: OPJ_UINT32) -> OPJ_BYTE {
  return lut_spb[lu as usize];
}

unsafe extern "C" fn opj_t1_getnmsedec_sig(mut x: OPJ_UINT32, mut bitpos: OPJ_UINT32) -> OPJ_INT16 {
  if bitpos > 0 {
    return lut_nmsedec_sig[((x >> bitpos) & ((1 << T1_NMSEDEC_BITS) - 1)) as usize];
  }
  return lut_nmsedec_sig0[(x & ((1 << T1_NMSEDEC_BITS) - 1)) as usize];
}

unsafe extern "C" fn opj_t1_getnmsedec_ref(mut x: OPJ_UINT32, mut bitpos: OPJ_UINT32) -> OPJ_INT16 {
  if bitpos > 0 {
    return lut_nmsedec_ref[((x >> bitpos) & ((1 << T1_NMSEDEC_BITS) - 1)) as usize];
  }
  return lut_nmsedec_ref0[(x & ((1 << T1_NMSEDEC_BITS) - 1)) as usize];
}

#[inline]
fn opj_t1_update_flags_macro(
  mut flagsp: *mut opj_flag_t,
  mut ci: OPJ_UINT32,
  mut s: OPJ_UINT32,
  mut stride: OPJ_UINT32,
  mut vsc: OPJ_UINT32,
) {
  unsafe {
    /* east */
    let ref mut fresh0 = *flagsp.offset(-1);
    *fresh0 |= T1_SIGMA_5 << 3_u32.wrapping_mul(ci);

    /* mark target as significant */
    *flagsp |= ((s << T1_CHI_1_I) | T1_SIGMA_4) << 3_u32.wrapping_mul(ci);

    /* west */
    let ref mut fresh1 = *flagsp.offset(1);
    *fresh1 |= T1_SIGMA_3 << 3_u32.wrapping_mul(ci);

    /* north-west, north, north-east */
    if ci == 0 && vsc == 0 {
      let mut north = flagsp.offset(-(stride as isize));
      *north |= (s << T1_CHI_5_I) | T1_SIGMA_16;
      let ref mut fresh2 = *north.offset(-1);
      *fresh2 |= T1_SIGMA_17;
      let ref mut fresh3 = *north.offset(1);
      *fresh3 |= T1_SIGMA_15;
    }

    /* south-west, south, south-east */
    if ci == 3 {
      let mut south = flagsp.offset(stride as isize);
      *south |= (s << T1_CHI_0_I) | T1_SIGMA_1;
      let ref mut fresh4 = *south.offset(-1);
      *fresh4 |= T1_SIGMA_2;
      let ref mut fresh5 = *south.offset(1);
      *fresh5 |= T1_SIGMA_0;
    }
  }
}

#[inline]
fn opj_t1_update_flags(
  mut flagsp: *mut opj_flag_t,
  mut ci: OPJ_UINT32,
  mut s: OPJ_UINT32,
  mut stride: OPJ_UINT32,
  mut vsc: OPJ_UINT32,
) {
  opj_t1_update_flags_macro(flagsp, ci, s, stride, vsc);
}

/* *
Decode significant pass
*/

/* *
Encode significant pass
*/
#[inline]
fn opj_t1_enc_sigpass_step_macro(
  mqc: &mut opj_mqc_t,
  w: OPJ_UINT32,
  flagsp: *mut opj_flag_t,
  l_datap: *const OPJ_INT32,
  bpno: OPJ_INT32,
  one: OPJ_UINT32,
  nmsedec: *mut OPJ_INT32,
  type_0: OPJ_BYTE,
  ci: OPJ_UINT32,
  vsc: OPJ_UINT32,
) {
  unsafe {
    let mut v = 0;
    let flags = *flagsp;
    if (flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == 0 &&
          (flags & (T1_SIGMA_NEIGHBOURS << ci.wrapping_mul(3))) != 0 {
      let ctxt1 = opj_t1_getctxno_zc(mqc, flags >> ci.wrapping_mul(3));
      v = if opj_smr_abs(*l_datap) & one != 0 { 1 } else { 0 };
      log::debug!("   ctxt1={}", ctxt1);
      opj_t1_setcurctx(mqc, ctxt1 as usize);

      if type_0 == T1_TYPE_RAW {  /* BYPASS/LAZY MODE */
        opj_mqc_bypass_enc_macro(mqc, v);
      } else {
        opj_mqc_encode_macro(mqc, v);
      }
      if v != 0 {
        let lu = opj_t1_getctxtno_sc_or_spb_index(
                    flags,
                    *flagsp.offset(-1), *flagsp.offset(1),
                    ci);
        let ctxt2 = opj_t1_getctxno_sc(lu);
        v = opj_smr_sign(*l_datap);
        *nmsedec += opj_t1_getnmsedec_sig(opj_smr_abs(*l_datap), bpno as u32) as i32;
        log::debug!("   ctxt2={}", ctxt2);
        opj_t1_setcurctx(mqc, ctxt2 as usize);
        if type_0 == T1_TYPE_RAW {  /* BYPASS/LAZY MODE */
            opj_mqc_bypass_enc_macro(mqc, v);
        } else {
            let spb = opj_t1_getspb(lu) as OPJ_UINT32;
            log::debug!("   spb={}", spb);
            opj_mqc_encode_macro(mqc, v ^ spb);
        }
        opj_t1_update_flags(flagsp, ci, v, w.wrapping_add(2), vsc);
      }
      *flagsp |= T1_PI_THIS << ci.wrapping_mul(3);
    }
  }
}

#[inline]
fn opj_t1_dec_sigpass_step_raw(
  mut t1: &mut opj_t1_t,
  mut flagsp: *mut opj_flag_t,
  mut datap: *mut OPJ_INT32,
  mut oneplushalf: OPJ_INT32,
  mut vsc: OPJ_UINT32,
  mut ci: OPJ_UINT32,
) {
  unsafe {
    let mut v = 0;
    let mut mqc = &mut t1.mqc; /* RAW component */
    let flags = *flagsp;
    if (flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == 0 &&
          (flags & (T1_SIGMA_NEIGHBOURS << ci.wrapping_mul(3))) != 0 {
      if opj_mqc_raw_decode(mqc) != 0 {
        v = opj_mqc_raw_decode(mqc);
        *datap = if v != 0 { -oneplushalf } else { oneplushalf };
        opj_t1_update_flags(flagsp, ci, v, t1.w.wrapping_add(2), vsc);
      }
      *flagsp |= T1_PI_THIS << ci.wrapping_mul(3);
    }
  }
}

#[inline]
fn opj_t1_dec_sigpass_step_mqc_macro(
  mut flagsp: *mut opj_flag_t,
  mut flags_stride: OPJ_UINT32,
  mut datap: *mut OPJ_INT32,
  mut data_stride: OPJ_UINT32,
  mut ci: OPJ_UINT32,
  mut mqc: &mut opj_mqc_t,
  mut v: OPJ_UINT32,
  mut oneplushalf: OPJ_INT32,
  mut vsc: OPJ_UINT32,
) {
  unsafe {
    let flags = *flagsp;
    if (flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == 0 &&
         (flags & (T1_SIGMA_NEIGHBOURS << ci.wrapping_mul(3))) != 0 {
      let ctxt1 = opj_t1_getctxno_zc(mqc, flags >> ci.wrapping_mul(3));
      opj_t1_setcurctx(mqc, ctxt1 as usize);
      opj_mqc_decode_macro(&mut v, mqc);
      if v != 0 {
        let mut lu = opj_t1_getctxtno_sc_or_spb_index(
                        flags,
                        *flagsp.offset(-1), *flagsp.offset(1),
                        ci);
        let mut ctxt2 = opj_t1_getctxno_sc(lu);
        let mut spb = opj_t1_getspb(lu) as OPJ_UINT32;
        opj_t1_setcurctx(mqc, ctxt2 as usize);
        opj_mqc_decode_macro(&mut v, mqc);
        v = v ^ spb;
        *datap.offset(ci.wrapping_mul(data_stride) as isize) =
          if v != 0 { -oneplushalf } else { oneplushalf };
        opj_t1_update_flags_macro(flagsp, ci, v, flags_stride, vsc);
      }
      *flagsp |= T1_PI_THIS << ci.wrapping_mul(3);
    }
  }
}

#[inline]
fn opj_t1_dec_sigpass_step_mqc(
  mut t1: &mut opj_t1_t,
  mut flagsp: *mut opj_flag_t,
  mut datap: *mut OPJ_INT32,
  mut oneplushalf: OPJ_INT32,
  mut ci: OPJ_UINT32,
  mut flags_stride: OPJ_UINT32,
  mut vsc: OPJ_UINT32,
) {
  let v = 0;
  let mut mqc = &mut t1.mqc; // MQC component
  opj_t1_dec_sigpass_step_mqc_macro(flagsp, flags_stride, datap, 0, ci, mqc, v, oneplushalf, vsc)
}

// #define T1_FLAGS(x, y)
fn t1_flags(t1: &mut opj_t1_t, x: u32, y: u32) -> *mut opj_flag_t {
  unsafe {
    &mut *t1.flags.offset(
      (x + 1).wrapping_add(
        (y / 4 + 1)
        .wrapping_mul(t1.w.wrapping_add(2))
      ) as isize
    )
  }
}

/* *
Encode significant pass
*/
fn opj_t1_enc_sigpass(
  mut t1: &mut opj_t1_t,
  mut bpno: OPJ_INT32,
  mut nmsedec: *mut OPJ_INT32,
  mut type_0: OPJ_BYTE,
  mut cblksty: OPJ_UINT32,
) {
  unsafe {
    let mut i = 0;
    let mut k = 0;
    let one = 1 << (bpno + T1_NMSEDEC_FRACBITS);
    let mut f = t1_flags(t1, 0, 0);
    let extra = 2;
    let mqc = &mut t1.mqc;
    let mut datap: *const OPJ_INT32 = t1.data;

    *nmsedec = 0;
    log::debug!("enc_sigpass: bpno={}", bpno);

    while k < t1.h & !(0x03) {
      let w = t1.w;
      log::debug!(" k={}", k);
      i = 0;
      while i < w {
        log::debug!(" i={}", i);
        if *f == 0 {
          /* Nothing to do for any of the 4 data points */
        } else {
          opj_t1_enc_sigpass_step_macro(
              mqc, t1.w,
              f,
              datap,
              bpno,
              one,
              nmsedec,
              type_0,
              0, cblksty & J2K_CCP_CBLKSTY_VSC);
          opj_t1_enc_sigpass_step_macro(
              mqc, t1.w,
              f,
              datap.offset(1),
              bpno,
              one,
              nmsedec,
              type_0,
              1, 0);
          opj_t1_enc_sigpass_step_macro(
              mqc, t1.w,
              f,
              datap.offset(2),
              bpno,
              one,
              nmsedec,
              type_0,
              2, 0);
          opj_t1_enc_sigpass_step_macro(
              mqc, t1.w,
              f,
              datap.offset(3),
              bpno,
              one,
              nmsedec,
              type_0,
              3, 0);
        }
        i = i.wrapping_add(1);
        f = f.offset(1);
        datap = datap.offset(4)
      }
      k = k.wrapping_add(4);
      f = f.offset(extra)
    }

    if k < t1.h {
      let mut j: OPJ_UINT32 = 0;
      log::debug!(" k={}", k);
      i = 0;
      while i < t1.w {
        log::debug!(" i={}", i);
        if *f == 0 {
          /* Nothing to do for any of the 4 data points */
          datap = datap.offset(t1.h.wrapping_sub(k) as isize)
        } else {
          j = k;
          while j < t1.h {
            opj_t1_enc_sigpass_step_macro(
                mqc, t1.w,
                f,
                datap,
                bpno,
                one,
                nmsedec,
                type_0,
                j - k,
                (j == k && (cblksty & J2K_CCP_CBLKSTY_VSC) != 0) as u32);
            j = j.wrapping_add(1);
            datap = datap.offset(1)
          }
        }
        i = i.wrapping_add(1);
        f = f.offset(1)
      }
    }
  }
}

/* *
Decode significant pass
*/
fn opj_t1_dec_sigpass_raw(
  mut t1: &mut opj_t1_t,
  mut bpno: OPJ_INT32,
  mut cblksty: OPJ_INT32,
) {
  unsafe {
    let mut one = 0;
    let mut half = 0;
    let mut oneplushalf = 0;
    let mut i = 0;
    let mut j = 0;
    let mut k = 0;
    let mut data = t1.data;
    let mut flagsp = t1_flags(t1, 0, 0);
    let l_w = t1.w;
    one = 1 << bpno;
    half = one >> 1;
    oneplushalf = one | half;

    k = 0;
    while k < t1.h & !(0x3) {
      i = 0;
      while i < l_w {
        let mut flags = *flagsp;
        if flags != 0 {
          opj_t1_dec_sigpass_step_raw(
            t1,
            flagsp,
            data,
            oneplushalf,
            cblksty as u32 & J2K_CCP_CBLKSTY_VSC, /* vsc */
            0,
          );
          opj_t1_dec_sigpass_step_raw(
            t1,
            flagsp,
            data.offset(l_w as isize),
            oneplushalf,
            0, /* vsc */
            1,
          );
          opj_t1_dec_sigpass_step_raw(
            t1,
            flagsp,
            data.offset(2_u32.wrapping_mul(l_w) as isize),
            oneplushalf,
            0, /* vsc */
            2,
          );
          opj_t1_dec_sigpass_step_raw(
            t1,
            flagsp,
            data.offset(3_u32.wrapping_mul(l_w) as isize),
            oneplushalf,
            0, /* vsc */
            3,
          );
        }
        i = i.wrapping_add(1);
        flagsp = flagsp.offset(1);
        data = data.offset(1)
      }
      k = k.wrapping_add(4);
      flagsp = flagsp.offset(2);
      data = data.offset(3_u32.wrapping_mul(l_w) as isize)
    }
    if k < t1.h {
      i = 0;
      while i < l_w {
        j = 0;
        while j < t1.h.wrapping_sub(k) {
          opj_t1_dec_sigpass_step_raw(
            t1,
            flagsp,
            data.offset(j.wrapping_mul(l_w) as isize),
            oneplushalf,
            cblksty as u32 & J2K_CCP_CBLKSTY_VSC,
            j,
          );
          j = j.wrapping_add(1)
        }
        i = i.wrapping_add(1);
        flagsp = flagsp.offset(1);
        data = data.offset(1)
      }
    }
  }
}

fn opj_t1_dec_sigpass_mqc_internal(
  mut t1: &mut opj_t1_t,
  mut bpno: OPJ_INT32,
  vsc: OPJ_UINT32,
  w: OPJ_UINT32,
  h: OPJ_UINT32,
  flags_stride: OPJ_UINT32,
) {
  unsafe {
    let mut one = 0;
    let mut half = 0;
    let mut oneplushalf = 0;
    let mut i = 0;
    let mut j = 0;
    let mut k = 0;
    let mut data = t1.data;
    let mut flagsp = &mut *t1.flags.offset(flags_stride as isize + 1) as *mut opj_flag_t;
    let l_w = w;
    let mqc = &mut t1.mqc;
    let mut v = 0;
    one = 1 << bpno;
    half = one >> 1;
    oneplushalf = one | half;

    k = 0;
    while k < h & !(0x03) {
      i = 0;
      while i < l_w {
        if *flagsp != 0 {
          opj_t1_dec_sigpass_step_mqc_macro(
              flagsp, flags_stride, data,
              l_w, 0, mqc, v, oneplushalf, vsc);
          opj_t1_dec_sigpass_step_mqc_macro(
              flagsp, flags_stride, data,
              l_w, 1, mqc, v, oneplushalf, OPJ_FALSE);
          opj_t1_dec_sigpass_step_mqc_macro(
              flagsp, flags_stride, data,
              l_w, 2, mqc, v, oneplushalf, OPJ_FALSE);
          opj_t1_dec_sigpass_step_mqc_macro(
              flagsp, flags_stride, data,
              l_w, 3, mqc, v, oneplushalf, OPJ_FALSE);
        }
        i = i.wrapping_add(1);
        data = data.offset(1);
        flagsp = flagsp.offset(1)
      }
      k = k.wrapping_add(4);
      data = data.offset(3_u32.wrapping_mul(l_w) as isize);
      flagsp = flagsp.offset(2)
    }
    if k < h {
      i = 0;
      while i < l_w {
        j = 0;
        while j < h.wrapping_sub(k) {
          opj_t1_dec_sigpass_step_mqc(t1, flagsp,
            data.offset(j.wrapping_mul(l_w) as isize),
            oneplushalf, j, flags_stride, vsc);
          j = j.wrapping_add(1)
        }
        i = i.wrapping_add(1);
        data = data.offset(1);
        flagsp = flagsp.offset(1)
      }
    }
  }
}

fn opj_t1_dec_sigpass_mqc_64x64_novsc(t1: &mut opj_t1_t, bpno: OPJ_INT32) {
  opj_t1_dec_sigpass_mqc_internal(t1, bpno, OPJ_FALSE, 64, 64, 66);
}

fn opj_t1_dec_sigpass_mqc_64x64_vsc(t1: &mut opj_t1_t, bpno: OPJ_INT32) {
  opj_t1_dec_sigpass_mqc_internal(t1, bpno, OPJ_TRUE, 64, 64, 66);
}

fn opj_t1_dec_sigpass_mqc_generic_novsc(t1: &mut opj_t1_t, bpno: OPJ_INT32) {
  opj_t1_dec_sigpass_mqc_internal(t1, bpno, OPJ_FALSE, t1.w, t1.h,
                                  t1.w + 2);
}

fn opj_t1_dec_sigpass_mqc_generic_vsc(t1: &mut opj_t1_t, bpno: OPJ_INT32) {
  opj_t1_dec_sigpass_mqc_internal(t1, bpno, OPJ_TRUE, t1.w, t1.h,
                                  t1.w + 2);
}

fn opj_t1_dec_sigpass_mqc(
  mut t1: &mut opj_t1_t,
  mut bpno: OPJ_INT32,
  mut cblksty: OPJ_INT32,
) {
  if t1.w == 64 && t1.h == 64 {
    if cblksty as u32 & J2K_CCP_CBLKSTY_VSC != 0 {
      opj_t1_dec_sigpass_mqc_64x64_vsc(t1, bpno);
    } else {
      opj_t1_dec_sigpass_mqc_64x64_novsc(t1, bpno);
    }
  } else if cblksty as u32 & J2K_CCP_CBLKSTY_VSC != 0 {
    opj_t1_dec_sigpass_mqc_generic_vsc(t1, bpno);
  } else {
    opj_t1_dec_sigpass_mqc_generic_novsc(t1, bpno);
  }
}

/* *
Decode refinement pass
*/

/**
Encode refinement pass step
*/
#[inline]
fn opj_t1_enc_refpass_step_macro(
  mqc: &mut opj_mqc_t,
  flagsp: *mut opj_flag_t,
  l_datap: *const OPJ_INT32,
  bpno: OPJ_INT32,
  one: OPJ_UINT32,
  nmsedec: *mut OPJ_INT32,
  type_0: OPJ_BYTE,
  ci: OPJ_UINT32,
) {
  unsafe {
    let mut v: OPJ_UINT32 = 0;
    let flags = *flagsp;
    if (flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == (T1_SIGMA_THIS << ci.wrapping_mul(3)) {
      let shift_flags = flags >> ci.wrapping_mul(3);
      let ctxt = opj_t1_getctxno_mag(shift_flags);
      let abs_data = opj_smr_abs(*l_datap);
      *nmsedec += opj_t1_getnmsedec_ref(abs_data, bpno as u32) as i32;
      v = if abs_data & one != 0 { 1 } else { 0 };
      log::debug!("   ctxt={}", ctxt);
      opj_t1_setcurctx(mqc, ctxt as usize);

      if type_0 == T1_TYPE_RAW {  /* BYPASS/LAZY MODE */
        opj_mqc_bypass_enc_macro(mqc, v);
      } else {
        opj_mqc_encode_macro(mqc, v);
      }
      *flagsp |= T1_MU_THIS << ci.wrapping_mul(3);
    }
  }
}

#[inline]
fn opj_t1_dec_refpass_step_raw(
  mut t1: &mut opj_t1_t,
  mut flagsp: *mut opj_flag_t,
  mut datap: *mut OPJ_INT32,
  mut poshalf: OPJ_INT32,
  mut ci: OPJ_UINT32,
) {
  unsafe {
    let mut v = 0;

    let mut mqc = &mut t1.mqc; /* RAW component */

    if (*flagsp & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == (T1_SIGMA_THIS << ci.wrapping_mul(3)) {
      v = opj_mqc_raw_decode(mqc);
      *datap += if v ^ (*datap < 0) as u32 != 0 {
        poshalf
      } else {
        -poshalf
      };
      *flagsp |= T1_MU_THIS << ci.wrapping_mul(3);
    }
  }
}

fn opj_t1_dec_refpass_step_mqc_macro(
  mut flagsp: *mut opj_flag_t,
  mut datap: *mut OPJ_INT32,
  mut data_stride: OPJ_UINT32,
  mut ci: OPJ_UINT32,
  mut mqc: &mut opj_mqc_t,
  mut v: &mut OPJ_UINT32,
  mut poshalf: OPJ_INT32,
) {
  unsafe {
    let flags = *flagsp;
    if (flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == (T1_SIGMA_THIS << ci.wrapping_mul(3)) {
      let ctxt = opj_t1_getctxno_mag(flags >> ci.wrapping_mul(3));
      opj_t1_setcurctx(mqc, ctxt as usize);
      opj_mqc_decode_macro(v, mqc);
      let datap = datap.offset(ci.wrapping_mul(data_stride) as isize);
      *datap += if *v ^ (*datap < 0) as u32 != 0 {
        poshalf
      } else {
        -poshalf
      };
      *flagsp |= T1_MU_THIS << ci.wrapping_mul(3);
    }
  }
}

#[inline]
fn opj_t1_dec_refpass_step_mqc(
  mut t1: &mut opj_t1_t,
  mut flagsp: *mut opj_flag_t,
  mut datap: *mut OPJ_INT32,
  mut poshalf: OPJ_INT32,
  mut ci: OPJ_UINT32,
) {
  let mut v = 0;

  let mut mqc = &mut t1.mqc; /* MQC component */
  opj_t1_dec_refpass_step_mqc_macro(flagsp, datap, 0, ci,
                                    mqc, &mut v, poshalf);
}

/* *
Encode refinement pass
*/
fn opj_t1_enc_refpass(
  mut t1: &mut opj_t1_t,
  mut bpno: OPJ_INT32,
  mut nmsedec: *mut OPJ_INT32,
  mut type_0: OPJ_BYTE,
) {
  unsafe {
    let mut i: OPJ_UINT32 = 0;
    let mut k: OPJ_UINT32 = 0;
    let one = 1 << (bpno + T1_NMSEDEC_FRACBITS);
    let mut f = t1_flags(t1, 0, 0);
    let extra = 2;
    let mqc = &mut t1.mqc;
    let mut datap: *const OPJ_INT32 = t1.data;

    *nmsedec = 0;
    log::debug!("enc_refpass: bpno={}", bpno);

    while k < t1.h & !(0x03) {
      log::debug!(" k={}", k);
      i = 0;
      while i < t1.w {
        let flags = *f;
        log::debug!(" i={}", i);
        if (flags & (T1_SIGMA_4 | T1_SIGMA_7 | T1_SIGMA_10 | T1_SIGMA_13)) == 0 {
            /* none significant */
        } else if (flags & (T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3)) ==
                (T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3) {
            /* all processed by sigpass */
        } else {
          opj_t1_enc_refpass_step_macro(
              mqc,
              f,
              datap,
              bpno,
              one,
              nmsedec,
              type_0,
              0);
          opj_t1_enc_refpass_step_macro(
              mqc,
              f,
              datap.offset(1),
              bpno,
              one,
              nmsedec,
              type_0,
              1);
          opj_t1_enc_refpass_step_macro(
              mqc,
              f,
              datap.offset(2),
              bpno,
              one,
              nmsedec,
              type_0,
              2);
          opj_t1_enc_refpass_step_macro(
              mqc,
              f,
              datap.offset(3),
              bpno,
              one,
              nmsedec,
              type_0,
              3);
        }
        i = i.wrapping_add(1);
        f = f.offset(1);
        datap = datap.offset(4)
      }
      k = k.wrapping_add(4);
      f = f.offset(extra);
    }

    if k < t1.h {
      let mut j: OPJ_UINT32 = 0;
      let remaining_lines = t1.h - k;
      log::debug!(" k={}", k);
      i = 0;
      while i < t1.w {
        log::debug!(" i={}", i);
        if (*f & (T1_SIGMA_4 | T1_SIGMA_7 | T1_SIGMA_10 | T1_SIGMA_13)) == 0 {
          /* none significant */
          datap = datap.offset(remaining_lines as isize);
        } else {
          j = 0;
          while j < remaining_lines {
            opj_t1_enc_refpass_step_macro(
                mqc,
                f,
                datap,
                bpno,
                one,
                nmsedec,
                type_0,
                j);
            j = j.wrapping_add(1);
            datap = datap.offset(1)
          }
        }
        i = i.wrapping_add(1);
        f = f.offset(1)
      }
    }
  }
}

/* *
Decode refinement pass
*/
fn opj_t1_dec_refpass_raw(mut t1: &mut opj_t1_t, mut bpno: OPJ_INT32) {
  unsafe {
    let mut one: OPJ_INT32 = 0;
    let mut poshalf: OPJ_INT32 = 0;
    let mut i: OPJ_UINT32 = 0;
    let mut j: OPJ_UINT32 = 0;
    let mut k: OPJ_UINT32 = 0;
    let mut data = t1.data;
    let mut flagsp = t1_flags(t1, 0, 0);
    let l_w = t1.w;
    one = 1 << bpno;
    poshalf = one >> 1;
    k = 0;
    while k < t1.h & !(0x03) {
      i = 0;
      while i < l_w {
        let mut flags = *flagsp;
        if flags != 0 {
          opj_t1_dec_refpass_step_raw(t1, flagsp, data, poshalf, 0);
          opj_t1_dec_refpass_step_raw(
            t1,
            flagsp,
            data.offset(l_w as isize),
            poshalf,
            1,
          );
          opj_t1_dec_refpass_step_raw(
            t1,
            flagsp,
            data.offset(2_u32.wrapping_mul(l_w) as isize),
            poshalf,
            2,
          );
          opj_t1_dec_refpass_step_raw(
            t1,
            flagsp,
            data.offset(3_u32.wrapping_mul(l_w) as isize),
            poshalf,
            3,
          );
        }
        i = i.wrapping_add(1);
        flagsp = flagsp.offset(1);
        data = data.offset(1)
      }
      k = k.wrapping_add(4);
      flagsp = flagsp.offset(2);
      data = data.offset(3_u32.wrapping_mul(l_w) as isize)
    }
    if k < t1.h {
      i = 0;
      while i < l_w {
        j = 0;
        while j < t1.h.wrapping_sub(k) {
          opj_t1_dec_refpass_step_raw(
            t1,
            flagsp,
            data.offset(j.wrapping_mul(l_w) as isize),
            poshalf,
            j,
          );
          j = j.wrapping_add(1)
        }
        i = i.wrapping_add(1);
        flagsp = flagsp.offset(1);
        data = data.offset(1)
      }
    }
  }
}

fn opj_t1_dec_refpass_mqc_internal(
  mut t1: &mut opj_t1_t,
  mut bpno: OPJ_INT32,
  w: OPJ_UINT32,
  h: OPJ_UINT32,
  flags_stride: OPJ_UINT32,
) {
  unsafe {
    let mut one= 0;
    let mut poshalf= 0;
    let mut i = 0;
    let mut j = 0;
    let mut k = 0;
    let mut data = t1.data;
    let mut flagsp = &mut *t1.flags.offset(flags_stride as isize + 1) as *mut opj_flag_t;
    let l_w = w;
    let mqc = &mut t1.mqc;
    let mut v = 0;
    one = 1 << bpno;
    poshalf = one >> 1;

    k = 0;
    while k < h & !(0x03) {
      i = 0;
      while i < l_w {
        if *flagsp != 0 {
          opj_t1_dec_refpass_step_mqc_macro(
              flagsp, data, l_w, 0,
              mqc, &mut v, poshalf);
          opj_t1_dec_refpass_step_mqc_macro(
              flagsp, data, l_w, 1,
              mqc, &mut v, poshalf);
          opj_t1_dec_refpass_step_mqc_macro(
              flagsp, data, l_w, 2,
              mqc, &mut v, poshalf);
          opj_t1_dec_refpass_step_mqc_macro(
              flagsp, data, l_w, 3,
              mqc, &mut v, poshalf);
        }
        i = i.wrapping_add(1);
        data = data.offset(1);
        flagsp = flagsp.offset(1)
      }
      k = k.wrapping_add(4);
      data = data.offset(3_u32.wrapping_mul(l_w) as isize);
      flagsp = flagsp.offset(2)
    }
    if k < h {
      i = 0;
      while i < l_w {
        j = 0;
        while j < h.wrapping_sub(k) {
          opj_t1_dec_refpass_step_mqc(t1, flagsp,
            data.offset(j.wrapping_mul(l_w) as isize),
            poshalf, j);
          j = j.wrapping_add(1)
        }
        i = i.wrapping_add(1);
        data = data.offset(1);
        flagsp = flagsp.offset(1)
      }
    }
  }
}

fn opj_t1_dec_refpass_mqc_64x64(mut t1: &mut opj_t1_t, mut bpno: OPJ_INT32) {
  opj_t1_dec_refpass_mqc_internal(t1, bpno, 64, 64, 66);
}

fn opj_t1_dec_refpass_mqc_generic(mut t1: &mut opj_t1_t, mut bpno: OPJ_INT32) {
  opj_t1_dec_refpass_mqc_internal(t1, bpno, t1.w, t1.h, t1.w + 2);
}

fn opj_t1_dec_refpass_mqc(mut t1: &mut opj_t1_t, mut bpno: OPJ_INT32) {
  if t1.w == 64 && t1.h == 64 {
    opj_t1_dec_refpass_mqc_64x64(t1, bpno);
  } else {
    opj_t1_dec_refpass_mqc_generic(t1, bpno);
  };
}

/**
Encode clean-up pass step
*/
#[inline]
fn opj_t1_enc_clnpass_step_macro(
  mqc: &mut opj_mqc_t,
  w: OPJ_UINT32,
  flagsp: *mut opj_flag_t,
  mut l_datap: *const OPJ_INT32,
  bpno: OPJ_INT32,
  one: OPJ_UINT32,
  nmsedec: *mut OPJ_INT32,
  agg: OPJ_BYTE,
  runlen: OPJ_UINT32,
  lim: OPJ_UINT32,
  cblksty: OPJ_UINT32,
) {
  const CHECK: opj_flag_t = (T1_SIGMA_4 | T1_SIGMA_7 | T1_SIGMA_10 | T1_SIGMA_13 |
                              T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3);
  unsafe {
    let mut v = 0;
    if (*flagsp & CHECK) == CHECK {
      if runlen == 0 {
          *flagsp &= !(T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3);
      } else if runlen == 1 {
          *flagsp &= !(T1_PI_1 | T1_PI_2 | T1_PI_3);
      } else if runlen == 2 {
          *flagsp &= !(T1_PI_2 | T1_PI_3);
      } else if runlen == 3 {
          *flagsp &= !(T1_PI_3);
      }
    } else {
      for ci in runlen..lim {
        let mut goto_PARTIAL = false;
        if agg != 0 && ci == runlen {
          goto_PARTIAL = true;
        } else if (*flagsp & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == 0 {
          let ctxt1 = opj_t1_getctxno_zc(mqc, *flagsp >> ci.wrapping_mul(3));
          log::debug!("   ctxt1={}", ctxt1);
          opj_t1_setcurctx(mqc, ctxt1 as usize);
          v = if opj_smr_abs(*l_datap) & one != 0 { 1 } else { 0 };
          opj_mqc_encode_macro(mqc, v);
          if v != 0 {
            goto_PARTIAL = true;
          }
        }
        if goto_PARTIAL {
          let lu = opj_t1_getctxtno_sc_or_spb_index(
                      *flagsp,
                      *flagsp.offset(-1), *flagsp.offset(1),
                      ci);
          *nmsedec += opj_t1_getnmsedec_sig(opj_smr_abs(*l_datap), bpno as u32) as i32;
          let ctxt2 = opj_t1_getctxno_sc(lu);
          log::debug!("   ctxt2={}", ctxt2);
          opj_t1_setcurctx(mqc, ctxt2 as usize);

          v = opj_smr_sign(*l_datap);
          let spb = opj_t1_getspb(lu);
          log::debug!("   spb={}", spb);
          opj_mqc_encode_macro(mqc, v ^ spb as u32);
          let vsc = if (cblksty & J2K_CCP_CBLKSTY_VSC) != 0 && ci == 0 { 1 } else { 0 };
          opj_t1_update_flags(flagsp, ci, v, w + 2, vsc);
        }
        *flagsp &= !(T1_PI_THIS << ci.wrapping_mul(3));
        l_datap = l_datap.offset(1);
      }
    }
  }
}

#[inline]
fn opj_t1_dec_clnpass_step_macro(
  check_flags: bool,
  partial: bool,
  flagsp: *mut opj_flag_t,
  flags_stride: OPJ_UINT32,
  datap: *mut OPJ_INT32,
  data_stride: OPJ_UINT32,
  ci: OPJ_UINT32,
  mqc: &mut opj_mqc_t,
  mut v: OPJ_UINT32,
  oneplushalf: OPJ_INT32,
  vsc: OPJ_UINT32,
) {
  unsafe {
    let flags = *flagsp;
    if !check_flags || (flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == 0 {
      if !partial  {
        let ctxt1 = opj_t1_getctxno_zc(mqc, flags >> ci.wrapping_mul(3));
        opj_t1_setcurctx(mqc, ctxt1 as usize);
        opj_mqc_decode_macro(&mut v, mqc);
        if v == 0 {
          return;
        }
      }
      let mut lu = opj_t1_getctxtno_sc_or_spb_index(
                      flags,
                      *flagsp.offset(-1), *flagsp.offset(1),
                      ci);
      opj_t1_setcurctx(mqc, opj_t1_getctxno_sc(lu) as usize);
      opj_mqc_decode_macro(&mut v, mqc);
      v = v ^ opj_t1_getspb(lu) as u32;
      *datap.offset(ci.wrapping_mul(data_stride) as isize) =
        if v != 0 { -oneplushalf } else { oneplushalf };
      opj_t1_update_flags_macro(flagsp, ci, v, flags_stride, vsc);
    }
  }
}

unsafe extern "C" fn opj_t1_dec_clnpass_step(
  mut t1: *mut opj_t1_t,
  mut flagsp: *mut opj_flag_t,
  mut datap: *mut OPJ_INT32,
  mut oneplushalf: OPJ_INT32,
  mut ci: OPJ_UINT32,
  mut vsc: OPJ_UINT32,
) {
  let v = 0;
  let mqc = &mut (*t1).mqc; /* MQC component */

  opj_t1_dec_clnpass_step_macro(true, false,
                                flagsp, (*t1).w + 2,
                                datap, 0,
                                ci, mqc,
                                v, oneplushalf, vsc);
}

/* *
Encode clean-up pass
*/
fn opj_t1_enc_clnpass(
  mut t1: &mut opj_t1_t,
  mut bpno: OPJ_INT32,
  mut nmsedec: *mut OPJ_INT32,
  mut cblksty: OPJ_UINT32,
) {
  unsafe {
    let mut i: OPJ_UINT32 = 0;
    let mut k: OPJ_UINT32 = 0;
    let one = 1 << (bpno + T1_NMSEDEC_FRACBITS);
    let mut f = t1_flags(t1, 0, 0);
    let mqc = &mut t1.mqc;
    let mut datap: *const OPJ_INT32 = t1.data;
    let extra = 2;

    *nmsedec = 0;
    log::debug!("enc_clnpass: bpno={}", bpno);
    k = 0;
    while k < (t1.h & !0x03) {
      log::debug!(" k={}", k);
      i = 0;
      while i < t1.w {
        log::debug!(" i={}", i);
        let mut agg = 0;
        let mut runlen = 0u32;
        agg = (*f == 0) as u8;
        log::debug!("   agg={}", agg);
        loop {
          if agg != 0 {
            runlen = 0;
            while runlen < 4 {
              if (opj_smr_abs(*datap) & one) != 0 {
                break;
              }
              runlen = runlen.wrapping_add(1);
              datap = datap.offset(1)
            }
            opj_t1_setcurctx(mqc, T1_CTXNO_AGG as usize);
            opj_mqc_encode_macro(mqc, (runlen != 4) as u32);
            if runlen == 4 {
              break;
            }
            opj_t1_setcurctx(mqc, T1_CTXNO_UNI as usize);
            opj_mqc_encode_macro(mqc, runlen >> 1);
            opj_mqc_encode_macro(mqc, runlen & 1);
          } else {
            runlen = 0;
          }
          opj_t1_enc_clnpass_step_macro(mqc, t1.w, f, datap, bpno, one, nmsedec, agg, runlen, 4, cblksty);
          datap = datap.offset(4_u32.wrapping_sub(runlen) as isize);
          break;
        }
        i = i.wrapping_add(1);
        f = f.offset(1)
      }
      k = k.wrapping_add(4);
      f = f.offset(extra as isize)
    }

    if k < t1.h {
      let agg = 0;
      let runlen = 0;
      log::debug!(" k={}", k);
      i = 0;
      while i < t1.w {
        log::debug!(" i={}", i);
        log::debug!("  agg={}", agg);
        opj_t1_enc_clnpass_step_macro(mqc, t1.w, f, datap, bpno, one, nmsedec, agg, runlen, t1.h - k, cblksty);
        datap = datap.offset((t1.h - k) as isize);
        i = i.wrapping_add(1);
        f = f.offset(1)
      }
    }
  }
}

fn opj_t1_dec_clnpass_internal(t1: &mut opj_t1_t, bpno: OPJ_INT32, vsc: bool, w: u32, h: u32, flags_stride: u32) {
  unsafe {
    let mut one= 0;
    let mut half= 0;
    let mut oneplushalf= 0;
    let mut runlen = 0u32;
    let mut i = 0;
    let mut j = 0;
    let mut k = 0;
    let mqc = &mut t1.mqc;
    let mut data = t1.data;
    let mut flagsp = &mut *t1.flags.offset(flags_stride as isize + 1) as *mut opj_flag_t;
    let l_w = w;
    let mut v = 0u32;
    one = 1 << bpno;
    half = one >> 1;
    oneplushalf = one | half;

    k = 0;
    while k < (h & !3) {
      i = 0;
      while i < l_w {
        if *flagsp == 0 {
          let mut partial = true;
          opj_t1_setcurctx(mqc, T1_CTXNO_AGG as usize);
          opj_mqc_decode_macro(&mut v, mqc);
          if v == 0 {
            // continue;
          } else {
            opj_t1_setcurctx(mqc, T1_CTXNO_UNI as usize);
            opj_mqc_decode_macro(&mut runlen, mqc);
            opj_mqc_decode_macro(&mut v, mqc);
            runlen = (runlen << 1) | v;
            if runlen == 0 {
                opj_t1_dec_clnpass_step_macro(false, true,
                          flagsp, flags_stride, data,
                          l_w, 0, mqc,
                          v, oneplushalf, vsc as u32);
                partial = false;
                /* FALLTHRU */
            }
            if runlen <= 1 {
                opj_t1_dec_clnpass_step_macro(false, partial,
                          flagsp, flags_stride, data,
                          l_w, 1, mqc,
                          v, oneplushalf, false as u32);
                partial = false;
                /* FALLTHRU */
            }
            if runlen <= 2 {
                opj_t1_dec_clnpass_step_macro(false, partial,
                          flagsp, flags_stride, data,
                          l_w, 2, mqc,
                          v, oneplushalf, false as u32);
                partial = false;
                /* FALLTHRU */
            }
            if runlen <= 3 {
                opj_t1_dec_clnpass_step_macro(false, partial,
                          flagsp, flags_stride, data,
                          l_w, 3, mqc,
                          v, oneplushalf, false as u32);
            }
            *flagsp &= !(T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3);
          }
        } else {
          opj_t1_dec_clnpass_step_macro(true, false,
                    flagsp, flags_stride, data,
                    l_w, 0, mqc,
                    v, oneplushalf, vsc as u32);
          opj_t1_dec_clnpass_step_macro(true, false,
                    flagsp, flags_stride, data,
                    l_w, 1, mqc,
                    v, oneplushalf, false as u32);
          opj_t1_dec_clnpass_step_macro(true, false,
                    flagsp, flags_stride, data,
                    l_w, 2, mqc,
                    v, oneplushalf, false as u32);
          opj_t1_dec_clnpass_step_macro(true, false,
                    flagsp, flags_stride, data,
                    l_w, 3, mqc,
                    v, oneplushalf, false as u32);
          *flagsp &= !(T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3);
        }
        i = i.wrapping_add(1);
        data = data.offset(1);
        flagsp = flagsp.offset(1)
      }
      k = k.wrapping_add(4);
      data = data.offset(3_u32.wrapping_mul(l_w) as isize);
      flagsp = flagsp.offset(2);
    }
    if k < h {
      i = 0;
      while i < l_w {
        j = 0;
        while j < h - k {
          opj_t1_dec_clnpass_step(
            t1,
            flagsp,
            data.offset(j.wrapping_mul(l_w) as isize),
            oneplushalf,
            j,
            vsc as u32,
          );
          j = j.wrapping_add(1)
        }
        *flagsp &= !(T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3);
        i = i.wrapping_add(1);
        flagsp = flagsp.offset(1);
        data = data.offset(1)
      }
    };
  }
}

fn opj_t1_dec_clnpass_check_segsym(
  mut t1: &mut opj_t1_t,
  mut cblksty: OPJ_INT32,
) {
  if (cblksty as u32 & J2K_CCP_CBLKSTY_SEGSYM) != 0 {
    let mqc = &mut t1.mqc;
    let mut v = 0;
    let mut v2 = 0;
    opj_mqc_setcurctx(mqc, T1_CTXNO_UNI as usize);
    opj_mqc_decode_macro(&mut v, mqc);
    opj_mqc_decode_macro(&mut v2, mqc);
    v = (v << 1) | v2;
    opj_mqc_decode_macro(&mut v2, mqc);
    v = (v << 1) | v2;
    opj_mqc_decode_macro(&mut v2, mqc);
    v = (v << 1) | v2;
    if v != 0xa {
      /*
      opj_event_msg(t1->cinfo, EVT_WARNING, "Bad segmentation symbol %x\n", v);
      */
    }
  }
}

fn opj_t1_dec_clnpass_64x64_novsc(t1: &mut opj_t1_t, mut bpno: OPJ_INT32) {
  opj_t1_dec_clnpass_internal(t1, bpno, false, 64, 64, 66);
}

fn opj_t1_dec_clnpass_64x64_vsc(t1: &mut opj_t1_t, mut bpno: OPJ_INT32) {
  opj_t1_dec_clnpass_internal(t1, bpno, true, 64, 64, 66);
}

fn opj_t1_dec_clnpass_generic_novsc(t1: &mut opj_t1_t, mut bpno: OPJ_INT32) {
  opj_t1_dec_clnpass_internal(t1, bpno, false, t1.w, t1.h, t1.w + 2);
}

fn opj_t1_dec_clnpass_generic_vsc(t1: &mut opj_t1_t, mut bpno: OPJ_INT32) {
  opj_t1_dec_clnpass_internal(t1, bpno, true, t1.w, t1.h, t1.w + 2);
}

fn opj_t1_dec_clnpass(
  mut t1: &mut opj_t1_t,
  mut bpno: OPJ_INT32,
  mut cblksty: OPJ_INT32,
) {
  if t1.w == 64 && t1.h == 64 {
    if (cblksty as u32 & J2K_CCP_CBLKSTY_VSC) != 0 {
      opj_t1_dec_clnpass_64x64_vsc(t1, bpno);
    } else {
      opj_t1_dec_clnpass_64x64_novsc(t1, bpno);
    }
  } else if (cblksty as u32 & J2K_CCP_CBLKSTY_VSC) != 0 {
    opj_t1_dec_clnpass_generic_vsc(t1, bpno);
  } else {
    opj_t1_dec_clnpass_generic_novsc(t1, bpno);
  }
  opj_t1_dec_clnpass_check_segsym(t1, cblksty);
}

/* * mod fixed_quality */
unsafe extern "C" fn opj_t1_getwmsedec(
  mut nmsedec: OPJ_INT32,
  mut compno: OPJ_UINT32,
  mut level: OPJ_UINT32,
  mut orient: OPJ_UINT32,
  mut bpno: OPJ_INT32,
  mut qmfbid: OPJ_UINT32,
  mut stepsize: OPJ_FLOAT64,
  mut _numcomps: OPJ_UINT32,
  mut mct_norms: *const OPJ_FLOAT64,
  mut mct_numcomps: OPJ_UINT32,
) -> OPJ_FLOAT64 {
  let mut w1 = 1 as libc::c_int as OPJ_FLOAT64; /* if (qmfbid == 0) */
  let mut w2: OPJ_FLOAT64 = 0.;
  let mut wmsedec: OPJ_FLOAT64 = 0.;
  if !mct_norms.is_null() && compno < mct_numcomps {
    w1 = *mct_norms.offset(compno as isize)
  }
  if qmfbid == 1 as libc::c_int as libc::c_uint {
    w2 = opj_dwt_getnorm(level, orient)
  } else {
    let log2_gain = if orient == 0 as libc::c_int as libc::c_uint {
      0 as libc::c_int
    } else if orient == 3 as libc::c_int as libc::c_uint {
      2 as libc::c_int
    } else {
      1 as libc::c_int
    };
    w2 = opj_dwt_getnorm_real(level, orient);
    /* Not sure this is right. But preserves past behaviour */
    stepsize /= ((1 as libc::c_int) << log2_gain) as libc::c_double
  }
  wmsedec = w1 * w2 * stepsize * ((1 as libc::c_int) << bpno) as libc::c_double;
  wmsedec *= wmsedec * nmsedec as libc::c_double / 8192.0f64;
  return wmsedec;
}
unsafe extern "C" fn opj_t1_allocate_buffers(
  mut t1: *mut opj_t1_t,
  mut w: OPJ_UINT32,
  mut h: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut flagssize: OPJ_UINT32 = 0;
  let mut flags_stride: OPJ_UINT32 = 0;
  /* No risk of overflow. Prior checks ensure those assert are met */
  /* They are per the specification */

  assert!(w <= 1024 as libc::c_int as libc::c_uint);
  assert!(h <= 1024 as libc::c_int as libc::c_uint);
  assert!(w.wrapping_mul(h) <= 4096 as libc::c_int as libc::c_uint);
  /* encoder uses tile buffer, so no need to allocate */
  let mut datasize = w.wrapping_mul(h);
  if datasize > (*t1).datasize {
    opj_aligned_free((*t1).data as *mut libc::c_void);
    (*t1).data = opj_aligned_malloc(
      (datasize as libc::c_ulong).wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
    ) as *mut OPJ_INT32;
    if (*t1).data.is_null() {
      /* FIXME event manager error callback */
      return 0 as libc::c_int;
    }
    (*t1).datasize = datasize
  }
  /* memset first arg is declared to never be null by gcc */
  if !(*t1).data.is_null() {
    memset(
      (*t1).data as *mut libc::c_void,
      0 as libc::c_int,
      (datasize as libc::c_ulong).wrapping_mul(::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
    ); /* can't be 0U */
  }
  flags_stride = w.wrapping_add(2 as libc::c_uint);
  flagssize = h
    .wrapping_add(3 as libc::c_uint)
    .wrapping_div(4 as libc::c_uint)
    .wrapping_add(2 as libc::c_uint);
  flagssize = (flagssize as libc::c_uint).wrapping_mul(flags_stride);
  let mut p = 0 as *mut opj_flag_t;
  let mut x: OPJ_UINT32 = 0;
  let mut flags_height = h
    .wrapping_add(3 as libc::c_uint)
    .wrapping_div(4 as libc::c_uint);
  if flagssize > (*t1).flagssize {
    opj_aligned_free((*t1).flags as *mut libc::c_void);
    (*t1).flags = opj_aligned_malloc(
      (flagssize as libc::c_ulong)
        .wrapping_mul(::std::mem::size_of::<opj_flag_t>() as libc::c_ulong),
    ) as *mut opj_flag_t;
    if (*t1).flags.is_null() {
      /* FIXME event manager error callback */
      return 0 as libc::c_int;
    }
  }
  (*t1).flagssize = flagssize;
  memset(
    (*t1).flags as *mut libc::c_void,
    0 as libc::c_int,
    (flagssize as libc::c_ulong).wrapping_mul(::std::mem::size_of::<opj_flag_t>() as libc::c_ulong),
  );
  p = &mut *(*t1).flags.offset(0 as libc::c_int as isize) as *mut opj_flag_t;
  x = 0 as libc::c_int as OPJ_UINT32;
  while x < flags_stride {
    /* magic value to hopefully stop any passes being interested in this entry */
    let fresh315 = p;
    p = p.offset(1);
    *fresh315 = (1 as libc::c_uint) << 21 as libc::c_int
      | (1 as libc::c_uint) << 24 as libc::c_int
      | (1 as libc::c_uint) << 27 as libc::c_int
      | (1 as libc::c_uint) << 30 as libc::c_int;
    x = x.wrapping_add(1)
  }
  p = &mut *(*t1).flags.offset(
    flags_height
      .wrapping_add(1 as libc::c_int as libc::c_uint)
      .wrapping_mul(flags_stride) as isize,
  ) as *mut opj_flag_t;
  x = 0 as libc::c_int as OPJ_UINT32;
  while x < flags_stride {
    /* magic value to hopefully stop any passes being interested in this entry */
    let fresh316 = p;
    p = p.offset(1);
    *fresh316 = (1 as libc::c_uint) << 21 as libc::c_int
      | (1 as libc::c_uint) << 24 as libc::c_int
      | (1 as libc::c_uint) << 27 as libc::c_int
      | (1 as libc::c_uint) << 30 as libc::c_int;
    x = x.wrapping_add(1)
  }
  if h.wrapping_rem(4 as libc::c_int as libc::c_uint) != 0 {
    let mut v = 0 as libc::c_int as OPJ_UINT32;
    p = &mut *(*t1)
      .flags
      .offset(flags_height.wrapping_mul(flags_stride) as isize) as *mut opj_flag_t;
    if h.wrapping_rem(4 as libc::c_int as libc::c_uint) == 1 as libc::c_int as libc::c_uint {
      v |= (1 as libc::c_uint) << 24 as libc::c_int
        | (1 as libc::c_uint) << 27 as libc::c_int
        | (1 as libc::c_uint) << 30 as libc::c_int
    } else if h.wrapping_rem(4 as libc::c_int as libc::c_uint) == 2 as libc::c_int as libc::c_uint {
      v |= (1 as libc::c_uint) << 27 as libc::c_int | (1 as libc::c_uint) << 30 as libc::c_int
    } else if h.wrapping_rem(4 as libc::c_int as libc::c_uint) == 3 as libc::c_int as libc::c_uint {
      v |= (1 as libc::c_uint) << 30 as libc::c_int
    }
    x = 0 as libc::c_int as OPJ_UINT32;
    while x < flags_stride {
      let fresh317 = p;
      p = p.offset(1);
      *fresh317 = v;
      x = x.wrapping_add(1)
    }
  }
  (*t1).w = w;
  (*t1).h = h;
  return 1 as libc::c_int;
}
/* ----------------------------------------------------------------------- */
/* ----------------------------------------------------------------------- */
/* *
 * Creates a new Tier 1 handle
 * and initializes the look-up tables of the Tier-1 coder/decoder
 * @return a new T1 handle if successful, returns NULL otherwise
*/
#[no_mangle]
pub unsafe extern "C" fn opj_t1_create(mut isEncoder: OPJ_BOOL) -> *mut opj_t1_t {
  let mut l_t1 = 0 as *mut opj_t1_t;
  l_t1 = opj_calloc(
    1 as libc::c_int as size_t,
    ::std::mem::size_of::<opj_t1_t>() as libc::c_ulong,
  ) as *mut opj_t1_t;
  if l_t1.is_null() {
    return 0 as *mut opj_t1_t;
  }
  (*l_t1).encoder = isEncoder;
  return l_t1;
}
/* *
 * Destroys a previously created T1 handle
 *
 * @param p_t1 Tier 1 handle to destroy
*/
#[no_mangle]
pub unsafe extern "C" fn opj_t1_destroy(mut p_t1: *mut opj_t1_t) {
  if p_t1.is_null() {
    return;
  }
  if !(*p_t1).data.is_null() {
    opj_aligned_free((*p_t1).data as *mut libc::c_void);
    (*p_t1).data = 0 as *mut OPJ_INT32
  }
  if !(*p_t1).flags.is_null() {
    opj_aligned_free((*p_t1).flags as *mut libc::c_void);
    (*p_t1).flags = 0 as *mut opj_flag_t
  }
  opj_free((*p_t1).cblkdatabuffer as *mut libc::c_void);
  opj_free(p_t1 as *mut libc::c_void);
}
unsafe extern "C" fn opj_t1_destroy_wrapper(mut t1: *mut libc::c_void) {
  opj_t1_destroy(t1 as *mut opj_t1_t);
}
unsafe extern "C" fn opj_t1_clbl_decode_processor(
  mut user_data: *mut libc::c_void,
  mut tls: *mut opj_tls_t,
) {
  let mut cblk = 0 as *mut opj_tcd_cblk_dec_t;
  let mut band = 0 as *mut opj_tcd_band_t;
  let mut tilec = 0 as *mut opj_tcd_tilecomp_t;
  let mut tccp = 0 as *mut opj_tccp_t;
  let mut datap = 0 as *mut OPJ_INT32;
  let mut cblk_w: OPJ_UINT32 = 0;
  let mut cblk_h: OPJ_UINT32 = 0;
  let mut x: OPJ_INT32 = 0;
  let mut y: OPJ_INT32 = 0;
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut job = 0 as *mut opj_t1_cblk_decode_processing_job_t;
  let mut t1 = 0 as *mut opj_t1_t;
  let mut resno: OPJ_UINT32 = 0;
  let mut tile_w: OPJ_UINT32 = 0;
  job = user_data as *mut opj_t1_cblk_decode_processing_job_t;
  cblk = (*job).cblk;
  if (*job).whole_tile_decoding == 0 {
    cblk_w = ((*cblk).x1 - (*cblk).x0) as OPJ_UINT32;
    cblk_h = ((*cblk).y1 - (*cblk).y0) as OPJ_UINT32;
    (*cblk).decoded_data = opj_aligned_malloc(
      (::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong)
        .wrapping_mul(cblk_w as libc::c_ulong)
        .wrapping_mul(cblk_h as libc::c_ulong),
    ) as *mut OPJ_INT32;
    if (*cblk).decoded_data.is_null() {
      if !(*job).p_manager_mutex.is_null() {
        opj_mutex_lock((*job).p_manager_mutex);
      }
      opj_event_msg(
        (*job).p_manager,
        1 as libc::c_int,
        b"Cannot allocate cblk->decoded_data\n\x00" as *const u8 as *const libc::c_char,
      );
      if !(*job).p_manager_mutex.is_null() {
        opj_mutex_unlock((*job).p_manager_mutex);
      }
      ::std::ptr::write_volatile((*job).pret, 0 as libc::c_int);
      opj_free(job as *mut libc::c_void);
      return;
    }
    /* Zero-init required */
    memset(
      (*cblk).decoded_data as *mut libc::c_void,
      0 as libc::c_int,
      (::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong)
        .wrapping_mul(cblk_w as libc::c_ulong)
        .wrapping_mul(cblk_h as libc::c_ulong),
    );
  } else if !(*cblk).decoded_data.is_null() {
    /* Not sure if that code path can happen, but better be */
    /* safe than sorry */
    opj_aligned_free((*cblk).decoded_data as *mut libc::c_void);
    (*cblk).decoded_data = 0 as *mut OPJ_INT32
  }
  resno = (*job).resno;
  band = (*job).band;
  tilec = (*job).tilec;
  tccp = (*job).tccp;
  tile_w = ((*(*tilec).resolutions.offset(
    (*tilec)
      .minimum_num_resolutions
      .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
  ))
  .x1
    - (*(*tilec).resolutions.offset(
      (*tilec)
        .minimum_num_resolutions
        .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
    ))
    .x0) as OPJ_UINT32;
  if *(*job).pret == 0 {
    opj_free(job as *mut libc::c_void);
    return;
  }
  t1 = opj_tls_get(tls, 0 as libc::c_int) as *mut opj_t1_t;
  if t1.is_null() {
    t1 = opj_t1_create(0 as libc::c_int);
    if t1.is_null() {
      opj_event_msg(
        (*job).p_manager,
        1 as libc::c_int,
        b"Cannot allocate Tier 1 handle\n\x00" as *const u8 as *const libc::c_char,
      );
      ::std::ptr::write_volatile((*job).pret, 0 as libc::c_int);
      opj_free(job as *mut libc::c_void);
      return;
    }
    if opj_tls_set(
      tls,
      0 as libc::c_int,
      t1 as *mut libc::c_void,
      Some(opj_t1_destroy_wrapper as unsafe extern "C" fn(_: *mut libc::c_void) -> ()),
    ) == 0
    {
      opj_event_msg(
        (*job).p_manager,
        1 as libc::c_int,
        b"Unable to set t1 handle as TLS\n\x00" as *const u8 as *const libc::c_char,
      );
      opj_t1_destroy(t1);
      ::std::ptr::write_volatile((*job).pret, 0 as libc::c_int);
      opj_free(job as *mut libc::c_void);
      return;
    }
  }
  (*t1).mustuse_cblkdatabuffer = (*job).mustuse_cblkdatabuffer;
  if (*tccp).cblksty & 0x40 as libc::c_int as libc::c_uint != 0 as libc::c_int as libc::c_uint {
    if 0 as libc::c_int
      == opj_t1_ht_decode_cblk(
        t1,
        cblk,
        (*band).bandno,
        (*tccp).roishift as OPJ_UINT32,
        (*tccp).cblksty,
        (*job).p_manager,
        (*job).p_manager_mutex,
        (*job).check_pterm,
      )
    {
      ::std::ptr::write_volatile((*job).pret, 0 as libc::c_int);
      opj_free(job as *mut libc::c_void);
      return;
    }
  } else if 0 as libc::c_int
    == opj_t1_decode_cblk(
      t1,
      cblk,
      (*band).bandno,
      (*tccp).roishift as OPJ_UINT32,
      (*tccp).cblksty,
      (*job).p_manager,
      (*job).p_manager_mutex,
      (*job).check_pterm,
    )
  {
    ::std::ptr::write_volatile((*job).pret, 0 as libc::c_int);
    opj_free(job as *mut libc::c_void);
    return;
  }
  x = (*cblk).x0 - (*band).x0;
  y = (*cblk).y0 - (*band).y0;
  if (*band).bandno & 1 as libc::c_int as libc::c_uint != 0 {
    let mut pres: *mut opj_tcd_resolution_t = &mut *(*tilec)
      .resolutions
      .offset(resno.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize)
      as *mut opj_tcd_resolution_t;
    x += (*pres).x1 - (*pres).x0
  }
  if (*band).bandno & 2 as libc::c_int as libc::c_uint != 0 {
    let mut pres_0: *mut opj_tcd_resolution_t = &mut *(*tilec)
      .resolutions
      .offset(resno.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize)
      as *mut opj_tcd_resolution_t;
    y += (*pres_0).y1 - (*pres_0).y0
  }
  datap = if !(*cblk).decoded_data.is_null() {
    (*cblk).decoded_data
  } else {
    (*t1).data
  };
  cblk_w = (*t1).w;
  cblk_h = (*t1).h;
  if (*tccp).roishift != 0 {
    if (*tccp).roishift >= 31 as libc::c_int {
      j = 0 as libc::c_int as OPJ_UINT32;
      while j < cblk_h {
        i = 0 as libc::c_int as OPJ_UINT32;
        while i < cblk_w {
          *datap.offset(j.wrapping_mul(cblk_w).wrapping_add(i) as isize) = 0 as libc::c_int;
          i = i.wrapping_add(1)
        }
        j = j.wrapping_add(1)
      }
    } else {
      let mut thresh = (1 as libc::c_int) << (*tccp).roishift;
      j = 0 as libc::c_int as OPJ_UINT32;
      while j < cblk_h {
        i = 0 as libc::c_int as OPJ_UINT32;
        while i < cblk_w {
          let mut val = *datap.offset(j.wrapping_mul(cblk_w).wrapping_add(i) as isize);
          let mut mag = abs(val);
          if mag >= thresh {
            mag >>= (*tccp).roishift;
            *datap.offset(j.wrapping_mul(cblk_w).wrapping_add(i) as isize) =
              if val < 0 as libc::c_int { -mag } else { mag }
          }
          i = i.wrapping_add(1)
        }
        j = j.wrapping_add(1)
      }
    }
  }
  /* Both can be non NULL if for example decoding a full tile and then */
  /* partially a tile. In which case partial decoding should be the */
  /* priority */
  assert!(!(*cblk).decoded_data.is_null() || !(*tilec).data.is_null()); /* if (tccp->qmfbid == 0) */
  if !(*cblk).decoded_data.is_null() {
    let mut cblk_size = cblk_w.wrapping_mul(cblk_h); /* resno */
    if (*tccp).qmfbid == 1 as libc::c_int as libc::c_uint {
      i = 0 as libc::c_int as OPJ_UINT32;
      while i < cblk_size {
        let ref mut fresh318 = *datap.offset(i as isize);
        *fresh318 /= 2 as libc::c_int;
        i = i.wrapping_add(1)
      }
    } else {
      let stepsize = 0.5f32 * (*band).stepsize;
      i = 0 as libc::c_int as OPJ_UINT32;
      while i < cblk_size {
        let mut tmp = *datap as OPJ_FLOAT32 * stepsize;
        memcpy(
          datap as *mut libc::c_void,
          &mut tmp as *mut OPJ_FLOAT32 as *const libc::c_void,
          ::std::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong,
        );
        datap = datap.offset(1);
        i = i.wrapping_add(1)
      }
    }
  } else if (*tccp).qmfbid == 1 as libc::c_int as libc::c_uint {
    let mut tiledp: *mut OPJ_INT32 = &mut *(*tilec).data.offset(
      (y as OPJ_SIZE_T)
        .wrapping_mul(tile_w as libc::c_ulong)
        .wrapping_add(x as OPJ_SIZE_T) as isize,
    ) as *mut OPJ_INT32;
    j = 0 as libc::c_int as OPJ_UINT32;
    while j < cblk_h {
      i = 0 as libc::c_int as OPJ_UINT32;
      while i < cblk_w & !(3 as libc::c_uint) {
        let mut tmp0 = *datap.offset(
          j.wrapping_mul(cblk_w)
            .wrapping_add(i)
            .wrapping_add(0 as libc::c_uint) as isize,
        );
        let mut tmp1 = *datap.offset(
          j.wrapping_mul(cblk_w)
            .wrapping_add(i)
            .wrapping_add(1 as libc::c_uint) as isize,
        );
        let mut tmp2 = *datap.offset(
          j.wrapping_mul(cblk_w)
            .wrapping_add(i)
            .wrapping_add(2 as libc::c_uint) as isize,
        );
        let mut tmp3 = *datap.offset(
          j.wrapping_mul(cblk_w)
            .wrapping_add(i)
            .wrapping_add(3 as libc::c_uint) as isize,
        );
        *tiledp.offset(
          (j as libc::c_ulong)
            .wrapping_mul(tile_w as OPJ_SIZE_T)
            .wrapping_add(i as libc::c_ulong)
            .wrapping_add(0 as libc::c_uint as libc::c_ulong) as isize,
        ) = tmp0 / 2 as libc::c_int;
        *tiledp.offset(
          (j as libc::c_ulong)
            .wrapping_mul(tile_w as OPJ_SIZE_T)
            .wrapping_add(i as libc::c_ulong)
            .wrapping_add(1 as libc::c_uint as libc::c_ulong) as isize,
        ) = tmp1 / 2 as libc::c_int;
        *tiledp.offset(
          (j as libc::c_ulong)
            .wrapping_mul(tile_w as OPJ_SIZE_T)
            .wrapping_add(i as libc::c_ulong)
            .wrapping_add(2 as libc::c_uint as libc::c_ulong) as isize,
        ) = tmp2 / 2 as libc::c_int;
        *tiledp.offset(
          (j as libc::c_ulong)
            .wrapping_mul(tile_w as OPJ_SIZE_T)
            .wrapping_add(i as libc::c_ulong)
            .wrapping_add(3 as libc::c_uint as libc::c_ulong) as isize,
        ) = tmp3 / 2 as libc::c_int;
        i = (i as libc::c_uint).wrapping_add(4 as libc::c_uint)
      }
      while i < cblk_w {
        let mut tmp_0 = *datap.offset(j.wrapping_mul(cblk_w).wrapping_add(i) as isize);
        *tiledp.offset(
          (j as libc::c_ulong)
            .wrapping_mul(tile_w as OPJ_SIZE_T)
            .wrapping_add(i as libc::c_ulong) as isize,
        ) = tmp_0 / 2 as libc::c_int;
        i = i.wrapping_add(1)
      }
      j = j.wrapping_add(1)
    }
  } else {
    let stepsize_0 = 0.5f32 * (*band).stepsize;
    let mut tiledp_0 = &mut *(*tilec).data.offset(
      (y as OPJ_SIZE_T)
        .wrapping_mul(tile_w as libc::c_ulong)
        .wrapping_add(x as OPJ_SIZE_T) as isize,
    ) as *mut OPJ_INT32 as *mut OPJ_FLOAT32;
    j = 0 as libc::c_int as OPJ_UINT32;
    while j < cblk_h {
      let mut tiledp2 = tiledp_0;
      i = 0 as libc::c_int as OPJ_UINT32;
      while i < cblk_w {
        let mut tmp_1 = *datap as OPJ_FLOAT32 * stepsize_0;
        *tiledp2 = tmp_1;
        datap = datap.offset(1);
        tiledp2 = tiledp2.offset(1);
        i = i.wrapping_add(1)
      }
      tiledp_0 = tiledp_0.offset(tile_w as isize);
      j = j.wrapping_add(1)
    }
  }
  opj_free(job as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn opj_t1_decode_cblks(
  mut tcd: *mut opj_tcd_t,
  mut pret: *mut OPJ_BOOL,
  mut tilec: *mut opj_tcd_tilecomp_t,
  mut tccp: *mut opj_tccp_t,
  mut p_manager: *mut opj_event_mgr_t,
  mut p_manager_mutex: *mut opj_mutex_t,
  mut check_pterm: OPJ_BOOL,
) {
  let mut tp = (*tcd).thread_pool;
  let mut resno: OPJ_UINT32 = 0;
  let mut bandno: OPJ_UINT32 = 0;
  let mut precno: OPJ_UINT32 = 0;
  let mut cblkno: OPJ_UINT32 = 0;
  resno = 0 as libc::c_int as OPJ_UINT32;
  while resno < (*tilec).minimum_num_resolutions {
    let mut res: *mut opj_tcd_resolution_t =
      &mut *(*tilec).resolutions.offset(resno as isize) as *mut opj_tcd_resolution_t;
    bandno = 0 as libc::c_int as OPJ_UINT32;
    while bandno < (*res).numbands {
      let mut band: *mut opj_tcd_band_t =
        &mut *(*res).bands.as_mut_ptr().offset(bandno as isize) as *mut opj_tcd_band_t;
      precno = 0 as libc::c_int as OPJ_UINT32;
      while precno < (*res).pw.wrapping_mul((*res).ph) {
        let mut precinct: *mut opj_tcd_precinct_t =
          &mut *(*band).precincts.offset(precno as isize) as *mut opj_tcd_precinct_t;
        if opj_tcd_is_subband_area_of_interest(
          tcd,
          (*tilec).compno,
          resno,
          (*band).bandno,
          (*precinct).x0 as OPJ_UINT32,
          (*precinct).y0 as OPJ_UINT32,
          (*precinct).x1 as OPJ_UINT32,
          (*precinct).y1 as OPJ_UINT32,
        ) == 0
        {
          cblkno = 0 as libc::c_int as OPJ_UINT32;
          while cblkno < (*precinct).cw.wrapping_mul((*precinct).ch) {
            let mut cblk: *mut opj_tcd_cblk_dec_t =
              &mut *(*precinct).cblks.dec.offset(cblkno as isize) as *mut opj_tcd_cblk_dec_t;
            if !(*cblk).decoded_data.is_null() {
              opj_aligned_free((*cblk).decoded_data as *mut libc::c_void);
              (*cblk).decoded_data = 0 as *mut OPJ_INT32
            }
            cblkno = cblkno.wrapping_add(1)
          }
        } else {
          let mut current_block_34: u64;
          cblkno = 0 as libc::c_int as OPJ_UINT32;
          while cblkno < (*precinct).cw.wrapping_mul((*precinct).ch) {
            let mut cblk_0: *mut opj_tcd_cblk_dec_t =
              &mut *(*precinct).cblks.dec.offset(cblkno as isize) as *mut opj_tcd_cblk_dec_t;
            let mut job = 0 as *mut opj_t1_cblk_decode_processing_job_t;
            if opj_tcd_is_subband_area_of_interest(
              tcd,
              (*tilec).compno,
              resno,
              (*band).bandno,
              (*cblk_0).x0 as OPJ_UINT32,
              (*cblk_0).y0 as OPJ_UINT32,
              (*cblk_0).x1 as OPJ_UINT32,
              (*cblk_0).y1 as OPJ_UINT32,
            ) == 0
            {
              if !(*cblk_0).decoded_data.is_null() {
                opj_aligned_free((*cblk_0).decoded_data as *mut libc::c_void);
                (*cblk_0).decoded_data = 0 as *mut OPJ_INT32
              }
            } else {
              if (*tcd).whole_tile_decoding == 0 {
                let mut cblk_w = ((*cblk_0).x1 - (*cblk_0).x0) as OPJ_UINT32;
                let mut cblk_h = ((*cblk_0).y1 - (*cblk_0).y0) as OPJ_UINT32;
                if !(*cblk_0).decoded_data.is_null() {
                  current_block_34 = 2370887241019905314;
                } else if cblk_w == 0 as libc::c_int as libc::c_uint
                  || cblk_h == 0 as libc::c_int as libc::c_uint
                {
                  current_block_34 = 2370887241019905314;
                } else {
                  current_block_34 = 11913429853522160501;
                }
              } else {
                current_block_34 = 11913429853522160501;
              }
              match current_block_34 {
                2370887241019905314 => {}
                _ => {
                  job = opj_calloc(
                    1 as libc::c_int as size_t,
                    ::std::mem::size_of::<opj_t1_cblk_decode_processing_job_t>() as libc::c_ulong,
                  ) as *mut opj_t1_cblk_decode_processing_job_t;
                  if job.is_null() {
                    ::std::ptr::write_volatile(pret, 0 as libc::c_int);
                    return;
                  }
                  (*job).whole_tile_decoding = (*tcd).whole_tile_decoding;
                  (*job).resno = resno;
                  (*job).cblk = cblk_0;
                  (*job).band = band;
                  (*job).tilec = tilec;
                  (*job).tccp = tccp;
                  (*job).pret = pret;
                  (*job).p_manager_mutex = p_manager_mutex;
                  (*job).p_manager = p_manager;
                  (*job).check_pterm = check_pterm;
                  (*job).mustuse_cblkdatabuffer =
                    (opj_thread_pool_get_thread_count(tp) > 1 as libc::c_int) as libc::c_int;
                  opj_thread_pool_submit_job(
                    tp,
                    Some(
                      opj_t1_clbl_decode_processor
                        as unsafe extern "C" fn(_: *mut libc::c_void, _: *mut opj_tls_t) -> (),
                    ),
                    job as *mut libc::c_void,
                  );
                  if *pret == 0 {
                    return;
                  }
                }
              }
            }
            cblkno = cblkno.wrapping_add(1)
          }
        }
        precno = precno.wrapping_add(1)
        /* bandno */
        /* precno */
        /* cblkno */
      }
      bandno = bandno.wrapping_add(1)
    }
    resno = resno.wrapping_add(1)
  }
}
/* *
Decode 1 code-block
@param t1 T1 handle
@param cblk Code-block coding parameters
@param orient
@param roishift Region of interest shifting value
@param cblksty Code-block style
@param p_manager the event manager
@param p_manager_mutex mutex for the event manager
@param check_pterm whether PTERM correct termination should be checked
*/
unsafe extern "C" fn opj_t1_decode_cblk(
  mut t1: *mut opj_t1_t,
  mut cblk: *mut opj_tcd_cblk_dec_t,
  mut orient: OPJ_UINT32,
  mut roishift: OPJ_UINT32,
  mut cblksty: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
  mut p_manager_mutex: *mut opj_mutex_t,
  mut check_pterm: OPJ_BOOL,
) -> OPJ_BOOL {
  let mqc = &mut (*t1).mqc; /* MQC component */
  let mut bpno_plus_one: OPJ_INT32 = 0; /* BYPASS mode */
  let mut passtype: OPJ_UINT32 = 0;
  let mut segno: OPJ_UINT32 = 0;
  let mut passno: OPJ_UINT32 = 0;
  let mut cblkdata = 0 as *mut OPJ_BYTE;
  let mut cblkdataindex = 0 as libc::c_int as OPJ_UINT32;
  let mut type_0 = 0 as libc::c_int as OPJ_BYTE;
  let mut original_t1_data = 0 as *mut OPJ_INT32;
  (*mqc).lut_ctxno_zc_orient = lut_ctxno_zc
    .as_ptr()
    .offset((orient << 9 as libc::c_int) as isize);
  if opj_t1_allocate_buffers(
    t1,
    ((*cblk).x1 - (*cblk).x0) as OPJ_UINT32,
    ((*cblk).y1 - (*cblk).y0) as OPJ_UINT32,
  ) == 0
  {
    return 0 as libc::c_int;
  }
  bpno_plus_one = roishift.wrapping_add((*cblk).numbps) as OPJ_INT32;
  if bpno_plus_one >= 31 as libc::c_int {
    if !p_manager_mutex.is_null() {
      opj_mutex_lock(p_manager_mutex);
    }
    opj_event_msg(
      p_manager,
      2 as libc::c_int,
      b"opj_t1_decode_cblk(): unsupported bpno_plus_one = %d >= 31\n\x00" as *const u8
        as *const libc::c_char,
      bpno_plus_one,
    );
    if !p_manager_mutex.is_null() {
      opj_mutex_unlock(p_manager_mutex);
    }
    return 0 as libc::c_int;
  }
  passtype = 2 as libc::c_int as OPJ_UINT32;
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
  /* Even if we have a single chunk, in multi-threaded decoding */
  /* the insertion of our synthetic marker might potentially override */
  /* valid codestream of other codeblocks decoded in parallel. */
  if (*cblk).numchunks > 1 as libc::c_int as libc::c_uint || (*t1).mustuse_cblkdatabuffer != 0 {
    let mut i: OPJ_UINT32 = 0;
    let mut cblk_len: OPJ_UINT32 = 0;
    /* Compute whole codeblock length from chunk lengths */
    cblk_len = 0 as libc::c_int as OPJ_UINT32;
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < (*cblk).numchunks {
      cblk_len = (cblk_len as libc::c_uint).wrapping_add((*(*cblk).chunks.offset(i as isize)).len)
        as OPJ_UINT32 as OPJ_UINT32;
      i = i.wrapping_add(1)
    }
    /* Allocate temporary memory if needed */
    if cblk_len.wrapping_add(2 as libc::c_int as libc::c_uint) > (*t1).cblkdatabuffersize {
      cblkdata = opj_realloc(
        (*t1).cblkdatabuffer as *mut libc::c_void,
        cblk_len.wrapping_add(2 as libc::c_int as libc::c_uint) as size_t,
      ) as *mut OPJ_BYTE;
      if cblkdata.is_null() {
        return 0 as libc::c_int;
      }
      (*t1).cblkdatabuffer = cblkdata;
      memset(
        (*t1).cblkdatabuffer.offset(cblk_len as isize) as *mut libc::c_void,
        0 as libc::c_int,
        2 as libc::c_int as libc::c_ulong,
      );
      (*t1).cblkdatabuffersize = cblk_len.wrapping_add(2 as libc::c_int as libc::c_uint)
    }
    /* Concatenate all chunks */
    cblkdata = (*t1).cblkdatabuffer;
    cblk_len = 0 as libc::c_int as OPJ_UINT32;
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < (*cblk).numchunks {
      memcpy(
        cblkdata.offset(cblk_len as isize) as *mut libc::c_void,
        (*(*cblk).chunks.offset(i as isize)).data as *const libc::c_void,
        (*(*cblk).chunks.offset(i as isize)).len as libc::c_ulong,
      );
      cblk_len = (cblk_len as libc::c_uint).wrapping_add((*(*cblk).chunks.offset(i as isize)).len)
        as OPJ_UINT32 as OPJ_UINT32;
      i = i.wrapping_add(1)
    }
  } else if (*cblk).numchunks == 1 as libc::c_int as libc::c_uint {
    cblkdata = (*(*cblk).chunks.offset(0 as libc::c_int as isize)).data
  } else {
    /* Not sure if that can happen in practice, but avoid Coverity to */
    /* think we will dereference a null cblkdta pointer */
    return 1 as libc::c_int;
  }
  /* For subtile decoding, directly decode in the decoded_data buffer of */
  /* the code-block. Hack t1->data to point to it, and restore it later */
  if !(*cblk).decoded_data.is_null() {
    original_t1_data = (*t1).data;
    (*t1).data = (*cblk).decoded_data
  }
  segno = 0 as libc::c_int as OPJ_UINT32;
  while segno < (*cblk).real_num_segs {
    let mut seg: *mut opj_tcd_seg_t =
      &mut *(*cblk).segs.offset(segno as isize) as *mut opj_tcd_seg_t;
    /* BYPASS mode */
    type_0 = if bpno_plus_one <= (*cblk).numbps as OPJ_INT32 - 4 as libc::c_int
      && passtype < 2 as libc::c_int as libc::c_uint
      && cblksty & 0x1 as libc::c_int as libc::c_uint != 0
    {
      1 as libc::c_int
    } else {
      0 as libc::c_int
    } as OPJ_BYTE;
    if type_0 as libc::c_int == 1 as libc::c_int {
      opj_mqc_raw_init_dec(
        mqc,
        cblkdata.offset(cblkdataindex as isize),
        (*seg).len,
        2 as libc::c_int as OPJ_UINT32,
      );
    } else {
      opj_mqc_init_dec(
        mqc,
        cblkdata.offset(cblkdataindex as isize),
        (*seg).len,
        2 as libc::c_int as OPJ_UINT32,
      );
    }
    cblkdataindex =
      (cblkdataindex as libc::c_uint).wrapping_add((*seg).len);
    passno = 0 as libc::c_int as OPJ_UINT32;
    while passno < (*seg).real_num_passes && bpno_plus_one >= 1 as libc::c_int {
      match passtype {
        0 => {
          if type_0 as libc::c_int == 1 as libc::c_int {
            opj_t1_dec_sigpass_raw(&mut *t1, bpno_plus_one, cblksty as OPJ_INT32);
          } else {
            opj_t1_dec_sigpass_mqc(&mut *t1, bpno_plus_one, cblksty as OPJ_INT32);
          }
        }
        1 => {
          if type_0 as libc::c_int == 1 as libc::c_int {
            opj_t1_dec_refpass_raw(&mut *t1, bpno_plus_one);
          } else {
            opj_t1_dec_refpass_mqc(&mut *t1, bpno_plus_one);
          }
        }
        2 => {
          opj_t1_dec_clnpass(&mut *t1, bpno_plus_one, cblksty as OPJ_INT32);
        }
        _ => {}
      }
      if cblksty & 0x2 as libc::c_int as libc::c_uint != 0
        && type_0 as libc::c_int == 0 as libc::c_int
      {
        opj_mqc_resetstates(mqc);
        opj_mqc_setstate(
          mqc,
          (0 as libc::c_int
            + 9 as libc::c_int
            + 5 as libc::c_int
            + 3 as libc::c_int
            + 1 as libc::c_int) as OPJ_UINT32,
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
      passtype = passtype.wrapping_add(1);
      if passtype == 3 as libc::c_int as libc::c_uint {
        passtype = 0 as libc::c_int as OPJ_UINT32;
        bpno_plus_one -= 1
      }
      passno = passno.wrapping_add(1)
    }
    opq_mqc_finish_dec(mqc);
    segno = segno.wrapping_add(1)
  }
  if check_pterm != 0 {
    if (*mqc).bp.offset(2 as libc::c_int as isize) < (*mqc).end {
      if !p_manager_mutex.is_null() {
        opj_mutex_lock(p_manager_mutex);
      }
      opj_event_msg(
        p_manager,
        2 as libc::c_int,
        b"PTERM check failure: %d remaining bytes in code block (%d used / %d)\n\x00" as *const u8
          as *const libc::c_char,
        (*mqc).end.wrapping_offset_from((*mqc).bp) as libc::c_long as libc::c_int
          - 2 as libc::c_int,
        (*mqc).bp.wrapping_offset_from((*mqc).start) as libc::c_long as libc::c_int,
        (*mqc).end.wrapping_offset_from((*mqc).start) as libc::c_long as libc::c_int,
      );
      if !p_manager_mutex.is_null() {
        opj_mutex_unlock(p_manager_mutex);
      }
    } else if (*mqc).end_of_byte_stream_counter > 2 as libc::c_int as libc::c_uint {
      if !p_manager_mutex.is_null() {
        opj_mutex_lock(p_manager_mutex);
      }
      opj_event_msg(
        p_manager,
        2 as libc::c_int,
        b"PTERM check failure: %d synthetized 0xFF markers read\n\x00" as *const u8
          as *const libc::c_char,
        (*mqc).end_of_byte_stream_counter,
      );
      if !p_manager_mutex.is_null() {
        opj_mutex_unlock(p_manager_mutex);
      }
    }
  }
  /* Restore original t1->data is needed */
  if !(*cblk).decoded_data.is_null() {
    (*t1).data = original_t1_data
  }
  return 1 as libc::c_int;
}
/* * Procedure to deal with a asynchronous code-block encoding job.
 *
 * @param user_data Pointer to a opj_t1_cblk_encode_processing_job_t* structure
 * @param tls       TLS handle.
 */
unsafe extern "C" fn opj_t1_cblk_encode_processor(
  mut user_data: *mut libc::c_void,
  mut tls: *mut opj_tls_t,
) {
  let mut job = user_data as *mut opj_t1_cblk_encode_processing_job_t; /* OPJ_TRUE == T1 for encoding */
  let mut cblk = (*job).cblk; /* if (tccp->qmfbid == 0) */
  let mut band: *const opj_tcd_band_t = (*job).band;
  let mut tilec: *const opj_tcd_tilecomp_t = (*job).tilec;
  let mut tccp: *const opj_tccp_t = (*job).tccp;
  let resno = (*job).resno;
  let mut t1 = 0 as *mut opj_t1_t;
  let tile_w = ((*tilec).x1 - (*tilec).x0) as OPJ_UINT32;
  let mut tiledp = 0 as *mut OPJ_INT32;
  let mut cblk_w: OPJ_UINT32 = 0;
  let mut cblk_h: OPJ_UINT32 = 0;
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut x = (*cblk).x0 - (*band).x0;
  let mut y = (*cblk).y0 - (*band).y0;
  if *(*job).pret == 0 {
    opj_free(job as *mut libc::c_void);
    return;
  }
  t1 = opj_tls_get(tls, 0 as libc::c_int) as *mut opj_t1_t;
  if t1.is_null() {
    t1 = opj_t1_create(1 as libc::c_int);
    opj_tls_set(
      tls,
      0 as libc::c_int,
      t1 as *mut libc::c_void,
      Some(opj_t1_destroy_wrapper as unsafe extern "C" fn(_: *mut libc::c_void) -> ()),
    );
  }
  if (*band).bandno & 1 as libc::c_int as libc::c_uint != 0 {
    let mut pres: *mut opj_tcd_resolution_t = &mut *(*tilec)
      .resolutions
      .offset(resno.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize)
      as *mut opj_tcd_resolution_t;
    x += (*pres).x1 - (*pres).x0
  }
  if (*band).bandno & 2 as libc::c_int as libc::c_uint != 0 {
    let mut pres_0: *mut opj_tcd_resolution_t = &mut *(*tilec)
      .resolutions
      .offset(resno.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize)
      as *mut opj_tcd_resolution_t;
    y += (*pres_0).y1 - (*pres_0).y0
  }
  if opj_t1_allocate_buffers(
    t1,
    ((*cblk).x1 - (*cblk).x0) as OPJ_UINT32,
    ((*cblk).y1 - (*cblk).y0) as OPJ_UINT32,
  ) == 0
  {
    ::std::ptr::write_volatile((*job).pret, 0 as libc::c_int);
    opj_free(job as *mut libc::c_void);
    return;
  }
  cblk_w = (*t1).w;
  cblk_h = (*t1).h;
  tiledp = &mut *(*tilec).data.offset(
    (y as OPJ_SIZE_T)
      .wrapping_mul(tile_w as libc::c_ulong)
      .wrapping_add(x as OPJ_SIZE_T) as isize,
  ) as *mut OPJ_INT32;
  if (*tccp).qmfbid == 1 as libc::c_int as libc::c_uint {
    let mut tiledp_u = tiledp as *mut OPJ_UINT32;
    let mut t1data = (*t1).data as *mut OPJ_UINT32;
    /* Do multiplication on unsigned type, even if the
     * underlying type is signed, to avoid potential
     * int overflow on large value (the output will be
     * incorrect in such situation, but whatever...)
     * This assumes complement-to-2 signed integer
     * representation
     * Fixes https://github.com/uclouvain/openjpeg/issues/1053
     */
    j = 0 as libc::c_int as OPJ_UINT32;
    while j < cblk_h & !(3 as libc::c_uint) {
      i = 0 as libc::c_int as OPJ_UINT32;
      while i < cblk_w {
        *t1data.offset(0 as libc::c_int as isize) = *tiledp_u.offset(
          j.wrapping_add(0 as libc::c_int as libc::c_uint)
            .wrapping_mul(tile_w)
            .wrapping_add(i) as isize,
        ) << 7 as libc::c_int - 1 as libc::c_int;
        *t1data.offset(1 as libc::c_int as isize) = *tiledp_u.offset(
          j.wrapping_add(1 as libc::c_int as libc::c_uint)
            .wrapping_mul(tile_w)
            .wrapping_add(i) as isize,
        ) << 7 as libc::c_int - 1 as libc::c_int;
        *t1data.offset(2 as libc::c_int as isize) = *tiledp_u.offset(
          j.wrapping_add(2 as libc::c_int as libc::c_uint)
            .wrapping_mul(tile_w)
            .wrapping_add(i) as isize,
        ) << 7 as libc::c_int - 1 as libc::c_int;
        *t1data.offset(3 as libc::c_int as isize) = *tiledp_u.offset(
          j.wrapping_add(3 as libc::c_int as libc::c_uint)
            .wrapping_mul(tile_w)
            .wrapping_add(i) as isize,
        ) << 7 as libc::c_int - 1 as libc::c_int;
        t1data = t1data.offset(4 as libc::c_int as isize);
        i = i.wrapping_add(1)
      }
      j = (j as libc::c_uint).wrapping_add(4 as libc::c_int as libc::c_uint) as OPJ_UINT32
        as OPJ_UINT32
    }
    if j < cblk_h {
      i = 0 as libc::c_int as OPJ_UINT32;
      while i < cblk_w {
        let mut k: OPJ_UINT32 = 0;
        k = j;
        while k < cblk_h {
          *t1data.offset(0 as libc::c_int as isize) = *tiledp_u
            .offset(k.wrapping_mul(tile_w).wrapping_add(i) as isize)
            << 7 as libc::c_int - 1 as libc::c_int;
          t1data = t1data.offset(1);
          k = k.wrapping_add(1)
        }
        i = i.wrapping_add(1)
      }
    }
  } else {
    let mut tiledp_f = tiledp as *mut OPJ_FLOAT32;
    let mut t1data_0 = (*t1).data;
    /* Change from "natural" order to "zigzag" order of T1 passes */
    /* Change from "natural" order to "zigzag" order of T1 passes */
    j = 0 as libc::c_int as OPJ_UINT32; /* fixed_quality */
    while j < cblk_h & !(3 as libc::c_uint) {
      i = 0 as libc::c_int as OPJ_UINT32; /* compno  */
      while i < cblk_w {
        *t1data_0.offset(0 as libc::c_int as isize) = opj_lrintf(
          *tiledp_f.offset(
            j.wrapping_add(0 as libc::c_int as libc::c_uint)
              .wrapping_mul(tile_w)
              .wrapping_add(i) as isize,
          ) / (*band).stepsize
            * ((1 as libc::c_int) << 7 as libc::c_int - 1 as libc::c_int) as libc::c_float,
        ) as OPJ_INT32;
        *t1data_0.offset(1 as libc::c_int as isize) = opj_lrintf(
          *tiledp_f.offset(
            j.wrapping_add(1 as libc::c_int as libc::c_uint)
              .wrapping_mul(tile_w)
              .wrapping_add(i) as isize,
          ) / (*band).stepsize
            * ((1 as libc::c_int) << 7 as libc::c_int - 1 as libc::c_int) as libc::c_float,
        ) as OPJ_INT32;
        *t1data_0.offset(2 as libc::c_int as isize) = opj_lrintf(
          *tiledp_f.offset(
            j.wrapping_add(2 as libc::c_int as libc::c_uint)
              .wrapping_mul(tile_w)
              .wrapping_add(i) as isize,
          ) / (*band).stepsize
            * ((1 as libc::c_int) << 7 as libc::c_int - 1 as libc::c_int) as libc::c_float,
        ) as OPJ_INT32;
        *t1data_0.offset(3 as libc::c_int as isize) = opj_lrintf(
          *tiledp_f.offset(
            j.wrapping_add(3 as libc::c_int as libc::c_uint)
              .wrapping_mul(tile_w)
              .wrapping_add(i) as isize,
          ) / (*band).stepsize
            * ((1 as libc::c_int) << 7 as libc::c_int - 1 as libc::c_int) as libc::c_float,
        ) as OPJ_INT32;
        t1data_0 = t1data_0.offset(4 as libc::c_int as isize);
        i = i.wrapping_add(1)
      }
      j = (j as libc::c_uint).wrapping_add(4 as libc::c_int as libc::c_uint) as OPJ_UINT32
        as OPJ_UINT32
    }
    if j < cblk_h {
      i = 0 as libc::c_int as OPJ_UINT32;
      while i < cblk_w {
        let mut k_0: OPJ_UINT32 = 0;
        k_0 = j;
        while k_0 < cblk_h {
          *t1data_0.offset(0 as libc::c_int as isize) = opj_lrintf(
            *tiledp_f.offset(k_0.wrapping_mul(tile_w).wrapping_add(i) as isize) / (*band).stepsize
              * ((1 as libc::c_int) << 7 as libc::c_int - 1 as libc::c_int) as libc::c_float,
          ) as OPJ_INT32;
          t1data_0 = t1data_0.offset(1);
          k_0 = k_0.wrapping_add(1)
        }
        i = i.wrapping_add(1)
      }
    }
  }
  let mut cumwmsedec = opj_t1_encode_cblk(
    t1,
    cblk,
    (*band).bandno,
    (*job).compno,
    (*tilec)
      .numresolutions
      .wrapping_sub(1 as libc::c_int as libc::c_uint)
      .wrapping_sub(resno),
    (*tccp).qmfbid,
    (*band).stepsize as OPJ_FLOAT64,
    (*tccp).cblksty,
    (*(*job).tile).numcomps,
    (*job).mct_norms,
    (*job).mct_numcomps,
  );
  if !(*job).mutex.is_null() {
    opj_mutex_lock((*job).mutex);
  }
  (*(*job).tile).distotile += cumwmsedec;
  if !(*job).mutex.is_null() {
    opj_mutex_unlock((*job).mutex);
  }
  opj_free(job as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn opj_t1_encode_cblks(
  mut tcd: *mut opj_tcd_t,
  mut tile: *mut opj_tcd_tile_t,
  mut tcp: *mut opj_tcp_t,
  mut mct_norms: *const OPJ_FLOAT64,
  mut mct_numcomps: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut ret = 1 as libc::c_int;
  let mut tp = (*tcd).thread_pool;
  let mut compno: OPJ_UINT32 = 0;
  let mut resno: OPJ_UINT32 = 0;
  let mut bandno: OPJ_UINT32 = 0;
  let mut precno: OPJ_UINT32 = 0;
  let mut cblkno: OPJ_UINT32 = 0;
  let mut mutex = opj_mutex_create();
  (*tile).distotile = 0 as libc::c_int as OPJ_FLOAT64;
  compno = 0 as libc::c_int as OPJ_UINT32;
  's_19: while compno < (*tile).numcomps {
    let mut tilec: *mut opj_tcd_tilecomp_t =
      &mut *(*tile).comps.offset(compno as isize) as *mut opj_tcd_tilecomp_t;
    let mut tccp: *mut opj_tccp_t = &mut *(*tcp).tccps.offset(compno as isize) as *mut opj_tccp_t;
    resno = 0 as libc::c_int as OPJ_UINT32;
    while resno < (*tilec).numresolutions {
      let mut res: *mut opj_tcd_resolution_t =
        &mut *(*tilec).resolutions.offset(resno as isize) as *mut opj_tcd_resolution_t;
      bandno = 0 as libc::c_int as OPJ_UINT32;
      while bandno < (*res).numbands {
        let mut band: *mut opj_tcd_band_t =
          &mut *(*res).bands.as_mut_ptr().offset(bandno as isize) as *mut opj_tcd_band_t;
        /* resno  */
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
              let mut job = opj_calloc(
                1 as libc::c_int as size_t,
                ::std::mem::size_of::<opj_t1_cblk_encode_processing_job_t>() as libc::c_ulong,
              ) as *mut opj_t1_cblk_encode_processing_job_t;
              if job.is_null() {
                ::std::ptr::write_volatile(&mut ret as *mut OPJ_BOOL, 0 as libc::c_int);
                break 's_19;
              } else {
                (*job).compno = compno;
                (*job).tile = tile;
                (*job).resno = resno;
                (*job).cblk = cblk;
                (*job).band = band;
                (*job).tilec = tilec;
                (*job).tccp = tccp;
                (*job).mct_norms = mct_norms;
                (*job).mct_numcomps = mct_numcomps;
                (*job).pret = &mut ret;
                (*job).mutex = mutex;
                opj_thread_pool_submit_job(
                  tp,
                  Some(
                    opj_t1_cblk_encode_processor
                      as unsafe extern "C" fn(_: *mut libc::c_void, _: *mut opj_tls_t) -> (),
                  ),
                  job as *mut libc::c_void,
                );
                cblkno = cblkno.wrapping_add(1)
              }
            }
            precno = precno.wrapping_add(1)
            /* cblkno */
          }
        }
        bandno = bandno.wrapping_add(1)
      }
      resno = resno.wrapping_add(1)
    }
    compno = compno.wrapping_add(1)
  }
  opj_thread_pool_wait_completion((*tcd).thread_pool, 0 as libc::c_int);
  if !mutex.is_null() {
    opj_mutex_destroy(mutex);
  }
  return ret;
}
/* Returns whether the pass (bpno, passtype) is terminated */
unsafe extern "C" fn opj_t1_enc_is_term_pass(
  mut cblk: *mut opj_tcd_cblk_enc_t,
  mut cblksty: OPJ_UINT32,
  mut bpno: OPJ_INT32,
  mut passtype: OPJ_UINT32,
) -> libc::c_int {
  /* Is it the last cleanup pass ? */
  if passtype == 2 as libc::c_int as libc::c_uint && bpno == 0 as libc::c_int {
    return 1 as libc::c_int;
  }
  if cblksty & 0x4 as libc::c_int as libc::c_uint != 0 {
    return 1 as libc::c_int;
  }
  if cblksty & 0x1 as libc::c_int as libc::c_uint != 0 {
    /* For bypass arithmetic bypass, terminate the 4th cleanup pass */
    if bpno == (*cblk).numbps as OPJ_INT32 - 4 as libc::c_int
      && passtype == 2 as libc::c_int as libc::c_uint
    {
      return 1 as libc::c_int;
    }
    /* and beyond terminate all the magnitude refinement passes (in raw) */
    /* and cleanup passes (in MQC) */
    if bpno < (*cblk).numbps as OPJ_INT32 - 4 as libc::c_int
      && passtype > 0 as libc::c_int as libc::c_uint
    {
      return 1 as libc::c_int;
    }
  }
  return 0 as libc::c_int;
}
/* * Return "cumwmsedec" that should be used to increase tile->distotile */
/* * mod fixed_quality */
unsafe extern "C" fn opj_t1_encode_cblk(
  mut t1: *mut opj_t1_t,
  mut cblk: *mut opj_tcd_cblk_enc_t,
  mut orient: OPJ_UINT32,
  mut compno: OPJ_UINT32,
  mut level: OPJ_UINT32,
  mut qmfbid: OPJ_UINT32,
  mut stepsize: OPJ_FLOAT64,
  mut cblksty: OPJ_UINT32,
  mut numcomps: OPJ_UINT32,
  mut mct_norms: *const OPJ_FLOAT64,
  mut mct_numcomps: OPJ_UINT32,
) -> libc::c_double {
  let mut cumwmsedec = 0.0f64; /* MQC component */
  let mqc = &mut (*t1).mqc;
  let mut passno: OPJ_UINT32 = 0;
  let mut bpno: OPJ_INT32 = 0;
  let mut passtype: OPJ_UINT32 = 0;
  let mut nmsedec = 0 as libc::c_int;
  let mut max: OPJ_INT32 = 0;
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut type_0 = 0 as libc::c_int as OPJ_BYTE;
  let mut tempwmsedec: OPJ_FLOAT64 = 0.;
  let mut datap = 0 as *mut OPJ_INT32;
  (*mqc).lut_ctxno_zc_orient = lut_ctxno_zc
    .as_ptr()
    .offset((orient << 9 as libc::c_int) as isize);
  max = 0 as libc::c_int;
  datap = (*t1).data;
  j = 0 as libc::c_int as OPJ_UINT32;
  while j < (*t1).h {
    let w = (*t1).w;
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < w {
      let mut tmp = *datap;
      if tmp < 0 as libc::c_int {
        let mut tmp_unsigned: OPJ_UINT32 = 0;
        max = opj_int_max(max, -tmp);
        tmp_unsigned = if tmp >= 0 as libc::c_int {
          tmp as OPJ_UINT32
        } else {
          (-tmp as OPJ_UINT32) | 0x80000000 as libc::c_uint
        };
        memcpy(
          datap as *mut libc::c_void,
          &mut tmp_unsigned as *mut OPJ_UINT32 as *const libc::c_void,
          ::std::mem::size_of::<OPJ_INT32>() as libc::c_ulong,
        );
      } else {
        max = opj_int_max(max, tmp)
      }
      i = i.wrapping_add(1);
      datap = datap.offset(1)
    }
    j = j.wrapping_add(1)
  }
  (*cblk).numbps = if max != 0 {
    (opj_int_floorlog2(max) + 1 as libc::c_int - (7 as libc::c_int - 1 as libc::c_int))
      as OPJ_UINT32
  } else {
    0 as libc::c_int as libc::c_uint
  };
  if (*cblk).numbps == 0 as libc::c_int as libc::c_uint {
    (*cblk).totalpasses = 0 as libc::c_int as OPJ_UINT32;
    return cumwmsedec;
  }
  bpno = (*cblk)
    .numbps
    .wrapping_sub(1 as libc::c_int as libc::c_uint) as OPJ_INT32;
  passtype = 2 as libc::c_int as OPJ_UINT32;
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
  opj_mqc_init_enc(mqc, (*cblk).data);
  passno = 0 as libc::c_int as OPJ_UINT32;
  while bpno >= 0 as libc::c_int {
    let mut pass: *mut opj_tcd_pass_t =
      &mut *(*cblk).passes.offset(passno as isize) as *mut opj_tcd_pass_t;
    type_0 = if bpno < (*cblk).numbps as OPJ_INT32 - 4 as libc::c_int
      && passtype < 2 as libc::c_int as libc::c_uint
      && cblksty & 0x1 as libc::c_int as libc::c_uint != 0
    {
      1 as libc::c_int
    } else {
      0 as libc::c_int
    } as OPJ_BYTE;
    /* If the previous pass was terminating, we need to reset the encoder */
    if passno > 0 as libc::c_int as libc::c_uint
      && (*(*cblk)
        .passes
        .offset(passno.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
      .term() as libc::c_int
        != 0
    {
      if type_0 as libc::c_int == 1 as libc::c_int {
        opj_mqc_bypass_init_enc(mqc);
      } else {
        opj_mqc_restart_init_enc(mqc);
      }
    }
    match passtype {
      0 => {
        opj_t1_enc_sigpass(&mut *t1, bpno, &mut nmsedec, type_0, cblksty);
      }
      1 => {
        opj_t1_enc_refpass(&mut *t1, bpno, &mut nmsedec, type_0);
      }
      2 => {
        opj_t1_enc_clnpass(&mut *t1, bpno, &mut nmsedec, cblksty);
        /* code switch SEGMARK (i.e. SEGSYM) */
        if cblksty & 0x20 as libc::c_int as libc::c_uint != 0 {
          opj_mqc_segmark_enc(mqc);
        }
      }
      _ => {}
    }
    /* fixed_quality */
    tempwmsedec = opj_t1_getwmsedec(
      nmsedec,
      compno,
      level,
      orient,
      bpno,
      qmfbid,
      stepsize,
      numcomps,
      mct_norms,
      mct_numcomps,
    );
    cumwmsedec += tempwmsedec;
    (*pass).distortiondec = cumwmsedec;
    if opj_t1_enc_is_term_pass(cblk, cblksty, bpno, passtype) != 0 {
      /* If it is a terminated pass, terminate it */
      if type_0 as libc::c_int == 1 as libc::c_int {
        opj_mqc_bypass_flush_enc(
          mqc,
          (cblksty & 0x10 as libc::c_int as libc::c_uint) as OPJ_BOOL,
        );
      } else if cblksty & 0x10 as libc::c_int as libc::c_uint != 0 {
        opj_mqc_erterm_enc(mqc);
      } else {
        opj_mqc_flush(mqc);
      }
      (*pass).set_term(1 as libc::c_int as OPJ_BITFIELD);
      (*pass).rate = opj_mqc_numbytes(mqc)
    } else {
      /* Non terminated pass */
      let mut rate_extra_bytes: OPJ_UINT32 = 0;
      if type_0 as libc::c_int == 1 as libc::c_int {
        rate_extra_bytes = opj_mqc_bypass_get_extra_bytes(
          mqc,
          (cblksty & 0x10 as libc::c_int as libc::c_uint) as OPJ_BOOL,
        )
      } else {
        rate_extra_bytes = 3 as libc::c_int as OPJ_UINT32
      }
      (*pass).set_term(0 as libc::c_int as OPJ_BITFIELD);
      (*pass).rate = opj_mqc_numbytes(mqc).wrapping_add(rate_extra_bytes)
    }
    passtype = passtype.wrapping_add(1);
    if passtype == 3 as libc::c_int as libc::c_uint {
      passtype = 0 as libc::c_int as OPJ_UINT32;
      bpno -= 1
    }
    /* Code-switch "RESET" */
    if cblksty & 0x2 as libc::c_int as libc::c_uint != 0 {
      opj_mqc_reset_enc(mqc);
    }
    passno = passno.wrapping_add(1)
  }
  (*cblk).totalpasses = passno;
  if (*cblk).totalpasses != 0 {
    /* Make sure that pass rates are increasing */
    let mut last_pass_rate = opj_mqc_numbytes(mqc);
    passno = (*cblk).totalpasses;
    while passno > 0 as libc::c_int as libc::c_uint {
      passno = passno.wrapping_sub(1);
      let mut pass_0: *mut opj_tcd_pass_t =
        &mut *(*cblk).passes.offset(passno as isize) as *mut opj_tcd_pass_t;
      if (*pass_0).rate > last_pass_rate {
        (*pass_0).rate = last_pass_rate
      } else {
        last_pass_rate = (*pass_0).rate
      }
    }
  }
  passno = 0 as libc::c_int as OPJ_UINT32;
  while passno < (*cblk).totalpasses {
    let mut pass_1: *mut opj_tcd_pass_t =
      &mut *(*cblk).passes.offset(passno as isize) as *mut opj_tcd_pass_t;
    /* Prevent generation of FF as last data byte of a pass*/
    /* For terminating passes, the flushing procedure ensured this already */
    assert!((*pass_1).rate > 0 as libc::c_int as libc::c_uint);
    if *(*cblk).data.offset(
      (*pass_1)
        .rate
        .wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
    ) as libc::c_int
      == 0xff as libc::c_int
    {
      (*pass_1).rate = (*pass_1).rate.wrapping_sub(1)
    }
    (*pass_1).len = (*pass_1)
      .rate
      .wrapping_sub(if passno == 0 as libc::c_int as libc::c_uint {
        0 as libc::c_int as libc::c_uint
      } else {
        (*(*cblk)
          .passes
          .offset(passno.wrapping_sub(1 as libc::c_int as libc::c_uint) as isize))
        .rate
      });
    passno = passno.wrapping_add(1)
  }
  return cumwmsedec;
}
