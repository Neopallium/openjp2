use super::consts::*;
use super::dwt::*;
use super::event::*;
use super::ht_dec::*;
use super::math::*;
use super::mqc::*;
use super::openjpeg::*;
use super::par;
use super::t1_luts::*;
use super::tcd::*;
use crate::image::CoreImageComp;

use core::{
  cell::Cell,
  ops::AddAssign,
};

// The T1 passes walk several aliasing cursors over the same coefficient and flag buffers, so the
// backing stores hold `Cell`s: that keeps the cursors `Copy` and aliasable without any `unsafe`.

#[derive(Default)]
pub(crate) struct T1Flags {
  flags: Vec<Cell<opj_flag_t>>,
}

#[derive(Copy, Clone)]
pub(crate) struct FlagsPtr<'a> {
  flags: &'a [Cell<opj_flag_t>],
  idx: usize,
}

impl<'a> FlagsPtr<'a> {
  #[inline]
  fn cell(&self, offset: isize) -> &'a Cell<opj_flag_t> {
    &self.flags[self.idx.wrapping_add_signed(offset)]
  }

  /// Value at `offset` elements from the cursor.
  #[inline]
  pub fn at(&self, offset: isize) -> opj_flag_t {
    self.cell(offset).get()
  }

  /// Value under the cursor.
  #[inline]
  pub fn get(&self) -> opj_flag_t {
    self.cell(0).get()
  }

  #[inline]
  pub fn set_at(&self, offset: isize, value: opj_flag_t) {
    self.cell(offset).set(value)
  }

  #[inline]
  pub fn set(&self, value: opj_flag_t) {
    self.set_at(0, value)
  }

  #[inline]
  pub fn or_at(&self, offset: isize, value: opj_flag_t) {
    let cell = self.cell(offset);
    cell.set(cell.get() | value);
  }

  #[inline]
  pub fn or(&self, value: opj_flag_t) {
    self.or_at(0, value)
  }

  /// `*p &= !mask` under the cursor.
  #[inline]
  pub fn clear(&self, mask: opj_flag_t) {
    let cell = self.cell(0);
    cell.set(cell.get() & !mask);
  }

  pub fn offset(&self, offset: isize) -> Self {
    Self {
      flags: self.flags,
      idx: self.idx.wrapping_add_signed(offset),
    }
  }
}

impl AddAssign<usize> for FlagsPtr<'_> {
  fn add_assign(&mut self, n: usize) {
    self.idx += n;
  }
}

impl T1Flags {
  fn new() -> Self {
    Self::default()
  }

  pub fn resize(&mut self, len: usize) {
    if self.flags.len() != len {
      self.flags = vec![Cell::new(0); len];
    } else {
      for flag in &mut self.flags {
        flag.set(0);
      }
    }
  }

  pub fn as_slice(&self) -> &[Cell<opj_flag_t>] {
    &self.flags
  }

  pub fn offset(&self, offset: isize) -> FlagsPtr<'_> {
    FlagsPtr {
      flags: &self.flags,
      idx: offset as usize,
    }
  }
}

#[derive(Default)]
pub(crate) struct T1Data {
  data: Vec<Cell<i32>>,
}

#[derive(Copy, Clone)]
pub(crate) struct DataPtr<'a> {
  data: &'a [Cell<i32>],
  idx: usize,
}

impl<'a> DataPtr<'a> {
  #[inline]
  fn cell(&self, offset: isize) -> &'a Cell<i32> {
    &self.data[self.idx.wrapping_add_signed(offset)]
  }

  /// Value at `offset` elements from the cursor.
  #[inline]
  pub fn at(&self, offset: isize) -> i32 {
    self.cell(offset).get()
  }

  /// Value under the cursor.
  #[inline]
  pub fn get(&self) -> i32 {
    self.cell(0).get()
  }

  #[inline]
  pub fn set_at(&self, offset: isize, value: i32) {
    self.cell(offset).set(value)
  }

  #[inline]
  pub fn set(&self, value: i32) {
    self.set_at(0, value)
  }

  #[inline]
  pub fn add_at(&self, offset: isize, value: i32) {
    let cell = self.cell(offset);
    /* The C's `*ptr += value` wraps on hardware (UB in the standard); the
    refinement pass can reach i32 overflow with roishift + numbps == 30.
    wrapping_add keeps debug builds from panicking (review round 2, M7). */
    cell.set(cell.get().wrapping_add(value));
  }

  pub fn offset(&self, offset: isize) -> Self {
    Self {
      data: self.data,
      idx: self.idx.wrapping_add_signed(offset),
    }
  }
}

impl AddAssign<usize> for DataPtr<'_> {
  fn add_assign(&mut self, n: usize) {
    self.idx += n;
  }
}

impl T1Data {
  fn new() -> Self {
    Self::default()
  }

  pub fn resize(&mut self, len: usize) {
    if self.data.len() != len {
      self.data = vec![Cell::new(0); len];
    } else {
      for data in &mut self.data {
        data.set(0);
      }
    }
  }

  pub fn as_slice(&self) -> &[Cell<i32>] {
    &self.data
  }

  /// Raw view of the coefficient buffer, kept only for the tile-interop loops in
  /// `opj_t1_clbl_decode_processor` / `opj_t1_cblk_encode_processor` that still walk
  /// `TcdTileComp::data` as a raw pointer. Dropped in phase 3, when the tile component
  /// owns its samples as a `Vec`.
  pub fn offset(&self, offset: isize) -> DataPtr<'_> {
    DataPtr {
      data: &self.data,
      idx: offset as usize,
    }
  }
}

#[derive(Default)]
pub(crate) struct T1Coder {
  pub mqc: Mqc,
  pub data: T1Data,
  pub flags: T1Flags,
  pub w: u32,
  pub h: u32,
  pub datasize: u32,
  pub encoder: OPJ_BOOL,
  pub mustuse_cblkdatabuffer: OPJ_BOOL,
  pub cblkdatabuffer: Vec<u8>,
}
impl T1Coder {
  fn new() -> Self {
    Self::default()
  }

  pub fn grow_cblkdatabuffer(&mut self, size: usize) {
    if self.cblkdatabuffer.len() < size {
      self.cblkdatabuffer.resize(size, 0);
    }
  }

  pub fn cblkdatabuffer(&mut self) -> &mut [u8] {
    self.cblkdatabuffer.as_mut_slice()
  }

  pub fn cblkdatabuffer_ref(&self) -> &[u8] {
    self.cblkdatabuffer.as_slice()
  }

  /// Disjoint borrows of the MQ coder and the coefficient/flag cursors, so a coding pass can
  /// drive the coder while walking both buffers.
  fn split<'a>(&'a mut self, flags_stride: u32) -> (&'a mut Mqc, DataPtr<'a>, FlagsPtr<'a>) {
    self.split_with(flags_stride, None)
  }

  /// Like `split`, but the coefficient cursors walk `data` instead of the internal buffer.
  /// The decode path passes the code-block's owned `decoded_data` cells here, which keeps the
  /// pooled T1 workspace from holding any reference to the code-block.
  fn split_with<'a>(
    &'a mut self,
    flags_stride: u32,
    data: Option<&'a [Cell<i32>]>,
  ) -> (&'a mut Mqc, DataPtr<'a>, FlagsPtr<'a>) {
    let data = match data {
      Some(data) => DataPtr { data, idx: 0 },
      None => self.data.offset(0),
    };
    let flags = self.flags.offset(flags_stride as isize + 1);
    (&mut self.mqc, data, flags)
  }
}

/* Per-worker T1 workspaces, one pool per direction.  The decoder and the
encoder each own their pool so a decoded MQ sentinel can never leak into
an encode running later on the same worker - the decode->encode sharing
hazard is gone structurally (opj_mqc_init_enc still zeroes the sentinel
as defense).  Each slot keeps the C's buffer-reuse amortization; the slot
mutexes are uncontended (a worker runs its jobs sequentially). */
pub(crate) struct T1Pool {
  slots: Vec<std::sync::Mutex<Option<T1Coder>>>,
}

static T1_DECODE_POOL: std::sync::OnceLock<T1Pool> = std::sync::OnceLock::new();
static T1_ENCODE_POOL: std::sync::OnceLock<T1Pool> = std::sync::OnceLock::new();

