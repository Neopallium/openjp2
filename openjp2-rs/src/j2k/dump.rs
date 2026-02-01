use crate::image::*;
use crate::openjpeg::*;

#[cfg(feature = "file-io")]
use ::libc::FILE;

use super::*;

#[cfg(feature = "file-io")]
fn opj_j2k_dump_tile_info(l_default_tile: &opj_tcp_t, numcomps: OPJ_INT32, out_stream: *mut FILE) {
  unsafe {
    let mut compno: OPJ_INT32 = 0;
    fprintf!(out_stream, "\t default tile {{\n");
    if (*l_default_tile).csty != 0 {
      fprintf!(out_stream, "\t\t csty={:#x}\n", (*l_default_tile).csty);
    } else {
      fprintf!(out_stream, "\t\t csty=0\n");
    }
    if (*l_default_tile).prg != 0 {
      fprintf!(
        out_stream,
        "\t\t prg={:#x}\n",
        (*l_default_tile).prg as core::ffi::c_int,
      );
    } else {
      fprintf!(out_stream, "\t\t prg=0\n");
    }
    fprintf!(
      out_stream,
      "\t\t numlayers={}\n",
      (*l_default_tile).numlayers,
    );
    fprintf!(out_stream, "\t\t mct={:x}\n", (*l_default_tile).mct);
    /*end of default tile*/
    compno = 0i32; /*end of component of default tile*/
    while compno < numcomps {
      let mut l_tccp: *mut opj_tccp_t =
        &mut *(*l_default_tile).tccps.offset(compno as isize) as *mut opj_tccp_t;
      let mut resno: OPJ_UINT32 = 0;
      let mut bandno: OPJ_INT32 = 0;
      let mut numbands: OPJ_INT32 = 0;
      /* coding style*/
      fprintf!(out_stream, "\t\t comp {} {{\n", compno);
      if (*l_tccp).csty != 0 {
        fprintf!(out_stream, "\t\t\t csty={:#x}\n", (*l_tccp).csty);
      } else {
        fprintf!(out_stream, "\t\t\t csty=0\n");
      }
      fprintf!(
        out_stream,
        "\t\t\t numresolutions={}\n",
        (*l_tccp).numresolutions,
      );
      fprintf!(out_stream, "\t\t\t cblkw=2^{}\n", (*l_tccp).cblkw);
      fprintf!(out_stream, "\t\t\t cblkh=2^{}\n", (*l_tccp).cblkh);
      if (*l_tccp).cblksty != 0 {
        fprintf!(out_stream, "\t\t\t cblksty={:#x}\n", (*l_tccp).cblksty);
      } else {
        fprintf!(out_stream, "\t\t\t cblksty=0\n");
      }
      fprintf!(out_stream, "\t\t\t qmfbid={}\n", (*l_tccp).qmfbid);
      fprintf!(out_stream, "\t\t\t preccintsize (w,h)=");
      resno = 0 as OPJ_UINT32;
      while resno < (*l_tccp).numresolutions {
        fprintf!(
          out_stream,
          "({},{}) ",
          (*l_tccp).prcw[resno as usize],
          (*l_tccp).prch[resno as usize],
        );
        resno += 1;
      }
      fprintf!(out_stream, "\n");
      /* quantization style*/
      fprintf!(out_stream, "\t\t\t qntsty={}\n", (*l_tccp).qntsty);
      fprintf!(out_stream, "\t\t\t numgbits={}\n", (*l_tccp).numgbits);
      fprintf!(out_stream, "\t\t\t stepsizes (m,e)=");
      numbands = if (*l_tccp).qntsty == 1u32 {
        1i32
      } else {
        ((*l_tccp).numresolutions as OPJ_INT32 * 3i32) - 2i32
      };
      bandno = 0i32;
      while bandno < numbands {
        fprintf!(
          out_stream,
          "({},{}) ",
          (*l_tccp).stepsizes[bandno as usize].mant,
          (*l_tccp).stepsizes[bandno as usize].expn,
        );
        bandno += 1
      }
      fprintf!(out_stream, "\n");
      /* RGN value*/
      fprintf!(out_stream, "\t\t\t roishift={}\n", (*l_tccp).roishift);
      fprintf!(out_stream, "\t\t }}\n");
      compno += 1
    }
    fprintf!(out_stream, "\t }}\n");
  };
}

