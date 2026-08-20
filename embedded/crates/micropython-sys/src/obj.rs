//! Bindings for MicroPython `py/obj.h`.

// C preprocessor macros which only perform object tagging are exposed as
// `const fn`s.  Macros which allocate, assert, or depend on a C lvalue remain
// functions on the C side and are not reproduced here.

#![allow(clippy::missing_safety_doc, clippy::ptr_eq)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

pub type size_t = usize;
pub type byte = u8;
pub type qstr = usize;
#[cfg(micropython = "MP_INT_TYPE_INTPTR")]
pub type mp_int_t = isize;
#[cfg(micropython = "MP_INT_TYPE_INTPTR")]
pub type mp_uint_t = usize;
#[cfg(micropython = "MP_INT_TYPE_INT64")]
pub type mp_int_t = i64;
#[cfg(micropython = "MP_INT_TYPE_INT64")]
pub type mp_uint_t = u64;
#[cfg(micropython = "MP_INT_TYPE_OTHER")]
compile_error!("micropython-sys does not yet support port-defined MP_INT_TYPE_OTHER integer types");

#[cfg(micropython = "MICROPY_FLOAT_IMPL_FLOAT")]
pub type mp_float_t = f32;
#[cfg(micropython = "MICROPY_FLOAT_IMPL_DOUBLE")]
pub type mp_float_t = f64;

#[cfg(micropython = "MICROPY_OBJ_REPR_D")]
pub type mp_obj_t = u64;
#[cfg(micropython = "MICROPY_OBJ_REPR_D")]
pub type mp_const_obj_t = u64;
#[cfg(not(micropython = "MICROPY_OBJ_REPR_D"))]
pub type mp_obj_t = *mut c_void;
#[cfg(not(micropython = "MICROPY_OBJ_REPR_D"))]
pub type mp_const_obj_t = *const c_void;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct mp_rom_obj_d_ptrs_t {
    pub lo: *const c_void,
    pub hi: *const c_void,
}

#[cfg(micropython = "MICROPY_OBJ_REPR_D")]
#[repr(C)]
#[derive(Copy, Clone)]
pub union mp_rom_obj_t {
    pub u64_: u64,
    pub u32_: mp_rom_obj_d_ptrs_t,
}
#[cfg(not(micropython = "MICROPY_OBJ_REPR_D"))]
pub type mp_rom_obj_t = mp_const_obj_t;

#[repr(C)]
pub struct mp_print_t {
    pub data: *mut c_void,
    pub print_strn: mp_print_strn_t,
}
pub type mp_print_strn_t = Option<unsafe extern "C" fn(*mut c_void, *const c_char, size_t)>;

// `runtime0.h` enum types used by obj.h callbacks.
pub type mp_unary_op_t = c_uint;
pub const MP_UNARY_OP_POSITIVE: mp_unary_op_t = 0;
pub const MP_UNARY_OP_NEGATIVE: mp_unary_op_t = 1;
pub const MP_UNARY_OP_INVERT: mp_unary_op_t = 2;
pub const MP_UNARY_OP_NOT: mp_unary_op_t = 3;
pub const MP_UNARY_OP_BOOL: mp_unary_op_t = 4;
pub const MP_UNARY_OP_LEN: mp_unary_op_t = 5;
pub const MP_UNARY_OP_HASH: mp_unary_op_t = 6;
pub const MP_UNARY_OP_ABS: mp_unary_op_t = 7;
pub const MP_UNARY_OP_INT_MAYBE: mp_unary_op_t = 8;
pub const MP_UNARY_OP_FLOAT_MAYBE: mp_unary_op_t = 9;
pub const MP_UNARY_OP_COMPLEX_MAYBE: mp_unary_op_t = 10;
pub const MP_UNARY_OP_SIZEOF: mp_unary_op_t = 11;

pub type mp_binary_op_t = c_uint;
pub const MP_BINARY_OP_LESS: mp_binary_op_t = 0;
pub const MP_BINARY_OP_MORE: mp_binary_op_t = 1;
pub const MP_BINARY_OP_EQUAL: mp_binary_op_t = 2;
pub const MP_BINARY_OP_LESS_EQUAL: mp_binary_op_t = 3;
pub const MP_BINARY_OP_MORE_EQUAL: mp_binary_op_t = 4;
pub const MP_BINARY_OP_NOT_EQUAL: mp_binary_op_t = 5;
pub const MP_BINARY_OP_IN: mp_binary_op_t = 6;
pub const MP_BINARY_OP_IS: mp_binary_op_t = 7;
pub const MP_BINARY_OP_EXCEPTION_MATCH: mp_binary_op_t = 8;
pub const MP_BINARY_OP_INPLACE_OR: mp_binary_op_t = 9;
pub const MP_BINARY_OP_INPLACE_XOR: mp_binary_op_t = 10;
pub const MP_BINARY_OP_INPLACE_AND: mp_binary_op_t = 11;
pub const MP_BINARY_OP_INPLACE_LSHIFT: mp_binary_op_t = 12;
pub const MP_BINARY_OP_INPLACE_RSHIFT: mp_binary_op_t = 13;
pub const MP_BINARY_OP_INPLACE_ADD: mp_binary_op_t = 14;
pub const MP_BINARY_OP_INPLACE_SUBTRACT: mp_binary_op_t = 15;
pub const MP_BINARY_OP_INPLACE_MULTIPLY: mp_binary_op_t = 16;
pub const MP_BINARY_OP_INPLACE_MAT_MULTIPLY: mp_binary_op_t = 17;
pub const MP_BINARY_OP_INPLACE_FLOOR_DIVIDE: mp_binary_op_t = 18;
pub const MP_BINARY_OP_INPLACE_TRUE_DIVIDE: mp_binary_op_t = 19;
pub const MP_BINARY_OP_INPLACE_MODULO: mp_binary_op_t = 20;
pub const MP_BINARY_OP_INPLACE_POWER: mp_binary_op_t = 21;
pub const MP_BINARY_OP_OR: mp_binary_op_t = 22;
pub const MP_BINARY_OP_XOR: mp_binary_op_t = 23;
pub const MP_BINARY_OP_AND: mp_binary_op_t = 24;
pub const MP_BINARY_OP_LSHIFT: mp_binary_op_t = 25;
pub const MP_BINARY_OP_RSHIFT: mp_binary_op_t = 26;
pub const MP_BINARY_OP_ADD: mp_binary_op_t = 27;
pub const MP_BINARY_OP_SUBTRACT: mp_binary_op_t = 28;
pub const MP_BINARY_OP_MULTIPLY: mp_binary_op_t = 29;
pub const MP_BINARY_OP_MAT_MULTIPLY: mp_binary_op_t = 30;
pub const MP_BINARY_OP_FLOOR_DIVIDE: mp_binary_op_t = 31;
pub const MP_BINARY_OP_TRUE_DIVIDE: mp_binary_op_t = 32;
pub const MP_BINARY_OP_MODULO: mp_binary_op_t = 33;
pub const MP_BINARY_OP_POWER: mp_binary_op_t = 34;
pub const MP_BINARY_OP_DIVMOD: mp_binary_op_t = 35;
pub const MP_BINARY_OP_CONTAINS: mp_binary_op_t = 36;
pub const MP_BINARY_OP_REVERSE_OR: mp_binary_op_t = 37;
pub const MP_BINARY_OP_REVERSE_XOR: mp_binary_op_t = 38;
pub const MP_BINARY_OP_REVERSE_AND: mp_binary_op_t = 39;
pub const MP_BINARY_OP_REVERSE_LSHIFT: mp_binary_op_t = 40;
pub const MP_BINARY_OP_REVERSE_RSHIFT: mp_binary_op_t = 41;
pub const MP_BINARY_OP_REVERSE_ADD: mp_binary_op_t = 42;
pub const MP_BINARY_OP_REVERSE_SUBTRACT: mp_binary_op_t = 43;
pub const MP_BINARY_OP_REVERSE_MULTIPLY: mp_binary_op_t = 44;
pub const MP_BINARY_OP_REVERSE_MAT_MULTIPLY: mp_binary_op_t = 45;
pub const MP_BINARY_OP_REVERSE_FLOOR_DIVIDE: mp_binary_op_t = 46;
pub const MP_BINARY_OP_REVERSE_TRUE_DIVIDE: mp_binary_op_t = 47;
pub const MP_BINARY_OP_REVERSE_MODULO: mp_binary_op_t = 48;
pub const MP_BINARY_OP_REVERSE_POWER: mp_binary_op_t = 49;
pub const MP_BINARY_OP_NOT_IN: mp_binary_op_t = 50;
pub const MP_BINARY_OP_IS_NOT: mp_binary_op_t = 51;

