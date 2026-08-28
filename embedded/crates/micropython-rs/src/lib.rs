#![no_std]

pub mod alloc;
pub mod except;
#[cfg(micropython = "MICROPY_ENABLE_GC")]
pub mod gc;
pub mod nlrshims;
pub mod obj;
pub mod qstr;
pub mod vm;

#[cfg(not(micropython_rs_qstr_scan))]
pub use micropython_macros::qstr;