#[cfg(feature = "file-io")]
pub(crate) fn j2k_dump(p_j2k: &opj_j2k, flag: OPJ_INT32, out_stream: *mut FILE) {
  unsafe {
    /* Check if the flag is compatible with j2k file*/
    if flag & 128i32 != 0 || flag & 256i32 != 0 {
      fprintf!(out_stream, "Wrong flag\n");
      return;
    }
    /* Dump the image_header */
    if flag & 1i32 != 0 && !p_j2k.m_private_image.is_null() {
      j2k_dump_image_header(&mut *p_j2k.m_private_image, 0i32, out_stream);
    }
    /* Dump the codestream info from main header */
    if flag & 2i32 != 0 && !p_j2k.m_private_image.is_null() {
      opj_j2k_dump_MH_info(p_j2k, out_stream);
    }
    /* Dump all tile/codestream info */
    if flag & 8i32 != 0 {
      let mut l_nb_tiles = p_j2k.m_cp.th.wrapping_mul(p_j2k.m_cp.tw);
      let mut i: OPJ_UINT32 = 0;
      let mut l_tcp = p_j2k.m_cp.tcps;
      if !p_j2k.m_private_image.is_null() {
        i = 0 as OPJ_UINT32;
        while i < l_nb_tiles {
          if !l_tcp.is_null() {
            opj_j2k_dump_tile_info(
              &*l_tcp,
              (*p_j2k.m_private_image).numcomps as OPJ_INT32,
              out_stream,
            );
          }
          l_tcp = l_tcp.offset(1);
          i += 1;
        }
      }
    }
    /* Dump the codestream info of the current tile */
    if flag & 4i32 != 0 {};
    /* Dump the codestream index from main header */
    if flag & 16i32 != 0 {
      opj_j2k_dump_MH_index(p_j2k, out_stream);
    }
    /* Dump the codestream index of the current tile */
    if flag & 32i32 != 0 {}
  }
}