#[repr(C)]
pub struct mp_obj_base_t {
    pub type_: *const mp_obj_type_t,
}

#[inline]
pub fn mp_obj_to_ptr(o: mp_obj_t) -> *mut c_void {
    #[cfg(micropython = "MICROPY_OBJ_REPR_D")]
    {
        o as usize as *mut c_void
    }
    #[cfg(not(micropython = "MICROPY_OBJ_REPR_D"))]
    {
        o
    }
}

#[inline]
pub fn mp_const_obj_to_ptr(o: mp_const_obj_t) -> *const c_void {
    #[cfg(micropython = "MICROPY_OBJ_REPR_D")]
    {
        o as usize as *const c_void
    }
    #[cfg(not(micropython = "MICROPY_OBJ_REPR_D"))]
    {
        o
    }
}

#[inline]
pub fn mp_obj_from_ptr(p: *mut c_void) -> mp_obj_t {
    #[cfg(micropython = "MICROPY_OBJ_REPR_D")]
    {
        p as usize as u64
    }
    #[cfg(not(micropython = "MICROPY_OBJ_REPR_D"))]
    {
        p
    }
}

#[cfg(micropython = "MICROPY_OBJ_REPR_D")]
pub const MP_OBJ_NULL: mp_obj_t = 0;
#[cfg(not(micropython = "MICROPY_OBJ_REPR_D"))]
pub const MP_OBJ_NULL: mp_obj_t = ptr::null_mut();

#[cfg(all(
    micropython = "MICROPY_DEBUG_MP_OBJ_SENTINELS",
    micropython = "MICROPY_OBJ_REPR_D"
))]
pub const MP_OBJ_STOP_ITERATION: mp_obj_t = 4;
#[cfg(all(
    micropython = "MICROPY_DEBUG_MP_OBJ_SENTINELS",
    micropython = "MICROPY_OBJ_REPR_D"
))]
pub const MP_OBJ_SENTINEL: mp_obj_t = 8;
#[cfg(all(
    not(micropython = "MICROPY_DEBUG_MP_OBJ_SENTINELS"),
    micropython = "MICROPY_OBJ_REPR_D"
))]
pub const MP_OBJ_STOP_ITERATION: mp_obj_t = 0;
#[cfg(all(
    not(micropython = "MICROPY_DEBUG_MP_OBJ_SENTINELS"),
    micropython = "MICROPY_OBJ_REPR_D"
))]
pub const MP_OBJ_SENTINEL: mp_obj_t = 4;
#[cfg(all(
    micropython = "MICROPY_DEBUG_MP_OBJ_SENTINELS",
    not(micropython = "MICROPY_OBJ_REPR_D")
))]
pub const MP_OBJ_STOP_ITERATION: mp_obj_t = 4usize as *mut c_void;
#[cfg(all(
    micropython = "MICROPY_DEBUG_MP_OBJ_SENTINELS",
    not(micropython = "MICROPY_OBJ_REPR_D")
))]
pub const MP_OBJ_SENTINEL: mp_obj_t = 8usize as *mut c_void;
#[cfg(all(
    not(micropython = "MICROPY_DEBUG_MP_OBJ_SENTINELS"),
    not(micropython = "MICROPY_OBJ_REPR_D")
))]
pub const MP_OBJ_STOP_ITERATION: mp_obj_t = ptr::null_mut();
#[cfg(all(
    not(micropython = "MICROPY_DEBUG_MP_OBJ_SENTINELS"),
    not(micropython = "MICROPY_OBJ_REPR_D")
))]
pub const MP_OBJ_SENTINEL: mp_obj_t = 4usize as *mut c_void;

#[cfg(micropython = "MICROPY_OBJ_REPR_A")]
mod tagging {
    use super::*;
    pub fn is_small_int(o: mp_const_obj_t) -> bool {
        (o as mp_int_t & 1) != 0
    }
    pub fn small_int_value(o: mp_const_obj_t) -> mp_int_t {
        (o as mp_int_t) >> 1
    }
    pub fn new_small_int(v: mp_int_t) -> mp_obj_t {
        (((v as mp_uint_t) << 1) | 1) as mp_obj_t
    }
    pub fn is_qstr(o: mp_const_obj_t) -> bool {
        (o as mp_int_t & 7) == 2
    }
    pub fn qstr_value(o: mp_const_obj_t) -> qstr {
        (o as mp_uint_t) >> 3
    }
    pub fn new_qstr(v: qstr) -> mp_obj_t {
        (((v as mp_uint_t) << 3) | 2) as mp_obj_t
    }
    pub fn is_immediate(o: mp_const_obj_t) -> bool {
        (o as mp_int_t & 7) == 6
    }
    pub fn immediate_value(o: mp_const_obj_t) -> mp_uint_t {
        (o as mp_uint_t) >> 3
    }
    pub fn new_immediate(v: mp_uint_t) -> mp_obj_t {
        ((v << 3) | 6) as mp_obj_t
    }
    pub fn is_obj(o: mp_const_obj_t) -> bool {
        (o as mp_int_t & 3) == 0
    }
}
#[cfg(micropython = "MICROPY_OBJ_REPR_B")]
mod tagging {
    use super::*;
    pub fn is_small_int(o: mp_const_obj_t) -> bool {
        (o as mp_int_t & 3) == 1
    }
    pub fn small_int_value(o: mp_const_obj_t) -> mp_int_t {
        (o as mp_int_t) >> 2
    }
    pub fn new_small_int(v: mp_int_t) -> mp_obj_t {
        (((v as mp_uint_t) << 2) | 1) as mp_obj_t
    }
    pub fn is_qstr(o: mp_const_obj_t) -> bool {
        (o as mp_int_t & 7) == 3
    }
    pub fn qstr_value(o: mp_const_obj_t) -> qstr {
        (o as mp_uint_t) >> 3
    }
    pub fn new_qstr(v: qstr) -> mp_obj_t {
        (((v as mp_uint_t) << 3) | 3) as mp_obj_t
    }
    pub fn is_immediate(o: mp_const_obj_t) -> bool {
        (o as mp_int_t & 7) == 7
    }
    pub fn immediate_value(o: mp_const_obj_t) -> mp_uint_t {
        (o as mp_uint_t) >> 3
    }
    pub fn new_immediate(v: mp_uint_t) -> mp_obj_t {
        ((v << 3) | 7) as mp_obj_t
    }
    pub fn is_obj(o: mp_const_obj_t) -> bool {
        (o as mp_int_t & 1) == 0
    }
}
#[cfg(micropython = "MICROPY_OBJ_REPR_C")]
mod tagging {
    use super::*;
    pub fn is_small_int(o: mp_const_obj_t) -> bool {
        (o as mp_int_t & 1) != 0
    }
    pub fn small_int_value(o: mp_const_obj_t) -> mp_int_t {
        (o as mp_int_t) >> 1
    }
    pub fn new_small_int(v: mp_int_t) -> mp_obj_t {
        (((v as mp_uint_t) << 1) | 1) as mp_obj_t
    }
    pub fn is_qstr(o: mp_const_obj_t) -> bool {
        (o as mp_uint_t & 0xff80_000f) == 6
    }
    pub fn qstr_value(o: mp_const_obj_t) -> qstr {
        (o as mp_uint_t) >> 4
    }
    pub fn new_qstr(v: qstr) -> mp_obj_t {
        (((v as mp_uint_t) << 4) | 6) as mp_obj_t
    }
    pub fn is_immediate(o: mp_const_obj_t) -> bool {
        (o as mp_uint_t & 0xff80_000f) == 0xe
    }
    pub fn immediate_value(o: mp_const_obj_t) -> mp_uint_t {
        (o as mp_uint_t) >> 4
    }
    pub fn new_immediate(v: mp_uint_t) -> mp_obj_t {
        ((v << 4) | 0xe) as mp_obj_t
    }
    pub fn is_obj(o: mp_const_obj_t) -> bool {
        (o as mp_int_t & 3) == 0
    }
}
#[cfg(micropython = "MICROPY_OBJ_REPR_D")]
mod tagging {
    use super::*;
    pub fn is_small_int(o: mp_const_obj_t) -> bool {
        (o & 0xffff_0000_0000_0000) == 0x0001_0000_0000_0000
    }
    pub fn small_int_value(o: mp_const_obj_t) -> mp_int_t {
        ((o << 16) as i64 >> 17) as mp_int_t
    }
    pub fn new_small_int(v: mp_int_t) -> mp_obj_t {
        (((v as u64) & 0x7fff_ffff_ffff) << 1) | 0x0001_0000_0000_0001
    }
    pub fn is_qstr(o: mp_const_obj_t) -> bool {
        (o & 0xffff_0000_0000_0000) == 0x0002_0000_0000_0000
    }
    pub fn qstr_value(o: mp_const_obj_t) -> qstr {
        ((o as u32) >> 1) as qstr
    }
    pub fn new_qstr(v: qstr) -> mp_obj_t {
        (((v as u32) as u64) << 1) | 0x0002_0000_0000_0001
    }
    pub fn is_immediate(o: mp_const_obj_t) -> bool {
        (o & 0xffff_0000_0000_0000) == 0x0003_0000_0000_0000
    }
    pub fn immediate_value(o: mp_const_obj_t) -> mp_uint_t {
        ((o >> 46) & 3) as mp_uint_t
    }
    pub fn new_immediate(v: mp_uint_t) -> mp_obj_t {
        (v << 46) | 0x0003_0000_0000_0000
    }
    pub fn is_obj(o: mp_const_obj_t) -> bool {
        (o & 0xffff_0000_0000_0000) == 0
    }
}
pub use tagging::{
    immediate_value as mp_obj_immediate_obj_value, is_immediate as mp_obj_is_immediate_obj,
    is_obj as mp_obj_is_obj, is_qstr as mp_obj_is_qstr, is_small_int as mp_obj_is_small_int,
    new_immediate as mp_obj_new_immediate_obj, new_qstr as mp_obj_new_qstr,
    new_small_int as mp_obj_new_small_int, qstr_value as mp_obj_qstr_value,
    small_int_value as mp_obj_small_int_value,
};

