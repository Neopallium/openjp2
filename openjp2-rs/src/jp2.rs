use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use std::collections::BTreeSet;
use std::io::{Read, Write};

use super::cio::*;
use super::consts::*;
use super::event::*;
use super::j2k::*;
use super::openjpeg::*;
use super::stream::*;

use super::malloc::*;

pub type C2RustUnnamed_2 = core::ffi::c_uint;
pub const JP2_STATE_UNKNOWN: C2RustUnnamed_2 = 2147483647;
pub const JP2_STATE_END_CODESTREAM: C2RustUnnamed_2 = 16;
pub const JP2_STATE_CODESTREAM: C2RustUnnamed_2 = 8;
pub const JP2_STATE_HEADER: C2RustUnnamed_2 = 4;
pub const JP2_STATE_FILE_TYPE: C2RustUnnamed_2 = 2;
pub const JP2_STATE_SIGNATURE: C2RustUnnamed_2 = 1;
pub const JP2_STATE_NONE: C2RustUnnamed_2 = 0;
pub type C2RustUnnamed_3 = core::ffi::c_uint;
pub const JP2_IMG_STATE_UNKNOWN: C2RustUnnamed_3 = 2147483647;
pub const JP2_IMG_STATE_NONE: C2RustUnnamed_3 = 0;

#[derive(Default, Debug, Clone, Copy)]
pub struct Jp2BoxHeader {
  pub length: u32,
  pub ty: u32,
  pub header_length: u32,
}

impl Jp2BoxHeader {
  fn new(ty: Jp2BoxType) -> Self {
    Self {
      length: 8,
      ty: ty.to_u32().unwrap(),
      header_length: 8,
    }
  }

  pub fn from_stream(reader: &mut Stream) -> Option<Self> {
    let mut header = Self::default();
    header.read(reader).ok()?;
    Some(header)
  }

  pub fn content_length(&self) -> u32 {
    self.length - self.header_length
  }

  /// Reads a box header. The box is the way data is packed inside a jpeg2000 file structure.
  pub fn read(
    &mut self,
    reader: &mut Stream,
  ) -> Result<(), String> {
    self.length = reader.read_u32::<BigEndian>().map_err(|e| format!("Truncated JP2 Box header: {e:?}"))?;
    self.ty = reader.read_u32::<BigEndian>().map_err(|e| format!("Truncated JP2 Box header: {e:?}"))?;
    self.header_length = 8;
    if self.length == 0 {
      /* last box */
      let bleft = opj_stream_get_number_byte_left(reader);
      if bleft > (u32::MAX - 8) as i64 {
        // TODO: Handle large boxes?
        return Err(format!("Cannot handle box sizes higher than 2^32"));
      }
      self.length = (bleft + 8) as u32;
      assert!(self.length as i64 == bleft + 8);
      return Ok(());
    }
    /* do we have a "special very large box ?" */
    /* read then the XLBox */
    if self.length == 1 {
      let xl_part_size = reader.read_u32::<BigEndian>().map_err(|e| format!("Truncated JP2 XLBox header: {e:?}"))?;
      let length = reader.read_u32::<BigEndian>().map_err(|e| format!("Truncated JP2 XLBox header: {e:?}"))?;
      self.header_length += 8;
      if xl_part_size != 0 {
        // TODO: Handle large boxes?
        return Err(format!("Cannot handle box sizes higher than 2^32"));
      }
      self.length = length;
    }
    Ok(())
  }

  fn write<W: Write>(&self, writer: &mut W) -> bool {
    writer.write_u32::<BigEndian>(self.length).is_ok()
      && writer.write_u32::<BigEndian>(self.ty).is_ok()
  }
}

#[derive(Copy, Clone)]
pub(crate) struct opj_jp2_header_handler {
  pub id: OPJ_UINT32,
  pub handler:
    fn(_: &mut opj_jp2, _: *mut OPJ_BYTE, _: OPJ_UINT32, _: &mut opj_event_mgr) -> OPJ_BOOL,
}

pub(crate) struct HeaderWriter {
  handler: fn(_: &mut opj_jp2, _: &mut Vec<u8>) -> bool,
  data: Vec<u8>,
}

impl HeaderWriter {
  fn new(handler: fn(_: &mut opj_jp2, _: &mut Vec<u8>) -> bool) -> Self {
    Self {
      handler: handler,
      data: Default::default(),
    }
  }

  fn run(&mut self, jp2: &mut opj_jp2) -> Option<u32> {
    if (self.handler)(jp2, &mut self.data) {
      Some(self.data.len() as u32)
    } else {
      None
    }
  }

  fn write<W: Write>(&self, writer: &mut W) -> bool {
    writer.write_all(self.data.as_slice()).is_ok()
  }
}

static jp2_header: [opj_jp2_header_handler; 3] = [
  {
    opj_jp2_header_handler {
      id: 0x6a502020,
      handler: opj_jp2_read_jp,
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x66747970,
      handler: opj_jp2_read_ftyp,
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x6a703268,
      handler: opj_jp2_read_jp2h,
    }
  },
];
static jp2_img_header: [opj_jp2_header_handler; 6] = [
  {
    opj_jp2_header_handler {
      id: 0x69686472,
      handler: opj_jp2_read_ihdr,
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x636f6c72,
      handler: opj_jp2_read_colr,
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x62706363,
      handler: opj_jp2_read_bpcc,
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x70636c72,
      handler: opj_jp2_read_pclr,
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x636d6170,
      handler: opj_jp2_read_cmap,
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x63646566,
      handler: opj_jp2_read_cdef,
    }
  },
];

/* *
 * Reads a IHDR box - Image Header box
 *
 * @param   p_image_header_data         pointer to actual data (already read from file)
 * @param   jp2                         the jpeg2000 file codec.
 * @param   p_image_header_size         the size of the image header
 * @param   p_manager                   the user event manager.
 *
 * @return  true if the image header is valid, false else.
 */
