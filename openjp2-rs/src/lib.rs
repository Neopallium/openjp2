#![feature(new_uninit)]
#![allow(dead_code)]
#![allow(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

#[macro_use]
extern crate c2rust_bitfields;
extern crate libc;

mod c_api_types;
mod consts;
mod types;

#[macro_use]
mod event;

// Public OpenJpeg interface.
pub mod openjpeg;
pub mod image;
pub mod cio;

mod bio;
mod codec;
mod dwt;
mod function_list;
mod ht_dec;
mod invert;
mod j2k;
mod jp2;
mod malloc;
mod math;
mod mct;
mod mqc;
mod pi;
mod sparse_array;
mod t1;
mod t1_ht_luts;
mod t1_luts;
mod t2;
mod tcd;
mod tgt;
mod thread;
