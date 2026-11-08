use crate::fprintf::StringWriter;
use crate::image::*;
use crate::openjpeg::*;

fn opj_j2k_dump_tile_info<W: StringWriter>(
  l_default_tile: &Tcp,
  numcomps: i32,
  out_stream: &mut W,
) {
  let mut compno: i32 = 0;
  fprintf!(out_stream, "\t default tile {{\n");
  if l_default_tile.csty != 0 {
    fprintf!(out_stream, "\t\t csty={:#x}\n", l_default_tile.csty);
  } else {
    fprintf!(out_stream, "\t\t csty=0\n");
  }
  if l_default_tile.prg != 0 {
    fprintf!(
      out_stream,
      "\t\t prg={:#x}\n",
      l_default_tile.prg as core::ffi::c_int,
    );
  } else {
    fprintf!(out_stream, "\t\t prg=0\n");
  }
  fprintf!(out_stream, "\t\t numlayers={}\n", l_default_tile.numlayers,);
  fprintf!(out_stream, "\t\t mct={:x}\n", l_default_tile.mct);
  /*end of default tile*/
  compno = 0i32; /*end of component of default tile*/
  while compno < numcomps {
    let l_tccp = &l_default_tile.tccps[compno as usize];
    let mut resno: u32 = 0;
    let mut bandno: i32 = 0;
    let mut numbands: i32 = 0;
    /* coding style*/
    fprintf!(out_stream, "\t\t comp {} {{\n", compno);
    if l_tccp.csty != 0 {
      fprintf!(out_stream, "\t\t\t csty={:#x}\n", l_tccp.csty);
    } else {
      fprintf!(out_stream, "\t\t\t csty=0\n");
    }
    fprintf!(
      out_stream,
      "\t\t\t numresolutions={}\n",
      l_tccp.numresolutions,
    );
    fprintf!(out_stream, "\t\t\t cblkw=2^{}\n", l_tccp.cblkw);
    fprintf!(out_stream, "\t\t\t cblkh=2^{}\n", l_tccp.cblkh);
    if l_tccp.cblksty != 0 {
      fprintf!(out_stream, "\t\t\t cblksty={:#x}\n", l_tccp.cblksty);
    } else {
      fprintf!(out_stream, "\t\t\t cblksty=0\n");
    }
    fprintf!(out_stream, "\t\t\t qmfbid={}\n", l_tccp.qmfbid);
    fprintf!(out_stream, "\t\t\t preccintsize (w,h)=");
    resno = 0_u32;
    while resno < l_tccp.numresolutions {
      fprintf!(
        out_stream,
        "({},{}) ",
        l_tccp.prcw[resno as usize],
        l_tccp.prch[resno as usize],
      );
      resno += 1;
    }
    fprintf!(out_stream, "\n");
    /* quantization style*/
    fprintf!(out_stream, "\t\t\t qntsty={}\n", l_tccp.qntsty);
    fprintf!(out_stream, "\t\t\t numgbits={}\n", l_tccp.numgbits);
    fprintf!(out_stream, "\t\t\t stepsizes (m,e)=");
    numbands = if l_tccp.qntsty == 1u32 {
      1i32
    } else {
      (l_tccp.numresolutions as i32 * 3i32) - 2i32
    };
    bandno = 0i32;
    while bandno < numbands {
      fprintf!(
        out_stream,
        "({},{}) ",
        l_tccp.stepsizes[bandno as usize].mant,
        l_tccp.stepsizes[bandno as usize].expn,
      );
      bandno += 1
    }
    fprintf!(out_stream, "\n");
    /* RGN value*/
    fprintf!(out_stream, "\t\t\t roishift={}\n", l_tccp.roishift);
    fprintf!(out_stream, "\t\t }}\n");
    compno += 1
  }
  fprintf!(out_stream, "\t }}\n");
}

