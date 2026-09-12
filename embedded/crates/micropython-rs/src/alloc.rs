use core::ffi::c_void;

use crate::{except::RstResult, obj::Restricted, shims, vm::Gc, vm::MicroPythonMut};

pub fn m_malloc<'gc>(
    // needed because finalisers may be triggered by the allocation call
    _mp: MicroPythonMut<'_>,
    gc: Gc<'gc>,
    size: usize,
) -> RstResult<'gc, *mut c_void> {
    let result = unsafe { shims::mprs_nlrshim_m_malloc(size) };
    if !result.ok {
        Err(unsafe { Restricted::from_raw(result.exception, gc) })
    } else {
        Ok(result.value)
    }
}
