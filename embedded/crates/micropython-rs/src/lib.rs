#![no_std]

#[cfg(micropython = "MICROPY_ENABLE_GC")]
pub mod gc;
pub mod obj;
pub mod qstr;
pub mod vm;
