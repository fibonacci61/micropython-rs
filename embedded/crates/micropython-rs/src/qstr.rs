use core::str;

use micropython_sys::{qstr, qstr_data};

use crate::vm::MicroPython;

#[derive(Clone, Copy, PartialEq, Eq)]
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

    pub fn str<'py>(self, _mp: &'py MicroPython) -> &'py str {
        let mut len = 0;
        let bytes = unsafe { qstr_data(self.inner, &raw mut len) };
        unsafe { str::from_utf8_unchecked(core::slice::from_raw_parts(bytes, len)) }
    }
}

#[macro_export]
#[cfg(micropython_rs_qstr_scan)]
macro_rules! qstr {
    ($q:literal) => {
        ("__MICROPYTHON_RS_QSTR_VALUE__", $q)
    };
}