pub(crate) fn j2k_dump<W: StringWriter>(p_j2k: &J2k, flag: i32, out_stream: &mut W) {
  /* Check if the flag is compatible with j2k file*/
  if flag & 128i32 != 0 || flag & 256i32 != 0 {
    fprintf!(out_stream, "Wrong flag\n");
    return;
  }
  /* Dump the image_header */
  if flag & 1i32 != 0 && p_j2k.m_private_image.is_some() {
    j2k_dump_image_header(
      p_j2k.m_private_image.as_deref().expect("m_private_image"),
      0i32,
      out_stream,
    );
  }
  /* Dump the codestream info from main header */
  if flag & 2i32 != 0 && p_j2k.m_private_image.is_some() {
    opj_j2k_dump_MH_info(p_j2k, out_stream);
  }
  /* Dump all tile/codestream info */
  if flag & 8i32 != 0 {
    let l_nb_tiles = p_j2k.m_cp.th.wrapping_mul(p_j2k.m_cp.tw);
    let mut i: u32 = 0;
    if p_j2k.m_private_image.is_some() {
      i = 0_u32;
      while i < l_nb_tiles {
        opj_j2k_dump_tile_info(
          &p_j2k.m_cp.tcps[i as usize],
          p_j2k
            .m_private_image
            .as_deref()
            .expect("m_private_image")
            .numcomps as i32,
          out_stream,
        );
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

fn opj_j2k_dump_MH_index<W: StringWriter>(p_j2k: &J2k, out_stream: &mut W) {
  let Some(cstr_index) = p_j2k.cstr_index.as_deref() else {
    return;
  };
  fprintf!(out_stream, "Codestream index from main header: {{\n");
  fprintf!(
    out_stream,
    "\t Main header start position={}\n\t Main header end position={}\n",
    cstr_index.main_head_start,
    cstr_index.main_head_end,
  );
  fprintf!(out_stream, "\t Marker list: {{\n");
  for marker in &cstr_index.markers {
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
  }
  fprintf!(out_stream, "\t }}\n");
  /* Simple test to avoid writing empty information */
  let l_acc_nb_of_tile_part: u32 = cstr_index.tiles.iter().map(|tile| tile.nb_tps).sum();
  if l_acc_nb_of_tile_part != 0 {
    fprintf!(out_stream, "\t Tile index: {{\n");
    for (it_tile, tile) in cstr_index.tiles.iter().enumerate() {
      let nb_of_tile_part = tile.nb_tps;
      fprintf!(
        out_stream,
        "\t\t nb of tile-part in tile [{}]={}\n",
        it_tile,
        nb_of_tile_part,
      );
      for (it_tile_part, tp) in tile
        .tp_index
        .iter()
        .enumerate()
        .take(nb_of_tile_part as usize)
      {
        fprintf!(
          out_stream,
          "\t\t\t tile-part[{}]: star_pos={}, end_header={}, end_pos={}.\n",
          it_tile_part,
          tp.start_pos,
          tp.end_header,
          tp.end_pos,
        );
      }
      for marker in &tile.markers {
        fprintf!(
          out_stream,
          "\t\t type={:#x}, pos={}, len={}\n",
          marker.type_ as core::ffi::c_int,
          marker.pos,
          marker.len,
        );
      }
    }
  }
  fprintf!(out_stream, "}}\n");
}

fn opj_j2k_dump_MH_info<W: StringWriter>(p_j2k: &J2k, out_stream: &mut W) {
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
  if p_j2k.m_private_image.is_some() {
    opj_j2k_dump_tile_info(
      p_j2k
        .m_specific_param
        .m_decoder
        .m_default_tcp
        .as_deref()
        .expect("decoder m_default_tcp is always set"),
      p_j2k
        .m_private_image
        .as_deref()
        .expect("m_private_image")
        .numcomps as i32,
      out_stream,
    );
  }
  fprintf!(out_stream, "}}\n");
}

pub(crate) fn j2k_dump_image_header<W: StringWriter>(
  img_header: &CoreImage,
  dev_dump_flag: OPJ_BOOL,
  out_stream: &mut W,
) {
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
    img_header.x0,
    img_header.y0,
  );
  fprintf!(
    out_stream,
    "{} x1={}, y1={}\n",
    tab,
    img_header.x1,
    img_header.y1,
  );
  fprintf!(out_stream, "{} numcomps={}\n", tab, img_header.numcomps);
  if !img_header.comps.is_empty() {
    let mut compno: u32 = 0;
    compno = 0_u32;
    while compno < img_header.numcomps {
      fprintf!(out_stream, "{}\t component {} {{\n", tab, compno);
      j2k_dump_image_comp_header(
        &img_header.comps[compno as usize],
        dev_dump_flag,
        out_stream,
      );
      fprintf!(out_stream, "{}}}\n", tab);
      compno += 1;
    }
    fprintf!(out_stream, "}}\n");
  }
}

pub(crate) fn j2k_dump_image_comp_header<W: StringWriter>(
  comp_header: &CoreImageComp,
  dev_dump_flag: OPJ_BOOL,
  out_stream: &mut W,
) {
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
    comp_header.dx,
    comp_header.dy,
  );
  fprintf!(out_stream, "{} prec={}\n", tab, comp_header.prec);
  fprintf!(out_stream, "{} sgnd={}\n", tab, comp_header.sgnd);
  if dev_dump_flag != 0 {
    fprintf!(out_stream, "}}\n");
  };
}