/// The whole-tile decode write-back target: the tile buffer guarded by the
/// tile mutex, typed by the transform (5-3 writes i32 coefficients, 9-7
/// writes the dequantized f32 words).
pub(crate) enum TileDataMutex<'a> {
  I32(std::sync::Mutex<&'a mut [i32]>),
  F32(std::sync::Mutex<&'a mut [f32]>),
}

/// Shared read-only view of the tile buffer for the parallel t1 encode
/// jobs, typed by the transform.
#[derive(Copy, Clone)]
pub(crate) enum TileDataView<'a> {
  I32(&'a [i32]),
  F32(&'a [f32]),
}

impl T1Pool {
  fn new() -> Self {
    let n = crate::par::current_num_threads().max(1);
    T1Pool {
      slots: (0..n).map(|_| std::sync::Mutex::new(None)).collect(),
    }
  }

  fn with<R>(&self, f: impl FnOnce(&mut T1Coder) -> R) -> R {
    let idx = crate::par::current_thread_index().unwrap_or(0) % self.slots.len();
    /* Recover from a poisoned slot (a panic mid-codeblock): the partially
    written workspace is exactly what the old thread-local would have
    retained too, and the panic itself is caught at the API boundary. */
    let mut slot = self.slots[idx]
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let t1 = slot.get_or_insert_with(T1Coder::new);
    f(t1)
  }
}

#[inline]
fn opj_t1_setcurctx(mqc: &mut Mqc, ctxno: u8) {
  mqc.set_curctx(ctxno);
}

/* Macros to deal with signed integer with just MSB bit set for
 * negative values (smr = signed magnitude representation) */
//#define opj_smr_abs(x)  (((u32)(x)) & 0x7FFFFFFFU)
fn opj_smr_abs(x: i32) -> u32 {
  x as u32 & 0x7FFFFFFFu32
}

//#define opj_smr_sign(x) (((u32)(x)) >> 31)
fn opj_smr_sign(x: i32) -> u32 {
  x as u32 >> 31
}

//#define opj_to_smr(x)   ((x) >= 0 ? (u32)(x) : ((u32)(-x) | 0x80000000U))
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
fn opj_t1_getctxno_zc(mut mqc: &mut Mqc, f: u32) -> u8 {
  mqc.lut_ctxno_zc_orient[(f & T1_SIGMA_NEIGHBOURS) as usize]
}

#[inline]
fn opj_t1_getctxtno_sc_or_spb_index(mut f_x: u32, mut p_f_x: u32, mut n_f_x: u32, mut ci: u32) -> u32 {
  /*
    0 p_f_x T1_CHI_THIS           T1_LUT_SGN_W
    1 tf_x T1_SIGMA_1            T1_LUT_SIG_N
    2 n_f_x T1_CHI_THIS           T1_LUT_SGN_E
    3 tf_x T1_SIGMA_3            T1_LUT_SIG_W
    4  f_x T1_CHI_(THIS - 1)     T1_LUT_SGN_N
    5 tf_x T1_SIGMA_5            T1_LUT_SIG_E
    6  f_x T1_CHI_(THIS + 1)     T1_LUT_SGN_S
    7 tf_x T1_SIGMA_7            T1_LUT_SIG_S
  */

  let mut lu = f_x >> ci.wrapping_mul(3) & (T1_SIGMA_1 | T1_SIGMA_3 | T1_SIGMA_5 | T1_SIGMA_7);

  lu |= (p_f_x >> T1_CHI_THIS_I.wrapping_add(ci.wrapping_mul(3))) & (1 << 0);
  lu |= (n_f_x >> (T1_CHI_THIS_I - 2).wrapping_add(ci.wrapping_mul(3))) & (1 << 2);
  if ci == 0 {
    lu |= (f_x >> (T1_CHI_0_I - 4)) & (1 << 4);
  } else {
    lu |= (f_x >> (T1_CHI_1_I - 4).wrapping_add(ci.wrapping_sub(1).wrapping_mul(3))) & (1 << 4);
  }
  lu |= (f_x >> (T1_CHI_2_I - 6).wrapping_add(ci.wrapping_mul(3))) & (1 << 6);
  lu
}

#[inline]
fn opj_t1_getctxno_sc(mut lu: u32) -> u8 {
  LUT_CTXNO_SC[lu as usize]
}

#[inline]
fn opj_t1_getctxno_mag(mut f: u32) -> u8 {
  let tmp = if f & T1_SIGMA_NEIGHBOURS != 0 {
    T1_CTXNO_MAG + 1
  } else {
    T1_CTXNO_MAG
  };

  if f & T1_MU_0 != 0 {
    T1_CTXNO_MAG + 2
  } else {
    tmp
  }
}

#[inline]
fn opj_t1_getspb(mut lu: u32) -> u8 {
  LUT_SPB[lu as usize]
}

fn opj_t1_getnmsedec_sig(mut x: u32, mut bitpos: u32) -> i16 {
  if bitpos > 0 {
    return LUT_NMSEDEC_SIG[((x >> bitpos) & ((1 << T1_NMSEDEC_BITS) - 1)) as usize];
  }
  LUT_NMSEDEC_SIG0[(x & ((1 << T1_NMSEDEC_BITS) - 1)) as usize]
}

fn opj_t1_getnmsedec_ref(mut x: u32, mut bitpos: u32) -> i16 {
  if bitpos > 0 {
    return LUT_NMSEDEC_REF[((x >> bitpos) & ((1 << T1_NMSEDEC_BITS) - 1)) as usize];
  }
  LUT_NMSEDEC_REF0[(x & ((1 << T1_NMSEDEC_BITS) - 1)) as usize]
}

#[inline]
fn opj_t1_update_flags_macro(flags: &mut u32, flagsp: FlagsPtr, ci: u32, s: u32, stride: u32, vsc: u32) {
  /* east */
  flagsp.or_at(-1, T1_SIGMA_5 << 3_u32.wrapping_mul(ci));

  /* mark target as significant */
  *flags |= ((s << T1_CHI_1_I) | T1_SIGMA_4) << 3_u32.wrapping_mul(ci);

  /* west */
  flagsp.or_at(1, T1_SIGMA_3 << 3_u32.wrapping_mul(ci));

  /* north-west, north, north-east */
  if ci == 0 && vsc == 0 {
    let north = flagsp.offset(-(stride as isize));
    north.or((s << T1_CHI_5_I) | T1_SIGMA_16);
    north.or_at(-1, T1_SIGMA_17);
    north.or_at(1, T1_SIGMA_15);
  }

  /* south-west, south, south-east */
  if ci == 3 {
    let south = flagsp.offset(stride as isize);
    south.or((s << T1_CHI_0_I) | T1_SIGMA_1);
    south.or_at(-1, T1_SIGMA_2);
    south.or_at(1, T1_SIGMA_0);
  }
}

#[inline]
fn opj_t1_update_flags(flags: &mut u32, flagsp: FlagsPtr, ci: u32, s: u32, stride: u32, vsc: u32) {
  opj_t1_update_flags_macro(flags, flagsp, ci, s, stride, vsc);
}

/* *
Decode significant pass
*/

/* *
Encode significant pass
*/
#[inline(always)]
fn opj_t1_enc_sigpass_step_macro(
  mqc: &mut Mqc,
  w: u32,
  flags: &mut u32,
  flagsp: FlagsPtr,
  l_datap: DataPtr,
  bpno: i32,
  one: u32,
  nmsedec: &mut i32,
  type_0: u8,
  ci: u32,
  vsc: u32,
) {
  let mut v = 0;
  if (*flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == 0
    && (*flags & (T1_SIGMA_NEIGHBOURS << ci.wrapping_mul(3))) != 0
  {
    let ctxt1 = opj_t1_getctxno_zc(mqc, *flags >> ci.wrapping_mul(3));
    v = if opj_smr_abs(l_datap.get()) & one != 0 {
      1
    } else {
      0
    };
    log::debug!("   ctxt1={}", ctxt1);
    opj_t1_setcurctx(mqc, ctxt1);

    if type_0 == T1_TYPE_RAW {
      /* BYPASS/LAZY MODE */
      opj_mqc_bypass_enc_macro(mqc, v);
    } else {
      opj_mqc_encode_macro(mqc, v);
    }
    if v != 0 {
      let lu = opj_t1_getctxtno_sc_or_spb_index(*flags, flagsp.at(-1), flagsp.at(1), ci);
      let ctxt2 = opj_t1_getctxno_sc(lu);
      v = opj_smr_sign(l_datap.get());
      *nmsedec += opj_t1_getnmsedec_sig(opj_smr_abs(l_datap.get()), bpno as u32) as i32;
      log::debug!("   ctxt2={}", ctxt2);
      opj_t1_setcurctx(mqc, ctxt2);
      if type_0 == T1_TYPE_RAW {
        /* BYPASS/LAZY MODE */
        opj_mqc_bypass_enc_macro(mqc, v);
      } else {
        let spb = opj_t1_getspb(lu) as u32;
        log::debug!("   spb={}", spb);
        opj_mqc_encode_macro(mqc, v ^ spb);
      }
      opj_t1_update_flags(flags, flagsp, ci, v, w.wrapping_add(2), vsc);
    }
    *flags |= T1_PI_THIS << ci.wrapping_mul(3);
  }
}

#[inline]
fn opj_t1_dec_sigpass_step_raw(
  mqc: &mut Mqc,
  w: u32,
  flags: &mut u32,
  flagsp: FlagsPtr,
  datap: DataPtr,
  mut oneplushalf: i32,
  mut vsc: u32,
  mut ci: u32,
) {
  let mut v = 0;
  if (*flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == 0
    && (*flags & (T1_SIGMA_NEIGHBOURS << ci.wrapping_mul(3))) != 0
  {
    if opj_mqc_raw_decode(mqc) != 0 {
      v = opj_mqc_raw_decode(mqc);
      datap.set(if v != 0 { -oneplushalf } else { oneplushalf });
      opj_t1_update_flags(flags, flagsp, ci, v, w.wrapping_add(2), vsc);
    }
    *flags |= T1_PI_THIS << ci.wrapping_mul(3);
  }
}

#[inline(always)]
fn opj_t1_dec_sigpass_step_mqc_macro(
  flags: &mut u32,
  flagsp: FlagsPtr,
  flags_stride: u32,
  datap: DataPtr,
  data_stride: u32,
  ci: u32,
  mqc: &mut Mqc,
  mut v: u32,
  oneplushalf: i32,
  vsc: u32,
) {
  if (*flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == 0
    && (*flags & (T1_SIGMA_NEIGHBOURS << ci.wrapping_mul(3))) != 0
  {
    let ctxt1 = opj_t1_getctxno_zc(mqc, *flags >> ci.wrapping_mul(3));
    opj_t1_setcurctx(mqc, ctxt1);
    opj_mqc_decode_macro(&mut v, mqc);
    if v != 0 {
      let lu = opj_t1_getctxtno_sc_or_spb_index(*flags, flagsp.at(-1), flagsp.at(1), ci);
      let ctxt2 = opj_t1_getctxno_sc(lu);
      let spb = opj_t1_getspb(lu) as u32;
      opj_t1_setcurctx(mqc, ctxt2);
      opj_mqc_decode_macro(&mut v, mqc);
      v ^= spb;
      datap.set_at(
        ci.wrapping_mul(data_stride) as isize,
        if v != 0 { -oneplushalf } else { oneplushalf },
      );
      opj_t1_update_flags_macro(flags, flagsp, ci, v, flags_stride, vsc);
    }
    *flags |= T1_PI_THIS << ci.wrapping_mul(3);
  }
}

#[inline]
fn opj_t1_dec_sigpass_step_mqc(
  mut t1: &mut T1Coder,
  mut flagsp: FlagsPtr,
  mut datap: DataPtr,
  mut oneplushalf: i32,
  mut ci: u32,
  mut flags_stride: u32,
  mut vsc: u32,
) {
  let mut v = 0;
  let mut mqc = &mut t1.mqc; // MQC component
  let mut flags = flagsp.get();
  opj_t1_dec_sigpass_step_mqc_macro(&mut flags, flagsp, flags_stride, datap, 0, ci, mqc, v, oneplushalf, vsc);
  flagsp.set(flags);
}

// #define T1_FLAGS(x, y)
fn t1_flags(flags: &T1Flags, w: u32, x: u32, y: u32) -> FlagsPtr<'_> {
  flags.offset((x + 1).wrapping_add((y / 4 + 1).wrapping_mul(w.wrapping_add(2))) as isize)
}

/* *
Encode significant pass
*/
fn opj_t1_enc_sigpass(
  mut t1: &mut T1Coder,
  mut bpno: i32,
  mut nmsedec: &mut i32,
  mut type_0: u8,
  mut cblksty: u32,
) {
  let mut i = 0;
  let mut k = 0;
  /* The C shifts 1i32 by (bpno + T1_NMSEDEC_FRACBITS), which is undefined
  behaviour for shift counts >= 32 (x86 masks the count; that is the
  behaviour the release build has been matching).  wrapping_shl makes every
  build mode produce those same bits instead of panicking in debug. */
  let one = 1u32.wrapping_shl((bpno + T1_NMSEDEC_FRACBITS) as u32);
  let extra = 2;
  let (w, h) = (t1.w, t1.h);
  let (mqc, mut datap, mut f) = t1.split(w + 2);

  *nmsedec = 0;
  log::debug!("enc_sigpass: bpno={}", bpno);

  while k < h & !(0x03) {
    log::debug!(" k={}", k);
    i = 0;
    while i < w {
      log::debug!(" i={}", i);
      let mut flags = f.get();
      if flags == 0 {
        /* Nothing to do for any of the 4 data points */
      } else {
        opj_t1_enc_sigpass_step_macro(
          mqc,
          w,
          &mut flags,
          f,
          datap,
          bpno,
          one,
          nmsedec,
          type_0,
          0,
          cblksty & J2K_CCP_CBLKSTY_VSC,
        );
        opj_t1_enc_sigpass_step_macro(mqc, w, &mut flags, f, datap.offset(1), bpno, one, nmsedec, type_0, 1, 0);
        opj_t1_enc_sigpass_step_macro(mqc, w, &mut flags, f, datap.offset(2), bpno, one, nmsedec, type_0, 2, 0);
        opj_t1_enc_sigpass_step_macro(mqc, w, &mut flags, f, datap.offset(3), bpno, one, nmsedec, type_0, 3, 0);
        f.set(flags);
      }
      i += 1;
      f += 1;
      datap += 4
    }
    k += 4;
    f += extra;
  }

  if k < h {
    let mut j: u32 = 0;
    log::debug!(" k={}", k);
    i = 0;
    while i < w {
      log::debug!(" i={}", i);
      let mut flags = f.get();
      if flags == 0 {
        /* Nothing to do for any of the 4 data points */
        datap = datap.offset(h.wrapping_sub(k) as isize)
      } else {
        j = k;
        while j < h {
          opj_t1_enc_sigpass_step_macro(
            mqc,
            w,
            &mut flags,
            f,
            datap,
            bpno,
            one,
            nmsedec,
            type_0,
            j - k,
            (j == k && (cblksty & J2K_CCP_CBLKSTY_VSC) != 0) as u32,
          );
          j += 1;
          datap += 1
        }
        f.set(flags);
      }
      i += 1;
      f += 1
    }
  }
}

/* *
Decode significant pass
*/
fn opj_t1_dec_sigpass_raw(
  mut t1: &mut T1Coder,
  mut bpno: i32,
  mut cblksty: i32,
  data_src: Option<&[Cell<i32>]>,
) {
  let mut one = 0;
  let mut half = 0;
  let mut oneplushalf = 0;
  let mut i = 0;
  let mut j = 0;
  let mut k = 0;
  let (l_w, l_h) = (t1.w, t1.h);
  let (mqc, mut data, mut flagsp) = t1.split_with(l_w + 2, data_src);
  one = 1 << bpno;
  half = one >> 1;
  oneplushalf = one | half;

  k = 0;
  while k < l_h & !(0x3) {
    i = 0;
    while i < l_w {
      let mut flags = flagsp.get();
      if flags != 0 {
        opj_t1_dec_sigpass_step_raw(
          mqc,
          l_w,
          &mut flags,
          flagsp,
          data,
          oneplushalf,
          cblksty as u32 & J2K_CCP_CBLKSTY_VSC, /* vsc */
          0,
        );
        opj_t1_dec_sigpass_step_raw(
          mqc,
          l_w,
          &mut flags,
          flagsp,
          data.offset(l_w as isize),
          oneplushalf,
          0, /* vsc */
          1,
        );
        opj_t1_dec_sigpass_step_raw(
          mqc,
          l_w,
          &mut flags,
          flagsp,
          data.offset(2_u32.wrapping_mul(l_w) as isize),
          oneplushalf,
          0, /* vsc */
          2,
        );
        opj_t1_dec_sigpass_step_raw(
          mqc,
          l_w,
          &mut flags,
          flagsp,
          data.offset(3_u32.wrapping_mul(l_w) as isize),
          oneplushalf,
          0, /* vsc */
          3,
        );
        flagsp.set(flags);
      }
      i += 1;
      flagsp += 1;
      data = data.offset(1)
    }
    k += 4;
    flagsp += 2;
    data = data.offset(3_u32.wrapping_mul(l_w) as isize)
  }
  if k < l_h {
    i = 0;
    while i < l_w {
      j = 0;
      let mut flags = flagsp.get();
      while j < l_h.wrapping_sub(k) {
        opj_t1_dec_sigpass_step_raw(
          mqc,
          l_w,
          &mut flags,
          flagsp,
          data.offset(j.wrapping_mul(l_w) as isize),
          oneplushalf,
          cblksty as u32 & J2K_CCP_CBLKSTY_VSC,
          j,
        );
        j += 1
      }
      flagsp.set(flags);
      i += 1;
      flagsp += 1;
      data = data.offset(1)
    }
  }
}

fn opj_t1_dec_sigpass_mqc_internal(
  mut t1: &mut T1Coder,
  mut bpno: i32,
  vsc: u32,
  w: u32,
  h: u32,
  flags_stride: u32,
  data_src: Option<&[Cell<i32>]>,
) {
  let mut one = 0;
  let mut half = 0;
  let mut oneplushalf = 0;
  let mut i = 0;
  let mut j = 0;
  let mut k = 0;
  let (mqc, mut data, mut flagsp) = t1.split_with(flags_stride, data_src);
  let l_w = w;
  let mut v = 0;
  one = 1 << bpno;
  half = one >> 1;
  oneplushalf = one | half;

  k = 0;
  while k < h & !(0x03) {
    i = 0;
    while i < l_w {
      let mut flags = flagsp.get();
      if flags != 0 {
        opj_t1_dec_sigpass_step_mqc_macro(
          &mut flags,
          flagsp,
          flags_stride,
          data,
          l_w,
          0,
          mqc,
          v,
          oneplushalf,
          vsc,
        );
        opj_t1_dec_sigpass_step_mqc_macro(
          &mut flags,
          flagsp,
          flags_stride,
          data,
          l_w,
          1,
          mqc,
          v,
          oneplushalf,
          OPJ_FALSE,
        );
        opj_t1_dec_sigpass_step_mqc_macro(
          &mut flags,
          flagsp,
          flags_stride,
          data,
          l_w,
          2,
          mqc,
          v,
          oneplushalf,
          OPJ_FALSE,
        );
        opj_t1_dec_sigpass_step_mqc_macro(
          &mut flags,
          flagsp,
          flags_stride,
          data,
          l_w,
          3,
          mqc,
          v,
          oneplushalf,
          OPJ_FALSE,
        );
        flagsp.set(flags);
      }
      i += 1;
      data = data.offset(1);
      flagsp += 1
    }
    k += 4;
    data = data.offset(3_u32.wrapping_mul(l_w) as isize);
    flagsp += 2
  }
  if k < h {
    i = 0;
    while i < l_w {
      j = 0;
      let mut flags = flagsp.get();
      while j < h.wrapping_sub(k) {
        opj_t1_dec_sigpass_step_mqc_macro(
          &mut flags,
          flagsp,
          flags_stride,
          data.offset(j.wrapping_mul(l_w) as isize),
          0,
          j,
          mqc,
          v,
          oneplushalf,
          vsc,
        );
        j += 1
      }
      flagsp.set(flags);
      i += 1;
      data = data.offset(1);
      flagsp += 1
    }
  }
}

fn opj_t1_dec_sigpass_mqc_64x64_novsc(t1: &mut T1Coder, bpno: i32, data_src: Option<&[Cell<i32>]>) {
  opj_t1_dec_sigpass_mqc_internal(t1, bpno, OPJ_FALSE, 64, 64, 66, data_src);
}

fn opj_t1_dec_sigpass_mqc_64x64_vsc(t1: &mut T1Coder, bpno: i32, data_src: Option<&[Cell<i32>]>) {
  opj_t1_dec_sigpass_mqc_internal(t1, bpno, OPJ_TRUE, 64, 64, 66, data_src);
}

fn opj_t1_dec_sigpass_mqc_generic_novsc(
  t1: &mut T1Coder,
  bpno: i32,
  data_src: Option<&[Cell<i32>]>,
) {
  opj_t1_dec_sigpass_mqc_internal(t1, bpno, OPJ_FALSE, t1.w, t1.h, t1.w + 2, data_src);
}

fn opj_t1_dec_sigpass_mqc_generic_vsc(t1: &mut T1Coder, bpno: i32, data_src: Option<&[Cell<i32>]>) {
  opj_t1_dec_sigpass_mqc_internal(t1, bpno, OPJ_TRUE, t1.w, t1.h, t1.w + 2, data_src);
}

fn opj_t1_dec_sigpass_mqc(
  mut t1: &mut T1Coder,
  mut bpno: i32,
  mut cblksty: i32,
  data_src: Option<&[Cell<i32>]>,
) {
  if t1.w == 64 && t1.h == 64 {
    if cblksty as u32 & J2K_CCP_CBLKSTY_VSC != 0 {
      opj_t1_dec_sigpass_mqc_64x64_vsc(t1, bpno, data_src);
    } else {
      opj_t1_dec_sigpass_mqc_64x64_novsc(t1, bpno, data_src);
    }
  } else if cblksty as u32 & J2K_CCP_CBLKSTY_VSC != 0 {
    opj_t1_dec_sigpass_mqc_generic_vsc(t1, bpno, data_src);
  } else {
    opj_t1_dec_sigpass_mqc_generic_novsc(t1, bpno, data_src);
  }
}

/* *
Decode refinement pass
*/

/**
Encode refinement pass step
*/
#[inline(always)]
fn opj_t1_enc_refpass_step_macro(
  mqc: &mut Mqc,
  flagsp: FlagsPtr,
  l_datap: DataPtr,
  bpno: i32,
  one: u32,
  nmsedec: &mut i32,
  type_0: u8,
  ci: u32,
) {
  let mut v: u32 = 0;
  let flags = flagsp.get();
  if (flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3)))
    == (T1_SIGMA_THIS << ci.wrapping_mul(3))
  {
    let shift_flags = flags >> ci.wrapping_mul(3);
    let ctxt = opj_t1_getctxno_mag(shift_flags);
    let abs_data = opj_smr_abs(l_datap.get());
    *nmsedec += opj_t1_getnmsedec_ref(abs_data, bpno as u32) as i32;
    v = if abs_data & one != 0 { 1 } else { 0 };
    log::debug!("   ctxt={}", ctxt);
    opj_t1_setcurctx(mqc, ctxt);

    if type_0 == T1_TYPE_RAW {
      /* BYPASS/LAZY MODE */
      opj_mqc_bypass_enc_macro(mqc, v);
    } else {
      opj_mqc_encode_macro(mqc, v);
    }
    flagsp.or(T1_MU_THIS << ci.wrapping_mul(3));
  }
}

#[inline]
fn opj_t1_dec_refpass_step_raw(
  mqc: &mut Mqc,
  flagsp: FlagsPtr,
  datap: DataPtr,
  mut poshalf: i32,
  mut ci: u32,
) {
  let mut v = 0;

  if (flagsp.get() & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3)))
    == (T1_SIGMA_THIS << ci.wrapping_mul(3))
  {
    v = opj_mqc_raw_decode(mqc);
    datap.add_at(
      0,
      if v ^ (datap.get() < 0) as u32 != 0 {
        poshalf
      } else {
        -poshalf
      },
    );
    flagsp.or(T1_MU_THIS << ci.wrapping_mul(3));
  }
}

