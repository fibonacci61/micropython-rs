use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

use micropython_sys::{gc_init, mp_deinit, mp_init};
use thiserror::Error;

static ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

#[derive(Default)]
pub struct VmData<'h> {
    #[cfg(micropython = "MICROPY_ENABLE_GC")]
    pub heap: Option<&'h mut [MaybeUninit<u8>]>,
    pub _heap_phantom: PhantomData<&'h mut [MaybeUninit<u8>]>,
}

#[derive(Default)]
pub struct VmBuilder<'h> {
    data: VmData<'h>,
}

struct VmInner<'h> {
    data: VmData<'h>,
    micropython: MicroPython,
}

pub struct Vm<'h> {
    inner: Option<VmInner<'h>>,
}

pub struct Deinitialized<'h> {
    inner: VmInner<'h>,
}

pub struct MicroPython {
    _not_send: PhantomData<*mut ()>,
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum InitError {
    #[error("vm currently initialized, cannot initialize again before deinit")]
    CurrentlyInitialized,
}

impl<'h> VmInner<'h> {
    fn init(&mut self) -> Result<(), InitError> {
        if ACTIVE
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(InitError::CurrentlyInitialized);
        }

        #[cfg(micropython = "MICROPY_ENABLE_GC")]
        if let Some(heap) = self.data.heap.as_mut() {
            unsafe {
                gc_init(
                    heap.as_mut_ptr().cast(),
                    heap.as_mut_ptr().add(heap.len()).cast(),
                )
            };
        }

        unsafe { mp_init() };
        Ok(())
    }

    fn deinit(&mut self) {
        unsafe { mp_deinit() };
        ACTIVE.store(false, Ordering::Release);
    }
}

impl<'h> VmBuilder<'h> {
    #[cfg(micropython = "MICROPY_ENABLE_GC")]
    pub fn heap(mut self, heap: &'h mut [MaybeUninit<u8>]) -> Self {
        self.data.heap = Some(heap);
        self
    }

    pub fn build(self) -> Result<Vm<'h>, InitError> {
        let mut vm_inner = VmInner {
            data: self.data,
            micropython: MicroPython {
                _not_send: PhantomData,
            },
        };
        vm_inner.init()?;

        Ok(Vm {
            inner: Some(vm_inner),
        })
    }
}

impl<'h> Vm<'h> {
    pub fn builder() -> VmBuilder<'h> {
        VmBuilder::default()
    }

    pub fn micropython(&mut self) -> &mut MicroPython {
        &mut self.inner.as_mut().unwrap().micropython
    }

    pub fn deinit(mut self) -> Deinitialized<'h> {
        let mut inner = self.inner.take().unwrap();
        inner.deinit();
        Deinitialized { inner }
    }
}

impl<'h> Deinitialized<'h> {
    pub fn reinit(mut self) -> Result<Vm<'h>, InitError> {
        self.inner.init()?;
        Ok(Vm {
            inner: Some(self.inner),
        })
    }

    pub fn into_data(self) -> VmData<'h> {
        self.inner.data
    }
}

impl Drop for Vm<'_> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            inner.deinit();
        }
    }
}

impl MicroPython {
    pub unsafe fn new() -> Self {
        Self {
            _not_send: PhantomData,
        }
    }
}
