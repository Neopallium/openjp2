use ::libc;
extern "C" {
    #[no_mangle]
    fn getrusage(__who: __rusage_who_t, __usage: *mut rusage) -> libc::c_int;
}
pub type __time_t = libc::c_long;
pub type __suseconds_t = libc::c_long;
pub type __syscall_slong_t = libc::c_long;
pub type OPJ_FLOAT64 = libc::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct rusage {
    pub ru_utime: timeval,
    pub ru_stime: timeval,
    pub c2rust_unnamed: C2RustUnnamed_12,
    pub c2rust_unnamed_0: C2RustUnnamed_11,
    pub c2rust_unnamed_1: C2RustUnnamed_10,
    pub c2rust_unnamed_2: C2RustUnnamed_9,
    pub c2rust_unnamed_3: C2RustUnnamed_8,
    pub c2rust_unnamed_4: C2RustUnnamed_7,
    pub c2rust_unnamed_5: C2RustUnnamed_6,
    pub c2rust_unnamed_6: C2RustUnnamed_5,
    pub c2rust_unnamed_7: C2RustUnnamed_4,
    pub c2rust_unnamed_8: C2RustUnnamed_3,
    pub c2rust_unnamed_9: C2RustUnnamed_2,
    pub c2rust_unnamed_10: C2RustUnnamed_1,
    pub c2rust_unnamed_11: C2RustUnnamed_0,
    pub c2rust_unnamed_12: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub ru_nivcsw: libc::c_long,
    pub __ru_nivcsw_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub ru_nvcsw: libc::c_long,
    pub __ru_nvcsw_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub ru_nsignals: libc::c_long,
    pub __ru_nsignals_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_2 {
    pub ru_msgrcv: libc::c_long,
    pub __ru_msgrcv_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_3 {
    pub ru_msgsnd: libc::c_long,
    pub __ru_msgsnd_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_4 {
    pub ru_oublock: libc::c_long,
    pub __ru_oublock_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_5 {
    pub ru_inblock: libc::c_long,
    pub __ru_inblock_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_6 {
    pub ru_nswap: libc::c_long,
    pub __ru_nswap_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_7 {
    pub ru_majflt: libc::c_long,
    pub __ru_majflt_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_8 {
    pub ru_minflt: libc::c_long,
    pub __ru_minflt_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_9 {
    pub ru_isrss: libc::c_long,
    pub __ru_isrss_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_10 {
    pub ru_idrss: libc::c_long,
    pub __ru_idrss_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_11 {
    pub ru_ixrss: libc::c_long,
    pub __ru_ixrss_word: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_12 {
    pub ru_maxrss: libc::c_long,
    pub __ru_maxrss_word: __syscall_slong_t,
}
pub type __rusage_who_t = libc::c_int;
/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2005, Herve Drolon, FreeImage Team
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
/* _WIN32 */
#[no_mangle]
pub unsafe extern "C" fn opj_clock() -> OPJ_FLOAT64 {
    /* Unix or Linux: use resource usage */
    let mut t =
        rusage{ru_utime: timeval{tv_sec: 0, tv_usec: 0,},
               ru_stime: timeval{tv_sec: 0, tv_usec: 0,},
               c2rust_unnamed: C2RustUnnamed_12{ru_maxrss: 0,},
               c2rust_unnamed_0: C2RustUnnamed_11{ru_ixrss: 0,},
               c2rust_unnamed_1: C2RustUnnamed_10{ru_idrss: 0,},
               c2rust_unnamed_2: C2RustUnnamed_9{ru_isrss: 0,},
               c2rust_unnamed_3: C2RustUnnamed_8{ru_minflt: 0,},
               c2rust_unnamed_4: C2RustUnnamed_7{ru_majflt: 0,},
               c2rust_unnamed_5: C2RustUnnamed_6{ru_nswap: 0,},
               c2rust_unnamed_6: C2RustUnnamed_5{ru_inblock: 0,},
               c2rust_unnamed_7: C2RustUnnamed_4{ru_oublock: 0,},
               c2rust_unnamed_8: C2RustUnnamed_3{ru_msgsnd: 0,},
               c2rust_unnamed_9: C2RustUnnamed_2{ru_msgrcv: 0,},
               c2rust_unnamed_10: C2RustUnnamed_1{ru_nsignals: 0,},
               c2rust_unnamed_11: C2RustUnnamed_0{ru_nvcsw: 0,},
               c2rust_unnamed_12: C2RustUnnamed{ru_nivcsw: 0,},};
    let mut procTime: OPJ_FLOAT64 = 0.;
    /* (1) Get the rusage data structure at this moment (man getrusage) */
    getrusage(0 as libc::c_int, &mut t);
    /* (2) What is the elapsed time ? - CPU time = User time + System time */
    /* (2a) Get the seconds */
    procTime = (t.ru_utime.tv_sec + t.ru_stime.tv_sec) as OPJ_FLOAT64;
    /* (2b) More precisely! Get the microseconds part ! */
    return procTime +
               (t.ru_utime.tv_usec + t.ru_stime.tv_usec) as OPJ_FLOAT64 *
                   1e-6f64;
}