fn opj_t1_dec_refpass_step_mqc_macro(
  flagsp: FlagsPtr,
  datap: DataPtr,
  mut data_stride: u32,
  mut ci: u32,
  mut mqc: &mut Mqc,
  mut v: &mut u32,
  mut poshalf: i32,
) {
  let flags = flagsp.get();
  if (flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3)))
    == (T1_SIGMA_THIS << ci.wrapping_mul(3))
  {
    let ctxt = opj_t1_getctxno_mag(flags >> ci.wrapping_mul(3));
    opj_t1_setcurctx(mqc, ctxt);
    opj_mqc_decode_macro(v, mqc);
    let datap = datap.offset(ci.wrapping_mul(data_stride) as isize);
    datap.add_at(
      0,
      if *v ^ (datap.get() < 0) as u32 != 0 {
        poshalf
      } else {
        -poshalf
      },
    );
    flagsp.or(T1_MU_THIS << ci.wrapping_mul(3));
  }
}

#[inline]
fn opj_t1_dec_refpass_step_mqc(
  mut t1: &mut T1Coder,
  mut flagsp: FlagsPtr,
  mut datap: DataPtr,
  mut poshalf: i32,
  mut ci: u32,
) {
  let mut v = 0;

  let mut mqc = &mut t1.mqc; /* MQC component */
  opj_t1_dec_refpass_step_mqc_macro(flagsp, datap, 0, ci, mqc, &mut v, poshalf);
}

/* *
Encode refinement pass
*/
fn opj_t1_enc_refpass(mut t1: &mut T1Coder, mut bpno: i32, mut nmsedec: &mut i32, mut type_0: u8) {
  let mut i: u32 = 0;
  let mut k: u32 = 0;
  /* See opj_t1_enc_sigpass: wrapping_shl matches the C's masked shift. */
  let one = 1u32.wrapping_shl((bpno + T1_NMSEDEC_FRACBITS) as u32);
  let extra = 2;
  let (w, h) = (t1.w, t1.h);
  let (mqc, mut datap, mut f) = t1.split(w + 2);

  *nmsedec = 0;
  log::debug!("enc_refpass: bpno={}", bpno);

  while k < h & !(0x03) {
    log::debug!(" k={}", k);
    i = 0;
    while i < w {
      let flags = f.get();
      log::debug!(" i={}", i);
      if (flags & (T1_SIGMA_4 | T1_SIGMA_7 | T1_SIGMA_10 | T1_SIGMA_13)) == 0 {
        /* none significant */
      } else if (flags & (T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3))
        == (T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3)
      {
        /* all processed by sigpass */
      } else {
        opj_t1_enc_refpass_step_macro(mqc, f, datap, bpno, one, nmsedec, type_0, 0);
        opj_t1_enc_refpass_step_macro(mqc, f, datap.offset(1), bpno, one, nmsedec, type_0, 1);
        opj_t1_enc_refpass_step_macro(mqc, f, datap.offset(2), bpno, one, nmsedec, type_0, 2);
        opj_t1_enc_refpass_step_macro(mqc, f, datap.offset(3), bpno, one, nmsedec, type_0, 3);
      }
      i += 1;
      f += 1;
      datap += 4
    }
    k += 4;
    f += extra;
  }

  if k < h {
    let mut j: u32 = 0;
    let remaining_lines = h - k;
    log::debug!(" k={}", k);
    i = 0;
    while i < w {
      log::debug!(" i={}", i);
      if (f.get() & (T1_SIGMA_4 | T1_SIGMA_7 | T1_SIGMA_10 | T1_SIGMA_13)) == 0 {
        /* none significant */
        datap = datap.offset(remaining_lines as isize);
      } else {
        j = 0;
        while j < remaining_lines {
          opj_t1_enc_refpass_step_macro(mqc, f, datap, bpno, one, nmsedec, type_0, j);
          j += 1;
          datap += 1
        }
      }
      i += 1;
      f += 1
    }
  }
}

