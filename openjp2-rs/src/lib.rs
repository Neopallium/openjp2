#![allow(dead_code)]
#![allow(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![feature(c_variadic)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(const_transmute)]
#![feature(extern_types)]
#![feature(label_break_value)]
#![feature(ptr_wrapping_offset_from)]
#![feature(register_tool)]
#![register_tool(c2rust)]

#[macro_use]
extern crate c2rust_bitfields;
extern crate libc;

pub mod bio;
pub mod cio;
pub mod dwt;
pub mod event;
pub mod function_list;
pub mod ht_dec;
pub mod image;
pub mod invert;
pub mod j2k;
pub mod jp2;
pub mod mct;
pub mod mqc;
pub mod openjpeg;
pub mod opj_clock;
pub mod opj_malloc;
pub mod pi;
pub mod sparse_array;
pub mod t1;
pub mod t2;
pub mod tcd;
pub mod tgt;
pub mod thread;