#[repr(C)]
pub struct mp_map_elem_t {
    pub key: mp_obj_t,
    pub value: mp_obj_t,
}
#[repr(C)]
pub struct mp_rom_map_elem_t {
    pub key: mp_rom_obj_t,
    pub value: mp_rom_obj_t,
}

/// The first word is the storage unit for obj.h's three `size_t` bitfields and
/// its `used` field. Use the accessors rather than depending on bitfield syntax.
#[repr(C)]
pub struct mp_map_t {
    pub _bitfield_1: size_t,
    pub alloc: size_t,
    pub table: *mut mp_map_elem_t,
}
impl mp_map_t {
    #[cfg(target_endian = "little")]
    const FLAGS_MASK: usize = 7;
    #[cfg(target_endian = "big")]
    const FLAGS_MASK: usize = 7usize << (usize::BITS - 3);

    pub fn all_keys_are_qstrs(&self) -> bool {
        #[cfg(target_endian = "little")]
        {
            self._bitfield_1 & 1 != 0
        }
        #[cfg(target_endian = "big")]
        {
            self._bitfield_1 & (1usize << (usize::BITS - 1)) != 0
        }
    }
    pub fn is_fixed(&self) -> bool {
        #[cfg(target_endian = "little")]
        {
            self._bitfield_1 & 2 != 0
        }
        #[cfg(target_endian = "big")]
        {
            self._bitfield_1 & (1usize << (usize::BITS - 2)) != 0
        }
    }
    pub fn is_ordered(&self) -> bool {
        #[cfg(target_endian = "little")]
        {
            self._bitfield_1 & 4 != 0
        }
        #[cfg(target_endian = "big")]
        {
            self._bitfield_1 & (1usize << (usize::BITS - 3)) != 0
        }
    }
    pub fn used(&self) -> size_t {
        #[cfg(target_endian = "little")]
        {
            self._bitfield_1 >> 3
        }
        #[cfg(target_endian = "big")]
        {
            self._bitfield_1 & (usize::MAX >> 3)
        }
    }
    pub fn set_all_keys_are_qstrs(&mut self, value: bool) {
        #[cfg(target_endian = "little")]
        let mask = 1;
        #[cfg(target_endian = "big")]
        let mask = 1usize << (usize::BITS - 1);
        self._bitfield_1 = (self._bitfield_1 & !mask) | if value { mask } else { 0 };
    }
    pub fn set_is_fixed(&mut self, value: bool) {
        #[cfg(target_endian = "little")]
        let mask = 2;
        #[cfg(target_endian = "big")]
        let mask = 1usize << (usize::BITS - 2);
        self._bitfield_1 = (self._bitfield_1 & !mask) | if value { mask } else { 0 };
    }
    pub fn set_is_ordered(&mut self, value: bool) {
        #[cfg(target_endian = "little")]
        let mask = 4;
        #[cfg(target_endian = "big")]
        let mask = 1usize << (usize::BITS - 3);
        self._bitfield_1 = (self._bitfield_1 & !mask) | if value { mask } else { 0 };
    }
    pub fn set_used(&mut self, value: size_t) {
        #[cfg(target_endian = "little")]
        let encoded = value << 3;
        #[cfg(target_endian = "big")]
        let encoded = value;
        self._bitfield_1 = (self._bitfield_1 & Self::FLAGS_MASK) | encoded;
    }
}

pub type mp_map_lookup_kind_t = c_uint;
pub const MP_MAP_LOOKUP: mp_map_lookup_kind_t = 0;
pub const MP_MAP_LOOKUP_ADD_IF_NOT_FOUND: mp_map_lookup_kind_t = 1;
pub const MP_MAP_LOOKUP_REMOVE_IF_FOUND: mp_map_lookup_kind_t = 2;
pub const MP_MAP_LOOKUP_ADD_IF_NOT_FOUND_OR_REMOVE_IF_FOUND: mp_map_lookup_kind_t = 3;

#[repr(C)]
pub struct mp_set_t {
    pub alloc: size_t,
    pub used: size_t,
    pub table: *mut mp_obj_t,
}

pub type mp_fun_0_t = Option<unsafe extern "C" fn() -> mp_obj_t>;
pub type mp_fun_1_t = Option<unsafe extern "C" fn(mp_obj_t) -> mp_obj_t>;
pub type mp_fun_2_t = Option<unsafe extern "C" fn(mp_obj_t, mp_obj_t) -> mp_obj_t>;
pub type mp_fun_3_t = Option<unsafe extern "C" fn(mp_obj_t, mp_obj_t, mp_obj_t) -> mp_obj_t>;
pub type mp_fun_var_t = Option<unsafe extern "C" fn(size_t, *const mp_obj_t) -> mp_obj_t>;
pub type mp_fun_kw_t =
    Option<unsafe extern "C" fn(size_t, *const mp_obj_t, *mut mp_map_t) -> mp_obj_t>;

