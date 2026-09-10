use core::{ffi::c_void, marker::PhantomData, ops::Deref, ptr::NonNull};

use micropython_sys::{mp_int_t, mp_obj_base_t, mp_obj_get_type, mp_obj_t, mp_uint_t};

use crate::{gc::Gc, qstr::Qstr, ty::Type, vm::MicroPython};

mod direct_ptr;

#[cfg_attr(micropython = "MICROPY_OBJ_REPR_A", path = "obj/repr_a.rs")]
#[cfg_attr(micropython = "MICROPY_OBJ_REPR_B", path = "obj/repr_b.rs")]
#[cfg_attr(micropython = "MICROPY_OBJ_REPR_C", path = "obj/repr_c.rs")]
#[cfg_attr(micropython = "MICROPY_OBJ_REPR_D", path = "obj/repr_d.rs")]
mod tagging;

#[derive(Clone, Copy)]
pub struct Obj {
    inner: mp_obj_t,
}

// SAFETY: safe to send/share between threads because only immutable access of small ints, qstrs
// and immediate objects is allowed
unsafe impl Send for Obj {}
unsafe impl Sync for Obj {}

#[repr(transparent)]
pub struct Immortal<T: 'static> {
    inner: mp_obj_t,
    phantom: PhantomData<&'static T>,
}

unsafe impl<T: Send> Send for Immortal<T> {}
unsafe impl<T: Sync> Sync for Immortal<T> {}

/// Object that temporarily restricts use of MicroPython
pub struct Restricted<'gc> {
    inner: mp_obj_t,
    _gc: &'gc mut Gc,
}

pub struct Rooted<'r, T> {
    inner: NonNull<T>,
    root: PhantomData<&'r Obj>,
}

pub struct Bound<'b, T> {
    inner: NonNull<T>,
    mp: &'b MicroPython,
    reference: PhantomData<&'b T>,
}

/// A Rust struct representing pointer objects of a particular MicroPython type.
///
/// # Safety
///
/// Implementations must guarantee that:
///
/// - `Self` has a stable, C-compatible layout with an
///   [`mp_obj_base_t`](micropython_sys::mp_obj_base_t) at offset zero.
/// - Every valid pointer object with the exact type pointer returned by
///   [`Self::type_object`] has sufficient size and alignment for `Self`, and
///   contains valid, initialized values for all of its Rust fields.
/// - Such an object can be borrowed as `&Self` while it is kept alive and
///   [`MicroPython`] is shared-borrowed. Its contents must respect Rust's rules
///   for shared references throughout that borrow.
/// - [`Self::type_object`] always returns the same type object, which remains
///   valid for the lifetime of the program.
pub unsafe trait Class: Sized {
    /// Returns the type object whose pointer identifies instances of `Self`.
    fn type_object() -> &'static Type;
}

pub trait RootProject: Sized {
    type Projection<'r>;

    fn project<'r>(root: Rooted<'r, Self>) -> Self::Projection<'r>;
}

impl Obj {
    pub const unsafe fn from_raw(o: mp_obj_t) -> Self {
        Self { inner: o }
    }

    pub const fn from_small_int(v: mp_int_t) -> Self {
        Self {
            inner: tagging::new_small_int(v),
        }
    }

    pub const fn from_qstr(q: Qstr) -> Self {
        Self {
            inner: tagging::new_qstr(q.into_raw()),
        }
    }

    pub const fn from_immediate(imm: mp_uint_t) -> Self {
        Self {
            inner: tagging::new_immediate(imm),
        }
    }

    pub fn is_small_int(self) -> bool {
        tagging::is_small_int(self.inner)
    }

    pub fn is_qstr(self) -> bool {
        tagging::is_qstr(self.inner)
    }

    pub fn is_immediate(self) -> bool {
        tagging::is_immediate(self.inner)
    }

    pub fn is_ptr(self) -> bool {
        tagging::is_ptr(self.inner)
    }

    pub fn small_int(self) -> Option<mp_int_t> {
        if tagging::is_small_int(self.inner) {
            Some(tagging::small_int_value(self.inner))
        } else {
            None
        }
    }

    pub fn qstr(self) -> Option<Qstr> {
        if tagging::is_qstr(self.inner) {
            Some(unsafe { Qstr::from_raw(tagging::qstr_value(self.inner)) })
        } else {
            None
        }
    }

    pub fn immediate(self) -> Option<mp_uint_t> {
        if tagging::is_immediate(self.inner) {
            Some(tagging::immediate_value(self.inner))
        } else {
            None
        }
    }

