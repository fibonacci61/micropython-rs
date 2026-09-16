use core::{
    ffi::c_void,
    marker::PhantomData,
    mem::{align_of, offset_of, size_of},
    ops::Deref,
    ptr::{self, NonNull},
};

use micropython_sys::{self as sys, mp_obj_type_t};

use crate::{obj::Class, qstr::Qstr};

const MAX_SLOTS: usize = 12;
const ITER_MASK: u16 = sys::MP_TYPE_FLAG_ITER_IS_STREAM as u16;

#[repr(transparent)]
pub struct Type {
    ty: mp_obj_type_t,
    slots: PhantomData<[*const c_void; 0]>,
}

#[repr(C)]
pub struct TypeStorage<const SLOTS: usize> {
    ty: mp_obj_type_t,
    slots: [Option<NonNull<c_void>>; SLOTS],
}

// SAFETY: shared access only reads the header and slot values; accessing their
// pointees or invoking callbacks remains unsafe.
unsafe impl Sync for Type {}
unsafe impl<const SLOTS: usize> Sync for TypeStorage<SLOTS> {}

unsafe impl Class for Type {
    const TYPE_OBJECT: &'static Type = unsafe { Type::from_raw(&raw const sys::mp_type_type) };
}

const _: () = {
    assert!(size_of::<Option<NonNull<c_void>>>() == size_of::<*const c_void>());
    assert!(align_of::<Option<NonNull<c_void>>>() == align_of::<*const c_void>());
    assert!(offset_of!(TypeStorage<0>, slots) == offset_of!(mp_obj_type_t, slots));
    assert!(offset_of!(TypeStorage<MAX_SLOTS>, slots) == offset_of!(mp_obj_type_t, slots));
};

macro_rules! callback_slots {
    ($($get:ident, $set:ident, $index:ident, $ty:ident;)*) => {
        impl Type {
            $(
                pub const fn $get(&self) -> sys::$ty {
                    unsafe { core::mem::transmute(self.slot(self.ty.$index)) }
                }
            )*
        }

        impl<const SLOTS: usize> TypeStorage<SLOTS> {
            $(
                pub const unsafe fn $set(&mut self, value: sys::$ty) {
                    let value = match value {
                        Some(function) => NonNull::new(function as *const () as *mut c_void),
                        None => None,
                    };
                    set_slot(&mut self.slots, &mut self.ty.$index, value);
                }
            )*
        }
    };
}

callback_slots! {
    make_new_raw, set_make_new_raw, slot_index_make_new, mp_make_new_fun_t;
    print_raw, set_print_raw, slot_index_print, mp_print_fun_t;
    call_raw, set_call_raw, slot_index_call, mp_call_fun_t;
    unary_op_raw, set_unary_op_raw, slot_index_unary_op, mp_unary_op_fun_t;
    binary_op_raw, set_binary_op_raw, slot_index_binary_op, mp_binary_op_fun_t;
    attr_raw, set_attr_raw, slot_index_attr, mp_attr_fun_t;
    subscr_raw, set_subscr_raw, slot_index_subscr, mp_subscr_fun_t;
    buffer_raw, set_buffer_raw, slot_index_buffer, mp_buffer_fun_t;
}

impl Type {
    pub const unsafe fn from_raw<'a>(ty: *const mp_obj_type_t) -> &'a Self {
        unsafe { &*ty.cast() }
    }

    pub const fn as_raw(&self) -> &mp_obj_type_t {
        &self.ty
    }

    pub const fn name(&self) -> Qstr {
        unsafe { Qstr::from_raw(self.ty.name as _) }
    }

    pub const fn flags(&self) -> u16 {
        self.ty.flags
    }

    pub const fn slot_count(&self) -> usize {
        let indices = slot_indices(&self.ty);
        let mut count = 0;
        let mut i = 0;
        while i < indices.len() {
            if indices[i] != 0 {
                count += 1;
            }
            i += 1;
        }
        count
    }

    pub const fn getiter_raw(&self) -> sys::mp_getiter_fun_t {
        if self.ty.flags & ITER_MASK == sys::MP_TYPE_FLAG_ITER_IS_GETITER as u16 {
            unsafe { core::mem::transmute(self.slot(self.ty.slot_index_iter)) }
        } else {
            None
        }
    }

    pub const fn iternext_raw(&self) -> sys::mp_iternext_fun_t {
        if self.ty.flags & ITER_MASK == sys::MP_TYPE_FLAG_ITER_IS_ITERNEXT as u16 {
            unsafe { core::mem::transmute(self.slot(self.ty.slot_index_iter)) }
        } else {
            None
        }
    }

