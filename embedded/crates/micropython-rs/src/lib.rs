#![no_std]

pub mod alloc;
pub mod except;
pub mod gc;
pub mod obj;
pub mod qstr;
pub mod shims;
pub mod vm;

pub use micropython_macros::RootProject;
pub use micropython_sys as sys;
pub use obj::RootProject;

#[cfg(not(micropython_rs_qstr_scan))]
pub use micropython_macros::qstr;