fn opj_jp2_read_ihdr(
  mut jp2: &mut opj_jp2,
  mut p_image_header_data: *mut OPJ_BYTE,
  mut p_image_header_size: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    /* preconditions */
    /* WIDTH */

    assert!(!p_image_header_data.is_null());
    if !jp2.comps.is_empty() {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "Ignoring ihdr box. First ihdr box already read\n",
      );
      return 1i32;
    }
    if p_image_header_size != 14u32 {
      event_msg!(p_manager, EVT_ERROR, "Bad image header box (bad size)\n",);
      return 0i32;
    }
    opj_read_bytes(p_image_header_data, &mut jp2.h, 4 as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(4);
    opj_read_bytes(p_image_header_data, &mut jp2.w, 4 as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(4);
    let mut numcomps = 0u32;
    opj_read_bytes(p_image_header_data, &mut numcomps, 2 as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(2);
    if jp2.h < 1u32 || jp2.w < 1u32 || numcomps < 1u32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Wrong values for: w(%d) h(%d) numcomps(%d) (ihdr)\n",
        jp2.w,
        jp2.h,
        numcomps,
      );
      return 0i32;
    }
    if numcomps.wrapping_sub(1u32) >= 16384u32 {
      /* unsigned underflow is well defined: 1U <= jp2->numcomps <= 16384U */
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Invalid number of components (ihdr)\n",
      );
      return 0;
    }
    /* allocate memory for components */
    jp2.comps = Vec::with_capacity(numcomps as usize);
    jp2
      .comps
      .resize(numcomps as usize, opj_jp2_comps::default());

    /* BPC */
    opj_read_bytes(p_image_header_data, &mut jp2.bpc, 1 as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(1);
    /* C */
    opj_read_bytes(p_image_header_data, &mut jp2.C, 1 as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(1);

    /* Should be equal to 7 cf. chapter about image header box of the norm */
    if jp2.C != 7u32 {
      event_msg!(
        p_manager,
        EVT_INFO,
        "JP2 IHDR box: compression type indicate that the file is not a conforming JP2 file (%d) \n",
        jp2.C
      );
    }
    /* UnkC */
    opj_read_bytes(p_image_header_data, &mut jp2.UnkC, 1 as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(1);
    /* IPR */
    opj_read_bytes(p_image_header_data, &mut jp2.IPR, 1 as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(1);

    jp2.j2k.m_cp.allow_different_bit_depth_sign = jp2.bpc == 255u32;
    jp2.j2k.ihdr_w = jp2.w;
    jp2.j2k.ihdr_h = jp2.h;
    jp2.has_ihdr = 1 as OPJ_BYTE;
    1i32
  }
}

/* *
 * Writes the Image Header box - Image Header box.
 *
*/
fn opj_jp2_write_ihdr(mut jp2: &mut opj_jp2, buf: &mut Vec<u8>) -> bool {
  /* IHDR */
  let mut header = Jp2BoxHeader::new(Jp2BoxType::IHDR);
  header.length += 14;
  header.write(buf);
  /* HEIGHT */
  buf.write_u32::<BigEndian>(jp2.h).unwrap();
  /* WIDTH */
  buf.write_u32::<BigEndian>(jp2.w).unwrap();
  /* NC */
  buf.write_u16::<BigEndian>(jp2.comps.len() as u16).unwrap();
  /* BPC */
  buf.push(jp2.bpc as u8);
  /* C : Always 7 */
  buf.push(jp2.C as u8);
  /* UnkC, colorspace unknown */
  buf.push(jp2.UnkC as u8);
  /* IPR, no intellectual property */
  buf.push(jp2.IPR as u8);
  true
}

/* *
 * Writes the Bit per Component box.
 *
*/
fn opj_jp2_write_bpcc(mut jp2: &mut opj_jp2, buf: &mut Vec<u8>) -> bool {
  let mut header = Jp2BoxHeader::new(Jp2BoxType::BPCC);
  header.length += jp2.comps.len() as u32;
  header.write(buf);
  for comp in &jp2.comps {
    buf.push(comp.bpcc as u8);
  }
  true
}

/* *
 * Reads a Bit per Component box.
 *
 * @param   p_bpc_header_data           pointer to actual data (already read from file)
 * @param   jp2                         the jpeg2000 file codec.
 * @param   p_bpc_header_size           the size of the bpc header
 * @param   p_manager                   the user event manager.
 *
 * @return  true if the bpc header is valid, false else.
 */
fn opj_jp2_read_bpcc(
  mut jp2: &mut opj_jp2,
  mut p_bpc_header_data: *mut OPJ_BYTE,
  mut p_bpc_header_size: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    /* preconditions */
    assert!(!p_bpc_header_data.is_null());

    if jp2.bpc != 255u32 {
      event_msg!(p_manager, EVT_WARNING,
                      "A BPCC header box is available although BPC given by the IHDR box (%d) indicate components bit depth is constant\n", jp2.bpc);
    }
    /* and length is relevant */
    if p_bpc_header_size as usize != jp2.comps.len() {
      event_msg!(p_manager, EVT_ERROR, "Bad BPCC header box (bad size)\n",);
      return 0i32;
    }
    /* read info for each component */
    for comp in &mut jp2.comps {
      /* read each BPCC component */
      opj_read_bytes(p_bpc_header_data, &mut comp.bpcc, 1 as OPJ_UINT32);
      p_bpc_header_data = p_bpc_header_data.offset(1);
    }
    1i32
  }
}

/* *
 * Writes the Channel Definition box.
 *
 */
fn opj_jp2_write_cdef(mut jp2: &mut opj_jp2, buf: &mut Vec<u8>) -> bool {
  let info = if let Some(cdef) = &jp2.color.jp2_cdef {
    &cdef.info
  } else {
    return false;
  };
  let len = info.len() as u32;
  assert!(info.len() > 0);
  let mut header = Jp2BoxHeader::new(Jp2BoxType::CDEF);
  header.length += 2 + (len * 6);
  header.write(buf);
  buf.write_u16::<BigEndian>(len as u16).unwrap();
  for info in info {
    buf.write_u16::<BigEndian>(info.cn as u16).unwrap();
    buf.write_u16::<BigEndian>(info.typ as u16).unwrap();
    buf.write_u16::<BigEndian>(info.asoc as u16).unwrap();
  }
  true
}

/* *
 * Writes the Colour Specification box.
 *
*/
fn opj_jp2_write_colr(mut jp2: &mut opj_jp2, buf: &mut Vec<u8>) -> bool {
  let meth = jp2.meth as u8;
  let mut header = Jp2BoxHeader::new(Jp2BoxType::COLR);
  header.length += 3;
  /* Meth value is restricted to 1 or 2 (Table I.9 of part 1) */
  assert!(meth == 1 || meth == 2);
  header.length += match meth {
    1 => 4,
    2 => {
      assert!(jp2.color.icc_profile_len != 0);
      jp2.color.icc_profile_len
    }
    _ => return false,
  };
  header.write(buf);
  buf.push(meth as u8);
  buf.push(jp2.precedence as u8);
  buf.push(jp2.approx as u8);
  match meth {
    1 => {
      /* EnumCS */
      buf.write_u32::<BigEndian>(jp2.enumcs as u32).unwrap();
      // TODO: Support CIELab? (enumcs == 14).
    }
    2 => {
      /* ICC profile */
      if let Some(icc_profile) = &jp2.color.icc_profile {
        buf.extend_from_slice(icc_profile.as_slice());
      } else {
        log::error!("Missing ICC profile");
      }
    }
    _ => return false,
  }
  true
}

fn opj_jp2_check_color(
  mut image: &mut opj_image_t,
  mut color: &mut opj_jp2_color,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* testcase 4149.pdf.SIGSEGV.cf7.3501 */
  if let Some(cdef) = &color.jp2_cdef {
    /* FIXME image->numcomps == jp2->numcomps before color is applied ??? */
    let mut nr_channels = image.numcomps;
    /* cdef applies to cmap channels if any */
    if let Some(pclr) = &color.jp2_pclr {
      if !pclr.cmap.is_empty() {
        nr_channels = pclr.nr_channels as u32
      }
    }
    for info in &cdef.info {
      if info.cn as u32 >= nr_channels {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Invalid component index %d (>= %d).\n",
          info.cn as i32,
          nr_channels,
        );
        return 0;
      }
      if info.asoc != 65535 && info.asoc > 0 && (info.asoc - 1) as u32 >= nr_channels {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Invalid component index %d (>= %d).\n",
          info.asoc as core::ffi::c_int - 1i32,
          nr_channels,
        );
        return 0;
      }
    }
    /* issue 397 */
    /* ISO 15444-1 states that if cdef is present, it shall contain a complete list of channel definitions. */
    for chan in 1..nr_channels {
      let has = cdef
        .info
        .iter()
        .position(|&x| x.cn as u32 == chan)
        .is_some();
      if !has {
        event_msg!(p_manager, EVT_ERROR, "Incomplete channel definitions.\n",);
        return 0;
      }
    }
  }
  /* testcases 451.pdf.SIGSEGV.f4c.3723, 451.pdf.SIGSEGV.5b5.3723 and
  66ea31acbb0f23a2bbc91f64d69a03f5_signal_sigsegv_13937c0_7030_5725.pdf */
  if let Some(pclr) = &mut color.jp2_pclr {
    let nr_channels = pclr.nr_channels as usize;
    if pclr.cmap.len() == nr_channels {
      let mut is_sane = true;
      /* verify that all original components match an existing one */
      for cmap in &pclr.cmap {
        if cmap.cmp as u32 >= image.numcomps {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Invalid component index %d (>= %d).\n",
            cmap.cmp as core::ffi::c_int,
            image.numcomps,
          );
          is_sane = false
        }
      }
      let mut pcol_usage = BTreeSet::new();
      /* verify that no component is targeted more than once */
      for i in 0..nr_channels {
        let cmap = pclr.cmap[i];
        let mut mtyp = cmap.mtyp;
        let mut pcol = cmap.pcol;
        /* See ISO 15444-1 Table I.14 – MTYPi field values */
        if mtyp != 0 && mtyp != 1 {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Invalid value for cmap[%d].mtyp = %d.\n",
            i as core::ffi::c_int,
            mtyp as core::ffi::c_int,
          );
          is_sane = false
        } else if pcol as usize >= nr_channels {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Invalid component/palette index for direct mapping %d.\n",
            pcol as core::ffi::c_int,
          );
          is_sane = false
        } else if pcol_usage.contains(&pcol) && mtyp == 1 {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Component %d is mapped twice.\n",
            pcol as core::ffi::c_int,
          );
          is_sane = false
        } else if mtyp == 0 && pcol != 0 {
          /* I.5.3.5 PCOL: If the value of the MTYP field for this channel is 0, then
           * the value of this field shall be 0. */
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Direct use at #%d however pcol=%d.\n",
            i as core::ffi::c_int,
            pcol as core::ffi::c_int,
          );
          is_sane = false
        } else if mtyp == 1 && pcol as usize != i {
          /* OpenJPEG implementation limitation. See assert(i == pcol); */
          /* in opj_jp2_apply_pclr() */
          event_msg!(p_manager, EVT_ERROR,
                              "Implementation limitation: for palette mapping, pcol[%d] should be equal to %d, but is equal to %d.\n",
                              i as core::ffi::c_int, i as core::ffi::c_int,
                              pcol as core::ffi::c_int);
          is_sane = false
        } else {
          pcol_usage.insert(pcol);
        }
      }
      /* verify that all components are targeted at least once */
      for i in 0..nr_channels {
        let cmap = pclr.cmap[i];
        if !pcol_usage.contains(&(i as u8)) && cmap.mtyp != 0 {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Component %d doesn\'t have a mapping.\n",
            i as core::ffi::c_int,
          );
          is_sane = false
        }
      }
      /* Issue 235/447 weird cmap */
      if is_sane && image.numcomps == 1 {
        for i in 0..nr_channels {
          if !pcol_usage.contains(&(i as u8)) {
            is_sane = false;
            event_msg!(
              p_manager,
              EVT_WARNING,
              "Component mapping seems wrong. Trying to correct.\n",
            );
            break;
          }
        }
        if !is_sane {
          is_sane = true;
          for i in 0..nr_channels {
            let cmap = &mut pclr.cmap[i];
            cmap.mtyp = 1 as OPJ_BYTE;
            cmap.pcol = i as OPJ_BYTE;
          }
        }
      }
      if !is_sane {
        return 0;
      }
    }
  }
  1
}

