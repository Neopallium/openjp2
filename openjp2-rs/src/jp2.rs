use std::io::Write;

use super::cio::*;
use super::consts::*;
use super::event::*;
use super::function_list::*;
use super::j2k::*;
use super::openjpeg::*;
use super::stream::*;
#[cfg(feature = "file-io")]
use ::libc::FILE;

use super::malloc::*;

extern "C" {
  fn memcpy(
    _: *mut core::ffi::c_void,
    _: *const core::ffi::c_void,
    _: usize,
  ) -> *mut core::ffi::c_void;
}

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

pub struct Jp2BoxHeader {
  pub length: u32,
  pub ty: u32,
}

impl Jp2BoxHeader {
  fn new(ty: Jp2BoxType) -> Self {
    Self {
      length: 8,
      ty: ty.to_u32().unwrap(),
    }
  }

  fn write<W: Write>(&self, writer: &mut W) -> bool {
    writer.write_all(&self.length.to_be_bytes()[..]).is_ok()
      && writer.write_all(&self.ty.to_be_bytes()[..]).is_ok()
  }
}

pub(crate) struct opj_jp2_header_handler {
  pub id: OPJ_UINT32,
  pub handler: Option<
    unsafe fn(_: &mut opj_jp2, _: *mut OPJ_BYTE, _: OPJ_UINT32, _: &mut opj_event_mgr) -> OPJ_BOOL,
  >,
}
pub(crate) type opj_jp2_header_handler_t = opj_jp2_header_handler;

pub(crate) struct HeaderWriter {
  handler: fn(_: &mut opj_jp2, _: &mut Vec<u8>) -> bool,
  m_data: Vec<u8>,
}

impl HeaderWriter {
  fn new(handler: fn(_: &mut opj_jp2, _: &mut Vec<u8>) -> bool) -> Self {
    Self {
      handler: handler,
      m_data: Default::default(),
    }
  }

  fn run(&mut self, jp2: &mut opj_jp2) -> Option<u32> {
    if (self.handler)(jp2, &mut self.m_data) {
      Some(self.m_data.len() as u32)
    } else {
      None
    }
  }

  fn write(&self, stream: &mut Stream, manager: &mut opj_event_mgr) -> bool {
    let size = self.m_data.len();
    opj_stream_write_data(stream, self.m_data.as_ptr(), size, manager) == size
  }
}

static mut jp2_header: [opj_jp2_header_handler_t; 3] = [
  {
    opj_jp2_header_handler {
      id: 0x6a502020 as OPJ_UINT32,
      handler: Some(
        opj_jp2_read_jp
          as unsafe fn(
            _: &mut opj_jp2,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: &mut opj_event_mgr,
          ) -> OPJ_BOOL,
      ),
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x66747970 as OPJ_UINT32,
      handler: Some(
        opj_jp2_read_ftyp
          as unsafe fn(
            _: &mut opj_jp2,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: &mut opj_event_mgr,
          ) -> OPJ_BOOL,
      ),
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x6a703268 as OPJ_UINT32,
      handler: Some(
        opj_jp2_read_jp2h
          as unsafe fn(
            _: &mut opj_jp2,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: &mut opj_event_mgr,
          ) -> OPJ_BOOL,
      ),
    }
  },
];
static mut jp2_img_header: [opj_jp2_header_handler_t; 6] = [
  {
    opj_jp2_header_handler {
      id: 0x69686472 as OPJ_UINT32,
      handler: Some(
        opj_jp2_read_ihdr
          as unsafe fn(
            _: &mut opj_jp2,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: &mut opj_event_mgr,
          ) -> OPJ_BOOL,
      ),
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x636f6c72 as OPJ_UINT32,
      handler: Some(
        opj_jp2_read_colr
          as unsafe fn(
            _: &mut opj_jp2,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: &mut opj_event_mgr,
          ) -> OPJ_BOOL,
      ),
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x62706363 as OPJ_UINT32,
      handler: Some(
        opj_jp2_read_bpcc
          as unsafe fn(
            _: &mut opj_jp2,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: &mut opj_event_mgr,
          ) -> OPJ_BOOL,
      ),
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x70636c72 as OPJ_UINT32,
      handler: Some(
        opj_jp2_read_pclr
          as unsafe fn(
            _: &mut opj_jp2,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: &mut opj_event_mgr,
          ) -> OPJ_BOOL,
      ),
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x636d6170 as OPJ_UINT32,
      handler: Some(
        opj_jp2_read_cmap
          as unsafe fn(
            _: &mut opj_jp2,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: &mut opj_event_mgr,
          ) -> OPJ_BOOL,
      ),
    }
  },
  {
    opj_jp2_header_handler {
      id: 0x63646566 as OPJ_UINT32,
      handler: Some(
        opj_jp2_read_cdef
          as unsafe fn(
            _: &mut opj_jp2,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: &mut opj_event_mgr,
          ) -> OPJ_BOOL,
      ),
    }
  },
];

