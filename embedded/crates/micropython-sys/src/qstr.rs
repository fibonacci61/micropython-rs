//! Bindings for MicroPython's `py/qstr.h`.

use core::ffi::c_char;

use crate::misc::byte;
#[cfg(micropython = "MICROPY_ROM_TEXT_COMPRESSION")]
use crate::misc::mp_rom_error_text_t;
use crate::mpconfig::mp_uint_t;

include!(concat!(env!("OUT_DIR"), "/qstr_types.rs"));

pub type qstr = usize;
pub type qstr_short_t = u16;

pub const MP_QSTRnull: qstr = 0;

#[repr(C)]
pub struct qstr_pool_t {
    pub prev: *const qstr_pool_t,
    pub _bitfield_1: usize,
    pub alloc: usize,
    pub len: usize,
    #[cfg(micropython = "MICROPY_QSTR_BYTES_IN_HASH")]
    pub hashes: *mut qstr_hash_t,
    pub lengths: *mut qstr_len_t,
    pub qstrs: [*const c_char; 0],
}

impl qstr_pool_t {
    pub fn total_prev_len(&self) -> usize {
        #[cfg(target_endian = "little")]
        {
            self._bitfield_1 & (usize::MAX >> 1)
        }
        #[cfg(target_endian = "big")]
        {
            self._bitfield_1 >> 1
        }
    }

    pub fn is_sorted(&self) -> bool {
        #[cfg(target_endian = "little")]
        {
            self._bitfield_1 >> (usize::BITS - 1) != 0
        }
        #[cfg(target_endian = "big")]
        {
            self._bitfield_1 & 1 != 0
        }
    }

    pub fn set_total_prev_len(&mut self, value: usize) {
        assert!(value <= usize::MAX >> 1);
        #[cfg(target_endian = "little")]
        {
            self._bitfield_1 = (self._bitfield_1 & !(usize::MAX >> 1)) | value;
        }
        #[cfg(target_endian = "big")]
        {
            self._bitfield_1 = (self._bitfield_1 & 1) | (value << 1);
        }
    }

    pub fn set_is_sorted(&mut self, value: bool) {
        #[cfg(target_endian = "little")]
        let mask = 1usize << (usize::BITS - 1);
        #[cfg(target_endian = "big")]
        let mask = 1usize;
        self._bitfield_1 = (self._bitfield_1 & !mask) | if value { mask } else { 0 };
    }
}

unsafe extern "C" {
    pub fn qstr_init();

    pub fn qstr_compute_hash(data: *const byte, len: usize) -> usize;
    pub fn qstr_find_strn(str_: *const c_char, str_len: usize) -> qstr;
    pub fn qstr_from_str(str_: *const c_char) -> qstr;
    pub fn qstr_from_strn(str_: *const c_char, len: usize) -> qstr;
    #[cfg(micropython = "MICROPY_VFS_ROM")]
    pub fn qstr_from_strn_static(str_: *const c_char, len: usize) -> qstr;

    pub fn qstr_hash(q: qstr) -> mp_uint_t;
    pub fn qstr_str(q: qstr) -> *const c_char;
    pub fn qstr_len(q: qstr) -> usize;
    pub fn qstr_data(q: qstr, len: *mut usize) -> *const byte;

    pub fn qstr_pool_info(
        n_pool: *mut usize,
        n_qstr: *mut usize,
        n_str_data_bytes: *mut usize,
        n_total_bytes: *mut usize,
    );
    pub fn qstr_dump_data();

    #[cfg(micropython = "MICROPY_ROM_TEXT_COMPRESSION")]
    pub fn mp_decompress_rom_string(dst: *mut byte, src: mp_rom_error_text_t);
}