/* *
Apply collected palette data
@param image Image.
@param color Collector for profile, cdef and pclr data.
@param p_manager the user event manager.
@return true in case of success
*/
fn opj_jp2_apply_pclr(
  mut image: &mut opj_image_t,
  mut pclr: &opj_jp2_pclr,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    let mut old_comps = std::ptr::null_mut::<opj_image_comp_t>();
    let mut new_comps = std::ptr::null_mut::<opj_image_comp_t>();
    let mut src = std::ptr::null_mut::<OPJ_INT32>();
    let mut dst = std::ptr::null_mut::<OPJ_INT32>();
    let mut max: OPJ_UINT32 = 0;
    let mut cmp: OPJ_UINT16 = 0;
    let mut pcol: OPJ_UINT16 = 0;
    let mut k: OPJ_INT32 = 0;
    let nr_channels = pclr.nr_channels as usize;
    for i in 0..nr_channels {
      /* Palette mapping: */
      let cmp = pclr.cmap[i].cmp;
      if (*image.comps.offset(cmp as isize)).data.is_null() {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "image->comps[%d].data == NULL in opj_jp2_apply_pclr().\n",
          i as core::ffi::c_int,
        );
        return 0i32;
      }
    }
    old_comps = image.comps;
    new_comps =
      opj_malloc((nr_channels as usize).wrapping_mul(core::mem::size_of::<opj_image_comp_t>()))
        as *mut opj_image_comp_t;
    if new_comps.is_null() {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Memory allocation failure in opj_jp2_apply_pclr().\n",
      );
      return 0i32;
    }
    for i in 0..nr_channels {
      let cmap = pclr.cmap[i];
      let channel = pclr.channel[i as usize];
      pcol = cmap.pcol as u16;
      cmp = cmap.cmp;
      /* Direct use */
      if cmap.mtyp as core::ffi::c_int == 0i32 {
        assert!(pcol as core::ffi::c_int == 0i32);
        *new_comps.offset(i as isize) = *old_comps.offset(cmp as isize)
      } else {
        assert!(i as core::ffi::c_int == pcol as core::ffi::c_int);
        *new_comps.offset(pcol as isize) = *old_comps.offset(cmp as isize)
      }
      /* Palette mapping: */
      let fresh0 = &mut (*new_comps.offset(i as isize)).data;
      *fresh0 = opj_image_data_alloc(
        core::mem::size_of::<OPJ_INT32>()
          .wrapping_mul((*old_comps.offset(cmp as isize)).w as usize)
          .wrapping_mul((*old_comps.offset(cmp as isize)).h as usize),
      ) as *mut OPJ_INT32;
      if (*new_comps.offset(i as isize)).data.is_null() {
        for x in 0..i {
          opj_image_data_free((*new_comps.offset(x as isize)).data as *mut core::ffi::c_void);
        }
        opj_free(new_comps as *mut core::ffi::c_void);
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Memory allocation failure in opj_jp2_apply_pclr().\n",
        );
        return 0;
      }
      (*new_comps.offset(i as isize)).prec = channel.size as OPJ_UINT32;
      (*new_comps.offset(i as isize)).sgnd = channel.sign as OPJ_UINT32;
    }
    let top_k = pclr.nr_entries as i32 - 1;
    for i in 0..nr_channels {
      let cmap = pclr.cmap[i];
      /* Palette mapping: */
      cmp = cmap.cmp; /* verified above */
      pcol = cmap.pcol as OPJ_UINT16;
      src = (*old_comps.offset(cmp as isize)).data;
      assert!(!src.is_null());
      max = (*new_comps.offset(i as isize)).w * (*new_comps.offset(i as isize)).h;

      /* Direct use: */
      if cmap.mtyp as core::ffi::c_int == 0i32 {
        dst = (*new_comps.offset(i as isize)).data;
        assert!(!dst.is_null());
        for j in 0..max {
          *dst.offset(j as isize) = *src.offset(j as isize);
        }
      } else {
        assert!(i as core::ffi::c_int == pcol as core::ffi::c_int);
        dst = (*new_comps.offset(pcol as isize)).data;
        assert!(!dst.is_null());
        for j in 0..max {
          /* The index */
          k = *src.offset(j as isize);
          if k < 0i32 {
            k = 0i32
          } else if k > top_k {
            k = top_k
          }
          /* The colour */
          *dst.offset(j as isize) =
            pclr.entries[(k * nr_channels as i32 + pcol as i32) as usize] as i32;
        }
      }
    }
    max = image.numcomps;
    for j in 0..max {
      if !(*old_comps.offset(j as isize)).data.is_null() {
        opj_image_data_free((*old_comps.offset(j as isize)).data as *mut core::ffi::c_void);
      }
    }
    opj_free(old_comps as *mut core::ffi::c_void);
    image.comps = new_comps;
    image.numcomps = nr_channels as OPJ_UINT32;
    1i32
  }
}

/* *
 * Collect palette data
 *
 * @param jp2 JP2 handle
 * @param p_pclr_header_data    FIXME DOC
 * @param p_pclr_header_size    FIXME DOC
 * @param p_manager
 *
 * @return Returns true if successful, returns false otherwise
*/
fn opj_jp2_read_pclr(
  mut jp2: &mut opj_jp2,
  mut p_pclr_header_data: *mut OPJ_BYTE,
  mut p_pclr_header_size: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    let mut nr_entries: OPJ_UINT16 = 0;
    let mut nr_channels: OPJ_UINT16 = 0;
    let mut l_value: OPJ_UINT32 = 0;
    let mut orig_header_data = p_pclr_header_data;
    /* preconditions */
    assert!(!p_pclr_header_data.is_null());
    if jp2.color.jp2_pclr.is_some() {
      return 0;
    }
    if p_pclr_header_size < 3 {
      return 0;
    }
    opj_read_bytes(p_pclr_header_data, &mut l_value, 2 as OPJ_UINT32);
    p_pclr_header_data = p_pclr_header_data.offset(2);
    nr_entries = l_value as OPJ_UINT16;
    if nr_entries == 0 || nr_entries > 1024 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Invalid PCLR box. Reports %d entries\n",
        nr_entries as core::ffi::c_int,
      );
      return 0;
    }
    opj_read_bytes(p_pclr_header_data, &mut l_value, 1 as OPJ_UINT32);
    p_pclr_header_data = p_pclr_header_data.offset(1);
    nr_channels = l_value as OPJ_UINT16;
    if nr_channels == 0 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Invalid PCLR box. Reports 0 palette columns\n",
      );
      return 0;
    }
    if p_pclr_header_size < (3u32).wrapping_add(nr_channels as OPJ_UINT32) {
      return 0;
    }
    let mut entries = Vec::with_capacity((nr_channels * nr_channels) as usize);
    let mut channel = Vec::with_capacity(nr_channels as usize);
    for _ in 0..nr_channels {
      opj_read_bytes(p_pclr_header_data, &mut l_value, 1 as OPJ_UINT32);
      p_pclr_header_data = p_pclr_header_data.offset(1);
      let size = (l_value & 0x7fu32).wrapping_add(1u32) as u8;
      let sign = if l_value & 0x80u32 != 0 { 1 } else { 0 } as u8;
      channel.push(Jp2ChannelSign { size, sign });
    }
    for _ in 0..nr_entries {
      for i in 0..nr_channels {
        let size = channel[i as usize].size;
        let mut bytes_to_read = (size as u32 + 7) >> 3;
        if bytes_to_read as usize > core::mem::size_of::<OPJ_UINT32>() {
          bytes_to_read = core::mem::size_of::<OPJ_UINT32>() as OPJ_UINT32
        }
        if (p_pclr_header_size as isize)
          < p_pclr_header_data.offset_from(orig_header_data) + bytes_to_read as isize
        {
          return 0;
        }
        opj_read_bytes(p_pclr_header_data, &mut l_value, bytes_to_read);
        p_pclr_header_data = p_pclr_header_data.offset(bytes_to_read as isize);
        entries.push(l_value);
      }
    }
    let jp2_pclr = opj_jp2_pclr {
      channel,
      entries,
      nr_entries,
      nr_channels: nr_channels as _,
      cmap: Default::default(),
    };
    jp2.color.jp2_pclr = Some(jp2_pclr);
    1
  }
}

/* *
 * Collect component mapping data
 *
 * @param jp2                 JP2 handle
 * @param p_cmap_header_data  FIXME DOC
 * @param p_cmap_header_size  FIXME DOC
 * @param p_manager           FIXME DOC
 *
 * @return Returns true if successful, returns false otherwise
*/
fn opj_jp2_read_cmap(
  mut jp2: &mut opj_jp2,
  mut p_cmap_header_data: *mut OPJ_BYTE,
  mut p_cmap_header_size: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    let mut l_value: OPJ_UINT32 = 0;
    /* preconditions */

    assert!(!p_cmap_header_data.is_null());
    /* Need nr_channels: */
    let mut pclr = if let Some(pclr) = &mut jp2.color.jp2_pclr {
      pclr
    } else {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Need to read a PCLR box before the CMAP box.\n",
      );
      return 0i32;
    };
    /* Part 1, I.5.3.5: 'There shall be at most one Component Mapping box
     * inside a JP2 Header box' :
     */
    if !pclr.cmap.is_empty() {
      event_msg!(p_manager, EVT_ERROR, "Only one CMAP box is allowed.\n",); /* CMP^i */
      return 0i32;
    }
    let nr_channels = pclr.nr_channels as usize;
    if p_cmap_header_size < (nr_channels as OPJ_UINT32).wrapping_mul(4u32) {
      event_msg!(p_manager, EVT_ERROR, "Insufficient data for CMAP box.\n",);
      return 0i32;
    }
    let mut cmap = Vec::with_capacity(nr_channels);
    for _ in 0..nr_channels {
      /* CMP^i */
      opj_read_bytes(p_cmap_header_data, &mut l_value, 2 as OPJ_UINT32);
      p_cmap_header_data = p_cmap_header_data.offset(2);
      let cmp = l_value as OPJ_UINT16;
      /* MTYP^i */
      opj_read_bytes(p_cmap_header_data, &mut l_value, 1 as OPJ_UINT32);
      p_cmap_header_data = p_cmap_header_data.offset(1);
      let mtyp = l_value as OPJ_BYTE;
      /* PCOL^i */
      opj_read_bytes(p_cmap_header_data, &mut l_value, 1 as OPJ_UINT32);
      p_cmap_header_data = p_cmap_header_data.offset(1);
      let pcol = l_value as OPJ_BYTE;
      cmap.push(opj_jp2_cmap_comp { cmp, mtyp, pcol });
    }
    pclr.cmap = cmap;
    1i32
  }
}

fn opj_jp2_apply_cdef(
  mut image: &mut opj_image_t,
  mut color: &mut opj_jp2_color,
  mut manager: &mut opj_event_mgr,
) {
  let cdef = if let Some(cdef) = &mut color.jp2_cdef {
    cdef
  } else {
    return;
  };
  unsafe {
    let n = cdef.info.len();
    for i in 0..n {
      let info = cdef.info[i];
      /* WATCH: acn = asoc - 1 ! */
      let asoc = info.asoc;
      let cn = info.cn;
      if cn as core::ffi::c_uint >= image.numcomps {
        event_msg!(
          manager,
          EVT_WARNING,
          "opj_jp2_apply_cdef: cn=%d, numcomps=%d\n",
          cn as core::ffi::c_int,
          image.numcomps,
        );
      } else if asoc as core::ffi::c_int == 0i32 || asoc as core::ffi::c_int == 65535i32 {
        (*image.comps.offset(cn as isize)).alpha = info.typ
      } else {
        let acn = asoc - 1;
        if acn as u32 >= image.numcomps {
          event_msg!(
            manager,
            EVT_WARNING,
            "opj_jp2_apply_cdef: acn=%d, numcomps=%d\n",
            acn as core::ffi::c_int,
            image.numcomps,
          );
        } else {
          let cn_comp = &mut *image.comps.offset(cn as isize);
          /* Swap only if color channel */
          if cn != acn && info.typ == 0 {
            let acn_comp = &mut *image.comps.offset(acn as isize);
            let saved = *cn_comp;
            *cn_comp = *acn_comp;
            *acn_comp = saved;
            /* Swap channels in following channel definitions, don't bother with j <= i that are already processed */
            for j in i..n {
              let info = &mut cdef.info[j];
              if info.cn == cn {
                info.cn = acn
              } else if info.cn == acn {
                info.cn = cn
              }
              /* asoc is related to color index. Do not update. */
            }
          }
          cn_comp.alpha = info.typ
        }
      }
    }
    cdef.info.clear();
    color.jp2_cdef = None;
  }
}

