#[allow(non_camel_case_types)]
mod raw {
    include!(concat!(env!("OUT_DIR"), "/shims.rs"));
}

pub use raw::*;