#[cfg(feature = "file-io")]
fn opj_j2k_dump_MH_index(p_j2k: &opj_j2k, out_stream: *mut FILE) {
  unsafe {
    let mut cstr_index = p_j2k.cstr_index;
    let mut it_marker: OPJ_UINT32 = 0;
    let mut it_tile: OPJ_UINT32 = 0;
    let mut it_tile_part: OPJ_UINT32 = 0;
    fprintf!(out_stream, "Codestream index from main header: {{\n");
    fprintf!(
      out_stream,
      "\t Main header start position={}\n\t Main header end position={}\n",
      (*cstr_index).main_head_start,
      (*cstr_index).main_head_end,
    );
    fprintf!(out_stream, "\t Marker list: {{\n");
    if !(*cstr_index).marker.is_null() {
      it_marker = 0 as OPJ_UINT32;
      while it_marker < (*cstr_index).marknum {
        let marker = *(*cstr_index).marker.offset(it_marker as isize);
        let ty = marker.type_ as i32;
        if ty != 0 {
          fprintf!(
            out_stream,
            "\t\t type={:#x}, pos={}, len={}\n",
            ty,
            marker.pos,
            marker.len,
          );
        } else {
          fprintf!(
            out_stream,
            "\t\t type={:x}, pos={}, len={}\n",
            ty,
            marker.pos,
            marker.len,
          );
        }
        it_marker += 1;
      }
    }
    fprintf!(out_stream, "\t }}\n");
    if !(*cstr_index).tile_index.is_null() {
      /* Simple test to avoid to write empty information*/
      let mut l_acc_nb_of_tile_part = 0 as OPJ_UINT32; /* Not fill from the main header*/
      it_tile = 0 as OPJ_UINT32;
      while it_tile < (*cstr_index).nb_of_tiles {
        l_acc_nb_of_tile_part = (l_acc_nb_of_tile_part as core::ffi::c_uint)
          .wrapping_add((*(*cstr_index).tile_index.offset(it_tile as isize)).nb_tps)
          as OPJ_UINT32;
        it_tile += 1;
      }
      if l_acc_nb_of_tile_part != 0 {
        fprintf!(out_stream, "\t Tile index: {{\n");
        it_tile = 0 as OPJ_UINT32;
        while it_tile < (*cstr_index).nb_of_tiles {
          let mut nb_of_tile_part = (*(*cstr_index).tile_index.offset(it_tile as isize)).nb_tps;
          fprintf!(
            out_stream,
            "\t\t nb of tile-part in tile [{}]={}\n",
            it_tile,
            nb_of_tile_part,
          );
          if !(*(*cstr_index).tile_index.offset(it_tile as isize))
            .tp_index
            .is_null()
          {
            it_tile_part = 0 as OPJ_UINT32;
            while it_tile_part < nb_of_tile_part {
              fprintf!(
                out_stream,
                "\t\t\t tile-part[{}]: star_pos={}, end_header={}, end_pos={}.\n",
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
              it_tile_part += 1;
            }
          }
          if !(*(*cstr_index).tile_index.offset(it_tile as isize))
            .marker
            .is_null()
          {
            it_marker = 0 as OPJ_UINT32;
            while it_marker < (*(*cstr_index).tile_index.offset(it_tile as isize)).marknum {
              fprintf!(
                out_stream,
                "\t\t type={:#x}, pos={}, len={}\n",
                (*(*(*cstr_index).tile_index.offset(it_tile as isize))
                  .marker
                  .offset(it_marker as isize))
                .type_ as core::ffi::c_int,
                (*(*(*cstr_index).tile_index.offset(it_tile as isize))
                  .marker
                  .offset(it_marker as isize))
                .pos,
                (*(*(*cstr_index).tile_index.offset(it_tile as isize))
                  .marker
                  .offset(it_marker as isize))
                .len,
              );
              it_marker += 1;
            }
          }
          it_tile += 1;
        }
        fprintf!(out_stream, "\t }}\n");
      }
    }
    fprintf!(out_stream, "}}\n");
  }
}

#[cfg(feature = "file-io")]
fn opj_j2k_dump_MH_info(p_j2k: &opj_j2k, out_stream: *mut FILE) {
  unsafe {
    fprintf!(out_stream, "Codestream info from main header: {{\n");
    fprintf!(
      out_stream,
      "\t tx0={}, ty0={}\n",
      p_j2k.m_cp.tx0,
      p_j2k.m_cp.ty0,
    );
    fprintf!(
      out_stream,
      "\t tdx={}, tdy={}\n",
      p_j2k.m_cp.tdx,
      p_j2k.m_cp.tdy,
    );
    fprintf!(
      out_stream,
      "\t tw={}, th={}\n",
      p_j2k.m_cp.tw,
      p_j2k.m_cp.th,
    );
    if !p_j2k.m_private_image.is_null() {
      opj_j2k_dump_tile_info(
        &*p_j2k.m_specific_param.m_decoder.m_default_tcp,
        (*p_j2k.m_private_image).numcomps as OPJ_INT32,
        out_stream,
      );
    }
    fprintf!(out_stream, "}}\n");
  }
}

#[cfg(feature = "file-io")]

pub(crate) fn j2k_dump_image_header(
  img_header: &opj_image,
  dev_dump_flag: OPJ_BOOL,
  out_stream: *mut FILE,
) {
  unsafe {
    let mut tab = "";
    if dev_dump_flag != 0 {
      fprintf!(out_stream, "[DEV] Dump an image_header struct {{\n");
    } else {
      fprintf!(out_stream, "Image info {{\n");
      tab = "\t";
    }
    fprintf!(
      out_stream,
      "{} x0={}, y0={}\n",
      tab,
      (*img_header).x0,
      (*img_header).y0,
    );
    fprintf!(
      out_stream,
      "{} x1={}, y1={}\n",
      tab,
      (*img_header).x1,
      (*img_header).y1,
    );
    fprintf!(out_stream, "{} numcomps={}\n", tab, (*img_header).numcomps);
    if !(*img_header).comps.is_null() {
      let mut compno: OPJ_UINT32 = 0;
      compno = 0 as OPJ_UINT32;
      while compno < (*img_header).numcomps {
        fprintf!(out_stream, "{}\t component {} {{\n", tab, compno);
        j2k_dump_image_comp_header(
          &mut *(*img_header).comps.offset(compno as isize),
          dev_dump_flag,
          out_stream,
        );
        fprintf!(out_stream, "{}}}\n", tab);
        compno += 1;
      }
    }
    fprintf!(out_stream, "}}\n");
  }
}

#[cfg(feature = "file-io")]

pub(crate) fn j2k_dump_image_comp_header(
  comp_header: &opj_image_comp_t,
  dev_dump_flag: OPJ_BOOL,
  out_stream: *mut FILE,
) {
  unsafe {
    let mut tab = "";
    if dev_dump_flag != 0 {
      fprintf!(out_stream, "[DEV] Dump an image_comp_header struct {{\n");
    } else {
      tab = "\t\t";
    }
    fprintf!(
      out_stream,
      "{} dx={}, dy={}\n",
      tab,
      (*comp_header).dx,
      (*comp_header).dy,
    );
    fprintf!(out_stream, "{} prec={}\n", tab, (*comp_header).prec);
    fprintf!(out_stream, "{} sgnd={}\n", tab, (*comp_header).sgnd);
    if dev_dump_flag != 0 {
      fprintf!(out_stream, "}}\n");
    };
  }
}
