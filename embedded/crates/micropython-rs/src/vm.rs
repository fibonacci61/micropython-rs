use core::{
    error::Error,
    fmt::Display,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use micropython_sys::{gc_init, mp_deinit, mp_init};

static ACTIVE: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

pub fn generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
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
}

pub struct Vm<'h> {
    inner: Option<VmInner<'h>>,
}

pub struct Deinitialized<'h> {
    inner: Option<VmInner<'h>>,
}

pub struct MicroPython<'py> {
    _vm: PhantomData<Vm<'py>>,
    _not_send: PhantomData<*mut ()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitError;

impl Display for InitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("vm currently initialized, cannot initialize again")
    }
}

impl Error for InitError {}

impl<'h> VmInner<'h> {
    fn init(&mut self) -> Result<(), InitError> {
        if ACTIVE
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(InitError);
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

        GENERATION.fetch_add(1, Ordering::Relaxed);
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
        let mut vm_inner = VmInner { data: self.data };
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

    pub fn micropython<'py>(&'py self) -> MicroPython<'py> {
        MicroPython {
            _vm: PhantomData,
            _not_send: PhantomData,
        }
    }

    pub fn deinit(mut self) -> Deinitialized<'h> {
        let mut inner = self.inner.take().unwrap();
        inner.deinit();
        Deinitialized { inner: Some(inner) }
    }
}

impl<'h> Deinitialized<'h> {
    pub fn reinit(mut self) -> Result<Vm<'h>, InitError> {
        let mut inner = self.inner.take().unwrap();
        inner.init()?;
        Ok(Vm { inner: Some(inner) })
    }

    pub fn into_inner(mut self) -> VmData<'h> {
        self.inner.take().unwrap().data
    }
}

impl Drop for Vm<'_> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            inner.deinit();
        }
    }
}

impl Drop for Deinitialized<'_> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            inner.deinit();
        }
    }
}