/* *
Decode refinement pass
*/
fn opj_t1_dec_refpass_raw(mut t1: &mut T1Coder, mut bpno: i32, data_src: Option<&[Cell<i32>]>) {
  let mut one: i32 = 0;
  let mut poshalf: i32 = 0;
  let mut i: u32 = 0;
  let mut j: u32 = 0;
  let mut k: u32 = 0;
  let (l_w, l_h) = (t1.w, t1.h);
  let (mqc, mut data, mut flagsp) = t1.split_with(l_w + 2, data_src);
  one = 1 << bpno;
  poshalf = one >> 1;
  k = 0;
  while k < l_h & !(0x03) {
    i = 0;
    while i < l_w {
      if flagsp.get() != 0 {
        opj_t1_dec_refpass_step_raw(mqc, flagsp, data, poshalf, 0);
        opj_t1_dec_refpass_step_raw(mqc, flagsp, data.offset(l_w as isize), poshalf, 1);
        opj_t1_dec_refpass_step_raw(
          mqc,
          flagsp,
          data.offset(2_u32.wrapping_mul(l_w) as isize),
          poshalf,
          2,
        );
        opj_t1_dec_refpass_step_raw(
          mqc,
          flagsp,
          data.offset(3_u32.wrapping_mul(l_w) as isize),
          poshalf,
          3,
        );
      }
      i += 1;
      flagsp += 1;
      data = data.offset(1)
    }
    k += 4;
    flagsp += 2;
    data = data.offset(3_u32.wrapping_mul(l_w) as isize)
  }
  if k < l_h {
    i = 0;
    while i < l_w {
      j = 0;
      while j < l_h.wrapping_sub(k) {
        opj_t1_dec_refpass_step_raw(
          mqc,
          flagsp,
          data.offset(j.wrapping_mul(l_w) as isize),
          poshalf,
          j,
        );
        j += 1
      }
      i += 1;
      flagsp += 1;
      data = data.offset(1)
    }
  }
}

fn opj_t1_dec_refpass_mqc_internal(
  mut t1: &mut T1Coder,
  mut bpno: i32,
  w: u32,
  h: u32,
  flags_stride: u32,
  data_src: Option<&[Cell<i32>]>,
) {
  let mut one = 0;
  let mut poshalf = 0;
  let mut i = 0;
  let mut j = 0;
  let mut k = 0;
  let (mqc, mut data, mut flagsp) = t1.split_with(flags_stride, data_src);
  let l_w = w;
  let mut v = 0;
  one = 1 << bpno;
  poshalf = one >> 1;

  k = 0;
  while k < h & !(0x03) {
    i = 0;
    while i < l_w {
      if flagsp.get() != 0 {
        opj_t1_dec_refpass_step_mqc_macro(flagsp, data, l_w, 0, mqc, &mut v, poshalf);
        opj_t1_dec_refpass_step_mqc_macro(flagsp, data, l_w, 1, mqc, &mut v, poshalf);
        opj_t1_dec_refpass_step_mqc_macro(flagsp, data, l_w, 2, mqc, &mut v, poshalf);
        opj_t1_dec_refpass_step_mqc_macro(flagsp, data, l_w, 3, mqc, &mut v, poshalf);
      }
      i += 1;
      data = data.offset(1);
      flagsp += 1
    }
    k += 4;
    data = data.offset(3_u32.wrapping_mul(l_w) as isize);
    flagsp += 2
  }
  if k < h {
    i = 0;
    while i < l_w {
      j = 0;
      while j < h.wrapping_sub(k) {
        opj_t1_dec_refpass_step_mqc_macro(
          flagsp,
          data.offset(j.wrapping_mul(l_w) as isize),
          0,
          j,
          mqc,
          &mut v,
          poshalf,
        );
        j += 1
      }
      i += 1;
      data = data.offset(1);
      flagsp += 1
    }
  }
}

fn opj_t1_dec_refpass_mqc_64x64(
  mut t1: &mut T1Coder,
  mut bpno: i32,
  data_src: Option<&[Cell<i32>]>,
) {
  opj_t1_dec_refpass_mqc_internal(t1, bpno, 64, 64, 66, data_src);
}

fn opj_t1_dec_refpass_mqc_generic(
  mut t1: &mut T1Coder,
  mut bpno: i32,
  data_src: Option<&[Cell<i32>]>,
) {
  opj_t1_dec_refpass_mqc_internal(t1, bpno, t1.w, t1.h, t1.w + 2, data_src);
}

fn opj_t1_dec_refpass_mqc(mut t1: &mut T1Coder, mut bpno: i32, data_src: Option<&[Cell<i32>]>) {
  if t1.w == 64 && t1.h == 64 {
    opj_t1_dec_refpass_mqc_64x64(t1, bpno, data_src);
  } else {
    opj_t1_dec_refpass_mqc_generic(t1, bpno, data_src);
  };
}

/**
Encode clean-up pass step
*/
#[inline(always)]
fn opj_t1_enc_clnpass_step_macro(
  mqc: &mut Mqc,
  w: u32,
  flags: &mut u32,
  flagsp: FlagsPtr,
  mut l_datap: DataPtr,
  bpno: i32,
  one: u32,
  nmsedec: &mut i32,
  agg: u8,
  runlen: u32,
  lim: u32,
  cblksty: u32,
) {
  const CHECK: opj_flag_t =
    T1_SIGMA_4 | T1_SIGMA_7 | T1_SIGMA_10 | T1_SIGMA_13 | T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3;
  {
    let mut v = 0;
    if (*flags & CHECK) == CHECK {
      if runlen == 0 {
        *flags &= !(T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3);
      } else if runlen == 1 {
        *flags &= !(T1_PI_1 | T1_PI_2 | T1_PI_3);
      } else if runlen == 2 {
        *flags &= !(T1_PI_2 | T1_PI_3);
      } else if runlen == 3 {
        *flags &= !T1_PI_3;
      }
    } else {
      for ci in runlen..lim {
        let mut goto_partial = false;
        if agg != 0 && ci == runlen {
          goto_partial = true;
        } else if (*flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == 0 {
          let ctxt1 = opj_t1_getctxno_zc(mqc, *flags >> ci.wrapping_mul(3));
          log::debug!("   ctxt1={}", ctxt1);
          opj_t1_setcurctx(mqc, ctxt1);
          v = if opj_smr_abs(l_datap.get()) & one != 0 {
            1
          } else {
            0
          };
          opj_mqc_encode_macro(mqc, v);
          if v != 0 {
            goto_partial = true;
          }
        }
        if goto_partial {
          let lu = opj_t1_getctxtno_sc_or_spb_index(*flags, flagsp.at(-1), flagsp.at(1), ci);
          *nmsedec += opj_t1_getnmsedec_sig(opj_smr_abs(l_datap.get()), bpno as u32) as i32;
          let ctxt2 = opj_t1_getctxno_sc(lu);
          log::debug!("   ctxt2={}", ctxt2);
          opj_t1_setcurctx(mqc, ctxt2);

          v = opj_smr_sign(l_datap.get());
          let spb = opj_t1_getspb(lu);
          log::debug!("   spb={}", spb);
          opj_mqc_encode_macro(mqc, v ^ spb as u32);
          let vsc = if (cblksty & J2K_CCP_CBLKSTY_VSC) != 0 && ci == 0 {
            1
          } else {
            0
          };
          opj_t1_update_flags(flags, flagsp, ci, v, w + 2, vsc);
        }
        *flags &= !(T1_PI_THIS << ci.wrapping_mul(3));
        l_datap = l_datap.offset(1);
      }
    }
  }
}

#[inline(always)]
fn opj_t1_dec_clnpass_step_macro(
  check_flags: bool,
  partial: bool,
  flags: &mut u32,
  flagsp: FlagsPtr,
  flags_stride: u32,
  datap: DataPtr,
  data_stride: u32,
  ci: u32,
  mqc: &mut Mqc,
  mut v: u32,
  oneplushalf: i32,
  vsc: u32,
) {
  if !check_flags || (*flags & ((T1_SIGMA_THIS | T1_PI_THIS) << ci.wrapping_mul(3))) == 0 {
    if !partial {
      let ctxt1 = opj_t1_getctxno_zc(mqc, *flags >> ci.wrapping_mul(3));
      opj_t1_setcurctx(mqc, ctxt1);
      opj_mqc_decode_macro(&mut v, mqc);
      if v == 0 {
        return;
      }
    }
    let lu = opj_t1_getctxtno_sc_or_spb_index(*flags, flagsp.at(-1), flagsp.at(1), ci);
    opj_t1_setcurctx(mqc, opj_t1_getctxno_sc(lu));
    opj_mqc_decode_macro(&mut v, mqc);
    v ^= opj_t1_getspb(lu) as u32;
    datap.set_at(
      ci.wrapping_mul(data_stride) as isize,
      if v != 0 { -oneplushalf } else { oneplushalf },
    );
    opj_t1_update_flags_macro(flags, flagsp, ci, v, flags_stride, vsc);
  }
}

fn opj_t1_dec_clnpass_step(
  mqc: &mut Mqc,
  w: u32,
  flagsp: FlagsPtr,
  datap: DataPtr,
  mut oneplushalf: i32,
  mut ci: u32,
  mut vsc: u32,
) {
  let mut v = 0;
  let mut flags = flagsp.get();

  opj_t1_dec_clnpass_step_macro(
    true,
    false,
    &mut flags,
    flagsp,
    w + 2,
    datap,
    0,
    ci,
    mqc,
    v,
    oneplushalf,
    vsc,
  );
  flagsp.set(flags);
}

/* *
Encode clean-up pass
*/
fn opj_t1_enc_clnpass(
  mut t1: &mut T1Coder,
  mut bpno: i32,
  mut nmsedec: &mut i32,
  mut cblksty: u32,
) {
  let mut i: u32 = 0;
  let mut k: u32 = 0;
  /* See opj_t1_enc_sigpass: wrapping_shl matches the C's masked shift. */
  let one = 1u32.wrapping_shl((bpno + T1_NMSEDEC_FRACBITS) as u32);
  let extra = 2;
  let (w, h) = (t1.w, t1.h);
  let (mqc, mut datap, mut f) = t1.split(w + 2);

  *nmsedec = 0;
  log::debug!("enc_clnpass: bpno={}", bpno);
  k = 0;
  while k < (h & !0x03) {
    log::debug!(" k={}", k);
    i = 0;
    while i < w {
      log::debug!(" i={}", i);
      let mut agg = 0;
      let mut runlen = 0u32;
      agg = (f.get() == 0) as u8;
      log::debug!("   agg={}", agg);
      'cln: {
        if agg != 0 {
          runlen = 0;
          while runlen < 4 {
            if (opj_smr_abs(datap.get()) & one) != 0 {
              break;
            }
            runlen = runlen.wrapping_add(1);
            datap += 1
          }
          opj_t1_setcurctx(mqc, T1_CTXNO_AGG);
          opj_mqc_encode_macro(mqc, (runlen != 4) as u32);
          if runlen == 4 {
            break 'cln;
          }
          opj_t1_setcurctx(mqc, T1_CTXNO_UNI);
          opj_mqc_encode_macro(mqc, runlen >> 1);
          opj_mqc_encode_macro(mqc, runlen & 1);
        } else {
          runlen = 0;
        }
        {
          let mut flags = f.get();
          opj_t1_enc_clnpass_step_macro(
            mqc, w, &mut flags, f, datap, bpno, one, nmsedec, agg, runlen, 4, cblksty,
          );
          f.set(flags);
        }
        datap = datap.offset(4_u32.wrapping_sub(runlen) as isize);
      }
      i += 1;
      f += 1
    }
    k += 4;
    f += extra;
  }

  if k < h {
    let agg = 0;
    let runlen = 0;
    log::debug!(" k={}", k);
    i = 0;
    while i < w {
      log::debug!(" i={}", i);
      log::debug!("  agg={}", agg);
      {
        let mut flags = f.get();
        opj_t1_enc_clnpass_step_macro(
          mqc,
          w,
          &mut flags,
          f,
          datap,
          bpno,
          one,
          nmsedec,
          agg,
          runlen,
          h - k,
          cblksty,
        );
        f.set(flags);
      }
      datap = datap.offset((h - k) as isize);
      i += 1;
      f += 1
    }
  }
}

