use super::openjpeg::*;
use super::math::*;
use super::pi::*;
use super::cio::*;
use super::event::*;
use super::tcd::*;
use super::mct::*;
use super::dwt::*;
use super::function_list::*;
use super::invert::*;
use super::image::*;
use super::thread::*;
use ::libc;

use super::malloc::*;

use ::libc::{
  FILE,
  fprintf,
  sprintf,
};

use bitflags::bitflags;

extern "C" {
  fn strcpy(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;

  fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;

  fn strncmp(_: *const libc::c_char, _: *const libc::c_char, _: libc::c_ulong) -> libc::c_int;

  fn strlen(_: *const libc::c_char) -> libc::c_ulong;

  fn atoi(__nptr: *const libc::c_char) -> libc::c_int;

  fn getenv(__name: *const libc::c_char) -> *mut libc::c_char;

  fn floor(_: libc::c_double) -> libc::c_double;
  static mut stdout: *mut FILE;

  fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;

  fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;

  fn memmove(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
}

bitflags! {
  pub struct J2KState: u16 {
    const ERR = 32768;
    const EOC = 256;
    const DATA = 128;
    const NEOC = 64;
    const MT = 32;
    const TPH = 16;
    const TPHSOT = 8;
    const MH = 4;
    const MHSIZ = 2;
    const MHSOC = 1;
    const NONE = 0;
  }
}

pub type opj_j2k_mct_function =
  Option<unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> ()>;
/*@}*/
/*@}*/
/* ----------------------------------------------------------------------- */

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum ProgressionStep {
  Unknown = 0,
  Component = 67,
  Resolution = 82,
  Precinct = 80,
  Layer = 76,
}

impl ProgressionStep {
  pub fn as_byte(&self) -> u8 {
    match self {
      Self::Component => b'C',
      Self::Resolution => b'R',
      Self::Precinct => b'P',
      Self::Layer => b'L',
      Self::Unknown => 0,
    }
  }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum ProgressionOrder {
  Unknown = 0,
  CPRL,
  PCRL,
  RLCP,
  LRCP,
}

impl ProgressionOrder {
  pub fn from_c_enum(enum_prog: OPJ_PROG_ORDER) -> Self {
    match enum_prog {
      OPJ_CPRL => Self::CPRL,
      OPJ_PCRL => Self::PCRL,
      OPJ_RLCP => Self::RLCP,
      OPJ_LRCP => Self::LRCP,
      _ => Self::Unknown,
    }
  }

  pub fn get_order(&self) -> &'static [ProgressionStep] {
    use ProgressionStep::*;
    match self {
      Self::CPRL => &[Component, Precinct, Resolution, Layer],
      Self::PCRL => &[Precinct, Component, Resolution, Layer],
      Self::RLCP => &[Resolution, Layer, Component, Precinct],
      Self::LRCP => &[Layer, Resolution, Component, Precinct],
      Self::Unknown => &[],
    }
  }

  pub fn get_order_str(&self) -> &'static str {
    match self {
      Self::CPRL => "CPRL",
      Self::PCRL => "PCRL",
      Self::RLCP => "RLCP",
      Self::LRCP => "LRCP",
      Self::Unknown => "",
    }
  }

  pub fn get_step(&self, pos: i32) -> ProgressionStep {
    let steps = self.get_order();
    steps.get(pos as usize).cloned().unwrap_or(ProgressionStep::Unknown)
  }
}

pub type opj_dec_memory_marker_handler_t = opj_dec_memory_marker_handler;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_dec_memory_marker_handler {
  pub id: OPJ_UINT32,
  pub states: J2KState,
  pub handler: Option<
    unsafe extern "C" fn(
      _: *mut opj_j2k_t,
      _: *mut OPJ_BYTE,
      _: OPJ_UINT32,
      _: *mut opj_event_mgr_t,
    ) -> OPJ_BOOL,
  >,
}

/* * marker value */
/* * value of the state when the marker can appear */
/* * action linked to the marker */
/* *
 * Updates the Tile Length Marker.
 */
unsafe fn opj_j2k_update_tlm(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_part_size: OPJ_UINT32,
) {
  if (*p_j2k).m_specific_param.m_encoder.m_Ttlmi_is_byte != 0 {
    opj_write_bytes_LE(
      (*p_j2k)
        .m_specific_param
        .m_encoder
        .m_tlm_sot_offsets_current,
      (*p_j2k).m_current_tile_number,
      1 as OPJ_UINT32,
    ); /* PSOT */
    (*p_j2k)
      .m_specific_param
      .m_encoder
      .m_tlm_sot_offsets_current = (*p_j2k)
      .m_specific_param
      .m_encoder
      .m_tlm_sot_offsets_current
      .offset(1)
  } else {
    opj_write_bytes_LE(
      (*p_j2k)
        .m_specific_param
        .m_encoder
        .m_tlm_sot_offsets_current,
      (*p_j2k).m_current_tile_number,
      2 as OPJ_UINT32,
    );
    (*p_j2k)
      .m_specific_param
      .m_encoder
      .m_tlm_sot_offsets_current = (*p_j2k)
      .m_specific_param
      .m_encoder
      .m_tlm_sot_offsets_current
      .offset(2)
  }
  opj_write_bytes_LE(
    (*p_j2k)
      .m_specific_param
      .m_encoder
      .m_tlm_sot_offsets_current,
    p_tile_part_size,
    4 as OPJ_UINT32,
  );
  (*p_j2k)
    .m_specific_param
    .m_encoder
    .m_tlm_sot_offsets_current = (*p_j2k)
    .m_specific_param
    .m_encoder
    .m_tlm_sot_offsets_current
    .offset(4);
}

/* *
 * FIXME DOC
 */
static mut MCT_ELEMENT_SIZE: [OPJ_UINT32; 4] = [
  2 as OPJ_UINT32,
  4 as OPJ_UINT32,
  4 as OPJ_UINT32,
  8 as OPJ_UINT32,
];
static mut j2k_mct_read_functions_to_float: [opj_j2k_mct_function; 4] = [
  Some(
    opj_j2k_read_int16_to_float
      as unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> (),
  ),
  Some(
    opj_j2k_read_int32_to_float
      as unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> (),
  ),
  Some(
    opj_j2k_read_float32_to_float
      as unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> (),
  ),
  Some(
    opj_j2k_read_float64_to_float
      as unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> (),
  ),
];
static mut j2k_mct_read_functions_to_int32: [opj_j2k_mct_function; 4] = [
  Some(
    opj_j2k_read_int16_to_int32
      as unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> (),
  ),
  Some(
    opj_j2k_read_int32_to_int32
      as unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> (),
  ),
  Some(
    opj_j2k_read_float32_to_int32
      as unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> (),
  ),
  Some(
    opj_j2k_read_float64_to_int32
      as unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> (),
  ),
];
static mut j2k_mct_write_functions_from_float: [opj_j2k_mct_function; 4] = [
  Some(
    opj_j2k_write_float_to_int16
      as unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> (),
  ),
  Some(
    opj_j2k_write_float_to_int32
      as unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> (),
  ),
  Some(
    opj_j2k_write_float_to_float
      as unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> (),
  ),
  Some(
    opj_j2k_write_float_to_float64
      as unsafe extern "C" fn(_: *const libc::c_void, _: *mut libc::c_void, _: OPJ_UINT32) -> (),
  ),
];
static mut j2k_memory_marker_handler_tab: [opj_dec_memory_marker_handler_t; 23] = [
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff90 as OPJ_UINT32,
      states: J2KState::from_bits_truncate(J2KState::MH.bits() | J2KState::TPHSOT.bits()),
      handler: Some(
        opj_j2k_read_sot
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff52 as OPJ_UINT32,
      states: J2KState::from_bits_truncate(J2KState::MH.bits() | J2KState::TPH.bits()),
      handler: Some(
        opj_j2k_read_cod
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff53 as OPJ_UINT32,
      states: J2KState::from_bits_truncate(J2KState::MH.bits() | J2KState::TPH.bits()),
      handler: Some(
        opj_j2k_read_coc
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff5e as OPJ_UINT32,
      states: J2KState::from_bits_truncate(J2KState::MH.bits() | J2KState::TPH.bits()),
      handler: Some(
        opj_j2k_read_rgn
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff5c as OPJ_UINT32,
      states: J2KState::from_bits_truncate(J2KState::MH.bits() | J2KState::TPH.bits()),
      handler: Some(
        opj_j2k_read_qcd
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff5d as OPJ_UINT32,
      states: J2KState::from_bits_truncate(J2KState::MH.bits() | J2KState::TPH.bits()),
      handler: Some(
        opj_j2k_read_qcc
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff5f as OPJ_UINT32,
      states: J2KState::from_bits_truncate(J2KState::MH.bits() | J2KState::TPH.bits()),
      handler: Some(
        opj_j2k_read_poc
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff51 as OPJ_UINT32,
      states: J2KState::MHSIZ,
      handler: Some(
        opj_j2k_read_siz
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff55 as OPJ_UINT32,
      states: J2KState::MH,
      handler: Some(
        opj_j2k_read_tlm
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff57 as OPJ_UINT32,
      states: J2KState::MH,
      handler: Some(
        opj_j2k_read_plm
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff58 as OPJ_UINT32,
      states: J2KState::TPH,
      handler: Some(
        opj_j2k_read_plt
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff60 as OPJ_UINT32,
      states: J2KState::MH,
      handler: Some(
        opj_j2k_read_ppm
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff61 as OPJ_UINT32,
      states: J2KState::TPH,
      handler: Some(
        opj_j2k_read_ppt
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff91 as OPJ_UINT32,
      states: J2KState::NONE,
      handler: None,
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff63 as OPJ_UINT32,
      states: J2KState::MH,
      handler: Some(
        opj_j2k_read_crg
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff64 as OPJ_UINT32,
      states: J2KState::from_bits_truncate(J2KState::MH.bits() | J2KState::TPH.bits()),
      handler: Some(
        opj_j2k_read_com
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff74 as OPJ_UINT32,
      states: J2KState::from_bits_truncate(J2KState::MH.bits() | J2KState::TPH.bits()),
      handler: Some(
        opj_j2k_read_mct
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff78 as OPJ_UINT32,
      states: J2KState::MH,
      handler: Some(
        opj_j2k_read_cbd
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff50 as OPJ_UINT32,
      states: J2KState::MH,
      handler: Some(
        opj_j2k_read_cap
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff59 as OPJ_UINT32,
      states: J2KState::MH,
      handler: Some(
        opj_j2k_read_cpf
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff75 as OPJ_UINT32,
      states: J2KState::from_bits_truncate(J2KState::MH.bits() | J2KState::TPH.bits()),
      handler: Some(
        opj_j2k_read_mcc
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0xff77 as OPJ_UINT32,
      states: J2KState::from_bits_truncate(J2KState::MH.bits() | J2KState::TPH.bits()),
      handler: Some(
        opj_j2k_read_mco
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut OPJ_BYTE,
            _: OPJ_UINT32,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      ),
    };
    init
  },
  {
    let mut init = opj_dec_memory_marker_handler {
      id: 0 as OPJ_UINT32,
      states: J2KState::from_bits_truncate(J2KState::MH.bits() | J2KState::TPH.bits()),
      handler: None,
    };
    init
  },
];
unsafe extern "C" fn opj_j2k_read_int16_to_float(
  mut p_src_data: *const libc::c_void,
  mut p_dest_data: *mut libc::c_void,
  mut p_nb_elem: OPJ_UINT32,
) {
  let mut l_src_data = p_src_data as *mut OPJ_BYTE;
  let mut l_dest_data = p_dest_data as *mut OPJ_FLOAT32;
  let mut i: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_UINT32 = 0;
  i = 0 as OPJ_UINT32;
  while i < p_nb_elem {
    opj_read_bytes_LE(l_src_data, &mut l_temp, 2 as OPJ_UINT32);
    l_src_data = l_src_data.offset(core::mem::size_of::<OPJ_INT16>() as isize);
    let fresh0 = l_dest_data;
    l_dest_data = l_dest_data.offset(1);
    *fresh0 = l_temp as OPJ_FLOAT32;
    i = i.wrapping_add(1)
  }
}
unsafe extern "C" fn opj_j2k_read_int32_to_float(
  mut p_src_data: *const libc::c_void,
  mut p_dest_data: *mut libc::c_void,
  mut p_nb_elem: OPJ_UINT32,
) {
  let mut l_src_data = p_src_data as *mut OPJ_BYTE;
  let mut l_dest_data = p_dest_data as *mut OPJ_FLOAT32;
  let mut i: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_UINT32 = 0;
  i = 0 as OPJ_UINT32;
  while i < p_nb_elem {
    opj_read_bytes_LE(l_src_data, &mut l_temp, 4 as OPJ_UINT32);
    l_src_data = l_src_data.offset(core::mem::size_of::<OPJ_INT32>() as isize);
    let fresh1 = l_dest_data;
    l_dest_data = l_dest_data.offset(1);
    *fresh1 = l_temp as OPJ_FLOAT32;
    i = i.wrapping_add(1)
  }
}
unsafe extern "C" fn opj_j2k_read_float32_to_float(
  mut p_src_data: *const libc::c_void,
  mut p_dest_data: *mut libc::c_void,
  mut p_nb_elem: OPJ_UINT32,
) {
  let mut l_src_data = p_src_data as *mut OPJ_BYTE;
  let mut l_dest_data = p_dest_data as *mut OPJ_FLOAT32;
  let mut i: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_FLOAT32 = 0.;
  i = 0 as OPJ_UINT32;
  while i < p_nb_elem {
    opj_read_float_LE(l_src_data, &mut l_temp);
    l_src_data = l_src_data.offset(core::mem::size_of::<OPJ_FLOAT32>() as isize);
    let fresh2 = l_dest_data;
    l_dest_data = l_dest_data.offset(1);
    *fresh2 = l_temp;
    i = i.wrapping_add(1)
  }
}
unsafe extern "C" fn opj_j2k_read_float64_to_float(
  mut p_src_data: *const libc::c_void,
  mut p_dest_data: *mut libc::c_void,
  mut p_nb_elem: OPJ_UINT32,
) {
  let mut l_src_data = p_src_data as *mut OPJ_BYTE;
  let mut l_dest_data = p_dest_data as *mut OPJ_FLOAT32;
  let mut i: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_FLOAT64 = 0.;
  i = 0 as OPJ_UINT32;
  while i < p_nb_elem {
    opj_read_double_LE(l_src_data, &mut l_temp);
    l_src_data = l_src_data.offset(core::mem::size_of::<OPJ_FLOAT64>() as isize);
    let fresh3 = l_dest_data;
    l_dest_data = l_dest_data.offset(1);
    *fresh3 = l_temp as OPJ_FLOAT32;
    i = i.wrapping_add(1)
  }
}
unsafe extern "C" fn opj_j2k_read_int16_to_int32(
  mut p_src_data: *const libc::c_void,
  mut p_dest_data: *mut libc::c_void,
  mut p_nb_elem: OPJ_UINT32,
) {
  let mut l_src_data = p_src_data as *mut OPJ_BYTE;
  let mut l_dest_data = p_dest_data as *mut OPJ_INT32;
  let mut i: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_UINT32 = 0;
  i = 0 as OPJ_UINT32;
  while i < p_nb_elem {
    opj_read_bytes_LE(l_src_data, &mut l_temp, 2 as OPJ_UINT32);
    l_src_data = l_src_data.offset(core::mem::size_of::<OPJ_INT16>() as isize);
    let fresh4 = l_dest_data;
    l_dest_data = l_dest_data.offset(1);
    *fresh4 = l_temp as OPJ_INT32;
    i = i.wrapping_add(1)
  }
}
unsafe extern "C" fn opj_j2k_read_int32_to_int32(
  mut p_src_data: *const libc::c_void,
  mut p_dest_data: *mut libc::c_void,
  mut p_nb_elem: OPJ_UINT32,
) {
  let mut l_src_data = p_src_data as *mut OPJ_BYTE;
  let mut l_dest_data = p_dest_data as *mut OPJ_INT32;
  let mut i: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_UINT32 = 0;
  i = 0 as OPJ_UINT32;
  while i < p_nb_elem {
    opj_read_bytes_LE(l_src_data, &mut l_temp, 4 as OPJ_UINT32);
    l_src_data = l_src_data.offset(core::mem::size_of::<OPJ_INT32>() as isize);
    let fresh5 = l_dest_data;
    l_dest_data = l_dest_data.offset(1);
    *fresh5 = l_temp as OPJ_INT32;
    i = i.wrapping_add(1)
  }
}
unsafe extern "C" fn opj_j2k_read_float32_to_int32(
  mut p_src_data: *const libc::c_void,
  mut p_dest_data: *mut libc::c_void,
  mut p_nb_elem: OPJ_UINT32,
) {
  let mut l_src_data = p_src_data as *mut OPJ_BYTE;
  let mut l_dest_data = p_dest_data as *mut OPJ_INT32;
  let mut i: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_FLOAT32 = 0.;
  i = 0 as OPJ_UINT32;
  while i < p_nb_elem {
    opj_read_float_LE(l_src_data, &mut l_temp);
    l_src_data = l_src_data.offset(core::mem::size_of::<OPJ_FLOAT32>() as isize);
    let fresh6 = l_dest_data;
    l_dest_data = l_dest_data.offset(1);
    *fresh6 = l_temp as OPJ_INT32;
    i = i.wrapping_add(1)
  }
}
unsafe extern "C" fn opj_j2k_read_float64_to_int32(
  mut p_src_data: *const libc::c_void,
  mut p_dest_data: *mut libc::c_void,
  mut p_nb_elem: OPJ_UINT32,
) {
  let mut l_src_data = p_src_data as *mut OPJ_BYTE;
  let mut l_dest_data = p_dest_data as *mut OPJ_INT32;
  let mut i: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_FLOAT64 = 0.;
  i = 0 as OPJ_UINT32;
  while i < p_nb_elem {
    opj_read_double_LE(l_src_data, &mut l_temp);
    l_src_data = l_src_data.offset(core::mem::size_of::<OPJ_FLOAT64>() as isize);
    let fresh7 = l_dest_data;
    l_dest_data = l_dest_data.offset(1);
    *fresh7 = l_temp as OPJ_INT32;
    i = i.wrapping_add(1)
  }
}
unsafe extern "C" fn opj_j2k_write_float_to_int16(
  mut p_src_data: *const libc::c_void,
  mut p_dest_data: *mut libc::c_void,
  mut p_nb_elem: OPJ_UINT32,
) {
  let mut l_dest_data = p_dest_data as *mut OPJ_BYTE;
  let mut l_src_data = p_src_data as *mut OPJ_FLOAT32;
  let mut i: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_UINT32 = 0;
  i = 0 as OPJ_UINT32;
  while i < p_nb_elem {
    let fresh8 = l_src_data;
    l_src_data = l_src_data.offset(1);
    l_temp = *fresh8 as OPJ_UINT32;
    opj_write_bytes_LE(
      l_dest_data,
      l_temp,
      core::mem::size_of::<OPJ_INT16>() as OPJ_UINT32,
    );
    l_dest_data = l_dest_data.offset(core::mem::size_of::<OPJ_INT16>() as isize);
    i = i.wrapping_add(1)
  }
}
unsafe extern "C" fn opj_j2k_write_float_to_int32(
  mut p_src_data: *const libc::c_void,
  mut p_dest_data: *mut libc::c_void,
  mut p_nb_elem: OPJ_UINT32,
) {
  let mut l_dest_data = p_dest_data as *mut OPJ_BYTE;
  let mut l_src_data = p_src_data as *mut OPJ_FLOAT32;
  let mut i: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_UINT32 = 0;
  i = 0 as OPJ_UINT32;
  while i < p_nb_elem {
    let fresh9 = l_src_data;
    l_src_data = l_src_data.offset(1);
    l_temp = *fresh9 as OPJ_UINT32;
    opj_write_bytes_LE(
      l_dest_data,
      l_temp,
      core::mem::size_of::<OPJ_INT32>() as OPJ_UINT32,
    );
    l_dest_data = l_dest_data.offset(core::mem::size_of::<OPJ_INT32>() as isize);
    i = i.wrapping_add(1)
  }
}
unsafe extern "C" fn opj_j2k_write_float_to_float(
  mut p_src_data: *const libc::c_void,
  mut p_dest_data: *mut libc::c_void,
  mut p_nb_elem: OPJ_UINT32,
) {
  let mut l_dest_data = p_dest_data as *mut OPJ_BYTE;
  let mut l_src_data = p_src_data as *mut OPJ_FLOAT32;
  let mut i: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_FLOAT32 = 0.;
  i = 0 as OPJ_UINT32;
  while i < p_nb_elem {
    let fresh10 = l_src_data;
    l_src_data = l_src_data.offset(1);
    l_temp = *fresh10;
    opj_write_float_LE(l_dest_data, l_temp);
    l_dest_data =
      l_dest_data.offset(core::mem::size_of::<OPJ_FLOAT32>() as isize);
    i = i.wrapping_add(1)
  }
}
unsafe extern "C" fn opj_j2k_write_float_to_float64(
  mut p_src_data: *const libc::c_void,
  mut p_dest_data: *mut libc::c_void,
  mut p_nb_elem: OPJ_UINT32,
) {
  let mut l_dest_data = p_dest_data as *mut OPJ_BYTE;
  let mut l_src_data = p_src_data as *mut OPJ_FLOAT32;
  let mut i: OPJ_UINT32 = 0;
  let mut l_temp: OPJ_FLOAT64 = 0.;
  i = 0 as OPJ_UINT32;
  while i < p_nb_elem {
    let fresh11 = l_src_data;
    l_src_data = l_src_data.offset(1);
    l_temp = *fresh11 as OPJ_FLOAT64;
    opj_write_double_LE(l_dest_data, l_temp);
    l_dest_data =
      l_dest_data.offset(core::mem::size_of::<OPJ_FLOAT64>() as isize);
    i = i.wrapping_add(1)
  }
}

pub(crate) fn opj_j2k_convert_progression_order(
  prg_order: OPJ_PROG_ORDER,
) -> ProgressionOrder {
  ProgressionOrder::from_c_enum(prg_order)
}
/* *
 * Checks the progression order changes values. Tells of the poc given as input are valid.
 * A nice message is outputted at errors.
 *
 * @param       p_pocs                  the progression order changes.
 * @param       tileno                  the tile number of interest
 * @param       p_nb_pocs               the number of progression order changes.
 * @param       p_nb_resolutions        the number of resolutions.
 * @param       numcomps                the number of components
 * @param       numlayers               the number of layers.
 * @param       p_manager               the user event manager.
 *
 * @return      true if the pocs are valid.
 */
unsafe fn opj_j2k_check_poc_val(
  mut p_pocs: *const opj_poc_t,
  mut tileno: OPJ_UINT32,
  mut p_nb_pocs: OPJ_UINT32,
  mut p_nb_resolutions: OPJ_UINT32,
  mut p_num_comps: OPJ_UINT32,
  mut p_num_layers: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut packet_array = 0 as *mut OPJ_UINT32;
  let mut index: OPJ_UINT32 = 0;
  let mut resno: OPJ_UINT32 = 0;
  let mut compno: OPJ_UINT32 = 0;
  let mut layno: OPJ_UINT32 = 0;
  let mut i: OPJ_UINT32 = 0;
  let mut step_c = 1 as OPJ_UINT32;
  let mut step_r = p_num_comps.wrapping_mul(step_c);
  let mut step_l = p_nb_resolutions.wrapping_mul(step_r);
  let mut loss = 0i32;
  assert!(p_nb_pocs > 0u32);
  packet_array = opj_calloc(
    (step_l as size_t).wrapping_mul(p_num_layers as libc::c_ulong),
    core::mem::size_of::<OPJ_UINT32>() as libc::c_ulong,
  ) as *mut OPJ_UINT32;
  if packet_array.is_null() {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory for checking the poc values.\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* iterate through all the pocs that match our tile of interest. */
  i = 0 as OPJ_UINT32;
  while i < p_nb_pocs {
    let mut poc: *const opj_poc_t = &*p_pocs.offset(i as isize) as *const opj_poc_t;
    if tileno.wrapping_add(1u32) == (*poc).tile {
      index = step_r.wrapping_mul((*poc).resno0);
      /* take each resolution for each poc */
      resno = (*poc).resno0;
      while resno < opj_uint_min((*poc).resno1, p_nb_resolutions) {
        let mut res_index = index.wrapping_add((*poc).compno0.wrapping_mul(step_c));
        /* take each comp of each resolution for each poc */
        compno = (*poc).compno0;
        while compno < opj_uint_min((*poc).compno1, p_num_comps) {
          /* The layer index always starts at zero for every progression. */
          let layno0 = 0 as OPJ_UINT32;
          let mut comp_index = res_index.wrapping_add(layno0.wrapping_mul(step_l));
          /* and finally take each layer of each res of ... */
          layno = layno0;
          while layno < opj_uint_min((*poc).layno1, p_num_layers) {
            *packet_array.offset(comp_index as isize) = 1 as OPJ_UINT32;
            comp_index =
              (comp_index as libc::c_uint).wrapping_add(step_l) as OPJ_UINT32;
            layno = layno.wrapping_add(1)
          }
          res_index = (res_index as libc::c_uint).wrapping_add(step_c) as OPJ_UINT32;
          compno = compno.wrapping_add(1)
        }
        index = (index as libc::c_uint).wrapping_add(step_r) as OPJ_UINT32;
        resno = resno.wrapping_add(1)
      }
    }
    i = i.wrapping_add(1)
  }
  index = 0 as OPJ_UINT32;
  layno = 0 as OPJ_UINT32;
  while layno < p_num_layers {
    resno = 0 as OPJ_UINT32;
    while resno < p_nb_resolutions {
      compno = 0 as OPJ_UINT32;
      while compno < p_num_comps {
        loss |=
          (*packet_array.offset(index as isize) != 1u32) as libc::c_int;
        index = (index as libc::c_uint).wrapping_add(step_c) as OPJ_UINT32;
        compno = compno.wrapping_add(1)
      }
      resno = resno.wrapping_add(1)
    }
    layno = layno.wrapping_add(1)
  }
  if loss != 0 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Missing packets possible loss of data\n\x00" as *const u8 as *const libc::c_char,
    );
  }
  opj_free(packet_array as *mut libc::c_void);
  return (loss == 0) as libc::c_int;
}
/* *
 * Gets the number of tile parts used for the given change of progression (if any) and the given tile.
 *
 * @param               cp                      the coding parameters.
 * @param               pino            the offset of the given poc (i.e. its position in the coding parameter).
 * @param               tileno          the given tile.
 *
 * @return              the number of tile parts.
 */
/* ----------------------------------------------------------------------- */
unsafe fn opj_j2k_get_num_tp(
  mut cp: *mut opj_cp_t,
  mut pino: OPJ_UINT32,
  mut tileno: OPJ_UINT32,
) -> OPJ_UINT32 {
  let mut i: OPJ_INT32 = 0;
  let mut tpnum = 1 as OPJ_UINT32;
  let mut tcp = 0 as *mut opj_tcp_t;
  let mut l_current_poc = 0 as *mut opj_poc_t;
  /*  preconditions */

  assert!(tileno < (*cp).tw.wrapping_mul((*cp).th));
  assert!(
    pino
      < (*(*cp).tcps.offset(tileno as isize))
        .numpocs
        .wrapping_add(1u32)
  );
  /* get the given tile coding parameter */
  tcp = &mut *(*cp).tcps.offset(tileno as isize) as *mut opj_tcp_t;
  assert!(!tcp.is_null());
  l_current_poc = &mut *(*tcp).pocs.as_mut_ptr().offset(pino as isize) as *mut opj_poc_t;
  assert!(!l_current_poc.is_null());
  /* get the progression order as a character string */
  let prog = opj_j2k_convert_progression_order((*tcp).prg);
  assert!(prog != ProgressionOrder::Unknown);
  if (*cp).m_specific_param.m_enc.m_tp_on() as libc::c_int == 1i32 {
    for step in prog.get_order() {
      match step {
        ProgressionStep::Component => {
          /* component wise */
          tpnum =
            (tpnum as libc::c_uint).wrapping_mul((*l_current_poc).compE) as OPJ_UINT32
        }
        ProgressionStep::Resolution => {
          /* resolution wise */
          tpnum =
            (tpnum as libc::c_uint).wrapping_mul((*l_current_poc).resE) as OPJ_UINT32
        }
        ProgressionStep::Precinct => {
          /* precinct wise */
          tpnum =
            (tpnum as libc::c_uint).wrapping_mul((*l_current_poc).prcE) as OPJ_UINT32
        }
        ProgressionStep::Layer => {
          /* layer wise */
          tpnum =
            (tpnum as libc::c_uint).wrapping_mul((*l_current_poc).layE) as OPJ_UINT32
        }
        ProgressionStep::Unknown => {}
      }
      /* would we split here ? */
      if (*cp).m_specific_param.m_enc.m_tp_flag == *step as u8 {
        (*cp).m_specific_param.m_enc.m_tp_pos = i;
        break;
      }
    }
  } else {
    tpnum = 1 as OPJ_UINT32
  }
  return tpnum;
}
/* *
 * Calculates the total number of tile parts needed by the encoder to
 * encode such an image. If not enough memory is available, then the function return false.
 *
 * @param       p_nb_tiles      pointer that will hold the number of tile parts.
 * @param       cp                      the coding parameters for the image.
 * @param       image           the image to encode.
 * @param       p_j2k                   the p_j2k encoder.
 * @param       p_manager       the user event manager.
 *
 * @return true if the function was successful, false else.
 */
unsafe fn opj_j2k_calculate_tp(
  mut p_j2k: *mut opj_j2k_t,
  mut cp: *mut opj_cp_t,
  mut p_nb_tiles: *mut OPJ_UINT32,
  mut image: *mut opj_image_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut pino: OPJ_UINT32 = 0;
  let mut tileno: OPJ_UINT32 = 0;
  let mut l_nb_tiles: OPJ_UINT32 = 0;
  let mut tcp = 0 as *mut opj_tcp_t;
  /* preconditions */

  assert!(!p_nb_tiles.is_null());
  assert!(!cp.is_null());
  assert!(!image.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_nb_tiles = (*cp).tw.wrapping_mul((*cp).th);
  *p_nb_tiles = 0 as OPJ_UINT32;
  tcp = (*cp).tcps;
  /* INDEX >> */
  /* TODO mergeV2: check this part which use cstr_info */
  /*if (p_j2k->cstr_info) {
          opj_tile_info_t * l_info_tile_ptr = p_j2k->cstr_info->tile;

          for (tileno = 0; tileno < l_nb_tiles; ++tileno) {
                  OPJ_UINT32 cur_totnum_tp = 0;

                  opj_pi_update_encoding_parameters(image,cp,tileno);

                  for (pino = 0; pino <= tcp->numpocs; ++pino)
                  {
                          OPJ_UINT32 tp_num = opj_j2k_get_num_tp(cp,pino,tileno);

                          *p_nb_tiles = *p_nb_tiles + tp_num;

                          cur_totnum_tp += tp_num;
                  }

                  tcp->m_nb_tile_parts = cur_totnum_tp;

                  l_info_tile_ptr->tp = (opj_tp_info_t *) opj_malloc(cur_totnum_tp * sizeof(opj_tp_info_t));
                  if (l_info_tile_ptr->tp == 00) {
                          return OPJ_FALSE;
                  }

                  memset(l_info_tile_ptr->tp,0,cur_totnum_tp * sizeof(opj_tp_info_t));

                  l_info_tile_ptr->num_tps = cur_totnum_tp;

                  ++l_info_tile_ptr;
                  ++tcp;
          }
  }
  else */
  tileno = 0 as OPJ_UINT32;
  while tileno < l_nb_tiles {
    let mut cur_totnum_tp = 0 as OPJ_UINT32;
    opj_pi_update_encoding_parameters(image, cp, tileno);
    pino = 0 as OPJ_UINT32;
    while pino <= (*tcp).numpocs {
      let mut tp_num = opj_j2k_get_num_tp(cp, pino, tileno);
      *p_nb_tiles = (*p_nb_tiles).wrapping_add(tp_num);
      cur_totnum_tp =
        (cur_totnum_tp as libc::c_uint).wrapping_add(tp_num) as OPJ_UINT32;
      pino = pino.wrapping_add(1)
    }
    (*tcp).m_nb_tile_parts = cur_totnum_tp;
    tcp = tcp.offset(1);
    tileno = tileno.wrapping_add(1)
  }
  return 1i32;
}
/*
 * -----------------------------------------------------------------------
 * -----------------------------------------------------------------------
 * -----------------------------------------------------------------------
 */
/* *
 * Writes the SOC marker (Start Of Codestream)
 *
 * @param       p_stream                        the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager       the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_soc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* 2 bytes will be written */
  let mut l_start_stream = 0 as *mut OPJ_BYTE;
  /* preconditions */

  assert!(!p_stream.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_start_stream = (*p_j2k).m_specific_param.m_encoder.m_header_tile_data;
  /* write SOC identifier */
  opj_write_bytes_LE(
    l_start_stream,
    0xff4f as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  if opj_stream_write_data(
    p_stream,
    l_start_stream,
    2 as OPJ_SIZE_T,
    p_manager,
  ) != 2u64
  {
    return 0i32;
  }
  /* UniPG>> */
  /* USE_JPWL */
  /* <<UniPG */
  return 1i32;
}
/* *
 * Reads a SOC marker (Start of Codestream)
 * @param       p_j2k           the jpeg2000 file codec.
 * @param       p_stream        XXX needs data
 * @param       p_manager       the user event manager.
*/
/* *
 * Reads a SOC marker (Start of Codestream)
 * @param       p_j2k           the jpeg2000 file codec.
 * @param       p_stream        FIXME DOC
 * @param       p_manager       the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_soc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_data: [OPJ_BYTE; 2] = [0; 2];
  let mut l_marker: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  if opj_stream_read_data(
    p_stream,
    l_data.as_mut_ptr(),
    2 as OPJ_SIZE_T,
    p_manager,
  ) != 2u64
  {
    return 0i32;
  }
  opj_read_bytes_LE(
    l_data.as_mut_ptr(),
    &mut l_marker,
    2 as OPJ_UINT32,
  );
  if l_marker != 0xff4fu32 {
    return 0i32;
  }
  /* Next marker should be a SIZ marker in the main header */
  (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::MHSIZ;
  /* FIXME move it in a index structure included in p_j2k*/
  (*(*p_j2k).cstr_index).main_head_start =
    opj_stream_tell(p_stream) - 2i64;
  opj_event_msg(
    p_manager,
    4i32,
    b"Start to read j2k main header (%ld).\n\x00" as *const u8 as *const libc::c_char,
    (*(*p_j2k).cstr_index).main_head_start,
  );
  /* Add the marker to the codestream index*/
  if 0i32
    == opj_j2k_add_mhmarker(
      (*p_j2k).cstr_index,
      0xff4f as OPJ_UINT32,
      (*(*p_j2k).cstr_index).main_head_start,
      2 as OPJ_UINT32,
    )
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to add mh marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  return 1i32;
}
/* *
 * Writes the SIZ marker (image and tile size)
 *
 * @param       p_j2k           J2K codec.
 * @param       p_stream        the stream to write data to.
 * @param       p_manager       the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_siz(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_size_len: OPJ_UINT32 = 0;
  let mut l_current_ptr = 0 as *mut OPJ_BYTE;
  let mut l_image = 0 as *mut opj_image_t;
  let mut cp = 0 as *mut opj_cp_t;
  let mut l_img_comp = 0 as *mut opj_image_comp_t;
  /* preconditions */

  assert!(!p_stream.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_image = (*p_j2k).m_private_image;
  cp = &mut (*p_j2k).m_cp;
  l_size_len = (40u32)
    .wrapping_add((3u32).wrapping_mul((*l_image).numcomps));
  l_img_comp = (*l_image).comps;
  if l_size_len > (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size {
    let mut new_header_tile_data = opj_realloc(
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
      l_size_len as size_t,
    ) as *mut OPJ_BYTE;
    if new_header_tile_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory for the SIZ marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = new_header_tile_data;
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = l_size_len
  }
  l_current_ptr = (*p_j2k).m_specific_param.m_encoder.m_header_tile_data;
  /* write SOC identifier */
  opj_write_bytes_LE(
    l_current_ptr,
    0xff51 as OPJ_UINT32,
    2 as OPJ_UINT32,
  ); /* SIZ */
  l_current_ptr = l_current_ptr.offset(2); /* L_SIZ */
  opj_write_bytes_LE(
    l_current_ptr,
    l_size_len.wrapping_sub(2u32),
    2 as OPJ_UINT32,
  ); /* Rsiz (capabilities) */
  l_current_ptr = l_current_ptr.offset(2); /* Xsiz */
  opj_write_bytes_LE(
    l_current_ptr,
    (*cp).rsiz as OPJ_UINT32,
    2 as OPJ_UINT32,
  ); /* Ysiz */
  l_current_ptr = l_current_ptr.offset(2); /* X0siz */
  opj_write_bytes_LE(l_current_ptr, (*l_image).x1, 4 as OPJ_UINT32); /* Y0siz */
  l_current_ptr = l_current_ptr.offset(4); /* XTsiz */
  opj_write_bytes_LE(l_current_ptr, (*l_image).y1, 4 as OPJ_UINT32); /* YTsiz */
  l_current_ptr = l_current_ptr.offset(4); /* XT0siz */
  opj_write_bytes_LE(l_current_ptr, (*l_image).x0, 4 as OPJ_UINT32); /* YT0siz */
  l_current_ptr = l_current_ptr.offset(4); /* Csiz */
  opj_write_bytes_LE(l_current_ptr, (*l_image).y0, 4 as OPJ_UINT32);
  l_current_ptr = l_current_ptr.offset(4);
  opj_write_bytes_LE(l_current_ptr, (*cp).tdx, 4 as OPJ_UINT32);
  l_current_ptr = l_current_ptr.offset(4);
  opj_write_bytes_LE(l_current_ptr, (*cp).tdy, 4 as OPJ_UINT32);
  l_current_ptr = l_current_ptr.offset(4);
  opj_write_bytes_LE(l_current_ptr, (*cp).tx0, 4 as OPJ_UINT32);
  l_current_ptr = l_current_ptr.offset(4);
  opj_write_bytes_LE(l_current_ptr, (*cp).ty0, 4 as OPJ_UINT32);
  l_current_ptr = l_current_ptr.offset(4);
  opj_write_bytes_LE(
    l_current_ptr,
    (*l_image).numcomps,
    2 as OPJ_UINT32,
  );
  l_current_ptr = l_current_ptr.offset(2);
  i = 0 as OPJ_UINT32;
  while i < (*l_image).numcomps {
    /* TODO here with MCT ? */
    opj_write_bytes_LE(
      l_current_ptr,
      (*l_img_comp)
        .prec
        .wrapping_sub(1u32)
        .wrapping_add((*l_img_comp).sgnd << 7i32),
      1 as OPJ_UINT32,
    ); /* Ssiz_i */
    l_current_ptr = l_current_ptr.offset(1); /* XRsiz_i */
    opj_write_bytes_LE(
      l_current_ptr,
      (*l_img_comp).dx,
      1 as OPJ_UINT32,
    ); /* YRsiz_i */
    l_current_ptr = l_current_ptr.offset(1);
    opj_write_bytes_LE(
      l_current_ptr,
      (*l_img_comp).dy,
      1 as OPJ_UINT32,
    );
    l_current_ptr = l_current_ptr.offset(1);
    l_img_comp = l_img_comp.offset(1);
    i = i.wrapping_add(1)
  }
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_size_len as OPJ_SIZE_T,
    p_manager,
  ) != l_size_len as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Reads a SIZ marker (image and tile size)
 * @param       p_j2k           the jpeg2000 file codec.
 * @param       p_header_data   the data contained in the SIZ box.
 * @param       p_header_size   the size of the data contained in the SIZ marker.
 * @param       p_manager       the user event manager.
*/
/* *
 * Reads a SIZ marker (image and tile size)
 * @param       p_j2k           the jpeg2000 file codec.
 * @param       p_header_data   the data contained in the SIZ box.
 * @param       p_header_size   the size of the data contained in the SIZ marker.
 * @param       p_manager       the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_siz(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_nb_comp: OPJ_UINT32 = 0;
  let mut l_nb_comp_remain: OPJ_UINT32 = 0;
  let mut l_remaining_size: OPJ_UINT32 = 0;
  let mut l_nb_tiles: OPJ_UINT32 = 0;
  let mut l_tmp: OPJ_UINT32 = 0;
  let mut l_tx1: OPJ_UINT32 = 0;
  let mut l_ty1: OPJ_UINT32 = 0;
  let mut l_prec0: OPJ_UINT32 = 0;
  let mut l_sgnd0: OPJ_UINT32 = 0;
  let mut l_image = 0 as *mut opj_image_t;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_img_comp = 0 as *mut opj_image_comp_t;
  let mut l_current_tile_param = 0 as *mut opj_tcp_t;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_header_data.is_null());
  l_image = (*p_j2k).m_private_image;
  l_cp = &mut (*p_j2k).m_cp;
  /* minimum size == 39 - 3 (= minimum component parameter) */
  if p_header_size < 36u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error with SIZ marker size\n\x00" as *const u8 as *const libc::c_char,
    ); /* Rsiz (capabilities) */
    return 0i32;
  } /* Xsiz */
  l_remaining_size = p_header_size.wrapping_sub(36u32); /* Ysiz */
  l_nb_comp = l_remaining_size.wrapping_div(3u32); /* X0siz */
  l_nb_comp_remain = l_remaining_size.wrapping_rem(3u32); /* Y0siz */
  if l_nb_comp_remain != 0u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error with SIZ marker size\n\x00" as *const u8 as *const libc::c_char,
    ); /* XTsiz */
    return 0i32;
  } /* YTsiz */
  opj_read_bytes_LE(p_header_data, &mut l_tmp, 2 as OPJ_UINT32); /* XT0siz */
  p_header_data = p_header_data.offset(2); /* YT0siz */
  (*l_cp).rsiz = l_tmp as OPJ_UINT16; /* Csiz */
  opj_read_bytes_LE(
    p_header_data,
    &mut (*l_image).x1 as *mut OPJ_UINT32,
    4 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(4);
  opj_read_bytes_LE(
    p_header_data,
    &mut (*l_image).y1 as *mut OPJ_UINT32,
    4 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(4);
  opj_read_bytes_LE(
    p_header_data,
    &mut (*l_image).x0 as *mut OPJ_UINT32,
    4 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(4);
  opj_read_bytes_LE(
    p_header_data,
    &mut (*l_image).y0 as *mut OPJ_UINT32,
    4 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(4);
  opj_read_bytes_LE(
    p_header_data,
    &mut (*l_cp).tdx as *mut OPJ_UINT32,
    4 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(4);
  opj_read_bytes_LE(
    p_header_data,
    &mut (*l_cp).tdy as *mut OPJ_UINT32,
    4 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(4);
  opj_read_bytes_LE(
    p_header_data,
    &mut (*l_cp).tx0 as *mut OPJ_UINT32,
    4 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(4);
  opj_read_bytes_LE(
    p_header_data,
    &mut (*l_cp).ty0 as *mut OPJ_UINT32,
    4 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(4);
  opj_read_bytes_LE(
    p_header_data,
    &mut l_tmp as *mut OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(2);
  if l_tmp < 16385u32 {
    (*l_image).numcomps = l_tmp as OPJ_UINT16 as OPJ_UINT32
  } else {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error with SIZ marker: number of component is illegal -> %d\n\x00" as *const u8
        as *const libc::c_char,
      l_tmp,
    );
    return 0i32;
  }
  if (*l_image).numcomps != l_nb_comp {
    opj_event_msg(p_manager, 1i32,
                      b"Error with SIZ marker: number of component is not compatible with the remaining number of parameters ( %d vs %d)\n\x00"
                          as *const u8 as *const libc::c_char,
                      (*l_image).numcomps, l_nb_comp);
    return 0i32;
  }
  /* testcase 4035.pdf.SIGSEGV.d8b.3375 */
  /* testcase issue427-null-image-size.jp2 */
  if (*l_image).x0 >= (*l_image).x1 || (*l_image).y0 >= (*l_image).y1 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error with SIZ marker: negative or zero image size (%ld x %ld)\n\x00" as *const u8
        as *const libc::c_char,
      (*l_image).x1 as OPJ_INT64 - (*l_image).x0 as libc::c_long,
      (*l_image).y1 as OPJ_INT64 - (*l_image).y0 as libc::c_long,
    );
    return 0i32;
  }
  /* testcase 2539.pdf.SIGFPE.706.1712 (also 3622.pdf.SIGFPE.706.2916 and 4008.pdf.SIGFPE.706.3345 and maybe more) */
  if (*l_cp).tdx == 0u32 || (*l_cp).tdy == 0u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error with SIZ marker: invalid tile size (tdx: %d, tdy: %d)\n\x00" as *const u8
        as *const libc::c_char,
      (*l_cp).tdx,
      (*l_cp).tdy,
    );
    return 0i32;
  }
  /* testcase issue427-illegal-tile-offset.jp2 */
  l_tx1 = opj_uint_adds((*l_cp).tx0, (*l_cp).tdx); /* manage overflow */
  l_ty1 = opj_uint_adds((*l_cp).ty0, (*l_cp).tdy); /* manage overflow */
  if (*l_cp).tx0 > (*l_image).x0
    || (*l_cp).ty0 > (*l_image).y0
    || l_tx1 <= (*l_image).x0
    || l_ty1 <= (*l_image).y0
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error with SIZ marker: illegal tile offset\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if (*p_j2k).dump_state == 0 {
    let mut siz_w: OPJ_UINT32 = 0;
    let mut siz_h: OPJ_UINT32 = 0;
    siz_w = (*l_image).x1.wrapping_sub((*l_image).x0);
    siz_h = (*l_image).y1.wrapping_sub((*l_image).y0);
    if (*p_j2k).ihdr_w > 0u32
      && (*p_j2k).ihdr_h > 0u32
      && ((*p_j2k).ihdr_w != siz_w || (*p_j2k).ihdr_h != siz_h)
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Error with SIZ marker: IHDR w(%u) h(%u) vs. SIZ w(%u) h(%u)\n\x00" as *const u8
          as *const libc::c_char,
        (*p_j2k).ihdr_w,
        (*p_j2k).ihdr_h,
        siz_w,
        siz_h,
      );
      return 0i32;
    }
  }
  /* USE_JPWL */
  /* Allocate the resulting image components */
  (*l_image).comps = opj_calloc(
    (*l_image).numcomps as size_t,
    core::mem::size_of::<opj_image_comp_t>() as libc::c_ulong,
  ) as *mut opj_image_comp_t;
  if (*l_image).comps.is_null() {
    (*l_image).numcomps = 0 as OPJ_UINT32;
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to take in charge SIZ marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  l_img_comp = (*l_image).comps;
  l_prec0 = 0 as OPJ_UINT32;
  l_sgnd0 = 0 as OPJ_UINT32;
  /* Read the component information */
  i = 0 as OPJ_UINT32; /* Ssiz_i */
  while i < (*l_image).numcomps {
    let mut tmp: OPJ_UINT32 = 0;
    opj_read_bytes_LE(p_header_data, &mut tmp, 1 as OPJ_UINT32);
    p_header_data = p_header_data.offset(1);
    (*l_img_comp).prec =
      (tmp & 0x7fu32).wrapping_add(1u32);
    (*l_img_comp).sgnd = tmp >> 7i32;
    if (*p_j2k).dump_state == 0u32 {
      if i == 0u32 {
        l_prec0 = (*l_img_comp).prec;
        l_sgnd0 = (*l_img_comp).sgnd
      } else if (*l_cp).allow_different_bit_depth_sign() == 0
        && ((*l_img_comp).prec != l_prec0 || (*l_img_comp).sgnd != l_sgnd0)
      {
        opj_event_msg(p_manager, 2i32,
                              b"Despite JP2 BPC!=255, precision and/or sgnd values for comp[%d] is different than comp[0]:\n        [0] prec(%d) sgnd(%d) [%d] prec(%d) sgnd(%d)\n\x00"
                                  as *const u8 as *const libc::c_char, i,
                              l_prec0, l_sgnd0, i, (*l_img_comp).prec,
                              (*l_img_comp).sgnd);
      }
      /* TODO: we should perhaps also check against JP2 BPCC values */
    } /* XRsiz_i */
    opj_read_bytes_LE(p_header_data, &mut tmp, 1 as OPJ_UINT32); /* should be between 1 and 255 */
    p_header_data = p_header_data.offset(1); /* YRsiz_i */
    (*l_img_comp).dx = tmp; /* should be between 1 and 255 */
    opj_read_bytes_LE(p_header_data, &mut tmp, 1 as OPJ_UINT32);
    p_header_data = p_header_data.offset(1);
    (*l_img_comp).dy = tmp;
    if (*l_img_comp).dx < 1u32
      || (*l_img_comp).dx > 255u32
      || (*l_img_comp).dy < 1u32
      || (*l_img_comp).dy > 255u32
    {
      opj_event_msg(p_manager, 1i32,
                          b"Invalid values for comp = %d : dx=%u dy=%u (should be between 1 and 255 according to the JPEG2000 norm)\n\x00"
                              as *const u8 as *const libc::c_char, i,
                          (*l_img_comp).dx, (*l_img_comp).dy);
      return 0i32;
    }
    /* Avoids later undefined shift in computation of */
    /* p_j2k->m_specific_param.m_decoder.m_default_tcp->tccps[i].m_dc_level_shift = 1
    << (l_image->comps[i].prec - 1); */
    if (*l_img_comp).prec > 31u32 {
      opj_event_msg(p_manager, 1i32,
                          b"Invalid values for comp = %d : prec=%u (should be between 1 and 38 according to the JPEG2000 norm. OpenJpeg only supports up to 31)\n\x00"
                              as *const u8 as *const libc::c_char, i,
                          (*l_img_comp).prec);
      return 0i32;
    }
    /* USE_JPWL */
    (*l_img_comp).resno_decoded = 0 as OPJ_UINT32; /* number of resolution decoded */
    (*l_img_comp).factor = (*l_cp).m_specific_param.m_dec.m_reduce; /* reducing factor per component */
    l_img_comp = l_img_comp.offset(1);
    i = i.wrapping_add(1)
  }
  if (*l_cp).tdx == 0u32
    || (*l_cp).tdy == 0u32
  {
    return 0i32;
  }
  /* Compute the number of tiles */
  (*l_cp).tw = opj_int_ceildiv(
    (*l_image).x1.wrapping_sub((*l_cp).tx0) as OPJ_INT32,
    (*l_cp).tdx as OPJ_INT32,
  ) as OPJ_UINT32;
  (*l_cp).th = opj_int_ceildiv(
    (*l_image).y1.wrapping_sub((*l_cp).ty0) as OPJ_INT32,
    (*l_cp).tdy as OPJ_INT32,
  ) as OPJ_UINT32;
  /* Check that the number of tiles is valid */
  if (*l_cp).tw == 0u32
    || (*l_cp).th == 0u32
    || (*l_cp).tw > (65535u32).wrapping_div((*l_cp).th)
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Invalid number of tiles : %u x %u (maximum fixed by jpeg2000 norm is 65535 tiles)\n\x00"
        as *const u8 as *const libc::c_char,
      (*l_cp).tw,
      (*l_cp).th,
    );
    return 0i32;
  }
  l_nb_tiles = (*l_cp).tw.wrapping_mul((*l_cp).th);
  /* Define the tiles which will be decoded */
  if (*p_j2k).m_specific_param.m_decoder.m_discard_tiles() != 0 {
    (*p_j2k).m_specific_param.m_decoder.m_start_tile_x = (*p_j2k)
      .m_specific_param
      .m_decoder
      .m_start_tile_x
      .wrapping_sub((*l_cp).tx0)
      .wrapping_div((*l_cp).tdx);
    (*p_j2k).m_specific_param.m_decoder.m_start_tile_y = (*p_j2k)
      .m_specific_param
      .m_decoder
      .m_start_tile_y
      .wrapping_sub((*l_cp).ty0)
      .wrapping_div((*l_cp).tdy);
    (*p_j2k).m_specific_param.m_decoder.m_end_tile_x = opj_int_ceildiv(
      (*p_j2k)
        .m_specific_param
        .m_decoder
        .m_end_tile_x
        .wrapping_sub((*l_cp).tx0) as OPJ_INT32,
      (*l_cp).tdx as OPJ_INT32,
    ) as OPJ_UINT32;
    (*p_j2k).m_specific_param.m_decoder.m_end_tile_y = opj_int_ceildiv(
      (*p_j2k)
        .m_specific_param
        .m_decoder
        .m_end_tile_y
        .wrapping_sub((*l_cp).ty0) as OPJ_INT32,
      (*l_cp).tdy as OPJ_INT32,
    ) as OPJ_UINT32
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_start_tile_x = 0 as OPJ_UINT32;
    (*p_j2k).m_specific_param.m_decoder.m_start_tile_y = 0 as OPJ_UINT32;
    (*p_j2k).m_specific_param.m_decoder.m_end_tile_x = (*l_cp).tw;
    (*p_j2k).m_specific_param.m_decoder.m_end_tile_y = (*l_cp).th
  }
  /* USE_JPWL */
  /* memory allocations */
  (*l_cp).tcps = opj_calloc(
    l_nb_tiles as size_t,
    core::mem::size_of::<opj_tcp_t>() as libc::c_ulong,
  ) as *mut opj_tcp_t;
  if (*l_cp).tcps.is_null() {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to take in charge SIZ marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* USE_JPWL */
  (*(*p_j2k).m_specific_param.m_decoder.m_default_tcp).tccps = opj_calloc(
    (*l_image).numcomps as size_t,
    core::mem::size_of::<opj_tccp_t>() as libc::c_ulong,
  ) as *mut opj_tccp_t;
  if (*(*p_j2k).m_specific_param.m_decoder.m_default_tcp)
    .tccps
    .is_null()
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to take in charge SIZ marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  (*(*p_j2k).m_specific_param.m_decoder.m_default_tcp).m_mct_records = opj_calloc(
    10i32 as size_t,
    core::mem::size_of::<opj_mct_data_t>() as libc::c_ulong,
  ) as *mut opj_mct_data_t;
  if (*(*p_j2k).m_specific_param.m_decoder.m_default_tcp)
    .m_mct_records
    .is_null()
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to take in charge SIZ marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  (*(*p_j2k).m_specific_param.m_decoder.m_default_tcp).m_nb_max_mct_records =
    10 as OPJ_UINT32;
  (*(*p_j2k).m_specific_param.m_decoder.m_default_tcp).m_mcc_records = opj_calloc(
    10i32 as size_t,
    core::mem::size_of::<opj_simple_mcc_decorrelation_data_t>() as libc::c_ulong,
  )
    as *mut opj_simple_mcc_decorrelation_data_t;
  if (*(*p_j2k).m_specific_param.m_decoder.m_default_tcp)
    .m_mcc_records
    .is_null()
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to take in charge SIZ marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  (*(*p_j2k).m_specific_param.m_decoder.m_default_tcp).m_nb_max_mcc_records =
    10 as OPJ_UINT32;
  /* set up default dc level shift */
  i = 0 as OPJ_UINT32;
  while i < (*l_image).numcomps {
    if (*(*l_image).comps.offset(i as isize)).sgnd == 0 {
      (*(*(*p_j2k).m_specific_param.m_decoder.m_default_tcp)
        .tccps
        .offset(i as isize))
      .m_dc_level_shift = (1i32)
        << (*(*l_image).comps.offset(i as isize))
          .prec
          .wrapping_sub(1u32)
    }
    i = i.wrapping_add(1)
  }
  l_current_tile_param = (*l_cp).tcps;
  i = 0 as OPJ_UINT32;
  while i < l_nb_tiles {
    (*l_current_tile_param).tccps = opj_calloc(
      (*l_image).numcomps as size_t,
      core::mem::size_of::<opj_tccp_t>() as libc::c_ulong,
    ) as *mut opj_tccp_t;
    if (*l_current_tile_param).tccps.is_null() {
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to take in charge SIZ marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    l_current_tile_param = l_current_tile_param.offset(1);
    i = i.wrapping_add(1)
  }
  (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::MH;
  opj_image_comp_header_update(l_image, l_cp);
  return 1i32;
}
/* *
 * Writes the COM marker (comment)
 *
 * @param       p_stream                        the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager       the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_com(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_comment_size: OPJ_UINT32 = 0;
  let mut l_total_com_size: OPJ_UINT32 = 0;
  let mut l_comment = 0 as *const OPJ_CHAR;
  let mut l_current_ptr = 0 as *mut OPJ_BYTE;
  /* preconditions */
  /* L_COM */

  assert!(!p_j2k.is_null());
  assert!(!p_stream.is_null());
  assert!(!p_manager.is_null());
  l_comment = (*p_j2k).m_cp.comment;
  l_comment_size = strlen(l_comment) as OPJ_UINT32;
  l_total_com_size = l_comment_size.wrapping_add(6u32);
  if l_total_com_size > (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size {
    let mut new_header_tile_data = opj_realloc(
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
      l_total_com_size as size_t,
    ) as *mut OPJ_BYTE;
    if new_header_tile_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to write the COM marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = new_header_tile_data;
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = l_total_com_size
  }
  l_current_ptr = (*p_j2k).m_specific_param.m_encoder.m_header_tile_data;
  opj_write_bytes_LE(
    l_current_ptr,
    0xff64 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_ptr = l_current_ptr.offset(2);
  opj_write_bytes_LE(
    l_current_ptr,
    l_total_com_size.wrapping_sub(2u32),
    2 as OPJ_UINT32,
  );
  l_current_ptr = l_current_ptr.offset(2);
  opj_write_bytes_LE(
    l_current_ptr,
    1 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_ptr = l_current_ptr.offset(2);
  memcpy(
    l_current_ptr as *mut libc::c_void,
    l_comment as *const libc::c_void,
    l_comment_size as libc::c_ulong,
  );
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_total_com_size as OPJ_SIZE_T,
    p_manager,
  ) != l_total_com_size as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Reads a COM marker (comments)
 * @param       p_j2k           the jpeg2000 file codec.
 * @param       p_header_data   the data contained in the COM box.
 * @param       p_header_size   the size of the data contained in the COM marker.
 * @param       p_manager       the user event manager.
*/
/* *
 * Reads a COM marker (comments)
 * @param       p_j2k           the jpeg2000 file codec.
 * @param       p_header_data   the data contained in the COM box.
 * @param       p_header_size   the size of the data contained in the COM marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_com(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut _p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_header_data.is_null());
  return 1i32;
}
/* *
 * Writes the COD marker (Coding style default)
 *
 * @param       p_stream                        the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager       the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_cod(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_code_size: OPJ_UINT32 = 0;
  let mut l_remaining_size: OPJ_UINT32 = 0;
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  /* preconditions */
  /* L_COD */
  /* SGcod (A) */
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null()); /* SGcod (C) */
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t;
  l_code_size = (9u32).wrapping_add(opj_j2k_get_SPCod_SPCoc_size(
    p_j2k,
    (*p_j2k).m_current_tile_number,
    0 as OPJ_UINT32,
  ));
  l_remaining_size = l_code_size;
  if l_code_size > (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size {
    let mut new_header_tile_data = opj_realloc(
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
      l_code_size as size_t,
    ) as *mut OPJ_BYTE;
    if new_header_tile_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to write COD marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = new_header_tile_data;
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = l_code_size
  }
  l_current_data = (*p_j2k).m_specific_param.m_encoder.m_header_tile_data;
  opj_write_bytes_LE(
    l_current_data,
    0xff52 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    l_code_size.wrapping_sub(2u32),
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    (*l_tcp).csty,
    1 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(1);
  opj_write_bytes_LE(
    l_current_data,
    (*l_tcp).prg as OPJ_UINT32,
    1 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(1);
  opj_write_bytes_LE(
    l_current_data,
    (*l_tcp).numlayers,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(l_current_data, (*l_tcp).mct, 1 as OPJ_UINT32);
  l_current_data = l_current_data.offset(1);
  l_remaining_size = (l_remaining_size as libc::c_uint)
    .wrapping_sub(9u32) as OPJ_UINT32
    as OPJ_UINT32;
  if opj_j2k_write_SPCod_SPCoc(
    p_j2k,
    (*p_j2k).m_current_tile_number,
    0 as OPJ_UINT32,
    l_current_data,
    &mut l_remaining_size,
    p_manager,
  ) == 0
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error writing COD marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if l_remaining_size != 0u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error writing COD marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_code_size as OPJ_SIZE_T,
    p_manager,
  ) != l_code_size as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Reads a COD marker (Coding style defaults)
 * @param       p_header_data   the data contained in the COD box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the COD marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a COD marker (Coding style defaults)
 * @param       p_header_data   the data contained in the COD box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the COD marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_cod(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* loop */
  let mut i: OPJ_UINT32 = 0;
  let mut l_tmp: OPJ_UINT32 = 0;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_image = 0 as *mut opj_image_t;
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_image = (*p_j2k).m_private_image;
  l_cp = &mut (*p_j2k).m_cp;
  /* If we are in the first tile-part header of the current tile */
  l_tcp = if (*p_j2k).m_specific_param.m_decoder.m_state
    == J2KState::TPH
  {
    &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_default_tcp
  };
  (*l_tcp).set_cod(1 as OPJ_BITFIELD);
  /* Make sure room is sufficient */
  if p_header_size < 5u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading COD marker\n\x00" as *const u8 as *const libc::c_char,
    ); /* Scod */
    return 0i32;
  }
  opj_read_bytes_LE(
    p_header_data,
    &mut (*l_tcp).csty,
    1 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(1);
  /* Make sure we know how to decode this */
  if (*l_tcp).csty & !((0x1i32 | 0x2i32 | 0x4i32) as OPJ_UINT32)
    != 0u32
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Unknown Scod value in COD marker\n\x00" as *const u8 as *const libc::c_char,
    ); /* SGcod (A) */
    return 0i32;
  }
  opj_read_bytes_LE(p_header_data, &mut l_tmp, 1 as OPJ_UINT32);
  p_header_data = p_header_data.offset(1);
  (*l_tcp).prg = l_tmp as OPJ_PROG_ORDER;
  /* Make sure progression order is valid */
  if (*l_tcp).prg as libc::c_int > OPJ_CPRL as libc::c_int {
    opj_event_msg(
      p_manager,
      1i32,
      b"Unknown progression order in COD marker\n\x00" as *const u8 as *const libc::c_char,
    ); /* SGcod (B) */
    (*l_tcp).prg = OPJ_PROG_UNKNOWN
  }
  opj_read_bytes_LE(
    p_header_data,
    &mut (*l_tcp).numlayers,
    2 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(2);
  if (*l_tcp).numlayers < 1u32 || (*l_tcp).numlayers > 65535u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Invalid number of layers in COD marker : %d not in range [1-65535]\n\x00" as *const u8
        as *const libc::c_char,
      (*l_tcp).numlayers,
    );
    return 0i32;
  }
  /* If user didn't set a number layer to decode take the max specify in the codestream. */
  if (*l_cp).m_specific_param.m_dec.m_layer != 0 {
    (*l_tcp).num_layers_to_decode = (*l_cp).m_specific_param.m_dec.m_layer
  } else {
    (*l_tcp).num_layers_to_decode = (*l_tcp).numlayers
  } /* SGcod (C) */
  opj_read_bytes_LE(
    p_header_data,
    &mut (*l_tcp).mct,
    1 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(1);
  if (*l_tcp).mct > 1u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Invalid multiple component transformation\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  p_header_size = (p_header_size as libc::c_uint).wrapping_sub(5u32)
    as OPJ_UINT32;
  i = 0 as OPJ_UINT32;
  while i < (*l_image).numcomps {
    (*(*l_tcp).tccps.offset(i as isize)).csty = (*l_tcp).csty & 0x1u32;
    i = i.wrapping_add(1)
  }
  if opj_j2k_read_SPCod_SPCoc(
    p_j2k,
    0 as OPJ_UINT32,
    p_header_data,
    &mut p_header_size,
    p_manager,
  ) == 0
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading COD marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if p_header_size != 0u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading COD marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* Apply the coding style to other components of the current tile or the m_default_tcp*/
  opj_j2k_copy_tile_component_parameters(p_j2k);
  /* Index */
  return 1i32;
}
/* *
 * Writes the COC marker (Coding style component)
 *
 * @param       p_j2k       J2K codec.
 * @param       p_comp_no   the index of the component to output.
 * @param       p_stream    the stream to write data to.
 * @param       p_manager   the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_coc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_comp_no: OPJ_UINT32,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_coc_size: OPJ_UINT32 = 0;
  let mut l_remaining_size: OPJ_UINT32 = 0;
  let mut l_comp_room: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  l_comp_room = if (*(*p_j2k).m_private_image).numcomps <= 256u32 {
    1i32
  } else {
    2i32
  } as OPJ_UINT32;
  l_coc_size = (5u32)
    .wrapping_add(l_comp_room)
    .wrapping_add(opj_j2k_get_SPCod_SPCoc_size(
      p_j2k,
      (*p_j2k).m_current_tile_number,
      p_comp_no,
    ));
  if l_coc_size > (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size {
    let mut new_header_tile_data = 0 as *mut OPJ_BYTE;
    /*p_j2k->m_specific_param.m_encoder.m_header_tile_data
    = (OPJ_BYTE*)opj_realloc(
            p_j2k->m_specific_param.m_encoder.m_header_tile_data,
            l_coc_size);*/
    new_header_tile_data = opj_realloc(
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
      l_coc_size as size_t,
    ) as *mut OPJ_BYTE;
    if new_header_tile_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to write COC marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = new_header_tile_data;
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = l_coc_size
  }
  opj_j2k_write_coc_in_memory(
    p_j2k,
    p_comp_no,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    &mut l_remaining_size,
    p_manager,
  );
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_coc_size as OPJ_SIZE_T,
    p_manager,
  ) != l_coc_size as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Compares 2 COC markers (Coding style component)
 *
 * @param       p_j2k            J2K codec.
 * @param       p_first_comp_no  the index of the first component to compare.
 * @param       p_second_comp_no the index of the second component to compare.
 *
 * @return      OPJ_TRUE if equals
 */
unsafe fn opj_j2k_compare_coc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_first_comp_no: OPJ_UINT32,
  mut p_second_comp_no: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  /* preconditions */
  assert!(!p_j2k.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t;
  if (*(*l_tcp).tccps.offset(p_first_comp_no as isize)).csty
    != (*(*l_tcp).tccps.offset(p_second_comp_no as isize)).csty
  {
    return 0i32;
  }
  return opj_j2k_compare_SPCod_SPCoc(
    p_j2k,
    (*p_j2k).m_current_tile_number,
    p_first_comp_no,
    p_second_comp_no,
  );
}
/* *
 * Writes the COC marker (Coding style component)
 *
 * @param       p_j2k                   J2K codec.
 * @param       p_comp_no               the index of the component to output.
 * @param       p_data          FIXME DOC
 * @param       p_data_written  FIXME DOC
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_coc_in_memory(
  mut p_j2k: *mut opj_j2k_t,
  mut p_comp_no: OPJ_UINT32,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_written: *mut OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) {
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_coc_size: OPJ_UINT32 = 0;
  let mut l_remaining_size: OPJ_UINT32 = 0;
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  let mut l_image = 0 as *mut opj_image_t;
  let mut l_comp_room: OPJ_UINT32 = 0;
  /* preconditions */
  /* L_COC */
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null()); /* Scoc */
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t;
  l_image = (*p_j2k).m_private_image;
  l_comp_room = if (*l_image).numcomps <= 256u32 {
    1i32
  } else {
    2i32
  } as OPJ_UINT32;
  l_coc_size = (5u32)
    .wrapping_add(l_comp_room)
    .wrapping_add(opj_j2k_get_SPCod_SPCoc_size(
      p_j2k,
      (*p_j2k).m_current_tile_number,
      p_comp_no,
    ));
  l_remaining_size = l_coc_size;
  l_current_data = p_data;
  opj_write_bytes_LE(
    l_current_data,
    0xff53 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    l_coc_size.wrapping_sub(2u32),
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(l_current_data, p_comp_no, l_comp_room);
  l_current_data = l_current_data.offset(l_comp_room as isize);
  opj_write_bytes_LE(
    l_current_data,
    (*(*l_tcp).tccps.offset(p_comp_no as isize)).csty,
    1 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(1);
  l_remaining_size = (l_remaining_size as libc::c_uint)
    .wrapping_sub((5u32).wrapping_add(l_comp_room))
    as OPJ_UINT32;
  opj_j2k_write_SPCod_SPCoc(
    p_j2k,
    (*p_j2k).m_current_tile_number,
    0 as OPJ_UINT32,
    l_current_data,
    &mut l_remaining_size,
    p_manager,
  );
  *p_data_written = l_coc_size;
}
/* *
 * Gets the maximum size taken by a coc.
 *
 * @param       p_j2k   the jpeg2000 codec to use.
 */
unsafe fn opj_j2k_get_max_coc_size(mut p_j2k: *mut opj_j2k_t) -> OPJ_UINT32 {
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut l_nb_comp: OPJ_UINT32 = 0;
  let mut l_nb_tiles: OPJ_UINT32 = 0;
  let mut l_max = 0 as OPJ_UINT32;
  /* preconditions */
  l_nb_tiles = (*p_j2k).m_cp.tw.wrapping_mul((*p_j2k).m_cp.th);
  l_nb_comp = (*(*p_j2k).m_private_image).numcomps;
  i = 0 as OPJ_UINT32;
  while i < l_nb_tiles {
    j = 0 as OPJ_UINT32;
    while j < l_nb_comp {
      l_max = opj_uint_max(l_max, opj_j2k_get_SPCod_SPCoc_size(p_j2k, i, j));
      j = j.wrapping_add(1)
    }
    i = i.wrapping_add(1)
  }
  return (6u32).wrapping_add(l_max);
}
/* *
 * Reads a COC marker (Coding Style Component)
 * @param       p_header_data   the data contained in the COC box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the COC marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a COC marker (Coding Style Component)
 * @param       p_header_data   the data contained in the COC box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the COC marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_coc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_image = 0 as *mut opj_image_t;
  let mut l_comp_room: OPJ_UINT32 = 0;
  let mut l_comp_no: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = if (*p_j2k).m_specific_param.m_decoder.m_state
    == J2KState::TPH
  {
    &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_default_tcp
  };
  l_image = (*p_j2k).m_private_image;
  l_comp_room = if (*l_image).numcomps <= 256u32 {
    1i32
  } else {
    2i32
  } as OPJ_UINT32;
  /* make sure room is sufficient*/
  if p_header_size < l_comp_room.wrapping_add(1u32) {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading COC marker\n\x00" as *const u8 as *const libc::c_char,
    ); /* Ccoc */
    return 0i32;
  } /* Scoc */
  p_header_size = (p_header_size as libc::c_uint)
    .wrapping_sub(l_comp_room.wrapping_add(1u32))
    as OPJ_UINT32;
  opj_read_bytes_LE(p_header_data, &mut l_comp_no, l_comp_room);
  p_header_data = p_header_data.offset(l_comp_room as isize);
  if l_comp_no >= (*l_image).numcomps {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading COC marker (bad number of components)\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 0i32;
  }
  opj_read_bytes_LE(
    p_header_data,
    &mut (*(*l_tcp).tccps.offset(l_comp_no as isize)).csty,
    1 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(1);
  if opj_j2k_read_SPCod_SPCoc(
    p_j2k,
    l_comp_no,
    p_header_data,
    &mut p_header_size,
    p_manager,
  ) == 0
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading COC marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if p_header_size != 0u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading COC marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  return 1i32;
}
/* *
 * Writes the QCD marker (quantization default)
 *
 * @param       p_j2k                   J2K codec.
 * @param       p_stream                the stream to write data to.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_qcd(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_qcd_size: OPJ_UINT32 = 0;
  let mut l_remaining_size: OPJ_UINT32 = 0;
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  /* preconditions */
  /* L_QCD */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  l_qcd_size = (4u32).wrapping_add(opj_j2k_get_SQcd_SQcc_size(
    p_j2k,
    (*p_j2k).m_current_tile_number,
    0 as OPJ_UINT32,
  ));
  l_remaining_size = l_qcd_size;
  if l_qcd_size > (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size {
    let mut new_header_tile_data = opj_realloc(
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
      l_qcd_size as size_t,
    ) as *mut OPJ_BYTE;
    if new_header_tile_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to write QCD marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = new_header_tile_data;
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = l_qcd_size
  }
  l_current_data = (*p_j2k).m_specific_param.m_encoder.m_header_tile_data;
  opj_write_bytes_LE(
    l_current_data,
    0xff5c as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    l_qcd_size.wrapping_sub(2u32),
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  l_remaining_size = (l_remaining_size as libc::c_uint)
    .wrapping_sub(4u32) as OPJ_UINT32
    as OPJ_UINT32;
  if opj_j2k_write_SQcd_SQcc(
    p_j2k,
    (*p_j2k).m_current_tile_number,
    0 as OPJ_UINT32,
    l_current_data,
    &mut l_remaining_size,
    p_manager,
  ) == 0
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error writing QCD marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if l_remaining_size != 0u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error writing QCD marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_qcd_size as OPJ_SIZE_T,
    p_manager,
  ) != l_qcd_size as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Reads a QCD marker (Quantization defaults)
 * @param       p_header_data   the data contained in the QCD box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the QCD marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a QCD marker (Quantization defaults)
 * @param       p_header_data   the data contained in the QCD box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the QCD marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_qcd(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  if opj_j2k_read_SQcd_SQcc(
    p_j2k,
    0 as OPJ_UINT32,
    p_header_data,
    &mut p_header_size,
    p_manager,
  ) == 0
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading QCD marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if p_header_size != 0u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading QCD marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* Apply the quantization parameters to other components of the current tile or the m_default_tcp */
  opj_j2k_copy_tile_quantization_parameters(p_j2k);
  return 1i32;
}
/* *
 * Writes the QCC marker (quantization component)
 *
 * @param       p_comp_no       the index of the component to output.
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_qcc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_comp_no: OPJ_UINT32,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_qcc_size: OPJ_UINT32 = 0;
  let mut l_remaining_size: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  l_qcc_size = (5u32).wrapping_add(opj_j2k_get_SQcd_SQcc_size(
    p_j2k,
    (*p_j2k).m_current_tile_number,
    p_comp_no,
  ));
  l_qcc_size = (l_qcc_size as libc::c_uint).wrapping_add(if (*(*p_j2k).m_private_image).numcomps
    <= 256u32
  {
    0i32
  } else {
    1i32
  } as libc::c_uint) as OPJ_UINT32;
  l_remaining_size = l_qcc_size;
  if l_qcc_size > (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size {
    let mut new_header_tile_data = opj_realloc(
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
      l_qcc_size as size_t,
    ) as *mut OPJ_BYTE;
    if new_header_tile_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to write QCC marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = new_header_tile_data;
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = l_qcc_size
  }
  opj_j2k_write_qcc_in_memory(
    p_j2k,
    p_comp_no,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    &mut l_remaining_size,
    p_manager,
  );
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_qcc_size as OPJ_SIZE_T,
    p_manager,
  ) != l_qcc_size as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Compare QCC markers (quantization component)
 *
 * @param       p_j2k                 J2K codec.
 * @param       p_first_comp_no       the index of the first component to compare.
 * @param       p_second_comp_no      the index of the second component to compare.
 *
 * @return OPJ_TRUE if equals.
 */
unsafe fn opj_j2k_compare_qcc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_first_comp_no: OPJ_UINT32,
  mut p_second_comp_no: OPJ_UINT32,
) -> OPJ_BOOL {
  return opj_j2k_compare_SQcd_SQcc(
    p_j2k,
    (*p_j2k).m_current_tile_number,
    p_first_comp_no,
    p_second_comp_no,
  );
}
/* *
 * Writes the QCC marker (quantization component)
 *
 * @param       p_j2k           J2K codec.
 * @param       p_comp_no       the index of the component to output.
 * @param       p_data          FIXME DOC
 * @param       p_data_written  the stream to write data to.
 * @param       p_manager       the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_qcc_in_memory(
  mut p_j2k: *mut opj_j2k_t,
  mut p_comp_no: OPJ_UINT32,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_written: *mut OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) {
  let mut l_qcc_size: OPJ_UINT32 = 0;
  let mut l_remaining_size: OPJ_UINT32 = 0;
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  /* preconditions */
  /* L_QCC */
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_qcc_size = (6u32).wrapping_add(opj_j2k_get_SQcd_SQcc_size(
    p_j2k,
    (*p_j2k).m_current_tile_number,
    p_comp_no,
  ));
  l_remaining_size = l_qcc_size;
  l_current_data = p_data;
  opj_write_bytes_LE(
    l_current_data,
    0xff5d as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  if (*(*p_j2k).m_private_image).numcomps <= 256u32 {
    l_qcc_size = l_qcc_size.wrapping_sub(1);
    opj_write_bytes_LE(
      l_current_data,
      l_qcc_size.wrapping_sub(2u32),
      2 as OPJ_UINT32,
    );
    l_current_data = l_current_data.offset(2);
    opj_write_bytes_LE(l_current_data, p_comp_no, 1 as OPJ_UINT32);
    l_current_data = l_current_data.offset(1);
    /* in the case only one byte is sufficient the last byte allocated is useless -> still do -6 for available */
    l_remaining_size = (l_remaining_size as libc::c_uint)
      .wrapping_sub(6u32) as OPJ_UINT32
      as OPJ_UINT32
  } else {
    opj_write_bytes_LE(
      l_current_data,
      l_qcc_size.wrapping_sub(2u32),
      2 as OPJ_UINT32,
    ); /* L_QCC */
    l_current_data = l_current_data.offset(2); /* Cqcc */
    opj_write_bytes_LE(l_current_data, p_comp_no, 2 as OPJ_UINT32);
    l_current_data = l_current_data.offset(2);
    l_remaining_size = (l_remaining_size as libc::c_uint)
      .wrapping_sub(6u32) as OPJ_UINT32
      as OPJ_UINT32
  }
  opj_j2k_write_SQcd_SQcc(
    p_j2k,
    (*p_j2k).m_current_tile_number,
    p_comp_no,
    l_current_data,
    &mut l_remaining_size,
    p_manager,
  );
  *p_data_written = l_qcc_size;
}
/* *
 * Gets the maximum size taken by a qcc.
 */
unsafe fn opj_j2k_get_max_qcc_size(mut p_j2k: *mut opj_j2k_t) -> OPJ_UINT32 {
  return opj_j2k_get_max_coc_size(p_j2k);
}
/* *
 * Reads a QCC marker (Quantization component)
 * @param       p_header_data   the data contained in the QCC box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the QCC marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a QCC marker (Quantization component)
 * @param       p_header_data   the data contained in the QCC box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the QCC marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_qcc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_num_comp: OPJ_UINT32 = 0;
  let mut l_comp_no: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_num_comp = (*(*p_j2k).m_private_image).numcomps;
  if l_num_comp <= 256u32 {
    if p_header_size < 1u32 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Error reading QCC marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    opj_read_bytes_LE(
      p_header_data,
      &mut l_comp_no,
      1 as OPJ_UINT32,
    );
    p_header_data = p_header_data.offset(1);
    p_header_size = p_header_size.wrapping_sub(1)
  } else {
    if p_header_size < 2u32 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Error reading QCC marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    opj_read_bytes_LE(
      p_header_data,
      &mut l_comp_no,
      2 as OPJ_UINT32,
    );
    p_header_data = p_header_data.offset(2);
    p_header_size = (p_header_size as libc::c_uint).wrapping_sub(2u32)
      as OPJ_UINT32
  }
  /* USE_JPWL */
  if l_comp_no >= (*(*p_j2k).m_private_image).numcomps {
    opj_event_msg(
      p_manager,
      1i32,
      b"Invalid component number: %d, regarding the number of components %d\n\x00" as *const u8
        as *const libc::c_char,
      l_comp_no,
      (*(*p_j2k).m_private_image).numcomps,
    );
    return 0i32;
  }
  if opj_j2k_read_SQcd_SQcc(
    p_j2k,
    l_comp_no,
    p_header_data,
    &mut p_header_size,
    p_manager,
  ) == 0
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading QCC marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if p_header_size != 0u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading QCC marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  return 1i32;
}
/* *
 * Writes the POC marker (Progression Order Change)
 *
 * @param       p_stream                                the stream to write data to.
 * @param       p_j2k                           J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_poc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_nb_comp: OPJ_UINT32 = 0;
  let mut l_nb_poc: OPJ_UINT32 = 0;
  let mut l_poc_size: OPJ_UINT32 = 0;
  let mut l_written_size = 0 as OPJ_UINT32;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_poc_room: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  l_tcp = &mut *(*p_j2k)
    .m_cp
    .tcps
    .offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t;
  l_nb_comp = (*(*p_j2k).m_private_image).numcomps;
  l_nb_poc = (1u32).wrapping_add((*l_tcp).numpocs);
  if l_nb_comp <= 256u32 {
    l_poc_room = 1 as OPJ_UINT32
  } else {
    l_poc_room = 2 as OPJ_UINT32
  }
  l_poc_size = (4u32).wrapping_add(
    (5u32)
      .wrapping_add((2u32).wrapping_mul(l_poc_room))
      .wrapping_mul(l_nb_poc),
  );
  if l_poc_size > (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size {
    let mut new_header_tile_data = opj_realloc(
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
      l_poc_size as size_t,
    ) as *mut OPJ_BYTE;
    if new_header_tile_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to write POC marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = new_header_tile_data;
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = l_poc_size
  }
  opj_j2k_write_poc_in_memory(
    p_j2k,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    &mut l_written_size,
    p_manager,
  );
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_poc_size as OPJ_SIZE_T,
    p_manager,
  ) != l_poc_size as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Writes the POC marker (Progression Order Change)
 *
 * @param       p_j2k          J2K codec.
 * @param       p_data         FIXME DOC
 * @param       p_data_written the stream to write data to.
 * @param       p_manager      the user event manager.
 */
unsafe extern "C" fn opj_j2k_write_poc_in_memory(
  mut p_j2k: *mut opj_j2k_t,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_written: *mut OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) {
  let mut i: OPJ_UINT32 = 0;
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  let mut l_nb_comp: OPJ_UINT32 = 0;
  let mut l_nb_poc: OPJ_UINT32 = 0;
  let mut l_poc_size: OPJ_UINT32 = 0;
  let mut l_image = 0 as *mut opj_image_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  let mut l_current_poc = 0 as *mut opj_poc_t;
  let mut l_poc_room: OPJ_UINT32 = 0;
  /* preconditions */
  /* Lpoc */
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null()); /* CSpoc_i */
  l_tcp = &mut *(*p_j2k)
    .m_cp
    .tcps
    .offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t; /* LYEpoc_i */
  l_tccp = &mut *(*l_tcp).tccps.offset(0) as *mut opj_tccp_t; /* REpoc_i */
  l_image = (*p_j2k).m_private_image; /* CEpoc_i */
  l_nb_comp = (*l_image).numcomps; /* Ppoc_i */
  l_nb_poc = (1u32).wrapping_add((*l_tcp).numpocs);
  if l_nb_comp <= 256u32 {
    l_poc_room = 1 as OPJ_UINT32
  } else {
    l_poc_room = 2 as OPJ_UINT32
  }
  l_poc_size = (4u32).wrapping_add(
    (5u32)
      .wrapping_add((2u32).wrapping_mul(l_poc_room))
      .wrapping_mul(l_nb_poc),
  );
  l_current_data = p_data;
  opj_write_bytes_LE(
    l_current_data,
    0xff5f as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    l_poc_size.wrapping_sub(2u32),
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  l_current_poc = (*l_tcp).pocs.as_mut_ptr();
  i = 0 as OPJ_UINT32;
  while i < l_nb_poc {
    opj_write_bytes_LE(
      l_current_data,
      (*l_current_poc).resno0,
      1 as OPJ_UINT32,
    );
    l_current_data = l_current_data.offset(1);
    opj_write_bytes_LE(l_current_data, (*l_current_poc).compno0, l_poc_room);
    l_current_data = l_current_data.offset(l_poc_room as isize);
    opj_write_bytes_LE(
      l_current_data,
      (*l_current_poc).layno1,
      2 as OPJ_UINT32,
    );
    l_current_data = l_current_data.offset(2);
    opj_write_bytes_LE(
      l_current_data,
      (*l_current_poc).resno1,
      1 as OPJ_UINT32,
    );
    l_current_data = l_current_data.offset(1);
    opj_write_bytes_LE(l_current_data, (*l_current_poc).compno1, l_poc_room);
    l_current_data = l_current_data.offset(l_poc_room as isize);
    opj_write_bytes_LE(
      l_current_data,
      (*l_current_poc).prg as OPJ_UINT32,
      1 as OPJ_UINT32,
    );
    l_current_data = l_current_data.offset(1);
    /* change the value of the max layer according to the actual number of layers in the file, components and resolutions*/
    (*l_current_poc).layno1 = opj_int_min(
      (*l_current_poc).layno1 as OPJ_INT32,
      (*l_tcp).numlayers as OPJ_INT32,
    ) as OPJ_UINT32;
    (*l_current_poc).resno1 = opj_int_min(
      (*l_current_poc).resno1 as OPJ_INT32,
      (*l_tccp).numresolutions as OPJ_INT32,
    ) as OPJ_UINT32;
    (*l_current_poc).compno1 = opj_int_min(
      (*l_current_poc).compno1 as OPJ_INT32,
      l_nb_comp as OPJ_INT32,
    ) as OPJ_UINT32;
    l_current_poc = l_current_poc.offset(1);
    i = i.wrapping_add(1)
  }
  *p_data_written = l_poc_size;
}
/* *
 * Gets the maximum size taken by the writing of a POC.
 */
unsafe fn opj_j2k_get_max_poc_size(mut p_j2k: *mut opj_j2k_t) -> OPJ_UINT32 {
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_nb_tiles = 0 as OPJ_UINT32;
  let mut l_max_poc = 0 as OPJ_UINT32;
  let mut i: OPJ_UINT32 = 0;
  l_tcp = (*p_j2k).m_cp.tcps;
  l_nb_tiles = (*p_j2k).m_cp.th.wrapping_mul((*p_j2k).m_cp.tw);
  i = 0 as OPJ_UINT32;
  while i < l_nb_tiles {
    l_max_poc = opj_uint_max(l_max_poc, (*l_tcp).numpocs);
    l_tcp = l_tcp.offset(1);
    i = i.wrapping_add(1)
  }
  l_max_poc = l_max_poc.wrapping_add(1);
  return (4u32)
    .wrapping_add((9u32).wrapping_mul(l_max_poc));
}
/* *
 * Gets the maximum size taken by the toc headers of all the tile parts of any given tile.
 */
unsafe fn opj_j2k_get_max_toc_size(mut p_j2k: *mut opj_j2k_t) -> OPJ_UINT32 {
  let mut i: OPJ_UINT32 = 0;
  let mut l_nb_tiles: OPJ_UINT32 = 0;
  let mut l_max = 0 as OPJ_UINT32;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  l_tcp = (*p_j2k).m_cp.tcps;
  l_nb_tiles = (*p_j2k).m_cp.tw.wrapping_mul((*p_j2k).m_cp.th);
  i = 0 as OPJ_UINT32;
  while i < l_nb_tiles {
    l_max = opj_uint_max(l_max, (*l_tcp).m_nb_tile_parts);
    l_tcp = l_tcp.offset(1);
    i = i.wrapping_add(1)
  }
  return (12u32).wrapping_mul(l_max);
}
/* *
 * Gets the maximum size taken by the headers of the SOT.
 *
 * @param       p_j2k   the jpeg2000 codec to use.
 */
unsafe fn opj_j2k_get_specific_header_sizes(mut p_j2k: *mut opj_j2k_t) -> OPJ_UINT32 {
  let mut l_nb_bytes = 0 as OPJ_UINT32;
  let mut l_nb_comps: OPJ_UINT32 = 0;
  let mut l_coc_bytes: OPJ_UINT32 = 0;
  let mut l_qcc_bytes: OPJ_UINT32 = 0;
  l_nb_comps = (*(*p_j2k).m_private_image)
    .numcomps
    .wrapping_sub(1u32);
  l_nb_bytes = (l_nb_bytes as libc::c_uint).wrapping_add(opj_j2k_get_max_toc_size(p_j2k))
    as OPJ_UINT32;
  if !((*p_j2k).m_cp.rsiz as libc::c_int >= 0x3i32
    && (*p_j2k).m_cp.rsiz as libc::c_int <= 0x6i32)
  {
    l_coc_bytes = opj_j2k_get_max_coc_size(p_j2k);
    l_nb_bytes = (l_nb_bytes as libc::c_uint).wrapping_add(l_nb_comps.wrapping_mul(l_coc_bytes))
      as OPJ_UINT32;
    l_qcc_bytes = opj_j2k_get_max_qcc_size(p_j2k);
    l_nb_bytes = (l_nb_bytes as libc::c_uint).wrapping_add(l_nb_comps.wrapping_mul(l_qcc_bytes))
      as OPJ_UINT32
  }
  l_nb_bytes = (l_nb_bytes as libc::c_uint).wrapping_add(opj_j2k_get_max_poc_size(p_j2k))
    as OPJ_UINT32;
  if (*p_j2k).m_specific_param.m_encoder.m_PLT != 0 {
    /* Reserve space for PLT markers */
    let mut i: OPJ_UINT32 = 0;
    let mut l_cp: *const opj_cp_t = &mut (*p_j2k).m_cp;
    let mut l_max_packet_count = 0 as OPJ_UINT32;
    i = 0 as OPJ_UINT32;
    while i < (*l_cp).th.wrapping_mul((*l_cp).tw) {
      l_max_packet_count = opj_uint_max(
        l_max_packet_count,
        opj_get_encoding_packet_count((*p_j2k).m_private_image, l_cp, i),
      );
      i = i.wrapping_add(1)
    }
    /* Minimum 6 bytes per PLT marker, and at a minimum (taking a pessimistic */
    /* estimate of 4 bytes for a packet size), one can write */
    /* (65536-6) / 4 = 16382 paquet sizes per PLT marker */
    (*p_j2k).m_specific_param.m_encoder.m_reserved_bytes_for_PLT =
      (6u32).wrapping_mul(opj_uint_ceildiv(
        l_max_packet_count,
        16382 as OPJ_UINT32,
      ));
    /* Maximum 5 bytes per packet to encode a full UINT32 */
    l_nb_bytes = (l_nb_bytes as libc::c_uint)
      .wrapping_add((5u32).wrapping_mul(l_max_packet_count))
      as OPJ_UINT32;
    (*p_j2k).m_specific_param.m_encoder.m_reserved_bytes_for_PLT =
      ((*p_j2k).m_specific_param.m_encoder.m_reserved_bytes_for_PLT as libc::c_uint)
        .wrapping_add(l_nb_bytes) as OPJ_UINT32;
    (*p_j2k).m_specific_param.m_encoder.m_reserved_bytes_for_PLT =
      ((*p_j2k).m_specific_param.m_encoder.m_reserved_bytes_for_PLT as libc::c_uint)
        .wrapping_add(1u32) as OPJ_UINT32;
    l_nb_bytes = (l_nb_bytes as libc::c_uint)
      .wrapping_add((*p_j2k).m_specific_param.m_encoder.m_reserved_bytes_for_PLT)
      as OPJ_UINT32
  }
  /* ** DEVELOPER CORNER, Add room for your headers ***/
  return l_nb_bytes;
}
/* *
 * Reads a POC marker (Progression Order Change)
 *
 * @param       p_header_data   the data contained in the POC box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the POC marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a POC marker (Progression Order Change)
 *
 * @param       p_header_data   the data contained in the POC box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the POC marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_poc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_nb_comp: OPJ_UINT32 = 0;
  let mut l_tmp: OPJ_UINT32 = 0;
  let mut l_image = 0 as *mut opj_image_t;
  let mut l_old_poc_nb: OPJ_UINT32 = 0;
  let mut l_current_poc_nb: OPJ_UINT32 = 0;
  let mut l_current_poc_remaining: OPJ_UINT32 = 0;
  let mut l_chunk_size: OPJ_UINT32 = 0;
  let mut l_comp_room: OPJ_UINT32 = 0;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_current_poc = 0 as *mut opj_poc_t;
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_image = (*p_j2k).m_private_image;
  l_nb_comp = (*l_image).numcomps;
  if l_nb_comp <= 256u32 {
    l_comp_room = 1 as OPJ_UINT32
  } else {
    l_comp_room = 2 as OPJ_UINT32
  }
  l_chunk_size = (5u32)
    .wrapping_add((2u32).wrapping_mul(l_comp_room));
  l_current_poc_nb = p_header_size.wrapping_div(l_chunk_size);
  l_current_poc_remaining = p_header_size.wrapping_rem(l_chunk_size);
  if l_current_poc_nb <= 0u32
    || l_current_poc_remaining != 0u32
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading POC marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = if (*p_j2k).m_specific_param.m_decoder.m_state
    == J2KState::TPH
  {
    &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_default_tcp
  };
  l_old_poc_nb = if (*l_tcp).POC() as libc::c_int != 0 {
    (*l_tcp)
      .numpocs
      .wrapping_add(1u32)
  } else {
    0u32
  };
  l_current_poc_nb =
    (l_current_poc_nb as libc::c_uint).wrapping_add(l_old_poc_nb) as OPJ_UINT32;
  if l_current_poc_nb >= 32u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Too many POCs %d\n\x00" as *const u8 as *const libc::c_char,
      l_current_poc_nb,
    );
    return 0i32;
  }
  /* now poc is in use.*/
  (*l_tcp).set_POC(1 as OPJ_BITFIELD); /* RSpoc_i */
  l_current_poc = &mut *(*l_tcp).pocs.as_mut_ptr().offset(l_old_poc_nb as isize) as *mut opj_poc_t; /* CSpoc_i */
  i = l_old_poc_nb; /* LYEpoc_i */
  while i < l_current_poc_nb {
    opj_read_bytes_LE(
      p_header_data,
      &mut (*l_current_poc).resno0,
      1 as OPJ_UINT32,
    );
    p_header_data = p_header_data.offset(1);
    opj_read_bytes_LE(p_header_data, &mut (*l_current_poc).compno0, l_comp_room);
    p_header_data = p_header_data.offset(l_comp_room as isize);
    opj_read_bytes_LE(
      p_header_data,
      &mut (*l_current_poc).layno1,
      2 as OPJ_UINT32,
    );
    /* make sure layer end is in acceptable bounds */
    (*l_current_poc).layno1 = opj_uint_min((*l_current_poc).layno1, (*l_tcp).numlayers); /* REpoc_i */
    p_header_data = p_header_data.offset(2); /* CEpoc_i */
    opj_read_bytes_LE(
      p_header_data,
      &mut (*l_current_poc).resno1,
      1 as OPJ_UINT32,
    ); /* Ppoc_i */
    p_header_data = p_header_data.offset(1);
    opj_read_bytes_LE(p_header_data, &mut (*l_current_poc).compno1, l_comp_room);
    p_header_data = p_header_data.offset(l_comp_room as isize);
    opj_read_bytes_LE(p_header_data, &mut l_tmp, 1 as OPJ_UINT32);
    p_header_data = p_header_data.offset(1);
    (*l_current_poc).prg = l_tmp as OPJ_PROG_ORDER;
    /* make sure comp is in acceptable bounds */
    (*l_current_poc).compno1 = opj_uint_min((*l_current_poc).compno1, l_nb_comp);
    l_current_poc = l_current_poc.offset(1);
    i = i.wrapping_add(1)
  }
  (*l_tcp).numpocs = l_current_poc_nb.wrapping_sub(1u32);
  return 1i32;
}
/* *
 * Reads a CRG marker (Component registration)
 *
 * @param       p_header_data   the data contained in the TLM box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the TLM marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a CRG marker (Component registration)
 *
 * @param       p_header_data   the data contained in the TLM box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the TLM marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_crg(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_nb_comp: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_nb_comp = (*(*p_j2k).m_private_image).numcomps;
  if p_header_size != l_nb_comp.wrapping_mul(4u32) {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading CRG marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* Do not care of this at the moment since only local variables are set here */
  /*
  for
          (i = 0; i < l_nb_comp; ++i)
  {
          opj_read_bytes(p_header_data,&l_Xcrg_i,2);                              // Xcrg_i
          p_header_data+=2;
          opj_read_bytes(p_header_data,&l_Ycrg_i,2);                              // Xcrg_i
          p_header_data+=2;
  }
  */
  return 1i32;
}
/* *
 * Reads a TLM marker (Tile Length Marker)
 *
 * @param       p_header_data   the data contained in the TLM box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the TLM marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a TLM marker (Tile Length Marker)
 *
 * @param       p_header_data   the data contained in the TLM box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the TLM marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_tlm(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_Ztlm: OPJ_UINT32 = 0;
  let mut l_Stlm: OPJ_UINT32 = 0;
  let mut l_ST: OPJ_UINT32 = 0;
  let mut l_SP: OPJ_UINT32 = 0;
  let mut l_tot_num_tp_remaining: OPJ_UINT32 = 0;
  let mut l_quotient: OPJ_UINT32 = 0;
  let mut l_Ptlm_size: OPJ_UINT32 = 0;
  /* preconditions */
  /* Stlm */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  if p_header_size < 2u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading TLM marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  p_header_size = (p_header_size as libc::c_uint).wrapping_sub(2u32)
    as OPJ_UINT32;
  opj_read_bytes_LE(p_header_data, &mut l_Ztlm, 1 as OPJ_UINT32);
  p_header_data = p_header_data.offset(1);
  opj_read_bytes_LE(p_header_data, &mut l_Stlm, 1 as OPJ_UINT32);
  p_header_data = p_header_data.offset(1);
  l_ST = l_Stlm >> 4i32 & 0x3u32;
  l_SP = l_Stlm >> 6i32 & 0x1u32;
  l_Ptlm_size = l_SP
    .wrapping_add(1u32)
    .wrapping_mul(2u32);
  l_quotient = l_Ptlm_size.wrapping_add(l_ST);
  l_tot_num_tp_remaining = p_header_size.wrapping_rem(l_quotient);
  if l_tot_num_tp_remaining != 0u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading TLM marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* FIXME Do not care of this at the moment since only local variables are set here */
  /*
  for
          (i = 0; i < l_tot_num_tp; ++i)
  {
          opj_read_bytes(p_header_data,&l_Ttlm_i,l_ST);                           // Ttlm_i
          p_header_data += l_ST;
          opj_read_bytes(p_header_data,&l_Ptlm_i,l_Ptlm_size);            // Ptlm_i
          p_header_data += l_Ptlm_size;
  }*/
  return 1i32;
}
/* *
 * Reads a PLM marker (Packet length, main header marker)
 *
 * @param       p_header_data   the data contained in the TLM box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the TLM marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a PLM marker (Packet length, main header marker)
 *
 * @param       p_header_data   the data contained in the TLM box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the TLM marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_plm(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  if p_header_size < 1u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading PLM marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* Do not care of this at the moment since only local variables are set here */
  /*
  opj_read_bytes(p_header_data,&l_Zplm,1);                                        // Zplm
  ++p_header_data;
  --p_header_size;

  while
          (p_header_size > 0)
  {
          opj_read_bytes(p_header_data,&l_Nplm,1);                                // Nplm
          ++p_header_data;
          p_header_size -= (1+l_Nplm);
          if
                  (p_header_size < 0)
          {
                  opj_event_msg(p_manager, EVT_ERROR, "Error reading PLM marker\n");
                  return false;
          }
          for
                  (i = 0; i < l_Nplm; ++i)
          {
                  opj_read_bytes(p_header_data,&l_tmp,1);                         // Iplm_ij
                  ++p_header_data;
                  // take only the last seven bytes
                  l_packet_len |= (l_tmp & 0x7f);
                  if
                          (l_tmp & 0x80)
                  {
                          l_packet_len <<= 7;
                  }
                  else
                  {
          // store packet length and proceed to next packet
                          l_packet_len = 0;
                  }
          }
          if
                  (l_packet_len != 0)
          {
                  opj_event_msg(p_manager, EVT_ERROR, "Error reading PLM marker\n");
                  return false;
          }
  }
  */
  return 1i32;
}
/* *
 * Reads a PLT marker (Packet length, tile-part header)
 *
 * @param       p_header_data   the data contained in the PLT box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the PLT marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a PLT marker (Packet length, tile-part header)
 *
 * @param       p_header_data   the data contained in the PLT box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the PLT marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_plt(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_Zplt: OPJ_UINT32 = 0;
  let mut l_tmp: OPJ_UINT32 = 0;
  let mut l_packet_len = 0 as OPJ_UINT32;
  let mut i: OPJ_UINT32 = 0;
  /* preconditions */
  /* Iplt_ij */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  if p_header_size < 1u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading PLT marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  opj_read_bytes_LE(p_header_data, &mut l_Zplt, 1 as OPJ_UINT32);
  p_header_data = p_header_data.offset(1);
  p_header_size = p_header_size.wrapping_sub(1);
  i = 0 as OPJ_UINT32;
  while i < p_header_size {
    opj_read_bytes_LE(p_header_data, &mut l_tmp, 1 as OPJ_UINT32);
    p_header_data = p_header_data.offset(1);
    /* take only the last seven bytes */
    l_packet_len |= l_tmp & 0x7fu32;
    if l_tmp & 0x80u32 != 0 {
      l_packet_len <<= 7i32
    } else {
      /* store packet length and proceed to next packet */
      l_packet_len = 0 as OPJ_UINT32
    }
    i = i.wrapping_add(1)
  }
  if l_packet_len != 0u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading PLT marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  return 1i32;
}
/* *
 * Reads a PPM marker (Packed headers, main header)
 *
 * @param       p_header_data   the data contained in the POC box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the POC marker.
 * @param       p_manager               the user event manager.
 */
/* *
 * Reads a PPM marker (Packed packet headers, main header)
 *
 * @param       p_header_data   the data contained in the POC box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the POC marker.
 * @param       p_manager               the user event manager.
 */
unsafe extern "C" fn opj_j2k_read_ppm(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_Z_ppm: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  /* We need to have the Z_ppm element + 1 byte of Nppm/Ippm at minimum */
  if p_header_size < 2u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading PPM marker\n\x00" as *const u8 as *const libc::c_char,
    ); /* Z_ppm */
    return 0i32;
  }
  l_cp = &mut (*p_j2k).m_cp;
  (*l_cp).set_ppm(1 as OPJ_BITFIELD);
  opj_read_bytes_LE(p_header_data, &mut l_Z_ppm, 1 as OPJ_UINT32);
  p_header_data = p_header_data.offset(1);
  p_header_size = p_header_size.wrapping_sub(1);
  /* check allocation needed */
  if (*l_cp).ppm_markers.is_null() {
    /* first PPM marker */
    let mut l_newCount = l_Z_ppm.wrapping_add(1u32); /* can't overflow, l_Z_ppm is UINT8 */
    assert!((*l_cp).ppm_markers_count == 0u32);
    (*l_cp).ppm_markers = opj_calloc(
      l_newCount as size_t,
      core::mem::size_of::<opj_ppx>() as libc::c_ulong,
    ) as *mut opj_ppx;
    if (*l_cp).ppm_markers.is_null() {
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to read PPM marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*l_cp).ppm_markers_count = l_newCount
  } else if (*l_cp).ppm_markers_count <= l_Z_ppm {
    let mut l_newCount_0 = l_Z_ppm.wrapping_add(1u32);
    let mut new_ppm_markers = 0 as *mut opj_ppx;
    new_ppm_markers = opj_realloc(
      (*l_cp).ppm_markers as *mut libc::c_void,
      (l_newCount_0 as libc::c_ulong)
        .wrapping_mul(core::mem::size_of::<opj_ppx>() as libc::c_ulong),
    ) as *mut opj_ppx;
    if new_ppm_markers.is_null() {
      /* clean up to be done on l_cp destruction */
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to read PPM marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*l_cp).ppm_markers = new_ppm_markers;
    memset(
      (*l_cp)
        .ppm_markers
        .offset((*l_cp).ppm_markers_count as isize) as *mut libc::c_void,
      0i32,
      (l_newCount_0.wrapping_sub((*l_cp).ppm_markers_count) as libc::c_ulong)
        .wrapping_mul(core::mem::size_of::<opj_ppx>() as libc::c_ulong),
    );
    (*l_cp).ppm_markers_count = l_newCount_0
  }
  if !(*(*l_cp).ppm_markers.offset(l_Z_ppm as isize))
    .m_data
    .is_null()
  {
    /* clean up to be done on l_cp destruction */
    opj_event_msg(
      p_manager,
      1i32,
      b"Zppm %u already read\n\x00" as *const u8 as *const libc::c_char,
      l_Z_ppm,
    );
    return 0i32;
  }
  let ref mut fresh12 = (*(*l_cp).ppm_markers.offset(l_Z_ppm as isize)).m_data;
  *fresh12 = opj_malloc(p_header_size as size_t) as *mut OPJ_BYTE;
  if (*(*l_cp).ppm_markers.offset(l_Z_ppm as isize))
    .m_data
    .is_null()
  {
    /* clean up to be done on l_cp destruction */
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to read PPM marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  (*(*l_cp).ppm_markers.offset(l_Z_ppm as isize)).m_data_size = p_header_size;
  memcpy(
    (*(*l_cp).ppm_markers.offset(l_Z_ppm as isize)).m_data as *mut libc::c_void,
    p_header_data as *const libc::c_void,
    p_header_size as libc::c_ulong,
  );
  return 1i32;
}
/* *
 * Merges all PPM markers read (Packed headers, main header)
 *
 * @param       p_cp      main coding parameters.
 * @param       p_manager the user event manager.
 */
/* *
 * Merges all PPM markers read (Packed headers, main header)
 *
 * @param       p_cp      main coding parameters.
 * @param       p_manager the user event manager.
 */
unsafe fn opj_j2k_merge_ppm(
  mut p_cp: *mut opj_cp_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_ppm_data_size: OPJ_UINT32 = 0;
  let mut l_N_ppm_remaining: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_cp.is_null());
  assert!(!p_manager.is_null());
  assert!((*p_cp).ppm_buffer.is_null());
  if (*p_cp).ppm() == 0u32 {
    return 1i32;
  }
  l_ppm_data_size = 0u32;
  l_N_ppm_remaining = 0u32;
  i = 0u32;
  while i < (*p_cp).ppm_markers_count {
    if !(*(*p_cp).ppm_markers.offset(i as isize)).m_data.is_null() {
      /* standard doesn't seem to require contiguous Zppm */
      let mut l_N_ppm: OPJ_UINT32 = 0;
      let mut l_data_size = (*(*p_cp).ppm_markers.offset(i as isize)).m_data_size;
      let mut l_data: *const OPJ_BYTE = (*(*p_cp).ppm_markers.offset(i as isize)).m_data;
      if l_N_ppm_remaining >= l_data_size {
        l_N_ppm_remaining =
          (l_N_ppm_remaining as libc::c_uint).wrapping_sub(l_data_size) as OPJ_UINT32;
        l_data_size = 0u32
      } else {
        l_data = l_data.offset(l_N_ppm_remaining as isize);
        l_data_size =
          (l_data_size as libc::c_uint).wrapping_sub(l_N_ppm_remaining) as OPJ_UINT32;
        l_N_ppm_remaining = 0u32
      }
      if l_data_size > 0u32 {
        loop {
          /* read Nppm */
          if l_data_size < 4u32 {
            /* clean up to be done on l_cp destruction */
            opj_event_msg(
              p_manager,
              1i32,
              b"Not enough bytes to read Nppm\n\x00" as *const u8 as *const libc::c_char,
            ); /* can't overflow, max 256 markers of max 65536 bytes, that is when PPM markers are not corrupted which is checked elsewhere */
            return 0i32;
          }
          opj_read_bytes_LE(l_data, &mut l_N_ppm, 4 as OPJ_UINT32);
          l_data = l_data.offset(4);
          l_data_size = (l_data_size as libc::c_uint).wrapping_sub(4u32)
            as OPJ_UINT32;
          l_ppm_data_size =
            (l_ppm_data_size as libc::c_uint).wrapping_add(l_N_ppm) as OPJ_UINT32;
          if l_data_size >= l_N_ppm {
            l_data_size =
              (l_data_size as libc::c_uint).wrapping_sub(l_N_ppm) as OPJ_UINT32;
            l_data = l_data.offset(l_N_ppm as isize)
          } else {
            l_N_ppm_remaining = l_N_ppm.wrapping_sub(l_data_size);
            l_data_size = 0u32
          }
          if !(l_data_size > 0u32) {
            break;
          }
        }
      }
    }
    i = i.wrapping_add(1)
  }
  if l_N_ppm_remaining != 0u32 {
    /* clean up to be done on l_cp destruction */
    opj_event_msg(
      p_manager,
      1i32,
      b"Corrupted PPM markers\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  (*p_cp).ppm_buffer = opj_malloc(l_ppm_data_size as size_t) as *mut OPJ_BYTE;
  if (*p_cp).ppm_buffer.is_null() {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to read PPM marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  (*p_cp).ppm_len = l_ppm_data_size;
  l_ppm_data_size = 0u32;
  l_N_ppm_remaining = 0u32;
  i = 0u32;
  while i < (*p_cp).ppm_markers_count {
    if !(*(*p_cp).ppm_markers.offset(i as isize)).m_data.is_null() {
      /* standard doesn't seem to require contiguous Zppm */
      let mut l_N_ppm_0: OPJ_UINT32 = 0;
      let mut l_data_size_0 = (*(*p_cp).ppm_markers.offset(i as isize)).m_data_size;
      let mut l_data_0: *const OPJ_BYTE = (*(*p_cp).ppm_markers.offset(i as isize)).m_data;
      if l_N_ppm_remaining >= l_data_size_0 {
        memcpy(
          (*p_cp).ppm_buffer.offset(l_ppm_data_size as isize) as *mut libc::c_void,
          l_data_0 as *const libc::c_void,
          l_data_size_0 as libc::c_ulong,
        );
        l_ppm_data_size =
          (l_ppm_data_size as libc::c_uint).wrapping_add(l_data_size_0) as OPJ_UINT32;
        l_N_ppm_remaining = (l_N_ppm_remaining as libc::c_uint).wrapping_sub(l_data_size_0)
          as OPJ_UINT32;
        l_data_size_0 = 0u32
      } else {
        memcpy(
          (*p_cp).ppm_buffer.offset(l_ppm_data_size as isize) as *mut libc::c_void,
          l_data_0 as *const libc::c_void,
          l_N_ppm_remaining as libc::c_ulong,
        );
        l_ppm_data_size = (l_ppm_data_size as libc::c_uint).wrapping_add(l_N_ppm_remaining)
          as OPJ_UINT32;
        l_data_0 = l_data_0.offset(l_N_ppm_remaining as isize);
        l_data_size_0 = (l_data_size_0 as libc::c_uint).wrapping_sub(l_N_ppm_remaining)
          as OPJ_UINT32;
        l_N_ppm_remaining = 0u32
      }
      if l_data_size_0 > 0u32 {
        loop {
          /* read Nppm */
          if l_data_size_0 < 4u32 {
            /* clean up to be done on l_cp destruction */
            opj_event_msg(
              p_manager,
              1i32,
              b"Not enough bytes to read Nppm\n\x00" as *const u8 as *const libc::c_char,
            );
            return 0i32;
          }
          opj_read_bytes_LE(l_data_0, &mut l_N_ppm_0, 4 as OPJ_UINT32);
          l_data_0 = l_data_0.offset(4);
          l_data_size_0 = (l_data_size_0 as libc::c_uint)
            .wrapping_sub(4u32) as OPJ_UINT32
            as OPJ_UINT32;
          if l_data_size_0 >= l_N_ppm_0 {
            memcpy(
              (*p_cp).ppm_buffer.offset(l_ppm_data_size as isize) as *mut libc::c_void,
              l_data_0 as *const libc::c_void,
              l_N_ppm_0 as libc::c_ulong,
            );
            l_ppm_data_size =
              (l_ppm_data_size as libc::c_uint).wrapping_add(l_N_ppm_0) as OPJ_UINT32;
            l_data_size_0 =
              (l_data_size_0 as libc::c_uint).wrapping_sub(l_N_ppm_0) as OPJ_UINT32;
            l_data_0 = l_data_0.offset(l_N_ppm_0 as isize)
          } else {
            memcpy(
              (*p_cp).ppm_buffer.offset(l_ppm_data_size as isize) as *mut libc::c_void,
              l_data_0 as *const libc::c_void,
              l_data_size_0 as libc::c_ulong,
            );
            l_ppm_data_size = (l_ppm_data_size as libc::c_uint).wrapping_add(l_data_size_0)
              as OPJ_UINT32;
            l_N_ppm_remaining = l_N_ppm_0.wrapping_sub(l_data_size_0);
            l_data_size_0 = 0u32
          }
          if !(l_data_size_0 > 0u32) {
            break;
          }
        }
      }
      opj_free((*(*p_cp).ppm_markers.offset(i as isize)).m_data as *mut libc::c_void);
      let ref mut fresh13 = (*(*p_cp).ppm_markers.offset(i as isize)).m_data;
      *fresh13 = 0 as *mut OPJ_BYTE;
      (*(*p_cp).ppm_markers.offset(i as isize)).m_data_size = 0u32
    }
    i = i.wrapping_add(1)
  }
  (*p_cp).ppm_data = (*p_cp).ppm_buffer;
  (*p_cp).ppm_data_size = (*p_cp).ppm_len;
  (*p_cp).ppm_markers_count = 0u32;
  opj_free((*p_cp).ppm_markers as *mut libc::c_void);
  (*p_cp).ppm_markers = 0 as *mut opj_ppx;
  return 1i32;
}
/* *
 * Reads a PPT marker (Packed packet headers, tile-part header)
 *
 * @param       p_header_data   the data contained in the PPT box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the PPT marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a PPT marker (Packed packet headers, tile-part header)
 *
 * @param       p_header_data   the data contained in the PPT box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the PPT marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_ppt(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_Z_ppt: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  /* We need to have the Z_ppt element + 1 byte of Ippt at minimum */
  if p_header_size < 2u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading PPT marker\n\x00" as *const u8 as *const libc::c_char,
    ); /* Z_ppt */
    return 0i32;
  }
  l_cp = &mut (*p_j2k).m_cp;
  if (*l_cp).ppm() != 0 {
    opj_event_msg(p_manager, 1i32,
                      b"Error reading PPT marker: packet header have been previously found in the main header (PPM marker).\n\x00"
                          as *const u8 as *const libc::c_char);
    return 0i32;
  }
  l_tcp = &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t;
  (*l_tcp).set_ppt(1 as OPJ_BITFIELD);
  opj_read_bytes_LE(p_header_data, &mut l_Z_ppt, 1 as OPJ_UINT32);
  p_header_data = p_header_data.offset(1);
  p_header_size = p_header_size.wrapping_sub(1);
  /* check allocation needed */
  if (*l_tcp).ppt_markers.is_null() {
    /* first PPT marker */
    let mut l_newCount = l_Z_ppt.wrapping_add(1u32); /* can't overflow, l_Z_ppt is UINT8 */
    assert!((*l_tcp).ppt_markers_count == 0u32);
    (*l_tcp).ppt_markers = opj_calloc(
      l_newCount as size_t,
      core::mem::size_of::<opj_ppx>() as libc::c_ulong,
    ) as *mut opj_ppx;
    if (*l_tcp).ppt_markers.is_null() {
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to read PPT marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*l_tcp).ppt_markers_count = l_newCount
  } else if (*l_tcp).ppt_markers_count <= l_Z_ppt {
    let mut l_newCount_0 = l_Z_ppt.wrapping_add(1u32);
    let mut new_ppt_markers = 0 as *mut opj_ppx;
    new_ppt_markers = opj_realloc(
      (*l_tcp).ppt_markers as *mut libc::c_void,
      (l_newCount_0 as libc::c_ulong)
        .wrapping_mul(core::mem::size_of::<opj_ppx>() as libc::c_ulong),
    ) as *mut opj_ppx;
    if new_ppt_markers.is_null() {
      /* clean up to be done on l_tcp destruction */
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to read PPT marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*l_tcp).ppt_markers = new_ppt_markers;
    memset(
      (*l_tcp)
        .ppt_markers
        .offset((*l_tcp).ppt_markers_count as isize) as *mut libc::c_void,
      0i32,
      (l_newCount_0.wrapping_sub((*l_tcp).ppt_markers_count) as libc::c_ulong)
        .wrapping_mul(core::mem::size_of::<opj_ppx>() as libc::c_ulong),
    );
    (*l_tcp).ppt_markers_count = l_newCount_0
  }
  if !(*(*l_tcp).ppt_markers.offset(l_Z_ppt as isize))
    .m_data
    .is_null()
  {
    /* clean up to be done on l_tcp destruction */
    opj_event_msg(
      p_manager,
      1i32,
      b"Zppt %u already read\n\x00" as *const u8 as *const libc::c_char,
      l_Z_ppt,
    );
    return 0i32;
  }
  let ref mut fresh14 = (*(*l_tcp).ppt_markers.offset(l_Z_ppt as isize)).m_data;
  *fresh14 = opj_malloc(p_header_size as size_t) as *mut OPJ_BYTE;
  if (*(*l_tcp).ppt_markers.offset(l_Z_ppt as isize))
    .m_data
    .is_null()
  {
    /* clean up to be done on l_tcp destruction */
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to read PPT marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  (*(*l_tcp).ppt_markers.offset(l_Z_ppt as isize)).m_data_size = p_header_size;
  memcpy(
    (*(*l_tcp).ppt_markers.offset(l_Z_ppt as isize)).m_data as *mut libc::c_void,
    p_header_data as *const libc::c_void,
    p_header_size as libc::c_ulong,
  );
  return 1i32;
}
/* *
 * Merges all PPT markers read (Packed headers, tile-part header)
 *
 * @param       p_tcp   the tile.
 * @param       p_manager               the user event manager.
 */
/* *
 * Merges all PPT markers read (Packed packet headers, tile-part header)
 *
 * @param       p_tcp   the tile.
 * @param       p_manager               the user event manager.
 */
unsafe fn opj_j2k_merge_ppt(
  mut p_tcp: *mut opj_tcp_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_ppt_data_size: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_tcp.is_null());
  assert!(!p_manager.is_null());
  if !(*p_tcp).ppt_buffer.is_null() {
    opj_event_msg(
      p_manager,
      1i32,
      b"opj_j2k_merge_ppt() has already been called\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if (*p_tcp).ppt() == 0u32 {
    return 1i32;
  }
  l_ppt_data_size = 0u32;
  i = 0u32;
  while i < (*p_tcp).ppt_markers_count {
    l_ppt_data_size = (l_ppt_data_size as libc::c_uint)
      .wrapping_add((*(*p_tcp).ppt_markers.offset(i as isize)).m_data_size)
      as OPJ_UINT32;
    i = i.wrapping_add(1)
    /* can't overflow, max 256 markers of max 65536 bytes */
  }
  (*p_tcp).ppt_buffer = opj_malloc(l_ppt_data_size as size_t) as *mut OPJ_BYTE;
  if (*p_tcp).ppt_buffer.is_null() {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to read PPT marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  (*p_tcp).ppt_len = l_ppt_data_size;
  l_ppt_data_size = 0u32;
  i = 0u32;
  while i < (*p_tcp).ppt_markers_count {
    if !(*(*p_tcp).ppt_markers.offset(i as isize)).m_data.is_null() {
      /* standard doesn't seem to require contiguous Zppt */
      memcpy(
        (*p_tcp).ppt_buffer.offset(l_ppt_data_size as isize) as *mut libc::c_void,
        (*(*p_tcp).ppt_markers.offset(i as isize)).m_data as *const libc::c_void,
        (*(*p_tcp).ppt_markers.offset(i as isize)).m_data_size as libc::c_ulong,
      ); /* can't overflow, max 256 markers of max 65536 bytes */
      l_ppt_data_size = (l_ppt_data_size as libc::c_uint)
        .wrapping_add((*(*p_tcp).ppt_markers.offset(i as isize)).m_data_size)
        as OPJ_UINT32;
      opj_free((*(*p_tcp).ppt_markers.offset(i as isize)).m_data as *mut libc::c_void);
      let ref mut fresh15 = (*(*p_tcp).ppt_markers.offset(i as isize)).m_data;
      *fresh15 = 0 as *mut OPJ_BYTE;
      (*(*p_tcp).ppt_markers.offset(i as isize)).m_data_size = 0u32
    }
    i = i.wrapping_add(1)
  }
  (*p_tcp).ppt_markers_count = 0u32;
  opj_free((*p_tcp).ppt_markers as *mut libc::c_void);
  (*p_tcp).ppt_markers = 0 as *mut opj_ppx;
  (*p_tcp).ppt_data = (*p_tcp).ppt_buffer;
  (*p_tcp).ppt_data_size = (*p_tcp).ppt_len;
  return 1i32;
}
/* *
 * Writes the TLM marker (Tile Length Marker)
 *
 * @param       p_stream                                the stream to write data to.
 * @param       p_j2k                           J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_tlm(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  let mut l_tlm_size: OPJ_UINT32 = 0;
  let mut size_per_tile_part: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  /* 10921 = (65535 - header_size) / size_per_tile_part where */
  /* header_size = 4 and size_per_tile_part = 6 */
  if (*p_j2k).m_specific_param.m_encoder.m_total_tile_parts > 10921u32 {
    /* We could do more but it would require writing several TLM markers */
    opj_event_msg(
      p_manager,
      1i32,
      b"A maximum of 10921 tile-parts are supported currently when writing TLM marker\n\x00"
        as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if (*p_j2k).m_specific_param.m_encoder.m_total_tile_parts <= 255u32 {
    size_per_tile_part = 5 as OPJ_UINT32;
    (*p_j2k).m_specific_param.m_encoder.m_Ttlmi_is_byte = 1i32
  } else {
    size_per_tile_part = 6 as OPJ_UINT32;
    (*p_j2k).m_specific_param.m_encoder.m_Ttlmi_is_byte = 0i32
  }
  l_tlm_size = ((2i32 + 4i32) as libc::c_uint).wrapping_add(
    size_per_tile_part.wrapping_mul((*p_j2k).m_specific_param.m_encoder.m_total_tile_parts),
  );
  if l_tlm_size > (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size {
    let mut new_header_tile_data = opj_realloc(
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
      l_tlm_size as size_t,
    ) as *mut OPJ_BYTE;
    if new_header_tile_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to write TLM marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = new_header_tile_data;
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = l_tlm_size
  }
  memset(
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
    0i32,
    l_tlm_size as libc::c_ulong,
  );
  l_current_data = (*p_j2k).m_specific_param.m_encoder.m_header_tile_data;
  /* change the way data is written to avoid seeking if possible */
  /* TODO */
  (*p_j2k).m_specific_param.m_encoder.m_tlm_start = opj_stream_tell(p_stream); /* TLM */
  opj_write_bytes_LE(
    l_current_data,
    0xff55 as OPJ_UINT32,
    2 as OPJ_UINT32,
  ); /* Lpoc */
  l_current_data = l_current_data.offset(2); /* Ztlm=0*/
  opj_write_bytes_LE(
    l_current_data,
    l_tlm_size.wrapping_sub(2u32),
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    0 as OPJ_UINT32,
    1 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(1);
  /* Stlm 0x50= ST=1(8bits-255 tiles max),SP=1(Ptlm=32bits) */
  /* Stlm 0x60= ST=2(16bits-65535 tiles max),SP=1(Ptlm=32bits) */
  opj_write_bytes_LE(
    l_current_data,
    if size_per_tile_part == 5u32 {
      0x50i32
    } else {
      0x60i32
    } as OPJ_UINT32,
    1 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(1);
  /* do nothing on the size_per_tile_part * l_j2k->m_specific_param.m_encoder.m_total_tile_parts remaining data */
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_tlm_size as OPJ_SIZE_T,
    p_manager,
  ) != l_tlm_size as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Writes the SOT marker (Start of tile-part)
 *
 * @param       p_j2k            J2K codec.
 * @param       p_data           Output buffer
 * @param       total_data_size  Output buffer size
 * @param       p_data_written   Number of bytes written into stream
 * @param       p_stream         the stream to write data to.
 * @param       p_manager        the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_sot(
  mut p_j2k: *mut opj_j2k_t,
  mut p_data: *mut OPJ_BYTE,
  mut total_data_size: OPJ_UINT32,
  mut p_data_written: *mut OPJ_UINT32,
  mut p_stream: *const opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */
  /* Lsot */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  if total_data_size < 12u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough bytes in output buffer to write SOT marker\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 0i32;
  }
  opj_write_bytes_LE(
    p_data,
    0xff90 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  p_data = p_data.offset(2);
  opj_write_bytes_LE(
    p_data,
    10 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  p_data = p_data.offset(2);
  opj_write_bytes_LE(
    p_data,
    (*p_j2k).m_current_tile_number,
    2 as OPJ_UINT32,
  );
  p_data = p_data.offset(2);
  /* Psot  */
  p_data = p_data.offset(4); /* TPsot */
  opj_write_bytes_LE(
    p_data,
    (*p_j2k)
      .m_specific_param
      .m_encoder
      .m_current_tile_part_number,
    1 as OPJ_UINT32,
  ); /* TNsot */
  p_data = p_data.offset(1);
  opj_write_bytes_LE(
    p_data,
    (*(*p_j2k)
      .m_cp
      .tcps
      .offset((*p_j2k).m_current_tile_number as isize))
    .m_nb_tile_parts,
    1 as OPJ_UINT32,
  );
  p_data = p_data.offset(1);
  /* UniPG>> */
  /* USE_JPWL */
  *p_data_written = 12 as OPJ_UINT32;
  return 1i32;
}
/* *
 * Reads values from a SOT marker (Start of tile-part)
 *
 * the j2k decoder state is not affected. No side effects, no checks except for p_header_size.
 *
 * @param       p_header_data   the data contained in the SOT marker.
 * @param       p_header_size   the size of the data contained in the SOT marker.
 * @param       p_tile_no       Isot.
 * @param       p_tot_len       Psot.
 * @param       p_current_part  TPsot.
 * @param       p_num_parts     TNsot.
 * @param       p_manager       the user event manager.
 */
unsafe fn opj_j2k_get_sot_values(
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_tile_no: *mut OPJ_UINT32,
  mut p_tot_len: *mut OPJ_UINT32,
  mut p_current_part: *mut OPJ_UINT32,
  mut p_num_parts: *mut OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_manager.is_null());
  /* Size of this marker is fixed = 12 (we have already read marker and its size)*/
  if p_header_size != 8u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading SOT marker\n\x00" as *const u8 as *const libc::c_char,
    ); /* Isot */
    return 0i32;
  } /* Psot */
  opj_read_bytes_LE(p_header_data, p_tile_no, 2 as OPJ_UINT32); /* TPsot */
  p_header_data = p_header_data.offset(2); /* TNsot */
  opj_read_bytes_LE(p_header_data, p_tot_len, 4 as OPJ_UINT32);
  p_header_data = p_header_data.offset(4);
  opj_read_bytes_LE(
    p_header_data,
    p_current_part,
    1 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(1);
  opj_read_bytes_LE(p_header_data, p_num_parts, 1 as OPJ_UINT32);
  p_header_data = p_header_data.offset(1);
  return 1i32;
}
/* *
 * Reads a SOT marker (Start of tile-part)
 *
 * @param       p_header_data   the data contained in the SOT marker.
 * @param       p_j2k           the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the PPT marker.
 * @param       p_manager       the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_sot(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tot_len: OPJ_UINT32 = 0;
  let mut l_num_parts = 0 as OPJ_UINT32;
  let mut l_current_part: OPJ_UINT32 = 0;
  let mut l_tile_x: OPJ_UINT32 = 0;
  let mut l_tile_y: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  if opj_j2k_get_sot_values(
    p_header_data,
    p_header_size,
    &mut (*p_j2k).m_current_tile_number,
    &mut l_tot_len,
    &mut l_current_part,
    &mut l_num_parts,
    p_manager,
  ) == 0
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading SOT marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  l_cp = &mut (*p_j2k).m_cp;
  /* testcase 2.pdf.SIGFPE.706.1112 */
  if (*p_j2k).m_current_tile_number >= (*l_cp).tw.wrapping_mul((*l_cp).th) {
    opj_event_msg(
      p_manager,
      1i32,
      b"Invalid tile number %d\n\x00" as *const u8 as *const libc::c_char,
      (*p_j2k).m_current_tile_number,
    );
    return 0i32;
  }
  l_tcp = &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t;
  l_tile_x = (*p_j2k).m_current_tile_number.wrapping_rem((*l_cp).tw);
  l_tile_y = (*p_j2k).m_current_tile_number.wrapping_div((*l_cp).tw);
  if (*p_j2k).m_specific_param.m_decoder.m_tile_ind_to_dec < 0i32
    || (*p_j2k).m_current_tile_number
      == (*p_j2k).m_specific_param.m_decoder.m_tile_ind_to_dec as OPJ_UINT32
  {
    /* Do only this check if we decode all tile part headers, or if */
    /* we decode one precise tile. Otherwise the m_current_tile_part_number */
    /* might not be valid */
    /* Fixes issue with id_000020,sig_06,src_001958,op_flip4,pos_149 */
    /* of https://github.com/uclouvain/openjpeg/issues/939 */
    /* We must avoid reading twice the same tile part number for a given tile */
    /* so as to avoid various issues, like opj_j2k_merge_ppt being called */
    /* several times. */
    /* ISO 15444-1 A.4.2 Start of tile-part (SOT) mandates that tile parts */
    /* should appear in increasing order. */
    if (*l_tcp).m_current_tile_part_number + 1i32 != l_current_part as OPJ_INT32 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Invalid tile part index for tile number %d. Got %d, expected %d\n\x00" as *const u8
          as *const libc::c_char,
        (*p_j2k).m_current_tile_number,
        l_current_part,
        (*l_tcp).m_current_tile_part_number + 1i32,
      );
      return 0i32;
    }
  }
  (*l_tcp).m_current_tile_part_number = l_current_part as OPJ_INT32;
  /* USE_JPWL */
  /* look for the tile in the list of already processed tile (in parts). */
  /* Optimization possible here with a more complex data structure and with the removing of tiles */
  /* since the time taken by this function can only grow at the time */
  /* PSot should be equal to zero or >=14 or <= 2^32-1 */
  if l_tot_len != 0u32 && l_tot_len < 14u32
  {
    if l_tot_len == 12u32 {
      /* MSD: Special case for the PHR data which are read by kakadu*/
      opj_event_msg(
        p_manager,
        2i32,
        b"Empty SOT marker detected: Psot=%d.\n\x00" as *const u8 as *const libc::c_char,
        l_tot_len,
      );
    } else {
      opj_event_msg(
        p_manager,
        1i32,
        b"Psot value is not correct regards to the JPEG2000 norm: %d.\n\x00" as *const u8
          as *const libc::c_char,
        l_tot_len,
      );
      return 0i32;
    }
  }
  /* USE_JPWL */
  /* Ref A.4.2: Psot could be equal zero if it is the last tile-part of the codestream.*/
  if l_tot_len == 0 {
    opj_event_msg(p_manager, 4i32,
                      b"Psot value of the current tile-part is equal to zero, we assuming it is the last tile-part of the codestream.\n\x00"
                          as *const u8 as *const libc::c_char);
    (*p_j2k).m_specific_param.m_decoder.m_last_tile_part = 1i32
  }
  if (*l_tcp).m_nb_tile_parts != 0u32
    && l_current_part >= (*l_tcp).m_nb_tile_parts
  {
    /* Fixes https://bugs.chromium.org/p/oss-fuzz/issues/detail?id=2851 */
    opj_event_msg(p_manager, 1i32,
                      b"In SOT marker, TPSot (%d) is not valid regards to the previous number of tile-part (%d), giving up\n\x00"
                          as *const u8 as *const libc::c_char, l_current_part,
                      (*l_tcp).m_nb_tile_parts);
    (*p_j2k).m_specific_param.m_decoder.m_last_tile_part = 1i32;
    return 0i32;
  }
  if l_num_parts != 0u32 {
    /* Number of tile-part header is provided by this tile-part header */
    l_num_parts = (l_num_parts as libc::c_uint).wrapping_add(
      (*p_j2k)
        .m_specific_param
        .m_decoder
        .m_nb_tile_parts_correction(),
    ) as OPJ_UINT32;
    /* Useful to manage the case of textGBR.jp2 file because two values of TNSot are allowed: the correct numbers of
     * tile-parts for that tile and zero (A.4.2 of 15444-1 : 2002). */
    if (*l_tcp).m_nb_tile_parts != 0 {
      if l_current_part >= (*l_tcp).m_nb_tile_parts {
        opj_event_msg(p_manager, 1i32,
                              b"In SOT marker, TPSot (%d) is not valid regards to the current number of tile-part (%d), giving up\n\x00"
                                  as *const u8 as *const libc::c_char,
                              l_current_part, (*l_tcp).m_nb_tile_parts);
        (*p_j2k).m_specific_param.m_decoder.m_last_tile_part = 1i32;
        return 0i32;
      }
    }
    if l_current_part >= l_num_parts {
      /* testcase 451.pdf.SIGSEGV.ce9.3723 */
      opj_event_msg(p_manager, 1i32,
                          b"In SOT marker, TPSot (%d) is not valid regards to the current number of tile-part (header) (%d), giving up\n\x00"
                              as *const u8 as *const libc::c_char,
                          l_current_part, l_num_parts);
      (*p_j2k).m_specific_param.m_decoder.m_last_tile_part = 1i32;
      return 0i32;
    }
    (*l_tcp).m_nb_tile_parts = l_num_parts
  }
  /* If know the number of tile part header we will check if we didn't read the last*/
  if (*l_tcp).m_nb_tile_parts != 0 {
    if (*l_tcp).m_nb_tile_parts == l_current_part.wrapping_add(1u32) {
      (*p_j2k)
        .m_specific_param
        .m_decoder
        .set_m_can_decode(1 as OPJ_BITFIELD)
      /* Process the last tile-part header*/
    }
  }
  if (*p_j2k).m_specific_param.m_decoder.m_last_tile_part == 0 {
    /* Keep the size of data to skip after this marker */
    (*p_j2k).m_specific_param.m_decoder.m_sot_length =
      l_tot_len.wrapping_sub(12u32)
  /* SOT_marker_size = 12 */
  } else {
    /* FIXME: need to be computed from the number of bytes remaining in the codestream */
    (*p_j2k).m_specific_param.m_decoder.m_sot_length = 0 as OPJ_UINT32
  }
  (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::TPH;
  /* Check if the current tile is outside the area we want decode or not corresponding to the tile index*/
  if (*p_j2k).m_specific_param.m_decoder.m_tile_ind_to_dec == -(1i32) {
    (*p_j2k).m_specific_param.m_decoder.set_m_skip_data(
      (l_tile_x < (*p_j2k).m_specific_param.m_decoder.m_start_tile_x
        || l_tile_x >= (*p_j2k).m_specific_param.m_decoder.m_end_tile_x
        || l_tile_y < (*p_j2k).m_specific_param.m_decoder.m_start_tile_y
        || l_tile_y >= (*p_j2k).m_specific_param.m_decoder.m_end_tile_y) as libc::c_int
        as OPJ_BITFIELD,
    )
  } else {
    assert!((*p_j2k).m_specific_param.m_decoder.m_tile_ind_to_dec >= 0i32);
    (*p_j2k).m_specific_param.m_decoder.set_m_skip_data(
      ((*p_j2k).m_current_tile_number
        != (*p_j2k).m_specific_param.m_decoder.m_tile_ind_to_dec as OPJ_UINT32) as libc::c_int
        as OPJ_BITFIELD,
    )
  }
  /* Index */
  if !(*p_j2k).cstr_index.is_null() {
    assert!(!(*(*p_j2k).cstr_index).tile_index.is_null());
    (*(*(*p_j2k).cstr_index)
      .tile_index
      .offset((*p_j2k).m_current_tile_number as isize))
    .tileno = (*p_j2k).m_current_tile_number;
    (*(*(*p_j2k).cstr_index)
      .tile_index
      .offset((*p_j2k).m_current_tile_number as isize))
    .current_tpsno = l_current_part;
    if l_num_parts != 0u32 {
      (*(*(*p_j2k).cstr_index)
        .tile_index
        .offset((*p_j2k).m_current_tile_number as isize))
      .nb_tps = l_num_parts;
      (*(*(*p_j2k).cstr_index)
        .tile_index
        .offset((*p_j2k).m_current_tile_number as isize))
      .current_nb_tps = l_num_parts;
      if (*(*(*p_j2k).cstr_index)
        .tile_index
        .offset((*p_j2k).m_current_tile_number as isize))
      .tp_index
      .is_null()
      {
        let ref mut fresh16 = (*(*(*p_j2k).cstr_index)
          .tile_index
          .offset((*p_j2k).m_current_tile_number as isize))
        .tp_index;
        *fresh16 = opj_calloc(
          l_num_parts as size_t,
          core::mem::size_of::<opj_tp_index_t>() as libc::c_ulong,
        ) as *mut opj_tp_index_t;
        if (*(*(*p_j2k).cstr_index)
          .tile_index
          .offset((*p_j2k).m_current_tile_number as isize))
        .tp_index
        .is_null()
        {
          opj_event_msg(
            p_manager,
            1i32,
            b"Not enough memory to read SOT marker. Tile index allocation failed\n\x00" as *const u8
              as *const libc::c_char,
          );
          return 0i32;
        }
      } else {
        let mut new_tp_index = opj_realloc(
          (*(*(*p_j2k).cstr_index)
            .tile_index
            .offset((*p_j2k).m_current_tile_number as isize))
          .tp_index as *mut libc::c_void,
          (l_num_parts as libc::c_ulong)
            .wrapping_mul(core::mem::size_of::<opj_tp_index_t>() as libc::c_ulong),
        ) as *mut opj_tp_index_t;
        if new_tp_index.is_null() {
          opj_free(
            (*(*(*p_j2k).cstr_index)
              .tile_index
              .offset((*p_j2k).m_current_tile_number as isize))
            .tp_index as *mut libc::c_void,
          );
          let ref mut fresh17 = (*(*(*p_j2k).cstr_index)
            .tile_index
            .offset((*p_j2k).m_current_tile_number as isize))
          .tp_index;
          *fresh17 = 0 as *mut opj_tp_index_t;
          opj_event_msg(
            p_manager,
            1i32,
            b"Not enough memory to read SOT marker. Tile index allocation failed\n\x00" as *const u8
              as *const libc::c_char,
          );
          return 0i32;
        }
        let ref mut fresh18 = (*(*(*p_j2k).cstr_index)
          .tile_index
          .offset((*p_j2k).m_current_tile_number as isize))
        .tp_index;
        *fresh18 = new_tp_index
      }
    } else {
      /*if (!p_j2k->cstr_index->tile_index[p_j2k->m_current_tile_number].tp_index)*/
      if (*(*(*p_j2k).cstr_index)
        .tile_index
        .offset((*p_j2k).m_current_tile_number as isize))
      .tp_index
      .is_null()
      {
        (*(*(*p_j2k).cstr_index)
          .tile_index
          .offset((*p_j2k).m_current_tile_number as isize))
        .current_nb_tps = 10 as OPJ_UINT32;
        let ref mut fresh19 = (*(*(*p_j2k).cstr_index)
          .tile_index
          .offset((*p_j2k).m_current_tile_number as isize))
        .tp_index;
        *fresh19 = opj_calloc(
          (*(*(*p_j2k).cstr_index)
            .tile_index
            .offset((*p_j2k).m_current_tile_number as isize))
          .current_nb_tps as size_t,
          core::mem::size_of::<opj_tp_index_t>() as libc::c_ulong,
        ) as *mut opj_tp_index_t;
        if (*(*(*p_j2k).cstr_index)
          .tile_index
          .offset((*p_j2k).m_current_tile_number as isize))
        .tp_index
        .is_null()
        {
          (*(*(*p_j2k).cstr_index)
            .tile_index
            .offset((*p_j2k).m_current_tile_number as isize))
          .current_nb_tps = 0 as OPJ_UINT32;
          opj_event_msg(
            p_manager,
            1i32,
            b"Not enough memory to read SOT marker. Tile index allocation failed\n\x00" as *const u8
              as *const libc::c_char,
          );
          return 0i32;
        }
      }
      if l_current_part
        >= (*(*(*p_j2k).cstr_index)
          .tile_index
          .offset((*p_j2k).m_current_tile_number as isize))
        .current_nb_tps
      {
        let mut new_tp_index_0 = 0 as *mut opj_tp_index_t;
        (*(*(*p_j2k).cstr_index)
          .tile_index
          .offset((*p_j2k).m_current_tile_number as isize))
        .current_nb_tps = l_current_part.wrapping_add(1u32);
        new_tp_index_0 = opj_realloc(
          (*(*(*p_j2k).cstr_index)
            .tile_index
            .offset((*p_j2k).m_current_tile_number as isize))
          .tp_index as *mut libc::c_void,
          ((*(*(*p_j2k).cstr_index)
            .tile_index
            .offset((*p_j2k).m_current_tile_number as isize))
          .current_nb_tps as libc::c_ulong)
            .wrapping_mul(core::mem::size_of::<opj_tp_index_t>() as libc::c_ulong),
        ) as *mut opj_tp_index_t;
        if new_tp_index_0.is_null() {
          opj_free(
            (*(*(*p_j2k).cstr_index)
              .tile_index
              .offset((*p_j2k).m_current_tile_number as isize))
            .tp_index as *mut libc::c_void,
          );
          let ref mut fresh20 = (*(*(*p_j2k).cstr_index)
            .tile_index
            .offset((*p_j2k).m_current_tile_number as isize))
          .tp_index;
          *fresh20 = 0 as *mut opj_tp_index_t;
          (*(*(*p_j2k).cstr_index)
            .tile_index
            .offset((*p_j2k).m_current_tile_number as isize))
          .current_nb_tps = 0 as OPJ_UINT32;
          opj_event_msg(
            p_manager,
            1i32,
            b"Not enough memory to read SOT marker. Tile index allocation failed\n\x00" as *const u8
              as *const libc::c_char,
          );
          return 0i32;
        }
        let ref mut fresh21 = (*(*(*p_j2k).cstr_index)
          .tile_index
          .offset((*p_j2k).m_current_tile_number as isize))
        .tp_index;
        *fresh21 = new_tp_index_0
      }
    }
  }
  /* FIXME move this onto a separate method to call before reading any SOT, remove part about main_end header, use a index struct inside p_j2k */
  /* if (p_j2k->cstr_info) {
  if (l_tcp->first) {
  if (tileno == 0) {
  p_j2k->cstr_info->main_head_end = p_stream_tell(p_stream) - 13;
  }

  p_j2k->cstr_info->tile[tileno].tileno = tileno;
  p_j2k->cstr_info->tile[tileno].start_pos = p_stream_tell(p_stream) - 12;
  p_j2k->cstr_info->tile[tileno].end_pos = p_j2k->cstr_info->tile[tileno].start_pos + totlen - 1;
  p_j2k->cstr_info->tile[tileno].num_tps = numparts;

  if (numparts) {
  p_j2k->cstr_info->tile[tileno].tp = (opj_tp_info_t *) opj_malloc(numparts * sizeof(opj_tp_info_t));
  }
  else {
  p_j2k->cstr_info->tile[tileno].tp = (opj_tp_info_t *) opj_malloc(10 * sizeof(opj_tp_info_t)); // Fixme (10)
  }
  }
  else {
  p_j2k->cstr_info->tile[tileno].end_pos += totlen;
  }

  p_j2k->cstr_info->tile[tileno].tp[partno].tp_start_pos = p_stream_tell(p_stream) - 12;
  p_j2k->cstr_info->tile[tileno].tp[partno].tp_end_pos =
  p_j2k->cstr_info->tile[tileno].tp[partno].tp_start_pos + totlen - 1;
  }*/
  return 1i32;
}
/* *
 * Write one or more PLT markers in the provided buffer
 */
unsafe extern "C" fn opj_j2k_write_plt_in_memory(
  mut _p_j2k: *mut opj_j2k_t,
  mut marker_info: *mut opj_tcd_marker_info_t,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_written: *mut OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut Zplt = 0 as OPJ_BYTE;
  let mut Lplt: OPJ_UINT16 = 0;
  let mut p_data_start = p_data;
  let mut p_data_Lplt = p_data.offset(2);
  let mut i: OPJ_UINT32 = 0;
  opj_write_bytes_LE(
    p_data,
    0xff58 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  p_data = p_data.offset(2);
  /* Reserve space for Lplt */
  p_data = p_data.offset(2);
  opj_write_bytes_LE(p_data, Zplt as OPJ_UINT32, 1 as OPJ_UINT32);
  p_data = p_data.offset(1);
  Lplt = 3 as OPJ_UINT16;
  i = 0 as OPJ_UINT32;
  while i < (*marker_info).packet_count {
    let mut var_bytes: [OPJ_BYTE; 5] = [0; 5];
    let mut var_bytes_size = 0 as OPJ_UINT8;
    let mut packet_size = *(*marker_info).p_packet_size.offset(i as isize);
    /* Packet size written in variable-length way, starting with LSB */
    var_bytes[var_bytes_size as usize] =
      (packet_size & 0x7fu32) as OPJ_BYTE;
    var_bytes_size = var_bytes_size.wrapping_add(1);
    packet_size >>= 7i32;
    while packet_size > 0u32 {
      var_bytes[var_bytes_size as usize] = (packet_size & 0x7fu32
        | 0x80u32)
        as OPJ_BYTE;
      var_bytes_size = var_bytes_size.wrapping_add(1);
      packet_size >>= 7i32
    }
    /* Check if that can fit in the current PLT marker. If not, finish */
    /* current one, and start a new one */
    if Lplt as libc::c_int + var_bytes_size as libc::c_int > 65535i32 {
      if Zplt as libc::c_int == 255i32 {
        opj_event_msg(
          p_manager,
          1i32,
          b"More than 255 PLT markers would be needed for current tile-part !\n\x00" as *const u8
            as *const libc::c_char,
        );
        return 0i32;
      }
      /* Patch Lplt */
      opj_write_bytes_LE(
        p_data_Lplt,
        Lplt as OPJ_UINT32,
        2 as OPJ_UINT32,
      );
      /* Start new segment */
      opj_write_bytes_LE(
        p_data,
        0xff58 as OPJ_UINT32,
        2 as OPJ_UINT32,
      );
      p_data = p_data.offset(2);
      /* Reserve space for Lplt */
      p_data_Lplt = p_data;
      p_data = p_data.offset(2);
      Zplt = Zplt.wrapping_add(1);
      opj_write_bytes_LE(p_data, Zplt as OPJ_UINT32, 1 as OPJ_UINT32);
      p_data = p_data.offset(1);
      Lplt = 3 as OPJ_UINT16
    }
    Lplt = (Lplt as libc::c_int + var_bytes_size as libc::c_int) as OPJ_UINT16;
    /* Serialize variable-length packet size, starting with MSB */
    while var_bytes_size as libc::c_int > 0i32 {
      opj_write_bytes_LE(
        p_data,
        var_bytes[(var_bytes_size as libc::c_int - 1i32) as usize] as OPJ_UINT32,
        1 as OPJ_UINT32,
      );
      p_data = p_data.offset(1);
      var_bytes_size = var_bytes_size.wrapping_sub(1)
    }
    i = i.wrapping_add(1)
  }
  *p_data_written = p_data.offset_from(p_data_start) as OPJ_UINT32;
  /* Patch Lplt */
  opj_write_bytes_LE(
    p_data_Lplt,
    Lplt as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  return 1i32;
}
/* *
 * Writes the SOD marker (Start of data)
 *
 * This also writes optional PLT markers (before SOD)
 *
 * @param       p_j2k               J2K codec.
 * @param       p_tile_coder        FIXME DOC
 * @param       p_data              FIXME DOC
 * @param       p_data_written      FIXME DOC
 * @param       total_data_size   FIXME DOC
 * @param       p_stream            the stream to write data to.
 * @param       p_manager           the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_sod(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_coder: *mut opj_tcd_t,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_written: *mut OPJ_UINT32,
  mut total_data_size: OPJ_UINT32,
  mut p_stream: *const opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_cstr_info = 0 as *mut opj_codestream_info_t;
  let mut l_remaining_data: OPJ_UINT32 = 0;
  let mut marker_info = 0 as *mut opj_tcd_marker_info_t;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  if total_data_size < 4u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough bytes in output buffer to write SOD marker\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 0i32;
  }
  opj_write_bytes_LE(
    p_data,
    0xff93 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  /* make room for the EOF marker */
  l_remaining_data = total_data_size.wrapping_sub(4u32);
  /* update tile coder */
  (*p_tile_coder).tp_num = (*p_j2k)
    .m_specific_param
    .m_encoder
    .m_current_poc_tile_part_number;
  (*p_tile_coder).cur_tp_num = (*p_j2k)
    .m_specific_param
    .m_encoder
    .m_current_tile_part_number;
  /* INDEX >> */
  /* TODO mergeV2: check this part which use cstr_info */
  /*l_cstr_info = p_j2k->cstr_info;
  if (l_cstr_info) {
          if (!p_j2k->m_specific_param.m_encoder.m_current_tile_part_number ) {
                  //TODO cstr_info->tile[p_j2k->m_current_tile_number].end_header = p_stream_tell(p_stream) + p_j2k->pos_correction - 1;
                  l_cstr_info->tile[p_j2k->m_current_tile_number].tileno = p_j2k->m_current_tile_number;
          }
          else {*/
  /*
  TODO
  if
          (cstr_info->tile[p_j2k->m_current_tile_number].packet[cstr_info->packno - 1].end_pos < p_stream_tell(p_stream))
  {
          cstr_info->tile[p_j2k->m_current_tile_number].packet[cstr_info->packno].start_pos = p_stream_tell(p_stream);
  }*/
  /*}*/
  /* UniPG>> */
  /* USE_JPWL */
  /* <<UniPG */
  /*}*/
  /* << INDEX */
  if (*p_j2k)
    .m_specific_param
    .m_encoder
    .m_current_tile_part_number
    == 0u32
  {
    (*(*(*p_tile_coder).tcd_image).tiles).packno = 0 as OPJ_UINT32
  }
  *p_data_written = 0 as OPJ_UINT32;
  if (*p_j2k).m_specific_param.m_encoder.m_PLT != 0 {
    marker_info = opj_tcd_marker_info_create((*p_j2k).m_specific_param.m_encoder.m_PLT);
    if marker_info.is_null() {
      opj_event_msg(
        p_manager,
        1i32,
        b"Cannot encode tile: opj_tcd_marker_info_create() failed\n\x00" as *const u8
          as *const libc::c_char,
      );
      return 0i32;
    }
  }
  if l_remaining_data < (*p_j2k).m_specific_param.m_encoder.m_reserved_bytes_for_PLT {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough bytes in output buffer to write SOD marker\n\x00" as *const u8
        as *const libc::c_char,
    );
    opj_tcd_marker_info_destroy(marker_info);
    return 0i32;
  }
  l_remaining_data = (l_remaining_data as libc::c_uint)
    .wrapping_sub((*p_j2k).m_specific_param.m_encoder.m_reserved_bytes_for_PLT)
    as OPJ_UINT32;
  if opj_tcd_encode_tile(
    p_tile_coder,
    (*p_j2k).m_current_tile_number,
    p_data.offset(2),
    p_data_written,
    l_remaining_data,
    l_cstr_info,
    marker_info,
    p_manager,
  ) == 0
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Cannot encode tile\n\x00" as *const u8 as *const libc::c_char,
    );
    opj_tcd_marker_info_destroy(marker_info);
    return 0i32;
  }
  /* For SOD */
  *p_data_written = (*p_data_written as libc::c_uint).wrapping_add(2u32)
    as OPJ_UINT32;
  if (*p_j2k).m_specific_param.m_encoder.m_PLT != 0 {
    let mut l_data_written_PLT = 0 as OPJ_UINT32;
    let mut p_PLT_buffer =
      opj_malloc((*p_j2k).m_specific_param.m_encoder.m_reserved_bytes_for_PLT as size_t)
        as *mut OPJ_BYTE;
    if p_PLT_buffer.is_null() {
      opj_event_msg(
        p_manager,
        1i32,
        b"Cannot allocate memory\n\x00" as *const u8 as *const libc::c_char,
      );
      opj_tcd_marker_info_destroy(marker_info);
      return 0i32;
    }
    if opj_j2k_write_plt_in_memory(
      p_j2k,
      marker_info,
      p_PLT_buffer,
      &mut l_data_written_PLT,
      p_manager,
    ) == 0
    {
      opj_tcd_marker_info_destroy(marker_info);
      opj_free(p_PLT_buffer as *mut libc::c_void);
      return 0i32;
    }
    assert!(l_data_written_PLT <= (*p_j2k).m_specific_param.m_encoder.m_reserved_bytes_for_PLT);
    /* Move PLT marker(s) before SOD */
    memmove(
      p_data.offset(l_data_written_PLT as isize) as *mut libc::c_void,
      p_data as *const libc::c_void,
      *p_data_written as libc::c_ulong,
    );
    memcpy(
      p_data as *mut libc::c_void,
      p_PLT_buffer as *const libc::c_void,
      l_data_written_PLT as libc::c_ulong,
    );
    opj_free(p_PLT_buffer as *mut libc::c_void);
    *p_data_written =
      (*p_data_written as libc::c_uint).wrapping_add(l_data_written_PLT) as OPJ_UINT32
  }
  opj_tcd_marker_info_destroy(marker_info);
  return 1i32;
}
/* *
 * Reads a SOD marker (Start Of Data)
 *
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_stream                FIXME DOC
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_sod(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_current_read_size: OPJ_SIZE_T = 0;
  let mut l_cstr_index = 0 as *mut opj_codestream_index_t;
  let mut l_current_data = 0 as *mut *mut OPJ_BYTE;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tile_len = 0 as *mut OPJ_UINT32;
  let mut l_sot_length_pb_detected = 0i32;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  l_tcp = &mut *(*p_j2k)
    .m_cp
    .tcps
    .offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t;
  if (*p_j2k).m_specific_param.m_decoder.m_last_tile_part != 0 {
    /* opj_stream_get_number_byte_left returns OPJ_OFF_T
    // but we are in the last tile part,
    // so its result will fit on OPJ_UINT32 unless we find
    // a file with a single tile part of more than 4 GB...*/
    (*p_j2k).m_specific_param.m_decoder.m_sot_length =
      (opj_stream_get_number_byte_left(p_stream) - 2i64) as OPJ_UINT32
  } else if (*p_j2k).m_specific_param.m_decoder.m_sot_length >= 2u32 {
    (*p_j2k).m_specific_param.m_decoder.m_sot_length =
      ((*p_j2k).m_specific_param.m_decoder.m_sot_length as libc::c_uint)
        .wrapping_sub(2u32) as OPJ_UINT32
  }
  l_current_data = &mut (*l_tcp).m_data;
  l_tile_len = &mut (*l_tcp).m_data_size;
  /* Check to avoid pass the limit of OPJ_UINT32 */
  /* Patch to support new PHR data */
  if (*p_j2k).m_specific_param.m_decoder.m_sot_length != 0 {
    /* If we are here, we'll try to read the data after allocation */
    /* Check enough bytes left in stream before allocation */
    if (*p_j2k).m_specific_param.m_decoder.m_sot_length as OPJ_OFF_T
      > opj_stream_get_number_byte_left(p_stream)
    {
      if (*p_j2k).m_cp.strict != 0 {
        opj_event_msg(
          p_manager,
          1i32,
          b"Tile part length size inconsistent with stream length\n\x00" as *const u8
            as *const libc::c_char,
        );
        return 0i32;
      } else {
        opj_event_msg(
          p_manager,
          2i32,
          b"Tile part length size inconsistent with stream length\n\x00" as *const u8
            as *const libc::c_char,
        );
      }
    }
    if (*p_j2k).m_specific_param.m_decoder.m_sot_length
      > (2147483647u32)
        .wrapping_mul(2u32)
        .wrapping_add(1u32)
        .wrapping_sub(2u32)
    {
      opj_event_msg(p_manager, 1i32,
                          b"p_j2k->m_specific_param.m_decoder.m_sot_length > UINT_MAX - OPJ_COMMON_CBLK_DATA_EXTRA\x00"
                              as *const u8 as *const libc::c_char);
      return 0i32;
    }
    /* Add a margin of OPJ_COMMON_CBLK_DATA_EXTRA to the allocation we */
    /* do so that opj_mqc_init_dec_common() can safely add a synthetic */
    /* 0xFFFF marker. */
    if (*l_current_data).is_null() {
      /* LH: oddly enough, in this path, l_tile_len!=0.
       * TODO: If this was consistent, we could simplify the code to only use realloc(), as realloc(0,...) default to malloc(0,...).
       */
      *l_current_data = opj_malloc(
        (*p_j2k)
          .m_specific_param
          .m_decoder
          .m_sot_length
          .wrapping_add(2u32) as size_t,
      ) as *mut OPJ_BYTE
    } else {
      let mut l_new_current_data = 0 as *mut OPJ_BYTE;
      if *l_tile_len
        > (2147483647u32)
          .wrapping_mul(2u32)
          .wrapping_add(1u32)
          .wrapping_sub(2u32)
          .wrapping_sub((*p_j2k).m_specific_param.m_decoder.m_sot_length)
      {
        opj_event_msg(p_manager, 1i32,
                              b"*l_tile_len > UINT_MAX - OPJ_COMMON_CBLK_DATA_EXTRA - p_j2k->m_specific_param.m_decoder.m_sot_length\x00"
                                  as *const u8 as *const libc::c_char);
        return 0i32;
      }
      l_new_current_data = opj_realloc(
        *l_current_data as *mut libc::c_void,
        (*l_tile_len)
          .wrapping_add((*p_j2k).m_specific_param.m_decoder.m_sot_length)
          .wrapping_add(2u32) as size_t,
      ) as *mut OPJ_BYTE;
      if l_new_current_data.is_null() {
        opj_free(*l_current_data as *mut libc::c_void);
        /*nothing more is done as l_current_data will be set to null, and just
        afterward we enter in the error path
        and the actual tile_len is updated (committed) at the end of the
        function. */
      }
      *l_current_data = l_new_current_data
    }
    if (*l_current_data).is_null() {
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to decode tile\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
  } else {
    l_sot_length_pb_detected = 1i32
  }
  /* Index */
  l_cstr_index = (*p_j2k).cstr_index;
  if !l_cstr_index.is_null() {
    let mut l_current_pos = opj_stream_tell(p_stream) - 2i64;
    let mut l_current_tile_part = (*(*l_cstr_index)
      .tile_index
      .offset((*p_j2k).m_current_tile_number as isize))
    .current_tpsno;
    (*(*(*l_cstr_index)
      .tile_index
      .offset((*p_j2k).m_current_tile_number as isize))
    .tp_index
    .offset(l_current_tile_part as isize))
    .end_header = l_current_pos;
    (*(*(*l_cstr_index)
      .tile_index
      .offset((*p_j2k).m_current_tile_number as isize))
    .tp_index
    .offset(l_current_tile_part as isize))
    .end_pos = l_current_pos
      + (*p_j2k).m_specific_param.m_decoder.m_sot_length as libc::c_long
      + 2i64;
    if 0i32
      == opj_j2k_add_tlmarker(
        (*p_j2k).m_current_tile_number,
        l_cstr_index,
        0xff93 as OPJ_UINT32,
        l_current_pos,
        (*p_j2k)
          .m_specific_param
          .m_decoder
          .m_sot_length
          .wrapping_add(2u32),
      )
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to add tl marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    /*l_cstr_index->packno = 0;*/
  }
  /* Patch to support new PHR data */
  if l_sot_length_pb_detected == 0 {
    l_current_read_size = opj_stream_read_data(
      p_stream,
      (*l_current_data).offset(*l_tile_len as isize),
      (*p_j2k).m_specific_param.m_decoder.m_sot_length as OPJ_SIZE_T,
      p_manager,
    )
  } else {
    l_current_read_size = 0 as OPJ_SIZE_T
  }
  if l_current_read_size != (*p_j2k).m_specific_param.m_decoder.m_sot_length as libc::c_ulong {
    (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::NEOC
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::TPHSOT
  }
  *l_tile_len = (*l_tile_len as libc::c_uint).wrapping_add(l_current_read_size as OPJ_UINT32)
    as OPJ_UINT32;
  return 1i32;
}
/* *
 * Writes the RGN marker (Region Of Interest)
 *
 * @param       p_tile_no               the tile to output
 * @param       p_comp_no               the component to output
 * @param       nb_comps                the number of components
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_rgn(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_no: OPJ_UINT32,
  mut p_comp_no: OPJ_UINT32,
  mut nb_comps: OPJ_UINT32,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  let mut l_rgn_size: OPJ_UINT32 = 0;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  let mut l_comp_room: OPJ_UINT32 = 0;
  /* preconditions */
  /* Lrgn */
  /* Srgn */
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = &mut *(*l_cp).tcps.offset(p_tile_no as isize) as *mut opj_tcp_t;
  l_tccp = &mut *(*l_tcp).tccps.offset(p_comp_no as isize) as *mut opj_tccp_t;
  if nb_comps <= 256u32 {
    l_comp_room = 1 as OPJ_UINT32
  } else {
    l_comp_room = 2 as OPJ_UINT32
  }
  l_rgn_size = (6u32).wrapping_add(l_comp_room);
  l_current_data = (*p_j2k).m_specific_param.m_encoder.m_header_tile_data;
  opj_write_bytes_LE(
    l_current_data,
    0xff5e as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    l_rgn_size.wrapping_sub(2u32),
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(l_current_data, p_comp_no, l_comp_room);
  l_current_data = l_current_data.offset(l_comp_room as isize);
  opj_write_bytes_LE(
    l_current_data,
    0 as OPJ_UINT32,
    1 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(1);
  opj_write_bytes_LE(
    l_current_data,
    (*l_tccp).roishift as OPJ_UINT32,
    1 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(1);
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_rgn_size as OPJ_SIZE_T,
    p_manager,
  ) != l_rgn_size as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Writes the EOC marker (End of Codestream)
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_eoc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  opj_write_bytes_LE(
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    0xffd9 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  /* UniPG>> */
  /* USE_JPWL */
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    2 as OPJ_SIZE_T,
    p_manager,
  ) != 2u64
  {
    return 0i32;
  }
  if opj_stream_flush(p_stream, p_manager) == 0 {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Reads a RGN marker (Region Of Interest)
 *
 * @param       p_header_data   the data contained in the POC box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the POC marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a RGN marker (Region Of Interest)
 *
 * @param       p_header_data   the data contained in the POC box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the POC marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_rgn(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_nb_comp: OPJ_UINT32 = 0;
  let mut l_image = 0 as *mut opj_image_t;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_comp_room: OPJ_UINT32 = 0;
  let mut l_comp_no: OPJ_UINT32 = 0;
  let mut l_roi_sty: OPJ_UINT32 = 0;
  /* preconditions*/
  /* Srgn */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_image = (*p_j2k).m_private_image;
  l_nb_comp = (*l_image).numcomps;
  if l_nb_comp <= 256u32 {
    l_comp_room = 1 as OPJ_UINT32
  } else {
    l_comp_room = 2 as OPJ_UINT32
  }
  if p_header_size != (2u32).wrapping_add(l_comp_room) {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading RGN marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = if (*p_j2k).m_specific_param.m_decoder.m_state
    == J2KState::TPH
  {
    &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_default_tcp
  };
  opj_read_bytes_LE(p_header_data, &mut l_comp_no, l_comp_room);
  p_header_data = p_header_data.offset(l_comp_room as isize);
  opj_read_bytes_LE(
    p_header_data,
    &mut l_roi_sty,
    1 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(1);
  /* USE_JPWL */
  /* testcase 3635.pdf.asan.77.2930 */
  if l_comp_no >= l_nb_comp {
    opj_event_msg(
      p_manager,
      1i32,
      b"bad component number in RGN (%d when there are only %d)\n\x00" as *const u8
        as *const libc::c_char,
      l_comp_no,
      l_nb_comp,
    ); /* SPrgn */
    return 0i32;
  }
  opj_read_bytes_LE(
    p_header_data,
    &mut (*(*l_tcp).tccps.offset(l_comp_no as isize)).roishift as *mut OPJ_INT32 as *mut OPJ_UINT32,
    1 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(1);
  return 1i32;
}
unsafe extern "C" fn opj_j2k_get_tp_stride(mut p_tcp: *mut opj_tcp_t) -> OPJ_FLOAT32 {
  return (*p_tcp)
    .m_nb_tile_parts
    .wrapping_sub(1u32)
    .wrapping_mul(14u32) as OPJ_FLOAT32;
}
unsafe extern "C" fn opj_j2k_get_default_stride(mut _p_tcp: *mut opj_tcp_t) -> OPJ_FLOAT32 {
  return 0 as OPJ_FLOAT32;
}
/* *
 * Updates the rates of the tcp.
 *
 * @param       p_stream                                the stream to write data to.
 * @param       p_j2k                           J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_update_rates(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_image = 0 as *mut opj_image_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_img_comp = 0 as *mut opj_image_comp_t;
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut k: OPJ_UINT32 = 0;
  let mut l_x0: OPJ_INT32 = 0;
  let mut l_y0: OPJ_INT32 = 0;
  let mut l_x1: OPJ_INT32 = 0;
  let mut l_y1: OPJ_INT32 = 0;
  let mut l_rates = 0 as *mut OPJ_FLOAT32;
  let mut l_sot_remove: OPJ_FLOAT32 = 0.;
  let mut l_bits_empty: OPJ_UINT32 = 0;
  let mut l_size_pixel: OPJ_UINT32 = 0;
  let mut l_tile_size = 0 as OPJ_UINT64;
  let mut l_last_res: OPJ_UINT32 = 0;
  let mut l_tp_stride_func: Option<unsafe extern "C" fn(_: *mut opj_tcp_t) -> OPJ_FLOAT32> = None;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_image = (*p_j2k).m_private_image;
  l_tcp = (*l_cp).tcps;
  l_bits_empty = (8u32)
    .wrapping_mul((*(*l_image).comps).dx)
    .wrapping_mul((*(*l_image).comps).dy);
  l_size_pixel = (*l_image).numcomps.wrapping_mul((*(*l_image).comps).prec);
  l_sot_remove =
    opj_stream_tell(p_stream) as OPJ_FLOAT32 / (*l_cp).th.wrapping_mul((*l_cp).tw) as OPJ_FLOAT32;
  if (*l_cp).m_specific_param.m_enc.m_tp_on() != 0 {
    l_tp_stride_func =
      Some(opj_j2k_get_tp_stride as unsafe extern "C" fn(_: *mut opj_tcp_t) -> OPJ_FLOAT32)
  } else {
    l_tp_stride_func =
      Some(opj_j2k_get_default_stride as unsafe extern "C" fn(_: *mut opj_tcp_t) -> OPJ_FLOAT32)
  }
  i = 0 as OPJ_UINT32;
  while i < (*l_cp).th {
    j = 0 as OPJ_UINT32;
    while j < (*l_cp).tw {
      let mut l_offset = Some(l_tp_stride_func.expect("non-null function pointer"))
        .expect("non-null function pointer")(l_tcp)
        / (*l_tcp).numlayers as OPJ_FLOAT32;
      /* 4 borders of the tile rescale on the image if necessary */
      l_x0 = opj_int_max(
        (*l_cp).tx0.wrapping_add(j.wrapping_mul((*l_cp).tdx)) as OPJ_INT32,
        (*l_image).x0 as OPJ_INT32,
      );
      l_y0 = opj_int_max(
        (*l_cp).ty0.wrapping_add(i.wrapping_mul((*l_cp).tdy)) as OPJ_INT32,
        (*l_image).y0 as OPJ_INT32,
      );
      l_x1 = opj_int_min(
        (*l_cp).tx0.wrapping_add(
          j.wrapping_add(1u32)
            .wrapping_mul((*l_cp).tdx),
        ) as OPJ_INT32,
        (*l_image).x1 as OPJ_INT32,
      );
      l_y1 = opj_int_min(
        (*l_cp).ty0.wrapping_add(
          i.wrapping_add(1u32)
            .wrapping_mul((*l_cp).tdy),
        ) as OPJ_INT32,
        (*l_image).y1 as OPJ_INT32,
      );
      l_rates = (*l_tcp).rates.as_mut_ptr();
      /* Modification of the RATE >> */
      k = 0 as OPJ_UINT32;
      while k < (*l_tcp).numlayers {
        if *l_rates > 0.0f32 {
          *l_rates = (l_size_pixel as OPJ_FLOAT64
            * (l_x1 - l_x0) as OPJ_UINT32 as libc::c_double
            * (l_y1 - l_y0) as OPJ_UINT32 as libc::c_double
            / (*l_rates * l_bits_empty as OPJ_FLOAT32) as libc::c_double)
            as OPJ_FLOAT32
            - l_offset
        }
        l_rates = l_rates.offset(1);
        k = k.wrapping_add(1)
      }
      l_tcp = l_tcp.offset(1);
      j = j.wrapping_add(1)
    }
    i = i.wrapping_add(1)
  }
  l_tcp = (*l_cp).tcps;
  i = 0 as OPJ_UINT32;
  while i < (*l_cp).th {
    j = 0 as OPJ_UINT32;
    while j < (*l_cp).tw {
      l_rates = (*l_tcp).rates.as_mut_ptr();
      if *l_rates > 0.0f32 {
        *l_rates -= l_sot_remove;
        if *l_rates < 30.0f32 {
          *l_rates = 30.0f32
        }
      }
      l_rates = l_rates.offset(1);
      l_last_res = (*l_tcp)
        .numlayers
        .wrapping_sub(1u32);
      k = 1 as OPJ_UINT32;
      while k < l_last_res {
        if *l_rates > 0.0f32 {
          *l_rates -= l_sot_remove;
          if *l_rates < *l_rates.offset(-1) + 10.0f32 {
            *l_rates = *l_rates.offset(-1) + 20.0f32
          }
        }
        l_rates = l_rates.offset(1);
        k = k.wrapping_add(1)
      }
      if *l_rates > 0.0f32 {
        *l_rates -= l_sot_remove + 2.0f32;
        if *l_rates < *l_rates.offset(-1) + 10.0f32 {
          *l_rates = *l_rates.offset(-1) + 20.0f32
        }
      }
      l_tcp = l_tcp.offset(1);
      j = j.wrapping_add(1)
    }
    i = i.wrapping_add(1)
  }
  l_img_comp = (*l_image).comps;
  l_tile_size = 0 as OPJ_UINT64;
  i = 0 as OPJ_UINT32;
  while i < (*l_image).numcomps {
    l_tile_size = (l_tile_size as libc::c_ulong).wrapping_add(
      (opj_uint_ceildiv((*l_cp).tdx, (*l_img_comp).dx) as OPJ_UINT64)
        .wrapping_mul(opj_uint_ceildiv((*l_cp).tdy, (*l_img_comp).dy) as libc::c_ulong)
        .wrapping_mul((*l_img_comp).prec as libc::c_ulong),
    ) as OPJ_UINT64 as OPJ_UINT64;
    l_img_comp = l_img_comp.offset(1);
    i = i.wrapping_add(1)
  }
  /* TODO: where does this magic value come from ? */
  /* This used to be 1.3 / 8, but with random data and very small code */
  /* block sizes, this is not enough. For example with */
  /* bin/test_tile_encoder 1 256 256 32 32 8 0 reversible_with_precinct.j2k 4 4 3 0 0 1 16 16 */
  /* TODO revise this to take into account the overhead linked to the */
  /* number of packets and number of code blocks in packets */
  l_tile_size =
    (l_tile_size as libc::c_double * 1.4f64 / 8 as libc::c_double) as OPJ_UINT64;
  /* Arbitrary amount to make the following work: */
  /* bin/test_tile_encoder 1 256 256 17 16 8 0 reversible_no_precinct.j2k 4 4 3 0 0 1 */
  l_tile_size = (l_tile_size as libc::c_ulong).wrapping_add(500u64)
    as OPJ_UINT64 as OPJ_UINT64;
  l_tile_size = (l_tile_size as libc::c_ulong)
    .wrapping_add(opj_j2k_get_specific_header_sizes(p_j2k) as libc::c_ulong)
    as OPJ_UINT64 as OPJ_UINT64;
  if l_tile_size
    > (2147483647u32)
      .wrapping_mul(2u32)
      .wrapping_add(1u32) as libc::c_ulong
  {
    l_tile_size = (2147483647u32)
      .wrapping_mul(2u32)
      .wrapping_add(1u32) as OPJ_UINT64
  }
  (*p_j2k).m_specific_param.m_encoder.m_encoded_tile_size = l_tile_size as OPJ_UINT32;
  (*p_j2k).m_specific_param.m_encoder.m_encoded_tile_data =
    opj_malloc((*p_j2k).m_specific_param.m_encoder.m_encoded_tile_size as size_t) as *mut OPJ_BYTE;
  if (*p_j2k)
    .m_specific_param
    .m_encoder
    .m_encoded_tile_data
    .is_null()
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to allocate m_encoded_tile_data. %u MB required\n\x00" as *const u8
        as *const libc::c_char,
      l_tile_size
        .wrapping_div(1024u64)
        .wrapping_div(1024u64) as OPJ_UINT32,
    );
    return 0i32;
  }
  if (*p_j2k).m_specific_param.m_encoder.m_TLM != 0 {
    (*p_j2k).m_specific_param.m_encoder.m_tlm_sot_offsets_buffer = opj_malloc(
      (6u32)
        .wrapping_mul((*p_j2k).m_specific_param.m_encoder.m_total_tile_parts) as size_t,
    ) as *mut OPJ_BYTE;
    if (*p_j2k)
      .m_specific_param
      .m_encoder
      .m_tlm_sot_offsets_buffer
      .is_null()
    {
      return 0i32;
    }
    (*p_j2k)
      .m_specific_param
      .m_encoder
      .m_tlm_sot_offsets_current = (*p_j2k).m_specific_param.m_encoder.m_tlm_sot_offsets_buffer
  }
  return 1i32;
}
/* *
 * Gets the offset of the header.
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_get_end_header(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  (*(*p_j2k).cstr_index).main_head_end = opj_stream_tell(p_stream);
  return 1i32;
}
/* *
 * Writes the CBD-MCT-MCC-MCO markers (Multi components transform)
 *
 * @param       p_stream                        the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager       the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_mct_data_group(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_mcc_record = 0 as *mut opj_simple_mcc_decorrelation_data_t;
  let mut l_mct_record = 0 as *mut opj_mct_data_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_stream.is_null());
  assert!(!p_manager.is_null());
  if opj_j2k_write_cbd(p_j2k, p_stream, p_manager) == 0 {
    return 0i32;
  }
  l_tcp = &mut *(*p_j2k)
    .m_cp
    .tcps
    .offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t;
  l_mct_record = (*l_tcp).m_mct_records;
  i = 0 as OPJ_UINT32;
  while i < (*l_tcp).m_nb_mct_records {
    if opj_j2k_write_mct_record(p_j2k, l_mct_record, p_stream, p_manager) == 0 {
      return 0i32;
    }
    l_mct_record = l_mct_record.offset(1);
    i = i.wrapping_add(1)
  }
  l_mcc_record = (*l_tcp).m_mcc_records;
  i = 0 as OPJ_UINT32;
  while i < (*l_tcp).m_nb_mcc_records {
    if opj_j2k_write_mcc_record(p_j2k, l_mcc_record, p_stream, p_manager) == 0 {
      return 0i32;
    }
    l_mcc_record = l_mcc_record.offset(1);
    i = i.wrapping_add(1)
  }
  if opj_j2k_write_mco(p_j2k, p_stream, p_manager) == 0 {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Writes COC marker for each component.
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_all_coc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut compno: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  compno = 1 as OPJ_UINT32;
  while compno < (*(*p_j2k).m_private_image).numcomps {
    /* cod is first component of first tile */
    if opj_j2k_compare_coc(p_j2k, 0 as OPJ_UINT32, compno) == 0 {
      if opj_j2k_write_coc(p_j2k, compno, p_stream, p_manager) == 0 {
        return 0i32;
      }
    }
    compno = compno.wrapping_add(1)
  }
  return 1i32;
}
/* *
 * Writes QCC marker for each component.
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_all_qcc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut compno: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  compno = 1 as OPJ_UINT32;
  while compno < (*(*p_j2k).m_private_image).numcomps {
    /* qcd is first component of first tile */
    if opj_j2k_compare_qcc(p_j2k, 0 as OPJ_UINT32, compno) == 0 {
      if opj_j2k_write_qcc(p_j2k, compno, p_stream, p_manager) == 0 {
        return 0i32;
      }
    }
    compno = compno.wrapping_add(1)
  }
  return 1i32;
}
/* *
 * Writes regions of interests.
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_regions(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut compno: OPJ_UINT32 = 0;
  let mut l_tccp = 0 as *const opj_tccp_t;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  l_tccp = (*(*p_j2k).m_cp.tcps).tccps;
  compno = 0 as OPJ_UINT32;
  while compno < (*(*p_j2k).m_private_image).numcomps {
    if (*l_tccp).roishift != 0 {
      if opj_j2k_write_rgn(
        p_j2k,
        0 as OPJ_UINT32,
        compno,
        (*(*p_j2k).m_private_image).numcomps,
        p_stream,
        p_manager,
      ) == 0
      {
        return 0i32;
      }
    }
    l_tccp = l_tccp.offset(1);
    compno = compno.wrapping_add(1)
  }
  return 1i32;
}
/* *
 * Writes EPC ????
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_epc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut l_cstr_index = 0 as *mut opj_codestream_index_t;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  l_cstr_index = (*p_j2k).cstr_index;
  if !l_cstr_index.is_null() {
    (*l_cstr_index).codestream_size = opj_stream_tell(p_stream) as OPJ_UINT64;
    /* <<UniPG */
    (*l_cstr_index).codestream_size = ((*l_cstr_index).codestream_size as libc::c_ulong)
      .wrapping_sub((*l_cstr_index).main_head_start as OPJ_UINT64)
      as OPJ_UINT64 as OPJ_UINT64
  }
  /* UniPG>> */
  /* The following adjustment is done to adjust the codestream size */
  /* if SOD is not at 0 in the buffer. Useful in case of JP2, where */
  /* the first bunch of bytes is not in the codestream              */
  /* USE_JPWL */
  return 1i32;
}
/* *
 * Reads an unknown marker
 *
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_stream                the stream object to read from.
 * @param       output_marker           FIXME DOC
 * @param       p_manager               the user event manager.
 *
 * @return      true                    if the marker could be deduced.
*/
unsafe extern "C" fn opj_j2k_read_unk(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut output_marker: *mut OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_unknown_marker: OPJ_UINT32 = 0;
  let mut l_marker_handler = 0 as *const opj_dec_memory_marker_handler_t;
  let mut l_size_unk = 2 as OPJ_UINT32;
  /* preconditions*/

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  opj_event_msg(
    p_manager,
    2i32,
    b"Unknown marker\n\x00" as *const u8 as *const libc::c_char,
  );
  loop {
    /* Try to read 2 bytes (the next marker ID) from stream and copy them into the buffer*/
    if opj_stream_read_data(
      p_stream,
      (*p_j2k).m_specific_param.m_decoder.m_header_data,
      2 as OPJ_SIZE_T,
      p_manager,
    ) != 2u64
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    /* read 2 bytes as the new marker ID*/
    opj_read_bytes_LE(
      (*p_j2k).m_specific_param.m_decoder.m_header_data,
      &mut l_unknown_marker,
      2 as OPJ_UINT32,
    );
    if l_unknown_marker < 0xff00u32 {
      continue;
    }
    /* Get the marker handler from the marker ID*/
    l_marker_handler = opj_j2k_get_marker_handler(l_unknown_marker);
    if (*p_j2k).m_specific_param.m_decoder.m_state & (*l_marker_handler).states == J2KState::NONE {
      opj_event_msg(
        p_manager,
        1i32,
        b"Marker is not compliant with its position\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    } else if (*l_marker_handler).id != 0u32 {
      /* Add the marker to the codestream index*/
      if (*l_marker_handler).id != 0xff90u32 {
        let mut res = opj_j2k_add_mhmarker(
          (*p_j2k).cstr_index,
          0 as OPJ_UINT32,
          (opj_stream_tell(p_stream) as OPJ_UINT32).wrapping_sub(l_size_unk) as OPJ_OFF_T,
          l_size_unk,
        );
        if res == 0i32 {
          opj_event_msg(
            p_manager,
            1i32,
            b"Not enough memory to add mh marker\n\x00" as *const u8 as *const libc::c_char,
          );
          return 0i32;
        }
      }
      break;
    /* next marker is known and well located */
    } else {
      l_size_unk = (l_size_unk as libc::c_uint).wrapping_add(2u32)
        as OPJ_UINT32
    }
  }
  *output_marker = (*l_marker_handler).id;
  return 1i32;
}
/* *
 * Writes the MCT marker (Multiple Component Transform)
 *
 * @param       p_j2k           J2K codec.
 * @param       p_mct_record    FIXME DOC
 * @param       p_stream        the stream to write data to.
 * @param       p_manager       the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_mct_record(
  mut p_j2k: *mut opj_j2k_t,
  mut p_mct_record: *mut opj_mct_data_t,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut l_mct_size: OPJ_UINT32 = 0;
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  let mut l_tmp: OPJ_UINT32 = 0;
  /* preconditions */
  /* Lmct */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  l_mct_size = (10u32).wrapping_add((*p_mct_record).m_data_size);
  if l_mct_size > (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size {
    let mut new_header_tile_data = opj_realloc(
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
      l_mct_size as size_t,
    ) as *mut OPJ_BYTE;
    if new_header_tile_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to write MCT marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = new_header_tile_data;
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = l_mct_size
  }
  l_current_data = (*p_j2k).m_specific_param.m_encoder.m_header_tile_data;
  opj_write_bytes_LE(
    l_current_data,
    0xff74 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    l_mct_size.wrapping_sub(2u32),
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    0 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  /* only one marker atm */
  l_tmp = (*p_mct_record).m_index & 0xffu32
    | ((*p_mct_record).m_array_type as libc::c_uint) << 8i32
    | ((*p_mct_record).m_element_type as libc::c_uint) << 10i32; /* Ymct */
  opj_write_bytes_LE(l_current_data, l_tmp, 2 as OPJ_UINT32);
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    0 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  memcpy(
    l_current_data as *mut libc::c_void,
    (*p_mct_record).m_data as *const libc::c_void,
    (*p_mct_record).m_data_size as libc::c_ulong,
  );
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_mct_size as OPJ_SIZE_T,
    p_manager,
  ) != l_mct_size as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Reads a MCT marker (Multiple Component Transform)
 *
 * @param       p_header_data   the data contained in the MCT box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the MCT marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a MCT marker (Multiple Component Transform)
 *
 * @param       p_header_data   the data contained in the MCT box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the MCT marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_mct(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tmp: OPJ_UINT32 = 0;
  let mut l_indix: OPJ_UINT32 = 0;
  let mut l_mct_data = 0 as *mut opj_mct_data_t;
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  l_tcp = if (*p_j2k).m_specific_param.m_decoder.m_state
    == J2KState::TPH
  {
    &mut *(*p_j2k)
      .m_cp
      .tcps
      .offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_default_tcp
  };
  if p_header_size < 2u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading MCT marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* first marker */
  opj_read_bytes_LE(p_header_data, &mut l_tmp, 2 as OPJ_UINT32); /* Zmct */
  p_header_data = p_header_data.offset(2);
  if l_tmp != 0u32 {
    opj_event_msg(
      p_manager,
      2i32,
      b"Cannot take in charge mct data within multiple MCT records\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 1i32;
  }
  if p_header_size <= 6u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading MCT marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* Imct -> no need for other values, take the first, type is double with decorrelation x0000 1101 0000 0000*/
  opj_read_bytes_LE(p_header_data, &mut l_tmp, 2 as OPJ_UINT32); /* Imct */
  p_header_data = p_header_data.offset(2);
  l_indix = l_tmp & 0xffu32;
  l_mct_data = (*l_tcp).m_mct_records;
  i = 0 as OPJ_UINT32;
  while i < (*l_tcp).m_nb_mct_records {
    if (*l_mct_data).m_index == l_indix {
      break;
    }
    l_mct_data = l_mct_data.offset(1);
    i = i.wrapping_add(1)
  }
  /* NOT FOUND */
  if i == (*l_tcp).m_nb_mct_records {
    if (*l_tcp).m_nb_mct_records == (*l_tcp).m_nb_max_mct_records {
      let mut new_mct_records = 0 as *mut opj_mct_data_t;
      (*l_tcp).m_nb_max_mct_records = ((*l_tcp).m_nb_max_mct_records as libc::c_uint)
        .wrapping_add(10u32)
        as OPJ_UINT32;
      new_mct_records = opj_realloc(
        (*l_tcp).m_mct_records as *mut libc::c_void,
        ((*l_tcp).m_nb_max_mct_records as libc::c_ulong)
          .wrapping_mul(core::mem::size_of::<opj_mct_data_t>() as libc::c_ulong),
      ) as *mut opj_mct_data_t;
      if new_mct_records.is_null() {
        opj_free((*l_tcp).m_mct_records as *mut libc::c_void);
        (*l_tcp).m_mct_records = 0 as *mut opj_mct_data_t;
        (*l_tcp).m_nb_max_mct_records = 0 as OPJ_UINT32;
        (*l_tcp).m_nb_mct_records = 0 as OPJ_UINT32;
        opj_event_msg(
          p_manager,
          1i32,
          b"Not enough memory to read MCT marker\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
      }
      /* Update m_mcc_records[].m_offset_array and m_decorrelation_array
       * to point to the new addresses */
      if new_mct_records != (*l_tcp).m_mct_records {
        i = 0 as OPJ_UINT32; /* Ymct */
        while i < (*l_tcp).m_nb_mcc_records {
          let mut l_mcc_record: *mut opj_simple_mcc_decorrelation_data_t =
            &mut *(*l_tcp).m_mcc_records.offset(i as isize)
              as *mut opj_simple_mcc_decorrelation_data_t;
          if !(*l_mcc_record).m_decorrelation_array.is_null() {
            (*l_mcc_record).m_decorrelation_array = new_mct_records.offset(
              (*l_mcc_record)
                .m_decorrelation_array
                .offset_from((*l_tcp).m_mct_records) as libc::c_long
                as isize,
            )
          }
          if !(*l_mcc_record).m_offset_array.is_null() {
            (*l_mcc_record).m_offset_array = new_mct_records.offset(
              (*l_mcc_record)
                .m_offset_array
                .offset_from((*l_tcp).m_mct_records) as libc::c_long
                as isize,
            )
          }
          i = i.wrapping_add(1)
        }
      }
      (*l_tcp).m_mct_records = new_mct_records;
      l_mct_data = (*l_tcp)
        .m_mct_records
        .offset((*l_tcp).m_nb_mct_records as isize);
      memset(
        l_mct_data as *mut libc::c_void,
        0i32,
        ((*l_tcp)
          .m_nb_max_mct_records
          .wrapping_sub((*l_tcp).m_nb_mct_records) as libc::c_ulong)
          .wrapping_mul(core::mem::size_of::<opj_mct_data_t>() as libc::c_ulong),
      );
    }
    l_mct_data = (*l_tcp)
      .m_mct_records
      .offset((*l_tcp).m_nb_mct_records as isize);
    (*l_tcp).m_nb_mct_records = (*l_tcp).m_nb_mct_records.wrapping_add(1)
  }
  if !(*l_mct_data).m_data.is_null() {
    opj_free((*l_mct_data).m_data as *mut libc::c_void);
    (*l_mct_data).m_data = 0 as *mut OPJ_BYTE;
    (*l_mct_data).m_data_size = 0 as OPJ_UINT32
  }
  (*l_mct_data).m_index = l_indix;
  (*l_mct_data).m_array_type =
    (l_tmp >> 8i32 & 3u32) as J2K_MCT_ARRAY_TYPE;
  (*l_mct_data).m_element_type =
    (l_tmp >> 10i32 & 3u32) as J2K_MCT_ELEMENT_TYPE;
  opj_read_bytes_LE(p_header_data, &mut l_tmp, 2 as OPJ_UINT32);
  p_header_data = p_header_data.offset(2);
  if l_tmp != 0u32 {
    opj_event_msg(
      p_manager,
      2i32,
      b"Cannot take in charge multiple MCT markers\n\x00" as *const u8 as *const libc::c_char,
    );
    return 1i32;
  }
  p_header_size = (p_header_size as libc::c_uint).wrapping_sub(6u32)
    as OPJ_UINT32;
  (*l_mct_data).m_data = opj_malloc(p_header_size as size_t) as *mut OPJ_BYTE;
  if (*l_mct_data).m_data.is_null() {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading MCT marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  memcpy(
    (*l_mct_data).m_data as *mut libc::c_void,
    p_header_data as *const libc::c_void,
    p_header_size as libc::c_ulong,
  );
  (*l_mct_data).m_data_size = p_header_size;
  return 1i32;
}
/* *
 * Writes the MCC marker (Multiple Component Collection)
 *
 * @param       p_j2k                   J2K codec.
 * @param       p_mcc_record            FIXME DOC
 * @param       p_stream                the stream to write data to.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_mcc_record(
  mut p_j2k: *mut opj_j2k_t,
  mut p_mcc_record: *mut opj_simple_mcc_decorrelation_data,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_mcc_size: OPJ_UINT32 = 0;
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  let mut l_nb_bytes_for_comp: OPJ_UINT32 = 0;
  let mut l_mask: OPJ_UINT32 = 0;
  let mut l_tmcc: OPJ_UINT32 = 0;
  /* preconditions */
  /* Lmcc */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  if (*p_mcc_record).m_nb_comps > 255u32 {
    l_nb_bytes_for_comp = 2 as OPJ_UINT32;
    l_mask = 0x8000 as OPJ_UINT32
  } else {
    l_nb_bytes_for_comp = 1 as OPJ_UINT32;
    l_mask = 0 as OPJ_UINT32
  }
  l_mcc_size = (*p_mcc_record)
    .m_nb_comps
    .wrapping_mul(2u32)
    .wrapping_mul(l_nb_bytes_for_comp)
    .wrapping_add(19u32);
  if l_mcc_size > (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size {
    let mut new_header_tile_data = opj_realloc(
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
      l_mcc_size as size_t,
    ) as *mut OPJ_BYTE;
    if new_header_tile_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to write MCC marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = new_header_tile_data;
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = l_mcc_size
  }
  l_current_data = (*p_j2k).m_specific_param.m_encoder.m_header_tile_data;
  opj_write_bytes_LE(
    l_current_data,
    0xff75 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    l_mcc_size.wrapping_sub(2u32),
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  /* first marker */
  opj_write_bytes_LE(
    l_current_data,
    0 as OPJ_UINT32,
    2 as OPJ_UINT32,
  ); /* Zmcc */
  l_current_data = l_current_data.offset(2); /* Imcc -> no need for other values, take the first */
  opj_write_bytes_LE(
    l_current_data,
    (*p_mcc_record).m_index,
    1 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(1);
  /* only one marker atm */
  opj_write_bytes_LE(
    l_current_data,
    0 as OPJ_UINT32,
    2 as OPJ_UINT32,
  ); /* Ymcc */
  l_current_data = l_current_data.offset(2); /* Qmcc -> number of collections -> 1 */
  opj_write_bytes_LE(
    l_current_data,
    1 as OPJ_UINT32,
    2 as OPJ_UINT32,
  ); /* Xmcci type of component transformation -> array based decorrelation */
  l_current_data = l_current_data.offset(2); /* Nmcci number of input components involved and size for each component offset = 8 bits */
  opj_write_bytes_LE(
    l_current_data,
    0x1 as OPJ_UINT32,
    1 as OPJ_UINT32,
  ); /* Cmccij Component offset*/
  l_current_data = l_current_data.offset(1); /* Mmcci number of output components involved and size for each component offset = 8 bits */
  opj_write_bytes_LE(
    l_current_data,
    (*p_mcc_record).m_nb_comps | l_mask,
    2 as OPJ_UINT32,
  ); /* Wmccij Component offset*/
  l_current_data = l_current_data.offset(2); /* Tmcci : use MCT defined as number 1 and irreversible array based. */
  i = 0 as OPJ_UINT32;
  while i < (*p_mcc_record).m_nb_comps {
    opj_write_bytes_LE(l_current_data, i, l_nb_bytes_for_comp);
    l_current_data = l_current_data.offset(l_nb_bytes_for_comp as isize);
    i = i.wrapping_add(1)
  }
  opj_write_bytes_LE(
    l_current_data,
    (*p_mcc_record).m_nb_comps | l_mask,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  i = 0 as OPJ_UINT32;
  while i < (*p_mcc_record).m_nb_comps {
    opj_write_bytes_LE(l_current_data, i, l_nb_bytes_for_comp);
    l_current_data = l_current_data.offset(l_nb_bytes_for_comp as isize);
    i = i.wrapping_add(1)
  }
  l_tmcc = (((*p_mcc_record).m_is_irreversible() == 0) as libc::c_uint
    & 1u32)
    << 16i32;
  if !(*p_mcc_record).m_decorrelation_array.is_null() {
    l_tmcc |= (*(*p_mcc_record).m_decorrelation_array).m_index
  }
  if !(*p_mcc_record).m_offset_array.is_null() {
    l_tmcc |= (*(*p_mcc_record).m_offset_array).m_index << 8i32
  }
  opj_write_bytes_LE(l_current_data, l_tmcc, 3 as OPJ_UINT32);
  l_current_data = l_current_data.offset(3);
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_mcc_size as OPJ_SIZE_T,
    p_manager,
  ) != l_mcc_size as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Reads a MCC marker (Multiple Component Collection)
 *
 * @param       p_header_data   the data contained in the MCC box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the MCC marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_mcc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut l_tmp: OPJ_UINT32 = 0;
  let mut l_indix: OPJ_UINT32 = 0;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_mcc_record = 0 as *mut opj_simple_mcc_decorrelation_data_t;
  let mut l_mct_data = 0 as *mut opj_mct_data_t;
  let mut l_nb_collections: OPJ_UINT32 = 0;
  let mut l_nb_comps: OPJ_UINT32 = 0;
  let mut l_nb_bytes_by_comp: OPJ_UINT32 = 0;
  let mut l_new_mcc = 0i32;
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_tcp = if (*p_j2k).m_specific_param.m_decoder.m_state
    == J2KState::TPH
  {
    &mut *(*p_j2k)
      .m_cp
      .tcps
      .offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_default_tcp
  };
  if p_header_size < 2u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading MCC marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* first marker */
  opj_read_bytes_LE(p_header_data, &mut l_tmp, 2 as OPJ_UINT32); /* Zmcc */
  p_header_data = p_header_data.offset(2); /* Imcc -> no need for other values, take the first */
  if l_tmp != 0u32 {
    opj_event_msg(
      p_manager,
      2i32,
      b"Cannot take in charge multiple data spanning\n\x00" as *const u8 as *const libc::c_char,
    );
    return 1i32;
  }
  if p_header_size < 7u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading MCC marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  opj_read_bytes_LE(p_header_data, &mut l_indix, 1 as OPJ_UINT32);
  p_header_data = p_header_data.offset(1);
  l_mcc_record = (*l_tcp).m_mcc_records;
  i = 0 as OPJ_UINT32;
  while i < (*l_tcp).m_nb_mcc_records {
    if (*l_mcc_record).m_index == l_indix {
      break;
    }
    l_mcc_record = l_mcc_record.offset(1);
    i = i.wrapping_add(1)
  }
  /* * NOT FOUND */
  if i == (*l_tcp).m_nb_mcc_records {
    if (*l_tcp).m_nb_mcc_records == (*l_tcp).m_nb_max_mcc_records {
      let mut new_mcc_records = 0 as *mut opj_simple_mcc_decorrelation_data_t;
      (*l_tcp).m_nb_max_mcc_records = ((*l_tcp).m_nb_max_mcc_records as libc::c_uint)
        .wrapping_add(10u32)
        as OPJ_UINT32;
      new_mcc_records = opj_realloc(
        (*l_tcp).m_mcc_records as *mut libc::c_void,
        ((*l_tcp).m_nb_max_mcc_records as libc::c_ulong).wrapping_mul(core::mem::size_of::<
          opj_simple_mcc_decorrelation_data_t,
        >() as libc::c_ulong),
      ) as *mut opj_simple_mcc_decorrelation_data_t;
      if new_mcc_records.is_null() {
        opj_free((*l_tcp).m_mcc_records as *mut libc::c_void);
        (*l_tcp).m_mcc_records = 0 as *mut opj_simple_mcc_decorrelation_data_t;
        (*l_tcp).m_nb_max_mcc_records = 0 as OPJ_UINT32;
        (*l_tcp).m_nb_mcc_records = 0 as OPJ_UINT32;
        opj_event_msg(
          p_manager,
          1i32,
          b"Not enough memory to read MCC marker\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
      }
      (*l_tcp).m_mcc_records = new_mcc_records;
      l_mcc_record = (*l_tcp)
        .m_mcc_records
        .offset((*l_tcp).m_nb_mcc_records as isize);
      memset(
        l_mcc_record as *mut libc::c_void,
        0i32,
        ((*l_tcp)
          .m_nb_max_mcc_records
          .wrapping_sub((*l_tcp).m_nb_mcc_records) as libc::c_ulong)
          .wrapping_mul(
            core::mem::size_of::<opj_simple_mcc_decorrelation_data_t>() as libc::c_ulong
          ),
      );
    }
    l_mcc_record = (*l_tcp)
      .m_mcc_records
      .offset((*l_tcp).m_nb_mcc_records as isize);
    l_new_mcc = 1i32
  }
  (*l_mcc_record).m_index = l_indix;
  /* only one marker atm */
  opj_read_bytes_LE(p_header_data, &mut l_tmp, 2 as OPJ_UINT32); /* Ymcc */
  p_header_data = p_header_data.offset(2); /* Qmcc -> number of collections -> 1 */
  if l_tmp != 0u32 {
    opj_event_msg(
      p_manager,
      2i32,
      b"Cannot take in charge multiple data spanning\n\x00" as *const u8 as *const libc::c_char,
    ); /* Xmcci type of component transformation -> array based decorrelation */
    return 1i32;
  } /* Cmccij Component offset*/
  opj_read_bytes_LE(
    p_header_data,
    &mut l_nb_collections,
    2 as OPJ_UINT32,
  ); /* Wmccij Component offset*/
  p_header_data = p_header_data.offset(2); /* Wmccij Component offset*/
  if l_nb_collections > 1u32 {
    opj_event_msg(
      p_manager,
      2i32,
      b"Cannot take in charge multiple collections\n\x00" as *const u8 as *const libc::c_char,
    );
    return 1i32;
  }
  p_header_size = (p_header_size as libc::c_uint).wrapping_sub(7u32)
    as OPJ_UINT32;
  i = 0 as OPJ_UINT32;
  while i < l_nb_collections {
    if p_header_size < 3u32 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Error reading MCC marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    opj_read_bytes_LE(p_header_data, &mut l_tmp, 1 as OPJ_UINT32);
    p_header_data = p_header_data.offset(1);
    if l_tmp != 1u32 {
      opj_event_msg(
        p_manager,
        2i32,
        b"Cannot take in charge collections other than array decorrelation\n\x00" as *const u8
          as *const libc::c_char,
      );
      return 1i32;
    }
    opj_read_bytes_LE(
      p_header_data,
      &mut l_nb_comps,
      2 as OPJ_UINT32,
    );
    p_header_data = p_header_data.offset(2);
    p_header_size = (p_header_size as libc::c_uint).wrapping_sub(3u32)
      as OPJ_UINT32;
    l_nb_bytes_by_comp =
      (1u32).wrapping_add(l_nb_comps >> 15i32);
    (*l_mcc_record).m_nb_comps = l_nb_comps & 0x7fffu32;
    if p_header_size
      < l_nb_bytes_by_comp
        .wrapping_mul((*l_mcc_record).m_nb_comps)
        .wrapping_add(2u32)
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Error reading MCC marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    p_header_size = (p_header_size as libc::c_uint).wrapping_sub(
      l_nb_bytes_by_comp
        .wrapping_mul((*l_mcc_record).m_nb_comps)
        .wrapping_add(2u32),
    ) as OPJ_UINT32;
    j = 0 as OPJ_UINT32;
    while j < (*l_mcc_record).m_nb_comps {
      opj_read_bytes_LE(p_header_data, &mut l_tmp, l_nb_bytes_by_comp);
      p_header_data = p_header_data.offset(l_nb_bytes_by_comp as isize);
      if l_tmp != j {
        opj_event_msg(
          p_manager,
          2i32,
          b"Cannot take in charge collections with indix shuffle\n\x00" as *const u8
            as *const libc::c_char,
        );
        return 1i32;
      }
      j = j.wrapping_add(1)
    }
    opj_read_bytes_LE(
      p_header_data,
      &mut l_nb_comps,
      2 as OPJ_UINT32,
    );
    p_header_data = p_header_data.offset(2);
    l_nb_bytes_by_comp =
      (1u32).wrapping_add(l_nb_comps >> 15i32);
    l_nb_comps &= 0x7fffu32;
    if l_nb_comps != (*l_mcc_record).m_nb_comps {
      opj_event_msg(
        p_manager,
        2i32,
        b"Cannot take in charge collections without same number of indixes\n\x00" as *const u8
          as *const libc::c_char,
      );
      return 1i32;
    }
    if p_header_size
      < l_nb_bytes_by_comp
        .wrapping_mul((*l_mcc_record).m_nb_comps)
        .wrapping_add(3u32)
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Error reading MCC marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    p_header_size = (p_header_size as libc::c_uint).wrapping_sub(
      l_nb_bytes_by_comp
        .wrapping_mul((*l_mcc_record).m_nb_comps)
        .wrapping_add(3u32),
    ) as OPJ_UINT32;
    j = 0 as OPJ_UINT32;
    while j < (*l_mcc_record).m_nb_comps {
      opj_read_bytes_LE(p_header_data, &mut l_tmp, l_nb_bytes_by_comp);
      p_header_data = p_header_data.offset(l_nb_bytes_by_comp as isize);
      if l_tmp != j {
        opj_event_msg(
          p_manager,
          2i32,
          b"Cannot take in charge collections with indix shuffle\n\x00" as *const u8
            as *const libc::c_char,
        );
        return 1i32;
      }
      j = j.wrapping_add(1)
    }
    opj_read_bytes_LE(p_header_data, &mut l_tmp, 3 as OPJ_UINT32);
    p_header_data = p_header_data.offset(3);
    (*l_mcc_record).set_m_is_irreversible(
      (l_tmp >> 16i32 & 1u32 == 0) as libc::c_int
        as OPJ_BITFIELD,
    );
    (*l_mcc_record).m_decorrelation_array = 0 as *mut opj_mct_data_t;
    (*l_mcc_record).m_offset_array = 0 as *mut opj_mct_data_t;
    l_indix = l_tmp & 0xffu32;
    if l_indix != 0u32 {
      l_mct_data = (*l_tcp).m_mct_records;
      j = 0 as OPJ_UINT32;
      while j < (*l_tcp).m_nb_mct_records {
        if (*l_mct_data).m_index == l_indix {
          (*l_mcc_record).m_decorrelation_array = l_mct_data;
          break;
        } else {
          l_mct_data = l_mct_data.offset(1);
          j = j.wrapping_add(1)
        }
      }
      if (*l_mcc_record).m_decorrelation_array.is_null() {
        opj_event_msg(
          p_manager,
          1i32,
          b"Error reading MCC marker\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
      }
    }
    l_indix = l_tmp >> 8i32 & 0xffu32;
    if l_indix != 0u32 {
      l_mct_data = (*l_tcp).m_mct_records;
      j = 0 as OPJ_UINT32;
      while j < (*l_tcp).m_nb_mct_records {
        if (*l_mct_data).m_index == l_indix {
          (*l_mcc_record).m_offset_array = l_mct_data;
          break;
        } else {
          l_mct_data = l_mct_data.offset(1);
          j = j.wrapping_add(1)
        }
      }
      if (*l_mcc_record).m_offset_array.is_null() {
        opj_event_msg(
          p_manager,
          1i32,
          b"Error reading MCC marker\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
      }
    }
    i = i.wrapping_add(1)
  }
  if p_header_size != 0u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading MCC marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if l_new_mcc != 0 {
    (*l_tcp).m_nb_mcc_records = (*l_tcp).m_nb_mcc_records.wrapping_add(1)
  }
  return 1i32;
}
/* *
 * Writes the MCO marker (Multiple component transformation ordering)
 *
 * @param       p_stream                                the stream to write data to.
 * @param       p_j2k                           J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_mco(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  let mut l_mco_size: OPJ_UINT32 = 0;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_mcc_record = 0 as *mut opj_simple_mcc_decorrelation_data_t;
  let mut i: OPJ_UINT32 = 0;
  /* preconditions */
  /* Lmco */
  /* Imco -> use the mcc indicated by 1*/
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  l_tcp = &mut *(*p_j2k)
    .m_cp
    .tcps
    .offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t;
  l_mco_size = (5u32).wrapping_add((*l_tcp).m_nb_mcc_records);
  if l_mco_size > (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size {
    let mut new_header_tile_data = opj_realloc(
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
      l_mco_size as size_t,
    ) as *mut OPJ_BYTE;
    if new_header_tile_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to write MCO marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = new_header_tile_data;
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = l_mco_size
  }
  l_current_data = (*p_j2k).m_specific_param.m_encoder.m_header_tile_data;
  opj_write_bytes_LE(
    l_current_data,
    0xff77 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    l_mco_size.wrapping_sub(2u32),
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    (*l_tcp).m_nb_mcc_records,
    1 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(1);
  l_mcc_record = (*l_tcp).m_mcc_records;
  i = 0 as OPJ_UINT32;
  while i < (*l_tcp).m_nb_mcc_records {
    opj_write_bytes_LE(
      l_current_data,
      (*l_mcc_record).m_index,
      1 as OPJ_UINT32,
    );
    l_current_data = l_current_data.offset(1);
    l_mcc_record = l_mcc_record.offset(1);
    i = i.wrapping_add(1)
  }
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_mco_size as OPJ_SIZE_T,
    p_manager,
  ) != l_mco_size as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Reads a MCO marker (Multiple Component Transform Ordering)
 *
 * @param       p_header_data   the data contained in the MCO box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the MCO marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a MCO marker (Multiple Component Transform Ordering)
 *
 * @param       p_header_data   the data contained in the MCO box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the MCO marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_mco(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_tmp: OPJ_UINT32 = 0;
  let mut i: OPJ_UINT32 = 0;
  let mut l_nb_stages: OPJ_UINT32 = 0;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  let mut l_image = 0 as *mut opj_image_t;
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_image = (*p_j2k).m_private_image;
  l_tcp = if (*p_j2k).m_specific_param.m_decoder.m_state
    == J2KState::TPH
  {
    &mut *(*p_j2k)
      .m_cp
      .tcps
      .offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_default_tcp
  };
  if p_header_size < 1u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading MCO marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  opj_read_bytes_LE(
    p_header_data,
    &mut l_nb_stages,
    1 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(1);
  if l_nb_stages > 1u32 {
    opj_event_msg(
      p_manager,
      2i32,
      b"Cannot take in charge multiple transformation stages.\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 1i32;
  }
  if p_header_size != l_nb_stages.wrapping_add(1u32) {
    opj_event_msg(
      p_manager,
      2i32,
      b"Error reading MCO marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  l_tccp = (*l_tcp).tccps;
  i = 0 as OPJ_UINT32;
  while i < (*l_image).numcomps {
    (*l_tccp).m_dc_level_shift = 0i32;
    l_tccp = l_tccp.offset(1);
    i = i.wrapping_add(1)
  }
  if !(*l_tcp).m_mct_decoding_matrix.is_null() {
    opj_free((*l_tcp).m_mct_decoding_matrix as *mut libc::c_void);
    (*l_tcp).m_mct_decoding_matrix = 0 as *mut OPJ_FLOAT32
  }
  i = 0 as OPJ_UINT32;
  while i < l_nb_stages {
    opj_read_bytes_LE(p_header_data, &mut l_tmp, 1 as OPJ_UINT32);
    p_header_data = p_header_data.offset(1);
    if opj_j2k_add_mct(l_tcp, (*p_j2k).m_private_image, l_tmp) == 0 {
      return 0i32;
    }
    i = i.wrapping_add(1)
  }
  return 1i32;
}
unsafe fn opj_j2k_add_mct(
  mut p_tcp: *mut opj_tcp_t,
  mut p_image: *mut opj_image_t,
  mut p_index: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_mcc_record = 0 as *mut opj_simple_mcc_decorrelation_data_t;
  let mut l_deco_array = 0 as *mut opj_mct_data_t;
  let mut l_offset_array = 0 as *mut opj_mct_data_t;
  let mut l_data_size: OPJ_UINT32 = 0;
  let mut l_mct_size: OPJ_UINT32 = 0;
  let mut l_offset_size: OPJ_UINT32 = 0;
  let mut l_nb_elem: OPJ_UINT32 = 0;
  let mut l_offset_data = 0 as *mut OPJ_UINT32;
  let mut l_current_offset_data = 0 as *mut OPJ_UINT32;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  /* preconditions */
  assert!(!p_tcp.is_null());
  l_mcc_record = (*p_tcp).m_mcc_records;
  i = 0 as OPJ_UINT32;
  while i < (*p_tcp).m_nb_mcc_records {
    if (*l_mcc_record).m_index == p_index {
      break;
    }
    i = i.wrapping_add(1)
  }
  if i == (*p_tcp).m_nb_mcc_records {
    /* * element discarded **/
    return 1i32;
  }
  if (*l_mcc_record).m_nb_comps != (*p_image).numcomps {
    /* * do not support number of comps != image */
    return 1i32;
  }
  l_deco_array = (*l_mcc_record).m_decorrelation_array;
  if !l_deco_array.is_null() {
    l_data_size = MCT_ELEMENT_SIZE[(*l_deco_array).m_element_type as usize]
      .wrapping_mul((*p_image).numcomps)
      .wrapping_mul((*p_image).numcomps);
    if (*l_deco_array).m_data_size != l_data_size {
      return 0i32;
    }
    l_nb_elem = (*p_image).numcomps.wrapping_mul((*p_image).numcomps);
    l_mct_size =
      l_nb_elem.wrapping_mul(core::mem::size_of::<OPJ_FLOAT32>() as OPJ_UINT32);
    (*p_tcp).m_mct_decoding_matrix = opj_malloc(l_mct_size as size_t) as *mut OPJ_FLOAT32;
    if (*p_tcp).m_mct_decoding_matrix.is_null() {
      return 0i32;
    }
    j2k_mct_read_functions_to_float[(*l_deco_array).m_element_type as usize]
      .expect("non-null function pointer")(
      (*l_deco_array).m_data as *const libc::c_void,
      (*p_tcp).m_mct_decoding_matrix as *mut libc::c_void,
      l_nb_elem,
    );
  }
  l_offset_array = (*l_mcc_record).m_offset_array;
  if !l_offset_array.is_null() {
    l_data_size =
      MCT_ELEMENT_SIZE[(*l_offset_array).m_element_type as usize].wrapping_mul((*p_image).numcomps);
    if (*l_offset_array).m_data_size != l_data_size {
      return 0i32;
    }
    l_nb_elem = (*p_image).numcomps;
    l_offset_size =
      l_nb_elem.wrapping_mul(core::mem::size_of::<OPJ_UINT32>() as OPJ_UINT32);
    l_offset_data = opj_malloc(l_offset_size as size_t) as *mut OPJ_UINT32;
    if l_offset_data.is_null() {
      return 0i32;
    }
    j2k_mct_read_functions_to_int32[(*l_offset_array).m_element_type as usize]
      .expect("non-null function pointer")(
      (*l_offset_array).m_data as *const libc::c_void,
      l_offset_data as *mut libc::c_void,
      l_nb_elem,
    );
    l_tccp = (*p_tcp).tccps;
    l_current_offset_data = l_offset_data;
    i = 0 as OPJ_UINT32;
    while i < (*p_image).numcomps {
      let fresh22 = l_current_offset_data;
      l_current_offset_data = l_current_offset_data.offset(1);
      (*l_tccp).m_dc_level_shift = *fresh22 as OPJ_INT32;
      l_tccp = l_tccp.offset(1);
      i = i.wrapping_add(1)
    }
    opj_free(l_offset_data as *mut libc::c_void);
  }
  return 1i32;
}
/* *
 * Writes the CBD marker (Component bit depth definition)
 *
 * @param       p_stream                                the stream to write data to.
 * @param       p_j2k                           J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_cbd(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_cbd_size: OPJ_UINT32 = 0;
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  let mut l_image = 0 as *mut opj_image_t;
  let mut l_comp = 0 as *mut opj_image_comp_t;
  /* preconditions */
  /* L_CBD */
  /* Component bit depth */
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  l_image = (*p_j2k).m_private_image;
  l_cbd_size =
    (6u32).wrapping_add((*(*p_j2k).m_private_image).numcomps);
  if l_cbd_size > (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size {
    let mut new_header_tile_data = opj_realloc(
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void,
      l_cbd_size as size_t,
    ) as *mut OPJ_BYTE;
    if new_header_tile_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to write CBD marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = new_header_tile_data;
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = l_cbd_size
  }
  l_current_data = (*p_j2k).m_specific_param.m_encoder.m_header_tile_data;
  opj_write_bytes_LE(
    l_current_data,
    0xff78 as OPJ_UINT32,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    l_cbd_size.wrapping_sub(2u32),
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  opj_write_bytes_LE(
    l_current_data,
    (*l_image).numcomps,
    2 as OPJ_UINT32,
  );
  l_current_data = l_current_data.offset(2);
  l_comp = (*l_image).comps;
  i = 0 as OPJ_UINT32;
  while i < (*l_image).numcomps {
    opj_write_bytes_LE(
      l_current_data,
      (*l_comp).sgnd << 7i32
        | (*l_comp)
          .prec
          .wrapping_sub(1u32),
      1 as OPJ_UINT32,
    );
    l_current_data = l_current_data.offset(1);
    l_comp = l_comp.offset(1);
    i = i.wrapping_add(1)
  }
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data,
    l_cbd_size as OPJ_SIZE_T,
    p_manager,
  ) != l_cbd_size as libc::c_ulong
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Reads a CBD marker (Component bit depth definition)
 * @param       p_header_data   the data contained in the CBD box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the CBD marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a CBD marker (Component bit depth definition)
 * @param       p_header_data   the data contained in the CBD box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the CBD marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_cbd(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_nb_comp: OPJ_UINT32 = 0;
  let mut l_num_comp: OPJ_UINT32 = 0;
  let mut l_comp_def: OPJ_UINT32 = 0;
  let mut i: OPJ_UINT32 = 0;
  let mut l_comp = 0 as *mut opj_image_comp_t;
  /* preconditions */
  /* Component bit depth */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  l_num_comp = (*(*p_j2k).m_private_image).numcomps;
  if p_header_size
    != (*(*p_j2k).m_private_image)
      .numcomps
      .wrapping_add(2u32)
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Crror reading CBD marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  opj_read_bytes_LE(
    p_header_data,
    &mut l_nb_comp,
    2 as OPJ_UINT32,
  );
  p_header_data = p_header_data.offset(2);
  if l_nb_comp != l_num_comp {
    opj_event_msg(
      p_manager,
      1i32,
      b"Crror reading CBD marker\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  l_comp = (*(*p_j2k).m_private_image).comps;
  i = 0 as OPJ_UINT32;
  while i < l_num_comp {
    opj_read_bytes_LE(
      p_header_data,
      &mut l_comp_def,
      1 as OPJ_UINT32,
    );
    p_header_data = p_header_data.offset(1);
    (*l_comp).sgnd = l_comp_def >> 7i32 & 1u32;
    (*l_comp).prec = (l_comp_def & 0x7fu32)
      .wrapping_add(1u32);
    if (*l_comp).prec > 31u32 {
      opj_event_msg(p_manager, 1i32,
                          b"Invalid values for comp = %d : prec=%u (should be between 1 and 38 according to the JPEG2000 norm. OpenJpeg only supports up to 31)\n\x00"
                              as *const u8 as *const libc::c_char, i,
                          (*l_comp).prec);
      return 0i32;
    }
    l_comp = l_comp.offset(1);
    i = i.wrapping_add(1)
  }
  return 1i32;
}
/* *
 * Reads a CAP marker (extended capabilities definition). Empty implementation.
 * Found in HTJ2K files
 *
 * @param       p_header_data   the data contained in the CAP box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the CAP marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a CAP marker (extended capabilities definition). Empty implementation.
 * Found in HTJ2K files.
 *
 * @param       p_header_data   the data contained in the CAP box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the CAP marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_cap(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut _p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  return 1i32;
}
/* *
 * Reads a CPF marker (corresponding profile). Empty implementation. Found in HTJ2K files
 * @param       p_header_data   the data contained in the CPF box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the CPF marker.
 * @param       p_manager               the user event manager.
*/
/* *
 * Reads a CPF marker (corresponding profile). Empty implementation. Found in HTJ2K files
 * @param       p_header_data   the data contained in the CPF box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the CPF marker.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_cpf(
  mut p_j2k: *mut opj_j2k_t,
  mut p_header_data: *mut OPJ_BYTE,
  mut _p_header_size: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_header_data.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  return 1i32;
}
/* ----------------------------------------------------------------------- */
/* J2K / JPT decoder interface                                             */
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_setup_decoder(
  mut j2k: *mut opj_j2k_t,
  mut parameters: *mut opj_dparameters_t,
) {
  if !j2k.is_null() && !parameters.is_null() {
    (*j2k).m_cp.m_specific_param.m_dec.m_layer = (*parameters).cp_layer;
    (*j2k).m_cp.m_specific_param.m_dec.m_reduce = (*parameters).cp_reduce;
    (*j2k).dump_state = (*parameters).flags & 0x2u32
    /* USE_JPWL */
  };
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_decoder_set_strict_mode(
  mut j2k: *mut opj_j2k_t,
  mut strict: OPJ_BOOL,
) {
  if !j2k.is_null() {
    (*j2k).m_cp.strict = strict
  };
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_set_threads(
  mut j2k: *mut opj_j2k_t,
  mut num_threads: OPJ_UINT32,
) -> OPJ_BOOL {
  /* Currently we pass the thread-pool to the tcd, so we cannot re-set it */
  /* afterwards */
  if opj_has_thread_support() != 0 && (*j2k).m_tcd.is_null() {
    opj_thread_pool_destroy((*j2k).m_tp);
    (*j2k).m_tp = 0 as *mut opj_thread_pool_t;
    if num_threads <= 2147483647 as OPJ_UINT32 {
      (*j2k).m_tp = opj_thread_pool_create(num_threads as libc::c_int)
    }
    if (*j2k).m_tp.is_null() {
      (*j2k).m_tp = opj_thread_pool_create(0i32);
      return 0i32;
    }
    return 1i32;
  }
  return 0i32;
}
unsafe fn opj_j2k_get_default_thread_count() -> libc::c_int {
  let mut num_threads_str: *const libc::c_char =
    getenv(b"OPJ_NUM_THREADS\x00" as *const u8 as *const libc::c_char);
  let mut num_cpus: libc::c_int = 0;
  let mut num_threads: libc::c_int = 0;
  if num_threads_str.is_null() || opj_has_thread_support() == 0 {
    return 0i32;
  }
  num_cpus = opj_get_num_cpus();
  if strcmp(
    num_threads_str,
    b"ALL_CPUS\x00" as *const u8 as *const libc::c_char,
  ) == 0i32
  {
    return num_cpus;
  }
  if num_cpus == 0i32 {
    num_cpus = 32i32
  }
  num_threads = atoi(num_threads_str);
  if num_threads < 0i32 {
    num_threads = 0i32
  } else if num_threads > 2i32 * num_cpus {
    num_threads = 2i32 * num_cpus
  }
  return num_threads;
}
/* ----------------------------------------------------------------------- */
/* J2K encoder interface                                                       */
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_create_compress() -> *mut opj_j2k_t {
  let mut l_j2k = opj_calloc(
    1i32 as size_t,
    core::mem::size_of::<opj_j2k_t>() as libc::c_ulong,
  ) as *mut opj_j2k_t;
  if l_j2k.is_null() {
    return 0 as *mut opj_j2k_t;
  }
  (*l_j2k).m_is_decoder = 0i32;
  (*l_j2k)
    .m_cp
    .set_m_is_decoder(0 as OPJ_BITFIELD);
  (*l_j2k).m_specific_param.m_encoder.m_header_tile_data =
    opj_malloc(1000i32 as size_t) as *mut OPJ_BYTE;
  if (*l_j2k)
    .m_specific_param
    .m_encoder
    .m_header_tile_data
    .is_null()
  {
    opj_j2k_destroy(l_j2k);
    return 0 as *mut opj_j2k_t;
  }
  (*l_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 1000 as OPJ_UINT32;
  /* validation list creation*/
  (*l_j2k).m_validation_list = opj_procedure_list_create();
  if (*l_j2k).m_validation_list.is_null() {
    opj_j2k_destroy(l_j2k);
    return 0 as *mut opj_j2k_t;
  }
  /* execution list creation*/
  (*l_j2k).m_procedure_list = opj_procedure_list_create();
  if (*l_j2k).m_procedure_list.is_null() {
    opj_j2k_destroy(l_j2k);
    return 0 as *mut opj_j2k_t;
  }
  (*l_j2k).m_tp = opj_thread_pool_create(opj_j2k_get_default_thread_count());
  if (*l_j2k).m_tp.is_null() {
    (*l_j2k).m_tp = opj_thread_pool_create(0i32)
  }
  if (*l_j2k).m_tp.is_null() {
    opj_j2k_destroy(l_j2k);
    return 0 as *mut opj_j2k_t;
  }
  return l_j2k;
}
unsafe fn opj_j2k_initialise_4K_poc(
  mut POC: *mut opj_poc_t,
  mut numres: libc::c_int,
) -> libc::c_int {
  (*POC.offset(0)).tile = 1 as OPJ_UINT32;
  (*POC.offset(0)).resno0 = 0 as OPJ_UINT32;
  (*POC.offset(0)).compno0 = 0 as OPJ_UINT32;
  (*POC.offset(0)).layno1 = 1 as OPJ_UINT32;
  (*POC.offset(0)).resno1 = (numres - 1i32) as OPJ_UINT32;
  (*POC.offset(0)).compno1 = 3 as OPJ_UINT32;
  (*POC.offset(0)).prg1 = OPJ_CPRL;
  (*POC.offset(1)).tile = 1 as OPJ_UINT32;
  (*POC.offset(1)).resno0 = (numres - 1i32) as OPJ_UINT32;
  (*POC.offset(1)).compno0 = 0 as OPJ_UINT32;
  (*POC.offset(1)).layno1 = 1 as OPJ_UINT32;
  (*POC.offset(1)).resno1 = numres as OPJ_UINT32;
  (*POC.offset(1)).compno1 = 3 as OPJ_UINT32;
  (*POC.offset(1)).prg1 = OPJ_CPRL;
  return 2i32;
}
unsafe fn opj_j2k_set_cinema_parameters(
  mut parameters: *mut opj_cparameters_t,
  mut image: *mut opj_image_t,
  mut p_manager: *mut opj_event_mgr_t,
) {
  /* Configure cinema parameters */
  let mut i: libc::c_int = 0;
  /* No tiling */
  (*parameters).tile_size_on = 0i32;
  (*parameters).cp_tdx = 1i32;
  (*parameters).cp_tdy = 1i32;
  /* One tile part for each component */
  (*parameters).tp_flag = 'C' as i32 as libc::c_char;
  (*parameters).tp_on = 1 as libc::c_char;
  /* Tile and Image shall be at (0,0) */
  (*parameters).cp_tx0 = 0i32;
  (*parameters).cp_ty0 = 0i32;
  (*parameters).image_offset_x0 = 0i32;
  (*parameters).image_offset_y0 = 0i32;
  /* Codeblock size= 32*32 */
  (*parameters).cblockw_init = 32i32;
  (*parameters).cblockh_init = 32i32;
  /* Codeblock style: no mode switch enabled */
  (*parameters).mode = 0i32;
  /* No ROI */
  (*parameters).roi_compno = -(1i32);
  /* No subsampling */
  (*parameters).subsampling_dx = 1i32;
  (*parameters).subsampling_dy = 1i32;
  /* 9-7 transform */
  (*parameters).irreversible = 1i32;
  /* Number of layers */
  if (*parameters).tcp_numlayers > 1i32 {
    opj_event_msg(p_manager, 2i32,
                      b"JPEG 2000 Profile-3 and 4 (2k/4k dc profile) requires:\n1 single quality layer-> Number of layers forced to 1 (rather than %d)\n-> Rate of the last layer (%3.1f) will be used\x00"
                          as *const u8 as *const libc::c_char,
                      (*parameters).tcp_numlayers,
                      (*parameters).tcp_rates[((*parameters).tcp_numlayers -
                                                   1i32) as usize]
                          as libc::c_double);
    (*parameters).tcp_rates[0 as usize] =
      (*parameters).tcp_rates[((*parameters).tcp_numlayers - 1i32) as usize];
    (*parameters).tcp_numlayers = 1i32
  }
  /* Resolution levels */
  match (*parameters).rsiz as libc::c_int {
    3 => {
      if (*parameters).numresolution > 6i32 {
        opj_event_msg(p_manager, 2i32,
                              b"JPEG 2000 Profile-3 (2k dc profile) requires:\nNumber of decomposition levels <= 5\n-> Number of decomposition levels forced to 5 (rather than %d)\n\x00"
                                  as *const u8 as *const libc::c_char,
                              (*parameters).numresolution + 1i32);
        (*parameters).numresolution = 6i32
      }
    }
    4 => {
      if (*parameters).numresolution < 2i32 {
        opj_event_msg(p_manager, 2i32,
                              b"JPEG 2000 Profile-4 (4k dc profile) requires:\nNumber of decomposition levels >= 1 && <= 6\n-> Number of decomposition levels forced to 1 (rather than %d)\n\x00"
                                  as *const u8 as *const libc::c_char,
                              (*parameters).numresolution + 1i32);
        (*parameters).numresolution = 1i32
      } else if (*parameters).numresolution > 7i32 {
        opj_event_msg(p_manager, 2i32,
                              b"JPEG 2000 Profile-4 (4k dc profile) requires:\nNumber of decomposition levels >= 1 && <= 6\n-> Number of decomposition levels forced to 6 (rather than %d)\n\x00"
                                  as *const u8 as *const libc::c_char,
                              (*parameters).numresolution + 1i32);
        (*parameters).numresolution = 7i32
      }
    }
    _ => {}
  }
  /* Precincts */
  (*parameters).csty |= 0x1i32;
  if (*parameters).numresolution == 1i32 {
    (*parameters).res_spec = 1i32;
    (*parameters).prcw_init[0 as usize] = 128i32;
    (*parameters).prch_init[0 as usize] = 128i32
  } else {
    (*parameters).res_spec = (*parameters).numresolution - 1i32;
    i = 0i32;
    while i < (*parameters).res_spec {
      (*parameters).prcw_init[i as usize] = 256i32;
      (*parameters).prch_init[i as usize] = 256i32;
      i += 1
    }
  }
  /* The progression order shall be CPRL */
  (*parameters).prog_order = OPJ_CPRL;
  /* Progression order changes for 4K, disallowed for 2K */
  if (*parameters).rsiz as libc::c_int == 0x4i32 {
    (*parameters).numpocs =
      opj_j2k_initialise_4K_poc((*parameters).POC.as_mut_ptr(), (*parameters).numresolution)
        as OPJ_UINT32
  } else {
    (*parameters).numpocs = 0 as OPJ_UINT32
  }
  /* Limited bit-rate */
  (*parameters).cp_disto_alloc = 1i32;
  if (*parameters).max_cs_size <= 0i32 {
    /* No rate has been introduced, 24 fps is assumed */
    (*parameters).max_cs_size = 1302083i32;
    opj_event_msg(p_manager, 2i32,
                      b"JPEG 2000 Profile-3 and 4 (2k/4k dc profile) requires:\nMaximum 1302083 compressed bytes @ 24fps\nAs no rate has been given, this limit will be used.\n\x00"
                          as *const u8 as *const libc::c_char);
  } else if (*parameters).max_cs_size > 1302083i32 {
    opj_event_msg(p_manager, 2i32,
                      b"JPEG 2000 Profile-3 and 4 (2k/4k dc profile) requires:\nMaximum 1302083 compressed bytes @ 24fps\n-> Specified rate exceeds this limit. Rate will be forced to 1302083 bytes.\n\x00"
                          as *const u8 as *const libc::c_char);
    (*parameters).max_cs_size = 1302083i32
  }
  if (*parameters).max_comp_size <= 0i32 {
    /* No rate has been introduced, 24 fps is assumed */
    (*parameters).max_comp_size = 1041666i32;
    opj_event_msg(p_manager, 2i32,
                      b"JPEG 2000 Profile-3 and 4 (2k/4k dc profile) requires:\nMaximum 1041666 compressed bytes @ 24fps\nAs no rate has been given, this limit will be used.\n\x00"
                          as *const u8 as *const libc::c_char);
  } else if (*parameters).max_comp_size > 1041666i32 {
    opj_event_msg(p_manager, 2i32,
                      b"JPEG 2000 Profile-3 and 4 (2k/4k dc profile) requires:\nMaximum 1041666 compressed bytes @ 24fps\n-> Specified rate exceeds this limit. Rate will be forced to 1041666 bytes.\n\x00"
                          as *const u8 as *const libc::c_char);
    (*parameters).max_comp_size = 1041666i32
  }
  (*parameters).tcp_rates[0 as usize] = (*image)
    .numcomps
    .wrapping_mul((*(*image).comps.offset(0)).w)
    .wrapping_mul((*(*image).comps.offset(0)).h)
    .wrapping_mul((*(*image).comps.offset(0)).prec)
    as OPJ_FLOAT32
    / ((*parameters).max_cs_size as OPJ_UINT32)
      .wrapping_mul(8u32)
      .wrapping_mul((*(*image).comps.offset(0)).dx)
      .wrapping_mul((*(*image).comps.offset(0)).dy) as OPJ_FLOAT32;
}
unsafe fn opj_j2k_is_cinema_compliant(
  mut image: *mut opj_image_t,
  mut rsiz: OPJ_UINT16,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  /* Number of components */
  if (*image).numcomps != 3u32 {
    opj_event_msg(p_manager, 2i32,
                      b"JPEG 2000 Profile-3 (2k dc profile) requires:\n3 components-> Number of components of input image (%d) is not compliant\n-> Non-profile-3 codestream will be generated\n\x00"
                          as *const u8 as *const libc::c_char,
                      (*image).numcomps);
    return 0i32;
  }
  /* Bitdepth */
  i = 0 as OPJ_UINT32;
  while i < (*image).numcomps {
    if ((*(*image).comps.offset(i as isize)).prec != 12u32)
      as libc::c_uint
      | (*(*image).comps.offset(i as isize)).sgnd
      != 0
    {
      let mut signed_str: [libc::c_char; 7] =
        *core::mem::transmute::<&[u8; 7], &mut [libc::c_char; 7]>(b"signed\x00");
      let mut unsigned_str: [libc::c_char; 9] =
        *core::mem::transmute::<&[u8; 9], &mut [libc::c_char; 9]>(b"unsigned\x00");
      let mut tmp_str = if (*(*image).comps.offset(i as isize)).sgnd != 0 {
        signed_str.as_mut_ptr()
      } else {
        unsigned_str.as_mut_ptr()
      };
      opj_event_msg(p_manager, 2i32,
                          b"JPEG 2000 Profile-3 (2k dc profile) requires:\nPrecision of each component shall be 12 bits unsigned-> At least component %d of input image (%d bits, %s) is not compliant\n-> Non-profile-3 codestream will be generated\n\x00"
                              as *const u8 as *const libc::c_char, i,
                          (*(*image).comps.offset(i as isize)).prec, tmp_str);
      return 0i32;
    }
    i = i.wrapping_add(1)
  }
  /* Image size */
  match rsiz as libc::c_int {
    3 => {
      if ((*(*image).comps.offset(0)).w
        > 2048u32) as libc::c_int
        | ((*(*image).comps.offset(0)).h
          > 1080u32) as libc::c_int
        != 0
      {
        opj_event_msg(p_manager, 2i32,
                              b"JPEG 2000 Profile-3 (2k dc profile) requires:\nwidth <= 2048 and height <= 1080\n-> Input image size %d x %d is not compliant\n-> Non-profile-3 codestream will be generated\n\x00"
                                  as *const u8 as *const libc::c_char,
                              (*(*image).comps.offset(0i32 as
                                                          isize)).w,
                              (*(*image).comps.offset(0i32 as
                                                          isize)).h);
        return 0i32;
      }
    }
    4 => {
      if ((*(*image).comps.offset(0)).w
        > 4096u32) as libc::c_int
        | ((*(*image).comps.offset(0)).h
          > 2160u32) as libc::c_int
        != 0
      {
        opj_event_msg(p_manager, 2i32,
                              b"JPEG 2000 Profile-4 (4k dc profile) requires:\nwidth <= 4096 and height <= 2160\n-> Image size %d x %d is not compliant\n-> Non-profile-4 codestream will be generated\n\x00"
                                  as *const u8 as *const libc::c_char,
                              (*(*image).comps.offset(0i32 as
                                                          isize)).w,
                              (*(*image).comps.offset(0i32 as
                                                          isize)).h);
        return 0i32;
      }
    }
    _ => {}
  }
  return 1i32;
}
unsafe fn opj_j2k_get_imf_max_NL(
  mut parameters: *mut opj_cparameters_t,
  mut image: *mut opj_image_t,
) -> libc::c_int {
  /* Decomposition levels */
  let rsiz = (*parameters).rsiz;
  let profile = (rsiz as libc::c_int & 0xff00i32) as OPJ_UINT16;
  let XTsiz = if (*parameters).tile_size_on != 0 {
    (*parameters).cp_tdx as OPJ_UINT32
  } else {
    (*image).x1
  };
  match profile as libc::c_int {
    1024 => return 5i32,
    1280 => return 6i32,
    1536 => return 7i32,
    1792 => {
      if XTsiz >= 2048u32 {
        return 5i32;
      } else {
        if XTsiz >= 1024u32 {
          return 4i32;
        }
      }
    }
    2048 => {
      if XTsiz >= 4096u32 {
        return 6i32;
      } else {
        if XTsiz >= 2048u32 {
          return 5i32;
        } else {
          if XTsiz >= 1024u32 {
            return 4i32;
          }
        }
      }
    }
    2304 => {
      if XTsiz >= 8192u32 {
        return 7i32;
      } else {
        if XTsiz >= 4096u32 {
          return 6i32;
        } else {
          if XTsiz >= 2048u32 {
            return 5i32;
          } else {
            if XTsiz >= 1024u32 {
              return 4i32;
            }
          }
        }
      }
    }
    _ => {}
  }
  return -(1i32);
}
unsafe fn opj_j2k_set_imf_parameters(
  mut parameters: *mut opj_cparameters_t,
  mut image: *mut opj_image_t,
  mut _p_manager: *mut opj_event_mgr_t,
) {
  let rsiz = (*parameters).rsiz;
  let profile = (rsiz as libc::c_int & 0xff00i32) as OPJ_UINT16;
  /* Override defaults set by opj_set_default_encoder_parameters */
  if (*parameters).cblockw_init == 64i32
    && (*parameters).cblockh_init == 64i32
  {
    (*parameters).cblockw_init = 32i32;
    (*parameters).cblockh_init = 32i32
  }
  /* One tile part for each component */
  (*parameters).tp_flag = 'C' as i32 as libc::c_char;
  (*parameters).tp_on = 1 as libc::c_char;
  if (*parameters).prog_order as libc::c_int == OPJ_LRCP as libc::c_int {
    (*parameters).prog_order = OPJ_CPRL
  }
  if profile as libc::c_int == 0x400i32
    || profile as libc::c_int == 0x500i32
    || profile as libc::c_int == 0x600i32
  {
    /* 9-7 transform */
    (*parameters).irreversible = 1i32
  }
  /* Adjust the number of resolutions if set to its defaults */
  if (*parameters).numresolution == 6i32
    && (*image).x0 == 0u32
    && (*image).y0 == 0u32
  {
    let max_NL = opj_j2k_get_imf_max_NL(parameters, image);
    if max_NL >= 0i32 && (*parameters).numresolution > max_NL {
      (*parameters).numresolution = max_NL + 1i32
    }
    /* Note: below is generic logic */
    if (*parameters).tile_size_on == 0 {
      while (*parameters).numresolution > 0i32 {
        if (*image).x1
          < (1u32)
            << ((*parameters).numresolution as OPJ_UINT32).wrapping_sub(1u32)
        {
          (*parameters).numresolution -= 1
        } else {
          if !((*image).y1
            < (1u32)
              << ((*parameters).numresolution as OPJ_UINT32).wrapping_sub(1u32))
          {
            break;
          }
          (*parameters).numresolution -= 1
        }
      }
    }
  }
  /* Set defaults precincts */
  if (*parameters).csty == 0i32 {
    (*parameters).csty |= 0x1i32;
    if (*parameters).numresolution == 1i32 {
      (*parameters).res_spec = 1i32;
      (*parameters).prcw_init[0 as usize] = 128i32;
      (*parameters).prch_init[0 as usize] = 128i32
    } else {
      let mut i: libc::c_int = 0;
      (*parameters).res_spec = (*parameters).numresolution - 1i32;
      i = 0i32;
      while i < (*parameters).res_spec {
        (*parameters).prcw_init[i as usize] = 256i32;
        (*parameters).prch_init[i as usize] = 256i32;
        i += 1
      }
    }
  };
}
/* Table A.53 from JPEG2000 standard */
static mut tabMaxSubLevelFromMainLevel: [OPJ_UINT16; 12] = [
  15 as OPJ_UINT16,
  1 as OPJ_UINT16,
  1 as OPJ_UINT16,
  1 as OPJ_UINT16,
  2 as OPJ_UINT16,
  3 as OPJ_UINT16,
  4 as OPJ_UINT16,
  5 as OPJ_UINT16,
  6 as OPJ_UINT16,
  7 as OPJ_UINT16,
  8 as OPJ_UINT16,
  9 as OPJ_UINT16,
];
unsafe fn opj_j2k_is_imf_compliant(
  mut parameters: *mut opj_cparameters_t,
  mut image: *mut opj_image_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let rsiz = (*parameters).rsiz;
  let profile = (rsiz as libc::c_int & 0xff00i32) as OPJ_UINT16;
  let mainlevel = (rsiz as libc::c_int & 0xfi32) as OPJ_UINT16;
  let sublevel = (rsiz as libc::c_int >> 4i32 & 0xfi32) as OPJ_UINT16;
  let NL = (*parameters).numresolution - 1i32;
  let XTsiz = if (*parameters).tile_size_on != 0 {
    (*parameters).cp_tdx as OPJ_UINT32
  } else {
    (*image).x1
  };
  let mut ret = 1i32;
  /* Validate mainlevel */
  if mainlevel as libc::c_int > 11i32 {
    opj_event_msg(p_manager, 2i32,
                      b"IMF profile require mainlevel <= 11.\n-> %d is thus not compliant\n-> Non-IMF codestream will be generated\n\x00"
                          as *const u8 as *const libc::c_char,
                      mainlevel as libc::c_int);
    ret = 0i32
  } else {
    /* Validate sublevel */
    assert!(
      core::mem::size_of::<[OPJ_UINT16; 12]>() as libc::c_ulong
        == ((11i32 + 1i32) as libc::c_ulong)
          .wrapping_mul(core::mem::size_of::<OPJ_UINT16>() as libc::c_ulong)
    );
    if sublevel as libc::c_int > tabMaxSubLevelFromMainLevel[mainlevel as usize] as libc::c_int {
      opj_event_msg(p_manager, 2i32,
                          b"IMF profile require sublevel <= %d for mainlevel = %d.\n-> %d is thus not compliant\n-> Non-IMF codestream will be generated\n\x00"
                              as *const u8 as *const libc::c_char,
                          tabMaxSubLevelFromMainLevel[mainlevel as usize] as
                              libc::c_int, mainlevel as libc::c_int,
                          sublevel as libc::c_int);
      ret = 0i32
    }
  }
  /* Number of components */
  if (*image).numcomps > 3u32 {
    opj_event_msg(p_manager, 2i32,
                      b"IMF profiles require at most 3 components.\n-> Number of components of input image (%d) is not compliant\n-> Non-IMF codestream will be generated\n\x00"
                          as *const u8 as *const libc::c_char,
                      (*image).numcomps);
    ret = 0i32
  }
  if (*image).x0 != 0u32
    || (*image).y0 != 0u32
  {
    opj_event_msg(p_manager, 2i32,
                      b"IMF profiles require image origin to be at 0,0.\n-> %d,%d is not compliant\n-> Non-IMF codestream will be generated\n\x00"
                          as *const u8 as *const libc::c_char, (*image).x0,
                      ((*image).y0 != 0u32) as
                          libc::c_int);
    ret = 0i32
  }
  if (*parameters).cp_tx0 != 0i32 || (*parameters).cp_ty0 != 0i32 {
    opj_event_msg(p_manager, 2i32,
                      b"IMF profiles require tile origin to be at 0,0.\n-> %d,%d is not compliant\n-> Non-IMF codestream will be generated\n\x00"
                          as *const u8 as *const libc::c_char,
                      (*parameters).cp_tx0, (*parameters).cp_ty0);
    ret = 0i32
  }
  if (*parameters).tile_size_on != 0 {
    if profile as libc::c_int == 0x400i32
      || profile as libc::c_int == 0x500i32
      || profile as libc::c_int == 0x600i32
    {
      if ((*parameters).cp_tdx as OPJ_UINT32) < (*image).x1
        || ((*parameters).cp_tdy as OPJ_UINT32) < (*image).y1
      {
        opj_event_msg(p_manager, 2i32,
                              b"IMF 2K/4K/8K single tile profiles require tile to be greater or equal to image size.\n-> %d,%d is lesser than %d,%d\n-> Non-IMF codestream will be generated\n\x00"
                                  as *const u8 as *const libc::c_char,
                              (*parameters).cp_tdx, (*parameters).cp_tdy,
                              (*image).x1, (*image).y1);
        ret = 0i32
      }
    } else if !((*parameters).cp_tdx as OPJ_UINT32 >= (*image).x1
      && (*parameters).cp_tdy as OPJ_UINT32 >= (*image).y1)
    {
      if !((*parameters).cp_tdx == 1024i32
        && (*parameters).cp_tdy == 1024i32)
      {
        if !((*parameters).cp_tdx == 2048i32
          && (*parameters).cp_tdy == 2048i32
          && (profile as libc::c_int == 0x500i32
            || profile as libc::c_int == 0x600i32))
        {
          if !((*parameters).cp_tdx == 4096i32
            && (*parameters).cp_tdy == 4096i32
            && profile as libc::c_int == 0x600i32)
          {
            opj_event_msg(p_manager, 2i32,
                                      b"IMF 2K_R/4K_R/8K_R single/multiple tile profiles require tile to be greater or equal to image size,\nor to be (1024,1024), or (2048,2048) for 4K_R/8K_R or (4096,4096) for 8K_R.\n-> %d,%d is non conformant\n-> Non-IMF codestream will be generated\n\x00"
                                          as *const u8 as *const libc::c_char,
                                      (*parameters).cp_tdx,
                                      (*parameters).cp_tdy);
            ret = 0i32
          }
        }
      }
    }
  }
  /* Bitdepth */
  i = 0 as OPJ_UINT32;
  while i < (*image).numcomps {
    if !((*(*image).comps.offset(i as isize)).prec >= 8u32
      && (*(*image).comps.offset(i as isize)).prec <= 16u32)
      || (*(*image).comps.offset(i as isize)).sgnd != 0
    {
      let mut signed_str: [libc::c_char; 7] =
        *core::mem::transmute::<&[u8; 7], &mut [libc::c_char; 7]>(b"signed\x00");
      let mut unsigned_str: [libc::c_char; 9] =
        *core::mem::transmute::<&[u8; 9], &mut [libc::c_char; 9]>(b"unsigned\x00");
      let mut tmp_str = if (*(*image).comps.offset(i as isize)).sgnd != 0 {
        signed_str.as_mut_ptr()
      } else {
        unsigned_str.as_mut_ptr()
      };
      opj_event_msg(p_manager, 2i32,
                          b"IMF profiles require precision of each component to b in [8-16] bits unsigned-> At least component %d of input image (%d bits, %s) is not compliant\n-> Non-IMF codestream will be generated\n\x00"
                              as *const u8 as *const libc::c_char, i,
                          (*(*image).comps.offset(i as isize)).prec, tmp_str);
      ret = 0i32
    }
    i = i.wrapping_add(1)
  }
  /* Sub-sampling */
  i = 0 as OPJ_UINT32;
  while i < (*image).numcomps {
    if i == 0u32
      && (*(*image).comps.offset(i as isize)).dx != 1u32
    {
      opj_event_msg(p_manager, 2i32,
                          b"IMF profiles require XRSiz1 == 1. Here it is set to %d.\n-> Non-IMF codestream will be generated\n\x00"
                              as *const u8 as *const libc::c_char,
                          (*(*image).comps.offset(i as isize)).dx);
      ret = 0i32
    }
    if i == 1u32
      && (*(*image).comps.offset(i as isize)).dx != 1u32
      && (*(*image).comps.offset(i as isize)).dx != 2u32
    {
      opj_event_msg(p_manager, 2i32,
                          b"IMF profiles require XRSiz2 == 1 or 2. Here it is set to %d.\n-> Non-IMF codestream will be generated\n\x00"
                              as *const u8 as *const libc::c_char,
                          (*(*image).comps.offset(i as isize)).dx);
      ret = 0i32
    }
    if i > 1u32
      && (*(*image).comps.offset(i as isize)).dx
        != (*(*image)
          .comps
          .offset(i.wrapping_sub(1u32) as isize))
        .dx
    {
      opj_event_msg(p_manager, 2i32,
                          b"IMF profiles require XRSiz%d to be the same as XRSiz2. Here it is set to %d instead of %d.\n-> Non-IMF codestream will be generated\n\x00"
                              as *const u8 as *const libc::c_char,
                          i.wrapping_add(1u32),
                          (*(*image).comps.offset(i as isize)).dx,
                          (*(*image).comps.offset(i.wrapping_sub(1 as
                                                                     libc::c_int
                                                                     as
                                                                     libc::c_uint)
                                                      as isize)).dx);
      ret = 0i32
    }
    if (*(*image).comps.offset(i as isize)).dy != 1u32 {
      opj_event_msg(p_manager, 2i32,
                          b"IMF profiles require YRsiz == 1. Here it is set to %d for component %d.\n-> Non-IMF codestream will be generated\n\x00"
                              as *const u8 as *const libc::c_char,
                          (*(*image).comps.offset(i as isize)).dy, i);
      ret = 0i32
    }
    i = i.wrapping_add(1)
  }
  /* Image size */
  match profile as libc::c_int {
    1024 | 1792 => {
      if ((*(*image).comps.offset(0)).w
        > 2048u32) as libc::c_int
        | ((*(*image).comps.offset(0)).h
          > 1556u32) as libc::c_int
        != 0
      {
        opj_event_msg(p_manager, 2i32,
                              b"IMF 2K/2K_R profile require:\nwidth <= 2048 and height <= 1556\n-> Input image size %d x %d is not compliant\n-> Non-IMF codestream will be generated\n\x00"
                                  as *const u8 as *const libc::c_char,
                              (*(*image).comps.offset(0i32 as
                                                          isize)).w,
                              (*(*image).comps.offset(0i32 as
                                                          isize)).h);
        ret = 0i32
      }
    }
    1280 | 2048 => {
      if ((*(*image).comps.offset(0)).w
        > 4096u32) as libc::c_int
        | ((*(*image).comps.offset(0)).h
          > 3112u32) as libc::c_int
        != 0
      {
        opj_event_msg(p_manager, 2i32,
                              b"IMF 4K/4K_R profile require:\nwidth <= 4096 and height <= 3112\n-> Input image size %d x %d is not compliant\n-> Non-IMF codestream will be generated\n\x00"
                                  as *const u8 as *const libc::c_char,
                              (*(*image).comps.offset(0i32 as
                                                          isize)).w,
                              (*(*image).comps.offset(0i32 as
                                                          isize)).h);
        ret = 0i32
      }
    }
    1536 | 2304 => {
      if ((*(*image).comps.offset(0)).w
        > 8192u32) as libc::c_int
        | ((*(*image).comps.offset(0)).h
          > 6224u32) as libc::c_int
        != 0
      {
        opj_event_msg(p_manager, 2i32,
                              b"IMF 8K/8K_R profile require:\nwidth <= 8192 and height <= 6224\n-> Input image size %d x %d is not compliant\n-> Non-IMF codestream will be generated\n\x00"
                                  as *const u8 as *const libc::c_char,
                              (*(*image).comps.offset(0i32 as
                                                          isize)).w,
                              (*(*image).comps.offset(0i32 as
                                                          isize)).h);
        ret = 0i32
      }
    }
    _ => {
      panic!("Unknown OPJ_PROFILE");
      //C: assert(0);
    }
  }
  if (*parameters).roi_compno != -(1i32) {
    opj_event_msg(p_manager, 2i32,
                      b"IMF profile forbid RGN / region of interest marker.\n-> Compression parameters specify a ROI\n-> Non-IMF codestream will be generated\n\x00"
                          as *const u8 as *const libc::c_char);
    ret = 0i32
  }
  if (*parameters).cblockw_init != 32i32
    || (*parameters).cblockh_init != 32i32
  {
    opj_event_msg(p_manager, 2i32,
                      b"IMF profile require code block size to be 32x32.\n-> Compression parameters set it to %dx%d.\n-> Non-IMF codestream will be generated\n\x00"
                          as *const u8 as *const libc::c_char,
                      (*parameters).cblockw_init, (*parameters).cblockh_init);
    ret = 0i32
  }
  if (*parameters).prog_order as libc::c_int != OPJ_CPRL as libc::c_int {
    opj_event_msg(p_manager, 2i32,
                      b"IMF profile require progression order to be CPRL.\n-> Compression parameters set it to %d.\n-> Non-IMF codestream will be generated\n\x00"
                          as *const u8 as *const libc::c_char,
                      (*parameters).prog_order as libc::c_int);
    ret = 0i32
  }
  if (*parameters).numpocs != 0u32 {
    opj_event_msg(p_manager, 2i32,
                      b"IMF profile forbid POC markers.\n-> Compression parameters set %d POC.\n-> Non-IMF codestream will be generated\n\x00"
                          as *const u8 as *const libc::c_char,
                      (*parameters).numpocs);
    ret = 0i32
  }
  /* Codeblock style: no mode switch enabled */
  if (*parameters).mode != 0i32 {
    opj_event_msg(p_manager, 2i32,
                      b"IMF profile forbid mode switch in code block style.\n-> Compression parameters set code block style to %d.\n-> Non-IMF codestream will be generated\n\x00"
                          as *const u8 as *const libc::c_char,
                      (*parameters).mode);
    ret = 0i32
  }
  if profile as libc::c_int == 0x400i32
    || profile as libc::c_int == 0x500i32
    || profile as libc::c_int == 0x600i32
  {
    /* Expect 9-7 transform */
    if (*parameters).irreversible != 1i32 {
      opj_event_msg(p_manager, 2i32,
                          b"IMF 2K/4K/8K profiles require 9-7 Irreversible Transform.\n-> Compression parameters set it to reversible.\n-> Non-IMF codestream will be generated\n\x00"
                              as *const u8 as *const libc::c_char);
      ret = 0i32
    }
  } else if (*parameters).irreversible != 0i32 {
    opj_event_msg(p_manager, 2i32,
                      b"IMF 2K/4K/8K profiles require 5-3 reversible Transform.\n-> Compression parameters set it to irreversible.\n-> Non-IMF codestream will be generated\n\x00"
                          as *const u8 as *const libc::c_char);
    ret = 0i32
  }
  /* Expect 5-3 transform */
  /* Number of layers */
  if (*parameters).tcp_numlayers != 1i32 {
    opj_event_msg(p_manager, 2i32,
                      b"IMF 2K/4K/8K profiles require 1 single quality layer.\n-> Number of layers is %d.\n-> Non-IMF codestream will be generated\n\x00"
                          as *const u8 as *const libc::c_char,
                      (*parameters).tcp_numlayers);
    ret = 0i32
  }
  /* Decomposition levels */
  match profile as libc::c_int {
    1024 => {
      if !(NL >= 1i32 && NL <= 5i32) {
        opj_event_msg(p_manager, 2i32,
                              b"IMF 2K profile requires 1 <= NL <= 5:\n-> Number of decomposition levels is %d.\n-> Non-IMF codestream will be generated\n\x00"
                                  as *const u8 as *const libc::c_char, NL);
        ret = 0i32
      }
    }
    1280 => {
      if !(NL >= 1i32 && NL <= 6i32) {
        opj_event_msg(p_manager, 2i32,
                              b"IMF 4K profile requires 1 <= NL <= 6:\n-> Number of decomposition levels is %d.\n-> Non-IMF codestream will be generated\n\x00"
                                  as *const u8 as *const libc::c_char, NL);
        ret = 0i32
      }
    }
    1536 => {
      if !(NL >= 1i32 && NL <= 7i32) {
        opj_event_msg(p_manager, 2i32,
                              b"IMF 8K profile requires 1 <= NL <= 7:\n-> Number of decomposition levels is %d.\n-> Non-IMF codestream will be generated\n\x00"
                                  as *const u8 as *const libc::c_char, NL);
        ret = 0i32
      }
    }
    1792 => {
      if XTsiz >= 2048u32 {
        if !(NL >= 1i32 && NL <= 5i32) {
          opj_event_msg(p_manager, 2i32,
                                  b"IMF 2K_R profile requires 1 <= NL <= 5 for XTsiz >= 2048:\n-> Number of decomposition levels is %d.\n-> Non-IMF codestream will be generated\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  NL);
          ret = 0i32
        }
      } else if XTsiz >= 1024u32 {
        if !(NL >= 1i32 && NL <= 4i32) {
          opj_event_msg(p_manager, 2i32,
                                  b"IMF 2K_R profile requires 1 <= NL <= 4 for XTsiz in [1024,2048[:\n-> Number of decomposition levels is %d.\n-> Non-IMF codestream will be generated\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  NL);
          ret = 0i32
        }
      }
    }
    2048 => {
      if XTsiz >= 4096u32 {
        if !(NL >= 1i32 && NL <= 6i32) {
          opj_event_msg(p_manager, 2i32,
                                  b"IMF 4K_R profile requires 1 <= NL <= 6 for XTsiz >= 4096:\n-> Number of decomposition levels is %d.\n-> Non-IMF codestream will be generated\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  NL);
          ret = 0i32
        }
      } else if XTsiz >= 2048u32 {
        if !(NL >= 1i32 && NL <= 5i32) {
          opj_event_msg(p_manager, 2i32,
                                  b"IMF 4K_R profile requires 1 <= NL <= 5 for XTsiz in [2048,4096[:\n-> Number of decomposition levels is %d.\n-> Non-IMF codestream will be generated\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  NL);
          ret = 0i32
        }
      } else if XTsiz >= 1024u32 {
        if !(NL >= 1i32 && NL <= 4i32) {
          opj_event_msg(p_manager, 2i32,
                                  b"IMF 4K_R profile requires 1 <= NL <= 4 for XTsiz in [1024,2048[:\n-> Number of decomposition levels is %d.\n-> Non-IMF codestream will be generated\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  NL);
          ret = 0i32
        }
      }
    }
    2304 => {
      if XTsiz >= 8192u32 {
        if !(NL >= 1i32 && NL <= 7i32) {
          opj_event_msg(p_manager, 2i32,
                                  b"IMF 4K_R profile requires 1 <= NL <= 7 for XTsiz >= 8192:\n-> Number of decomposition levels is %d.\n-> Non-IMF codestream will be generated\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  NL);
          ret = 0i32
        }
      } else if XTsiz >= 4096u32 {
        if !(NL >= 1i32 && NL <= 6i32) {
          opj_event_msg(p_manager, 2i32,
                                  b"IMF 4K_R profile requires 1 <= NL <= 6 for XTsiz in [4096,8192[:\n-> Number of decomposition levels is %d.\n-> Non-IMF codestream will be generated\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  NL);
          ret = 0i32
        }
      } else if XTsiz >= 2048u32 {
        if !(NL >= 1i32 && NL <= 5i32) {
          opj_event_msg(p_manager, 2i32,
                                  b"IMF 4K_R profile requires 1 <= NL <= 5 for XTsiz in [2048,4096[:\n-> Number of decomposition levels is %d.\n-> Non-IMF codestream will be generated\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  NL);
          ret = 0i32
        }
      } else if XTsiz >= 1024u32 {
        if !(NL >= 1i32 && NL <= 4i32) {
          opj_event_msg(p_manager, 2i32,
                                  b"IMF 4K_R profile requires 1 <= NL <= 4 for XTsiz in [1024,2048[:\n-> Number of decomposition levels is %d.\n-> Non-IMF codestream will be generated\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  NL);
          ret = 0i32
        }
      }
    }
    _ => {}
  }
  if (*parameters).numresolution == 1i32 {
    if (*parameters).res_spec != 1i32
      || (*parameters).prcw_init[0 as usize] != 128i32
      || (*parameters).prch_init[0 as usize] != 128i32
    {
      opj_event_msg(p_manager, 2i32,
                          b"IMF profiles require PPx = PPy = 7 for NLLL band, else 8.\n-> Supplied values are different from that.\n-> Non-IMF codestream will be generated\n\x00"
                              as *const u8 as *const libc::c_char);
      ret = 0i32
    }
  } else {
    let mut i_0: libc::c_int = 0;
    i_0 = 0i32;
    while i_0 < (*parameters).res_spec {
      if (*parameters).prcw_init[i_0 as usize] != 256i32
        || (*parameters).prch_init[i_0 as usize] != 256i32
      {
        opj_event_msg(p_manager, 2i32,
                              b"IMF profiles require PPx = PPy = 7 for NLLL band, else 8.\n-> Supplied values are different from that.\n-> Non-IMF codestream will be generated\n\x00"
                                  as *const u8 as *const libc::c_char);
        ret = 0i32
      }
      i_0 += 1
    }
  }
  return ret;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_setup_encoder(
  mut p_j2k: *mut opj_j2k_t,
  mut parameters: *mut opj_cparameters_t,
  mut image: *mut opj_image_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut tileno: OPJ_UINT32 = 0;
  let mut numpocs_tile: OPJ_UINT32 = 0;
  let mut cp = 0 as *mut opj_cp_t;
  let mut cblkw: OPJ_UINT32 = 0;
  let mut cblkh: OPJ_UINT32 = 0;
  if p_j2k.is_null() || parameters.is_null() || image.is_null() {
    return 0i32;
  }
  if (*parameters).numresolution <= 0i32
    || (*parameters).numresolution > 33i32
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Invalid number of resolutions : %d not in range [1,%d]\n\x00" as *const u8
        as *const libc::c_char,
      (*parameters).numresolution,
      33i32,
    );
    return 0i32;
  }
  if (*parameters).cblockw_init < 4i32
    || (*parameters).cblockw_init > 1024i32
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Invalid value for cblockw_init: %d not a power of 2 in range [4,1024]\n\x00" as *const u8
        as *const libc::c_char,
      (*parameters).cblockw_init,
    );
    return 0i32;
  }
  if (*parameters).cblockh_init < 4i32
    || (*parameters).cblockh_init > 1024i32
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Invalid value for cblockh_init: %d not a power of 2 not in range [4,1024]\n\x00"
        as *const u8 as *const libc::c_char,
      (*parameters).cblockh_init,
    );
    return 0i32;
  }
  if (*parameters).cblockw_init * (*parameters).cblockh_init > 4096i32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Invalid value for cblockw_init * cblockh_init: should be <= 4096\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 0i32;
  }
  cblkw = opj_int_floorlog2((*parameters).cblockw_init) as OPJ_UINT32;
  cblkh = opj_int_floorlog2((*parameters).cblockh_init) as OPJ_UINT32;
  if (*parameters).cblockw_init != (1i32) << cblkw {
    opj_event_msg(
      p_manager,
      1i32,
      b"Invalid value for cblockw_init: %d not a power of 2 in range [4,1024]\n\x00" as *const u8
        as *const libc::c_char,
      (*parameters).cblockw_init,
    );
    return 0i32;
  }
  if (*parameters).cblockh_init != (1i32) << cblkh {
    opj_event_msg(
      p_manager,
      1i32,
      b"Invalid value for cblockw_init: %d not a power of 2 in range [4,1024]\n\x00" as *const u8
        as *const libc::c_char,
      (*parameters).cblockh_init,
    );
    return 0i32;
  }
  (*p_j2k).m_specific_param.m_encoder.m_nb_comps = (*image).numcomps;
  /* keep a link to cp so that we can destroy it later in j2k_destroy_compress */
  cp = &mut (*p_j2k).m_cp;
  /* set default values for cp */
  (*cp).tw = 1 as OPJ_UINT32;
  (*cp).th = 1 as OPJ_UINT32;
  /* FIXME ADE: to be removed once deprecated cp_cinema and cp_rsiz have been removed */
  if (*parameters).rsiz as libc::c_int == 0i32 {
    /* consider deprecated fields only if RSIZ has not been set */
    let mut deprecated_used = 0i32;
    match (*parameters).cp_cinema as libc::c_uint {
      1 => {
        (*parameters).rsiz = 0x3 as OPJ_UINT16;
        (*parameters).max_cs_size = 1302083i32;
        (*parameters).max_comp_size = 1041666i32;
        deprecated_used = 1i32
      }
      2 => {
        (*parameters).rsiz = 0x3 as OPJ_UINT16;
        (*parameters).max_cs_size = 651041i32;
        (*parameters).max_comp_size = 520833i32;
        deprecated_used = 1i32
      }
      3 => {
        (*parameters).rsiz = 0x4 as OPJ_UINT16;
        (*parameters).max_cs_size = 1302083i32;
        (*parameters).max_comp_size = 1041666i32;
        deprecated_used = 1i32
      }
      0 | _ => {}
    }
    match (*parameters).cp_rsiz as libc::c_uint {
      3 => {
        (*parameters).rsiz = 0x3 as OPJ_UINT16;
        deprecated_used = 1i32
      }
      4 => {
        (*parameters).rsiz = 0x4 as OPJ_UINT16;
        deprecated_used = 1i32
      }
      33024 => {
        (*parameters).rsiz = (0x8000i32 | 0x100i32) as OPJ_UINT16;
        deprecated_used = 1i32
      }
      0 | _ => {}
    }
    if deprecated_used != 0 {
      opj_event_msg(p_manager, 2i32,
                          b"Deprecated fields cp_cinema or cp_rsiz are used\nPlease consider using only the rsiz field\nSee openjpeg.h documentation for more details\n\x00"
                              as *const u8 as *const libc::c_char);
    }
  }
  /* If no explicit layers are provided, use lossless settings */
  if (*parameters).tcp_numlayers == 0i32 {
    (*parameters).tcp_numlayers = 1i32;
    (*parameters).cp_disto_alloc = 1i32;
    (*parameters).tcp_rates[0 as usize] = 0 as libc::c_float
  }
  if (*parameters).cp_disto_alloc != 0 {
    /* Emit warnings if tcp_rates are not decreasing */
    i = 1 as OPJ_UINT32;
    while i < (*parameters).tcp_numlayers as OPJ_UINT32 {
      let mut rate_i_corr = (*parameters).tcp_rates[i as usize];
      let mut rate_i_m_1_corr =
        (*parameters).tcp_rates[i.wrapping_sub(1u32) as usize];
      if rate_i_corr as libc::c_double <= 1.0f64 {
        rate_i_corr = 1.0f64 as OPJ_FLOAT32
      }
      if rate_i_m_1_corr as libc::c_double <= 1.0f64 {
        rate_i_m_1_corr = 1.0f64 as OPJ_FLOAT32
      }
      if rate_i_corr >= rate_i_m_1_corr {
        if rate_i_corr != (*parameters).tcp_rates[i as usize]
          && rate_i_m_1_corr
            != (*parameters).tcp_rates[i.wrapping_sub(1u32) as usize]
        {
          opj_event_msg(p_manager, 2i32,
                                  b"tcp_rates[%d]=%f (corrected as %f) should be strictly lesser than tcp_rates[%d]=%f (corrected as %f)\n\x00"
                                      as *const u8 as *const libc::c_char, i,
                                  (*parameters).tcp_rates[i as usize] as
                                      libc::c_double,
                                  rate_i_corr as libc::c_double,
                                  i.wrapping_sub(1i32 as
                                                     libc::c_uint),
                                  (*parameters).tcp_rates[i.wrapping_sub(1 as
                                                                             libc::c_int
                                                                             as
                                                                             libc::c_uint)
                                                              as usize] as
                                      libc::c_double,
                                  rate_i_m_1_corr as libc::c_double);
        } else if rate_i_corr != (*parameters).tcp_rates[i as usize] {
          opj_event_msg(p_manager, 2i32,
                                  b"tcp_rates[%d]=%f (corrected as %f) should be strictly lesser than tcp_rates[%d]=%f\n\x00"
                                      as *const u8 as *const libc::c_char, i,
                                  (*parameters).tcp_rates[i as usize] as
                                      libc::c_double,
                                  rate_i_corr as libc::c_double,
                                  i.wrapping_sub(1i32 as
                                                     libc::c_uint),
                                  (*parameters).tcp_rates[i.wrapping_sub(1 as
                                                                             libc::c_int
                                                                             as
                                                                             libc::c_uint)
                                                              as usize] as
                                      libc::c_double);
        } else if rate_i_m_1_corr
          != (*parameters).tcp_rates[i.wrapping_sub(1u32) as usize]
        {
          opj_event_msg(p_manager, 2i32,
                                  b"tcp_rates[%d]=%f should be strictly lesser than tcp_rates[%d]=%f (corrected as %f)\n\x00"
                                      as *const u8 as *const libc::c_char, i,
                                  (*parameters).tcp_rates[i as usize] as
                                      libc::c_double,
                                  i.wrapping_sub(1i32 as
                                                     libc::c_uint),
                                  (*parameters).tcp_rates[i.wrapping_sub(1 as
                                                                             libc::c_int
                                                                             as
                                                                             libc::c_uint)
                                                              as usize] as
                                      libc::c_double,
                                  rate_i_m_1_corr as libc::c_double);
        } else {
          opj_event_msg(
            p_manager,
            2i32,
            b"tcp_rates[%d]=%f should be strictly lesser than tcp_rates[%d]=%f\n\x00" as *const u8
              as *const libc::c_char,
            i,
            (*parameters).tcp_rates[i as usize] as libc::c_double,
            i.wrapping_sub(1u32),
            (*parameters).tcp_rates[i.wrapping_sub(1u32) as usize]
              as libc::c_double,
          );
        }
      }
      i = i.wrapping_add(1)
    }
  } else if (*parameters).cp_fixed_quality != 0 {
    /* Emit warnings if tcp_distoratio are not increasing */
    i = 1 as OPJ_UINT32;
    while i < (*parameters).tcp_numlayers as OPJ_UINT32 {
      if (*parameters).tcp_distoratio[i as usize]
        < (*parameters).tcp_distoratio[i.wrapping_sub(1u32) as usize]
        && !(i
          == ((*parameters).tcp_numlayers as OPJ_UINT32)
            .wrapping_sub(1u32)
          && (*parameters).tcp_distoratio[i as usize] == 0 as libc::c_float)
      {
        opj_event_msg(
          p_manager,
          2i32,
          b"tcp_distoratio[%d]=%f should be strictly greater than tcp_distoratio[%d]=%f\n\x00"
            as *const u8 as *const libc::c_char,
          i,
          (*parameters).tcp_distoratio[i as usize] as libc::c_double,
          i.wrapping_sub(1u32),
          (*parameters).tcp_distoratio[i.wrapping_sub(1u32) as usize]
            as libc::c_double,
        );
      }
      i = i.wrapping_add(1)
    }
  }
  /* see if max_codestream_size does limit input rate */
  if (*parameters).max_cs_size <= 0i32 {
    if (*parameters).tcp_rates[((*parameters).tcp_numlayers - 1i32) as usize]
      > 0 as libc::c_float
    {
      let mut temp_size: OPJ_FLOAT32 = 0.;
      temp_size = ((*image).numcomps as libc::c_double
        * (*(*image).comps.offset(0)).w as libc::c_double
        * (*(*image).comps.offset(0)).h as libc::c_double
        * (*(*image).comps.offset(0)).prec as libc::c_double
        / ((*parameters).tcp_rates[((*parameters).tcp_numlayers - 1i32) as usize]
          as libc::c_double
          * 8 as libc::c_double
          * (*(*image).comps.offset(0)).dx as libc::c_double
          * (*(*image).comps.offset(0)).dy as libc::c_double))
        as OPJ_FLOAT32;
      if temp_size > 2147483647 as libc::c_float {
        (*parameters).max_cs_size = 2147483647i32
      } else {
        (*parameters).max_cs_size = floor(temp_size as libc::c_double) as libc::c_int
      }
    } else {
      (*parameters).max_cs_size = 0i32
    }
  } else {
    let mut temp_rate: OPJ_FLOAT32 = 0.;
    let mut cap = 0i32;
    if (*parameters).rsiz as libc::c_int >= 0x400i32
      && (*parameters).rsiz as libc::c_int <= 0x900i32 | 0x9bi32
      && (*parameters).max_cs_size > 0i32
      && (*parameters).tcp_numlayers == 1i32
      && (*parameters).tcp_rates[0 as usize] == 0 as libc::c_float
    {
      (*parameters).tcp_rates[0 as usize] = (*image)
        .numcomps
        .wrapping_mul((*(*image).comps.offset(0)).w)
        .wrapping_mul((*(*image).comps.offset(0)).h)
        .wrapping_mul((*(*image).comps.offset(0)).prec)
        as OPJ_FLOAT32
        / ((*parameters).max_cs_size as OPJ_UINT32)
          .wrapping_mul(8u32)
          .wrapping_mul((*(*image).comps.offset(0)).dx)
          .wrapping_mul((*(*image).comps.offset(0)).dy)
          as OPJ_FLOAT32
    }
    temp_rate = ((*image).numcomps as libc::c_double
      * (*(*image).comps.offset(0)).w as libc::c_double
      * (*(*image).comps.offset(0)).h as libc::c_double
      * (*(*image).comps.offset(0)).prec as libc::c_double
      / ((*parameters).max_cs_size as libc::c_double
        * 8 as libc::c_double
        * (*(*image).comps.offset(0)).dx as libc::c_double
        * (*(*image).comps.offset(0)).dy as libc::c_double))
      as OPJ_FLOAT32;
    i = 0 as OPJ_UINT32;
    while i < (*parameters).tcp_numlayers as OPJ_UINT32 {
      if (*parameters).tcp_rates[i as usize] < temp_rate {
        (*parameters).tcp_rates[i as usize] = temp_rate;
        cap = 1i32
      }
      i = i.wrapping_add(1)
    }
    if cap != 0 {
      opj_event_msg(p_manager, 2i32,
                          b"The desired maximum codestream size has limited\nat least one of the desired quality layers\n\x00"
                              as *const u8 as *const libc::c_char);
    }
  }
  if (*parameters).rsiz as libc::c_int >= 0x3i32
    && (*parameters).rsiz as libc::c_int <= 0x6i32
    || (*parameters).rsiz as libc::c_int >= 0x400i32
      && (*parameters).rsiz as libc::c_int <= 0x900i32 | 0x9bi32
  {
    (*p_j2k).m_specific_param.m_encoder.m_TLM = 1i32
  }
  /* Manage profiles and applications and set RSIZ */
  /* set cinema parameters if required */
  if (*parameters).rsiz as libc::c_int >= 0x3i32
    && (*parameters).rsiz as libc::c_int <= 0x6i32
  {
    if (*parameters).rsiz as libc::c_int == 0x5i32
      || (*parameters).rsiz as libc::c_int == 0x6i32
    {
      opj_event_msg(
        p_manager,
        2i32,
        b"JPEG 2000 Scalable Digital Cinema profiles not yet supported\n\x00" as *const u8
          as *const libc::c_char,
      );
      (*parameters).rsiz = 0 as OPJ_UINT16
    } else {
      opj_j2k_set_cinema_parameters(parameters, image, p_manager);
      if opj_j2k_is_cinema_compliant(image, (*parameters).rsiz, p_manager) == 0 {
        (*parameters).rsiz = 0 as OPJ_UINT16
      }
    }
  } else if (*parameters).rsiz as libc::c_int == 0x7i32 {
    opj_event_msg(
      p_manager,
      2i32,
      b"JPEG 2000 Long Term Storage profile not yet supported\n\x00" as *const u8
        as *const libc::c_char,
    );
    (*parameters).rsiz = 0 as OPJ_UINT16
  } else if (*parameters).rsiz as libc::c_int >= 0x100i32
    && (*parameters).rsiz as libc::c_int <= 0x300i32 | 0xbi32
  {
    opj_event_msg(
      p_manager,
      2i32,
      b"JPEG 2000 Broadcast profiles not yet supported\n\x00" as *const u8 as *const libc::c_char,
    );
    (*parameters).rsiz = 0 as OPJ_UINT16
  } else if (*parameters).rsiz as libc::c_int >= 0x400i32
    && (*parameters).rsiz as libc::c_int <= 0x900i32 | 0x9bi32
  {
    opj_j2k_set_imf_parameters(parameters, image, p_manager);
    if opj_j2k_is_imf_compliant(parameters, image, p_manager) == 0 {
      (*parameters).rsiz = 0 as OPJ_UINT16
    }
  } else if (*parameters).rsiz as libc::c_int & 0x8000i32 != 0 {
    if (*parameters).rsiz as libc::c_int == 0x8000i32 | 0i32 {
      opj_event_msg(p_manager, 2i32,
                          b"JPEG 2000 Part-2 profile defined\nbut no Part-2 extension enabled.\nProfile set to NONE.\n\x00"
                              as *const u8 as *const libc::c_char);
      (*parameters).rsiz = 0 as OPJ_UINT16
    } else if (*parameters).rsiz as libc::c_int != 0x8000i32 | 0x100i32 {
      opj_event_msg(
        p_manager,
        2i32,
        b"Unsupported Part-2 extension enabled\nProfile set to NONE.\n\x00" as *const u8
          as *const libc::c_char,
      );
      (*parameters).rsiz = 0 as OPJ_UINT16
    }
  }
  /*
  copy user encoding parameters
  */
  (*cp).m_specific_param.m_enc.m_max_comp_size = (*parameters).max_comp_size as OPJ_UINT32;
  (*cp).rsiz = (*parameters).rsiz;
  (*cp)
    .m_specific_param
    .m_enc
    .set_m_disto_alloc((*parameters).cp_disto_alloc as OPJ_UINT32 & 1u32);
  (*cp)
    .m_specific_param
    .m_enc
    .set_m_fixed_alloc((*parameters).cp_fixed_alloc as OPJ_UINT32 & 1u32);
  (*cp)
    .m_specific_param
    .m_enc
    .set_m_fixed_quality((*parameters).cp_fixed_quality as OPJ_UINT32 & 1u32);
  /* mod fixed_quality */
  if (*parameters).cp_fixed_alloc != 0 && !(*parameters).cp_matrice.is_null() {
    let mut array_size = ((*parameters).tcp_numlayers as size_t)
      .wrapping_mul((*parameters).numresolution as size_t)
      .wrapping_mul(3u64)
      .wrapping_mul(core::mem::size_of::<OPJ_INT32>() as libc::c_ulong);
    (*cp).m_specific_param.m_enc.m_matrice = opj_malloc(array_size) as *mut OPJ_INT32;
    if (*cp).m_specific_param.m_enc.m_matrice.is_null() {
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to allocate copy of user encoding parameters matrix \n\x00" as *const u8
          as *const libc::c_char,
      );
      return 0i32;
    }
    memcpy(
      (*cp).m_specific_param.m_enc.m_matrice as *mut libc::c_void,
      (*parameters).cp_matrice as *const libc::c_void,
      array_size,
    );
  }
  /* tiles */
  (*cp).tdx = (*parameters).cp_tdx as OPJ_UINT32;
  (*cp).tdy = (*parameters).cp_tdy as OPJ_UINT32;
  /* tile offset */
  (*cp).tx0 = (*parameters).cp_tx0 as OPJ_UINT32;
  (*cp).ty0 = (*parameters).cp_ty0 as OPJ_UINT32;
  /* comment string */
  if !(*parameters).cp_comment.is_null() {
    (*cp).comment =
      opj_malloc(strlen((*parameters).cp_comment).wrapping_add(1u64))
        as *mut libc::c_char;
    if (*cp).comment.is_null() {
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to allocate copy of comment string\n\x00" as *const u8
          as *const libc::c_char,
      );
      return 0i32;
    }
    strcpy((*cp).comment, (*parameters).cp_comment);
  } else {
    /* Create default comment for codestream */
    let comment: [libc::c_char; 29] =
      *core::mem::transmute::<&[u8; 29], &[libc::c_char; 29]>(b"Created by OpenJPEG version \x00");
    let clen = strlen(comment.as_ptr());
    let mut version = opj_version();
    /* UniPG>> */
    (*cp).comment = opj_malloc(
      clen
        .wrapping_add(strlen(version))
        .wrapping_add(1u64),
    ) as *mut libc::c_char;
    if (*cp).comment.is_null() {
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to allocate comment string\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    sprintf(
      (*cp).comment,
      b"%s%s\x00" as *const u8 as *const libc::c_char,
      comment.as_ptr(),
      version,
    );
  }
  /*
  calculate other encoding parameters
  */
  if (*parameters).tile_size_on != 0 {
    if (*cp).tdx == 0u32 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Invalid tile width\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    if (*cp).tdy == 0u32 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Invalid tile height\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    (*cp).tw = opj_int_ceildiv(
      (*image).x1.wrapping_sub((*cp).tx0) as OPJ_INT32,
      (*cp).tdx as OPJ_INT32,
    ) as OPJ_UINT32;
    (*cp).th = opj_int_ceildiv(
      (*image).y1.wrapping_sub((*cp).ty0) as OPJ_INT32,
      (*cp).tdy as OPJ_INT32,
    ) as OPJ_UINT32;
    /* Check that the number of tiles is valid */
    if (*cp).tw > (65535u32).wrapping_div((*cp).th) {
      opj_event_msg(
        p_manager,
        1i32,
        b"Invalid number of tiles : %u x %u (maximum fixed by jpeg2000 norm is 65535 tiles)\n\x00"
          as *const u8 as *const libc::c_char,
        (*cp).tw,
        (*cp).th,
      );
      return 0i32;
    }
  } else {
    (*cp).tdx = (*image).x1.wrapping_sub((*cp).tx0);
    (*cp).tdy = (*image).y1.wrapping_sub((*cp).ty0)
  }
  if (*parameters).tp_on != 0 {
    (*cp).m_specific_param.m_enc.m_tp_flag = (*parameters).tp_flag as OPJ_BYTE;
    (*cp)
      .m_specific_param
      .m_enc
      .set_m_tp_on(1 as OPJ_BITFIELD)
  }
  /* USE_JPWL */
  /* initialize the multiple tiles */
  /* ---------------------------- */
  (*cp).tcps = opj_calloc(
    (*cp).tw.wrapping_mul((*cp).th) as size_t,
    core::mem::size_of::<opj_tcp_t>() as libc::c_ulong,
  ) as *mut opj_tcp_t;
  if (*cp).tcps.is_null() {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to allocate tile coding parameters\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 0i32;
  }
  tileno = 0 as OPJ_UINT32;
  while tileno < (*cp).tw.wrapping_mul((*cp).th) {
    let mut tcp: *mut opj_tcp_t = &mut *(*cp).tcps.offset(tileno as isize) as *mut opj_tcp_t;
    (*tcp).numlayers = (*parameters).tcp_numlayers as OPJ_UINT32;
    j = 0 as OPJ_UINT32;
    while j < (*tcp).numlayers {
      if (*cp).rsiz as libc::c_int >= 0x3i32
        && (*cp).rsiz as libc::c_int <= 0x6i32
        || (*cp).rsiz as libc::c_int >= 0x400i32
          && (*cp).rsiz as libc::c_int <= 0x900i32 | 0x9bi32
      {
        if (*cp).m_specific_param.m_enc.m_fixed_quality() != 0 {
          (*tcp).distoratio[j as usize] = (*parameters).tcp_distoratio[j as usize]
        }
        (*tcp).rates[j as usize] = (*parameters).tcp_rates[j as usize]
      } else if (*cp).m_specific_param.m_enc.m_fixed_quality() != 0 {
        /* add fixed_quality */
        (*tcp).distoratio[j as usize] = (*parameters).tcp_distoratio[j as usize]
      } else {
        (*tcp).rates[j as usize] = (*parameters).tcp_rates[j as usize]
      }
      if (*cp).m_specific_param.m_enc.m_fixed_quality() == 0
        && (*tcp).rates[j as usize] as libc::c_double <= 1.0f64
      {
        (*tcp).rates[j as usize] = 0.0f64 as OPJ_FLOAT32
        /* force lossless */
      }
      j = j.wrapping_add(1)
    }
    (*tcp).csty = (*parameters).csty as OPJ_UINT32;
    (*tcp).prg = (*parameters).prog_order;
    (*tcp).mct = (*parameters).tcp_mct as OPJ_UINT32;
    numpocs_tile = 0 as OPJ_UINT32;
    (*tcp).set_POC(0 as OPJ_BITFIELD);
    if (*parameters).numpocs != 0 {
      /* initialisation of POC */
      i = 0 as OPJ_UINT32;
      while i < (*parameters).numpocs {
        if tileno.wrapping_add(1u32)
          == (*parameters).POC[i as usize].tile
        {
          let mut tcp_poc: *mut opj_poc_t =
            &mut *(*tcp).pocs.as_mut_ptr().offset(numpocs_tile as isize) as *mut opj_poc_t;
          if (*parameters).POC[numpocs_tile as usize].compno0 >= (*image).numcomps {
            opj_event_msg(
              p_manager,
              1i32,
              b"Invalid compno0 for POC %d\n\x00" as *const u8 as *const libc::c_char,
              i,
            );
            return 0i32;
          }
          (*tcp_poc).resno0 = (*parameters).POC[numpocs_tile as usize].resno0;
          (*tcp_poc).compno0 = (*parameters).POC[numpocs_tile as usize].compno0;
          (*tcp_poc).layno1 = (*parameters).POC[numpocs_tile as usize].layno1;
          (*tcp_poc).resno1 = (*parameters).POC[numpocs_tile as usize].resno1;
          (*tcp_poc).compno1 = opj_uint_min(
            (*parameters).POC[numpocs_tile as usize].compno1,
            (*image).numcomps,
          );
          (*tcp_poc).prg1 = (*parameters).POC[numpocs_tile as usize].prg1;
          (*tcp_poc).tile = (*parameters).POC[numpocs_tile as usize].tile;
          numpocs_tile = numpocs_tile.wrapping_add(1)
        }
        i = i.wrapping_add(1)
      }
      if numpocs_tile != 0 {
        /* TODO MSD use the return value*/
        opj_j2k_check_poc_val(
          (*parameters).POC.as_mut_ptr(),
          tileno,
          (*parameters).numpocs,
          (*parameters).numresolution as OPJ_UINT32,
          (*image).numcomps,
          (*parameters).tcp_numlayers as OPJ_UINT32,
          p_manager,
        );
        (*tcp).set_POC(1 as OPJ_BITFIELD);
        (*tcp).numpocs = numpocs_tile.wrapping_sub(1u32)
      }
    } else {
      (*tcp).numpocs = 0 as OPJ_UINT32
    }
    (*tcp).tccps = opj_calloc(
      (*image).numcomps as size_t,
      core::mem::size_of::<opj_tccp_t>() as libc::c_ulong,
    ) as *mut opj_tccp_t;
    if (*tcp).tccps.is_null() {
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to allocate tile component coding parameters\n\x00" as *const u8
          as *const libc::c_char,
      );
      return 0i32;
    }
    if !(*parameters).mct_data.is_null() {
      let mut lMctSize = (*image)
        .numcomps
        .wrapping_mul((*image).numcomps)
        .wrapping_mul(core::mem::size_of::<OPJ_FLOAT32>() as OPJ_UINT32);
      let mut lTmpBuf = opj_malloc(lMctSize as size_t) as *mut OPJ_FLOAT32;
      let mut l_dc_shift =
        ((*parameters).mct_data as *mut OPJ_BYTE).offset(lMctSize as isize) as *mut OPJ_INT32;
      if lTmpBuf.is_null() {
        opj_event_msg(
          p_manager,
          1i32,
          b"Not enough memory to allocate temp buffer\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
      }
      (*tcp).mct = 2 as OPJ_UINT32;
      (*tcp).m_mct_coding_matrix = opj_malloc(lMctSize as size_t) as *mut OPJ_FLOAT32;
      if (*tcp).m_mct_coding_matrix.is_null() {
        opj_free(lTmpBuf as *mut libc::c_void);
        lTmpBuf = 0 as *mut OPJ_FLOAT32;
        opj_event_msg(
          p_manager,
          1i32,
          b"Not enough memory to allocate encoder MCT coding matrix \n\x00" as *const u8
            as *const libc::c_char,
        );
        return 0i32;
      }
      memcpy(
        (*tcp).m_mct_coding_matrix as *mut libc::c_void,
        (*parameters).mct_data,
        lMctSize as libc::c_ulong,
      );
      memcpy(
        lTmpBuf as *mut libc::c_void,
        (*parameters).mct_data,
        lMctSize as libc::c_ulong,
      );
      (*tcp).m_mct_decoding_matrix = opj_malloc(lMctSize as size_t) as *mut OPJ_FLOAT32;
      if (*tcp).m_mct_decoding_matrix.is_null() {
        opj_free(lTmpBuf as *mut libc::c_void);
        lTmpBuf = 0 as *mut OPJ_FLOAT32;
        opj_event_msg(
          p_manager,
          1i32,
          b"Not enough memory to allocate encoder MCT decoding matrix \n\x00" as *const u8
            as *const libc::c_char,
        );
        return 0i32;
      }
      if opj_matrix_inversion_f(lTmpBuf, (*tcp).m_mct_decoding_matrix, (*image).numcomps)
        == 0i32
      {
        opj_free(lTmpBuf as *mut libc::c_void);
        lTmpBuf = 0 as *mut OPJ_FLOAT32;
        opj_event_msg(
          p_manager,
          1i32,
          b"Failed to inverse encoder MCT decoding matrix \n\x00" as *const u8
            as *const libc::c_char,
        );
        return 0i32;
      }
      (*tcp).mct_norms = opj_malloc(
        ((*image).numcomps as libc::c_ulong)
          .wrapping_mul(core::mem::size_of::<OPJ_FLOAT64>() as libc::c_ulong),
      ) as *mut OPJ_FLOAT64;
      if (*tcp).mct_norms.is_null() {
        opj_free(lTmpBuf as *mut libc::c_void);
        lTmpBuf = 0 as *mut OPJ_FLOAT32;
        opj_event_msg(
          p_manager,
          1i32,
          b"Not enough memory to allocate encoder MCT norms \n\x00" as *const u8
            as *const libc::c_char,
        );
        return 0i32;
      }
      opj_calculate_norms(
        (*tcp).mct_norms,
        (*image).numcomps,
        (*tcp).m_mct_decoding_matrix,
      );
      opj_free(lTmpBuf as *mut libc::c_void);
      i = 0 as OPJ_UINT32;
      while i < (*image).numcomps {
        let mut tccp: *mut opj_tccp_t = &mut *(*tcp).tccps.offset(i as isize) as *mut opj_tccp_t;
        (*tccp).m_dc_level_shift = *l_dc_shift.offset(i as isize);
        i = i.wrapping_add(1)
      }
      if opj_j2k_setup_mct_encoding(tcp, image) == 0i32 {
        /* free will be handled by opj_j2k_destroy */
        opj_event_msg(
          p_manager,
          1i32,
          b"Failed to setup j2k mct encoding\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
      }
    } else {
      if (*tcp).mct == 1u32
        && (*image).numcomps >= 3u32
      {
        /* RGB->YCC MCT is enabled */
        if (*(*image).comps.offset(0)).dx
          != (*(*image).comps.offset(1)).dx
          || (*(*image).comps.offset(0)).dx
            != (*(*image).comps.offset(2)).dx
          || (*(*image).comps.offset(0)).dy
            != (*(*image).comps.offset(1)).dy
          || (*(*image).comps.offset(0)).dy
            != (*(*image).comps.offset(2)).dy
        {
          opj_event_msg(
            p_manager,
            2i32,
            b"Cannot perform MCT on components with different sizes. Disabling MCT.\n\x00"
              as *const u8 as *const libc::c_char,
          ); /* 0 => one precinct || 1 => custom precinct  */
          (*tcp).mct = 0 as OPJ_UINT32
        }
      }
      i = 0 as OPJ_UINT32;
      while i < (*image).numcomps {
        let mut tccp_0: *mut opj_tccp_t = &mut *(*tcp).tccps.offset(i as isize) as *mut opj_tccp_t;
        let mut l_comp: *mut opj_image_comp_t =
          &mut *(*image).comps.offset(i as isize) as *mut opj_image_comp_t;
        if (*l_comp).sgnd == 0 {
          (*tccp_0).m_dc_level_shift = (1i32)
            << (*l_comp)
              .prec
              .wrapping_sub(1u32)
        }
        i = i.wrapping_add(1)
      }
    }
    i = 0 as OPJ_UINT32;
    while i < (*image).numcomps {
      let mut tccp_1: *mut opj_tccp_t = &mut *(*tcp).tccps.offset(i as isize) as *mut opj_tccp_t;
      (*tccp_1).csty = ((*parameters).csty & 0x1i32) as OPJ_UINT32;
      (*tccp_1).numresolutions = (*parameters).numresolution as OPJ_UINT32;
      (*tccp_1).cblkw = opj_int_floorlog2((*parameters).cblockw_init) as OPJ_UINT32;
      (*tccp_1).cblkh = opj_int_floorlog2((*parameters).cblockh_init) as OPJ_UINT32;
      (*tccp_1).cblksty = (*parameters).mode as OPJ_UINT32;
      (*tccp_1).qmfbid = if (*parameters).irreversible != 0 {
        0i32
      } else {
        1i32
      } as OPJ_UINT32;
      (*tccp_1).qntsty = if (*parameters).irreversible != 0 {
        2i32
      } else {
        0i32
      } as OPJ_UINT32;
      (*tccp_1).numgbits = 2 as OPJ_UINT32;
      if i as OPJ_INT32 == (*parameters).roi_compno {
        (*tccp_1).roishift = (*parameters).roi_shift
      } else {
        (*tccp_1).roishift = 0i32
      }
      if (*parameters).csty & 0x1i32 != 0 {
        let mut p = 0i32;
        let mut it_res: OPJ_INT32 = 0;
        assert!((*tccp_1).numresolutions > 0u32);
        it_res = (*tccp_1).numresolutions as OPJ_INT32 - 1i32;
        while it_res >= 0i32 {
          if p < (*parameters).res_spec {
            if (*parameters).prcw_init[p as usize] < 1i32 {
              (*tccp_1).prcw[it_res as usize] = 1 as OPJ_UINT32
            } else {
              (*tccp_1).prcw[it_res as usize] =
                opj_int_floorlog2((*parameters).prcw_init[p as usize]) as OPJ_UINT32
            }
            if (*parameters).prch_init[p as usize] < 1i32 {
              (*tccp_1).prch[it_res as usize] = 1 as OPJ_UINT32
            } else {
              (*tccp_1).prch[it_res as usize] =
                opj_int_floorlog2((*parameters).prch_init[p as usize]) as OPJ_UINT32
            }
          } else {
            let mut res_spec = (*parameters).res_spec;
            let mut size_prcw = 0i32;
            let mut size_prch = 0i32;
            /*end for*/
            assert!(res_spec > 0i32);
            size_prcw = (*parameters).prcw_init[(res_spec - 1i32) as usize]
              >> p - (res_spec - 1i32);
            size_prch = (*parameters).prch_init[(res_spec - 1i32) as usize]
              >> p - (res_spec - 1i32);
            if size_prcw < 1i32 {
              (*tccp_1).prcw[it_res as usize] = 1 as OPJ_UINT32
            } else {
              (*tccp_1).prcw[it_res as usize] = opj_int_floorlog2(size_prcw) as OPJ_UINT32
            }
            if size_prch < 1i32 {
              (*tccp_1).prch[it_res as usize] = 1 as OPJ_UINT32
            } else {
              (*tccp_1).prch[it_res as usize] = opj_int_floorlog2(size_prch) as OPJ_UINT32
            }
          }
          p += 1;
          it_res -= 1
          /*printf("\nsize precinct for level %d : %d,%d\n", it_res,tccp->prcw[it_res], tccp->prch[it_res]); */
        }
      } else {
        j = 0 as OPJ_UINT32;
        while j < (*tccp_1).numresolutions {
          (*tccp_1).prcw[j as usize] = 15 as OPJ_UINT32;
          (*tccp_1).prch[j as usize] = 15 as OPJ_UINT32;
          j = j.wrapping_add(1)
        }
      }
      opj_dwt_calc_explicit_stepsizes(tccp_1, (*(*image).comps.offset(i as isize)).prec);
      i = i.wrapping_add(1)
    }
    tileno = tileno.wrapping_add(1)
  }
  if !(*parameters).mct_data.is_null() {
    opj_free((*parameters).mct_data);
    (*parameters).mct_data = 0 as *mut libc::c_void
  }
  return 1i32;
}
/* *
Add main header marker information
@param cstr_index    Codestream information structure
@param type         marker type
@param pos          byte offset of marker segment
@param len          length of marker segment
 */
unsafe fn opj_j2k_add_mhmarker(
  mut cstr_index: *mut opj_codestream_index_t,
  mut type_0: OPJ_UINT32,
  mut pos: OPJ_OFF_T,
  mut len: OPJ_UINT32,
) -> OPJ_BOOL {
  assert!(!cstr_index.is_null());
  /* expand the list? */
  if (*cstr_index)
    .marknum
    .wrapping_add(1u32)
    > (*cstr_index).maxmarknum
  {
    let mut new_marker = 0 as *mut opj_marker_info_t;
    (*cstr_index).maxmarknum =
      (100 as libc::c_float + (*cstr_index).maxmarknum as OPJ_FLOAT32) as OPJ_UINT32;
    new_marker = opj_realloc(
      (*cstr_index).marker as *mut libc::c_void,
      ((*cstr_index).maxmarknum as libc::c_ulong)
        .wrapping_mul(core::mem::size_of::<opj_marker_info_t>() as libc::c_ulong),
    ) as *mut opj_marker_info_t;
    if new_marker.is_null() {
      opj_free((*cstr_index).marker as *mut libc::c_void);
      (*cstr_index).marker = 0 as *mut opj_marker_info_t;
      (*cstr_index).maxmarknum = 0 as OPJ_UINT32;
      (*cstr_index).marknum = 0 as OPJ_UINT32;
      /* opj_event_msg(p_manager, EVT_ERROR, "Not enough memory to add mh marker\n"); */
      return 0i32;
    }
    (*cstr_index).marker = new_marker
  }
  /* add the marker */
  (*(*cstr_index).marker.offset((*cstr_index).marknum as isize)).type_0 = type_0 as OPJ_UINT16;
  (*(*cstr_index).marker.offset((*cstr_index).marknum as isize)).pos =
    pos as OPJ_OFF_T;
  (*(*cstr_index).marker.offset((*cstr_index).marknum as isize)).len = len as OPJ_INT32;
  (*cstr_index).marknum = (*cstr_index).marknum.wrapping_add(1);
  return 1i32;
}
/* *
Add tile header marker information
@param tileno       tile index number
@param cstr_index   Codestream information structure
@param type         marker type
@param pos          byte offset of marker segment
@param len          length of marker segment
 */
unsafe fn opj_j2k_add_tlmarker(
  mut tileno: OPJ_UINT32,
  mut cstr_index: *mut opj_codestream_index_t,
  mut type_0: OPJ_UINT32,
  mut pos: OPJ_OFF_T,
  mut len: OPJ_UINT32,
) -> OPJ_BOOL {
  assert!(!cstr_index.is_null());
  assert!(!(*cstr_index).tile_index.is_null());
  /* expand the list? */
  if (*(*cstr_index).tile_index.offset(tileno as isize))
    .marknum
    .wrapping_add(1u32)
    > (*(*cstr_index).tile_index.offset(tileno as isize)).maxmarknum
  {
    let mut new_marker = 0 as *mut opj_marker_info_t;
    (*(*cstr_index).tile_index.offset(tileno as isize)).maxmarknum = (100i32
      as libc::c_float
      + (*(*cstr_index).tile_index.offset(tileno as isize)).maxmarknum as OPJ_FLOAT32)
      as OPJ_UINT32;
    new_marker = opj_realloc(
      (*(*cstr_index).tile_index.offset(tileno as isize)).marker as *mut libc::c_void,
      ((*(*cstr_index).tile_index.offset(tileno as isize)).maxmarknum as libc::c_ulong)
        .wrapping_mul(core::mem::size_of::<opj_marker_info_t>() as libc::c_ulong),
    ) as *mut opj_marker_info_t;
    if new_marker.is_null() {
      opj_free((*(*cstr_index).tile_index.offset(tileno as isize)).marker as *mut libc::c_void);
      let ref mut fresh23 = (*(*cstr_index).tile_index.offset(tileno as isize)).marker;
      *fresh23 = 0 as *mut opj_marker_info_t;
      (*(*cstr_index).tile_index.offset(tileno as isize)).maxmarknum =
        0 as OPJ_UINT32;
      (*(*cstr_index).tile_index.offset(tileno as isize)).marknum = 0 as OPJ_UINT32;
      /* opj_event_msg(p_manager, EVT_ERROR, "Not enough memory to add tl marker\n"); */
      return 0i32;
    }
    let ref mut fresh24 = (*(*cstr_index).tile_index.offset(tileno as isize)).marker;
    *fresh24 = new_marker
  }
  /* add the marker */
  (*(*(*cstr_index).tile_index.offset(tileno as isize))
    .marker
    .offset((*(*cstr_index).tile_index.offset(tileno as isize)).marknum as isize))
  .type_0 = type_0 as OPJ_UINT16;
  (*(*(*cstr_index).tile_index.offset(tileno as isize))
    .marker
    .offset((*(*cstr_index).tile_index.offset(tileno as isize)).marknum as isize))
  .pos = pos as OPJ_OFF_T;
  (*(*(*cstr_index).tile_index.offset(tileno as isize))
    .marker
    .offset((*(*cstr_index).tile_index.offset(tileno as isize)).marknum as isize))
  .len = len as OPJ_INT32;
  let ref mut fresh25 = (*(*cstr_index).tile_index.offset(tileno as isize)).marknum;
  *fresh25 = (*fresh25).wrapping_add(1);
  if type_0 == 0xff90u32 {
    let mut l_current_tile_part = (*(*cstr_index).tile_index.offset(tileno as isize)).current_tpsno;
    if !(*(*cstr_index).tile_index.offset(tileno as isize))
      .tp_index
      .is_null()
    {
      (*(*(*cstr_index).tile_index.offset(tileno as isize))
        .tp_index
        .offset(l_current_tile_part as isize))
      .start_pos = pos
    }
  }
  return 1i32;
}
/*
 * -----------------------------------------------------------------------
 * -----------------------------------------------------------------------
 * -----------------------------------------------------------------------
 */
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_end_decompress(
  mut _p_j2k: *mut opj_j2k_t,
  mut _p_stream: *mut opj_stream_private_t,
  mut _p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_read_header(
  mut p_stream: *mut opj_stream_private_t,
  mut p_j2k: *mut opj_j2k_t,
  mut p_image: *mut *mut opj_image_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_stream.is_null());
  assert!(!p_manager.is_null());
  /* create an empty image header */
  (*p_j2k).m_private_image = opj_image_create0();
  if (*p_j2k).m_private_image.is_null() {
    return 0i32;
  }
  /* customization of the validation */
  if opj_j2k_setup_decoding_validation(p_j2k, p_manager) == 0 {
    opj_image_destroy((*p_j2k).m_private_image);
    (*p_j2k).m_private_image = 0 as *mut opj_image_t;
    return 0i32;
  }
  /* validation of the parameters codec */
  if opj_j2k_exec(p_j2k, (*p_j2k).m_validation_list, p_stream, p_manager) == 0 {
    opj_image_destroy((*p_j2k).m_private_image);
    (*p_j2k).m_private_image = 0 as *mut opj_image_t;
    return 0i32;
  }
  /* customization of the encoding */
  if opj_j2k_setup_header_reading(p_j2k, p_manager) == 0 {
    opj_image_destroy((*p_j2k).m_private_image);
    (*p_j2k).m_private_image = 0 as *mut opj_image_t;
    return 0i32;
  }
  /* read header */
  if opj_j2k_exec(p_j2k, (*p_j2k).m_procedure_list, p_stream, p_manager) == 0 {
    opj_image_destroy((*p_j2k).m_private_image);
    (*p_j2k).m_private_image = 0 as *mut opj_image_t;
    return 0i32;
  }
  *p_image = opj_image_create0();
  if (*p_image).is_null() {
    return 0i32;
  }
  /* Copy codestream image information to the output image */
  opj_copy_image_header((*p_j2k).m_private_image, *p_image);
  /*Allocate and initialize some elements of codestrem index*/
  if opj_j2k_allocate_tile_element_cstr_index(p_j2k) == 0 {
    opj_image_destroy(*p_image);
    *p_image = 0 as *mut opj_image_t;
    return 0i32;
  }
  return 1i32;
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
 * Copyright (c) 2008, Jerome Fimes, Communications & Systemes <jerome.fimes@c-s.fr>
 * Copyright (c) 2006-2007, Parvatha Elangovan
 * Copyright (c) 2010-2011, Kaori Hagihara
 * Copyright (c) 2011-2012, Centre National d'Etudes Spatiales (CNES), France
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
/* * @defgroup J2K J2K - JPEG-2000 codestream reader/writer */
/*@{*/
/* * @name Local static functions */
/*@{*/
/* *
 * Sets up the procedures to do on reading header. Developers wanting to extend the library can add their own reading procedures.
 */
unsafe fn opj_j2k_setup_header_reading(
  mut p_j2k: *mut opj_j2k_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions*/

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_read_header_procedure
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  /* DEVELOPER CORNER, add your custom procedures */
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_copy_default_tcp_and_create_tcd
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Sets up the validation ,i.e. adds the procedures to launch to make sure the codec parameters
 * are valid. Developers wanting to extend the library can add their own validation procedures.
 */
unsafe fn opj_j2k_setup_decoding_validation(
  mut p_j2k: *mut opj_j2k_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions*/

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_validation_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_build_decoder
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_validation_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_decoding_validation
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  /* DEVELOPER CORNER, add your custom validation procedure */
  return 1i32;
}
/* *
 * The mct encoding validation procedure.
 *
 * @param       p_j2k                   the jpeg2000 codec to validate.
 * @param       p_stream                                the input stream to validate.
 * @param       p_manager               the user event manager.
 *
 * @return true if the parameters are correct.
 */
unsafe extern "C" fn opj_j2k_mct_validation(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_is_valid = 1i32;
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_stream.is_null());
  assert!(!p_manager.is_null());
  if (*p_j2k).m_cp.rsiz as libc::c_int & 0x8200i32 == 0x8200i32 {
    let mut l_nb_tiles = (*p_j2k).m_cp.th.wrapping_mul((*p_j2k).m_cp.tw);
    let mut l_tcp = (*p_j2k).m_cp.tcps;
    i = 0 as OPJ_UINT32;
    while i < l_nb_tiles {
      if (*l_tcp).mct == 2u32 {
        let mut l_tccp = (*l_tcp).tccps;
        l_is_valid &= ((*l_tcp).m_mct_coding_matrix != 0 as *mut OPJ_FLOAT32) as libc::c_int;
        j = 0 as OPJ_UINT32;
        while j < (*(*p_j2k).m_private_image).numcomps {
          l_is_valid &= ((*l_tccp).qmfbid & 1u32 == 0) as libc::c_int;
          l_tccp = l_tccp.offset(1);
          j = j.wrapping_add(1)
        }
      }
      l_tcp = l_tcp.offset(1);
      i = i.wrapping_add(1)
    }
  }
  return l_is_valid;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_setup_mct_encoding(
  mut p_tcp: *mut opj_tcp_t,
  mut p_image: *mut opj_image_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_indix = 1 as OPJ_UINT32;
  let mut l_mct_deco_data = 0 as *mut opj_mct_data_t;
  let mut l_mct_offset_data = 0 as *mut opj_mct_data_t;
  let mut l_mcc_data = 0 as *mut opj_simple_mcc_decorrelation_data_t;
  let mut l_mct_size: OPJ_UINT32 = 0;
  let mut l_nb_elem: OPJ_UINT32 = 0;
  let mut l_data = 0 as *mut OPJ_FLOAT32;
  let mut l_current_data = 0 as *mut OPJ_FLOAT32;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  /* preconditions */
  assert!(!p_tcp.is_null());
  if (*p_tcp).mct != 2u32 {
    return 1i32;
  }
  if !(*p_tcp).m_mct_decoding_matrix.is_null() {
    if (*p_tcp).m_nb_mct_records == (*p_tcp).m_nb_max_mct_records {
      let mut new_mct_records = 0 as *mut opj_mct_data_t;
      (*p_tcp).m_nb_max_mct_records = ((*p_tcp).m_nb_max_mct_records as libc::c_uint)
        .wrapping_add(10u32)
        as OPJ_UINT32;
      new_mct_records = opj_realloc(
        (*p_tcp).m_mct_records as *mut libc::c_void,
        ((*p_tcp).m_nb_max_mct_records as libc::c_ulong)
          .wrapping_mul(core::mem::size_of::<opj_mct_data_t>() as libc::c_ulong),
      ) as *mut opj_mct_data_t;
      if new_mct_records.is_null() {
        opj_free((*p_tcp).m_mct_records as *mut libc::c_void);
        (*p_tcp).m_mct_records = 0 as *mut opj_mct_data_t;
        (*p_tcp).m_nb_max_mct_records = 0 as OPJ_UINT32;
        (*p_tcp).m_nb_mct_records = 0 as OPJ_UINT32;
        /* opj_event_msg(p_manager, EVT_ERROR, "Not enough memory to setup mct encoding\n"); */
        return 0i32;
      }
      (*p_tcp).m_mct_records = new_mct_records;
      l_mct_deco_data = (*p_tcp)
        .m_mct_records
        .offset((*p_tcp).m_nb_mct_records as isize);
      memset(
        l_mct_deco_data as *mut libc::c_void,
        0i32,
        ((*p_tcp)
          .m_nb_max_mct_records
          .wrapping_sub((*p_tcp).m_nb_mct_records) as libc::c_ulong)
          .wrapping_mul(core::mem::size_of::<opj_mct_data_t>() as libc::c_ulong),
      );
    }
    l_mct_deco_data = (*p_tcp)
      .m_mct_records
      .offset((*p_tcp).m_nb_mct_records as isize);
    if !(*l_mct_deco_data).m_data.is_null() {
      opj_free((*l_mct_deco_data).m_data as *mut libc::c_void);
      (*l_mct_deco_data).m_data = 0 as *mut OPJ_BYTE
    }
    let fresh26 = l_indix;
    l_indix = l_indix.wrapping_add(1);
    (*l_mct_deco_data).m_index = fresh26;
    (*l_mct_deco_data).m_array_type = MCT_TYPE_DECORRELATION;
    (*l_mct_deco_data).m_element_type = MCT_TYPE_FLOAT;
    l_nb_elem = (*p_image).numcomps.wrapping_mul((*p_image).numcomps);
    l_mct_size =
      l_nb_elem.wrapping_mul(MCT_ELEMENT_SIZE[(*l_mct_deco_data).m_element_type as usize]);
    (*l_mct_deco_data).m_data = opj_malloc(l_mct_size as size_t) as *mut OPJ_BYTE;
    if (*l_mct_deco_data).m_data.is_null() {
      return 0i32;
    }
    j2k_mct_write_functions_from_float[(*l_mct_deco_data).m_element_type as usize]
      .expect("non-null function pointer")(
      (*p_tcp).m_mct_decoding_matrix as *const libc::c_void,
      (*l_mct_deco_data).m_data as *mut libc::c_void,
      l_nb_elem,
    );
    (*l_mct_deco_data).m_data_size = l_mct_size;
    (*p_tcp).m_nb_mct_records = (*p_tcp).m_nb_mct_records.wrapping_add(1)
  }
  if (*p_tcp).m_nb_mct_records == (*p_tcp).m_nb_max_mct_records {
    let mut new_mct_records_0 = 0 as *mut opj_mct_data_t;
    (*p_tcp).m_nb_max_mct_records = ((*p_tcp).m_nb_max_mct_records as libc::c_uint)
      .wrapping_add(10u32)
      as OPJ_UINT32;
    new_mct_records_0 = opj_realloc(
      (*p_tcp).m_mct_records as *mut libc::c_void,
      ((*p_tcp).m_nb_max_mct_records as libc::c_ulong)
        .wrapping_mul(core::mem::size_of::<opj_mct_data_t>() as libc::c_ulong),
    ) as *mut opj_mct_data_t;
    if new_mct_records_0.is_null() {
      opj_free((*p_tcp).m_mct_records as *mut libc::c_void);
      (*p_tcp).m_mct_records = 0 as *mut opj_mct_data_t;
      (*p_tcp).m_nb_max_mct_records = 0 as OPJ_UINT32;
      (*p_tcp).m_nb_mct_records = 0 as OPJ_UINT32;
      /* opj_event_msg(p_manager, EVT_ERROR, "Not enough memory to setup mct encoding\n"); */
      return 0i32;
    }
    (*p_tcp).m_mct_records = new_mct_records_0;
    l_mct_offset_data = (*p_tcp)
      .m_mct_records
      .offset((*p_tcp).m_nb_mct_records as isize);
    memset(
      l_mct_offset_data as *mut libc::c_void,
      0i32,
      ((*p_tcp)
        .m_nb_max_mct_records
        .wrapping_sub((*p_tcp).m_nb_mct_records) as libc::c_ulong)
        .wrapping_mul(core::mem::size_of::<opj_mct_data_t>() as libc::c_ulong),
    );
    if !l_mct_deco_data.is_null() {
      l_mct_deco_data = l_mct_offset_data.offset(-1)
    }
  }
  l_mct_offset_data = (*p_tcp)
    .m_mct_records
    .offset((*p_tcp).m_nb_mct_records as isize);
  if !(*l_mct_offset_data).m_data.is_null() {
    opj_free((*l_mct_offset_data).m_data as *mut libc::c_void);
    (*l_mct_offset_data).m_data = 0 as *mut OPJ_BYTE
  }
  let fresh27 = l_indix;
  l_indix = l_indix.wrapping_add(1);
  (*l_mct_offset_data).m_index = fresh27;
  (*l_mct_offset_data).m_array_type = MCT_TYPE_OFFSET;
  (*l_mct_offset_data).m_element_type = MCT_TYPE_FLOAT;
  l_nb_elem = (*p_image).numcomps;
  l_mct_size =
    l_nb_elem.wrapping_mul(MCT_ELEMENT_SIZE[(*l_mct_offset_data).m_element_type as usize]);
  (*l_mct_offset_data).m_data = opj_malloc(l_mct_size as size_t) as *mut OPJ_BYTE;
  if (*l_mct_offset_data).m_data.is_null() {
    return 0i32;
  }
  l_data = opj_malloc(
    (l_nb_elem as libc::c_ulong)
      .wrapping_mul(core::mem::size_of::<OPJ_FLOAT32>() as libc::c_ulong),
  ) as *mut OPJ_FLOAT32;
  if l_data.is_null() {
    opj_free((*l_mct_offset_data).m_data as *mut libc::c_void);
    (*l_mct_offset_data).m_data = 0 as *mut OPJ_BYTE;
    return 0i32;
  }
  l_tccp = (*p_tcp).tccps;
  l_current_data = l_data;
  i = 0 as OPJ_UINT32;
  while i < l_nb_elem {
    let fresh28 = l_current_data;
    l_current_data = l_current_data.offset(1);
    *fresh28 = (*l_tccp).m_dc_level_shift as OPJ_FLOAT32;
    l_tccp = l_tccp.offset(1);
    i = i.wrapping_add(1)
  }
  j2k_mct_write_functions_from_float[(*l_mct_offset_data).m_element_type as usize]
    .expect("non-null function pointer")(
    l_data as *const libc::c_void,
    (*l_mct_offset_data).m_data as *mut libc::c_void,
    l_nb_elem,
  );
  opj_free(l_data as *mut libc::c_void);
  (*l_mct_offset_data).m_data_size = l_mct_size;
  (*p_tcp).m_nb_mct_records = (*p_tcp).m_nb_mct_records.wrapping_add(1);
  if (*p_tcp).m_nb_mcc_records == (*p_tcp).m_nb_max_mcc_records {
    let mut new_mcc_records = 0 as *mut opj_simple_mcc_decorrelation_data_t;
    (*p_tcp).m_nb_max_mcc_records = ((*p_tcp).m_nb_max_mcc_records as libc::c_uint)
      .wrapping_add(10u32)
      as OPJ_UINT32;
    new_mcc_records =
      opj_realloc(
        (*p_tcp).m_mcc_records as *mut libc::c_void,
        ((*p_tcp).m_nb_max_mcc_records as libc::c_ulong).wrapping_mul(core::mem::size_of::<
          opj_simple_mcc_decorrelation_data_t,
        >() as libc::c_ulong),
      ) as *mut opj_simple_mcc_decorrelation_data_t;
    if new_mcc_records.is_null() {
      opj_free((*p_tcp).m_mcc_records as *mut libc::c_void);
      (*p_tcp).m_mcc_records = 0 as *mut opj_simple_mcc_decorrelation_data_t;
      (*p_tcp).m_nb_max_mcc_records = 0 as OPJ_UINT32;
      (*p_tcp).m_nb_mcc_records = 0 as OPJ_UINT32;
      /* opj_event_msg(p_manager, EVT_ERROR, "Not enough memory to setup mct encoding\n"); */
      return 0i32;
    }
    (*p_tcp).m_mcc_records = new_mcc_records;
    l_mcc_data = (*p_tcp)
      .m_mcc_records
      .offset((*p_tcp).m_nb_mcc_records as isize);
    memset(
            l_mcc_data as *mut libc::c_void,
            0i32,
            ((*p_tcp)
                .m_nb_max_mcc_records
                .wrapping_sub((*p_tcp).m_nb_mcc_records) as libc::c_ulong)
                .wrapping_mul(
                    core::mem::size_of::<opj_simple_mcc_decorrelation_data_t>() as libc::c_ulong
                ),
        );
  }
  l_mcc_data = (*p_tcp)
    .m_mcc_records
    .offset((*p_tcp).m_nb_mcc_records as isize);
  (*l_mcc_data).m_decorrelation_array = l_mct_deco_data;
  (*l_mcc_data).set_m_is_irreversible(1 as OPJ_BITFIELD);
  (*l_mcc_data).m_nb_comps = (*p_image).numcomps;
  let fresh29 = l_indix;
  l_indix = l_indix.wrapping_add(1);
  (*l_mcc_data).m_index = fresh29;
  (*l_mcc_data).m_offset_array = l_mct_offset_data;
  (*p_tcp).m_nb_mcc_records = (*p_tcp).m_nb_mcc_records.wrapping_add(1);
  return 1i32;
}
/* *
 * Builds the tcd decoder to use to decode tile.
 */
unsafe extern "C" fn opj_j2k_build_decoder(
  mut _p_j2k: *mut opj_j2k_t,
  mut _p_stream: *mut opj_stream_private_t,
  mut _p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* add here initialization of cp
  copy paste of setup_decoder */
  return 1i32;
}
/* *
 * Builds the tcd encoder to use to encode tile.
 */
unsafe extern "C" fn opj_j2k_build_encoder(
  mut _p_j2k: *mut opj_j2k_t,
  mut _p_stream: *mut opj_stream_private_t,
  mut _p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* add here initialization of cp
  copy paste of setup_encoder */
  return 1i32;
}
/* *
 * The default encoding validation procedure without any extension.
 *
 * @param       p_j2k                   the jpeg2000 codec to validate.
 * @param       p_stream                the input stream to validate.
 * @param       p_manager               the user event manager.
 *
 * @return true if the parameters are correct.
 */
unsafe extern "C" fn opj_j2k_encoding_validation(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_is_valid = 1i32;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_stream.is_null());
  assert!(!p_manager.is_null());
  /* STATE checking */
  /* make sure the state is at 0 */
  l_is_valid &= ((*p_j2k).m_specific_param.m_decoder.m_state
    == J2KState::NONE) as libc::c_int;
  /* POINTER validation */
  /* make sure a p_j2k codec is present */
  l_is_valid &= ((*p_j2k).m_procedure_list != 0 as *mut opj_procedure_list_t) as libc::c_int;
  /* make sure a validation list is present */
  l_is_valid &= ((*p_j2k).m_validation_list != 0 as *mut opj_procedure_list_t) as libc::c_int;
  /* ISO 15444-1:2004 states between 1 & 33 (0 -> 32) */
  /* 33 (32) would always fail the check below (if a cast to 64bits was done) */
  /* FIXME Shall we change OPJ_J2K_MAXRLVLS to 32 ? */
  if (*(*(*p_j2k).m_cp.tcps).tccps).numresolutions <= 0u32
    || (*(*(*p_j2k).m_cp.tcps).tccps).numresolutions > 32u32
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Number of resolutions is too high in comparison to the size of tiles\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 0i32;
  }
  if (*p_j2k).m_cp.tdx
    < ((1i32)
      << (*(*(*p_j2k).m_cp.tcps).tccps)
        .numresolutions
        .wrapping_sub(1u32)) as OPJ_UINT32
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Number of resolutions is too high in comparison to the size of tiles\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 0i32;
  }
  if (*p_j2k).m_cp.tdy
    < ((1i32)
      << (*(*(*p_j2k).m_cp.tcps).tccps)
        .numresolutions
        .wrapping_sub(1u32)) as OPJ_UINT32
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Number of resolutions is too high in comparison to the size of tiles\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 0i32;
  }
  /* PARAMETER VALIDATION */
  return l_is_valid;
}
/* *
 * The default decoding validation procedure without any extension.
 *
 * @param       p_j2k                   the jpeg2000 codec to validate.
 * @param       p_stream                                the input stream to validate.
 * @param       p_manager               the user event manager.
 *
 * @return true if the parameters are correct.
 */
unsafe extern "C" fn opj_j2k_decoding_validation(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_is_valid = 1i32;
  /* preconditions*/

  assert!(!p_j2k.is_null());
  assert!(!p_stream.is_null());
  assert!(!p_manager.is_null());
  /* STATE checking */
  /* make sure the state is at 0 */
  l_is_valid &= ((*p_j2k).m_specific_param.m_decoder.m_state == J2KState::NONE)
    as libc::c_int;
  /* POINTER validation */
  /* make sure a p_j2k codec is present */
  /* make sure a procedure list is present */
  l_is_valid &= ((*p_j2k).m_procedure_list != 0 as *mut opj_procedure_list_t) as libc::c_int;
  /* make sure a validation list is present */
  l_is_valid &= ((*p_j2k).m_validation_list != 0 as *mut opj_procedure_list_t) as libc::c_int;
  /* PARAMETER VALIDATION */
  return l_is_valid;
}
/* *
 * The read header procedure.
 */
unsafe extern "C" fn opj_j2k_read_header_procedure(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_current_marker: OPJ_UINT32 = 0;
  let mut l_marker_size: OPJ_UINT32 = 0;
  let mut l_marker_handler = 0 as *const opj_dec_memory_marker_handler_t;
  let mut l_has_siz = 0i32;
  let mut l_has_cod = 0i32;
  let mut l_has_qcd = 0i32;
  /* preconditions */

  assert!(!p_stream.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  /*  We enter in the main header */
  (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::MHSOC;
  /* Try to read the SOC marker, the codestream must begin with SOC marker */
  if opj_j2k_read_soc(p_j2k, p_stream, p_manager) == 0 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Expected a SOC marker \n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* Try to read 2 bytes (the next marker ID) from stream and copy them into the buffer */
  if opj_stream_read_data(
    p_stream,
    (*p_j2k).m_specific_param.m_decoder.m_header_data,
    2 as OPJ_SIZE_T,
    p_manager,
  ) != 2u64
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* Read 2 bytes as the new marker ID */
  opj_read_bytes_LE(
    (*p_j2k).m_specific_param.m_decoder.m_header_data,
    &mut l_current_marker,
    2 as OPJ_UINT32,
  );
  /* Try to read until the SOT is detected */
  while l_current_marker != 0xff90u32 {
    /* Check if the current marker ID is valid */
    if l_current_marker < 0xff00u32 {
      opj_event_msg(
        p_manager,
        1i32,
        b"A marker ID was expected (0xff--) instead of %.8x\n\x00" as *const u8
          as *const libc::c_char,
        l_current_marker,
      );
      return 0i32;
    }
    /* Get the marker handler from the marker ID */
    l_marker_handler = opj_j2k_get_marker_handler(l_current_marker);
    /* Manage case where marker is unknown */
    if (*l_marker_handler).id == 0u32 {
      if opj_j2k_read_unk(p_j2k, p_stream, &mut l_current_marker, p_manager) == 0 {
        opj_event_msg(
          p_manager,
          1i32,
          b"Unknown marker has been detected and generated error.\n\x00" as *const u8
            as *const libc::c_char,
        );
        return 0i32;
      }
      if l_current_marker == 0xff90u32 {
        break;
      }
      l_marker_handler = opj_j2k_get_marker_handler(l_current_marker)
    }
    if (*l_marker_handler).id == 0xff51u32 {
      /* Mark required SIZ marker as found */
      l_has_siz = 1i32
    }
    if (*l_marker_handler).id == 0xff52u32 {
      /* Mark required COD marker as found */
      l_has_cod = 1i32
    }
    if (*l_marker_handler).id == 0xff5cu32 {
      /* Mark required QCD marker as found */
      l_has_qcd = 1i32
    }
    /* Check if the marker is known and if it is the right place to find it */
    if (*p_j2k).m_specific_param.m_decoder.m_state & (*l_marker_handler).states == J2KState::NONE {
      opj_event_msg(
        p_manager,
        1i32,
        b"Marker is not compliant with its position\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    /* Try to read 2 bytes (the marker size) from stream and copy them into the buffer */
    if opj_stream_read_data(
      p_stream,
      (*p_j2k).m_specific_param.m_decoder.m_header_data,
      2 as OPJ_SIZE_T,
      p_manager,
    ) != 2u64
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    /* read 2 bytes as the marker size */
    opj_read_bytes_LE(
      (*p_j2k).m_specific_param.m_decoder.m_header_data,
      &mut l_marker_size,
      2 as OPJ_UINT32,
    ); /* Subtract the size of the marker ID already read */
    if l_marker_size < 2u32 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Invalid marker size\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    l_marker_size = (l_marker_size as libc::c_uint).wrapping_sub(2u32)
      as OPJ_UINT32;
    /* Check if the marker size is compatible with the header data size */
    if l_marker_size > (*p_j2k).m_specific_param.m_decoder.m_header_data_size {
      let mut new_header_data = opj_realloc(
        (*p_j2k).m_specific_param.m_decoder.m_header_data as *mut libc::c_void,
        l_marker_size as size_t,
      ) as *mut OPJ_BYTE;
      if new_header_data.is_null() {
        opj_free((*p_j2k).m_specific_param.m_decoder.m_header_data as *mut libc::c_void);
        (*p_j2k).m_specific_param.m_decoder.m_header_data = 0 as *mut OPJ_BYTE;
        (*p_j2k).m_specific_param.m_decoder.m_header_data_size = 0 as OPJ_UINT32;
        opj_event_msg(
          p_manager,
          1i32,
          b"Not enough memory to read header\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
      }
      (*p_j2k).m_specific_param.m_decoder.m_header_data = new_header_data;
      (*p_j2k).m_specific_param.m_decoder.m_header_data_size = l_marker_size
    }
    /* Try to read the rest of the marker segment from stream and copy them into the buffer */
    if opj_stream_read_data(
      p_stream,
      (*p_j2k).m_specific_param.m_decoder.m_header_data,
      l_marker_size as OPJ_SIZE_T,
      p_manager,
    ) != l_marker_size as libc::c_ulong
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    /* Read the marker segment with the correct marker handler */
    if Some(
      (*l_marker_handler)
        .handler
        .expect("non-null function pointer"),
    )
    .expect("non-null function pointer")(
      p_j2k,
      (*p_j2k).m_specific_param.m_decoder.m_header_data,
      l_marker_size,
      p_manager,
    ) == 0
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Marker handler function failed to read the marker segment\n\x00" as *const u8
          as *const libc::c_char,
      );
      return 0i32;
    }
    /* Add the marker to the codestream index*/
    if 0i32
      == opj_j2k_add_mhmarker(
        (*p_j2k).cstr_index,
        (*l_marker_handler).id,
        (opj_stream_tell(p_stream) as OPJ_UINT32)
          .wrapping_sub(l_marker_size)
          .wrapping_sub(4u32) as OPJ_OFF_T,
        l_marker_size.wrapping_add(4u32),
      )
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to add mh marker\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    /* Try to read 2 bytes (the next marker ID) from stream and copy them into the buffer */
    if opj_stream_read_data(
      p_stream,
      (*p_j2k).m_specific_param.m_decoder.m_header_data,
      2 as OPJ_SIZE_T,
      p_manager,
    ) != 2u64
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    /* read 2 bytes as the new marker ID */
    opj_read_bytes_LE(
      (*p_j2k).m_specific_param.m_decoder.m_header_data,
      &mut l_current_marker,
      2 as OPJ_UINT32,
    );
  }
  if l_has_siz == 0i32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"required SIZ marker not found in main header\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if l_has_cod == 0i32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"required COD marker not found in main header\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if l_has_qcd == 0i32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"required QCD marker not found in main header\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if opj_j2k_merge_ppm(&mut (*p_j2k).m_cp, p_manager) == 0 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Failed to merge PPM data\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  opj_event_msg(
    p_manager,
    4i32,
    b"Main header has been correctly decoded.\n\x00" as *const u8 as *const libc::c_char,
  );
  /* Position of the last element if the main header */
  (*(*p_j2k).cstr_index).main_head_end = (opj_stream_tell(p_stream) as OPJ_UINT32)
    .wrapping_sub(2u32)
    as OPJ_OFF_T;
  /* Next step: read a tile-part header */
  (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::TPHSOT;
  return 1i32;
}
/* *
 * Executes the given procedures on the given codec.
 *
 * @param       p_procedure_list        the list of procedures to execute
 * @param       p_j2k                           the jpeg2000 codec to execute the procedures on.
 * @param       p_stream                        the stream to execute the procedures on.
 * @param       p_manager                       the user manager.
 *
 * @return      true                            if all the procedures were successfully executed.
 */
unsafe fn opj_j2k_exec(
  mut p_j2k: *mut opj_j2k_t,
  mut p_procedure_list: *mut opj_procedure_list_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_procedure = 0 as *mut Option<
    unsafe extern "C" fn(
      _: *mut opj_j2k_t,
      _: *mut opj_stream_private_t,
      _: *mut opj_event_mgr_t,
    ) -> OPJ_BOOL,
  >;
  let mut l_result = 1i32;
  let mut l_nb_proc: OPJ_UINT32 = 0;
  let mut i: OPJ_UINT32 = 0;
  /* preconditions*/

  assert!(!p_procedure_list.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_stream.is_null());
  assert!(!p_manager.is_null());
  l_nb_proc = opj_procedure_list_get_nb_procedures(p_procedure_list);
  l_procedure = opj_procedure_list_get_first_procedure(p_procedure_list)
    as *mut Option<
      unsafe extern "C" fn(
        _: *mut opj_j2k_t,
        _: *mut opj_stream_private_t,
        _: *mut opj_event_mgr_t,
      ) -> OPJ_BOOL,
    >;
  i = 0 as OPJ_UINT32;
  while i < l_nb_proc {
    l_result = (l_result != 0
      && (*l_procedure).expect("non-null function pointer")(p_j2k, p_stream, p_manager) != 0)
      as libc::c_int;
    l_procedure = l_procedure.offset(1);
    i = i.wrapping_add(1)
  }
  /* and clear the procedure list at the end.*/
  opj_procedure_list_clear(p_procedure_list);
  return l_result;
}
/* *
 * Copies the decoding tile parameters onto all the tile parameters.
 * Creates also the tile decoder.
 */
/* FIXME DOC*/
unsafe extern "C" fn opj_j2k_copy_default_tcp_and_create_tcd(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_default_tcp = 0 as *mut opj_tcp_t;
  let mut l_nb_tiles: OPJ_UINT32 = 0;
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut l_current_tccp = 0 as *mut opj_tccp_t;
  let mut l_tccp_size: OPJ_UINT32 = 0;
  let mut l_mct_size: OPJ_UINT32 = 0;
  let mut l_image = 0 as *mut opj_image_t;
  let mut l_mcc_records_size: OPJ_UINT32 = 0;
  let mut l_mct_records_size: OPJ_UINT32 = 0;
  let mut l_src_mct_rec = 0 as *mut opj_mct_data_t;
  let mut l_dest_mct_rec = 0 as *mut opj_mct_data_t;
  let mut l_src_mcc_rec = 0 as *mut opj_simple_mcc_decorrelation_data_t;
  let mut l_dest_mcc_rec = 0 as *mut opj_simple_mcc_decorrelation_data_t;
  let mut l_offset: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_stream.is_null());
  assert!(!p_manager.is_null());
  l_image = (*p_j2k).m_private_image;
  l_nb_tiles = (*p_j2k).m_cp.th.wrapping_mul((*p_j2k).m_cp.tw);
  l_tcp = (*p_j2k).m_cp.tcps;
  l_tccp_size = (*l_image)
    .numcomps
    .wrapping_mul(core::mem::size_of::<opj_tccp_t>() as OPJ_UINT32);
  l_default_tcp = (*p_j2k).m_specific_param.m_decoder.m_default_tcp;
  l_mct_size = (*l_image)
    .numcomps
    .wrapping_mul((*l_image).numcomps)
    .wrapping_mul(core::mem::size_of::<OPJ_FLOAT32>() as OPJ_UINT32);
  /* For each tile */
  i = 0 as OPJ_UINT32;
  while i < l_nb_tiles {
    /* keep the tile-compo coding parameters pointer of the current tile coding parameters*/
    l_current_tccp = (*l_tcp).tccps;
    /*Copy default coding parameters into the current tile coding parameters*/
    memcpy(
      l_tcp as *mut libc::c_void,
      l_default_tcp as *const libc::c_void,
      core::mem::size_of::<opj_tcp_t>() as libc::c_ulong,
    );
    /* Initialize some values of the current tile coding parameters*/
    (*l_tcp).set_cod(0 as OPJ_BITFIELD);
    (*l_tcp).set_ppt(0 as OPJ_BITFIELD);
    (*l_tcp).ppt_data = 0 as *mut OPJ_BYTE;
    (*l_tcp).m_current_tile_part_number = -(1i32);
    /* Remove memory not owned by this tile in case of early error return. */
    (*l_tcp).m_mct_decoding_matrix = 0 as *mut OPJ_FLOAT32;
    (*l_tcp).m_nb_max_mct_records = 0 as OPJ_UINT32;
    (*l_tcp).m_mct_records = 0 as *mut opj_mct_data_t;
    (*l_tcp).m_nb_max_mcc_records = 0 as OPJ_UINT32;
    (*l_tcp).m_mcc_records = 0 as *mut opj_simple_mcc_decorrelation_data_t;
    /* Reconnect the tile-compo coding parameters pointer to the current tile coding parameters*/
    (*l_tcp).tccps = l_current_tccp;
    /* Get the mct_decoding_matrix of the dflt_tile_cp and copy them into the current tile cp*/
    if !(*l_default_tcp).m_mct_decoding_matrix.is_null() {
      (*l_tcp).m_mct_decoding_matrix = opj_malloc(l_mct_size as size_t) as *mut OPJ_FLOAT32;
      if (*l_tcp).m_mct_decoding_matrix.is_null() {
        return 0i32;
      }
      memcpy(
        (*l_tcp).m_mct_decoding_matrix as *mut libc::c_void,
        (*l_default_tcp).m_mct_decoding_matrix as *const libc::c_void,
        l_mct_size as libc::c_ulong,
      );
    }
    /* Get the mct_record of the dflt_tile_cp and copy them into the current tile cp*/
    l_mct_records_size = (*l_default_tcp)
      .m_nb_max_mct_records
      .wrapping_mul(core::mem::size_of::<opj_mct_data_t>() as OPJ_UINT32);
    (*l_tcp).m_mct_records = opj_malloc(l_mct_records_size as size_t) as *mut opj_mct_data_t;
    if (*l_tcp).m_mct_records.is_null() {
      return 0i32;
    }
    memcpy(
      (*l_tcp).m_mct_records as *mut libc::c_void,
      (*l_default_tcp).m_mct_records as *const libc::c_void,
      l_mct_records_size as libc::c_ulong,
    );
    /* Copy the mct record data from dflt_tile_cp to the current tile*/
    l_src_mct_rec = (*l_default_tcp).m_mct_records;
    l_dest_mct_rec = (*l_tcp).m_mct_records;
    j = 0 as OPJ_UINT32;
    while j < (*l_default_tcp).m_nb_mct_records {
      if !(*l_src_mct_rec).m_data.is_null() {
        (*l_dest_mct_rec).m_data =
          opj_malloc((*l_src_mct_rec).m_data_size as size_t) as *mut OPJ_BYTE;
        if (*l_dest_mct_rec).m_data.is_null() {
          return 0i32;
        }
        memcpy(
          (*l_dest_mct_rec).m_data as *mut libc::c_void,
          (*l_src_mct_rec).m_data as *const libc::c_void,
          (*l_src_mct_rec).m_data_size as libc::c_ulong,
        );
      }
      l_src_mct_rec = l_src_mct_rec.offset(1);
      l_dest_mct_rec = l_dest_mct_rec.offset(1);
      /* Update with each pass to free exactly what has been allocated on early return. */
      (*l_tcp).m_nb_max_mct_records = ((*l_tcp).m_nb_max_mct_records as libc::c_uint)
        .wrapping_add(1u32)
        as OPJ_UINT32;
      j = j.wrapping_add(1)
    }
    /* Get the mcc_record of the dflt_tile_cp and copy them into the current tile cp*/
    l_mcc_records_size = (*l_default_tcp)
      .m_nb_max_mcc_records
      .wrapping_mul(
        core::mem::size_of::<opj_simple_mcc_decorrelation_data_t>() as OPJ_UINT32,
      );
    (*l_tcp).m_mcc_records =
      opj_malloc(l_mcc_records_size as size_t) as *mut opj_simple_mcc_decorrelation_data_t;
    if (*l_tcp).m_mcc_records.is_null() {
      return 0i32;
    }
    memcpy(
      (*l_tcp).m_mcc_records as *mut libc::c_void,
      (*l_default_tcp).m_mcc_records as *const libc::c_void,
      l_mcc_records_size as libc::c_ulong,
    );
    (*l_tcp).m_nb_max_mcc_records = (*l_default_tcp).m_nb_max_mcc_records;
    /* Copy the mcc record data from dflt_tile_cp to the current tile*/
    l_src_mcc_rec = (*l_default_tcp).m_mcc_records;
    l_dest_mcc_rec = (*l_tcp).m_mcc_records;
    j = 0 as OPJ_UINT32;
    while j < (*l_default_tcp).m_nb_max_mcc_records {
      if !(*l_src_mcc_rec).m_decorrelation_array.is_null() {
        l_offset = (*l_src_mcc_rec)
          .m_decorrelation_array
          .offset_from((*l_default_tcp).m_mct_records) as libc::c_long
          as OPJ_UINT32;
        (*l_dest_mcc_rec).m_decorrelation_array = (*l_tcp).m_mct_records.offset(l_offset as isize)
      }
      if !(*l_src_mcc_rec).m_offset_array.is_null() {
        l_offset = (*l_src_mcc_rec)
          .m_offset_array
          .offset_from((*l_default_tcp).m_mct_records) as libc::c_long
          as OPJ_UINT32;
        (*l_dest_mcc_rec).m_offset_array = (*l_tcp).m_mct_records.offset(l_offset as isize)
      }
      l_src_mcc_rec = l_src_mcc_rec.offset(1);
      l_dest_mcc_rec = l_dest_mcc_rec.offset(1);
      j = j.wrapping_add(1)
    }
    /* Copy all the dflt_tile_compo_cp to the current tile cp */
    memcpy(
      l_current_tccp as *mut libc::c_void,
      (*l_default_tcp).tccps as *const libc::c_void,
      l_tccp_size as libc::c_ulong,
    );
    /* Move to next tile cp*/
    l_tcp = l_tcp.offset(1);
    i = i.wrapping_add(1)
  }
  /* Create the current tile decoder*/
  (*p_j2k).m_tcd = opj_tcd_create(1i32);
  if (*p_j2k).m_tcd.is_null() {
    return 0i32;
  }
  if opj_tcd_init((*p_j2k).m_tcd, l_image, &mut (*p_j2k).m_cp, (*p_j2k).m_tp) == 0 {
    opj_tcd_destroy((*p_j2k).m_tcd);
    (*p_j2k).m_tcd = 0 as *mut opj_tcd;
    opj_event_msg(
      p_manager,
      1i32,
      b"Cannot decode tile, memory error\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  return 1i32;
}
/* *
 * Reads the lookup table containing all the marker, status and action, and returns the handler associated
 * with the marker value.
 * @param       p_id            Marker value to look up
 *
 * @return      the handler associated with the id.
*/
unsafe fn opj_j2k_get_marker_handler(
  mut p_id: OPJ_UINT32,
) -> *const opj_dec_memory_marker_handler {
  let mut e = 0 as *const opj_dec_memory_marker_handler_t;
  e = j2k_memory_marker_handler_tab.as_ptr();
  while (*e).id != 0u32 {
    if (*e).id == p_id {
      break;
    }
    e = e.offset(1)
  }
  return e;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_destroy(mut p_j2k: *mut opj_j2k_t) {
  if p_j2k.is_null() {
    return;
  }
  if (*p_j2k).m_is_decoder != 0 {
    if !(*p_j2k).m_specific_param.m_decoder.m_default_tcp.is_null() {
      opj_j2k_tcp_destroy((*p_j2k).m_specific_param.m_decoder.m_default_tcp);
      opj_free((*p_j2k).m_specific_param.m_decoder.m_default_tcp as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_decoder.m_default_tcp = 0 as *mut opj_tcp_t
    }
    if !(*p_j2k).m_specific_param.m_decoder.m_header_data.is_null() {
      opj_free((*p_j2k).m_specific_param.m_decoder.m_header_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_decoder.m_header_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_decoder.m_header_data_size = 0 as OPJ_UINT32
    }
    opj_free(
      (*p_j2k)
        .m_specific_param
        .m_decoder
        .m_comps_indices_to_decode as *mut libc::c_void,
    );
    (*p_j2k)
      .m_specific_param
      .m_decoder
      .m_comps_indices_to_decode = 0 as *mut OPJ_UINT32;
    (*p_j2k).m_specific_param.m_decoder.m_numcomps_to_decode = 0 as OPJ_UINT32
  } else {
    if !(*p_j2k)
      .m_specific_param
      .m_encoder
      .m_encoded_tile_data
      .is_null()
    {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_encoded_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_encoded_tile_data = 0 as *mut OPJ_BYTE
    }
    if !(*p_j2k)
      .m_specific_param
      .m_encoder
      .m_tlm_sot_offsets_buffer
      .is_null()
    {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_tlm_sot_offsets_buffer as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_tlm_sot_offsets_buffer = 0 as *mut OPJ_BYTE;
      (*p_j2k)
        .m_specific_param
        .m_encoder
        .m_tlm_sot_offsets_current = 0 as *mut OPJ_BYTE
    }
    if !(*p_j2k)
      .m_specific_param
      .m_encoder
      .m_header_tile_data
      .is_null()
    {
      opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE;
      (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32
    }
  }
  opj_tcd_destroy((*p_j2k).m_tcd);
  opj_j2k_cp_destroy(&mut (*p_j2k).m_cp);
  memset(
    &mut (*p_j2k).m_cp as *mut opj_cp_t as *mut libc::c_void,
    0i32,
    core::mem::size_of::<opj_cp_t>() as libc::c_ulong,
  );
  opj_procedure_list_destroy((*p_j2k).m_procedure_list);
  (*p_j2k).m_procedure_list = 0 as *mut opj_procedure_list_t;
  opj_procedure_list_destroy((*p_j2k).m_validation_list);
  (*p_j2k).m_procedure_list = 0 as *mut opj_procedure_list_t;
  j2k_destroy_cstr_index((*p_j2k).cstr_index);
  (*p_j2k).cstr_index = 0 as *mut opj_codestream_index_t;
  opj_image_destroy((*p_j2k).m_private_image);
  (*p_j2k).m_private_image = 0 as *mut opj_image_t;
  opj_image_destroy((*p_j2k).m_output_image);
  (*p_j2k).m_output_image = 0 as *mut opj_image_t;
  opj_thread_pool_destroy((*p_j2k).m_tp);
  (*p_j2k).m_tp = 0 as *mut opj_thread_pool_t;
  opj_free(p_j2k as *mut libc::c_void);
}
#[no_mangle]
pub(crate) unsafe extern "C" fn j2k_destroy_cstr_index(mut p_cstr_ind: *mut opj_codestream_index_t) {
  if !p_cstr_ind.is_null() {
    if !(*p_cstr_ind).marker.is_null() {
      opj_free((*p_cstr_ind).marker as *mut libc::c_void);
      (*p_cstr_ind).marker = 0 as *mut opj_marker_info_t
    }
    if !(*p_cstr_ind).tile_index.is_null() {
      let mut it_tile = 0 as OPJ_UINT32;
      it_tile = 0 as OPJ_UINT32;
      while it_tile < (*p_cstr_ind).nb_of_tiles {
        if !(*(*p_cstr_ind).tile_index.offset(it_tile as isize))
          .packet_index
          .is_null()
        {
          opj_free(
            (*(*p_cstr_ind).tile_index.offset(it_tile as isize)).packet_index as *mut libc::c_void,
          );
          let ref mut fresh30 = (*(*p_cstr_ind).tile_index.offset(it_tile as isize)).packet_index;
          *fresh30 = 0 as *mut opj_packet_info_t
        }
        if !(*(*p_cstr_ind).tile_index.offset(it_tile as isize))
          .tp_index
          .is_null()
        {
          opj_free(
            (*(*p_cstr_ind).tile_index.offset(it_tile as isize)).tp_index as *mut libc::c_void,
          );
          let ref mut fresh31 = (*(*p_cstr_ind).tile_index.offset(it_tile as isize)).tp_index;
          *fresh31 = 0 as *mut opj_tp_index_t
        }
        if !(*(*p_cstr_ind).tile_index.offset(it_tile as isize))
          .marker
          .is_null()
        {
          opj_free(
            (*(*p_cstr_ind).tile_index.offset(it_tile as isize)).marker as *mut libc::c_void,
          );
          let ref mut fresh32 = (*(*p_cstr_ind).tile_index.offset(it_tile as isize)).marker;
          *fresh32 = 0 as *mut opj_marker_info_t
        }
        it_tile = it_tile.wrapping_add(1)
      }
      opj_free((*p_cstr_ind).tile_index as *mut libc::c_void);
      (*p_cstr_ind).tile_index = 0 as *mut opj_tile_index_t
    }
    opj_free(p_cstr_ind as *mut libc::c_void);
  };
}
/* *
 * Destroys a tile coding parameter structure.
 *
 * @param       p_tcp           the tile coding parameter to destroy.
 */
unsafe fn opj_j2k_tcp_destroy(mut p_tcp: *mut opj_tcp_t) {
  if p_tcp.is_null() {
    return;
  }
  if !(*p_tcp).ppt_markers.is_null() {
    let mut i: OPJ_UINT32 = 0;
    i = 0u32;
    while i < (*p_tcp).ppt_markers_count {
      if !(*(*p_tcp).ppt_markers.offset(i as isize)).m_data.is_null() {
        opj_free((*(*p_tcp).ppt_markers.offset(i as isize)).m_data as *mut libc::c_void);
      }
      i = i.wrapping_add(1)
    }
    (*p_tcp).ppt_markers_count = 0u32;
    opj_free((*p_tcp).ppt_markers as *mut libc::c_void);
    (*p_tcp).ppt_markers = 0 as *mut opj_ppx
  }
  if !(*p_tcp).ppt_buffer.is_null() {
    opj_free((*p_tcp).ppt_buffer as *mut libc::c_void);
    (*p_tcp).ppt_buffer = 0 as *mut OPJ_BYTE
  }
  if !(*p_tcp).tccps.is_null() {
    opj_free((*p_tcp).tccps as *mut libc::c_void);
    (*p_tcp).tccps = 0 as *mut opj_tccp_t
  }
  if !(*p_tcp).m_mct_coding_matrix.is_null() {
    opj_free((*p_tcp).m_mct_coding_matrix as *mut libc::c_void);
    (*p_tcp).m_mct_coding_matrix = 0 as *mut OPJ_FLOAT32
  }
  if !(*p_tcp).m_mct_decoding_matrix.is_null() {
    opj_free((*p_tcp).m_mct_decoding_matrix as *mut libc::c_void);
    (*p_tcp).m_mct_decoding_matrix = 0 as *mut OPJ_FLOAT32
  }
  if !(*p_tcp).m_mcc_records.is_null() {
    opj_free((*p_tcp).m_mcc_records as *mut libc::c_void);
    (*p_tcp).m_mcc_records = 0 as *mut opj_simple_mcc_decorrelation_data_t;
    (*p_tcp).m_nb_max_mcc_records = 0 as OPJ_UINT32;
    (*p_tcp).m_nb_mcc_records = 0 as OPJ_UINT32
  }
  if !(*p_tcp).m_mct_records.is_null() {
    let mut l_mct_data = (*p_tcp).m_mct_records;
    let mut i_0: OPJ_UINT32 = 0;
    i_0 = 0 as OPJ_UINT32;
    while i_0 < (*p_tcp).m_nb_mct_records {
      if !(*l_mct_data).m_data.is_null() {
        opj_free((*l_mct_data).m_data as *mut libc::c_void);
        (*l_mct_data).m_data = 0 as *mut OPJ_BYTE
      }
      l_mct_data = l_mct_data.offset(1);
      i_0 = i_0.wrapping_add(1)
    }
    opj_free((*p_tcp).m_mct_records as *mut libc::c_void);
    (*p_tcp).m_mct_records = 0 as *mut opj_mct_data_t
  }
  if !(*p_tcp).mct_norms.is_null() {
    opj_free((*p_tcp).mct_norms as *mut libc::c_void);
    (*p_tcp).mct_norms = 0 as *mut OPJ_FLOAT64
  }
  opj_j2k_tcp_data_destroy(p_tcp);
}
/* *
 * Destroys the data inside a tile coding parameter structure.
 *
 * @param       p_tcp           the tile coding parameter which contain data to destroy.
 */
unsafe fn opj_j2k_tcp_data_destroy(mut p_tcp: *mut opj_tcp_t) {
  if !(*p_tcp).m_data.is_null() {
    opj_free((*p_tcp).m_data as *mut libc::c_void);
    (*p_tcp).m_data = 0 as *mut OPJ_BYTE;
    (*p_tcp).m_data_size = 0 as OPJ_UINT32
  };
}
/* *
 * Destroys a coding parameter structure.
 *
 * @param       p_cp            the coding parameter to destroy.
 */
unsafe fn opj_j2k_cp_destroy(mut p_cp: *mut opj_cp_t) {
  let mut l_nb_tiles: OPJ_UINT32 = 0; /* ppm_data belongs to the allocated buffer pointed by ppm_buffer */
  let mut l_current_tile = 0 as *mut opj_tcp_t;
  if p_cp.is_null() {
    return;
  }
  if !(*p_cp).tcps.is_null() {
    let mut i: OPJ_UINT32 = 0;
    l_current_tile = (*p_cp).tcps;
    l_nb_tiles = (*p_cp).th.wrapping_mul((*p_cp).tw);
    i = 0u32;
    while i < l_nb_tiles {
      opj_j2k_tcp_destroy(l_current_tile);
      l_current_tile = l_current_tile.offset(1);
      i = i.wrapping_add(1)
    }
    opj_free((*p_cp).tcps as *mut libc::c_void);
    (*p_cp).tcps = 0 as *mut opj_tcp_t
  }
  if !(*p_cp).ppm_markers.is_null() {
    let mut i_0: OPJ_UINT32 = 0;
    i_0 = 0u32;
    while i_0 < (*p_cp).ppm_markers_count {
      if !(*(*p_cp).ppm_markers.offset(i_0 as isize)).m_data.is_null() {
        opj_free((*(*p_cp).ppm_markers.offset(i_0 as isize)).m_data as *mut libc::c_void);
      }
      i_0 = i_0.wrapping_add(1)
    }
    (*p_cp).ppm_markers_count = 0u32;
    opj_free((*p_cp).ppm_markers as *mut libc::c_void);
    (*p_cp).ppm_markers = 0 as *mut opj_ppx
  }
  opj_free((*p_cp).ppm_buffer as *mut libc::c_void);
  (*p_cp).ppm_buffer = 0 as *mut OPJ_BYTE;
  (*p_cp).ppm_data = 0 as *mut OPJ_BYTE;
  opj_free((*p_cp).comment as *mut libc::c_void);
  (*p_cp).comment = 0 as *mut OPJ_CHAR;
  if (*p_cp).m_is_decoder() == 0 {
    opj_free((*p_cp).m_specific_param.m_enc.m_matrice as *mut libc::c_void);
    (*p_cp).m_specific_param.m_enc.m_matrice = 0 as *mut OPJ_INT32
  };
}
/* *
 * Checks for invalid number of tile-parts in SOT marker (TPsot==TNsot). See issue 254.
 *
 * @param       p_stream            the stream to read data from.
 * @param       tile_no             tile number we're looking for.
 * @param       p_correction_needed output value. if true, non conformant codestream needs TNsot correction.
 * @param       p_manager       the user event manager.
 *
 * @return true if the function was successful, false else.
 */
unsafe fn opj_j2k_need_nb_tile_parts_correction(
  mut p_stream: *mut opj_stream_private_t,
  mut tile_no: OPJ_UINT32,
  mut p_correction_needed: *mut OPJ_BOOL,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_header_data: [OPJ_BYTE; 10] = [0; 10];
  let mut l_stream_pos_backup: OPJ_OFF_T = 0;
  let mut l_current_marker: OPJ_UINT32 = 0;
  let mut l_marker_size: OPJ_UINT32 = 0;
  let mut l_tile_no: OPJ_UINT32 = 0;
  let mut l_tot_len: OPJ_UINT32 = 0;
  let mut l_current_part: OPJ_UINT32 = 0;
  let mut l_num_parts: OPJ_UINT32 = 0;
  /* initialize to no correction needed */
  *p_correction_needed = 0i32;
  if opj_stream_has_seek(p_stream) == 0 {
    /* We can't do much in this case, seek is needed */
    return 1i32;
  }
  l_stream_pos_backup = opj_stream_tell(p_stream);
  if l_stream_pos_backup == -(1i32) as libc::c_long {
    /* let's do nothing */
    return 1i32;
  }
  loop {
    /* Try to read 2 bytes (the next marker ID) from stream and copy them into the buffer */
    if opj_stream_read_data(
      p_stream,
      l_header_data.as_mut_ptr(),
      2 as OPJ_SIZE_T,
      p_manager,
    ) != 2u64
    {
      /* assume all is OK */
      if opj_stream_seek(p_stream, l_stream_pos_backup, p_manager) == 0 {
        return 0i32;
      }
      return 1i32;
    }
    /* Read 2 bytes from buffer as the new marker ID */
    opj_read_bytes_LE(
      l_header_data.as_mut_ptr(),
      &mut l_current_marker,
      2 as OPJ_UINT32,
    );
    if l_current_marker != 0xff90u32 {
      /* assume all is OK */
      if opj_stream_seek(p_stream, l_stream_pos_backup, p_manager) == 0 {
        return 0i32;
      }
      return 1i32;
    }
    /* Try to read 2 bytes (the marker size) from stream and copy them into the buffer */
    if opj_stream_read_data(
      p_stream,
      l_header_data.as_mut_ptr(),
      2 as OPJ_SIZE_T,
      p_manager,
    ) != 2u64
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    /* Read 2 bytes from the buffer as the marker size */
    opj_read_bytes_LE(
      l_header_data.as_mut_ptr(),
      &mut l_marker_size,
      2 as OPJ_UINT32,
    );
    /* Check marker size for SOT Marker */
    if l_marker_size != 10u32 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Inconsistent marker size\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    l_marker_size = (l_marker_size as libc::c_uint).wrapping_sub(2u32)
      as OPJ_UINT32;
    if opj_stream_read_data(
      p_stream,
      l_header_data.as_mut_ptr(),
      l_marker_size as OPJ_SIZE_T,
      p_manager,
    ) != l_marker_size as libc::c_ulong
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    if opj_j2k_get_sot_values(
      l_header_data.as_mut_ptr(),
      l_marker_size,
      &mut l_tile_no,
      &mut l_tot_len,
      &mut l_current_part,
      &mut l_num_parts,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    if l_tile_no == tile_no {
      break;
    }
    if l_tot_len < 14u32 {
      /* last SOT until EOC or invalid Psot value */
      /* assume all is OK */
      if opj_stream_seek(p_stream, l_stream_pos_backup, p_manager) == 0 {
        return 0i32;
      }
      return 1i32;
    }
    l_tot_len =
      (l_tot_len as libc::c_uint).wrapping_sub(12u32) as OPJ_UINT32;
    /* look for next SOT marker */
    if opj_stream_skip(p_stream, l_tot_len as OPJ_OFF_T, p_manager) != l_tot_len as OPJ_OFF_T {
      /* assume all is OK */
      if opj_stream_seek(p_stream, l_stream_pos_backup, p_manager) == 0 {
        return 0i32;
      }
      return 1i32;
    }
  }
  /* check for correction */
  if l_current_part == l_num_parts {
    *p_correction_needed = 1i32
  }
  if opj_stream_seek(p_stream, l_stream_pos_backup, p_manager) == 0 {
    return 0i32;
  }
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_read_tile_header(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_index: *mut OPJ_UINT32,
  mut p_data_size: *mut OPJ_UINT32,
  mut p_tile_x0: *mut OPJ_INT32,
  mut p_tile_y0: *mut OPJ_INT32,
  mut p_tile_x1: *mut OPJ_INT32,
  mut p_tile_y1: *mut OPJ_INT32,
  mut p_nb_comps: *mut OPJ_UINT32,
  mut p_go_on: *mut OPJ_BOOL,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_current_marker = 0xff90 as OPJ_UINT32;
  let mut l_marker_size: OPJ_UINT32 = 0;
  let mut l_marker_handler = 0 as *const opj_dec_memory_marker_handler_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let l_nb_tiles = (*p_j2k).m_cp.tw.wrapping_mul((*p_j2k).m_cp.th);
  /* preconditions */

  assert!(!p_stream.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  /* Reach the End Of Codestream ?*/
  if (*p_j2k).m_specific_param.m_decoder.m_state == J2KState::EOC {
    l_current_marker = 0xffd9 as OPJ_UINT32
  } else if (*p_j2k).m_specific_param.m_decoder.m_state
    != J2KState::TPHSOT
  {
    return 0i32;
  }
  /* We need to encounter a SOT marker (a new tile-part header) */
  /* Read into the codestream until reach the EOC or ! can_decode ??? FIXME */
  while (*p_j2k).m_specific_param.m_decoder.m_can_decode() == 0
    && l_current_marker != 0xffd9u32
  {
    /* Try to read until the Start Of Data is detected */
    while l_current_marker != 0xff93u32 {
      if opj_stream_get_number_byte_left(p_stream) == 0i64 {
        (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::NEOC;
        break;
      } else {
        /* Try to read 2 bytes (the marker size) from stream and copy them into the buffer */
        if opj_stream_read_data(
          p_stream,
          (*p_j2k).m_specific_param.m_decoder.m_header_data,
          2 as OPJ_SIZE_T,
          p_manager,
        ) != 2u64
        {
          opj_event_msg(
            p_manager,
            1i32,
            b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
          );
          return 0i32;
        }
        /* Read 2 bytes from the buffer as the marker size */
        opj_read_bytes_LE(
          (*p_j2k).m_specific_param.m_decoder.m_header_data,
          &mut l_marker_size,
          2 as OPJ_UINT32,
        );
        /* Check marker size (does not include marker ID but includes marker size) */
        if l_marker_size < 2u32 {
          opj_event_msg(
            p_manager,
            1i32,
            b"Inconsistent marker size\n\x00" as *const u8 as *const libc::c_char,
          );
          return 0i32;
        }
        /* cf. https://code.google.com/p/openjpeg/issues/detail?id=226 */
        if l_current_marker == 0x8080u32
          && opj_stream_get_number_byte_left(p_stream) == 0i64
        {
          (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::NEOC;
          break;
        } else {
          /* Why this condition? FIXME */
          if (*p_j2k).m_specific_param.m_decoder.m_state
            & J2KState::TPH
            != J2KState::NONE
          {
            (*p_j2k).m_specific_param.m_decoder.m_sot_length =
              ((*p_j2k).m_specific_param.m_decoder.m_sot_length as libc::c_uint)
                .wrapping_sub(l_marker_size.wrapping_add(2u32))
                as OPJ_UINT32
          } /* Subtract the size of the marker ID already read */
          l_marker_size = (l_marker_size as libc::c_uint)
            .wrapping_sub(2u32) as OPJ_UINT32
            as OPJ_UINT32;
          /* Get the marker handler from the marker ID */
          l_marker_handler = opj_j2k_get_marker_handler(l_current_marker);
          /* Check if the marker is known and if it is the right place to find it */
          if (*p_j2k).m_specific_param.m_decoder.m_state & (*l_marker_handler).states == J2KState::NONE {
            opj_event_msg(
              p_manager,
              1i32,
              b"Marker is not compliant with its position\n\x00" as *const u8
                as *const libc::c_char,
            );
            return 0i32;
          }
          /* FIXME manage case of unknown marker as in the main header ? */
          /* Check if the marker size is compatible with the header data size */
          if l_marker_size > (*p_j2k).m_specific_param.m_decoder.m_header_data_size {
            let mut new_header_data = 0 as *mut OPJ_BYTE;
            /* If we are here, this means we consider this marker as known & we will read it */
            /* Check enough bytes left in stream before allocation */
            if l_marker_size as OPJ_OFF_T > opj_stream_get_number_byte_left(p_stream) {
              opj_event_msg(
                p_manager,
                1i32,
                b"Marker size inconsistent with stream length\n\x00" as *const u8
                  as *const libc::c_char,
              );
              return 0i32;
            }
            new_header_data = opj_realloc(
              (*p_j2k).m_specific_param.m_decoder.m_header_data as *mut libc::c_void,
              l_marker_size as size_t,
            ) as *mut OPJ_BYTE;
            if new_header_data.is_null() {
              opj_free((*p_j2k).m_specific_param.m_decoder.m_header_data as *mut libc::c_void);
              (*p_j2k).m_specific_param.m_decoder.m_header_data = 0 as *mut OPJ_BYTE;
              (*p_j2k).m_specific_param.m_decoder.m_header_data_size =
                0 as OPJ_UINT32;
              opj_event_msg(
                p_manager,
                1i32,
                b"Not enough memory to read header\n\x00" as *const u8 as *const libc::c_char,
              );
              return 0i32;
            }
            (*p_j2k).m_specific_param.m_decoder.m_header_data = new_header_data;
            (*p_j2k).m_specific_param.m_decoder.m_header_data_size = l_marker_size
          }
          /* Try to read the rest of the marker segment from stream and copy them into the buffer */
          if opj_stream_read_data(
            p_stream,
            (*p_j2k).m_specific_param.m_decoder.m_header_data,
            l_marker_size as OPJ_SIZE_T,
            p_manager,
          ) != l_marker_size as libc::c_ulong
          {
            opj_event_msg(
              p_manager,
              1i32,
              b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
            );
            return 0i32;
          }
          if (*l_marker_handler).handler.is_none() {
            /* See issue #175 */
            opj_event_msg(
              p_manager,
              1i32,
              b"Not sure how that happened.\n\x00" as *const u8 as *const libc::c_char,
            );
            return 0i32;
          }
          /* Read the marker segment with the correct marker handler */
          if Some(
            (*l_marker_handler)
              .handler
              .expect("non-null function pointer"),
          )
          .expect("non-null function pointer")(
            p_j2k,
            (*p_j2k).m_specific_param.m_decoder.m_header_data,
            l_marker_size,
            p_manager,
          ) == 0
          {
            opj_event_msg(
              p_manager,
              1i32,
              b"Fail to read the current marker segment (%#x)\n\x00" as *const u8
                as *const libc::c_char,
              l_current_marker,
            );
            return 0i32;
          }
          /* Add the marker to the codestream index*/
          if 0i32
            == opj_j2k_add_tlmarker(
              (*p_j2k).m_current_tile_number,
              (*p_j2k).cstr_index,
              (*l_marker_handler).id,
              (opj_stream_tell(p_stream) as OPJ_UINT32)
                .wrapping_sub(l_marker_size)
                .wrapping_sub(4u32) as OPJ_OFF_T,
              l_marker_size.wrapping_add(4u32),
            )
          {
            opj_event_msg(
              p_manager,
              1i32,
              b"Not enough memory to add tl marker\n\x00" as *const u8 as *const libc::c_char,
            );
            return 0i32;
          }
          /* Keep the position of the last SOT marker read */
          if (*l_marker_handler).id == 0xff90u32 {
            let mut sot_pos = (opj_stream_tell(p_stream) as OPJ_UINT32)
              .wrapping_sub(l_marker_size)
              .wrapping_sub(4u32);
            if sot_pos as libc::c_long > (*p_j2k).m_specific_param.m_decoder.m_last_sot_read_pos {
              (*p_j2k).m_specific_param.m_decoder.m_last_sot_read_pos = sot_pos as OPJ_OFF_T
            }
          }
          if (*p_j2k).m_specific_param.m_decoder.m_skip_data() != 0 {
            /* Skip the rest of the tile part header*/
            if opj_stream_skip(
              p_stream,
              (*p_j2k).m_specific_param.m_decoder.m_sot_length as OPJ_OFF_T,
              p_manager,
            ) != (*p_j2k).m_specific_param.m_decoder.m_sot_length as libc::c_long
            {
              opj_event_msg(
                p_manager,
                1i32,
                b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
              );
              return 0i32;
            }
            l_current_marker = 0xff93 as OPJ_UINT32
          /* Normally we reached a SOD */
          } else {
            /* Try to read 2 bytes (the next marker ID) from stream and copy them into the buffer*/
            if opj_stream_read_data(
              p_stream,
              (*p_j2k).m_specific_param.m_decoder.m_header_data,
              2 as OPJ_SIZE_T,
              p_manager,
            ) != 2u64
            {
              opj_event_msg(
                p_manager,
                1i32,
                b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
              );
              return 0i32;
            }
            /* Read 2 bytes from the buffer as the new marker ID */
            opj_read_bytes_LE(
              (*p_j2k).m_specific_param.m_decoder.m_header_data,
              &mut l_current_marker,
              2 as OPJ_UINT32,
            );
          }
        }
      }
    }
    if opj_stream_get_number_byte_left(p_stream) == 0i64
      && (*p_j2k).m_specific_param.m_decoder.m_state
        == J2KState::NEOC
    {
      break;
    }
    /* If we didn't skip data before, we need to read the SOD marker*/
    if (*p_j2k).m_specific_param.m_decoder.m_skip_data() == 0 {
      /* Try to read the SOD marker and skip data ? FIXME */
      if opj_j2k_read_sod(p_j2k, p_stream, p_manager) == 0 {
        return 0i32;
      }
      if (*p_j2k).m_specific_param.m_decoder.m_can_decode() as libc::c_int != 0
        && (*p_j2k)
          .m_specific_param
          .m_decoder
          .m_nb_tile_parts_correction_checked()
          == 0
      {
        /* Issue 254 */
        let mut l_correction_needed: OPJ_BOOL = 0;
        (*p_j2k)
          .m_specific_param
          .m_decoder
          .set_m_nb_tile_parts_correction_checked(1 as OPJ_BITFIELD);
        if opj_j2k_need_nb_tile_parts_correction(
          p_stream,
          (*p_j2k).m_current_tile_number,
          &mut l_correction_needed,
          p_manager,
        ) == 0
        {
          opj_event_msg(
            p_manager,
            1i32,
            b"opj_j2k_apply_nb_tile_parts_correction error\n\x00" as *const u8
              as *const libc::c_char,
          );
          return 0i32;
        }
        if l_correction_needed != 0 {
          let mut l_tile_no: OPJ_UINT32 = 0;
          (*p_j2k)
            .m_specific_param
            .m_decoder
            .set_m_can_decode(0 as OPJ_BITFIELD);
          (*p_j2k)
            .m_specific_param
            .m_decoder
            .set_m_nb_tile_parts_correction(1 as OPJ_BITFIELD);
          /* correct tiles */
          l_tile_no = 0u32;
          while l_tile_no < l_nb_tiles {
            if (*(*p_j2k).m_cp.tcps.offset(l_tile_no as isize)).m_nb_tile_parts != 0u32
            {
              let ref mut fresh33 =
                (*(*p_j2k).m_cp.tcps.offset(l_tile_no as isize)).m_nb_tile_parts;
              *fresh33 = (*fresh33 as libc::c_uint).wrapping_add(1u32)
                as OPJ_UINT32
            }
            l_tile_no = l_tile_no.wrapping_add(1)
          }
          opj_event_msg(
            p_manager,
            2i32,
            b"Non conformant codestream TPsot==TNsot.\n\x00" as *const u8 as *const libc::c_char,
          );
        }
      }
    } else {
      /* Indicate we will try to read a new tile-part header*/
      (*p_j2k)
        .m_specific_param
        .m_decoder
        .set_m_skip_data(0 as OPJ_BITFIELD);
      (*p_j2k)
        .m_specific_param
        .m_decoder
        .set_m_can_decode(0 as OPJ_BITFIELD);
      (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::TPHSOT
    }
    if !((*p_j2k).m_specific_param.m_decoder.m_can_decode() == 0) {
      continue;
    }
    /* Try to read 2 bytes (the next marker ID) from stream and copy them into the buffer */
    if opj_stream_read_data(
      p_stream,
      (*p_j2k).m_specific_param.m_decoder.m_header_data,
      2 as OPJ_SIZE_T,
      p_manager,
    ) != 2u64
    {
      /* Deal with likely non conformant SPOT6 files, where the last */
      /* row of tiles have TPsot == 0 and TNsot == 0, and missing EOC, */
      /* but no other tile-parts were found. */
      if (*p_j2k)
        .m_current_tile_number
        .wrapping_add(1u32)
        == l_nb_tiles
      {
        let mut l_tile_no_0: OPJ_UINT32 = 0;
        l_tile_no_0 = 0u32;
        while l_tile_no_0 < l_nb_tiles {
          if (*(*p_j2k).m_cp.tcps.offset(l_tile_no_0 as isize)).m_current_tile_part_number
            == 0i32
            && (*(*p_j2k).m_cp.tcps.offset(l_tile_no_0 as isize)).m_nb_tile_parts
              == 0u32
          {
            break;
          }
          l_tile_no_0 = l_tile_no_0.wrapping_add(1)
        }
        if l_tile_no_0 < l_nb_tiles {
          opj_event_msg(p_manager, 4i32,
                                  b"Tile %u has TPsot == 0 and TNsot == 0, but no other tile-parts were found. EOC is also missing.\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  l_tile_no_0);
          (*p_j2k).m_current_tile_number = l_tile_no_0;
          l_current_marker = 0xffd9 as OPJ_UINT32;
          (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::EOC;
          break;
        }
      }
      opj_event_msg(
        p_manager,
        1i32,
        b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    } else {
      /* Read 2 bytes from buffer as the new marker ID */
      opj_read_bytes_LE(
        (*p_j2k).m_specific_param.m_decoder.m_header_data,
        &mut l_current_marker,
        2 as OPJ_UINT32,
      );
    }
  }
  /* Current marker is the EOC marker ?*/
  if l_current_marker == 0xffd9u32 {
    if (*p_j2k).m_specific_param.m_decoder.m_state != J2KState::EOC {
      (*p_j2k).m_current_tile_number = 0 as OPJ_UINT32;
      (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::EOC
    }
  }
  /* Deal with tiles that have a single tile-part with TPsot == 0 and TNsot == 0 */
  if (*p_j2k).m_specific_param.m_decoder.m_can_decode() == 0 {
    l_tcp = (*p_j2k)
      .m_cp
      .tcps
      .offset((*p_j2k).m_current_tile_number as isize);
    while (*p_j2k).m_current_tile_number < l_nb_tiles && (*l_tcp).m_data.is_null() {
      (*p_j2k).m_current_tile_number = (*p_j2k).m_current_tile_number.wrapping_add(1);
      l_tcp = l_tcp.offset(1)
    }
    if (*p_j2k).m_current_tile_number == l_nb_tiles {
      *p_go_on = 0i32;
      return 1i32;
    }
  }
  if opj_j2k_merge_ppt(
    (*p_j2k)
      .m_cp
      .tcps
      .offset((*p_j2k).m_current_tile_number as isize),
    p_manager,
  ) == 0
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Failed to merge PPT data\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /*FIXME ???*/
  if opj_tcd_init_decode_tile((*p_j2k).m_tcd, (*p_j2k).m_current_tile_number, p_manager) == 0 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Cannot decode tile, memory error\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  opj_event_msg(
    p_manager,
    4i32,
    b"Header of tile %d / %d has been read.\n\x00" as *const u8 as *const libc::c_char,
    (*p_j2k)
      .m_current_tile_number
      .wrapping_add(1u32),
    (*p_j2k).m_cp.th.wrapping_mul((*p_j2k).m_cp.tw),
  );
  *p_tile_index = (*p_j2k).m_current_tile_number;
  *p_go_on = 1i32;
  if !p_data_size.is_null() {
    /* For internal use in j2k.c, we don't need this */
    /* This is just needed for folks using the opj_read_tile_header() / opj_decode_tile_data() combo */
    *p_data_size = opj_tcd_get_decoded_tile_size((*p_j2k).m_tcd, 0i32);
    if *p_data_size
      == (2147483647u32)
        .wrapping_mul(2u32)
        .wrapping_add(1u32)
    {
      return 0i32;
    }
  }
  *p_tile_x0 = (*(*(*(*p_j2k).m_tcd).tcd_image).tiles).x0;
  *p_tile_y0 = (*(*(*(*p_j2k).m_tcd).tcd_image).tiles).y0;
  *p_tile_x1 = (*(*(*(*p_j2k).m_tcd).tcd_image).tiles).x1;
  *p_tile_y1 = (*(*(*(*p_j2k).m_tcd).tcd_image).tiles).y1;
  *p_nb_comps = (*(*(*(*p_j2k).m_tcd).tcd_image).tiles).numcomps;
  (*p_j2k).m_specific_param.m_decoder.m_state |= J2KState::DATA;
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_decode_tile(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_index: OPJ_UINT32,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_size: OPJ_UINT32,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_current_marker: OPJ_UINT32 = 0;
  let mut l_data: [OPJ_BYTE; 2] = [0; 2];
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_image_for_bounds = 0 as *mut opj_image_t;
  /* preconditions */

  assert!(!p_stream.is_null());
  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  if (*p_j2k).m_specific_param.m_decoder.m_state & J2KState::DATA
    == J2KState::NONE
    || p_tile_index != (*p_j2k).m_current_tile_number
  {
    return 0i32;
  }
  l_tcp = &mut *(*p_j2k).m_cp.tcps.offset(p_tile_index as isize) as *mut opj_tcp_t;
  if (*l_tcp).m_data.is_null() {
    opj_j2k_tcp_destroy(l_tcp);
    return 0i32;
  }
  /* When using the opj_read_tile_header / opj_decode_tile_data API */
  /* such as in test_tile_decoder, m_output_image is NULL, so fall back */
  /* to the full image dimension. This is a bit surprising that */
  /* opj_set_decode_area() is only used to determine intersecting tiles, */
  /* but full tile decoding is done */
  l_image_for_bounds = if !(*p_j2k).m_output_image.is_null() {
    (*p_j2k).m_output_image
  } else {
    (*p_j2k).m_private_image
  };
  if opj_tcd_decode_tile(
    (*p_j2k).m_tcd,
    (*l_image_for_bounds).x0,
    (*l_image_for_bounds).y0,
    (*l_image_for_bounds).x1,
    (*l_image_for_bounds).y1,
    (*p_j2k).m_specific_param.m_decoder.m_numcomps_to_decode,
    (*p_j2k)
      .m_specific_param
      .m_decoder
      .m_comps_indices_to_decode,
    (*l_tcp).m_data,
    (*l_tcp).m_data_size,
    p_tile_index,
    (*p_j2k).cstr_index,
    p_manager,
  ) == 0
  {
    opj_j2k_tcp_destroy(l_tcp);
    (*p_j2k).m_specific_param.m_decoder.m_state |= J2KState::ERR;
    opj_event_msg(
      p_manager,
      1i32,
      b"Failed to decode.\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* p_data can be set to NULL when the call will take care of using */
  /* itself the TCD data. This is typically the case for whole single */
  /* tile decoding optimization. */
  if !p_data.is_null() {
    if opj_tcd_update_tile_data((*p_j2k).m_tcd, p_data, p_data_size) == 0 {
      return 0i32;
    }
    /* To avoid to destroy the tcp which can be useful when we try to decode a tile decoded before (cf j2k_random_tile_access)
     * we destroy just the data which will be re-read in read_tile_header*/
    /*opj_j2k_tcp_destroy(l_tcp);
    p_j2k->m_tcd->tcp = 0;*/
    opj_j2k_tcp_data_destroy(l_tcp);
  }
  (*p_j2k)
    .m_specific_param
    .m_decoder
    .set_m_can_decode(0 as OPJ_BITFIELD);
  (*p_j2k).m_specific_param.m_decoder.m_state &= !(J2KState::DATA);
  if opj_stream_get_number_byte_left(p_stream) == 0i64
    && (*p_j2k).m_specific_param.m_decoder.m_state == J2KState::NEOC
  {
    return 1i32;
  }
  if (*p_j2k).m_specific_param.m_decoder.m_state != J2KState::EOC {
    if opj_stream_read_data(
      p_stream,
      l_data.as_mut_ptr(),
      2 as OPJ_SIZE_T,
      p_manager,
    ) != 2u64
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Stream too short\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    opj_read_bytes_LE(
      l_data.as_mut_ptr(),
      &mut l_current_marker,
      2 as OPJ_UINT32,
    );
    if l_current_marker == 0xffd9u32 {
      (*p_j2k).m_current_tile_number = 0 as OPJ_UINT32;
      (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::EOC
    } else if l_current_marker != 0xff90u32 {
      if opj_stream_get_number_byte_left(p_stream) == 0i64 {
        (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::NEOC;
        opj_event_msg(
          p_manager,
          2i32,
          b"Stream does not end with EOC\n\x00" as *const u8 as *const libc::c_char,
        );
        return 1i32;
      }
      opj_event_msg(
        p_manager,
        1i32,
        b"Stream too short, expected SOT\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
  }
  return 1i32;
}
unsafe fn opj_j2k_update_image_data(
  mut p_tcd: *mut opj_tcd_t,
  mut p_output_image: *mut opj_image_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut l_width_src: OPJ_UINT32 = 0;
  let mut l_height_src: OPJ_UINT32 = 0;
  let mut l_width_dest: OPJ_UINT32 = 0;
  let mut l_height_dest: OPJ_UINT32 = 0;
  let mut l_offset_x0_src: OPJ_INT32 = 0;
  let mut l_offset_y0_src: OPJ_INT32 = 0;
  let mut l_offset_x1_src: OPJ_INT32 = 0;
  let mut l_offset_y1_src: OPJ_INT32 = 0;
  let mut l_start_offset_src: OPJ_SIZE_T = 0;
  let mut l_start_x_dest: OPJ_UINT32 = 0;
  let mut l_start_y_dest: OPJ_UINT32 = 0;
  let mut l_x0_dest: OPJ_UINT32 = 0;
  let mut l_y0_dest: OPJ_UINT32 = 0;
  let mut l_x1_dest: OPJ_UINT32 = 0;
  let mut l_y1_dest: OPJ_UINT32 = 0;
  let mut l_start_offset_dest: OPJ_SIZE_T = 0;
  let mut l_img_comp_src = 0 as *mut opj_image_comp_t;
  let mut l_img_comp_dest = 0 as *mut opj_image_comp_t;
  let mut l_tilec = 0 as *mut opj_tcd_tilecomp_t;
  let mut l_image_src = 0 as *mut opj_image_t;
  let mut l_dest_ptr = 0 as *mut OPJ_INT32;
  l_tilec = (*(*(*p_tcd).tcd_image).tiles).comps;
  l_image_src = (*p_tcd).image;
  l_img_comp_src = (*l_image_src).comps;
  l_img_comp_dest = (*p_output_image).comps;
  i = 0 as OPJ_UINT32;
  while i < (*l_image_src).numcomps {
    let mut res_x0: OPJ_INT32 = 0;
    let mut res_x1: OPJ_INT32 = 0;
    let mut res_y0: OPJ_INT32 = 0;
    let mut res_y1: OPJ_INT32 = 0;
    let mut src_data_stride: OPJ_UINT32 = 0;
    let mut p_src_data = 0 as *const OPJ_INT32;
    /* Copy info from decoded comp image to output image */
    (*l_img_comp_dest).resno_decoded = (*l_img_comp_src).resno_decoded;
    if (*p_tcd).whole_tile_decoding != 0 {
      let mut l_res = (*l_tilec)
        .resolutions
        .offset((*l_img_comp_src).resno_decoded as isize);
      res_x0 = (*l_res).x0;
      res_y0 = (*l_res).y0;
      res_x1 = (*l_res).x1;
      res_y1 = (*l_res).y1;
      src_data_stride = ((*(*l_tilec).resolutions.offset(
        (*l_tilec)
          .minimum_num_resolutions
          .wrapping_sub(1u32) as isize,
      ))
      .x1
        - (*(*l_tilec).resolutions.offset(
          (*l_tilec)
            .minimum_num_resolutions
            .wrapping_sub(1u32) as isize,
        ))
        .x0) as OPJ_UINT32;
      p_src_data = (*l_tilec).data
    } else {
      let mut l_res_0 = (*l_tilec)
        .resolutions
        .offset((*l_img_comp_src).resno_decoded as isize);
      res_x0 = (*l_res_0).win_x0 as OPJ_INT32;
      res_y0 = (*l_res_0).win_y0 as OPJ_INT32;
      res_x1 = (*l_res_0).win_x1 as OPJ_INT32;
      res_y1 = (*l_res_0).win_y1 as OPJ_INT32;
      src_data_stride = (*l_res_0).win_x1.wrapping_sub((*l_res_0).win_x0);
      p_src_data = (*l_tilec).data_win
    }
    if !p_src_data.is_null() {
      l_width_src = (res_x1 - res_x0) as OPJ_UINT32;
      l_height_src = (res_y1 - res_y0) as OPJ_UINT32;
      /* Current tile component size*/
      /*if (i == 0) {
      fprintf(stdout, "SRC: l_res_x0=%d, l_res_x1=%d, l_res_y0=%d, l_res_y1=%d\n",
                      res_x0, res_x1, res_y0, res_y1);
      }*/
      /* Border of the current output component*/
      l_x0_dest = opj_uint_ceildivpow2((*l_img_comp_dest).x0, (*l_img_comp_dest).factor); /* can't overflow given that image->x1 is uint32 */
      l_y0_dest = opj_uint_ceildivpow2((*l_img_comp_dest).y0, (*l_img_comp_dest).factor);
      l_x1_dest = l_x0_dest.wrapping_add((*l_img_comp_dest).w);
      l_y1_dest = l_y0_dest.wrapping_add((*l_img_comp_dest).h);
      /*if (i == 0) {
      fprintf(stdout, "DEST: l_x0_dest=%d, l_x1_dest=%d, l_y0_dest=%d, l_y1_dest=%d (%d)\n",
                      l_x0_dest, l_x1_dest, l_y0_dest, l_y1_dest, l_img_comp_dest->factor );
      }*/
      /*-----*/
      /* Compute the area (l_offset_x0_src, l_offset_y0_src, l_offset_x1_src, l_offset_y1_src)
       * of the input buffer (decoded tile component) which will be move
       * in the output buffer. Compute the area of the output buffer (l_start_x_dest,
       * l_start_y_dest, l_width_dest, l_height_dest)  which will be modified
       * by this input area.
       * */

      assert!(res_x0 >= 0i32);
      assert!(res_x1 >= 0i32);
      if l_x0_dest < res_x0 as OPJ_UINT32 {
        l_start_x_dest = (res_x0 as OPJ_UINT32).wrapping_sub(l_x0_dest);
        l_offset_x0_src = 0i32;
        if l_x1_dest >= res_x1 as OPJ_UINT32 {
          l_width_dest = l_width_src;
          l_offset_x1_src = 0i32
        } else {
          l_width_dest = l_x1_dest.wrapping_sub(res_x0 as OPJ_UINT32);
          l_offset_x1_src = l_width_src.wrapping_sub(l_width_dest) as OPJ_INT32
        }
      } else {
        l_start_x_dest = 0u32;
        l_offset_x0_src = l_x0_dest as OPJ_INT32 - res_x0;
        if l_x1_dest >= res_x1 as OPJ_UINT32 {
          l_width_dest = l_width_src.wrapping_sub(l_offset_x0_src as OPJ_UINT32);
          l_offset_x1_src = 0i32
        } else {
          l_width_dest = (*l_img_comp_dest).w;
          l_offset_x1_src = res_x1 - l_x1_dest as OPJ_INT32
        }
      }
      if l_y0_dest < res_y0 as OPJ_UINT32 {
        l_start_y_dest = (res_y0 as OPJ_UINT32).wrapping_sub(l_y0_dest);
        l_offset_y0_src = 0i32;
        if l_y1_dest >= res_y1 as OPJ_UINT32 {
          l_height_dest = l_height_src;
          l_offset_y1_src = 0i32
        } else {
          l_height_dest = l_y1_dest.wrapping_sub(res_y0 as OPJ_UINT32);
          l_offset_y1_src = l_height_src.wrapping_sub(l_height_dest) as OPJ_INT32
        }
      } else {
        l_start_y_dest = 0u32;
        l_offset_y0_src = l_y0_dest as OPJ_INT32 - res_y0;
        if l_y1_dest >= res_y1 as OPJ_UINT32 {
          l_height_dest = l_height_src.wrapping_sub(l_offset_y0_src as OPJ_UINT32);
          l_offset_y1_src = 0i32
        } else {
          l_height_dest = (*l_img_comp_dest).h;
          l_offset_y1_src = res_y1 - l_y1_dest as OPJ_INT32
        }
      }
      if l_offset_x0_src < 0i32
        || l_offset_y0_src < 0i32
        || l_offset_x1_src < 0i32
        || l_offset_y1_src < 0i32
      {
        return 0i32;
      }
      /* testcase 2977.pdf.asan.67.2198 */
      if (l_width_dest as OPJ_INT32) < 0i32
        || (l_height_dest as OPJ_INT32) < 0i32
      {
        return 0i32;
      }
      /*-----*/
      /* Compute the input buffer offset */
      l_start_offset_src = (l_offset_x0_src as OPJ_SIZE_T)
        .wrapping_add((l_offset_y0_src as OPJ_SIZE_T).wrapping_mul(src_data_stride as OPJ_SIZE_T));
      /* Compute the output buffer offset */
      l_start_offset_dest = (l_start_x_dest as OPJ_SIZE_T).wrapping_add(
        (l_start_y_dest as OPJ_SIZE_T).wrapping_mul((*l_img_comp_dest).w as OPJ_SIZE_T),
      );
      /* Allocate output component buffer if necessary */
      if (*l_img_comp_dest).data.is_null()
        && l_start_offset_src == 0u64
        && l_start_offset_dest == 0u64
        && src_data_stride == (*l_img_comp_dest).w
        && l_width_dest == (*l_img_comp_dest).w
        && l_height_dest == (*l_img_comp_dest).h
      {
        /* If the final image matches the tile buffer, then borrow it */
        /* directly to save a copy */
        if (*p_tcd).whole_tile_decoding != 0 {
          (*l_img_comp_dest).data = (*l_tilec).data;
          (*l_tilec).data = 0 as *mut OPJ_INT32
        } else {
          (*l_img_comp_dest).data = (*l_tilec).data_win;
          (*l_tilec).data_win = 0 as *mut OPJ_INT32
        }
      } else {
        if (*l_img_comp_dest).data.is_null() {
          let mut l_width = (*l_img_comp_dest).w as OPJ_SIZE_T;
          let mut l_height = (*l_img_comp_dest).h as OPJ_SIZE_T;
          if l_height == 0u64
            || l_width > (18446744073709551615u64).wrapping_div(l_height)
            || l_width.wrapping_mul(l_height)
              > (18446744073709551615u64)
                .wrapping_div(core::mem::size_of::<OPJ_INT32>() as libc::c_ulong)
          {
            /* would overflow */
            return 0i32;
          }
          (*l_img_comp_dest).data = opj_image_data_alloc(
            l_width
              .wrapping_mul(l_height)
              .wrapping_mul(core::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
          ) as *mut OPJ_INT32;
          if (*l_img_comp_dest).data.is_null() {
            return 0i32;
          }
          if (*l_img_comp_dest).w != l_width_dest || (*l_img_comp_dest).h != l_height_dest {
            memset(
              (*l_img_comp_dest).data as *mut libc::c_void,
              0i32,
              ((*l_img_comp_dest).w as OPJ_SIZE_T)
                .wrapping_mul((*l_img_comp_dest).h as libc::c_ulong)
                .wrapping_mul(core::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
            );
          }
        }
        /* Move the output buffer to the first place where we will write*/
        l_dest_ptr = (*l_img_comp_dest).data.offset(l_start_offset_dest as isize);
        let mut l_src_ptr = p_src_data;
        l_src_ptr = l_src_ptr.offset(l_start_offset_src as isize);
        j = 0 as OPJ_UINT32;
        while j < l_height_dest {
          memcpy(
            l_dest_ptr as *mut libc::c_void,
            l_src_ptr as *const libc::c_void,
            (l_width_dest as libc::c_ulong)
              .wrapping_mul(core::mem::size_of::<OPJ_INT32>() as libc::c_ulong),
          );
          l_dest_ptr = l_dest_ptr.offset((*l_img_comp_dest).w as isize);
          l_src_ptr = l_src_ptr.offset(src_data_stride as isize);
          j = j.wrapping_add(1)
        }
      }
    }
    /* Happens for partial component decoding */
    i = i.wrapping_add(1);
    l_img_comp_dest = l_img_comp_dest.offset(1);
    l_img_comp_src = l_img_comp_src.offset(1);
    l_tilec = l_tilec.offset(1)
  }
  return 1i32;
}
unsafe fn opj_j2k_update_image_dimensions(
  mut p_image: *mut opj_image_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut it_comp: OPJ_UINT32 = 0;
  let mut l_comp_x1: OPJ_INT32 = 0;
  let mut l_comp_y1: OPJ_INT32 = 0;
  let mut l_img_comp = 0 as *mut opj_image_comp_t;
  l_img_comp = (*p_image).comps;
  it_comp = 0 as OPJ_UINT32;
  while it_comp < (*p_image).numcomps {
    let mut l_h: OPJ_INT32 = 0;
    let mut l_w: OPJ_INT32 = 0;
    if (*p_image).x0 > 2147483647 as OPJ_UINT32
      || (*p_image).y0 > 2147483647 as OPJ_UINT32
      || (*p_image).x1 > 2147483647 as OPJ_UINT32
      || (*p_image).y1 > 2147483647 as OPJ_UINT32
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Image coordinates above INT_MAX are not supported\n\x00" as *const u8
          as *const libc::c_char,
      );
      return 0i32;
    }
    (*l_img_comp).x0 =
      opj_int_ceildiv((*p_image).x0 as OPJ_INT32, (*l_img_comp).dx as OPJ_INT32) as OPJ_UINT32;
    (*l_img_comp).y0 =
      opj_int_ceildiv((*p_image).y0 as OPJ_INT32, (*l_img_comp).dy as OPJ_INT32) as OPJ_UINT32;
    l_comp_x1 = opj_int_ceildiv((*p_image).x1 as OPJ_INT32, (*l_img_comp).dx as OPJ_INT32);
    l_comp_y1 = opj_int_ceildiv((*p_image).y1 as OPJ_INT32, (*l_img_comp).dy as OPJ_INT32);
    l_w = opj_int_ceildivpow2(l_comp_x1, (*l_img_comp).factor as OPJ_INT32)
      - opj_int_ceildivpow2(
        (*l_img_comp).x0 as OPJ_INT32,
        (*l_img_comp).factor as OPJ_INT32,
      );
    if l_w < 0i32 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Size x of the decoded component image is incorrect (comp[%d].w=%d).\n\x00" as *const u8
          as *const libc::c_char,
        it_comp,
        l_w,
      );
      return 0i32;
    }
    (*l_img_comp).w = l_w as OPJ_UINT32;
    l_h = opj_int_ceildivpow2(l_comp_y1, (*l_img_comp).factor as OPJ_INT32)
      - opj_int_ceildivpow2(
        (*l_img_comp).y0 as OPJ_INT32,
        (*l_img_comp).factor as OPJ_INT32,
      );
    if l_h < 0i32 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Size y of the decoded component image is incorrect (comp[%d].h=%d).\n\x00" as *const u8
          as *const libc::c_char,
        it_comp,
        l_h,
      );
      return 0i32;
    }
    (*l_img_comp).h = l_h as OPJ_UINT32;
    l_img_comp = l_img_comp.offset(1);
    it_comp = it_comp.wrapping_add(1)
  }
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_set_decoded_components(
  mut p_j2k: *mut opj_j2k_t,
  mut numcomps: OPJ_UINT32,
  mut comps_indices: *const OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut already_mapped = 0 as *mut OPJ_BOOL;
  if (*p_j2k).m_private_image.is_null() {
    opj_event_msg(
      p_manager,
      1i32,
      b"opj_read_header() should be called before opj_set_decoded_components().\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 0i32;
  }
  already_mapped = opj_calloc(
    core::mem::size_of::<OPJ_BOOL>() as libc::c_ulong,
    (*(*p_j2k).m_private_image).numcomps as size_t,
  ) as *mut OPJ_BOOL;
  if already_mapped.is_null() {
    return 0i32;
  }
  i = 0 as OPJ_UINT32;
  while i < numcomps {
    if *comps_indices.offset(i as isize) >= (*(*p_j2k).m_private_image).numcomps {
      opj_event_msg(
        p_manager,
        1i32,
        b"Invalid component index: %u\n\x00" as *const u8 as *const libc::c_char,
        *comps_indices.offset(i as isize),
      );
      opj_free(already_mapped as *mut libc::c_void);
      return 0i32;
    }
    if *already_mapped.offset(*comps_indices.offset(i as isize) as isize) != 0 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Component index %u used several times\n\x00" as *const u8 as *const libc::c_char,
        *comps_indices.offset(i as isize),
      );
      opj_free(already_mapped as *mut libc::c_void);
      return 0i32;
    }
    *already_mapped.offset(*comps_indices.offset(i as isize) as isize) = 1i32;
    i = i.wrapping_add(1)
  }
  opj_free(already_mapped as *mut libc::c_void);
  opj_free(
    (*p_j2k)
      .m_specific_param
      .m_decoder
      .m_comps_indices_to_decode as *mut libc::c_void,
  );
  if numcomps != 0 {
    (*p_j2k)
      .m_specific_param
      .m_decoder
      .m_comps_indices_to_decode = opj_malloc(
      (numcomps as libc::c_ulong)
        .wrapping_mul(core::mem::size_of::<OPJ_UINT32>() as libc::c_ulong),
    ) as *mut OPJ_UINT32;
    if (*p_j2k)
      .m_specific_param
      .m_decoder
      .m_comps_indices_to_decode
      .is_null()
    {
      (*p_j2k).m_specific_param.m_decoder.m_numcomps_to_decode = 0 as OPJ_UINT32;
      return 0i32;
    }
    memcpy(
      (*p_j2k)
        .m_specific_param
        .m_decoder
        .m_comps_indices_to_decode as *mut libc::c_void,
      comps_indices as *const libc::c_void,
      (numcomps as libc::c_ulong)
        .wrapping_mul(core::mem::size_of::<OPJ_UINT32>() as libc::c_ulong),
    );
  } else {
    (*p_j2k)
      .m_specific_param
      .m_decoder
      .m_comps_indices_to_decode = 0 as *mut OPJ_UINT32
  }
  (*p_j2k).m_specific_param.m_decoder.m_numcomps_to_decode = numcomps;
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_set_decode_area(
  mut p_j2k: *mut opj_j2k_t,
  mut p_image: *mut opj_image_t,
  mut p_start_x: OPJ_INT32,
  mut p_start_y: OPJ_INT32,
  mut p_end_x: OPJ_INT32,
  mut p_end_y: OPJ_INT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_cp: *mut opj_cp_t = &mut (*p_j2k).m_cp;
  let mut l_image = (*p_j2k).m_private_image;
  let mut ret: OPJ_BOOL = 0;
  let mut it_comp: OPJ_UINT32 = 0;
  if !((*p_j2k).m_cp.tw == 1u32
    && (*p_j2k).m_cp.th == 1u32
    && !(*(*p_j2k).m_cp.tcps.offset(0))
      .m_data
      .is_null())
  {
    /* Check if we are read the main header */
    if (*p_j2k).m_specific_param.m_decoder.m_state
      != J2KState::TPHSOT
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Need to decode the main header before begin to decode the remaining codestream.\n\x00"
          as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
  }
  /* Update the comps[].factor member of the output image with the one */
  /* of m_reduce */
  it_comp = 0 as OPJ_UINT32;
  while it_comp < (*p_image).numcomps {
    (*(*p_image).comps.offset(it_comp as isize)).factor =
      (*p_j2k).m_cp.m_specific_param.m_dec.m_reduce;
    it_comp = it_comp.wrapping_add(1)
  }
  if p_start_x == 0 && p_start_y == 0 && p_end_x == 0 && p_end_y == 0 {
    opj_event_msg(
      p_manager,
      4i32,
      b"No decoded area parameters, set the decoded area to the whole image\n\x00" as *const u8
        as *const libc::c_char,
    );
    (*p_j2k).m_specific_param.m_decoder.m_start_tile_x = 0 as OPJ_UINT32;
    (*p_j2k).m_specific_param.m_decoder.m_start_tile_y = 0 as OPJ_UINT32;
    (*p_j2k).m_specific_param.m_decoder.m_end_tile_x = (*l_cp).tw;
    (*p_j2k).m_specific_param.m_decoder.m_end_tile_y = (*l_cp).th;
    (*p_image).x0 = (*l_image).x0;
    (*p_image).y0 = (*l_image).y0;
    (*p_image).x1 = (*l_image).x1;
    (*p_image).y1 = (*l_image).y1;
    return opj_j2k_update_image_dimensions(p_image, p_manager);
  }
  /* ----- */
  /* Check if the positions provided by the user are correct */
  /* Left */
  if p_start_x < 0i32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Left position of the decoded area (region_x0=%d) should be >= 0.\n\x00" as *const u8
        as *const libc::c_char,
      p_start_x,
    );
    return 0i32;
  } else {
    if p_start_x as OPJ_UINT32 > (*l_image).x1 {
      opj_event_msg(p_manager, 1i32,
                          b"Left position of the decoded area (region_x0=%d) is outside the image area (Xsiz=%d).\n\x00"
                              as *const u8 as *const libc::c_char, p_start_x,
                          (*l_image).x1);
      return 0i32;
    } else {
      if (p_start_x as OPJ_UINT32) < (*l_image).x0 {
        opj_event_msg(p_manager, 2i32,
                              b"Left position of the decoded area (region_x0=%d) is outside the image area (XOsiz=%d).\n\x00"
                                  as *const u8 as *const libc::c_char,
                              p_start_x, (*l_image).x0);
        (*p_j2k).m_specific_param.m_decoder.m_start_tile_x = 0 as OPJ_UINT32;
        (*p_image).x0 = (*l_image).x0
      } else {
        (*p_j2k).m_specific_param.m_decoder.m_start_tile_x = (p_start_x as OPJ_UINT32)
          .wrapping_sub((*l_cp).tx0)
          .wrapping_div((*l_cp).tdx);
        (*p_image).x0 = p_start_x as OPJ_UINT32
      }
    }
  }
  /* Up */
  if p_start_y < 0i32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Up position of the decoded area (region_y0=%d) should be >= 0.\n\x00" as *const u8
        as *const libc::c_char,
      p_start_y,
    );
    return 0i32;
  } else {
    if p_start_y as OPJ_UINT32 > (*l_image).y1 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Up position of the decoded area (region_y0=%d) is outside the image area (Ysiz=%d).\n\x00"
          as *const u8 as *const libc::c_char,
        p_start_y,
        (*l_image).y1,
      );
      return 0i32;
    } else {
      if (p_start_y as OPJ_UINT32) < (*l_image).y0 {
        opj_event_msg(p_manager, 2i32,
                              b"Up position of the decoded area (region_y0=%d) is outside the image area (YOsiz=%d).\n\x00"
                                  as *const u8 as *const libc::c_char,
                              p_start_y, (*l_image).y0);
        (*p_j2k).m_specific_param.m_decoder.m_start_tile_y = 0 as OPJ_UINT32;
        (*p_image).y0 = (*l_image).y0
      } else {
        (*p_j2k).m_specific_param.m_decoder.m_start_tile_y = (p_start_y as OPJ_UINT32)
          .wrapping_sub((*l_cp).ty0)
          .wrapping_div((*l_cp).tdy);
        (*p_image).y0 = p_start_y as OPJ_UINT32
      }
    }
  }
  /* Right */
  if p_end_x <= 0i32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Right position of the decoded area (region_x1=%d) should be > 0.\n\x00" as *const u8
        as *const libc::c_char,
      p_end_x,
    );
    return 0i32;
  } else {
    if (p_end_x as OPJ_UINT32) < (*l_image).x0 {
      opj_event_msg(p_manager, 1i32,
                          b"Right position of the decoded area (region_x1=%d) is outside the image area (XOsiz=%d).\n\x00"
                              as *const u8 as *const libc::c_char, p_end_x,
                          (*l_image).x0);
      return 0i32;
    } else {
      if p_end_x as OPJ_UINT32 > (*l_image).x1 {
        opj_event_msg(p_manager, 2i32,
                              b"Right position of the decoded area (region_x1=%d) is outside the image area (Xsiz=%d).\n\x00"
                                  as *const u8 as *const libc::c_char,
                              p_end_x, (*l_image).x1);
        (*p_j2k).m_specific_param.m_decoder.m_end_tile_x = (*l_cp).tw;
        (*p_image).x1 = (*l_image).x1
      } else {
        (*p_j2k).m_specific_param.m_decoder.m_end_tile_x =
          opj_int_ceildiv(p_end_x - (*l_cp).tx0 as OPJ_INT32, (*l_cp).tdx as OPJ_INT32)
            as OPJ_UINT32;
        (*p_image).x1 = p_end_x as OPJ_UINT32
      }
    }
  }
  /* Bottom */
  if p_end_y <= 0i32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Bottom position of the decoded area (region_y1=%d) should be > 0.\n\x00" as *const u8
        as *const libc::c_char,
      p_end_y,
    );
    return 0i32;
  } else {
    if (p_end_y as OPJ_UINT32) < (*l_image).y0 {
      opj_event_msg(p_manager, 1i32,
                          b"Bottom position of the decoded area (region_y1=%d) is outside the image area (YOsiz=%d).\n\x00"
                              as *const u8 as *const libc::c_char, p_end_y,
                          (*l_image).y0);
      return 0i32;
    }
  }
  if p_end_y as OPJ_UINT32 > (*l_image).y1 {
    opj_event_msg(p_manager, 2i32,
                      b"Bottom position of the decoded area (region_y1=%d) is outside the image area (Ysiz=%d).\n\x00"
                          as *const u8 as *const libc::c_char, p_end_y,
                      (*l_image).y1);
    (*p_j2k).m_specific_param.m_decoder.m_end_tile_y = (*l_cp).th;
    (*p_image).y1 = (*l_image).y1
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_end_tile_y =
      opj_int_ceildiv(p_end_y - (*l_cp).ty0 as OPJ_INT32, (*l_cp).tdy as OPJ_INT32) as OPJ_UINT32;
    (*p_image).y1 = p_end_y as OPJ_UINT32
  }
  /* ----- */
  (*p_j2k)
    .m_specific_param
    .m_decoder
    .set_m_discard_tiles(1 as OPJ_BITFIELD);
  ret = opj_j2k_update_image_dimensions(p_image, p_manager);
  if ret != 0 {
    opj_event_msg(
      p_manager,
      4i32,
      b"Setting decoding area to %d,%d,%d,%d\n\x00" as *const u8 as *const libc::c_char,
      (*p_image).x0,
      (*p_image).y0,
      (*p_image).x1,
      (*p_image).y1,
    );
  }
  return ret;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_create_decompress() -> *mut opj_j2k_t {
  let mut l_j2k = opj_calloc(
    1i32 as size_t,
    core::mem::size_of::<opj_j2k_t>() as libc::c_ulong,
  ) as *mut opj_j2k_t;
  if l_j2k.is_null() {
    return 0 as *mut opj_j2k_t;
  }
  (*l_j2k).m_is_decoder = 1i32;
  (*l_j2k)
    .m_cp
    .set_m_is_decoder(1 as OPJ_BITFIELD);
  /* in the absence of JP2 boxes, consider different bit depth / sign */
  /* per component is allowed */
  (*l_j2k)
    .m_cp
    .set_allow_different_bit_depth_sign(1 as OPJ_BITFIELD);
  /* Default to using strict mode. */
  (*l_j2k).m_cp.strict = 1i32;
  (*l_j2k).m_specific_param.m_decoder.m_default_tcp = opj_calloc(
    1i32 as size_t,
    core::mem::size_of::<opj_tcp_t>() as libc::c_ulong,
  ) as *mut opj_tcp_t;
  if (*l_j2k).m_specific_param.m_decoder.m_default_tcp.is_null() {
    opj_j2k_destroy(l_j2k);
    return 0 as *mut opj_j2k_t;
  }
  (*l_j2k).m_specific_param.m_decoder.m_header_data =
    opj_calloc(1i32 as size_t, 1000i32 as size_t) as *mut OPJ_BYTE;
  if (*l_j2k).m_specific_param.m_decoder.m_header_data.is_null() {
    opj_j2k_destroy(l_j2k);
    return 0 as *mut opj_j2k_t;
  }
  (*l_j2k).m_specific_param.m_decoder.m_header_data_size = 1000 as OPJ_UINT32;
  (*l_j2k).m_specific_param.m_decoder.m_tile_ind_to_dec = -(1i32);
  (*l_j2k).m_specific_param.m_decoder.m_last_sot_read_pos = 0 as OPJ_OFF_T;
  /* codestream index creation */
  (*l_j2k).cstr_index = opj_j2k_create_cstr_index();
  if (*l_j2k).cstr_index.is_null() {
    opj_j2k_destroy(l_j2k);
    return 0 as *mut opj_j2k_t;
  }
  /* validation list creation */
  (*l_j2k).m_validation_list = opj_procedure_list_create();
  if (*l_j2k).m_validation_list.is_null() {
    opj_j2k_destroy(l_j2k);
    return 0 as *mut opj_j2k_t;
  }
  /* execution list creation */
  (*l_j2k).m_procedure_list = opj_procedure_list_create();
  if (*l_j2k).m_procedure_list.is_null() {
    opj_j2k_destroy(l_j2k);
    return 0 as *mut opj_j2k_t;
  }
  (*l_j2k).m_tp = opj_thread_pool_create(opj_j2k_get_default_thread_count());
  if (*l_j2k).m_tp.is_null() {
    (*l_j2k).m_tp = opj_thread_pool_create(0i32)
  }
  if (*l_j2k).m_tp.is_null() {
    opj_j2k_destroy(l_j2k);
    return 0 as *mut opj_j2k_t;
  }
  return l_j2k;
}
unsafe fn opj_j2k_create_cstr_index() -> *mut opj_codestream_index_t {
  let mut cstr_index = opj_calloc(
    1i32 as size_t,
    core::mem::size_of::<opj_codestream_index_t>() as libc::c_ulong,
  ) as *mut opj_codestream_index_t;
  if cstr_index.is_null() {
    return 0 as *mut opj_codestream_index_t;
  }
  (*cstr_index).maxmarknum = 100 as OPJ_UINT32;
  (*cstr_index).marknum = 0 as OPJ_UINT32;
  (*cstr_index).marker = opj_calloc(
    (*cstr_index).maxmarknum as size_t,
    core::mem::size_of::<opj_marker_info_t>() as libc::c_ulong,
  ) as *mut opj_marker_info_t;
  if (*cstr_index).marker.is_null() {
    opj_free(cstr_index as *mut libc::c_void);
    return 0 as *mut opj_codestream_index_t;
  }
  (*cstr_index).tile_index = 0 as *mut opj_tile_index_t;
  return cstr_index;
}
/* *
 * Gets the size taken by writing a SPCod or SPCoc for the given tile and component.
 *
 * @param       p_j2k                   the J2K codec.
 * @param       p_tile_no               the tile index.
 * @param       p_comp_no               the component being outputted.
 *
 * @return      the number of bytes taken by the SPCod element.
 */
unsafe fn opj_j2k_get_SPCod_SPCoc_size(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_no: OPJ_UINT32,
  mut p_comp_no: OPJ_UINT32,
) -> OPJ_UINT32 {
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  /* preconditions */
  assert!(!p_j2k.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = &mut *(*l_cp).tcps.offset(p_tile_no as isize) as *mut opj_tcp_t;
  l_tccp = &mut *(*l_tcp).tccps.offset(p_comp_no as isize) as *mut opj_tccp_t;
  /* preconditions again */

  assert!(p_tile_no < (*l_cp).tw.wrapping_mul((*l_cp).th));
  assert!(p_comp_no < (*(*p_j2k).m_private_image).numcomps);
  if (*l_tccp).csty & 0x1u32 != 0 {
    return (5u32).wrapping_add((*l_tccp).numresolutions);
  } else {
    return 5 as OPJ_UINT32;
  };
}
/* *
 * Compare 2 a SPCod/ SPCoc elements, i.e. the coding style of a given component of a tile.
 *
 * @param       p_j2k            J2K codec.
 * @param       p_tile_no        Tile number
 * @param       p_first_comp_no  The 1st component number to compare.
 * @param       p_second_comp_no The 1st component number to compare.
 *
 * @return OPJ_TRUE if SPCdod are equals.
 */
unsafe fn opj_j2k_compare_SPCod_SPCoc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_no: OPJ_UINT32,
  mut p_first_comp_no: OPJ_UINT32,
  mut p_second_comp_no: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tccp0 = 0 as *mut opj_tccp_t;
  let mut l_tccp1 = 0 as *mut opj_tccp_t;
  /* preconditions */
  assert!(!p_j2k.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = &mut *(*l_cp).tcps.offset(p_tile_no as isize) as *mut opj_tcp_t;
  l_tccp0 = &mut *(*l_tcp).tccps.offset(p_first_comp_no as isize) as *mut opj_tccp_t;
  l_tccp1 = &mut *(*l_tcp).tccps.offset(p_second_comp_no as isize) as *mut opj_tccp_t;
  if (*l_tccp0).numresolutions != (*l_tccp1).numresolutions {
    return 0i32;
  }
  if (*l_tccp0).cblkw != (*l_tccp1).cblkw {
    return 0i32;
  }
  if (*l_tccp0).cblkh != (*l_tccp1).cblkh {
    return 0i32;
  }
  if (*l_tccp0).cblksty != (*l_tccp1).cblksty {
    return 0i32;
  }
  if (*l_tccp0).qmfbid != (*l_tccp1).qmfbid {
    return 0i32;
  }
  if (*l_tccp0).csty & 0x1u32
    != (*l_tccp1).csty & 0x1u32
  {
    return 0i32;
  }
  i = 0u32;
  while i < (*l_tccp0).numresolutions {
    if (*l_tccp0).prcw[i as usize] != (*l_tccp1).prcw[i as usize] {
      return 0i32;
    }
    if (*l_tccp0).prch[i as usize] != (*l_tccp1).prch[i as usize] {
      return 0i32;
    }
    i = i.wrapping_add(1)
  }
  return 1i32;
}
/* *
 * Writes a SPCod or SPCoc element, i.e. the coding style of a given component of a tile.
 *
 * @param       p_j2k           J2K codec.
 * @param       p_tile_no       FIXME DOC
 * @param       p_comp_no       the component number to output.
 * @param       p_data          FIXME DOC
 * @param       p_header_size   FIXME DOC
 * @param       p_manager       the user event manager.
 *
 * @return FIXME DOC
*/
unsafe extern "C" fn opj_j2k_write_SPCod_SPCoc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_no: OPJ_UINT32,
  mut p_comp_no: OPJ_UINT32,
  mut p_data: *mut OPJ_BYTE,
  mut p_header_size: *mut OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_header_size.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_data.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = &mut *(*l_cp).tcps.offset(p_tile_no as isize) as *mut opj_tcp_t;
  l_tccp = &mut *(*l_tcp).tccps.offset(p_comp_no as isize) as *mut opj_tccp_t;
  /* preconditions again */
  /* SPcoc (E) */
  assert!(p_tile_no < (*l_cp).tw.wrapping_mul((*l_cp).th));
  assert!(p_comp_no < (*(*p_j2k).m_private_image).numcomps); /* SPcoc (G) */
  if *p_header_size < 5u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error writing SPCod SPCoc element\n\x00" as *const u8 as *const libc::c_char,
    ); /* SPcoc (H) */
    return 0i32;
  } /* SPcoc (I_i) */
  opj_write_bytes_LE(
    p_data,
    (*l_tccp)
      .numresolutions
      .wrapping_sub(1u32),
    1 as OPJ_UINT32,
  );
  p_data = p_data.offset(1);
  opj_write_bytes_LE(
    p_data,
    (*l_tccp)
      .cblkw
      .wrapping_sub(2u32),
    1 as OPJ_UINT32,
  );
  p_data = p_data.offset(1);
  opj_write_bytes_LE(
    p_data,
    (*l_tccp)
      .cblkh
      .wrapping_sub(2u32),
    1 as OPJ_UINT32,
  );
  p_data = p_data.offset(1);
  opj_write_bytes_LE(p_data, (*l_tccp).cblksty, 1 as OPJ_UINT32);
  p_data = p_data.offset(1);
  opj_write_bytes_LE(p_data, (*l_tccp).qmfbid, 1 as OPJ_UINT32);
  p_data = p_data.offset(1);
  *p_header_size = (*p_header_size).wrapping_sub(5u32);
  if (*l_tccp).csty & 0x1u32 != 0 {
    if *p_header_size < (*l_tccp).numresolutions {
      opj_event_msg(
        p_manager,
        1i32,
        b"Error writing SPCod SPCoc element\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    i = 0 as OPJ_UINT32;
    while i < (*l_tccp).numresolutions {
      opj_write_bytes_LE(
        p_data,
        (*l_tccp).prcw[i as usize].wrapping_add((*l_tccp).prch[i as usize] << 4i32),
        1 as OPJ_UINT32,
      );
      p_data = p_data.offset(1);
      i = i.wrapping_add(1)
    }
    *p_header_size = (*p_header_size).wrapping_sub((*l_tccp).numresolutions)
  }
  return 1i32;
}
/* *
 * Reads a SPCod or SPCoc element, i.e. the coding style of a given component of a tile.
 * @param       p_j2k           the jpeg2000 codec.
 * @param       compno          FIXME DOC
 * @param       p_header_data   the data contained in the COM box.
 * @param       p_header_size   the size of the data contained in the COM marker.
 * @param       p_manager       the user event manager.
*/
unsafe extern "C" fn opj_j2k_read_SPCod_SPCoc(
  mut p_j2k: *mut opj_j2k_t,
  mut compno: OPJ_UINT32,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: *mut OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut l_tmp: OPJ_UINT32 = 0;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  let mut l_current_ptr = 0 as *mut OPJ_BYTE;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_header_data.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = if (*p_j2k).m_specific_param.m_decoder.m_state
    == J2KState::TPH
  {
    &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_default_tcp
  };
  /* precondition again */
  assert!(compno < (*(*p_j2k).m_private_image).numcomps);
  l_tccp = &mut *(*l_tcp).tccps.offset(compno as isize) as *mut opj_tccp_t;
  l_current_ptr = p_header_data;
  /* make sure room is sufficient */
  if *p_header_size < 5u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading SPCod SPCoc element\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* SPcod (D) / SPcoc (A) */
  opj_read_bytes_LE(
    l_current_ptr,
    &mut (*l_tccp).numresolutions,
    1 as OPJ_UINT32,
  ); /* tccp->numresolutions = read() + 1 */
  (*l_tccp).numresolutions = (*l_tccp).numresolutions.wrapping_add(1);
  if (*l_tccp).numresolutions > 33u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Invalid value for numresolutions : %d, max value is set in openjpeg.h at %d\n\x00"
        as *const u8 as *const libc::c_char,
      (*l_tccp).numresolutions,
      33i32,
    );
    return 0i32;
  }
  l_current_ptr = l_current_ptr.offset(1);
  /* If user wants to remove more resolutions than the codestream contains, return error */
  if (*l_cp).m_specific_param.m_dec.m_reduce >= (*l_tccp).numresolutions {
    opj_event_msg(p_manager, 1i32,
                      b"Error decoding component %d.\nThe number of resolutions to remove (%d) is greater or equal than the number of resolutions of this component (%d)\nModify the cp_reduce parameter.\n\n\x00"
                          as *const u8 as *const libc::c_char, compno,
                      (*l_cp).m_specific_param.m_dec.m_reduce,
                      (*l_tccp).numresolutions);
    (*p_j2k).m_specific_param.m_decoder.m_state |= J2KState::ERR;
    return 0i32;
  }
  /* SPcod (E) / SPcoc (B) */
  opj_read_bytes_LE(
    l_current_ptr,
    &mut (*l_tccp).cblkw,
    1 as OPJ_UINT32,
  );
  l_current_ptr = l_current_ptr.offset(1);
  (*l_tccp).cblkw = ((*l_tccp).cblkw as libc::c_uint).wrapping_add(2u32)
    as OPJ_UINT32;
  /* SPcod (F) / SPcoc (C) */
  opj_read_bytes_LE(
    l_current_ptr,
    &mut (*l_tccp).cblkh,
    1 as OPJ_UINT32,
  );
  l_current_ptr = l_current_ptr.offset(1);
  (*l_tccp).cblkh = ((*l_tccp).cblkh as libc::c_uint).wrapping_add(2u32)
    as OPJ_UINT32;
  if (*l_tccp).cblkw > 10u32
    || (*l_tccp).cblkh > 10u32
    || (*l_tccp).cblkw.wrapping_add((*l_tccp).cblkh) > 12u32
  {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading SPCod SPCoc element, Invalid cblkw/cblkh combination\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 0i32;
  }
  /* SPcod (G) / SPcoc (D) */
  opj_read_bytes_LE(
    l_current_ptr,
    &mut (*l_tccp).cblksty,
    1 as OPJ_UINT32,
  );
  l_current_ptr = l_current_ptr.offset(1);
  if (*l_tccp).cblksty & 0x80u32 != 0u32 {
    /* We do not support HT mixed mode yet.  For conformance, it should be supported.*/
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading SPCod SPCoc element. Unsupported Mixed HT code-block style found\n\x00"
        as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  /* SPcod (H) / SPcoc (E) */
  opj_read_bytes_LE(
    l_current_ptr,
    &mut (*l_tccp).qmfbid,
    1 as OPJ_UINT32,
  );
  l_current_ptr = l_current_ptr.offset(1);
  if (*l_tccp).qmfbid > 1u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading SPCod SPCoc element, Invalid transformation found\n\x00" as *const u8
        as *const libc::c_char,
    );
    return 0i32;
  }
  *p_header_size = (*p_header_size).wrapping_sub(5u32);
  /* use custom precinct size ? */
  if (*l_tccp).csty & 0x1u32 != 0 {
    if *p_header_size < (*l_tccp).numresolutions {
      opj_event_msg(
        p_manager,
        1i32,
        b"Error reading SPCod SPCoc element\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    /* SPcod (I_i) / SPcoc (F_i) */
    i = 0 as OPJ_UINT32;
    while i < (*l_tccp).numresolutions {
      opj_read_bytes_LE(l_current_ptr, &mut l_tmp, 1 as OPJ_UINT32);
      l_current_ptr = l_current_ptr.offset(1);
      /* Precinct exponent 0 is only allowed for lowest resolution level (Table A.21) */
      if i != 0u32
        && (l_tmp & 0xfu32 == 0u32
          || l_tmp >> 4i32 == 0u32)
      {
        opj_event_msg(
          p_manager,
          1i32,
          b"Invalid precinct size\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
      }
      (*l_tccp).prcw[i as usize] = l_tmp & 0xfu32;
      (*l_tccp).prch[i as usize] = l_tmp >> 4i32;
      i = i.wrapping_add(1)
    }
    *p_header_size = (*p_header_size).wrapping_sub((*l_tccp).numresolutions)
  } else {
    /* set default size for the precinct width and height */
    i = 0 as OPJ_UINT32;
    while i < (*l_tccp).numresolutions {
      (*l_tccp).prcw[i as usize] = 15 as OPJ_UINT32;
      (*l_tccp).prch[i as usize] = 15 as OPJ_UINT32;
      i = i.wrapping_add(1)
    }
  }
  return 1i32;
}
/* *
 * Copies the tile component parameters of all the component from the first tile component.
 *
 * @param               p_j2k           the J2k codec.
 */
unsafe fn opj_j2k_copy_tile_component_parameters(mut p_j2k: *mut opj_j2k_t) {
  /* loop */
  let mut i: OPJ_UINT32 = 0;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_ref_tccp = 0 as *mut opj_tccp_t;
  let mut l_copied_tccp = 0 as *mut opj_tccp_t;
  let mut l_prc_size: OPJ_UINT32 = 0;
  /* preconditions */
  assert!(!p_j2k.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = if (*p_j2k).m_specific_param.m_decoder.m_state
    == J2KState::TPH
  {
    &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_default_tcp
  };
  l_ref_tccp = &mut *(*l_tcp).tccps.offset(0) as *mut opj_tccp_t;
  l_copied_tccp = l_ref_tccp.offset(1);
  l_prc_size = (*l_ref_tccp)
    .numresolutions
    .wrapping_mul(core::mem::size_of::<OPJ_UINT32>() as OPJ_UINT32);
  i = 1 as OPJ_UINT32;
  while i < (*(*p_j2k).m_private_image).numcomps {
    (*l_copied_tccp).numresolutions = (*l_ref_tccp).numresolutions;
    (*l_copied_tccp).cblkw = (*l_ref_tccp).cblkw;
    (*l_copied_tccp).cblkh = (*l_ref_tccp).cblkh;
    (*l_copied_tccp).cblksty = (*l_ref_tccp).cblksty;
    (*l_copied_tccp).qmfbid = (*l_ref_tccp).qmfbid;
    memcpy(
      (*l_copied_tccp).prcw.as_mut_ptr() as *mut libc::c_void,
      (*l_ref_tccp).prcw.as_mut_ptr() as *const libc::c_void,
      l_prc_size as libc::c_ulong,
    );
    memcpy(
      (*l_copied_tccp).prch.as_mut_ptr() as *mut libc::c_void,
      (*l_ref_tccp).prch.as_mut_ptr() as *const libc::c_void,
      l_prc_size as libc::c_ulong,
    );
    l_copied_tccp = l_copied_tccp.offset(1);
    i = i.wrapping_add(1)
  }
}
/* *
 * Gets the size taken by writing SQcd or SQcc element, i.e. the quantization values of a band in the QCD or QCC.
 *
 * @param       p_tile_no               the tile index.
 * @param       p_comp_no               the component being outputted.
 * @param       p_j2k                   the J2K codec.
 *
 * @return      the number of bytes taken by the SPCod element.
 */
unsafe fn opj_j2k_get_SQcd_SQcc_size(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_no: OPJ_UINT32,
  mut p_comp_no: OPJ_UINT32,
) -> OPJ_UINT32 {
  let mut l_num_bands: OPJ_UINT32 = 0;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  /* preconditions */
  assert!(!p_j2k.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = &mut *(*l_cp).tcps.offset(p_tile_no as isize) as *mut opj_tcp_t;
  l_tccp = &mut *(*l_tcp).tccps.offset(p_comp_no as isize) as *mut opj_tccp_t;
  /* preconditions again */

  assert!(p_tile_no < (*l_cp).tw.wrapping_mul((*l_cp).th));
  assert!(p_comp_no < (*(*p_j2k).m_private_image).numcomps);
  l_num_bands = if (*l_tccp).qntsty == 1u32 {
    1u32
  } else {
    (*l_tccp)
      .numresolutions
      .wrapping_mul(3u32)
      .wrapping_sub(2u32)
  };
  if (*l_tccp).qntsty == 0u32 {
    return (1u32).wrapping_add(l_num_bands);
  } else {
    return (1u32)
      .wrapping_add((2u32).wrapping_mul(l_num_bands));
  };
}
/* *
 * Compares 2 SQcd or SQcc element, i.e. the quantization values of a band in the QCD or QCC.
 *
 * @param       p_j2k                   J2K codec.
 * @param       p_tile_no               the tile to output.
 * @param       p_first_comp_no         the first component number to compare.
 * @param       p_second_comp_no        the second component number to compare.
 *
 * @return OPJ_TRUE if equals.
 */
unsafe fn opj_j2k_compare_SQcd_SQcc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_no: OPJ_UINT32,
  mut p_first_comp_no: OPJ_UINT32,
  mut p_second_comp_no: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tccp0 = 0 as *mut opj_tccp_t;
  let mut l_tccp1 = 0 as *mut opj_tccp_t;
  let mut l_band_no: OPJ_UINT32 = 0;
  let mut l_num_bands: OPJ_UINT32 = 0;
  /* preconditions */
  assert!(!p_j2k.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = &mut *(*l_cp).tcps.offset(p_tile_no as isize) as *mut opj_tcp_t;
  l_tccp0 = &mut *(*l_tcp).tccps.offset(p_first_comp_no as isize) as *mut opj_tccp_t;
  l_tccp1 = &mut *(*l_tcp).tccps.offset(p_second_comp_no as isize) as *mut opj_tccp_t;
  if (*l_tccp0).qntsty != (*l_tccp1).qntsty {
    return 0i32;
  }
  if (*l_tccp0).numgbits != (*l_tccp1).numgbits {
    return 0i32;
  }
  if (*l_tccp0).qntsty == 1u32 {
    l_num_bands = 1u32
  } else {
    l_num_bands = (*l_tccp0)
      .numresolutions
      .wrapping_mul(3u32)
      .wrapping_sub(2u32);
    if l_num_bands
      != (*l_tccp1)
        .numresolutions
        .wrapping_mul(3u32)
        .wrapping_sub(2u32)
    {
      return 0i32;
    }
  }
  l_band_no = 0 as OPJ_UINT32;
  while l_band_no < l_num_bands {
    if (*l_tccp0).stepsizes[l_band_no as usize].expn
      != (*l_tccp1).stepsizes[l_band_no as usize].expn
    {
      return 0i32;
    }
    l_band_no = l_band_no.wrapping_add(1)
  }
  if (*l_tccp0).qntsty != 0u32 {
    l_band_no = 0 as OPJ_UINT32;
    while l_band_no < l_num_bands {
      if (*l_tccp0).stepsizes[l_band_no as usize].mant
        != (*l_tccp1).stepsizes[l_band_no as usize].mant
      {
        return 0i32;
      }
      l_band_no = l_band_no.wrapping_add(1)
    }
  }
  return 1i32;
}
/* *
 * Writes a SQcd or SQcc element, i.e. the quantization values of a band in the QCD or QCC.
 *
 * @param       p_tile_no               the tile to output.
 * @param       p_comp_no               the component number to output.
 * @param       p_data                  the data buffer.
 * @param       p_header_size   pointer to the size of the data buffer, it is changed by the function.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
 *
*/
unsafe extern "C" fn opj_j2k_write_SQcd_SQcc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_no: OPJ_UINT32,
  mut p_comp_no: OPJ_UINT32,
  mut p_data: *mut OPJ_BYTE,
  mut p_header_size: *mut OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut l_header_size: OPJ_UINT32 = 0;
  let mut l_band_no: OPJ_UINT32 = 0;
  let mut l_num_bands: OPJ_UINT32 = 0;
  let mut l_expn: OPJ_UINT32 = 0;
  let mut l_mant: OPJ_UINT32 = 0;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_header_size.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_data.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = &mut *(*l_cp).tcps.offset(p_tile_no as isize) as *mut opj_tcp_t;
  l_tccp = &mut *(*l_tcp).tccps.offset(p_comp_no as isize) as *mut opj_tccp_t;
  /* preconditions again */
  /* SPqcx_i */
  assert!(p_tile_no < (*l_cp).tw.wrapping_mul((*l_cp).th));
  assert!(p_comp_no < (*(*p_j2k).m_private_image).numcomps); /* SPqcx_i */
  l_num_bands = if (*l_tccp).qntsty == 1u32 {
    1u32
  } else {
    (*l_tccp)
      .numresolutions
      .wrapping_mul(3u32)
      .wrapping_sub(2u32)
  };
  if (*l_tccp).qntsty == 0u32 {
    l_header_size = (1u32).wrapping_add(l_num_bands);
    if *p_header_size < l_header_size {
      opj_event_msg(
        p_manager,
        1i32,
        b"Error writing SQcd SQcc element\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    opj_write_bytes_LE(
      p_data,
      (*l_tccp)
        .qntsty
        .wrapping_add((*l_tccp).numgbits << 5i32),
      1 as OPJ_UINT32,
    );
    p_data = p_data.offset(1);
    l_band_no = 0 as OPJ_UINT32;
    while l_band_no < l_num_bands {
      l_expn = (*l_tccp).stepsizes[l_band_no as usize].expn as OPJ_UINT32;
      opj_write_bytes_LE(
        p_data,
        l_expn << 3i32,
        1 as OPJ_UINT32,
      );
      p_data = p_data.offset(1);
      l_band_no = l_band_no.wrapping_add(1)
    }
  } else {
    l_header_size = (1u32)
      .wrapping_add((2u32).wrapping_mul(l_num_bands));
    if *p_header_size < l_header_size {
      opj_event_msg(
        p_manager,
        1i32,
        b"Error writing SQcd SQcc element\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    opj_write_bytes_LE(
      p_data,
      (*l_tccp)
        .qntsty
        .wrapping_add((*l_tccp).numgbits << 5i32),
      1 as OPJ_UINT32,
    );
    p_data = p_data.offset(1);
    l_band_no = 0 as OPJ_UINT32;
    while l_band_no < l_num_bands {
      l_expn = (*l_tccp).stepsizes[l_band_no as usize].expn as OPJ_UINT32;
      l_mant = (*l_tccp).stepsizes[l_band_no as usize].mant as OPJ_UINT32;
      opj_write_bytes_LE(
        p_data,
        (l_expn << 11i32).wrapping_add(l_mant),
        2 as OPJ_UINT32,
      );
      p_data = p_data.offset(2);
      l_band_no = l_band_no.wrapping_add(1)
    }
  }
  *p_header_size = (*p_header_size).wrapping_sub(l_header_size);
  return 1i32;
}
/* *
 * Reads a SQcd or SQcc element, i.e. the quantization values of a band in the QCD or QCC.
 *
 * @param       p_j2k           J2K codec.
 * @param       compno          the component number to output.
 * @param       p_header_data   the data buffer.
 * @param       p_header_size   pointer to the size of the data buffer, it is changed by the function.
 * @param       p_manager       the user event manager.
 *
*/
unsafe extern "C" fn opj_j2k_read_SQcd_SQcc(
  mut p_j2k: *mut opj_j2k_t,
  mut p_comp_no: OPJ_UINT32,
  mut p_header_data: *mut OPJ_BYTE,
  mut p_header_size: *mut OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* loop*/
  let mut l_band_no: OPJ_UINT32 = 0;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tccp = 0 as *mut opj_tccp_t;
  let mut l_current_ptr = 0 as *mut OPJ_BYTE;
  let mut l_tmp: OPJ_UINT32 = 0;
  let mut l_num_band: OPJ_UINT32 = 0;
  /* preconditions*/

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_header_data.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  /* come from tile part header or main header ?*/
  l_tcp = if (*p_j2k).m_specific_param.m_decoder.m_state
    == J2KState::TPH
  {
    &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_default_tcp
  };
  /* precondition again*/
  assert!(p_comp_no < (*(*p_j2k).m_private_image).numcomps);
  l_tccp = &mut *(*l_tcp).tccps.offset(p_comp_no as isize) as *mut opj_tccp_t;
  l_current_ptr = p_header_data;
  if *p_header_size < 1u32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error reading SQcd or SQcc element\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  *p_header_size = (*p_header_size as libc::c_uint).wrapping_sub(1u32)
    as OPJ_UINT32;
  opj_read_bytes_LE(l_current_ptr, &mut l_tmp, 1 as OPJ_UINT32);
  l_current_ptr = l_current_ptr.offset(1);
  (*l_tccp).qntsty = l_tmp & 0x1fu32;
  (*l_tccp).numgbits = l_tmp >> 5i32;
  if (*l_tccp).qntsty == 1u32 {
    l_num_band = 1 as OPJ_UINT32
  } else {
    l_num_band = if (*l_tccp).qntsty == 0u32 {
      *p_header_size
    } else {
      (*p_header_size).wrapping_div(2u32)
    };
    if l_num_band > (3i32 * 33i32 - 2i32) as libc::c_uint {
      opj_event_msg(p_manager, 2i32,
                          b"While reading CCP_QNTSTY element inside QCD or QCC marker segment, number of subbands (%d) is greater to OPJ_J2K_MAXBANDS (%d). So we limit the number of elements stored to OPJ_J2K_MAXBANDS (%d) and skip the rest. \n\x00"
                              as *const u8 as *const libc::c_char, l_num_band,
                          3i32 * 33i32 -
                              2i32,
                          3i32 * 33i32 -
                              2i32);
      /*return OPJ_FALSE;*/
    }
  }
  /* USE_JPWL */
  if (*l_tccp).qntsty == 0u32 {
    l_band_no = 0 as OPJ_UINT32; /* SPqcx_i */
    while l_band_no < l_num_band {
      opj_read_bytes_LE(l_current_ptr, &mut l_tmp, 1 as OPJ_UINT32); /* SPqcx_i */
      l_current_ptr = l_current_ptr.offset(1);
      if l_band_no < (3i32 * 33i32 - 2i32) as libc::c_uint {
        (*l_tccp).stepsizes[l_band_no as usize].expn = (l_tmp >> 3i32) as OPJ_INT32;
        (*l_tccp).stepsizes[l_band_no as usize].mant = 0i32
      }
      l_band_no = l_band_no.wrapping_add(1)
    }
    *p_header_size = (*p_header_size).wrapping_sub(l_num_band)
  } else {
    l_band_no = 0 as OPJ_UINT32;
    while l_band_no < l_num_band {
      opj_read_bytes_LE(l_current_ptr, &mut l_tmp, 2 as OPJ_UINT32);
      l_current_ptr = l_current_ptr.offset(2);
      if l_band_no < (3i32 * 33i32 - 2i32) as libc::c_uint {
        (*l_tccp).stepsizes[l_band_no as usize].expn = (l_tmp >> 11i32) as OPJ_INT32;
        (*l_tccp).stepsizes[l_band_no as usize].mant =
          (l_tmp & 0x7ffu32) as OPJ_INT32
      }
      l_band_no = l_band_no.wrapping_add(1)
    }
    *p_header_size =
      (*p_header_size).wrapping_sub((2u32).wrapping_mul(l_num_band))
  }
  /* Add Antonin : if scalar_derived -> compute other stepsizes */
  if (*l_tccp).qntsty == 1u32 {
    l_band_no = 1 as OPJ_UINT32;
    while l_band_no < (3i32 * 33i32 - 2i32) as libc::c_uint {
      (*l_tccp).stepsizes[l_band_no as usize].expn =
        if (*l_tccp).stepsizes[0 as usize].expn
          - l_band_no
            .wrapping_sub(1u32)
            .wrapping_div(3u32) as OPJ_INT32
          > 0i32
        {
          ((*l_tccp).stepsizes[0 as usize].expn)
            - l_band_no
              .wrapping_sub(1u32)
              .wrapping_div(3u32) as OPJ_INT32
        } else {
          0i32
        };
      (*l_tccp).stepsizes[l_band_no as usize].mant =
        (*l_tccp).stepsizes[0 as usize].mant;
      l_band_no = l_band_no.wrapping_add(1)
    }
  }
  return 1i32;
}
/* *
 * Copies the tile quantization parameters of all the component from the first tile component.
 *
 * @param               p_j2k           the J2k codec.
 */
unsafe fn opj_j2k_copy_tile_quantization_parameters(mut p_j2k: *mut opj_j2k_t) {
  let mut i: OPJ_UINT32 = 0;
  let mut l_cp = 0 as *mut opj_cp_t;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_ref_tccp = 0 as *mut opj_tccp_t;
  let mut l_copied_tccp = 0 as *mut opj_tccp_t;
  let mut l_size: OPJ_UINT32 = 0;
  /* preconditions */
  assert!(!p_j2k.is_null());
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = if (*p_j2k).m_specific_param.m_decoder.m_state
    == J2KState::TPH
  {
    &mut *(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize) as *mut opj_tcp_t
  } else {
    (*p_j2k).m_specific_param.m_decoder.m_default_tcp
  };
  l_ref_tccp = &mut *(*l_tcp).tccps.offset(0) as *mut opj_tccp_t;
  l_copied_tccp = l_ref_tccp.offset(1);
  l_size = ((3i32 * 33i32 - 2i32) as libc::c_ulong)
    .wrapping_mul(core::mem::size_of::<opj_stepsize_t>() as libc::c_ulong)
    as OPJ_UINT32;
  i = 1 as OPJ_UINT32;
  while i < (*(*p_j2k).m_private_image).numcomps {
    (*l_copied_tccp).qntsty = (*l_ref_tccp).qntsty;
    (*l_copied_tccp).numgbits = (*l_ref_tccp).numgbits;
    memcpy(
      (*l_copied_tccp).stepsizes.as_mut_ptr() as *mut libc::c_void,
      (*l_ref_tccp).stepsizes.as_mut_ptr() as *const libc::c_void,
      l_size as libc::c_ulong,
    );
    l_copied_tccp = l_copied_tccp.offset(1);
    i = i.wrapping_add(1)
  }
}
unsafe fn opj_j2k_dump_tile_info(
  mut l_default_tile: *mut opj_tcp_t,
  mut numcomps: OPJ_INT32,
  mut out_stream: *mut FILE,
) {
  if !l_default_tile.is_null() {
    let mut compno: OPJ_INT32 = 0;
    fprintf(
      out_stream,
      b"\t default tile {\n\x00" as *const u8 as *const libc::c_char,
    );
    fprintf(
      out_stream,
      b"\t\t csty=%#x\n\x00" as *const u8 as *const libc::c_char,
      (*l_default_tile).csty,
    );
    fprintf(
      out_stream,
      b"\t\t prg=%#x\n\x00" as *const u8 as *const libc::c_char,
      (*l_default_tile).prg as libc::c_int,
    );
    fprintf(
      out_stream,
      b"\t\t numlayers=%d\n\x00" as *const u8 as *const libc::c_char,
      (*l_default_tile).numlayers,
    );
    fprintf(
      out_stream,
      b"\t\t mct=%x\n\x00" as *const u8 as *const libc::c_char,
      (*l_default_tile).mct,
    );
    /*end of default tile*/
    compno = 0i32; /*end of component of default tile*/
    while compno < numcomps {
      let mut l_tccp: *mut opj_tccp_t =
        &mut *(*l_default_tile).tccps.offset(compno as isize) as *mut opj_tccp_t;
      let mut resno: OPJ_UINT32 = 0;
      let mut bandno: OPJ_INT32 = 0;
      let mut numbands: OPJ_INT32 = 0;
      /* coding style*/
      fprintf(
        out_stream,
        b"\t\t comp %d {\n\x00" as *const u8 as *const libc::c_char,
        compno,
      );
      fprintf(
        out_stream,
        b"\t\t\t csty=%#x\n\x00" as *const u8 as *const libc::c_char,
        (*l_tccp).csty,
      );
      fprintf(
        out_stream,
        b"\t\t\t numresolutions=%d\n\x00" as *const u8 as *const libc::c_char,
        (*l_tccp).numresolutions,
      );
      fprintf(
        out_stream,
        b"\t\t\t cblkw=2^%d\n\x00" as *const u8 as *const libc::c_char,
        (*l_tccp).cblkw,
      );
      fprintf(
        out_stream,
        b"\t\t\t cblkh=2^%d\n\x00" as *const u8 as *const libc::c_char,
        (*l_tccp).cblkh,
      );
      fprintf(
        out_stream,
        b"\t\t\t cblksty=%#x\n\x00" as *const u8 as *const libc::c_char,
        (*l_tccp).cblksty,
      );
      fprintf(
        out_stream,
        b"\t\t\t qmfbid=%d\n\x00" as *const u8 as *const libc::c_char,
        (*l_tccp).qmfbid,
      );
      fprintf(
        out_stream,
        b"\t\t\t preccintsize (w,h)=\x00" as *const u8 as *const libc::c_char,
      );
      resno = 0 as OPJ_UINT32;
      while resno < (*l_tccp).numresolutions {
        fprintf(
          out_stream,
          b"(%d,%d) \x00" as *const u8 as *const libc::c_char,
          (*l_tccp).prcw[resno as usize],
          (*l_tccp).prch[resno as usize],
        );
        resno = resno.wrapping_add(1)
      }
      fprintf(out_stream, b"\n\x00" as *const u8 as *const libc::c_char);
      /* quantization style*/
      fprintf(
        out_stream,
        b"\t\t\t qntsty=%d\n\x00" as *const u8 as *const libc::c_char,
        (*l_tccp).qntsty,
      );
      fprintf(
        out_stream,
        b"\t\t\t numgbits=%d\n\x00" as *const u8 as *const libc::c_char,
        (*l_tccp).numgbits,
      );
      fprintf(
        out_stream,
        b"\t\t\t stepsizes (m,e)=\x00" as *const u8 as *const libc::c_char,
      );
      numbands = if (*l_tccp).qntsty == 1u32 {
        1i32
      } else {
        ((*l_tccp).numresolutions as OPJ_INT32 * 3i32) - 2i32
      };
      bandno = 0i32;
      while bandno < numbands {
        fprintf(
          out_stream,
          b"(%d,%d) \x00" as *const u8 as *const libc::c_char,
          (*l_tccp).stepsizes[bandno as usize].mant,
          (*l_tccp).stepsizes[bandno as usize].expn,
        );
        bandno += 1
      }
      fprintf(out_stream, b"\n\x00" as *const u8 as *const libc::c_char);
      /* RGN value*/
      fprintf(
        out_stream,
        b"\t\t\t roishift=%d\n\x00" as *const u8 as *const libc::c_char,
        (*l_tccp).roishift,
      );
      fprintf(
        out_stream,
        b"\t\t }\n\x00" as *const u8 as *const libc::c_char,
      );
      compno += 1
    }
    fprintf(
      out_stream,
      b"\t }\n\x00" as *const u8 as *const libc::c_char,
    );
  };
}
#[no_mangle]
pub(crate) unsafe extern "C" fn j2k_dump(
  mut p_j2k: *mut opj_j2k_t,
  mut flag: OPJ_INT32,
  mut out_stream: *mut FILE,
) {
  /* Check if the flag is compatible with j2k file*/
  if flag & 128i32 != 0 || flag & 256i32 != 0 {
    fprintf(
      out_stream,
      b"Wrong flag\n\x00" as *const u8 as *const libc::c_char,
    );
    return;
  }
  /* Dump the image_header */
  if flag & 1i32 != 0 {
    if !(*p_j2k).m_private_image.is_null() {
      j2k_dump_image_header((*p_j2k).m_private_image, 0i32, out_stream);
    }
  }
  /* Dump the codestream info from main header */
  if flag & 2i32 != 0 {
    if !(*p_j2k).m_private_image.is_null() {
      opj_j2k_dump_MH_info(p_j2k, out_stream);
    }
  }
  /* Dump all tile/codestream info */
  if flag & 8i32 != 0 {
    let mut l_nb_tiles = (*p_j2k).m_cp.th.wrapping_mul((*p_j2k).m_cp.tw);
    let mut i: OPJ_UINT32 = 0;
    let mut l_tcp = (*p_j2k).m_cp.tcps;
    if !(*p_j2k).m_private_image.is_null() {
      i = 0 as OPJ_UINT32;
      while i < l_nb_tiles {
        opj_j2k_dump_tile_info(
          l_tcp,
          (*(*p_j2k).m_private_image).numcomps as OPJ_INT32,
          out_stream,
        );
        l_tcp = l_tcp.offset(1);
        i = i.wrapping_add(1)
      }
    }
  }
  /* Dump the codestream info of the current tile */
  if (flag & 4i32) != 0 {}
  /* Dump the codestream index from main header */
  if flag & 16i32 != 0 {
    opj_j2k_dump_MH_index(p_j2k, out_stream);
  }
  /* Dump the codestream index of the current tile */
  if (flag & 32i32) != 0 {}
}
unsafe fn opj_j2k_dump_MH_index(mut p_j2k: *mut opj_j2k_t, mut out_stream: *mut FILE) {
  let mut cstr_index = (*p_j2k).cstr_index;
  let mut it_marker: OPJ_UINT32 = 0;
  let mut it_tile: OPJ_UINT32 = 0;
  let mut it_tile_part: OPJ_UINT32 = 0;
  fprintf(
    out_stream,
    b"Codestream index from main header: {\n\x00" as *const u8 as *const libc::c_char,
  );
  fprintf(
    out_stream,
    b"\t Main header start position=%li\n\t Main header end position=%li\n\x00" as *const u8
      as *const libc::c_char,
    (*cstr_index).main_head_start,
    (*cstr_index).main_head_end,
  );
  fprintf(
    out_stream,
    b"\t Marker list: {\n\x00" as *const u8 as *const libc::c_char,
  );
  if !(*cstr_index).marker.is_null() {
    it_marker = 0 as OPJ_UINT32;
    while it_marker < (*cstr_index).marknum {
      fprintf(
        out_stream,
        b"\t\t type=%#x, pos=%li, len=%d\n\x00" as *const u8 as *const libc::c_char,
        (*(*cstr_index).marker.offset(it_marker as isize)).type_0 as libc::c_int,
        (*(*cstr_index).marker.offset(it_marker as isize)).pos,
        (*(*cstr_index).marker.offset(it_marker as isize)).len,
      );
      it_marker = it_marker.wrapping_add(1)
    }
  }
  fprintf(
    out_stream,
    b"\t }\n\x00" as *const u8 as *const libc::c_char,
  );
  if !(*cstr_index).tile_index.is_null() {
    /* Simple test to avoid to write empty information*/
    let mut l_acc_nb_of_tile_part = 0 as OPJ_UINT32; /* Not fill from the main header*/
    it_tile = 0 as OPJ_UINT32;
    while it_tile < (*cstr_index).nb_of_tiles {
      l_acc_nb_of_tile_part = (l_acc_nb_of_tile_part as libc::c_uint)
        .wrapping_add((*(*cstr_index).tile_index.offset(it_tile as isize)).nb_tps)
        as OPJ_UINT32;
      it_tile = it_tile.wrapping_add(1)
    }
    if l_acc_nb_of_tile_part != 0 {
      fprintf(
        out_stream,
        b"\t Tile index: {\n\x00" as *const u8 as *const libc::c_char,
      );
      it_tile = 0 as OPJ_UINT32;
      while it_tile < (*cstr_index).nb_of_tiles {
        let mut nb_of_tile_part = (*(*cstr_index).tile_index.offset(it_tile as isize)).nb_tps;
        fprintf(
          out_stream,
          b"\t\t nb of tile-part in tile [%d]=%d\n\x00" as *const u8 as *const libc::c_char,
          it_tile,
          nb_of_tile_part,
        );
        if !(*(*cstr_index).tile_index.offset(it_tile as isize))
          .tp_index
          .is_null()
        {
          it_tile_part = 0 as OPJ_UINT32;
          while it_tile_part < nb_of_tile_part {
            fprintf(
              out_stream,
              b"\t\t\t tile-part[%d]: star_pos=%li, end_header=%li, end_pos=%li.\n\x00" as *const u8
                as *const libc::c_char,
              it_tile_part,
              (*(*(*cstr_index).tile_index.offset(it_tile as isize))
                .tp_index
                .offset(it_tile_part as isize))
              .start_pos,
              (*(*(*cstr_index).tile_index.offset(it_tile as isize))
                .tp_index
                .offset(it_tile_part as isize))
              .end_header,
              (*(*(*cstr_index).tile_index.offset(it_tile as isize))
                .tp_index
                .offset(it_tile_part as isize))
              .end_pos,
            );
            it_tile_part = it_tile_part.wrapping_add(1)
          }
        }
        if !(*(*cstr_index).tile_index.offset(it_tile as isize))
          .marker
          .is_null()
        {
          it_marker = 0 as OPJ_UINT32;
          while it_marker < (*(*cstr_index).tile_index.offset(it_tile as isize)).marknum {
            fprintf(
              out_stream,
              b"\t\t type=%#x, pos=%li, len=%d\n\x00" as *const u8 as *const libc::c_char,
              (*(*(*cstr_index).tile_index.offset(it_tile as isize))
                .marker
                .offset(it_marker as isize))
              .type_0 as libc::c_int,
              (*(*(*cstr_index).tile_index.offset(it_tile as isize))
                .marker
                .offset(it_marker as isize))
              .pos,
              (*(*(*cstr_index).tile_index.offset(it_tile as isize))
                .marker
                .offset(it_marker as isize))
              .len,
            );
            it_marker = it_marker.wrapping_add(1)
          }
        }
        it_tile = it_tile.wrapping_add(1)
      }
      fprintf(
        out_stream,
        b"\t }\n\x00" as *const u8 as *const libc::c_char,
      );
    }
  }
  fprintf(out_stream, b"}\n\x00" as *const u8 as *const libc::c_char);
}
unsafe fn opj_j2k_dump_MH_info(mut p_j2k: *mut opj_j2k_t, mut out_stream: *mut FILE) {
  fprintf(
    out_stream,
    b"Codestream info from main header: {\n\x00" as *const u8 as *const libc::c_char,
  );
  fprintf(
    out_stream,
    b"\t tx0=%d, ty0=%d\n\x00" as *const u8 as *const libc::c_char,
    (*p_j2k).m_cp.tx0,
    (*p_j2k).m_cp.ty0,
  );
  fprintf(
    out_stream,
    b"\t tdx=%d, tdy=%d\n\x00" as *const u8 as *const libc::c_char,
    (*p_j2k).m_cp.tdx,
    (*p_j2k).m_cp.tdy,
  );
  fprintf(
    out_stream,
    b"\t tw=%d, th=%d\n\x00" as *const u8 as *const libc::c_char,
    (*p_j2k).m_cp.tw,
    (*p_j2k).m_cp.th,
  );
  opj_j2k_dump_tile_info(
    (*p_j2k).m_specific_param.m_decoder.m_default_tcp,
    (*(*p_j2k).m_private_image).numcomps as OPJ_INT32,
    out_stream,
  );
  fprintf(out_stream, b"}\n\x00" as *const u8 as *const libc::c_char);
}
#[no_mangle]
pub(crate) unsafe extern "C" fn j2k_dump_image_header(
  mut img_header: *mut opj_image_t,
  mut dev_dump_flag: OPJ_BOOL,
  mut out_stream: *mut FILE,
) {
  let mut tab: [libc::c_char; 2] = [0; 2];
  if dev_dump_flag != 0 {
    fprintf(
      stdout,
      b"[DEV] Dump an image_header struct {\n\x00" as *const u8 as *const libc::c_char,
    );
    tab[0 as usize] = '\u{0}' as i32 as libc::c_char
  } else {
    fprintf(
      out_stream,
      b"Image info {\n\x00" as *const u8 as *const libc::c_char,
    );
    tab[0 as usize] = '\t' as i32 as libc::c_char;
    tab[1 as usize] = '\u{0}' as i32 as libc::c_char
  }
  fprintf(
    out_stream,
    b"%s x0=%d, y0=%d\n\x00" as *const u8 as *const libc::c_char,
    tab.as_mut_ptr(),
    (*img_header).x0,
    (*img_header).y0,
  );
  fprintf(
    out_stream,
    b"%s x1=%d, y1=%d\n\x00" as *const u8 as *const libc::c_char,
    tab.as_mut_ptr(),
    (*img_header).x1,
    (*img_header).y1,
  );
  fprintf(
    out_stream,
    b"%s numcomps=%d\n\x00" as *const u8 as *const libc::c_char,
    tab.as_mut_ptr(),
    (*img_header).numcomps,
  );
  if !(*img_header).comps.is_null() {
    let mut compno: OPJ_UINT32 = 0;
    compno = 0 as OPJ_UINT32;
    while compno < (*img_header).numcomps {
      fprintf(
        out_stream,
        b"%s\t component %d {\n\x00" as *const u8 as *const libc::c_char,
        tab.as_mut_ptr(),
        compno,
      );
      j2k_dump_image_comp_header(
        &mut *(*img_header).comps.offset(compno as isize),
        dev_dump_flag,
        out_stream,
      );
      fprintf(
        out_stream,
        b"%s}\n\x00" as *const u8 as *const libc::c_char,
        tab.as_mut_ptr(),
      );
      compno = compno.wrapping_add(1)
    }
  }
  fprintf(out_stream, b"}\n\x00" as *const u8 as *const libc::c_char);
}
#[no_mangle]
pub(crate) unsafe extern "C" fn j2k_dump_image_comp_header(
  mut comp_header: *mut opj_image_comp_t,
  mut dev_dump_flag: OPJ_BOOL,
  mut out_stream: *mut FILE,
) {
  let mut tab: [libc::c_char; 3] = [0; 3];
  if dev_dump_flag != 0 {
    fprintf(
      stdout,
      b"[DEV] Dump an image_comp_header struct {\n\x00" as *const u8 as *const libc::c_char,
    );
    tab[0 as usize] = '\u{0}' as i32 as libc::c_char
  } else {
    tab[0 as usize] = '\t' as i32 as libc::c_char;
    tab[1 as usize] = '\t' as i32 as libc::c_char;
    tab[2 as usize] = '\u{0}' as i32 as libc::c_char
  }
  fprintf(
    out_stream,
    b"%s dx=%d, dy=%d\n\x00" as *const u8 as *const libc::c_char,
    tab.as_mut_ptr(),
    (*comp_header).dx,
    (*comp_header).dy,
  );
  fprintf(
    out_stream,
    b"%s prec=%d\n\x00" as *const u8 as *const libc::c_char,
    tab.as_mut_ptr(),
    (*comp_header).prec,
  );
  fprintf(
    out_stream,
    b"%s sgnd=%d\n\x00" as *const u8 as *const libc::c_char,
    tab.as_mut_ptr(),
    (*comp_header).sgnd,
  );
  if dev_dump_flag != 0 {
    fprintf(out_stream, b"}\n\x00" as *const u8 as *const libc::c_char);
  };
}
#[no_mangle]
pub(crate) unsafe extern "C" fn j2k_get_cstr_info(
  mut p_j2k: *mut opj_j2k_t,
) -> *mut opj_codestream_info_v2_t {
  let mut compno: OPJ_UINT32 = 0;
  let mut numcomps = (*(*p_j2k).m_private_image).numcomps;
  let mut l_default_tile = 0 as *mut opj_tcp_t;
  let mut cstr_info = opj_calloc(
    1i32 as size_t,
    core::mem::size_of::<opj_codestream_info_v2_t>() as libc::c_ulong,
  ) as *mut opj_codestream_info_v2_t;
  if cstr_info.is_null() {
    return 0 as *mut opj_codestream_info_v2_t;
  }
  (*cstr_info).nbcomps = (*(*p_j2k).m_private_image).numcomps;
  (*cstr_info).tx0 = (*p_j2k).m_cp.tx0;
  (*cstr_info).ty0 = (*p_j2k).m_cp.ty0;
  (*cstr_info).tdx = (*p_j2k).m_cp.tdx;
  (*cstr_info).tdy = (*p_j2k).m_cp.tdy;
  (*cstr_info).tw = (*p_j2k).m_cp.tw;
  (*cstr_info).th = (*p_j2k).m_cp.th;
  (*cstr_info).tile_info = 0 as *mut opj_tile_info_v2_t;
  l_default_tile = (*p_j2k).m_specific_param.m_decoder.m_default_tcp;
  (*cstr_info).m_default_tile_info.csty = (*l_default_tile).csty;
  (*cstr_info).m_default_tile_info.prg = (*l_default_tile).prg;
  (*cstr_info).m_default_tile_info.numlayers = (*l_default_tile).numlayers;
  (*cstr_info).m_default_tile_info.mct = (*l_default_tile).mct;
  (*cstr_info).m_default_tile_info.tccp_info = opj_calloc(
    (*cstr_info).nbcomps as size_t,
    core::mem::size_of::<opj_tccp_info_t>() as libc::c_ulong,
  ) as *mut opj_tccp_info_t;
  if (*cstr_info).m_default_tile_info.tccp_info.is_null() {
    opj_destroy_cstr_info(&mut cstr_info);
    return 0 as *mut opj_codestream_info_v2_t;
  }
  compno = 0 as OPJ_UINT32;
  while compno < numcomps {
    let mut l_tccp: *mut opj_tccp_t =
      &mut *(*l_default_tile).tccps.offset(compno as isize) as *mut opj_tccp_t;
    let mut l_tccp_info: *mut opj_tccp_info_t = &mut *(*cstr_info)
      .m_default_tile_info
      .tccp_info
      .offset(compno as isize)
      as *mut opj_tccp_info_t;
    let mut bandno: OPJ_INT32 = 0;
    let mut numbands: OPJ_INT32 = 0;
    /* coding style*/
    (*l_tccp_info).csty = (*l_tccp).csty;
    (*l_tccp_info).numresolutions = (*l_tccp).numresolutions;
    (*l_tccp_info).cblkw = (*l_tccp).cblkw;
    (*l_tccp_info).cblkh = (*l_tccp).cblkh;
    (*l_tccp_info).cblksty = (*l_tccp).cblksty;
    (*l_tccp_info).qmfbid = (*l_tccp).qmfbid;
    if (*l_tccp).numresolutions < 33u32 {
      memcpy(
        (*l_tccp_info).prch.as_mut_ptr() as *mut libc::c_void,
        (*l_tccp).prch.as_mut_ptr() as *const libc::c_void,
        (*l_tccp).numresolutions as libc::c_ulong,
      );
      memcpy(
        (*l_tccp_info).prcw.as_mut_ptr() as *mut libc::c_void,
        (*l_tccp).prcw.as_mut_ptr() as *const libc::c_void,
        (*l_tccp).numresolutions as libc::c_ulong,
      );
    }
    /* quantization style*/
    (*l_tccp_info).qntsty = (*l_tccp).qntsty;
    (*l_tccp_info).numgbits = (*l_tccp).numgbits;
    numbands = if (*l_tccp).qntsty == 1u32 {
      1i32
    } else {
      ((*l_tccp).numresolutions as OPJ_INT32 * 3i32) - 2i32
    };
    if numbands < 3i32 * 33i32 - 2i32 {
      bandno = 0i32;
      while bandno < numbands {
        (*l_tccp_info).stepsizes_mant[bandno as usize] =
          (*l_tccp).stepsizes[bandno as usize].mant as OPJ_UINT32;
        (*l_tccp_info).stepsizes_expn[bandno as usize] =
          (*l_tccp).stepsizes[bandno as usize].expn as OPJ_UINT32;
        bandno += 1
      }
    }
    /* RGN value*/
    (*l_tccp_info).roishift = (*l_tccp).roishift;
    compno = compno.wrapping_add(1)
  }
  return cstr_info;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn j2k_get_cstr_index(
  mut p_j2k: *mut opj_j2k_t,
) -> *mut opj_codestream_index_t {
  let mut l_cstr_index = opj_calloc(
    1i32 as size_t,
    core::mem::size_of::<opj_codestream_index_t>() as libc::c_ulong,
  ) as *mut opj_codestream_index_t;
  if l_cstr_index.is_null() {
    return 0 as *mut opj_codestream_index_t;
  }
  (*l_cstr_index).main_head_start = (*(*p_j2k).cstr_index).main_head_start;
  (*l_cstr_index).main_head_end = (*(*p_j2k).cstr_index).main_head_end;
  (*l_cstr_index).codestream_size = (*(*p_j2k).cstr_index).codestream_size;
  (*l_cstr_index).marknum = (*(*p_j2k).cstr_index).marknum;
  (*l_cstr_index).marker = opj_malloc(
    ((*l_cstr_index).marknum as libc::c_ulong)
      .wrapping_mul(core::mem::size_of::<opj_marker_info_t>() as libc::c_ulong),
  ) as *mut opj_marker_info_t;
  if (*l_cstr_index).marker.is_null() {
    opj_free(l_cstr_index as *mut libc::c_void);
    return 0 as *mut opj_codestream_index_t;
  }
  if !(*(*p_j2k).cstr_index).marker.is_null() {
    memcpy(
      (*l_cstr_index).marker as *mut libc::c_void,
      (*(*p_j2k).cstr_index).marker as *const libc::c_void,
      ((*l_cstr_index).marknum as libc::c_ulong)
        .wrapping_mul(core::mem::size_of::<opj_marker_info_t>() as libc::c_ulong),
    );
  } else {
    opj_free((*l_cstr_index).marker as *mut libc::c_void);
    (*l_cstr_index).marker = 0 as *mut opj_marker_info_t
  }
  (*l_cstr_index).nb_of_tiles = (*(*p_j2k).cstr_index).nb_of_tiles;
  (*l_cstr_index).tile_index = opj_calloc(
    (*l_cstr_index).nb_of_tiles as size_t,
    core::mem::size_of::<opj_tile_index_t>() as libc::c_ulong,
  ) as *mut opj_tile_index_t;
  if (*l_cstr_index).tile_index.is_null() {
    opj_free((*l_cstr_index).marker as *mut libc::c_void);
    opj_free(l_cstr_index as *mut libc::c_void);
    return 0 as *mut opj_codestream_index_t;
  }
  if (*(*p_j2k).cstr_index).tile_index.is_null() {
    opj_free((*l_cstr_index).tile_index as *mut libc::c_void);
    (*l_cstr_index).tile_index = 0 as *mut opj_tile_index_t
  } else {
    let mut it_tile = 0 as OPJ_UINT32;
    it_tile = 0 as OPJ_UINT32;
    while it_tile < (*l_cstr_index).nb_of_tiles {
      /* Tile Marker*/
      (*(*l_cstr_index).tile_index.offset(it_tile as isize)).marknum =
        (*(*(*p_j2k).cstr_index).tile_index.offset(it_tile as isize)).marknum;
      let ref mut fresh34 = (*(*l_cstr_index).tile_index.offset(it_tile as isize)).marker;
      *fresh34 = opj_malloc(
        ((*(*l_cstr_index).tile_index.offset(it_tile as isize)).marknum as libc::c_ulong)
          .wrapping_mul(core::mem::size_of::<opj_marker_info_t>() as libc::c_ulong),
      ) as *mut opj_marker_info_t;
      if (*(*l_cstr_index).tile_index.offset(it_tile as isize))
        .marker
        .is_null()
      {
        let mut it_tile_free: OPJ_UINT32 = 0;
        it_tile_free = 0 as OPJ_UINT32;
        while it_tile_free < it_tile {
          opj_free(
            (*(*l_cstr_index).tile_index.offset(it_tile_free as isize)).marker as *mut libc::c_void,
          );
          it_tile_free = it_tile_free.wrapping_add(1)
        }
        opj_free((*l_cstr_index).tile_index as *mut libc::c_void);
        opj_free((*l_cstr_index).marker as *mut libc::c_void);
        opj_free(l_cstr_index as *mut libc::c_void);
        return 0 as *mut opj_codestream_index_t;
      }
      if !(*(*(*p_j2k).cstr_index).tile_index.offset(it_tile as isize))
        .marker
        .is_null()
      {
        memcpy(
          (*(*l_cstr_index).tile_index.offset(it_tile as isize)).marker as *mut libc::c_void,
          (*(*(*p_j2k).cstr_index).tile_index.offset(it_tile as isize)).marker
            as *const libc::c_void,
          ((*(*l_cstr_index).tile_index.offset(it_tile as isize)).marknum as libc::c_ulong)
            .wrapping_mul(core::mem::size_of::<opj_marker_info_t>() as libc::c_ulong),
        );
      } else {
        opj_free(
          (*(*l_cstr_index).tile_index.offset(it_tile as isize)).marker as *mut libc::c_void,
        );
        let ref mut fresh35 = (*(*l_cstr_index).tile_index.offset(it_tile as isize)).marker;
        *fresh35 = 0 as *mut opj_marker_info_t
      }
      /* Tile part index*/
      (*(*l_cstr_index).tile_index.offset(it_tile as isize)).nb_tps =
        (*(*(*p_j2k).cstr_index).tile_index.offset(it_tile as isize)).nb_tps;
      let ref mut fresh36 = (*(*l_cstr_index).tile_index.offset(it_tile as isize)).tp_index;
      *fresh36 = opj_malloc(
        ((*(*l_cstr_index).tile_index.offset(it_tile as isize)).nb_tps as libc::c_ulong)
          .wrapping_mul(core::mem::size_of::<opj_tp_index_t>() as libc::c_ulong),
      ) as *mut opj_tp_index_t;
      if (*(*l_cstr_index).tile_index.offset(it_tile as isize))
        .tp_index
        .is_null()
      {
        let mut it_tile_free_0: OPJ_UINT32 = 0;
        it_tile_free_0 = 0 as OPJ_UINT32;
        while it_tile_free_0 < it_tile {
          opj_free(
            (*(*l_cstr_index).tile_index.offset(it_tile_free_0 as isize)).marker
              as *mut libc::c_void,
          );
          opj_free(
            (*(*l_cstr_index).tile_index.offset(it_tile_free_0 as isize)).tp_index
              as *mut libc::c_void,
          );
          it_tile_free_0 = it_tile_free_0.wrapping_add(1)
        }
        opj_free((*l_cstr_index).tile_index as *mut libc::c_void);
        opj_free((*l_cstr_index).marker as *mut libc::c_void);
        opj_free(l_cstr_index as *mut libc::c_void);
        return 0 as *mut opj_codestream_index_t;
      }
      if !(*(*(*p_j2k).cstr_index).tile_index.offset(it_tile as isize))
        .tp_index
        .is_null()
      {
        memcpy(
          (*(*l_cstr_index).tile_index.offset(it_tile as isize)).tp_index as *mut libc::c_void,
          (*(*(*p_j2k).cstr_index).tile_index.offset(it_tile as isize)).tp_index
            as *const libc::c_void,
          ((*(*l_cstr_index).tile_index.offset(it_tile as isize)).nb_tps as libc::c_ulong)
            .wrapping_mul(core::mem::size_of::<opj_tp_index_t>() as libc::c_ulong),
        );
      } else {
        opj_free(
          (*(*l_cstr_index).tile_index.offset(it_tile as isize)).tp_index as *mut libc::c_void,
        );
        let ref mut fresh37 = (*(*l_cstr_index).tile_index.offset(it_tile as isize)).tp_index;
        *fresh37 = 0 as *mut opj_tp_index_t
      }
      /* Packet index (NOT USED)*/
      (*(*l_cstr_index).tile_index.offset(it_tile as isize)).nb_packet =
        0 as OPJ_UINT32;
      let ref mut fresh38 = (*(*l_cstr_index).tile_index.offset(it_tile as isize)).packet_index;
      *fresh38 = 0 as *mut opj_packet_info_t;
      it_tile = it_tile.wrapping_add(1)
    }
  }
  return l_cstr_index;
}
unsafe fn opj_j2k_allocate_tile_element_cstr_index(
  mut p_j2k: *mut opj_j2k_t,
) -> OPJ_BOOL {
  let mut it_tile = 0 as OPJ_UINT32;
  (*(*p_j2k).cstr_index).nb_of_tiles = (*p_j2k).m_cp.tw.wrapping_mul((*p_j2k).m_cp.th);
  (*(*p_j2k).cstr_index).tile_index = opj_calloc(
    (*(*p_j2k).cstr_index).nb_of_tiles as size_t,
    core::mem::size_of::<opj_tile_index_t>() as libc::c_ulong,
  ) as *mut opj_tile_index_t;
  if (*(*p_j2k).cstr_index).tile_index.is_null() {
    return 0i32;
  }
  it_tile = 0 as OPJ_UINT32;
  while it_tile < (*(*p_j2k).cstr_index).nb_of_tiles {
    (*(*(*p_j2k).cstr_index).tile_index.offset(it_tile as isize)).maxmarknum =
      100 as OPJ_UINT32;
    (*(*(*p_j2k).cstr_index).tile_index.offset(it_tile as isize)).marknum =
      0 as OPJ_UINT32;
    let ref mut fresh39 = (*(*(*p_j2k).cstr_index).tile_index.offset(it_tile as isize)).marker;
    *fresh39 = opj_calloc(
      (*(*(*p_j2k).cstr_index).tile_index.offset(it_tile as isize)).maxmarknum as size_t,
      core::mem::size_of::<opj_marker_info_t>() as libc::c_ulong,
    ) as *mut opj_marker_info_t;
    if (*(*(*p_j2k).cstr_index).tile_index.offset(it_tile as isize))
      .marker
      .is_null()
    {
      return 0i32;
    }
    it_tile = it_tile.wrapping_add(1)
  }
  return 1i32;
}
unsafe fn opj_j2k_are_all_used_components_decoded(
  mut p_j2k: *mut opj_j2k_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut compno: OPJ_UINT32 = 0;
  let mut decoded_all_used_components = 1i32;
  if (*p_j2k).m_specific_param.m_decoder.m_numcomps_to_decode != 0 {
    compno = 0 as OPJ_UINT32;
    while compno < (*p_j2k).m_specific_param.m_decoder.m_numcomps_to_decode {
      let mut dec_compno = *(*p_j2k)
        .m_specific_param
        .m_decoder
        .m_comps_indices_to_decode
        .offset(compno as isize);
      if (*(*(*p_j2k).m_output_image).comps.offset(dec_compno as isize))
        .data
        .is_null()
      {
        opj_event_msg(
          p_manager,
          2i32,
          b"Failed to decode component %d\n\x00" as *const u8 as *const libc::c_char,
          dec_compno,
        );
        decoded_all_used_components = 0i32
      }
      compno = compno.wrapping_add(1)
    }
  } else {
    compno = 0 as OPJ_UINT32;
    while compno < (*(*p_j2k).m_output_image).numcomps {
      if (*(*(*p_j2k).m_output_image).comps.offset(compno as isize))
        .data
        .is_null()
      {
        opj_event_msg(
          p_manager,
          2i32,
          b"Failed to decode component %d\n\x00" as *const u8 as *const libc::c_char,
          compno,
        );
        decoded_all_used_components = 0i32
      }
      compno = compno.wrapping_add(1)
    }
  }
  if decoded_all_used_components == 0i32 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Failed to decode all used components\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  return 1i32;
}
/* *
 * Reads the tiles.
 */
unsafe extern "C" fn opj_j2k_decode_tiles(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_go_on = 1i32;
  let mut l_current_tile_no: OPJ_UINT32 = 0;
  let mut l_tile_x0: OPJ_INT32 = 0;
  let mut l_tile_y0: OPJ_INT32 = 0;
  let mut l_tile_x1: OPJ_INT32 = 0;
  let mut l_tile_y1: OPJ_INT32 = 0;
  let mut l_nb_comps: OPJ_UINT32 = 0;
  let mut nr_tiles = 0 as OPJ_UINT32;
  /* Particular case for whole single tile decoding */
  /* We can avoid allocating intermediate tile buffers */
  if (*p_j2k).m_cp.tw == 1u32
    && (*p_j2k).m_cp.th == 1u32
    && (*p_j2k).m_cp.tx0 == 0u32
    && (*p_j2k).m_cp.ty0 == 0u32
    && (*(*p_j2k).m_output_image).x0 == 0u32
    && (*(*p_j2k).m_output_image).y0 == 0u32
    && (*(*p_j2k).m_output_image).x1 == (*p_j2k).m_cp.tdx
    && (*(*p_j2k).m_output_image).y1 == (*p_j2k).m_cp.tdy
  {
    let mut i: OPJ_UINT32 = 0;
    if opj_j2k_read_tile_header(
      p_j2k,
      &mut l_current_tile_no,
      0 as *mut OPJ_UINT32,
      &mut l_tile_x0,
      &mut l_tile_y0,
      &mut l_tile_x1,
      &mut l_tile_y1,
      &mut l_nb_comps,
      &mut l_go_on,
      p_stream,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    if opj_j2k_decode_tile(
      p_j2k,
      l_current_tile_no,
      0 as *mut OPJ_BYTE,
      0 as OPJ_UINT32,
      p_stream,
      p_manager,
    ) == 0
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Failed to decode tile 1/1\n\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    /* Transfer TCD data to output image data */
    i = 0 as OPJ_UINT32;
    while i < (*(*p_j2k).m_output_image).numcomps {
      opj_image_data_free(
        (*(*(*p_j2k).m_output_image).comps.offset(i as isize)).data as *mut libc::c_void,
      );
      let ref mut fresh40 = (*(*(*p_j2k).m_output_image).comps.offset(i as isize)).data;
      *fresh40 = (*(*(*(*(*p_j2k).m_tcd).tcd_image).tiles)
        .comps
        .offset(i as isize))
      .data;
      (*(*(*p_j2k).m_output_image).comps.offset(i as isize)).resno_decoded =
        (*(*(*(*p_j2k).m_tcd).image).comps.offset(i as isize)).resno_decoded;
      let ref mut fresh41 = (*(*(*(*(*p_j2k).m_tcd).tcd_image).tiles)
        .comps
        .offset(i as isize))
      .data;
      *fresh41 = 0 as *mut OPJ_INT32;
      i = i.wrapping_add(1)
    }
    return 1i32;
  }
  loop {
    if (*p_j2k).m_cp.tw == 1u32
      && (*p_j2k).m_cp.th == 1u32
      && !(*(*p_j2k).m_cp.tcps.offset(0))
        .m_data
        .is_null()
    {
      l_current_tile_no = 0 as OPJ_UINT32;
      (*p_j2k).m_current_tile_number = 0 as OPJ_UINT32;
      (*p_j2k).m_specific_param.m_decoder.m_state |= J2KState::DATA
    } else {
      if opj_j2k_read_tile_header(
        p_j2k,
        &mut l_current_tile_no,
        0 as *mut OPJ_UINT32,
        &mut l_tile_x0,
        &mut l_tile_y0,
        &mut l_tile_x1,
        &mut l_tile_y1,
        &mut l_nb_comps,
        &mut l_go_on,
        p_stream,
        p_manager,
      ) == 0
      {
        return 0i32;
      }
      if l_go_on == 0 {
        break;
      }
    }
    if opj_j2k_decode_tile(
      p_j2k,
      l_current_tile_no,
      0 as *mut OPJ_BYTE,
      0 as OPJ_UINT32,
      p_stream,
      p_manager,
    ) == 0
    {
      opj_event_msg(
        p_manager,
        1i32,
        b"Failed to decode tile %d/%d\n\x00" as *const u8 as *const libc::c_char,
        l_current_tile_no.wrapping_add(1u32),
        (*p_j2k).m_cp.th.wrapping_mul((*p_j2k).m_cp.tw),
      );
      return 0i32;
    }
    opj_event_msg(
      p_manager,
      4i32,
      b"Tile %d/%d has been decoded.\n\x00" as *const u8 as *const libc::c_char,
      l_current_tile_no.wrapping_add(1u32),
      (*p_j2k).m_cp.th.wrapping_mul((*p_j2k).m_cp.tw),
    );
    if opj_j2k_update_image_data((*p_j2k).m_tcd, (*p_j2k).m_output_image) == 0 {
      return 0i32;
    }
    if !((*p_j2k).m_cp.tw == 1u32
      && (*p_j2k).m_cp.th == 1u32
      && !((*(*p_j2k).m_output_image).x0 == (*(*p_j2k).m_private_image).x0
        && (*(*p_j2k).m_output_image).y0 == (*(*p_j2k).m_private_image).y0
        && (*(*p_j2k).m_output_image).x1 == (*(*p_j2k).m_private_image).x1
        && (*(*p_j2k).m_output_image).y1 == (*(*p_j2k).m_private_image).y1))
    {
      opj_j2k_tcp_data_destroy(&mut *(*p_j2k).m_cp.tcps.offset(l_current_tile_no as isize));
    }
    opj_event_msg(
      p_manager,
      4i32,
      b"Image data has been updated with tile %d.\n\n\x00" as *const u8 as *const libc::c_char,
      l_current_tile_no.wrapping_add(1u32),
    );
    if opj_stream_get_number_byte_left(p_stream) == 0i64
      && (*p_j2k).m_specific_param.m_decoder.m_state
        == J2KState::NEOC
    {
      break;
    }
    nr_tiles = nr_tiles.wrapping_add(1);
    if nr_tiles == (*p_j2k).m_cp.th.wrapping_mul((*p_j2k).m_cp.tw) {
      break;
    }
  }
  if opj_j2k_are_all_used_components_decoded(p_j2k, p_manager) == 0 {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Sets up the procedures to do on decoding data. Developers wanting to extend the library can add their own reading procedures.
 */
unsafe fn opj_j2k_setup_decoding(
  mut p_j2k: *mut opj_j2k_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions*/

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_decode_tiles
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  /* DEVELOPER CORNER, add your custom procedures */
  return 1i32;
}
/*
 * Read and decode one tile.
 */
unsafe extern "C" fn opj_j2k_decode_one_tile(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_go_on = 1i32;
  let mut l_current_tile_no: OPJ_UINT32 = 0;
  let mut l_tile_no_to_dec: OPJ_UINT32 = 0;
  let mut l_tile_x0: OPJ_INT32 = 0;
  let mut l_tile_y0: OPJ_INT32 = 0;
  let mut l_tile_x1: OPJ_INT32 = 0;
  let mut l_tile_y1: OPJ_INT32 = 0;
  let mut l_nb_comps: OPJ_UINT32 = 0;
  let mut l_nb_tiles: OPJ_UINT32 = 0;
  let mut i: OPJ_UINT32 = 0;
  /*Allocate and initialize some elements of codestrem index if not already done*/
  if (*(*p_j2k).cstr_index).tile_index.is_null() {
    if opj_j2k_allocate_tile_element_cstr_index(p_j2k) == 0 {
      return 0i32;
    }
  }
  /* Move into the codestream to the first SOT used to decode the desired tile */
  l_tile_no_to_dec = (*p_j2k).m_specific_param.m_decoder.m_tile_ind_to_dec as OPJ_UINT32;
  if !(*(*p_j2k).cstr_index).tile_index.is_null() {
    if !(*(*(*p_j2k).cstr_index).tile_index).tp_index.is_null() {
      if (*(*(*p_j2k).cstr_index)
        .tile_index
        .offset(l_tile_no_to_dec as isize))
      .nb_tps
        == 0
      {
        /* the index for this tile has not been built,
         *  so move to the last SOT read */
        if opj_stream_read_seek(
          p_stream,
          (*p_j2k).m_specific_param.m_decoder.m_last_sot_read_pos
            + 2i64,
          p_manager,
        ) == 0
        {
          opj_event_msg(
            p_manager,
            1i32,
            b"Problem with seek function\n\x00" as *const u8 as *const libc::c_char,
          );
          return 0i32;
        }
      } else if opj_stream_read_seek(
        p_stream,
        (*(*(*(*p_j2k).cstr_index)
          .tile_index
          .offset(l_tile_no_to_dec as isize))
        .tp_index
        .offset(0))
        .start_pos
          + 2i64,
        p_manager,
      ) == 0
      {
        opj_event_msg(
          p_manager,
          1i32,
          b"Problem with seek function\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
      }
      /* Special case if we have previously read the EOC marker (if the previous tile getted is the last ) */
      if (*p_j2k).m_specific_param.m_decoder.m_state == J2KState::EOC
      {
        (*p_j2k).m_specific_param.m_decoder.m_state = J2KState::TPHSOT
      }
    }
  }
  /* Reset current tile part number for all tiles, and not only the one */
  /* of interest. */
  /* Not completely sure this is always correct but required for */
  /* ./build/bin/j2k_random_tile_access ./build/tests/tte1.j2k */
  l_nb_tiles = (*p_j2k).m_cp.tw.wrapping_mul((*p_j2k).m_cp.th);
  i = 0 as OPJ_UINT32;
  while i < l_nb_tiles {
    (*(*p_j2k).m_cp.tcps.offset(i as isize)).m_current_tile_part_number = -(1i32);
    i = i.wrapping_add(1)
  }
  loop {
    if opj_j2k_read_tile_header(
      p_j2k,
      &mut l_current_tile_no,
      0 as *mut OPJ_UINT32,
      &mut l_tile_x0,
      &mut l_tile_y0,
      &mut l_tile_x1,
      &mut l_tile_y1,
      &mut l_nb_comps,
      &mut l_go_on,
      p_stream,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    if l_go_on == 0 {
      break;
    }
    if opj_j2k_decode_tile(
      p_j2k,
      l_current_tile_no,
      0 as *mut OPJ_BYTE,
      0 as OPJ_UINT32,
      p_stream,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    opj_event_msg(
      p_manager,
      4i32,
      b"Tile %d/%d has been decoded.\n\x00" as *const u8 as *const libc::c_char,
      l_current_tile_no.wrapping_add(1u32),
      (*p_j2k).m_cp.th.wrapping_mul((*p_j2k).m_cp.tw),
    );
    if opj_j2k_update_image_data((*p_j2k).m_tcd, (*p_j2k).m_output_image) == 0 {
      return 0i32;
    }
    opj_j2k_tcp_data_destroy(&mut *(*p_j2k).m_cp.tcps.offset(l_current_tile_no as isize));
    opj_event_msg(
      p_manager,
      4i32,
      b"Image data has been updated with tile %d.\n\n\x00" as *const u8 as *const libc::c_char,
      l_current_tile_no.wrapping_add(1u32),
    );
    if l_current_tile_no == l_tile_no_to_dec {
      /* move into the codestream to the first SOT (FIXME or not move?)*/
      if opj_stream_read_seek(
        p_stream,
        (*(*p_j2k).cstr_index).main_head_end + 2i64,
        p_manager,
      ) == 0
      {
        opj_event_msg(
          p_manager,
          1i32,
          b"Problem with seek function\n\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
      }
      break;
    } else {
      opj_event_msg(
        p_manager,
        2i32,
        b"Tile read, decoded and updated is not the desired one (%d vs %d).\n\x00" as *const u8
          as *const libc::c_char,
        l_current_tile_no.wrapping_add(1u32),
        l_tile_no_to_dec.wrapping_add(1u32),
      );
    }
  }
  if opj_j2k_are_all_used_components_decoded(p_j2k, p_manager) == 0 {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Sets up the procedures to do on decoding one tile. Developers wanting to extend the library can add their own reading procedures.
 */
unsafe fn opj_j2k_setup_decoding_tile(
  mut p_j2k: *mut opj_j2k_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions*/

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_decode_one_tile
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  /* DEVELOPER CORNER, add your custom procedures */
  return 1i32;
}
unsafe fn opj_j2k_move_data_from_codec_to_output_image(
  mut p_j2k: *mut opj_j2k_t,
  mut p_image: *mut opj_image_t,
) -> OPJ_BOOL {
  let mut compno: OPJ_UINT32 = 0;
  /* Move data and copy one information from codec to output image*/
  if (*p_j2k).m_specific_param.m_decoder.m_numcomps_to_decode > 0u32 {
    let mut newcomps = opj_malloc(
      ((*p_j2k).m_specific_param.m_decoder.m_numcomps_to_decode as libc::c_ulong)
        .wrapping_mul(core::mem::size_of::<opj_image_comp_t>() as libc::c_ulong),
    ) as *mut opj_image_comp_t;
    if newcomps.is_null() {
      opj_image_destroy((*p_j2k).m_private_image);
      (*p_j2k).m_private_image = 0 as *mut opj_image_t;
      return 0i32;
    }
    compno = 0 as OPJ_UINT32;
    while compno < (*p_image).numcomps {
      opj_image_data_free((*(*p_image).comps.offset(compno as isize)).data as *mut libc::c_void);
      let ref mut fresh42 = (*(*p_image).comps.offset(compno as isize)).data;
      *fresh42 = 0 as *mut OPJ_INT32;
      compno = compno.wrapping_add(1)
    }
    compno = 0 as OPJ_UINT32;
    while compno < (*p_j2k).m_specific_param.m_decoder.m_numcomps_to_decode {
      let mut src_compno = *(*p_j2k)
        .m_specific_param
        .m_decoder
        .m_comps_indices_to_decode
        .offset(compno as isize);
      memcpy(
        &mut *newcomps.offset(compno as isize) as *mut opj_image_comp_t as *mut libc::c_void,
        &mut *(*(*p_j2k).m_output_image).comps.offset(src_compno as isize) as *mut opj_image_comp_t
          as *const libc::c_void,
        core::mem::size_of::<opj_image_comp_t>() as libc::c_ulong,
      );
      (*newcomps.offset(compno as isize)).resno_decoded =
        (*(*(*p_j2k).m_output_image).comps.offset(src_compno as isize)).resno_decoded;
      let ref mut fresh43 = (*newcomps.offset(compno as isize)).data;
      *fresh43 = (*(*(*p_j2k).m_output_image).comps.offset(src_compno as isize)).data;
      let ref mut fresh44 = (*(*(*p_j2k).m_output_image).comps.offset(src_compno as isize)).data;
      *fresh44 = 0 as *mut OPJ_INT32;
      compno = compno.wrapping_add(1)
    }
    compno = 0 as OPJ_UINT32;
    while compno < (*p_image).numcomps {
      assert!((*(*(*p_j2k).m_output_image).comps.offset(compno as isize))
        .data
        .is_null());
      opj_image_data_free(
        (*(*(*p_j2k).m_output_image).comps.offset(compno as isize)).data as *mut libc::c_void,
      );
      let ref mut fresh45 = (*(*(*p_j2k).m_output_image).comps.offset(compno as isize)).data;
      *fresh45 = 0 as *mut OPJ_INT32;
      compno = compno.wrapping_add(1)
    }
    (*p_image).numcomps = (*p_j2k).m_specific_param.m_decoder.m_numcomps_to_decode;
    opj_free((*p_image).comps as *mut libc::c_void);
    (*p_image).comps = newcomps
  } else {
    compno = 0 as OPJ_UINT32;
    while compno < (*p_image).numcomps {
      (*(*p_image).comps.offset(compno as isize)).resno_decoded =
        (*(*(*p_j2k).m_output_image).comps.offset(compno as isize)).resno_decoded;
      opj_image_data_free((*(*p_image).comps.offset(compno as isize)).data as *mut libc::c_void);
      let ref mut fresh46 = (*(*p_image).comps.offset(compno as isize)).data;
      *fresh46 = (*(*(*p_j2k).m_output_image).comps.offset(compno as isize)).data;
      let ref mut fresh47 = (*(*(*p_j2k).m_output_image).comps.offset(compno as isize)).data;
      *fresh47 = 0 as *mut OPJ_INT32;
      compno = compno.wrapping_add(1)
    }
  }
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_decode(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_image: *mut opj_image_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  if p_image.is_null() {
    return 0i32;
  }
  /* Heuristics to detect sequence opj_read_header(), opj_set_decoded_resolution_factor() */
  /* and finally opj_decode_image() without manual setting of comps[].factor */
  /* We could potentially always execute it, if we don't allow people to do */
  /* opj_read_header(), modify x0,y0,x1,y1 of returned image an call opj_decode_image() */
  if (*p_j2k).m_cp.m_specific_param.m_dec.m_reduce > 0u32
    && !(*p_j2k).m_private_image.is_null()
    && (*(*p_j2k).m_private_image).numcomps > 0u32
    && (*(*(*p_j2k).m_private_image)
      .comps
      .offset(0))
    .factor
      == (*p_j2k).m_cp.m_specific_param.m_dec.m_reduce
    && (*p_image).numcomps > 0u32
    && (*(*p_image).comps.offset(0)).factor
      == 0u32
    && (*(*p_image).comps.offset(0))
      .data
      .is_null()
  {
    let mut it_comp: OPJ_UINT32 = 0;
    /* Update the comps[].factor member of the output image with the one */
    /* of m_reduce */
    it_comp = 0 as OPJ_UINT32;
    while it_comp < (*p_image).numcomps {
      (*(*p_image).comps.offset(it_comp as isize)).factor =
        (*p_j2k).m_cp.m_specific_param.m_dec.m_reduce;
      it_comp = it_comp.wrapping_add(1)
    }
    if opj_j2k_update_image_dimensions(p_image, p_manager) == 0 {
      return 0i32;
    }
  }
  if (*p_j2k).m_output_image.is_null() {
    (*p_j2k).m_output_image = opj_image_create0();
    if (*p_j2k).m_output_image.is_null() {
      return 0i32;
    }
  }
  opj_copy_image_header(p_image, (*p_j2k).m_output_image);
  /* customization of the decoding */
  if opj_j2k_setup_decoding(p_j2k, p_manager) == 0 {
    return 0i32;
  }
  /* Decode the codestream */
  if opj_j2k_exec(p_j2k, (*p_j2k).m_procedure_list, p_stream, p_manager) == 0 {
    opj_image_destroy((*p_j2k).m_private_image);
    (*p_j2k).m_private_image = 0 as *mut opj_image_t;
    return 0i32;
  }
  /* Move data and copy one information from codec to output image*/
  return opj_j2k_move_data_from_codec_to_output_image(p_j2k, p_image);
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_get_tile(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_image: *mut opj_image_t,
  mut p_manager: *mut opj_event_mgr_t,
  mut tile_index: OPJ_UINT32,
) -> OPJ_BOOL {
  let mut compno: OPJ_UINT32 = 0;
  let mut l_tile_x: OPJ_UINT32 = 0;
  let mut l_tile_y: OPJ_UINT32 = 0;
  let mut l_img_comp = 0 as *mut opj_image_comp_t;
  if p_image.is_null() {
    opj_event_msg(
      p_manager,
      1i32,
      b"We need an image previously created.\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if (*p_image).numcomps < (*(*p_j2k).m_private_image).numcomps {
    opj_event_msg(
      p_manager,
      1i32,
      b"Image has less components than codestream.\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if tile_index >= (*p_j2k).m_cp.tw.wrapping_mul((*p_j2k).m_cp.th) {
    opj_event_msg(
      p_manager,
      1i32,
      b"Tile index provided by the user is incorrect %d (max = %d) \n\x00" as *const u8
        as *const libc::c_char,
      tile_index,
      (*p_j2k)
        .m_cp
        .tw
        .wrapping_mul((*p_j2k).m_cp.th)
        .wrapping_sub(1u32),
    );
    return 0i32;
  }
  /* Compute the dimension of the desired tile*/
  l_tile_x = tile_index.wrapping_rem((*p_j2k).m_cp.tw);
  l_tile_y = tile_index.wrapping_div((*p_j2k).m_cp.tw);
  (*p_image).x0 = l_tile_x
    .wrapping_mul((*p_j2k).m_cp.tdx)
    .wrapping_add((*p_j2k).m_cp.tx0);
  if (*p_image).x0 < (*(*p_j2k).m_private_image).x0 {
    (*p_image).x0 = (*(*p_j2k).m_private_image).x0
  }
  (*p_image).x1 = l_tile_x
    .wrapping_add(1u32)
    .wrapping_mul((*p_j2k).m_cp.tdx)
    .wrapping_add((*p_j2k).m_cp.tx0);
  if (*p_image).x1 > (*(*p_j2k).m_private_image).x1 {
    (*p_image).x1 = (*(*p_j2k).m_private_image).x1
  }
  (*p_image).y0 = l_tile_y
    .wrapping_mul((*p_j2k).m_cp.tdy)
    .wrapping_add((*p_j2k).m_cp.ty0);
  if (*p_image).y0 < (*(*p_j2k).m_private_image).y0 {
    (*p_image).y0 = (*(*p_j2k).m_private_image).y0
  }
  (*p_image).y1 = l_tile_y
    .wrapping_add(1u32)
    .wrapping_mul((*p_j2k).m_cp.tdy)
    .wrapping_add((*p_j2k).m_cp.ty0);
  if (*p_image).y1 > (*(*p_j2k).m_private_image).y1 {
    (*p_image).y1 = (*(*p_j2k).m_private_image).y1
  }
  l_img_comp = (*p_image).comps;
  compno = 0 as OPJ_UINT32;
  while compno < (*(*p_j2k).m_private_image).numcomps {
    let mut l_comp_x1: OPJ_INT32 = 0;
    let mut l_comp_y1: OPJ_INT32 = 0;
    (*l_img_comp).factor = (*(*(*p_j2k).m_private_image).comps.offset(compno as isize)).factor;
    (*l_img_comp).x0 =
      opj_int_ceildiv((*p_image).x0 as OPJ_INT32, (*l_img_comp).dx as OPJ_INT32) as OPJ_UINT32;
    (*l_img_comp).y0 =
      opj_int_ceildiv((*p_image).y0 as OPJ_INT32, (*l_img_comp).dy as OPJ_INT32) as OPJ_UINT32;
    l_comp_x1 = opj_int_ceildiv((*p_image).x1 as OPJ_INT32, (*l_img_comp).dx as OPJ_INT32);
    l_comp_y1 = opj_int_ceildiv((*p_image).y1 as OPJ_INT32, (*l_img_comp).dy as OPJ_INT32);
    (*l_img_comp).w = (opj_int_ceildivpow2(l_comp_x1, (*l_img_comp).factor as OPJ_INT32)
      - opj_int_ceildivpow2(
        (*l_img_comp).x0 as OPJ_INT32,
        (*l_img_comp).factor as OPJ_INT32,
      )) as OPJ_UINT32;
    (*l_img_comp).h = (opj_int_ceildivpow2(l_comp_y1, (*l_img_comp).factor as OPJ_INT32)
      - opj_int_ceildivpow2(
        (*l_img_comp).y0 as OPJ_INT32,
        (*l_img_comp).factor as OPJ_INT32,
      )) as OPJ_UINT32;
    l_img_comp = l_img_comp.offset(1);
    compno = compno.wrapping_add(1)
  }
  if (*p_image).numcomps > (*(*p_j2k).m_private_image).numcomps {
    /* Can happen when calling repeatdly opj_get_decoded_tile() on an
     * image with a color palette, where color palette expansion is done
     * later in jp2.c */
    compno = (*(*p_j2k).m_private_image).numcomps;
    while compno < (*p_image).numcomps {
      opj_image_data_free((*(*p_image).comps.offset(compno as isize)).data as *mut libc::c_void);
      let ref mut fresh48 = (*(*p_image).comps.offset(compno as isize)).data;
      *fresh48 = 0 as *mut OPJ_INT32;
      compno = compno.wrapping_add(1)
    }
    (*p_image).numcomps = (*(*p_j2k).m_private_image).numcomps
  }
  /* Destroy the previous output image*/
  if !(*p_j2k).m_output_image.is_null() {
    opj_image_destroy((*p_j2k).m_output_image);
  }
  /* Create the output image from the information previously computed*/
  (*p_j2k).m_output_image = opj_image_create0();
  if (*p_j2k).m_output_image.is_null() {
    return 0i32;
  }
  opj_copy_image_header(p_image, (*p_j2k).m_output_image);
  (*p_j2k).m_specific_param.m_decoder.m_tile_ind_to_dec = tile_index as OPJ_INT32;
  /* customization of the decoding */
  if opj_j2k_setup_decoding_tile(p_j2k, p_manager) == 0 {
    return 0i32;
  }
  /* Decode the codestream */
  if opj_j2k_exec(p_j2k, (*p_j2k).m_procedure_list, p_stream, p_manager) == 0 {
    opj_image_destroy((*p_j2k).m_private_image);
    (*p_j2k).m_private_image = 0 as *mut opj_image_t;
    return 0i32;
  }
  /* Move data and copy one information from codec to output image*/
  return opj_j2k_move_data_from_codec_to_output_image(p_j2k, p_image);
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_set_decoded_resolution_factor(
  mut p_j2k: *mut opj_j2k_t,
  mut res_factor: OPJ_UINT32,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut it_comp: OPJ_UINT32 = 0;
  (*p_j2k).m_cp.m_specific_param.m_dec.m_reduce = res_factor;
  if !(*p_j2k).m_private_image.is_null() {
    if !(*(*p_j2k).m_private_image).comps.is_null() {
      if !(*p_j2k).m_specific_param.m_decoder.m_default_tcp.is_null() {
        if !(*(*p_j2k).m_specific_param.m_decoder.m_default_tcp)
          .tccps
          .is_null()
        {
          it_comp = 0 as OPJ_UINT32;
          while it_comp < (*(*p_j2k).m_private_image).numcomps {
            let mut max_res = (*(*(*p_j2k).m_specific_param.m_decoder.m_default_tcp)
              .tccps
              .offset(it_comp as isize))
            .numresolutions;
            if res_factor >= max_res {
              opj_event_msg(
                p_manager,
                1i32,
                b"Resolution factor is greater than the maximum resolution in the component.\n\x00"
                  as *const u8 as *const libc::c_char,
              );
              return 0i32;
            }
            (*(*(*p_j2k).m_private_image).comps.offset(it_comp as isize)).factor = res_factor;
            it_comp = it_comp.wrapping_add(1)
          }
          return 1i32;
        }
      }
    }
  }
  return 0i32;
}
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_encoder_set_extra_options(
  mut p_j2k: *mut opj_j2k_t,
  mut p_options: *const *const libc::c_char,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut p_option_iter = 0 as *const *const libc::c_char;
  if p_options.is_null() {
    return 1i32;
  }
  p_option_iter = p_options;
  while !(*p_option_iter).is_null() {
    if strncmp(
      *p_option_iter,
      b"PLT=\x00" as *const u8 as *const libc::c_char,
      4u64,
    ) == 0i32
    {
      if strcmp(
        *p_option_iter,
        b"PLT=YES\x00" as *const u8 as *const libc::c_char,
      ) == 0i32
      {
        (*p_j2k).m_specific_param.m_encoder.m_PLT = 1i32
      } else if strcmp(
        *p_option_iter,
        b"PLT=NO\x00" as *const u8 as *const libc::c_char,
      ) == 0i32
      {
        (*p_j2k).m_specific_param.m_encoder.m_PLT = 0i32
      } else {
        opj_event_msg(
          p_manager,
          1i32,
          b"Invalid value for option: %s.\n\x00" as *const u8 as *const libc::c_char,
          *p_option_iter,
        );
        return 0i32;
      }
    } else if strncmp(
      *p_option_iter,
      b"TLM=\x00" as *const u8 as *const libc::c_char,
      4u64,
    ) == 0i32
    {
      if strcmp(
        *p_option_iter,
        b"TLM=YES\x00" as *const u8 as *const libc::c_char,
      ) == 0i32
      {
        (*p_j2k).m_specific_param.m_encoder.m_TLM = 1i32
      } else if strcmp(
        *p_option_iter,
        b"TLM=NO\x00" as *const u8 as *const libc::c_char,
      ) == 0i32
      {
        (*p_j2k).m_specific_param.m_encoder.m_TLM = 0i32
      } else {
        opj_event_msg(
          p_manager,
          1i32,
          b"Invalid value for option: %s.\n\x00" as *const u8 as *const libc::c_char,
          *p_option_iter,
        );
        return 0i32;
      }
    } else if strncmp(
      *p_option_iter,
      b"GUARD_BITS=\x00" as *const u8 as *const libc::c_char,
      strlen(b"GUARD_BITS=\x00" as *const u8 as *const libc::c_char),
    ) == 0i32
    {
      let mut tileno: OPJ_UINT32 = 0;
      let mut cp: *mut opj_cp_t = 0 as *mut opj_cp_t;
      cp = &mut (*p_j2k).m_cp;
      cp = cp;
      let mut numgbits = atoi(
        (*p_option_iter)
          .offset(strlen(b"GUARD_BITS=\x00" as *const u8 as *const libc::c_char) as isize),
      );
      if numgbits < 0i32 || numgbits > 7i32 {
        opj_event_msg(
          p_manager,
          1i32,
          b"Invalid value for option: %s. Should be in [0,7]\n\x00" as *const u8
            as *const libc::c_char,
          *p_option_iter,
        );
        return 0i32;
      }
      tileno = 0 as OPJ_UINT32;
      while tileno < (*cp).tw.wrapping_mul((*cp).th) {
        let mut i: OPJ_UINT32 = 0;
        let mut tcp: *mut opj_tcp_t = &mut *(*cp).tcps.offset(tileno as isize) as *mut opj_tcp_t;
        i = 0 as OPJ_UINT32;
        while i < (*p_j2k).m_specific_param.m_encoder.m_nb_comps {
          let mut tccp: *mut opj_tccp_t = &mut *(*tcp).tccps.offset(i as isize) as *mut opj_tccp_t;
          (*tccp).numgbits = numgbits as OPJ_UINT32;
          i = i.wrapping_add(1)
        }
        tileno = tileno.wrapping_add(1)
      }
    } else {
      opj_event_msg(
        p_manager,
        1i32,
        b"Invalid option: %s.\n\x00" as *const u8 as *const libc::c_char,
        *p_option_iter,
      );
      return 0i32;
    }
    p_option_iter = p_option_iter.offset(1)
  }
  return 1i32;
}
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_encode(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut l_nb_tiles: OPJ_UINT32 = 0;
  let mut l_max_tile_size = 0 as OPJ_SIZE_T;
  let mut l_current_tile_size: OPJ_SIZE_T = 0;
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  let mut l_reuse_data = 0i32;
  let mut p_tcd = 0 as *mut opj_tcd_t;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_stream.is_null());
  assert!(!p_manager.is_null());
  p_tcd = (*p_j2k).m_tcd;
  l_nb_tiles = (*p_j2k).m_cp.th.wrapping_mul((*p_j2k).m_cp.tw);
  if l_nb_tiles == 1u32 {
    l_reuse_data = 1i32
  }
  i = 0 as OPJ_UINT32;
  while i < l_nb_tiles {
    if opj_j2k_pre_write_tile(p_j2k, i, p_stream, p_manager) == 0 {
      if !l_current_data.is_null() {
        opj_free(l_current_data as *mut libc::c_void);
      }
      return 0i32;
    }
    /* if we only have one tile, then simply set tile component data equal to image component data */
    /* otherwise, allocate the data */
    j = 0 as OPJ_UINT32;
    while j < (*(*(*p_j2k).m_tcd).image).numcomps {
      let mut l_tilec = (*(*(*p_tcd).tcd_image).tiles).comps.offset(j as isize);
      if l_reuse_data != 0 {
        let mut l_img_comp = (*(*p_tcd).image).comps.offset(j as isize);
        (*l_tilec).data = (*l_img_comp).data;
        (*l_tilec).ownsData = 0i32
      } else if opj_alloc_tile_component_data(l_tilec) == 0 {
        opj_event_msg(
          p_manager,
          1i32,
          b"Error allocating tile component data.\x00" as *const u8 as *const libc::c_char,
        );
        if !l_current_data.is_null() {
          opj_free(l_current_data as *mut libc::c_void);
        }
        return 0i32;
      }
      j = j.wrapping_add(1)
    }
    l_current_tile_size = opj_tcd_get_encoder_input_buffer_size((*p_j2k).m_tcd);
    if l_reuse_data == 0 {
      if l_current_tile_size > l_max_tile_size {
        let mut l_new_current_data =
          opj_realloc(l_current_data as *mut libc::c_void, l_current_tile_size) as *mut OPJ_BYTE;
        if l_new_current_data.is_null() {
          if !l_current_data.is_null() {
            opj_free(l_current_data as *mut libc::c_void);
          }
          opj_event_msg(
            p_manager,
            1i32,
            b"Not enough memory to encode all tiles\n\x00" as *const u8 as *const libc::c_char,
          );
          return 0i32;
        }
        l_current_data = l_new_current_data;
        l_max_tile_size = l_current_tile_size
      }
      if l_current_data.is_null() {
        /* Should not happen in practice, but will avoid Coverity to */
        /* complain about a null pointer dereference */
        panic!("");
        // C: assert(0);
      }
      /* copy image data (32 bit) to l_current_data as contiguous, all-component, zero offset buffer */
      /* 32 bit components @ 8 bit precision get converted to 8 bit */
      /* 32 bit components @ 16 bit precision get converted to 16 bit */
      opj_j2k_get_tile_data((*p_j2k).m_tcd, l_current_data);
      /* now copy this data into the tile component */
      if opj_tcd_copy_tile_data((*p_j2k).m_tcd, l_current_data, l_current_tile_size) == 0 {
        opj_event_msg(
          p_manager,
          1i32,
          b"Size mismatch between tile data and sent data.\x00" as *const u8 as *const libc::c_char,
        );
        opj_free(l_current_data as *mut libc::c_void);
        return 0i32;
      }
    }
    if opj_j2k_post_write_tile(p_j2k, p_stream, p_manager) == 0 {
      if !l_current_data.is_null() {
        opj_free(l_current_data as *mut libc::c_void);
      }
      return 0i32;
    }
    i = i.wrapping_add(1)
  }
  if !l_current_data.is_null() {
    opj_free(l_current_data as *mut libc::c_void);
  }
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_end_compress(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* customization of the encoding */
  if opj_j2k_setup_end_compress(p_j2k, p_manager) == 0 {
    return 0i32;
  }
  if opj_j2k_exec(p_j2k, (*p_j2k).m_procedure_list, p_stream, p_manager) == 0 {
    return 0i32;
  }
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_start_compress(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_image: *mut opj_image_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_stream.is_null());
  assert!(!p_manager.is_null());
  (*p_j2k).m_private_image = opj_image_create0();
  if (*p_j2k).m_private_image.is_null() {
    opj_event_msg(
      p_manager,
      1i32,
      b"Failed to allocate image header.\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  opj_copy_image_header(p_image, (*p_j2k).m_private_image);
  /* TODO_MSD: Find a better way */
  if !(*p_image).comps.is_null() {
    let mut it_comp: OPJ_UINT32 = 0;
    it_comp = 0 as OPJ_UINT32;
    while it_comp < (*p_image).numcomps {
      if !(*(*p_image).comps.offset(it_comp as isize)).data.is_null() {
        let ref mut fresh49 = (*(*(*p_j2k).m_private_image).comps.offset(it_comp as isize)).data;
        *fresh49 = (*(*p_image).comps.offset(it_comp as isize)).data;
        let ref mut fresh50 = (*(*p_image).comps.offset(it_comp as isize)).data;
        *fresh50 = 0 as *mut OPJ_INT32
      }
      it_comp = it_comp.wrapping_add(1)
    }
  }
  /* customization of the validation */
  if opj_j2k_setup_encoding_validation(p_j2k, p_manager) == 0 {
    return 0i32;
  }
  /* validation of the parameters codec */
  if opj_j2k_exec(p_j2k, (*p_j2k).m_validation_list, p_stream, p_manager) == 0 {
    return 0i32;
  }
  /* customization of the encoding */
  if opj_j2k_setup_header_writing(p_j2k, p_manager) == 0 {
    return 0i32;
  }
  /* write header */
  if opj_j2k_exec(p_j2k, (*p_j2k).m_procedure_list, p_stream, p_manager) == 0 {
    return 0i32;
  }
  return 1i32;
}
unsafe fn opj_j2k_pre_write_tile(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_index: OPJ_UINT32,
  mut _p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  if p_tile_index != (*p_j2k).m_current_tile_number {
    opj_event_msg(
      p_manager,
      1i32,
      b"The given tile index does not match.\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  opj_event_msg(
    p_manager,
    4i32,
    b"tile number %d / %d\n\x00" as *const u8 as *const libc::c_char,
    (*p_j2k)
      .m_current_tile_number
      .wrapping_add(1u32),
    (*p_j2k).m_cp.tw.wrapping_mul((*p_j2k).m_cp.th),
  );
  (*p_j2k)
    .m_specific_param
    .m_encoder
    .m_current_tile_part_number = 0 as OPJ_UINT32;
  (*(*p_j2k).m_tcd).cur_totnum_tp =
    (*(*p_j2k).m_cp.tcps.offset(p_tile_index as isize)).m_nb_tile_parts;
  (*p_j2k)
    .m_specific_param
    .m_encoder
    .m_current_poc_tile_part_number = 0 as OPJ_UINT32;
  /* initialisation before tile encoding  */
  if opj_tcd_init_encode_tile((*p_j2k).m_tcd, (*p_j2k).m_current_tile_number, p_manager) == 0 {
    return 0i32;
  } /* (/8) */
  return 1i32; /* (%8) */
}
unsafe fn opj_get_tile_dimensions(
  mut l_image: *mut opj_image_t,
  mut l_tilec: *mut opj_tcd_tilecomp_t,
  mut l_img_comp: *mut opj_image_comp_t,
  mut l_size_comp: *mut OPJ_UINT32,
  mut l_width: *mut OPJ_UINT32,
  mut l_height: *mut OPJ_UINT32,
  mut l_offset_x: *mut OPJ_UINT32,
  mut l_offset_y: *mut OPJ_UINT32,
  mut l_image_width: *mut OPJ_UINT32,
  mut l_stride: *mut OPJ_UINT32,
  mut l_tile_offset: *mut OPJ_UINT32,
) {
  let mut l_remaining: OPJ_UINT32 = 0;
  *l_size_comp = (*l_img_comp).prec >> 3i32;
  l_remaining = (*l_img_comp).prec & 7u32;
  if l_remaining != 0 {
    *l_size_comp = (*l_size_comp as libc::c_uint).wrapping_add(1u32)
      as OPJ_UINT32
  }
  if *l_size_comp == 3u32 {
    *l_size_comp = 4 as OPJ_UINT32
  }
  *l_width = ((*l_tilec).x1 - (*l_tilec).x0) as OPJ_UINT32;
  *l_height = ((*l_tilec).y1 - (*l_tilec).y0) as OPJ_UINT32;
  *l_offset_x =
    opj_int_ceildiv((*l_image).x0 as OPJ_INT32, (*l_img_comp).dx as OPJ_INT32) as OPJ_UINT32;
  *l_offset_y =
    opj_int_ceildiv((*l_image).y0 as OPJ_INT32, (*l_img_comp).dy as OPJ_INT32) as OPJ_UINT32;
  *l_image_width = opj_int_ceildiv(
    (*l_image).x1 as OPJ_INT32 - (*l_image).x0 as OPJ_INT32,
    (*l_img_comp).dx as OPJ_INT32,
  ) as OPJ_UINT32;
  *l_stride = (*l_image_width).wrapping_sub(*l_width);
  *l_tile_offset = ((*l_tilec).x0 as OPJ_UINT32)
    .wrapping_sub(*l_offset_x)
    .wrapping_add(
      ((*l_tilec).y0 as OPJ_UINT32)
        .wrapping_sub(*l_offset_y)
        .wrapping_mul(*l_image_width),
    );
}
unsafe fn opj_j2k_get_tile_data(mut p_tcd: *mut opj_tcd_t, mut p_data: *mut OPJ_BYTE) {
  let mut i: OPJ_UINT32 = 0;
  let mut j: OPJ_UINT32 = 0;
  let mut k = 0 as OPJ_UINT32;
  i = 0 as OPJ_UINT32;
  while i < (*(*p_tcd).image).numcomps {
    let mut l_image = (*p_tcd).image;
    let mut l_src_ptr = 0 as *mut OPJ_INT32;
    let mut l_tilec = (*(*(*p_tcd).tcd_image).tiles).comps.offset(i as isize);
    let mut l_img_comp = (*l_image).comps.offset(i as isize);
    let mut l_size_comp: OPJ_UINT32 = 0;
    let mut l_width: OPJ_UINT32 = 0;
    let mut l_height: OPJ_UINT32 = 0;
    let mut l_offset_x: OPJ_UINT32 = 0;
    let mut l_offset_y: OPJ_UINT32 = 0;
    let mut l_image_width: OPJ_UINT32 = 0;
    let mut l_stride: OPJ_UINT32 = 0;
    let mut l_tile_offset: OPJ_UINT32 = 0;
    opj_get_tile_dimensions(
      l_image,
      l_tilec,
      l_img_comp,
      &mut l_size_comp,
      &mut l_width,
      &mut l_height,
      &mut l_offset_x,
      &mut l_offset_y,
      &mut l_image_width,
      &mut l_stride,
      &mut l_tile_offset,
    );
    l_src_ptr = (*l_img_comp).data.offset(l_tile_offset as isize);
    match l_size_comp {
      1 => {
        let mut l_dest_ptr = p_data as *mut OPJ_CHAR;
        if (*l_img_comp).sgnd != 0 {
          j = 0 as OPJ_UINT32;
          while j < l_height {
            k = 0 as OPJ_UINT32;
            while k < l_width {
              *l_dest_ptr = *l_src_ptr as OPJ_CHAR;
              l_dest_ptr = l_dest_ptr.offset(1);
              l_src_ptr = l_src_ptr.offset(1);
              k = k.wrapping_add(1)
            }
            l_src_ptr = l_src_ptr.offset(l_stride as isize);
            j = j.wrapping_add(1)
          }
        } else {
          j = 0 as OPJ_UINT32;
          while j < l_height {
            k = 0 as OPJ_UINT32;
            while k < l_width {
              *l_dest_ptr = (*l_src_ptr & 0xffi32) as OPJ_CHAR;
              l_dest_ptr = l_dest_ptr.offset(1);
              l_src_ptr = l_src_ptr.offset(1);
              k = k.wrapping_add(1)
            }
            l_src_ptr = l_src_ptr.offset(l_stride as isize);
            j = j.wrapping_add(1)
          }
        }
        p_data = l_dest_ptr as *mut OPJ_BYTE
      }
      2 => {
        let mut l_dest_ptr_0 = p_data as *mut OPJ_INT16;
        if (*l_img_comp).sgnd != 0 {
          j = 0 as OPJ_UINT32;
          while j < l_height {
            k = 0 as OPJ_UINT32;
            while k < l_width {
              let fresh51 = l_src_ptr;
              l_src_ptr = l_src_ptr.offset(1);
              let fresh52 = l_dest_ptr_0;
              l_dest_ptr_0 = l_dest_ptr_0.offset(1);
              *fresh52 = *fresh51 as OPJ_INT16;
              k = k.wrapping_add(1)
            }
            l_src_ptr = l_src_ptr.offset(l_stride as isize);
            j = j.wrapping_add(1)
          }
        } else {
          j = 0 as OPJ_UINT32;
          while j < l_height {
            k = 0 as OPJ_UINT32;
            while k < l_width {
              let fresh53 = l_src_ptr;
              l_src_ptr = l_src_ptr.offset(1);
              let fresh54 = l_dest_ptr_0;
              l_dest_ptr_0 = l_dest_ptr_0.offset(1);
              *fresh54 = (*fresh53 & 0xffffi32) as OPJ_INT16;
              k = k.wrapping_add(1)
            }
            l_src_ptr = l_src_ptr.offset(l_stride as isize);
            j = j.wrapping_add(1)
          }
        }
        p_data = l_dest_ptr_0 as *mut OPJ_BYTE
      }
      4 => {
        let mut l_dest_ptr_1 = p_data as *mut OPJ_INT32;
        j = 0 as OPJ_UINT32;
        while j < l_height {
          k = 0 as OPJ_UINT32;
          while k < l_width {
            let fresh55 = l_src_ptr;
            l_src_ptr = l_src_ptr.offset(1);
            let fresh56 = l_dest_ptr_1;
            l_dest_ptr_1 = l_dest_ptr_1.offset(1);
            *fresh56 = *fresh55;
            k = k.wrapping_add(1)
          }
          l_src_ptr = l_src_ptr.offset(l_stride as isize);
          j = j.wrapping_add(1)
        }
        p_data = l_dest_ptr_1 as *mut OPJ_BYTE
      }
      _ => {}
    }
    i = i.wrapping_add(1)
  }
}
unsafe fn opj_j2k_post_write_tile(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  let mut l_nb_bytes_written: OPJ_UINT32 = 0;
  let mut l_current_data = 0 as *mut OPJ_BYTE;
  let mut l_tile_size = 0 as OPJ_UINT32;
  let mut l_available_data: OPJ_UINT32 = 0;
  /* preconditions */
  assert!(!(*p_j2k)
    .m_specific_param
    .m_encoder
    .m_encoded_tile_data
    .is_null());
  l_tile_size = (*p_j2k).m_specific_param.m_encoder.m_encoded_tile_size;
  l_available_data = l_tile_size;
  l_current_data = (*p_j2k).m_specific_param.m_encoder.m_encoded_tile_data;
  l_nb_bytes_written = 0 as OPJ_UINT32;
  if opj_j2k_write_first_tile_part(
    p_j2k,
    l_current_data,
    &mut l_nb_bytes_written,
    l_available_data,
    p_stream,
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  l_current_data = l_current_data.offset(l_nb_bytes_written as isize);
  l_available_data =
    (l_available_data as libc::c_uint).wrapping_sub(l_nb_bytes_written) as OPJ_UINT32;
  l_nb_bytes_written = 0 as OPJ_UINT32;
  if opj_j2k_write_all_tile_parts(
    p_j2k,
    l_current_data,
    &mut l_nb_bytes_written,
    l_available_data,
    p_stream,
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  l_available_data =
    (l_available_data as libc::c_uint).wrapping_sub(l_nb_bytes_written) as OPJ_UINT32;
  l_nb_bytes_written = l_tile_size.wrapping_sub(l_available_data);
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_encoded_tile_data,
    l_nb_bytes_written as OPJ_SIZE_T,
    p_manager,
  ) != l_nb_bytes_written as libc::c_ulong
  {
    return 0i32;
  }
  (*p_j2k).m_current_tile_number = (*p_j2k).m_current_tile_number.wrapping_add(1);
  return 1i32;
}
/* *
 * Sets up the validation ,i.e. adds the procedures to launch to make sure the codec parameters
 * are valid. Developers wanting to extend the library can add their own validation procedures.
 */
unsafe fn opj_j2k_setup_end_compress(
  mut p_j2k: *mut opj_j2k_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  /* DEVELOPER CORNER, insert your custom procedures */
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_write_eoc
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if (*p_j2k).m_specific_param.m_encoder.m_TLM != 0 {
    if opj_procedure_list_add_procedure(
      (*p_j2k).m_procedure_list,
      core::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        opj_procedure,
      >(Some(
        opj_j2k_write_updated_tlm
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      )),
      p_manager,
    ) == 0
    {
      return 0i32;
    }
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_write_epc
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_end_encoding
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_destroy_header_memory
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Sets up the validation ,i.e. adds the procedures to launch to make sure the codec parameters
 * are valid. Developers wanting to extend the library can add their own validation procedures.
 */
unsafe fn opj_j2k_setup_encoding_validation(
  mut p_j2k: *mut opj_j2k_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_validation_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_build_encoder
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_validation_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_encoding_validation
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  /* DEVELOPER CORNER, add your custom validation procedure */
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_validation_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_mct_validation
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Sets up the procedures to do on writing header.
 * Developers wanting to extend the library can add their own writing procedures.
 */
unsafe fn opj_j2k_setup_header_writing(
  mut p_j2k: *mut opj_j2k_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_init_info
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_write_soc
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_write_siz
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_write_cod
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_write_qcd
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_write_all_coc
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_write_all_qcc
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if (*p_j2k).m_specific_param.m_encoder.m_TLM != 0 {
    if opj_procedure_list_add_procedure(
      (*p_j2k).m_procedure_list,
      core::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        opj_procedure,
      >(Some(
        opj_j2k_write_tlm
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      )),
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    if (*p_j2k).m_cp.rsiz as libc::c_int == 0x4i32 {
      if opj_procedure_list_add_procedure(
        (*p_j2k).m_procedure_list,
        core::mem::transmute::<
          Option<
            unsafe extern "C" fn(
              _: *mut opj_j2k_t,
              _: *mut opj_stream_private_t,
              _: *mut opj_event_mgr_t,
            ) -> OPJ_BOOL,
          >,
          opj_procedure,
        >(Some(
          opj_j2k_write_poc
            as unsafe extern "C" fn(
              _: *mut opj_j2k_t,
              _: *mut opj_stream_private_t,
              _: *mut opj_event_mgr_t,
            ) -> OPJ_BOOL,
        )),
        p_manager,
      ) == 0
      {
        return 0i32;
      }
    }
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_write_regions
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if !(*p_j2k).m_cp.comment.is_null() {
    if opj_procedure_list_add_procedure(
      (*p_j2k).m_procedure_list,
      core::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        opj_procedure,
      >(Some(
        opj_j2k_write_com
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      )),
      p_manager,
    ) == 0
    {
      return 0i32;
    }
  }
  /* DEVELOPER CORNER, insert your custom procedures */
  if (*p_j2k).m_cp.rsiz as libc::c_int & (0x8000i32 | 0x100i32)
    == 0x8000i32 | 0x100i32
  {
    if opj_procedure_list_add_procedure(
      (*p_j2k).m_procedure_list,
      core::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        opj_procedure,
      >(Some(
        opj_j2k_write_mct_data_group
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      )),
      p_manager,
    ) == 0
    {
      return 0i32;
    }
  }
  /* End of Developer Corner */
  if !(*p_j2k).cstr_index.is_null() {
    if opj_procedure_list_add_procedure(
      (*p_j2k).m_procedure_list,
      core::mem::transmute::<
        Option<
          unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
        >,
        opj_procedure,
      >(Some(
        opj_j2k_get_end_header
          as unsafe extern "C" fn(
            _: *mut opj_j2k_t,
            _: *mut opj_stream_private_t,
            _: *mut opj_event_mgr_t,
          ) -> OPJ_BOOL,
      )),
      p_manager,
    ) == 0
    {
      return 0i32;
    }
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_create_tcd
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  if opj_procedure_list_add_procedure(
    (*p_j2k).m_procedure_list,
    core::mem::transmute::<
      Option<
        unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
      >,
      opj_procedure,
    >(Some(
      opj_j2k_update_rates
        as unsafe extern "C" fn(
          _: *mut opj_j2k_t,
          _: *mut opj_stream_private_t,
          _: *mut opj_event_mgr_t,
        ) -> OPJ_BOOL,
    )),
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  return 1i32;
}
unsafe extern "C" fn opj_j2k_write_first_tile_part(
  mut p_j2k: *mut opj_j2k_t,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_written: *mut OPJ_UINT32,
  mut total_data_size: OPJ_UINT32,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut l_nb_bytes_written = 0 as OPJ_UINT32;
  let mut l_current_nb_bytes_written: OPJ_UINT32 = 0;
  let mut l_begin_data = 0 as *mut OPJ_BYTE;
  let mut l_tcd = 0 as *mut opj_tcd_t;
  let mut l_cp = 0 as *mut opj_cp_t;
  l_tcd = (*p_j2k).m_tcd;
  l_cp = &mut (*p_j2k).m_cp;
  (*l_tcd).cur_pino = 0 as OPJ_UINT32;
  /*Get number of tile parts*/
  (*p_j2k)
    .m_specific_param
    .m_encoder
    .m_current_poc_tile_part_number = 0 as OPJ_UINT32;
  /* INDEX >> */
  /* << INDEX */
  l_current_nb_bytes_written = 0 as OPJ_UINT32;
  l_begin_data = p_data;
  if opj_j2k_write_sot(
    p_j2k,
    p_data,
    total_data_size,
    &mut l_current_nb_bytes_written,
    p_stream,
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  l_nb_bytes_written = (l_nb_bytes_written as libc::c_uint).wrapping_add(l_current_nb_bytes_written)
    as OPJ_UINT32;
  p_data = p_data.offset(l_current_nb_bytes_written as isize);
  total_data_size = (total_data_size as libc::c_uint).wrapping_sub(l_current_nb_bytes_written)
    as OPJ_UINT32;
  if !((*l_cp).rsiz as libc::c_int >= 0x3i32
    && (*l_cp).rsiz as libc::c_int <= 0x6i32)
  {
    if (*(*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize)).POC() != 0 {
      l_current_nb_bytes_written = 0 as OPJ_UINT32;
      opj_j2k_write_poc_in_memory(p_j2k, p_data, &mut l_current_nb_bytes_written, p_manager);
      l_nb_bytes_written = (l_nb_bytes_written as libc::c_uint)
        .wrapping_add(l_current_nb_bytes_written) as OPJ_UINT32
        as OPJ_UINT32;
      p_data = p_data.offset(l_current_nb_bytes_written as isize);
      total_data_size = (total_data_size as libc::c_uint).wrapping_sub(l_current_nb_bytes_written)
        as OPJ_UINT32
    }
  }
  l_current_nb_bytes_written = 0 as OPJ_UINT32;
  if opj_j2k_write_sod(
    p_j2k,
    l_tcd,
    p_data,
    &mut l_current_nb_bytes_written,
    total_data_size,
    p_stream,
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  l_nb_bytes_written = (l_nb_bytes_written as libc::c_uint).wrapping_add(l_current_nb_bytes_written)
    as OPJ_UINT32;
  *p_data_written = l_nb_bytes_written;
  /* Writing Psot in SOT marker */
  opj_write_bytes_LE(
    l_begin_data.offset(6),
    l_nb_bytes_written,
    4 as OPJ_UINT32,
  ); /* PSOT */
  if (*p_j2k).m_specific_param.m_encoder.m_TLM != 0 {
    opj_j2k_update_tlm(p_j2k, l_nb_bytes_written);
  }
  return 1i32;
}
unsafe extern "C" fn opj_j2k_write_all_tile_parts(
  mut p_j2k: *mut opj_j2k_t,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_written: *mut OPJ_UINT32,
  mut total_data_size: OPJ_UINT32,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut tilepartno = 0 as OPJ_UINT32;
  let mut l_nb_bytes_written = 0 as OPJ_UINT32;
  let mut l_current_nb_bytes_written: OPJ_UINT32 = 0;
  let mut l_part_tile_size: OPJ_UINT32 = 0;
  let mut tot_num_tp: OPJ_UINT32 = 0;
  let mut pino: OPJ_UINT32 = 0;
  let mut l_begin_data = 0 as *mut OPJ_BYTE;
  let mut l_tcp = 0 as *mut opj_tcp_t;
  let mut l_tcd = 0 as *mut opj_tcd_t;
  let mut l_cp = 0 as *mut opj_cp_t;
  l_tcd = (*p_j2k).m_tcd;
  l_cp = &mut (*p_j2k).m_cp;
  l_tcp = (*l_cp).tcps.offset((*p_j2k).m_current_tile_number as isize);
  /*Get number of tile parts*/
  tot_num_tp = opj_j2k_get_num_tp(
    l_cp,
    0 as OPJ_UINT32,
    (*p_j2k).m_current_tile_number,
  );
  /* start writing remaining tile parts */
  (*p_j2k)
    .m_specific_param
    .m_encoder
    .m_current_tile_part_number = (*p_j2k)
    .m_specific_param
    .m_encoder
    .m_current_tile_part_number
    .wrapping_add(1);
  tilepartno = 1 as OPJ_UINT32;
  while tilepartno < tot_num_tp {
    (*p_j2k)
      .m_specific_param
      .m_encoder
      .m_current_poc_tile_part_number = tilepartno;
    l_current_nb_bytes_written = 0 as OPJ_UINT32;
    l_part_tile_size = 0 as OPJ_UINT32;
    l_begin_data = p_data;
    if opj_j2k_write_sot(
      p_j2k,
      p_data,
      total_data_size,
      &mut l_current_nb_bytes_written,
      p_stream,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    l_nb_bytes_written = (l_nb_bytes_written as libc::c_uint)
      .wrapping_add(l_current_nb_bytes_written) as OPJ_UINT32
      as OPJ_UINT32;
    p_data = p_data.offset(l_current_nb_bytes_written as isize);
    total_data_size = (total_data_size as libc::c_uint).wrapping_sub(l_current_nb_bytes_written)
      as OPJ_UINT32;
    l_part_tile_size = (l_part_tile_size as libc::c_uint).wrapping_add(l_current_nb_bytes_written)
      as OPJ_UINT32;
    l_current_nb_bytes_written = 0 as OPJ_UINT32;
    if opj_j2k_write_sod(
      p_j2k,
      l_tcd,
      p_data,
      &mut l_current_nb_bytes_written,
      total_data_size,
      p_stream,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    p_data = p_data.offset(l_current_nb_bytes_written as isize);
    l_nb_bytes_written = (l_nb_bytes_written as libc::c_uint)
      .wrapping_add(l_current_nb_bytes_written) as OPJ_UINT32
      as OPJ_UINT32;
    total_data_size = (total_data_size as libc::c_uint).wrapping_sub(l_current_nb_bytes_written)
      as OPJ_UINT32;
    l_part_tile_size = (l_part_tile_size as libc::c_uint).wrapping_add(l_current_nb_bytes_written)
      as OPJ_UINT32;
    /* Writing Psot in SOT marker */
    opj_write_bytes_LE(
      l_begin_data.offset(6),
      l_part_tile_size,
      4 as OPJ_UINT32,
    ); /* PSOT */
    if (*p_j2k).m_specific_param.m_encoder.m_TLM != 0 {
      opj_j2k_update_tlm(p_j2k, l_part_tile_size);
    }
    (*p_j2k)
      .m_specific_param
      .m_encoder
      .m_current_tile_part_number = (*p_j2k)
      .m_specific_param
      .m_encoder
      .m_current_tile_part_number
      .wrapping_add(1);
    tilepartno = tilepartno.wrapping_add(1)
  }
  pino = 1 as OPJ_UINT32;
  while pino <= (*l_tcp).numpocs {
    (*l_tcd).cur_pino = pino;
    /*Get number of tile parts*/
    tot_num_tp = opj_j2k_get_num_tp(l_cp, pino, (*p_j2k).m_current_tile_number);
    tilepartno = 0 as OPJ_UINT32;
    while tilepartno < tot_num_tp {
      (*p_j2k)
        .m_specific_param
        .m_encoder
        .m_current_poc_tile_part_number = tilepartno;
      l_current_nb_bytes_written = 0 as OPJ_UINT32;
      l_part_tile_size = 0 as OPJ_UINT32;
      l_begin_data = p_data;
      if opj_j2k_write_sot(
        p_j2k,
        p_data,
        total_data_size,
        &mut l_current_nb_bytes_written,
        p_stream,
        p_manager,
      ) == 0
      {
        return 0i32;
      }
      l_nb_bytes_written = (l_nb_bytes_written as libc::c_uint)
        .wrapping_add(l_current_nb_bytes_written) as OPJ_UINT32
        as OPJ_UINT32;
      p_data = p_data.offset(l_current_nb_bytes_written as isize);
      total_data_size = (total_data_size as libc::c_uint).wrapping_sub(l_current_nb_bytes_written)
        as OPJ_UINT32;
      l_part_tile_size = (l_part_tile_size as libc::c_uint).wrapping_add(l_current_nb_bytes_written)
        as OPJ_UINT32;
      l_current_nb_bytes_written = 0 as OPJ_UINT32;
      if opj_j2k_write_sod(
        p_j2k,
        l_tcd,
        p_data,
        &mut l_current_nb_bytes_written,
        total_data_size,
        p_stream,
        p_manager,
      ) == 0
      {
        return 0i32;
      }
      l_nb_bytes_written = (l_nb_bytes_written as libc::c_uint)
        .wrapping_add(l_current_nb_bytes_written) as OPJ_UINT32
        as OPJ_UINT32;
      p_data = p_data.offset(l_current_nb_bytes_written as isize);
      total_data_size = (total_data_size as libc::c_uint).wrapping_sub(l_current_nb_bytes_written)
        as OPJ_UINT32;
      l_part_tile_size = (l_part_tile_size as libc::c_uint).wrapping_add(l_current_nb_bytes_written)
        as OPJ_UINT32;
      /* Writing Psot in SOT marker */
      opj_write_bytes_LE(
        l_begin_data.offset(6),
        l_part_tile_size,
        4 as OPJ_UINT32,
      ); /* PSOT */
      if (*p_j2k).m_specific_param.m_encoder.m_TLM != 0 {
        opj_j2k_update_tlm(p_j2k, l_part_tile_size);
      }
      (*p_j2k)
        .m_specific_param
        .m_encoder
        .m_current_tile_part_number = (*p_j2k)
        .m_specific_param
        .m_encoder
        .m_current_tile_part_number
        .wrapping_add(1);
      tilepartno = tilepartno.wrapping_add(1)
    }
    pino = pino.wrapping_add(1)
  }
  *p_data_written = l_nb_bytes_written;
  return 1i32;
}
/* *
 * Writes the updated tlm.
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_write_updated_tlm(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  let mut l_tlm_size: OPJ_UINT32 = 0;
  let mut l_tlm_position: OPJ_OFF_T = 0;
  let mut l_current_position: OPJ_OFF_T = 0;
  let mut size_per_tile_part: OPJ_UINT32 = 0;
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  size_per_tile_part = if (*p_j2k).m_specific_param.m_encoder.m_Ttlmi_is_byte != 0 {
    5i32
  } else {
    6i32
  } as OPJ_UINT32;
  l_tlm_size =
    size_per_tile_part.wrapping_mul((*p_j2k).m_specific_param.m_encoder.m_total_tile_parts);
  l_tlm_position =
    6i64 + (*p_j2k).m_specific_param.m_encoder.m_tlm_start;
  l_current_position = opj_stream_tell(p_stream);
  if opj_stream_seek(p_stream, l_tlm_position, p_manager) == 0 {
    return 0i32;
  }
  if opj_stream_write_data(
    p_stream,
    (*p_j2k).m_specific_param.m_encoder.m_tlm_sot_offsets_buffer,
    l_tlm_size as OPJ_SIZE_T,
    p_manager,
  ) != l_tlm_size as libc::c_ulong
  {
    return 0i32;
  }
  if opj_stream_seek(p_stream, l_current_position, p_manager) == 0 {
    return 0i32;
  }
  return 1i32;
}
/* *
 * Ends the encoding, i.e. frees memory.
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_end_encoding(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  opj_tcd_destroy((*p_j2k).m_tcd);
  (*p_j2k).m_tcd = 0 as *mut opj_tcd;
  if !(*p_j2k)
    .m_specific_param
    .m_encoder
    .m_tlm_sot_offsets_buffer
    .is_null()
  {
    opj_free((*p_j2k).m_specific_param.m_encoder.m_tlm_sot_offsets_buffer as *mut libc::c_void);
    (*p_j2k).m_specific_param.m_encoder.m_tlm_sot_offsets_buffer = 0 as *mut OPJ_BYTE;
    (*p_j2k)
      .m_specific_param
      .m_encoder
      .m_tlm_sot_offsets_current = 0 as *mut OPJ_BYTE
  }
  if !(*p_j2k)
    .m_specific_param
    .m_encoder
    .m_encoded_tile_data
    .is_null()
  {
    opj_free((*p_j2k).m_specific_param.m_encoder.m_encoded_tile_data as *mut libc::c_void);
    (*p_j2k).m_specific_param.m_encoder.m_encoded_tile_data = 0 as *mut OPJ_BYTE
  }
  (*p_j2k).m_specific_param.m_encoder.m_encoded_tile_size = 0 as OPJ_UINT32;
  return 1i32;
}
/* *
 * Destroys the memory associated with the decoding of headers.
 */
/* *
 * Destroys the memory associated with the decoding of headers.
 */
unsafe extern "C" fn opj_j2k_destroy_header_memory(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_stream.is_null());
  assert!(!p_manager.is_null());
  if !(*p_j2k)
    .m_specific_param
    .m_encoder
    .m_header_tile_data
    .is_null()
  {
    opj_free((*p_j2k).m_specific_param.m_encoder.m_header_tile_data as *mut libc::c_void);
    (*p_j2k).m_specific_param.m_encoder.m_header_tile_data = 0 as *mut OPJ_BYTE
  }
  (*p_j2k).m_specific_param.m_encoder.m_header_tile_data_size = 0 as OPJ_UINT32;
  return 1i32;
}
/* *
 * Inits the Info
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_init_info(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  /* TODO mergeV2: check this part which use cstr_info */
  /*
  let mut l_cstr_info = 0 as *mut opj_codestream_info_t;
  l_cstr_info = p_j2k->cstr_info;

  if (l_cstr_info)  {
          OPJ_UINT32 compno;
          l_cstr_info->tile = (opj_tile_info_t *) opj_malloc(p_j2k->m_cp.tw * p_j2k->m_cp.th * sizeof(opj_tile_info_t));

          l_cstr_info->image_w = p_j2k->m_image->x1 - p_j2k->m_image->x0;
          l_cstr_info->image_h = p_j2k->m_image->y1 - p_j2k->m_image->y0;

          l_cstr_info->prog = (&p_j2k->m_cp.tcps[0])->prg;

          l_cstr_info->tw = p_j2k->m_cp.tw;
          l_cstr_info->th = p_j2k->m_cp.th;

          l_cstr_info->tile_x = p_j2k->m_cp.tdx;*/
  /* new version parser */
  /*l_cstr_info->tile_y = p_j2k->m_cp.tdy;*/
  /* new version parser */
  /*l_cstr_info->tile_Ox = p_j2k->m_cp.tx0;*/
  /* new version parser */
  /*l_cstr_info->tile_Oy = p_j2k->m_cp.ty0;*/
  /* new version parser */
  /*l_cstr_info->numcomps = p_j2k->m_image->numcomps;

  l_cstr_info->numlayers = (&p_j2k->m_cp.tcps[0])->numlayers;

  l_cstr_info->numdecompos = (OPJ_INT32*) opj_malloc(p_j2k->m_image->numcomps * sizeof(OPJ_INT32));

  for (compno=0; compno < p_j2k->m_image->numcomps; compno++) {
          l_cstr_info->numdecompos[compno] = (&p_j2k->m_cp.tcps[0])->tccps->numresolutions - 1;
  }

  l_cstr_info->D_max = 0.0;       */
  /* ADD Marcela */
  /*l_cstr_info->main_head_start = opj_stream_tell(p_stream);*/
  /* position of SOC */
  /*l_cstr_info->maxmarknum = 100;
  l_cstr_info->marker = (opj_marker_info_t *) opj_malloc(l_cstr_info->maxmarknum * sizeof(opj_marker_info_t));
  l_cstr_info->marknum = 0;
  }*/
  return opj_j2k_calculate_tp(
    p_j2k,
    &mut (*p_j2k).m_cp,
    &mut (*p_j2k).m_specific_param.m_encoder.m_total_tile_parts,
    (*p_j2k).m_private_image,
    p_manager,
  );
}
/* *
 * Creates a tile-coder encoder.
 *
 * @param       p_stream                        the stream to write data to.
 * @param       p_j2k                           J2K codec.
 * @param       p_manager                   the user event manager.
*/
/* *
 * Creates a tile-coder encoder.
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
unsafe extern "C" fn opj_j2k_create_tcd(
  mut p_j2k: *mut opj_j2k_t,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  /* preconditions */

  assert!(!p_j2k.is_null());
  assert!(!p_manager.is_null());
  assert!(!p_stream.is_null());
  (*p_j2k).m_tcd = opj_tcd_create(0i32);
  if (*p_j2k).m_tcd.is_null() {
    opj_event_msg(
      p_manager,
      1i32,
      b"Not enough memory to create Tile Coder\n\x00" as *const u8 as *const libc::c_char,
    );
    return 0i32;
  }
  if opj_tcd_init(
    (*p_j2k).m_tcd,
    (*p_j2k).m_private_image,
    &mut (*p_j2k).m_cp,
    (*p_j2k).m_tp,
  ) == 0
  {
    opj_tcd_destroy((*p_j2k).m_tcd);
    (*p_j2k).m_tcd = 0 as *mut opj_tcd;
    return 0i32;
  }
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe extern "C" fn opj_j2k_write_tile(
  mut p_j2k: *mut opj_j2k_t,
  mut p_tile_index: OPJ_UINT32,
  mut p_data: *mut OPJ_BYTE,
  mut p_data_size: OPJ_UINT32,
  mut p_stream: *mut opj_stream_private_t,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  if opj_j2k_pre_write_tile(p_j2k, p_tile_index, p_stream, p_manager) == 0 {
    opj_event_msg(
      p_manager,
      1i32,
      b"Error while opj_j2k_pre_write_tile with tile index = %d\n\x00" as *const u8
        as *const libc::c_char,
      p_tile_index,
    );
    return 0i32;
  } else {
    let mut j: OPJ_UINT32 = 0;
    /* Allocate data */
    j = 0 as OPJ_UINT32;
    while j < (*(*(*p_j2k).m_tcd).image).numcomps {
      let mut l_tilec = (*(*(*(*p_j2k).m_tcd).tcd_image).tiles)
        .comps
        .offset(j as isize);
      if opj_alloc_tile_component_data(l_tilec) == 0 {
        opj_event_msg(
          p_manager,
          1i32,
          b"Error allocating tile component data.\x00" as *const u8 as *const libc::c_char,
        );
        return 0i32;
      }
      j = j.wrapping_add(1)
    }
    /* now copy data into the tile component */
    if opj_tcd_copy_tile_data((*p_j2k).m_tcd, p_data, p_data_size as OPJ_SIZE_T) == 0 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Size mismatch between tile data and sent data.\x00" as *const u8 as *const libc::c_char,
      );
      return 0i32;
    }
    if opj_j2k_post_write_tile(p_j2k, p_stream, p_manager) == 0 {
      opj_event_msg(
        p_manager,
        1i32,
        b"Error while opj_j2k_post_write_tile with tile index = %d\n\x00" as *const u8
          as *const libc::c_char,
        p_tile_index,
      );
      return 0i32;
    }
  }
  return 1i32;
}