/* jp2_apply_cdef() */
fn opj_jp2_read_cdef(
  mut jp2: &mut opj_jp2,
  mut p_cdef_header_data: *mut OPJ_BYTE,
  mut p_cdef_header_size: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    let mut l_value: OPJ_UINT32 = 0;
    /* preconditions */

    assert!(!p_cdef_header_data.is_null());
    /* Part 1, I.5.3.6: 'The shall be at most one Channel Definition box
     * inside a JP2 Header box.'*/
    if jp2.color.jp2_cdef.is_some() {
      return 0i32;
    } /* N */
    if p_cdef_header_size < 2u32 {
      event_msg!(p_manager, EVT_ERROR, "Insufficient data for CDEF box.\n",);
      return 0i32;
    }
    opj_read_bytes(p_cdef_header_data, &mut l_value, 2 as OPJ_UINT32);
    p_cdef_header_data = p_cdef_header_data.offset(2);
    if l_value as OPJ_UINT16 as core::ffi::c_int == 0i32 {
      /* szukw000: FIXME */
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Number of channel description is equal to zero in CDEF box.\n",
      ); /* Cn^i */
      return 0i32;
    } /* Typ^i */
    if p_cdef_header_size
      < (2u32).wrapping_add((l_value as OPJ_UINT16 as OPJ_UINT32).wrapping_mul(6u32))
    {
      event_msg!(p_manager, EVT_ERROR, "Insufficient data for CDEF box.\n",); /* Asoc^i */
      return 0i32;
    }
    let n = l_value as usize;
    let mut info = Vec::with_capacity(n);
    for _i in 0..n {
      opj_read_bytes(p_cdef_header_data, &mut l_value, 2 as OPJ_UINT32);
      p_cdef_header_data = p_cdef_header_data.offset(2);
      let cn = l_value as OPJ_UINT16;
      opj_read_bytes(p_cdef_header_data, &mut l_value, 2 as OPJ_UINT32);
      p_cdef_header_data = p_cdef_header_data.offset(2);
      let typ = l_value as OPJ_UINT16;
      opj_read_bytes(p_cdef_header_data, &mut l_value, 2 as OPJ_UINT32);
      p_cdef_header_data = p_cdef_header_data.offset(2);
      let asoc = l_value as OPJ_UINT16;
      info.push(opj_jp2_cdef_info { cn, typ, asoc });
    }
    jp2.color.jp2_cdef = Some(opj_jp2_cdef { info });
    1i32
  }
}

/* *
 * Reads the Color Specification box.
 *
 * @param   p_colr_header_data          pointer to actual data (already read from file)
 * @param   jp2                         the jpeg2000 file codec.
 * @param   p_colr_header_size          the size of the color header
 * @param   p_manager                   the user event manager.
 *
 * @return  true if the bpc header is valid, false else.
*/
fn opj_jp2_read_colr(
  mut jp2: &mut opj_jp2,
  mut p_colr_header_data: *mut OPJ_BYTE,
  mut p_colr_header_size: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    /* preconditions */

    assert!(!p_colr_header_data.is_null());
    if p_colr_header_size < 3u32 {
      event_msg!(p_manager, EVT_ERROR, "Bad COLR header box (bad size)\n",);
      return 0i32;
    }
    /* Part 1, I.5.3.3 : 'A conforming JP2 reader shall ignore all Colour
     * Specification boxes after the first.'
     */
    if jp2.color.jp2_has_colr != 0 {
      event_msg!(p_manager, EVT_INFO,
                      "A conforming JP2 reader shall ignore all Colour Specification boxes after the first, so we ignore this one.\n"); /* METH */
      p_colr_header_data = p_colr_header_data.offset(p_colr_header_size as isize); /* PRECEDENCE */
      return 1i32;
    } /* APPROX */
    opj_read_bytes(p_colr_header_data, &mut jp2.meth, 1 as OPJ_UINT32);
    p_colr_header_data = p_colr_header_data.offset(1);
    opj_read_bytes(p_colr_header_data, &mut jp2.precedence, 1 as OPJ_UINT32);
    p_colr_header_data = p_colr_header_data.offset(1);
    opj_read_bytes(p_colr_header_data, &mut jp2.approx, 1 as OPJ_UINT32);
    p_colr_header_data = p_colr_header_data.offset(1);
    if jp2.meth == 1u32 {
      if p_colr_header_size < 7u32 {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Bad COLR header box (bad size: %d)\n",
          p_colr_header_size,
        );
        return 0i32;
      }
      if p_colr_header_size > 7u32 && jp2.enumcs != 14u32 {
        /* handled below for CIELab) */
        /* testcase Altona_Technical_v20_x4.pdf */
        event_msg!(
          p_manager,
          EVT_WARNING,
          "Bad COLR header box (bad size: %d)\n",
          p_colr_header_size,
        ); /* EnumCS */
      }
      opj_read_bytes(p_colr_header_data, &mut jp2.enumcs, 4 as OPJ_UINT32);
      p_colr_header_data = p_colr_header_data.offset(4);
      if jp2.enumcs == 14u32 {
        /* CIELab */
        /* default values */
        let mut rl = 0u32;
        let mut ol = 0u32;
        let mut ra = 0u32;
        let mut oa = 0u32;
        let mut rb = 0u32;
        let mut ob = 0u32;
        let mut il = 0x443530u32;
        let mut icc_profile = Vec::with_capacity(9 * core::mem::size_of::<OPJ_UINT32>());
        icc_profile.write_all(&14u32.to_ne_bytes()).unwrap();
        if p_colr_header_size == 35u32 {
          opj_read_bytes(p_colr_header_data, &mut rl, 4 as OPJ_UINT32);
          p_colr_header_data = p_colr_header_data.offset(4);
          opj_read_bytes(p_colr_header_data, &mut ol, 4 as OPJ_UINT32);
          p_colr_header_data = p_colr_header_data.offset(4);
          opj_read_bytes(p_colr_header_data, &mut ra, 4 as OPJ_UINT32);
          p_colr_header_data = p_colr_header_data.offset(4);
          opj_read_bytes(p_colr_header_data, &mut oa, 4 as OPJ_UINT32);
          p_colr_header_data = p_colr_header_data.offset(4);
          opj_read_bytes(p_colr_header_data, &mut rb, 4 as OPJ_UINT32);
          p_colr_header_data = p_colr_header_data.offset(4);
          opj_read_bytes(p_colr_header_data, &mut ob, 4 as OPJ_UINT32);
          p_colr_header_data = p_colr_header_data.offset(4);
          opj_read_bytes(p_colr_header_data, &mut il, 4 as OPJ_UINT32);
          p_colr_header_data = p_colr_header_data.offset(4);
          icc_profile.write_all(&0u32.to_ne_bytes()).unwrap();
        } else if p_colr_header_size != 7u32 {
          icc_profile.write_all(&0x44454600u32.to_ne_bytes()).unwrap();
          event_msg!(
            p_manager,
            EVT_WARNING,
            "Bad COLR header box (CIELab, bad size: %d)\n",
            p_colr_header_size,
          );
        }
        icc_profile.write_all(&rl.to_ne_bytes()).unwrap();
        icc_profile.write_all(&ol.to_ne_bytes()).unwrap();
        icc_profile.write_all(&ra.to_ne_bytes()).unwrap();
        icc_profile.write_all(&oa.to_ne_bytes()).unwrap();
        icc_profile.write_all(&rb.to_ne_bytes()).unwrap();
        icc_profile.write_all(&ob.to_ne_bytes()).unwrap();
        icc_profile.write_all(&il.to_ne_bytes()).unwrap();
        jp2.color.icc_profile = Some(icc_profile);
        jp2.color.icc_profile_len = 0 as OPJ_UINT32
      }
      jp2.color.jp2_has_colr = 1 as OPJ_BYTE
    } else if jp2.meth == 2u32 {
      /* ICC profile */
      let mut icc_len = p_colr_header_size as OPJ_INT32 - 3i32;
      let mut icc_profile = Vec::with_capacity(icc_len as usize);
      let buf = std::slice::from_raw_parts(p_colr_header_data, icc_len as usize);
      jp2.color.icc_profile_len = icc_len as OPJ_UINT32;
      icc_profile.extend_from_slice(buf);
      jp2.color.icc_profile = Some(icc_profile);
      jp2.color.jp2_has_colr = 1 as OPJ_BYTE
    } else if jp2.meth > 2u32 {
      /*  ISO/IEC 15444-1:2004 (E), Table I.9 Legal METH values:
      conforming JP2 reader shall ignore the entire Colour Specification box.*/
      event_msg!(p_manager, EVT_INFO,
                      "COLR BOX meth value is not a regular value (%d), so we will ignore the entire Colour Specification box. \n", jp2.meth);
    }
    1i32
  }
}

pub(crate) fn opj_jp2_apply_color_postprocessing(
  mut jp2: &mut opj_jp2,
  mut p_image: &mut opj_image,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    if jp2.j2k.m_specific_param.m_decoder.m_numcomps_to_decode != 0 {
      /* Bypass all JP2 component transforms */
      return 1i32;
    }
    if jp2.ignore_pclr_cmap_cdef == 0 {
      if opj_jp2_check_color(p_image, &mut jp2.color, p_manager) == 0 {
        return 0i32;
      }

      if let Some(pclr) = &jp2.color.jp2_pclr {
        /* Part 1, I.5.3.4: Either both or none : */
        if pclr.cmap.is_empty() {
          jp2.color.jp2_pclr = None;
        } else if opj_jp2_apply_pclr(p_image, pclr, p_manager) == 0 {
          return 0i32;
        }
      }
      /* Apply the color space if needed */
      if jp2.color.jp2_cdef.is_some() {
        opj_jp2_apply_cdef(p_image, &mut jp2.color, p_manager);
      }
    }
    1i32
  }
}

pub(crate) fn opj_jp2_decode(
  mut jp2: &mut opj_jp2,
  mut p_stream: &mut Stream,
  mut p_image: &mut opj_image,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* J2K decoding */
  if opj_j2k_decode(&mut jp2.j2k, p_stream, p_image, p_manager) == 0 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Failed to decode the codestream in the JP2 file\n",
    );
    return 0i32;
  }

  opj_jp2_apply_color_postprocessing(jp2, p_image, p_manager)
}