    pub const fn custom_iter_raw(&self) -> Option<NonNull<sys::mp_getiter_iternext_custom_t>> {
        if self.ty.flags & ITER_MASK == sys::MP_TYPE_FLAG_ITER_IS_CUSTOM as u16 {
            match self.slot(self.ty.slot_index_iter) {
                Some(ptr) => Some(ptr.cast()),
                None => None,
            }
        } else {
            None
        }
    }

    pub const fn is_stream_iter_raw(&self) -> bool {
        self.ty.flags & ITER_MASK == sys::MP_TYPE_FLAG_ITER_IS_STREAM as u16
    }

    pub const fn protocol_raw(&self) -> Option<NonNull<c_void>> {
        self.slot(self.ty.slot_index_protocol)
    }

    pub const fn parent_raw(&self) -> Option<NonNull<sys::mp_obj_base_t>> {
        match self.slot(self.ty.slot_index_parent) {
            Some(ptr) => Some(ptr.cast()),
            None => None,
        }
    }

    pub const fn locals_dict_raw(&self) -> Option<NonNull<sys::mp_obj_dict_t>> {
        match self.slot(self.ty.slot_index_locals_dict) {
            Some(ptr) => Some(ptr.cast()),
            None => None,
        }
    }

    const fn slot(&self, index: u8) -> Option<NonNull<c_void>> {
        if index == 0 {
            None
        } else {
            let slots = (&raw const self.slots).cast::<*const c_void>();
            unsafe { NonNull::new((*slots.add(index as usize - 1)).cast_mut()) }
        }
    }
}

impl<const SLOTS: usize> TypeStorage<SLOTS> {
    pub const fn new(name: Qstr, flags: u16) -> Self {
        assert!(SLOTS <= MAX_SLOTS, "too many type slots");
        assert!(
            name.into_raw() <= u16::MAX as _,
            "type name does not fit in u16"
        );
        Self {
            ty: mp_obj_type_t {
                base: sys::mp_obj_base_t {
                    type_: &raw const sys::mp_type_type,
                },
                flags,
                name: name.into_raw() as u16,
                slot_index_make_new: 0,
                slot_index_print: 0,
                slot_index_call: 0,
                slot_index_unary_op: 0,
                slot_index_binary_op: 0,
                slot_index_attr: 0,
                slot_index_subscr: 0,
                slot_index_iter: 0,
                slot_index_buffer: 0,
                slot_index_protocol: 0,
                slot_index_parent: 0,
                slot_index_locals_dict: 0,
                slots: sys::__IncompleteArrayField::new(),
            },
            slots: [None; SLOTS],
        }
    }

    pub const fn as_type(&self) -> &Type {
        unsafe { &*ptr::from_ref(self).cast() }
    }

    pub const fn as_raw(&self) -> &mp_obj_type_t {
        &self.ty
    }

    pub const fn capacity(&self) -> usize {
        SLOTS
    }

    pub const fn slot_count(&self) -> usize {
        self.as_type().slot_count()
    }

    pub const fn set_name(&mut self, name: Qstr) {
        assert!(
            name.into_raw() <= u16::MAX as _,
            "type name does not fit in u16"
        );
        self.ty.name = name.into_raw() as u16;
    }

    pub const unsafe fn set_flags(&mut self, flags: u16) {
        assert!(
            self.ty.slot_index_iter == 0 || self.ty.flags & ITER_MASK == flags & ITER_MASK,
            "use an iterator setter to change an active iterator's mode",
        );
        self.ty.flags = flags;
    }

    pub const unsafe fn set_getiter_raw(&mut self, value: sys::mp_getiter_fun_t) {
        let value = match value {
            Some(function) => NonNull::new(function as *const () as *mut c_void),
            None => None,
        };
        set_slot(&mut self.slots, &mut self.ty.slot_index_iter, value);
        self.ty.flags &= !ITER_MASK;
    }

    pub const unsafe fn set_iternext_raw(&mut self, value: sys::mp_iternext_fun_t) {
        let value = match value {
            Some(function) => NonNull::new(function as *const () as *mut c_void),
            None => None,
        };
        set_slot(&mut self.slots, &mut self.ty.slot_index_iter, value);
        self.ty.flags &= !ITER_MASK;
        if value.is_some() {
            self.ty.flags |= sys::MP_TYPE_FLAG_ITER_IS_ITERNEXT as u16;
        }
    }