pub const MP_TYPE_FLAG_NONE: u16 = 0;
pub const MP_TYPE_FLAG_IS_SUBCLASSED: u16 = 0x0001;
pub const MP_TYPE_FLAG_HAS_SPECIAL_ACCESSORS: u16 = 0x0002;
pub const MP_TYPE_FLAG_EQ_NOT_REFLEXIVE: u16 = 0x0004;
pub const MP_TYPE_FLAG_EQ_CHECKS_OTHER_TYPE: u16 = 0x0008;
pub const MP_TYPE_FLAG_EQ_HAS_NEQ_TEST: u16 = 0x0010;
pub const MP_TYPE_FLAG_BINDS_SELF: u16 = 0x0020;
pub const MP_TYPE_FLAG_BUILTIN_FUN: u16 = 0x0040;
pub const MP_TYPE_FLAG_ITER_IS_GETITER: u16 = 0;
pub const MP_TYPE_FLAG_ITER_IS_ITERNEXT: u16 = 0x0080;
pub const MP_TYPE_FLAG_ITER_IS_CUSTOM: u16 = 0x0100;
pub const MP_TYPE_FLAG_ITER_IS_STREAM: u16 = 0x0180;
pub const MP_TYPE_FLAG_INSTANCE_TYPE: u16 = 0x0200;
pub const MP_TYPE_FLAG_SUBSCR_ALLOWS_STACK_SLICE: u16 = 0x0400;

pub type mp_print_kind_t = c_uint;
pub const PRINT_STR: mp_print_kind_t = 0;
pub const PRINT_REPR: mp_print_kind_t = 1;
pub const PRINT_EXC: mp_print_kind_t = 2;
pub const PRINT_JSON: mp_print_kind_t = 3;
pub const PRINT_RAW: mp_print_kind_t = 4;
pub const PRINT_EXC_SUBCLASS: mp_print_kind_t = 0x80;

#[repr(C)]
pub struct mp_obj_iter_buf_t {
    pub base: mp_obj_base_t,
    pub buf: [mp_obj_t; 3],
}
pub type mp_print_fun_t =
    Option<unsafe extern "C" fn(*const mp_print_t, mp_obj_t, mp_print_kind_t)>;
pub type mp_make_new_fun_t =
    Option<unsafe extern "C" fn(*const mp_obj_type_t, size_t, size_t, *const mp_obj_t) -> mp_obj_t>;
pub type mp_call_fun_t =
    Option<unsafe extern "C" fn(mp_obj_t, size_t, size_t, *const mp_obj_t) -> mp_obj_t>;
pub type mp_unary_op_fun_t = Option<unsafe extern "C" fn(mp_unary_op_t, mp_obj_t) -> mp_obj_t>;
pub type mp_binary_op_fun_t =
    Option<unsafe extern "C" fn(mp_binary_op_t, mp_obj_t, mp_obj_t) -> mp_obj_t>;
pub type mp_attr_fun_t = Option<unsafe extern "C" fn(mp_obj_t, qstr, *mut mp_obj_t)>;
pub type mp_subscr_fun_t = Option<unsafe extern "C" fn(mp_obj_t, mp_obj_t, mp_obj_t) -> mp_obj_t>;
pub type mp_getiter_fun_t =
    Option<unsafe extern "C" fn(mp_obj_t, *mut mp_obj_iter_buf_t) -> mp_obj_t>;
pub type mp_iternext_fun_t = mp_fun_1_t;

#[repr(C)]
pub struct mp_getiter_iternext_custom_t {
    pub getiter: mp_getiter_fun_t,
    pub iternext: mp_iternext_fun_t,
}
#[repr(C)]
pub struct mp_buffer_info_t {
    pub buf: *mut c_void,
    pub len: size_t,
    pub typecode: c_int,
}
pub const MP_BUFFER_READ: mp_uint_t = 1;
pub const MP_BUFFER_WRITE: mp_uint_t = 2;
pub const MP_BUFFER_RW: mp_uint_t = 3;
pub const MP_BUFFER_RAISE_IF_UNSUPPORTED: mp_uint_t = 4;
pub type mp_buffer_fun_t =
    Option<unsafe extern "C" fn(mp_obj_t, *mut mp_buffer_info_t, mp_uint_t) -> mp_int_t>;

macro_rules! type_struct {
    ($name:ident, $slots:ty) => {
        #[repr(C)]
        pub struct $name {
            pub base: mp_obj_base_t,
            pub flags: u16,
            pub name: u16,
            pub slot_index_make_new: u8,
            pub slot_index_print: u8,
            pub slot_index_call: u8,
            pub slot_index_unary_op: u8,
            pub slot_index_binary_op: u8,
            pub slot_index_attr: u8,
            pub slot_index_subscr: u8,
            pub slot_index_iter: u8,
            pub slot_index_buffer: u8,
            pub slot_index_protocol: u8,
            pub slot_index_parent: u8,
            pub slot_index_locals_dict: u8,
            pub slots: $slots,
        }
    };
}
type_struct!(mp_obj_type_t, [*const c_void; 0]);
type_struct!(mp_obj_empty_type_t, [*const c_void; 0]);
type_struct!(mp_obj_full_type_t, [*const c_void; 11]);

#[repr(C)]
pub struct mp_obj_cell_t {
    pub base: mp_obj_base_t,
    pub obj: mp_obj_t,
}
#[repr(C)]
pub struct mp_obj_dict_t {
    pub base: mp_obj_base_t,
    pub map: mp_map_t,
}
#[repr(C)]
pub struct mp_bound_slice_t {
    pub start: mp_int_t,
    pub stop: mp_int_t,
    pub step: mp_int_t,
}
#[repr(C)]
pub struct mp_obj_slice_t {
    pub base: mp_obj_base_t,
    pub start: mp_obj_t,
    pub stop: mp_obj_t,
    pub step: mp_obj_t,
}

#[repr(C)]
pub union mp_obj_fun_builtin_fixed_fun_t {
    pub _0: mp_fun_0_t,
    pub _1: mp_fun_1_t,
    pub _2: mp_fun_2_t,
    pub _3: mp_fun_3_t,
}
#[repr(C)]
pub struct mp_obj_fun_builtin_fixed_t {
    pub base: mp_obj_base_t,
    pub fun: mp_obj_fun_builtin_fixed_fun_t,
}
#[repr(C)]
pub union mp_obj_fun_builtin_var_fun_t {
    pub var: mp_fun_var_t,
    pub kw: mp_fun_kw_t,
}
#[repr(C)]
pub struct mp_obj_fun_builtin_var_t {
    pub base: mp_obj_base_t,
    pub sig: u32,
    pub fun: mp_obj_fun_builtin_var_fun_t,
}
#[repr(C)]
pub struct mp_obj_module_t {
    pub base: mp_obj_base_t,
    pub globals: *mut mp_obj_dict_t,
}
#[repr(C)]
pub struct mp_obj_static_class_method_t {
    pub base: mp_obj_base_t,
    pub fun: mp_obj_t,
}
#[repr(C)]
pub struct mp_rom_obj_static_class_method_t {
    pub base: mp_obj_base_t,
    pub fun: mp_rom_obj_t,
}

