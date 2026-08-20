//! Foundational ABI types from MicroPython's `py/misc.h`.

use core::ffi::c_char;

pub type byte = u8;

#[repr(C)]
pub struct vstr_t {
    pub alloc: usize,
    pub len: usize,
    pub buf: *mut c_char,
    pub fixed_buf: bool,
}

#[cfg(micropython = "MICROPY_ROM_TEXT_COMPRESSION")]
#[repr(C)]
pub struct mp_rom_error_text_opaque_t {
    _private: [u8; 0],
}

#[cfg(micropython = "MICROPY_ROM_TEXT_COMPRESSION")]
pub type mp_rom_error_text_t = *mut mp_rom_error_text_opaque_t;

#[cfg(not(micropython = "MICROPY_ROM_TEXT_COMPRESSION"))]
pub type mp_rom_error_text_t = *const c_char;
