use core::ffi::c_void;

use crate::{except::RstResult, obj::Restricted, shims, vm::MicroPython};

pub fn m_malloc<'py>(mp: &'py mut MicroPython, size: usize) -> RstResult<'py, *mut c_void> {
    let result = unsafe { shims::mprs_nlrshim_m_malloc(size) };
    if !result.ok {
        Err(unsafe { Restricted::from_raw(result.exception, mp) })
    } else {
        Ok(result.value)
    }
}
