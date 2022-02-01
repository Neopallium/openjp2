use ::libc;
extern "C" {
    #[no_mangle]
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong)
     -> *mut libc::c_void;
    #[no_mangle]
    fn opj_calloc(numOfElements: size_t, sizeOfElements: size_t)
     -> *mut libc::c_void;
    #[no_mangle]
    fn opj_realloc(m: *mut libc::c_void, s: size_t) -> *mut libc::c_void;
    #[no_mangle]
    fn opj_free(m: *mut libc::c_void);
    #[no_mangle]
    fn opj_event_msg(event_mgr: *mut opj_event_mgr_t, event_type: OPJ_INT32,
                     fmt: *const libc::c_char, _: ...) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_bio_write(bio: *mut opj_bio_t, v: OPJ_UINT32, n: OPJ_UINT32);
    #[no_mangle]
    fn opj_bio_read(bio: *mut opj_bio_t, n: OPJ_UINT32) -> OPJ_UINT32;
}
pub type size_t = libc::c_ulong;
pub type __int32_t = libc::c_int;
pub type __uint32_t = libc::c_uint;
pub type OPJ_BOOL = libc::c_int;
pub type OPJ_BYTE = libc::c_uchar;
pub type int32_t = __int32_t;
pub type uint32_t = __uint32_t;
pub type OPJ_INT32 = int32_t;
pub type OPJ_UINT32 = uint32_t;
pub type opj_msg_callback
    =
    Option<unsafe extern "C" fn(_: *const libc::c_char, _: *mut libc::c_void)
               -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_event_mgr {
    pub m_error_data: *mut libc::c_void,
    pub m_warning_data: *mut libc::c_void,
    pub m_info_data: *mut libc::c_void,
    pub error_handler: opj_msg_callback,
    pub warning_handler: opj_msg_callback,
    pub info_handler: opj_msg_callback,
}
pub type opj_event_mgr_t = opj_event_mgr;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_bio {
    pub start: *mut OPJ_BYTE,
    pub end: *mut OPJ_BYTE,
    pub bp: *mut OPJ_BYTE,
    pub buf: OPJ_UINT32,
    pub ct: OPJ_UINT32,
}
pub type opj_bio_t = opj_bio;
pub type opj_tgt_tree_t = opj_tgt_tree;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tgt_tree {
    pub numleafsh: OPJ_UINT32,
    pub numleafsv: OPJ_UINT32,
    pub numnodes: OPJ_UINT32,
    pub nodes: *mut opj_tgt_node_t,
    pub nodes_size: OPJ_UINT32,
}
pub type opj_tgt_node_t = opj_tgt_node;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tgt_node {
    pub parent: *mut opj_tgt_node,
    pub value: OPJ_INT32,
    pub low: OPJ_INT32,
    pub known: OPJ_UINT32,
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
 * Copyright (c) 2008, 2011-2012, Centre National d'Etudes Spatiales (CNES), FR
 * Copyright (c) 2012, CS Systemes d'Information, France
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
/*
==========================================================
   Tag-tree coder interface
==========================================================
*/
#[no_mangle]
pub unsafe extern "C" fn opj_tgt_create(mut numleafsh: OPJ_UINT32,
                                        mut numleafsv: OPJ_UINT32,
                                        mut p_manager: *mut opj_event_mgr_t)
 -> *mut opj_tgt_tree_t {
    let mut nplh: [OPJ_INT32; 32] = [0; 32];
    let mut nplv: [OPJ_INT32; 32] = [0; 32];
    let mut node = 0 as *mut opj_tgt_node_t;
    let mut l_parent_node = 0 as *mut opj_tgt_node_t;
    let mut l_parent_node0 = 0 as *mut opj_tgt_node_t;
    let mut tree = 0 as *mut opj_tgt_tree_t;
    let mut i: OPJ_UINT32 = 0;
    let mut j: OPJ_INT32 = 0;
    let mut k: OPJ_INT32 = 0;
    let mut numlvls: OPJ_UINT32 = 0;
    let mut n: OPJ_UINT32 = 0;
    tree =
        opj_calloc(1 as libc::c_int as size_t,
                   ::std::mem::size_of::<opj_tgt_tree_t>() as libc::c_ulong)
            as *mut opj_tgt_tree_t;
    if tree.is_null() {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Not enough memory to create Tag-tree\n\x00" as
                          *const u8 as *const libc::c_char);
        return 0 as *mut opj_tgt_tree_t
    }
    (*tree).numleafsh = numleafsh;
    (*tree).numleafsv = numleafsv;
    numlvls = 0 as libc::c_int as OPJ_UINT32;
    nplh[0 as libc::c_int as usize] = numleafsh as OPJ_INT32;
    nplv[0 as libc::c_int as usize] = numleafsv as OPJ_INT32;
    (*tree).numnodes = 0 as libc::c_int as OPJ_UINT32;
    loop  {
        n = (nplh[numlvls as usize] * nplv[numlvls as usize]) as OPJ_UINT32;
        nplh[numlvls.wrapping_add(1 as libc::c_int as libc::c_uint) as usize]
            = (nplh[numlvls as usize] + 1 as libc::c_int) / 2 as libc::c_int;
        nplv[numlvls.wrapping_add(1 as libc::c_int as libc::c_uint) as usize]
            = (nplv[numlvls as usize] + 1 as libc::c_int) / 2 as libc::c_int;
        (*tree).numnodes =
            ((*tree).numnodes as libc::c_uint).wrapping_add(n) as OPJ_UINT32
                as OPJ_UINT32;
        numlvls = numlvls.wrapping_add(1);
        if !(n > 1 as libc::c_int as libc::c_uint) { break ; }
    }
    /* ADD */
    if (*tree).numnodes == 0 as libc::c_int as libc::c_uint {
        opj_free(tree as *mut libc::c_void);
        return 0 as *mut opj_tgt_tree_t
    }
    (*tree).nodes =
        opj_calloc((*tree).numnodes as size_t,
                   ::std::mem::size_of::<opj_tgt_node_t>() as libc::c_ulong)
            as *mut opj_tgt_node_t;
    if (*tree).nodes.is_null() {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Not enough memory to create Tag-tree nodes\n\x00" as
                          *const u8 as *const libc::c_char);
        opj_free(tree as *mut libc::c_void);
        return 0 as *mut opj_tgt_tree_t
    }
    (*tree).nodes_size =
        (*tree).numnodes.wrapping_mul(::std::mem::size_of::<opj_tgt_node_t>()
                                          as libc::c_ulong as OPJ_UINT32);
    node = (*tree).nodes;
    l_parent_node =
        &mut *(*tree).nodes.offset((*tree).numleafsh.wrapping_mul((*tree).numleafsv)
                                       as isize) as *mut opj_tgt_node_t;
    l_parent_node0 = l_parent_node;
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < numlvls.wrapping_sub(1 as libc::c_int as libc::c_uint) {
        j = 0 as libc::c_int;
        while j < nplv[i as usize] {
            k = nplh[i as usize];
            loop  {
                k -= 1;
                if !(k >= 0 as libc::c_int) { break ; }
                (*node).parent = l_parent_node;
                node = node.offset(1);
                k -= 1;
                if k >= 0 as libc::c_int {
                    (*node).parent = l_parent_node;
                    node = node.offset(1)
                }
                l_parent_node = l_parent_node.offset(1)
            }
            if j & 1 as libc::c_int != 0 ||
                   j == nplv[i as usize] - 1 as libc::c_int {
                l_parent_node0 = l_parent_node
            } else {
                l_parent_node = l_parent_node0;
                l_parent_node0 =
                    l_parent_node0.offset(nplh[i as usize] as isize)
            }
            j += 1
        }
        i = i.wrapping_add(1)
    }
    (*node).parent = 0 as *mut opj_tgt_node;
    opj_tgt_reset(tree);
    return tree;
}
/* *
 * Reinitialises a tag-tree from an existing one.
 *
 * @param       p_tree                          the tree to reinitialize.
 * @param       p_num_leafs_h           the width of the array of leafs of the tree
 * @param       p_num_leafs_v           the height of the array of leafs of the tree
 * @return      a new tag-tree if successful, NULL otherwise
*/
#[no_mangle]
pub unsafe extern "C" fn opj_tgt_init(mut p_tree: *mut opj_tgt_tree_t,
                                      mut p_num_leafs_h: OPJ_UINT32,
                                      mut p_num_leafs_v: OPJ_UINT32,
                                      mut p_manager: *mut opj_event_mgr_t)
 -> *mut opj_tgt_tree_t {
    let mut l_nplh: [OPJ_INT32; 32] = [0; 32];
    let mut l_nplv: [OPJ_INT32; 32] = [0; 32];
    let mut l_node = 0 as *mut opj_tgt_node_t;
    let mut l_parent_node = 0 as *mut opj_tgt_node_t;
    let mut l_parent_node0 = 0 as *mut opj_tgt_node_t;
    let mut i: OPJ_UINT32 = 0;
    let mut j: OPJ_INT32 = 0;
    let mut k: OPJ_INT32 = 0;
    let mut l_num_levels: OPJ_UINT32 = 0;
    let mut n: OPJ_UINT32 = 0;
    let mut l_node_size: OPJ_UINT32 = 0;
    if p_tree.is_null() { return 0 as *mut opj_tgt_tree_t }
    if (*p_tree).numleafsh != p_num_leafs_h ||
           (*p_tree).numleafsv != p_num_leafs_v {
        (*p_tree).numleafsh = p_num_leafs_h;
        (*p_tree).numleafsv = p_num_leafs_v;
        l_num_levels = 0 as libc::c_int as OPJ_UINT32;
        l_nplh[0 as libc::c_int as usize] = p_num_leafs_h as OPJ_INT32;
        l_nplv[0 as libc::c_int as usize] = p_num_leafs_v as OPJ_INT32;
        (*p_tree).numnodes = 0 as libc::c_int as OPJ_UINT32;
        loop  {
            n =
                (l_nplh[l_num_levels as usize] *
                     l_nplv[l_num_levels as usize]) as OPJ_UINT32;
            l_nplh[l_num_levels.wrapping_add(1 as libc::c_int as libc::c_uint)
                       as usize] =
                (l_nplh[l_num_levels as usize] + 1 as libc::c_int) /
                    2 as libc::c_int;
            l_nplv[l_num_levels.wrapping_add(1 as libc::c_int as libc::c_uint)
                       as usize] =
                (l_nplv[l_num_levels as usize] + 1 as libc::c_int) /
                    2 as libc::c_int;
            (*p_tree).numnodes =
                ((*p_tree).numnodes as libc::c_uint).wrapping_add(n) as
                    OPJ_UINT32 as OPJ_UINT32;
            l_num_levels = l_num_levels.wrapping_add(1);
            if !(n > 1 as libc::c_int as libc::c_uint) { break ; }
        }
        /* ADD */
        if (*p_tree).numnodes == 0 as libc::c_int as libc::c_uint {
            opj_tgt_destroy(p_tree);
            return 0 as *mut opj_tgt_tree_t
        }
        l_node_size =
            (*p_tree).numnodes.wrapping_mul(::std::mem::size_of::<opj_tgt_node_t>()
                                                as libc::c_ulong as
                                                OPJ_UINT32);
        if l_node_size > (*p_tree).nodes_size {
            let mut new_nodes =
                opj_realloc((*p_tree).nodes as *mut libc::c_void,
                            l_node_size as size_t) as *mut opj_tgt_node_t;
            if new_nodes.is_null() {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Not enough memory to reinitialize the tag tree\n\x00"
                                  as *const u8 as *const libc::c_char);
                opj_tgt_destroy(p_tree);
                return 0 as *mut opj_tgt_tree_t
            }
            (*p_tree).nodes = new_nodes;
            memset(((*p_tree).nodes as
                        *mut libc::c_char).offset((*p_tree).nodes_size as
                                                      isize) as
                       *mut libc::c_void, 0 as libc::c_int,
                   l_node_size.wrapping_sub((*p_tree).nodes_size) as
                       libc::c_ulong);
            (*p_tree).nodes_size = l_node_size
        }
        l_node = (*p_tree).nodes;
        l_parent_node =
            &mut *(*p_tree).nodes.offset((*p_tree).numleafsh.wrapping_mul((*p_tree).numleafsv)
                                             as isize) as *mut opj_tgt_node_t;
        l_parent_node0 = l_parent_node;
        i = 0 as libc::c_int as OPJ_UINT32;
        while i < l_num_levels.wrapping_sub(1 as libc::c_int as libc::c_uint)
              {
            j = 0 as libc::c_int;
            while j < l_nplv[i as usize] {
                k = l_nplh[i as usize];
                loop  {
                    k -= 1;
                    if !(k >= 0 as libc::c_int) { break ; }
                    (*l_node).parent = l_parent_node;
                    l_node = l_node.offset(1);
                    k -= 1;
                    if k >= 0 as libc::c_int {
                        (*l_node).parent = l_parent_node;
                        l_node = l_node.offset(1)
                    }
                    l_parent_node = l_parent_node.offset(1)
                }
                if j & 1 as libc::c_int != 0 ||
                       j == l_nplv[i as usize] - 1 as libc::c_int {
                    l_parent_node0 = l_parent_node
                } else {
                    l_parent_node = l_parent_node0;
                    l_parent_node0 =
                        l_parent_node0.offset(l_nplh[i as usize] as isize)
                }
                j += 1
            }
            i = i.wrapping_add(1)
        }
        (*l_node).parent = 0 as *mut opj_tgt_node
    }
    opj_tgt_reset(p_tree);
    return p_tree;
}
#[no_mangle]
pub unsafe extern "C" fn opj_tgt_destroy(mut p_tree: *mut opj_tgt_tree_t) {
    if p_tree.is_null() { return }
    if !(*p_tree).nodes.is_null() {
        opj_free((*p_tree).nodes as *mut libc::c_void);
        (*p_tree).nodes = 0 as *mut opj_tgt_node_t
    }
    opj_free(p_tree as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn opj_tgt_reset(mut p_tree: *mut opj_tgt_tree_t) {
    let mut i: OPJ_UINT32 = 0;
    let mut l_current_node = 0 as *mut opj_tgt_node_t;
    if p_tree.is_null() { return }
    l_current_node = (*p_tree).nodes;
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < (*p_tree).numnodes {
        (*l_current_node).value = 999 as libc::c_int;
        (*l_current_node).low = 0 as libc::c_int;
        (*l_current_node).known = 0 as libc::c_int as OPJ_UINT32;
        l_current_node = l_current_node.offset(1);
        i = i.wrapping_add(1)
    };
}
#[no_mangle]
pub unsafe extern "C" fn opj_tgt_setvalue(mut tree: *mut opj_tgt_tree_t,
                                          mut leafno: OPJ_UINT32,
                                          mut value: OPJ_INT32) {
    let mut node = 0 as *mut opj_tgt_node_t;
    node = &mut *(*tree).nodes.offset(leafno as isize) as *mut opj_tgt_node_t;
    while !node.is_null() && (*node).value > value {
        (*node).value = value;
        node = (*node).parent
    };
}
#[no_mangle]
pub unsafe extern "C" fn opj_tgt_encode(mut bio: *mut opj_bio_t,
                                        mut tree: *mut opj_tgt_tree_t,
                                        mut leafno: OPJ_UINT32,
                                        mut threshold: OPJ_INT32) {
    let mut stk: [*mut opj_tgt_node_t; 31] = [0 as *mut opj_tgt_node_t; 31];
    let mut stkptr = 0 as *mut *mut opj_tgt_node_t;
    let mut node = 0 as *mut opj_tgt_node_t;
    let mut low: OPJ_INT32 = 0;
    stkptr = stk.as_mut_ptr();
    node = &mut *(*tree).nodes.offset(leafno as isize) as *mut opj_tgt_node_t;
    while !(*node).parent.is_null() {
        let fresh0 = stkptr;
        stkptr = stkptr.offset(1);
        *fresh0 = node;
        node = (*node).parent
    }
    low = 0 as libc::c_int;
    loop  {
        if low > (*node).low { (*node).low = low } else { low = (*node).low }
        while low < threshold {
            if low >= (*node).value {
                if (*node).known == 0 {
                    opj_bio_write(bio, 1 as libc::c_int as OPJ_UINT32,
                                  1 as libc::c_int as OPJ_UINT32);
                    (*node).known = 1 as libc::c_int as OPJ_UINT32
                }
                break ;
            } else {
                opj_bio_write(bio, 0 as libc::c_int as OPJ_UINT32,
                              1 as libc::c_int as OPJ_UINT32);
                low += 1
            }
        }
        (*node).low = low;
        if stkptr == stk.as_mut_ptr() { break ; }
        stkptr = stkptr.offset(-1);
        node = *stkptr
    };
}
#[no_mangle]
pub unsafe extern "C" fn opj_tgt_decode(mut bio: *mut opj_bio_t,
                                        mut tree: *mut opj_tgt_tree_t,
                                        mut leafno: OPJ_UINT32,
                                        mut threshold: OPJ_INT32)
 -> OPJ_UINT32 {
    let mut stk: [*mut opj_tgt_node_t; 31] = [0 as *mut opj_tgt_node_t; 31];
    let mut stkptr = 0 as *mut *mut opj_tgt_node_t;
    let mut node = 0 as *mut opj_tgt_node_t;
    let mut low: OPJ_INT32 = 0;
    stkptr = stk.as_mut_ptr();
    node = &mut *(*tree).nodes.offset(leafno as isize) as *mut opj_tgt_node_t;
    while !(*node).parent.is_null() {
        let fresh1 = stkptr;
        stkptr = stkptr.offset(1);
        *fresh1 = node;
        node = (*node).parent
    }
    low = 0 as libc::c_int;
    loop  {
        if low > (*node).low { (*node).low = low } else { low = (*node).low }
        while low < threshold && low < (*node).value {
            if opj_bio_read(bio, 1 as libc::c_int as OPJ_UINT32) != 0 {
                (*node).value = low
            } else { low += 1 }
        }
        (*node).low = low;
        if stkptr == stk.as_mut_ptr() { break ; }
        stkptr = stkptr.offset(-1);
        node = *stkptr
    }
    return if (*node).value < threshold {
               1 as libc::c_int
           } else { 0 as libc::c_int } as OPJ_UINT32;
}