    pub unsafe fn assume_rooted<T>(&self) -> Rooted<'_, T> {
        Rooted {
            inner: unsafe { NonNull::new_unchecked(tagging::ptr_value(self.inner) as *mut _) },
            root: PhantomData,
        }
    }

    pub unsafe fn assume_bound<'obj, 'py, 'bound, T>(
        &'obj self,
        mp: &'py MicroPython,
    ) -> Bound<'bound, T>
    where
        'obj: 'bound,
        'py: 'bound,
    {
        Bound {
            inner: unsafe { NonNull::new_unchecked(tagging::ptr_value(self.inner) as *mut _) },
            mp,
            reference: PhantomData,
        }
    }

    pub fn into_raw(self) -> mp_obj_t {
        self.inner
    }
}

#[cfg(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS")]
impl Obj {
    pub const NONE: Self = Self::from_immediate(0);
    pub const FALSE: Self = Self::from_immediate(1);
    pub const TRUE: Self = Self::from_immediate(3);
}

// TODO: add using Immortal<T>
#[cfg(not(micropython = "MICROPY_OBJ_IMMEDIATE_OBJS"))]
impl Obj {}

impl<T: 'static> Immortal<T> {
    pub const fn new(inner: &'static T) -> Self {
        Self {
            inner: tagging::new_ptr(inner as *const T as *mut c_void),
            phantom: PhantomData,
        }
    }

    pub fn bind(&self) -> &T {
        // SAFETY: `self.inner` was constructed from a `&'static T`
        unsafe { &*tagging::ptr_value(self.inner).cast() }
    }
}

impl<'gc> Restricted<'gc> {
    pub unsafe fn from_raw(o: mp_obj_t, gc: &'gc mut Gc) -> Self {
        Self { inner: o, _gc: gc }
    }

    pub fn as_obj(self) -> Obj {
        Obj { inner: self.inner }
    }

    pub fn try_bind<'bound, T>(&'bound self, mp: &'bound MicroPython) -> Option<Bound<'bound, T>>
    where
        T: Class,
    {
        if !tagging::is_ptr(self.inner) {
            return None;
        }

        let ptr = tagging::ptr_value(self.inner);
        let base = ptr as *const mp_obj_base_t;
        let type_match = unsafe { (*base).type_ == T::type_object().as_raw() as *const _ };
        if type_match {
            Some(Bound {
                inner: NonNull::new(ptr)?.cast(),
                mp,
                reference: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn bind<'bound, T>(&'bound self, mp: &'bound MicroPython) -> Bound<'bound, T>
    where
        T: Class,
    {
        Self::try_bind(&self, mp).unwrap_or_else(|| panic!(""))
    }
}

impl<'r, T> Rooted<'r, T> {
    pub unsafe fn project_unchecked<U>(
        &self,
        project: impl FnOnce(NonNull<T>) -> NonNull<U>,
    ) -> Rooted<'r, U> {
        Rooted {
            inner: project(self.inner),
            root: PhantomData,
        }
    }

    pub fn project(&self) -> T::Projection<'r>
    where
        T: RootProject,
    {
        T::project(Rooted {
            inner: self.inner,
            root: PhantomData,
        })
    }

    pub fn bind<'bound>(&'bound self, mp: &'bound MicroPython) -> Bound<'bound, T> {
        Bound {
            inner: self.inner,
            mp,
            reference: PhantomData,
        }
    }
}

impl<'b, T> Bound<'b, T> {
    pub const fn mp(&self) -> &'b MicroPython {
        self.mp
    }
}

impl<'b, T> Deref for Bound<'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl<'b> Bound<'b, Obj> {
    pub fn ty(&self) -> &Type {
        unsafe { Type::from_raw(mp_obj_get_type(self.deref().inner)) }
    }

    pub fn try_downcast<T>(&self) -> Option<Bound<'b, T>>
    where
        T: Class,
    {
        let obj = self.deref().inner;
        if !tagging::is_ptr(obj) {
            return None;
        }

        let base = obj as *const mp_obj_base_t;
        let type_match = unsafe { (*base).type_ == T::type_object().as_raw() as *const _ };
        if type_match {
            Some(Bound {
                inner: NonNull::new(obj)?.cast(),
                mp: self.mp,
                reference: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn downcast<T>(&self) -> Bound<'b, T>
    where
        T: Class,
    {
        self.try_downcast().unwrap_or_else(|| panic!(""))
    }
}
