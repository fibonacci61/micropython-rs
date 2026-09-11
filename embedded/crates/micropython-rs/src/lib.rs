#![no_std]

pub mod alloc;
pub mod dict;
pub mod except;
pub mod function;
pub mod gc;
pub mod map;
pub mod obj;
pub mod qstr;
pub mod shims;
pub mod str;
pub mod ty;
pub mod vm;

pub use micropython_macros::RootProject;
pub use micropython_sys as sys;
pub use obj::RootProject;

#[cfg(not(micropython_rs_qstr_scan))]
pub use micropython_macros::qstr;