fn opj_t1_dec_clnpass_internal(
  t1: &mut T1Coder,
  bpno: i32,
  vsc: bool,
  w: u32,
  h: u32,
  flags_stride: u32,
  data_src: Option<&[Cell<i32>]>,
) {
  let mut one = 0;
  let mut half = 0;
  let mut oneplushalf = 0;
  let mut runlen = 0u32;
  let mut i = 0;
  let mut j = 0;
  let mut k = 0;
  let (mqc, mut data, mut flagsp) = t1.split_with(flags_stride, data_src);
  let l_w = w;
  let mut v = 0u32;
  one = 1 << bpno;
  half = one >> 1;
  oneplushalf = one | half;

  k = 0;
  while k < (h & !3) {
    i = 0;
    while i < l_w {
      let mut flags = flagsp.get();
      if flags == 0 {
        let mut partial = true;
        opj_t1_setcurctx(mqc, T1_CTXNO_AGG);
        opj_mqc_decode_macro(&mut v, mqc);
        if v == 0 {
          // continue;
        } else {
          opj_t1_setcurctx(mqc, T1_CTXNO_UNI);
          opj_mqc_decode_macro(&mut runlen, mqc);
          opj_mqc_decode_macro(&mut v, mqc);
          runlen = (runlen << 1) | v;
          if runlen == 0 {
            opj_t1_dec_clnpass_step_macro(
              false,
              true,
              &mut flags,
              flagsp,
              flags_stride,
              data,
              l_w,
              0,
              mqc,
              v,
              oneplushalf,
              vsc as u32,
            );
            partial = false;
            /* FALLTHRU */
          }
          if runlen <= 1 {
            opj_t1_dec_clnpass_step_macro(
              false,
              partial,
              &mut flags,
              flagsp,
              flags_stride,
              data,
              l_w,
              1,
              mqc,
              v,
              oneplushalf,
              false as u32,
            );
            partial = false;
            /* FALLTHRU */
          }
          if runlen <= 2 {
            opj_t1_dec_clnpass_step_macro(
              false,
              partial,
              &mut flags,
              flagsp,
              flags_stride,
              data,
              l_w,
              2,
              mqc,
              v,
              oneplushalf,
              false as u32,
            );
            partial = false;
            /* FALLTHRU */
          }
          if runlen <= 3 {
            opj_t1_dec_clnpass_step_macro(
              false,
              partial,
              &mut flags,
              flagsp,
              flags_stride,
              data,
              l_w,
              3,
              mqc,
              v,
              oneplushalf,
              false as u32,
            );
          }
          flags &= !(T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3);
        }
      } else {
        opj_t1_dec_clnpass_step_macro(
          true,
          false,
          &mut flags,
          flagsp,
          flags_stride,
          data,
          l_w,
          0,
          mqc,
          v,
          oneplushalf,
          vsc as u32,
        );
        opj_t1_dec_clnpass_step_macro(
          true,
          false,
          &mut flags,
          flagsp,
          flags_stride,
          data,
          l_w,
          1,
          mqc,
          v,
          oneplushalf,
          false as u32,
        );
        opj_t1_dec_clnpass_step_macro(
          true,
          false,
          &mut flags,
          flagsp,
          flags_stride,
          data,
          l_w,
          2,
          mqc,
          v,
          oneplushalf,
          false as u32,
        );
        opj_t1_dec_clnpass_step_macro(
          true,
          false,
          &mut flags,
          flagsp,
          flags_stride,
          data,
          l_w,
          3,
          mqc,
          v,
          oneplushalf,
          false as u32,
        );
        flags &= !(T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3);
      }
      flagsp.set(flags);
      i += 1;
      data = data.offset(1);
      flagsp += 1
    }
    k += 4;
    data = data.offset(3_u32.wrapping_mul(l_w) as isize);
    flagsp += 2;
  }
  if k < h {
    i = 0;
    while i < l_w {
      j = 0;
      while j < h - k {
        opj_t1_dec_clnpass_step(
          mqc,
          l_w,
          flagsp,
          data.offset(j.wrapping_mul(l_w) as isize),
          oneplushalf,
          j,
          vsc as u32,
        );
        j += 1
      }
      flagsp.clear(T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3);
      i += 1;
      flagsp += 1;
      data = data.offset(1)
    }
  }
}

fn opj_t1_dec_clnpass_check_segsym(mut t1: &mut T1Coder, mut cblksty: i32) {
  if (cblksty as u32 & J2K_CCP_CBLKSTY_SEGSYM) != 0 {
    let mqc = &mut t1.mqc;
    let mut v = 0;
    let mut v2 = 0;
    opj_t1_setcurctx(mqc, T1_CTXNO_UNI);
    opj_mqc_decode_macro(&mut v, mqc);
    opj_mqc_decode_macro(&mut v2, mqc);
    v = (v << 1) | v2;
    opj_mqc_decode_macro(&mut v2, mqc);
    v = (v << 1) | v2;
    opj_mqc_decode_macro(&mut v2, mqc);
    v = (v << 1) | v2;
    if v != 0xa {
      /*
      event_msg!(t1->cinfo, EVT_WARNING, "Bad segmentation symbol %x\n", v);
      */
    }
  }
}

fn opj_t1_dec_clnpass_64x64_novsc(t1: &mut T1Coder, mut bpno: i32, data_src: Option<&[Cell<i32>]>) {
  opj_t1_dec_clnpass_internal(t1, bpno, false, 64, 64, 66, data_src);
}

fn opj_t1_dec_clnpass_64x64_vsc(t1: &mut T1Coder, mut bpno: i32, data_src: Option<&[Cell<i32>]>) {
  opj_t1_dec_clnpass_internal(t1, bpno, true, 64, 64, 66, data_src);
}

fn opj_t1_dec_clnpass_generic_novsc(
  t1: &mut T1Coder,
  mut bpno: i32,
  data_src: Option<&[Cell<i32>]>,
) {
  opj_t1_dec_clnpass_internal(t1, bpno, false, t1.w, t1.h, t1.w + 2, data_src);
}

fn opj_t1_dec_clnpass_generic_vsc(t1: &mut T1Coder, mut bpno: i32, data_src: Option<&[Cell<i32>]>) {
  opj_t1_dec_clnpass_internal(t1, bpno, true, t1.w, t1.h, t1.w + 2, data_src);
}

fn opj_t1_dec_clnpass(
  mut t1: &mut T1Coder,
  mut bpno: i32,
  mut cblksty: i32,
  data_src: Option<&[Cell<i32>]>,
) {
  if t1.w == 64 && t1.h == 64 {
    if (cblksty as u32 & J2K_CCP_CBLKSTY_VSC) != 0 {
      opj_t1_dec_clnpass_64x64_vsc(t1, bpno, data_src);
    } else {
      opj_t1_dec_clnpass_64x64_novsc(t1, bpno, data_src);
    }
  } else if (cblksty as u32 & J2K_CCP_CBLKSTY_VSC) != 0 {
    opj_t1_dec_clnpass_generic_vsc(t1, bpno, data_src);
  } else {
    opj_t1_dec_clnpass_generic_novsc(t1, bpno, data_src);
  }
  opj_t1_dec_clnpass_check_segsym(t1, cblksty);
}

fn opj_t1_getwmsedec(
  mut nmsedec: i32,
  mut compno: u32,
  mut level: u32,
  mut orient: u32,
  mut bpno: i32,
  mut qmfbid: u32,
  mut stepsize: f64,
  mut _numcomps: u32,
  mut mct_norms: Option<&[f64]>,
  mut mct_numcomps: u32,
) -> f64 {
  let mut w1 = 1.0;
  let mut w2 = 0.0;
  let mut wmsedec = 0.0;
  if let Some(norms) = mct_norms {
    if compno < mct_numcomps {
      w1 = norms[compno as usize]
    }
  }
  if qmfbid == 1 {
    w2 = opj_dwt_getnorm(level, orient)
  } else {
    /* if (qmfbid == 0) */
    let log2_gain = if orient == 0 {
      0
    } else if orient == 3 {
      2
    } else {
      1
    };
    w2 = opj_dwt_getnorm_real(level, orient);
    /* Not sure this is right. But preserves past behaviour */
    stepsize /= (1 << log2_gain) as f64;
  }
  wmsedec = w1 * w2 * stepsize * (1 << bpno) as f64;
  wmsedec *= wmsedec * nmsedec as f64 / 8192.0f64;
  wmsedec
}

fn opj_t1_allocate_buffers(mut t1: &mut T1Coder, mut w: u32, mut h: u32) -> OPJ_BOOL {
  let mut flagssize: u32 = 0;
  let mut flags_stride: u32 = 0;
  /* No risk of overflow. Prior checks ensure those assert are met */
  /* They are per the specification */

  debug_assert!(w <= 1024);
  debug_assert!(h <= 1024);
  debug_assert!(w.wrapping_mul(h) <= 4096);
  /* encoder uses tile buffer, so no need to allocate */
  let datasize = w.wrapping_mul(h);
  t1.data.resize(datasize as usize);
  flags_stride = w.wrapping_add(2u32);
  flagssize = h.wrapping_add(3u32).wrapping_div(4u32).wrapping_add(2u32);
  flagssize = (flagssize as core::ffi::c_uint).wrapping_mul(flags_stride);
  let mut flags_height = h.wrapping_add(3u32).wrapping_div(4u32);
  t1.flags.resize(flagssize as usize);
  let flags = t1.flags.as_slice();
  /* magic value to hopefully stop any passes being interested in this entry */
  const MAGIC: opj_flag_t = T1_PI_0 | T1_PI_1 | T1_PI_2 | T1_PI_3;
  let stride = flags_stride as usize;
  for cell in &flags[..stride] {
    cell.set(MAGIC);
  }
  let base = flags_height.wrapping_add(1).wrapping_mul(flags_stride) as usize;
  for cell in &flags[base..base + stride] {
    cell.set(MAGIC);
  }
  if !h.is_multiple_of(4) {
    let base = flags_height.wrapping_mul(flags_stride) as usize;
    let v = if h % 4 == 1 {
      T1_PI_1 | T1_PI_2 | T1_PI_3
    } else if h % 4 == 2 {
      T1_PI_2 | T1_PI_3
    } else if h % 4 == 3 {
      T1_PI_3
    } else {
      0
    };
    for cell in &flags[base..base + stride] {
      cell.set(v);
    }
  }
  t1.w = w;
  t1.h = h;
  1i32
}

/* ----------------------------------------------------------------------- */
/* ----------------------------------------------------------------------- */

