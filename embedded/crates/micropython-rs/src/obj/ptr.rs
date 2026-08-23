//! Direct `mp_obj_t` <-> pointer conversions, valid for repr A, B, and C

use core::ffi::c_void;

use micropython_sys::mp_obj_t;

pub fn ptr_value(o: mp_obj_t) -> *mut c_void {
    o as *mut c_void
}

pub const fn new_ptr(p: *mut c_void) -> mp_obj_t {
    p as mp_obj_t
}
