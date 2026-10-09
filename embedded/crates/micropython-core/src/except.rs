use core::ffi::c_void;

use micropython_sys::{mp_obj_exception_make_new, mp_type_TypeError, mp_type_ValueError};

use crate::{
    obj::{Restricted, Rooted, tagging},
    str::Str,
    ty::Type,
    vm::{Gc, MicroPythonMut},
};

pub type RstResult<'gc, T> = Result<T, Restricted<'gc>>;

/// A native MicroPython exception type
#[repr(transparent)]
pub struct ExceptionType(Type);

impl ExceptionType {
    /// # Safety
    /// `ty` must use `mp_obj_exception_make_new` as its constructor.
    pub const unsafe fn from_type(ty: &Type) -> &Self {
        // SAFETY: Self is transparent over Type; the caller guarantees the
        // native exception constructor invariant.
        unsafe { core::mem::transmute::<&Type, &Self>(ty) }
    }

    pub const fn as_type(&self) -> &Type {
        &self.0
    }
}

// SAFETY: these built-in types use mp_obj_exception_make_new.
pub const TYPE_ERROR_TYPE: &'static ExceptionType =
    unsafe { ExceptionType::from_type(Type::from_raw(&raw const mp_type_TypeError)) };
pub const VALUE_ERROR_TYPE: &'static ExceptionType =
    unsafe { ExceptionType::from_type(Type::from_raw(&raw const mp_type_ValueError)) };

pub fn new_exception_msg<'mp, 'r, 'gc>(
    _mp: MicroPythonMut<'mp>,
    gc: Gc<'gc>,
    ty: &'static ExceptionType,
    s: Rooted<'r, Str>,
) -> Restricted<'gc> {
    let s_ptr = s.as_non_null().as_ptr();
    let s_obj = tagging::new_ptr(s_ptr as *mut c_void);
    unsafe {
        Restricted::from_raw(
            mp_obj_exception_make_new(ty.as_type().as_raw() as *const _, 1, 0, &raw const s_obj),
            gc,
        )
    }
}
