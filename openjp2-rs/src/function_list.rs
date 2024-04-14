use super::event::*;
use super::openjpeg::*;

#[derive(Clone)]
pub(crate) struct ProcedureList<P> {
  list: Vec<P>,
}

impl<P> ProcedureList<P> {
  pub fn new() -> Self {
    Self {
      list: Default::default(),
    }
  }

  pub fn add(&mut self, procedure: P) {
    self.list.push(procedure)
  }

  pub fn execute<F: FnMut(P) -> bool>(&mut self, mut eval: F) -> bool {
    for p in self.list.drain(..) {
      if !eval(p) {
        return false;
      }
    }
    true
  }
}

pub(crate) unsafe fn opj_procedure_list_create<P>() -> *mut ProcedureList<P> {
  /* memory allocation */
  let mut l_validation = Box::new(ProcedureList { list: Vec::new() });
  Box::into_raw(l_validation)
}

pub(crate) unsafe fn opj_procedure_list_destroy<P>(mut p_list: *mut ProcedureList<P>) {
  if p_list.is_null() {
    return;
  }
  let _ = Box::from_raw(p_list);
}

pub(crate) unsafe fn opj_procedure_list_add_procedure<P>(
  mut p_validation_list: *mut ProcedureList<P>,
  mut p_procedure: P,
  mut _p_manager: &mut opj_event_mgr,
) -> OPJ_BOOL {
  (*p_validation_list).list.push(p_procedure);
  1i32
}

pub(crate) unsafe fn opj_procedure_list_get_nb_procedures<P>(
  mut p_validation_list: *mut ProcedureList<P>,
) -> OPJ_UINT32 {
  (*p_validation_list).list.len() as u32
}

pub(crate) unsafe fn opj_procedure_list_get_first_procedure<P>(
  mut p_validation_list: *mut ProcedureList<P>,
) -> *mut P {
  (*p_validation_list).list.as_mut_ptr()
}

pub(crate) unsafe fn opj_procedure_list_clear<P>(mut p_validation_list: *mut ProcedureList<P>) {
  (*p_validation_list).list.clear();
}

pub(crate) fn opj_procedure_list_execute<P, F: FnMut(P) -> bool>(
  mut p_validation_list: *mut ProcedureList<P>,
  mut eval: F,
) -> bool {
  let mut p_list = unsafe { &mut *p_validation_list };
  p_list.execute(eval)
}