/* *
 * Reads a box header. The box is the way data is packed inside a jpeg2000 file structure.
 *
 * @param   stream                     the input stream to read data from.
 * @param   box                     the box structure to fill.
 * @param   p_number_bytes_read     pointer to an int that will store the number of bytes read from the stream (shoul usually be 2).
 * @param   p_manager               user event manager.
 *
 * @return  true if the box is recognized, false otherwise
*/
fn opj_jp2_read_boxhdr(
  mut box_0: &mut Jp2BoxHeader,
  mut p_number_bytes_read: &mut OPJ_UINT32,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    /* read header from file */
    let mut l_data_header: [OPJ_BYTE; 8] = [0; 8];
    /* preconditions */

    *p_number_bytes_read = opj_stream_read_data(
      stream,
      l_data_header.as_mut_ptr(),
      8 as OPJ_SIZE_T,
      p_manager,
    ) as OPJ_UINT32;
    if *p_number_bytes_read != 8u32 {
      return 0i32;
    }
    /* process read data */
    opj_read_bytes(
      l_data_header.as_mut_ptr(),
      &mut (*box_0).length,
      4 as OPJ_UINT32,
    );
    opj_read_bytes(
      l_data_header.as_mut_ptr().offset(4),
      &mut (*box_0).ty,
      4 as OPJ_UINT32,
    );
    if (*box_0).length == 0u32 {
      /* last box */
      let bleft = opj_stream_get_number_byte_left(stream);
      if bleft > (0xffffffffu32).wrapping_sub(8u32) as OPJ_OFF_T {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Cannot handle box sizes higher than 2^32\n",
        );
        return 0i32;
      }
      (*box_0).length = (bleft as OPJ_UINT32).wrapping_add(8u32);
      assert!((*box_0).length as OPJ_OFF_T == bleft + 8i64);
      return 1i32;
    }
    /* do we have a "special very large box ?" */
    /* read then the XLBox */
    if (*box_0).length == 1u32 {
      let mut l_xl_part_size: OPJ_UINT32 = 0;
      let mut l_nb_bytes_read = opj_stream_read_data(
        stream,
        l_data_header.as_mut_ptr(),
        8 as OPJ_SIZE_T,
        p_manager,
      ) as OPJ_UINT32;
      if l_nb_bytes_read != 8u32 {
        if l_nb_bytes_read > 0u32 {
          *p_number_bytes_read =
            (*p_number_bytes_read as core::ffi::c_uint).wrapping_add(l_nb_bytes_read) as OPJ_UINT32
        }
        return 0i32;
      }
      *p_number_bytes_read = 16 as OPJ_UINT32;
      opj_read_bytes(
        l_data_header.as_mut_ptr(),
        &mut l_xl_part_size,
        4 as OPJ_UINT32,
      );
      if l_xl_part_size != 0u32 {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Cannot handle box sizes higher than 2^32\n",
        );
        return 0i32;
      }
      opj_read_bytes(
        l_data_header.as_mut_ptr().offset(4),
        &mut (*box_0).length,
        4 as OPJ_UINT32,
      );
    }
    1i32
  }
}

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
    if !jp2.comps.is_null() {
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
    opj_read_bytes(p_image_header_data, &mut jp2.numcomps, 2 as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(2);
    if jp2.h < 1u32 || jp2.w < 1u32 || jp2.numcomps < 1u32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Wrong values for: w(%d) h(%d) numcomps(%d) (ihdr)\n",
        jp2.w,
        jp2.h,
        jp2.numcomps,
      );
      return 0i32;
    }
    if jp2.numcomps.wrapping_sub(1u32) >= 16384u32 {
      /* unsigned underflow is well defined: 1U <= jp2->numcomps <= 16384U */
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Invalid number of components (ihdr)\n",
      );
      return 0i32;
    }
    /* allocate memory for components */
    jp2.comps = opj_calloc(
      jp2.numcomps as size_t,
      core::mem::size_of::<opj_jp2_comps_t>(),
    ) as *mut opj_jp2_comps_t; /* BPC */
    if jp2.comps.is_null() {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Not enough memory to handle image header (ihdr)\n",
      ); /* C */
      return 0i32;
    }
    opj_read_bytes(p_image_header_data, &mut jp2.bpc, 1 as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(1);
    opj_read_bytes(p_image_header_data, &mut jp2.C, 1 as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(1);
    /* Should be equal to 7 cf. chapter about image header box of the norm */
    if jp2.C != 7u32 {
      event_msg!(
      p_manager,
      EVT_INFO,
      "JP2 IHDR box: compression type indicate that the file is not a conforming JP2 file (%d) \n",
      jp2.C
    ); /* UnkC */
    } /* IPR */
    opj_read_bytes(p_image_header_data, &mut jp2.UnkC, 1 as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(1);
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
  buf.write_all(&jp2.h.to_be_bytes()).unwrap();
  /* WIDTH */
  buf.write_all(&jp2.w.to_be_bytes()).unwrap();
  /* NC */
  buf.write_all(&(jp2.numcomps as u16).to_be_bytes()).unwrap();
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
  let comps = unsafe { std::slice::from_raw_parts(jp2.comps, jp2.numcomps as usize) };
  let mut header = Jp2BoxHeader::new(Jp2BoxType::BPCC);
  header.length += comps.len() as u32;
  header.write(buf);
  for comp in comps {
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
    let mut i: OPJ_UINT32 = 0;
    /* preconditions */

    assert!(!p_bpc_header_data.is_null());
    if jp2.bpc != 255u32 {
      event_msg!(p_manager, EVT_WARNING,
                      "A BPCC header box is available although BPC given by the IHDR box (%d) indicate components bit depth is constant\n", jp2.bpc);
    }
    /* and length is relevant */
    if p_bpc_header_size != jp2.numcomps {
      event_msg!(p_manager, EVT_ERROR, "Bad BPCC header box (bad size)\n",);
      return 0i32;
    }
    /* read info for each component */
    i = 0 as OPJ_UINT32; /* read each BPCC component */
    while i < jp2.numcomps {
      opj_read_bytes(
        p_bpc_header_data,
        &mut (*jp2.comps.offset(i as isize)).bpcc,
        1 as OPJ_UINT32,
      );
      p_bpc_header_data = p_bpc_header_data.offset(1);
      i += 1;
    }
    1i32
  }
}

/* *
 * Writes the Channel Definition box.
 *
 */
fn opj_jp2_write_cdef(mut jp2: &mut opj_jp2, buf: &mut Vec<u8>) -> bool {
  let cdef = if !jp2.color.jp2_cdef.is_null() {
    unsafe { &*jp2.color.jp2_cdef }
  } else {
    return false;
  };
  assert!(!cdef.info.is_null());
  assert!(cdef.n as core::ffi::c_uint > 0u32);
  let info = unsafe { std::slice::from_raw_parts(cdef.info, cdef.n as usize) };
  let mut header = Jp2BoxHeader::new(Jp2BoxType::CDEF);
  header.length += 2 + (info.len() * 6) as u32;
  header.write(buf);
  buf.write_all(&(cdef.n as u16).to_be_bytes()).unwrap();
  for info in info {
    buf.write_all(&(info.cn as u16).to_be_bytes()).unwrap();
    buf.write_all(&(info.typ as u16).to_be_bytes()).unwrap();
    buf.write_all(&(info.asoc as u16).to_be_bytes()).unwrap();
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
      buf.write_all(&(jp2.enumcs as u32).to_be_bytes()).unwrap();
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

fn opj_jp2_free_pclr(mut color: *mut opj_jp2_color_t) {
  unsafe {
    opj_free((*(*color).jp2_pclr).channel_sign as *mut core::ffi::c_void);
    opj_free((*(*color).jp2_pclr).channel_size as *mut core::ffi::c_void);
    opj_free((*(*color).jp2_pclr).entries as *mut core::ffi::c_void);
    if !(*(*color).jp2_pclr).cmap.is_null() {
      opj_free((*(*color).jp2_pclr).cmap as *mut core::ffi::c_void);
    }
    opj_free((*color).jp2_pclr as *mut core::ffi::c_void);
    (*color).jp2_pclr = std::ptr::null_mut::<opj_jp2_pclr_t>();
  }
}

fn opj_jp2_check_color(
  mut image: &mut opj_image_t,
  mut color: *mut opj_jp2_color_t,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    let mut i: OPJ_UINT16 = 0;
    /* testcase 4149.pdf.SIGSEGV.cf7.3501 */
    if !(*color).jp2_cdef.is_null() {
      let mut info = (*(*color).jp2_cdef).info; /* FIXME image->numcomps == jp2->numcomps before color is applied ??? */
      let mut n = (*(*color).jp2_cdef).n;
      let mut nr_channels = image.numcomps;
      /* cdef applies to cmap channels if any */
      if !(*color).jp2_pclr.is_null() && !(*(*color).jp2_pclr).cmap.is_null() {
        nr_channels = (*(*color).jp2_pclr).nr_channels as OPJ_UINT32
      }
      i = 0 as OPJ_UINT16;
      while (i as core::ffi::c_int) < n as core::ffi::c_int {
        if (*info.offset(i as isize)).cn as core::ffi::c_uint >= nr_channels {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Invalid component index %d (>= %d).\n",
            (*info.offset(i as isize)).cn as core::ffi::c_int,
            nr_channels,
          );
          return 0i32;
        }
        if (*info.offset(i as isize)).asoc as core::ffi::c_uint != 65535u32
          && (*info.offset(i as isize)).asoc as core::ffi::c_int > 0i32
          && ((*info.offset(i as isize)).asoc as core::ffi::c_int - 1i32) as OPJ_UINT32
            >= nr_channels
        {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Invalid component index %d (>= %d).\n",
            (*info.offset(i as isize)).asoc as core::ffi::c_int - 1i32,
            nr_channels,
          );
          return 0i32;
        }
        i += 1;
      }
      /* issue 397 */
      /* ISO 15444-1 states that if cdef is present, it shall contain a complete list of channel definitions. */
      while nr_channels > 0u32 {
        i = 0 as OPJ_UINT16;
        while (i as core::ffi::c_int) < n as core::ffi::c_int {
          if (*info.offset(i as isize)).cn as OPJ_UINT32 == nr_channels.wrapping_sub(1u32) {
            break;
          }
          i += 1;
        }
        if i as core::ffi::c_int == n as core::ffi::c_int {
          event_msg!(p_manager, EVT_ERROR, "Incomplete channel definitions.\n",);
          return 0i32;
        }
        nr_channels = nr_channels.wrapping_sub(1)
      }
    }
    /* testcases 451.pdf.SIGSEGV.f4c.3723, 451.pdf.SIGSEGV.5b5.3723 and
    66ea31acbb0f23a2bbc91f64d69a03f5_signal_sigsegv_13937c0_7030_5725.pdf */
    if !(*color).jp2_pclr.is_null() && !(*(*color).jp2_pclr).cmap.is_null() {
      let mut nr_channels_0 = (*(*color).jp2_pclr).nr_channels as OPJ_UINT16;
      let mut cmap = (*(*color).jp2_pclr).cmap;
      let mut pcol_usage = std::ptr::null_mut::<OPJ_BOOL>();
      let mut is_sane = 1i32;
      /* verify that all original components match an existing one */
      i = 0 as OPJ_UINT16;
      while (i as core::ffi::c_int) < nr_channels_0 as core::ffi::c_int {
        if (*cmap.offset(i as isize)).cmp as core::ffi::c_uint >= image.numcomps {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Invalid component index %d (>= %d).\n",
            (*cmap.offset(i as isize)).cmp as core::ffi::c_int,
            image.numcomps,
          );
          is_sane = 0i32
        }
        i += 1;
      }
      pcol_usage =
        opj_calloc(nr_channels_0 as size_t, core::mem::size_of::<OPJ_BOOL>()) as *mut OPJ_BOOL;
      if pcol_usage.is_null() {
        event_msg!(p_manager, EVT_ERROR, "Unexpected OOM.\n",);
        return 0i32;
      }
      /* verify that no component is targeted more than once */
      i = 0 as OPJ_UINT16;
      while (i as core::ffi::c_int) < nr_channels_0 as core::ffi::c_int {
        let mut mtyp = (*cmap.offset(i as isize)).mtyp;
        let mut pcol = (*cmap.offset(i as isize)).pcol;
        /* See ISO 15444-1 Table I.14 – MTYPi field values */
        if mtyp as core::ffi::c_int != 0i32 && mtyp as core::ffi::c_int != 1i32 {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Invalid value for cmap[%d].mtyp = %d.\n",
            i as core::ffi::c_int,
            mtyp as core::ffi::c_int,
          );
          is_sane = 0i32
        } else if pcol as core::ffi::c_int >= nr_channels_0 as core::ffi::c_int {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Invalid component/palette index for direct mapping %d.\n",
            pcol as core::ffi::c_int,
          );
          is_sane = 0i32
        } else if *pcol_usage.offset(pcol as isize) != 0 && mtyp as core::ffi::c_int == 1i32 {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Component %d is mapped twice.\n",
            pcol as core::ffi::c_int,
          );
          is_sane = 0i32
        } else if mtyp as core::ffi::c_int == 0i32 && pcol as core::ffi::c_int != 0i32 {
          /* I.5.3.5 PCOL: If the value of the MTYP field for this channel is 0, then
           * the value of this field shall be 0. */
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Direct use at #%d however pcol=%d.\n",
            i as core::ffi::c_int,
            pcol as core::ffi::c_int,
          );
          is_sane = 0i32
        } else if mtyp as core::ffi::c_int == 1i32
          && pcol as core::ffi::c_int != i as core::ffi::c_int
        {
          /* OpenJPEG implementation limitation. See assert(i == pcol); */
          /* in opj_jp2_apply_pclr() */
          event_msg!(p_manager, EVT_ERROR,
                              "Implementation limitation: for palette mapping, pcol[%d] should be equal to %d, but is equal to %d.\n",
                              i as core::ffi::c_int, i as core::ffi::c_int,
                              pcol as core::ffi::c_int);
          is_sane = 0i32
        } else {
          *pcol_usage.offset(pcol as isize) = 1i32
        }
        i += 1;
      }
      /* verify that all components are targeted at least once */
      i = 0 as OPJ_UINT16;
      while (i as core::ffi::c_int) < nr_channels_0 as core::ffi::c_int {
        if *pcol_usage.offset(i as isize) == 0
          && (*cmap.offset(i as isize)).mtyp as core::ffi::c_int != 0i32
        {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Component %d doesn\'t have a mapping.\n",
            i as core::ffi::c_int,
          );
          is_sane = 0i32
        }
        i += 1;
      }
      /* Issue 235/447 weird cmap */
      if 1i32 != 0 && is_sane != 0 && image.numcomps == 1u32 {
        i = 0 as OPJ_UINT16;
        while (i as core::ffi::c_int) < nr_channels_0 as core::ffi::c_int {
          if *pcol_usage.offset(i as isize) == 0 {
            is_sane = 0 as OPJ_BOOL;
            event_msg!(
              p_manager,
              EVT_WARNING,
              "Component mapping seems wrong. Trying to correct.\n",
            );
            break;
          } else {
            i += 1;
          }
        }
        if is_sane == 0 {
          is_sane = 1i32;
          i = 0 as OPJ_UINT16;
          while (i as core::ffi::c_int) < nr_channels_0 as core::ffi::c_int {
            (*cmap.offset(i as isize)).mtyp = 1 as OPJ_BYTE;
            (*cmap.offset(i as isize)).pcol = i as OPJ_BYTE;
            i += 1;
          }
        }
      }
      opj_free(pcol_usage as *mut core::ffi::c_void);
      if is_sane == 0 {
        return 0i32;
      }
    }
    1i32
  }
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
  mut color: *mut opj_jp2_color_t,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    let mut old_comps = std::ptr::null_mut::<opj_image_comp_t>();
    let mut new_comps = std::ptr::null_mut::<opj_image_comp_t>();
    let mut channel_size = std::ptr::null_mut::<OPJ_BYTE>();
    let mut channel_sign = std::ptr::null_mut::<OPJ_BYTE>();
    let mut entries = std::ptr::null_mut::<OPJ_UINT32>();
    let mut cmap = std::ptr::null_mut::<opj_jp2_cmap_comp_t>();
    let mut src = std::ptr::null_mut::<OPJ_INT32>();
    let mut dst = std::ptr::null_mut::<OPJ_INT32>();
    let mut j: OPJ_UINT32 = 0;
    let mut max: OPJ_UINT32 = 0;
    let mut i: OPJ_UINT16 = 0;
    let mut nr_channels: OPJ_UINT16 = 0;
    let mut cmp: OPJ_UINT16 = 0;
    let mut pcol: OPJ_UINT16 = 0;
    let mut k: OPJ_INT32 = 0;
    let mut top_k: OPJ_INT32 = 0;
    channel_size = (*(*color).jp2_pclr).channel_size;
    channel_sign = (*(*color).jp2_pclr).channel_sign;
    entries = (*(*color).jp2_pclr).entries;
    cmap = (*(*color).jp2_pclr).cmap;
    nr_channels = (*(*color).jp2_pclr).nr_channels as OPJ_UINT16;
    i = 0 as OPJ_UINT16;
    while (i as core::ffi::c_int) < nr_channels as core::ffi::c_int {
      /* Palette mapping: */
      cmp = (*cmap.offset(i as isize)).cmp;
      if (*image.comps.offset(cmp as isize)).data.is_null() {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "image->comps[%d].data == NULL in opj_jp2_apply_pclr().\n",
          i as core::ffi::c_int,
        );
        return 0i32;
      }
      i += 1;
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
    i = 0 as OPJ_UINT16;
    while (i as core::ffi::c_int) < nr_channels as core::ffi::c_int {
      pcol = (*cmap.offset(i as isize)).pcol as OPJ_UINT16;
      cmp = (*cmap.offset(i as isize)).cmp;
      /* Direct use */
      if (*cmap.offset(i as isize)).mtyp as core::ffi::c_int == 0i32 {
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
        while i as core::ffi::c_int > 0i32 {
          i = i.wrapping_sub(1);
          opj_image_data_free((*new_comps.offset(i as isize)).data as *mut core::ffi::c_void);
        }
        opj_free(new_comps as *mut core::ffi::c_void);
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Memory allocation failure in opj_jp2_apply_pclr().\n",
        );
        return 0i32;
      }
      (*new_comps.offset(i as isize)).prec = *channel_size.offset(i as isize) as OPJ_UINT32;
      (*new_comps.offset(i as isize)).sgnd = *channel_sign.offset(i as isize) as OPJ_UINT32;
      i += 1;
    }
    top_k = (*(*color).jp2_pclr).nr_entries as core::ffi::c_int - 1i32;
    i = 0 as OPJ_UINT16;
    while (i as core::ffi::c_int) < nr_channels as core::ffi::c_int {
      /* Palette mapping: */
      cmp = (*cmap.offset(i as isize)).cmp; /* verified above */
      pcol = (*cmap.offset(i as isize)).pcol as OPJ_UINT16;
      src = (*old_comps.offset(cmp as isize)).data;
      assert!(!src.is_null());
      max = (*new_comps.offset(i as isize)).w * (*new_comps.offset(i as isize)).h;

      /* Direct use: */
      if (*cmap.offset(i as isize)).mtyp as core::ffi::c_int == 0i32 {
        dst = (*new_comps.offset(i as isize)).data;
        assert!(!dst.is_null());
        j = 0 as OPJ_UINT32;
        while j < max {
          *dst.offset(j as isize) = *src.offset(j as isize);
          j += 1;
        }
      } else {
        assert!(i as core::ffi::c_int == pcol as core::ffi::c_int);
        dst = (*new_comps.offset(pcol as isize)).data;
        assert!(!dst.is_null());
        j = 0 as OPJ_UINT32;
        while j < max {
          /* The index */
          k = *src.offset(j as isize);
          if k < 0i32 {
            k = 0i32
          } else if k > top_k {
            k = top_k
          }
          /* The colour */
          *dst.offset(j as isize) = *entries
            .offset((k * nr_channels as core::ffi::c_int + pcol as core::ffi::c_int) as isize)
            as OPJ_INT32;
          j += 1;
        }
      }
      i += 1;
    }
    max = image.numcomps;
    j = 0 as OPJ_UINT32;
    while j < max {
      if !(*old_comps.offset(j as isize)).data.is_null() {
        opj_image_data_free((*old_comps.offset(j as isize)).data as *mut core::ffi::c_void);
      }
      j += 1;
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
/* apply_pclr() */
fn opj_jp2_read_pclr(
  mut jp2: &mut opj_jp2,
  mut p_pclr_header_data: *mut OPJ_BYTE,
  mut p_pclr_header_size: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    let mut jp2_pclr = std::ptr::null_mut::<opj_jp2_pclr_t>();
    let mut channel_size = std::ptr::null_mut::<OPJ_BYTE>();
    let mut channel_sign = std::ptr::null_mut::<OPJ_BYTE>();
    let mut entries = std::ptr::null_mut::<OPJ_UINT32>();
    let mut nr_entries: OPJ_UINT16 = 0;
    let mut nr_channels: OPJ_UINT16 = 0;
    let mut i: OPJ_UINT16 = 0;
    let mut j: OPJ_UINT16 = 0;
    let mut l_value: OPJ_UINT32 = 0;
    let mut orig_header_data = p_pclr_header_data;
    /* preconditions */
    /* NPC */
    /* Cji */
    assert!(!p_pclr_header_data.is_null());
    if !jp2.color.jp2_pclr.is_null() {
      return 0i32;
    }
    if p_pclr_header_size < 3u32 {
      return 0i32;
    }
    opj_read_bytes(p_pclr_header_data, &mut l_value, 2 as OPJ_UINT32);
    p_pclr_header_data = p_pclr_header_data.offset(2);
    nr_entries = l_value as OPJ_UINT16;
    if nr_entries as core::ffi::c_uint == 0u32 || nr_entries as core::ffi::c_uint > 1024u32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Invalid PCLR box. Reports %d entries\n",
        nr_entries as core::ffi::c_int,
      );
      return 0i32;
    }
    opj_read_bytes(p_pclr_header_data, &mut l_value, 1 as OPJ_UINT32);
    p_pclr_header_data = p_pclr_header_data.offset(1);
    nr_channels = l_value as OPJ_UINT16;
    if nr_channels as core::ffi::c_uint == 0u32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Invalid PCLR box. Reports 0 palette columns\n",
      );
      return 0i32;
    }
    if p_pclr_header_size < (3u32).wrapping_add(nr_channels as OPJ_UINT32) {
      return 0i32;
    }
    entries = opj_malloc(
      core::mem::size_of::<OPJ_UINT32>()
        .wrapping_mul(nr_channels as usize)
        .wrapping_mul(nr_entries as usize),
    ) as *mut OPJ_UINT32;
    if entries.is_null() {
      return 0i32;
    }
    channel_size = opj_malloc(nr_channels as size_t) as *mut OPJ_BYTE;
    if channel_size.is_null() {
      opj_free(entries as *mut core::ffi::c_void);
      return 0i32;
    }
    channel_sign = opj_malloc(nr_channels as size_t) as *mut OPJ_BYTE;
    if channel_sign.is_null() {
      opj_free(entries as *mut core::ffi::c_void);
      opj_free(channel_size as *mut core::ffi::c_void);
      return 0i32;
    }
    jp2_pclr = opj_malloc(core::mem::size_of::<opj_jp2_pclr_t>()) as *mut opj_jp2_pclr_t;
    if jp2_pclr.is_null() {
      opj_free(entries as *mut core::ffi::c_void);
      opj_free(channel_size as *mut core::ffi::c_void);
      opj_free(channel_sign as *mut core::ffi::c_void);
      return 0i32;
    }
    (*jp2_pclr).channel_sign = channel_sign;
    (*jp2_pclr).channel_size = channel_size;
    (*jp2_pclr).entries = entries;
    (*jp2_pclr).nr_entries = nr_entries;
    (*jp2_pclr).nr_channels = l_value as OPJ_BYTE;
    (*jp2_pclr).cmap = std::ptr::null_mut::<opj_jp2_cmap_comp_t>();
    jp2.color.jp2_pclr = jp2_pclr;
    i = 0 as OPJ_UINT16;
    while (i as core::ffi::c_int) < nr_channels as core::ffi::c_int {
      opj_read_bytes(p_pclr_header_data, &mut l_value, 1 as OPJ_UINT32);
      p_pclr_header_data = p_pclr_header_data.offset(1);
      *channel_size.offset(i as isize) = (l_value & 0x7fu32).wrapping_add(1u32) as OPJ_BYTE;
      *channel_sign.offset(i as isize) =
        if l_value & 0x80u32 != 0 { 1i32 } else { 0i32 } as OPJ_BYTE;
      i += 1;
    }
    j = 0 as OPJ_UINT16;
    while (j as core::ffi::c_int) < nr_entries as core::ffi::c_int {
      i = 0 as OPJ_UINT16;
      while (i as core::ffi::c_int) < nr_channels as core::ffi::c_int {
        let mut bytes_to_read =
          ((*channel_size.offset(i as isize) as core::ffi::c_int + 7i32) >> 3i32) as OPJ_UINT32;
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
        *entries = l_value;
        entries = entries.offset(1);
        i += 1;
      }
      j += 1;
    }
    1i32
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
    let mut cmap = std::ptr::null_mut::<opj_jp2_cmap_comp_t>();
    let mut i: OPJ_BYTE = 0;
    let mut nr_channels: OPJ_BYTE = 0;
    let mut l_value: OPJ_UINT32 = 0;
    /* preconditions */

    assert!(!p_cmap_header_data.is_null());
    /* Need nr_channels: */
    if jp2.color.jp2_pclr.is_null() {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Need to read a PCLR box before the CMAP box.\n",
      );
      return 0i32;
    }
    /* Part 1, I.5.3.5: 'There shall be at most one Component Mapping box
     * inside a JP2 Header box' :
     */
    if !(*jp2.color.jp2_pclr).cmap.is_null() {
      event_msg!(p_manager, EVT_ERROR, "Only one CMAP box is allowed.\n",); /* CMP^i */
      return 0i32;
    } /* MTYP^i */
    nr_channels = (*jp2.color.jp2_pclr).nr_channels; /* PCOL^i */
    if p_cmap_header_size < (nr_channels as OPJ_UINT32).wrapping_mul(4u32) {
      event_msg!(p_manager, EVT_ERROR, "Insufficient data for CMAP box.\n",);
      return 0i32;
    }
    cmap =
      opj_malloc((nr_channels as usize).wrapping_mul(core::mem::size_of::<opj_jp2_cmap_comp_t>()))
        as *mut opj_jp2_cmap_comp_t;
    if cmap.is_null() {
      return 0i32;
    }
    i = 0 as OPJ_BYTE;
    while (i as core::ffi::c_int) < nr_channels as core::ffi::c_int {
      opj_read_bytes(p_cmap_header_data, &mut l_value, 2 as OPJ_UINT32);
      p_cmap_header_data = p_cmap_header_data.offset(2);
      (*cmap.offset(i as isize)).cmp = l_value as OPJ_UINT16;
      opj_read_bytes(p_cmap_header_data, &mut l_value, 1 as OPJ_UINT32);
      p_cmap_header_data = p_cmap_header_data.offset(1);
      (*cmap.offset(i as isize)).mtyp = l_value as OPJ_BYTE;
      opj_read_bytes(p_cmap_header_data, &mut l_value, 1 as OPJ_UINT32);
      p_cmap_header_data = p_cmap_header_data.offset(1);
      (*cmap.offset(i as isize)).pcol = l_value as OPJ_BYTE;
      i += 1;
    }
    (*jp2.color.jp2_pclr).cmap = cmap;
    1i32
  }
}

