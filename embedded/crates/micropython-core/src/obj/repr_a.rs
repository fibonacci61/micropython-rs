use micropython_sys::{mp_const_obj_t, mp_int_t, mp_obj_t, mp_uint_t, qstr};

pub fn is_small_int(o: mp_const_obj_t) -> bool {
    (o as mp_int_t & 1) != 0
}

pub fn small_int_value(o: mp_const_obj_t) -> mp_int_t {
    (o as mp_int_t) >> 1
}

pub const fn new_small_int(v: mp_int_t) -> mp_obj_t {
    (((v as mp_uint_t) << 1) | 1) as mp_obj_t
}

pub fn is_qstr(o: mp_const_obj_t) -> bool {
    (o as mp_int_t & 7) == 2
}

pub fn qstr_value(o: mp_const_obj_t) -> qstr {
    (o as mp_uint_t) >> 3
}

pub const fn new_qstr(v: qstr) -> mp_obj_t {
    (((v as mp_uint_t) << 3) | 2) as mp_obj_t
}

pub fn is_immediate(o: mp_const_obj_t) -> bool {
    (o as mp_int_t & 7) == 6
}

pub fn immediate_value(o: mp_const_obj_t) -> mp_uint_t {
    (o as mp_uint_t) >> 3
}

pub const fn new_immediate(v: mp_uint_t) -> mp_obj_t {
    ((v << 3) | 6) as mp_obj_t
}

pub fn is_ptr(o: mp_const_obj_t) -> bool {
    (o as mp_int_t & 3) == 0
}

pub use super::ptr::new_ptr;
pub use super::ptr::ptr_value;
