use ::libc;
extern "C" {
  #[no_mangle]
  fn __assert_fail(
    __assertion: *const libc::c_char,
    __file: *const libc::c_char,
    __line: libc::c_uint,
    __function: *const libc::c_char,
  ) -> !;
  #[no_mangle]
  fn opj_calloc(numOfElements: size_t, sizeOfElements: size_t) -> *mut libc::c_void;
  #[no_mangle]
  fn opj_realloc(m: *mut libc::c_void, s: size_t) -> *mut libc::c_void;
  #[no_mangle]
  fn opj_free(m: *mut libc::c_void);
  #[no_mangle]
  fn opj_event_msg(
    event_mgr: *mut opj_event_mgr_t,
    event_type: OPJ_INT32,
    fmt: *const libc::c_char,
    _: ...
  ) -> OPJ_BOOL;
}
pub type size_t = libc::c_ulong;
pub type __int32_t = libc::c_int;
pub type __uint32_t = libc::c_uint;
pub type OPJ_BOOL = libc::c_int;
pub type int32_t = __int32_t;
pub type uint32_t = __uint32_t;
pub type OPJ_INT32 = int32_t;
pub type OPJ_UINT32 = uint32_t;
pub type opj_msg_callback =
  Option<unsafe extern "C" fn(_: *const libc::c_char, _: *mut libc::c_void) -> ()>;
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
pub type opj_procedure = Option<unsafe extern "C" fn() -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_procedure_list {
  pub m_nb_procedures: OPJ_UINT32,
  pub m_nb_max_procedures: OPJ_UINT32,
  pub m_procedures: *mut opj_procedure,
}
pub type opj_procedure_list_t = opj_procedure_list;
#[no_mangle]
pub unsafe extern "C" fn opj_procedure_list_create() -> *mut opj_procedure_list_t {
  /* memory allocation */
  let mut l_validation = opj_calloc(
    1 as libc::c_int as size_t,
    ::std::mem::size_of::<opj_procedure_list_t>() as libc::c_ulong,
  ) as *mut opj_procedure_list_t;
  if l_validation.is_null() {
    return 0 as *mut opj_procedure_list_t;
  }
  /* initialization */
  (*l_validation).m_nb_max_procedures = 10 as libc::c_int as OPJ_UINT32;
  (*l_validation).m_procedures = opj_calloc(
    10 as libc::c_int as size_t,
    ::std::mem::size_of::<opj_procedure>() as libc::c_ulong,
  ) as *mut opj_procedure;
  if (*l_validation).m_procedures.is_null() {
    opj_free(l_validation as *mut libc::c_void);
    return 0 as *mut opj_procedure_list_t;
  }
  return l_validation;
}
#[no_mangle]
pub unsafe extern "C" fn opj_procedure_list_destroy(mut p_list: *mut opj_procedure_list_t) {
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
pub unsafe extern "C" fn opj_procedure_list_add_procedure(
  mut p_validation_list: *mut opj_procedure_list_t,
  mut p_procedure: opj_procedure,
  mut p_manager: *mut opj_event_mgr_t,
) -> OPJ_BOOL {
  if !p_manager.is_null() {
  } else {
    __assert_fail(b"p_manager != NULL\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/function_list.c\x00" as
                          *const u8 as *const libc::c_char,
                      74 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 100],
                                                &[libc::c_char; 100]>(b"OPJ_BOOL opj_procedure_list_add_procedure(opj_procedure_list_t *, opj_procedure, opj_event_mgr_t *)\x00")).as_ptr());
  }
  if (*p_validation_list).m_nb_max_procedures == (*p_validation_list).m_nb_procedures {
    let mut new_procedures = 0 as *mut opj_procedure;
    (*p_validation_list).m_nb_max_procedures =
      ((*p_validation_list).m_nb_max_procedures as libc::c_uint)
        .wrapping_add(10 as libc::c_int as libc::c_uint) as OPJ_UINT32 as OPJ_UINT32;
    new_procedures = opj_realloc(
      (*p_validation_list).m_procedures as *mut libc::c_void,
      ((*p_validation_list).m_nb_max_procedures as libc::c_ulong)
        .wrapping_mul(::std::mem::size_of::<opj_procedure>() as libc::c_ulong),
    ) as *mut opj_procedure;
    if new_procedures.is_null() {
      opj_free((*p_validation_list).m_procedures as *mut libc::c_void);
      (*p_validation_list).m_nb_max_procedures = 0 as libc::c_int as OPJ_UINT32;
      (*p_validation_list).m_nb_procedures = 0 as libc::c_int as OPJ_UINT32;
      opj_event_msg(
        p_manager,
        1 as libc::c_int,
        b"Not enough memory to add a new validation procedure\n\x00" as *const u8
          as *const libc::c_char,
      );
      return 0 as libc::c_int;
    } else {
      (*p_validation_list).m_procedures = new_procedures
    }
  }
  let ref mut fresh0 = *(*p_validation_list)
    .m_procedures
    .offset((*p_validation_list).m_nb_procedures as isize);
  *fresh0 = p_procedure;
  (*p_validation_list).m_nb_procedures = (*p_validation_list).m_nb_procedures.wrapping_add(1);
  return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_procedure_list_get_nb_procedures(
  mut p_validation_list: *mut opj_procedure_list_t,
) -> OPJ_UINT32 {
  return (*p_validation_list).m_nb_procedures;
}
#[no_mangle]
pub unsafe extern "C" fn opj_procedure_list_get_first_procedure(
  mut p_validation_list: *mut opj_procedure_list_t,
) -> *mut opj_procedure {
  return (*p_validation_list).m_procedures;
}
#[no_mangle]
pub unsafe extern "C" fn opj_procedure_list_clear(
  mut p_validation_list: *mut opj_procedure_list_t,
) {
  (*p_validation_list).m_nb_procedures = 0 as libc::c_int as OPJ_UINT32;
}