/* *
 * Writes the Jpeg2000 file Header box - JP2 Header box (warning, this is a super box).
 *
 * @param  jp2      the jpeg2000 file codec.
 * @param  stream      the stream to write data to.
 * @param  p_manager  user event manager.
 *
 * @return true if writing was successful.
 */
fn opj_jp2_write_jp2h(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut writers = Vec::with_capacity(4);
  // JP2H box type
  let mut jp2h = Jp2BoxHeader::new(Jp2BoxType::JP2H);

  writers.push(HeaderWriter::new(opj_jp2_write_ihdr));
  if jp2.bpc == 255u32 {
    writers.push(HeaderWriter::new(opj_jp2_write_bpcc));
  }
  writers.push(HeaderWriter::new(opj_jp2_write_colr));
  if jp2.color.jp2_cdef.is_some() {
    writers.push(HeaderWriter::new(opj_jp2_write_cdef));
  }
  /* write box header */
  for writer in &mut writers {
    match writer.run(jp2) {
      Some(size) => {
        jp2h.length += size;
      }
      None => {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Not enough memory to hold JP2 Header data\n",
        );
        return 0i32;
      }
    }
  }
  // write super box header to stream
  if !jp2h.write(stream) {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Stream error while writing JP2 Header box\n",
    );
    return 0;
  }
  for writer in &writers {
    if !writer.write(stream) {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Stream error while writing JP2 Header box\n",
      );
      return 0i32;
    }
  }
  1
}

/* *
 * Writes a FTYP box - File type box
 *
 * @param   stream         the stream to write data to.
 * @param   jp2         the jpeg2000 file codec.
 * @param   p_manager   the user event manager.
 *
 * @return  true if writing was successful.
 */
fn opj_jp2_write_ftyp(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* FTYP */
  let mut header = Jp2BoxHeader::new(Jp2BoxType::FTYP);
  header.length += 8 + (4 * jp2.cl.len() as u32);
  let mut buf = Vec::with_capacity(header.length as usize);
  header.write(&mut buf);

  buf.write_u32::<BigEndian>(jp2.brand).unwrap();
  buf.write_u32::<BigEndian>(jp2.minversion).unwrap();
  /* CL */
  for cl in &jp2.cl {
    buf.write_u32::<BigEndian>(*cl).unwrap();
  }
  if stream.write_all(buf.as_slice()).is_err() {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error while writing ftyp data to stream\n",
    );
    return 0;
  }
  1
}

/* *
 * Writes the Jpeg2000 codestream Header box - JP2C Header box. This function must be called AFTER the coding has been done.
 *
 * @param   stream         the stream to write data to.
 * @param   jp2         the jpeg2000 file codec.
 * @param   p_manager   user event manager.
 *
 * @return true if writing was successful.
*/
fn opj_jp2_write_jp2c(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  assert!(opj_stream_has_seek(stream) != 0);
  let j2k_codestream_exit = opj_stream_tell(stream);
  let mut header = Jp2BoxHeader::new(Jp2BoxType::JP2C);
  header.length = (j2k_codestream_exit - jp2.j2k_codestream_offset) as u32;
  if opj_stream_seek(stream, jp2.j2k_codestream_offset, p_manager) == 0 {
    event_msg!(p_manager, EVT_ERROR, "Failed to seek in the stream.\n",);
    return 0;
  }
  if !header.write(stream) {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error while writing jp2c header to stream\n",
    );
    return 0;
  }
  if opj_stream_seek(stream, j2k_codestream_exit, p_manager) == 0 {
    event_msg!(p_manager, EVT_ERROR, "Failed to seek in the stream.\n",);
    return 0;
  }
  1
}

/* *
 * Writes a jpeg2000 file signature box.
 *
 * @param stream the stream to write data to.
 * @param   jp2         the jpeg2000 file codec.
 * @param p_manager the user event manager.
 *
 * @return true if writing was successful.
 */
fn opj_jp2_write_jp(
  mut _jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut tmp_buf = [0u8; 12];
  let mut buf = &mut tmp_buf[..];
  let mut header = Jp2BoxHeader::new(Jp2BoxType::JP);
  header.length += 4;
  header.write(&mut buf);
  // writes magic number
  buf.write_u32::<BigEndian>(0xd0a870a).unwrap();

  if stream.write_all(&tmp_buf).is_err() {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error while writing jp data to stream\n",
    );
    return 0;
  }
  1
}

/* ----------------------------------------------------------------------- */
/* JP2 decoder interface                                             */
/* ----------------------------------------------------------------------- */
pub(crate) fn opj_jp2_setup_decoder(mut jp2: &mut opj_jp2, mut parameters: &mut opj_dparameters_t) {
  /* setup the J2K codec */
  opj_j2k_setup_decoder(&mut jp2.j2k, parameters);
  /* further JP2 initializations go here */
  jp2.color.jp2_has_colr = 0 as OPJ_BYTE;
  jp2.ignore_pclr_cmap_cdef = (parameters.flags & 0x1u32) as OPJ_BOOL;
}

pub(crate) fn opj_jp2_decoder_set_strict_mode(mut jp2: &mut opj_jp2, mut strict: OPJ_BOOL) {
  opj_j2k_decoder_set_strict_mode(&mut jp2.j2k, strict);
}

pub(crate) fn opj_jp2_set_threads(mut jp2: &mut opj_jp2, mut num_threads: OPJ_UINT32) -> OPJ_BOOL {
  opj_j2k_set_threads(&mut jp2.j2k, num_threads)
}

/* ----------------------------------------------------------------------- */
/* JP2 encoder interface                                             */
/* ----------------------------------------------------------------------- */
pub(crate) fn opj_jp2_setup_encoder(
  mut jp2: &mut opj_jp2,
  mut parameters: &mut opj_cparameters_t,
  mut image: &mut opj_image_t,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* setup the J2K codec */
  /* ------------------- */
  /* Check if number of components respects standard */
  let numcomps = image.numcomps;
  if numcomps < 1 || numcomps > 16384 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid number of components specified while setting up JP2 encoder\n",
    );
    return 0;
  }
  if opj_j2k_setup_encoder(&mut jp2.j2k, parameters, image, p_manager) == 0 {
    return 0;
  }
  /* setup the JP2 codec */

  /* Profile box */

  jp2.brand = Jp2BoxType::JP2.to_u32().unwrap(); /* BR */
  jp2.minversion = 0 as OPJ_UINT32; /* MinV */
  jp2.cl = Vec::with_capacity(1);
  /* CL0 : JP2 */
  jp2.cl.push(Jp2BoxType::JP2.to_u32().unwrap());

  /* Image Header box */
  let comps = image.comps().unwrap();
  jp2.comps = Vec::with_capacity(comps.len());
  jp2.comps.resize(comps.len(), opj_jp2_comps::default());

  /* HEIGHT */
  jp2.h = image.y1.wrapping_sub(image.y0);
  /* WIDTH */
  jp2.w = image.x1.wrapping_sub(image.x0);

  /* BPC */
  let depth_0 = comps[0].prec.wrapping_sub(1);
  let mut sign = comps[0].sgnd;
  jp2.bpc = depth_0.wrapping_add(sign << 7);
  for comp in &comps[1..] {
    let mut depth = comp.prec.wrapping_sub(1);
    sign = comp.sgnd;
    if depth_0 != depth {
      jp2.bpc = 255 as OPJ_UINT32
    }
  }
  /* C : Always 7 */
  jp2.C = 7 as OPJ_UINT32;
  /* UnkC, colorspace specified in colr box */
  jp2.UnkC = 0 as OPJ_UINT32;
  /* IPR, no intellectual property */
  jp2.IPR = 0 as OPJ_UINT32;

  /* BitsPerComponent box */
  for (j_comp, comp) in jp2.comps.iter_mut().zip(&comps[..]) {
    j_comp.bpcc = comp.prec.wrapping_sub(1).wrapping_add(comp.sgnd << 7);
  }

  /* Colour Specification box */
  if image.icc_profile_len != 0 {
    jp2.meth = 2 as OPJ_UINT32;
    jp2.enumcs = 0 as OPJ_UINT32
  } else {
    jp2.meth = 1 as OPJ_UINT32;
    if image.color_space as core::ffi::c_int == 1 {
      jp2.enumcs = 16 as OPJ_UINT32
    /* sRGB as defined by IEC 61966-2-1 */
    } else if image.color_space as core::ffi::c_int == 2 {
      jp2.enumcs = 17 as OPJ_UINT32
    /* greyscale */
    } else if image.color_space as core::ffi::c_int == 3 {
      jp2.enumcs = 18 as OPJ_UINT32
      /* YUV */
    }
  }
  /* Channel Definition box */
  /* FIXME not provided by parameters */
  /* We try to do what we can... */
  let mut alpha_count = 0;
  let mut alpha_channel = 0;
  let mut color_channels = 0u32;
  for (i, comp) in comps.iter().enumerate() {
    if comp.alpha != 0 {
      alpha_count = alpha_count + 1;
      alpha_channel = i as u32
    }
  }
  if alpha_count == 1 {
    /* no way to deal with more than 1 alpha channel */
    match jp2.enumcs {
      16 | 18 => color_channels = 3,
      17 => color_channels = 1,
      _ => alpha_count = 0,
    }
    if alpha_count == 0 {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "Alpha channel specified but unknown enumcs. No cdef box will be created.\n",
      );
    } else if numcomps < color_channels + 1 {
      event_msg!(p_manager, EVT_WARNING,
                          "Alpha channel specified but not enough image components for an automatic cdef box creation.\n");
      alpha_count = 0
    } else if alpha_channel < color_channels {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "Alpha channel position conflicts with color channel. No cdef box will be created.\n",
      );
      alpha_count = 0
    }
  } else if alpha_count > 1 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "Multiple alpha channels specified. No cdef box will be created.\n",
    );
  }
  if alpha_count == 1 {
    /* if here, we know what we can do */
    let len = numcomps as usize;
    let mut cdef = opj_jp2_cdef {
      info: Vec::with_capacity(len),
    };
    /* no memset needed, all values will be overwritten except if jp2->color.jp2_cdef->info allocation fails, */
    /* in which case jp2->color.jp2_cdef->info will be NULL => valid for destruction */
    for i in 0..len {
      let cn = i as u16;
      if i < color_channels as usize {
        cdef.info.push(opj_jp2_cdef_info {
          cn,
          typ: 0,
          asoc: cn + 1,
        })
      } else {
        if comps[i].alpha != 0 {
          /* we'll be here exactly once */
          cdef.info.push(opj_jp2_cdef_info {
            cn,
            typ: 1,  /* Opacity channel */
            asoc: 0, /* Apply alpha channel to the whole image */
          })
        } else {
          /* Unknown channel */
          cdef.info.push(opj_jp2_cdef_info {
            cn,
            typ: u16::MAX,
            asoc: u16::MAX,
          })
        }
      }
    }
    jp2.color.jp2_cdef = Some(cdef);
  }
  jp2.precedence = 0 as OPJ_UINT32;
  jp2.approx = 0 as OPJ_UINT32;
  jp2.jpip_on = parameters.jpip_on;
  1
}

