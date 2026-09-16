use core::{ffi::c_void, marker::PhantomData, ops::Deref, ptr::NonNull};

use micropython_sys::{mp_int_t, mp_obj_base_t, mp_obj_get_type, mp_obj_t, mp_uint_t};

use crate::{
    qstr::Qstr,
    shims::mprs_function_ret_t,
    ty::Type,
    vm::{Gc, MicroPython, MicroPythonMut},
};

mod direct_ptr;

#[cfg_attr(micropython = "MICROPY_OBJ_REPR_A", path = "obj/repr_a.rs")]
#[cfg_attr(micropython = "MICROPY_OBJ_REPR_B", path = "obj/repr_b.rs")]
#[cfg_attr(micropython = "MICROPY_OBJ_REPR_C", path = "obj/repr_c.rs")]
#[cfg_attr(micropython = "MICROPY_OBJ_REPR_D", path = "obj/repr_d.rs")]
mod tagging;

#[repr(transparent)]
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
    _gc: Gc<'gc>,
}

pub struct Rooted<'r, T: ?Sized> {
    inner: NonNull<T>,
    root: PhantomData<&'r T>,
}

pub struct Bound<'r, 'mp, T: ?Sized> {
    inner: NonNull<T>,
    root: PhantomData<&'r T>,
    mp: MicroPython<'mp>,
}

impl<T: ?Sized> Clone for Bound<'_, '_, T> {
    fn clone(&self) -> Self {
        Bound {
            inner: self.inner,
            root: self.root,
            mp: self.mp,
        }
    }
}

impl<T: ?Sized> Copy for Bound<'_, '_, T> {}

/// A Rust struct representing pointer objects of a particular MicroPython type.
///
/// # Safety
///
/// Implementations must guarantee that:
///
/// - `Self` has a stable, C-compatible layout with an
///   [`mp_obj_base_t`](micropython_sys::mp_obj_base_t) at offset zero.
/// - Every valid pointer object with the exact type pointer
///   [`Self::TYPE_OBJECT`] has sufficient size and alignment for `Self`, and
///   contains valid, initialized values for all of its Rust fields.
/// - Such an object can be borrowed as `&Self` while it is kept alive and
///   [`MicroPython`] is shared-borrowed. Its contents must respect Rust's rules
///   for shared references throughout that borrow.
pub unsafe trait Class: Sized + 'static {
    /// The type object whose pointer identifies instances of `Self`.
    const TYPE_OBJECT: &'static Type;
}

pub trait RootProject: Sized {
    type Projection<'r>
    where
        Self: 'r;

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

    pub unsafe fn assume_bound<'obj, 'mp, T>(
        &'obj self,
        mp: MicroPython<'mp>,
    ) -> Bound<'obj, 'mp, T> {
        Bound {
            inner: unsafe { NonNull::new_unchecked(tagging::ptr_value(self.inner) as *mut _) },
            root: PhantomData,
            mp,
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
    pub unsafe fn from_raw(o: mp_obj_t, gc: Gc<'gc>) -> Self {
        Self { inner: o, _gc: gc }
    }

    pub fn as_obj(self) -> Obj {
        Obj { inner: self.inner }
    }

    pub fn try_bind<'obj, 'mp, T>(&'obj self, mp: MicroPython<'mp>) -> Option<Bound<'obj, 'mp, T>>
    where
        T: Class,
    {
        if !tagging::is_ptr(self.inner) {
            return None;
        }

        let ptr = tagging::ptr_value(self.inner);
        let base = ptr as *const mp_obj_base_t;
        let type_match = unsafe { (*base).type_ == T::TYPE_OBJECT.as_raw() as *const _ };
        if type_match {
            Some(Bound {
                inner: NonNull::new(ptr)?.cast(),
                root: PhantomData,
                mp,
            })
        } else {
            None
        }
    }

    pub fn bind<'obj, 'mp, T>(&'obj self, mp: MicroPython<'mp>) -> Bound<'obj, 'mp, T>
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

    pub fn bind<'mp>(&self, mp: MicroPython<'mp>) -> Bound<'r, 'mp, T> {
        Bound {
            inner: self.inner,
            root: PhantomData,
            mp,
        }
    }
}

impl<'r, 'mp, T> Bound<'r, 'mp, T> {
    pub const fn as_rooted(self) -> Rooted<'r, T> {
        Rooted {
            inner: self.inner,
            root: PhantomData,
        }
    }

    pub const fn mp(self) -> MicroPython<'mp> {
        self.mp
    }
}