fn opj_jp2_apply_cdef(
  mut image: &mut opj_image_t,
  mut color: *mut opj_jp2_color_t,
  mut manager: &mut opj_event_mgr,
) {
  unsafe {
    let mut info = std::ptr::null_mut::<opj_jp2_cdef_info_t>();
    let mut i: OPJ_UINT16 = 0;
    let mut n: OPJ_UINT16 = 0;
    let mut cn: OPJ_UINT16 = 0;
    let mut asoc: OPJ_UINT16 = 0;
    let mut acn: OPJ_UINT16 = 0;
    info = (*(*color).jp2_cdef).info;
    n = (*(*color).jp2_cdef).n;
    i = 0 as OPJ_UINT16;
    while (i as core::ffi::c_int) < n as core::ffi::c_int {
      /* WATCH: acn = asoc - 1 ! */
      asoc = (*info.offset(i as isize)).asoc;
      cn = (*info.offset(i as isize)).cn;
      if cn as core::ffi::c_uint >= image.numcomps {
        event_msg!(
          manager,
          EVT_WARNING,
          "opj_jp2_apply_cdef: cn=%d, numcomps=%d\n",
          cn as core::ffi::c_int,
          image.numcomps,
        );
      } else if asoc as core::ffi::c_int == 0i32 || asoc as core::ffi::c_int == 65535i32 {
        (*image.comps.offset(cn as isize)).alpha = (*info.offset(i as isize)).typ
      } else {
        acn = (asoc as core::ffi::c_int - 1i32) as OPJ_UINT16;
        if acn as core::ffi::c_uint >= image.numcomps {
          event_msg!(
            manager,
            EVT_WARNING,
            "opj_jp2_apply_cdef: acn=%d, numcomps=%d\n",
            acn as core::ffi::c_int,
            image.numcomps,
          );
        } else {
          /* Swap only if color channel */
          if cn as core::ffi::c_int != acn as core::ffi::c_int
            && (*info.offset(i as isize)).typ as core::ffi::c_int == 0i32
          {
            let mut saved = opj_image_comp_t {
              dx: 0,
              dy: 0,
              w: 0,
              h: 0,
              x0: 0,
              y0: 0,
              prec: 0,
              bpp: 0,
              sgnd: 0,
              resno_decoded: 0,
              factor: 0,
              data: std::ptr::null_mut::<OPJ_INT32>(),
              alpha: 0,
            };
            let mut j: OPJ_UINT16 = 0;
            memcpy(
              &mut saved as *mut opj_image_comp_t as *mut core::ffi::c_void,
              &mut *image.comps.offset(cn as isize) as *mut opj_image_comp_t
                as *const core::ffi::c_void,
              core::mem::size_of::<opj_image_comp_t>(),
            );
            memcpy(
              &mut *image.comps.offset(cn as isize) as *mut opj_image_comp_t
                as *mut core::ffi::c_void,
              &mut *image.comps.offset(acn as isize) as *mut opj_image_comp_t
                as *const core::ffi::c_void,
              core::mem::size_of::<opj_image_comp_t>(),
            );
            memcpy(
              &mut *image.comps.offset(acn as isize) as *mut opj_image_comp_t
                as *mut core::ffi::c_void,
              &mut saved as *mut opj_image_comp_t as *const core::ffi::c_void,
              core::mem::size_of::<opj_image_comp_t>(),
            );
            /* Swap channels in following channel definitions, don't bother with j <= i that are already processed */
            j = (i as core::ffi::c_uint).wrapping_add(1u32) as OPJ_UINT16;
            while (j as core::ffi::c_int) < n as core::ffi::c_int {
              if (*info.offset(j as isize)).cn as core::ffi::c_int == cn as core::ffi::c_int {
                (*info.offset(j as isize)).cn = acn
              } else if (*info.offset(j as isize)).cn as core::ffi::c_int == acn as core::ffi::c_int
              {
                (*info.offset(j as isize)).cn = cn
              }
              j += 1;
              /* asoc is related to color index. Do not update. */
            }
          }
          (*image.comps.offset(cn as isize)).alpha = (*info.offset(i as isize)).typ
        }
      }
      i += 1;
    }
    if !(*(*color).jp2_cdef).info.is_null() {
      opj_free((*(*color).jp2_cdef).info as *mut core::ffi::c_void);
    }
    opj_free((*color).jp2_cdef as *mut core::ffi::c_void);
    (*color).jp2_cdef = std::ptr::null_mut::<opj_jp2_cdef_t>();
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
    let mut cdef_info = std::ptr::null_mut::<opj_jp2_cdef_info_t>();
    let mut i: OPJ_UINT16 = 0;
    let mut l_value: OPJ_UINT32 = 0;
    /* preconditions */

    assert!(!p_cdef_header_data.is_null());
    /* Part 1, I.5.3.6: 'The shall be at most one Channel Definition box
     * inside a JP2 Header box.'*/
    if !jp2.color.jp2_cdef.is_null() {
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
    cdef_info =
      opj_malloc((l_value as usize).wrapping_mul(core::mem::size_of::<opj_jp2_cdef_info_t>()))
        as *mut opj_jp2_cdef_info_t;
    if cdef_info.is_null() {
      return 0i32;
    }
    jp2.color.jp2_cdef = opj_malloc(core::mem::size_of::<opj_jp2_cdef_t>()) as *mut opj_jp2_cdef_t;
    if jp2.color.jp2_cdef.is_null() {
      opj_free(cdef_info as *mut core::ffi::c_void);
      return 0i32;
    }
    (*jp2.color.jp2_cdef).info = cdef_info;
    (*jp2.color.jp2_cdef).n = l_value as OPJ_UINT16;
    i = 0 as OPJ_UINT16;
    while (i as core::ffi::c_int) < (*jp2.color.jp2_cdef).n as core::ffi::c_int {
      opj_read_bytes(p_cdef_header_data, &mut l_value, 2 as OPJ_UINT32);
      p_cdef_header_data = p_cdef_header_data.offset(2);
      (*cdef_info.offset(i as isize)).cn = l_value as OPJ_UINT16;
      opj_read_bytes(p_cdef_header_data, &mut l_value, 2 as OPJ_UINT32);
      p_cdef_header_data = p_cdef_header_data.offset(2);
      (*cdef_info.offset(i as isize)).typ = l_value as OPJ_UINT16;
      opj_read_bytes(p_cdef_header_data, &mut l_value, 2 as OPJ_UINT32);
      p_cdef_header_data = p_cdef_header_data.offset(2);
      (*cdef_info.offset(i as isize)).asoc = l_value as OPJ_UINT16;
      i += 1;
    }
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

      if !jp2.color.jp2_pclr.is_null() {
        /* Part 1, I.5.3.4: Either both or none : */
        if (*jp2.color.jp2_pclr).cmap.is_null() {
          opj_jp2_free_pclr(&mut jp2.color);
        } else if opj_jp2_apply_pclr(p_image, &mut jp2.color, p_manager) == 0 {
          return 0i32;
        }
      }
      /* Apply the color space if needed */
      if !jp2.color.jp2_cdef.is_null() {
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
  if !jp2.color.jp2_cdef.is_null() {
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
    if !writer.write(stream, p_manager) {
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
  unsafe {
    let mut i: OPJ_UINT32 = 0;
    let mut l_ftyp_size: OPJ_UINT32 = 0;
    let mut l_ftyp_data = std::ptr::null_mut::<OPJ_BYTE>();
    let mut l_current_data_ptr = std::ptr::null_mut::<OPJ_BYTE>();
    let mut l_result: OPJ_BOOL = 0;
    /* preconditions */
    /* FTYP */
    /* MinV */
    l_ftyp_size = (16u32).wrapping_add((4u32).wrapping_mul(jp2.numcl));
    l_ftyp_data = opj_calloc(1i32 as size_t, l_ftyp_size as size_t) as *mut OPJ_BYTE;
    if l_ftyp_data.is_null() {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Not enough memory to handle ftyp data\n",
      );
      return 0i32;
    }
    l_current_data_ptr = l_ftyp_data;
    opj_write_bytes(l_current_data_ptr, l_ftyp_size, 4 as OPJ_UINT32);
    l_current_data_ptr = l_current_data_ptr.offset(4);
    opj_write_bytes(
      l_current_data_ptr,
      0x66747970 as OPJ_UINT32,
      4 as OPJ_UINT32,
    );
    l_current_data_ptr = l_current_data_ptr.offset(4);
    opj_write_bytes(l_current_data_ptr, jp2.brand, 4 as OPJ_UINT32);
    l_current_data_ptr = l_current_data_ptr.offset(4);
    opj_write_bytes(l_current_data_ptr, jp2.minversion, 4 as OPJ_UINT32);
    l_current_data_ptr = l_current_data_ptr.offset(4);
    i = 0 as OPJ_UINT32;
    while i < jp2.numcl {
      opj_write_bytes(
        l_current_data_ptr,
        *jp2.cl.offset(i as isize),
        4 as OPJ_UINT32,
      );
      i += 1;
      /* CL */
    }
    l_result = (opj_stream_write_data(stream, l_ftyp_data, l_ftyp_size as OPJ_SIZE_T, p_manager)
      == l_ftyp_size as usize) as core::ffi::c_int;
    if l_result == 0 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Error while writing ftyp data to stream\n",
      );
    }
    opj_free(l_ftyp_data as *mut core::ffi::c_void);
    l_result
  }
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
  unsafe {
    let mut j2k_codestream_exit: OPJ_OFF_T = 0;
    let mut l_data_header: [OPJ_BYTE; 8] = [0; 8];
    /* preconditions */
    /* JP2C */

    assert!(opj_stream_has_seek(stream) != 0);
    j2k_codestream_exit = opj_stream_tell(stream);
    opj_write_bytes(
      l_data_header.as_mut_ptr(),
      (j2k_codestream_exit - jp2.j2k_codestream_offset) as OPJ_UINT32,
      4 as OPJ_UINT32,
    );
    opj_write_bytes(
      l_data_header.as_mut_ptr().offset(4),
      0x6a703263 as OPJ_UINT32,
      4 as OPJ_UINT32,
    );
    if opj_stream_seek(stream, jp2.j2k_codestream_offset, p_manager) == 0 {
      event_msg!(p_manager, EVT_ERROR, "Failed to seek in the stream.\n",);
      return 0i32;
    }
    if opj_stream_write_data(
      stream,
      l_data_header.as_mut_ptr(),
      8 as OPJ_SIZE_T,
      p_manager,
    ) != 8
    {
      event_msg!(p_manager, EVT_ERROR, "Failed to seek in the stream.\n",);
      return 0i32;
    }
    if opj_stream_seek(stream, j2k_codestream_exit, p_manager) == 0 {
      event_msg!(p_manager, EVT_ERROR, "Failed to seek in the stream.\n",);
      return 0i32;
    }
    1i32
  }
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
  unsafe {
    /* 12 bytes will be read */
    let mut l_signature_data: [OPJ_BYTE; 12] = [0; 12];
    /* preconditions */

    /* write box length */
    opj_write_bytes(
      l_signature_data.as_mut_ptr(),
      12 as OPJ_UINT32,
      4 as OPJ_UINT32,
    );
    /* writes box type */
    opj_write_bytes(
      l_signature_data.as_mut_ptr().offset(4),
      0x6a502020 as OPJ_UINT32,
      4 as OPJ_UINT32,
    );
    /* writes magic number*/
    opj_write_bytes(
      l_signature_data.as_mut_ptr().offset(8),
      0xd0a870a as OPJ_UINT32,
      4 as OPJ_UINT32,
    );
    if opj_stream_write_data(
      stream,
      l_signature_data.as_mut_ptr(),
      12 as OPJ_SIZE_T,
      p_manager,
    ) != 12
    {
      return 0i32;
    }
    1i32
  }
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
  unsafe {
    let mut i: OPJ_UINT32 = 0;
    let mut depth_0: OPJ_UINT32 = 0;
    let mut sign: OPJ_UINT32 = 0;
    let mut alpha_count: OPJ_UINT32 = 0;
    let mut color_channels = 0u32;
    let mut alpha_channel = 0u32;
    /* setup the J2K codec */
    /* ------------------- */
    /* Check if number of components respects standard */
    if image.numcomps < 1u32 || image.numcomps > 16384u32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Invalid number of components specified while setting up JP2 encoder\n",
      );
      return 0i32;
    }
    if opj_j2k_setup_encoder(&mut jp2.j2k, parameters, image, p_manager) == 0i32 {
      return 0i32;
    }
    /* setup the JP2 codec */
    /* ------------------- */
    /* Profile box */
    jp2.brand = 0x6a703220 as OPJ_UINT32; /* BR */
    jp2.minversion = 0 as OPJ_UINT32; /* MinV */
    jp2.numcl = 1 as OPJ_UINT32; /* CL0 : JP2 */
    jp2.cl = opj_malloc((jp2.numcl as usize).wrapping_mul(core::mem::size_of::<OPJ_UINT32>()))
      as *mut OPJ_UINT32;
    if jp2.cl.is_null() {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Not enough memory when setup the JP2 encoder\n",
      );
      return 0i32;
    }
    *jp2.cl.offset(0) = 0x6a703220 as OPJ_UINT32;
    /* Image Header box */
    jp2.numcomps = image.numcomps; /* NC */
    jp2.comps =
      opj_malloc((jp2.numcomps as usize).wrapping_mul(core::mem::size_of::<opj_jp2_comps_t>()))
        as *mut opj_jp2_comps_t;
    if jp2.comps.is_null() {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Not enough memory when setup the JP2 encoder\n",
      );
      /* Memory of jp2->cl will be freed by opj_jp2_destroy */
      return 0i32;
    } /* HEIGHT */
    jp2.h = image.y1.wrapping_sub(image.y0); /* WIDTH */
    jp2.w = image.x1.wrapping_sub(image.x0);
    /* BPC */
    depth_0 = (*image.comps.offset(0)).prec.wrapping_sub(1u32); /* C : Always 7 */
    sign = (*image.comps.offset(0)).sgnd; /* UnkC, colorspace specified in colr box */
    jp2.bpc = depth_0.wrapping_add(sign << 7i32); /* IPR, no intellectual property */
    i = 1 as OPJ_UINT32;
    while i < image.numcomps {
      let mut depth = (*image.comps.offset(i as isize)).prec.wrapping_sub(1u32);
      sign = (*image.comps.offset(i as isize)).sgnd;
      if depth_0 != depth {
        jp2.bpc = 255 as OPJ_UINT32
      }
      i += 1;
    }
    jp2.C = 7 as OPJ_UINT32;
    jp2.UnkC = 0 as OPJ_UINT32;
    jp2.IPR = 0 as OPJ_UINT32;
    /* BitsPerComponent box */
    i = 0 as OPJ_UINT32;
    while i < image.numcomps {
      (*jp2.comps.offset(i as isize)).bpcc = (*image.comps.offset(i as isize))
        .prec
        .wrapping_sub(1u32)
        .wrapping_add((*image.comps.offset(i as isize)).sgnd << 7i32);
      i += 1;
    }
    /* Colour Specification box */
    if image.icc_profile_len != 0 {
      jp2.meth = 2 as OPJ_UINT32;
      jp2.enumcs = 0 as OPJ_UINT32
    } else {
      jp2.meth = 1 as OPJ_UINT32;
      if image.color_space as core::ffi::c_int == 1i32 {
        jp2.enumcs = 16 as OPJ_UINT32
      /* sRGB as defined by IEC 61966-2-1 */
      } else if image.color_space as core::ffi::c_int == 2i32 {
        jp2.enumcs = 17 as OPJ_UINT32
      /* greyscale */
      } else if image.color_space as core::ffi::c_int == 3i32 {
        jp2.enumcs = 18 as OPJ_UINT32
        /* YUV */
      }
    }
    /* Channel Definition box */
    /* FIXME not provided by parameters */
    /* We try to do what we can... */
    alpha_count = 0u32;
    i = 0 as OPJ_UINT32;
    while i < image.numcomps {
      if (*image.comps.offset(i as isize)).alpha as core::ffi::c_int != 0i32 {
        alpha_count = alpha_count.wrapping_add(1);
        alpha_channel = i
      }
      i += 1;
    }
    if alpha_count == 1u32 {
      /* no way to deal with more than 1 alpha channel */
      match jp2.enumcs {
        16 | 18 => color_channels = 3 as OPJ_UINT32,
        17 => color_channels = 1 as OPJ_UINT32,
        _ => alpha_count = 0u32,
      }
      if alpha_count == 0u32 {
        event_msg!(
          p_manager,
          EVT_WARNING,
          "Alpha channel specified but unknown enumcs. No cdef box will be created.\n",
        );
      } else if image.numcomps < color_channels.wrapping_add(1u32) {
        event_msg!(p_manager, EVT_WARNING,
                          "Alpha channel specified but not enough image components for an automatic cdef box creation.\n");
        alpha_count = 0u32
      } else if alpha_channel < color_channels {
        event_msg!(
          p_manager,
          EVT_WARNING,
          "Alpha channel position conflicts with color channel. No cdef box will be created.\n",
        );
        alpha_count = 0u32
      }
    } else if alpha_count > 1u32 {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "Multiple alpha channels specified. No cdef box will be created.\n",
      );
    }
    if alpha_count == 1u32 {
      /* if here, we know what we can do */
      jp2.color.jp2_cdef =
        opj_malloc(core::mem::size_of::<opj_jp2_cdef_t>()) as *mut opj_jp2_cdef_t;
      if jp2.color.jp2_cdef.is_null() {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Not enough memory to setup the JP2 encoder\n",
        );
        return 0i32;
      }
      /* no memset needed, all values will be overwritten except if jp2->color.jp2_cdef->info allocation fails, */
      /* in which case jp2->color.jp2_cdef->info will be NULL => valid for destruction */
      (*jp2.color.jp2_cdef).info = opj_malloc(
        (image.numcomps as usize).wrapping_mul(core::mem::size_of::<opj_jp2_cdef_info_t>()),
      ) as *mut opj_jp2_cdef_info_t;
      if (*jp2.color.jp2_cdef).info.is_null() {
        /* memory will be freed by opj_jp2_destroy */
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Not enough memory to setup the JP2 encoder\n",
        ); /* cast is valid : image->numcomps [1,16384] */
        return 0i32;
      } /* cast is valid : image->numcomps [1,16384] */
      (*jp2.color.jp2_cdef).n = image.numcomps as OPJ_UINT16;
      i = 0u32;
      while i < color_channels {
        (*(*jp2.color.jp2_cdef).info.offset(i as isize)).cn = i as OPJ_UINT16;
        (*(*jp2.color.jp2_cdef).info.offset(i as isize)).typ = 0 as OPJ_UINT16;
        (*(*jp2.color.jp2_cdef).info.offset(i as isize)).asoc = i.wrapping_add(1u32) as OPJ_UINT16;
        i += 1;
        /* No overflow + cast is valid : image->numcomps [1,16384] */
      }
      while i < image.numcomps {
        if (*image.comps.offset(i as isize)).alpha as core::ffi::c_int != 0i32 {
          /* we'll be here exactly once */
          (*(*jp2.color.jp2_cdef).info.offset(i as isize)).cn = i as OPJ_UINT16; /* cast is valid : image->numcomps [1,16384] */
          /* Apply alpha channel to the whole image */
          (*(*jp2.color.jp2_cdef).info.offset(i as isize)).typ = 1 as OPJ_UINT16; /* Opacity channel */
          (*(*jp2.color.jp2_cdef).info.offset(i as isize)).asoc = 0 as OPJ_UINT16
        } else {
          /* Unknown channel */
          (*(*jp2.color.jp2_cdef).info.offset(i as isize)).cn = i as OPJ_UINT16; /* cast is valid : image->numcomps [1,16384] */
          (*(*jp2.color.jp2_cdef).info.offset(i as isize)).typ = 65535 as OPJ_UINT16; /* PRECEDENCE */
          (*(*jp2.color.jp2_cdef).info.offset(i as isize)).asoc = 65535 as OPJ_UINT16
        } /* APPROX */
        i += 1;
      }
    }
    jp2.precedence = 0 as OPJ_UINT32;
    jp2.approx = 0 as OPJ_UINT32;
    jp2.jpip_on = parameters.jpip_on;
    1i32
  }
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

  /* customization of the end encoding */
  if opj_jp2_setup_end_header_reading(jp2, p_manager) == 0 {
    return 0i32;
  }
  /* write header */
  if opj_jp2_exec(jp2, jp2.m_procedure_list, stream, p_manager) == 0 {
    return 0i32;
  }
  opj_j2k_end_decompress(&mut jp2.j2k, stream, p_manager)
}