pub(crate) fn opj_jp2_encode(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  opj_j2k_encode(&mut jp2.j2k, stream, p_manager)
}

pub(crate) fn opj_jp2_end_decompress(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* preconditions */
  let mut procedure_list = opj_jp2_proc_list_t::new();

  /* customization of the end encoding */
  if opj_jp2_setup_end_header_reading(jp2, &mut procedure_list, p_manager) == 0 {
    return 0i32;
  }
  /* write header */
  if opj_jp2_exec(jp2, &mut procedure_list, stream, p_manager) == 0 {
    return 0i32;
  }
  opj_j2k_end_decompress(&mut jp2.j2k, stream, p_manager)
}

pub(crate) fn opj_jp2_end_compress(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut procedure_list = opj_jp2_proc_list_t::new();
  /* preconditions */

  /* customization of the end encoding */
  if opj_jp2_setup_end_header_writing(jp2, &mut procedure_list, p_manager) == 0 {
    return 0i32;
  }
  if opj_j2k_end_compress(&mut jp2.j2k, stream, p_manager) == 0 {
    return 0i32;
  }
  /* write header */
  opj_jp2_exec(jp2, &mut procedure_list, stream, p_manager)
}

/* *
 * Sets up the procedures to do on writing header after the codestream.
 * Developers wanting to extend the library can add their own writing procedures.
 */
fn opj_jp2_setup_end_header_writing(
  _jp2: &mut opj_jp2,
  list: &mut opj_jp2_proc_list_t,
  _p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  list.add(opj_jp2_write_jp2c);
  /* DEVELOPER CORNER, add your custom procedures */
  1i32
}

/* *
 * Sets up the procedures to do on reading header after the codestream.
 * Developers wanting to extend the library can add their own writing procedures.
 */
fn opj_jp2_setup_end_header_reading(
  _jp2: &mut opj_jp2,
  list: &mut opj_jp2_proc_list_t,
  _p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  list.add(opj_jp2_read_header_procedure);
  /* DEVELOPER CORNER, add your custom procedures */
  1i32
}

fn opj_jp2_default_validation(
  jp2: &mut opj_jp2,
  stream: &mut Stream,
  _p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut l_is_valid = 1i32;

  /* JPEG2000 codec validation */
  /* STATE checking */
  /* make sure the state is at 0 */
  l_is_valid &= (jp2.jp2_state == JP2_STATE_NONE as core::ffi::c_uint) as core::ffi::c_int;
  /* make sure not reading a jp2h ???? WEIRD */
  l_is_valid &= (jp2.jp2_img_state == JP2_IMG_STATE_NONE as core::ffi::c_uint) as core::ffi::c_int;
  /* PARAMETER VALIDATION */
  /* number of components */
  l_is_valid &= (jp2.cl.len() > 0) as core::ffi::c_int;
  /* width */
  l_is_valid &= (jp2.h > 0u32) as core::ffi::c_int;
  /* height */
  l_is_valid &= (jp2.w > 0u32) as core::ffi::c_int;
  /* precision */
  for i in 0..jp2.comps.len() {
    l_is_valid &= ((jp2.comps[i].bpcc & 0x7fu32) < 38u32) as core::ffi::c_int;
    /* 0 is valid, ignore sign for check */
  }
  /* METH */
  l_is_valid &= (jp2.meth > 0u32 && jp2.meth < 3u32) as core::ffi::c_int;
  /* stream validation */
  /* back and forth is needed */
  l_is_valid &= opj_stream_has_seek(stream);
  l_is_valid
}

/* *
 * Reads a jpeg2000 file header structure.
 *
 * @param jp2 the jpeg2000 file header structure.
 * @param stream the stream to read data from.
 * @param p_manager the user event manager.
 *
 * @return true if the box is valid.
 */
fn opj_jp2_read_header_procedure(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut data = Vec::<u8>::new();

  while let Some(header) = Jp2BoxHeader::from_stream(stream) {
    /* is it the codestream box ? */
    if header.ty == 0x6a703263u32 {
      if jp2.jp2_state & JP2_STATE_HEADER as core::ffi::c_uint != 0 {
        jp2.jp2_state |= JP2_STATE_CODESTREAM as core::ffi::c_uint;
        return 1i32;
      } else {
        event_msg!(p_manager, EVT_ERROR, "bad placed jpeg codestream\n",);
        return 0i32;
      }
    } else if header.length == 0u32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Cannot handle box of undefined sizes\n",
      );
      return 0i32;
    } else {
      /* testcase 1851.pdf.SIGSEGV.ce9.948 */
      if header.length < header.header_length {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "invalid box size %d (%x)\n",
          header.length,
          header.ty,
        );
        return 0i32;
      }
    }
    let mut l_current_handler = opj_jp2_find_handler(header.ty);
    let l_current_handler_misplaced = opj_jp2_img_find_handler(header.ty);
    let data_size = header.content_length() as usize;
    if l_current_handler.is_some() || l_current_handler_misplaced.is_some() {
      if l_current_handler.is_none() {
        event_msg!(
          p_manager,
          EVT_WARNING,
          "Found a misplaced \'%c%c%c%c\' box outside jp2h box\n",
          (header.ty >> 24i32) as OPJ_BYTE as core::ffi::c_int,
          (header.ty >> 16i32) as OPJ_BYTE as core::ffi::c_int,
          (header.ty >> 8i32) as OPJ_BYTE as core::ffi::c_int,
          header.ty as OPJ_BYTE as core::ffi::c_int,
        );
        if jp2.jp2_state & JP2_STATE_HEADER as core::ffi::c_uint != 0 {
          /* read anyway, we already have jp2h */
          l_current_handler = l_current_handler_misplaced
        } else {
          event_msg!(
            p_manager,
            EVT_WARNING,
            "JPEG2000 Header box not read yet, \'%c%c%c%c\' box will be ignored\n",
            (header.ty >> 24i32) as OPJ_BYTE as core::ffi::c_int,
            (header.ty >> 16i32) as OPJ_BYTE as core::ffi::c_int,
            (header.ty >> 8i32) as OPJ_BYTE as core::ffi::c_int,
            header.ty as OPJ_BYTE as core::ffi::c_int,
          );
          jp2.jp2_state |= JP2_STATE_UNKNOWN as core::ffi::c_uint;
          if opj_stream_skip(stream, data_size as OPJ_OFF_T, p_manager) != data_size as i64 {
            event_msg!(
              p_manager,
              EVT_ERROR,
              "Problem with skipping JPEG2000 box, stream error\n",
            );
            return 0i32;
          }
          continue;
        }
      }
      if data_size as OPJ_OFF_T > opj_stream_get_number_byte_left(stream) {
        /* do not even try to malloc if we can't read */
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Invalid box size %d for box \'%c%c%c%c\'. Need %d bytes, %d bytes remaining \n",
          header.length,
          (header.ty >> 24i32) as OPJ_BYTE as core::ffi::c_int,
          (header.ty >> 16i32) as OPJ_BYTE as core::ffi::c_int,
          (header.ty >> 8i32) as OPJ_BYTE as core::ffi::c_int,
          header.ty as OPJ_BYTE as core::ffi::c_int,
          data_size,
          opj_stream_get_number_byte_left(stream) as OPJ_UINT32,
        );
        return 0i32;
      }
      data.resize(data_size as usize, 0);
      if stream.read_exact(data.as_mut_slice()).is_err() {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Problem with reading JPEG2000 box, stream error\n",
        );
        return 0i32;
      }
      if (l_current_handler.unwrap().handler)(jp2, data.as_mut_ptr(), data_size as u32, p_manager)
        == 0
      {
        return 0i32;
      }
    } else {
      if jp2.jp2_state & JP2_STATE_SIGNATURE as core::ffi::c_uint == 0 {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Malformed JP2 file format: first box must be JPEG 2000 signature box\n",
        );
        return 0i32;
      }
      if jp2.jp2_state & JP2_STATE_FILE_TYPE as core::ffi::c_uint == 0 {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Malformed JP2 file format: second box must be file type box\n",
        );
        return 0i32;
      }
      jp2.jp2_state |= JP2_STATE_UNKNOWN as core::ffi::c_uint;
      if opj_stream_skip(stream, data_size as OPJ_OFF_T, p_manager) != data_size as i64 {
        if jp2.jp2_state & JP2_STATE_CODESTREAM as core::ffi::c_uint != 0 {
          /* If we already read the codestream, do not error out */
          /* Needed for data/input/nonregression/issue254.jp2 */
          event_msg!(
            p_manager,
            EVT_WARNING,
            "Problem with skipping JPEG2000 box, stream error\n",
          );
          return 1i32;
        } else {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Problem with skipping JPEG2000 box, stream error\n",
          );
          return 0i32;
        }
      }
    }
  }
  1i32
}

/* *
 * Executes the given procedures on the given codec.
 *
 * @param   p_procedure_list    the list of procedures to execute
 * @param   jp2                 the jpeg2000 file codec to execute the procedures on.
 * @param   stream                  the stream to execute the procedures on.
 * @param   p_manager           the user manager.
 *
 * @return  true                if all the procedures were successfully executed.
 */
fn opj_jp2_exec(
  jp2: &mut opj_jp2,
  list: &mut opj_jp2_proc_list_t,
  stream: &mut Stream,
  p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  list.execute(|p| (p)(jp2, stream, p_manager) != 0) as i32
}

