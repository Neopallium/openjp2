use super::cio::{opj_read_bytes, opj_write_bytes};
use super::consts::*;
use super::dwt::*;
use super::event::*;
use super::image::*;
use super::invert::*;
use super::math::*;
use super::mct::*;
use super::par;

// Owned codestream index (Phase 5.d). The `#[repr(C)]` mirror in c_api_types.rs
// is built only when a C caller asks for it via opj_get_cstr_index.
#[derive(Clone, Default)]
pub struct CodestreamIndex {
  pub main_head_start: i64,
  pub main_head_end: i64,
  pub codestream_size: u64,
  pub markers: Vec<opj_marker_info>,
  pub tiles: Vec<CodestreamTileIndex>,
}

#[derive(Clone, Default)]
pub struct CodestreamTileIndex {
  pub tileno: u32,
  pub nb_tps: u32,
  pub current_nb_tps: u32,
  pub current_tpsno: u32,
  pub tp_index: Vec<opj_tp_index>,
  pub markers: Vec<opj_marker_info>,
  pub nb_packet: u32,
  pub packet_index: Vec<opj_packet_info>,
}

/// Safe, owned codestream info v2 (the C mirror is built by the capi crate).
#[derive(Clone, Default)]
pub struct CstrInfoV2 {
  pub nbcomps: u32,
  pub tx0: u32,
  pub ty0: u32,
  pub tdx: u32,
  pub tdy: u32,
  pub tw: u32,
  pub th: u32,
  pub default_tile: CstrTccpInfoV2,
}

/// Per-component default-tile coding parameters of the codestream header.
#[derive(Clone, Default)]
pub struct CstrTccpInfoV2 {
  pub csty: u32,
  pub prg: OPJ_PROG_ORDER,
  pub numlayers: u32,
  pub mct: u32,
  pub tccps: Vec<CstrCompInfoV2>,
}

/// Per-component tccp info of the codestream header.
#[derive(Clone)]
pub struct CstrCompInfoV2 {
  pub csty: u32,
  pub numresolutions: u32,
  pub cblkw: u32,
  pub cblkh: u32,
  pub cblksty: u32,
  pub qmfbid: u32,
  pub qntsty: u32,
  pub stepsizes_mant: [u32; OPJ_J2K_MAXBANDS as usize],
  pub stepsizes_expn: [u32; OPJ_J2K_MAXBANDS as usize],
  pub numgbits: u32,
  pub roishift: i32,
  pub prcw: [u32; OPJ_J2K_MAXRLVLS as usize],
  pub prch: [u32; OPJ_J2K_MAXRLVLS as usize],
}

impl Default for CstrCompInfoV2 {
  fn default() -> Self {
    Self {
      csty: 0,
      numresolutions: 0,
      cblkw: 0,
      cblkh: 0,
      cblksty: 0,
      qmfbid: 0,
      qntsty: 0,
      stepsizes_mant: [0; OPJ_J2K_MAXBANDS as usize],
      stepsizes_expn: [0; OPJ_J2K_MAXBANDS as usize],
      numgbits: 0,
      roishift: 0,
      prcw: [0; OPJ_J2K_MAXRLVLS as usize],
      prch: [0; OPJ_J2K_MAXRLVLS as usize],
    }
  }
}

/// Builds the owned codestream info v2 of a j2k codec (header summary).
/// Returns None when the codec has no decoded header yet (e.g. it is an
/// encoder), so the C API can return a null pointer instead of panicking.
pub(crate) fn j2k_get_cstr_info_owned(p_j2k: &J2k) -> Option<CstrInfoV2> {
  let numcomps = p_j2k.m_private_image.as_deref()?.numcomps;
  let mut cstr_info = CstrInfoV2 {
    nbcomps: numcomps,
    tx0: p_j2k.m_cp.tx0,
    ty0: p_j2k.m_cp.ty0,
    tdx: p_j2k.m_cp.tdx,
    tdy: p_j2k.m_cp.tdy,
    tw: p_j2k.m_cp.tw,
    th: p_j2k.m_cp.th,
    ..Default::default()
  };
  let l_default_tile = p_j2k
    .m_specific_param
    .m_decoder
    .m_default_tcp
    .as_deref()?;
  cstr_info.default_tile.csty = l_default_tile.csty;
  cstr_info.default_tile.prg = l_default_tile.prg;
  cstr_info.default_tile.numlayers = l_default_tile.numlayers;
  cstr_info.default_tile.mct = l_default_tile.mct;
  for l_tccp in l_default_tile.tccps.iter() {
    let mut info = CstrCompInfoV2 {
      csty: l_tccp.csty,
      numresolutions: l_tccp.numresolutions,
      cblkw: l_tccp.cblkw,
      cblkh: l_tccp.cblkh,
      cblksty: l_tccp.cblksty,
      qmfbid: l_tccp.qmfbid,
      qntsty: l_tccp.qntsty,
      numgbits: l_tccp.numgbits,
      roishift: l_tccp.roishift,
      ..Default::default()
    };
    /* Note (open-findings OF-27): the C memcpy's `numresolutions` BYTES of
    prch/prcw here (a bug - it copies half the values); the port copies the
    ELEMENTS, so `opj_get_cstr_info` consumers see complete data where the C
    sees truncated data - a deliberate, documented positive divergence. */
    if l_tccp.numresolutions < OPJ_J2K_MAXRLVLS {
      info.prch[..l_tccp.numresolutions as usize]
        .copy_from_slice(&l_tccp.prch[..l_tccp.numresolutions as usize]);
      info.prcw[..l_tccp.numresolutions as usize]
        .copy_from_slice(&l_tccp.prcw[..l_tccp.numresolutions as usize]);
    }
    let numbands: i32 = if l_tccp.qntsty == 1u32 {
      1i32
    } else {
      (l_tccp.numresolutions as i32 * 3i32) - 2i32
    };
    /* Guard the negative case too: an incomplete header (no COD) leaves
    numresolutions at 0, giving numbands = -2 which would otherwise run
    the band loop to usize::MAX. */
    if numbands > 0i32 && numbands < OPJ_J2K_MAXBANDS as i32 {
      for bandno in 0..numbands as usize {
        info.stepsizes_mant[bandno] = l_tccp.stepsizes[bandno].mant as u32;
        info.stepsizes_expn[bandno] = l_tccp.stepsizes[bandno].expn as u32;
      }
    }
    cstr_info.default_tile.tccps.push(info);
  }
  Some(cstr_info)
}

/// jpeg2k-parity accessors (the C mirror exposes the same fields).
impl CstrInfoV2 {
  pub fn tile_info(&self) -> &CstrTccpInfoV2 {
    &self.default_tile
  }

  pub fn default_tile_info(&self) -> &CstrTccpInfoV2 {
    &self.default_tile
  }

  pub fn tx0(&self) -> u32 {
    self.tx0
  }

  pub fn ty0(&self) -> u32 {
    self.ty0
  }

  pub fn tdx(&self) -> u32 {
    self.tdx
  }

  pub fn tdy(&self) -> u32 {
    self.tdy
  }

  pub fn tw(&self) -> u32 {
    self.tw
  }

  pub fn th(&self) -> u32 {
    self.th
  }

  pub fn nbcomps(&self) -> u32 {
    self.nbcomps
  }
}

impl CstrTccpInfoV2 {
  pub fn csty(&self) -> u32 {
    self.csty
  }

  pub fn prg(&self) -> OPJ_PROG_ORDER {
    self.prg
  }

  pub fn numlayers(&self) -> u32 {
    self.numlayers
  }

  pub fn mct(&self) -> u32 {
    self.mct
  }

  pub fn tccp_infos(&self) -> &[CstrCompInfoV2] {
    &self.tccps
  }
}

impl CodestreamIndex {
  pub fn markers(&self) -> &[opj_marker_info] {
    &self.markers
  }

  pub fn tile_indices(&self) -> &[CodestreamTileIndex] {
    &self.tiles
  }
}

impl CodestreamTileIndex {
  pub fn tileno(&self) -> u32 {
    self.tileno
  }

  pub fn nb_tile_parts(&self) -> u32 {
    self.nb_tps
  }

  pub fn tile_part_indexes(&self) -> &[opj_tp_index] {
    &self.tp_index
  }

  pub fn markers(&self) -> &[opj_marker_info] {
    &self.markers
  }

  pub fn packet_infos(&self) -> &[opj_packet_info] {
    &self.packet_index
  }
}
use super::openjpeg::*;
use super::pi::*;
use super::stream::*;
use super::tcd::*;

use bitflags::bitflags;

mod dump;
pub(crate) use dump::*;

bitflags! {
  pub struct J2KState: u32 {
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

impl Default for J2KState {
  fn default() -> Self {
    Self::NONE
  }
}

#[allow(clippy::upper_case_acronyms)]
#[repr(u32)]
#[derive(Debug, Clone, Copy, Default)]
pub enum MCTElementType {
  #[default]
  DOUBLE = 3,
  FLOAT = 2,
  INT32 = 1,
  INT16 = 0,
}

impl MCTElementType {
  pub fn new(num: u32) -> Self {
    match num & 0b_11_u32 {
      0 => Self::INT16,
      1 => Self::INT32,
      2 => Self::FLOAT,
      3 => Self::DOUBLE,
      _ => Self::INT16,
    }
  }

  pub fn size(&self) -> u32 {
    match self {
      Self::INT16 => 2,
      Self::INT32 => 4,
      Self::FLOAT => 4,
      Self::DOUBLE => 8,
    }
  }

  /// Read big-endian MCT array data of this element type into `dest` floats.
  pub fn read_to_float(&self, src: &[u8], dest: &mut [f32]) {
    match self {
      Self::INT16 => {
        for (i, d) in dest.iter_mut().enumerate() {
          *d = u16::from_be_bytes([src[2 * i], src[2 * i + 1]]) as f32;
        }
      }
      Self::INT32 => {
        for (i, d) in dest.iter_mut().enumerate() {
          *d =
            u32::from_be_bytes([src[4 * i], src[4 * i + 1], src[4 * i + 2], src[4 * i + 3]]) as f32;
        }
      }
      Self::FLOAT => {
        for (i, d) in dest.iter_mut().enumerate() {
          *d = f32::from_be_bytes([src[4 * i], src[4 * i + 1], src[4 * i + 2], src[4 * i + 3]]);
        }
      }
      Self::DOUBLE => {
        for (i, d) in dest.iter_mut().enumerate() {
          *d = f64::from_be_bytes([
            src[8 * i],
            src[8 * i + 1],
            src[8 * i + 2],
            src[8 * i + 3],
            src[8 * i + 4],
            src[8 * i + 5],
            src[8 * i + 6],
            src[8 * i + 7],
          ]) as f32;
        }
      }
    }
  }

  /// Read big-endian MCT array data of this element type into `dest` int32s.
  pub fn read_to_int32(&self, src: &[u8], dest: &mut [u32]) {
    match self {
      Self::INT16 => {
        for (i, d) in dest.iter_mut().enumerate() {
          *d = u16::from_be_bytes([src[2 * i], src[2 * i + 1]]) as u32;
        }
      }
      Self::INT32 => {
        for (i, d) in dest.iter_mut().enumerate() {
          *d = u32::from_be_bytes([src[4 * i], src[4 * i + 1], src[4 * i + 2], src[4 * i + 3]]);
        }
      }
      Self::FLOAT => {
        for (i, d) in dest.iter_mut().enumerate() {
          *d = f32::from_be_bytes([src[4 * i], src[4 * i + 1], src[4 * i + 2], src[4 * i + 3]])
            as i32 as u32;
        }
      }
      Self::DOUBLE => {
        for (i, d) in dest.iter_mut().enumerate() {
          *d = f64::from_be_bytes([
            src[8 * i],
            src[8 * i + 1],
            src[8 * i + 2],
            src[8 * i + 3],
            src[8 * i + 4],
            src[8 * i + 5],
            src[8 * i + 6],
            src[8 * i + 7],
          ]) as i32 as u32;
        }
      }
    }
  }

  /// Write big-endian MCT array data of this element type from `src` floats.
  pub fn write_from_float(&self, src: &[f32], dest: &mut [u8]) {
    match self {
      Self::INT16 => {
        for (i, s) in src.iter().enumerate() {
          let v = *s as u32;
          dest[2 * i..2 * i + 2].copy_from_slice(&(v as u16).to_be_bytes());
        }
      }
      Self::INT32 => {
        for (i, s) in src.iter().enumerate() {
          let v = *s as u32;
          dest[4 * i..4 * i + 4].copy_from_slice(&v.to_be_bytes());
        }
      }
      Self::FLOAT => {
        for (i, s) in src.iter().enumerate() {
          dest[4 * i..4 * i + 4].copy_from_slice(&s.to_be_bytes());
        }
      }
      Self::DOUBLE => {
        for (i, s) in src.iter().enumerate() {
          let v = *s as f64;
          dest[8 * i..8 * i + 8].copy_from_slice(&v.to_be_bytes());
        }
      }
    }
  }
}

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

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum ProgressionOrder {
  Unknown = 0,
  CPRL,
  PCRL,
  RLCP,
  LRCP,
  RPCL,
}

impl ProgressionOrder {
  pub fn from_c_enum(enum_prog: OPJ_PROG_ORDER) -> Self {
    match enum_prog {
      OPJ_CPRL => Self::CPRL,
      OPJ_PCRL => Self::PCRL,
      OPJ_RLCP => Self::RLCP,
      OPJ_LRCP => Self::LRCP,
      OPJ_RPCL => Self::RPCL,
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
      Self::RPCL => &[Resolution, Precinct, Component, Layer],
      Self::Unknown => &[],
    }
  }

  pub fn get_order_str(&self) -> &'static str {
    match self {
      Self::CPRL => "CPRL",
      Self::PCRL => "PCRL",
      Self::RLCP => "RLCP",
      Self::LRCP => "LRCP",
      Self::RPCL => "RPCL",
      Self::Unknown => "",
    }
  }

  pub fn get_step(&self, pos: i32) -> ProgressionStep {
    let steps = self.get_order();
    steps
      .get(pos as usize)
      .cloned()
      .unwrap_or(ProgressionStep::Unknown)
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum J2KMarker {
  /// UNKNOWN marker value
  UNK(u32),
  /// SOC marker value
  SOC,
  /// SOT marker value
  SOT,
  /// SOD marker value
  SOD,
  /// EOC marker value
  EOC,
  /// CAP marker value
  CAP,
  /// SIZ marker value
  SIZ,
  /// COD marker value
  COD,
  /// COC marker value
  COC,
  /// CPF marker value
  CPF,
  /// RGN marker value
  RGN,
  /// QCD marker value
  QCD,
  /// QCC marker value
  QCC,
  /// POC marker value
  POC,
  /// TLM marker value
  TLM,
  /// PLM marker value
  PLM,
  /// PLT marker value
  PLT,
  /// PPM marker value
  PPM,
  /// PPT marker value
  PPT,
  /// SOP marker value
  SOP,
  /// EPH marker value
  EPH,
  /// CRG marker value
  CRG,
  /// COM marker value
  COM,
  /// CBD marker value
  CBD,
  /// MCC marker value
  MCC,
  /// MCT marker value
  MCT,
  /// MCO marker value
  MCO,
}

impl From<u32> for J2KMarker {
  fn from(num: u32) -> Self {
    match num {
      0xff4f => Self::SOC,
      0xff90 => Self::SOT,
      0xff93 => Self::SOD,
      0xffd9 => Self::EOC,
      0xff50 => Self::CAP,
      0xff51 => Self::SIZ,
      0xff52 => Self::COD,
      0xff53 => Self::COC,
      0xff59 => Self::CPF,
      0xff5e => Self::RGN,
      0xff5c => Self::QCD,
      0xff5d => Self::QCC,
      0xff5f => Self::POC,
      0xff55 => Self::TLM,
      0xff57 => Self::PLM,
      0xff58 => Self::PLT,
      0xff60 => Self::PPM,
      0xff61 => Self::PPT,
      0xff91 => Self::SOP,
      0xff92 => Self::EPH,
      0xff63 => Self::CRG,
      0xff64 => Self::COM,
      0xff78 => Self::CBD,
      0xff75 => Self::MCC,
      0xff74 => Self::MCT,
      0xff77 => Self::MCO,
      num => Self::UNK(num),
    }
  }
}

impl J2KMarker {
  pub fn from_buffer(buffer: &[u8]) -> Self {
    /* Read 2 bytes as the new marker ID */
    Self::from(opj_read_bytes(buffer, 2_u32))
  }

  pub fn as_u32(&self) -> u32 {
    match self {
      Self::SOC => 0xff4f,
      Self::SOT => 0xff90,
      Self::SOD => 0xff93,
      Self::EOC => 0xffd9,
      Self::CAP => 0xff50,
      Self::SIZ => 0xff51,
      Self::COD => 0xff52,
      Self::COC => 0xff53,
      Self::CPF => 0xff59,
      Self::RGN => 0xff5e,
      Self::QCD => 0xff5c,
      Self::QCC => 0xff5d,
      Self::POC => 0xff5f,
      Self::TLM => 0xff55,
      Self::PLM => 0xff57,
      Self::PLT => 0xff58,
      Self::PPM => 0xff60,
      Self::PPT => 0xff61,
      Self::SOP => 0xff91,
      Self::EPH => 0xff92,
      Self::CRG => 0xff63,
      Self::COM => 0xff64,
      Self::CBD => 0xff78,
      Self::MCC => 0xff75,
      Self::MCT => 0xff74,
      Self::MCO => 0xff77,
      Self::UNK(num) => *num,
    }
  }

  pub fn is_invalid(&self) -> bool {
    self.as_u32() < 0xff00u32
  }

  pub fn is_unknown(&self) -> bool {
    matches!(self, Self::UNK(_))
  }

  pub fn states(&self) -> J2KState {
    match self {
      Self::SOT => J2KState::MH | J2KState::TPHSOT,
      Self::COD => J2KState::MH | J2KState::TPH,
      Self::COC => J2KState::MH | J2KState::TPH,
      Self::RGN => J2KState::MH | J2KState::TPH,
      Self::QCD => J2KState::MH | J2KState::TPH,
      Self::QCC => J2KState::MH | J2KState::TPH,
      Self::POC => J2KState::MH | J2KState::TPH,
      Self::SIZ => J2KState::MHSIZ,
      Self::TLM => J2KState::MH,
      Self::PLM => J2KState::MH,
      Self::PLT => J2KState::TPH,
      Self::PPM => J2KState::MH,
      Self::PPT => J2KState::TPH,
      Self::SOP => J2KState::NONE,
      Self::CRG => J2KState::MH,
      Self::COM => J2KState::MH | J2KState::TPH,
      Self::MCT => J2KState::MH | J2KState::TPH,
      Self::CBD => J2KState::MH,
      Self::CAP => J2KState::MH,
      Self::CPF => J2KState::MH,
      Self::MCC => J2KState::MH | J2KState::TPH,
      Self::MCO => J2KState::MH | J2KState::TPH,
      _ => J2KState::MH | J2KState::TPH,
    }
  }

  /// Whether the marker has a real header-reading handler.  EPH/SOC/SOD/
  /// EOC (and SOP) have none: the C's marker table has no entries for the
  /// former four (its `read_unk` skips them and rescans), and SOP's entry
  /// is a NULL handler that errors at the position check.  The dispatch
  /// fallback in `handler()` is the "in table, NULL handler" case.
  pub fn has_handler(&self) -> bool {
    matches!(
      self,
      Self::SOT
        | Self::COD
        | Self::COC
        | Self::RGN
        | Self::QCD
        | Self::QCC
        | Self::POC
        | Self::SIZ
        | Self::TLM
        | Self::PLM
        | Self::PLT
        | Self::PPM
        | Self::PPT
        | Self::CRG
        | Self::COM
        | Self::MCT
        | Self::CBD
        | Self::CAP
        | Self::CPF
        | Self::MCC
        | Self::MCO
    )
  }

  pub fn handler(&self, p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
    match self {
      Self::SOT => opj_j2k_read_sot(p_j2k, header_data, p_manager),
      Self::COD => opj_j2k_read_cod(p_j2k, header_data, p_manager),
      Self::COC => opj_j2k_read_coc(p_j2k, header_data, p_manager),
      Self::RGN => opj_j2k_read_rgn(p_j2k, header_data, p_manager),
      Self::QCD => opj_j2k_read_qcd(p_j2k, header_data, p_manager),
      Self::QCC => opj_j2k_read_qcc(p_j2k, header_data, p_manager),
      Self::POC => opj_j2k_read_poc(p_j2k, header_data, p_manager),
      Self::SIZ => opj_j2k_read_siz(p_j2k, header_data, p_manager),
      Self::TLM => opj_j2k_read_tlm(p_j2k, header_data, p_manager),
      Self::PLM => opj_j2k_read_plm(p_j2k, header_data, p_manager),
      Self::PLT => opj_j2k_read_plt(p_j2k, header_data, p_manager),
      Self::PPM => opj_j2k_read_ppm(p_j2k, header_data, p_manager),
      Self::PPT => opj_j2k_read_ppt(p_j2k, header_data, p_manager),
      Self::CRG => opj_j2k_read_crg(p_j2k, header_data, p_manager),
      Self::COM => opj_j2k_read_com(p_j2k, header_data, p_manager),
      Self::MCT => opj_j2k_read_mct(p_j2k, header_data, p_manager),
      Self::CBD => opj_j2k_read_cbd(p_j2k, header_data, p_manager),
      Self::CAP => opj_j2k_read_cap(p_j2k, header_data, p_manager),
      Self::CPF => opj_j2k_read_cpf(p_j2k, header_data, p_manager),
      Self::MCC => opj_j2k_read_mcc(p_j2k, header_data, p_manager),
      Self::MCO => opj_j2k_read_mco(p_j2k, header_data, p_manager),
      _ => {
        // No handler for this marker.
        event_msg!(
          &mut *p_manager,
          EVT_ERROR,
          "No handler for unknown marker.\n",
        );
        0i32
      }
    }
  }
}

/* *
 * Updates the Tile Length Marker.
 */
/// Merges a worker's per-tile-part TLM entries (recorded into the clone of
/// the master's TLM buffer) into the master's buffer, in tile order.
fn merge_tile_tlm(p_j2k: &mut J2k, worker_buf: &[u8], worker_cur: usize, base: usize) {
  if worker_cur <= base {
    return;
  }
  let new_entries = &worker_buf[base..worker_cur];
  let enc = &mut p_j2k.m_specific_param.m_encoder;
  let start = enc.m_tlm_sot_offsets_current;
  let buf = &mut enc.m_tlm_sot_offsets_buffer;
  if buf.len() < start + new_entries.len() {
    buf.resize(start + new_entries.len(), 0);
  }
  buf[start..start + new_entries.len()].copy_from_slice(new_entries);
  enc.m_tlm_sot_offsets_current += new_entries.len();
}

fn opj_j2k_update_tlm(mut p_j2k: &mut J2k, mut p_tile_part_size: u32) {
  let l_tile_number = p_j2k.m_current_tile_number;
  let l_is_byte = p_j2k.m_specific_param.m_encoder.m_ttlmi_is_byte != 0;
  let l_enc = &mut p_j2k.m_specific_param.m_encoder;
  let l_ttlmi_size = if l_is_byte { 1 } else { 2 } as u32;
  crate::cio::opj_write_bytes(
    &mut l_enc.m_tlm_sot_offsets_buffer[l_enc.m_tlm_sot_offsets_current..],
    l_tile_number,
    l_ttlmi_size,
  );
  l_enc.m_tlm_sot_offsets_current += l_ttlmi_size as usize;
  /* PSOT */
  crate::cio::opj_write_bytes(
    &mut l_enc.m_tlm_sot_offsets_buffer[l_enc.m_tlm_sot_offsets_current..],
    p_tile_part_size,
    4_u32,
  );
  l_enc.m_tlm_sot_offsets_current += 4;
}

pub(crate) fn opj_j2k_convert_progression_order(prg_order: OPJ_PROG_ORDER) -> ProgressionOrder {
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
fn opj_j2k_check_poc_val(
  p_pocs: &[opj_poc_t],
  tileno: u32,
  p_nb_pocs: u32,
  p_nb_resolutions: u32,
  p_num_comps: u32,
  p_num_layers: u32,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut index: u32 = 0;
  let mut resno: u32 = 0;
  let mut compno: u32 = 0;
  let mut layno: u32 = 0;
  let mut i: u32 = 0;
  let step_c = 1_u32;
  let step_r = p_num_comps.wrapping_mul(step_c);
  let step_l = p_nb_resolutions.wrapping_mul(step_r);
  let mut loss = 0i32;
  assert!(p_nb_pocs > 0u32);
  /* The C's calloc fails gracefully on OOM; a vec! would abort.  Reserve
  first (encoder parameters, but the shape is attacker-influenced through
  the low-level API). */
  let mut packet_array: Vec<u32> = Vec::new();
  let packet_len = (step_l as usize).wrapping_mul(p_num_layers as usize);
  if packet_array.try_reserve_exact(packet_len).is_err() {
    event_msg!(p_manager, EVT_ERROR, "Not enough memory to check POC values\n",);
    return 0i32;
  }
  packet_array.resize(packet_len, 0);
  /* iterate through all the pocs that match our tile of interest. */
  i = 0_u32;
  while i < p_nb_pocs {
    let poc = &p_pocs[i as usize];
    if tileno.wrapping_add(1u32) == poc.tile {
      index = step_r.wrapping_mul(poc.resno0);
      /* take each resolution for each poc */
      resno = poc.resno0;
      while resno < opj_uint_min(poc.resno1, p_nb_resolutions) {
        let mut res_index = index.wrapping_add(poc.compno0.wrapping_mul(step_c));
        /* take each comp of each resolution for each poc */
        compno = poc.compno0;
        while compno < opj_uint_min(poc.compno1, p_num_comps) {
          /* The layer index always starts at zero for every progression. */
          let layno0 = 0_u32;
          let mut comp_index = res_index.wrapping_add(layno0.wrapping_mul(step_l));
          /* and finally take each layer of each res of ... */
          layno = layno0;
          while layno < opj_uint_min(poc.layno1, p_num_layers) {
            packet_array[comp_index as usize] = 1_u32;
            comp_index = comp_index.wrapping_add(step_l);
            layno += 1;
          }
          res_index = res_index.wrapping_add(step_c);
          compno += 1;
        }
        index = index.wrapping_add(step_r);
        resno += 1;
      }
    }
    i += 1;
  }
  index = 0_u32;
  layno = 0_u32;
  while layno < p_num_layers {
    resno = 0_u32;
    while resno < p_nb_resolutions {
      compno = 0_u32;
      while compno < p_num_comps {
        loss |= (packet_array[index as usize] != 1u32) as core::ffi::c_int;
        index = index.wrapping_add(step_c);
        compno += 1;
      }
      resno += 1;
    }
    layno += 1;
  }
  if loss != 0 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Missing packets possible loss of data\n",
    );
  }
  (loss == 0) as core::ffi::c_int
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
fn opj_j2k_get_num_tp(cp: &mut Cp, pino: u32, tileno: u32) -> u32 {
  let mut tpnum = 1_u32;
  /*  preconditions */

  assert!(tileno < cp.tw.wrapping_mul(cp.th));
  assert!(pino < cp.tcps[tileno as usize].numpocs.wrapping_add(1u32));
  /* get the given tile coding parameter */
  let tcp = &cp.tcps[tileno as usize];
  let l_current_poc = &tcp.pocs[pino as usize];
  /* get the progression order as a character string */
  let prog = opj_j2k_convert_progression_order(tcp.prg);
  assert!(prog != ProgressionOrder::Unknown);
  if cp.m_specific_param.m_enc.m_tp_on {
    // C: for (i = 0; i < 4; ++i) — the index is also written to m_tp_pos on the split step.
    for (i, step) in prog.get_order().iter().enumerate() {
      match step {
        ProgressionStep::Component => {
          /* component wise */
          tpnum = tpnum.wrapping_mul(l_current_poc.compE)
        }
        ProgressionStep::Resolution => {
          /* resolution wise */
          tpnum = tpnum.wrapping_mul(l_current_poc.resE)
        }
        ProgressionStep::Precinct => {
          /* precinct wise */
          tpnum = tpnum.wrapping_mul(l_current_poc.prcE)
        }
        ProgressionStep::Layer => {
          /* layer wise */
          tpnum = tpnum.wrapping_mul(l_current_poc.layE)
        }
        ProgressionStep::Unknown => {}
      }
      /* would we split here ? */
      if cp.m_specific_param.m_enc.m_tp_flag == *step as u8 {
        cp.m_specific_param.m_enc.m_tp_pos = i as i32;
        break;
      }
    }
  } else {
    tpnum = 1_u32
  }
  tpnum
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
fn opj_j2k_calculate_tp(
  cp: &mut Cp,
  p_nb_tiles: &mut u32,
  image: &mut CoreImage,
  _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut pino: u32 = 0;
  let mut tileno: u32 = 0;
  /* preconditions */

  let l_nb_tiles: u32 = cp.tw.wrapping_mul(cp.th);
  *p_nb_tiles = 0_u32;
  tileno = 0_u32;
  while tileno < l_nb_tiles {
    let mut cur_totnum_tp = 0_u32;
    opj_pi_update_encoding_parameters(image, cp, tileno);
    pino = 0_u32;
    while pino <= cp.tcps[tileno as usize].numpocs {
      let tp_num = opj_j2k_get_num_tp(cp, pino, tileno);
      *p_nb_tiles = (*p_nb_tiles).wrapping_add(tp_num);
      cur_totnum_tp = cur_totnum_tp.wrapping_add(tp_num);
      pino += 1;
    }
    cp.tcps[tileno as usize].m_nb_tile_parts = cur_totnum_tp;
    tileno += 1;
  }
  1i32
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
fn opj_j2k_write_soc(
  mut p_j2k: &mut J2k,
  mut p_stream: &mut Stream,
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* 2 bytes will be written */
  let mut l_buf = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_buf.resize(2, 0);
  /* preconditions */

  /* write SOC identifier */
  opj_write_bytes(&mut l_buf[..], J2KMarker::SOC.as_u32(), 2_u32);
  if opj_stream_write_data(p_stream, &l_buf[..2], p_manager) != 2 {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_buf;
  /* UniPG>> */
  /* USE_JPWL */
  /* <<UniPG */
  1i32
}

/* *
 * Reads a SOC marker (Start of Codestream)
 * @param       p_j2k           the jpeg2000 file codec.
 * @param       p_stream        the codestream to read from
 * @param       p_manager       the user event manager.
*/
/* *
 * Reads a SOC marker (Start of Codestream)
 * @param       p_j2k           the jpeg2000 file codec.
 * @param p_stream
 * @param       p_manager       the user event manager.
*/
fn opj_j2k_read_soc(p_j2k: &mut J2k, p_stream: &mut Stream, p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut l_data: [u8; 2] = [0; 2];
  /* preconditions */

  if opj_stream_read_data(p_stream, &mut l_data[..2], p_manager) != 2 {
    return 0i32;
  }
  let l_marker = J2KMarker::from_buffer(&l_data);
  if l_marker != J2KMarker::SOC {
    return 0i32;
  }
  /* Next marker should be a SIZ marker in the main header */
  p_j2k.m_specific_param.m_decoder.m_state = J2KState::MHSIZ;
  /* Record the main-header start position in cstr_index */
  p_j2k
    .cstr_index
    .as_deref_mut()
    .expect("cstr_index")
    .main_head_start = opj_stream_tell(p_stream) - 2i64;
  event_msg!(
    p_manager,
    EVT_INFO,
    "Start to read j2k main header ({}).\n",
    p_j2k
      .cstr_index
      .as_deref()
      .expect("cstr_index")
      .main_head_start,
  );
  /* Add the marker to the codestream index*/
  let l_main_head_start = p_j2k
    .cstr_index
    .as_deref()
    .expect("cstr_index")
    .main_head_start;
  if 0i32
    == opj_j2k_add_mhmarker(
      p_j2k.cstr_index.as_deref_mut().expect("cstr_index"),
      J2KMarker::SOC,
      l_main_head_start,
      2_u32,
    )
  {
    event_msg!(p_manager, EVT_ERROR, "Not enough memory to add mh marker\n",);
    return 0i32;
  }
  1i32
}

/* *
 * Writes the SIZ marker (image and tile size)
 *
 * @param       p_j2k           J2K codec.
 * @param       p_stream        the stream to write data to.
 * @param       p_manager       the user event manager.
*/
fn opj_j2k_write_siz(p_j2k: &mut J2k, p_stream: &mut Stream, p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut i: u32 = 0;
  /* preconditions */

  let l_image = p_j2k.m_private_image.as_deref().expect("m_private_image");
  let cp = &mut p_j2k.m_cp;
  let l_size_len: u32 = (40u32).wrapping_add((3u32).wrapping_mul(l_image.numcomps));
  let mut l_buf = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_buf.resize(l_size_len as usize, 0);
  let mut pos: usize = 0;
  /* write SOC identifier */
  opj_write_bytes(&mut l_buf[pos..], J2KMarker::SIZ.as_u32(), 2_u32); /* SIZ */
  pos += 2usize; /* L_SIZ */
  opj_write_bytes(&mut l_buf[pos..], l_size_len.wrapping_sub(2u32), 2_u32); /* Rsiz (capabilities) */
  pos += 2usize; /* Xsiz */
  opj_write_bytes(&mut l_buf[pos..], cp.rsiz as u32, 2_u32); /* Ysiz */
  pos += 2usize; /* X0siz */
  opj_write_bytes(&mut l_buf[pos..], l_image.x1, 4_u32); /* Y0siz */
  pos += 4usize; /* x_tsiz */
  opj_write_bytes(&mut l_buf[pos..], l_image.y1, 4_u32); /* YTsiz */
  pos += 4usize; /* XT0siz */
  opj_write_bytes(&mut l_buf[pos..], l_image.x0, 4_u32); /* YT0siz */
  pos += 4usize; /* Csiz */
  opj_write_bytes(&mut l_buf[pos..], l_image.y0, 4_u32);
  pos += 4usize;
  opj_write_bytes(&mut l_buf[pos..], cp.tdx, 4_u32);
  pos += 4usize;
  opj_write_bytes(&mut l_buf[pos..], cp.tdy, 4_u32);
  pos += 4usize;
  opj_write_bytes(&mut l_buf[pos..], cp.tx0, 4_u32);
  pos += 4usize;
  opj_write_bytes(&mut l_buf[pos..], cp.ty0, 4_u32);
  pos += 4usize;
  opj_write_bytes(&mut l_buf[pos..], l_image.numcomps, 2_u32);
  pos += 2usize;
  i = 0_u32;
  while i < l_image.numcomps {
    let comp = &l_image.comps[i as usize];
    opj_write_bytes(
      &mut l_buf[pos..],
      comp.prec.wrapping_sub(1u32).wrapping_add(comp.sgnd << 7i32),
      1_u32,
    ); /* Ssiz_i */
    pos += 1usize; /* XRsiz_i */
    opj_write_bytes(&mut l_buf[pos..], comp.dx, 1_u32); /* YRsiz_i */
    pos += 1usize;
    opj_write_bytes(&mut l_buf[pos..], comp.dy, 1_u32);
    pos += 1usize;
    i += 1;
  }
  if opj_stream_write_data(p_stream, &l_buf[..l_size_len as usize], p_manager)
    != l_size_len as usize
  {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_buf;
  1i32
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
fn opj_j2k_read_siz(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  let mut i: u32 = 0;
  let mut l_nb_tiles: u32 = 0;
  let mut l_tmp: u32 = 0;
  let mut l_tx1: u32 = 0;
  let mut l_ty1: u32 = 0;
  let mut l_prec0: u32 = 0;
  let mut l_sgnd0: u32 = 0;
  /* preconditions */

  let l_image = p_j2k
    .m_private_image
    .as_deref_mut()
    .expect("m_private_image");
  let cp = &mut p_j2k.m_cp;
  /* minimum size == 39 - 3 (= minimum component parameter) */
  if p_header_size < 36u32 {
    event_msg!(p_manager, EVT_ERROR, "Error with SIZ marker size\n",); /* Rsiz (capabilities) */
    return 0i32;
  } /* Xsiz */
  let l_remaining_size: u32 = p_header_size.wrapping_sub(36u32);
  let l_nb_comp: u32 = l_remaining_size.wrapping_div(3u32);
  let l_nb_comp_remain: u32 = l_remaining_size.wrapping_rem(3u32);
  if l_nb_comp_remain != 0u32 {
    event_msg!(p_manager, EVT_ERROR, "Error with SIZ marker size\n",); /* x_tsiz */
    return 0i32;
  } /* YTsiz */
  l_tmp = opj_read_bytes(&header_data[cursor..], 2_u32); /* XT0siz */
  cursor += 2usize; /* YT0siz */
  cp.rsiz = l_tmp as u16; /* Csiz */
  l_image.x1 = opj_read_bytes(&header_data[cursor..], 4_u32);
  cursor += 4usize;
  l_image.y1 = opj_read_bytes(&header_data[cursor..], 4_u32);
  cursor += 4usize;
  l_image.x0 = opj_read_bytes(&header_data[cursor..], 4_u32);
  cursor += 4usize;
  l_image.y0 = opj_read_bytes(&header_data[cursor..], 4_u32);
  cursor += 4usize;
  cp.tdx = opj_read_bytes(&header_data[cursor..], 4_u32);
  cursor += 4usize;
  cp.tdy = opj_read_bytes(&header_data[cursor..], 4_u32);
  cursor += 4usize;
  cp.tx0 = opj_read_bytes(&header_data[cursor..], 4_u32);
  cursor += 4usize;
  cp.ty0 = opj_read_bytes(&header_data[cursor..], 4_u32);
  cursor += 4usize;
  l_tmp = opj_read_bytes(&header_data[cursor..], 2_u32);
  cursor += 2usize;
  /* Port-specific hardening: the C accepts Csiz == 0 and then dereferences
  tccps[0] in the mandatory COD/QCD handlers (undefined behaviour); the
  port rejects it here so a crafted codestream cannot panic the decoder. */
  if !(1u32..=16384u32).contains(&l_tmp) {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error with SIZ marker: number of component is illegal -> {}\n",
      l_tmp,
    );
    return 0i32;
  }
  l_image.numcomps = l_tmp;
  if l_image.numcomps != l_nb_comp {
    event_msg!(p_manager, EVT_ERROR,
                    "Error with SIZ marker: number of component is not compatible with the remaining number of parameters ( {} vs {})\n",
                    l_image.numcomps, l_nb_comp);
    return 0i32;
  }
  /* testcase 4035.pdf.SIGSEGV.d8b.3375 */
  /* testcase issue427-null-image-size.jp2 */
  if l_image.x0 >= l_image.x1 || l_image.y0 >= l_image.y1 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error with SIZ marker: negative or zero image size ({} x {})\n",
      l_image.x1 as i64 - l_image.x0 as i64,
      l_image.y1 as i64 - l_image.y0 as i64,
    );
    return 0i32;
  }
  /* testcase 2539.pdf.SIGFPE.706.1712 (also 3622.pdf.SIGFPE.706.2916 and 4008.pdf.SIGFPE.706.3345 and maybe more) */
  if cp.tdx == 0u32 || cp.tdy == 0u32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error with SIZ marker: invalid tile size (tdx: {}, tdy: {})\n",
      cp.tdx,
      cp.tdy,
    );
    return 0i32;
  }
  /* testcase issue427-illegal-tile-offset.jp2 */
  l_tx1 = opj_uint_adds(cp.tx0, cp.tdx); /* manage overflow */
  l_ty1 = opj_uint_adds(cp.ty0, cp.tdy); /* manage overflow */
  if cp.tx0 > l_image.x0 || cp.ty0 > l_image.y0 || l_tx1 <= l_image.x0 || l_ty1 <= l_image.y0 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error with SIZ marker: illegal tile offset\n",
    );
    return 0i32;
  }
  if p_j2k.dump_state == 0 {
    let mut siz_w: u32 = 0;
    let mut siz_h: u32 = 0;
    siz_w = l_image.x1.wrapping_sub(l_image.x0);
    siz_h = l_image.y1.wrapping_sub(l_image.y0);
    if p_j2k.ihdr_w > 0u32
      && p_j2k.ihdr_h > 0u32
      && (p_j2k.ihdr_w != siz_w || p_j2k.ihdr_h != siz_h)
    {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Error with SIZ marker: IHDR w({}) h({}) vs. SIZ w({}) h({})\n",
        p_j2k.ihdr_w,
        p_j2k.ihdr_h,
        siz_w,
        siz_h,
      );
      return 0i32;
    }
  }
  /* USE_JPWL */
  /* Allocate the resulting image components */
  let numcomps = l_image.numcomps;
  if !l_image.alloc_comps(numcomps) {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Not enough memory to take in charge SIZ marker\n",
    );
    return 0i32;
  }
  l_prec0 = 0_u32;
  l_sgnd0 = 0_u32;
  /* Read the component information */
  i = 0_u32; /* Ssiz_i */
  while i < l_image.numcomps {
    let tmp: u32 = opj_read_bytes(&header_data[cursor..], 1_u32);
    cursor += 1usize;
    l_image.comps[i as usize].prec = (tmp & 0x7fu32).wrapping_add(1u32);
    l_image.comps[i as usize].sgnd = tmp >> 7i32;
    if p_j2k.dump_state == 0u32 {
      if i == 0u32 {
        l_prec0 = l_image.comps[i as usize].prec;
        l_sgnd0 = l_image.comps[i as usize].sgnd
      } else if !cp.allow_different_bit_depth_sign
        && (l_image.comps[i as usize].prec != l_prec0 || l_image.comps[i as usize].sgnd != l_sgnd0)
      {
        event_msg!(p_manager, EVT_WARNING,
                            "Despite JP2 BPC!=255, precision and/or sgnd values for comp[{}] is different than comp[0]:\n        [0] prec({}) sgnd({}) [{}] prec({}) sgnd({})\n", i,
                            l_prec0, l_sgnd0, i, l_image.comps[i as usize].prec,
                            l_image.comps[i as usize].sgnd);
      }
      /* Note: could also check against JP2 BPCC values */
    } /* XRsiz_i */
    let tmp_0 = opj_read_bytes(&header_data[cursor..], 1_u32); /* should be between 1 and 255 */
    cursor += 1usize; /* YRsiz_i */
    l_image.comps[i as usize].dx = tmp_0; /* should be between 1 and 255 */
    let tmp_1 = opj_read_bytes(&header_data[cursor..], 1_u32);
    cursor += 1usize;
    l_image.comps[i as usize].dy = tmp_1;
    if l_image.comps[i as usize].dx < 1u32
      || l_image.comps[i as usize].dx > 255u32
      || l_image.comps[i as usize].dy < 1u32
      || l_image.comps[i as usize].dy > 255u32
    {
      event_msg!(p_manager, EVT_ERROR,
                        "Invalid values for comp = {} : dx={} dy={} (should be between 1 and 255 according to the JPEG2000 norm)\n", i,
                        l_image.comps[i as usize].dx, l_image.comps[i as usize].dy);
      return 0i32;
    }
    /* Avoids later undefined shift in computation of */
    /* p_j2k->m_specific_param.m_decoder.m_default_tcp->tccps[i].m_dc_level_shift = 1
    << (l_image->comps[i].prec - 1); */
    if l_image.comps[i as usize].prec > 31u32 {
      event_msg!(p_manager, EVT_ERROR,
                        "Invalid values for comp = {} : prec={} (should be between 1 and 38 according to the JPEG2000 norm. OpenJpeg only supports up to 31)\n", i,
                        l_image.comps[i as usize].prec);
      return 0i32;
    }
    /* USE_JPWL */
    l_image.comps[i as usize].resno_decoded = 0_u32; /* number of resolution decoded */
    l_image.comps[i as usize].factor = cp.m_specific_param.m_dec.m_reduce; /* reducing factor per component */
    i += 1;
  }
  if cp.tdx == 0u32 || cp.tdy == 0u32 {
    return 0i32;
  }
  /* Decoder resource limits (Rust core API): check the codestream's declared
  geometry against the configured caps BEFORE anything is allocated from it.
  This is the first defence against malicious headers with huge image/tile
  dimensions. */
  {
    let limits = p_j2k.decode_limits;
    let siz_w = l_image.x1.wrapping_sub(l_image.x0);
    let siz_h = l_image.y1.wrapping_sub(l_image.y0);
    if limits.max_image_width != 0 && siz_w > limits.max_image_width {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "SIZ image width {} exceeds the decoder limit {}
",
        siz_w,
        limits.max_image_width,
      );
      return 0i32;
    }
    if limits.max_image_height != 0 && siz_h > limits.max_image_height {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "SIZ image height {} exceeds the decoder limit {}
",
        siz_h,
        limits.max_image_height,
      );
      return 0i32;
    }
    if limits.max_tile_pixels != 0 {
      let tile_pixels = (cp.tdx as u64).wrapping_mul(cp.tdy as u64);
      if tile_pixels > limits.max_tile_pixels {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "SIZ tile area {} exceeds the decoder limit {}
",
          tile_pixels,
          limits.max_tile_pixels,
        );
        return 0i32;
      }
    }
    /* Decoder resource limit (Rust core API): when the caller sets an
    explicit max_mem_alloc, reject oversized declared geometry up front
    (fail-fast before any decode work).  With the default settings the
    built-in 1 GiB ceiling still governs the ACTUAL allocations (tile data
    in opj_j2k_decode_tile, output image data in opj_j2k_update_image_data),
    but the declared component geometry itself allocates nothing at header
    time - header-only inspection (opj_dump) of huge-dimension files must
    keep working like the C reference. */
    if limits.max_mem_alloc != 0 {
      let max_mem = limits.effective_max_mem();
      let mut total: u64 = 0;
      let mut c: u32 = 0;
      while c < l_image.numcomps {
        let comp = &l_image.comps[c as usize];
        let cw = siz_w.div_ceil(comp.dx);
        let ch = siz_h.div_ceil(comp.dy);
        total = total.saturating_add(
          (cw as u64)
            .wrapping_mul(ch as u64)
            .wrapping_mul(core::mem::size_of::<i32>() as u64),
        );
        c += 1;
      }
      if total > max_mem {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "SIZ image component data {} bytes exceeds the decoder memory limit {} bytes\n",
          total,
          max_mem,
        );
        return 0i32;
      }
    }
  }

  /* Compute the number of tiles */
  cp.tw = opj_uint_ceildiv(l_image.x1 - cp.tx0, cp.tdx);
  cp.th = opj_uint_ceildiv(l_image.y1 - cp.ty0, cp.tdy);
  /* Check that the number of tiles is valid */
  if cp.tw == 0u32 || cp.th == 0u32 || cp.tw > (65535u32).wrapping_div(cp.th) {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid number of tiles : {} x {} (maximum fixed by jpeg2000 norm is 65535 tiles)\n",
      cp.tw,
      cp.th,
    );
    return 0i32;
  }
  l_nb_tiles = cp.tw.wrapping_mul(cp.th);
  /* Decoder resource limit (Rust core API): the per-tile coding-parameter
  arrays (numcomps x Tccp per tile, plus the per-tile Tcp) are allocated
  below from the codestream geometry alone.  The C's per-tile calloc fails
  gracefully when it exhausts memory; a Vec resize would abort the process,
  so size-check the total against the effective memory ceiling first (a
  crafted SIZ can request up to ~1.2 TB here from a ~49 KB input). */
  {
    let limits = p_j2k.decode_limits;
    let tccps_bytes = (l_image.numcomps as u64)
      .saturating_mul(core::mem::size_of::<Tccp>() as u64)
      .saturating_mul(l_nb_tiles as u64);
    let tcps_bytes = (l_nb_tiles as u64).saturating_mul(core::mem::size_of::<Tcp>() as u64);
    if tccps_bytes.saturating_add(tcps_bytes) > limits.effective_max_mem() {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "SIZ coding parameters {} bytes exceeds the decoder memory limit {} bytes\n",
        tccps_bytes.saturating_add(tcps_bytes),
        limits.effective_max_mem(),
      );
      return 0i32;
    }
  }
  /* Define the tiles which will be decoded */
  if p_j2k.m_specific_param.m_decoder.m_discard_tiles {
    p_j2k.m_specific_param.m_decoder.m_start_tile_x = p_j2k
      .m_specific_param
      .m_decoder
      .m_start_tile_x
      .wrapping_sub(cp.tx0)
      .wrapping_div(cp.tdx);
    p_j2k.m_specific_param.m_decoder.m_start_tile_y = p_j2k
      .m_specific_param
      .m_decoder
      .m_start_tile_y
      .wrapping_sub(cp.ty0)
      .wrapping_div(cp.tdy);
    p_j2k.m_specific_param.m_decoder.m_end_tile_x = opj_uint_ceildiv(
      p_j2k.m_specific_param.m_decoder.m_end_tile_x.wrapping_sub(cp.tx0),
      cp.tdx,
    );
    p_j2k.m_specific_param.m_decoder.m_end_tile_y = opj_uint_ceildiv(
      p_j2k.m_specific_param.m_decoder.m_end_tile_y.wrapping_sub(cp.ty0),
      cp.tdy,
    );
  } else {
    p_j2k.m_specific_param.m_decoder.m_start_tile_x = 0_u32;
    p_j2k.m_specific_param.m_decoder.m_start_tile_y = 0_u32;
    p_j2k.m_specific_param.m_decoder.m_end_tile_x = cp.tw;
    p_j2k.m_specific_param.m_decoder.m_end_tile_y = cp.th
  }
  /* USE_JPWL */
  /* memory allocations */
  cp.tcps = (0..l_nb_tiles).map(|_| Default::default()).collect();
  /* USE_JPWL */
  let default_tcp = p_j2k
    .m_specific_param
    .m_decoder
    .m_default_tcp
    .as_deref_mut()
    .expect("decoder m_default_tcp is always set");
  default_tcp
    .tccps
    .resize(l_image.numcomps as usize, Default::default());
  default_tcp.m_mct_records = Vec::with_capacity(10);
  default_tcp.m_mcc_records = Vec::with_capacity(10);
  /* set up default dc level shift */
  i = 0_u32;
  while i < l_image.numcomps {
    if l_image.comps[i as usize].sgnd == 0 {
      (&mut default_tcp.tccps)[i as usize].m_dc_level_shift =
        (1i32) << l_image.comps[i as usize].prec.wrapping_sub(1u32)
    }
    i += 1;
  }
  i = 0_u32;
  while i < l_nb_tiles {
    cp.tcps[i as usize]
      .tccps
      .resize(l_image.numcomps as usize, Default::default());
    i += 1;
  }
  /* Allocate and initialize some elements of the codestream index */
  {
    let nb_of_tiles = cp.tw.wrapping_mul(cp.th);
    if opj_j2k_allocate_tile_element_cstr_index(
      p_j2k.cstr_index.as_deref_mut().expect("cstr_index"),
      nb_of_tiles,
    ) == 0
    {
      return 0i32;
    }
  }
  p_j2k.m_specific_param.m_decoder.m_state = J2KState::MH;
  opj_image_comp_header_update(
    p_j2k
      .m_private_image
      .as_deref_mut()
      .expect("m_private_image"),
    cp,
  );
  1i32
}

/* *
 * Writes the COM marker (comment)
 *
 * @param       p_stream                        the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager       the user event manager.
*/
fn opj_j2k_write_com(p_j2k: &mut J2k, p_stream: &mut Stream, p_manager: &mut EventMgr) -> OPJ_BOOL {
  /* preconditions */
  /* L_COM */

  let comment_bytes = &p_j2k.m_cp.comment[..];
  let l_comment_size: u32 = comment_bytes.len() as u32;
  let l_total_com_size: u32 = l_comment_size.wrapping_add(6u32);
  let mut l_buf = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_buf.resize(l_total_com_size as usize, 0);
  let mut pos: usize = 0;
  opj_write_bytes(&mut l_buf[pos..], J2KMarker::COM.as_u32(), 2_u32);
  pos += 2usize;
  opj_write_bytes(
    &mut l_buf[pos..],
    l_total_com_size.wrapping_sub(2u32),
    2_u32,
  );
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], 1_u32, 2_u32);
  pos += 2usize;
  l_buf[pos..pos + l_comment_size as usize].copy_from_slice(comment_bytes);
  pos += l_comment_size as usize;
  if opj_stream_write_data(p_stream, &l_buf[..l_total_com_size as usize], p_manager)
    != l_total_com_size as usize
  {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_buf;
  1i32
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
fn opj_j2k_read_com(
  mut _p_j2k: &mut J2k,
  mut _header_data: &[u8],
  mut _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */

  1i32
}

/* *
 * Writes the COD marker (Coding style default)
 *
 * @param       p_stream                        the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager       the user event manager.
*/
fn opj_j2k_write_cod(p_j2k: &mut J2k, p_stream: &mut Stream, p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut l_remaining_size: u32 = 0;
  /* preconditions */
  /* L_COD */
  /* SGcod (A) */
  /* SGcod (C) */
  let l_code_size: u32 = (9u32).wrapping_add(opj_j2k_get_spcod_spcoc_size(
    p_j2k,
    p_j2k.m_current_tile_number,
    0_u32,
  ));
  l_remaining_size = l_code_size;
  let l_tcp = &p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize];
  let mut l_buf = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_buf.resize(l_code_size as usize, 0);
  let mut pos: usize = 0;
  opj_write_bytes(&mut l_buf[pos..], J2KMarker::COD.as_u32(), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], l_code_size.wrapping_sub(2u32), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], l_tcp.csty, 1_u32);
  pos += 1usize;
  opj_write_bytes(&mut l_buf[pos..], l_tcp.prg as u32, 1_u32);
  pos += 1usize;
  opj_write_bytes(&mut l_buf[pos..], l_tcp.numlayers, 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], l_tcp.mct, 1_u32);
  pos += 1usize;
  l_remaining_size = l_remaining_size.wrapping_sub(9u32);
  if opj_j2k_write_spcod_spcoc(
    p_j2k,
    p_j2k.m_current_tile_number,
    0_u32,
    &mut l_buf[pos..],
    &mut l_remaining_size,
    p_manager,
  ) == 0
  {
    event_msg!(p_manager, EVT_ERROR, "Error writing COD marker\n",);
    return 0i32;
  }
  if l_remaining_size != 0u32 {
    event_msg!(p_manager, EVT_ERROR, "Error writing COD marker\n",);
    return 0i32;
  }
  if opj_stream_write_data(p_stream, &l_buf[..l_code_size as usize], p_manager)
    != l_code_size as usize
  {
    return 0i32;
  }
  1i32
}

/* *
 * Reads a COD marker (Coding style defaults)
 * @param       p_header_data   the data contained in the COD box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the COD marker.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_read_cod(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  /* loop */
  let mut i: u32 = 0;
  let mut l_tmp: u32 = 0;
  /* preconditions */

  let l_image_numcomps = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  /* If we are in the first tile-part header of the current tile */
  let is_tph = p_j2k.m_specific_param.m_decoder.m_state == J2KState::TPH;
  let l_cp = &mut p_j2k.m_cp;
  let l_tcp = if is_tph {
    &mut l_cp.tcps[p_j2k.m_current_tile_number as usize]
  } else {
    p_j2k
      .m_specific_param
      .m_decoder
      .m_default_tcp
      .as_deref_mut()
      .expect("decoder m_default_tcp is always set")
  };
  l_tcp.cod = true;
  /* Make sure room is sufficient */
  if p_header_size < 5u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading COD marker\n",); /* Scod */
    return 0i32;
  }
  l_tcp.csty = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  /* Make sure we know how to decode this */
  if l_tcp.csty & !((0x1i32 | 0x2i32 | 0x4i32) as u32) != 0u32 {
    event_msg!(p_manager, EVT_ERROR, "Unknown Scod value in COD marker\n",); /* SGcod (A) */
    return 0i32;
  }
  l_tmp = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  l_tcp.prg = l_tmp as OPJ_PROG_ORDER;
  /* Make sure progression order is valid */
  if l_tcp.prg as core::ffi::c_int > OPJ_CPRL as core::ffi::c_int {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Unknown progression order in COD marker\n",
    ); /* SGcod (B) */
    l_tcp.prg = OPJ_PROG_UNKNOWN
  }
  l_tcp.numlayers = opj_read_bytes(&header_data[cursor..], 2_u32);
  cursor += 2usize;
  if l_tcp.numlayers < 1u32 || l_tcp.numlayers > 65535u32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid number of layers in COD marker : {} not in range [1-65535]\n",
      l_tcp.numlayers,
    );
    return 0i32;
  }
  /* If user didn't set a number layer to decode take the max specify in the codestream. */
  if l_cp.m_specific_param.m_dec.m_layer != 0 {
    l_tcp.num_layers_to_decode = l_cp.m_specific_param.m_dec.m_layer
  } else {
    l_tcp.num_layers_to_decode = l_tcp.numlayers
  } /* SGcod (C) */
  l_tcp.mct = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  if l_tcp.mct > 1u32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid multiple component transformation\n",
    );
    return 0i32;
  }
  p_header_size = p_header_size.wrapping_sub(5u32);
  i = 0_u32;
  while i < l_image_numcomps {
    l_tcp.tccps[i as usize].csty = l_tcp.csty & J2K_CP_CSTY_PRT;
    i += 1;
  }
  if opj_j2k_read_spcod_spcoc(
    p_j2k,
    0_u32,
    &header_data[cursor..],
    &mut p_header_size,
    p_manager,
  ) == 0
  {
    event_msg!(p_manager, EVT_ERROR, "Error reading COD marker\n",);
    return 0i32;
  }
  if p_header_size != 0u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading COD marker\n",);
    return 0i32;
  }
  /* Apply the coding style to other components of the current tile or the m_default_tcp*/
  opj_j2k_copy_tile_component_parameters(p_j2k);
  /* Index */
  1i32
}

/* *
 * Writes the COC marker (Coding style component)
 *
 * @param       p_j2k       J2K codec.
 * @param       p_comp_no   the index of the component to output.
 * @param       p_stream    the stream to write data to.
 * @param       p_manager   the user event manager.
*/
fn opj_j2k_write_coc(
  p_j2k: &mut J2k,
  p_comp_no: u32,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_coc_size: u32 = 0;
  let mut l_remaining_size: u32 = 0;
  /* preconditions */

  let l_comp_room = if p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps
    <= 256u32
  {
    1i32
  } else {
    2i32
  } as u32;
  l_coc_size = (5u32)
    .wrapping_add(l_comp_room)
    .wrapping_add(opj_j2k_get_spcod_spcoc_size(
      p_j2k,
      p_j2k.m_current_tile_number,
      p_comp_no,
    ));
  let mut l_coc_data = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_coc_data.resize(l_coc_size as usize, 0);
  opj_j2k_write_coc_in_memory(
    p_j2k,
    p_comp_no,
    &mut l_coc_data[..],
    &mut l_remaining_size,
    p_manager,
  );
  if opj_stream_write_data(p_stream, &l_coc_data[..l_coc_size as usize], p_manager)
    != l_coc_size as usize
  {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_coc_data;
  1i32
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
fn opj_j2k_compare_coc(p_j2k: &mut J2k, p_first_comp_no: u32, p_second_comp_no: u32) -> OPJ_BOOL {
  /* preconditions */
  let csty_eq = {
    let l_tcp = &p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize];
    l_tcp.tccps[p_first_comp_no as usize].csty == l_tcp.tccps[p_second_comp_no as usize].csty
  };
  if !csty_eq {
    return 0i32;
  }
  opj_j2k_compare_spcod_spcoc(
    p_j2k,
    p_j2k.m_current_tile_number,
    p_first_comp_no,
    p_second_comp_no,
  )
}

/* *
 * Writes the COC marker (Coding style component)
 *
 * @param       p_j2k                   J2K codec.
 * @param       p_comp_no               the index of the component to output.
 * @param p_data
 * @param p_data_written
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_write_coc_in_memory(
  p_j2k: &mut J2k,
  p_comp_no: u32,
  data: &mut [u8],
  p_data_written: &mut u32,
  p_manager: &mut EventMgr,
) {
  let mut pos: usize = 0;
  /* preconditions */
  /* L_COC */
  let (l_comp_room, l_csty) = {
    let l_tcp = &p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize];
    let l_image = p_j2k.m_private_image.as_deref().expect("m_private_image");
    let room = if l_image.numcomps <= 256u32 {
      1i32
    } else {
      2i32
    } as u32;
    (room, l_tcp.tccps[p_comp_no as usize].csty)
  };
  let l_coc_size = (5u32)
    .wrapping_add(l_comp_room)
    .wrapping_add(opj_j2k_get_spcod_spcoc_size(
      p_j2k,
      p_j2k.m_current_tile_number,
      p_comp_no,
    ));
  let mut l_remaining_size = l_coc_size;
  opj_write_bytes(&mut data[pos..], J2KMarker::COC.as_u32(), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut data[pos..], l_coc_size.wrapping_sub(2u32), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut data[pos..], p_comp_no, l_comp_room);
  pos += l_comp_room as usize;
  opj_write_bytes(&mut data[pos..], l_csty, 1_u32);
  pos += 1usize;
  l_remaining_size = l_remaining_size.wrapping_sub((5u32).wrapping_add(l_comp_room));
  opj_j2k_write_spcod_spcoc(
    p_j2k,
    p_j2k.m_current_tile_number,
    0_u32,
    &mut data[pos..],
    &mut l_remaining_size,
    p_manager,
  );
  pos += *p_data_written as usize;
  *p_data_written = pos as u32;
}

/* *
 * Gets the maximum size taken by a coc.
 *
 * @param       p_j2k   the jpeg2000 codec to use.
 */
fn opj_j2k_get_max_coc_size(p_j2k: &mut J2k) -> u32 {
  let mut i: u32 = 0;
  let mut j: u32 = 0;
  /* preconditions */
  let l_nb_tiles = p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th);
  let l_nb_comp = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  let mut l_max = 0_u32;
  i = 0_u32;
  while i < l_nb_tiles {
    j = 0_u32;
    while j < l_nb_comp {
      l_max = opj_uint_max(l_max, opj_j2k_get_spcod_spcoc_size(p_j2k, i, j));
      j += 1;
    }
    i += 1;
  }
  (6u32).wrapping_add(l_max)
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
fn opj_j2k_read_coc(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  /* preconditions */

  let l_image_numcomps = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  let l_comp_room: u32 = if l_image_numcomps <= 256u32 {
    1i32
  } else {
    2i32
  } as u32;
  /* make sure room is sufficient*/
  if p_header_size < l_comp_room.wrapping_add(1u32) {
    event_msg!(p_manager, EVT_ERROR, "Error reading COC marker\n",); /* Ccoc */
    return 0i32;
  } /* Scoc */
  p_header_size = p_header_size.wrapping_sub(l_comp_room.wrapping_add(1u32));
  let l_comp_no: u32 = opj_read_bytes(&header_data[cursor..], l_comp_room);
  cursor += l_comp_room as usize;
  if l_comp_no >= l_image_numcomps {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error reading COC marker (bad number of components)\n",
    );
    return 0i32;
  }
  {
    /* If we are in the first tile-part header of the current tile, patch the tile's tcp,
     * otherwise the default tcp. */
    let is_tph = p_j2k.m_specific_param.m_decoder.m_state == J2KState::TPH;
    let l_cp = &mut p_j2k.m_cp;
    let l_tcp = if is_tph {
      &mut l_cp.tcps[p_j2k.m_current_tile_number as usize]
    } else {
      p_j2k
        .m_specific_param
        .m_decoder
        .m_default_tcp
        .as_deref_mut()
        .expect("decoder m_default_tcp is always set")
    };
    l_tcp.tccps[l_comp_no as usize].csty = opj_read_bytes(&header_data[cursor..], 1_u32);
    cursor += 1usize;
  }
  if opj_j2k_read_spcod_spcoc(
    p_j2k,
    l_comp_no,
    &header_data[cursor..],
    &mut p_header_size,
    p_manager,
  ) == 0
  {
    event_msg!(p_manager, EVT_ERROR, "Error reading COC marker\n",);
    return 0i32;
  }
  if p_header_size != 0u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading COC marker\n",);
    return 0i32;
  }
  1i32
}

/* *
 * Writes the QCD marker (quantization default)
 *
 * @param       p_j2k                   J2K codec.
 * @param       p_stream                the stream to write data to.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_write_qcd(
  mut p_j2k: &mut J2k,
  mut p_stream: &mut Stream,
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_qcd_size: u32 = 0;
  let mut l_remaining_size: u32 = 0;
  /* preconditions */
  /* L_QCD */

  l_qcd_size = (4u32).wrapping_add(opj_j2k_get_sqcd_sqcc_size(
    p_j2k,
    p_j2k.m_current_tile_number,
    0_u32,
  ));
  l_remaining_size = l_qcd_size;
  let mut l_buf = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_buf.resize(l_qcd_size as usize, 0);
  let mut pos: usize = 0;
  opj_write_bytes(&mut l_buf[pos..], J2KMarker::QCD.as_u32(), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], l_qcd_size.wrapping_sub(2u32), 2_u32);
  pos += 2usize;
  l_remaining_size = (l_remaining_size as core::ffi::c_uint).wrapping_sub(4u32);
  if opj_j2k_write_sqcd_sqcc(
    p_j2k,
    p_j2k.m_current_tile_number,
    0_u32,
    &mut l_buf[pos..],
    &mut l_remaining_size,
    p_manager,
  ) == 0
  {
    event_msg!(p_manager, EVT_ERROR, "Error writing QCD marker\n",);
    return 0i32;
  }
  if l_remaining_size != 0u32 {
    event_msg!(p_manager, EVT_ERROR, "Error writing QCD marker\n",);
    return 0i32;
  }
  if opj_stream_write_data(p_stream, &l_buf[..l_qcd_size as usize], p_manager)
    != l_qcd_size as usize
  {
    return 0i32;
  }
  1i32
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
fn opj_j2k_read_qcd(
  mut p_j2k: &mut J2k,
  header_data: &[u8],
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  /* preconditions */

  if opj_j2k_read_sqcd_sqcc(
    p_j2k,
    0_u32,
    &header_data[cursor..],
    &mut p_header_size,
    p_manager,
  ) == 0
  {
    event_msg!(p_manager, EVT_ERROR, "Error reading QCD marker\n",);
    return 0i32;
  }
  if p_header_size != 0u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading QCD marker\n",);
    return 0i32;
  }
  /* Apply the quantization parameters to other components of the current tile or the m_default_tcp */
  opj_j2k_copy_tile_quantization_parameters(p_j2k);
  1i32
}

/* *
 * Writes the QCC marker (quantization component)
 *
 * @param       p_comp_no       the index of the component to output.
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_write_qcc(
  p_j2k: &mut J2k,
  p_comp_no: u32,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_qcc_size: u32 = 0;
  let mut l_remaining_size: u32 = 0;
  /* preconditions */

  l_qcc_size = (5u32).wrapping_add(opj_j2k_get_sqcd_sqcc_size(
    p_j2k,
    p_j2k.m_current_tile_number,
    p_comp_no,
  ));
  let numcomps = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  l_qcc_size =
    l_qcc_size.wrapping_add(if numcomps <= 256u32 { 0i32 } else { 1i32 } as core::ffi::c_uint);
  l_remaining_size = l_qcc_size;
  let mut l_qcc_data = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_qcc_data.resize(l_qcc_size as usize, 0);
  opj_j2k_write_qcc_in_memory(
    p_j2k,
    p_comp_no,
    &mut l_qcc_data[..],
    &mut l_remaining_size,
    p_manager,
  );
  if opj_stream_write_data(p_stream, &l_qcc_data[..l_qcc_size as usize], p_manager)
    != l_qcc_size as usize
  {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_qcc_data;
  1i32
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
fn opj_j2k_compare_qcc(
  mut p_j2k: &mut J2k,
  mut p_first_comp_no: u32,
  mut p_second_comp_no: u32,
) -> OPJ_BOOL {
  opj_j2k_compare_sqcd_sqcc(
    p_j2k,
    p_j2k.m_current_tile_number,
    p_first_comp_no,
    p_second_comp_no,
  )
}

/* *
 * Writes the QCC marker (quantization component)
 *
 * @param       p_j2k           J2K codec.
 * @param       p_comp_no       the index of the component to output.
 * @param p_data
 * @param       p_data_written  the stream to write data to.
 * @param       p_manager       the user event manager.
*/
fn opj_j2k_write_qcc_in_memory(
  p_j2k: &mut J2k,
  p_comp_no: u32,
  data: &mut [u8],
  p_data_written: &mut u32,
  p_manager: &mut EventMgr,
) {
  let mut pos: usize = 0;
  let mut l_qcc_size: u32 = 0;
  /* preconditions */
  /* L_QCC */
  l_qcc_size = (6u32).wrapping_add(opj_j2k_get_sqcd_sqcc_size(
    p_j2k,
    p_j2k.m_current_tile_number,
    p_comp_no,
  ));
  let mut l_remaining_size = l_qcc_size;
  opj_write_bytes(&mut data[pos..], J2KMarker::QCC.as_u32(), 2_u32);
  pos += 2usize;
  let numcomps = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  if numcomps <= 256u32 {
    l_qcc_size = l_qcc_size.wrapping_sub(1);
    opj_write_bytes(&mut data[pos..], l_qcc_size.wrapping_sub(2u32), 2_u32);
    pos += 2usize;
    opj_write_bytes(&mut data[pos..], p_comp_no, 1_u32);
    pos += 1usize;
    /* in the case only one byte is sufficient the last byte allocated is useless -> still do -6 for available */
    l_remaining_size = l_remaining_size.wrapping_sub(6u32)
  } else {
    opj_write_bytes(&mut data[pos..], l_qcc_size.wrapping_sub(2u32), 2_u32); /* L_QCC */
    pos += 2usize; /* Cqcc */
    opj_write_bytes(&mut data[pos..], p_comp_no, 2_u32);
    pos += 2usize;
    l_remaining_size = l_remaining_size.wrapping_sub(6u32)
  }
  opj_j2k_write_sqcd_sqcc(
    p_j2k,
    p_j2k.m_current_tile_number,
    p_comp_no,
    &mut data[pos..],
    &mut l_remaining_size,
    p_manager,
  );
  pos += *p_data_written as usize;
  *p_data_written = pos as u32;
}

/* *
 * Gets the maximum size taken by a qcc.
 */
fn opj_j2k_get_max_qcc_size(mut p_j2k: &mut J2k) -> u32 {
  opj_j2k_get_max_coc_size(p_j2k)
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
fn opj_j2k_read_qcc(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  let l_comp_no: u32;
  /* preconditions */

  let l_num_comp = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  if l_num_comp <= 256u32 {
    if p_header_size < 1u32 {
      event_msg!(p_manager, EVT_ERROR, "Error reading QCC marker\n",);
      return 0i32;
    }
    l_comp_no = opj_read_bytes(&header_data[cursor..], 1_u32);
    cursor += 1usize;
    p_header_size = p_header_size.wrapping_sub(1)
  } else {
    if p_header_size < 2u32 {
      event_msg!(p_manager, EVT_ERROR, "Error reading QCC marker\n",);
      return 0i32;
    }
    l_comp_no = opj_read_bytes(&header_data[cursor..], 2_u32);
    cursor += 2usize;
    p_header_size = p_header_size.wrapping_sub(2u32)
  }
  /* USE_JPWL */
  if l_comp_no >= l_num_comp {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid component number: {}, regarding the number of components {}\n",
      l_comp_no,
      l_num_comp,
    );
    return 0i32;
  }
  if opj_j2k_read_sqcd_sqcc(
    p_j2k,
    l_comp_no,
    &header_data[cursor..],
    &mut p_header_size,
    p_manager,
  ) == 0
  {
    event_msg!(p_manager, EVT_ERROR, "Error reading QCC marker\n",);
    return 0i32;
  }
  if p_header_size != 0u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading QCC marker\n",);
    return 0i32;
  }
  1i32
}

fn opj_j2k_write_poc(p_j2k: &mut J2k, p_stream: &mut Stream, p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut l_written_size = 0_u32;
  /* preconditions */

  let l_tcp = &p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize];
  let l_nb_comp: u32 = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  let l_nb_poc: u32 = (1u32).wrapping_add(l_tcp.numpocs);
  let l_poc_room: u32 = if l_nb_comp <= 256u32 { 1_u32 } else { 2_u32 };
  let l_poc_size: u32 = (4u32).wrapping_add(
    (5u32)
      .wrapping_add((2u32).wrapping_mul(l_poc_room))
      .wrapping_mul(l_nb_poc),
  );
  let mut l_poc_data = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_poc_data.resize(l_poc_size as usize, 0);
  opj_j2k_write_poc_in_memory(p_j2k, &mut l_poc_data[..], &mut l_written_size, p_manager);
  if opj_stream_write_data(p_stream, &l_poc_data[..l_poc_size as usize], p_manager)
    != l_poc_size as usize
  {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_poc_data;
  1i32
}

/* *
 * Writes the POC marker (Progression Order Change)
 *
 * @param       p_j2k          J2K codec.
 * @param p_data
 * @param       p_data_written the stream to write data to.
 * @param       p_manager      the user event manager.
 */
fn opj_j2k_write_poc_in_memory(
  p_j2k: &mut J2k,
  data: &mut [u8],
  p_data_written: &mut u32,
  _p_manager: &mut EventMgr,
) {
  let mut pos: usize = 0;
  let mut i: u32 = 0;
  /* preconditions */
  /* Lpoc */
  let l_image = p_j2k.m_private_image.as_deref().expect("m_private_image");
  let l_tcp = &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize]; /* LYEpoc_i */
  let l_tccp = &l_tcp.tccps[0]; /* REpoc_i */
  let l_nb_comp: u32 = l_image.numcomps;
  let l_nb_poc: u32 = (1u32).wrapping_add(l_tcp.numpocs);
  let l_poc_room: u32 = if l_nb_comp <= 256u32 { 1_u32 } else { 2_u32 };
  let l_poc_size: u32 = (4u32).wrapping_add(
    (5u32)
      .wrapping_add((2u32).wrapping_mul(l_poc_room))
      .wrapping_mul(l_nb_poc),
  );
  opj_write_bytes(&mut data[pos..], J2KMarker::POC.as_u32(), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut data[pos..], l_poc_size.wrapping_sub(2u32), 2_u32);
  pos += 2usize;
  i = 0_u32;
  while i < l_nb_poc {
    let l_current_poc = &mut l_tcp.pocs[i as usize];
    opj_write_bytes(&mut data[pos..], l_current_poc.resno0, 1_u32);
    pos += 1usize;
    opj_write_bytes(&mut data[pos..], l_current_poc.compno0, l_poc_room);
    pos += l_poc_room as usize;
    opj_write_bytes(&mut data[pos..], l_current_poc.layno1, 2_u32);
    pos += 2usize;
    opj_write_bytes(&mut data[pos..], l_current_poc.resno1, 1_u32);
    pos += 1usize;
    opj_write_bytes(&mut data[pos..], l_current_poc.compno1, l_poc_room);
    pos += l_poc_room as usize;
    opj_write_bytes(&mut data[pos..], l_current_poc.prg as u32, 1_u32);
    pos += 1usize;
    /* change the value of the max layer according to the actual number of layers in the file, components and resolutions*/
    l_current_poc.layno1 = opj_int_min(l_current_poc.layno1 as i32, l_tcp.numlayers as i32) as u32;
    l_current_poc.resno1 =
      opj_int_min(l_current_poc.resno1 as i32, l_tccp.numresolutions as i32) as u32;
    l_current_poc.compno1 = opj_int_min(l_current_poc.compno1 as i32, l_nb_comp as i32) as u32;
    i += 1;
  }
  *p_data_written = pos as u32;
}

/* *
 * Gets the maximum size taken by the writing of a POC.
 */
fn opj_j2k_get_max_poc_size(p_j2k: &mut J2k) -> u32 {
  let l_nb_tiles = p_j2k.m_cp.th.wrapping_mul(p_j2k.m_cp.tw);
  let mut l_max_poc = 0_u32;
  let mut i: u32 = 0;
  while i < l_nb_tiles {
    l_max_poc = opj_uint_max(l_max_poc, p_j2k.m_cp.tcps[i as usize].numpocs);
    i += 1;
  }
  l_max_poc = l_max_poc.wrapping_add(1);
  (4u32).wrapping_add((9u32).wrapping_mul(l_max_poc))
}
/* *
 * Gets the maximum size taken by the toc headers of all the tile parts of any given tile.
 */
fn opj_j2k_get_max_toc_size(p_j2k: &mut J2k) -> u32 {
  let l_nb_tiles: u32 = p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th);
  let mut l_max = 0_u32;
  let mut i: u32 = 0;
  while i < l_nb_tiles {
    l_max = opj_uint_max(l_max, p_j2k.m_cp.tcps[i as usize].m_nb_tile_parts);
    i += 1;
  }
  (12u32).wrapping_mul(l_max)
}
/* *
 * Gets the maximum size taken by the headers of the SOT.
 *
 * @param       p_j2k   the jpeg2000 codec to use.
 */
fn opj_j2k_get_specific_header_sizes(p_j2k: &mut J2k) -> u32 {
  let mut l_nb_bytes = 0_u32;
  let mut l_coc_bytes: u32 = 0;
  let mut l_qcc_bytes: u32 = 0;
  let l_nb_comps: u32 = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps
    .wrapping_sub(1u32);
  l_nb_bytes = l_nb_bytes.wrapping_add(opj_j2k_get_max_toc_size(p_j2k));
  if !(p_j2k.m_cp.rsiz as core::ffi::c_int >= 0x3i32
    && p_j2k.m_cp.rsiz as core::ffi::c_int <= 0x6i32)
  {
    l_coc_bytes = opj_j2k_get_max_coc_size(p_j2k);
    l_nb_bytes = l_nb_bytes.wrapping_add(l_nb_comps.wrapping_mul(l_coc_bytes));
    l_qcc_bytes = opj_j2k_get_max_qcc_size(p_j2k);
    l_nb_bytes = l_nb_bytes.wrapping_add(l_nb_comps.wrapping_mul(l_qcc_bytes))
  }
  l_nb_bytes = l_nb_bytes.wrapping_add(opj_j2k_get_max_poc_size(p_j2k));
  if p_j2k.m_specific_param.m_encoder.m_plt != 0 {
    /* Reserve space for PLT markers */
    let l_cp: &Cp = &p_j2k.m_cp;
    let mut l_max_packet_count = 0_u32;
    let mut i: u32 = 0;
    while i < l_cp.th.wrapping_mul(l_cp.tw) {
      l_max_packet_count = opj_uint_max(
        l_max_packet_count,
        opj_get_encoding_packet_count(
          p_j2k
            .m_private_image
            .as_deref_mut()
            .expect("m_private_image"),
          l_cp,
          i,
        ),
      );
      i += 1;
    }
    /* Minimum 6 bytes per PLT marker, and at a minimum (taking a pessimistic */
    /* estimate of 4 bytes for a packet size), one can write */
    /* (65536-6) / 4 = 16382 paquet sizes per PLT marker */
    p_j2k.m_specific_param.m_encoder.m_reserved_bytes_for_plt =
      (6u32).wrapping_mul(opj_uint_ceildiv(l_max_packet_count, 16382_u32));
    /* Maximum 5 bytes per packet to encode a full UINT32 */
    l_nb_bytes = l_nb_bytes.wrapping_add((5u32).wrapping_mul(l_max_packet_count));
    p_j2k.m_specific_param.m_encoder.m_reserved_bytes_for_plt = p_j2k
      .m_specific_param
      .m_encoder
      .m_reserved_bytes_for_plt
      .wrapping_add(l_nb_bytes)
      .wrapping_add(1u32);
    l_nb_bytes = l_nb_bytes.wrapping_add(p_j2k.m_specific_param.m_encoder.m_reserved_bytes_for_plt)
  }
  /* ** DEVELOPER CORNER, Add room for your headers ***/
  l_nb_bytes
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
fn opj_j2k_read_poc(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  let mut i: u32 = 0;
  let mut l_tmp: u32 = 0;
  let mut l_current_poc_nb: u32 = 0;
  /* preconditions */

  let l_image = p_j2k.m_private_image.as_deref().expect("m_private_image");
  let l_nb_comp: u32 = l_image.numcomps;
  let l_comp_room: u32 = if l_nb_comp <= 256u32 { 1_u32 } else { 2_u32 };
  let l_chunk_size: u32 = (5u32).wrapping_add((2u32).wrapping_mul(l_comp_room));
  l_current_poc_nb = p_header_size.wrapping_div(l_chunk_size);
  let l_current_poc_remaining: u32 = p_header_size.wrapping_rem(l_chunk_size);
  if l_current_poc_nb == 0u32 || l_current_poc_remaining != 0u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading POC marker\n",);
    return 0i32;
  }
  let is_tph = p_j2k.m_specific_param.m_decoder.m_state == J2KState::TPH;
  let l_tcp = if is_tph {
    &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize]
  } else {
    p_j2k
      .m_specific_param
      .m_decoder
      .m_default_tcp
      .as_deref_mut()
      .expect("decoder m_default_tcp is always set")
  };
  let l_old_poc_nb: u32 = if l_tcp.poc {
    l_tcp.numpocs.wrapping_add(1u32)
  } else {
    0u32
  };
  l_current_poc_nb = l_current_poc_nb.wrapping_add(l_old_poc_nb);
  if l_current_poc_nb >= 32u32 {
    event_msg!(p_manager, EVT_ERROR, "Too many POCs {}\n", l_current_poc_nb,);
    return 0i32;
  }
  /* now poc is in use.*/
  l_tcp.poc = true; /* RSpoc_i */
  i = l_old_poc_nb; /* CSpoc_i */
  while i < l_current_poc_nb {
    /* LYEpoc_i */
    let resno0 = opj_read_bytes(&header_data[cursor..], 1_u32);
    cursor += 1usize;
    let compno0 = opj_read_bytes(&header_data[cursor..], l_comp_room);
    cursor += l_comp_room as usize;
    let layno1 = opj_read_bytes(&header_data[cursor..], 2_u32);
    cursor += 2usize;
    let resno1 = opj_read_bytes(&header_data[cursor..], 1_u32);
    cursor += 1usize;
    let compno1 = opj_read_bytes(&header_data[cursor..], l_comp_room);
    cursor += l_comp_room as usize;
    l_tmp = opj_read_bytes(&header_data[cursor..], 1_u32);
    cursor += 1usize;
    let poc = &mut l_tcp.pocs[i as usize];
    poc.resno0 = resno0;
    poc.compno0 = compno0;
    poc.layno1 = opj_uint_min(layno1, l_tcp.numlayers); /* REpoc_i */
    poc.resno1 = resno1; /* CEpoc_i */
    poc.compno1 = compno1; /* Ppoc_i */
    poc.prg = l_tmp as OPJ_PROG_ORDER;
    /* make sure comp is in acceptable bounds */
    poc.compno1 = opj_uint_min(poc.compno1, l_nb_comp);
    i += 1;
  }
  l_tcp.numpocs = l_current_poc_nb.wrapping_sub(1u32);
  1i32
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
fn opj_j2k_read_crg(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  /* preconditions */

  let l_nb_comp = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  if p_header_size != l_nb_comp.wrapping_mul(4u32) {
    event_msg!(p_manager, EVT_ERROR, "Error reading CRG marker\n",);
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
  1i32
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
fn opj_j2k_read_tlm(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  if header_data.len() < 2 {
    event_msg!(p_manager, EVT_ERROR, "Error reading TLM marker.\n",);
    return 0i32;
  }
  let p_header_size = header_data.len() as u32 - 2u32;
  let mut cursor: usize = 0usize;
  if p_j2k.m_specific_param.m_decoder.m_tlm.m_is_invalid {
    return 1i32;
  }
  let _l_ztlm = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  let l_stlm = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  let l_st: u32 = (l_stlm >> 4) & 0x3u32;
  if l_st == 3u32 {
    p_j2k.m_specific_param.m_decoder.m_tlm.m_is_invalid = true;
    event_msg!(
      p_manager,
      EVT_WARNING,
      "opj_j2k_read_tlm(): ST = 3 is invalid.\n",
    );
    return 1i32;
  }
  let l_sp: u32 = (l_stlm >> 6) & 0x1u32;
  let l_ptlm_size: u32 = (l_sp + 1u32) * 2u32;
  let l_entry_size: u32 = l_ptlm_size + l_st;
  if !p_header_size.is_multiple_of(l_entry_size) {
    p_j2k.m_specific_param.m_decoder.m_tlm.m_is_invalid = true;
    event_msg!(
      p_manager,
      EVT_WARNING,
      "opj_j2k_read_tlm(): TLM marker not of expected size.\n",
    );
    return 1i32;
  }
  let l_num_tileparts: u32 = p_header_size / l_entry_size;
  if l_num_tileparts == 0 {
    /* not totally sure if this is valid... */
    return 1i32;
  }
  {
    let tlm = &mut p_j2k.m_specific_param.m_decoder.m_tlm;
    /* Highly unlikely, unless there are gazillions of TLM markers */
    if tlm.m_entries_count > u32::MAX - l_num_tileparts
      || tlm.m_entries_count + l_num_tileparts
        > (u32::MAX / core::mem::size_of::<TlmTilePartInfo>() as u32)
    {
      tlm.m_is_invalid = true;
      event_msg!(
        p_manager,
        EVT_WARNING,
        "opj_j2k_read_tlm(): too many TLM markers.\n",
      );
      return 1i32;
    }
    /* The C's realloc fails gracefully on OOM; a Vec resize would abort the
    process.  Reserve first so a crafted stream (many TLM markers) fails
    cleanly (review round 1, "infallible-allocation aborts"). */
    let new_len = (tlm.m_entries_count + l_num_tileparts) as usize;
    if tlm
      .m_tile_part_infos
      .try_reserve_exact(new_len.saturating_sub(tlm.m_tile_part_infos.len()))
      .is_err()
    {
      tlm.m_is_invalid = true;
      event_msg!(
        p_manager,
        EVT_WARNING,
        "opj_j2k_read_tlm(): not enough memory for the TLM index.\n",
      );
      return 1i32;
    }
    tlm.m_tile_part_infos.resize(new_len, Default::default());
  }
  let num_tiles = p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th);
  for _ in 0..l_num_tileparts {
    let l_tile_index: u32 = if l_st == 0 {
      p_j2k.m_specific_param.m_decoder.m_tlm.m_entries_count
    } else {
      let v = opj_read_bytes(&header_data[cursor..], l_st);
      cursor += l_st as usize;
      v
    };
    if l_tile_index >= num_tiles {
      p_j2k.m_specific_param.m_decoder.m_tlm.m_is_invalid = true;
      event_msg!(
        p_manager,
        EVT_WARNING,
        "opj_j2k_read_tlm(): invalid tile number {}\n",
        l_tile_index,
      );
      return 1i32;
    }
    let l_length = opj_read_bytes(&header_data[cursor..], l_ptlm_size);
    cursor += l_ptlm_size as usize;
    let entries = p_j2k.m_specific_param.m_decoder.m_tlm.m_entries_count;
    p_j2k
      .m_specific_param
      .m_decoder
      .m_tlm
      .m_tile_part_infos[entries as usize] = TlmTilePartInfo {
      m_tile_index: l_tile_index as u16,
      m_length: l_length,
    };
    p_j2k.m_specific_param.m_decoder.m_tlm.m_entries_count += 1;
  }
  1i32
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
fn opj_j2k_read_plm(
  mut _p_j2k: &mut J2k,
  header_data: &[u8],
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  /* preconditions */

  if p_header_size < 1u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading PLM marker\n",);
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
                  event_msg!(p_manager, EVT_ERROR, "Error reading PLM marker\n");
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
                  event_msg!(p_manager, EVT_ERROR, "Error reading PLM marker\n");
                  return false;
          }
  }
  */
  1i32
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
fn opj_j2k_read_plt(_p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  let mut l_tmp: u32 = 0;
  let mut l_packet_len = 0_u32;
  let mut i: u32 = 0;
  /* preconditions */
  /* Iplt_ij */

  if p_header_size < 1u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading PLT marker\n",);
    return 0i32;
  }
  let _ = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  let remaining = p_header_size.wrapping_sub(1);
  i = 0_u32;
  while i < remaining {
    l_tmp = opj_read_bytes(&header_data[cursor..], 1_u32);
    cursor += 1usize;
    /* take only the last seven bytes */
    l_packet_len |= l_tmp & 0x7fu32;
    if l_tmp & 0x80u32 != 0 {
      /* The C's unsigned shift wraps; wrapping_shl keeps debug builds from
      panicking on 5+ continuation bytes without changing any output. */
      l_packet_len = l_packet_len.wrapping_shl(7)
    } else {
      /* store packet length and proceed to next packet */
      l_packet_len = 0_u32
    }
    i += 1;
  }
  if l_packet_len != 0u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading PLT marker\n",);
    return 0i32;
  }
  1i32
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
fn opj_j2k_read_ppm(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  /* preconditions */

  /* We need to have the Z_ppm element + 1 byte of Nppm/Ippm at minimum */
  if p_header_size < 2u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading PPM marker\n",); /* Z_ppm */
    return 0i32;
  }
  let l_cp = &mut p_j2k.m_cp;
  l_cp.ppm = true;
  let l_z_ppm: u32 = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  /* check allocation needed */
  if l_cp.ppm_markers.len() as u32 <= l_z_ppm {
    /* grow-only; the tail is default-initialised, as in the C realloc+memset */
    let l_new_count = l_z_ppm.wrapping_add(1u32) as usize; /* can't overflow, l_z_ppm is UINT8 */
    l_cp.ppm_markers.resize_with(l_new_count, Default::default);
  }
  if !l_cp.ppm_markers[l_z_ppm as usize].m_data.is_empty() {
    /* clean up to be done on l_cp destruction */
    event_msg!(p_manager, EVT_ERROR, "Zppm {} already read\n", l_z_ppm,);
    return 0i32;
  }
  l_cp.ppm_markers[l_z_ppm as usize].m_data = header_data[cursor..].to_vec();
  1i32
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
fn opj_j2k_merge_ppm(p_cp: &mut Cp, p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut i: u32 = 0;
  let mut l_ppm_data_size: u32 = 0;
  let mut l_n_ppm_remaining: u32 = 0;
  /* preconditions */

  assert!(p_cp.ppm_buffer.is_empty());
  if !p_cp.ppm {
    return 1i32;
  }

  /* First pass: total up the Nppm payload lengths and validate. The logical stream is
  [Nppm][Nppm bytes of packet headers]... and it may straddle marker boundaries, but an
  Nppm field itself may not. */
  l_ppm_data_size = 0u32;
  l_n_ppm_remaining = 0u32;
  i = 0u32;
  while (i as usize) < p_cp.ppm_markers.len() {
    if !(&p_cp.ppm_markers)[i as usize].m_data.is_empty() {
      /* standard doesn't seem to require contiguous Zppm */
      let mut l_data: &[u8] = &(&p_cp.ppm_markers)[i as usize].m_data[..];
      if l_n_ppm_remaining >= l_data.len() as u32 {
        l_n_ppm_remaining -= l_data.len() as u32;
        l_data = &[];
      } else {
        l_data = &l_data[l_n_ppm_remaining as usize..];
        l_n_ppm_remaining = 0u32
      }
      while !l_data.is_empty() {
        /* read Nppm */
        if l_data.len() < 4 {
          /* clean up to be done on l_cp destruction */
          event_msg!(p_manager, EVT_ERROR, "Not enough bytes to read Nppm\n",);
          return 0i32;
        }
        let l_n_ppm = crate::cio::opj_read_bytes(l_data, 4_u32);
        l_data = &l_data[4..];

        if l_ppm_data_size > u32::MAX - l_n_ppm {
          event_msg!(p_manager, EVT_ERROR, "Too large value for Nppm\n",);
          return 0i32;
        }
        l_ppm_data_size += l_n_ppm;
        if l_data.len() as u32 >= l_n_ppm {
          l_data = &l_data[l_n_ppm as usize..]
        } else {
          l_n_ppm_remaining = l_n_ppm - l_data.len() as u32;
          l_data = &[]
        }
      }
    }
    i += 1;
  }
  if l_n_ppm_remaining != 0u32 {
    /* clean up to be done on l_cp destruction */
    event_msg!(p_manager, EVT_ERROR, "Corrupted PPM markers\n",);
    return 0i32;
  }
  /* Second pass: concatenate the payloads, dropping the Nppm fields. */
  let mut l_buffer: Vec<u8> = Vec::with_capacity(l_ppm_data_size as usize);
  p_cp.ppm_len = l_ppm_data_size;
  l_n_ppm_remaining = 0u32;
  i = 0u32;
  while (i as usize) < p_cp.ppm_markers.len() {
    if !(&p_cp.ppm_markers)[i as usize].m_data.is_empty() {
      /* standard doesn't seem to require contiguous Zppm */
      let mut l_data: &[u8] = &(&p_cp.ppm_markers)[i as usize].m_data[..];
      if l_n_ppm_remaining >= l_data.len() as u32 {
        l_buffer.extend_from_slice(l_data);
        l_n_ppm_remaining -= l_data.len() as u32;
        l_data = &[];
      } else {
        l_buffer.extend_from_slice(&l_data[..l_n_ppm_remaining as usize]);
        l_data = &l_data[l_n_ppm_remaining as usize..];
        l_n_ppm_remaining = 0u32
      }
      while !l_data.is_empty() {
        /* read Nppm */
        if l_data.len() < 4 {
          /* clean up to be done on l_cp destruction */
          event_msg!(p_manager, EVT_ERROR, "Not enough bytes to read Nppm\n",);
          return 0i32;
        }
        let l_n_ppm = crate::cio::opj_read_bytes(l_data, 4_u32);
        l_data = &l_data[4..];
        if l_data.len() as u32 >= l_n_ppm {
          l_buffer.extend_from_slice(&l_data[..l_n_ppm as usize]);
          l_data = &l_data[l_n_ppm as usize..]
        } else {
          l_buffer.extend_from_slice(l_data);
          l_n_ppm_remaining = l_n_ppm - l_data.len() as u32;
          l_data = &[]
        }
      }
      (&mut p_cp.ppm_markers)[i as usize].m_data = Vec::new()
    }
    i += 1;
  }
  p_cp.ppm_data_pos = 0;
  p_cp.ppm_buffer = l_buffer;
  p_cp.ppm_markers = Vec::new();
  1i32
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
fn opj_j2k_read_ppt(
  mut p_j2k: &mut J2k,
  header_data: &[u8],
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  let mut l_z_ppt: u32 = 0;
  /* preconditions */

  /* We need to have the Z_ppt element + 1 byte of Ippt at minimum */
  if p_header_size < 2u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading PPT marker\n",); /* Z_ppt */
    return 0i32;
  }
  let cp = &mut p_j2k.m_cp;
  if cp.ppm {
    event_msg!(p_manager, EVT_ERROR,
                    "Error reading PPT marker: packet header have been previously found in the main header (PPM marker).\n");
    return 0i32;
  }
  let tcp = &mut cp.tcps[p_j2k.m_current_tile_number as usize];
  tcp.ppt = true;
  l_z_ppt = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  p_header_size = p_header_size.wrapping_sub(1);
  /* check allocation needed */
  if (tcp.ppt_markers.len() as u32) <= l_z_ppt {
    /* grow-only; the tail is default-initialised, as in the C realloc+memset */
    let l_new_count = l_z_ppt.wrapping_add(1u32) as usize; /* can't overflow, l_z_ppt is UINT8 */
    tcp.ppt_markers.resize_with(l_new_count, Default::default);
  }
  if !tcp.ppt_markers[l_z_ppt as usize].m_data.is_empty() {
    /* clean up to be done on l_tcp destruction */
    event_msg!(p_manager, EVT_ERROR, "Zppt {} already read\n", l_z_ppt,);
    return 0i32;
  }
  tcp.ppt_markers[l_z_ppt as usize].m_data = header_data[cursor..].to_vec();
  1i32
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
fn opj_j2k_merge_ppt(p_tcp: &mut Tcp, p_manager: &mut EventMgr) -> OPJ_BOOL {
  /* preconditions */

  if !p_tcp.ppt_buffer.is_empty() {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "opj_j2k_merge_ppt() has already been called\n",
    );
    return 0i32;
  }
  if !p_tcp.ppt {
    return 1i32;
  }
  /* concatenate in Zppt order; the standard doesn't seem to require contiguous Zppt,
  and can't overflow, max 256 markers of max 65536 bytes */
  let mut l_buffer = Vec::new();
  for l_marker in p_tcp.ppt_markers.iter_mut() {
    l_buffer.append(&mut l_marker.m_data);
  }
  p_tcp.ppt_markers = Vec::new();
  p_tcp.ppt_len = l_buffer.len() as u32;
  p_tcp.ppt_data_pos = 0;
  p_tcp.ppt_buffer = l_buffer;
  1i32
}
/* *
 * Writes the TLM marker (Tile Length Marker)
 *
 * @param       p_stream                                the stream to write data to.
 * @param       p_j2k                           J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_write_tlm(p_j2k: &mut J2k, p_stream: &mut Stream, p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut l_tlm_size: u32 = 0;
  let mut size_per_tile_part: u32 = 0;
  /* preconditions */

  /* 10921 = (65535 - header_size) / size_per_tile_part where */
  /* header_size = 4 and size_per_tile_part = 6 */
  if p_j2k.m_specific_param.m_encoder.m_total_tile_parts > 10921u32 {
    /* We could do more but it would require writing several TLM markers */
    event_msg!(
      p_manager,
      EVT_ERROR,
      "A maximum of 10921 tile-parts are supported currently when writing TLM marker\n",
    );
    return 0i32;
  }
  if p_j2k.m_specific_param.m_encoder.m_total_tile_parts <= 255u32 {
    size_per_tile_part = 5_u32;
    p_j2k.m_specific_param.m_encoder.m_ttlmi_is_byte = 1i32
  } else {
    size_per_tile_part = 6_u32;
    p_j2k.m_specific_param.m_encoder.m_ttlmi_is_byte = 0i32
  }
  l_tlm_size = ((2i32 + 4i32) as core::ffi::c_uint).wrapping_add(
    size_per_tile_part.wrapping_mul(p_j2k.m_specific_param.m_encoder.m_total_tile_parts),
  );
  let mut l_buf = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_buf.resize(l_tlm_size as usize, 0);
  let mut pos: usize = 0;
  /* change the way data is written to avoid seeking if possible */
  p_j2k.m_specific_param.m_encoder.m_tlm_start = opj_stream_tell(p_stream); /* TLM */
  opj_write_bytes(&mut l_buf[pos..], J2KMarker::TLM.as_u32(), 2_u32); /* Lpoc */
  pos += 2usize; /* Ztlm=0*/
  opj_write_bytes(&mut l_buf[pos..], l_tlm_size.wrapping_sub(2u32), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], 0_u32, 1_u32);
  pos += 1usize;
  /* Stlm 0x50= ST=1(8bits-255 tiles max),SP=1(Ptlm=32bits) */
  /* Stlm 0x60= ST=2(16bits-65535 tiles max),SP=1(Ptlm=32bits) */
  opj_write_bytes(
    &mut l_buf[pos..],
    if size_per_tile_part == 5u32 {
      0x50i32
    } else {
      0x60i32
    } as u32,
    1_u32,
  );
  pos += 1usize;
  /* do nothing on the size_per_tile_part * l_j2k->m_specific_param.m_encoder.m_total_tile_parts remaining data */
  if opj_stream_write_data(p_stream, &l_buf[..l_tlm_size as usize], p_manager)
    != l_tlm_size as usize
  {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_buf;
  1i32
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
fn opj_j2k_write_sot(
  p_j2k: &mut J2k,
  data: &mut [u8],
  p_data_written: &mut u32,
  _p_stream: &Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut pos: usize = 0;
  /* preconditions */
  /* Lsot */

  if data.len() < 12 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Not enough bytes in output buffer to write SOT marker\n",
    );
    return 0i32;
  }
  opj_write_bytes(&mut data[pos..], J2KMarker::SOT.as_u32(), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut data[pos..], 10_u32, 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut data[pos..], p_j2k.m_current_tile_number, 2_u32);
  pos += 2usize;
  /* Psot  */
  pos += 4usize; /* TPsot */
  opj_write_bytes(
    &mut data[pos..],
    p_j2k.m_specific_param.m_encoder.m_current_tile_part_number,
    1_u32,
  ); /* TNsot */
  pos += 1usize;
  opj_write_bytes(
    &mut data[pos..],
    p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize].m_nb_tile_parts,
    1_u32,
  );
  pos += 1usize;
  /* UniPG>> */
  /* USE_JPWL */
  *p_data_written = pos as u32;
  1i32
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
fn opj_j2k_get_sot_values(
  header_data: &[u8],
  p_tile_no: &mut u32,
  p_tot_len: &mut u32,
  p_current_part: &mut u32,
  p_num_parts: &mut u32,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut cursor: usize = 0;
  /* preconditions */

  /* Size of this marker is fixed = 12 (we have already read marker and its size)*/
  if header_data.len() != 8u32 as usize {
    event_msg!(p_manager, EVT_ERROR, "Error reading SOT marker\n",); /* Isot */
    return 0i32;
  } /* Psot */
  *p_tile_no = opj_read_bytes(&header_data[cursor..], 2_u32); /* TPsot */
  cursor += 2usize; /* TNsot */
  *p_tot_len = opj_read_bytes(&header_data[cursor..], 4_u32);
  cursor += 4usize;
  *p_current_part = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  *p_num_parts = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  1i32
}
/* *
 * Reads a SOT marker (Start of tile-part)
 *
 * @param       p_header_data   the data contained in the SOT marker.
 * @param       p_j2k           the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the PPT marker.
 * @param       p_manager       the user event manager.
*/
fn opj_j2k_read_sot(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut l_tot_len: u32 = 0;
  let mut l_num_parts = 0_u32;
  let mut l_current_part: u32 = 0;
  let mut l_tile_x: u32 = 0;
  let mut l_tile_y: u32 = 0;
  /* preconditions */

  if opj_j2k_get_sot_values(
    header_data,
    &mut p_j2k.m_current_tile_number,
    &mut l_tot_len,
    &mut l_current_part,
    &mut l_num_parts,
    p_manager,
  ) == 0
  {
    event_msg!(p_manager, EVT_ERROR, "Error reading SOT marker\n",);
    return 0i32;
  }
  /* testcase 2.pdf.SIGFPE.706.1112 */
  if p_j2k.m_current_tile_number >= p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th) {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid tile number {}\n",
      p_j2k.m_current_tile_number,
    );
    return 0i32;
  }
  let l_tcp = &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize];
  l_tile_x = p_j2k.m_current_tile_number.wrapping_rem(p_j2k.m_cp.tw);
  l_tile_y = p_j2k.m_current_tile_number.wrapping_div(p_j2k.m_cp.tw);
  if p_j2k.m_specific_param.m_decoder.m_tile_ind_to_dec < 0i32
    || p_j2k.m_current_tile_number == p_j2k.m_specific_param.m_decoder.m_tile_ind_to_dec as u32
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
    if l_tcp.m_current_tile_part_number + 1i32 != l_current_part as i32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Invalid tile part index for tile number {}. Got {}, expected {}\n",
        p_j2k.m_current_tile_number,
        l_current_part,
        l_tcp.m_current_tile_part_number + 1i32,
      );
      return 0i32;
    }
  }
  l_tcp.m_current_tile_part_number = l_current_part as i32;
  /* USE_JPWL */
  /* look for the tile in the list of already processed tile (in parts). */
  /* Optimization possible here with a more complex data structure and with the removing of tiles */
  /* since the time taken by this function can only grow at the time */
  /* PSot should be equal to zero or >=14 or <= 2^32-1 */
  if l_tot_len != 0u32 && l_tot_len < 14u32 {
    if l_tot_len == 12u32 {
      /* MSD: Special case for the PHR data which are read by kakadu*/
      event_msg!(
        p_manager,
        EVT_WARNING,
        "Empty SOT marker detected: Psot={}.\n",
        l_tot_len,
      );
    } else {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Psot value is not correct regards to the JPEG2000 norm: {}.\n",
        l_tot_len,
      );
      return 0i32;
    }
  }
  /* USE_JPWL */
  /* Ref A.4.2: Psot could be equal zero if it is the last tile-part of the codestream.*/
  if l_tot_len == 0 {
    event_msg!(p_manager, EVT_INFO,
                      "Psot value of the current tile-part is equal to zero, we assuming it is the last tile-part of the codestream.\n");
    p_j2k.m_specific_param.m_decoder.m_last_tile_part = 1i32
  }
  if l_tcp.m_nb_tile_parts != 0u32 && l_current_part >= l_tcp.m_nb_tile_parts {
    /* Fixes https://bugs.chromium.org/p/oss-fuzz/issues/detail?id=2851 */
    event_msg!(p_manager, EVT_ERROR,
                      "In SOT marker, TPSot ({}) is not valid regards to the previous number of tile-part ({}), giving up\n", l_current_part,
                      l_tcp.m_nb_tile_parts);
    p_j2k.m_specific_param.m_decoder.m_last_tile_part = 1i32;
    return 0i32;
  }
  if l_num_parts != 0u32 {
    /* Number of tile-part header is provided by this tile-part header */
    l_num_parts = (l_num_parts as core::ffi::c_uint)
      .wrapping_add(p_j2k.m_specific_param.m_decoder.m_nb_tile_parts_correction as _);
    /* Useful to manage the case of textGBR.jp2 file because two values of TNSot are allowed: the correct numbers of
     * tile-parts for that tile and zero (A.4.2 of 15444-1 : 2002). */
    if l_tcp.m_nb_tile_parts != 0 && l_current_part >= l_tcp.m_nb_tile_parts {
      event_msg!(p_manager, EVT_ERROR,
                            "In SOT marker, TPSot ({}) is not valid regards to the current number of tile-part ({}), giving up\n",
                            l_current_part, l_tcp.m_nb_tile_parts);
      p_j2k.m_specific_param.m_decoder.m_last_tile_part = 1i32;
      return 0i32;
    }
    if l_current_part >= l_num_parts {
      /* testcase 451.pdf.SIGSEGV.ce9.3723 */
      event_msg!(p_manager, EVT_ERROR,
                          "In SOT marker, TPSot ({}) is not valid regards to the current number of tile-part (header) ({}), giving up\n",
                          l_current_part, l_num_parts);
      p_j2k.m_specific_param.m_decoder.m_last_tile_part = 1i32;
      return 0i32;
    }
    l_tcp.m_nb_tile_parts = l_num_parts
  }
  /* If know the number of tile part header we will check if we didn't read the last*/
  if l_tcp.m_nb_tile_parts != 0 && l_tcp.m_nb_tile_parts == l_current_part.wrapping_add(1u32) {
    p_j2k.m_specific_param.m_decoder.m_can_decode = true;
    /* Process the last tile-part header*/
  }
  if p_j2k.m_specific_param.m_decoder.m_last_tile_part == 0 {
    /* Keep the size of data to skip after this marker */
    p_j2k.m_specific_param.m_decoder.m_sot_length = l_tot_len.wrapping_sub(12u32)
  /* SOT_marker_size = 12 */
  } else {
    /* Note: could be computed from the bytes remaining in the codestream */
    p_j2k.m_specific_param.m_decoder.m_sot_length = 0_u32
  }
  p_j2k.m_specific_param.m_decoder.m_state = J2KState::TPH;
  /* Check if the current tile is outside the area we want decode or not corresponding to the tile index*/
  if p_j2k.m_specific_param.m_decoder.m_tile_ind_to_dec == -(1i32) {
    p_j2k.m_specific_param.m_decoder.m_skip_data = l_tile_x
      < p_j2k.m_specific_param.m_decoder.m_start_tile_x
      || l_tile_x >= p_j2k.m_specific_param.m_decoder.m_end_tile_x
      || l_tile_y < p_j2k.m_specific_param.m_decoder.m_start_tile_y
      || l_tile_y >= p_j2k.m_specific_param.m_decoder.m_end_tile_y;
  } else {
    assert!(p_j2k.m_specific_param.m_decoder.m_tile_ind_to_dec >= 0i32);
    p_j2k.m_specific_param.m_decoder.m_skip_data =
      p_j2k.m_current_tile_number != p_j2k.m_specific_param.m_decoder.m_tile_ind_to_dec as u32;
  }
  /* Index */
  if let Some(cstr_index) = p_j2k.cstr_index.as_deref_mut() {
    let tile = &mut cstr_index.tiles[p_j2k.m_current_tile_number as usize];
    tile.tileno = p_j2k.m_current_tile_number;
    tile.current_tpsno = l_current_part;
    let mut tlm_invalid = p_j2k.m_specific_param.m_decoder.m_tlm.m_is_invalid;
    if !tlm_invalid && l_num_parts > tile.nb_tps {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "SOT marker for tile {} declares more tile-parts than found in TLM marker.",
        p_j2k.m_current_tile_number,
      );
      tlm_invalid = true;
      p_j2k.m_specific_param.m_decoder.m_tlm.m_is_invalid = true;
    }
    if !tlm_invalid {
      /* The tile-part index was already built from the TLM markers */
    } else if l_num_parts != 0u32 {
      tile.nb_tps = l_num_parts;
      tile.current_nb_tps = l_num_parts;
      tile
        .tp_index
        .resize(l_num_parts as usize, Default::default());
    } else {
      if tile.tp_index.is_empty() {
        tile.current_nb_tps = 10_u32;
        tile.tp_index.resize(10, Default::default());
      }
      if l_current_part >= tile.current_nb_tps {
        tile.current_nb_tps = l_current_part.wrapping_add(1u32);
        tile
          .tp_index
          .resize(l_current_part as usize + 1, Default::default());
      }
    }
  }
  1i32
}
/* *
 * Write one or more PLT markers in the provided buffer
 */
fn opj_j2k_write_plt_in_memory(
  _p_j2k: &J2k,
  marker_info: &TcdMarkerInfo,
  data: &mut [u8],
  p_data_written: &mut u32,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut z_plt = 0_u8;
  let mut l_plt: u16 = 0;
  let pos_start: usize = 0;
  let mut pos_l_plt: usize = 2;
  let mut pos: usize = 0;
  let mut i: u32 = 0;
  let packet_sizes = &marker_info.p_packet_size[..];
  opj_write_bytes(&mut data[pos..], J2KMarker::PLT.as_u32(), 2_u32);
  pos += 2usize;
  /* Reserve space for l_plt */
  pos += 2usize;
  opj_write_bytes(&mut data[pos..], z_plt as u32, 1_u32);
  pos += 1usize;
  l_plt = 3_u16;
  i = 0_u32;
  while i < marker_info.packet_count {
    let mut var_bytes: [u8; 5] = [0; 5];
    let mut var_bytes_size = 0_u8;
    let mut packet_size = packet_sizes[i as usize];
    /* Packet size written in variable-length way, starting with LSB */
    var_bytes[var_bytes_size as usize] = (packet_size & 0x7fu32) as u8;
    var_bytes_size = var_bytes_size.wrapping_add(1);
    packet_size >>= 7i32;
    while packet_size > 0u32 {
      var_bytes[var_bytes_size as usize] = (packet_size & 0x7fu32 | 0x80u32) as u8;
      var_bytes_size = var_bytes_size.wrapping_add(1);
      packet_size >>= 7i32
    }
    /* Check if that can fit in the current PLT marker. If not, finish */
    /* current one, and start a new one */
    if l_plt as core::ffi::c_int + var_bytes_size as core::ffi::c_int > 65535i32 {
      if z_plt as core::ffi::c_int == 255i32 {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "More than 255 PLT markers would be needed for current tile-part !\n",
        );
        return 0i32;
      }
      /* Patch l_plt */
      opj_write_bytes(&mut data[pos_l_plt..], l_plt as u32, 2_u32);
      /* Start new segment */
      opj_write_bytes(&mut data[pos..], J2KMarker::PLT.as_u32(), 2_u32);
      pos += 2usize;
      /* Reserve space for l_plt */
      pos_l_plt = pos;
      pos += 2usize;
      z_plt = z_plt.wrapping_add(1);
      opj_write_bytes(&mut data[pos..], z_plt as u32, 1_u32);
      pos += 1usize;
      l_plt = 3_u16
    }
    l_plt = (l_plt as core::ffi::c_int + var_bytes_size as core::ffi::c_int) as u16;
    /* Serialize variable-length packet size, starting with MSB */
    while var_bytes_size as core::ffi::c_int > 0i32 {
      opj_write_bytes(
        &mut data[pos..],
        var_bytes[(var_bytes_size as core::ffi::c_int - 1i32) as usize] as u32,
        1_u32,
      );
      pos += 1usize;
      var_bytes_size = var_bytes_size.wrapping_sub(1)
    }
    i += 1;
  }
  *p_data_written = (pos - pos_start) as u32;
  /* Patch l_plt */
  opj_write_bytes(&mut data[pos_l_plt..], l_plt as u32, 2_u32);
  1i32
}

/* *
 * Writes the SOD marker (Start of data)
 *
 * This also writes optional PLT markers (before SOD)
 *
 * @param       p_j2k               J2K codec.
 * @param p_data
 * @param p_data_written
 * @param total_data_size
 * @param       p_stream            the stream to write data to.
 * @param       p_manager           the user event manager.
*/
fn opj_j2k_write_sod(
  p_j2k: &mut J2k,
  data: &mut [u8],
  p_data_written: &mut u32,
  _p_stream: &Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_remaining_data: u32 = 0;
  /* preconditions */

  if data.len() < 4 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Not enough bytes in output buffer to write SOD marker\n",
    );
    return 0i32;
  }
  opj_write_bytes(&mut data[..], J2KMarker::SOD.as_u32(), 2_u32);
  /* make room for the EOF marker */
  l_remaining_data = (data.len() as u32).wrapping_sub(4u32);
  /* update tile coder */
  p_j2k.m_tcd.tp_num = p_j2k
    .m_specific_param
    .m_encoder
    .m_current_poc_tile_part_number;
  p_j2k.m_tcd.cur_tp_num = p_j2k.m_specific_param.m_encoder.m_current_tile_part_number;
  if p_j2k.m_specific_param.m_encoder.m_current_tile_part_number == 0u32 {
    p_j2k.m_tcd.tcd_image.tiles.packno = 0_u32
  }
  *p_data_written = 0_u32;
  let mut marker_info = TcdMarkerInfo::new(p_j2k.m_specific_param.m_encoder.m_plt);
  if l_remaining_data < p_j2k.m_specific_param.m_encoder.m_reserved_bytes_for_plt {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Not enough bytes in output buffer to write SOD marker\n",
    );
    return 0i32;
  }
  l_remaining_data = (l_remaining_data as core::ffi::c_uint)
    .wrapping_sub(p_j2k.m_specific_param.m_encoder.m_reserved_bytes_for_plt);
  if opj_tcd_encode_tile(
    &mut p_j2k.m_tcd,
    p_j2k.m_current_tile_number,
    &mut data[2..],
    p_data_written,
    l_remaining_data,
    Some(&mut marker_info),
    p_manager,
    p_j2k
      .m_private_image
      .as_deref_mut()
      .expect("m_private_image"),
    &mut p_j2k.m_cp,
  ) == 0
  {
    event_msg!(p_manager, EVT_ERROR, "Cannot encode tile\n",);
    return 0i32;
  }
  /* For SOD */
  *p_data_written = (*p_data_written as core::ffi::c_uint).wrapping_add(2u32);
  if p_j2k.m_specific_param.m_encoder.m_plt != 0 {
    let mut l_data_written_plt = 0_u32;
    let mut p_plt_buffer =
      vec![0u8; p_j2k.m_specific_param.m_encoder.m_reserved_bytes_for_plt as usize];
    if opj_j2k_write_plt_in_memory(
      &*p_j2k,
      &marker_info,
      &mut p_plt_buffer[..],
      &mut l_data_written_plt,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    assert!(l_data_written_plt <= p_j2k.m_specific_param.m_encoder.m_reserved_bytes_for_plt);
    /* Move PLT marker(s) before SOD */
    data.copy_within(0..*p_data_written as usize, l_data_written_plt as usize);
    data[..l_data_written_plt as usize]
      .copy_from_slice(&p_plt_buffer[..l_data_written_plt as usize]);
    *p_data_written = (*p_data_written as core::ffi::c_uint).wrapping_add(l_data_written_plt)
  }
  1i32
}
/* *
 * Reads a SOD marker (Start Of Data)
 *
 * @param       p_j2k                   the jpeg2000 codec.
 * @param p_stream
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_read_sod(p_j2k: &mut J2k, p_stream: &mut Stream, p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut l_current_read_size: usize = 0;
  let mut l_sot_length_pb_detected = 0i32;
  /* preconditions */

  let strict = p_j2k.m_cp.strict;
  let l_tcp = &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize];
  if p_j2k.m_specific_param.m_decoder.m_last_tile_part != 0 {
    /* opj_stream_get_number_byte_left returns i64
    // but we are in the last tile part,
    // so its result will fit on u32 unless we find
    // a file with a single tile part of more than 4 GB...*/
    p_j2k.m_specific_param.m_decoder.m_sot_length =
      (opj_stream_get_number_byte_left(p_stream) - 2i64) as u32
  } else if p_j2k.m_specific_param.m_decoder.m_sot_length >= 2u32 {
    p_j2k.m_specific_param.m_decoder.m_sot_length =
      (p_j2k.m_specific_param.m_decoder.m_sot_length as core::ffi::c_uint).wrapping_sub(2u32)
  }
  /* Check to avoid pass the limit of u32 */
  /* Patch to support new PHR data */
  if p_j2k.m_specific_param.m_decoder.m_sot_length != 0 {
    /* If we are here, we'll try to read the data after allocation */
    /* Check enough bytes left in stream before allocation */
    if p_j2k.m_specific_param.m_decoder.m_sot_length as i64
      > opj_stream_get_number_byte_left(p_stream)
      && p_stream.m_stream_length != 0
    {
      if strict != 0 {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Tile part length size inconsistent with stream length\n",
        );
        return 0i32;
      } else {
        event_msg!(
          p_manager,
          EVT_WARNING,
          "Tile part length size inconsistent with stream length\n",
        );
        /* Progressive download: a Psot that exceeds the available bytes must
        not drive a multi-gigabyte allocation - clamp the read to what is
        actually in the stream (the tile decode then stops cleanly when the
        data runs out).  A negative byte count (seek past the declared EOF
        via crafted TLM offsets) must clamp to zero, not wrap to a ~4 GiB
        value that slips past the u32 guard below. */
        p_j2k.m_specific_param.m_decoder.m_sot_length =
          opj_stream_get_number_byte_left(p_stream).max(0) as u32;
        /* The C's lenient path exits the tile loop through the NEOC state
        with no error event; mirror it so the next read_tile_header round
        does not re-enter at EOF and emit an unconditional error. */
        p_j2k.m_specific_param.m_decoder.m_state = J2KState::NEOC;
      }
    }
    if p_j2k.m_specific_param.m_decoder.m_sot_length > u32::MAX - OPJ_COMMON_CBLK_DATA_EXTRA {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "p_j2k->m_specific_param.m_decoder.m_sot_length > UINT_MAX - OPJ_COMMON_CBLK_DATA_EXTRA"
      );
      return 0i32;
    }
    /* Add a margin of OPJ_COMMON_CBLK_DATA_EXTRA to the allocation we */
    /* do so that opj_mqc_init_dec_common() can safely add a synthetic */
    /* 0xFFFF marker. */
    if l_tcp.m_data_size
      > u32::MAX - OPJ_COMMON_CBLK_DATA_EXTRA - p_j2k.m_specific_param.m_decoder.m_sot_length
    {
      event_msg!(p_manager, EVT_ERROR,
                          "*l_tile_len > UINT_MAX - OPJ_COMMON_CBLK_DATA_EXTRA - p_j2k->m_specific_param.m_decoder.m_sot_length");
      return 0i32;
    }
    let new_len = l_tcp
      .m_data_size
      .wrapping_add(p_j2k.m_specific_param.m_decoder.m_sot_length) as usize;
    if l_tcp
      .m_data
      .try_reserve_exact(new_len + 2 - l_tcp.m_data.len())
      .is_err()
    {
      event_msg!(p_manager, EVT_ERROR, "Not enough memory to decode tile\n",);
      return 0i32;
    }
    /* OPJ_COMMON_CBLK_DATA_EXTRA: margin for a fake 0xFFFF marker */
    l_tcp
      .m_data
      .resize(new_len + OPJ_COMMON_CBLK_DATA_EXTRA as usize, 0);
  } else {
    l_sot_length_pb_detected = 1i32
  }
  /* Index */
  if let Some(l_cstr_index) = p_j2k.cstr_index.as_deref_mut() {
    let mut l_current_pos = opj_stream_tell(p_stream) - 2i64;
    let tile = &mut l_cstr_index.tiles[p_j2k.m_current_tile_number as usize];
    let l_current_tile_part = tile.current_tpsno;
    if l_current_tile_part < tile.nb_tps {
      tile.tp_index[l_current_tile_part as usize].end_header = l_current_pos;
      tile.tp_index[l_current_tile_part as usize].end_pos =
        l_current_pos + p_j2k.m_specific_param.m_decoder.m_sot_length as i64 + 2i64;
    }
    if 0i32
      == opj_j2k_add_tlmarker(
        p_j2k.m_current_tile_number,
        p_j2k.cstr_index.as_deref_mut().expect("cstr_index"),
        J2KMarker::SOD,
        l_current_pos,
        p_j2k
          .m_specific_param
          .m_decoder
          .m_sot_length
          .wrapping_add(2u32),
      )
    {
      event_msg!(p_manager, EVT_ERROR, "Not enough memory to add tl marker\n",);
      return 0i32;
    }
    /*l_cstr_index->packno = 0;*/
  }
  /* Patch to support new PHR data */
  if l_sot_length_pb_detected == 0 {
    let tile_len = l_tcp.m_data_size as usize;
    l_current_read_size = opj_stream_read_data(
      p_stream,
      &mut l_tcp.m_data
        [tile_len..tile_len + p_j2k.m_specific_param.m_decoder.m_sot_length as usize],
      p_manager,
    )
  } else {
    l_current_read_size = 0_usize
  }
  if l_current_read_size != p_j2k.m_specific_param.m_decoder.m_sot_length as usize {
    if l_current_read_size == usize::MAX {
      /* opj_stream_read_data failed; avoid a later heap-buffer-overflow
      in opj_t1_decode_cblk in non-strict mode (upstream #1534/#1533) */
      event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
      return 0i32;
    }
    p_j2k.m_specific_param.m_decoder.m_state = J2KState::NEOC
  } else {
    p_j2k.m_specific_param.m_decoder.m_state = J2KState::TPHSOT
  }
  l_tcp.m_data_size = l_tcp.m_data_size.wrapping_add(l_current_read_size as u32);
  1i32
}

fn opj_j2k_write_rgn(
  p_j2k: &mut J2k,
  p_tile_no: u32,
  p_comp_no: u32,
  nb_comps: u32,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */
  /* Lrgn */
  /* Srgn */
  let l_tccp = &p_j2k.m_cp.tcps[p_tile_no as usize].tccps[p_comp_no as usize];
  let l_comp_room: u32 = if nb_comps <= 256u32 { 1_u32 } else { 2_u32 };
  let l_rgn_size: u32 = (6u32).wrapping_add(l_comp_room);
  let mut l_buf = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_buf.resize(l_rgn_size as usize, 0);
  let mut pos: usize = 0;
  opj_write_bytes(&mut l_buf[pos..], J2KMarker::RGN.as_u32(), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], l_rgn_size.wrapping_sub(2u32), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], p_comp_no, l_comp_room);
  pos += l_comp_room as usize;
  opj_write_bytes(&mut l_buf[pos..], 0_u32, 1_u32);
  pos += 1usize;
  opj_write_bytes(&mut l_buf[pos..], l_tccp.roishift as u32, 1_u32);
  pos += 1usize;
  if opj_stream_write_data(p_stream, &l_buf[..l_rgn_size as usize], p_manager)
    != l_rgn_size as usize
  {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_buf;
  1i32
}

/* *
 * Writes the EOC marker (End of Codestream)
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_write_eoc(
  mut p_j2k: &mut J2k,
  mut p_stream: &mut Stream,
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */

  let mut l_buf = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_buf.resize(2, 0);
  opj_write_bytes(&mut l_buf[..], J2KMarker::EOC.as_u32(), 2_u32);
  /* UniPG>> */
  /* USE_JPWL */
  if opj_stream_write_data(p_stream, &l_buf[..2], p_manager) != 2 {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_buf;
  if opj_stream_flush(p_stream, p_manager) == 0 {
    return 0i32;
  }
  1i32
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
fn opj_j2k_read_rgn(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  /* preconditions*/
  /* Srgn */

  let l_image = p_j2k.m_private_image.as_deref().expect("m_private_image");
  let l_nb_comp: u32 = l_image.numcomps;
  let l_comp_room: u32 = if l_nb_comp <= 256u32 { 1_u32 } else { 2_u32 };
  if p_header_size != (2u32).wrapping_add(l_comp_room) {
    event_msg!(p_manager, EVT_ERROR, "Error reading RGN marker\n",);
    return 0i32;
  }
  let is_tph = p_j2k.m_specific_param.m_decoder.m_state == J2KState::TPH;
  let l_tcp = if is_tph {
    &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize]
  } else {
    p_j2k
      .m_specific_param
      .m_decoder
      .m_default_tcp
      .as_deref_mut()
      .expect("decoder m_default_tcp is always set")
  };
  let l_comp_no: u32 = opj_read_bytes(&header_data[cursor..], l_comp_room);
  cursor += l_comp_room as usize;
  let _ = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  /* USE_JPWL */
  /* testcase 3635.pdf.asan.77.2930 */
  if l_comp_no >= l_nb_comp {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "bad component number in RGN ({} when there are only {})\n",
      l_comp_no,
      l_nb_comp,
    ); /* SPrgn */
    return 0i32;
  }
  l_tcp.tccps[l_comp_no as usize].roishift = opj_read_bytes(&header_data[cursor..], 1_u32) as i32;
  cursor += 1usize;
  1i32
}

fn opj_j2k_get_tp_stride(p_tcp: &Tcp) -> f32 {
  p_tcp.m_nb_tile_parts.wrapping_sub(1u32).wrapping_mul(14u32) as f32
}

fn opj_j2k_get_default_stride(_p_tcp: &Tcp) -> f32 {
  0 as f32
}

/* *
 * Updates the rates of the tcp.
 *
 * @param       p_stream                                the stream to write data to.
 * @param       p_j2k                           J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_update_rates(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut i: u32 = 0;
  let mut j: u32 = 0;
  let mut k: u32 = 0;
  let mut l_x0: i32 = 0;
  let mut l_y0: i32 = 0;
  let mut l_x1: i32 = 0;
  let mut l_y1: i32 = 0;
  let mut l_last_res: u32 = 0;
  /* preconditions */

  let l_cp = &mut p_j2k.m_cp;
  let l_image = p_j2k.m_private_image.as_deref().expect("m_private_image");
  let l_bits_empty = (8u32)
    .wrapping_mul(l_image.comps[0].dx)
    .wrapping_mul(l_image.comps[0].dy);
  let l_size_pixel = l_image.numcomps.wrapping_mul(l_image.comps[0].prec);
  let l_sot_remove = opj_stream_tell(p_stream) as f32 / l_cp.th.wrapping_mul(l_cp.tw) as f32;
  let l_tp_stride_func: fn(&Tcp) -> f32 = if l_cp.m_specific_param.m_enc.m_tp_on {
    opj_j2k_get_tp_stride
  } else {
    opj_j2k_get_default_stride
  };
  let mut tcp_idx = 0usize;
  i = 0_u32;
  while i < l_cp.th {
    j = 0_u32;
    while j < l_cp.tw {
      let l_offset = l_tp_stride_func(&l_cp.tcps[tcp_idx]) / l_cp.tcps[tcp_idx].numlayers as f32;
      /* 4 borders of the tile rescale on the image if necessary */
      l_x0 = opj_int_max(
        l_cp.tx0.wrapping_add(j.wrapping_mul(l_cp.tdx)) as i32,
        l_image.x0 as i32,
      );
      l_y0 = opj_int_max(
        l_cp.ty0.wrapping_add(i.wrapping_mul(l_cp.tdy)) as i32,
        l_image.y0 as i32,
      );
      l_x1 = opj_int_min(
        l_cp
          .tx0
          .wrapping_add(j.wrapping_add(1u32).wrapping_mul(l_cp.tdx)) as i32,
        l_image.x1 as i32,
      );
      l_y1 = opj_int_min(
        l_cp
          .ty0
          .wrapping_add(i.wrapping_add(1u32).wrapping_mul(l_cp.tdy)) as i32,
        l_image.y1 as i32,
      );
      /* Modification of the RATE >> */
      k = 0_u32;
      while k < l_cp.tcps[tcp_idx].numlayers {
        if l_cp.tcps[tcp_idx].rates[k as usize] > 0.0f32 {
          l_cp.tcps[tcp_idx].rates[k as usize] = (l_size_pixel as f64
            * (l_x1 - l_x0) as u32 as core::ffi::c_double
            * (l_y1 - l_y0) as u32 as core::ffi::c_double
            / (l_cp.tcps[tcp_idx].rates[k as usize] * l_bits_empty as f32) as core::ffi::c_double)
            as f32
            - l_offset
        }
        k += 1;
      }
      tcp_idx += 1;
      j += 1;
    }
    i += 1;
  }
  tcp_idx = 0usize;
  i = 0_u32;
  while i < l_cp.th {
    j = 0_u32;
    while j < l_cp.tw {
      if l_cp.tcps[tcp_idx].rates[0] > 0.0f32 {
        l_cp.tcps[tcp_idx].rates[0] -= l_sot_remove;
        if l_cp.tcps[tcp_idx].rates[0] < 30.0f32 {
          l_cp.tcps[tcp_idx].rates[0] = 30.0f32
        }
      }
      l_last_res = l_cp.tcps[tcp_idx].numlayers.wrapping_sub(1u32);
      k = 1_u32;
      while k < l_last_res {
        if l_cp.tcps[tcp_idx].rates[k as usize] > 0.0f32 {
          l_cp.tcps[tcp_idx].rates[k as usize] -= l_sot_remove;
          if l_cp.tcps[tcp_idx].rates[k as usize]
            < l_cp.tcps[tcp_idx].rates[k as usize - 1] + 10.0f32
          {
            l_cp.tcps[tcp_idx].rates[k as usize] =
              l_cp.tcps[tcp_idx].rates[k as usize - 1] + 20.0f32
          }
        }
        k += 1;
      }
      if l_cp.tcps[tcp_idx].rates[k as usize] > 0.0f32 {
        l_cp.tcps[tcp_idx].rates[k as usize] -= l_sot_remove + 2.0f32;
        if l_cp.tcps[tcp_idx].rates[k as usize] < l_cp.tcps[tcp_idx].rates[k as usize - 1] + 10.0f32
        {
          l_cp.tcps[tcp_idx].rates[k as usize] = l_cp.tcps[tcp_idx].rates[k as usize - 1] + 20.0f32
        }
      }
      tcp_idx += 1;
      j += 1;
    }
    i += 1;
  }
  opj_j2k_alloc_encoded_tile_data(p_j2k, p_manager)
}

/// Sizes the per-tile encoded-data scratch buffer (the C allocates it in
/// opj_j2k_update_rates; the parallel tile encoder needs it per worker).
pub(crate) fn opj_j2k_alloc_encoded_tile_data(
  p_j2k: &mut J2k,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let l_cp = &p_j2k.m_cp;
  let l_image = p_j2k.m_private_image.as_deref().expect("m_private_image");
  let mut l_tile_size = 0u64;
  let mut comp_idx = 0usize;
  let mut i = 0u32;
  while i < l_image.numcomps {
    l_tile_size += opj_uint_ceildiv(l_cp.tdx, l_image.comps[comp_idx].dx) as u64
      * opj_uint_ceildiv(l_cp.tdy, l_image.comps[comp_idx].dy) as u64
      * l_image.comps[comp_idx].prec as u64;
    comp_idx += 1;
    i += 1;
  }
  /* Where does this magic value come from? */
  /* This used to be 1.3 / 8, but with random data and very small code */
  /* block sizes, this is not enough. For example with */
  /* bin/test_tile_encoder 1 256 256 32 32 8 0 reversible_with_precinct.j2k 4 4 3 0 0 1 16 16 */
  /* Note: this does not account for the overhead of the */
  /* number of packets and number of code blocks in packets */
  l_tile_size = (l_tile_size as f64 * 1.4 / 8.0) as u64;

  /* Arbitrary amount to make the following work: */
  /* bin/test_tile_encoder 1 256 256 17 16 8 0 reversible_no_precinct.j2k 4 4 3 0 0 1 */
  l_tile_size += 500;

  l_tile_size += opj_j2k_get_specific_header_sizes(p_j2k) as u64;

  if l_tile_size > u32::MAX as u64 {
    l_tile_size = u32::MAX as u64;
  }

  /* the tile buffer can be up to 4 GB, so keep the C code's out-of-memory diagnostic
  rather than letting the allocator abort */
  let l_encoded_tile_data = &mut p_j2k.m_specific_param.m_encoder.m_encoded_tile_data;
  l_encoded_tile_data.clear();
  if l_encoded_tile_data
    .try_reserve_exact(l_tile_size as usize)
    .is_err()
  {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Not enough memory to allocate m_encoded_tile_data. {} MB required\n",
      l_tile_size.wrapping_div(1024u64).wrapping_div(1024u64) as u32,
    );
    return 0i32;
  }
  l_encoded_tile_data.resize(l_tile_size as usize, 0);
  if p_j2k.m_specific_param.m_encoder.m_tlm != 0 {
    p_j2k.m_specific_param.m_encoder.m_tlm_sot_offsets_buffer =
      vec![0u8; (6u32).wrapping_mul(p_j2k.m_specific_param.m_encoder.m_total_tile_parts) as usize];
    p_j2k.m_specific_param.m_encoder.m_tlm_sot_offsets_current = 0
  }
  1i32
}

/* *
 * Gets the offset of the header.
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_get_end_header(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */

  p_j2k
    .cstr_index
    .as_deref_mut()
    .expect("cstr_index")
    .main_head_end = opj_stream_tell(p_stream);
  1i32
}
/* *
 * Writes the CBD-MCT-MCC-MCO markers (Multi components transform)
 *
 * @param       p_stream                        the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager       the user event manager.
*/
fn opj_j2k_write_mct_data_group(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */

  if opj_j2k_write_cbd(p_j2k, p_stream, p_manager) == 0 {
    return 0i32;
  }
  // The write helpers borrow all of `p_j2k`, so clone the (tiny) record lists upfront and
  // release the borrow of `p_j2k.m_cp` before calling them.
  let tile = p_j2k.m_current_tile_number as usize;
  let mct_records = p_j2k.m_cp.tcps[tile].m_mct_records.clone();
  let mcc_records = p_j2k.m_cp.tcps[tile].m_mcc_records.clone();
  for record in &mct_records {
    if opj_j2k_write_mct_record(p_j2k, record, p_stream, p_manager) == 0 {
      return 0i32;
    }
  }
  for record in &mcc_records {
    if opj_j2k_write_mcc_record(p_j2k, record, &mct_records, p_stream, p_manager) == 0 {
      return 0i32;
    }
  }
  if opj_j2k_write_mco(p_j2k, p_stream, p_manager) == 0 {
    return 0i32;
  }
  1i32
}
/* *
 * Writes COC marker for each component.
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_write_all_coc(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut compno: u32 = 0;
  /* preconditions */

  compno = 1_u32;
  let numcomps = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  while compno < numcomps {
    /* cod is first component of first tile */
    if opj_j2k_compare_coc(p_j2k, 0_u32, compno) == 0
      && opj_j2k_write_coc(p_j2k, compno, p_stream, p_manager) == 0
    {
      return 0i32;
    }
    compno += 1;
  }
  1i32
}
/* *
 * Writes QCC marker for each component.
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_write_all_qcc(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut compno: u32 = 0;
  /* preconditions */

  compno = 1_u32;
  let numcomps = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  while compno < numcomps {
    /* qcd is first component of first tile */
    if opj_j2k_compare_qcc(p_j2k, 0_u32, compno) == 0
      && opj_j2k_write_qcc(p_j2k, compno, p_stream, p_manager) == 0
    {
      return 0i32;
    }
    compno += 1;
  }
  1i32
}
/* *
 * Writes regions of interests.
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_write_regions(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut compno: u32 = 0;
  /* preconditions */

  let numcomps = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  compno = 0_u32;
  while compno < numcomps {
    if p_j2k.m_cp.tcps[0].tccps[compno as usize].roishift != 0
      && opj_j2k_write_rgn(p_j2k, 0_u32, compno, numcomps, p_stream, p_manager) == 0
    {
      return 0i32;
    }
    compno += 1;
  }
  1i32
}
/* *
 * Writes EPC ????
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_write_epc(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */

  if let Some(l_cstr_index) = p_j2k.cstr_index.as_deref_mut() {
    l_cstr_index.codestream_size = opj_stream_tell(p_stream) as u64;
    /* UniPG>> */
    /* The following adjustment is done to adjust the codestream size */
    /* if SOD is not at 0 in the buffer. Useful in case of JP2, where */
    /* the first bunch of bytes is not in the codestream              */
    l_cstr_index.codestream_size -= l_cstr_index.main_head_start as u64;
    /* <<UniPG */
  }
  /* USE_JPWL */
  1i32
}
/* *
 * Reads an unknown marker
 *
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_stream                the stream object to read from.
 * @param output_marker
 * @param       p_manager               the user event manager.
 *
 * @return      true                    if the marker could be deduced.
*/
fn opj_j2k_read_unk(
  mut p_j2k: &mut J2k,
  mut p_stream: &mut Stream,
  mut output_marker: &mut J2KMarker,
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_unknown_marker: u32 = 0;
  let mut l_size_unk = 2_u32;
  let mut l_marker_handler = J2KMarker::UNK(0);
  /* preconditions*/

  event_msg!(p_manager, EVT_WARNING, "Unknown marker\n",);
  loop {
    /* Try to read 2 bytes (the next marker ID) from stream and copy them into the buffer*/
    if opj_stream_read_data(
      p_stream,
      &mut p_j2k.m_specific_param.m_decoder.m_header_data[..2],
      p_manager,
    ) != 2
    {
      event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
      return 0i32;
    }
    /* read 2 bytes as the new marker ID*/
    l_unknown_marker = opj_read_bytes(&p_j2k.m_specific_param.m_decoder.m_header_data, 2_u32);
    if l_unknown_marker < 0xff00u32 {
      continue;
    }
    /* Get the marker handler from the marker ID*/
    l_marker_handler = J2KMarker::from(l_unknown_marker);
    if p_j2k.m_specific_param.m_decoder.m_state & l_marker_handler.states() == J2KState::NONE {
      p_j2k.record_error_context(
        l_marker_handler.as_u32(),
        opj_stream_tell(p_stream).max(0) as u64,
      );
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Marker is not compliant with its position\n",
      );
      return 0i32;
    } else if !l_marker_handler.is_unknown() && l_marker_handler.has_handler() {
      /* Add the marker to the codestream index*/
      if l_marker_handler != J2KMarker::SOT {
        let mut res = opj_j2k_add_mhmarker(
          p_j2k.cstr_index.as_deref_mut().expect("cstr_index"),
          J2KMarker::UNK(0),
          (opj_stream_tell(p_stream) as u32).wrapping_sub(l_size_unk) as i64,
          l_size_unk,
        );
        if res == 0i32 {
          event_msg!(p_manager, EVT_ERROR, "Not enough memory to add mh marker\n",);
          return 0i32;
        }
      }
      break;
    /* next marker is known and well located */
    } else {
      l_size_unk = (l_size_unk as core::ffi::c_uint).wrapping_add(2u32)
    }
  }
  *output_marker = l_marker_handler;
  1i32
}
/* *
 * Writes the MCT marker (Multiple Component Transform)
 *
 * @param       p_j2k           J2K codec.
 * @param p_mct_record
 * @param       p_stream        the stream to write data to.
 * @param       p_manager       the user event manager.
*/
fn opj_j2k_write_mct_record(
  p_j2k: &mut J2k,
  p_mct_record: &MctData,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */
  /* Lmct */

  let l_mct_size: u32 = (10u32).wrapping_add(p_mct_record.m_data.len() as u32);
  let mut l_buf = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_buf.resize(l_mct_size as usize, 0);
  let mut pos: usize = 0;
  opj_write_bytes(&mut l_buf[pos..], J2KMarker::MCT.as_u32(), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], l_mct_size.wrapping_sub(2u32), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], 0_u32, 2_u32);
  pos += 2usize;
  /* only one marker atm */
  let l_tmp: u32 = p_mct_record.m_index & 0xffu32
    | (p_mct_record.m_array_type as core::ffi::c_uint) << 8i32
    | (p_mct_record.m_element_type as core::ffi::c_uint) << 10i32;
  opj_write_bytes(&mut l_buf[pos..], l_tmp, 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], 0_u32, 2_u32);
  pos += 2usize;
  l_buf[pos..].copy_from_slice(&p_mct_record.m_data);
  pos += p_mct_record.m_data.len();
  if opj_stream_write_data(p_stream, &l_buf[..l_mct_size as usize], p_manager)
    != l_mct_size as usize
  {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_buf;
  1i32
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
fn opj_j2k_read_mct(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  let mut l_tmp: u32 = 0;
  /* preconditions */

  let is_tph = p_j2k.m_specific_param.m_decoder.m_state == J2KState::TPH;
  let l_tcp = if is_tph {
    &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize]
  } else {
    p_j2k
      .m_specific_param
      .m_decoder
      .m_default_tcp
      .as_deref_mut()
      .expect("decoder m_default_tcp is always set")
  };
  if p_header_size < 2u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading MCT marker\n",);
    return 0i32;
  }
  /* first marker */
  l_tmp = opj_read_bytes(&header_data[cursor..], 2_u32); /* Zmct */
  cursor += 2usize;
  if l_tmp != 0u32 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "Cannot take in charge mct data within multiple MCT records\n",
    );
    return 1i32;
  }
  if p_header_size <= 6u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading MCT marker\n",);
    return 0i32;
  }
  /* Imct -> no need for other values, take the first, type is double with decorrelation x0000 1101 0000 0000*/
  l_tmp = opj_read_bytes(&header_data[cursor..], 2_u32); /* Imct */
  cursor += 2usize;
  let l_indix: u32 = l_tmp & 0xffu32;
  /* find an existing record with this index, or append a fresh one */
  let mut l_mct_idx: usize = 0;
  while l_mct_idx < l_tcp.m_mct_records.len() {
    if l_tcp.m_mct_records[l_mct_idx].m_index == l_indix {
      break;
    }
    l_mct_idx += 1;
  }
  /* NOT FOUND */
  if l_mct_idx == l_tcp.m_mct_records.len() {
    l_tcp.m_mct_records.push(Default::default());
  }
  let l_mct_data = &mut l_tcp.m_mct_records[l_mct_idx];
  l_mct_data.m_index = l_indix;
  l_mct_data.m_array_type = (l_tmp >> 8i32 & 3u32) as J2K_MCT_ARRAY_TYPE;
  l_mct_data.m_element_type = MCTElementType::new(l_tmp >> 10i32 & 3u32);
  l_tmp = opj_read_bytes(&header_data[cursor..], 2_u32); /* Ymct */
  cursor += 2usize;
  if l_tmp != 0u32 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "Cannot take in charge multiple MCT markers\n",
    );
    return 1i32;
  }
  p_header_size = p_header_size.wrapping_sub(6u32);
  l_mct_data.m_data.clear();
  /* C: memcpy(m_data, p_header_data + 6, p_header_size - 6) — the payload starts after the
   * Zmct/Imct/Ymct header fields. */
  l_mct_data
    .m_data
    .extend_from_slice(&header_data[cursor..cursor + p_header_size as usize]);
  1i32
}

/* *
 * Writes the MCC marker (Multiple Component Collection)
 *
 * @param       p_j2k                   J2K codec.
 * @param p_mcc_record
 * @param       p_stream                the stream to write data to.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_write_mcc_record(
  p_j2k: &mut J2k,
  p_mcc_record: &SimpleMccData,
  mct_records: &[MctData],
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut i: u32 = 0;
  let mut l_tmcc: u32 = 0;
  /* preconditions */
  /* Lmcc */

  let (l_nb_bytes_for_comp, l_mask): (u32, u32) = if p_mcc_record.m_nb_comps > 255u32 {
    (2_u32, 0x8000_u32)
  } else {
    (1_u32, 0_u32)
  };
  let l_mcc_size: u32 = p_mcc_record
    .m_nb_comps
    .wrapping_mul(2u32)
    .wrapping_mul(l_nb_bytes_for_comp)
    .wrapping_add(19u32);
  let mut l_buf = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_buf.resize(l_mcc_size as usize, 0);
  let mut pos: usize = 0;
  opj_write_bytes(&mut l_buf[pos..], J2KMarker::MCC.as_u32(), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], l_mcc_size.wrapping_sub(2u32), 2_u32);
  pos += 2usize;
  /* first marker */
  opj_write_bytes(&mut l_buf[pos..], 0_u32, 2_u32); /* Zmcc */
  pos += 2usize; /* Imcc -> no need for other values, take the first */
  opj_write_bytes(&mut l_buf[pos..], p_mcc_record.m_index, 1_u32);
  pos += 1usize;
  /* only one marker atm */
  opj_write_bytes(&mut l_buf[pos..], 0_u32, 2_u32); /* Ymcc */
  pos += 2usize; /* Qmcc -> number of collections -> 1 */
  opj_write_bytes(&mut l_buf[pos..], 1_u32, 2_u32); /* Xmcci type of component transformation -> array based decorrelation */
  pos += 2usize; /* Nmcci number of input components involved and size for each component offset = 8 bits */
  opj_write_bytes(&mut l_buf[pos..], 0x1_u32, 1_u32); /* Cmccij Component offset*/
  pos += 1usize; /* Mmcci number of output components involved and size for each component offset = 8 bits */
  opj_write_bytes(&mut l_buf[pos..], p_mcc_record.m_nb_comps | l_mask, 2_u32); /* Wmccij Component offset*/
  pos += 2usize; /* Tmcci : use MCT defined as number 1 and irreversible array based. */
  i = 0_u32;
  while i < p_mcc_record.m_nb_comps {
    opj_write_bytes(&mut l_buf[pos..], i, l_nb_bytes_for_comp);
    pos += l_nb_bytes_for_comp as usize;
    i += 1;
  }
  opj_write_bytes(&mut l_buf[pos..], p_mcc_record.m_nb_comps | l_mask, 2_u32);
  pos += 2usize;
  i = 0_u32;
  while i < p_mcc_record.m_nb_comps {
    opj_write_bytes(&mut l_buf[pos..], i, l_nb_bytes_for_comp);
    pos += l_nb_bytes_for_comp as usize;
    i += 1;
  }
  l_tmcc = ((!p_mcc_record.m_is_irreversible) as core::ffi::c_uint & 1u32) << 16i32;
  if let Some(deco_idx) = p_mcc_record.m_decorrelation_array {
    l_tmcc |= mct_records[deco_idx].m_index
  }
  if let Some(offset_idx) = p_mcc_record.m_offset_array {
    l_tmcc |= mct_records[offset_idx].m_index << 8i32
  }
  opj_write_bytes(&mut l_buf[pos..], l_tmcc, 3_u32);
  pos += 3usize;
  if opj_stream_write_data(p_stream, &l_buf[..l_mcc_size as usize], p_manager)
    != l_mcc_size as usize
  {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_buf;
  1i32
}

/* *
 * Reads a MCC marker (Multiple Component Collection)
 *
 * @param       p_header_data   the data contained in the MCC box.
 * @param       p_j2k                   the jpeg2000 codec.
 * @param       p_header_size   the size of the data contained in the MCC marker.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_read_mcc(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  let mut i: u32 = 0;
  let mut j: u32 = 0;
  let mut l_tmp: u32 = 0;
  let mut l_indix: u32 = 0;
  let mut l_nb_collections: u32 = 0;
  let mut l_nb_comps: u32 = 0;
  let mut l_nb_bytes_by_comp: u32 = 0;
  /* preconditions */

  let is_tph = p_j2k.m_specific_param.m_decoder.m_state == J2KState::TPH;
  let l_tcp = if is_tph {
    &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize]
  } else {
    p_j2k
      .m_specific_param
      .m_decoder
      .m_default_tcp
      .as_deref_mut()
      .expect("decoder m_default_tcp is always set")
  };
  if p_header_size < 2u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading MCC marker\n",);
    return 0i32;
  }
  /* first marker */
  l_tmp = opj_read_bytes(&header_data[cursor..], 2_u32); /* Zmcc */
  cursor += 2usize; /* Imcc -> no need for other values, take the first */
  if l_tmp != 0u32 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "Cannot take in charge multiple data spanning\n",
    );
    return 1i32;
  }
  if p_header_size < 7u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading MCC marker\n",);
    return 0i32;
  }
  l_indix = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  /* find an existing record with this index, or append a fresh one */
  let mut l_mcc_idx: usize = 0;
  while l_mcc_idx < l_tcp.m_mcc_records.len() {
    if l_tcp.m_mcc_records[l_mcc_idx].m_index == l_indix {
      break;
    }
    l_mcc_idx += 1;
  }
  /* * NOT FOUND */
  if l_mcc_idx == l_tcp.m_mcc_records.len() {
    l_tcp.m_mcc_records.push(Default::default());
  }
  let l_mcc_record = &mut l_tcp.m_mcc_records[l_mcc_idx];
  l_mcc_record.m_index = l_indix;
  /* only one marker atm */
  l_tmp = opj_read_bytes(&header_data[cursor..], 2_u32); /* Ymcc */
  cursor += 2usize; /* Qmcc -> number of collections -> 1 */
  if l_tmp != 0u32 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "Cannot take in charge multiple data spanning\n",
    ); /* Xmcci type of component transformation -> array based decorrelation */
    return 1i32;
  } /* Cmccij Component offset*/
  l_nb_collections = opj_read_bytes(&header_data[cursor..], 2_u32); /* Wmccij Component offset*/
  cursor += 2usize; /* Wmccij Component offset*/
  if l_nb_collections > 1u32 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "Cannot take in charge multiple collections\n",
    );
    return 1i32;
  }
  p_header_size = (p_header_size as core::ffi::c_uint).wrapping_sub(7u32);
  i = 0_u32;
  while i < l_nb_collections {
    if p_header_size < 3u32 {
      event_msg!(p_manager, EVT_ERROR, "Error reading MCC marker\n",);
      return 0i32;
    }
    l_tmp = opj_read_bytes(&header_data[cursor..], 1_u32);
    cursor += 1usize;
    if l_tmp != 1u32 {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "Cannot take in charge collections other than array decorrelation\n",
      );
      return 1i32;
    }
    l_nb_comps = opj_read_bytes(&header_data[cursor..], 2_u32);
    cursor += 2usize;
    p_header_size = (p_header_size as core::ffi::c_uint).wrapping_sub(3u32);
    l_nb_bytes_by_comp = (1u32).wrapping_add(l_nb_comps >> 15i32);
    l_mcc_record.m_nb_comps = l_nb_comps & 0x7fffu32;
    if p_header_size
      < l_nb_bytes_by_comp
        .wrapping_mul(l_mcc_record.m_nb_comps)
        .wrapping_add(2u32)
    {
      event_msg!(p_manager, EVT_ERROR, "Error reading MCC marker\n",);
      return 0i32;
    }
    p_header_size = (p_header_size as core::ffi::c_uint).wrapping_sub(
      l_nb_bytes_by_comp
        .wrapping_mul(l_mcc_record.m_nb_comps)
        .wrapping_add(2u32),
    );
    j = 0_u32;
    while j < l_mcc_record.m_nb_comps {
      l_tmp = opj_read_bytes(&header_data[cursor..], l_nb_bytes_by_comp);
      cursor += l_nb_bytes_by_comp as isize as usize;
      if l_tmp != j {
        event_msg!(
          p_manager,
          EVT_WARNING,
          "Cannot take in charge collections with indix shuffle\n",
        );
        return 1i32;
      }
      j += 1;
    }
    l_nb_comps = opj_read_bytes(&header_data[cursor..], 2_u32);
    cursor += 2usize;
    l_nb_bytes_by_comp = (1u32).wrapping_add(l_nb_comps >> 15i32);
    l_nb_comps &= 0x7fffu32;
    if l_nb_comps != l_mcc_record.m_nb_comps {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "Cannot take in charge collections without same number of indixes\n",
      );
      return 1i32;
    }
    if p_header_size
      < l_nb_bytes_by_comp
        .wrapping_mul(l_mcc_record.m_nb_comps)
        .wrapping_add(3u32)
    {
      event_msg!(p_manager, EVT_ERROR, "Error reading MCC marker\n",);
      return 0i32;
    }
    p_header_size = (p_header_size as core::ffi::c_uint).wrapping_sub(
      l_nb_bytes_by_comp
        .wrapping_mul(l_mcc_record.m_nb_comps)
        .wrapping_add(3u32),
    );
    j = 0_u32;
    while j < l_mcc_record.m_nb_comps {
      l_tmp = opj_read_bytes(&header_data[cursor..], l_nb_bytes_by_comp);
      cursor += l_nb_bytes_by_comp as isize as usize;
      if l_tmp != j {
        event_msg!(
          p_manager,
          EVT_WARNING,
          "Cannot take in charge collections with indix shuffle\n",
        );
        return 1i32;
      }
      j += 1;
    }
    l_tmp = opj_read_bytes(&header_data[cursor..], 3_u32);
    cursor += 3usize;
    l_mcc_record.m_is_irreversible = l_tmp >> 16i32 & 1u32 == 0;
    l_mcc_record.m_decorrelation_array = None;
    l_mcc_record.m_offset_array = None;
    l_indix = l_tmp & 0xffu32;
    if l_indix != 0u32 {
      match l_tcp
        .m_mct_records
        .iter()
        .position(|rec| rec.m_index == l_indix)
      {
        Some(pos) => l_mcc_record.m_decorrelation_array = Some(pos),
        None => {
          event_msg!(p_manager, EVT_ERROR, "Error reading MCC marker\n",);
          return 0i32;
        }
      }
    }
    l_indix = l_tmp >> 8i32 & 0xffu32;
    if l_indix != 0u32 {
      match l_tcp
        .m_mct_records
        .iter()
        .position(|rec| rec.m_index == l_indix)
      {
        Some(pos) => l_mcc_record.m_offset_array = Some(pos),
        None => {
          event_msg!(p_manager, EVT_ERROR, "Error reading MCC marker\n",);
          return 0i32;
        }
      }
    }
    i += 1;
  }
  if p_header_size != 0u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading MCC marker\n",);
    return 0i32;
  }
  1i32
}
/* *
 * Writes the MCO marker (Multiple component transformation ordering)
 *
 * @param       p_stream                                the stream to write data to.
 * @param       p_j2k                           J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_write_mco(p_j2k: &mut J2k, p_stream: &mut Stream, p_manager: &mut EventMgr) -> OPJ_BOOL {
  /* preconditions */
  /* Lmco */
  /* Imco -> use the mcc indicated by 1*/
  let l_tcp = &p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize];
  let l_mco_size: u32 = (5u32).wrapping_add(l_tcp.m_mcc_records.len() as u32);
  let mut l_buf = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_buf.resize(l_mco_size as usize, 0);
  let mut pos: usize = 0;
  opj_write_bytes(&mut l_buf[pos..], J2KMarker::MCO.as_u32(), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], l_mco_size.wrapping_sub(2u32), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], l_tcp.m_mcc_records.len() as u32, 1_u32);
  pos += 1usize;
  for l_mcc_record in &l_tcp.m_mcc_records {
    opj_write_bytes(&mut l_buf[pos..], l_mcc_record.m_index, 1_u32);
    pos += 1usize;
  }
  if opj_stream_write_data(p_stream, &l_buf[..l_mco_size as usize], p_manager)
    != l_mco_size as usize
  {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_buf;
  1i32
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
fn opj_j2k_read_mco(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  let mut l_tmp: u32 = 0;
  let mut i: u32 = 0;
  /* preconditions */

  let l_image_numcomps = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  let is_tph = p_j2k.m_specific_param.m_decoder.m_state == J2KState::TPH;
  let l_tcp = if is_tph {
    &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize]
  } else {
    p_j2k
      .m_specific_param
      .m_decoder
      .m_default_tcp
      .as_deref_mut()
      .expect("decoder m_default_tcp is always set")
  };
  if p_header_size < 1u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading MCO marker\n",);
    return 0i32;
  }
  let l_nb_stages: u32 = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  if l_nb_stages > 1u32 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "Cannot take in charge multiple transformation stages.\n",
    );
    return 1i32;
  }
  if p_header_size != l_nb_stages.wrapping_add(1u32) {
    event_msg!(p_manager, EVT_WARNING, "Error reading MCO marker\n",);
    return 0i32;
  }
  if l_nb_stages > 0u32 {
    i = 0_u32;
    while i < l_image_numcomps {
      l_tcp.tccps[i as usize].m_dc_level_shift = 0i32;
      i += 1;
    }
  }
  l_tcp.m_mct_decoding_matrix.clear();
  i = 0_u32;
  while i < l_nb_stages {
    l_tmp = opj_read_bytes(&header_data[cursor..], 1_u32);
    cursor += 1usize;
    if opj_j2k_add_mct(
      l_tcp,
      p_j2k
        .m_private_image
        .as_deref_mut()
        .expect("m_private_image"),
      l_tmp,
    ) == 0
    {
      return 0i32;
    }
    i += 1;
  }
  1i32
}

fn opj_j2k_add_mct(p_tcp: &mut Tcp, p_image: &mut CoreImage, p_index: u32) -> OPJ_BOOL {
  let mut l_data_size: u32 = 0;
  let mut l_nb_elem: u32 = 0;
  /* preconditions */
  let p_tcp_ref = &mut *p_tcp;
  /* find the MCC record with this index */
  let Some(l_mcc_idx) = p_tcp_ref
    .m_mcc_records
    .iter()
    .position(|rec| rec.m_index == p_index)
  else {
    /* element discarded */
    return 1i32;
  };
  if p_tcp_ref.m_mcc_records[l_mcc_idx].m_nb_comps != p_image.numcomps {
    /* do not support number of comps != image */
    return 1i32;
  }
  let l_deco_idx = p_tcp_ref.m_mcc_records[l_mcc_idx].m_decorrelation_array;
  if let Some(l_deco_idx) = l_deco_idx {
    let l_deco_array = &p_tcp_ref.m_mct_records[l_deco_idx];
    l_data_size = l_deco_array
      .m_element_type
      .size()
      .wrapping_mul(p_image.numcomps)
      .wrapping_mul(p_image.numcomps);
    if l_deco_array.m_data.len() as u32 != l_data_size {
      return 0i32;
    }
    l_nb_elem = p_image.numcomps.wrapping_mul(p_image.numcomps);
    p_tcp_ref.m_mct_decoding_matrix.clear();
    p_tcp_ref
      .m_mct_decoding_matrix
      .resize(l_nb_elem as usize, 0.0);
    l_deco_array.m_element_type.read_to_float(
      l_deco_array.m_data.as_slice(),
      &mut p_tcp_ref.m_mct_decoding_matrix,
    );
  }
  let l_offset_idx = p_tcp_ref.m_mcc_records[l_mcc_idx].m_offset_array;
  if let Some(l_offset_idx) = l_offset_idx {
    let l_offset_array = &p_tcp_ref.m_mct_records[l_offset_idx];
    l_data_size = l_offset_array
      .m_element_type
      .size()
      .wrapping_mul(p_image.numcomps);
    if l_offset_array.m_data.len() as u32 != l_data_size {
      return 0i32;
    }
    l_nb_elem = p_image.numcomps;
    let mut l_offset_data: Vec<u32> = vec![0; l_nb_elem as usize];
    l_offset_array
      .m_element_type
      .read_to_int32(l_offset_array.m_data.as_slice(), &mut l_offset_data);
    for (tccp, &offset) in p_tcp_ref.tccps.iter_mut().zip(l_offset_data.iter()) {
      tccp.m_dc_level_shift = offset as i32;
    }
  }
  1i32
}
/* *
 * Writes the CBD marker (Component bit depth definition)
 *
 * @param       p_stream                                the stream to write data to.
 * @param       p_j2k                           J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_write_cbd(p_j2k: &mut J2k, p_stream: &mut Stream, p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut i: u32 = 0;
  /* preconditions */
  /* L_CBD */
  /* Component bit depth */
  let l_image = p_j2k.m_private_image.as_deref().expect("m_private_image");
  let l_cbd_size: u32 = (6u32).wrapping_add(l_image.numcomps);
  let mut l_buf = core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_header_tile_data);
  l_buf.resize(l_cbd_size as usize, 0);
  let mut pos: usize = 0;
  opj_write_bytes(&mut l_buf[pos..], J2KMarker::CBD.as_u32(), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], l_cbd_size.wrapping_sub(2u32), 2_u32);
  pos += 2usize;
  opj_write_bytes(&mut l_buf[pos..], l_image.numcomps, 2_u32);
  pos += 2usize;
  i = 0_u32;
  while i < l_image.numcomps {
    let comp = &l_image.comps[i as usize];
    opj_write_bytes(
      &mut l_buf[pos..],
      comp.sgnd << 7i32 | comp.prec.wrapping_sub(1u32),
      1_u32,
    );
    pos += 1usize;
    i += 1;
  }
  if opj_stream_write_data(p_stream, &l_buf[..l_cbd_size as usize], p_manager)
    != l_cbd_size as usize
  {
    return 0i32;
  }
  p_j2k.m_specific_param.m_encoder.m_header_tile_data = l_buf;
  1i32
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
fn opj_j2k_read_cbd(p_j2k: &mut J2k, header_data: &[u8], p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut p_header_size = header_data.len() as u32;
  let mut cursor: usize = 0;
  let mut l_comp_def: u32 = 0;
  let mut i: u32 = 0;
  /* preconditions */
  /* Component bit depth */

  let l_image = p_j2k
    .m_private_image
    .as_deref_mut()
    .expect("m_private_image");
  let l_num_comp: u32 = l_image.numcomps;
  if p_header_size != l_image.numcomps.wrapping_add(2u32) {
    event_msg!(p_manager, EVT_ERROR, "Crror reading CBD marker\n",);
    return 0i32;
  }
  let l_nb_comp: u32 = opj_read_bytes(&header_data[cursor..], 2_u32);
  cursor += 2usize;
  if l_nb_comp != l_num_comp {
    event_msg!(p_manager, EVT_ERROR, "Crror reading CBD marker\n",);
    return 0i32;
  }
  i = 0_u32;
  while i < l_num_comp {
    l_comp_def = opj_read_bytes(&header_data[cursor..], 1_u32);
    cursor += 1usize;
    let comp = &mut l_image.comps[i as usize];
    comp.sgnd = l_comp_def >> 7i32 & 1u32;
    comp.prec = (l_comp_def & 0x7fu32).wrapping_add(1u32);
    if comp.prec > 31u32 {
      event_msg!(p_manager, EVT_ERROR,
                        "Invalid values for comp = {} : prec={} (should be between 1 and 38 according to the JPEG2000 norm. OpenJpeg only supports up to 31)\n", i,
                        comp.prec);
      return 0i32;
    }
    i += 1;
  }
  1i32
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
fn opj_j2k_read_cap(
  mut _p_j2k: &mut J2k,
  mut _header_data: &[u8],
  mut _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */

  1i32
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
fn opj_j2k_read_cpf(
  mut _p_j2k: &mut J2k,
  mut _header_data: &[u8],
  mut _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */

  1i32
}

/* ----------------------------------------------------------------------- */
/* J2K / JPT decoder interface                                             */
/* ----------------------------------------------------------------------- */
pub(crate) fn opj_j2k_setup_decoder(j2k: &mut J2k, settings: &crate::codec::DecoderSettings) {
  j2k.m_cp.m_specific_param.m_dec.m_layer = settings.layers;
  j2k.m_cp.m_specific_param.m_dec.m_reduce = settings.reduce;
  j2k.dump_state = if settings.dump_state { 2 } else { 0 };
}

pub(crate) fn opj_j2k_decoder_set_strict_mode(mut j2k: &mut J2k, mut strict: OPJ_BOOL) {
  j2k.m_cp.strict = strict;
  if strict != 0 {
    /* The nb-tile-parts correction hack is not applied in strict mode
    (upstream #1560) */
    j2k
      .m_specific_param
      .m_decoder
      .m_nb_tile_parts_correction_checked = true
  }
}

pub(crate) fn opj_j2k_set_threads(_j2k: &mut J2k, num_threads: u32) -> OPJ_BOOL {
  /* The C keeps a per-codec thread pool; the Rust port uses the
  process-wide rayon pool, sized on the first successful call.  Mirror the
  C's failure cases: no thread support and out-of-range counts return
  FALSE (the C's "cannot re-set after the tcd exists" nuance does not
  apply - the port's pool is process-wide by design, see doc/differences). */
  if crate::has_thread_support() == 0 || num_threads > i32::MAX as u32 {
    return 0i32;
  }
  crate::par::configure_pool(num_threads as usize);
  1i32
}

/* ----------------------------------------------------------------------- */
/* J2K encoder interface                                                       */
/* ----------------------------------------------------------------------- */
pub(crate) fn opj_j2k_create_compress() -> Option<J2k> {
  let mut l_j2k = J2k::new(0);
  l_j2k.m_cp.m_is_decoder = false;
  l_j2k.m_specific_param.m_encoder.m_header_tile_data = vec![0u8; 1000];
  Some(l_j2k)
}

fn opj_j2k_initialise_4k_poc(poc: &mut [opj_poc_t], numres: core::ffi::c_int) -> core::ffi::c_int {
  poc[0].tile = 1_u32;
  poc[0].resno0 = 0_u32;
  poc[0].compno0 = 0_u32;
  poc[0].layno1 = 1_u32;
  poc[0].resno1 = (numres - 1i32) as u32;
  poc[0].compno1 = 3_u32;
  poc[0].prg1 = OPJ_CPRL;
  poc[1].tile = 1_u32;
  poc[1].resno0 = (numres - 1i32) as u32;
  poc[1].compno0 = 0_u32;
  poc[1].layno1 = 1_u32;
  poc[1].resno1 = numres as u32;
  poc[1].compno1 = 3_u32;
  poc[1].prg1 = OPJ_CPRL;
  2i32
}
fn opj_j2k_set_cinema_parameters(
  mut parameters: &mut EncoderParameters,
  image: &mut CoreImage,
  mut p_manager: &mut EventMgr,
) {
  let comps = &image.comps;
  /* Configure cinema parameters */
  let mut i: core::ffi::c_int = 0;
  /* No tiling */
  parameters.tile_size_on = 0i32;
  parameters.cp_tdx = 1i32;
  parameters.cp_tdy = 1i32;
  /* One tile part for each component */
  parameters.tp_flag = 'C' as i32 as core::ffi::c_char;
  parameters.tp_on = 1 as core::ffi::c_char;
  /* Tile and Image shall be at (0,0) */
  parameters.cp_tx0 = 0i32;
  parameters.cp_ty0 = 0i32;
  parameters.image_offset_x0 = 0i32;
  parameters.image_offset_y0 = 0i32;
  /* Codeblock size= 32*32 */
  parameters.cblockw_init = 32i32;
  parameters.cblockh_init = 32i32;
  /* Codeblock style: no mode switch enabled */
  parameters.mode = 0i32;
  /* No ROI */
  parameters.roi_compno = -(1i32);
  /* No subsampling */
  parameters.subsampling_dx = 1i32;
  parameters.subsampling_dy = 1i32;
  /* 9-7 transform */
  parameters.irreversible = 1i32;
  /* Number of layers */
  if parameters.tcp_numlayers > 1i32 {
    event_msg!(p_manager, EVT_WARNING,
                      "JPEG 2000 Profile-3 and 4 (2k/4k dc profile) requires:\n1 single quality layer-> Number of layers forced to 1 (rather than {})\n-> Rate of the last layer ({:.1}) will be used",
                      parameters.tcp_numlayers,
                      parameters.tcp_rates[(parameters.tcp_numlayers -
                                                   1i32) as usize]
                          as core::ffi::c_double);
    parameters.tcp_rates[0_usize] =
      parameters.tcp_rates[(parameters.tcp_numlayers - 1i32) as usize];
    parameters.tcp_numlayers = 1i32
  }
  /* Resolution levels */
  match parameters.rsiz as core::ffi::c_int {
    3 => {
      if parameters.numresolution > 6i32 {
        event_msg!(p_manager, EVT_WARNING,
                              "JPEG 2000 Profile-3 (2k dc profile) requires:\nNumber of decomposition levels <= 5\n-> Number of decomposition levels forced to 5 (rather than {})\n",
                              parameters.numresolution + 1i32);
        parameters.numresolution = 6i32
      }
    }
    4 => {
      if parameters.numresolution < 2i32 {
        event_msg!(p_manager, EVT_WARNING,
                              "JPEG 2000 Profile-4 (4k dc profile) requires:\nNumber of decomposition levels >= 1 && <= 6\n-> Number of decomposition levels forced to 1 (rather than {})\n",
                              parameters.numresolution + 1i32);
        parameters.numresolution = 1i32
      } else if parameters.numresolution > 7i32 {
        event_msg!(p_manager, EVT_WARNING,
                              "JPEG 2000 Profile-4 (4k dc profile) requires:\nNumber of decomposition levels >= 1 && <= 6\n-> Number of decomposition levels forced to 6 (rather than {})\n",
                              parameters.numresolution + 1i32);
        parameters.numresolution = 7i32
      }
    }
    _ => {}
  }
  /* Precincts */
  parameters.csty |= 0x1i32;
  if parameters.numresolution == 1i32 {
    parameters.res_spec = 1i32;
    parameters.prcw_init[0_usize] = 128i32;
    parameters.prch_init[0_usize] = 128i32
  } else {
    parameters.res_spec = parameters.numresolution - 1i32;
    i = 0i32;
    while i < parameters.res_spec {
      parameters.prcw_init[i as usize] = 256i32;
      parameters.prch_init[i as usize] = 256i32;
      i += 1
    }
  }
  /* The progression order shall be CPRL */
  parameters.prog_order = OPJ_CPRL;
  /* Progression order changes for 4K, disallowed for 2K */
  if parameters.rsiz as core::ffi::c_int == 0x4i32 {
    parameters.numpocs =
      opj_j2k_initialise_4k_poc(&mut parameters.pocs[..2], parameters.numresolution) as u32
  } else {
    parameters.numpocs = 0_u32
  }
  /* Limited bit-rate */
  parameters.cp_disto_alloc = 1i32;
  if parameters.max_cs_size <= 0i32 {
    /* No rate has been introduced, 24 fps is assumed */
    parameters.max_cs_size = 1302083i32;
    event_msg!(p_manager, EVT_WARNING,
                      "JPEG 2000 Profile-3 and 4 (2k/4k dc profile) requires:\nMaximum 1302083 compressed bytes @ 24fps\nAs no rate has been given, this limit will be used.\n");
  } else if parameters.max_cs_size > 1302083i32 {
    event_msg!(p_manager, EVT_WARNING,
                      "JPEG 2000 Profile-3 and 4 (2k/4k dc profile) requires:\nMaximum 1302083 compressed bytes @ 24fps\n-> Specified rate exceeds this limit. Rate will be forced to 1302083 bytes.\n");
    parameters.max_cs_size = 1302083i32
  }
  if parameters.max_comp_size <= 0i32 {
    /* No rate has been introduced, 24 fps is assumed */
    parameters.max_comp_size = 1041666i32;
    event_msg!(p_manager, EVT_WARNING,
                      "JPEG 2000 Profile-3 and 4 (2k/4k dc profile) requires:\nMaximum 1041666 compressed bytes @ 24fps\nAs no rate has been given, this limit will be used.\n");
  } else if parameters.max_comp_size > 1041666i32 {
    event_msg!(p_manager, EVT_WARNING,
                      "JPEG 2000 Profile-3 and 4 (2k/4k dc profile) requires:\nMaximum 1041666 compressed bytes @ 24fps\n-> Specified rate exceeds this limit. Rate will be forced to 1041666 bytes.\n");
    parameters.max_comp_size = 1041666i32
  }
  parameters.tcp_rates[0_usize] = image
    .numcomps
    .wrapping_mul(comps[0].w)
    .wrapping_mul(comps[0].h)
    .wrapping_mul(comps[0].prec) as f32
    / (parameters.max_cs_size as u32)
      .wrapping_mul(8u32)
      .wrapping_mul(comps[0].dx)
      .wrapping_mul(comps[0].dy) as f32;
}
fn opj_j2k_is_cinema_compliant(
  mut image: &mut CoreImage,
  mut rsiz: u16,
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut i: u32 = 0;
  let comps = &image.comps;
  /* Number of components */
  if image.numcomps != 3u32 {
    event_msg!(p_manager, EVT_WARNING,
                      "JPEG 2000 Profile-3 (2k dc profile) requires:\n3 components-> Number of components of input image ({}) is not compliant\n-> Non-profile-3 codestream will be generated\n",
                      image.numcomps);
    return 0i32;
  }
  /* Bitdepth */
  i = 0_u32;
  while i < image.numcomps {
    if (comps[i as usize].prec != 12u32) as core::ffi::c_uint | comps[i as usize].sgnd != 0 {
      let tmp_str = if comps[i as usize].sgnd != 0 {
        "signed"
      } else {
        "unsigned"
      };
      event_msg!(p_manager, EVT_WARNING,
                          "JPEG 2000 Profile-3 (2k dc profile) requires:\nPrecision of each component shall be 12 bits unsigned-> At least component {} of input image ({} bits, {}) is not compliant\n-> Non-profile-3 codestream will be generated\n", i,
                          comps[i as usize].prec, tmp_str);
      return 0i32;
    }
    i += 1;
  }
  /* Image size */
  match rsiz as core::ffi::c_int {
    3 => {
      if (comps[0].w > 2048u32) as core::ffi::c_int | (comps[0].h > 1080u32) as core::ffi::c_int
        != 0
      {
        event_msg!(p_manager, EVT_WARNING,
                              "JPEG 2000 Profile-3 (2k dc profile) requires:\nwidth <= 2048 and height <= 1080\n-> Input image size {} x {} is not compliant\n-> Non-profile-3 codestream will be generated\n",
                              comps[0].w,
                              comps[0].h);
        return 0i32;
      }
    }
    4 if comps[0].w > 4096u32 || comps[0].h > 2160u32 => {
      event_msg!(p_manager, EVT_WARNING,
                            "JPEG 2000 Profile-4 (4k dc profile) requires:\nwidth <= 4096 and height <= 2160\n-> Image size {} x {} is not compliant\n-> Non-profile-4 codestream will be generated\n",
                            comps[0].w,
                            comps[0].h);
      return 0i32;
    }
    4 => {}
    _ => {}
  }
  1i32
}
fn opj_j2k_get_imf_max_nl(
  mut parameters: &mut EncoderParameters,
  mut image: &mut CoreImage,
) -> core::ffi::c_int {
  /* Decomposition levels */
  let rsiz = parameters.rsiz;
  let profile = (rsiz as core::ffi::c_int & 0xff00i32) as u16;
  let x_tsiz = if parameters.tile_size_on != 0 {
    parameters.cp_tdx as u32
  } else {
    image.x1
  };
  match profile as core::ffi::c_int {
    1024 => return 5i32,
    1280 => return 6i32,
    1536 => return 7i32,
    1792 => {
      if x_tsiz >= 2048u32 {
        return 5i32;
      } else if x_tsiz >= 1024u32 {
        return 4i32;
      }
    }
    2048 => {
      if x_tsiz >= 4096u32 {
        return 6i32;
      } else if x_tsiz >= 2048u32 {
        return 5i32;
      } else if x_tsiz >= 1024u32 {
        return 4i32;
      }
    }
    2304 => {
      if x_tsiz >= 8192u32 {
        return 7i32;
      } else if x_tsiz >= 4096u32 {
        return 6i32;
      } else if x_tsiz >= 2048u32 {
        return 5i32;
      } else if x_tsiz >= 1024u32 {
        return 4i32;
      }
    }
    _ => {}
  }
  -(1i32)
}
fn opj_j2k_set_imf_parameters(
  mut parameters: &mut EncoderParameters,
  mut image: &mut CoreImage,
  mut _p_manager: &mut EventMgr,
) {
  let rsiz = parameters.rsiz;
  let profile = (rsiz as core::ffi::c_int & 0xff00i32) as u16;
  /* Override defaults set by opj_set_default_encoder_parameters */
  if parameters.cblockw_init == 64i32 && parameters.cblockh_init == 64i32 {
    parameters.cblockw_init = 32i32;
    parameters.cblockh_init = 32i32
  }
  /* One tile part for each component */
  parameters.tp_flag = 'C' as i32 as core::ffi::c_char;
  parameters.tp_on = 1 as core::ffi::c_char;
  if parameters.prog_order as core::ffi::c_int == OPJ_LRCP as core::ffi::c_int {
    parameters.prog_order = OPJ_CPRL
  }
  if profile as core::ffi::c_int == 0x400i32
    || profile as core::ffi::c_int == 0x500i32
    || profile as core::ffi::c_int == 0x600i32
  {
    /* 9-7 transform */
    parameters.irreversible = 1i32
  }
  /* Adjust the number of resolutions if set to its defaults */
  if parameters.numresolution == 6i32 && image.x0 == 0u32 && image.y0 == 0u32 {
    let max_nl = opj_j2k_get_imf_max_nl(parameters, image);
    if max_nl >= 0i32 && parameters.numresolution > max_nl {
      parameters.numresolution = max_nl + 1i32
    }
    /* Note: below is generic logic */
    if parameters.tile_size_on == 0 {
      while parameters.numresolution > 0i32 {
        if image.x1 < (1u32) << (parameters.numresolution as u32).wrapping_sub(1u32) {
          parameters.numresolution -= 1
        } else {
          if image.y1 >= (1u32) << (parameters.numresolution as u32).wrapping_sub(1u32) {
            break;
          }
          parameters.numresolution -= 1
        }
      }
    }
  }
  /* Set defaults precincts */
  if parameters.csty == 0i32 {
    parameters.csty |= 0x1i32;
    if parameters.numresolution == 1i32 {
      parameters.res_spec = 1i32;
      parameters.prcw_init[0_usize] = 128i32;
      parameters.prch_init[0_usize] = 128i32
    } else {
      let mut i: core::ffi::c_int = 0;
      parameters.res_spec = parameters.numresolution - 1i32;
      i = 0i32;
      while i < parameters.res_spec {
        parameters.prcw_init[i as usize] = 256i32;
        parameters.prch_init[i as usize] = 256i32;
        i += 1
      }
    }
  };
}
/* Table A.53 from JPEG2000 standard */
static TAB_MAX_SUB_LEVEL_FROM_MAIN_LEVEL: [u16; 12] = [
  15_u16, 1_u16, 1_u16, 1_u16, 2_u16, 3_u16, 4_u16, 5_u16, 6_u16, 7_u16, 8_u16, 9_u16,
];
fn opj_j2k_is_imf_compliant(
  mut parameters: &mut EncoderParameters,
  mut image: &mut CoreImage,
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut i: u32 = 0;
  let comps = &image.comps;
  let rsiz = parameters.rsiz;
  let profile = (rsiz as core::ffi::c_int & 0xff00i32) as u16;
  let mainlevel = (rsiz as core::ffi::c_int & 0xfi32) as u16;
  let sublevel = (rsiz as core::ffi::c_int >> 4i32 & 0xfi32) as u16;
  let nl = parameters.numresolution - 1i32;
  let x_tsiz = if parameters.tile_size_on != 0 {
    parameters.cp_tdx as u32
  } else {
    image.x1
  };
  let mut ret = 1i32;
  /* Validate mainlevel */
  if mainlevel as core::ffi::c_int > 11i32 {
    event_msg!(p_manager, EVT_WARNING,
                      "IMF profile require mainlevel <= 11.\n-> {} is thus not compliant\n-> Non-IMF codestream will be generated\n",
                      mainlevel as core::ffi::c_int);
    ret = 0i32
  } else {
    /* Validate sublevel */
    assert!(
      core::mem::size_of::<[u16; 12]>()
        == ((11i32 + 1i32) as usize).wrapping_mul(core::mem::size_of::<u16>())
    );
    if sublevel as core::ffi::c_int
      > TAB_MAX_SUB_LEVEL_FROM_MAIN_LEVEL[mainlevel as usize] as core::ffi::c_int
    {
      event_msg!(p_manager, EVT_WARNING,
                          "IMF profile require sublevel <= {} for mainlevel = {}.\n-> {} is thus not compliant\n-> Non-IMF codestream will be generated\n",
                          TAB_MAX_SUB_LEVEL_FROM_MAIN_LEVEL[mainlevel as usize] as
                              core::ffi::c_int, mainlevel as core::ffi::c_int,
                          sublevel as core::ffi::c_int);
      ret = 0i32
    }
  }
  /* Number of components */
  if image.numcomps > 3u32 {
    event_msg!(p_manager, EVT_WARNING,
                      "IMF profiles require at most 3 components.\n-> Number of components of input image ({}) is not compliant\n-> Non-IMF codestream will be generated\n",
                      image.numcomps);
    ret = 0i32
  }
  if image.x0 != 0u32 || image.y0 != 0u32 {
    event_msg!(p_manager, EVT_WARNING,
                      "IMF profiles require image origin to be at 0,0.\n-> {},{} is not compliant\n-> Non-IMF codestream will be generated\n", image.x0,
                      (image.y0 != 0u32) as
                          core::ffi::c_int);
    ret = 0i32
  }
  if parameters.cp_tx0 != 0i32 || parameters.cp_ty0 != 0i32 {
    event_msg!(p_manager, EVT_WARNING,
                      "IMF profiles require tile origin to be at 0,0.\n-> {},{} is not compliant\n-> Non-IMF codestream will be generated\n",
                      parameters.cp_tx0, parameters.cp_ty0);
    ret = 0i32
  }
  if parameters.tile_size_on != 0 {
    if profile as core::ffi::c_int == 0x400i32
      || profile as core::ffi::c_int == 0x500i32
      || profile as core::ffi::c_int == 0x600i32
    {
      if (parameters.cp_tdx as u32) < image.x1 || (parameters.cp_tdy as u32) < image.y1 {
        event_msg!(p_manager, EVT_WARNING,
                              "IMF 2K/4K/8K single tile profiles require tile to be greater or equal to image size.\n-> {},{} is lesser than {},{}\n-> Non-IMF codestream will be generated\n",
                              parameters.cp_tdx, parameters.cp_tdy,
                              image.x1, image.y1);
        ret = 0i32
      }
    } else if !(parameters.cp_tdx as u32 >= image.x1 && parameters.cp_tdy as u32 >= image.y1)
      && !(parameters.cp_tdx == 1024i32 && parameters.cp_tdy == 1024i32)
      && !(parameters.cp_tdx == 2048i32
        && parameters.cp_tdy == 2048i32
        && (profile as core::ffi::c_int == 0x500i32 || profile as core::ffi::c_int == 0x600i32))
      && !(parameters.cp_tdx == 4096i32
        && parameters.cp_tdy == 4096i32
        && profile as core::ffi::c_int == 0x600i32)
    {
      event_msg!(p_manager, EVT_WARNING,
                                "IMF 2K_R/4K_R/8K_R single/multiple tile profiles require tile to be greater or equal to image size,\nor to be (1024,1024), or (2048,2048) for 4K_R/8K_R or (4096,4096) for 8K_R.\n-> {},{} is non conformant\n-> Non-IMF codestream will be generated\n",
                                parameters.cp_tdx,
                                parameters.cp_tdy);
      ret = 0i32
    }
  }
  /* Bitdepth */
  i = 0_u32;
  while i < image.numcomps {
    if !(comps[i as usize].prec >= 8u32 && comps[i as usize].prec <= 16u32)
      || comps[i as usize].sgnd != 0
    {
      let tmp_str = if comps[i as usize].sgnd != 0 {
        "signed"
      } else {
        "unsigned"
      };
      event_msg!(p_manager, EVT_WARNING,
                          "IMF profiles require precision of each component to b in [8-16] bits unsigned-> At least component {} of input image ({} bits, {}) is not compliant\n-> Non-IMF codestream will be generated\n", i,
                          comps[i as usize].prec, tmp_str);
      ret = 0i32
    }
    i += 1;
  }
  /* Sub-sampling */
  i = 0_u32;
  while i < image.numcomps {
    if i == 0u32 && comps[i as usize].dx != 1u32 {
      event_msg!(p_manager, EVT_WARNING,
                          "IMF profiles require XRSiz1 == 1. Here it is set to {}.\n-> Non-IMF codestream will be generated\n",
                          comps[i as usize].dx);
      ret = 0i32
    }
    if i == 1u32 && comps[i as usize].dx != 1u32 && comps[i as usize].dx != 2u32 {
      event_msg!(p_manager, EVT_WARNING,
                          "IMF profiles require XRSiz2 == 1 or 2. Here it is set to {}.\n-> Non-IMF codestream will be generated\n",
                          comps[i as usize].dx);
      ret = 0i32
    }
    if i > 1u32 && comps[i as usize].dx != comps[i as usize - 1].dx {
      event_msg!(p_manager, EVT_WARNING,
                          "IMF profiles require XRSiz{} to be the same as XRSiz2. Here it is set to {} instead of {}.\n-> Non-IMF codestream will be generated\n",
                          i.wrapping_add(1u32),
                          comps[i as usize].dx,
                          comps[i as usize - 1].dx);
      ret = 0i32
    }
    if comps[i as usize].dy != 1u32 {
      event_msg!(p_manager, EVT_WARNING,
                          "IMF profiles require YRsiz == 1. Here it is set to {} for component {}.\n-> Non-IMF codestream will be generated\n",
                          comps[i as usize].dy, i);
      ret = 0i32
    }
    i += 1;
  }
  /* Image size */
  match profile as core::ffi::c_int {
    1024 | 1792 => {
      if (comps[0].w > 2048u32) as core::ffi::c_int | (comps[0].h > 1556u32) as core::ffi::c_int
        != 0
      {
        event_msg!(p_manager, EVT_WARNING,
                              "IMF 2K/2K_R profile require:\nwidth <= 2048 and height <= 1556\n-> Input image size {} x {} is not compliant\n-> Non-IMF codestream will be generated\n",
                              comps[0].w,
                              comps[0].h);
        ret = 0i32
      }
    }
    1280 | 2048 => {
      if (comps[0].w > 4096u32) as core::ffi::c_int | (comps[0].h > 3112u32) as core::ffi::c_int
        != 0
      {
        event_msg!(p_manager, EVT_WARNING,
                              "IMF 4K/4K_R profile require:\nwidth <= 4096 and height <= 3112\n-> Input image size {} x {} is not compliant\n-> Non-IMF codestream will be generated\n",
                              comps[0].w,
                              comps[0].h);
        ret = 0i32
      }
    }
    1536 | 2304 => {
      if (comps[0].w > 8192u32) as core::ffi::c_int | (comps[0].h > 6224u32) as core::ffi::c_int
        != 0
      {
        event_msg!(p_manager, EVT_WARNING,
                              "IMF 8K/8K_R profile require:\nwidth <= 8192 and height <= 6224\n-> Input image size {} x {} is not compliant\n-> Non-IMF codestream will be generated\n",
                              comps[0].w,
                              comps[0].h);
        ret = 0i32
      }
    }
    _ => {
      /* C: assert(0); - unknown profiles are validated away, and the C's
      release build (NDEBUG) falls through here anyway. */
      debug_assert!(false, "Unknown OPJ_PROFILE");
    }
  }
  if parameters.roi_compno != -(1i32) {
    event_msg!(p_manager, EVT_WARNING,
                      "IMF profile forbid RGN / region of interest marker.\n-> Compression parameters specify a ROI\n-> Non-IMF codestream will be generated\n");
    ret = 0i32
  }
  if parameters.cblockw_init != 32i32 || parameters.cblockh_init != 32i32 {
    event_msg!(p_manager, EVT_WARNING,
                      "IMF profile require code block size to be 32x32.\n-> Compression parameters set it to {}x{}.\n-> Non-IMF codestream will be generated\n",
                      parameters.cblockw_init, parameters.cblockh_init);
    ret = 0i32
  }
  if parameters.prog_order as core::ffi::c_int != OPJ_CPRL as core::ffi::c_int {
    event_msg!(p_manager, EVT_WARNING,
                      "IMF profile require progression order to be CPRL.\n-> Compression parameters set it to {}.\n-> Non-IMF codestream will be generated\n",
                      parameters.prog_order as core::ffi::c_int);
    ret = 0i32
  }
  if parameters.numpocs != 0u32 {
    event_msg!(p_manager, EVT_WARNING,
                      "IMF profile forbid POC markers.\n-> Compression parameters set {} POC.\n-> Non-IMF codestream will be generated\n",
                      parameters.numpocs);
    ret = 0i32
  }
  /* Codeblock style: no mode switch enabled */
  if parameters.mode != 0i32 {
    event_msg!(p_manager, EVT_WARNING,
                      "IMF profile forbid mode switch in code block style.\n-> Compression parameters set code block style to {}.\n-> Non-IMF codestream will be generated\n",
                      parameters.mode);
    ret = 0i32
  }
  if profile as core::ffi::c_int == 0x400i32
    || profile as core::ffi::c_int == 0x500i32
    || profile as core::ffi::c_int == 0x600i32
  {
    /* Expect 9-7 transform */
    if parameters.irreversible != 1i32 {
      event_msg!(p_manager, EVT_WARNING,
                          "IMF 2K/4K/8K profiles require 9-7 Irreversible Transform.\n-> Compression parameters set it to reversible.\n-> Non-IMF codestream will be generated\n");
      ret = 0i32
    }
  } else if parameters.irreversible != 0i32 {
    event_msg!(p_manager, EVT_WARNING,
                      "IMF 2K/4K/8K profiles require 5-3 reversible Transform.\n-> Compression parameters set it to irreversible.\n-> Non-IMF codestream will be generated\n");
    ret = 0i32
  }
  /* Expect 5-3 transform */
  /* Number of layers */
  if parameters.tcp_numlayers != 1i32 {
    event_msg!(p_manager, EVT_WARNING,
                      "IMF 2K/4K/8K profiles require 1 single quality layer.\n-> Number of layers is {}.\n-> Non-IMF codestream will be generated\n",
                      parameters.tcp_numlayers);
    ret = 0i32
  }
  /* Decomposition levels */
  match profile as core::ffi::c_int {
    1024 => {
      if !(1i32..=5i32).contains(&nl) {
        event_msg!(p_manager, EVT_WARNING,
                              "IMF 2K profile requires 1 <= NL <= 5:\n-> Number of decomposition levels is {}.\n-> Non-IMF codestream will be generated\n", nl);
        ret = 0i32
      }
    }
    1280 => {
      if !(1i32..=6i32).contains(&nl) {
        event_msg!(p_manager, EVT_WARNING,
                              "IMF 4K profile requires 1 <= NL <= 6:\n-> Number of decomposition levels is {}.\n-> Non-IMF codestream will be generated\n", nl);
        ret = 0i32
      }
    }
    1536 => {
      if !(1i32..=7i32).contains(&nl) {
        event_msg!(p_manager, EVT_WARNING,
                              "IMF 8K profile requires 1 <= NL <= 7:\n-> Number of decomposition levels is {}.\n-> Non-IMF codestream will be generated\n", nl);
        ret = 0i32
      }
    }
    1792 => {
      if x_tsiz >= 2048u32 {
        if !(1i32..=5i32).contains(&nl) {
          event_msg!(p_manager, EVT_WARNING,
                                  "IMF 2K_R profile requires 1 <= NL <= 5 for x_tsiz >= 2048:\n-> Number of decomposition levels is {}.\n-> Non-IMF codestream will be generated\n",
                                  nl);
          ret = 0i32
        }
      } else if x_tsiz >= 1024u32 && !(1i32..=4i32).contains(&nl) {
        event_msg!(p_manager, EVT_WARNING,
                                "IMF 2K_R profile requires 1 <= nl <= 4 for x_tsiz in [1024,2048[:\n-> Number of decomposition levels is {}.\n-> Non-IMF codestream will be generated\n",
                                nl);
        ret = 0i32
      }
    }
    2048 => {
      if x_tsiz >= 4096u32 {
        if !(1i32..=6i32).contains(&nl) {
          event_msg!(p_manager, EVT_WARNING,
                                  "IMF 4K_R profile requires 1 <= NL <= 6 for x_tsiz >= 4096:\n-> Number of decomposition levels is {}.\n-> Non-IMF codestream will be generated\n",
                                  nl);
          ret = 0i32
        }
      } else if x_tsiz >= 2048u32 {
        if !(1i32..=5i32).contains(&nl) {
          event_msg!(p_manager, EVT_WARNING,
                                  "IMF 4K_R profile requires 1 <= NL <= 5 for x_tsiz in [2048,4096[:\n-> Number of decomposition levels is {}.\n-> Non-IMF codestream will be generated\n",
                                  nl);
          ret = 0i32
        }
      } else if x_tsiz >= 1024u32 && !(1i32..=4i32).contains(&nl) {
        event_msg!(p_manager, EVT_WARNING,
                                "IMF 4K_R profile requires 1 <= nl <= 4 for x_tsiz in [1024,2048[:\n-> Number of decomposition levels is {}.\n-> Non-IMF codestream will be generated\n",
                                nl);
        ret = 0i32
      }
    }
    2304 => {
      if x_tsiz >= 8192u32 {
        if !(1i32..=7i32).contains(&nl) {
          event_msg!(p_manager, EVT_WARNING,
                                  "IMF 4K_R profile requires 1 <= NL <= 7 for x_tsiz >= 8192:\n-> Number of decomposition levels is {}.\n-> Non-IMF codestream will be generated\n",
                                  nl);
          ret = 0i32
        }
      } else if x_tsiz >= 4096u32 {
        if !(1i32..=6i32).contains(&nl) {
          event_msg!(p_manager, EVT_WARNING,
                                  "IMF 4K_R profile requires 1 <= NL <= 6 for x_tsiz in [4096,8192[:\n-> Number of decomposition levels is {}.\n-> Non-IMF codestream will be generated\n",
                                  nl);
          ret = 0i32
        }
      } else if x_tsiz >= 2048u32 {
        if !(1i32..=5i32).contains(&nl) {
          event_msg!(p_manager, EVT_WARNING,
                                  "IMF 4K_R profile requires 1 <= NL <= 5 for x_tsiz in [2048,4096[:\n-> Number of decomposition levels is {}.\n-> Non-IMF codestream will be generated\n",
                                  nl);
          ret = 0i32
        }
      } else if x_tsiz >= 1024u32 && !(1i32..=4i32).contains(&nl) {
        event_msg!(p_manager, EVT_WARNING,
                                "IMF 4K_R profile requires 1 <= nl <= 4 for x_tsiz in [1024,2048[:\n-> Number of decomposition levels is {}.\n-> Non-IMF codestream will be generated\n",
                                nl);
        ret = 0i32
      }
    }
    _ => {}
  }
  if parameters.numresolution == 1i32 {
    if parameters.res_spec != 1i32
      || parameters.prcw_init[0_usize] != 128i32
      || parameters.prch_init[0_usize] != 128i32
    {
      event_msg!(p_manager, EVT_WARNING,
                          "IMF profiles require PPx = PPy = 7 for NLLL band, else 8.\n-> Supplied values are different from that.\n-> Non-IMF codestream will be generated\n");
      ret = 0i32
    }
  } else {
    /* The C indexes prcw_init[i] with the caller-supplied res_spec (OOB
    read for res_spec > 33); clamp the walk to the fixed array instead. */
    let mut i_0: core::ffi::c_int = 0;
    i_0 = 0i32;
    while i_0 < parameters.res_spec && (i_0 as usize) < parameters.prcw_init.len() {
      if parameters.prcw_init[i_0 as usize] != 256i32
        || parameters.prch_init[i_0 as usize] != 256i32
      {
        event_msg!(p_manager, EVT_WARNING,
                              "IMF profiles require PPx = PPy = 7 for NLLL band, else 8.\n-> Supplied values are different from that.\n-> Non-IMF codestream will be generated\n");
        ret = 0i32
      }
      i_0 += 1
    }
  }
  ret
}

pub(crate) fn opj_j2k_setup_encoder(
  p_j2k: &mut J2k,
  parameters: &mut EncoderParameters,
  image: &mut CoreImage,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut i: u32 = 0;
  let mut j: u32 = 0;
  let mut tileno: u32 = 0;
  let mut numpocs_tile: u32 = 0;
  let cp = &mut p_j2k.m_cp;
  let mut cblkw: u32 = 0;
  let mut cblkh: u32 = 0;
  if parameters.numresolution <= 0i32 || parameters.numresolution > 33i32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid number of resolutions : {} not in range [1,{}]\n",
      parameters.numresolution,
      33i32,
    );
    return 0i32;
  }
  /* Port-specific hardening: the C overflows its fixed-size arrays (or has
  undefined-behaviour shifts) for out-of-range encoder parameters; reject
  them here so the port fails cleanly instead of panicking. */
  if parameters.tcp_numlayers < 0i32
    || parameters.tcp_numlayers > parameters.tcp_rates.len() as i32
  {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid number of layers : {} not in range [0,{}]\n",
      parameters.tcp_numlayers,
      parameters.tcp_rates.len(),
    );
    return 0i32;
  }
  if parameters.numpocs > j2k::J2K_MAX_POCS {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid number of progression orders : {} exceeds the maximum of {}\n",
      parameters.numpocs,
      j2k::J2K_MAX_POCS,
    );
    return 0i32;
  }
  if parameters.csty & 1i32 != 0i32
    && (parameters.res_spec < 1i32 || parameters.res_spec > OPJ_J2K_MAXRLVLS as i32)
  {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid number of specified resolutions : {} not in range [1,{}]\n",
      parameters.res_spec,
      OPJ_J2K_MAXRLVLS,
    );
    return 0i32;
  }
  /* The precinct-size derivation shifts prcw_init[res_spec-1] right by
  (numresolution - res_spec) bits; shift counts >= 32 are a debug-build
  panic (the C has shift UB).  Reject the combination instead. */
  if parameters.csty & 1i32 != 0i32
    && parameters.numresolution as i64 - parameters.res_spec as i64 >= 32i64
  {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid resolution/precinct combination : numresolution ({}) - res_spec ({}) must be < 32\n",
      parameters.numresolution,
      parameters.res_spec,
    );
    return 0i32;
  }
  /* The C's opj_j2k_write_cod asserts on an unknown progression order; the
  port's write path uses it as a marker value (garbage output where the C
  has UB) - reject it up front. */
  if !(0i32..=4i32).contains(&parameters.prog_order) {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid progression order : {}\n",
      parameters.prog_order,
    );
    return 0i32;
  }
  for comp in image.comps.iter() {
    if !crate::types::prec_valid(comp.prec) {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Invalid image component precision : {} not in range [1,31]\n",
        comp.prec,
      );
      return 0i32;
    }
  }
  /* The custom-MCT matrix and dc-shift vectors are sized by the caller
  (public EncoderParameters / C-ABI opj_set_MCT); the C reads them
  unconditionally (heap overread UB).  Validate the lengths so the port
  fails cleanly instead of panicking in the matrix inversion or the
  dc_shift indexing. */
  if let Some((matrix, dc_shift)) = parameters.mct_data.as_ref() {
    let n = image.numcomps as usize;
    if matrix.len() != n * n || dc_shift.len() != n {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Invalid MCT data : matrix {} entries, dc_shift {} entries, expected {} and {}\n",
        matrix.len(),
        dc_shift.len(),
        n * n,
        n,
      );
      return 0i32;
    }
  }
  if parameters.cblockw_init < 4i32 || parameters.cblockw_init > 1024i32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid value for cblockw_init: {} not a power of 2 in range [4,1024]\n",
      parameters.cblockw_init,
    );
    return 0i32;
  }
  if parameters.cblockh_init < 4i32 || parameters.cblockh_init > 1024i32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid value for cblockh_init: {} not a power of 2 not in range [4,1024]\n",
      parameters.cblockh_init,
    );
    return 0i32;
  }
  if parameters.cblockw_init * parameters.cblockh_init > 4096i32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid value for cblockw_init * cblockh_init: should be <= 4096\n",
    );
    return 0i32;
  }
  cblkw = opj_int_floorlog2(parameters.cblockw_init) as u32;
  cblkh = opj_int_floorlog2(parameters.cblockh_init) as u32;
  if parameters.cblockw_init != (1i32) << cblkw {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid value for cblockw_init: {} not a power of 2 in range [4,1024]\n",
      parameters.cblockw_init,
    );
    return 0i32;
  }
  if parameters.cblockh_init != (1i32) << cblkh {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid value for cblockw_init: {} not a power of 2 in range [4,1024]\n",
      parameters.cblockh_init,
    );
    return 0i32;
  }

  if parameters.cp_fixed_alloc != 0 {
    if parameters.cp_matrice.is_none() {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "cp_fixed_alloc set, but cp_matrice missing\n"
      );
      return 0;
    }

    if parameters.tcp_numlayers > j2k::J2K_TCD_MATRIX_MAX_LAYER_COUNT {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "tcp_numlayers when cp_fixed_alloc set should not exceed {}\n",
        j2k::J2K_TCD_MATRIX_MAX_LAYER_COUNT
      );
      return 0;
    }
    if parameters.numresolution > j2k::J2K_TCD_MATRIX_MAX_RESOLUTION_COUNT {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "numresolution when cp_fixed_alloc set should not exceed {}\n",
        j2k::J2K_TCD_MATRIX_MAX_RESOLUTION_COUNT
      );
      return 0;
    }
    /* makelayer_fixed indexes m_matrice[layer * numresolutions * 3 +
    band * 3 + k] per tile (the C reads the caller's array OOB when it is
    short — UB); reject a too-short matrix up front. */
    let matrix_need =
      parameters.tcp_numlayers * parameters.numresolution * 3;
    if let Some(matrice) = parameters.cp_matrice.as_deref() {
      if (matrice.len() as i32) < matrix_need {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "cp_matrice is too short for {} layers x {} resolutions (need {} entries)\n",
          parameters.tcp_numlayers,
          parameters.numresolution,
          matrix_need
        );
        return 0;
      }
    }
  }

  p_j2k.m_specific_param.m_encoder.m_nb_comps = image.numcomps;
  /* keep a link to cp so that we can destroy it later in j2k_destroy_compress */
  /* set default values for cp */
  cp.tw = 1_u32;
  cp.th = 1_u32;
  /* Deprecated cp_cinema/cp_rsiz are honored only when RSIZ is 0 */
  if parameters.rsiz as core::ffi::c_int == 0i32 {
    /* consider deprecated fields only if RSIZ has not been set */
    let mut deprecated_used = 0i32;
    match parameters.cp_cinema as core::ffi::c_uint {
      1 => {
        parameters.rsiz = 0x3_u16;
        parameters.max_cs_size = 1302083i32;
        parameters.max_comp_size = 1041666i32;
        deprecated_used = 1i32
      }
      2 => {
        parameters.rsiz = 0x3_u16;
        parameters.max_cs_size = 651041i32;
        parameters.max_comp_size = 520833i32;
        deprecated_used = 1i32
      }
      3 => {
        parameters.rsiz = 0x4_u16;
        parameters.max_cs_size = 1302083i32;
        parameters.max_comp_size = 1041666i32;
        deprecated_used = 1i32
      }
      _ => {}
    }
    match parameters.cp_rsiz as core::ffi::c_uint {
      3 => {
        parameters.rsiz = 0x3_u16;
        deprecated_used = 1i32
      }
      4 => {
        parameters.rsiz = 0x4_u16;
        deprecated_used = 1i32
      }
      33024 => {
        parameters.rsiz = (0x8000i32 | 0x100i32) as u16;
        deprecated_used = 1i32
      }
      _ => {}
    }
    if deprecated_used != 0 {
      event_msg!(p_manager, EVT_WARNING,
                          "Deprecated fields cp_cinema or cp_rsiz are used\nPlease consider using only the rsiz field\nSee openjpeg.h documentation for more details\n");
    }
  }
  /* If no explicit layers are provided, use lossless settings */
  if parameters.tcp_numlayers == 0i32 {
    parameters.tcp_numlayers = 1i32;
    parameters.cp_disto_alloc = 1i32;
    parameters.tcp_rates[0_usize] = 0 as core::ffi::c_float
  }
  if parameters.cp_disto_alloc != 0 {
    /* Emit warnings if tcp_rates are not decreasing */
    i = 1_u32;
    while i < parameters.tcp_numlayers as u32 {
      let mut rate_i_corr = parameters.tcp_rates[i as usize];
      let mut rate_i_m_1_corr = parameters.tcp_rates[i.wrapping_sub(1u32) as usize];
      if rate_i_corr as core::ffi::c_double <= 1.0f64 {
        rate_i_corr = 1.0f64 as f32
      }
      if rate_i_m_1_corr as core::ffi::c_double <= 1.0f64 {
        rate_i_m_1_corr = 1.0f64 as f32
      }
      if rate_i_corr >= rate_i_m_1_corr {
        if rate_i_corr != parameters.tcp_rates[i as usize]
          && rate_i_m_1_corr != parameters.tcp_rates[i.wrapping_sub(1u32) as usize]
        {
          event_msg!(p_manager, EVT_WARNING,
                                  "tcp_rates[{}]={} (corrected as {}) should be strictly lesser than tcp_rates[{}]={} (corrected as {})\n", i,
                                  parameters.tcp_rates[i as usize] as
                                      core::ffi::c_double,
                                  rate_i_corr as core::ffi::c_double,
                                  i.wrapping_sub(1i32 as
                                                     core::ffi::c_uint),
                                  parameters.tcp_rates[i.wrapping_sub(1 as
                                                                             core::ffi::c_int
                                                                             as
                                                                             core::ffi::c_uint)
                                                              as usize] as
                                      core::ffi::c_double,
                                  rate_i_m_1_corr as core::ffi::c_double);
        } else if rate_i_corr != parameters.tcp_rates[i as usize] {
          event_msg!(
            p_manager,
            EVT_WARNING,
            "tcp_rates[{}]={} (corrected as {}) should be strictly lesser than tcp_rates[{}]={}\n",
            i,
            parameters.tcp_rates[i as usize] as core::ffi::c_double,
            rate_i_corr as core::ffi::c_double,
            i.wrapping_sub(1i32 as core::ffi::c_uint),
            parameters.tcp_rates
              [i.wrapping_sub(1 as core::ffi::c_int as core::ffi::c_uint) as usize]
              as core::ffi::c_double
          );
        } else if rate_i_m_1_corr != parameters.tcp_rates[i.wrapping_sub(1u32) as usize] {
          event_msg!(
            p_manager,
            EVT_WARNING,
            "tcp_rates[{}]={} should be strictly lesser than tcp_rates[{}]={} (corrected as {})\n",
            i,
            parameters.tcp_rates[i as usize] as core::ffi::c_double,
            i.wrapping_sub(1i32 as core::ffi::c_uint),
            parameters.tcp_rates
              [i.wrapping_sub(1 as core::ffi::c_int as core::ffi::c_uint) as usize]
              as core::ffi::c_double,
            rate_i_m_1_corr as core::ffi::c_double
          );
        } else {
          event_msg!(
            p_manager,
            EVT_WARNING,
            "tcp_rates[{}]={} should be strictly lesser than tcp_rates[{}]={}\n",
            i,
            parameters.tcp_rates[i as usize] as core::ffi::c_double,
            i.wrapping_sub(1u32),
            parameters.tcp_rates[i.wrapping_sub(1u32) as usize] as core::ffi::c_double,
          );
        }
      }
      i += 1;
    }
  } else if parameters.cp_fixed_quality != 0 {
    /* Emit warnings if tcp_distoratio are not increasing */
    i = 1_u32;
    while i < parameters.tcp_numlayers as u32 {
      if parameters.tcp_distoratio[i as usize]
        < parameters.tcp_distoratio[i.wrapping_sub(1u32) as usize]
        && !(i == (parameters.tcp_numlayers as u32).wrapping_sub(1u32)
          && parameters.tcp_distoratio[i as usize] == 0 as core::ffi::c_float)
      {
        event_msg!(
          p_manager,
          EVT_WARNING,
          "tcp_distoratio[{}]={} should be strictly greater than tcp_distoratio[{}]={}\n",
          i,
          parameters.tcp_distoratio[i as usize] as core::ffi::c_double,
          i.wrapping_sub(1u32),
          parameters.tcp_distoratio[i.wrapping_sub(1u32) as usize] as core::ffi::c_double,
        );
      }
      i += 1;
    }
  }
  /* see if max_codestream_size does limit input rate */
  if parameters.max_cs_size <= 0i32 {
    if parameters.tcp_rates[(parameters.tcp_numlayers - 1i32) as usize] > 0 as core::ffi::c_float {
      let mut temp_size: f32 = 0.;
      temp_size = (image.numcomps as core::ffi::c_double
        * image.comps[0].w as core::ffi::c_double
        * image.comps[0].h as core::ffi::c_double
        * image.comps[0].prec as core::ffi::c_double
        / (parameters.tcp_rates[(parameters.tcp_numlayers - 1i32) as usize] as core::ffi::c_double
          * 8 as core::ffi::c_double
          * image.comps[0].dx as core::ffi::c_double
          * image.comps[0].dy as core::ffi::c_double)) as f32;
      if temp_size > 2147483647 as core::ffi::c_float {
        parameters.max_cs_size = 2147483647i32
      } else {
        parameters.max_cs_size = temp_size.floor() as core::ffi::c_int
      }
    } else {
      parameters.max_cs_size = 0i32
    }
  } else {
    let mut temp_rate: f32 = 0.;
    let mut cap = 0i32;
    if parameters.rsiz as core::ffi::c_int >= 0x400i32
      && parameters.rsiz as core::ffi::c_int <= 0x900i32 | 0x9bi32
      && parameters.max_cs_size > 0i32
      && parameters.tcp_numlayers == 1i32
      && parameters.tcp_rates[0_usize] == 0 as core::ffi::c_float
    {
      parameters.tcp_rates[0_usize] = image
        .numcomps
        .wrapping_mul(image.comps[0].w)
        .wrapping_mul(image.comps[0].h)
        .wrapping_mul(image.comps[0].prec) as f32
        / (parameters.max_cs_size as u32)
          .wrapping_mul(8u32)
          .wrapping_mul(image.comps[0].dx)
          .wrapping_mul(image.comps[0].dy) as f32
    }
    temp_rate = (image.numcomps as core::ffi::c_double
      * image.comps[0].w as core::ffi::c_double
      * image.comps[0].h as core::ffi::c_double
      * image.comps[0].prec as core::ffi::c_double
      / (parameters.max_cs_size as core::ffi::c_double
        * 8 as core::ffi::c_double
        * image.comps[0].dx as core::ffi::c_double
        * image.comps[0].dy as core::ffi::c_double)) as f32;
    i = 0_u32;
    while i < parameters.tcp_numlayers as u32 {
      if parameters.tcp_rates[i as usize] < temp_rate {
        parameters.tcp_rates[i as usize] = temp_rate;
        cap = 1i32
      }
      i += 1;
    }
    if cap != 0 {
      event_msg!(p_manager, EVT_WARNING,
                          "The desired maximum codestream size has limited\nat least one of the desired quality layers\n");
    }
  }
  if parameters.rsiz as core::ffi::c_int >= 0x3i32 && parameters.rsiz as core::ffi::c_int <= 0x6i32
    || parameters.rsiz as core::ffi::c_int >= 0x400i32
      && parameters.rsiz as core::ffi::c_int <= 0x900i32 | 0x9bi32
  {
    p_j2k.m_specific_param.m_encoder.m_tlm = 1i32
  }
  /* Manage profiles and applications and set RSIZ */
  /* set cinema parameters if required */
  if parameters.rsiz as core::ffi::c_int >= 0x3i32 && parameters.rsiz as core::ffi::c_int <= 0x6i32
  {
    if parameters.rsiz as core::ffi::c_int == 0x5i32
      || parameters.rsiz as core::ffi::c_int == 0x6i32
    {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "JPEG 2000 Scalable Digital Cinema profiles not yet supported\n",
      );
      parameters.rsiz = 0_u16
    } else {
      opj_j2k_set_cinema_parameters(parameters, image, p_manager);
      if opj_j2k_is_cinema_compliant(image, parameters.rsiz, p_manager) == 0 {
        parameters.rsiz = 0_u16
      }
    }
  } else if parameters.rsiz as core::ffi::c_int == 0x7i32 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "JPEG 2000 Long Term Storage profile not yet supported\n",
    );
    parameters.rsiz = 0_u16
  } else if parameters.rsiz as core::ffi::c_int >= 0x100i32
    && parameters.rsiz as core::ffi::c_int <= 0x300i32 | 0xbi32
  {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "JPEG 2000 Broadcast profiles not yet supported\n",
    );
    parameters.rsiz = 0_u16
  } else if parameters.rsiz as core::ffi::c_int >= 0x400i32
    && parameters.rsiz as core::ffi::c_int <= 0x900i32 | 0x9bi32
  {
    opj_j2k_set_imf_parameters(parameters, image, p_manager);
    if opj_j2k_is_imf_compliant(parameters, image, p_manager) == 0 {
      parameters.rsiz = 0_u16
    }
  } else if parameters.rsiz as core::ffi::c_int & 0x8000i32 != 0 {
    if parameters.rsiz as core::ffi::c_int == 0x8000i32 {
      event_msg!(p_manager, EVT_WARNING,
                          "JPEG 2000 Part-2 profile defined\nbut no Part-2 extension enabled.\nProfile set to NONE.\n");
      parameters.rsiz = 0_u16
    } else if parameters.rsiz as core::ffi::c_int != 0x8000i32 | 0x100i32 {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "Unsupported Part-2 extension enabled\nProfile set to NONE.\n",
      );
      parameters.rsiz = 0_u16
    }
  }
  /*
  copy user encoding parameters
  */
  cp.m_specific_param.m_enc.m_max_comp_size = parameters.max_comp_size as u32;
  cp.rsiz = parameters.rsiz;

  if parameters.cp_fixed_alloc != 0 {
    cp.m_specific_param.m_enc.m_quality_layer_alloc_strategy =
      J2K_QUALITY_LAYER_ALLOCATION_STRATEGY::FIXED_LAYER;
  } else if parameters.cp_fixed_quality != 0 {
    cp.m_specific_param.m_enc.m_quality_layer_alloc_strategy =
      J2K_QUALITY_LAYER_ALLOCATION_STRATEGY::FIXED_DISTORTION_RATIO;
  } else {
    cp.m_specific_param.m_enc.m_quality_layer_alloc_strategy =
      J2K_QUALITY_LAYER_ALLOCATION_STRATEGY::RATE_DISTORTION_RATIO;
  }

  if parameters.cp_fixed_alloc != 0 {
    if let Some(matrice) = parameters.cp_matrice.as_deref() {
      cp.m_specific_param.m_enc.m_matrice = matrice.to_vec();
    }
  }
  /* tiles */
  cp.tdx = parameters.cp_tdx as u32;
  cp.tdy = parameters.cp_tdy as u32;
  /* tile offset */
  cp.tx0 = parameters.cp_tx0 as u32;
  cp.ty0 = parameters.cp_ty0 as u32;
  /* comment string */
  if let Some(comment) = parameters.cp_comment.as_deref() {
    cp.comment = comment.to_vec();
    /* An explicitly-set EMPTY comment must still emit the COM marker (the C
    checks `comment != NULL`, not the length). */
    cp.comment_set = true;
  } else {
    /* Create default comment for codestream */
    /* UniPG>> */
    cp.comment = crate::openjpeg::default_codestream_comment().into_bytes();
    cp.comment_set = true;
  }
  /*
  calculate other encoding parameters
  */
  if parameters.tile_size_on != 0 {
    if cp.tdx == 0u32 {
      event_msg!(p_manager, EVT_ERROR, "Invalid tile width\n",);
      return 0i32;
    }
    if cp.tdy == 0u32 {
      event_msg!(p_manager, EVT_ERROR, "Invalid tile height\n",);
      return 0i32;
    }
    /* Reject empty tile grids up front: an image whose y1 <= ty0 (or
    x1 <= tx0) yields th == 0 (tw == 0) and the C divides by zero at the
    tile-count check below (SIGFPE); a zero tw later panics on
    tcps[0] in the validation step. */
    if image.y1 <= cp.ty0 || image.x1 <= cp.tx0 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Invalid tile grid: image [({},{}) - ({},{})] does not contain the tile origin ({},{})\n",
        image.x0,
        image.y0,
        image.x1,
        image.y1,
        cp.tx0,
        cp.ty0,
      );
      return 0i32;
    }
    cp.tw = opj_uint_ceildiv(image.x1 - (cp.tx0), cp.tdx);
    cp.th = opj_uint_ceildiv(image.y1 - (cp.ty0), cp.tdy);
    /* Check that the number of tiles is valid */
    if cp.tw == 0u32 || cp.th == 0u32 || cp.tw > (65535u32).wrapping_div(cp.th) {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Invalid number of tiles : {} x {} (maximum fixed by jpeg2000 norm is 65535 tiles)\n",
        cp.tw,
        cp.th,
      );
      return 0i32;
    }
  } else {
    cp.tdx = image.x1.wrapping_sub(cp.tx0);
    cp.tdy = image.y1.wrapping_sub(cp.ty0)
  }
  if parameters.tp_on != 0 {
    cp.m_specific_param.m_enc.m_tp_flag = parameters.tp_flag as u8;
    cp.m_specific_param.m_enc.m_tp_on = true;
  }
  /* USE_JPWL */
  /* initialize the multiple tiles */
  /* ---------------------------- */
  cp.tcps = (0..cp.tw.wrapping_mul(cp.th))
    .map(|_| Default::default())
    .collect();
  tileno = 0_u32;
  while tileno < cp.tw.wrapping_mul(cp.th) {
    let tcp = &mut cp.tcps[tileno as usize];
    let fixed_distoratio = cp.m_specific_param.m_enc.m_quality_layer_alloc_strategy
      == J2K_QUALITY_LAYER_ALLOCATION_STRATEGY::FIXED_DISTORTION_RATIO;
    tcp.numlayers = parameters.tcp_numlayers as u32;

    j = 0_u32;
    while j < tcp.numlayers {
      if cp.rsiz as core::ffi::c_int >= 0x3i32 && cp.rsiz as core::ffi::c_int <= 0x6i32
        || cp.rsiz as core::ffi::c_int >= 0x400i32
          && cp.rsiz as core::ffi::c_int <= 0x900i32 | 0x9bi32
      {
        if fixed_distoratio {
          tcp.distoratio[j as usize] = parameters.tcp_distoratio[j as usize]
        }
        tcp.rates[j as usize] = parameters.tcp_rates[j as usize]
      } else if fixed_distoratio {
        /* add fixed_quality */
        tcp.distoratio[j as usize] = parameters.tcp_distoratio[j as usize]
      } else {
        tcp.rates[j as usize] = parameters.tcp_rates[j as usize]
      }
      if !fixed_distoratio && tcp.rates[j as usize] as core::ffi::c_double <= 1.0f64 {
        tcp.rates[j as usize] = 0.0f64 as f32
        /* force lossless */
      }
      j += 1;
    }
    tcp.csty = parameters.csty as u32;
    tcp.prg = parameters.prog_order;
    tcp.mct = parameters.tcp_mct as u32;
    numpocs_tile = 0_u32;
    tcp.poc = false;
    if parameters.numpocs != 0 {
      /* initialisation of POC */
      i = 0_u32;
      while i < parameters.numpocs {
        if tileno.wrapping_add(1u32) == parameters.pocs[i as usize].tile {
          let tcp_poc = &mut tcp.pocs[numpocs_tile as usize];
          if parameters.pocs[numpocs_tile as usize].compno0 >= image.numcomps {
            event_msg!(p_manager, EVT_ERROR, "Invalid compno0 for POC {}\n", i,);
            return 0i32;
          }
          tcp_poc.resno0 = parameters.pocs[numpocs_tile as usize].resno0;
          tcp_poc.compno0 = parameters.pocs[numpocs_tile as usize].compno0;
          tcp_poc.layno1 = parameters.pocs[numpocs_tile as usize].layno1;
          tcp_poc.resno1 = parameters.pocs[numpocs_tile as usize].resno1;
          tcp_poc.compno1 = opj_uint_min(
            parameters.pocs[numpocs_tile as usize].compno1,
            image.numcomps,
          );
          tcp_poc.prg1 = parameters.pocs[numpocs_tile as usize].prg1;
          tcp_poc.tile = parameters.pocs[numpocs_tile as usize].tile;
          numpocs_tile += 1;
        }
        i += 1;
      }
      if numpocs_tile != 0 {
        /* Check the POC values (errors are reported via the manager) */
        opj_j2k_check_poc_val(
          &parameters.pocs[..parameters.numpocs as usize],
          tileno,
          parameters.numpocs,
          parameters.numresolution as u32,
          image.numcomps,
          parameters.tcp_numlayers as u32,
          p_manager,
        );
        tcp.poc = true;
        tcp.numpocs = numpocs_tile.wrapping_sub(1u32)
      }
    } else {
      tcp.numpocs = 0_u32
    }
    tcp
      .tccps
      .resize(image.numcomps as usize, Default::default());
    if let Some((l_tmp_buf, dc_shift)) = parameters.mct_data.as_ref() {
      let numcomps = image.numcomps;
      let l_mct_len = numcomps * numcomps;
      /* copy the caller-provided coding matrix into a scratch buffer */
      let mut l_tmp_buf = l_tmp_buf.clone();
      tcp.mct = 2_u32;
      tcp.m_mct_coding_matrix.clear();
      tcp.m_mct_coding_matrix.extend_from_slice(&l_tmp_buf);
      tcp.m_mct_decoding_matrix.clear();
      tcp.m_mct_decoding_matrix.resize(l_mct_len as usize, 0.0);
      if !opj_matrix_inversion_f(
        &mut l_tmp_buf,
        &mut tcp.m_mct_decoding_matrix,
        numcomps as usize,
      ) {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Failed to inverse encoder MCT decoding matrix \n",
        );
        return 0i32;
      }
      tcp.mct_norms.clear();
      tcp.mct_norms.resize(image.numcomps as usize, 0.0);
      opj_calculate_norms(
        &mut tcp.mct_norms,
        image.numcomps,
        &tcp.m_mct_decoding_matrix,
      );
      for (i, tccp_1) in tcp.tccps.iter_mut().enumerate().take(numcomps as usize) {
        tccp_1.m_dc_level_shift = dc_shift[i];
      }
      if opj_j2k_setup_mct_encoding(tcp, image) == 0i32 {
        event_msg!(p_manager, EVT_ERROR, "Failed to setup j2k mct encoding\n",);
        return 0i32;
      }
    } else {
      if tcp.mct == 1u32 && image.numcomps >= 3u32 {
        /* RGB->YCC MCT is enabled */
        if image.comps[0].dx != image.comps[1].dx
          || image.comps[0].dx != image.comps[2].dx
          || image.comps[0].dy != image.comps[1].dy
          || image.comps[0].dy != image.comps[2].dy
        {
          event_msg!(
            p_manager,
            EVT_WARNING,
            "Cannot perform MCT on components with different sizes. Disabling MCT.\n",
          ); /* 0 => one precinct || 1 => custom precinct  */
          tcp.mct = 0_u32
        }
      }
      i = 0_u32;
      while i < image.numcomps {
        let comp = &image.comps[i as usize];
        if comp.sgnd == 0 {
          tcp.tccps[i as usize].m_dc_level_shift = (1i32) << comp.prec.wrapping_sub(1u32)
        }
        i += 1;
      }
    }
    i = 0_u32;
    while i < image.numcomps {
      let tccp_1 = &mut tcp.tccps[i as usize];
      tccp_1.csty = (parameters.csty & 0x1i32) as u32;
      tccp_1.numresolutions = parameters.numresolution as u32;
      tccp_1.cblkw = opj_int_floorlog2(parameters.cblockw_init) as u32;
      tccp_1.cblkh = opj_int_floorlog2(parameters.cblockh_init) as u32;
      tccp_1.cblksty = parameters.mode as u32;
      tccp_1.qmfbid = if parameters.irreversible != 0 {
        0i32
      } else {
        1i32
      } as u32;
      tccp_1.qntsty = if parameters.irreversible != 0 {
        2i32
      } else {
        0i32
      } as u32;
      /* From https://github.com/uclouvain/openjpeg/issues/1340
      (upstream #1559): Cinema2K uses 1 guard bit */
      tccp_1.numgbits = if parameters.rsiz == OPJ_PROFILE_CINEMA_2K as u16 {
        1_u32
      } else {
        2_u32
      };
      if i as i32 == parameters.roi_compno {
        tccp_1.roishift = parameters.roi_shift
      } else {
        tccp_1.roishift = 0i32
      }
      if parameters.csty & 0x1i32 != 0 {
        let mut p = 0i32;
        let mut it_res: i32 = 0;
        assert!(tccp_1.numresolutions > 0u32);
        it_res = tccp_1.numresolutions as i32 - 1i32;
        while it_res >= 0i32 {
          if p < parameters.res_spec {
            if parameters.prcw_init[p as usize] < 1i32 {
              tccp_1.prcw[it_res as usize] = 1_u32
            } else {
              tccp_1.prcw[it_res as usize] =
                opj_int_floorlog2(parameters.prcw_init[p as usize]) as u32
            }
            if parameters.prch_init[p as usize] < 1i32 {
              tccp_1.prch[it_res as usize] = 1_u32
            } else {
              tccp_1.prch[it_res as usize] =
                opj_int_floorlog2(parameters.prch_init[p as usize]) as u32
            }
          } else {
            let mut res_spec = parameters.res_spec;
            let mut size_prcw = 0i32;
            let mut size_prch = 0i32;
            /*end for*/
            assert!(res_spec > 0i32);
            size_prcw = parameters.prcw_init[(res_spec - 1i32) as usize] >> (p - (res_spec - 1i32));
            size_prch = parameters.prch_init[(res_spec - 1i32) as usize] >> (p - (res_spec - 1i32));
            if size_prcw < 1i32 {
              tccp_1.prcw[it_res as usize] = 1_u32
            } else {
              tccp_1.prcw[it_res as usize] = opj_int_floorlog2(size_prcw) as u32
            }
            if size_prch < 1i32 {
              tccp_1.prch[it_res as usize] = 1_u32
            } else {
              tccp_1.prch[it_res as usize] = opj_int_floorlog2(size_prch) as u32
            }
          }
          p += 1;
          it_res -= 1
          /*printf("\nsize precinct for level {} : {},{}\n", it_res,tccp->prcw[it_res], tccp->prch[it_res]); */
        }
      } else {
        j = 0_u32;
        while j < tccp_1.numresolutions {
          tccp_1.prcw[j as usize] = 15_u32;
          tccp_1.prch[j as usize] = 15_u32;
          j += 1;
        }
      }
      opj_dwt_calc_explicit_stepsizes(tccp_1, image.comps[i as usize].prec);
      i += 1;
    }
    tileno += 1;
  }

  1i32
}
/* *
Add main header marker information
@param cstr_index    Codestream information structure
@param type         marker type
@param pos          byte offset of marker segment
@param len          length of marker segment
 */
fn opj_j2k_add_mhmarker(
  cstr_index: &mut CodestreamIndex,
  marker: J2KMarker,
  pos: i64,
  len: u32,
) -> OPJ_BOOL {
  cstr_index.markers.push(opj_marker_info {
    type_: marker.as_u32() as u16,
    pos,
    len: len as i32,
  });
  1i32
}
/* *
Add tile header marker information
@param tileno       tile index number
@param cstr_index   Codestream information structure
@param type         marker type
@param pos          byte offset of marker segment
@param len          length of marker segment
 */
fn opj_j2k_add_tlmarker(
  tileno: u32,
  cstr_index: &mut CodestreamIndex,
  marker: J2KMarker,
  pos: i64,
  len: u32,
) -> OPJ_BOOL {
  let tile = &mut cstr_index.tiles[tileno as usize];
  tile.markers.push(opj_marker_info {
    type_: marker.as_u32() as u16,
    pos,
    len: len as i32,
  });
  if marker == J2KMarker::SOT {
    let l_current_tile_part = tile.current_tpsno;
    if l_current_tile_part < tile.nb_tps {
      tile.tp_index[l_current_tile_part as usize].start_pos = pos;
    }
  }
  1i32
}
/*
 * -----------------------------------------------------------------------
 * -----------------------------------------------------------------------
 * -----------------------------------------------------------------------
 */

pub(crate) fn opj_j2k_end_decompress(
  mut _p_j2k: &mut J2k,
  mut _p_stream: &mut Stream,
  mut _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  1i32
}

pub(crate) fn opj_j2k_read_header(
  p_stream: &mut Stream,
  p_j2k: &mut J2k,
  p_manager: &mut EventMgr,
) -> Option<CoreImage> {
  /* preconditions */

  /* create an empty image header */
  p_j2k.m_private_image = Some(Box::new(CoreImage::default()));
  /* validation of the parameters codec (the C's fixed validation list:
  build_decoder + decoding_validation). */
  if opj_j2k_build_decoder(p_j2k, p_stream, p_manager) == 0
    || opj_j2k_decoding_validation(p_j2k, p_stream, p_manager) == 0
  {
    p_j2k.m_private_image = None;
    return None;
  }
  /* read header (the C's fixed header-reading list:
  read_header_procedure + copy_default_tcp_and_create_tcd). */
  if opj_j2k_read_header_procedure(p_j2k, p_stream, p_manager) == 0
    || opj_j2k_copy_default_tcp_and_create_tcd(p_j2k, p_stream, p_manager) == 0
  {
    p_j2k.m_private_image = None;
    return None;
  }
  /* The caller receives a header-only copy (component data empty), like the
  C opj_read_header image; the codec keeps its own copy. */
  Some(
    p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .clone_header_only(),
  )
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
fn opj_j2k_mct_validation(
  p_j2k: &mut J2k,
  _p_stream: &mut Stream,
  _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_is_valid = 1i32;
  let mut i: u32 = 0;
  let mut j: u32 = 0;
  /* preconditions */

  if p_j2k.m_cp.rsiz as core::ffi::c_int & 0x8200i32 == 0x8200i32 {
    let l_nb_tiles = p_j2k.m_cp.th.wrapping_mul(p_j2k.m_cp.tw);
    let numcomps = p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .numcomps;
    i = 0_u32;
    while i < l_nb_tiles {
      let l_tcp = &p_j2k.m_cp.tcps[i as usize];
      if l_tcp.mct == 2u32 {
        l_is_valid &= (!l_tcp.m_mct_coding_matrix.is_empty()) as core::ffi::c_int;
        j = 0_u32;
        while j < numcomps {
          l_is_valid &= (l_tcp.tccps[j as usize].qmfbid & 1u32 == 0) as core::ffi::c_int;
          j += 1;
        }
      }
      i += 1;
    }
  }
  l_is_valid
}

pub(crate) fn opj_j2k_setup_mct_encoding(p_tcp: &mut Tcp, p_image: &mut CoreImage) -> OPJ_BOOL {
  let mut l_indix = 1_u32;
  /* preconditions */
  let p_tcp_ref = &mut *p_tcp;
  if p_tcp_ref.mct != 2u32 {
    return 1i32;
  }
  /* decorrelation record, if a decoding matrix was provided */
  let mut l_mct_deco_idx: Option<usize> = None;
  if !p_tcp_ref.m_mct_decoding_matrix.is_empty() {
    p_tcp_ref.m_mct_records.push(Default::default());
    l_mct_deco_idx = Some(p_tcp_ref.m_mct_records.len() - 1);
    let l_mct_deco_data = &mut p_tcp_ref.m_mct_records[l_mct_deco_idx.unwrap()];
    let fresh26 = l_indix;
    l_indix = l_indix.wrapping_add(1);
    l_mct_deco_data.m_index = fresh26;
    l_mct_deco_data.m_array_type = MCT_TYPE_DECORRELATION;
    l_mct_deco_data.m_element_type = MCTElementType::FLOAT;
    let l_nb_elem = p_image.numcomps.wrapping_mul(p_image.numcomps);
    let l_mct_size = l_nb_elem.wrapping_mul(l_mct_deco_data.m_element_type.size());
    l_mct_deco_data.m_data.clear();
    l_mct_deco_data.m_data.resize(l_mct_size as usize, 0);
    l_mct_deco_data.m_element_type.write_from_float(
      p_tcp_ref.m_mct_decoding_matrix.as_slice(),
      l_mct_deco_data.m_data.as_mut_slice(),
    );
  }
  /* offset record holding the dc level shifts */
  p_tcp_ref.m_mct_records.push(Default::default());
  let l_mct_offset_idx = p_tcp_ref.m_mct_records.len() - 1;
  let l_mct_offset_data = &mut p_tcp_ref.m_mct_records[l_mct_offset_idx];
  let fresh27 = l_indix;
  l_indix = l_indix.wrapping_add(1);
  l_mct_offset_data.m_index = fresh27;
  l_mct_offset_data.m_array_type = MCT_TYPE_OFFSET;
  l_mct_offset_data.m_element_type = MCTElementType::FLOAT;
  let l_nb_elem = p_image.numcomps;
  let l_mct_size = l_nb_elem.wrapping_mul(l_mct_offset_data.m_element_type.size());
  let mut l_data: Vec<f32> = vec![0.0; l_nb_elem as usize];
  for (i, item) in l_data.iter_mut().enumerate() {
    *item = p_tcp_ref.tccps[i].m_dc_level_shift as f32;
  }
  l_mct_offset_data.m_data.clear();
  l_mct_offset_data.m_data.resize(l_mct_size as usize, 0);
  l_mct_offset_data
    .m_element_type
    .write_from_float(l_data.as_slice(), l_mct_offset_data.m_data.as_mut_slice());
  /* mcc record linking the two */
  p_tcp_ref.m_mcc_records.push(Default::default());
  let l_mcc_idx = p_tcp_ref.m_mcc_records.len() - 1;
  let l_mcc_data = &mut p_tcp_ref.m_mcc_records[l_mcc_idx];
  l_mcc_data.m_decorrelation_array = l_mct_deco_idx;
  l_mcc_data.m_is_irreversible = true;
  l_mcc_data.m_nb_comps = p_image.numcomps;
  l_mcc_data.m_index = l_indix;
  l_indix = l_indix.wrapping_add(1);
  l_mcc_data.m_offset_array = Some(l_mct_offset_idx);
  1i32
}
fn opj_j2k_build_decoder(
  mut _p_j2k: &mut J2k,
  mut _p_stream: &mut Stream,
  mut _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* add here initialization of cp
  copy paste of setup_decoder */
  1i32
}

/* *
 * Builds the tcd encoder to use to encode tile.
 */
fn opj_j2k_init_info(
  p_j2k: &mut J2k,
  _p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* The encode-side codestream-info v1 (index file) mirror is not
  supported by this port (mergeV2 TODO in the C code); the tile-part
  layout still needs to be computed here. */
  opj_j2k_calculate_tp(
    &mut p_j2k.m_cp,
    &mut p_j2k.m_specific_param.m_encoder.m_total_tile_parts,
    p_j2k
      .m_private_image
      .as_deref_mut()
      .expect("m_private_image"),
    p_manager,
  )
}

fn opj_j2k_build_encoder(
  mut _p_j2k: &mut J2k,
  mut _p_stream: &mut Stream,
  mut _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* add here initialization of cp
  copy paste of setup_encoder */
  1i32
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
fn opj_j2k_encoding_validation(
  mut p_j2k: &mut J2k,
  mut _p_stream: &mut Stream,
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_is_valid = 1i32;
  /* preconditions */

  /* STATE checking */
  /* make sure the state is at 0 */
  l_is_valid &= (p_j2k.m_specific_param.m_decoder.m_state == J2KState::NONE) as core::ffi::c_int;
  /* ISO 15444-1:2004 states between 1 & 33 (0 -> 32) */
  /* 33 (32) would always fail the check below (if a cast to 64bits was done) */
  if p_j2k.m_cp.tcps[0].tccps[0].numresolutions == 0u32
    || p_j2k.m_cp.tcps[0].tccps[0].numresolutions > 32u32
  {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Number of resolutions is too high in comparison to the size of tiles\n",
    );
    return 0i32;
  }
  if p_j2k.m_cp.tdx
    < ((1i32)
      << p_j2k.m_cp.tcps[0].tccps[0]
        .numresolutions
        .wrapping_sub(1u32)) as u32
  {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Number of resolutions is too high in comparison to the size of tiles\n",
    );
    return 0i32;
  }
  if p_j2k.m_cp.tdy
    < ((1i32)
      << p_j2k.m_cp.tcps[0].tccps[0]
        .numresolutions
        .wrapping_sub(1u32)) as u32
  {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Number of resolutions is too high in comparison to the size of tiles\n",
    );
    return 0i32;
  }
  /* PARAMETER VALIDATION */
  l_is_valid
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
fn opj_j2k_decoding_validation(
  mut p_j2k: &mut J2k,
  mut _p_stream: &mut Stream,
  mut _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_is_valid = 1i32;
  /* preconditions*/

  /* STATE checking */
  /* make sure the state is at 0 */
  l_is_valid &= (p_j2k.m_specific_param.m_decoder.m_state == J2KState::NONE) as core::ffi::c_int;
  /* PARAMETER VALIDATION */
  l_is_valid
}
/* *
 * The read header procedure.
 */
fn opj_j2k_read_header_procedure(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_marker_size: u32 = 0;
  let mut l_has_siz = 0i32;
  let mut l_has_cod = 0i32;
  let mut l_has_qcd = 0i32;
  let strict = p_j2k.m_cp.strict;
  /* preconditions */

  /*  We enter in the main header */
  p_j2k.m_specific_param.m_decoder.m_state = J2KState::MHSOC;
  /* Try to read the SOC marker, the codestream must begin with SOC marker */
  if opj_j2k_read_soc(p_j2k, p_stream, p_manager) == 0 {
    event_msg!(p_manager, EVT_ERROR, "Expected a SOC marker \n",);
    return 0i32;
  }
  /* Try to read 2 bytes (the next marker ID) from stream and copy them into the buffer */
  if opj_stream_read_data(
    p_stream,
    &mut p_j2k.m_specific_param.m_decoder.m_header_data[..2],
    p_manager,
  ) != 2
  {
    /* Progressive-download support (lenient mode): a header that ends after
    SIZ still yields the image dimensions and component count - accept the
    header read so far instead of failing hard. */
    if strict == 0 && l_has_siz != 0 {
      p_j2k.m_header_partial = true;
      return 1i32;
    }
    event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
    return 0i32;
  }
  /* Read 2 bytes as the new marker ID */
  let mut l_current_marker =
    J2KMarker::from_buffer(&p_j2k.m_specific_param.m_decoder.m_header_data);
  /* Try to read until the SOT is detected */
  while l_current_marker != J2KMarker::SOT {
    /* Check if the current marker ID is valid */
    if l_current_marker.is_invalid() {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "A marker ID was expected (0xff--) instead of {:.8x}\n",
        l_current_marker.as_u32(),
      );
      return 0i32;
    }
    /* Get the marker handler from the marker ID */
    let mut l_marker_handler = l_current_marker;
    /* Manage case where marker is unknown (or has no table entry in the C:
    EPH/SOC/SOD/EOC are skipped by the C's read_unk scan, so corrupt
    streams with stray FF 92/4F/93/D9 recover there - review round 3 M3). */
    if l_marker_handler.is_unknown() || !l_marker_handler.has_handler() {
      if opj_j2k_read_unk(p_j2k, p_stream, &mut l_current_marker, p_manager) == 0 {
        p_j2k.record_error_context(
          l_current_marker.as_u32(),
          opj_stream_tell(p_stream).max(0) as u64,
        );
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Unknown marker has been detected and generated error.\n",
        );
        return 0i32;
      }
      if l_current_marker == J2KMarker::SOT {
        break;
      }
      l_marker_handler = l_current_marker;
    }
    if l_marker_handler == J2KMarker::SIZ {
      /* Mark required SIZ marker as found */
      l_has_siz = 1i32
    }
    if l_marker_handler == J2KMarker::COD {
      /* Mark required COD marker as found */
      l_has_cod = 1i32
    }
    if l_marker_handler == J2KMarker::QCD {
      /* Mark required QCD marker as found */
      l_has_qcd = 1i32
    }
    /* Check if the marker is known and if it is the right place to find it */
    if p_j2k.m_specific_param.m_decoder.m_state & l_marker_handler.states() == J2KState::NONE {
      p_j2k.record_error_context(
        l_current_marker.as_u32(),
        opj_stream_tell(p_stream).max(0) as u64,
      );
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Marker is not compliant with its position\n",
      );
      return 0i32;
    }
    /* Try to read 2 bytes (the marker size) from stream and copy them into the buffer */
    if opj_stream_read_data(
      p_stream,
      &mut p_j2k.m_specific_param.m_decoder.m_header_data[..2],
      p_manager,
    ) != 2
    {
      if strict == 0 && l_has_siz != 0 {
        p_j2k.m_header_partial = true;
        return 1i32;
      }
      event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
      return 0i32;
    }
    /* read 2 bytes as the marker size */
    l_marker_size = opj_read_bytes(&p_j2k.m_specific_param.m_decoder.m_header_data, 2_u32); /* Subtract the size of the marker ID already read */
    if l_marker_size < 2u32 {
      event_msg!(p_manager, EVT_ERROR, "Invalid marker size\n",);
      return 0i32;
    }
    l_marker_size = (l_marker_size as core::ffi::c_uint).wrapping_sub(2u32);
    /* Check if the marker size is compatible with the header data size */
    let l_header_data = p_j2k.m_specific_param.m_decoder.header_data(l_marker_size);
    /* Try to read the rest of the marker segment from stream and copy them into the buffer */
    if opj_stream_read_data(p_stream, l_header_data, p_manager) != l_marker_size as usize {
      if strict == 0 && l_has_siz != 0 {
        p_j2k.m_header_partial = true;
        return 1i32;
      }
      event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
      return 0i32;
    }
    /* Read the marker segment with the correct marker handler */
    let header_data =
      p_j2k.m_specific_param.m_decoder.m_header_data[..l_marker_size as usize].to_vec();
    if l_marker_handler.handler(p_j2k, &header_data, p_manager) == 0 {
      p_j2k.record_error_context(
        l_current_marker.as_u32(),
        opj_stream_tell(p_stream).max(0) as u64,
      );
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Marker handler function failed to read the marker segment\n",
      );
      return 0i32;
    }
    /* Add the marker to the codestream index*/
    if 0i32
      == opj_j2k_add_mhmarker(
        p_j2k.cstr_index.as_deref_mut().expect("cstr_index"),
        l_marker_handler,
        (opj_stream_tell(p_stream) as u32)
          .wrapping_sub(l_marker_size)
          .wrapping_sub(4u32) as i64,
        l_marker_size.wrapping_add(4u32),
      )
    {
      event_msg!(p_manager, EVT_ERROR, "Not enough memory to add mh marker\n",);
      return 0i32;
    }
    /* Try to read 2 bytes (the next marker ID) from stream and copy them into the buffer */
    if opj_stream_read_data(
      p_stream,
      &mut p_j2k.m_specific_param.m_decoder.m_header_data[..2],
      p_manager,
    ) != 2
    {
      event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
      return 0i32;
    }
    /* read 2 bytes as the new marker ID */
    l_current_marker = J2KMarker::from_buffer(&p_j2k.m_specific_param.m_decoder.m_header_data);
  }
  if l_has_siz == 0i32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "required SIZ marker not found in main header\n",
    );
    return 0i32;
  }
  if l_has_cod == 0i32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "required COD marker not found in main header\n",
    );
    return 0i32;
  }
  if l_has_qcd == 0i32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "required QCD marker not found in main header\n",
    );
    return 0i32;
  }
  if opj_j2k_merge_ppm(&mut p_j2k.m_cp, p_manager) == 0 {
    event_msg!(p_manager, EVT_ERROR, "Failed to merge PPM data\n",);
    return 0i32;
  }
  event_msg!(
    p_manager,
    EVT_INFO,
    "Main header has been correctly decoded.\n",
  );
  /* Position of the last element if the main header */
  p_j2k
    .cstr_index
    .as_deref_mut()
    .expect("cstr_index")
    .main_head_end = (opj_stream_tell(p_stream) as u32).wrapping_sub(2u32) as i64;

  /* Build the tile-part index from the TLM information */
  opj_j2k_build_tp_index_from_tlm(p_j2k, p_manager);

  /* Next step: read a tile-part header */
  p_j2k.m_specific_param.m_decoder.m_state = J2KState::TPHSOT;
  1i32
}

/// Fill cstr_index.tiles[].tp_index[] start/end positions from the TLM
/// marker segments (upstream #1538).
fn opj_j2k_build_tp_index_from_tlm(p_j2k: &mut J2k, p_manager: &mut EventMgr) {
  let (mut tlm, mut cstr_index) = {
    let cstr_index = p_j2k.cstr_index.as_deref_mut().expect("cstr_index");
    let tlm = &mut p_j2k.m_specific_param.m_decoder.m_tlm;
    (&mut *tlm, &mut *cstr_index)
  };
  debug_assert!(cstr_index.main_head_end > 0);
  debug_assert!(!cstr_index.tiles.is_empty());

  if tlm.m_entries_count == 0 {
    tlm.m_is_invalid = true;
    return;
  }
  if tlm.m_is_invalid {
    return;
  }

  /* First pass: count the number of tile-parts per tile */
  for info in tlm.m_tile_part_infos.iter() {
    let tile = &mut cstr_index.tiles[info.m_tile_index as usize];
    tile.tileno = info.m_tile_index as u32;
    tile.current_nb_tps += 1;
  }

  /* Check that all tiles have at least one tile-part */
  let mut err = false;
  for tile in cstr_index.tiles.iter() {
    if tile.current_nb_tps == 0 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "opj_j2k_build_tp_index_from_tlm(): tile {} has no registered tile-part in TLM marker segments.\n",
        tile.tileno,
      );
      err = true;
      break;
    }
  }
  if err {
    tlm.m_is_invalid = true;
    for info in tlm.m_tile_part_infos.iter() {
      let tile = &mut cstr_index.tiles[info.m_tile_index as usize];
      tile.current_nb_tps = 0;
      tile.tp_index.clear();
    }
    return;
  }

  /* Final pass: fill the tile-part index */
  let mut l_cur_offset = cstr_index.main_head_end;
  for info in tlm.m_tile_part_infos.iter() {
    let tile = &mut cstr_index.tiles[info.m_tile_index as usize];
    if tile.tp_index.is_empty() {
      tile.tp_index = vec![Default::default(); tile.current_nb_tps as usize];
    }
    tile.tp_index[tile.nb_tps as usize].start_pos = l_cur_offset;
    /* The end_header field is not needed here; if there were no markers
    between SOT and SOD it would be l_cur_offset + 12. */
    tile.tp_index[tile.nb_tps as usize].end_pos = l_cur_offset + info.m_length as i64;
    tile.nb_tps += 1;
    l_cur_offset += info.m_length as i64;
  }
}
/* *
 * Copies the decoding tile parameters onto all the tile parameters.
 * Creates also the tile decoder.
 */
fn opj_j2k_copy_default_tcp_and_create_tcd(
  p_j2k: &mut J2k,
  _p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut i: u32 = 0;
  /* preconditions */
  /* A lenient partial header (SIZ decoded, the rest cut off) has no
  COD/QCD: the tcd creation would run on default (possibly zeroed) coding
  parameters.  Skip it; the decode stage returns the partial image. */
  if p_j2k.m_header_partial {
    return 1i32;
  }

  let l_nb_tiles: u32 = p_j2k.m_cp.th.wrapping_mul(p_j2k.m_cp.tw);
  let l_default_tcp = p_j2k
    .m_specific_param
    .m_decoder
    .m_default_tcp
    .as_deref_mut()
    .expect("decoder m_default_tcp is always set");
  /* For each tile */
  i = 0_u32;
  while i < l_nb_tiles {
    /* keep the tile-compo coding parameters of the current tile coding parameters*/
    let l_current_tccp = core::mem::take(&mut p_j2k.m_cp.tcps[i as usize].tccps);
    /* Deep-copy the default coding parameters into the current tile coding parameters.
    (The C code memcpy's the whole struct and then rebuilds every owned buffer; clone()
    does the deep copies directly.) */
    p_j2k.m_cp.tcps[i as usize] = l_default_tcp.clone();
    let l_tcp = &mut p_j2k.m_cp.tcps[i as usize];
    /* Initialize some values of the current tile coding parameters*/
    l_tcp.cod = false;
    l_tcp.ppt = false;
    l_tcp.ppt_markers = Vec::new();
    l_tcp.ppt_buffer = Vec::new();
    l_tcp.ppt_data_pos = 0;
    l_tcp.m_current_tile_part_number = -(1i32);
    /* Reconnect this tile's own tccps allocation, to keep buffer reuse. */
    l_tcp.tccps = l_current_tccp;
    /* Copy all the dflt_tile_compo_cp to the current tile cp */
    l_tcp.tccps.copy_from_slice(&l_default_tcp.tccps[..]);
    /* Move to next tile cp*/
    i += 1;
  }
  // Init the current tile decoder
  if opj_tcd_init(
    &mut p_j2k.m_tcd,
    p_j2k.m_private_image.as_deref().expect("m_private_image"),
    &mut p_j2k.m_cp,
  ) == 0
  {
    event_msg!(p_manager, EVT_ERROR, "Cannot decode tile, memory error\n",);
    return 0i32;
  }
  1i32
}

impl Drop for J2k {
  fn drop(&mut self) {
    if self.m_is_decoder != 0 {
      if let Some(mut tcp) = self.m_specific_param.m_decoder.m_default_tcp.take() {
        opj_j2k_tcp_destroy(tcp.as_mut());
      }
      self.m_specific_param.m_decoder.m_header_data = Vec::new();
      self.m_specific_param.m_decoder.m_comps_indices_to_decode = Vec::new();
      self.m_specific_param.m_decoder.m_numcomps_to_decode = 0_u32
    } else {
      self.m_specific_param.m_encoder.m_encoded_tile_data = Vec::new();
      self.m_specific_param.m_encoder.m_tlm_sot_offsets_buffer = Vec::new();
      self.m_specific_param.m_encoder.m_tlm_sot_offsets_current = 0;
      self.m_specific_param.m_encoder.m_header_tile_data = Vec::new();
    }
    opj_j2k_cp_destroy(&mut self.m_cp);
    self.m_cp = Default::default();
    self.cstr_index = None;
    self.m_private_image = None;
    self.m_output_image = None;
  }
}

/* *
 * Destroys a tile coding parameter structure.
 *
 * @param       p_tcp           the tile coding parameter to destroy.
 */
fn opj_j2k_tcp_destroy(p_tcp: &mut Tcp) {
  p_tcp.ppt_markers = Vec::new();
  p_tcp.ppt_buffer = Vec::new();
  p_tcp.tccps = Vec::new();
  p_tcp.m_mct_coding_matrix = Vec::new();
  p_tcp.m_mct_decoding_matrix = Vec::new();
  p_tcp.m_mcc_records = Vec::new();
  p_tcp.m_mct_records = Vec::new();
  p_tcp.mct_norms = Vec::new();
  opj_j2k_tcp_data_destroy(p_tcp);
}

/* *
 * Destroys the data inside a tile coding parameter structure.
 *
 * @param       p_tcp           the tile coding parameter which contain data to destroy.
 */
fn opj_j2k_tcp_data_destroy(p_tcp: &mut Tcp) {
  p_tcp.m_data.clear();
  p_tcp.m_data_size = 0_u32;
}

/* *
 * Destroys a coding parameter structure.
 *
 * @param       p_cp            the coding parameter to destroy.
 */
fn opj_j2k_cp_destroy(p_cp: &mut Cp) {
  let l_nb_tiles: u32; /* ppm_data belongs to the allocated buffer pointed by ppm_buffer */
  if !p_cp.tcps.is_empty() {
    l_nb_tiles = p_cp.th.wrapping_mul(p_cp.tw);
    let mut i: u32 = 0;
    while i < l_nb_tiles {
      opj_j2k_tcp_destroy(&mut p_cp.tcps[i as usize]);
      i += 1;
    }
    p_cp.tcps = Vec::new()
  }
  p_cp.ppm_markers = Vec::new();
  p_cp.ppm_buffer = Vec::new();
  p_cp.comment.clear();
  if !p_cp.m_is_decoder {
    p_cp.m_specific_param.m_enc.m_matrice = Vec::new()
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
fn opj_j2k_need_nb_tile_parts_correction(
  mut p_stream: &mut Stream,
  mut tile_no: u32,
  p_correction_needed: &mut OPJ_BOOL,
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_header_data: [u8; 10] = [0; 10];
  let mut l_stream_pos_backup: i64 = 0;
  let mut l_current_marker = J2KMarker::UNK(0);
  let mut l_marker_size: u32 = 0;
  let mut l_tile_no: u32 = 0;
  let mut l_tot_len: u32 = 0;
  let mut l_current_part: u32 = 0;
  let mut l_num_parts: u32 = 0;
  /* initialize to no correction needed */
  *p_correction_needed = 0i32;
  if opj_stream_has_seek(p_stream) == 0 {
    /* We can't do much in this case, seek is needed */
    return 1i32;
  }
  l_stream_pos_backup = opj_stream_tell(p_stream);
  if l_stream_pos_backup == -(1i32) as i64 {
    /* let's do nothing */
    return 1i32;
  }
  loop {
    /* Try to read 2 bytes (the next marker ID) from stream and copy them into the buffer */
    if opj_stream_read_data(p_stream, &mut l_header_data[..2], p_manager) != 2 {
      /* assume all is OK */
      if opj_stream_seek(p_stream, l_stream_pos_backup, p_manager) == 0 {
        return 0i32;
      }
      return 1i32;
    }
    /* Read 2 bytes from buffer as the new marker ID */
    l_current_marker = J2KMarker::from_buffer(&l_header_data);
    if l_current_marker != J2KMarker::SOT {
      /* assume all is OK */
      if opj_stream_seek(p_stream, l_stream_pos_backup, p_manager) == 0 {
        return 0i32;
      }
      return 1i32;
    }
    /* Try to read 2 bytes (the marker size) from stream and copy them into the buffer */
    if opj_stream_read_data(p_stream, &mut l_header_data[..2], p_manager) != 2 {
      event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
      return 0i32;
    }
    /* Read 2 bytes from the buffer as the marker size */
    l_marker_size = opj_read_bytes(&l_header_data[..2], 2_u32);
    /* Check marker size for SOT Marker */
    if l_marker_size != 10u32 {
      event_msg!(p_manager, EVT_ERROR, "Inconsistent marker size\n",);
      return 0i32;
    }
    l_marker_size = (l_marker_size as core::ffi::c_uint).wrapping_sub(2u32);
    if opj_stream_read_data(
      p_stream,
      &mut l_header_data[..l_marker_size as usize],
      p_manager,
    ) != l_marker_size as usize
    {
      event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
      return 0i32;
    }
    if opj_j2k_get_sot_values(
      &l_header_data[..l_marker_size as usize],
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
    l_tot_len = (l_tot_len as core::ffi::c_uint).wrapping_sub(12u32);
    /* look for next SOT marker */
    if opj_stream_skip(p_stream, l_tot_len as i64, p_manager) != l_tot_len as i64 {
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
  1i32
}

pub(crate) fn opj_j2k_read_tile_header(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  tile_info: &mut TileInfo,
  p_manager: &mut EventMgr,
) -> bool {
  let mut l_current_marker = J2KMarker::SOT;
  let mut l_marker_size: u32 = 0;
  let l_nb_tiles = p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th);
  /* preconditions */

  /* Reach the End Of Codestream ?*/
  if p_j2k.m_specific_param.m_decoder.m_state == J2KState::EOC {
    l_current_marker = J2KMarker::EOC;
  } else if p_j2k.m_specific_param.m_decoder.m_state != J2KState::TPHSOT {
    return false;
  }
  /* We need to encounter a SOT marker (a new tile-part header) */
  /* Read the codestream until we can decode or reach EOC */
  while !p_j2k.m_specific_param.m_decoder.m_can_decode && l_current_marker != J2KMarker::EOC {
    if p_j2k.m_specific_param.m_decoder.m_num_intersecting_tile_parts > 0
      && p_j2k
        .m_specific_param
        .m_decoder
        .m_idx_intersecting_tile_parts
        < p_j2k.m_specific_param.m_decoder.m_num_intersecting_tile_parts
    {
      /* The TLM index planned the SOT positions of the intersecting
      tile-parts: seek straight to the next one (upstream #1538) */
      let next_tp_sot_pos = p_j2k
        .m_specific_param
        .m_decoder
        .m_intersecting_tile_parts_offset[p_j2k
          .m_specific_param
          .m_decoder
          .m_idx_intersecting_tile_parts as usize];
      p_j2k.m_specific_param.m_decoder.m_idx_intersecting_tile_parts += 1;
      if opj_stream_seek(p_stream, next_tp_sot_pos, p_manager) == 0 {
        event_msg!(p_manager, EVT_ERROR, "Problem with seek function\n",);
        return false;
      }
      if opj_stream_read_data(
        p_stream,
        &mut p_j2k.m_specific_param.m_decoder.m_header_data[..2],
        p_manager,
      ) != 2
      {
        event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
        return false;
      }
      l_current_marker =
        J2KMarker::from_buffer(&p_j2k.m_specific_param.m_decoder.m_header_data);
      if l_current_marker != J2KMarker::SOT {
        event_msg!(p_manager, EVT_ERROR, "Did not get expected SOT marker\n",);
        return false;
      }
    }
    /* Try to read until the Start Of Data is detected */
    while l_current_marker != J2KMarker::SOD {
      if opj_stream_get_number_byte_left(p_stream) == 0i64 {
        p_j2k.m_specific_param.m_decoder.m_state = J2KState::NEOC;
        break;
      } else {
        /* Try to read 2 bytes (the marker size) from stream and copy them into the buffer */
        if opj_stream_read_data(
          p_stream,
          &mut p_j2k.m_specific_param.m_decoder.m_header_data[..2],
          p_manager,
        ) != 2
        {
          event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
          return false;
        }
        /* Read 2 bytes from the buffer as the marker size */
        l_marker_size = opj_read_bytes(&p_j2k.m_specific_param.m_decoder.m_header_data, 2_u32);
        /* Check marker size (does not include marker ID but includes marker size) */
        if l_marker_size < 2u32 {
          event_msg!(p_manager, EVT_ERROR, "Inconsistent marker size\n",);
          return false;
        }
        /* cf. https://code.google.com/p/openjpeg/issues/detail?id=226 */
        if l_current_marker == J2KMarker::UNK(0x8080u32)
          && opj_stream_get_number_byte_left(p_stream) == 0i64
        {
          p_j2k.m_specific_param.m_decoder.m_state = J2KState::NEOC;
          break;
        } else {
          /* Subtract the marker ID size already read from the SOT length */
          if p_j2k.m_specific_param.m_decoder.m_state & J2KState::TPH != J2KState::NONE
            && p_j2k.m_specific_param.m_decoder.m_sot_length != 0
          {
            if p_j2k.m_specific_param.m_decoder.m_sot_length < l_marker_size.wrapping_add(2u32) {
              /* upstream #1561 */
              event_msg!(
                p_manager,
                EVT_ERROR,
                "Sot length is less than marker size + marker ID\n",
              );
              return false;
            }
            p_j2k.m_specific_param.m_decoder.m_sot_length =
              (p_j2k.m_specific_param.m_decoder.m_sot_length as core::ffi::c_uint)
                .wrapping_sub(l_marker_size.wrapping_add(2u32))
          } /* Subtract the size of the marker ID already read */
          l_marker_size = (l_marker_size as core::ffi::c_uint).wrapping_sub(2u32);
          /* Get the marker handler from the marker ID */
          let l_marker_handler = l_current_marker;
          /* Check if the marker is known and if it is the right place to find it */
          if p_j2k.m_specific_param.m_decoder.m_state & l_marker_handler.states() == J2KState::NONE
          {
            event_msg!(
              p_manager,
              EVT_ERROR,
              "Marker is not compliant with its position\n",
            );
            return false;
          }
          /* Note (work plan): unknown markers in tile-part headers are rejected */
          /* Check if the marker size is compatible with the header data size */
          if l_marker_size as usize > p_j2k.m_specific_param.m_decoder.m_header_data.len() {
            /* If we are here, this means we consider this marker as known & we will read it */
            /* Check enough bytes left in stream before allocation */
            if l_marker_size as i64 > opj_stream_get_number_byte_left(p_stream) {
              event_msg!(
                p_manager,
                EVT_ERROR,
                "Marker size inconsistent with stream length\n",
              );
              return false;
            }
          }
          let l_header_data = p_j2k.m_specific_param.m_decoder.header_data(l_marker_size);
          /* Try to read the rest of the marker segment from stream and copy them into the buffer */
          if opj_stream_read_data(p_stream, l_header_data, p_manager) != l_marker_size as usize {
            event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
            return false;
          }
          /* Read the marker segment with the correct marker handler */
          let header_data =
            p_j2k.m_specific_param.m_decoder.m_header_data[..l_marker_size as usize].to_vec();
          if l_marker_handler.handler(p_j2k, &header_data, p_manager) == 0 {
            p_j2k.record_error_context(
              l_current_marker.as_u32(),
              opj_stream_tell(p_stream).max(0) as u64,
            );
            event_msg!(
              p_manager,
              EVT_ERROR,
              "Fail to read the current marker segment ({:X})\n",
              l_current_marker.as_u32(),
            );
            return false;
          }
          /* Add the marker to the codestream index*/
          if 0i32
            == opj_j2k_add_tlmarker(
              p_j2k.m_current_tile_number,
              p_j2k.cstr_index.as_deref_mut().expect("cstr_index"),
              l_marker_handler,
              (opj_stream_tell(p_stream) as u32)
                .wrapping_sub(l_marker_size)
                .wrapping_sub(4u32) as i64,
              l_marker_size.wrapping_add(4u32),
            )
          {
            event_msg!(p_manager, EVT_ERROR, "Not enough memory to add tl marker\n",);
            return false;
          }
          /* Keep the position of the last SOT marker read */
          if l_marker_handler == J2KMarker::SOT {
            let mut sot_pos = (opj_stream_tell(p_stream) as u32)
              .wrapping_sub(l_marker_size)
              .wrapping_sub(4u32);
            if sot_pos as i64 > p_j2k.m_specific_param.m_decoder.m_last_sot_read_pos {
              p_j2k.m_specific_param.m_decoder.m_last_sot_read_pos = sot_pos as i64
            }
          }
          if p_j2k.m_specific_param.m_decoder.m_skip_data {
            /* Skip the rest of the tile part header*/
            if opj_stream_skip(
              p_stream,
              p_j2k.m_specific_param.m_decoder.m_sot_length as i64,
              p_manager,
            ) != p_j2k.m_specific_param.m_decoder.m_sot_length as i64
            {
              event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
              return false;
            }
            l_current_marker = J2KMarker::SOD
          /* Normally we reached a SOD */
          } else {
            /* Try to read 2 bytes (the next marker ID) from stream and copy them into the buffer*/
            if opj_stream_read_data(
              p_stream,
              &mut p_j2k.m_specific_param.m_decoder.m_header_data[..2],
              p_manager,
            ) != 2
            {
              event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
              return false;
            }
            /* Read 2 bytes from the buffer as the new marker ID */
            l_current_marker =
              J2KMarker::from_buffer(&p_j2k.m_specific_param.m_decoder.m_header_data);
          }
        }
      }
    }
    if opj_stream_get_number_byte_left(p_stream) == 0i64
      && p_j2k.m_specific_param.m_decoder.m_state == J2KState::NEOC
    {
      break;
    }
    /* If we didn't skip data before, we need to read the SOD marker*/
    if !p_j2k.m_specific_param.m_decoder.m_skip_data {
      /* Read the SOD marker */
      if opj_j2k_read_sod(p_j2k, p_stream, p_manager) == 0 {
        return false;
      }
      /* Check if we can use the TLM index to access the next tile-part
      (upstream #1538) */
      if !p_j2k.m_specific_param.m_decoder.m_can_decode
        && p_j2k.m_specific_param.m_decoder.m_tile_ind_to_dec >= 0
        && p_j2k.m_current_tile_number == p_j2k.m_specific_param.m_decoder.m_tile_ind_to_dec as u32
        && !p_j2k.m_specific_param.m_decoder.m_tlm.m_is_invalid
        && opj_stream_has_seek(p_stream) != 0
      {
        let l_tcp = &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize];
        let tile = &p_j2k
          .cstr_index
          .as_deref()
          .expect("cstr_index")
          .tiles[p_j2k.m_current_tile_number as usize];
        if l_tcp.m_nb_tile_parts == tile.nb_tps
          && l_tcp.m_current_tile_part_number as u32 + 1 < l_tcp.m_nb_tile_parts
        {
          let next_tp_sot_pos =
            tile.tp_index[l_tcp.m_current_tile_part_number as usize + 1].start_pos;
          if next_tp_sot_pos != opj_stream_tell(p_stream)
            && opj_stream_seek(p_stream, next_tp_sot_pos, p_manager) == 0
          {
            event_msg!(p_manager, EVT_ERROR, "Problem with seek function\n",);
            return false;
          }
          /* Validate that the expected SOT marker is there */
          if opj_stream_read_data(
            p_stream,
            &mut p_j2k.m_specific_param.m_decoder.m_header_data[..2],
            p_manager,
          ) != 2
          {
            event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
            return false;
          }
          l_current_marker =
            J2KMarker::from_buffer(&p_j2k.m_specific_param.m_decoder.m_header_data);
          if l_current_marker != J2KMarker::SOT {
            event_msg!(p_manager, EVT_ERROR, "Did not get expected SOT marker\n",);
            return false;
          }
          continue;
        }
      }
      if p_j2k.m_specific_param.m_decoder.m_can_decode
        && !p_j2k
          .m_specific_param
          .m_decoder
          .m_nb_tile_parts_correction_checked
      {
        /* Issue 254 */
        let mut l_correction_needed: OPJ_BOOL = 0;
        p_j2k
          .m_specific_param
          .m_decoder
          .m_nb_tile_parts_correction_checked = true;
        /* Skip the correction hack when a single tile part is declared:
        it hurts performance on files with many single-part tiles
        (upstream #1560) */
        if p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize].m_nb_tile_parts == 1u32 {
        } else if opj_j2k_need_nb_tile_parts_correction(
          p_stream,
          p_j2k.m_current_tile_number,
          &mut l_correction_needed,
          p_manager,
        ) == 0
        {
          event_msg!(
            p_manager,
            EVT_ERROR,
            "opj_j2k_apply_nb_tile_parts_correction error\n",
          );
          return false;
        }
        if l_correction_needed != 0 {
          let mut l_tile_no: u32 = 0;
          p_j2k.m_specific_param.m_decoder.m_can_decode = false;
          p_j2k.m_specific_param.m_decoder.m_nb_tile_parts_correction = true;
          /* correct tiles */
          l_tile_no = 0u32;
          while l_tile_no < l_nb_tiles {
            if p_j2k.m_cp.tcps[l_tile_no as usize].m_nb_tile_parts != 0u32 {
              p_j2k.m_cp.tcps[l_tile_no as usize].m_nb_tile_parts = p_j2k.m_cp.tcps
                [l_tile_no as usize]
                .m_nb_tile_parts
                .wrapping_add(1u32)
            }
            l_tile_no += 1;
          }
          event_msg!(
            p_manager,
            EVT_WARNING,
            "Non conformant codestream TPsot==TNsot.\n",
          );
        }
      }
    } else {
      /* Indicate we will try to read a new tile-part header*/
      p_j2k.m_specific_param.m_decoder.m_skip_data = false;
      p_j2k.m_specific_param.m_decoder.m_can_decode = false;
      p_j2k.m_specific_param.m_decoder.m_state = J2KState::TPHSOT
    }
    if p_j2k.m_specific_param.m_decoder.m_can_decode {
      continue;
    }
    /* Try to read 2 bytes (the next marker ID) from stream and copy them into the buffer */
    if opj_stream_read_data(
      p_stream,
      &mut p_j2k.m_specific_param.m_decoder.m_header_data[..2],
      p_manager,
    ) != 2
    {
      /* Deal with likely non conformant SPOT6 files, where the last */
      /* row of tiles have TPsot == 0 and TNsot == 0, and missing EOC, */
      /* but no other tile-parts were found. */
      if p_j2k.m_current_tile_number.wrapping_add(1u32) == l_nb_tiles {
        let mut l_tile_no_0: u32 = 0;
        l_tile_no_0 = 0u32;
        while l_tile_no_0 < l_nb_tiles {
          if p_j2k.m_cp.tcps[l_tile_no_0 as usize].m_current_tile_part_number == 0i32
            && p_j2k.m_cp.tcps[l_tile_no_0 as usize].m_nb_tile_parts == 0u32
          {
            break;
          }
          l_tile_no_0 += 1;
        }
        if l_tile_no_0 < l_nb_tiles {
          event_msg!(p_manager, EVT_INFO,
                                  "Tile {} has TPsot == 0 and TNsot == 0, but no other tile-parts were found. EOC is also missing.\n",
                                  l_tile_no_0);
          p_j2k.m_current_tile_number = l_tile_no_0;
          l_current_marker = J2KMarker::EOC;
          p_j2k.m_specific_param.m_decoder.m_state = J2KState::EOC;
          break;
        }
      }
      event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
      return false;
    } else {
      /* Read 2 bytes from buffer as the new marker ID */
      l_current_marker = J2KMarker::from_buffer(&p_j2k.m_specific_param.m_decoder.m_header_data);
    }
  }
  /* Current marker is the EOC marker ?*/
  if l_current_marker == J2KMarker::EOC && p_j2k.m_specific_param.m_decoder.m_state != J2KState::EOC
  {
    p_j2k.m_current_tile_number = 0_u32;
    p_j2k.m_specific_param.m_decoder.m_state = J2KState::EOC
  }
  /* Deal with tiles that have a single tile-part with TPsot == 0 and TNsot == 0 */
  if !p_j2k.m_specific_param.m_decoder.m_can_decode {
    while p_j2k.m_current_tile_number < l_nb_tiles
      && p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize]
        .m_data
        .is_empty()
    {
      p_j2k.m_current_tile_number = p_j2k.m_current_tile_number.wrapping_add(1);
    }
    if p_j2k.m_current_tile_number == l_nb_tiles {
      tile_info.go_on = false;
      return true;
    }
  }
  if opj_j2k_merge_ppt(
    &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize],
    p_manager,
  ) == 0
  {
    event_msg!(p_manager, EVT_ERROR, "Failed to merge PPT data\n",);
    return false;
  }
  /* Decoder resource limit (Rust core API): bound the tile buffers before
  opj_tcd_init_decode_tile allocates them.  tdx*tdy is the worst-case tile
  area (edge tiles are smaller), so this pre-check is conservative. */
  {
    /* Always runs: an unset limit falls back to the built-in 1 GiB ceiling
    so a corrupt tile header cannot drive a huge allocation.  The bound is
    the ACTUAL per-tile area: the tcd clamps each tile to the image
    extents (a header may declare a raw tdx/tdy far larger than the
    image - cf. huge-tile-size.jp2). */
    let max_mem = p_j2k.decode_limits.effective_max_mem();
    let numcomps = p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .numcomps;
    let img = p_j2k.m_private_image.as_deref().expect("m_private_image");
    let image_pixels = (img.x1.wrapping_sub(img.x0) as u64)
      .wrapping_mul(img.y1.wrapping_sub(img.y0) as u64);
    let cp_ref = &p_j2k.m_cp;
    let raw_tile_pixels = (cp_ref.tdx as u64).wrapping_mul(cp_ref.tdy as u64);
    let tile_pixels = raw_tile_pixels.min(image_pixels);
    let tile_bytes = tile_pixels
      .wrapping_mul(core::mem::size_of::<i32>() as u64)
      .wrapping_mul(numcomps as u64);
    if tile_bytes > max_mem {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Tile data {} bytes exceeds the decoder memory limit {} bytes\n",
        tile_bytes,
        max_mem,
      );
      return false;
    }
  }
  if opj_tcd_init_decode_tile(
    &mut p_j2k.m_tcd,
    p_j2k.m_current_tile_number,
    p_manager,
    p_j2k
      .m_private_image
      .as_deref_mut()
      .expect("m_private_image"),
    &mut p_j2k.m_cp,
  ) == 0
  {
    event_msg!(p_manager, EVT_ERROR, "Cannot decode tile, memory error\n",);
    return false;
  }
  event_msg!(
    p_manager,
    EVT_INFO,
    "Header of tile {} / {} has been read.\n",
    p_j2k.m_current_tile_number.wrapping_add(1u32),
    p_j2k.m_cp.th.wrapping_mul(p_j2k.m_cp.tw),
  );
  tile_info.index = p_j2k.m_current_tile_number;
  tile_info.go_on = true;
  /* For internal use in j2k.c, we don't need this */
  /* This is just needed for folks using the opj_read_tile_header() / opj_decode_tile_data() combo */
  if let Some(data_size) = &mut tile_info.data_size {
    *data_size = opj_tcd_get_decoded_tile_size(
      &mut p_j2k.m_tcd,
      0i32,
      p_j2k
        .m_private_image
        .as_deref_mut()
        .expect("m_private_image"),
    );
    if *data_size == (2147483647u32).wrapping_mul(2u32).wrapping_add(1u32) {
      return false;
    }
  }
  tile_info.x0 = p_j2k.m_tcd.tcd_image.tiles.x0;
  tile_info.y0 = p_j2k.m_tcd.tcd_image.tiles.y0;
  tile_info.x1 = p_j2k.m_tcd.tcd_image.tiles.x1;
  tile_info.y1 = p_j2k.m_tcd.tcd_image.tiles.y1;
  tile_info.nb_comps = p_j2k.m_tcd.tcd_image.tiles.numcomps;
  p_j2k.m_specific_param.m_decoder.m_state |= J2KState::DATA;
  true
}
pub(crate) fn opj_j2k_decode_tile(
  p_j2k: &mut J2k,
  p_tile_index: u32,
  p_data: Option<&mut [u8]>,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_current_marker = J2KMarker::UNK(0);
  let mut l_data: [u8; 2] = [0; 2];
  /* preconditions */

  if p_j2k.m_specific_param.m_decoder.m_state & J2KState::DATA == J2KState::NONE
    || p_tile_index != p_j2k.m_current_tile_number
  {
    return 0i32;
  }
  let mut l_tile_data;
  let l_tile_data_size;
  {
    let l_tcp = &mut p_j2k.m_cp.tcps[p_tile_index as usize];
    if l_tcp.m_data.is_empty() {
      opj_j2k_tcp_destroy(l_tcp);
      return 0i32;
    }
    /* The C passes m_data_size (not the buffer length: the buffer carries
    a 2-byte fake-0xFFFF margin the tile decoder must not see as data). */
    l_tile_data_size = l_tcp.m_data_size as usize;
    // Take the tile data out so the `&mut p_j2k.m_cp` borrow below does not
    // conflict with it; the decode reads through the local buffer and the
    // data is destroyed afterwards anyway (opj_j2k_tcp_data_destroy).
    l_tile_data = std::mem::take(&mut l_tcp.m_data);
  }
  /* When using the opj_read_tile_header / opj_decode_tile_data API */
  /* such as in test_tile_decoder, m_output_image is NULL, so fall back */
  /* to the full image dimension. This is a bit surprising that */
  /* opj_set_decode_area() is only used to determine intersecting tiles, */
  /* but full tile decoding is done */
  let (l_bounds_x0, l_bounds_y0, l_bounds_x1, l_bounds_y1) =
    if let Some(image) = p_j2k.m_output_image.as_ref() {
      (image.x0, image.y0, image.x1, image.y1)
    } else {
      let image = p_j2k.m_private_image.as_deref().expect("m_private_image");
      (image.x0, image.y0, image.x1, image.y1)
    };
  if opj_tcd_decode_tile(
    &mut p_j2k.m_tcd,
    l_bounds_x0,
    l_bounds_y0,
    l_bounds_x1,
    l_bounds_y1,
    p_j2k.m_specific_param.m_decoder.m_numcomps_to_decode,
    &p_j2k.m_specific_param.m_decoder.m_comps_indices_to_decode[..],
    &mut l_tile_data[..l_tile_data_size],
    p_tile_index,
    p_manager,
    p_j2k
      .m_private_image
      .as_deref_mut()
      .expect("m_private_image"),
    &mut p_j2k.m_cp,
  ) == 0
  {
    opj_j2k_tcp_destroy(&mut p_j2k.m_cp.tcps[p_tile_index as usize]);
    p_j2k.m_specific_param.m_decoder.m_state |= J2KState::ERR;
    event_msg!(p_manager, EVT_ERROR, "Failed to decode.\n",);
    return 0i32;
  }
  /* p_data can be set to NULL when the call will take care of using */
  /* itself the TCD data. This is typically the case for whole single */
  /* tile decoding optimization. */
  if let Some(p_data) = p_data {
    if opj_tcd_update_tile_data(
      &mut p_j2k.m_tcd,
      p_data,
      p_j2k
        .m_private_image
        .as_deref_mut()
        .expect("m_private_image"),
    ) == 0
    {
      return 0i32;
    }
    /* To avoid to destroy the tcp which can be useful when we try to decode a tile decoded before (cf j2k_random_tile_access)
     * we destroy just the data which will be re-read in read_tile_header*/
    /*opj_j2k_tcp_destroy(l_tcp);
    p_j2k->m_tcd->tcp = 0;*/
    opj_j2k_tcp_data_destroy(&mut p_j2k.m_cp.tcps[p_tile_index as usize]);
  } else {
    /* Strip decoding keeps the tile data for the next decode-area pass
     * (the C skips tcp_data_destroy when p_data is NULL). */
    p_j2k.m_cp.tcps[p_tile_index as usize].m_data = l_tile_data;
  }
  p_j2k.m_specific_param.m_decoder.m_can_decode = false;
  p_j2k.m_specific_param.m_decoder.m_state &= !(J2KState::DATA);
  if opj_stream_get_number_byte_left(p_stream) == 0i64
    && p_j2k.m_specific_param.m_decoder.m_state == J2KState::NEOC
  {
    return 1i32;
  }
  if p_j2k.m_specific_param.m_decoder.m_state != J2KState::EOC {
    if opj_stream_read_data(p_stream, &mut l_data[..2], p_manager) != 2 {
      /* C parity (j2k.c:10191-10194): in lenient mode a codestream that
      runs out after the tile data still delivers the decoded tiles - the
      strict mode keeps the hard failure. */
      let strict = p_j2k.m_cp.strict != 0;
      event_msg!(
        p_manager,
        if strict { EVT_ERROR } else { EVT_WARNING },
        "Stream too short\n",
      );
      return if strict { 0i32 } else { 1i32 };
    }
    l_current_marker = J2KMarker::from_buffer(&l_data);
    if l_current_marker == J2KMarker::EOC {
      p_j2k.m_current_tile_number = 0_u32;
      p_j2k.m_specific_param.m_decoder.m_state = J2KState::EOC
    } else if l_current_marker == J2KMarker::UNK(0x8080u32) {
      /* cf. https://code.google.com/p/openjpeg/issues/detail?id=226 */
      if opj_stream_get_number_byte_left(p_stream) == 2i64 {
        p_j2k.m_specific_param.m_decoder.m_state = J2KState::NEOC;
        event_msg!(
          p_manager,
          EVT_WARNING,
          "Expected EOC or SOT marker, got unknown marker 0x8080.  Stream does not end with EOC\n",
        );
        return 1i32;
      }
    } else if l_current_marker != J2KMarker::SOT {
      if opj_stream_get_number_byte_left(p_stream) == 0i64 {
        p_j2k.m_specific_param.m_decoder.m_state = J2KState::NEOC;
        event_msg!(p_manager, EVT_WARNING, "Stream does not end with EOC\n",);
        return 1i32;
      }
      log::error!(
        "Expected SOT marker: got {:?}, bytes left: {}",
        l_current_marker,
        opj_stream_get_number_byte_left(p_stream)
      );
      event_msg!(p_manager, EVT_ERROR, "Stream too short, expected SOT\n",);
      return 0i32;
    }
  }
  1i32
}
fn opj_j2k_update_image_data(
  p_tcd: &mut Tcd,
  p_output_image: &mut CoreImage,
  p_image_src: &mut CoreImage,
  max_mem: u64,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let whole_tile = p_tcd.whole_tile_decoding != 0;
  let img_comps_src = &p_image_src.comps[..];
  let numcomps = p_image_src.numcomps as usize;
  /* Decoder resource limit (Rust core API): the destination component
  buffers are sized from the image dimensions - check the total before the
  first allocation (an unset limit falls back to the built-in 1 GiB
  ceiling). */
  {
    let mut total: u64 = 0;
    for comp in &p_output_image.comps {
      total = total.saturating_add(
        (comp.w as u64)
          .wrapping_mul(comp.h as u64)
          .wrapping_mul(core::mem::size_of::<i32>() as u64),
      );
    }
    if total > max_mem {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Output image component data {} bytes exceeds the decoder memory limit {} bytes\\n",
        total,
        max_mem,
      );
      return 0i32;
    }
  }
  let results: Vec<OPJ_BOOL> = par::zip_map_collect(
    &mut p_tcd.tcd_image.tiles.comps,
    &mut p_output_image.comps,
    numcomps,
    |i, l_tilec_i, dest| update_image_data_comp(l_tilec_i, &img_comps_src[i], dest, whole_tile),
  );
  if results.contains(&0) {
    return 0i32;
  }
  1i32
}

fn update_image_data_comp(
  l_tilec_i: &mut TcdTileComp,
  l_img_comp_src: &CoreImageComp,
  dest: &mut CoreImageComp,
  whole_tile: bool,
) -> OPJ_BOOL {
  let mut res_x0: i32 = 0;
  let mut res_x1: i32 = 0;
  let mut res_y0: i32 = 0;
  let mut res_y1: i32 = 0;
  let mut src_data_stride: u32 = 0;
  let mut l_width_src: u32 = 0;
  let mut l_height_src: u32 = 0;
  let mut l_width_dest: u32 = 0;
  let mut l_height_dest: u32 = 0;
  let mut l_offset_x0_src: i32 = 0;
  let mut l_offset_y0_src: i32 = 0;
  let mut l_offset_x1_src: i32 = 0;
  let mut l_offset_y1_src: i32 = 0;
  let mut l_start_offset_src: usize = 0;
  let mut l_start_x_dest: u32 = 0;
  let mut l_start_y_dest: u32 = 0;
  let mut l_x0_dest: u32 = 0;
  let mut l_y0_dest: u32 = 0;
  let mut l_x1_dest: u32 = 0;
  let mut l_y1_dest: u32 = 0;
  let mut l_start_offset_dest: usize = 0;
  let mut j: u32 = 0;
  /* Copy info from decoded comp image to output image */
  dest.resno_decoded = l_img_comp_src.resno_decoded;
  if whole_tile {
    let l_res = &l_tilec_i.resolutions[l_img_comp_src.resno_decoded as usize];
    res_x0 = l_res.x0;
    res_y0 = l_res.y0;
    res_x1 = l_res.x1;
    res_y1 = l_res.y1;
    src_data_stride = (l_tilec_i.resolutions[l_tilec_i.minimum_num_resolutions as usize - 1].x1
      - l_tilec_i.resolutions[l_tilec_i.minimum_num_resolutions as usize - 1].x0)
      as u32;
    !l_tilec_i.data.is_null()
  } else {
    let l_res = &l_tilec_i.resolutions[l_img_comp_src.resno_decoded as usize];
    res_x0 = l_res.win_x0 as i32;
    res_y0 = l_res.win_y0 as i32;
    res_x1 = l_res.win_x1 as i32;
    res_y1 = l_res.win_y1 as i32;
    src_data_stride = l_res.win_x1.wrapping_sub(l_res.win_x0);
    !l_tilec_i.data_win.is_null()
  };
  let src_available = if whole_tile {
    !l_tilec_i.data.is_null()
  } else {
    !l_tilec_i.data_win.is_null()
  };
  if src_available {
    l_width_src = (res_x1 - res_x0) as u32;
    l_height_src = (res_y1 - res_y0) as u32;
    /* Current tile component size*/
    /* Border of the current output component*/
    l_x0_dest = opj_uint_ceildivpow2(dest.x0, dest.factor); /* can't overflow given that image->x1 is uint32 */
    l_y0_dest = opj_uint_ceildivpow2(dest.y0, dest.factor);
    l_x1_dest = l_x0_dest.wrapping_add(dest.w);
    l_y1_dest = l_y0_dest.wrapping_add(dest.h);
    /*-----*/
    /* Compute the area (l_offset_x0_src, l_offset_y0_src, l_offset_x1_src, l_offset_y1_src)
     * of the input buffer (decoded tile component) which will be move
     * in the output buffer. Compute the area of the output buffer (l_start_x_dest,
     * l_start_y_dest, l_width_dest, l_height_dest)  which will be modified
     * by this input area.
     * */

    assert!(res_x0 >= 0i32);
    assert!(res_x1 >= 0i32);
    if l_x0_dest < res_x0 as u32 {
      l_start_x_dest = (res_x0 as u32).wrapping_sub(l_x0_dest);
      l_offset_x0_src = 0i32;
      if l_x1_dest >= res_x1 as u32 {
        l_width_dest = l_width_src;
        l_offset_x1_src = 0i32
      } else {
        l_width_dest = l_x1_dest.wrapping_sub(res_x0 as u32);
        l_offset_x1_src = l_width_src.wrapping_sub(l_width_dest) as i32
      }
    } else {
      l_start_x_dest = 0u32;
      l_offset_x0_src = l_x0_dest as i32 - res_x0;
      if l_x1_dest >= res_x1 as u32 {
        l_width_dest = l_width_src.wrapping_sub(l_offset_x0_src as u32);
        l_offset_x1_src = 0i32
      } else {
        l_width_dest = dest.w;
        l_offset_x1_src = res_x1 - l_x1_dest as i32
      }
    }
    if l_y0_dest < res_y0 as u32 {
      l_start_y_dest = (res_y0 as u32).wrapping_sub(l_y0_dest);
      l_offset_y0_src = 0i32;
      if l_y1_dest >= res_y1 as u32 {
        l_height_dest = l_height_src;
        l_offset_y1_src = 0i32
      } else {
        l_height_dest = l_y1_dest.wrapping_sub(res_y0 as u32);
        l_offset_y1_src = l_height_src.wrapping_sub(l_height_dest) as i32
      }
    } else {
      l_start_y_dest = 0u32;
      l_offset_y0_src = l_y0_dest as i32 - res_y0;
      if l_y1_dest >= res_y1 as u32 {
        l_height_dest = l_height_src.wrapping_sub(l_offset_y0_src as u32);
        l_offset_y1_src = 0i32
      } else {
        l_height_dest = dest.h;
        l_offset_y1_src = res_y1 - l_y1_dest as i32
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
    if (l_width_dest as i32) < 0i32 || (l_height_dest as i32) < 0i32 {
      return 0i32;
    }
    /*-----*/
    /* Compute the input buffer offset */
    l_start_offset_src = (l_offset_x0_src as usize)
      .wrapping_add((l_offset_y0_src as usize).wrapping_mul(src_data_stride as usize));
    /* Compute the output buffer offset */
    l_start_offset_dest = (l_start_x_dest as usize)
      .wrapping_add((l_start_y_dest as usize).wrapping_mul(dest.w as usize));
    /* Allocate output component buffer if necessary */
    if dest.data.is_empty()
      && l_start_offset_src == 0
      && l_start_offset_dest == 0
      && src_data_stride == dest.w
      && l_width_dest == dest.w
      && l_height_dest == dest.h
    {
      /* If the final image matches the tile buffer, then borrow it */
      /* directly to save a copy */
      let len = (dest.w as usize).wrapping_mul(dest.h as usize);
      if whole_tile {
        dest.data = l_tilec_i.data.take_vec_i32(len);
      } else {
        dest.data = l_tilec_i.data_win.take_vec_i32(len);
      }
    } else {
      if dest.data.is_empty() {
        let l_width = dest.w as usize;
        let l_height = dest.h as usize;
        if l_height == 0
          || l_width > (usize::MAX).wrapping_div(l_height)
          || l_width.wrapping_mul(l_height) > (usize::MAX).wrapping_div(core::mem::size_of::<i32>())
        {
          /* would overflow */
          return 0i32;
        }
        dest.data = vec![0i32; l_width * l_height];
      }
      /* Move the output buffer to the first place where we will write*/
      let src_len = (l_offset_y0_src as usize + l_height_dest as usize) * src_data_stride as usize;
      let src_is_f32 = if whole_tile {
        l_tilec_i.data.is_f32()
      } else {
        l_tilec_i.data_win.is_f32()
      };
      let dest_w = dest.w as usize;
      if src_is_f32 {
        /* 9-7: the buffer holds f32 words; after the dc-shift they carry
        the exact int bit patterns (the C's shared-array reinterpretation). */
        let src = if whole_tile {
          l_tilec_i.data.as_mut_f32(src_len)
        } else {
          l_tilec_i.data_win.as_mut_f32(src_len)
        };
        j = 0_u32;
        while j < l_height_dest {
          let src_row = l_start_offset_src + j as usize * src_data_stride as usize;
          let dest_row = l_start_offset_dest + j as usize * dest_w;
          let mut k = 0usize;
          while k < l_width_dest as usize {
            dest.data[dest_row + k] = src[src_row + k].to_bits() as i32;
            k += 1;
          }
          j += 1;
        }
      } else {
        let src = if whole_tile {
          l_tilec_i.data.as_mut_i32(src_len)
        } else {
          l_tilec_i.data_win.as_mut_i32(src_len)
        };
        j = 0_u32;
        while j < l_height_dest {
          let src_row = l_start_offset_src + j as usize * src_data_stride as usize;
          let dest_row = l_start_offset_dest + j as usize * dest_w;
          dest.data[dest_row..dest_row + l_width_dest as usize]
            .copy_from_slice(&src[src_row..src_row + l_width_dest as usize]);
          j += 1;
        }
      }
    }
  }
  1i32
}

pub(crate) fn opj_j2k_update_image_dimensions(p_image: &mut CoreImage, p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut it_comp: u32 = 0;
  let mut l_comp_x1: i32 = 0;
  let mut l_comp_y1: i32 = 0;
  let img_x0 = p_image.x0;
  let img_y0 = p_image.y0;
  let img_x1 = p_image.x1;
  let img_y1 = p_image.y1;
  it_comp = 0_u32;
  while it_comp < p_image.numcomps {
    let mut l_h: i32 = 0;
    let mut l_w: i32 = 0;
    if img_x0 > 2147483647_u32
      || img_y0 > 2147483647_u32
      || img_x1 > 2147483647_u32
      || img_y1 > 2147483647_u32
    {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Image coordinates above INT_MAX are not supported\n",
      );
      return 0i32;
    }

    let comps = &mut p_image.comps;
    let l_img_comp = &mut comps[it_comp as usize];
    l_img_comp.x0 = opj_uint_ceildiv(img_x0, l_img_comp.dx);
    l_img_comp.y0 = opj_uint_ceildiv(img_y0, l_img_comp.dy);
    l_comp_x1 = opj_int_ceildiv(img_x1 as i32, l_img_comp.dx as i32);
    l_comp_y1 = opj_int_ceildiv(img_y1 as i32, l_img_comp.dy as i32);

    l_w = opj_int_ceildivpow2(l_comp_x1, l_img_comp.factor as i32)
      - opj_int_ceildivpow2(l_img_comp.x0 as i32, l_img_comp.factor as i32);
    if l_w < 0i32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Size x of the decoded component image is incorrect (comp[{}].w={}).\n",
        it_comp,
        l_w,
      );
      return 0i32;
    }
    l_img_comp.w = l_w as u32;
    l_h = opj_int_ceildivpow2(l_comp_y1, l_img_comp.factor as i32)
      - opj_int_ceildivpow2(l_img_comp.y0 as i32, l_img_comp.factor as i32);
    if l_h < 0i32 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Size y of the decoded component image is incorrect (comp[{}].h={}).\n",
        it_comp,
        l_h,
      );
      return 0i32;
    }
    l_img_comp.h = l_h as u32;
    it_comp += 1;
  }
  1i32
}

pub(crate) fn opj_j2k_set_decoded_components(
  p_j2k: &mut J2k,
  compenents: &[u32],
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let numcomps = compenents.len() as u32;
  if p_j2k.m_private_image.is_none() {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "opj_read_header() should be called before opj_set_decoded_components().\n",
    );
    return 0i32;
  }
  let mut already_mapped = std::collections::BTreeSet::new();
  for comp in compenents {
    if *comp
      >= p_j2k
        .m_private_image
        .as_deref()
        .expect("m_private_image")
        .numcomps
    {
      event_msg!(p_manager, EVT_ERROR, "Invalid component index: {}\n", *comp,);
      return 0i32;
    }
    if !already_mapped.insert(*comp) {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Component index {} used several times\n",
        *comp,
      );
      return 0i32;
    }
  }
  p_j2k.m_specific_param.m_decoder.m_comps_indices_to_decode = compenents.to_vec();
  p_j2k.m_specific_param.m_decoder.m_numcomps_to_decode = numcomps;
  1i32
}

pub(crate) fn opj_j2k_set_decode_area(
  p_j2k: &mut J2k,
  p_image: &mut CoreImage,
  p_start_x: i32,
  p_start_y: i32,
  p_end_x: i32,
  p_end_y: i32,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let l_cp = &p_j2k.m_cp;
  let mut l_image = p_j2k
    .m_private_image
    .as_deref_mut()
    .expect("m_private_image");
  let mut ret: OPJ_BOOL = 0;
  let mut it_comp: u32 = 0;
  if !(p_j2k.m_cp.tw == 1u32 && p_j2k.m_cp.th == 1u32 && !p_j2k.m_cp.tcps[0].m_data.is_empty()) {
    /* Check if we are read the main header */
    if p_j2k.m_specific_param.m_decoder.m_state != J2KState::TPHSOT {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Need to decode the main header before begin to decode the remaining codestream.\n",
      );
      return 0i32;
    }
  }
  /* Update the comps[].factor member of the output image with the one */
  /* of m_reduce */
  it_comp = 0_u32;
  while it_comp < p_image.numcomps {
    p_image.comps[it_comp as usize].factor = p_j2k.m_cp.m_specific_param.m_dec.m_reduce;
    it_comp += 1;
  }
  if p_start_x == 0 && p_start_y == 0 && p_end_x == 0 && p_end_y == 0 {
    event_msg!(
      p_manager,
      EVT_INFO,
      "No decoded area parameters, set the decoded area to the whole image\n",
    );
    p_j2k.m_specific_param.m_decoder.m_start_tile_x = 0_u32;
    p_j2k.m_specific_param.m_decoder.m_start_tile_y = 0_u32;
    p_j2k.m_specific_param.m_decoder.m_end_tile_x = l_cp.tw;
    p_j2k.m_specific_param.m_decoder.m_end_tile_y = l_cp.th;
    p_image.x0 = l_image.x0;
    p_image.y0 = l_image.y0;
    p_image.x1 = l_image.x1;
    p_image.y1 = l_image.y1;
    return opj_j2k_update_image_dimensions(p_image, p_manager);
  }
  /* ----- */
  /* Check if the positions provided by the user are correct */
  /* Left */
  if p_start_x < 0i32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Left position of the decoded area (region_x0={}) should be >= 0.\n",
      p_start_x,
    );
    return 0i32;
  } else if p_start_x as u32 > l_image.x1 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Left position of the decoded area (region_x0={}) is outside the image area (Xsiz={}).\n",
      p_start_x,
      l_image.x1
    );
    return 0i32;
  } else if (p_start_x as u32) < l_image.x0 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "Left position of the decoded area (region_x0={}) is outside the image area (XOsiz={}).\n",
      p_start_x,
      l_image.x0
    );
    p_j2k.m_specific_param.m_decoder.m_start_tile_x = 0_u32;
    p_image.x0 = l_image.x0
  } else {
    p_j2k.m_specific_param.m_decoder.m_start_tile_x = (p_start_x as u32)
      .wrapping_sub(l_cp.tx0)
      .wrapping_div(l_cp.tdx);
    p_image.x0 = p_start_x as u32
  }
  /* Up */
  if p_start_y < 0i32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Up position of the decoded area (region_y0={}) should be >= 0.\n",
      p_start_y,
    );
    return 0i32;
  } else if p_start_y as u32 > l_image.y1 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Up position of the decoded area (region_y0={}) is outside the image area (Ysiz={}).\n",
      p_start_y,
      l_image.y1,
    );
    return 0i32;
  } else if (p_start_y as u32) < l_image.y0 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "Up position of the decoded area (region_y0={}) is outside the image area (YOsiz={}).\n",
      p_start_y,
      l_image.y0
    );
    p_j2k.m_specific_param.m_decoder.m_start_tile_y = 0_u32;
    p_image.y0 = l_image.y0
  } else {
    p_j2k.m_specific_param.m_decoder.m_start_tile_y = (p_start_y as u32)
      .wrapping_sub(l_cp.ty0)
      .wrapping_div(l_cp.tdy);
    p_image.y0 = p_start_y as u32
  }
  /* Right */
  if p_end_x <= 0i32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Right position of the decoded area (region_x1={}) should be > 0.\n",
      p_end_x,
    );
    return 0i32;
  } else if (p_end_x as u32) < l_image.x0 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Right position of the decoded area (region_x1={}) is outside the image area (XOsiz={}).\n",
      p_end_x,
      l_image.x0
    );
    return 0i32;
  } else if p_end_x as u32 > l_image.x1 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "Right position of the decoded area (region_x1={}) is outside the image area (Xsiz={}).\n",
      p_end_x,
      l_image.x1
    );
    p_j2k.m_specific_param.m_decoder.m_end_tile_x = l_cp.tw;
    p_image.x1 = l_image.x1
  } else {
    p_j2k.m_specific_param.m_decoder.m_end_tile_x =
      opj_uint_ceildiv(p_end_x as u32 - l_cp.tx0, l_cp.tdx);
    p_image.x1 = p_end_x as u32
  }
  /* Bottom */
  if p_end_y <= 0i32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Bottom position of the decoded area (region_y1={}) should be > 0.\n",
      p_end_y,
    );
    return 0i32;
  } else if (p_end_y as u32) < l_image.y0 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Bottom position of the decoded area (region_y1={}) is outside the image area (YOsiz={}).\n",
      p_end_y,
      l_image.y0
    );
    return 0i32;
  }
  if p_end_y as u32 > l_image.y1 {
    event_msg!(
      p_manager,
      EVT_WARNING,
      "Bottom position of the decoded area (region_y1={}) is outside the image area (Ysiz={}).\n",
      p_end_y,
      l_image.y1
    );
    p_j2k.m_specific_param.m_decoder.m_end_tile_y = l_cp.th;
    p_image.y1 = l_image.y1
  } else {
    p_j2k.m_specific_param.m_decoder.m_end_tile_y =
      opj_uint_ceildiv(p_end_y as u32 - l_cp.ty0, l_cp.tdy);
    p_image.y1 = p_end_y as u32
  }
  /* ----- */
  p_j2k.m_specific_param.m_decoder.m_discard_tiles = true;
  ret = opj_j2k_update_image_dimensions(p_image, p_manager);
  if ret != 0 {
    event_msg!(
      p_manager,
      EVT_INFO,
      "Setting decoding area to {},{},{},{}\n",
      p_image.x0,
      p_image.y0,
      p_image.x1,
      p_image.y1,
    );
  }
  ret
}

impl J2k {
  pub fn new(m_is_decoder: i32) -> Self {
    Self {
      last_marker: None,
      last_pos: None,
      m_is_decoder,
      m_specific_param: Default::default(),
      m_private_image: None,
      m_output_image: None,
      m_cp: Default::default(),
      cstr_index: None,
      m_current_tile_number: 0,
      m_tcd: Tcd::new(m_is_decoder != 0),
      ihdr_w: 0,
      ihdr_h: 0,
      dump_state: 0,
      decode_limits: Default::default(),
      m_header_partial: false,
    }
  }
}

pub(crate) fn opj_j2k_create_decompress() -> Option<J2k> {
  let mut l_j2k = J2k::new(1);
  l_j2k.m_cp.m_is_decoder = true;
  /* in the absence of JP2 boxes, consider different bit depth / sign */
  /* per component is allowed */
  l_j2k.m_cp.allow_different_bit_depth_sign = true;
  /* Default to using strict mode. */
  l_j2k.m_cp.strict = 1i32;
  l_j2k.m_specific_param.m_decoder.m_default_tcp = Some(Box::default());
  l_j2k.m_specific_param.m_decoder.m_header_data = vec![0u8; 1000];
  l_j2k.m_specific_param.m_decoder.m_tile_ind_to_dec = -(1i32);
  l_j2k.m_specific_param.m_decoder.m_last_sot_read_pos = 0_i64;
  /* codestream index creation */
  l_j2k.cstr_index = Some(Box::new(CodestreamIndex::default()));
  Some(l_j2k)
}

fn opj_j2k_create_cstr_index() -> Box<CodestreamIndex> {
  Box::new(CodestreamIndex {
    markers: Vec::with_capacity(100),
    ..Default::default()
  })
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
fn opj_j2k_get_spcod_spcoc_size(p_j2k: &mut J2k, p_tile_no: u32, p_comp_no: u32) -> u32 {
  /* preconditions */
  let l_tccp = &p_j2k.m_cp.tcps[p_tile_no as usize].tccps[p_comp_no as usize];
  /* preconditions again */

  assert!(p_tile_no < p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th));
  assert!(
    p_comp_no
      < p_j2k
        .m_private_image
        .as_deref()
        .expect("m_private_image")
        .numcomps
  );
  if l_tccp.csty & J2K_CP_CSTY_PRT != 0 {
    (5u32).wrapping_add(l_tccp.numresolutions)
  } else {
    5_u32
  }
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
fn opj_j2k_compare_spcod_spcoc(
  p_j2k: &mut J2k,
  p_tile_no: u32,
  p_first_comp_no: u32,
  p_second_comp_no: u32,
) -> OPJ_BOOL {
  let mut i: u32 = 0;
  /* preconditions */
  let l_tcp = &p_j2k.m_cp.tcps[p_tile_no as usize];
  let l_tccp0 = &l_tcp.tccps[p_first_comp_no as usize];
  let l_tccp1 = &l_tcp.tccps[p_second_comp_no as usize];
  if l_tccp0.numresolutions != l_tccp1.numresolutions {
    return 0i32;
  }
  if l_tccp0.cblkw != l_tccp1.cblkw {
    return 0i32;
  }
  if l_tccp0.cblkh != l_tccp1.cblkh {
    return 0i32;
  }
  if l_tccp0.cblksty != l_tccp1.cblksty {
    return 0i32;
  }
  if l_tccp0.qmfbid != l_tccp1.qmfbid {
    return 0i32;
  }
  if l_tccp0.csty & J2K_CP_CSTY_PRT != l_tccp1.csty & J2K_CP_CSTY_PRT {
    return 0i32;
  }
  i = 0u32;
  while i < l_tccp0.numresolutions {
    if l_tccp0.prcw[i as usize] != l_tccp1.prcw[i as usize] {
      return 0i32;
    }
    if l_tccp0.prch[i as usize] != l_tccp1.prch[i as usize] {
      return 0i32;
    }
    i += 1;
  }
  1i32
}
/* *
 * Writes a SPCod or SPCoc element, i.e. the coding style of a given component of a tile.
 *
 * @param       p_j2k           J2K codec.
 * @param p_tile_no
 * @param       p_comp_no       the component number to output.
 * @param p_data
 * @param p_header_size
 * @param       p_manager       the user event manager.
 *
 * @return
*/
fn opj_j2k_write_spcod_spcoc(
  p_j2k: &mut J2k,
  p_tile_no: u32,
  p_comp_no: u32,
  data: &mut [u8],
  p_header_size: &mut u32,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut pos: usize = 0;
  let mut i: u32 = 0;
  /* preconditions */

  let l_tccp = &p_j2k.m_cp.tcps[p_tile_no as usize].tccps[p_comp_no as usize];
  /* preconditions again */
  /* SPcoc (E) */
  assert!(p_tile_no < p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th));
  assert!(
    p_comp_no
      < p_j2k
        .m_private_image
        .as_deref()
        .expect("m_private_image")
        .numcomps
  ); /* SPcoc (G) */
  if *p_header_size < 5u32 {
    event_msg!(p_manager, EVT_ERROR, "Error writing SPCod SPCoc element\n",); /* SPcoc (H) */
    return 0i32;
  } /* SPcoc (I_i) */
  opj_write_bytes(
    &mut data[pos..],
    l_tccp.numresolutions.wrapping_sub(1u32),
    1_u32,
  );
  pos += 1usize;
  opj_write_bytes(&mut data[pos..], l_tccp.cblkw.wrapping_sub(2u32), 1_u32);
  pos += 1usize;
  opj_write_bytes(&mut data[pos..], l_tccp.cblkh.wrapping_sub(2u32), 1_u32);
  pos += 1usize;
  opj_write_bytes(&mut data[pos..], l_tccp.cblksty, 1_u32);
  pos += 1usize;
  opj_write_bytes(&mut data[pos..], l_tccp.qmfbid, 1_u32);
  pos += 1usize;
  *p_header_size = (*p_header_size).wrapping_sub(5u32);
  if l_tccp.csty & J2K_CP_CSTY_PRT != 0 {
    if *p_header_size < l_tccp.numresolutions {
      event_msg!(p_manager, EVT_ERROR, "Error writing SPCod SPCoc element\n",);
      return 0i32;
    }
    i = 0_u32;
    while i < l_tccp.numresolutions {
      opj_write_bytes(
        &mut data[pos..],
        l_tccp.prcw[i as usize].wrapping_add(l_tccp.prch[i as usize] << 4i32),
        1_u32,
      );
      pos += 1usize;
      i += 1;
    }
    *p_header_size = (*p_header_size).wrapping_sub(l_tccp.numresolutions)
  }
  1i32
}
/* *
 * Reads a SPCod or SPCoc element, i.e. the coding style of a given component of a tile.
 * @param       p_j2k           the jpeg2000 codec.
 * @param compno
 * @param       p_header_data   the data contained in the COM box.
 * @param       p_header_size   the size of the data contained in the COM marker.
 * @param       p_manager       the user event manager.
*/
fn opj_j2k_read_spcod_spcoc(
  p_j2k: &mut J2k,
  compno: u32,
  header_data: &[u8],
  p_header_size: &mut u32,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut cursor: usize = 0;
  let mut i: u32 = 0;
  let mut l_tmp: u32 = 0;
  /* preconditions */

  /* If we are in the first tile-part header of the current tile */
  let is_tph = p_j2k.m_specific_param.m_decoder.m_state == J2KState::TPH;
  let m_reduce = p_j2k.m_cp.m_specific_param.m_dec.m_reduce;
  /* precondition again */
  assert!(
    compno
      < p_j2k
        .m_private_image
        .as_deref()
        .expect("m_private_image")
        .numcomps
  );
  let l_tcp = if is_tph {
    &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize]
  } else {
    p_j2k
      .m_specific_param
      .m_decoder
      .m_default_tcp
      .as_deref_mut()
      .expect("decoder m_default_tcp is always set")
  };
  let mut reduce_err = false;
  let l_tccp = &mut l_tcp.tccps[compno as usize];
  /* make sure room is sufficient */
  if *p_header_size < 5u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading SPCod SPCoc element\n",);
    return 0i32;
  }
  /* SPcod (D) / SPcoc (A) */
  l_tccp.numresolutions = opj_read_bytes(&header_data[cursor..], 1_u32); /* tccp->numresolutions = read() + 1 */
  l_tccp.numresolutions = l_tccp.numresolutions.wrapping_add(1);
  if l_tccp.numresolutions > 33u32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Invalid value for numresolutions : {}, max value is set in openjpeg.h at {}\n",
      l_tccp.numresolutions,
      33i32,
    );
    return 0i32;
  }
  cursor += 1usize;
  /* If user wants to remove more resolutions than the codestream contains, return error */
  if m_reduce >= l_tccp.numresolutions {
    event_msg!(p_manager, EVT_ERROR,
                    "Error decoding component {}.\nThe number of resolutions to remove ({}) is greater or equal than the number of resolutions of this component ({})\nModify the cp_reduce parameter.\n\n", compno,
                    m_reduce,
                    l_tccp.numresolutions);
    reduce_err = true;
  }
  /* SPcod (E) / SPcoc (B) */
  l_tccp.cblkw = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  l_tccp.cblkw = l_tccp.cblkw.wrapping_add(2u32);
  /* SPcod (F) / SPcoc (C) */
  l_tccp.cblkh = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  l_tccp.cblkh = l_tccp.cblkh.wrapping_add(2u32);
  if l_tccp.cblkw > 10u32 || l_tccp.cblkh > 10u32 || l_tccp.cblkw.wrapping_add(l_tccp.cblkh) > 12u32
  {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error reading SPCod SPCoc element, Invalid cblkw/cblkh combination\n",
    );
    return 0i32;
  }
  /* SPcod (G) / SPcoc (D) */
  l_tccp.cblksty = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  if l_tccp.cblksty & J2K_CCP_CBLKSTY_HTMIXED != 0u32 {
    /* We do not support HT mixed mode yet.  For conformance, it should be supported.*/
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error reading SPCod SPCoc element. Unsupported Mixed HT code-block style found\n",
    );
    return 0i32;
  }
  /* SPcod (H) / SPcoc (E) */
  l_tccp.qmfbid = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  if l_tccp.qmfbid > 1u32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error reading SPCod SPCoc element, Invalid transformation found\n",
    );
    return 0i32;
  }
  *p_header_size = (*p_header_size).wrapping_sub(5u32);
  /* use custom precinct size ? */
  if l_tccp.csty & J2K_CP_CSTY_PRT != 0 {
    if *p_header_size < l_tccp.numresolutions {
      event_msg!(p_manager, EVT_ERROR, "Error reading SPCod SPCoc element\n",);
      return 0i32;
    }
    /* SPcod (I_i) / SPcoc (F_i) */
    i = 0_u32;
    while i < l_tccp.numresolutions {
      l_tmp = opj_read_bytes(&header_data[cursor..], 1_u32);
      cursor += 1usize;
      /* Precinct exponent 0 is only allowed for lowest resolution level (Table A.21) */
      if i != 0u32 && (l_tmp & 0xfu32 == 0u32 || l_tmp >> 4i32 == 0u32) {
        event_msg!(p_manager, EVT_ERROR, "Invalid precinct size\n",);
        return 0i32;
      }
      l_tccp.prcw[i as usize] = l_tmp & 0xfu32;
      l_tccp.prch[i as usize] = l_tmp >> 4i32;
      i += 1;
    }
    *p_header_size = (*p_header_size).wrapping_sub(l_tccp.numresolutions)
  } else {
    /* set default size for the precinct width and height */
    i = 0_u32;
    while i < l_tccp.numresolutions {
      l_tccp.prcw[i as usize] = 15_u32;
      l_tccp.prch[i as usize] = 15_u32;
      i += 1;
    }
  }
  if reduce_err {
    p_j2k.m_specific_param.m_decoder.m_state |= J2KState::ERR;
    return 0i32;
  }
  1i32
}

/* *
 * Copies the tile component parameters of all the component from the first tile component.
 *
 * @param               p_j2k           the J2k codec.
 */
fn opj_j2k_copy_tile_component_parameters(p_j2k: &mut J2k) {
  /* loop */
  let mut i: u32 = 0;
  /* preconditions */
  let numcomps = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  let is_tph = p_j2k.m_specific_param.m_decoder.m_state == J2KState::TPH;
  let l_tcp = if is_tph {
    &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize]
  } else {
    p_j2k
      .m_specific_param
      .m_decoder
      .m_default_tcp
      .as_deref_mut()
      .expect("decoder m_default_tcp is always set")
  };
  i = 1_u32;
  while i < numcomps {
    let ref_ = &l_tcp.tccps[0];
    let (numresolutions, cblkw, cblkh, cblksty, qmfbid, prcw, prch) = (
      ref_.numresolutions,
      ref_.cblkw,
      ref_.cblkh,
      ref_.cblksty,
      ref_.qmfbid,
      ref_.prcw,
      ref_.prch,
    );
    let copied = &mut l_tcp.tccps[i as usize];
    copied.numresolutions = numresolutions;
    copied.cblkw = cblkw;
    copied.cblkh = cblkh;
    copied.cblksty = cblksty;
    copied.qmfbid = qmfbid;
    copied.prcw.copy_from_slice(&prcw[..]);
    copied.prch.copy_from_slice(&prch[..]);
    i += 1;
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
fn opj_j2k_get_sqcd_sqcc_size(p_j2k: &mut J2k, p_tile_no: u32, p_comp_no: u32) -> u32 {
  /* preconditions */
  let l_tccp = &p_j2k.m_cp.tcps[p_tile_no as usize].tccps[p_comp_no as usize];
  /* preconditions again */

  assert!(p_tile_no < p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th));
  assert!(
    p_comp_no
      < p_j2k
        .m_private_image
        .as_deref()
        .expect("m_private_image")
        .numcomps
  );
  let l_num_bands: u32 = if l_tccp.qntsty == 1u32 {
    1u32
  } else {
    l_tccp.numresolutions.wrapping_mul(3u32).wrapping_sub(2u32)
  };
  if l_tccp.qntsty == 0u32 {
    (1u32).wrapping_add(l_num_bands)
  } else {
    (1u32).wrapping_add((2u32).wrapping_mul(l_num_bands))
  }
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
fn opj_j2k_compare_sqcd_sqcc(
  p_j2k: &mut J2k,
  p_tile_no: u32,
  p_first_comp_no: u32,
  p_second_comp_no: u32,
) -> OPJ_BOOL {
  let mut l_band_no: u32 = 0;
  let mut l_num_bands: u32 = 0;
  /* preconditions */
  let l_tcp = &p_j2k.m_cp.tcps[p_tile_no as usize];
  let l_tccp0 = &l_tcp.tccps[p_first_comp_no as usize];
  let l_tccp1 = &l_tcp.tccps[p_second_comp_no as usize];
  if l_tccp0.qntsty != l_tccp1.qntsty {
    return 0i32;
  }
  if l_tccp0.numgbits != l_tccp1.numgbits {
    return 0i32;
  }
  if l_tccp0.qntsty == 1u32 {
    l_num_bands = 1u32
  } else {
    l_num_bands = l_tccp0.numresolutions.wrapping_mul(3u32).wrapping_sub(2u32);
    if l_num_bands != l_tccp1.numresolutions.wrapping_mul(3u32).wrapping_sub(2u32) {
      return 0i32;
    }
  }
  l_band_no = 0_u32;
  while l_band_no < l_num_bands {
    if l_tccp0.stepsizes[l_band_no as usize].expn != l_tccp1.stepsizes[l_band_no as usize].expn {
      return 0i32;
    }
    l_band_no += 1;
  }
  if l_tccp0.qntsty != 0u32 {
    l_band_no = 0_u32;
    while l_band_no < l_num_bands {
      if l_tccp0.stepsizes[l_band_no as usize].mant != l_tccp1.stepsizes[l_band_no as usize].mant {
        return 0i32;
      }
      l_band_no += 1;
    }
  }
  1i32
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
fn opj_j2k_write_sqcd_sqcc(
  p_j2k: &mut J2k,
  p_tile_no: u32,
  p_comp_no: u32,
  data: &mut [u8],
  p_header_size: &mut u32,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut pos: usize = 0;
  let mut l_header_size: u32 = 0;
  let mut l_band_no: u32 = 0;
  let mut l_num_bands: u32 = 0;
  let mut l_expn: u32 = 0;
  let mut l_mant: u32 = 0;
  /* preconditions */

  let l_tccp = &p_j2k.m_cp.tcps[p_tile_no as usize].tccps[p_comp_no as usize];
  /* preconditions again */
  /* SPqcx_i */
  assert!(p_tile_no < p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th));
  assert!(
    p_comp_no
      < p_j2k
        .m_private_image
        .as_deref()
        .expect("m_private_image")
        .numcomps
  ); /* SPqcx_i */
  l_num_bands = if l_tccp.qntsty == 1u32 {
    1u32
  } else {
    l_tccp.numresolutions.wrapping_mul(3u32).wrapping_sub(2u32)
  };
  if l_tccp.qntsty == 0u32 {
    l_header_size = (1u32).wrapping_add(l_num_bands);
    if *p_header_size < l_header_size {
      event_msg!(p_manager, EVT_ERROR, "Error writing SQcd SQcc element\n",);
      return 0i32;
    }
    opj_write_bytes(
      &mut data[pos..],
      l_tccp.qntsty.wrapping_add(l_tccp.numgbits << 5i32),
      1_u32,
    );
    pos += 1usize;
    l_band_no = 0_u32;
    while l_band_no < l_num_bands {
      l_expn = l_tccp.stepsizes[l_band_no as usize].expn as u32;
      opj_write_bytes(&mut data[pos..], l_expn << 3i32, 1_u32);
      pos += 1usize;
      l_band_no += 1;
    }
  } else {
    l_header_size = (1u32).wrapping_add((2u32).wrapping_mul(l_num_bands));
    if *p_header_size < l_header_size {
      event_msg!(p_manager, EVT_ERROR, "Error writing SQcd SQcc element\n",);
      return 0i32;
    }
    opj_write_bytes(
      &mut data[pos..],
      l_tccp.qntsty.wrapping_add(l_tccp.numgbits << 5i32),
      1_u32,
    );
    pos += 1usize;
    l_band_no = 0_u32;
    while l_band_no < l_num_bands {
      l_expn = l_tccp.stepsizes[l_band_no as usize].expn as u32;
      l_mant = l_tccp.stepsizes[l_band_no as usize].mant as u32;
      opj_write_bytes(
        &mut data[pos..],
        (l_expn << 11i32).wrapping_add(l_mant),
        2_u32,
      );
      pos += 2usize;
      l_band_no += 1;
    }
  }
  *p_header_size = (*p_header_size).wrapping_sub(l_header_size);
  1i32
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
fn opj_j2k_read_sqcd_sqcc(
  p_j2k: &mut J2k,
  p_comp_no: u32,
  header_data: &[u8],
  p_header_size: &mut u32,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut cursor: usize = 0;
  /* loop*/
  let mut l_band_no: u32 = 0;
  let mut l_tmp: u32 = 0;
  let mut l_num_band: u32 = 0;
  /* preconditions*/

  /* come from tile part header or main header ?*/
  let is_tph = p_j2k.m_specific_param.m_decoder.m_state == J2KState::TPH;
  /* precondition again*/
  assert!(
    p_comp_no
      < p_j2k
        .m_private_image
        .as_deref()
        .expect("m_private_image")
        .numcomps
  );
  let l_tcp = if is_tph {
    &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize]
  } else {
    p_j2k
      .m_specific_param
      .m_decoder
      .m_default_tcp
      .as_deref_mut()
      .expect("decoder m_default_tcp is always set")
  };
  let l_tccp = &mut l_tcp.tccps[p_comp_no as usize];
  if *p_header_size < 1u32 {
    event_msg!(p_manager, EVT_ERROR, "Error reading SQcd or SQcc element\n",);
    return 0i32;
  }
  *p_header_size = (*p_header_size).wrapping_sub(1u32);
  l_tmp = opj_read_bytes(&header_data[cursor..], 1_u32);
  cursor += 1usize;
  l_tccp.qntsty = l_tmp & 0x1fu32;
  l_tccp.numgbits = l_tmp >> 5i32;
  if l_tccp.qntsty == 1u32 {
    l_num_band = 1_u32
  } else {
    l_num_band = if l_tccp.qntsty == 0u32 {
      *p_header_size
    } else {
      (*p_header_size).wrapping_div(2u32)
    };
    if l_num_band > (3i32 * 33i32 - 2i32) as core::ffi::c_uint {
      event_msg!(p_manager, EVT_WARNING,
                        "While reading CCP_QNTSTY element inside QCD or QCC marker segment, number of subbands ({}) is greater to OPJ_J2K_MAXBANDS ({}). So we limit the number of elements stored to OPJ_J2K_MAXBANDS ({}) and skip the rest. \n", l_num_band,
                        3i32 * 33i32 -
                            2i32,
                        3i32 * 33i32 -
                            2i32);
      /*return OPJ_FALSE;*/
    }
  }
  /* USE_JPWL */

  if l_tccp.qntsty == 0u32 {
    l_band_no = 0_u32; /* SPqcx_i */
    while l_band_no < l_num_band {
      l_tmp = opj_read_bytes(&header_data[cursor..], 1_u32); /* SPqcx_i */
      cursor += 1usize;
      if l_band_no < (3i32 * 33i32 - 2i32) as core::ffi::c_uint {
        l_tccp.stepsizes[l_band_no as usize].expn = (l_tmp >> 3i32) as i32;
        l_tccp.stepsizes[l_band_no as usize].mant = 0i32
      }
      l_band_no += 1;
    }
    if *p_header_size < l_num_band {
      return 0i32;
    }
    *p_header_size = (*p_header_size).wrapping_sub(l_num_band)
  } else {
    /* Hoisted before the band loop: with qntsty == 1 (SIQNT) the declared
    l_num_band is forced to 1 even when no SPqcd bytes remain, and reading
    2 bytes past the end of the marker content is a slice OOB panic (the C
    reads stale scratch bytes in-bounds and then fails the same check). */
    if *p_header_size < 2 * l_num_band {
      return 0i32;
    }
    l_band_no = 0_u32;
    while l_band_no < l_num_band {
      l_tmp = opj_read_bytes(&header_data[cursor..], 2_u32);
      cursor += 2usize;
      if l_band_no < (3i32 * 33i32 - 2i32) as core::ffi::c_uint {
        l_tccp.stepsizes[l_band_no as usize].expn = (l_tmp >> 11i32) as i32;
        l_tccp.stepsizes[l_band_no as usize].mant = (l_tmp & 0x7ffu32) as i32
      }
      l_band_no += 1;
    }
    *p_header_size = (*p_header_size).wrapping_sub((2u32).wrapping_mul(l_num_band))
  }
  /* Add Antonin : if scalar_derived -> compute other stepsizes */
  if l_tccp.qntsty == 1u32 {
    l_band_no = 1_u32;
    while l_band_no < (3i32 * 33i32 - 2i32) as core::ffi::c_uint {
      l_tccp.stepsizes[l_band_no as usize].expn = if l_tccp.stepsizes[0_usize].expn
        - l_band_no.wrapping_sub(1u32).wrapping_div(3u32) as i32
        > 0i32
      {
        (l_tccp.stepsizes[0_usize].expn) - l_band_no.wrapping_sub(1u32).wrapping_div(3u32) as i32
      } else {
        0i32
      };
      l_tccp.stepsizes[l_band_no as usize].mant = l_tccp.stepsizes[0_usize].mant;
      l_band_no += 1;
    }
  }
  1i32
}

/* *
 * Copies the tile quantization parameters of all the component from the first tile component.
 *
 * @param               p_j2k           the J2k codec.
 */
fn opj_j2k_copy_tile_quantization_parameters(p_j2k: &mut J2k) {
  let mut i: u32 = 0;
  /* preconditions */
  let numcomps = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .numcomps;
  let is_tph = p_j2k.m_specific_param.m_decoder.m_state == J2KState::TPH;
  let l_tcp = if is_tph {
    &mut p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize]
  } else {
    p_j2k
      .m_specific_param
      .m_decoder
      .m_default_tcp
      .as_deref_mut()
      .expect("decoder m_default_tcp is always set")
  };
  i = 1_u32;
  while i < numcomps {
    let ref_ = &l_tcp.tccps[0];
    let (qntsty, numgbits, stepsizes) = (ref_.qntsty, ref_.numgbits, ref_.stepsizes);
    let copied = &mut l_tcp.tccps[i as usize];
    copied.qntsty = qntsty;
    copied.numgbits = numgbits;
    copied.stepsizes.copy_from_slice(&stepsizes[..]);
    i += 1;
  }
}

fn opj_j2k_allocate_tile_element_cstr_index(
  cstr_index: &mut CodestreamIndex,
  nb_of_tiles: u32,
) -> OPJ_BOOL {
  cstr_index
    .tiles
    .resize(nb_of_tiles as usize, CodestreamTileIndex::default());
  for tile in &mut cstr_index.tiles {
    tile.markers = Vec::with_capacity(100);
  }
  1i32
}

fn opj_j2k_are_all_used_components_decoded(p_j2k: &mut J2k, p_manager: &mut EventMgr) -> OPJ_BOOL {
  let mut compno: u32 = 0;
  let mut decoded_all_used_components = 1i32;
  if let Some(image) = p_j2k.m_output_image.as_ref() {
    if p_j2k.m_specific_param.m_decoder.m_numcomps_to_decode != 0 {
      compno = 0_u32;
      while compno < p_j2k.m_specific_param.m_decoder.m_numcomps_to_decode {
        let dec_compno =
          (&p_j2k.m_specific_param.m_decoder.m_comps_indices_to_decode)[compno as usize];
        if image.comps[dec_compno as usize].data.is_empty() {
          event_msg!(
            p_manager,
            EVT_WARNING,
            "Failed to decode component {}\n",
            dec_compno,
          );
          decoded_all_used_components = 0i32
        }
        compno += 1;
      }
    } else {
      compno = 0_u32;
      while compno < image.numcomps {
        if image.comps[compno as usize].data.is_empty() {
          event_msg!(
            p_manager,
            EVT_WARNING,
            "Failed to decode component {}\n",
            compno,
          );
          decoded_all_used_components = 0i32
        }
        compno += 1;
      }
    }
  }
  if decoded_all_used_components == 0i32 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Failed to decode all used components\n",
    );
    return 0i32;
  }
  1i32
}

/* *
 * Reads the tiles.
 */
fn opj_j2k_decode_tiles(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut tile_info = TileInfo::default();
  let mut nr_tiles = 0_u32;
  /* Particular case for whole single tile decoding */
  /* We can avoid allocating intermediate tile buffers */
  let single_tile_fast = p_j2k.m_cp.tw == 1u32
    && p_j2k.m_cp.th == 1u32
    && p_j2k.m_cp.tx0 == 0u32
    && p_j2k.m_cp.ty0 == 0u32
    && p_j2k.m_output_image.as_ref().is_some_and(|image| {
      image.x0 == 0u32
        && image.y0 == 0u32
        && image.x1 == p_j2k.m_cp.tdx
        && image.y1 == p_j2k.m_cp.tdy
    });
  if single_tile_fast {
    if !opj_j2k_read_tile_header(p_j2k, p_stream, &mut tile_info, p_manager) {
      return 0i32;
    }
    if opj_j2k_decode_tile(p_j2k, tile_info.index, None, p_stream, p_manager) == 0 {
      event_msg!(p_manager, EVT_ERROR, "Failed to decode tile 1/1\n",);
      return 0i32;
    }
    /* Transfer TCD data to output image data */
    let image = p_j2k.m_output_image.as_mut().expect("m_output_image");
    let mut i: u32 = 0;
    while i < image.numcomps {
      let comp = &mut image.comps[i as usize];
      let len = (comp.w as usize).wrapping_mul(comp.h as usize);
      comp.data = p_j2k.m_tcd.tcd_image.tiles.comps[i as usize]
        .data
        .take_vec_i32(len);
      comp.resno_decoded = p_j2k
        .m_private_image
        .as_deref()
        .expect("m_private_image")
        .comps[i as usize]
        .resno_decoded;
      i += 1;
    }
    return 1i32;
  }
  p_j2k.m_specific_param.m_decoder.m_num_intersecting_tile_parts = 0;
  p_j2k.m_specific_param.m_decoder.m_idx_intersecting_tile_parts = 0;
  p_j2k.m_specific_param.m_decoder.m_intersecting_tile_parts_offset.clear();

  /* If the area to decode only intersects a subset of tiles, and we have
  valid TLM information, then use it to plan the tile-part offsets to
  seek to (upstream #1538) */
  let dec = &mut p_j2k.m_specific_param.m_decoder;
  let full_extent = dec.m_start_tile_x == 0
    && dec.m_start_tile_y == 0
    && dec.m_end_tile_x == p_j2k.m_cp.tw
    && dec.m_end_tile_y == p_j2k.m_cp.th;
  let mut end_pos: i64 = 0;
  let cstr_index = p_j2k.cstr_index.as_deref().expect("cstr_index");
  if !full_extent && !dec.m_tlm.m_is_invalid && opj_stream_has_seek(p_stream) != 0 {
    let mut m_num_intersecting_tile_parts: u32 = 0;
    for tile in cstr_index.tiles.iter() {
      if tile.nb_tps > 0
        && tile.tp_index.len() >= tile.nb_tps as usize
        && tile.tp_index[tile.nb_tps as usize - 1].end_pos > end_pos
      {
        end_pos = tile.tp_index[tile.nb_tps as usize - 1].end_pos;
      }
    }
    let mut offsets: Vec<i64> = Vec::new();
    for j in dec.m_start_tile_y..dec.m_end_tile_y {
      for i in dec.m_start_tile_x..dec.m_end_tile_x {
        let tile_number = (j * p_j2k.m_cp.tw + i) as usize;
        for k in 0..cstr_index.tiles[tile_number].nb_tps as usize {
          offsets.push(cstr_index.tiles[tile_number].tp_index[k].start_pos);
          m_num_intersecting_tile_parts += 1;
        }
      }
    }
    offsets.sort_unstable();
    dec.m_intersecting_tile_parts_offset = offsets;
    dec.m_num_intersecting_tile_parts = m_num_intersecting_tile_parts;
  }
  loop {
    if p_j2k.m_cp.tw == 1u32 && p_j2k.m_cp.th == 1u32 && !p_j2k.m_cp.tcps[0].m_data.is_empty() {
      tile_info.index = 0_u32;
      p_j2k.m_current_tile_number = 0_u32;
      p_j2k.m_specific_param.m_decoder.m_state |= J2KState::DATA
    } else {
      if !opj_j2k_read_tile_header(p_j2k, p_stream, &mut tile_info, p_manager) {
        /* Lenient mode (progressive download): stop decoding tiles and keep
        the ones already merged (the C's decode_tile returns OPJ_TRUE for a
        short stream in lenient mode and the tile loop ends the same way). */
        if p_j2k.m_cp.strict != 0 {
          return 0i32;
        }
        break;
      }
      if !tile_info.go_on {
        break;
      }
    }
    if opj_j2k_decode_tile(p_j2k, tile_info.index, None, p_stream, p_manager) == 0 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Failed to decode tile {}/{}\n",
        tile_info.index.wrapping_add(1u32),
        p_j2k.m_cp.th.wrapping_mul(p_j2k.m_cp.tw),
      );
      return 0i32;
    }
    event_msg!(
      p_manager,
      EVT_INFO,
      "Tile {}/{} has been decoded.\n",
      tile_info.index.wrapping_add(1u32),
      p_j2k.m_cp.th.wrapping_mul(p_j2k.m_cp.tw),
    );
    if opj_j2k_update_image_data(
      &mut p_j2k.m_tcd,
      p_j2k.m_output_image.as_mut().expect("m_output_image"),
      p_j2k
        .m_private_image
        .as_deref_mut()
        .expect("m_private_image"),
      p_j2k.decode_limits.effective_max_mem(),
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    let same_extent = p_j2k.m_output_image.as_ref().is_some_and(|image| {
      let private = p_j2k.m_private_image.as_deref().expect("m_private_image");
      image.x0 == private.x0
        && image.y0 == private.y0
        && image.x1 == private.x1
        && image.y1 == private.y1
    });
    if !(p_j2k.m_cp.tw == 1u32 && p_j2k.m_cp.th == 1u32 && !same_extent) {
      opj_j2k_tcp_data_destroy(&mut p_j2k.m_cp.tcps[tile_info.index as usize]);
    }
    event_msg!(
      p_manager,
      EVT_INFO,
      "Image data has been updated with tile {}.\n\n",
      tile_info.index.wrapping_add(1u32),
    );
    if opj_stream_get_number_byte_left(p_stream) == 0i64
      && p_j2k.m_specific_param.m_decoder.m_state == J2KState::NEOC
    {
      break;
    }
    nr_tiles = nr_tiles.wrapping_add(1);
    if nr_tiles == p_j2k.m_cp.th.wrapping_mul(p_j2k.m_cp.tw) {
      break;
    }
    if p_j2k.m_specific_param.m_decoder.m_num_intersecting_tile_parts > 0
      && p_j2k
        .m_specific_param
        .m_decoder
        .m_idx_intersecting_tile_parts
        == p_j2k.m_specific_param.m_decoder.m_num_intersecting_tile_parts
    {
      let _ = opj_stream_seek(p_stream, end_pos + 2i64, p_manager);
      break;
    }
  }
  if opj_j2k_are_all_used_components_decoded(p_j2k, p_manager) == 0 {
    return 0i32;
  }
  1i32
}

/*
 * Read and decode one tile.
 */
fn opj_j2k_decode_one_tile(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut tile_info = TileInfo::default();
  let mut l_nb_tiles: u32 = 0;
  let mut i: u32 = 0;
  /*Allocate and initialize some elements of codestrem index if not already done*/
  if p_j2k
    .cstr_index
    .as_deref()
    .expect("cstr_index")
    .tiles
    .is_empty()
  {
    let nb_of_tiles = p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th);
    if opj_j2k_allocate_tile_element_cstr_index(
      p_j2k.cstr_index.as_deref_mut().expect("cstr_index"),
      nb_of_tiles,
    ) == 0
    {
      return 0i32;
    }
  }
  /* Move into the codestream to the first SOT used to decode the desired tile */
  let l_tile_no_to_dec = p_j2k.m_specific_param.m_decoder.m_tile_ind_to_dec as u32;
  if !p_j2k
    .cstr_index
    .as_deref()
    .expect("cstr_index")
    .tiles
    .is_empty()
    && !p_j2k.cstr_index.as_deref().expect("cstr_index").tiles[0]
      .tp_index
      .is_empty()
  {
    if p_j2k.cstr_index.as_deref().expect("cstr_index").tiles[l_tile_no_to_dec as usize].nb_tps == 0
    {
      /* The index for this tile has not been built: the tile's tile-parts
      are interleaved with other tiles' tile-parts and TNsot was only
      declared in a tile-part not reached yet.  m_last_sot_read_pos may
      point past the first tile-part of the requested tile, so seek back
      to the start of the codestream and read the tile-part headers in
      order (upstream #1558). */
      if opj_stream_seek(
        p_stream,
        p_j2k
          .cstr_index
          .as_deref()
          .expect("cstr_index")
          .main_head_end
          + 2i64,
        p_manager,
      ) == 0
      {
        event_msg!(p_manager, EVT_ERROR, "Problem with seek function\n",);
        return 0i32;
      }
    } else {
      let sot_pos = p_j2k.cstr_index.as_deref().expect("cstr_index").tiles
        [l_tile_no_to_dec as usize]
        .tp_index[0]
        .start_pos;
      if opj_stream_seek(p_stream, sot_pos, p_manager) == 0 {
        event_msg!(p_manager, EVT_ERROR, "Problem with seek function\n",);
        return 0i32;
      }
      /* Validate that the expected SOT marker is there (upstream #1558) */
      if opj_stream_read_data(
        p_stream,
        &mut p_j2k.m_specific_param.m_decoder.m_header_data[..2],
        p_manager,
      ) != 2
      {
        event_msg!(p_manager, EVT_ERROR, "Stream too short\n",);
        return 0i32;
      }
      if J2KMarker::from_buffer(&p_j2k.m_specific_param.m_decoder.m_header_data) != J2KMarker::SOT {
        event_msg!(p_manager, EVT_ERROR, "Did not get expected SOT marker\n",);
        return 0i32;
      }
    }
    /* Special case if we have previously read the EOC marker (if the previous tile getted is the last ) */
    if p_j2k.m_specific_param.m_decoder.m_state == J2KState::EOC {
      p_j2k.m_specific_param.m_decoder.m_state = J2KState::TPHSOT
    }
  }
  /* Reset current tile part number for all tiles, and not only the one */
  /* of interest. */
  /* Not completely sure this is always correct but required for */
  /* ./build/bin/j2k_random_tile_access ./build/tests/tte1.j2k */
  l_nb_tiles = p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th);
  i = 0_u32;
  while i < l_nb_tiles {
    p_j2k.m_cp.tcps[i as usize].m_current_tile_part_number = -(1i32);
    i += 1;
  }
  loop {
    if !opj_j2k_read_tile_header(p_j2k, p_stream, &mut tile_info, p_manager) {
      return 0i32;
    }
    if !tile_info.go_on {
      break;
    }
    if opj_j2k_decode_tile(p_j2k, tile_info.index, None, p_stream, p_manager) == 0 {
      return 0i32;
    }
    event_msg!(
      p_manager,
      EVT_INFO,
      "Tile {}/{} has been decoded.\n",
      tile_info.index.wrapping_add(1u32),
      p_j2k.m_cp.th.wrapping_mul(p_j2k.m_cp.tw),
    );
    if opj_j2k_update_image_data(
      &mut p_j2k.m_tcd,
      p_j2k.m_output_image.as_mut().expect("m_output_image"),
      p_j2k
        .m_private_image
        .as_deref_mut()
        .expect("m_private_image"),
      p_j2k.decode_limits.effective_max_mem(),
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    opj_j2k_tcp_data_destroy(&mut p_j2k.m_cp.tcps[tile_info.index as usize]);
    event_msg!(
      p_manager,
      EVT_INFO,
      "Image data has been updated with tile {}.\n\n",
      tile_info.index.wrapping_add(1u32),
    );
    if tile_info.index == l_tile_no_to_dec {
      /* Move to the first SOT */
      if opj_stream_seek(
        p_stream,
        p_j2k
          .cstr_index
          .as_deref()
          .expect("cstr_index")
          .main_head_end
          + 2i64,
        p_manager,
      ) == 0
      {
        event_msg!(p_manager, EVT_ERROR, "Problem with seek function\n",);
        return 0i32;
      }
      break;
    } else {
      event_msg!(
        p_manager,
        EVT_WARNING,
        "Tile read, decoded and updated is not the desired one ({} vs {}).\n",
        tile_info.index.wrapping_add(1u32),
        l_tile_no_to_dec.wrapping_add(1u32),
      );
    }
  }
  if opj_j2k_are_all_used_components_decoded(p_j2k, p_manager) == 0 {
    return 0i32;
  }
  1i32
}

/* *
 * Sets up the procedures to do on decoding one tile. Developers wanting to extend the library can add their own reading procedures.
 */
/// Decodes a single tile with a fresh codec state over a stream positioned
/// at the tile's first SOT (used by the parallel tile-decode driver in the
/// safe API).  Returns the decoded tile cache.
pub(crate) fn j2k_decode_tile_worker(
  cp: &Cp,
  header: &CoreImage,
  tile_no: u32,
  layer: u32,
  reduce: u32,
  strict: bool,
  decode_bounds: Option<&CoreImage>,
  stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> Option<(Tcd, Vec<u32>)> {
  let mut j2k = opj_j2k_create_decompress()?;
  j2k.m_cp = cp.clone();
  j2k.m_cp.m_specific_param.m_dec.m_layer = layer;
  j2k.m_cp.m_specific_param.m_dec.m_reduce = reduce;
  j2k.m_private_image = Some(Box::new(header.clone()));
  /* The serial decode_tile takes the window bounds from m_output_image;
  for a decode-area request it is the (small) area image. */
  j2k.m_output_image = decode_bounds.map(|i| Box::new(i.clone()));
  {
    let dec = &mut j2k.m_specific_param.m_decoder;
    dec.m_tile_ind_to_dec = tile_no as i32;
    dec.m_state = J2KState::TPHSOT;
    dec.m_can_decode = false;
    /* opj_j2k_decoder_set_strict_mode also disables the nb-tile-parts
    correction hack; mirror it so the strict decode matches the serial. */
    dec.m_nb_tile_parts_correction_checked = strict;
  }
  j2k.m_current_tile_number = tile_no;
  /* The serial path initialises the tcd from the private image at the end
  of opj_j2k_read_header; the worker codec skipped read_header, so do it
  here (sizes tcd.tcd_image.tiles.comps from the header image). */
  if opj_tcd_init(
    &mut j2k.m_tcd,
    j2k.m_private_image.as_deref().expect("m_private_image"),
    &mut j2k.m_cp,
  ) == 0
  {
    return None;
  }
  /* Decode every tile-part of the requested tile.  The read_tile_header
  walk skips the (possibly interleaved) tile-parts of the other tiles,
  so the stream only needs to start at the tile's first SOT. */
  let nb_of_tiles = j2k.m_cp.tw.wrapping_mul(j2k.m_cp.th);
  if opj_j2k_allocate_tile_element_cstr_index(
    j2k.cstr_index.as_deref_mut().expect("cstr_index"),
    nb_of_tiles,
  ) == 0
  {
    return None;
  }
  let mut tile_info = TileInfo::default();
  let mut decoded_any = false;
  loop {
    if !opj_j2k_read_tile_header(&mut j2k, stream, &mut tile_info, p_manager) {
      return None;
    }
    if !tile_info.go_on {
      break;
    }
    if tile_info.index != tile_no {
      if decoded_any {
        break;
      }
      /* Unexpected tile (should not happen from the tile's own SOT):
      keep skipping until the requested tile shows up or EOC is hit. */
      event_msg!(
        p_manager,
        EVT_WARNING,
        "j2k_decode_tile_worker: skipping tile {} while decoding tile {}\n",
        tile_info.index.wrapping_add(1u32),
        tile_no.wrapping_add(1u32),
      );
      continue;
    }
    if opj_j2k_decode_tile(&mut j2k, tile_info.index, None, stream, p_manager) == 0 {
      return None;
    }
    opj_j2k_tcp_data_destroy(&mut j2k.m_cp.tcps[tile_info.index as usize]);
    decoded_any = true;
    /* Stop as soon as the tile's tile-parts are exhausted instead of
    walking to the EOC marker (the nb-tile-parts correction has already
    run by now, so the count is reliable). */
    let tcp = &j2k.m_cp.tcps[tile_no as usize];
    if tcp.m_current_tile_part_number as u32 + 1 >= tcp.m_nb_tile_parts {
      break;
    }
  }
  if !decoded_any {
    return None;
  }
  /* The t2 decode updates the private image's resno_decoded to the
  highest resolution actually decoded (a quality-layer limit can
  truncate it below minimum_num_resolutions - 1). */
  let resno = j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .comps
    .iter()
    .map(|c| c.resno_decoded)
    .collect();
  Some((core::mem::take(&mut j2k.m_tcd), resno))
}

/// Merges per-tile decode results into the output image (the existing
/// sequential update path, run once per tile in order).
pub(crate) fn j2k_merge_tile_results(
  tiles: Vec<(u32, Tcd, Vec<u32>)>,
  header: &CoreImage,
  p_output_image: &mut CoreImage,
  p_manager: &mut EventMgr,
  max_mem: u64,
) -> OPJ_BOOL {
  for (_tile_no, mut tcd, resno) in tiles {
    let mut private = header.clone();
    /* The header image carries resno_decoded == 0; the update path indexes
    the tile-comp resolutions with it.  The serial flow learns the decoded
    resolution count from the t2 decode; the workers carry it out. */
    for (i, comp) in private.comps.iter_mut().enumerate() {
      comp.resno_decoded = *resno.get(i).unwrap_or(&0);
    }
    if opj_j2k_update_image_data(&mut tcd, p_output_image, &mut private, max_mem, p_manager) == 0 {
      event_msg!(p_manager, EVT_ERROR, "Failed to update image with tile data\n",);
      return 0i32;
    }
  }
  1i32
}

/// Parallel whole-image decode (feature `rayon`): snapshot the codestream
/// into memory, locate every tile's first SOT (from the TLM-built index
/// when present, else by scanning for SOT markers - bit-stuffing guarantees
/// FF 90 never occurs inside coded tile data), decode each tile with a
/// fresh codec state over an independent cursor into the shared buffer,
/// and merge the tile caches in order with the existing update path.
///
/// Returns None when the layout is not supported (the caller then falls
/// back to the sequential decode), so the result is always identical to
/// the serial path.
/// Shared read-only buffer adapter so `std::io::Cursor` (which needs
/// `AsRef<[u8]>`) can borrow the snapshot without copying it.
#[cfg(feature = "rayon")]
struct SharedBuf(std::sync::Arc<[u8]>);

#[cfg(feature = "rayon")]
impl AsRef<[u8]> for SharedBuf {
  fn as_ref(&self) -> &[u8] {
    &self.0[..]
  }
}

#[cfg(feature = "rayon")]
pub(crate) fn j2k_decode_tiles_parallel(
  cp: &Cp,
  header: &CoreImage,
  num_tiles: u32,
  tiles: &[u32],
  cstr_index: &CodestreamIndex,
  layer: u32,
  reduce: u32,
  strict: bool,
  decode_bounds: Option<&CoreImage>,
  merge_image: &CoreImage,
  stream: &mut Stream,
  max_mem: u64,
) -> Option<CoreImage> {
  use rayon::prelude::*;
  use std::io::Read as _;

  /* Snapshot the whole codestream: workers seek within it independently.
  The stream position is restored so a serial fallback stays possible.
  For in-memory streams the buffer is shared zero-copy (B.1.6); other
  sources are read once. */
  let anchor = stream.tell();
  let bytes: std::sync::Arc<[u8]> = if let Some(shared) = stream.shared_bytes() {
    shared
  } else {
    if stream.seek(0).is_err() {
      return None;
    }
    let mut buf = Vec::new();
    if stream.read_to_end(&mut buf).is_err() {
      return None;
    }
    std::sync::Arc::<[u8]>::from(buf)
  };
  let _ = stream.seek(anchor);
  let len = bytes.len() as u64;
  /* The snapshot is decode memory: count it against the caller's limit and
  decline the parallel path when it would exceed it (graceful serial
  fallback - open-findings OF-18). */
  if len > max_mem {
    return None;
  }

  /* First SOT offset per tile (absolute positions in the buffer). */
  let mut first_sot: Vec<i64> = vec![-1; num_tiles as usize];
  let tlm_ok = !cstr_index.tiles.is_empty()
    && cstr_index.tiles[0].nb_tps > 0
    && !cstr_index.tiles[0].tp_index.is_empty();
  if tlm_ok {
    for &t in tiles {
      let tile = &cstr_index.tiles[t as usize];
      if tile.tp_index.is_empty() {
        return None;
      }
      first_sot[t as usize] = tile.tp_index[0].start_pos;
    }
  } else {
    /* Structured SOT walk: step the tile-parts by their declared lengths
    (segment-aware, so packet data is never scanned - the raw FF 90 00 0A
    byte scan could false-positive on packet bytes, review_3 L13; B.1.6). */
    let mut pos = cstr_index.main_head_end.max(0) as usize;
    while pos + 4 <= bytes.len() {
      let marker = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]);
      if marker == 0xFFD9 {
        /* EOC */
        break;
      }
      if marker == 0xFF90 {
        /* SOT: record the tile-part start (the Isot sits at +4..+6), then
        skip the whole part via Psot. */
        if pos + 12 > bytes.len()
          || u16::from_be_bytes([bytes[pos + 2], bytes[pos + 3]]) != 10
        {
          return None;
        }
        let isot = u16::from_be_bytes([bytes[pos + 4], bytes[pos + 5]]) as u32;
        if (isot as usize) < first_sot.len() && first_sot[isot as usize] < 0 {
          first_sot[isot as usize] = pos as i64;
        }
        let psot = u32::from_be_bytes([
          bytes[pos + 6],
          bytes[pos + 7],
          bytes[pos + 8],
          bytes[pos + 9],
        ]) as usize;
        if psot < 10 || pos + psot > bytes.len() {
          /* Unknown or overrunning part length: the walk cannot be trusted
          - decline (the caller falls back to the serial path). */
          return None;
        }
        pos += psot;
      } else if marker == 0xFF4F || marker == 0xFF93 {
        /* SOC/SOD carry no length field (2 bytes). */
        pos += 2;
      } else {
        /* A marker with a length field (Lmar includes itself). */
        let lmar = u16::from_be_bytes([bytes[pos + 2], bytes[pos + 3]]) as usize;
        if lmar < 2 || pos + lmar > bytes.len() {
          return None;
        }
        pos += lmar;
      }
    }
  }
  if tiles.iter().any(|t| first_sot[*t as usize] < 0) {
    return None;
  }

  /* Decode the requested tiles in parallel. */
  let decoded: Vec<Option<(Tcd, Vec<u32>)>> = tiles
    .into_par_iter()
    .map(|&tile_no| {
      let mut worker_stream = Stream::new_input(
        OPJ_J2K_STREAM_CHUNK_SIZE as usize,
        std::io::Cursor::new(SharedBuf(bytes.clone())),
        len,
      );
      if worker_stream.seek(first_sot[tile_no as usize] + 2).is_err() {
        return None;
      }
      let mut manager = EventMgr::default();
      /* The sequential decode (and the C reference) reuses code-block
      state ACROSS tiles: for "tolerant" codestreams whose packet
      segments overrun their packets (g1_colr.j2c), the serial walk
      only works because the previous tile's segment state is still
      there, and the t2 lenient recovery ("segment too long") would
      silently produce DIFFERENT data in a fresh per-tile decode.
      Treat that recovery as a worker failure so the caller falls
      back to the (correct) sequential decode. */
      let recovered = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
      let recovered2 = recovered.clone();
      manager.set_warning_handler(move |msg: &str| {
        if msg.contains("segment too long") {
          recovered2.store(true, std::sync::atomic::Ordering::Relaxed);
        }
      });
      let result = j2k_decode_tile_worker(
        cp,
        header,
        tile_no,
        layer,
        reduce,
        strict,
        decode_bounds,
        &mut worker_stream,
        &mut manager,
      );
      if recovered.load(std::sync::atomic::Ordering::Relaxed) {
        return None;
      }
      result
    })
    .collect();
  if decoded.iter().any(Option::is_none) {
    return None;
  }
  let decoded: Vec<(u32, Tcd, Vec<u32>)> = tiles
    .iter()
    .copied()
    .zip(
      decoded
        .into_iter()
        .map(|t| {
          let (tcd, resno) = t.expect("checked above");
          (tcd, resno)
        }),
    )
    .map(|(tile, (tcd, resno))| (tile, tcd, resno))
    .collect();

  /* Merge in tile order (sequential; the per-tile update is already
  parallel across components). */
  let mut output = merge_image.clone();
  let mut manager = EventMgr::default();
  if j2k_merge_tile_results(decoded, header, &mut output, &mut manager, max_mem) == 0 {
    return None;
  }
  Some(output)
}

fn opj_j2k_move_data_from_codec_to_output_image(
  p_j2k: &mut J2k,
  p_image: &mut CoreImage,
) -> OPJ_BOOL {
  /* Move data and copy one information from codec to output image. */
  let numcomps_to_decode = p_j2k.m_specific_param.m_decoder.m_numcomps_to_decode;
  let out_image = p_j2k.m_output_image.as_mut().expect("m_output_image");
  if numcomps_to_decode > 0u32 {
    p_image.numcomps = numcomps_to_decode;
    p_image.comps.clear();
    for compno in 0..numcomps_to_decode as usize {
      let src_compno = p_j2k.m_specific_param.m_decoder.m_comps_indices_to_decode[compno] as usize;
      p_image.comps.push(out_image.comps[src_compno].clone());
    }
    for comp in &mut out_image.comps {
      comp.data.clear();
    }
  } else {
    for (comp, src_comp) in p_image.comps.iter_mut().zip(out_image.comps.iter_mut()) {
      comp.resno_decoded = src_comp.resno_decoded;
      comp.data = core::mem::take(&mut src_comp.data);
    }
  }
  1i32
}

pub(crate) fn opj_j2k_decode(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_image: &mut CoreImage,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* A lenient partial header has no codestream to decode: return the
  partial (header-only) image data instead of walking an uninitialized
  TCD. */
  if p_j2k.m_header_partial {
    return 0i32;
  }
  /* Heuristics to detect sequence opj_read_header(), opj_set_decoded_resolution_factor() */
  /* and finally opj_decode_image() without manual setting of comps[].factor */
  /* We could potentially always execute it, if we don't allow people to do */
  /* opj_read_header(), modify x0,y0,x1,y1 of returned image an call opj_decode_image() */
  if p_j2k.m_cp.m_specific_param.m_dec.m_reduce > 0u32
    && p_j2k.m_private_image.is_some()
    && p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .numcomps
      > 0u32
    && p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .comps[0]
      .factor
      == p_j2k.m_cp.m_specific_param.m_dec.m_reduce
    && p_image.numcomps > 0u32
    && p_image.comps[0].factor == 0u32
    && p_image.comps[0].data.is_empty()
  {
    let mut it_comp: u32 = 0;
    /* Update the comps[].factor member of the output image with the one */
    /* of m_reduce */
    it_comp = 0_u32;
    while it_comp < p_image.numcomps {
      p_image.comps[it_comp as usize].factor = p_j2k.m_cp.m_specific_param.m_dec.m_reduce;
      it_comp += 1;
    }
    if opj_j2k_update_image_dimensions(p_image, p_manager) == 0 {
      return 0i32;
    }
  }
  if p_j2k.m_output_image.is_none() {
    p_j2k.m_output_image = Some(Box::new(CoreImage::default()));
  }
  opj_copy_image_header_to_core(
    p_image,
    p_j2k.m_output_image.as_mut().expect("m_output_image"),
  );
  /* Decode the codestream (the C's fixed decoding list: decode_tiles). */
  if opj_j2k_decode_tiles(p_j2k, p_stream, p_manager) == 0 {
    p_j2k.m_private_image = None;
    return 0i32;
  }
  /* Move data and copy one information from codec to output image*/
  opj_j2k_move_data_from_codec_to_output_image(p_j2k, p_image)
}

pub(crate) fn opj_j2k_get_tile(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_image: &mut CoreImage,
  p_manager: &mut EventMgr,
  tile_index: u32,
) -> OPJ_BOOL {
  let mut compno: u32 = 0;
  let mut l_tile_x: u32 = 0;
  let mut l_tile_y: u32 = 0;
  if p_image.numcomps
    < p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .numcomps
  {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Image has less components than codestream.\n",
    );
    return 0i32;
  }
  if tile_index >= p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th) {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Tile index provided by the user is incorrect {} (max = {}) \n",
      tile_index,
      p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th).wrapping_sub(1u32),
    );
    return 0i32;
  }
  /* Compute the dimension of the desired tile*/
  l_tile_x = tile_index.wrapping_rem(p_j2k.m_cp.tw);
  l_tile_y = tile_index.wrapping_div(p_j2k.m_cp.tw);
  p_image.x0 = l_tile_x
    .wrapping_mul(p_j2k.m_cp.tdx)
    .wrapping_add(p_j2k.m_cp.tx0);
  if p_image.x0
    < p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .x0
  {
    p_image.x0 = p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .x0
  }
  p_image.x1 = l_tile_x
    .wrapping_add(1u32)
    .wrapping_mul(p_j2k.m_cp.tdx)
    .wrapping_add(p_j2k.m_cp.tx0);
  if p_image.x1
    > p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .x1
  {
    p_image.x1 = p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .x1
  }
  p_image.y0 = l_tile_y
    .wrapping_mul(p_j2k.m_cp.tdy)
    .wrapping_add(p_j2k.m_cp.ty0);
  if p_image.y0
    < p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .y0
  {
    p_image.y0 = p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .y0
  }
  p_image.y1 = l_tile_y
    .wrapping_add(1u32)
    .wrapping_mul(p_j2k.m_cp.tdy)
    .wrapping_add(p_j2k.m_cp.ty0);
  if p_image.y1
    > p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .y1
  {
    p_image.y1 = p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .y1
  }

  compno = 0_u32;
  while compno
    < p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .numcomps
  {
    let mut l_comp_x1: i32 = 0;
    let mut l_comp_y1: i32 = 0;
    let l_img_comp = &mut p_image.comps[compno as usize];

    l_img_comp.factor = p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .comps[compno as usize]
      .factor;

    l_img_comp.x0 = opj_uint_ceildiv(p_image.x0, l_img_comp.dx);
    l_img_comp.y0 = opj_uint_ceildiv(p_image.y0, l_img_comp.dy);
    l_comp_x1 = opj_int_ceildiv(p_image.x1 as i32, l_img_comp.dx as i32);
    l_comp_y1 = opj_int_ceildiv(p_image.y1 as i32, l_img_comp.dy as i32);

    l_img_comp.w = (opj_int_ceildivpow2(l_comp_x1, l_img_comp.factor as i32)
      - opj_int_ceildivpow2(l_img_comp.x0 as i32, l_img_comp.factor as i32))
      as u32;
    l_img_comp.h = (opj_int_ceildivpow2(l_comp_y1, l_img_comp.factor as i32)
      - opj_int_ceildivpow2(l_img_comp.y0 as i32, l_img_comp.factor as i32))
      as u32;
    compno += 1;
  }
  if p_image.numcomps
    > p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .numcomps
  {
    /* Can happen when calling repeatdly opj_get_decoded_tile() on an
     * image with a color palette, where color palette expansion is done
     * later in jp2.c */
    let n = p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .numcomps;
    p_image.comps.truncate(n as usize);
    p_image.numcomps = n
  }
  /* Destroy the previous output image and create a new one from the information
   * previously computed */
  p_j2k.m_output_image = Some(Box::new(CoreImage::default()));
  opj_copy_image_header_to_core(
    p_image,
    p_j2k.m_output_image.as_mut().expect("m_output_image"),
  );
  p_j2k.m_specific_param.m_decoder.m_tile_ind_to_dec = tile_index as i32;
  /* Decode the codestream (the C's fixed decoding-tile list:
  decode_one_tile). */
  if opj_j2k_decode_one_tile(p_j2k, p_stream, p_manager) == 0 {
    p_j2k.m_private_image = None;
    return 0i32;
  }
  /* Move data and copy one information from codec to output image*/
  opj_j2k_move_data_from_codec_to_output_image(p_j2k, p_image)
}

pub(crate) fn opj_j2k_set_decoded_resolution_factor(
  p_j2k: &mut J2k,
  res_factor: u32,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut it_comp: u32 = 0;
  p_j2k.m_cp.m_specific_param.m_dec.m_reduce = res_factor;
  let Some(default_tcp) = p_j2k.m_specific_param.m_decoder.m_default_tcp.as_deref() else {
    return 0i32;
  };
  if p_j2k.m_private_image.is_some()
    && !p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .comps
      .is_empty()
    && !default_tcp.tccps.is_empty()
  {
    let image = p_j2k
      .m_private_image
      .as_deref_mut()
      .expect("m_private_image");
    it_comp = 0_u32;
    while it_comp < image.numcomps {
      let max_res = default_tcp.tccps[it_comp as usize].numresolutions;
      if res_factor >= max_res {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Resolution factor is greater than the maximum resolution in the component.\n",
        );
        return 0i32;
      }
      image.comps[it_comp as usize].factor = res_factor;
      it_comp += 1;
    }
    return 1i32;
  }
  0i32
}

pub(crate) fn opj_j2k_encoder_set_extra_options(
  p_j2k: &mut J2k,
  options: &[&str],
  p_manager: &mut EventMgr,
) -> bool {
  for option in options {
    if option.starts_with("PLT=") {
      if *option == "PLT=YES" {
        p_j2k.m_specific_param.m_encoder.m_plt = 1i32
      } else if *option == "PLT=NO" {
        p_j2k.m_specific_param.m_encoder.m_plt = 0i32
      } else {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Invalid value for option: {}.\n",
          *option,
        );
        return false;
      }
    } else if option.starts_with("TLM=") {
      if *option == "TLM=YES" {
        p_j2k.m_specific_param.m_encoder.m_tlm = 1i32
      } else if *option == "TLM=NO" {
        p_j2k.m_specific_param.m_encoder.m_tlm = 0i32
      } else {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Invalid value for option: {}.\n",
          *option,
        );
        return false;
      }
    } else if option.starts_with("GUARD_BITS=") {
      let mut tileno: u32 = 0;
      let mut cp = &mut p_j2k.m_cp;
      /* The C parses the value with atoi: leading digits, stop at the
      first non-digit ("3x" -> 3, "" -> 0). */
      let gb_str = option.strip_prefix("GUARD_BITS=").expect("starts_with check above");
      let numgbits = gb_str
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .fold(0i32, |acc, c| {
          acc.saturating_mul(10)
            .saturating_add((c as u8).wrapping_sub(b'0') as i32)
        });
      if !(0..=7).contains(&numgbits) {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Invalid value for option: {}. Should be in [0,7]\n",
          *option,
        );
        return false;
      }
      tileno = 0_u32;
      while tileno < cp.tw.wrapping_mul(cp.th) {
        let mut i: u32 = 0;
        i = 0_u32;
        while i < p_j2k.m_specific_param.m_encoder.m_nb_comps {
          cp.tcps[tileno as usize].tccps[i as usize].numgbits = numgbits as u32;
          i += 1;
        }
        tileno += 1;
      }
    } else {
      event_msg!(p_manager, EVT_ERROR, "Invalid option: {}.\n", *option);
      return false;
    }
  }
  true
}

pub(crate) fn opj_j2k_encode(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */
  let num_tiles = p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th);
  let header = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .clone_header_only();
  let cp = p_j2k.m_cp.clone();
  let enc = p_j2k.m_specific_param.m_encoder.clone();
  /* The worker TLM buffers are clones of the master's buffer at this
  point; their new entries start at this cursor. */
  let tlm_base = p_j2k.m_specific_param.m_encoder.m_tlm_sot_offsets_current;
  let mut tile_no: u32 = 0;
  while tile_no < num_tiles {
    /* The master keeps the tile-part bookkeeping and the event (the C's
    opj_j2k_pre_write_tile); the per-tile codec state lives in the fresh
    codec created by j2k_encode_tile. */
    event_msg!(
      p_manager,
      EVT_INFO,
      "tile number {} / {}\n",
      p_j2k.m_current_tile_number.wrapping_add(1u32),
      num_tiles,
    );
    p_j2k.m_specific_param.m_encoder.m_current_tile_part_number = 0_u32;
    p_j2k.m_tcd.cur_totnum_tp = cp.tcps[tile_no as usize].m_nb_tile_parts;
    p_j2k
      .m_specific_param
      .m_encoder
      .m_current_poc_tile_part_number = 0_u32;
    let (bytes, tlm_buf, tlm_cur) = match j2k_encode_tile(
      &cp,
      &header,
      &enc,
      tile_no,
      p_j2k.m_private_image.as_deref().expect("m_private_image"),
      p_manager,
    ) {
      Some(r) => r,
      None => {
        return 0i32;
      }
    };
    if opj_stream_write_data(p_stream, &bytes, p_manager) != bytes.len() {
      return 0i32;
    }
    merge_tile_tlm(p_j2k, &tlm_buf, tlm_cur, tlm_base);
    p_j2k.m_current_tile_number = p_j2k.m_current_tile_number.wrapping_add(1);
    tile_no += 1;
  }
  1i32
}

/// Encodes one tile into an in-memory buffer with a fresh codec state
/// (the C's pre_write_tile + tile-data copy + post_write_tile flow, minus
/// the stream writes).  THE single per-tile encode path: the serial
/// `opj_j2k_encode` calls it in a loop and the parallel driver under
/// rayon, so byte-identity is structural.  `private` is a header-only
/// image; `real` carries the shared read-only pixel data.
pub(crate) fn j2k_encode_tile(
  cp: &Cp,
  private: &CoreImage,
  enc: &J2kEnc,
  tile_no: u32,
  real: &CoreImage,
  p_manager: &mut EventMgr,
) -> Option<(Vec<u8>, Vec<u8>, usize)> {
  let mut j2k = opj_j2k_create_compress()?;
  j2k.m_cp = cp.clone();
  j2k.m_private_image = Some(Box::new(private.clone()));
  j2k.m_specific_param.m_encoder = enc.clone();
  /* The worker records its own tile-part TLM entries (the buffer is a
  clone of the master's at call time); the master merges them in tile
  order. */
  j2k.m_current_tile_number = tile_no;
  if opj_tcd_init(&mut j2k.m_tcd, private, &mut j2k.m_cp) == 0 {
    return None;
  }
  if opj_j2k_alloc_encoded_tile_data(&mut j2k, p_manager) == 0 {
    return None;
  }
  /* opj_j2k_pre_write_tile equivalent */
  j2k.m_specific_param.m_encoder.m_current_tile_part_number = 0_u32;
  j2k.m_tcd.cur_totnum_tp = j2k.m_cp.tcps[tile_no as usize].m_nb_tile_parts;
  j2k
    .m_specific_param
    .m_encoder
    .m_current_poc_tile_part_number = 0_u32;
  if opj_tcd_init_encode_tile(
    &mut j2k.m_tcd,
    tile_no,
    p_manager,
    j2k.m_private_image.as_deref_mut().expect("m_private_image"),
    &mut j2k.m_cp,
  ) == 0
  {
    return None;
  }
  /* Extract this tile's pixels (shared read-only image) into the tile
  components. */
  {
    let image = j2k.m_private_image.as_deref_mut().expect("m_private_image");
    let mut j = 0u32;
    while j < image.numcomps {
      let l_tilec = &mut j2k.m_tcd.tcd_image.tiles.comps[j as usize];
      if opj_alloc_tile_component_data(l_tilec, false) == 0 {
        return None;
      }
      j += 1;
    }
    let tile_size = opj_tcd_get_encoder_input_buffer_size(&mut j2k.m_tcd, real);
    let mut buf = vec![0u8; tile_size];
    opj_j2k_get_tile_data(&mut j2k.m_tcd, &mut buf[..], real);
    if opj_tcd_copy_tile_data(&mut j2k.m_tcd, &buf[..], image) == 0 {
      return None;
    }
  }
  /* opj_j2k_post_write_tile against a per-worker memory stream. */
  let writer = SharedVecWriter {
    buf: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
    pos: 0,
  };
  let out_buf = writer.buf.clone();
  let mut worker_stream = Stream::new_output(OPJ_J2K_STREAM_CHUNK_SIZE as usize, writer);
  if opj_j2k_post_write_tile(&mut j2k, &mut worker_stream, p_manager) == 0 {
    return None;
  }
  drop(worker_stream);
  let bytes = match std::rc::Rc::try_unwrap(out_buf) {
    Ok(b) => b.into_inner(),
    Err(_) => return None,
  };
  let enc = &j2k.m_specific_param.m_encoder;
  let tlm_buf = enc.m_tlm_sot_offsets_buffer.clone();
  let tlm_cur = enc.m_tlm_sot_offsets_current;
  Some((bytes, tlm_buf, tlm_cur))
}

/// Encodes every tile in parallel (feature `rayon`) and writes the
/// tile-parts sequentially, so the codestream is byte-identical to the
/// serial encode.  Returns 0 when a worker fails (the caller then falls
/// back to the serial encode).  The master's TLM state is updated in
/// tile order.
#[cfg(feature = "rayon")]
pub(crate) fn j2k_encode_tiles_parallel(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  use rayon::prelude::*;
  let num_tiles = p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th);
  if num_tiles <= 1 {
    return 0i32;
  }
  let header = p_j2k
    .m_private_image
    .as_deref()
    .expect("m_private_image")
    .clone_header_only();
  let cp = p_j2k.m_cp.clone();
  let enc = p_j2k.m_specific_param.m_encoder.clone();
  let tlm_base = p_j2k.m_specific_param.m_encoder.m_tlm_sot_offsets_current;
  /* Encode every tile in parallel; the workers read the pixels from the
  shared image (read-only) and write their tile-parts (and their TLM
  entries) into per-worker buffers. */
  let results: Vec<Option<(Vec<u8>, Vec<u8>, usize)>> = {
    let real = p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image");
    (0..num_tiles)
      .into_par_iter()
      .map(|tile_no| {
        let mut manager = EventMgr::default();
        j2k_encode_tile(&cp, &header, &enc, tile_no, real, &mut manager)
      })
      .collect()
  };
  if results.iter().any(Option::is_none) {
    return 0i32;
  }
  /* Write the tile-parts in order, merging the per-part TLM entries the
  same way the serial loop does. */
  for result in results {
    let (bytes, tlm_buf, tlm_cur) = result.expect("checked above");
    if opj_stream_write_data(p_stream, &bytes, p_manager) != bytes.len() {
      return 0i32;
    }
    merge_tile_tlm(p_j2k, &tlm_buf, tlm_cur, tlm_base);
    p_j2k.m_current_tile_number = p_j2k.m_current_tile_number.wrapping_add(1);
  }
  1i32
}

pub(crate) fn opj_j2k_end_compress(
  mut p_j2k: &mut J2k,
  mut p_stream: &mut Stream,
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* The C's fixed end-of-compress procedure list. */
  (opj_j2k_write_eoc(p_j2k, p_stream, p_manager) != 0
    && (p_j2k.m_specific_param.m_encoder.m_tlm == 0
      || opj_j2k_write_updated_tlm(p_j2k, p_stream, p_manager) != 0)
    && opj_j2k_write_epc(p_j2k, p_stream, p_manager) != 0
    && opj_j2k_end_encoding(p_j2k, p_stream, p_manager) != 0
    && opj_j2k_destroy_header_memory(p_j2k, p_stream, p_manager) != 0) as i32
}

pub(crate) fn opj_j2k_start_compress(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_image: &mut CoreImage,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */

  {
    /* The C code moves ownership of the component data and ICC profile into
    the codec but keeps the caller's header fields (x0/y0/x1/y1/numcomps)
    readable, so only the owned buffers are taken. */
    let mut private = CoreImage {
      x0: p_image.x0,
      y0: p_image.y0,
      x1: p_image.x1,
      y1: p_image.y1,
      numcomps: p_image.numcomps,
      color_space: p_image.color_space,
      comps: core::mem::take(&mut p_image.comps),
      icc_profile_buf: core::mem::take(&mut p_image.icc_profile_buf),
      icc_profile_is_cielab: p_image.icc_profile_is_cielab,
    };
    p_image.icc_profile_is_cielab = false;
    p_j2k.m_private_image = Some(Box::new(private));
    /* validation of the parameters codec (the C's fixed encoding
    validation list: build_encoder, encoding_validation, mct_validation). */
    if opj_j2k_build_encoder(p_j2k, p_stream, p_manager) == 0
      || opj_j2k_encoding_validation(p_j2k, p_stream, p_manager) == 0
      || opj_j2k_mct_validation(p_j2k, p_stream, p_manager) == 0
    {
      return 0i32;
    }
    /* write header (the C's fixed header-writing list). */
    if opj_j2k_write_header(p_j2k, p_stream, p_manager) == 0 {
      return 0i32;
    }
    1i32
  }
}

fn opj_j2k_pre_write_tile(
  mut p_j2k: &mut J2k,
  mut p_tile_index: u32,
  mut _p_stream: &mut Stream,
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  if p_tile_index != p_j2k.m_current_tile_number {
    event_msg!(p_manager, EVT_ERROR, "The given tile index does not match.",);
    return 0i32;
  }
  event_msg!(
    p_manager,
    EVT_INFO,
    "tile number {} / {}\n",
    p_j2k.m_current_tile_number.wrapping_add(1u32),
    p_j2k.m_cp.tw.wrapping_mul(p_j2k.m_cp.th),
  );
  p_j2k.m_specific_param.m_encoder.m_current_tile_part_number = 0_u32;
  p_j2k.m_tcd.cur_totnum_tp = p_j2k.m_cp.tcps[p_tile_index as usize].m_nb_tile_parts;
  p_j2k
    .m_specific_param
    .m_encoder
    .m_current_poc_tile_part_number = 0_u32;
  /* initialisation before tile encoding  */
  if opj_tcd_init_encode_tile(
    &mut p_j2k.m_tcd,
    p_j2k.m_current_tile_number,
    p_manager,
    p_j2k
      .m_private_image
      .as_deref_mut()
      .expect("m_private_image"),
    &mut p_j2k.m_cp,
  ) == 0
  {
    return 0i32;
  } /* (/8) */
  1i32 /* (%8) */
}

fn opj_get_tile_dimensions(
  _l_image: &CoreImage,
  l_tilec: &TcdTileComp,
  l_img_comp: &CoreImageComp,
  l_size_comp: &mut u32,
  l_width: &mut u32,
  l_height: &mut u32,
  l_offset_x: &mut u32,
  l_offset_y: &mut u32,
  l_image_width: &mut u32,
  l_stride: &mut u32,
  l_tile_offset: &mut u32,
) {
  let mut l_remaining: u32 = 0;
  *l_size_comp = l_img_comp.prec >> 3i32;
  l_remaining = l_img_comp.prec & 7u32;
  if l_remaining != 0 {
    *l_size_comp = l_size_comp.wrapping_add(1u32)
  }
  if *l_size_comp == 3u32 {
    *l_size_comp = 4_u32
  }

  *l_width = (l_tilec.x1 - l_tilec.x0) as u32;
  *l_height = (l_tilec.y1 - l_tilec.y0) as u32;
  /* The component plane dimensions as stored: ceildiv(x1, dx) -
  ceildiv(x0, dx).  The C computes the width as ceildiv(x1 - x0, dx),
  which disagrees when the image origin is not aligned to the
  subsampling factor (e.g. issue726.j2k: x0=123, dx=2 gives 3 instead
  of 2) and makes the C read past the component buffer. */
  *l_offset_x = l_img_comp.x0;
  *l_offset_y = l_img_comp.y0;
  *l_image_width = l_img_comp.w;
  *l_stride = l_image_width.wrapping_sub(*l_width);
  *l_tile_offset = (l_tilec.x0 as u32).wrapping_sub(*l_offset_x).wrapping_add(
    (l_tilec.y0 as u32)
      .wrapping_sub(*l_offset_y)
      .wrapping_mul(*l_image_width),
  );
}

fn opj_j2k_get_tile_data(p_tcd: &mut Tcd, mut p_data: &mut [u8], l_image: &CoreImage) {
  let numcomps = l_image.numcomps as usize;
  let l_tilec = &p_tcd.tcd_image.tiles.comps[..numcomps];
  for (l_tilec, l_img_comp) in l_tilec.iter().zip(l_image.comps.iter()) {
    let mut l_size_comp: u32 = 0;
    let mut l_width: u32 = 0;
    let mut l_height: u32 = 0;
    let mut l_offset_x: u32 = 0;
    let mut l_offset_y: u32 = 0;
    let mut l_image_width: u32 = 0;
    let mut l_stride: u32 = 0;
    let mut l_tile_offset: u32 = 0;
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
    let l_height = l_height as usize;
    let l_width = l_width as usize;
    let l_stride = l_stride as usize;
    let l_nb_elem = l_height * l_width;
    /* A component can subsample to a zero width/height inside a tile
    (subsampling larger than the tile size): the C's copy loops no-op and
    the component contributes nothing.  chunks(0)/chunks_exact(0) would
    panic here. */
    if l_width == 0usize || l_height == 0usize {
      continue;
    }
    /* The C reads `l_height` rows of `l_width` samples with `l_stride` between rows. The
     * c2rust transcription modelled that as `chunks_exact(l_width + l_stride)`, which demands a
     * full trailing stride that the C never reads (and the component buffer may not have, for
     * bottom-edge tiles); `chunks` accepts the short final chunk instead. */
    let l_src =
      &l_img_comp.data[(l_tile_offset as usize).min(l_img_comp.data.len())..];
    match l_size_comp {
      1 => {
        let (dest, remain) = p_data.split_at_mut(l_nb_elem);
        p_data = remain;
        if l_img_comp.sgnd != 0 {
          for (src, dest) in l_src
            .chunks(l_width + l_stride)
            .zip(dest.chunks_exact_mut(l_width))
          {
            let src = &src[..l_width.min(src.len())]; /* pad with zeros if the component buffer is short */
            for (src, dest) in src.iter().zip(dest.iter_mut()) {
              *dest = *src as i8 as u8;
            }
          }
        } else {
          for (src, dest) in l_src
            .chunks(l_width + l_stride)
            .zip(dest.chunks_exact_mut(l_width))
          {
            let src = &src[..l_width.min(src.len())]; /* pad with zeros if the component buffer is short */
            for (src, dest) in src.iter().zip(dest.iter_mut()) {
              *dest = (*src & 0xffi32) as u8;
            }
          }
        }
      }
      2 => {
        let (dest, remain) = p_data.split_at_mut(l_nb_elem * 2);
        p_data = remain;
        if l_img_comp.sgnd != 0 {
          for (src, dest) in l_src
            .chunks(l_width + l_stride)
            .zip(dest.chunks_exact_mut(l_width * 2))
          {
            let src = &src[..l_width.min(src.len())]; /* pad with zeros if the component buffer is short */
            let (chunks, _) = dest.as_chunks_mut::<2>();
            for (src, dest) in src.iter().zip(chunks) {
              let val = *src as i16;
              *dest = val.to_ne_bytes();
            }
          }
        } else {
          for (src, dest) in l_src
            .chunks(l_width + l_stride)
            .zip(dest.chunks_exact_mut(l_width * 2))
          {
            let src = &src[..l_width.min(src.len())]; /* pad with zeros if the component buffer is short */
            let (chunks, _) = dest.as_chunks_mut::<2>();
            for (src, dest) in src.iter().zip(chunks) {
              let val = (*src & 0xffffi32) as i16;
              *dest = val.to_ne_bytes();
            }
          }
        }
      }
      4 => {
        let (dest, remain) = p_data.split_at_mut(l_nb_elem * 4);
        p_data = remain;
        for (src, dest) in l_src
          .chunks(l_width + l_stride)
          .zip(dest.chunks_exact_mut(l_width * 4))
        {
          let src = &src[..l_width.min(src.len())]; /* pad with zeros if the component buffer is short */
          let (chunks, _) = dest.as_chunks_mut::<4>();
          for (src, dest) in src.iter().zip(chunks) {
            *dest = src.to_ne_bytes();
          }
        }
      }
      _ => {}
    }
  }
}

fn opj_j2k_post_write_tile(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_nb_bytes_written: u32 = 0;
  let mut l_tile_size = 0_u32;
  let mut l_available_data: u32 = 0;
  /* preconditions */
  assert!(!p_j2k
    .m_specific_param
    .m_encoder
    .m_encoded_tile_data
    .is_empty());
  l_tile_size = p_j2k.m_specific_param.m_encoder.m_encoded_tile_data.len() as u32;
  l_available_data = l_tile_size;
  let mut l_encoded_buf =
    core::mem::take(&mut p_j2k.m_specific_param.m_encoder.m_encoded_tile_data);
  l_nb_bytes_written = 0_u32;
  /* The take() above moved the buffer out; restore it on every failure path
  so a subsequent write_tile call does not hit the assert at the top (the C
  keeps the buffer). */
  macro_rules! fail {
    () => {
      p_j2k.m_specific_param.m_encoder.m_encoded_tile_data = l_encoded_buf;
      return 0i32;
    };
  }
  if opj_j2k_write_first_tile_part(
    p_j2k,
    &mut l_encoded_buf[..],
    &mut l_nb_bytes_written,
    p_stream,
    p_manager,
  ) == 0
  {
    fail!();
  }
  let l_current_offset = l_nb_bytes_written as usize;
  l_available_data = (l_available_data as core::ffi::c_uint).wrapping_sub(l_nb_bytes_written);
  l_nb_bytes_written = 0_u32;
  if opj_j2k_write_all_tile_parts(
    p_j2k,
    &mut l_encoded_buf[l_current_offset..],
    &mut l_nb_bytes_written,
    p_stream,
    p_manager,
  ) == 0
  {
    fail!();
  }
  l_available_data = (l_available_data as core::ffi::c_uint).wrapping_sub(l_nb_bytes_written);
  l_nb_bytes_written = l_tile_size.wrapping_sub(l_available_data);
  if opj_stream_write_data(
    p_stream,
    &l_encoded_buf[..l_nb_bytes_written as usize],
    p_manager,
  ) != l_nb_bytes_written as usize
  {
    fail!();
  }
  p_j2k.m_specific_param.m_encoder.m_encoded_tile_data = l_encoded_buf;
  p_j2k.m_current_tile_number = p_j2k.m_current_tile_number.wrapping_add(1);
  1i32
}

/* *
 * Writes the codestream header (the C's fixed header-writing procedure
 * list, inlined: init_info, SOC, SIZ, COD, QCD, all COC/QCC, optional TLM
 * (+POC for profile 4), regions, optional COM, optional MCT data group,
 * optional end-header marker, create_tcd, update_rates).  Extensions go
 * here directly.
 */
fn opj_j2k_write_header(
  p_j2k: &mut J2k,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  (opj_j2k_init_info(p_j2k, p_stream, p_manager) != 0
    && opj_j2k_write_soc(p_j2k, p_stream, p_manager) != 0
    && opj_j2k_write_siz(p_j2k, p_stream, p_manager) != 0
    && opj_j2k_write_cod(p_j2k, p_stream, p_manager) != 0
    && opj_j2k_write_qcd(p_j2k, p_stream, p_manager) != 0
    && opj_j2k_write_all_coc(p_j2k, p_stream, p_manager) != 0
    && opj_j2k_write_all_qcc(p_j2k, p_stream, p_manager) != 0
    && (p_j2k.m_specific_param.m_encoder.m_tlm == 0
      || (opj_j2k_write_tlm(p_j2k, p_stream, p_manager) != 0
        && (p_j2k.m_cp.rsiz as core::ffi::c_int != 0x4i32
          || opj_j2k_write_poc(p_j2k, p_stream, p_manager) != 0)))
    && opj_j2k_write_regions(p_j2k, p_stream, p_manager) != 0
    && ((p_j2k.m_cp.comment.is_empty() && !p_j2k.m_cp.comment_set)
      || opj_j2k_write_com(p_j2k, p_stream, p_manager) != 0)
    && (p_j2k.m_cp.rsiz as core::ffi::c_int & (0x8000i32 | 0x100i32)
      != 0x8000i32 | 0x100i32
      || opj_j2k_write_mct_data_group(p_j2k, p_stream, p_manager) != 0)
    && (p_j2k.cstr_index.is_none()
      || opj_j2k_get_end_header(p_j2k, p_stream, p_manager) != 0)
    && opj_j2k_create_tcd(p_j2k, p_stream, p_manager) != 0
    && opj_j2k_update_rates(p_j2k, p_stream, p_manager) != 0) as i32
}

fn opj_j2k_write_first_tile_part(
  p_j2k: &mut J2k,
  data: &mut [u8],
  p_data_written: &mut u32,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut pos: usize = 0;
  let mut l_begin_pos: usize = 0;
  let mut total_data_size = data.len() as u32;
  let mut l_nb_bytes_written = 0_u32;
  let mut l_current_nb_bytes_written: u32 = 0;
  p_j2k.m_tcd.cur_pino = 0_u32;
  /*Get number of tile parts*/
  p_j2k
    .m_specific_param
    .m_encoder
    .m_current_poc_tile_part_number = 0_u32;
  /* INDEX >> */
  /* << INDEX */
  l_current_nb_bytes_written = 0_u32;
  l_begin_pos = pos;
  if opj_j2k_write_sot(
    p_j2k,
    &mut data[pos..],
    &mut l_current_nb_bytes_written,
    p_stream,
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  l_nb_bytes_written =
    (l_nb_bytes_written as core::ffi::c_uint).wrapping_add(l_current_nb_bytes_written);
  pos += l_current_nb_bytes_written as usize;
  total_data_size = (total_data_size as core::ffi::c_uint).wrapping_sub(l_current_nb_bytes_written);
  let rsiz = p_j2k.m_cp.rsiz as core::ffi::c_int;
  let tile_has_poc = p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize].poc;
  if !(0x3i32..=0x6i32).contains(&rsiz) && tile_has_poc {
    l_current_nb_bytes_written = 0_u32;
    opj_j2k_write_poc_in_memory(
      p_j2k,
      &mut data[pos..],
      &mut l_current_nb_bytes_written,
      p_manager,
    );
    l_nb_bytes_written =
      (l_nb_bytes_written as core::ffi::c_uint).wrapping_add(l_current_nb_bytes_written);
    pos += l_current_nb_bytes_written as usize;
    total_data_size =
      (total_data_size as core::ffi::c_uint).wrapping_sub(l_current_nb_bytes_written)
  }
  l_current_nb_bytes_written = 0_u32;
  if opj_j2k_write_sod(
    p_j2k,
    &mut data[pos..],
    &mut l_current_nb_bytes_written,
    p_stream,
    p_manager,
  ) == 0
  {
    return 0i32;
  }
  l_nb_bytes_written =
    (l_nb_bytes_written as core::ffi::c_uint).wrapping_add(l_current_nb_bytes_written);
  *p_data_written = l_nb_bytes_written;
  /* Writing Psot in SOT marker */
  opj_write_bytes(&mut data[l_begin_pos + 6..], l_nb_bytes_written, 4_u32); /* PSOT */
  if p_j2k.m_specific_param.m_encoder.m_tlm != 0 {
    opj_j2k_update_tlm(p_j2k, l_nb_bytes_written);
  }
  1i32
}
fn opj_j2k_write_all_tile_parts(
  p_j2k: &mut J2k,
  data: &mut [u8],
  p_data_written: &mut u32,
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut pos: usize = 0;
  let mut l_begin_pos: usize = 0;
  let mut total_data_size = data.len() as u32;
  let mut tilepartno = 0_u32;
  let mut l_nb_bytes_written = 0_u32;
  let mut l_current_nb_bytes_written: u32 = 0;
  let mut l_part_tile_size: u32 = 0;
  let mut tot_num_tp: u32 = 0;
  let mut pino: u32 = 0;
  /*Get number of tile parts*/
  tot_num_tp = opj_j2k_get_num_tp(&mut p_j2k.m_cp, 0_u32, p_j2k.m_current_tile_number);
  /* start writing remaining tile parts */
  p_j2k.m_specific_param.m_encoder.m_current_tile_part_number = p_j2k
    .m_specific_param
    .m_encoder
    .m_current_tile_part_number
    .wrapping_add(1);
  tilepartno = 1_u32;
  while tilepartno < tot_num_tp {
    p_j2k
      .m_specific_param
      .m_encoder
      .m_current_poc_tile_part_number = tilepartno;
    l_current_nb_bytes_written = 0_u32;
    l_part_tile_size = 0_u32;
    l_begin_pos = pos;
    if opj_j2k_write_sot(
      p_j2k,
      &mut data[pos..],
      &mut l_current_nb_bytes_written,
      p_stream,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    l_nb_bytes_written =
      (l_nb_bytes_written as core::ffi::c_uint).wrapping_add(l_current_nb_bytes_written);
    pos += l_current_nb_bytes_written as usize;
    total_data_size =
      (total_data_size as core::ffi::c_uint).wrapping_sub(l_current_nb_bytes_written);
    l_part_tile_size =
      (l_part_tile_size as core::ffi::c_uint).wrapping_add(l_current_nb_bytes_written);
    l_current_nb_bytes_written = 0_u32;
    if opj_j2k_write_sod(
      p_j2k,
      &mut data[pos..],
      &mut l_current_nb_bytes_written,
      p_stream,
      p_manager,
    ) == 0
    {
      return 0i32;
    }
    pos += l_current_nb_bytes_written as usize;
    l_nb_bytes_written =
      (l_nb_bytes_written as core::ffi::c_uint).wrapping_add(l_current_nb_bytes_written);
    total_data_size =
      (total_data_size as core::ffi::c_uint).wrapping_sub(l_current_nb_bytes_written);
    l_part_tile_size =
      (l_part_tile_size as core::ffi::c_uint).wrapping_add(l_current_nb_bytes_written);
    /* Writing Psot in SOT marker */
    opj_write_bytes(&mut data[l_begin_pos + 6..], l_part_tile_size, 4_u32); /* PSOT */
    if p_j2k.m_specific_param.m_encoder.m_tlm != 0 {
      opj_j2k_update_tlm(p_j2k, l_part_tile_size);
    }
    p_j2k.m_specific_param.m_encoder.m_current_tile_part_number = p_j2k
      .m_specific_param
      .m_encoder
      .m_current_tile_part_number
      .wrapping_add(1);
    tilepartno += 1;
  }
  pino = 1_u32;
  while pino <= p_j2k.m_cp.tcps[p_j2k.m_current_tile_number as usize].numpocs {
    p_j2k.m_tcd.cur_pino = pino;
    /*Get number of tile parts*/
    tot_num_tp = opj_j2k_get_num_tp(&mut p_j2k.m_cp, pino, p_j2k.m_current_tile_number);
    tilepartno = 0_u32;
    while tilepartno < tot_num_tp {
      p_j2k
        .m_specific_param
        .m_encoder
        .m_current_poc_tile_part_number = tilepartno;
      l_current_nb_bytes_written = 0_u32;
      l_part_tile_size = 0_u32;
      l_begin_pos = pos;
      if opj_j2k_write_sot(
        p_j2k,
        &mut data[pos..],
        &mut l_current_nb_bytes_written,
        p_stream,
        p_manager,
      ) == 0
      {
        return 0i32;
      }
      l_nb_bytes_written =
        (l_nb_bytes_written as core::ffi::c_uint).wrapping_add(l_current_nb_bytes_written);
      pos += l_current_nb_bytes_written as usize;
      total_data_size =
        (total_data_size as core::ffi::c_uint).wrapping_sub(l_current_nb_bytes_written);
      l_part_tile_size =
        (l_part_tile_size as core::ffi::c_uint).wrapping_add(l_current_nb_bytes_written);
      l_current_nb_bytes_written = 0_u32;
      if opj_j2k_write_sod(
        p_j2k,
        &mut data[pos..],
        &mut l_current_nb_bytes_written,
        p_stream,
        p_manager,
      ) == 0
      {
        return 0i32;
      }
      l_nb_bytes_written =
        (l_nb_bytes_written as core::ffi::c_uint).wrapping_add(l_current_nb_bytes_written);
      pos += l_current_nb_bytes_written as usize;
      total_data_size =
        (total_data_size as core::ffi::c_uint).wrapping_sub(l_current_nb_bytes_written);
      l_part_tile_size =
        (l_part_tile_size as core::ffi::c_uint).wrapping_add(l_current_nb_bytes_written);
      /* Writing Psot in SOT marker */
      opj_write_bytes(&mut data[l_begin_pos + 6..], l_part_tile_size, 4_u32); /* PSOT */
      if p_j2k.m_specific_param.m_encoder.m_tlm != 0 {
        opj_j2k_update_tlm(p_j2k, l_part_tile_size);
      }
      p_j2k.m_specific_param.m_encoder.m_current_tile_part_number = p_j2k
        .m_specific_param
        .m_encoder
        .m_current_tile_part_number
        .wrapping_add(1);
      tilepartno += 1;
    }
    pino += 1;
  }
  *p_data_written = l_nb_bytes_written;
  1i32
}
fn opj_j2k_write_updated_tlm(
  mut p_j2k: &mut J2k,
  mut p_stream: &mut Stream,
  mut p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  let mut l_tlm_size: u32 = 0;
  let mut l_tlm_position: i64 = 0;
  let mut l_current_position: i64 = 0;
  let mut size_per_tile_part: u32 = 0;
  /* preconditions */

  size_per_tile_part = if p_j2k.m_specific_param.m_encoder.m_ttlmi_is_byte != 0 {
    5i32
  } else {
    6i32
  } as u32;
  l_tlm_size = size_per_tile_part.wrapping_mul(p_j2k.m_specific_param.m_encoder.m_total_tile_parts);
  l_tlm_position = 6i64 + p_j2k.m_specific_param.m_encoder.m_tlm_start;
  l_current_position = opj_stream_tell(p_stream);
  if opj_stream_seek(p_stream, l_tlm_position, p_manager) == 0 {
    return 0i32;
  }
  if opj_stream_write_data(
    p_stream,
    &p_j2k.m_specific_param.m_encoder.m_tlm_sot_offsets_buffer[..l_tlm_size as usize],
    p_manager,
  ) != l_tlm_size as usize
  {
    return 0i32;
  }
  if opj_stream_seek(p_stream, l_current_position, p_manager) == 0 {
    return 0i32;
  }
  1i32
}

/* *
 * Ends the encoding, i.e. frees memory.
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_end_encoding(
  mut p_j2k: &mut J2k,
  mut _p_stream: &mut Stream,
  mut _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */
  p_j2k.m_specific_param.m_encoder.m_tlm_sot_offsets_buffer = Vec::new();
  p_j2k.m_specific_param.m_encoder.m_tlm_sot_offsets_current = 0;
  p_j2k.m_specific_param.m_encoder.m_encoded_tile_data = Vec::new();
  1i32
}

/* *
 * Destroys the memory associated with the decoding of headers.
 */
/* *
 * Destroys the memory associated with the decoding of headers.
 */
fn opj_j2k_destroy_header_memory(
  mut p_j2k: &mut J2k,
  mut _p_stream: &mut Stream,
  mut _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */

  p_j2k.m_specific_param.m_encoder.m_header_tile_data = Vec::new();
  1i32
}

/* *
 * Inits the Info
 *
 * @param       p_stream                the stream to write data to.
 * @param       p_j2k                   J2K codec.
 * @param       p_manager               the user event manager.
*/
fn opj_j2k_create_tcd(
  mut p_j2k: &mut J2k,
  mut _p_stream: &mut Stream,
  mut _p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  /* preconditions */

  if opj_tcd_init(
    &mut p_j2k.m_tcd,
    p_j2k.m_private_image.as_deref().expect("m_private_image"),
    &mut p_j2k.m_cp,
  ) == 0
  {
    return 0i32;
  }
  1i32
}

pub(crate) fn opj_j2k_write_tile(
  p_j2k: &mut J2k,
  p_tile_index: u32,
  p_data: &[u8],
  p_stream: &mut Stream,
  p_manager: &mut EventMgr,
) -> OPJ_BOOL {
  if opj_j2k_pre_write_tile(p_j2k, p_tile_index, p_stream, p_manager) == 0 {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Error while opj_j2k_pre_write_tile with tile index = {}\n",
      p_tile_index,
    );
    return 0i32;
  } else {
    let mut j: u32 = 0;
    /* Allocate data */
    j = 0_u32;
    let numcomps = p_j2k
      .m_private_image
      .as_deref()
      .expect("m_private_image")
      .numcomps;
    while j < numcomps {
      let l_tilec = &mut p_j2k.m_tcd.tcd_image.tiles.comps[j as usize];
      if opj_alloc_tile_component_data(l_tilec, false) == 0 {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Error allocating tile component data.",
        );
        return 0i32;
      }
      j += 1;
    }
    /* now copy data into the tile component */
    if opj_tcd_copy_tile_data(
      &mut p_j2k.m_tcd,
      p_data,
      p_j2k
        .m_private_image
        .as_deref_mut()
        .expect("m_private_image"),
    ) == 0
    {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Size mismatch between tile data and sent data.",
      );
      return 0i32;
    }
    if opj_j2k_post_write_tile(p_j2k, p_stream, p_manager) == 0 {
      event_msg!(
        p_manager,
        EVT_ERROR,
        "Error while opj_j2k_post_write_tile with tile index = {}\n",
        p_tile_index,
      );
      return 0i32;
    }
  }
  1i32
}