pub(crate) fn opj_jp2_end_compress(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* preconditions */

  /* customization of the end encoding */
  if opj_jp2_setup_end_header_writing(jp2, p_manager) == 0 {
    return 0i32;
  }
  if opj_j2k_end_compress(&mut jp2.j2k, stream, p_manager) == 0 {
    return 0i32;
  }
  /* write header */
  opj_jp2_exec(jp2, jp2.m_procedure_list, stream, p_manager)
}

/* *
 * Sets up the procedures to do on writing header after the codestream.
 * Developers wanting to extend the library can add their own writing procedures.
 */
fn opj_jp2_setup_end_header_writing(
  mut jp2: &mut opj_jp2,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    /* preconditions */

    if opj_procedure_list_add_procedure(jp2.m_procedure_list, opj_jp2_write_jp2c, p_manager) == 0 {
      return 0i32;
    }
    /* DEVELOPER CORNER, add your custom procedures */
    1i32
  }
}
/* *
 * Sets up the procedures to do on reading header after the codestream.
 * Developers wanting to extend the library can add their own writing procedures.
 */
fn opj_jp2_setup_end_header_reading(
  mut jp2: &mut opj_jp2,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    /* preconditions */

    if opj_procedure_list_add_procedure(
      jp2.m_procedure_list,
      opj_jp2_read_header_procedure,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    /* DEVELOPER CORNER, add your custom procedures */
    1i32
  }
}
fn opj_jp2_default_validation(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut _p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    let mut l_is_valid = 1i32;
    let mut i: OPJ_UINT32 = 0;
    /* preconditions */

    /* JPEG2000 codec validation */
    /* STATE checking */
    /* make sure the state is at 0 */
    l_is_valid &= (jp2.jp2_state == JP2_STATE_NONE as core::ffi::c_uint) as core::ffi::c_int;
    /* make sure not reading a jp2h ???? WEIRD */
    l_is_valid &=
      (jp2.jp2_img_state == JP2_IMG_STATE_NONE as core::ffi::c_uint) as core::ffi::c_int;
    /* POINTER validation */
    /* make sure a procedure list is present */
    l_is_valid &= (!jp2.m_procedure_list.is_null()) as core::ffi::c_int;
    /* make sure a validation list is present */
    l_is_valid &= (!jp2.m_validation_list.is_null()) as core::ffi::c_int;
    /* PARAMETER VALIDATION */
    /* number of components */
    l_is_valid &= (jp2.numcl > 0u32) as core::ffi::c_int;
    /* width */
    l_is_valid &= (jp2.h > 0u32) as core::ffi::c_int;
    /* height */
    l_is_valid &= (jp2.w > 0u32) as core::ffi::c_int;
    /* precision */
    i = 0 as OPJ_UINT32;
    while i < jp2.numcomps {
      l_is_valid &= (((*jp2.comps.offset(i as isize)).bpcc & 0x7fu32) < 38u32) as core::ffi::c_int;
      i += 1;
      /* 0 is valid, ignore sign for check */
    }
    /* METH */
    l_is_valid &= (jp2.meth > 0u32 && jp2.meth < 3u32) as core::ffi::c_int;
    /* stream validation */
    /* back and forth is needed */
    l_is_valid &= opj_stream_has_seek(stream);
    l_is_valid
  }
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
  unsafe {
    let mut box_0 = Jp2BoxHeader { length: 0, ty: 0 };
    let mut l_nb_bytes_read: OPJ_UINT32 = 0;
    let mut l_current_handler = std::ptr::null::<opj_jp2_header_handler_t>();
    let mut l_current_handler_misplaced = std::ptr::null::<opj_jp2_header_handler_t>();
    let mut l_last_data_size = 1024 as OPJ_UINT32;
    let mut l_current_data_size: OPJ_UINT32 = 0;
    let mut l_current_data = std::ptr::null_mut::<OPJ_BYTE>();
    /* preconditions */

    l_current_data = opj_calloc(1i32 as size_t, l_last_data_size as size_t) as *mut OPJ_BYTE;
    if l_current_data.is_null() {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Not enough memory to handle jpeg2000 file header\n",
      );
      return 0i32;
    }
    while opj_jp2_read_boxhdr(&mut box_0, &mut l_nb_bytes_read, stream, p_manager) != 0 {
      /* is it the codestream box ? */
      if box_0.ty == 0x6a703263u32 {
        if jp2.jp2_state & JP2_STATE_HEADER as core::ffi::c_uint != 0 {
          jp2.jp2_state |= JP2_STATE_CODESTREAM as core::ffi::c_uint;
          opj_free(l_current_data as *mut core::ffi::c_void);
          return 1i32;
        } else {
          event_msg!(p_manager, EVT_ERROR, "bad placed jpeg codestream\n",);
          opj_free(l_current_data as *mut core::ffi::c_void);
          return 0i32;
        }
      } else if box_0.length == 0u32 {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Cannot handle box of undefined sizes\n",
        );
        opj_free(l_current_data as *mut core::ffi::c_void);
        return 0i32;
      } else {
        /* testcase 1851.pdf.SIGSEGV.ce9.948 */
        if box_0.length < l_nb_bytes_read {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "invalid box size %d (%x)\n",
            box_0.length,
            box_0.ty,
          );
          opj_free(l_current_data as *mut core::ffi::c_void);
          return 0i32;
        }
      }
      l_current_handler = opj_jp2_find_handler(box_0.ty);
      l_current_handler_misplaced = opj_jp2_img_find_handler(box_0.ty);
      l_current_data_size = box_0.length.wrapping_sub(l_nb_bytes_read);
      if !l_current_handler.is_null() || !l_current_handler_misplaced.is_null() {
        if l_current_handler.is_null() {
          event_msg!(
            p_manager,
            EVT_WARNING,
            "Found a misplaced \'%c%c%c%c\' box outside jp2h box\n",
            (box_0.ty >> 24i32) as OPJ_BYTE as core::ffi::c_int,
            (box_0.ty >> 16i32) as OPJ_BYTE as core::ffi::c_int,
            (box_0.ty >> 8i32) as OPJ_BYTE as core::ffi::c_int,
            box_0.ty as OPJ_BYTE as core::ffi::c_int,
          );
          if jp2.jp2_state & JP2_STATE_HEADER as core::ffi::c_uint != 0 {
            /* read anyway, we already have jp2h */
            l_current_handler = l_current_handler_misplaced
          } else {
            event_msg!(
              p_manager,
              EVT_WARNING,
              "JPEG2000 Header box not read yet, \'%c%c%c%c\' box will be ignored\n",
              (box_0.ty >> 24i32) as OPJ_BYTE as core::ffi::c_int,
              (box_0.ty >> 16i32) as OPJ_BYTE as core::ffi::c_int,
              (box_0.ty >> 8i32) as OPJ_BYTE as core::ffi::c_int,
              box_0.ty as OPJ_BYTE as core::ffi::c_int,
            );
            jp2.jp2_state |= JP2_STATE_UNKNOWN as core::ffi::c_uint;
            if opj_stream_skip(stream, l_current_data_size as OPJ_OFF_T, p_manager)
              != l_current_data_size as i64
            {
              event_msg!(
                p_manager,
                EVT_ERROR,
                "Problem with skipping JPEG2000 box, stream error\n",
              );
              opj_free(l_current_data as *mut core::ffi::c_void);
              return 0i32;
            }
            continue;
          }
        }
        if l_current_data_size as OPJ_OFF_T > opj_stream_get_number_byte_left(stream) {
          /* do not even try to malloc if we can't read */
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Invalid box size %d for box \'%c%c%c%c\'. Need %d bytes, %d bytes remaining \n",
            box_0.length,
            (box_0.ty >> 24i32) as OPJ_BYTE as core::ffi::c_int,
            (box_0.ty >> 16i32) as OPJ_BYTE as core::ffi::c_int,
            (box_0.ty >> 8i32) as OPJ_BYTE as core::ffi::c_int,
            box_0.ty as OPJ_BYTE as core::ffi::c_int,
            l_current_data_size,
            opj_stream_get_number_byte_left(stream) as OPJ_UINT32,
          );
          opj_free(l_current_data as *mut core::ffi::c_void);
          return 0i32;
        }
        if l_current_data_size > l_last_data_size {
          let mut new_current_data = opj_realloc(
            l_current_data as *mut core::ffi::c_void,
            l_current_data_size as size_t,
          ) as *mut OPJ_BYTE;
          if new_current_data.is_null() {
            opj_free(l_current_data as *mut core::ffi::c_void);
            event_msg!(
              p_manager,
              EVT_ERROR,
              "Not enough memory to handle jpeg2000 box\n",
            );
            return 0i32;
          }
          l_current_data = new_current_data;
          l_last_data_size = l_current_data_size
        }
        l_nb_bytes_read = opj_stream_read_data(
          stream,
          l_current_data,
          l_current_data_size as OPJ_SIZE_T,
          p_manager,
        ) as OPJ_UINT32;
        if l_nb_bytes_read != l_current_data_size {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Problem with reading JPEG2000 box, stream error\n",
          );
          opj_free(l_current_data as *mut core::ffi::c_void);
          return 0i32;
        }
        if (*l_current_handler)
          .handler
          .expect("non-null function pointer")(
          jp2, l_current_data, l_current_data_size, p_manager
        ) == 0
        {
          opj_free(l_current_data as *mut core::ffi::c_void);
          return 0i32;
        }
      } else {
        if jp2.jp2_state & JP2_STATE_SIGNATURE as core::ffi::c_uint == 0 {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Malformed JP2 file format: first box must be JPEG 2000 signature box\n",
          );
          opj_free(l_current_data as *mut core::ffi::c_void);
          return 0i32;
        }
        if jp2.jp2_state & JP2_STATE_FILE_TYPE as core::ffi::c_uint == 0 {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "Malformed JP2 file format: second box must be file type box\n",
          );
          opj_free(l_current_data as *mut core::ffi::c_void);
          return 0i32;
        }
        jp2.jp2_state |= JP2_STATE_UNKNOWN as core::ffi::c_uint;
        if opj_stream_skip(stream, l_current_data_size as OPJ_OFF_T, p_manager)
          != l_current_data_size as i64
        {
          if jp2.jp2_state & JP2_STATE_CODESTREAM as core::ffi::c_uint != 0 {
            /* If we already read the codestream, do not error out */
            /* Needed for data/input/nonregression/issue254.jp2 */
            event_msg!(
              p_manager,
              EVT_WARNING,
              "Problem with skipping JPEG2000 box, stream error\n",
            );
            opj_free(l_current_data as *mut core::ffi::c_void);
            return 1i32;
          } else {
            event_msg!(
              p_manager,
              EVT_ERROR,
              "Problem with skipping JPEG2000 box, stream error\n",
            );
            opj_free(l_current_data as *mut core::ffi::c_void);
            return 0i32;
          }
        }
      }
    }
    opj_free(l_current_data as *mut core::ffi::c_void);
    1i32
  }
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
  mut jp2: &mut opj_jp2,
  mut p_procedure_list: *mut opj_jp2_proc_list_t,
  mut stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  assert!(!p_procedure_list.is_null());

  opj_procedure_list_execute(p_procedure_list, |p| unsafe {
    (p)(jp2, stream, p_manager) != 0
  }) as i32
}