pub(crate) fn opj_jp2_start_compress(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_image: &mut opj_image,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut validation_list = opj_jp2_proc_list_t::new();
  let mut procedure_list = opj_jp2_proc_list_t::new();
  /* preconditions */

  /* customization of the validation */
  if opj_jp2_setup_encoding_validation(jp2, &mut validation_list, p_manager) == 0 {
    return 0i32;
  }
  /* validation of the parameters codec */
  if opj_jp2_exec(jp2, &mut validation_list, stream, p_manager) == 0 {
    return 0i32;
  }
  /* customization of the encoding */
  if opj_jp2_setup_header_writing(jp2, &mut procedure_list, p_manager) == 0 {
    return 0i32;
  }
  /* write header */
  if opj_jp2_exec(jp2, &mut procedure_list, stream, p_manager) == 0 {
    return 0i32;
  }
  opj_j2k_start_compress(&mut jp2.j2k, stream, p_image, p_manager)
}

/* *
 * Finds the execution function related to the given box id.
 *
 * @param   p_id    the id of the handler to fetch.
 *
 * @return  the given handler or NULL if it could not be found.
 */
fn opj_jp2_find_handler(mut p_id: OPJ_UINT32) -> Option<opj_jp2_header_handler> {
  for handler in jp2_header {
    if handler.id == p_id {
      return Some(handler);
    }
  }
  None
}

/* *
 * Finds the image execution function related to the given box id.
 *
 * @param   p_id    the id of the handler to fetch.
 *
 * @return  the given handler or 00 if it could not be found.
 */
fn opj_jp2_img_find_handler(mut p_id: OPJ_UINT32) -> Option<opj_jp2_header_handler> {
  for handler in jp2_img_header {
    if handler.id == p_id {
      return Some(handler);
    }
  }
  None
}

/* *
 * Reads a jpeg2000 file signature box.
 *
 * @param   p_header_data   the data contained in the signature box.
 * @param   jp2             the jpeg2000 file codec.
 * @param   p_header_size   the size of the data contained in the signature box.
 * @param   p_manager       the user event manager.
 *
 * @return true if the file signature box is valid.
 */
fn opj_jp2_read_jp(
  mut jp2: &mut opj_jp2,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut l_magic_number: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_header_data.is_null());
  if jp2.jp2_state != JP2_STATE_NONE as core::ffi::c_uint {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "The signature box must be the first box in the file.\n",
    );
    return 0i32;
  }
  /* assure length of data is correct (4 -> magic number) */
  if p_header_size != 4u32 {
    event_msg!(p_manager, EVT_ERROR, "Error with JP signature Box size\n",);
    return 0i32;
  }
  /* rearrange data */
  opj_read_bytes(p_header_data, &mut l_magic_number, 4 as OPJ_UINT32);
  if l_magic_number != 0xd0a870au32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error with JP Signature : bad magic number\n",
    );
    return 0i32;
  }
  jp2.jp2_state |= JP2_STATE_SIGNATURE as core::ffi::c_uint;
  1i32
}

/* *
 * Reads a a FTYP box - File type box
 *
 * @param   p_header_data   the data contained in the FTYP box.
 * @param   jp2             the jpeg2000 file codec.
 * @param   p_header_size   the size of the data contained in the FTYP box.
 * @param   p_manager       the user event manager.
 *
 * @return true if the FTYP box is valid.
 */
fn opj_jp2_read_ftyp(
  mut jp2: &mut opj_jp2,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    let mut l_remaining_bytes: OPJ_UINT32 = 0;
    /* preconditions */

    assert!(!p_header_data.is_null());
    if jp2.jp2_state != JP2_STATE_SIGNATURE as core::ffi::c_uint {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "The ftyp box must be the second box in the file.\n",
      );
      return 0i32;
    }
    /* assure length of data is correct */
    if p_header_size < 8u32 {
      event_msg!(p_manager, EVT_ERROR, "Error with FTYP signature Box size\n",); /* BR */
      return 0i32;
    } /* MinV */
    opj_read_bytes(p_header_data, &mut jp2.brand, 4 as OPJ_UINT32);
    p_header_data = p_header_data.offset(4);
    opj_read_bytes(p_header_data, &mut jp2.minversion, 4 as OPJ_UINT32);
    p_header_data = p_header_data.offset(4);
    l_remaining_bytes = p_header_size.wrapping_sub(8u32);
    /* the number of remaining bytes should be a multiple of 4 */
    if l_remaining_bytes & 0x3u32 != 0u32 {
      event_msg!(p_manager, EVT_ERROR, "Error with FTYP signature Box size\n",);
      return 0i32;
    }
    /* div by 4 */
    let numcl = l_remaining_bytes >> 2i32;
    jp2.cl = Vec::with_capacity(numcl as usize);
    for _ in 0..numcl {
      /* CLi */
      let mut value = 0u32;
      opj_read_bytes(p_header_data, &mut value, 4 as OPJ_UINT32);
      jp2.cl.push(value);
      p_header_data = p_header_data.offset(4);
    }
    jp2.jp2_state |= JP2_STATE_FILE_TYPE as core::ffi::c_uint;
    1i32
  }
}

fn opj_jp2_skip_jp2c(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* preconditions */

  jp2.j2k_codestream_offset = opj_stream_tell(stream);
  if opj_stream_skip(stream, 8 as OPJ_OFF_T, p_manager) != 8i64 {
    return 0i32;
  }
  1i32
}

fn opj_jpip_skip_iptr(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* preconditions */

  jp2.jpip_iptr_offset = opj_stream_tell(stream);
  if opj_stream_skip(stream, 24 as OPJ_OFF_T, p_manager) != 24i64 {
    return 0i32;
  }
  1i32
}

/* *
 * Reads the Jpeg2000 file Header box - JP2 Header box (warning, this is a super box).
 *
 * @param   p_header_data   the data contained in the file header box.
 * @param   jp2             the jpeg2000 file codec.
 * @param   p_header_size   the size of the data contained in the file header box.
 * @param   p_manager       the user event manager.
 *
 * @return true if the JP2 Header box was successfully recognized.
*/
fn opj_jp2_read_jp2h(
  mut jp2: &mut opj_jp2,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    let mut l_box_size = 0 as OPJ_UINT32;
    let mut box_0 = Jp2BoxHeader { length: 0, ty: 0, header_length: 0 };
    let mut l_has_ihdr = 0i32;
    /* preconditions */

    assert!(!p_header_data.is_null());
    /* make sure the box is well placed */
    if jp2.jp2_state & JP2_STATE_FILE_TYPE as core::ffi::c_uint
      != JP2_STATE_FILE_TYPE as core::ffi::c_uint
    {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "The  box must be the first box in the file.\n",
      );
      return 0i32;
    }
    jp2.jp2_img_state = JP2_IMG_STATE_NONE as OPJ_UINT32;
    /* iterate while remaining data */
    while p_header_size > 0u32 {
      if opj_jp2_read_boxhdr_char(
        &mut box_0,
        p_header_data,
        &mut l_box_size,
        p_header_size,
        p_manager,
      ) == 0
      {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Stream error while reading JP2 Header box\n",
        );
        return 0i32;
      }
      if box_0.length > p_header_size {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Stream error while reading JP2 Header box: box length is inconsistent.\n",
        );
        return 0i32;
      }
      let l_current_handler = opj_jp2_img_find_handler(box_0.ty);
      let l_current_data_size = box_0.length.wrapping_sub(l_box_size);
      p_header_data = p_header_data.offset(l_box_size as isize);
      if let Some(handler) = l_current_handler {
        if (handler.handler)(jp2, p_header_data, l_current_data_size, p_manager) == 0 {
          return 0i32;
        }
      } else {
        jp2.jp2_img_state |= JP2_IMG_STATE_UNKNOWN as core::ffi::c_uint
      }
      if box_0.ty == 0x69686472u32 {
        l_has_ihdr = 1i32
      }
      p_header_data = p_header_data.offset(l_current_data_size as isize);
      p_header_size = (p_header_size as core::ffi::c_uint).wrapping_sub(box_0.length) as OPJ_UINT32
    }
    if l_has_ihdr == 0i32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Stream error while reading JP2 Header box: no \'ihdr\' box.\n",
      );
      return 0i32;
    }
    jp2.jp2_state |= JP2_STATE_HEADER as core::ffi::c_uint;
    jp2.has_jp2h = 1 as OPJ_BYTE;
    1i32
  }
}

/* *
 * Reads a box header. The box is the way data is packed inside a jpeg2000 file structure. Data is read from a character string
 *
 * @param   box                     the box structure to fill.
 * @param   p_data                  the character string to read data from.
 * @param   p_number_bytes_read     pointer to an int that will store the number of bytes read from the stream (shoul usually be 2).
 * @param   p_box_max_size          the maximum number of bytes in the box.
 * @param   p_manager         FIXME DOC
 *
 * @return  true if the box is recognized, false otherwise
*/
fn opj_jp2_read_boxhdr_char(
  box_0: &mut Jp2BoxHeader,
  mut p_data: *mut OPJ_BYTE,
  p_number_bytes_read: &mut OPJ_UINT32,
  p_box_max_size: OPJ_UINT32,
  p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    let mut l_value: OPJ_UINT32 = 0;
    /* preconditions */

    assert!(!p_data.is_null());
    if p_box_max_size < 8u32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Cannot handle box of less than 8 bytes\n",
      );
      return 0i32;
    }
    /* process read data */
    opj_read_bytes(p_data, &mut l_value, 4 as OPJ_UINT32);
    p_data = p_data.offset(4);
    (*box_0).length = l_value;
    opj_read_bytes(p_data, &mut l_value, 4 as OPJ_UINT32);
    p_data = p_data.offset(4);
    (*box_0).ty = l_value;
    *p_number_bytes_read = 8 as OPJ_UINT32;
    /* do we have a "special very large box ?" */
    /* read then the XLBox */
    if (*box_0).length == 1u32 {
      let mut l_xl_part_size: OPJ_UINT32 = 0;
      if p_box_max_size < 16u32 {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Cannot handle XL box of less than 16 bytes\n",
        );
        return 0i32;
      }
      opj_read_bytes(p_data, &mut l_xl_part_size, 4 as OPJ_UINT32);
      p_data = p_data.offset(4);
      *p_number_bytes_read =
        (*p_number_bytes_read as core::ffi::c_uint).wrapping_add(4u32) as OPJ_UINT32 as OPJ_UINT32;
      if l_xl_part_size != 0u32 {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Cannot handle box sizes higher than 2^32\n",
        );
        return 0i32;
      }
      opj_read_bytes(p_data, &mut l_value, 4 as OPJ_UINT32);
      *p_number_bytes_read =
        (*p_number_bytes_read as core::ffi::c_uint).wrapping_add(4u32) as OPJ_UINT32 as OPJ_UINT32;
      (*box_0).length = l_value;
      if (*box_0).length == 0u32 {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Cannot handle box of undefined sizes\n",
        );
        return 0i32;
      }
    } else if (*box_0).length == 0u32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Cannot handle box of undefined sizes\n",
      );
      return 0i32;
    }
    if (*box_0).length < *p_number_bytes_read {
      event_msg!(p_manager, EVT_ERROR, "Box length is inconsistent.\n",);
      return 0i32;
    }
    1i32
  }
}

