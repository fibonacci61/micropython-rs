use core::marker::PhantomData;

use micropython_sys::{gc_init, mp_deinit, mp_init};

use crate::gc::Heap;

#[derive(Default)]
pub struct VmBuilder {
    #[cfg(micropython = "MICROPY_ENABLE_GC")]
    heap: Option<Heap>,
}

pub struct Vm {
    #[cfg(micropython = "MICROPY_ENABLE_GC")]
    heap: Option<Heap>,
}

pub struct MicroPython<'py> {
    _vm: PhantomData<&'py Vm>,
    _not_send: PhantomData<*mut ()>,
}

pub struct Deinitialized {
    #[cfg(micropython = "MICROPY_ENABLE_GC")]
    heap: Option<Heap>,
}

impl VmBuilder {
    #[cfg(micropython = "MICROPY_ENABLE_GC")]
    pub fn heap(mut self, heap: Heap) -> Self {
        self.heap = Some(heap);
        self
    }

    pub fn build(self) -> Vm {
        #[cfg(micropython = "MICROPY_ENABLE_GC")]
        if let Some(heap) = self.heap.as_ref() {
            unsafe { gc_init(heap.start().as_ptr().cast(), heap.end().as_ptr().cast()) };
        }

        unsafe { mp_init() };

        Vm {
            #[cfg(micropython = "MICROPY_ENABLE_GC")]
            heap: self.heap,
        }
    }
}

impl Vm {
    pub fn builder() -> VmBuilder {
        VmBuilder::default()
    }

    pub fn micropython<'py>(&'py self) -> MicroPython<'py> {
        MicroPython {
            _vm: PhantomData,
            _not_send: PhantomData,
        }
    }

    pub fn deinit(self) -> Deinitialized {
        unsafe { mp_deinit() };
        Deinitialized {
            #[cfg(micropython = "MICROPY_ENABLE_GC")]
            heap: self.heap,
        }
    }
}

impl Deinitialized {
    pub fn reinit(self) -> Vm {
        Vm {
            #[cfg(micropython = "MICROPY_ENABLE_GC")]
            heap: self.heap,
        }
    }
}