impl<T: ?Sized> Deref for Bound<'_, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl<'r, 'mp> Bound<'r, 'mp, Obj> {
    pub fn ty(self) -> Bound<'r, 'mp, Type> {
        Bound {
            // SAFETY: `Obj` is a valid object
            inner: unsafe {
                NonNull::new_unchecked(mp_obj_get_type(self.deref().inner) as *mut Type)
            },
            mp: self.mp,
            root: PhantomData,
        }
    }

    pub fn try_downcast<T>(self) -> Option<Bound<'r, 'mp, T>>
    where
        T: Class,
    {
        let obj = self.deref().inner;
        if !tagging::is_ptr(obj) {
            return None;
        }

        let base = obj as *const mp_obj_base_t;
        let type_match = unsafe { (*base).type_ == T::TYPE_OBJECT.as_raw() as *const _ };
        if type_match {
            Some(Bound {
                inner: NonNull::new(obj)?.cast(),
                root: PhantomData,
                mp: self.mp,
            })
        } else {
            None
        }
    }

    pub fn downcast<T>(self) -> Bound<'r, 'mp, T>
    where
        T: Class,
    {
        self.try_downcast().unwrap_or_else(|| panic!(""))
    }
}

impl<'r, 'mp, T> Bound<'r, 'mp, [T]> {
    /// Bind a rooted slice supplied by the VM.
    ///
    /// # Safety
    /// The slice must remain live and rooted for `'r`. Its contents must be
    /// valid `T` values, and shared access must be permitted while `mp` is live.
    #[doc(hidden)]
    pub unsafe fn from_rooted_slice(slice: &'r [T], mp: MicroPython<'mp>) -> Self {
        Self {
            inner: NonNull::from_ref(slice),
            root: PhantomData,
            mp,
        }
    }

    pub fn get(self, index: usize) -> Option<Bound<'r, 'mp, T>> {
        let slice = self.deref();
        slice.get(index).map(|elem| Bound {
            inner: NonNull::from_ref(elem),
            root: PhantomData,
            mp: self.mp,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = Bound<'r, 'mp, T>> {
        self.deref().iter().map(|elem| Bound {
            inner: NonNull::from_ref(elem),
            root: PhantomData,
            mp: self.mp,
        })
    }

    pub fn chunks_exact(&self, chunk_size: usize) -> impl Iterator<Item = Bound<'r, 'mp, [T]>> {
        self.deref().chunks_exact(chunk_size).map(|elem| Bound {
            inner: NonNull::from_ref(elem),
            root: PhantomData,
            mp: self.mp,
        })
    }
}

#[derive(Clone, Copy)]
pub struct PyResult {
    inner: mprs_function_ret_t,
}

impl PyResult {
    // SAFETY: mp_obj_t must be a valid object now, non dangling if it is a pointer
    pub const unsafe fn new(ok: bool, value: mp_obj_t) -> Self {
        Self {
            inner: mprs_function_ret_t { ok, value },
        }
    }

    pub const fn inner(&self) -> mprs_function_ret_t {
        self.inner
    }
}

pub unsafe trait IntoRawObj {
    fn into_raw_obj(self, mp: MicroPythonMut, gc: Gc) -> mp_obj_t;
}

pub trait IntoPyResult {
    fn into_py_result(self, mp: MicroPythonMut, gc: Gc) -> PyResult;
}

unsafe impl IntoRawObj for mp_int_t {
    fn into_raw_obj(self, _mp: MicroPythonMut, _gc: Gc) -> mp_obj_t {
        Obj::from_small_int(self).inner
    }
}

unsafe impl IntoRawObj for Qstr {
    fn into_raw_obj(self, _mp: MicroPythonMut, _gc: Gc) -> mp_obj_t {
        Obj::from_qstr(self).inner
    }
}

unsafe impl IntoRawObj for () {
    fn into_raw_obj(self, _mp: MicroPythonMut, _gc: Gc) -> mp_obj_t {
        Obj::NONE.inner
    }
}

unsafe impl IntoRawObj for bool {
    fn into_raw_obj(self, _mp: MicroPythonMut, _gc: Gc) -> mp_obj_t {
        match self {
            true => Obj::TRUE.inner,
            false => Obj::FALSE.inner,
        }
    }
}

unsafe impl<T: IntoRawObj> IntoRawObj for Option<T> {
    fn into_raw_obj(self, mp: MicroPythonMut, gc: Gc) -> mp_obj_t {
        match self {
            Some(v) => v.into_raw_obj(mp, gc),
            None => Obj::NONE.inner,
        }
    }
}

impl<T: IntoRawObj> IntoPyResult for T {
    fn into_py_result(self, mp: MicroPythonMut, gc: Gc) -> PyResult {
        unsafe { PyResult::new(true, self.into_raw_obj(mp, gc)) }
    }
}

impl<T: IntoRawObj, E: IntoRawObj> IntoPyResult for Result<T, E> {
    fn into_py_result(self, mp: MicroPythonMut, gc: Gc) -> PyResult {
        unsafe {
            match self {
                Ok(v) => PyResult::new(true, v.into_raw_obj(mp, gc)),
                Err(e) => PyResult::new(false, e.into_raw_obj(mp, gc)),
            }
        }
    }
}
