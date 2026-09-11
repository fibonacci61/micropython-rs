use micropython_sys::{mp_obj_base_t, mp_obj_dict_t, mp_type_dict};

use crate::{map::Map, obj::Class, ty::Type};

#[repr(transparent)]
pub struct Dict {
    dict: mp_obj_dict_t,
}

unsafe impl Class for Dict {
    const TYPE_OBJECT: &'static Type = unsafe { Type::from_raw(&raw const mp_type_dict) };
}

impl Dict {
    pub const fn new(map: Map) -> Self {
        Self {
            dict: mp_obj_dict_t {
                base: mp_obj_base_t {
                    type_: Self::TYPE_OBJECT.as_raw() as *const _,
                },
                map: map.into_raw(),
            },
        }
    }
}
