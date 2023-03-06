use super::openjpeg::*;
use super::event::*;
use ::libc;

use super::malloc::*;

#[no_mangle]
pub(crate) unsafe fn opj_procedure_list_create() -> *mut opj_procedure_list_t {
  /* memory allocation */
  let mut l_validation = opj_calloc(
    1i32 as size_t,
    core::mem::size_of::<opj_procedure_list_t>() as usize,
  ) as *mut opj_procedure_list_t;
  if l_validation.is_null() {
    return 0 as *mut opj_procedure_list_t;
  }
  /* initialization */
  (*l_validation).m_nb_max_procedures = 10 as OPJ_UINT32;
  (*l_validation).m_procedures = opj_calloc(
    10i32 as size_t,
    core::mem::size_of::<opj_procedure>() as usize,
  ) as *mut opj_procedure;
  if (*l_validation).m_procedures.is_null() {
    opj_free(l_validation as *mut libc::c_void);
    return 0 as *mut opj_procedure_list_t;
  }
  return l_validation;
}
#[no_mangle]
pub(crate) unsafe fn opj_procedure_list_destroy(mut p_list: *mut opj_procedure_list_t) {
  if p_list.is_null() {
    return;
  }
  /* initialization */
  if !(*p_list).m_procedures.is_null() {
    opj_free((*p_list).m_procedures as *mut libc::c_void);
  }
  opj_free(p_list as *mut libc::c_void);
}
#[no_mangle]
pub(crate) unsafe fn opj_procedure_list_add_procedure(
  mut p_validation_list: *mut opj_procedure_list_t,
  mut p_procedure: opj_procedure,
  mut p_manager: *mut opj_event_mgr,
) -> OPJ_BOOL {
  assert!(!p_manager.is_null());
  if (*p_validation_list).m_nb_max_procedures == (*p_validation_list).m_nb_procedures {
    let mut new_procedures = 0 as *mut opj_procedure;
    (*p_validation_list).m_nb_max_procedures =
      ((*p_validation_list).m_nb_max_procedures as libc::c_uint)
        .wrapping_add(10u32) as OPJ_UINT32;
    new_procedures = opj_realloc(
      (*p_validation_list).m_procedures as *mut libc::c_void,
      ((*p_validation_list).m_nb_max_procedures as usize)
        .wrapping_mul(core::mem::size_of::<opj_procedure>() as usize),
    ) as *mut opj_procedure;
    if new_procedures.is_null() {
      opj_free((*p_validation_list).m_procedures as *mut libc::c_void);
      (*p_validation_list).m_nb_max_procedures = 0 as OPJ_UINT32;
      (*p_validation_list).m_nb_procedures = 0 as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1i32,
        b"Not enough memory to add a new validation procedure\n\x00" as *const u8
          as *const libc::c_char,
      );
      return 0i32;
    } else {
      (*p_validation_list).m_procedures = new_procedures
    }
  }
  let ref mut fresh0 = *(*p_validation_list)
    .m_procedures
    .offset((*p_validation_list).m_nb_procedures as isize);
  *fresh0 = p_procedure;
  (*p_validation_list).m_nb_procedures = (*p_validation_list).m_nb_procedures.wrapping_add(1);
  return 1i32;
}
#[no_mangle]
pub(crate) unsafe fn opj_procedure_list_get_nb_procedures(
  mut p_validation_list: *mut opj_procedure_list_t,
) -> OPJ_UINT32 {
  return (*p_validation_list).m_nb_procedures;
}
#[no_mangle]
pub(crate) unsafe fn opj_procedure_list_get_first_procedure(
  mut p_validation_list: *mut opj_procedure_list_t,
) -> *mut opj_procedure {
  return (*p_validation_list).m_procedures;
}
#[no_mangle]
pub(crate) unsafe fn opj_procedure_list_clear(
  mut p_validation_list: *mut opj_procedure_list_t,
) {
  (*p_validation_list).m_nb_procedures = 0 as OPJ_UINT32;
}