// Object structs whose complete definitions live in other py/obj*.h headers.
macro_rules! opaque_structs {
    ($($name:ident),* $(,)?) => { $(#[repr(C)] pub struct $name { _private: [u8; 0] })* };
}
opaque_structs!(
    mp_obj_float_t,
    mp_obj_none_t,
    mp_obj_bool_t,
    mp_obj_str_t,
    mp_obj_tuple_t,
    mp_obj_singleton_t,
    mp_obj_exception_t,
    vstr_t
);

pub const MP_OBJ_FUN_ARGS_MAX: u32 = 0xffff;
pub fn mp_obj_fun_make_sig(n_args_min: u32, n_args_max: u32, takes_kw: bool) -> u32 {
    (n_args_min << 17) | (n_args_max << 1) | takes_kw as u32
}
pub fn mp_obj_iter_buf_nslots() -> usize {
    core::mem::size_of::<mp_obj_iter_buf_t>().div_ceil(core::mem::size_of::<mp_obj_t>())
}

#[inline]
pub unsafe fn mp_map_slot_is_filled(map: *const mp_map_t, pos: size_t) -> bool {
    let map = unsafe { &*map };
    assert!(pos < map.alloc);
    let key = unsafe { (*map.table.add(pos)).key };
    key != MP_OBJ_NULL && key != MP_OBJ_SENTINEL
}

#[inline]
pub unsafe fn mp_set_slot_is_filled(set: *const mp_set_t, pos: size_t) -> bool {
    let set = unsafe { &*set };
    let value = unsafe { *set.table.add(pos) };
    value != MP_OBJ_NULL && value != MP_OBJ_SENTINEL
}

#[inline]
pub unsafe fn mp_obj_cell_get(self_: mp_obj_t) -> mp_obj_t {
    unsafe { (*mp_obj_to_ptr(self_).cast::<mp_obj_cell_t>()).obj }
}

#[inline]
pub unsafe fn mp_obj_cell_set(self_: mp_obj_t, obj: mp_obj_t) {
    unsafe { (*mp_obj_to_ptr(self_).cast::<mp_obj_cell_t>()).obj = obj };
}

#[inline]
pub unsafe fn mp_obj_dict_get_map(dict: mp_obj_t) -> *mut mp_map_t {
    unsafe { ptr::addr_of_mut!((*mp_obj_to_ptr(dict).cast::<mp_obj_dict_t>()).map) }
}

#[inline]
pub fn mp_obj_new_bool(value: mp_int_t) -> mp_obj_t {
    if value != 0 {
        mp_const_true()
    } else {
        mp_const_false()
    }
}

#[inline]
pub unsafe fn mp_obj_is_exact_type(o: mp_const_obj_t, type_: *const mp_obj_type_t) -> bool {
    mp_obj_is_obj(o)
        && unsafe { (*(mp_const_obj_to_ptr(o).cast::<mp_obj_base_t>())).type_ == type_ }
}

#[inline]
pub unsafe fn mp_obj_is_type(o: mp_const_obj_t, type_: *const mp_obj_type_t) -> bool {
    assert!(type_ != ptr::addr_of!(mp_type_bool));
    assert!(type_ != ptr::addr_of!(mp_type_int));
    assert!(type_ != ptr::addr_of!(mp_type_str));
    assert!(type_ != ptr::addr_of!(mp_type_NoneType));
    unsafe { mp_obj_is_exact_type(o, type_) }
}

#[inline]
pub unsafe fn mp_obj_is_int(o: mp_const_obj_t) -> bool {
    mp_obj_is_small_int(o) || unsafe { mp_obj_is_exact_type(o, ptr::addr_of!(mp_type_int)) }
}

#[inline]
pub unsafe fn mp_obj_is_str(o: mp_const_obj_t) -> bool {
    mp_obj_is_qstr(o) || unsafe { mp_obj_is_exact_type(o, ptr::addr_of!(mp_type_str)) }
}

#[cfg(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS")]
#[inline]
pub unsafe fn mp_obj_is_bool(o: mp_const_obj_t) -> bool {
    o == mp_const_false() as mp_const_obj_t || o == mp_const_true() as mp_const_obj_t
}

#[cfg(not(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS"))]
#[inline]
pub unsafe fn mp_obj_is_bool(o: mp_const_obj_t) -> bool {
    unsafe { mp_obj_is_exact_type(o, ptr::addr_of!(mp_type_bool)) }
}

#[inline]
pub unsafe fn mp_obj_is_integer(o: mp_const_obj_t) -> bool {
    (unsafe { mp_obj_is_int(o) }) || (unsafe { mp_obj_is_bool(o) })
}

#[cfg(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS")]
#[inline]
pub fn mp_const_none() -> mp_obj_t {
    mp_obj_new_immediate_obj(0)
}
#[cfg(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS")]
#[inline]
pub fn mp_const_false() -> mp_obj_t {
    mp_obj_new_immediate_obj(1)
}
#[cfg(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS")]
#[inline]
pub fn mp_const_true() -> mp_obj_t {
    mp_obj_new_immediate_obj(3)
}

#[cfg(not(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS"))]
#[inline]
pub fn mp_const_none() -> mp_obj_t {
    mp_obj_from_ptr(ptr::addr_of!(mp_const_none_obj).cast_mut().cast())
}
#[cfg(not(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS"))]
#[inline]
pub fn mp_const_false() -> mp_obj_t {
    mp_obj_from_ptr(ptr::addr_of!(mp_const_false_obj).cast_mut().cast())
}
#[cfg(not(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS"))]
#[inline]
pub fn mp_const_true() -> mp_obj_t {
    mp_obj_from_ptr(ptr::addr_of!(mp_const_true_obj).cast_mut().cast())
}

#[cfg(all(
    micropython = "MICROPY_PY_BUILTINS_FLOAT",
    any(micropython = "MICROPY_OBJ_REPR_A", micropython = "MICROPY_OBJ_REPR_B")
))]
#[inline]
pub unsafe fn mp_obj_is_float(o: mp_const_obj_t) -> bool {
    unsafe { mp_obj_is_exact_type(o, ptr::addr_of!(mp_type_float)) }
}

#[cfg(all(
    micropython = "MICROPY_PY_BUILTINS_FLOAT",
    micropython = "MICROPY_OBJ_REPR_C"
))]
#[inline]
pub fn mp_obj_is_float(o: mp_const_obj_t) -> bool {
    let bits = o as mp_uint_t;
    bits & 3 == 2 && bits & 0xff80_0007 != 6
}

#[cfg(all(
    micropython = "MICROPY_PY_BUILTINS_FLOAT",
    micropython = "MICROPY_OBJ_REPR_C"
))]
#[inline]
pub fn mp_obj_float_get(o: mp_obj_t) -> mp_float_t {
    let mut bits = ((o as mp_uint_t).wrapping_sub(0x8080_0000)) & !3;
    bits |= (bits >> 2) & 3;
    f32::from_bits(bits as u32)
}

#[cfg(all(
    micropython = "MICROPY_PY_BUILTINS_FLOAT",
    micropython = "MICROPY_OBJ_REPR_C"
))]
#[inline]
pub fn mp_obj_new_float(value: mp_float_t) -> mp_obj_t {
    let bits = if value.is_nan() {
        0x7fc0_0000
    } else {
        value.to_bits()
    };
    (((bits & !3) | 2).wrapping_add(0x8080_0000) as usize) as mp_obj_t
}

#[cfg(all(
    micropython = "MICROPY_PY_BUILTINS_FLOAT",
    micropython = "MICROPY_OBJ_REPR_D"
))]
#[inline]
pub fn mp_obj_is_float(o: mp_const_obj_t) -> bool {
    o & 0xfffc_0000_0000_0000 != 0
}

#[cfg(all(
    micropython = "MICROPY_PY_BUILTINS_FLOAT",
    micropython = "MICROPY_OBJ_REPR_D"
))]
#[inline]
pub fn mp_obj_float_get(o: mp_obj_t) -> mp_float_t {
    f64::from_bits(o.wrapping_sub(0x8004_0000_0000_0000))
}

