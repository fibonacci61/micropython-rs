use core::{mem::MaybeUninit, ptr::NonNull};

pub struct Heap {
    start: NonNull<MaybeUninit<u8>>,
    end: NonNull<MaybeUninit<u8>>,
}

impl Heap {
    pub fn from_static(memory: &'static mut [MaybeUninit<u8>]) -> Self {
        let start = NonNull::from(&mut *memory).cast::<MaybeUninit<u8>>();
        let end = unsafe { NonNull::new_unchecked(start.as_ptr().add(memory.len())) };
        Self { start, end }
    }

    pub fn start(&self) -> NonNull<MaybeUninit<u8>> {
        self.start
    }

    pub fn end(&self) -> NonNull<MaybeUninit<u8>> {
        self.end
    }
}
