use ::libc;
extern "C" {
    #[no_mangle]
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong)
     -> *mut libc::c_void;
    #[no_mangle]
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong)
     -> *mut libc::c_void;
    #[no_mangle]
    fn __assert_fail(__assertion: *const libc::c_char,
                     __file: *const libc::c_char, __line: libc::c_uint,
                     __function: *const libc::c_char) -> !;
    #[no_mangle]
    fn opj_calloc(numOfElements: size_t, sizeOfElements: size_t)
     -> *mut libc::c_void;
    #[no_mangle]
    fn opj_free(m: *mut libc::c_void);
}
pub type size_t = libc::c_ulong;
pub type __int32_t = libc::c_int;
pub type __uint32_t = libc::c_uint;
pub type __uint64_t = libc::c_ulong;
pub type OPJ_BOOL = libc::c_int;
pub type int32_t = __int32_t;
pub type uint32_t = __uint32_t;
pub type uint64_t = __uint64_t;
pub type OPJ_INT32 = int32_t;
pub type OPJ_UINT32 = uint32_t;
pub type OPJ_UINT64 = uint64_t;
pub type OPJ_SIZE_T = size_t;
/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2017, IntoPix SA <contact@intopix.com>
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_sparse_array_int32 {
    pub width: OPJ_UINT32,
    pub height: OPJ_UINT32,
    pub block_width: OPJ_UINT32,
    pub block_height: OPJ_UINT32,
    pub block_count_hor: OPJ_UINT32,
    pub block_count_ver: OPJ_UINT32,
    pub data_blocks: *mut *mut OPJ_INT32,
}
pub type opj_sparse_array_int32_t = opj_sparse_array_int32;
#[inline]
unsafe extern "C" fn opj_uint_ceildiv(mut a: OPJ_UINT32, mut b: OPJ_UINT32)
 -> OPJ_UINT32 {
    if b != 0 {
    } else {
        __assert_fail(b"b\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/opj_intmath.h\x00" as
                          *const u8 as *const libc::c_char,
                      172 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 52],
                                                &[libc::c_char; 52]>(b"OPJ_UINT32 opj_uint_ceildiv(OPJ_UINT32, OPJ_UINT32)\x00")).as_ptr());
    }
    return (a as
                OPJ_UINT64).wrapping_add(b as
                                             libc::c_ulong).wrapping_sub(1 as
                                                                             libc::c_int
                                                                             as
                                                                             libc::c_ulong).wrapping_div(b
                                                                                                             as
                                                                                                             libc::c_ulong)
               as OPJ_UINT32;
}
#[inline]
unsafe extern "C" fn opj_uint_min(mut a: OPJ_UINT32, mut b: OPJ_UINT32)
 -> OPJ_UINT32 {
    return if a < b { a } else { b };
}
#[no_mangle]
pub unsafe extern "C" fn opj_sparse_array_int32_create(mut width: OPJ_UINT32,
                                                       mut height: OPJ_UINT32,
                                                       mut block_width:
                                                           OPJ_UINT32,
                                                       mut block_height:
                                                           OPJ_UINT32)
 -> *mut opj_sparse_array_int32_t {
    let mut sa = 0 as *mut opj_sparse_array_int32_t;
    if width == 0 as libc::c_int as libc::c_uint ||
           height == 0 as libc::c_int as libc::c_uint ||
           block_width == 0 as libc::c_int as libc::c_uint ||
           block_height == 0 as libc::c_int as libc::c_uint {
        return 0 as *mut opj_sparse_array_int32_t
    }
    if block_width as libc::c_ulong >
           ((!(0 as libc::c_uint)).wrapping_div(block_height) as
                libc::c_ulong).wrapping_div(::std::mem::size_of::<OPJ_INT32>()
                                                as libc::c_ulong) {
        return 0 as *mut opj_sparse_array_int32_t
    }
    sa =
        opj_calloc(1 as libc::c_int as size_t,
                   ::std::mem::size_of::<opj_sparse_array_int32_t>() as
                       libc::c_ulong) as *mut opj_sparse_array_int32_t;
    (*sa).width = width;
    (*sa).height = height;
    (*sa).block_width = block_width;
    (*sa).block_height = block_height;
    (*sa).block_count_hor = opj_uint_ceildiv(width, block_width);
    (*sa).block_count_ver = opj_uint_ceildiv(height, block_height);
    if (*sa).block_count_hor >
           (!(0 as libc::c_uint)).wrapping_div((*sa).block_count_ver) {
        opj_free(sa as *mut libc::c_void);
        return 0 as *mut opj_sparse_array_int32_t
    }
    (*sa).data_blocks =
        opj_calloc(::std::mem::size_of::<*mut OPJ_INT32>() as libc::c_ulong,
                   ((*sa).block_count_hor as
                        size_t).wrapping_mul((*sa).block_count_ver as
                                                 libc::c_ulong)) as
            *mut *mut OPJ_INT32;
    if (*sa).data_blocks.is_null() {
        opj_free(sa as *mut libc::c_void);
        return 0 as *mut opj_sparse_array_int32_t
    }
    return sa;
}
#[no_mangle]
pub unsafe extern "C" fn opj_sparse_array_int32_free(mut sa:
                                                         *mut opj_sparse_array_int32_t) {
    if !sa.is_null() {
        let mut i: OPJ_UINT32 = 0;
        i = 0 as libc::c_int as OPJ_UINT32;
        while i < (*sa).block_count_hor.wrapping_mul((*sa).block_count_ver) {
            if !(*(*sa).data_blocks.offset(i as isize)).is_null() {
                opj_free(*(*sa).data_blocks.offset(i as isize) as
                             *mut libc::c_void);
            }
            i = i.wrapping_add(1)
        }
        opj_free((*sa).data_blocks as *mut libc::c_void);
        opj_free(sa as *mut libc::c_void);
    };
}
#[no_mangle]
pub unsafe extern "C" fn opj_sparse_array_is_region_valid(mut sa:
                                                              *const opj_sparse_array_int32_t,
                                                          mut x0: OPJ_UINT32,
                                                          mut y0: OPJ_UINT32,
                                                          mut x1: OPJ_UINT32,
                                                          mut y1: OPJ_UINT32)
 -> OPJ_BOOL {
    return !(x0 >= (*sa).width || x1 <= x0 || x1 > (*sa).width ||
                 y0 >= (*sa).height || y1 <= y0 || y1 > (*sa).height) as
               libc::c_int;
}
unsafe extern "C" fn opj_sparse_array_int32_read_or_write(mut sa:
                                                              *const opj_sparse_array_int32_t,
                                                          mut x0: OPJ_UINT32,
                                                          mut y0: OPJ_UINT32,
                                                          mut x1: OPJ_UINT32,
                                                          mut y1: OPJ_UINT32,
                                                          mut buf:
                                                              *mut OPJ_INT32,
                                                          mut buf_col_stride:
                                                              OPJ_UINT32,
                                                          mut buf_line_stride:
                                                              OPJ_UINT32,
                                                          mut forgiving:
                                                              OPJ_BOOL,
                                                          mut is_read_op:
                                                              OPJ_BOOL)
 -> OPJ_BOOL {
    let mut y: OPJ_UINT32 = 0;
    let mut block_y: OPJ_UINT32 = 0;
    let mut y_incr = 0 as libc::c_int as OPJ_UINT32;
    let block_width = (*sa).block_width;
    if opj_sparse_array_is_region_valid(sa, x0, y0, x1, y1) == 0 {
        return forgiving
    }
    block_y = y0.wrapping_div((*sa).block_height);
    y = y0;
    while y < y1 {
        let mut x: OPJ_UINT32 = 0;
        let mut block_x: OPJ_UINT32 = 0;
        let mut x_incr = 0 as libc::c_int as OPJ_UINT32;
        let mut block_y_offset: OPJ_UINT32 = 0;
        y_incr =
            if y == y0 {
                (*sa).block_height.wrapping_sub(y0.wrapping_rem((*sa).block_height))
            } else { (*sa).block_height };
        block_y_offset = (*sa).block_height.wrapping_sub(y_incr);
        y_incr = opj_uint_min(y_incr, y1.wrapping_sub(y));
        block_x = x0.wrapping_div(block_width);
        x = x0;
        while x < x1 {
            let mut j: OPJ_UINT32 = 0;
            let mut block_x_offset: OPJ_UINT32 = 0;
            let mut src_block = 0 as *mut OPJ_INT32;
            x_incr =
                if x == x0 {
                    block_width.wrapping_sub(x0.wrapping_rem(block_width))
                } else { block_width };
            block_x_offset = block_width.wrapping_sub(x_incr);
            x_incr = opj_uint_min(x_incr, x1.wrapping_sub(x));
            src_block =
                *(*sa).data_blocks.offset(block_y.wrapping_mul((*sa).block_count_hor).wrapping_add(block_x)
                                              as isize);
            if is_read_op != 0 {
                if src_block.is_null() {
                    if buf_col_stride == 1 as libc::c_int as libc::c_uint {
                        let mut dest_ptr =
                            buf.offset((y.wrapping_sub(y0) as
                                            libc::c_ulong).wrapping_mul(buf_line_stride
                                                                            as
                                                                            OPJ_SIZE_T)
                                           as
                                           isize).offset(x.wrapping_sub(x0).wrapping_mul(buf_col_stride)
                                                             as isize);
                        j = 0 as libc::c_int as OPJ_UINT32;
                        while j < y_incr {
                            memset(dest_ptr as *mut libc::c_void,
                                   0 as libc::c_int,
                                   (::std::mem::size_of::<OPJ_INT32>() as
                                        libc::c_ulong).wrapping_mul(x_incr as
                                                                        libc::c_ulong));
                            dest_ptr =
                                dest_ptr.offset(buf_line_stride as isize);
                            j = j.wrapping_add(1)
                        }
                    } else {
                        let mut dest_ptr_0 =
                            buf.offset((y.wrapping_sub(y0) as
                                            libc::c_ulong).wrapping_mul(buf_line_stride
                                                                            as
                                                                            OPJ_SIZE_T)
                                           as
                                           isize).offset(x.wrapping_sub(x0).wrapping_mul(buf_col_stride)
                                                             as isize);
                        j = 0 as libc::c_int as OPJ_UINT32;
                        while j < y_incr {
                            let mut k: OPJ_UINT32 = 0;
                            k = 0 as libc::c_int as OPJ_UINT32;
                            while k < x_incr {
                                *dest_ptr_0.offset(k.wrapping_mul(buf_col_stride)
                                                       as isize) =
                                    0 as libc::c_int;
                                k = k.wrapping_add(1)
                            }
                            dest_ptr_0 =
                                dest_ptr_0.offset(buf_line_stride as isize);
                            j = j.wrapping_add(1)
                        }
                    }
                } else {
                    let mut src_ptr: *const OPJ_INT32 =
                        src_block.offset((block_y_offset as
                                              libc::c_ulong).wrapping_mul(block_width
                                                                              as
                                                                              OPJ_SIZE_T)
                                             as
                                             isize).offset(block_x_offset as
                                                               isize);
                    if buf_col_stride == 1 as libc::c_int as libc::c_uint {
                        let mut dest_ptr_1 =
                            buf.offset((y.wrapping_sub(y0) as
                                            libc::c_ulong).wrapping_mul(buf_line_stride
                                                                            as
                                                                            OPJ_SIZE_T)
                                           as
                                           isize).offset(x.wrapping_sub(x0).wrapping_mul(buf_col_stride)
                                                             as isize);
                        if x_incr == 4 as libc::c_int as libc::c_uint {
                            /* Same code as general branch, but the compiler */
                            /* can have an efficient memcpy() */
                            /* trick to silent cppcheck duplicateBranch warning */
                            j = 0 as libc::c_int as OPJ_UINT32;
                            while j < y_incr {
                                memcpy(dest_ptr_1 as *mut libc::c_void,
                                       src_ptr as *const libc::c_void,
                                       (::std::mem::size_of::<OPJ_INT32>() as
                                            libc::c_ulong).wrapping_mul(x_incr
                                                                            as
                                                                            libc::c_ulong));
                                dest_ptr_1 =
                                    dest_ptr_1.offset(buf_line_stride as
                                                          isize);
                                src_ptr =
                                    src_ptr.offset(block_width as isize);
                                j = j.wrapping_add(1)
                            }
                        } else {
                            j = 0 as libc::c_int as OPJ_UINT32;
                            while j < y_incr {
                                memcpy(dest_ptr_1 as *mut libc::c_void,
                                       src_ptr as *const libc::c_void,
                                       (::std::mem::size_of::<OPJ_INT32>() as
                                            libc::c_ulong).wrapping_mul(x_incr
                                                                            as
                                                                            libc::c_ulong));
                                dest_ptr_1 =
                                    dest_ptr_1.offset(buf_line_stride as
                                                          isize);
                                src_ptr =
                                    src_ptr.offset(block_width as isize);
                                j = j.wrapping_add(1)
                            }
                        }
                    } else {
                        let mut dest_ptr_2 =
                            buf.offset((y.wrapping_sub(y0) as
                                            libc::c_ulong).wrapping_mul(buf_line_stride
                                                                            as
                                                                            OPJ_SIZE_T)
                                           as
                                           isize).offset(x.wrapping_sub(x0).wrapping_mul(buf_col_stride)
                                                             as isize);
                        if x_incr == 1 as libc::c_int as libc::c_uint {
                            j = 0 as libc::c_int as OPJ_UINT32;
                            while j < y_incr {
                                *dest_ptr_2 = *src_ptr;
                                dest_ptr_2 =
                                    dest_ptr_2.offset(buf_line_stride as
                                                          isize);
                                src_ptr =
                                    src_ptr.offset(block_width as isize);
                                j = j.wrapping_add(1)
                            }
                        } else if y_incr == 1 as libc::c_int as libc::c_uint
                                      &&
                                      buf_col_stride ==
                                          2 as libc::c_int as libc::c_uint {
                            let mut k_0: OPJ_UINT32 = 0;
                            k_0 = 0 as libc::c_int as OPJ_UINT32;
                            while k_0 < x_incr & !(3 as libc::c_uint) {
                                *dest_ptr_2.offset(k_0.wrapping_mul(buf_col_stride)
                                                       as isize) =
                                    *src_ptr.offset(k_0 as isize);
                                *dest_ptr_2.offset(k_0.wrapping_add(1 as
                                                                        libc::c_int
                                                                        as
                                                                        libc::c_uint).wrapping_mul(buf_col_stride)
                                                       as isize) =
                                    *src_ptr.offset(k_0.wrapping_add(1 as
                                                                         libc::c_int
                                                                         as
                                                                         libc::c_uint)
                                                        as isize);
                                *dest_ptr_2.offset(k_0.wrapping_add(2 as
                                                                        libc::c_int
                                                                        as
                                                                        libc::c_uint).wrapping_mul(buf_col_stride)
                                                       as isize) =
                                    *src_ptr.offset(k_0.wrapping_add(2 as
                                                                         libc::c_int
                                                                         as
                                                                         libc::c_uint)
                                                        as isize);
                                *dest_ptr_2.offset(k_0.wrapping_add(3 as
                                                                        libc::c_int
                                                                        as
                                                                        libc::c_uint).wrapping_mul(buf_col_stride)
                                                       as isize) =
                                    *src_ptr.offset(k_0.wrapping_add(3 as
                                                                         libc::c_int
                                                                         as
                                                                         libc::c_uint)
                                                        as isize);
                                k_0 =
                                    (k_0 as
                                         libc::c_uint).wrapping_add(4 as
                                                                        libc::c_int
                                                                        as
                                                                        libc::c_uint)
                                        as OPJ_UINT32 as OPJ_UINT32
                            }
                            while k_0 < x_incr {
                                *dest_ptr_2.offset(k_0.wrapping_mul(buf_col_stride)
                                                       as isize) =
                                    *src_ptr.offset(k_0 as isize);
                                k_0 = k_0.wrapping_add(1)
                            }
                        } else if x_incr >= 8 as libc::c_int as libc::c_uint
                                      &&
                                      buf_col_stride ==
                                          8 as libc::c_int as libc::c_uint {
                            j = 0 as libc::c_int as OPJ_UINT32;
                            while j < y_incr {
                                let mut k_1: OPJ_UINT32 = 0;
                                k_1 = 0 as libc::c_int as OPJ_UINT32;
                                while k_1 < x_incr & !(3 as libc::c_uint) {
                                    *dest_ptr_2.offset(k_1.wrapping_mul(buf_col_stride)
                                                           as isize) =
                                        *src_ptr.offset(k_1 as isize);
                                    *dest_ptr_2.offset(k_1.wrapping_add(1 as
                                                                            libc::c_int
                                                                            as
                                                                            libc::c_uint).wrapping_mul(buf_col_stride)
                                                           as isize) =
                                        *src_ptr.offset(k_1.wrapping_add(1 as
                                                                             libc::c_int
                                                                             as
                                                                             libc::c_uint)
                                                            as isize);
                                    *dest_ptr_2.offset(k_1.wrapping_add(2 as
                                                                            libc::c_int
                                                                            as
                                                                            libc::c_uint).wrapping_mul(buf_col_stride)
                                                           as isize) =
                                        *src_ptr.offset(k_1.wrapping_add(2 as
                                                                             libc::c_int
                                                                             as
                                                                             libc::c_uint)
                                                            as isize);
                                    *dest_ptr_2.offset(k_1.wrapping_add(3 as
                                                                            libc::c_int
                                                                            as
                                                                            libc::c_uint).wrapping_mul(buf_col_stride)
                                                           as isize) =
                                        *src_ptr.offset(k_1.wrapping_add(3 as
                                                                             libc::c_int
                                                                             as
                                                                             libc::c_uint)
                                                            as isize);
                                    k_1 =
                                        (k_1 as
                                             libc::c_uint).wrapping_add(4 as
                                                                            libc::c_int
                                                                            as
                                                                            libc::c_uint)
                                            as OPJ_UINT32 as OPJ_UINT32
                                }
                                while k_1 < x_incr {
                                    *dest_ptr_2.offset(k_1.wrapping_mul(buf_col_stride)
                                                           as isize) =
                                        *src_ptr.offset(k_1 as isize);
                                    k_1 = k_1.wrapping_add(1)
                                }
                                dest_ptr_2 =
                                    dest_ptr_2.offset(buf_line_stride as
                                                          isize);
                                src_ptr =
                                    src_ptr.offset(block_width as isize);
                                j = j.wrapping_add(1)
                            }
                        } else {
                            /* General case */
                            j = 0 as libc::c_int as OPJ_UINT32;
                            while j < y_incr {
                                let mut k_2: OPJ_UINT32 = 0;
                                k_2 = 0 as libc::c_int as OPJ_UINT32;
                                while k_2 < x_incr {
                                    *dest_ptr_2.offset(k_2.wrapping_mul(buf_col_stride)
                                                           as isize) =
                                        *src_ptr.offset(k_2 as isize);
                                    k_2 = k_2.wrapping_add(1)
                                }
                                dest_ptr_2 =
                                    dest_ptr_2.offset(buf_line_stride as
                                                          isize);
                                src_ptr =
                                    src_ptr.offset(block_width as isize);
                                j = j.wrapping_add(1)
                            }
                        }
                    }
                }
            } else {
                if src_block.is_null() {
                    src_block =
                        opj_calloc(1 as libc::c_int as size_t,
                                   ((*sa).block_width as
                                        size_t).wrapping_mul((*sa).block_height
                                                                 as
                                                                 libc::c_ulong).wrapping_mul(::std::mem::size_of::<OPJ_INT32>()
                                                                                                 as
                                                                                                 libc::c_ulong))
                            as *mut OPJ_INT32;
                    if src_block.is_null() { return 0 as libc::c_int }
                    let ref mut fresh0 =
                        *(*sa).data_blocks.offset(block_y.wrapping_mul((*sa).block_count_hor).wrapping_add(block_x)
                                                      as isize);
                    *fresh0 = src_block
                }
                if buf_col_stride == 1 as libc::c_int as libc::c_uint {
                    let mut dest_ptr_3 =
                        src_block.offset((block_y_offset as
                                              libc::c_ulong).wrapping_mul(block_width
                                                                              as
                                                                              OPJ_SIZE_T)
                                             as
                                             isize).offset(block_x_offset as
                                                               isize);
                    let mut src_ptr_0: *const OPJ_INT32 =
                        buf.offset((y.wrapping_sub(y0) as
                                        libc::c_ulong).wrapping_mul(buf_line_stride
                                                                        as
                                                                        OPJ_SIZE_T)
                                       as
                                       isize).offset(x.wrapping_sub(x0).wrapping_mul(buf_col_stride)
                                                         as isize);
                    if x_incr == 4 as libc::c_int as libc::c_uint {
                        /* Same code as general branch, but the compiler */
                        /* can have an efficient memcpy() */
                        /* trick to silent cppcheck duplicateBranch warning */
                        j = 0 as libc::c_int as OPJ_UINT32;
                        while j < y_incr {
                            memcpy(dest_ptr_3 as *mut libc::c_void,
                                   src_ptr_0 as *const libc::c_void,
                                   (::std::mem::size_of::<OPJ_INT32>() as
                                        libc::c_ulong).wrapping_mul(x_incr as
                                                                        libc::c_ulong));
                            dest_ptr_3 =
                                dest_ptr_3.offset(block_width as isize);
                            src_ptr_0 =
                                src_ptr_0.offset(buf_line_stride as isize);
                            j = j.wrapping_add(1)
                        }
                    } else {
                        j = 0 as libc::c_int as OPJ_UINT32;
                        while j < y_incr {
                            memcpy(dest_ptr_3 as *mut libc::c_void,
                                   src_ptr_0 as *const libc::c_void,
                                   (::std::mem::size_of::<OPJ_INT32>() as
                                        libc::c_ulong).wrapping_mul(x_incr as
                                                                        libc::c_ulong));
                            dest_ptr_3 =
                                dest_ptr_3.offset(block_width as isize);
                            src_ptr_0 =
                                src_ptr_0.offset(buf_line_stride as isize);
                            j = j.wrapping_add(1)
                        }
                    }
                } else {
                    let mut dest_ptr_4 =
                        src_block.offset((block_y_offset as
                                              libc::c_ulong).wrapping_mul(block_width
                                                                              as
                                                                              OPJ_SIZE_T)
                                             as
                                             isize).offset(block_x_offset as
                                                               isize);
                    let mut src_ptr_1: *const OPJ_INT32 =
                        buf.offset((y.wrapping_sub(y0) as
                                        libc::c_ulong).wrapping_mul(buf_line_stride
                                                                        as
                                                                        OPJ_SIZE_T)
                                       as
                                       isize).offset(x.wrapping_sub(x0).wrapping_mul(buf_col_stride)
                                                         as isize);
                    if x_incr == 1 as libc::c_int as libc::c_uint {
                        j = 0 as libc::c_int as OPJ_UINT32;
                        while j < y_incr {
                            *dest_ptr_4 = *src_ptr_1;
                            src_ptr_1 =
                                src_ptr_1.offset(buf_line_stride as isize);
                            dest_ptr_4 =
                                dest_ptr_4.offset(block_width as isize);
                            j = j.wrapping_add(1)
                        }
                    } else if x_incr >= 8 as libc::c_int as libc::c_uint &&
                                  buf_col_stride ==
                                      8 as libc::c_int as libc::c_uint {
                        j = 0 as libc::c_int as OPJ_UINT32;
                        while j < y_incr {
                            let mut k_3: OPJ_UINT32 = 0;
                            k_3 = 0 as libc::c_int as OPJ_UINT32;
                            while k_3 < x_incr & !(3 as libc::c_uint) {
                                *dest_ptr_4.offset(k_3 as isize) =
                                    *src_ptr_1.offset(k_3.wrapping_mul(buf_col_stride)
                                                          as isize);
                                *dest_ptr_4.offset(k_3.wrapping_add(1 as
                                                                        libc::c_int
                                                                        as
                                                                        libc::c_uint)
                                                       as isize) =
                                    *src_ptr_1.offset(k_3.wrapping_add(1 as
                                                                           libc::c_int
                                                                           as
                                                                           libc::c_uint).wrapping_mul(buf_col_stride)
                                                          as isize);
                                *dest_ptr_4.offset(k_3.wrapping_add(2 as
                                                                        libc::c_int
                                                                        as
                                                                        libc::c_uint)
                                                       as isize) =
                                    *src_ptr_1.offset(k_3.wrapping_add(2 as
                                                                           libc::c_int
                                                                           as
                                                                           libc::c_uint).wrapping_mul(buf_col_stride)
                                                          as isize);
                                *dest_ptr_4.offset(k_3.wrapping_add(3 as
                                                                        libc::c_int
                                                                        as
                                                                        libc::c_uint)
                                                       as isize) =
                                    *src_ptr_1.offset(k_3.wrapping_add(3 as
                                                                           libc::c_int
                                                                           as
                                                                           libc::c_uint).wrapping_mul(buf_col_stride)
                                                          as isize);
                                k_3 =
                                    (k_3 as
                                         libc::c_uint).wrapping_add(4 as
                                                                        libc::c_int
                                                                        as
                                                                        libc::c_uint)
                                        as OPJ_UINT32 as OPJ_UINT32
                            }
                            while k_3 < x_incr {
                                *dest_ptr_4.offset(k_3 as isize) =
                                    *src_ptr_1.offset(k_3.wrapping_mul(buf_col_stride)
                                                          as isize);
                                k_3 = k_3.wrapping_add(1)
                            }
                            src_ptr_1 =
                                src_ptr_1.offset(buf_line_stride as isize);
                            dest_ptr_4 =
                                dest_ptr_4.offset(block_width as isize);
                            j = j.wrapping_add(1)
                        }
                    } else {
                        /* General case */
                        j = 0 as libc::c_int as OPJ_UINT32;
                        while j < y_incr {
                            let mut k_4: OPJ_UINT32 = 0;
                            k_4 = 0 as libc::c_int as OPJ_UINT32;
                            while k_4 < x_incr {
                                *dest_ptr_4.offset(k_4 as isize) =
                                    *src_ptr_1.offset(k_4.wrapping_mul(buf_col_stride)
                                                          as isize);
                                k_4 = k_4.wrapping_add(1)
                            }
                            src_ptr_1 =
                                src_ptr_1.offset(buf_line_stride as isize);
                            dest_ptr_4 =
                                dest_ptr_4.offset(block_width as isize);
                            j = j.wrapping_add(1)
                        }
                    }
                }
            }
            block_x = block_x.wrapping_add(1);
            x =
                (x as libc::c_uint).wrapping_add(x_incr) as OPJ_UINT32 as
                    OPJ_UINT32
        }
        block_y = block_y.wrapping_add(1);
        y =
            (y as libc::c_uint).wrapping_add(y_incr) as OPJ_UINT32 as
                OPJ_UINT32
    }
    return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_sparse_array_int32_read(mut sa:
                                                         *const opj_sparse_array_int32_t,
                                                     mut x0: OPJ_UINT32,
                                                     mut y0: OPJ_UINT32,
                                                     mut x1: OPJ_UINT32,
                                                     mut y1: OPJ_UINT32,
                                                     mut dest: *mut OPJ_INT32,
                                                     mut dest_col_stride:
                                                         OPJ_UINT32,
                                                     mut dest_line_stride:
                                                         OPJ_UINT32,
                                                     mut forgiving: OPJ_BOOL)
 -> OPJ_BOOL {
    return opj_sparse_array_int32_read_or_write(sa as
                                                    *mut opj_sparse_array_int32_t,
                                                x0, y0, x1, y1, dest,
                                                dest_col_stride,
                                                dest_line_stride, forgiving,
                                                1 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn opj_sparse_array_int32_write(mut sa:
                                                          *mut opj_sparse_array_int32_t,
                                                      mut x0: OPJ_UINT32,
                                                      mut y0: OPJ_UINT32,
                                                      mut x1: OPJ_UINT32,
                                                      mut y1: OPJ_UINT32,
                                                      mut src:
                                                          *const OPJ_INT32,
                                                      mut src_col_stride:
                                                          OPJ_UINT32,
                                                      mut src_line_stride:
                                                          OPJ_UINT32,
                                                      mut forgiving: OPJ_BOOL)
 -> OPJ_BOOL {
    return opj_sparse_array_int32_read_or_write(sa, x0, y0, x1, y1,
                                                src as *mut OPJ_INT32,
                                                src_col_stride,
                                                src_line_stride, forgiving,
                                                0 as libc::c_int);
}
