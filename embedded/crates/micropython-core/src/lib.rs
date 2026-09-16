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

pub use obj::RootProject;