pub(crate) fn opj_jp2_start_compress(
  mut jp2: &mut opj_jp2,
  mut stream: &mut Stream,
  mut p_image: &mut opj_image,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* preconditions */

  /* customization of the validation */
  if opj_jp2_setup_encoding_validation(jp2, p_manager) == 0 {
    return 0i32;
  }
  /* validation of the parameters codec */
  if opj_jp2_exec(jp2, jp2.m_validation_list, stream, p_manager) == 0 {
    return 0i32;
  }
  /* customization of the encoding */
  if opj_jp2_setup_header_writing(jp2, p_manager) == 0 {
    return 0i32;
  }
  /* write header */
  if opj_jp2_exec(jp2, jp2.m_procedure_list, stream, p_manager) == 0 {
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
fn opj_jp2_find_handler(mut p_id: OPJ_UINT32) -> *const opj_jp2_header_handler_t {
  unsafe {
    let mut i: OPJ_UINT32 = 0;
    let mut l_handler_size = core::mem::size_of::<[opj_jp2_header_handler_t; 3]>()
      .wrapping_div(core::mem::size_of::<opj_jp2_header_handler_t>())
      as OPJ_UINT32;
    i = 0 as OPJ_UINT32;
    while i < l_handler_size {
      if jp2_header[i as usize].id == p_id {
        return &*jp2_header.as_ptr().offset(i as isize) as *const opj_jp2_header_handler_t;
      }
      i += 1;
    }
    std::ptr::null::<opj_jp2_header_handler_t>()
  }
}

/* *
 * Finds the image execution function related to the given box id.
 *
 * @param   p_id    the id of the handler to fetch.
 *
 * @return  the given handler or 00 if it could not be found.
 */
fn opj_jp2_img_find_handler(mut p_id: OPJ_UINT32) -> *const opj_jp2_header_handler_t {
  unsafe {
    let mut i: OPJ_UINT32 = 0;
    let mut l_handler_size = core::mem::size_of::<[opj_jp2_header_handler_t; 6]>()
      .wrapping_div(core::mem::size_of::<opj_jp2_header_handler_t>())
      as OPJ_UINT32;
    i = 0 as OPJ_UINT32;
    while i < l_handler_size {
      if jp2_img_header[i as usize].id == p_id {
        return &*jp2_img_header.as_ptr().offset(i as isize) as *const opj_jp2_header_handler_t;
      }
      i += 1;
    }
    std::ptr::null::<opj_jp2_header_handler_t>()
  }
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
    let mut i: OPJ_UINT32 = 0;
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
    jp2.numcl = l_remaining_bytes >> 2i32; /* CLi */
    if jp2.numcl != 0 {
      jp2.cl =
        opj_calloc(jp2.numcl as size_t, core::mem::size_of::<OPJ_UINT32>()) as *mut OPJ_UINT32;
      if jp2.cl.is_null() {
        event_msg!(p_manager, EVT_ERROR, "Not enough memory with FTYP Box\n",);
        return 0i32;
      }
    }
    i = 0 as OPJ_UINT32;
    while i < jp2.numcl {
      opj_read_bytes(
        p_header_data,
        &mut *jp2.cl.offset(i as isize),
        4 as OPJ_UINT32,
      );
      p_header_data = p_header_data.offset(4);
      i += 1;
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
    let mut l_current_data_size = 0 as OPJ_UINT32;
    let mut box_0 = Jp2BoxHeader { length: 0, ty: 0 };
    let mut l_current_handler = std::ptr::null::<opj_jp2_header_handler_t>();
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
      l_current_handler = opj_jp2_img_find_handler(box_0.ty);
      l_current_data_size = box_0.length.wrapping_sub(l_box_size);
      p_header_data = p_header_data.offset(l_box_size as isize);
      if !l_current_handler.is_null() {
        if (*l_current_handler)
          .handler
          .expect("non-null function pointer")(
          jp2, p_header_data, l_current_data_size, p_manager
        ) == 0
        {
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
  mut box_0: &mut Jp2BoxHeader,
  mut p_data: *mut OPJ_BYTE,
  mut p_number_bytes_read: *mut OPJ_UINT32,
  mut p_box_max_size: OPJ_UINT32,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    let mut l_value: OPJ_UINT32 = 0;
    /* preconditions */

    assert!(!p_data.is_null());
    assert!(!p_number_bytes_read.is_null());
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
  mut p_stream: &mut Stream,
  mut jp2: &mut opj_jp2,
  mut p_image: *mut *mut opj_image_t,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* customization of the validation */
  if opj_jp2_setup_decoding_validation(jp2, p_manager) == 0 {
    return 0i32;
  }
  /* customization of the encoding */
  if opj_jp2_setup_header_reading(jp2, p_manager) == 0 {
    return 0i32;
  }
  /* validation of the parameters codec */
  if opj_jp2_exec(jp2, jp2.m_validation_list, p_stream, p_manager) == 0 {
    return 0i32;
  }
  /* read header */
  if opj_jp2_exec(jp2, jp2.m_procedure_list, p_stream, p_manager) == 0 {
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
      let len = icc_profile.len();
      // Allocate raw buffer and copy data from icc_profile.
      let (icc_profile_buf, buf) = unsafe {
        let icc_profile_buf = opj_malloc(len) as *mut OPJ_BYTE;
        let mut buf = std::slice::from_raw_parts_mut(icc_profile_buf, len);
        (icc_profile_buf, buf)
      };
      buf.copy_from_slice(icc_profile.as_slice());
      image.icc_profile_buf = icc_profile_buf;
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
  mut jp2: &mut opj_jp2,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    /* preconditions */

    if opj_procedure_list_add_procedure(
      jp2.m_validation_list,
      opj_jp2_default_validation,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    /* DEVELOPER CORNER, add your custom validation procedure */
    1i32
  }
}

/* *
 * Sets up the validation ,i.e. adds the procedures to launch to make sure the codec parameters
 * are valid. Developers wanting to extend the library can add their own validation procedures.
 */
fn opj_jp2_setup_decoding_validation(
  mut _jp2: &mut opj_jp2,
  mut _p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  /* preconditions */

  /* DEVELOPER CORNER, add your custom validation procedure */
  1i32
}

/* *
 * Sets up the procedures to do on writing header. Developers wanting to extend the library can add their own writing procedures.
 */
fn opj_jp2_setup_header_writing(
  mut jp2: &mut opj_jp2,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    /* preconditions */

    if opj_procedure_list_add_procedure(jp2.m_procedure_list, opj_jp2_write_jp, p_manager) == 0 {
      return 0i32;
    }
    if opj_procedure_list_add_procedure(jp2.m_procedure_list, opj_jp2_write_ftyp, p_manager) == 0 {
      return 0i32;
    }
    if opj_procedure_list_add_procedure(jp2.m_procedure_list, opj_jp2_write_jp2h, p_manager) == 0 {
      return 0i32;
    }
    if jp2.jpip_on != 0
      && opj_procedure_list_add_procedure(jp2.m_procedure_list, opj_jpip_skip_iptr, p_manager) == 0
    {
      return 0i32;
    }
    if opj_procedure_list_add_procedure(jp2.m_procedure_list, opj_jp2_skip_jp2c, p_manager) == 0 {
      return 0i32;
    }
    /* DEVELOPER CORNER, insert your custom procedures */
    1i32
  }
}

/* *
 * Sets up the procedures to do on reading header.
 * Developers wanting to extend the library can add their own writing procedures.
 */
fn opj_jp2_setup_header_reading(
  mut jp2: &mut opj_jp2,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  unsafe {
    /* preconditions */

    if opj_procedure_list_add_procedure(
      jp2.m_procedure_list,
      opj_jp2_read_header_procedure,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    /* DEVELOPER CORNER, add your custom procedures */
    1i32
  }
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
  mut p_jp2: &mut opj_jp2,
  mut p_tile_index: OPJ_UINT32,
  mut p_data: &[u8],
  mut p_stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  opj_j2k_write_tile(&mut p_jp2.j2k, p_tile_index, p_data, p_stream, p_manager)
}

pub(crate) fn opj_jp2_decode_tile(
  mut p_jp2: &mut opj_jp2,
  mut p_tile_index: OPJ_UINT32,
  mut p_data: Option<&mut [u8]>,
  mut p_stream: &mut Stream,
  mut p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  opj_j2k_decode_tile(&mut p_jp2.j2k, p_tile_index, p_data, p_stream, p_manager)
}

impl Drop for opj_jp2 {
  fn drop(&mut self) {
    unsafe {
      if !self.comps.is_null() {
        opj_free(self.comps as *mut core::ffi::c_void);
        self.comps = std::ptr::null_mut::<opj_jp2_comps_t>()
      }
      if !self.cl.is_null() {
        opj_free(self.cl as *mut core::ffi::c_void);
        self.cl = std::ptr::null_mut::<OPJ_UINT32>()
      }
      if !self.color.jp2_cdef.is_null() {
        if !(*self.color.jp2_cdef).info.is_null() {
          opj_free((*self.color.jp2_cdef).info as *mut core::ffi::c_void);
          (*self.color.jp2_cdef).info = std::ptr::null_mut::<opj_jp2_cdef_info_t>()
        }
        opj_free(self.color.jp2_cdef as *mut core::ffi::c_void);
        self.color.jp2_cdef = std::ptr::null_mut::<opj_jp2_cdef_t>()
      }
      if !self.color.jp2_pclr.is_null() {
        if !(*self.color.jp2_pclr).cmap.is_null() {
          opj_free((*self.color.jp2_pclr).cmap as *mut core::ffi::c_void);
          (*self.color.jp2_pclr).cmap = std::ptr::null_mut::<opj_jp2_cmap_comp_t>()
        }
        if !(*self.color.jp2_pclr).channel_sign.is_null() {
          opj_free((*self.color.jp2_pclr).channel_sign as *mut core::ffi::c_void);
          (*self.color.jp2_pclr).channel_sign = std::ptr::null_mut::<OPJ_BYTE>()
        }
        if !(*self.color.jp2_pclr).channel_size.is_null() {
          opj_free((*self.color.jp2_pclr).channel_size as *mut core::ffi::c_void);
          (*self.color.jp2_pclr).channel_size = std::ptr::null_mut::<OPJ_BYTE>()
        }
        if !(*self.color.jp2_pclr).entries.is_null() {
          opj_free((*self.color.jp2_pclr).entries as *mut core::ffi::c_void);
          (*self.color.jp2_pclr).entries = std::ptr::null_mut::<OPJ_UINT32>()
        }
        opj_free(self.color.jp2_pclr as *mut core::ffi::c_void);
        self.color.jp2_pclr = std::ptr::null_mut::<opj_jp2_pclr_t>()
      }
      if !self.m_validation_list.is_null() {
        opj_procedure_list_destroy(self.m_validation_list);
        self.m_validation_list = std::ptr::null_mut();
      }
      if !self.m_procedure_list.is_null() {
        opj_procedure_list_destroy(self.m_procedure_list);
        self.m_procedure_list = std::ptr::null_mut();
      }
    }
  }
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
    numcomps: 0,
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
    numcl: 0,
    cl: std::ptr::null_mut(),
    comps: std::ptr::null_mut(),
    j2k_codestream_offset: 0,
    jpip_iptr_offset: 0,
    jpip_on: 0,
    jp2_state: 0,
    jp2_img_state: 0,
    ignore_pclr_cmap_cdef: 0,
    has_jp2h: 0,
    has_ihdr: 0,
    /* Color structure */
    color: opj_jp2_color_t {
      icc_profile: None,
      icc_profile_len: 0 as OPJ_UINT32,
      jp2_cdef: std::ptr::null_mut::<opj_jp2_cdef_t>(),
      jp2_pclr: std::ptr::null_mut::<opj_jp2_pclr_t>(),
      jp2_has_colr: 0 as OPJ_BYTE,
    },
    m_validation_list: opj_procedure_list_create(),
    m_procedure_list: opj_procedure_list_create(),
  };
  /* validation list creation */
  if jp2.m_validation_list.is_null() {
    return None;
  }
  /* execution list creation */
  if jp2.m_procedure_list.is_null() {
    return None;
  }
  Some(jp2)
}

#[cfg(feature = "file-io")]
pub(crate) fn jp2_dump(mut p_jp2: &mut opj_jp2, mut flag: OPJ_INT32, mut out_stream: *mut FILE) {
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
