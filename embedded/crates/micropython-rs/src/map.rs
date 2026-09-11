use micropython_sys::{mp_map_elem_t, mp_map_t};

#[repr(transparent)]
pub struct Map {
    map: mp_map_t,
}

impl Map {
    pub const unsafe fn from_raw(
        all_keys_are_qstrs: bool,
        is_fixed: bool,
        is_ordered: bool,
        used: usize,
        alloc: usize,
        table: *mut mp_map_elem_t,
    ) -> Self {
        // need to manually construct bitfield because bindgen functions are not const
        let used = used & (usize::MAX >> 3);
        let bitfield = if cfg!(target_endian = "big") {
            ((all_keys_are_qstrs as usize) << (usize::BITS - 1))
                | ((is_fixed as usize) << (usize::BITS - 2))
                | ((is_ordered as usize) << (usize::BITS - 3))
                | used
        } else {
            (all_keys_are_qstrs as usize)
                | ((is_fixed as usize) << 1)
                | ((is_ordered as usize) << 2)
                | (used << 3)
        };

        Self {
            map: mp_map_t {
                _bitfield_align_1: [],
                _bitfield_1: micropython_sys::__BindgenBitfieldUnit::new(bitfield.to_ne_bytes()),
                alloc,
                table,
            },
        }
    }
}
