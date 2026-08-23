use micropython_sys::qstr;

#[derive(Clone, Copy)]
pub struct Qstr {
    inner: qstr,
}

impl Qstr {
    pub const unsafe fn from_raw(q: qstr) -> Self {
        Self { inner: q }
    }

    pub const fn into_raw(self) -> qstr {
        self.inner
    }
}
