//! Bindings for MicroPython's `py/mpprint.h`.

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

#[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
use crate::mpconfig::mp_float_t;

pub const PF_FLAG_LEFT_ADJUST: c_uint = 0x001;
pub const PF_FLAG_SHOW_SIGN: c_uint = 0x002;
pub const PF_FLAG_SPACE_SIGN: c_uint = 0x004;
pub const PF_FLAG_SHOW_PREFIX: c_uint = 0x008;
pub const PF_FLAG_PAD_AFTER_SIGN: c_uint = 0x010;
pub const PF_FLAG_CENTER_ADJUST: c_uint = 0x020;
pub const PF_FLAG_ADD_PERCENT: c_uint = 0x040;
pub const PF_FLAG_SHOW_OCTAL_LETTER: c_uint = 0x080;
pub const PF_FLAG_ALWAYS_DECIMAL: c_uint = 0x100;
pub const PF_FLAG_SEP_POS: c_uint = 9;

pub type mp_print_strn_t =
    Option<unsafe extern "C" fn(data: *mut c_void, str_: *const c_char, len: usize)>;

#[repr(C)]
pub struct mp_print_t {
    pub data: *mut c_void,
    pub print_strn: mp_print_strn_t,
}

#[repr(C)]
pub struct mp_print_ext_t {
    pub base: mp_print_t,
    pub item_separator: *const c_char,
    pub key_separator: *const c_char,
}

#[inline]
pub fn mp_print_get_ext(print: *const mp_print_t) -> *const mp_print_ext_t {
    print.cast()
}

#[inline]
pub fn mp_print_get_ext_mut(print: *mut mp_print_t) -> *mut mp_print_ext_t {
    print.cast()
}

#[inline]
pub fn mp_python_printer() -> *const mp_print_t {
    #[cfg(all(micropython = "MICROPY_PY_IO", micropython = "MICROPY_PY_SYS_STDFILES"))]
    {
        ptr::addr_of!(mp_sys_stdout_print)
    }
    #[cfg(not(all(micropython = "MICROPY_PY_IO", micropython = "MICROPY_PY_SYS_STDFILES")))]
    {
        ptr::addr_of!(mp_plat_print)
    }
}

unsafe extern "C" {
    pub static mp_plat_print: mp_print_t;
    #[cfg(all(micropython = "MICROPY_PY_IO", micropython = "MICROPY_PY_SYS_STDFILES"))]
    pub static mp_sys_stdout_print: mp_print_t;

    pub fn mp_print_str(print: *const mp_print_t, str_: *const c_char) -> c_int;
    pub fn mp_print_strn(
        print: *const mp_print_t,
        str_: *const c_char,
        len: usize,
        flags: c_uint,
        fill: c_char,
        width: c_int,
    ) -> c_int;
    #[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
    pub fn mp_print_float(
        print: *const mp_print_t,
        value: mp_float_t,
        fmt: c_char,
        flags: c_uint,
        fill: c_char,
        width: c_int,
        precision: c_int,
    ) -> c_int;

    pub fn mp_printf(print: *const mp_print_t, fmt: *const c_char, ...) -> c_int;

    // `mp_vprintf` is conditionally declared by C only when `va_start` is
    // already visible. Rust has no stable, target-independent `va_list` ABI.
}
