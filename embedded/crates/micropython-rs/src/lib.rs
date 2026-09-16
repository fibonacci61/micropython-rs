#![no_std]

pub use micropython_args as args;
pub use micropython_core as core;
pub use micropython_core::*;
pub use micropython_macros as macros;
pub use micropython_macros::{RootProject, pyfunction};
pub use micropython_sys as sys;

#[cfg(not(micropython_rs_qstr_scan))]
pub use micropython_macros::qstr;

#[macro_export]
#[cfg(micropython_rs_qstr_scan)]
macro_rules! qstr {
    ($q:literal) => {
        ("__MICROPYTHON_RS_QSTR_VALUE__", $q)
    };
}
