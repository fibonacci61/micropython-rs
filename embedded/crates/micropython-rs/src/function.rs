// perhaps this module belongs in a separate crate since it's a new type and not just a binding?

use crate::{
    obj::Class,
    shims::{mp_obj_base_t, mprs_function_t, mprs_obj_function_t, mprs_type_function},
    ty::Type,
};

#[repr(transparent)]
pub struct Function {
    inner: mprs_obj_function_t,
}

unsafe impl Class for Function {
    fn type_object() -> &'static Type {
        unsafe { Type::from_raw((&raw const mprs_type_function).cast()) }
    }
}

impl Function {
    pub const unsafe fn new(f: mprs_function_t) -> Self {
        Self {
            inner: mprs_obj_function_t {
                base: mp_obj_base_t {
                    type_: &raw const mprs_type_function,
                },
                function: f,
            },
        }
    }
}