#[cfg(all(
    micropython = "MICROPY_PY_BUILTINS_FLOAT",
    micropython = "MICROPY_OBJ_REPR_D"
))]
#[inline]
pub fn mp_obj_new_float(value: mp_float_t) -> mp_obj_t {
    let bits = if value.is_nan() {
        0x7ff8_0000_0000_0000
    } else {
        value.to_bits()
    };
    bits.wrapping_add(0x8004_0000_0000_0000)
}

#[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
#[inline]
pub unsafe fn mp_obj_get_float_to_f(o: mp_obj_t) -> f32 {
    unsafe { mp_obj_get_float(o) as f32 }
}

#[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
#[inline]
pub unsafe fn mp_obj_get_float_to_d(o: mp_obj_t) -> f64 {
    unsafe { mp_obj_get_float(o) as f64 }
}

#[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
#[inline]
pub unsafe fn mp_obj_new_float_from_f(value: f32) -> mp_obj_t {
    #[cfg(any(micropython = "MICROPY_OBJ_REPR_A", micropython = "MICROPY_OBJ_REPR_B"))]
    {
        unsafe { mp_obj_new_float(value as mp_float_t) }
    }
    #[cfg(any(micropython = "MICROPY_OBJ_REPR_C", micropython = "MICROPY_OBJ_REPR_D"))]
    {
        mp_obj_new_float(value as mp_float_t)
    }
}

#[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
#[inline]
pub unsafe fn mp_obj_new_float_from_d(value: f64) -> mp_obj_t {
    #[cfg(any(micropython = "MICROPY_OBJ_REPR_A", micropython = "MICROPY_OBJ_REPR_B"))]
    {
        unsafe { mp_obj_new_float(value as mp_float_t) }
    }
    #[cfg(any(micropython = "MICROPY_OBJ_REPR_C", micropython = "MICROPY_OBJ_REPR_D"))]
    {
        mp_obj_new_float(value as mp_float_t)
    }
}

#[cfg(not(all(
    micropython = "MICROPY_PY_BUILTINS_STR_UNICODE",
    micropython = "MICROPY_PY_BUILTINS_STR_UNICODE_CHECK"
)))]
#[inline]
pub unsafe fn mp_obj_new_str_from_utf8_vstr(vstr: *mut vstr_t) -> mp_obj_t {
    unsafe { mp_obj_new_str_from_vstr(vstr) }
}

#[cfg(all(
    micropython = "MICROPY_PY_BUILTINS_FLOAT",
    not(micropython = "MICROPY_FLOAT_HIGH_QUALITY_HASH")
))]
#[inline]
pub fn mp_float_hash(value: mp_float_t) -> mp_int_t {
    value as mp_int_t
}

#[cfg(micropython = "MICROPY_ERROR_REPORTING_NONE")]
#[inline]
pub unsafe fn mp_obj_new_exception_msg(
    exc_type: *const mp_obj_type_t,
    _msg: *const c_char,
) -> mp_obj_t {
    unsafe { mp_obj_new_exception(exc_type) }
}

#[inline]
pub unsafe fn mp_get_buffer_raise(obj: mp_obj_t, bufinfo: *mut mp_buffer_info_t, flags: mp_uint_t) {
    unsafe {
        mp_get_buffer(obj, bufinfo, flags | MP_BUFFER_RAISE_IF_UNSUPPORTED);
    }
}

#[inline]
pub unsafe fn mp_obj_new_exception_arg1(exc_type: *const mp_obj_type_t, arg: mp_obj_t) -> mp_obj_t {
    unsafe { mp_obj_exception_make_new(exc_type, 1, 0, ptr::addr_of!(arg)) }
}