#[allow(clippy::too_many_arguments)]
fn opj_t1_clbl_decode_processor(
  _resno: u32,
  off_x: u32,
  off_y: u32,
  src: &[u8],
  tccp: &Tccp,
  bandno: u32,
  band_x0: i32,
  band_y0: i32,
  band_stepsize: f32,
  cblk: &mut TcdCblkDec,
  p_manager: &EventMgr,
  check_pterm: OPJ_BOOL,
  mustuse_cblkdatabuffer: OPJ_BOOL,
  whole_tile_mutex: Option<&TileDataMutex>,
  tile_w: u32,
) -> OPJ_BOOL {
  let cblk_w = (cblk.x1 - cblk.x0) as u32;
  let cblk_h = (cblk.y1 - cblk.y0) as u32;
  let whole_tile = whole_tile_mutex.is_some();
  if !whole_tile {
    /* Window (partial) decoding keeps a per-codeblock buffer, like the C's
    decoded_data.  Reuse the buffer across re-decodes of the same code-block
    (heaptrack: this was the top allocation site). */
    cblk.decoded_data.clear();
    cblk
      .decoded_data
      .resize(cblk_w as usize * cblk_h as usize, Cell::new(0));
  }

  // Per-worker decode workspace (see T1Pool).
  let ret = T1_DECODE_POOL.get_or_init(T1Pool::new).with(|t1| {
    t1.mustuse_cblkdatabuffer = mustuse_cblkdatabuffer;
    let r = if tccp.cblksty & J2K_CCP_CBLKSTY_HT != 0 {
      opj_t1_ht_decode_cblk(
        t1,
        cblk,
        src,
        bandno,
        tccp.roishift as u32,
        tccp.cblksty,
        p_manager,
        check_pterm,
      )
    } else {
      opj_t1_decode_cblk(
        t1,
        cblk,
        src,
        bandno,
        tccp.roishift as u32,
        tccp.cblksty,
        p_manager,
        check_pterm,
      )
    };
    if r == 0 {
      return 0i32;
    }
    if whole_tile {
      /* Whole-tile decoding: like the C, the coefficients land in the
      worker's t1.data (no per-codeblock buffer); apply roishift +
      dequant and copy them straight into the codeblock's rectangle of
      the tile buffer (guarded by the tile mutex; rectangles are
      disjoint, so the lock is effectively uncontended). */
      let data = t1.data.as_slice();
      if tccp.roishift != 0 {
        let thresh = if tccp.roishift >= 31i32 {
          i32::MAX
        } else {
          (1i32) << tccp.roishift
        };
        let mut idx = 0usize;
        while idx < cblk_w.wrapping_mul(cblk_h) as usize {
          if tccp.roishift >= 31i32 {
            data[idx].set(0);
          } else {
            let mut val = data[idx].get();
            let mut mag = val.abs();
            if mag >= thresh {
              mag >>= tccp.roishift;
              val = if val < 0i32 { -mag } else { mag };
            }
            data[idx].set(val);
          }
          idx += 1
        }
      }
      let cblk_size = cblk_w.wrapping_mul(cblk_h);
      if tccp.qmfbid == 1 {
        let mut idx = 0usize;
        while idx < cblk_size as usize {
          data[idx].set(data[idx].get() / 2i32);
          idx += 1
        }
      } else {
        let stepsize = 0.5f32 * band_stepsize;
        let mut idx = 0usize;
        while idx < cblk_size as usize {
          let tmp = data[idx].get() as f32 * stepsize;
          // C reinterprets the f32 bits as an i32 via memcpy.
          data[idx].set(tmp.to_bits() as i32);
          idx += 1
        }
      }
      let x = (cblk.x0 - band_x0) as u32 + off_x;
      let y = (cblk.y0 - band_y0) as u32 + off_y;
      {
        match whole_tile_mutex.unwrap() {
          TileDataMutex::I32(m) => {
            let mut tile_data = m.lock().unwrap();
            let mut jj = 0u32;
            while jj < cblk_h {
              let base = (y.wrapping_add(jj)).wrapping_mul(tile_w).wrapping_add(x) as usize;
              let src = jj.wrapping_mul(cblk_w) as usize;
              for ii in 0..cblk_w as usize {
                tile_data[base + ii] = data[src + ii].get();
              }
              jj += 1
            }
          }
          TileDataMutex::F32(m) => {
            /* 9-7: the dequantized t1 cells hold the f32 bit patterns;
            store the words (the C reinterprets the shared array). */
            let mut tile_data = m.lock().unwrap();
            let mut jj = 0u32;
            while jj < cblk_h {
              let base = (y.wrapping_add(jj)).wrapping_mul(tile_w).wrapping_add(x) as usize;
              let src = jj.wrapping_mul(cblk_w) as usize;
              for ii in 0..cblk_w as usize {
                tile_data[base + ii] = f32::from_bits(data[src + ii].get() as u32);
              }
              jj += 1
            }
          }
        }
      }
    }
    1i32
  });
  if ret == 0 {
    return 0i32;
  }
  if !whole_tile {
    let mut i: u32 = 0;
    let mut j: u32 = 0;
    if tccp.roishift != 0 {
      let thresh = if tccp.roishift >= 31i32 {
        i32::MAX
      } else {
        (1i32) << tccp.roishift
      };
      j = 0_u32;
      while j < cblk_h {
        i = 0_u32;
        while i < cblk_w {
          let idx = j.wrapping_mul(cblk_w).wrapping_add(i) as usize;
          if tccp.roishift >= 31i32 {
            cblk.decoded_data[idx].set(0);
          } else {
            let mut val = cblk.decoded_data[idx].get();
            let mut mag = val.abs();
            if mag >= thresh {
              mag >>= tccp.roishift;
              val = if val < 0i32 { -mag } else { mag };
            }
            cblk.decoded_data[idx].set(val);
          }
          i += 1
        }
        j += 1
      }
    }
    let cblk_size = cblk_w.wrapping_mul(cblk_h);
    if tccp.qmfbid == 1 {
      i = 0_u32;
      while i < cblk_size {
        let idx = i as usize;
        cblk.decoded_data[idx].set(cblk.decoded_data[idx].get() / 2i32);
        i += 1
      }
    } else {
      let stepsize = 0.5f32 * band_stepsize;
      i = 0_u32;
      while i < cblk_size {
        let idx = i as usize;
        let tmp = cblk.decoded_data[idx].get() as f32 * stepsize;
        // C reinterprets the f32 bits as an i32 via memcpy.
        cblk.decoded_data[idx].set(tmp.to_bits() as i32);
        i += 1
      }
    }
  }
  1i32
}

pub(crate) fn opj_t1_decode_cblks(
  tilec_view: &TcdTilecView,
  _tccps: &[Tccp],
  image_comps: &[CoreImageComp],
  pret: &mut OPJ_BOOL,
  tilec: &mut TcdTileComp,
  tccp: &Tccp,
  p_manager: &EventMgr,
  check_pterm: OPJ_BOOL,
  src: &mut [u8],
) {
  /* Snapshot the tile geometry (values), so the walks below can borrow
  tilec.resolutions without aliasing the data buffer. */
  let tile_w = (tilec.resolutions[(tilec.minimum_num_resolutions.wrapping_sub(1u32)) as usize].x1
    - tilec.resolutions[(tilec.minimum_num_resolutions.wrapping_sub(1u32)) as usize].x0)
    as usize;
  let tile_h = (tilec.resolutions[(tilec.minimum_num_resolutions.wrapping_sub(1u32)) as usize].y1
    - tilec.resolutions[(tilec.minimum_num_resolutions.wrapping_sub(1u32)) as usize].y0)
    as usize;
  /* Collect the decode jobs: every codeblock whose area is of interest.
  The per-codeblock buffers carry the decoded coefficients so the decode
  can run in parallel and the tile-buffer write-back stays sequential. */
  let mut jobs: Vec<(u32, i32, i32, f32, u32, u32, &mut TcdCblkDec)> = Vec::new();

  /* The previous (coarser) resolution's extents feed the band offsets;
  snapshot them first so the collection walk only needs `&mut`. */
  let prev_dims: Vec<(i32, i32)> = (0..tilec.minimum_num_resolutions)
    .map(|r| {
      if r > 0 {
        let p = &tilec.resolutions[(r - 1) as usize];
        (p.x1 - p.x0, p.y1 - p.y0)
      } else {
        (0, 0)
      }
    })
    .collect();
  for (resno, res) in tilec
    .resolutions
    .iter_mut()
    .enumerate()
    .take(tilec.minimum_num_resolutions as usize)
  {
    let (prev_w, prev_h) = prev_dims[resno];
    for band in res.bands.iter_mut().take(res.numbands as usize) {
      /* Per-band offsets (band 0 is the LL band: no offset). */
      let bandno_val = band.bandno;
      let band_x0 = band.x0;
      let band_y0 = band.y0;
      let band_stepsize = band.stepsize;
      let off_x = if bandno_val & 1 != 0 { prev_w } else { 0 };
      let off_y = if bandno_val & 2 != 0 { prev_h } else { 0 };
      for precinct in band
        .precincts
        .iter_mut()
        .take((res.pw.wrapping_mul(res.ph)) as usize)
      {
        if opj_tcd_is_subband_area_of_interest(
          tilec_view,
          tccp,
          image_comps,
          resno as u32,
          bandno_val,
          precinct.x0 as u32,
          precinct.y0 as u32,
          precinct.x1 as u32,
          precinct.y1 as u32,
        ) == 0
        {
          if let CodeBlocks::Dec(cblks) = &mut precinct.cblks {
            for cblk in cblks
              .iter_mut()
              .take((precinct.cw.wrapping_mul(precinct.ch)) as usize)
            {
              if !cblk.decoded_data.is_empty() {
                cblk.decoded_data = Vec::new();
              }
            }
          }
        } else if let CodeBlocks::Dec(cblks) = &mut precinct.cblks {
          for cblk in cblks
            .iter_mut()
            .take((precinct.cw.wrapping_mul(precinct.ch)) as usize)
          {
            if opj_tcd_is_subband_area_of_interest(
              tilec_view,
              tccp,
              image_comps,
              resno as u32,
              bandno_val,
              cblk.x0 as u32,
              cblk.y0 as u32,
              cblk.x1 as u32,
              cblk.y1 as u32,
            ) == 0
            {
              if !cblk.decoded_data.is_empty() {
                cblk.decoded_data = Vec::new();
              }
            } else {
              let mut do_decode = true;
              if tilec_view.whole_tile_decoding == 0 {
                let cblk_w = (cblk.x1 - cblk.x0) as u32;
                let cblk_h = (cblk.y1 - cblk.y0) as u32;
                if !cblk.decoded_data.is_empty() || cblk_w == 0 || cblk_h == 0 {
                  do_decode = false;
                }
              }
              if do_decode {
                jobs.push((
                  bandno_val,
                  band_x0,
                  band_y0,
                  band_stepsize,
                  off_x as u32,
                  off_y as u32,
                  cblk,
                ));
              }
            }
          }
        }
      }
    }
  }

  if *pret == 0 {
    return;
  }

  /* Parallel decode: each job owns its codeblock (and the per-worker T1
  workspace from the decode pool).  Whole-tile decoding writes the
  dequantized coefficients straight into the codeblock's rectangle of the
  tile buffer through a shared Cell view (the C's design: the worker's
  t1.data holds the coefficients, no per-codeblock buffers); window
  decoding keeps cblk.decoded_data.  Small job sets stay sequential: the
  rayon overhead would dominate. */
  /* Whole-tile decoding shares the tile buffer across the parallel jobs
  through a mutex: each job only ever writes its own codeblock rectangle
  (disjoint regions), and the copy is a handful of cache lines, so the lock
  is effectively uncontended. */
  let tile_mutex: Option<TileDataMutex> = if tilec_view.whole_tile_decoding != 0 {
    let len = tile_w.wrapping_mul(tile_h);
    Some(if tilec.data.is_f32() {
      TileDataMutex::F32(std::sync::Mutex::new(tilec.data.as_mut_f32(len)))
    } else {
      TileDataMutex::I32(std::sync::Mutex::new(tilec.data.as_mut_i32(len)))
    })
  } else {
    None
  };
  let src_shared: &[u8] = &*src;
  let decode_job = |(bandno, band_x0, band_y0, band_stepsize, off_x, off_y): (
    &u32,
    &i32,
    &i32,
    &f32,
    &u32,
    &u32,
  ),
                    cblk: &mut TcdCblkDec|
   -> OPJ_BOOL {
    opj_t1_clbl_decode_processor(
      0,
      *off_x,
      *off_y,
      src_shared,
      tccp,
      *bandno,
      *band_x0,
      *band_y0,
      *band_stepsize,
      cblk,
      p_manager,
      check_pterm,
      0,
      tile_mutex.as_ref(),
      tile_w as u32,
    )
  };
  let ok_flags: Vec<OPJ_BOOL> = par::map_collect(
    &mut jobs,
    8,
    |_i, (bandno, band_x0, band_y0, band_stepsize, off_x, off_y, cblk)| {
      decode_job(
        (bandno, band_x0, band_y0, band_stepsize, off_x, off_y),
        cblk,
      )
    },
  );
  if ok_flags.contains(&0) {
    *pret = 0;
  }
}

