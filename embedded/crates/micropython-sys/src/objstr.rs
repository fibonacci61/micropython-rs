//! Bindings for MicroPython `py/objstr.h`.

#![allow(clippy::missing_safety_doc)]

use core::ffi::c_int;

use crate::misc::byte;
use crate::mpconfig::{mp_int_t, mp_uint_t};
#[cfg(micropython = "MICROPY_PY_JSON")]
use crate::mpprint::mp_print_t;
use crate::obj::{
    mp_buffer_info_t, mp_map_t, mp_obj_base_t, mp_obj_dict_t, mp_obj_t, mp_obj_type_t, size_t,
};
use crate::runtime0::mp_binary_op_t;

#[repr(C)]
pub struct mp_obj_str_t {
    pub base: mp_obj_base_t,
    pub hash: size_t,
    pub len: size_t,
    pub data: *const byte,
}

// A few declarations are unconditional in objstr.h even though their C symbols
// are only built with the corresponding feature enabled.
unsafe extern "C" {
    #[cfg(any(micropython = "MICROPY_OBJ_REPR_C", micropython = "MICROPY_OBJ_REPR_D"))]
    pub fn mp_obj_str_get_data_no_check(self_in: mp_obj_t, len: *mut size_t) -> *const byte;

    pub fn mp_obj_str_make_new(
        type_in: *const mp_obj_type_t,
        n_args: size_t,
        n_kw: size_t,
        args: *const mp_obj_t,
    ) -> mp_obj_t;
    #[cfg(micropython = "MICROPY_PY_JSON")]
    pub fn mp_str_print_json(print: *const mp_print_t, str_data: *const byte, str_len: size_t);
    pub fn mp_obj_str_format(
        n_args: size_t,
        args: *const mp_obj_t,
        kwargs: *mut mp_map_t,
    ) -> mp_obj_t;
    pub fn mp_obj_str_split(n_args: size_t, args: *const mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_new_str_copy(
        type_: *const mp_obj_type_t,
        data: *const byte,
        len: size_t,
    ) -> mp_obj_t;
    pub fn mp_obj_new_str_of_type(
        type_: *const mp_obj_type_t,
        data: *const byte,
        len: size_t,
    ) -> mp_obj_t;

    pub fn mp_obj_str_binary_op(op: mp_binary_op_t, lhs_in: mp_obj_t, rhs_in: mp_obj_t)
    -> mp_obj_t;
    pub fn mp_obj_str_get_buffer(
        self_in: mp_obj_t,
        bufinfo: *mut mp_buffer_info_t,
        flags: mp_uint_t,
    ) -> mp_int_t;

    pub fn mp_obj_str_set_data(str_: *mut mp_obj_str_t, data: *const byte, len: size_t);

    pub fn str_index_to_ptr(
        type_: *const mp_obj_type_t,
        self_data: *const byte,
        self_len: size_t,
        index: mp_obj_t,
        is_slice: bool,
    ) -> *const byte;
    pub fn find_subbytes(
        haystack: *const byte,
        hlen: size_t,
        needle: *const byte,
        nlen: size_t,
        direction: c_int,
    ) -> *const byte;

    #[cfg(micropython = "MICROPY_PY_BUILTINS_BYTES_HEX")]
    pub fn mp_obj_bytes_hex(
        n_args: size_t,
        args: *const mp_obj_t,
        type_: *const mp_obj_type_t,
    ) -> mp_obj_t;
    #[cfg(micropython = "MICROPY_PY_BUILTINS_BYTES_HEX")]
    pub fn mp_obj_bytes_fromhex(type_in: mp_obj_t, data: mp_obj_t) -> mp_obj_t;

    pub static mp_obj_str_locals_dict: mp_obj_dict_t;

    #[cfg(all(
        micropython = "MICROPY_PY_BUILTINS_MEMORYVIEW",
        micropython = "MICROPY_PY_BUILTINS_BYTES_HEX"
    ))]
    pub static mp_obj_memoryview_locals_dict: mp_obj_dict_t;

    #[cfg(micropython = "MICROPY_PY_BUILTINS_BYTEARRAY")]
    pub static mp_obj_bytearray_locals_dict: mp_obj_dict_t;

    #[cfg(micropython = "MICROPY_PY_ARRAY")]
    pub static mp_obj_array_locals_dict: mp_obj_dict_t;
}