unsafe extern "C" {
    pub fn mp_map_init(map: *mut mp_map_t, n: size_t);
    pub fn mp_map_init_fixed_table(map: *mut mp_map_t, n: size_t, table: *const mp_obj_t);
    pub fn mp_map_deinit(map: *mut mp_map_t);
    pub fn mp_map_lookup(
        map: *mut mp_map_t,
        index: mp_obj_t,
        lookup_kind: mp_map_lookup_kind_t,
    ) -> *mut mp_map_elem_t;
    pub fn mp_map_clear(map: *mut mp_map_t);
    pub fn mp_map_dump(map: *mut mp_map_t);
    pub fn mp_set_init(set: *mut mp_set_t, n: size_t);
    pub fn mp_set_lookup(
        set: *mut mp_set_t,
        index: mp_obj_t,
        lookup_kind: mp_map_lookup_kind_t,
    ) -> mp_obj_t;
    pub fn mp_set_remove_first(set: *mut mp_set_t) -> mp_obj_t;
    pub fn mp_set_clear(set: *mut mp_set_t);
    pub fn mp_get_buffer(obj: mp_obj_t, bufinfo: *mut mp_buffer_info_t, flags: mp_uint_t) -> bool;

    pub fn mp_obj_malloc_helper(num_bytes: size_t, type_: *const mp_obj_type_t) -> *mut c_void;
    #[cfg(micropython = "MICROPY_ENABLE_FINALISER")]
    pub fn mp_obj_malloc_with_finaliser_helper(
        num_bytes: size_t,
        type_: *const mp_obj_type_t,
    ) -> *mut c_void;
    pub fn mp_obj_is_dict_or_ordereddict(o: mp_obj_t) -> bool;
    pub fn mp_obj_new_cell(obj: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_new_int(value: mp_int_t) -> mp_obj_t;
    pub fn mp_obj_new_int_from_uint(value: mp_uint_t) -> mp_obj_t;
    pub fn mp_obj_new_int_from_str_len(
        str_: *mut *const c_char,
        len: size_t,
        neg: bool,
        base: c_uint,
    ) -> mp_obj_t;
    pub fn mp_obj_new_int_from_ll(value: i64) -> mp_obj_t;
    pub fn mp_obj_new_int_from_ull(value: u64) -> mp_obj_t;
    pub fn mp_obj_new_str(data: *const c_char, len: size_t) -> mp_obj_t;
    pub fn mp_obj_new_str_from_cstr(str_: *const c_char) -> mp_obj_t;
    pub fn mp_obj_new_str_via_qstr(data: *const c_char, len: size_t) -> mp_obj_t;
    pub fn mp_obj_new_str_from_vstr(vstr: *mut vstr_t) -> mp_obj_t;
    #[cfg(all(
        micropython = "MICROPY_PY_BUILTINS_STR_UNICODE",
        micropython = "MICROPY_PY_BUILTINS_STR_UNICODE_CHECK"
    ))]
    pub fn mp_obj_new_str_from_utf8_vstr(vstr: *mut vstr_t) -> mp_obj_t;
    pub fn mp_obj_new_bytes_from_vstr(vstr: *mut vstr_t) -> mp_obj_t;
    pub fn mp_obj_new_bytes(data: *const byte, len: size_t) -> mp_obj_t;
    pub fn mp_obj_new_bytearray(n: size_t, items: *const c_void) -> mp_obj_t;
    pub fn mp_obj_new_bytearray_by_ref(n: size_t, items: *mut c_void) -> mp_obj_t;
    #[cfg(micropython = "MICROPY_PY_TSTRINGS")]
    pub fn mp_obj_new_template(n_args: size_t, args: *const mp_obj_t) -> mp_obj_t;
    #[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
    pub fn mp_obj_new_int_from_float(value: mp_float_t) -> mp_obj_t;
    #[cfg(all(
        micropython = "MICROPY_PY_BUILTINS_FLOAT",
        any(micropython = "MICROPY_OBJ_REPR_A", micropython = "MICROPY_OBJ_REPR_B")
    ))]
    pub fn mp_obj_new_float(value: mp_float_t) -> mp_obj_t;
    #[cfg(all(
        micropython = "MICROPY_PY_BUILTINS_FLOAT",
        any(micropython = "MICROPY_OBJ_REPR_A", micropython = "MICROPY_OBJ_REPR_B")
    ))]
    pub fn mp_obj_float_get(obj: mp_obj_t) -> mp_float_t;
    #[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
    pub fn mp_obj_new_complex(real: mp_float_t, imag: mp_float_t) -> mp_obj_t;
    pub fn mp_obj_new_exception(exc_type: *const mp_obj_type_t) -> mp_obj_t;
    pub fn mp_obj_new_exception_args(
        exc_type: *const mp_obj_type_t,
        n_args: size_t,
        args: *const mp_obj_t,
    ) -> mp_obj_t;
    #[cfg(not(micropython = "MICROPY_ERROR_REPORTING_NONE"))]
    pub fn mp_obj_new_exception_msg(exc_type: *const mp_obj_type_t, msg: *const c_char)
        -> mp_obj_t;
    #[cfg(not(micropython = "MICROPY_ERROR_REPORTING_NONE"))]
    pub fn mp_obj_new_exception_msg_varg(
        exc_type: *const mp_obj_type_t,
        fmt: *const c_char,
        ...
    ) -> mp_obj_t;
    pub fn mp_obj_new_gen_wrap(fun: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_new_closure(fun: mp_obj_t, n_closed: size_t, closed: *const mp_obj_t)
        -> mp_obj_t;
    pub fn mp_obj_new_tuple(n: size_t, items: *const mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_new_list(n: size_t, items: *mut mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_new_dict(n_args: size_t) -> mp_obj_t;
    pub fn mp_obj_new_set(n_args: size_t, items: *mut mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_new_slice(start: mp_obj_t, stop: mp_obj_t, step: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_new_bound_meth(meth: mp_obj_t, self_: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_new_getitem_iter(
        args: *mut mp_obj_t,
        iter_buf: *mut mp_obj_iter_buf_t,
    ) -> mp_obj_t;
    pub fn mp_obj_new_module(module_name: qstr) -> mp_obj_t;
    pub fn mp_obj_new_memoryview(typecode: byte, nitems: size_t, items: *mut c_void) -> mp_obj_t;

    pub fn mp_obj_get_type(o: mp_const_obj_t) -> *const mp_obj_type_t;
    pub fn mp_obj_get_type_str(o: mp_const_obj_t) -> *const c_char;
    pub fn mp_obj_is_subclass_fast(object: mp_const_obj_t, classinfo: mp_const_obj_t) -> bool;
    pub fn mp_obj_cast_to_native_base(self_: mp_obj_t, native_type: mp_const_obj_t) -> mp_obj_t;
    pub fn mp_obj_print_helper(print: *const mp_print_t, o: mp_obj_t, kind: mp_print_kind_t);
    pub fn mp_obj_print(o: mp_obj_t, kind: mp_print_kind_t);
    pub fn mp_obj_print_exception(print: *const mp_print_t, exc: mp_obj_t);
    pub fn mp_obj_is_true(arg: mp_obj_t) -> bool;
    pub fn mp_obj_is_callable(o: mp_obj_t) -> bool;
    pub fn mp_obj_equal_not_equal(op: mp_binary_op_t, o1: mp_obj_t, o2: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_equal(o1: mp_obj_t, o2: mp_obj_t) -> bool;
    pub fn mp_obj_get_int(arg: mp_const_obj_t) -> mp_int_t;
    pub fn mp_obj_get_uint(arg: mp_const_obj_t) -> mp_uint_t;
    pub fn mp_obj_get_ll(arg: mp_const_obj_t) -> i64;
    pub fn mp_obj_get_int_truncated(arg: mp_const_obj_t) -> mp_int_t;
    pub fn mp_obj_get_int_maybe(arg: mp_const_obj_t, value: *mut mp_int_t) -> bool;
    #[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
    pub fn mp_obj_get_float(obj: mp_obj_t) -> mp_float_t;
    #[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
    pub fn mp_obj_get_float_maybe(obj: mp_obj_t, value: *mut mp_float_t) -> bool;
    #[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
    pub fn mp_obj_get_complex(obj: mp_obj_t, real: *mut mp_float_t, imag: *mut mp_float_t);
    #[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
    pub fn mp_obj_get_complex_maybe(
        obj: mp_obj_t,
        real: *mut mp_float_t,
        imag: *mut mp_float_t,
    ) -> bool;
    pub fn mp_obj_get_array(o: mp_obj_t, len: *mut size_t, items: *mut *mut mp_obj_t);
    pub fn mp_obj_get_array_fixed_n(o: mp_obj_t, len: size_t, items: *mut *mut mp_obj_t);
    pub fn mp_get_index(
        type_: *const mp_obj_type_t,
        len: size_t,
        index: mp_obj_t,
        is_slice: bool,
    ) -> size_t;
    pub fn mp_obj_id(o: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_len(o: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_len_maybe(o: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_subscr(base: mp_obj_t, index: mp_obj_t, val: mp_obj_t) -> mp_obj_t;

    pub fn mp_obj_int_get_truncated(o: mp_const_obj_t) -> mp_int_t;
    pub fn mp_obj_int_get_checked(o: mp_const_obj_t) -> mp_int_t;
    pub fn mp_obj_int_get_uint_checked(o: mp_const_obj_t) -> mp_uint_t;
    pub fn mp_obj_is_native_exception_instance(o: mp_obj_t) -> bool;
    pub fn mp_obj_is_exception_type(o: mp_obj_t) -> bool;
    pub fn mp_obj_is_exception_instance(o: mp_obj_t) -> bool;
    pub fn mp_obj_exception_match(exc: mp_obj_t, exc_type: mp_const_obj_t) -> bool;
    pub fn mp_obj_exception_clear_traceback(o: mp_obj_t);
    pub fn mp_obj_exception_add_traceback(o: mp_obj_t, file: qstr, line: size_t, block: qstr);
    pub fn mp_obj_exception_get_traceback(o: mp_obj_t, n: *mut size_t, values: *mut *mut size_t);
    pub fn mp_obj_exception_get_value(o: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_exception_make_new(
        type_: *const mp_obj_type_t,
        n_args: size_t,
        n_kw: size_t,
        args: *const mp_obj_t,
    ) -> mp_obj_t;
    pub fn mp_alloc_emergency_exception_buf(size: mp_obj_t) -> mp_obj_t;
    pub fn mp_init_emergency_exception_buf();

    pub fn mp_obj_str_equal(s1: mp_obj_t, s2: mp_obj_t) -> bool;
    pub fn mp_obj_str_get_qstr(o: mp_obj_t) -> qstr;
    pub fn mp_obj_str_get_str(o: mp_obj_t) -> *const c_char;
    pub fn mp_obj_str_get_data(o: mp_obj_t, len: *mut size_t) -> *const c_char;
    pub fn mp_obj_str_intern(o: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_str_intern_checked(o: mp_obj_t) -> mp_obj_t;
    pub fn mp_str_print_quoted(
        print: *const mp_print_t,
        data: *const byte,
        len: size_t,
        is_bytes: bool,
    );
    #[cfg(all(
        micropython = "MICROPY_PY_BUILTINS_FLOAT",
        micropython = "MICROPY_FLOAT_HIGH_QUALITY_HASH"
    ))]
    pub fn mp_float_hash(value: mp_float_t) -> mp_int_t;
    #[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
    pub fn mp_obj_float_binary_op(op: mp_binary_op_t, lhs: mp_float_t, rhs: mp_obj_t) -> mp_obj_t;
    #[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
    pub fn mp_obj_complex_get(o: mp_obj_t, real: *mut mp_float_t, imag: *mut mp_float_t);
    #[cfg(micropython = "MICROPY_PY_BUILTINS_FLOAT")]
    pub fn mp_obj_complex_binary_op(
        op: mp_binary_op_t,
        lhs_real: mp_float_t,
        lhs_imag: mp_float_t,
        rhs: mp_obj_t,
    ) -> mp_obj_t;

    pub fn mp_obj_dict_make_new(
        type_: *const mp_obj_type_t,
        n_args: size_t,
        n_kw: size_t,
        args: *const mp_obj_t,
    ) -> mp_obj_t;
    pub fn mp_obj_dict_init(dict: *mut mp_obj_dict_t, n_args: size_t);
    pub fn mp_obj_dict_len(o: mp_obj_t) -> size_t;
    pub fn mp_obj_dict_get(o: mp_obj_t, index: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_dict_store(o: mp_obj_t, key: mp_obj_t, value: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_dict_delete(o: mp_obj_t, key: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_dict_copy(o: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_set_store(o: mp_obj_t, item: mp_obj_t);
    pub fn mp_obj_slice_indices(o: mp_obj_t, length: mp_int_t, result: *mut mp_bound_slice_t);
    pub fn mp_identity(o: mp_obj_t) -> mp_obj_t;
    pub fn mp_obj_property_get(o: mp_obj_t) -> *const mp_obj_t;

    pub fn mp_seq_multiply(
        items: *const c_void,
        item_sz: size_t,
        len: size_t,
        times: size_t,
        dest: *mut c_void,
    );
    #[cfg(micropython = "MICROPY_PY_BUILTINS_SLICE")]
    pub fn mp_seq_get_fast_slice_indexes(
        len: mp_uint_t,
        slice: mp_obj_t,
        indexes: *mut mp_bound_slice_t,
    ) -> bool;
    pub fn mp_seq_cmp_bytes(
        op: mp_uint_t,
        data1: *const byte,
        len1: size_t,
        data2: *const byte,
        len2: size_t,
    ) -> bool;
    pub fn mp_seq_cmp_objs(
        op: mp_uint_t,
        items1: *const mp_obj_t,
        len1: size_t,
        items2: *const mp_obj_t,
        len2: size_t,
    ) -> bool;
    pub fn mp_seq_index_obj(
        items: *const mp_obj_t,
        len: size_t,
        n_args: size_t,
        args: *const mp_obj_t,
    ) -> mp_obj_t;
    pub fn mp_seq_count_obj(items: *const mp_obj_t, len: size_t, value: mp_obj_t) -> mp_obj_t;
    pub fn mp_seq_extract_slice(seq: *const mp_obj_t, indexes: *mut mp_bound_slice_t) -> mp_obj_t;
}

macro_rules! extern_types {
    ($($name:ident),* $(,)?) => { unsafe extern "C" { $(pub static $name: mp_obj_type_t;)* } };
}
extern_types!(
    mp_type_type,
    mp_type_object,
    mp_type_NoneType,
    mp_type_bool,
    mp_type_int,
    mp_type_str,
    mp_type_template,
    mp_type_interpolation,
    mp_type_bytes,
    mp_type_bytearray,
    mp_type_memoryview,
    mp_type_float,
    mp_type_complex,
    mp_type_tuple,
    mp_type_list,
    mp_type_map,
    mp_type_enumerate,
    mp_type_filter,
    mp_type_deque,
    mp_type_dict,
    mp_type_ordereddict,
    mp_type_range,
    mp_type_set,
    mp_type_frozenset,
    mp_type_slice,
    mp_type_zip,
    mp_type_array,
    mp_type_super,
    mp_type_gen_wrap,
    mp_type_native_gen_wrap,
    mp_type_gen_instance,
    mp_type_fun_builtin_0,
    mp_type_fun_builtin_1,
    mp_type_fun_builtin_2,
    mp_type_fun_builtin_3,
    mp_type_fun_builtin_var,
    mp_type_fun_bc,
    mp_type_fun_native,
    mp_type_fun_viper,
    mp_type_fun_asm,
    mp_type_code,
    mp_type_module,
    mp_type_staticmethod,
    mp_type_classmethod,
    mp_type_bound_meth,
    mp_type_property,
    mp_type_stringio,
    mp_type_bytesio,
    mp_type_ringio,
    mp_type_reversed,
    mp_type_polymorph_iter,
    mp_type_BaseException,
    mp_type_ArithmeticError,
    mp_type_AssertionError,
    mp_type_AttributeError,
    mp_type_EOFError,
    mp_type_Exception,
    mp_type_GeneratorExit,
    mp_type_ImportError,
    mp_type_IndentationError,
    mp_type_IndexError,
    mp_type_KeyboardInterrupt,
    mp_type_KeyError,
    mp_type_LookupError,
    mp_type_MemoryError,
    mp_type_NameError,
    mp_type_NotImplementedError,
    mp_type_OSError,
    mp_type_OverflowError,
    mp_type_RuntimeError,
    mp_type_StopAsyncIteration,
    mp_type_StopIteration,
    mp_type_SyntaxError,
    mp_type_SystemExit,
    mp_type_TypeError,
    mp_type_UnicodeError,
    mp_type_ValueError,
    mp_type_ViperTypeError,
    mp_type_ZeroDivisionError,
);
#[cfg(micropython = "MICROPY_ENABLE_FINALISER")]
unsafe extern "C" {
    pub static mp_type_polymorph_iter_with_finaliser: mp_obj_type_t;
}

unsafe extern "C" {
    #[cfg(not(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS"))]
    pub static mp_const_none_obj: mp_obj_none_t;
    #[cfg(not(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS"))]
    pub static mp_const_false_obj: mp_obj_bool_t;
    #[cfg(not(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS"))]
    pub static mp_const_true_obj: mp_obj_bool_t;
    pub static mp_const_empty_bytes_obj: mp_obj_str_t;
    pub static mp_const_empty_tuple_obj: mp_obj_tuple_t;
    pub static mp_const_empty_dict_obj: mp_obj_dict_t;
    pub static mp_const_ellipsis_obj: mp_obj_singleton_t;
    pub static mp_const_notimplemented_obj: mp_obj_singleton_t;
    pub static mp_const_GeneratorExit_obj: mp_obj_exception_t;
    pub static mp_identity_obj: mp_obj_fun_builtin_fixed_t;
    #[cfg(all(
        micropython = "MICROPY_PY_BUILTINS_FLOAT",
        not(any(micropython = "MICROPY_OBJ_REPR_C", micropython = "MICROPY_OBJ_REPR_D"))
    ))]
    pub static mp_const_float_e_obj: mp_obj_float_t;
    #[cfg(all(
        micropython = "MICROPY_PY_BUILTINS_FLOAT",
        not(any(micropython = "MICROPY_OBJ_REPR_C", micropython = "MICROPY_OBJ_REPR_D"))
    ))]
    pub static mp_const_float_pi_obj: mp_obj_float_t;
    #[cfg(all(
        micropython = "MICROPY_PY_BUILTINS_FLOAT",
        micropython = "MICROPY_PY_MATH_CONSTANTS",
        not(any(micropython = "MICROPY_OBJ_REPR_C", micropython = "MICROPY_OBJ_REPR_D"))
    ))]
    pub static mp_const_float_tau_obj: mp_obj_float_t;
    #[cfg(all(
        micropython = "MICROPY_PY_BUILTINS_FLOAT",
        micropython = "MICROPY_PY_MATH_CONSTANTS",
        not(any(micropython = "MICROPY_OBJ_REPR_C", micropython = "MICROPY_OBJ_REPR_D"))
    ))]
    pub static mp_const_float_inf_obj: mp_obj_float_t;
    #[cfg(all(
        micropython = "MICROPY_PY_BUILTINS_FLOAT",
        micropython = "MICROPY_PY_MATH_CONSTANTS",
        not(any(micropython = "MICROPY_OBJ_REPR_C", micropython = "MICROPY_OBJ_REPR_D"))
    ))]
    pub static mp_const_float_nan_obj: mp_obj_float_t;
}
