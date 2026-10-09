use micropython_sys::{mp_obj_str_t, mp_type_str};

use crate::{obj::Class, ty::Type};

/// A MicroPython string object.
#[repr(transparent)]
pub struct Str {
    inner: mp_obj_str_t,
}

// SAFETY: exact str instances have the mp_obj_str_t layout. The private fields
// expose no mutation, and bound access holds a shared MicroPython borrow.
unsafe impl Class for Str {
    const TYPE_OBJECT: &'static Type = unsafe { Type::from_raw(&raw const mp_type_str) };
}
