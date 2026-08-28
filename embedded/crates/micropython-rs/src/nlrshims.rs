#[allow(non_camel_case_types)]
mod raw {
    include!(concat!(env!("OUT_DIR"), "/nlrshim_bindings.rs"));
}

pub use raw::*;
