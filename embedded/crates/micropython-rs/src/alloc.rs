use core::ffi::c_void;

use crate::{except::RstResult, gc::Gc, obj::Restricted, shims, vm::MicroPython};

pub fn m_malloc<'gc>(
    _mp: &mut MicroPython,
    gc: &'gc mut Gc,
    size: usize,
) -> RstResult<'gc, *mut c_void> {
    let result = unsafe { shims::mprs_nlrshim_m_malloc(size) };
    if !result.ok {
        Err(unsafe { Restricted::from_raw(result.exception, gc) })
    } else {
        Ok(result.value)
    }
}