fn opj_t1_decode_cblk(
  t1: &mut T1Coder,
  cblk: &mut TcdCblkDec,
  src: &[u8],
  mut orient: u32,
  mut roishift: u32,
  mut cblksty: u32,
  p_manager: &EventMgr,
  mut check_pterm: OPJ_BOOL,
) -> OPJ_BOOL {
  let mut bpno_plus_one: i32 = 0; /* BYPASS mode */
  let mut passtype: u32 = 0;
  let mut segno: u32 = 0;
  let mut passno: u32 = 0;
  let mut cblkdataindex = 0_u32;
  let mut type_0 = 0_u8;
  t1.mqc.lut_ctxno_zc_orient = &LUT_CTXNO_ZC[orient as usize];
  if opj_t1_allocate_buffers(t1, (cblk.x1 - cblk.x0) as u32, (cblk.y1 - cblk.y0) as u32) == 0 {
    return 0i32;
  }
  bpno_plus_one = roishift.wrapping_add(cblk.numbps) as i32;
  if bpno_plus_one >= 31i32 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "opj_t1_decode_cblk(): unsupported bpno_plus_one = {} >= 31\n",
      bpno_plus_one,
    );
    return 0i32;
  }
  passtype = 2;

  opj_mqc_resetstates(&mut t1.mqc);
  opj_mqc_setstate(&mut t1.mqc, T1_CTXNO_UNI, 0, 46);
  opj_mqc_setstate(&mut t1.mqc, T1_CTXNO_AGG, 0, 3);
  opj_mqc_setstate(&mut t1.mqc, T1_CTXNO_ZC, 0, 4);

  if cblk.corrupted {
    assert_eq!(cblk.numchunks, 0);
    return 1i32;
  }

  /* C parity (t1.c): a codeblock with no data has nothing to decode -
  return before the segment loop so the PTERM check below does not run
  against the previous codeblock's stale MQ state. */
  if cblk.numchunks == 0 {
    return 1i32;
  }

  /* The C code reads the codeblock payload either in place (single chunk,
  no cblkdatabuffer) or through a concatenation buffer (multi-chunk /
  multi-threaded cases). Here the MQ coder owns its buffer, so we always
  copy the chunks into it: one copy per codeblock. */
  {
    let mut i: u32 = 0;
    let mut cblk_len: u32 = 0;
    /* Compute whole codeblock length from chunk lengths */
    cblk_len = 0_u32;
    i = 0_u32;
    while i < cblk.numchunks {
      cblk_len = (cblk_len as core::ffi::c_uint).wrapping_add(cblk.chunks[i as usize].len);
      i += 1
    }
    /* Allocate temporary memory if needed */
    t1.mqc
      .grow_buf((cblk_len + OPJ_COMMON_CBLK_DATA_EXTRA) as usize);
    /* Concatenate all chunks into the MQ coder's buffer */
    let buf = t1.mqc.buf_mut();
    let mut off = 0usize;
    i = 0_u32;
    while i < cblk.numchunks {
      let chunk = &cblk.chunks[i as usize];
      let src_chunk = &src[chunk.data_offset..chunk.data_offset + chunk.len as usize];
      for (dst, &b) in buf[off..off + chunk.len as usize].iter_mut().zip(src_chunk) {
        dst.set(b);
      }
      off += chunk.len as usize;
      i += 1
    }
  }
  /* For subtile decoding, directly decode in the decoded_data buffer of */
  /* the code-block. Hack t1->data to point to it, and restore it later */
  let decoded_data_src: Option<&[Cell<i32>]> = if !cblk.decoded_data.is_empty() {
    Some(cblk.decoded_data.as_slice())
  } else {
    None
  };
  segno = 0_u32;
  while segno < cblk.real_num_segs {
    let seg = &cblk.segs[segno as usize];
    /* BYPASS mode */
    type_0 = if bpno_plus_one <= cblk.numbps as i32 - 4i32 && passtype < 2 && cblksty & 0x1 != 0 {
      1i32
    } else {
      0i32
    } as u8;
    if type_0 as core::ffi::c_int == 1i32 {
      opj_mqc_raw_init_dec(&mut t1.mqc, cblkdataindex as usize, seg.len, 2_u32);
    } else {
      opj_mqc_init_dec(&mut t1.mqc, cblkdataindex as usize, seg.len, 2_u32);
    }
    cblkdataindex = (cblkdataindex as core::ffi::c_uint).wrapping_add(seg.len);
    passno = 0_u32;
    while passno < seg.real_num_passes && bpno_plus_one >= 1i32 {
      match passtype {
        0 => {
          if type_0 as core::ffi::c_int == 1i32 {
            opj_t1_dec_sigpass_raw(t1, bpno_plus_one, cblksty as i32, decoded_data_src);
          } else {
            opj_t1_dec_sigpass_mqc(t1, bpno_plus_one, cblksty as i32, decoded_data_src);
          }
        }
        1 => {
          if type_0 as core::ffi::c_int == 1i32 {
            opj_t1_dec_refpass_raw(t1, bpno_plus_one, decoded_data_src);
          } else {
            opj_t1_dec_refpass_mqc(t1, bpno_plus_one, decoded_data_src);
          }
        }
        2 => {
          opj_t1_dec_clnpass(t1, bpno_plus_one, cblksty as i32, decoded_data_src);
        }
        _ => {}
      }
      if (cblksty & J2K_CCP_CBLKSTY_RESET) != 0 && type_0 == T1_TYPE_MQ {
        opj_mqc_resetstates(&mut t1.mqc);
        opj_mqc_setstate(&mut t1.mqc, T1_CTXNO_UNI, 0, 46);
        opj_mqc_setstate(&mut t1.mqc, T1_CTXNO_AGG, 0, 3);
        opj_mqc_setstate(&mut t1.mqc, T1_CTXNO_ZC, 0, 4);
      }
      passtype = passtype.wrapping_add(1);
      if passtype == 3 {
        passtype = 0;
        bpno_plus_one -= 1
      }
      passno += 1;
    }
    opq_mqc_finish_dec(&mut t1.mqc);
    segno += 1;
  }
  if check_pterm != 0 {
    let mqc = &mut t1.mqc; /* MQC component */
    if mqc.remaining() > 2 {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "PTERM check failure: {} remaining bytes in code block ({} used / {})\n",
        mqc.remaining() as core::ffi::c_int - 2i32,
        mqc.consumed() as core::ffi::c_int,
        mqc.seg_len() as core::ffi::c_int,
      );
    } else if mqc.end_of_byte_stream_counter > 2 {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "PTERM check failure: {} synthesized 0xFF markers read\n",
        mqc.end_of_byte_stream_counter,
      );
    }
  }
  1i32
}

#[allow(clippy::too_many_arguments)]
/* * Procedure to deal with a asynchronous code-block encoding job.
 *
 * @param user_data Pointer to a opj_t1_cblk_encode_processing_job_t* structure
 * @param tls       TLS handle.
 */
fn opj_t1_cblk_encode_processor(
  compno: u32,
  _resno: u32,
  off_x: u32,
  off_y: u32,
  tile_data: TileDataView,
  tile_w: u32,
  tile_numcomps: u32,
  numresolutions_minus_resno: u32,
  bandno: u32,
  band_x0: i32,
  band_y0: i32,
  band_stepsize: f32,
  cblk: &mut TcdCblkEnc,
  tccp: &Tccp,
  mct_norms: Option<&[f64]>,
  mct_numcomps: u32,
) -> (OPJ_BOOL, f64) {
  let mut cblk_w: u32 = 0;
  let mut cblk_h: u32 = 0;
  let mut i: u32 = 0;
  let mut j: u32 = 0;
  let mut x = cblk.x0 - band_x0;
  let mut y = cblk.y0 - band_y0;

  // Per-worker encode workspace (see T1Pool).
  let (ret, cumwmsedec) = T1_ENCODE_POOL.get_or_init(T1Pool::new).with(|t1| {
    let mut ret = 0i32;

    x += off_x as i32;
    y += off_y as i32;
    if opj_t1_allocate_buffers(t1, (cblk.x1 - cblk.x0) as u32, (cblk.y1 - cblk.y0) as u32) == 0 {
      return (0i32, 0.0);
    }
    cblk_w = t1.w;
    cblk_h = t1.h;
    let base = (y as u32).wrapping_mul(tile_w).wrapping_add(x as u32);
    let ti32: &[i32] = match tile_data {
      TileDataView::I32(d) => d,
      TileDataView::F32(_) => &[],
    };
    let tf32: &[f32] = match tile_data {
      TileDataView::F32(d) => d,
      TileDataView::I32(_) => &[],
    };
    if tccp.qmfbid == 1 {
      let mut t1data = t1.data.offset(0);
      /* Do multiplication on unsigned type, even if the
       * underlying type is signed, to avoid potential
       * int overflow on large value (the output will be
       * incorrect in such situation, but whatever...)
       * This assumes complement-to-2 signed integer
       * representation
       * Fixes https://github.com/uclouvain/openjpeg/issues/1053
       */
      j = 0_u32;
      while j < cblk_h & !(3u32) {
        i = 0_u32;
        while i < cblk_w {
          t1data.set_at(
            0,
            ((ti32[base.wrapping_add(j.wrapping_mul(tile_w)).wrapping_add(i) as usize] as u32)
              << (7i32 - 1i32)) as i32,
          );
          t1data.set_at(
            1,
            ((ti32[base
              .wrapping_add(j.wrapping_add(1).wrapping_mul(tile_w))
              .wrapping_add(i) as usize] as u32)
              << (7i32 - 1i32)) as i32,
          );
          t1data.set_at(
            2,
            ((ti32[base
              .wrapping_add(j.wrapping_add(2).wrapping_mul(tile_w))
              .wrapping_add(i) as usize] as u32)
              << (7i32 - 1i32)) as i32,
          );
          t1data.set_at(
            3,
            ((ti32[base
              .wrapping_add(j.wrapping_add(3).wrapping_mul(tile_w))
              .wrapping_add(i) as usize] as u32)
              << (7i32 - 1i32)) as i32,
          );
          t1data = t1data.offset(4);
          i += 1
        }
        j = (j as core::ffi::c_uint).wrapping_add(4)
      }
      if j < cblk_h {
        i = 0_u32;
        while i < cblk_w {
          let mut k: u32 = 0;
          k = j;
          while k < cblk_h {
            t1data.set_at(
              0,
              ((ti32[base.wrapping_add(k.wrapping_mul(tile_w)).wrapping_add(i) as usize]
                as u32)
                << (7i32 - 1i32)) as i32,
            );
            t1data = t1data.offset(1);
            k += 1
          }
          i += 1
        }
      }
    } else {
      let mut t1data_0 = t1.data.offset(0);
      /* Change from "natural" order to "zigzag" order of T1 passes */
      /* Change from "natural" order to "zigzag" order of T1 passes */
      j = 0_u32; /* fixed_quality */
      while j < cblk_h & !(3u32) {
        i = 0_u32; /* compno  */
        while i < cblk_w {
          t1data_0.set_at(
            0,
            opj_lrintf(
              tf32[base.wrapping_add(j.wrapping_mul(tile_w)).wrapping_add(i) as usize]
                / band_stepsize
                * ((1i32) << (7i32 - 1i32)) as core::ffi::c_float,
            ) as i32,
          );
          t1data_0.set_at(
            1,
            opj_lrintf(
              tf32[base
                  .wrapping_add(j.wrapping_add(1).wrapping_mul(tile_w))
                  .wrapping_add(i) as usize] / band_stepsize
                * ((1i32) << (7i32 - 1i32)) as core::ffi::c_float,
            ) as i32,
          );
          t1data_0.set_at(
            2,
            opj_lrintf(
              tf32[base
                  .wrapping_add(j.wrapping_add(2).wrapping_mul(tile_w))
                  .wrapping_add(i) as usize] / band_stepsize
                * ((1i32) << (7i32 - 1i32)) as core::ffi::c_float,
            ) as i32,
          );
          t1data_0.set_at(
            3,
            opj_lrintf(
              tf32[base
                  .wrapping_add(j.wrapping_add(3).wrapping_mul(tile_w))
                  .wrapping_add(i) as usize] / band_stepsize
                * ((1i32) << (7i32 - 1i32)) as core::ffi::c_float,
            ) as i32,
          );
          t1data_0 = t1data_0.offset(4);
          i += 1
        }
        j = (j as core::ffi::c_uint).wrapping_add(4)
      }
      if j < cblk_h {
        i = 0_u32;
        while i < cblk_w {
          let mut k_0: u32 = 0;
          k_0 = j;
          while k_0 < cblk_h {
            t1data_0.set_at(
              0,
              opj_lrintf(
                tf32[base.wrapping_add(k_0.wrapping_mul(tile_w)).wrapping_add(i) as usize]
                  / band_stepsize
                  * ((1i32) << (7i32 - 1i32)) as core::ffi::c_float,
              ) as i32,
            );
            t1data_0 = t1data_0.offset(1);
            k_0 += 1;
          }
          i += 1
        }
      }
    }
    let cumwmsedec = opj_t1_encode_cblk(
      t1,
      cblk,
      bandno,
      compno,
      numresolutions_minus_resno,
      tccp.qmfbid,
      band_stepsize as f64,
      tccp.cblksty,
      tile_numcomps,
      mct_norms,
      mct_numcomps,
    );
    ret = 1i32;
    (ret, cumwmsedec)
  });
  (ret, cumwmsedec)
}
pub(crate) fn opj_t1_encode_cblks(
  tile: &mut TcdTile,
  tcp: &Tcp,
  mct_norms: Option<&[f64]>,
  mct_numcomps: u32,
) -> OPJ_BOOL {
  let mut ret = 1i32;
  let mut compno: u32 = 0;
  tile.distotile = 0 as f64;
  compno = 0_u32;
  while compno < tile.numcomps {
    let tilec = &mut tile.comps[compno as usize];
    let tccp = &tcp.tccps[compno as usize];
    let tile_w = (tilec.x1 - tilec.x0) as u32;
    let tile_h = (tilec.y1 - tilec.y0) as u32;
    let tile_numcomps = tile.numcomps;
    let numresolutions_minus_1 = tilec.numresolutions.wrapping_sub(1u32);
    /* Snapshot the coarser-resolution dims first (the collection walk only
    needs `&mut`), then take the shared read-only view of the tile data. */
    let prev_dims: Vec<(i32, i32)> = (0..tilec.numresolutions)
      .map(|r| {
        if r > 0 {
          let p = &tilec.resolutions[(r - 1) as usize];
          (p.x1 - p.x0, p.y1 - p.y0)
        } else {
          (0, 0)
        }
      })
      .collect();
    let tile_len = (tile_w as usize).wrapping_mul(tile_h as usize);
    let tile_data_shared: TileDataView = if tilec.data.is_f32() {
      TileDataView::F32(tilec.data.as_mut_f32(tile_len))
    } else {
      TileDataView::I32(tilec.data.as_mut_i32(tile_len))
    };
    /* Collect the encode jobs (the codeblock walk mutates only the cblk
    state, and each job writes only its own codeblock). */
    let mut jobs: Vec<(u32, u32, u32, u32, u32, i32, i32, f32, &mut TcdCblkEnc)> = Vec::new();
    for (resno, res) in tilec
      .resolutions
      .iter_mut()
      .enumerate()
      .take(tilec.numresolutions as usize)
    {
      let (prev_w, prev_h) = prev_dims[resno];
      for band in res.bands.iter_mut().take(res.numbands as usize) {
        let bandno_val = band.bandno;
        let band_x0 = band.x0;
        let band_y0 = band.y0;
        let band_stepsize = band.stepsize;
        let off_x = if bandno_val & 1 != 0 { prev_w } else { 0 };
        let off_y = if bandno_val & 2 != 0 { prev_h } else { 0 };
        /* Skip empty bands */
        if !opj_tcd_is_band_empty(band) {
          for prc in band
            .precincts
            .iter_mut()
            .take((res.pw.wrapping_mul(res.ph)) as usize)
          {
            if let CodeBlocks::Enc(cblks) = &mut prc.cblks {
              for cblk in cblks
                .iter_mut()
                .take((prc.cw.wrapping_mul(prc.ch)) as usize)
              {
                jobs.push((
                  resno as u32,
                  off_x as u32,
                  off_y as u32,
                  bandno_val,
                  numresolutions_minus_1.wrapping_sub(resno as u32),
                  band_x0,
                  band_y0,
                  band_stepsize,
                  cblk,
                ));
              }
            }
          }
        }
      }
    }
    /* Parallel encode: each job owns its codeblock; tile_data is read-only.
    Small job sets stay sequential (rayon overhead would dominate). */
    let encode_job = |(resno, off_x, off_y, bandno, nrr, band_x0, band_y0, band_stepsize): (
      &u32,
      &u32,
      &u32,
      &u32,
      &u32,
      &i32,
      &i32,
      &f32,
    ),
                      cblk: &mut TcdCblkEnc|
     -> (OPJ_BOOL, f64) {
      opj_t1_cblk_encode_processor(
        compno,
        *resno,
        *off_x,
        *off_y,
        tile_data_shared,
        tile_w,
        tile_numcomps,
        *nrr,
        *bandno,
        *band_x0,
        *band_y0,
        *band_stepsize,
        cblk,
        tccp,
        mct_norms,
        mct_numcomps,
      )
    };
    let results: Vec<(OPJ_BOOL, f64)> = par::map_collect(
      &mut jobs,
      8,
      |_i, (resno, off_x, off_y, bandno, nrr, band_x0, band_y0, band_stepsize, cblk)| {
        encode_job(
          (
            resno,
            off_x,
            off_y,
            bandno,
            nrr,
            band_x0,
            band_y0,
            band_stepsize,
          ),
          cblk,
        )
      },
    );
    let mut cumdisto: f64 = 0.0;
    for (ok, cumwmsedec) in results {
      if ok == 0 {
        ret = 0i32;
      }
      /* Same reduction order as the sequential walk. */
      cumdisto += cumwmsedec;
    }
    tile.distotile += cumdisto;
    compno += 1;
  }
  ret
}

