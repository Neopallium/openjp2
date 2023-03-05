use super::openjpeg::*;
use ::libc;

use super::malloc::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_thread_t(usize);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_mutex_t(usize);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_cond_t(usize);

pub type opj_thread_fn = Option<unsafe extern "C" fn(_: *mut libc::c_void) -> ()>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_thread_pool_t {
  pub worker_threads: *mut opj_worker_thread_t,
  pub worker_threads_count: libc::c_int,
  pub cond: *mut opj_cond_t,
  pub mutex: *mut opj_mutex_t,
  pub state: opj_worker_thread_state,
  pub job_queue: *mut opj_job_list_t,
  pub pending_jobs_count: libc::c_int,
  pub waiting_worker_thread_list: *mut opj_worker_thread_list_t,
  pub waiting_worker_thread_count: libc::c_int,
  pub signaling_threshold: libc::c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_worker_thread_list_t {
  pub worker_thread: *mut opj_worker_thread_t,
  pub next: *mut opj_worker_thread_list_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_worker_thread_t {
  pub tp: *mut opj_thread_pool_t,
  pub thread: *mut opj_thread_t,
  pub marked_as_waiting: libc::c_int,
  pub mutex: *mut opj_mutex_t,
  pub cond: *mut opj_cond_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_job_list_t {
  pub job: *mut opj_worker_thread_job_t,
  pub next: *mut opj_job_list_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct opj_worker_thread_job_t {
  pub job_fn: opj_job_fn,
  pub user_data: *mut libc::c_void,
}
pub type opj_job_fn = Option<unsafe extern "C" fn(_: *mut libc::c_void) -> ()>;
pub type opj_worker_thread_state = libc::c_uint;
pub const OPJWTS_ERROR: opj_worker_thread_state = 2;
pub const OPJWTS_STOP: opj_worker_thread_state = 1;
pub const OPJWTS_OK: opj_worker_thread_state = 0;
/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2016, Even Rouault
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
/* Stub implementation */
#[no_mangle]
pub fn opj_has_thread_support() -> OPJ_BOOL {
  return 0i32;
}
#[no_mangle]
pub fn opj_get_num_cpus() -> libc::c_int {
  return 1i32;
}

pub(crate) fn opj_mutex_create() -> *mut opj_mutex_t {
  return 0 as *mut opj_mutex_t;
}
pub(crate) fn opj_mutex_lock(mut _mutex: *mut opj_mutex_t) {}
pub(crate) fn opj_mutex_unlock(mut _mutex: *mut opj_mutex_t) {}
pub(crate) fn opj_mutex_destroy(mut _mutex: *mut opj_mutex_t) {}
pub(crate) fn opj_cond_create() -> *mut opj_cond_t {
  return 0 as *mut opj_cond_t;
}

pub(crate) fn opj_cond_wait(mut _cond: *mut opj_cond_t, mut _mutex: *mut opj_mutex_t) {}
pub(crate) fn opj_cond_signal(mut _cond: *mut opj_cond_t) {}
pub(crate) fn opj_cond_destroy(mut _cond: *mut opj_cond_t) {}

pub(crate) fn opj_thread_create(
  mut _thread_fn: opj_thread_fn,
  mut _user_data: *mut libc::c_void,
) -> *mut opj_thread_t {
  return 0 as *mut opj_thread_t;
}

pub(crate) fn opj_thread_join(mut _thread: *mut opj_thread_t) {}

pub(crate) unsafe fn opj_thread_pool_create(
  mut num_threads: libc::c_int,
) -> *mut opj_thread_pool_t {
  let mut tp = 0 as *mut opj_thread_pool_t;
  tp = opj_calloc(
    1i32 as size_t,
    core::mem::size_of::<opj_thread_pool_t>() as usize,
  ) as *mut opj_thread_pool_t;
  if tp.is_null() {
    return 0 as *mut opj_thread_pool_t;
  }
  core::ptr::write_volatile(&mut (*tp).state as *mut opj_worker_thread_state, OPJWTS_OK);
  if num_threads <= 0i32 {
    return tp;
  }
  (*tp).mutex = opj_mutex_create();
  if (*tp).mutex.is_null() {
    opj_free(tp as *mut libc::c_void);
    return 0 as *mut opj_thread_pool_t;
  }
  if opj_thread_pool_setup(tp, num_threads) == 0 {
    opj_thread_pool_destroy(tp);
    return 0 as *mut opj_thread_pool_t;
  }
  return tp;
}

unsafe extern "C" fn opj_worker_thread_function(mut user_data: *mut libc::c_void) {
  let mut worker_thread = 0 as *mut opj_worker_thread_t;
  let mut tp = 0 as *mut opj_thread_pool_t;
  let mut job_finished = 0i32;
  worker_thread = user_data as *mut opj_worker_thread_t;
  tp = (*worker_thread).tp;
  loop {
    let mut job = opj_thread_pool_get_next_job(tp, worker_thread, job_finished);
    if job.is_null() {
      break;
    }
    if (*job).job_fn.is_some() {
      (*job).job_fn.expect("non-null function pointer")((*job).user_data);
    }
    opj_free(job as *mut libc::c_void);
    job_finished = 1i32
  }
}

unsafe fn opj_thread_pool_setup(
  mut tp: *mut opj_thread_pool_t,
  mut num_threads: libc::c_int,
) -> OPJ_BOOL {
  let mut i: libc::c_int = 0;
  let mut bRet = 1i32;
  assert!(num_threads > 0i32);
  (*tp).cond = opj_cond_create();
  if (*tp).cond.is_null() {
    return 0i32;
  }
  (*tp).worker_threads = opj_calloc(
    num_threads as size_t,
    core::mem::size_of::<opj_worker_thread_t>() as usize,
  ) as *mut opj_worker_thread_t;
  if (*tp).worker_threads.is_null() {
    return 0i32;
  }
  (*tp).worker_threads_count = num_threads;
  i = 0i32;
  while i < num_threads {
    let ref mut fresh4 = (*(*tp).worker_threads.offset(i as isize)).tp;
    *fresh4 = tp;
    let ref mut fresh5 = (*(*tp).worker_threads.offset(i as isize)).mutex;
    *fresh5 = opj_mutex_create();
    if (*(*tp).worker_threads.offset(i as isize)).mutex.is_null() {
      (*tp).worker_threads_count = i;
      bRet = 0i32;
      break;
    } else {
      let ref mut fresh6 = (*(*tp).worker_threads.offset(i as isize)).cond;
      *fresh6 = opj_cond_create();
      if (*(*tp).worker_threads.offset(i as isize)).cond.is_null() {
        opj_mutex_destroy((*(*tp).worker_threads.offset(i as isize)).mutex);
        (*tp).worker_threads_count = i;
        bRet = 0i32;
        break;
      } else {
        (*(*tp).worker_threads.offset(i as isize)).marked_as_waiting = 0i32;
        let ref mut fresh7 = (*(*tp).worker_threads.offset(i as isize)).thread;
        *fresh7 = opj_thread_create(
          Some(opj_worker_thread_function as unsafe extern "C" fn(_: *mut libc::c_void) -> ()),
          &mut *(*tp).worker_threads.offset(i as isize) as *mut opj_worker_thread_t
            as *mut libc::c_void,
        );
        if (*(*tp).worker_threads.offset(i as isize)).thread.is_null() {
          opj_mutex_destroy((*(*tp).worker_threads.offset(i as isize)).mutex);
          opj_cond_destroy((*(*tp).worker_threads.offset(i as isize)).cond);
          (*tp).worker_threads_count = i;
          bRet = 0i32;
          break;
        } else {
          i += 1
        }
      }
    }
  }
  /* Wait all threads to be started */
  /* printf("waiting for all threads to be started\n"); */
  opj_mutex_lock((*tp).mutex);
  while (*tp).waiting_worker_thread_count < (*tp).worker_threads_count {
    opj_cond_wait((*tp).cond, (*tp).mutex);
  }
  opj_mutex_unlock((*tp).mutex);
  /* printf("all threads started\n"); */
  if (*tp).state as libc::c_uint == OPJWTS_ERROR as libc::c_uint {
    bRet = 0i32
  }
  return bRet;
}

unsafe fn opj_thread_pool_get_next_job(
  mut tp: *mut opj_thread_pool_t,
  mut worker_thread: *mut opj_worker_thread_t,
  mut signal_job_finished: OPJ_BOOL,
) -> *mut opj_worker_thread_job_t {
  loop {
    let mut top_job_iter = 0 as *mut opj_job_list_t;
    opj_mutex_lock((*tp).mutex);
    if signal_job_finished != 0 {
      signal_job_finished = 0i32;
      core::ptr::write_volatile(
        &mut (*tp).pending_jobs_count as *mut libc::c_int,
        core::ptr::read_volatile::<libc::c_int>(&(*tp).pending_jobs_count as *const libc::c_int)
          - 1,
      );
      /* printf("got job\n"); */
      /*printf("tp=%p, remaining jobs: %d\n", tp, tp->pending_jobs_count);*/
      if (*tp).pending_jobs_count <= (*tp).signaling_threshold {
        opj_cond_signal((*tp).cond);
      }
    }
    if (*tp).state as libc::c_uint == OPJWTS_STOP as libc::c_uint {
      opj_mutex_unlock((*tp).mutex);
      return 0 as *mut opj_worker_thread_job_t;
    }
    top_job_iter = (*tp).job_queue;
    if !top_job_iter.is_null() {
      let mut job = 0 as *mut opj_worker_thread_job_t;
      (*tp).job_queue = (*top_job_iter).next;
      job = (*top_job_iter).job;
      opj_mutex_unlock((*tp).mutex);
      opj_free(top_job_iter as *mut libc::c_void);
      return job;
    }
    if (*worker_thread).marked_as_waiting == 0 {
      let mut item = 0 as *mut opj_worker_thread_list_t;
      (*worker_thread).marked_as_waiting = 1i32;
      (*tp).waiting_worker_thread_count += 1;
      assert!((*tp).waiting_worker_thread_count <= (*tp).worker_threads_count);
      item = opj_malloc(core::mem::size_of::<opj_worker_thread_list_t>() as usize)
        as *mut opj_worker_thread_list_t;
      if item.is_null() {
        core::ptr::write_volatile(
          &mut (*tp).state as *mut opj_worker_thread_state,
          OPJWTS_ERROR,
        );
        opj_cond_signal((*tp).cond);
        opj_mutex_unlock((*tp).mutex);
        return 0 as *mut opj_worker_thread_job_t;
      }
      (*item).worker_thread = worker_thread;
      (*item).next = (*tp).waiting_worker_thread_list;
      (*tp).waiting_worker_thread_list = item
    }
    opj_cond_signal((*tp).cond);
    opj_mutex_lock((*worker_thread).mutex);
    opj_mutex_unlock((*tp).mutex);
    opj_cond_wait((*worker_thread).cond, (*worker_thread).mutex);
    opj_mutex_unlock((*worker_thread).mutex);
  }
}

pub(crate) unsafe fn opj_thread_pool_submit_job(
  mut tp: *mut opj_thread_pool_t,
  mut job_fn: opj_job_fn,
  mut user_data: *mut libc::c_void,
) -> OPJ_BOOL {
  let mut job = 0 as *mut opj_worker_thread_job_t;
  let mut item = 0 as *mut opj_job_list_t;
  if (*tp).mutex.is_null() {
    job_fn.expect("non-null function pointer")(user_data);
    return 1i32;
  }
  job = opj_malloc(core::mem::size_of::<opj_worker_thread_job_t>() as usize)
    as *mut opj_worker_thread_job_t;
  if job.is_null() {
    return 0i32;
  }
  (*job).job_fn = job_fn;
  (*job).user_data = user_data;
  item =
    opj_malloc(core::mem::size_of::<opj_job_list_t>() as usize) as *mut opj_job_list_t;
  if item.is_null() {
    opj_free(job as *mut libc::c_void);
    return 0i32;
  }
  (*item).job = job;
  opj_mutex_lock((*tp).mutex);
  (*tp).signaling_threshold = 100i32 * (*tp).worker_threads_count;
  while (*tp).pending_jobs_count > (*tp).signaling_threshold {
    /* opj_waiting(); */
    /* printf("signaling that worker thread is ready\n"); */
    /* printf("waiting for job\n"); */
    /* printf("%d jobs enqueued. Waiting\n", tp->pending_jobs_count); */
    opj_cond_wait((*tp).cond, (*tp).mutex);
    /* printf("...%d jobs enqueued.\n", tp->pending_jobs_count); */
  }
  (*item).next = (*tp).job_queue;
  (*tp).job_queue = item;
  core::ptr::write_volatile(
    &mut (*tp).pending_jobs_count as *mut libc::c_int,
    core::ptr::read_volatile::<libc::c_int>(&(*tp).pending_jobs_count as *const libc::c_int) + 1,
  );
  if !(*tp).waiting_worker_thread_list.is_null() {
    let mut worker_thread = 0 as *mut opj_worker_thread_t;
    let mut next = 0 as *mut opj_worker_thread_list_t;
    let mut to_opj_free = 0 as *mut opj_worker_thread_list_t;
    worker_thread = (*(*tp).waiting_worker_thread_list).worker_thread;
    assert!((*worker_thread).marked_as_waiting != 0);
    (*worker_thread).marked_as_waiting = 0i32;
    next = (*(*tp).waiting_worker_thread_list).next;
    to_opj_free = (*tp).waiting_worker_thread_list;
    (*tp).waiting_worker_thread_list = next;
    (*tp).waiting_worker_thread_count -= 1;
    opj_mutex_lock((*worker_thread).mutex);
    opj_mutex_unlock((*tp).mutex);
    opj_cond_signal((*worker_thread).cond);
    opj_mutex_unlock((*worker_thread).mutex);
    opj_free(to_opj_free as *mut libc::c_void);
  } else {
    opj_mutex_unlock((*tp).mutex);
  }
  return 1i32;
}

pub(crate) unsafe fn opj_thread_pool_wait_completion(
  mut tp: *mut opj_thread_pool_t,
  mut max_remaining_jobs: libc::c_int,
) {
  if (*tp).mutex.is_null() {
    return;
  }
  if max_remaining_jobs < 0i32 {
    max_remaining_jobs = 0i32
  }
  opj_mutex_lock((*tp).mutex);
  (*tp).signaling_threshold = max_remaining_jobs;
  while (*tp).pending_jobs_count > max_remaining_jobs {
    /*printf("tp=%p, jobs before wait = %d, max_remaining_jobs = %d\n", tp, tp->pending_jobs_count, max_remaining_jobs);*/
    opj_cond_wait((*tp).cond, (*tp).mutex);
    /*printf("tp=%p, jobs after wait = %d\n", tp, tp->pending_jobs_count);*/
  }
  opj_mutex_unlock((*tp).mutex);
}

pub(crate) unsafe fn opj_thread_pool_get_thread_count(
  mut tp: *mut opj_thread_pool_t,
) -> libc::c_int {
  return (*tp).worker_threads_count;
}

pub(crate) unsafe fn opj_thread_pool_destroy(mut tp: *mut opj_thread_pool_t) {
  if tp.is_null() {
    return;
  }
  if !(*tp).cond.is_null() {
    let mut i: libc::c_int = 0;
    opj_thread_pool_wait_completion(tp, 0i32);
    opj_mutex_lock((*tp).mutex);
    core::ptr::write_volatile(
      &mut (*tp).state as *mut opj_worker_thread_state,
      OPJWTS_STOP,
    );
    opj_mutex_unlock((*tp).mutex);
    i = 0i32;
    while i < (*tp).worker_threads_count {
      opj_mutex_lock((*(*tp).worker_threads.offset(i as isize)).mutex);
      opj_cond_signal((*(*tp).worker_threads.offset(i as isize)).cond);
      opj_mutex_unlock((*(*tp).worker_threads.offset(i as isize)).mutex);
      opj_thread_join((*(*tp).worker_threads.offset(i as isize)).thread);
      opj_cond_destroy((*(*tp).worker_threads.offset(i as isize)).cond);
      opj_mutex_destroy((*(*tp).worker_threads.offset(i as isize)).mutex);
      i += 1
    }
    opj_free((*tp).worker_threads as *mut libc::c_void);
    while !(*tp).waiting_worker_thread_list.is_null() {
      let mut next = (*(*tp).waiting_worker_thread_list).next;
      opj_free((*tp).waiting_worker_thread_list as *mut libc::c_void);
      (*tp).waiting_worker_thread_list = next
    }
    opj_cond_destroy((*tp).cond);
  }
  opj_mutex_destroy((*tp).mutex);
  opj_free(tp as *mut libc::c_void);
}