    pub const unsafe fn set_custom_iter_raw(
        &mut self,
        value: Option<NonNull<sys::mp_getiter_iternext_custom_t>>,
    ) {
        let value = match value {
            Some(ptr) => Some(ptr.cast()),
            None => None,
        };
        set_slot(&mut self.slots, &mut self.ty.slot_index_iter, value);
        self.ty.flags &= !ITER_MASK;
        if value.is_some() {
            self.ty.flags |= sys::MP_TYPE_FLAG_ITER_IS_CUSTOM as u16;
        }
    }

    pub const unsafe fn set_stream_iter_raw(&mut self) {
        set_slot(&mut self.slots, &mut self.ty.slot_index_iter, None);
        self.ty.flags = (self.ty.flags & !ITER_MASK) | sys::MP_TYPE_FLAG_ITER_IS_STREAM as u16;
    }

    pub const unsafe fn set_protocol_raw(&mut self, value: Option<NonNull<c_void>>) {
        set_slot(&mut self.slots, &mut self.ty.slot_index_protocol, value);
    }

    pub const unsafe fn set_parent_raw(&mut self, value: Option<NonNull<mp_obj_type_t>>) {
        let value = match value {
            Some(ptr) => Some(ptr.cast()),
            None => None,
        };
        set_slot(&mut self.slots, &mut self.ty.slot_index_parent, value);
    }

    pub const unsafe fn set_parents_raw(&mut self, value: Option<NonNull<sys::mp_obj_tuple_t>>) {
        let value = match value {
            Some(ptr) => Some(ptr.cast()),
            None => None,
        };
        set_slot(&mut self.slots, &mut self.ty.slot_index_parent, value);
    }

    pub const unsafe fn set_locals_dict_raw(&mut self, value: Option<NonNull<sys::mp_obj_dict_t>>) {
        let value = match value {
            Some(ptr) => Some(ptr.cast()),
            None => None,
        };
        set_slot(&mut self.slots, &mut self.ty.slot_index_locals_dict, value);
    }

    pub const fn resize<const NEW_SLOTS: usize>(self) -> TypeStorage<NEW_SLOTS> {
        assert!(NEW_SLOTS <= MAX_SLOTS, "too many type slots");
        let mut indices = slot_indices(&self.ty);
        let mut slots = [None; NEW_SLOTS];
        let mut count = 0;
        let mut i = 0;
        while i < indices.len() {
            let index = indices[i];
            if index != 0 {
                assert!(
                    count < NEW_SLOTS,
                    "not enough capacity for active type slots"
                );
                slots[count] = self.slots[index as usize - 1];
                indices[i] = (count + 1) as u8;
                count += 1;
            }
            i += 1;
        }
        let mut ty = self.ty;
        ty.slot_index_make_new = indices[0];
        ty.slot_index_print = indices[1];
        ty.slot_index_call = indices[2];
        ty.slot_index_unary_op = indices[3];
        ty.slot_index_binary_op = indices[4];
        ty.slot_index_attr = indices[5];
        ty.slot_index_subscr = indices[6];
        ty.slot_index_iter = indices[7];
        ty.slot_index_buffer = indices[8];
        ty.slot_index_protocol = indices[9];
        ty.slot_index_parent = indices[10];
        ty.slot_index_locals_dict = indices[11];
        TypeStorage { ty, slots }
    }
}

impl<const SLOTS: usize> Deref for TypeStorage<SLOTS> {
    type Target = Type;

    fn deref(&self) -> &Self::Target {
        self.as_type()
    }
}

const fn slot_indices(ty: &mp_obj_type_t) -> [u8; MAX_SLOTS] {
    [
        ty.slot_index_make_new,
        ty.slot_index_print,
        ty.slot_index_call,
        ty.slot_index_unary_op,
        ty.slot_index_binary_op,
        ty.slot_index_attr,
        ty.slot_index_subscr,
        ty.slot_index_iter,
        ty.slot_index_buffer,
        ty.slot_index_protocol,
        ty.slot_index_parent,
        ty.slot_index_locals_dict,
    ]
}

const fn set_slot(
    slots: &mut [Option<NonNull<c_void>>],
    index: &mut u8,
    value: Option<NonNull<c_void>>,
) {
    if *index != 0 {
        slots[*index as usize - 1] = value;
        if value.is_none() {
            *index = 0;
        }
    } else if value.is_some() {
        let mut i = 0;
        while i < slots.len() {
            if slots[i].is_none() {
                slots[i] = value;
                *index = (i + 1) as u8;
                return;
            }
            i += 1;
        }
        panic!("type slot capacity exhausted");
    }
}
