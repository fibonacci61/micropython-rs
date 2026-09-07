use core::marker::PhantomData;

pub struct Gc {
    _not_send: PhantomData<*mut ()>,
}

impl Gc {
    pub unsafe fn new() -> Self {
        Self {
            _not_send: PhantomData,
        }
    }
}