pub(crate) fn opj_jp2_read_header(
  p_stream: &mut Stream,
  jp2: &mut opj_jp2,
  p_image: *mut *mut opj_image_t,
  p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut validation_list = opj_jp2_proc_list_t::new();
  let mut procedure_list = opj_jp2_proc_list_t::new();

  /* customization of the validation */
  if opj_jp2_setup_decoding_validation(jp2, &mut validation_list, p_manager) == 0 {
    return 0i32;
  }
  /* customization of the encoding */
  if opj_jp2_setup_header_reading(jp2, &mut procedure_list, p_manager) == 0 {
    return 0i32;
  }
  /* validation of the parameters codec */
  if opj_jp2_exec(jp2, &mut validation_list, p_stream, p_manager) == 0 {
    return 0i32;
  }
  /* read header */
  if opj_jp2_exec(jp2, &mut procedure_list, p_stream, p_manager) == 0 {
    return 0i32;
  }
  if jp2.has_jp2h as core::ffi::c_int == 0i32 {
    event_msg!(p_manager, EVT_ERROR, "JP2H box missing. Required.\n",);
    return 0i32;
  }
  if jp2.has_ihdr as core::ffi::c_int == 0i32 {
    event_msg!(p_manager, EVT_ERROR, "IHDR box_missing. Required.\n",);
    return 0i32;
  }

  let ret = opj_j2k_read_header(p_stream, &mut jp2.j2k, p_image, p_manager);

  let image = unsafe {
    if !p_image.is_null() && !(*p_image).is_null() {
      Some(&mut *(*p_image))
    } else {
      None
    }
  };
  if let Some(image) = image {
    /* Set Image Color Space */
    if jp2.enumcs == 16u32 {
      image.color_space = OPJ_CLRSPC_SRGB
    } else if jp2.enumcs == 17u32 {
      image.color_space = OPJ_CLRSPC_GRAY
    } else if jp2.enumcs == 18u32 {
      image.color_space = OPJ_CLRSPC_SYCC
    } else if jp2.enumcs == 24u32 {
      image.color_space = OPJ_CLRSPC_EYCC
    } else if jp2.enumcs == 12u32 {
      image.color_space = OPJ_CLRSPC_CMYK
    } else {
      image.color_space = OPJ_CLRSPC_UNKNOWN
    }

    if let Some(icc_profile) = &jp2.color.icc_profile {
      image.copy_icc_profile(icc_profile);
      image.icc_profile_len = jp2.color.icc_profile_len;
    }
  }
  ret
}

/* *
 * Sets up the validation ,i.e. adds the procedures to launch to make sure the codec parameters
 * are valid. Developers wanting to extend the library can add their own validation procedures.
 */
fn opj_jp2_setup_encoding_validation(
  _jp2: &mut opj_jp2,
  list: &mut opj_jp2_proc_list_t,
  _p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  list.add(opj_jp2_default_validation);
  /* DEVELOPER CORNER, add your custom validation procedure */
  1i32
}

/* *
 * Sets up the validation ,i.e. adds the procedures to launch to make sure the codec parameters
 * are valid. Developers wanting to extend the library can add their own validation procedures.
 */
fn opj_jp2_setup_decoding_validation(
  _jp2: &mut opj_jp2,
  _list: &mut opj_jp2_proc_list_t,
  _p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* DEVELOPER CORNER, add your custom validation procedure */
  1i32
}

/* *
 * Sets up the procedures to do on writing header. Developers wanting to extend the library can add their own writing procedures.
 */
fn opj_jp2_setup_header_writing(
  jp2: &mut opj_jp2,
  list: &mut opj_jp2_proc_list_t,
  _p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  list.add(opj_jp2_write_jp);
  list.add(opj_jp2_write_ftyp);
  list.add(opj_jp2_write_jp2h);
  if jp2.jpip_on != 0 {
    list.add(opj_jpip_skip_iptr);
  }
  list.add(opj_jp2_skip_jp2c);
  /* DEVELOPER CORNER, insert your custom procedures */
  1i32
}

/* *
 * Sets up the procedures to do on reading header.
 * Developers wanting to extend the library can add their own writing procedures.
 */
fn opj_jp2_setup_header_reading(
  _jp2: &mut opj_jp2,
  list: &mut opj_jp2_proc_list_t,
  _p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  list.add(opj_jp2_read_header_procedure);
  /* DEVELOPER CORNER, add your custom procedures */
  1i32
}

pub(crate) fn opj_jp2_read_tile_header(
  p_jp2: &mut opj_jp2,
  p_stream: &mut Stream,
  tile_info: &mut TileInfo,
  p_manager: &mut opj_event_mgr,
) -> bool {
  opj_j2k_read_tile_header(&mut p_jp2.j2k, p_stream, tile_info, p_manager)
}

pub(crate) fn opj_jp2_write_tile(
  p_jp2: &mut opj_jp2,
  p_tile_index: OPJ_UINT32,
  p_data: &[u8],
  p_stream: &mut Stream,
  p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  opj_j2k_write_tile(&mut p_jp2.j2k, p_tile_index, p_data, p_stream, p_manager)
}

pub(crate) fn opj_jp2_decode_tile(
  p_jp2: &mut opj_jp2,
  p_tile_index: OPJ_UINT32,
  p_data: Option<&mut [u8]>,
  p_stream: &mut Stream,
  p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  opj_j2k_decode_tile(&mut p_jp2.j2k, p_tile_index, p_data, p_stream, p_manager)
}

pub(crate) fn opj_jp2_set_decoded_components(
  p_jp2: &mut opj_jp2,
  components: &[u32],
  p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  opj_j2k_set_decoded_components(&mut p_jp2.j2k, components, p_manager)
}

pub(crate) fn opj_jp2_set_decode_area(
  mut p_jp2: &mut opj_jp2,
  mut p_image: &mut opj_image,
  mut p_start_x: OPJ_INT32,
  mut p_start_y: OPJ_INT32,
  mut p_end_x: OPJ_INT32,
  mut p_end_y: OPJ_INT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  opj_j2k_set_decode_area(
    &mut p_jp2.j2k,
    p_image,
    p_start_x,
    p_start_y,
    p_end_x,
    p_end_y,
    p_manager,
  )
}

pub(crate) fn opj_jp2_get_tile(
  mut p_jp2: &mut opj_jp2,
  mut p_stream: &mut Stream,
  mut p_image: &mut opj_image,
  mut p_manager: &mut opj_event_mgr,
  mut tile_index: OPJ_UINT32,
) -> OPJ_BOOL {
  event_msg!(
    p_manager,
    EVT_WARNING,
    "JP2 box which are after the codestream will not be read by this function.\n",
  );
  if opj_j2k_get_tile(&mut p_jp2.j2k, p_stream, p_image, p_manager, tile_index) == 0 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Failed to decode the codestream in the JP2 file\n",
    );
    return 0i32;
  }

  opj_jp2_apply_color_postprocessing(p_jp2, p_image, p_manager)
}

/// JP2 encoder interface
pub(crate) fn opj_jp2_create(mut p_is_decoder: OPJ_BOOL) -> Option<opj_jp2> {
  /* create the J2K codec */
  let mut jp2 = opj_jp2 {
    j2k: if p_is_decoder == 0 {
      opj_j2k_create_compress()?
    } else {
      opj_j2k_create_decompress()?
    },
    w: 0,
    h: 0,
    bpc: 0,
    C: 0,
    UnkC: 0,
    IPR: 0,
    meth: 0,
    approx: 0,
    enumcs: 0,
    precedence: 0,
    brand: 0,
    minversion: 0,
    cl: Vec::new(),
    comps: Vec::new(),
    j2k_codestream_offset: 0,
    jpip_iptr_offset: 0,
    jpip_on: 0,
    jp2_state: 0,
    jp2_img_state: 0,
    ignore_pclr_cmap_cdef: 0,
    has_jp2h: 0,
    has_ihdr: 0,
    /* Color structure */
    color: opj_jp2_color {
      icc_profile: None,
      icc_profile_len: 0 as OPJ_UINT32,
      jp2_cdef: None,
      jp2_pclr: None,
      jp2_has_colr: 0 as OPJ_BYTE,
    },
  };
  Some(jp2)
}

#[cfg(feature = "file-io")]
pub(crate) fn jp2_dump(mut p_jp2: &mut opj_jp2, mut flag: OPJ_INT32, mut out_stream: *mut ::libc::FILE) {
  /* preconditions */
  j2k_dump(&mut p_jp2.j2k, flag, out_stream);
}

pub(crate) fn jp2_get_cstr_index(mut p_jp2: &mut opj_jp2) -> *mut opj_codestream_index_t {
  j2k_get_cstr_index(&mut p_jp2.j2k)
}

pub(crate) fn jp2_get_cstr_info(mut p_jp2: &mut opj_jp2) -> *mut opj_codestream_info_v2_t {
  j2k_get_cstr_info(&mut p_jp2.j2k)
}

pub(crate) fn opj_jp2_set_decoded_resolution_factor(
  mut p_jp2: &mut opj_jp2,
  mut res_factor: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  opj_j2k_set_decoded_resolution_factor(&mut p_jp2.j2k, res_factor, p_manager)
}

pub(crate) fn opj_jp2_encoder_set_extra_options(
  p_jp2: &mut opj_jp2,
  options: &[&str],
  p_manager: &mut opj_event_mgr,
) -> bool {
  opj_j2k_encoder_set_extra_options(&mut p_jp2.j2k, options, p_manager)
}