fn opj_t1_enc_is_term_pass(
  cblk: &TcdCblkEnc,
  mut cblksty: u32,
  mut bpno: i32,
  mut passtype: u32,
) -> core::ffi::c_int {
  /* Is it the last cleanup pass ? */
  if passtype == 2 && bpno == 0i32 {
    return 1i32;
  }
  if cblksty & 0x4 != 0 {
    return 1i32;
  }
  if cblksty & 0x1 != 0 {
    /* For bypass arithmetic bypass, terminate the 4th cleanup pass */
    if bpno == cblk.numbps as i32 - 4i32 && passtype == 2 {
      return 1i32;
    }
    /* and beyond terminate all the magnitude refinement passes (in raw) */
    /* and cleanup passes (in MQC) */
    if bpno < cblk.numbps as i32 - 4i32 && passtype > 0 {
      return 1i32;
    }
  }
  0i32
}

/* * Return "cumwmsedec" that should be used to increase tile->distotile */
/* * mod fixed_quality */
fn opj_t1_encode_cblk(
  t1: &mut T1Coder,
  cblk: &mut TcdCblkEnc,
  mut orient: u32,
  mut compno: u32,
  mut level: u32,
  mut qmfbid: u32,
  mut stepsize: f64,
  mut cblksty: u32,
  mut numcomps: u32,
  mut mct_norms: Option<&[f64]>,
  mut mct_numcomps: u32,
) -> core::ffi::c_double {
  let mut cumwmsedec = 0.0f64; /* MQC component */
  let mut passno: u32 = 0;
  let mut bpno: i32 = 0;
  let mut passtype: u32 = 0;
  let mut nmsedec = 0i32;
  let mut max: i32 = 0;
  let mut i: u32 = 0;
  let mut j: u32 = 0;
  let mut type_0 = 0_u8;
  let mut tempwmsedec: f64 = 0.;
  t1.mqc.lut_ctxno_zc_orient = &LUT_CTXNO_ZC[orient as usize];
  max = 0i32;
  let mut datap = t1.data.offset(0);
  j = 0_u32;
  while j < t1.h {
    let w = t1.w;
    i = 0_u32;
    while i < w {
      let mut tmp = datap.get();
      if tmp < 0i32 {
        let mut tmp_unsigned: u32 = 0;
        if tmp == i32::MIN {
          /* To avoid undefined behaviour when negating INT_MIN */
          /* but if we go here, it means we have supplied an input */
          /* with more bit depth than we we can really support. */
          /* Cf https://github.com/uclouvain/openjpeg/issues/1432 */
          tmp = i32::MIN + 1;
        }
        max = opj_int_max(max, -tmp);
        tmp_unsigned = if tmp >= 0i32 {
          tmp as u32
        } else {
          (-tmp as u32) | 0x80000000u32
        };
        datap.set(tmp_unsigned as i32);
      } else {
        max = opj_int_max(max, tmp)
      }
      i += 1;
      datap = datap.offset(1);
    }
    j += 1
  }
  cblk.numbps = if max != 0 {
    (opj_int_floorlog2(max) + 1i32 - (7i32 - 1i32)) as u32
  } else {
    0
  };
  if cblk.numbps == 0 {
    cblk.totalpasses = 0_u32;
    return cumwmsedec;
  }
  bpno = cblk.numbps.wrapping_sub(1) as i32;
  passtype = 2_u32;

  opj_mqc_resetstates(&mut t1.mqc);
  opj_mqc_setstate(&mut t1.mqc, T1_CTXNO_UNI, 0, 46);
  opj_mqc_setstate(&mut t1.mqc, T1_CTXNO_AGG, 0, 3);
  opj_mqc_setstate(&mut t1.mqc, T1_CTXNO_ZC, 0, 4);
  opj_mqc_init_enc(&mut t1.mqc, cblk.data_size);

  passno = 0_u32;
  while bpno >= 0i32 {
    type_0 = if bpno < cblk.numbps as i32 - 4i32 && passtype < 2 && cblksty & 0x1 != 0 {
      1i32
    } else {
      0i32
    } as u8;
    /* If the previous pass was terminating, we need to reset the encoder */
    if passno > 0 && cblk.passes[passno.wrapping_sub(1) as usize].term {
      if type_0 as core::ffi::c_int == 1i32 {
        opj_mqc_bypass_init_enc(&mut t1.mqc);
      } else {
        opj_mqc_restart_init_enc(&mut t1.mqc);
      }
    }
    match passtype {
      0 => {
        opj_t1_enc_sigpass(t1, bpno, &mut nmsedec, type_0, cblksty);
      }
      1 => {
        opj_t1_enc_refpass(t1, bpno, &mut nmsedec, type_0);
      }
      2 => {
        opj_t1_enc_clnpass(t1, bpno, &mut nmsedec, cblksty);
        /* code switch SEGMARK (i.e. SEGSYM) */
        if cblksty & 0x20 != 0 {
          opj_mqc_segmark_enc(&mut t1.mqc);
        }
      }
      _ => {}
    }

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
    cblk.passes[passno as usize].distortiondec = cumwmsedec;
    if opj_t1_enc_is_term_pass(cblk, cblksty, bpno, passtype) != 0 {
      /* If it is a terminated pass, terminate it */
      if type_0 as core::ffi::c_int == 1i32 {
        opj_mqc_bypass_flush_enc(&mut t1.mqc, (cblksty & 0x10) as OPJ_BOOL);
      } else if cblksty & 0x10 != 0 {
        opj_mqc_erterm_enc(&mut t1.mqc);
      } else {
        opj_mqc_flush(&mut t1.mqc);
      }
      cblk.passes[passno as usize].term = true;
      cblk.passes[passno as usize].rate = opj_mqc_numbytes(&mut t1.mqc)
    } else {
      /* Non terminated pass */
      let mut rate_extra_bytes: u32 = 0;
      if type_0 as core::ffi::c_int == 1i32 {
        rate_extra_bytes = opj_mqc_bypass_get_extra_bytes(&mut t1.mqc, (cblksty & 0x10) as OPJ_BOOL)
      } else {
        rate_extra_bytes = 3_u32
      }
      cblk.passes[passno as usize].term = false;
      cblk.passes[passno as usize].rate =
        opj_mqc_numbytes(&mut t1.mqc).wrapping_add(rate_extra_bytes)
    }
    passtype = passtype.wrapping_add(1);
    if passtype == 3 {
      passtype = 0_u32;
      bpno -= 1
    }
    /* Code-switch "RESET" */
    if cblksty & 0x2 != 0 {
      opj_mqc_reset_enc(&mut t1.mqc);
    }
    passno += 1;
  }
  cblk.totalpasses = passno;
  if cblk.totalpasses != 0 {
    {
      /* Copy the encoded bytes from the MQ coder's buffer into the codeblock
       * data buffer (the C encoder wrote them in place through a pointer). */
      let nb = opj_mqc_numbytes(&mut t1.mqc) as usize;
      cblk.data.resize(nb.wrapping_add(1), 0);
      let mq_buf = t1.mqc.buf_ref();
      for (i, cell) in mq_buf[1..1 + nb].iter().enumerate() {
        cblk.data[1 + i] = cell.get();
      }
    }
    /* Make sure that pass rates are increasing */
    let mut last_pass_rate = opj_mqc_numbytes(&mut t1.mqc);
    passno = cblk.totalpasses;
    while passno > 0 {
      passno = passno.wrapping_sub(1);
      if cblk.passes[passno as usize].rate > last_pass_rate {
        cblk.passes[passno as usize].rate = last_pass_rate
      } else {
        last_pass_rate = cblk.passes[passno as usize].rate
      }
    }
  }
  passno = 0_u32;
  while passno < cblk.totalpasses {
    /* Prevent generation of FF as last data byte of a pass*/
    /* For terminating passes, the flushing procedure ensured this already */
    debug_assert!(cblk.passes[passno as usize].rate > 0);
    if cblk.data_byte(cblk.passes[passno as usize].rate.wrapping_sub(1) as usize)
      as core::ffi::c_int
      == 0xffi32
    {
      cblk.passes[passno as usize].rate = cblk.passes[passno as usize].rate.wrapping_sub(1)
    }
    cblk.passes[passno as usize].len =
      cblk.passes[passno as usize]
        .rate
        .wrapping_sub(if passno == 0 {
          0
        } else {
          cblk.passes[passno.wrapping_sub(1) as usize].rate
        });
    passno += 1;
  }
  cumwmsedec
}
