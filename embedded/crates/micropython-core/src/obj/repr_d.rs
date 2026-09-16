use core::ffi::c_void;

use micropython_sys::{mp_const_obj_t, mp_int_t, mp_obj_t, mp_uint_t, qstr};

pub fn is_small_int(o: mp_const_obj_t) -> bool {
    (o & 0xffff_0000_0000_0000) == 0x0001_0000_0000_0000
}

pub fn small_int_value(o: mp_const_obj_t) -> mp_int_t {
    ((o << 16) as i64 >> 17) as mp_int_t
}

pub const fn new_small_int(v: mp_int_t) -> mp_obj_t {
    (((v as u64) & 0x7fff_ffff_ffff) << 1) | 0x0001_0000_0000_0001
}

pub fn is_qstr(o: mp_const_obj_t) -> bool {
    (o & 0xffff_0000_0000_0000) == 0x0002_0000_0000_0000
}

pub fn qstr_value(o: mp_const_obj_t) -> qstr {
    ((o as u32) >> 1) as qstr
}

pub const fn new_qstr(v: qstr) -> mp_obj_t {
    (((v as u32) as u64) << 1) | 0x0002_0000_0000_0001
}

pub fn is_immediate(o: mp_const_obj_t) -> bool {
    (o & 0xffff_0000_0000_0000) == 0x0003_0000_0000_0000
}

pub fn immediate_value(o: mp_const_obj_t) -> mp_uint_t {
    ((o >> 46) & 3) as mp_uint_t
}

pub const fn new_immediate(v: mp_uint_t) -> mp_obj_t {
    (v << 46) | 0x0003_0000_0000_0000
}

pub fn is_ptr(o: mp_const_obj_t) -> bool {
    (o & 0xffff_0000_0000_0000) == 0
}

// must cast through `usize` first, mirroring `uintptr_t` conversion in C

pub fn ptr_value(o: mp_obj_t) -> *mut c_void {
    o as usize as *mut c_void
}

pub const fn new_ptr(p: *mut c_void) -> mp_obj_t {
    p as usize as mp_obj_t
}
