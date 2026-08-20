//! ABI types selected by MicroPython's `py/mpconfig.h`.

#[cfg(micropython = "MP_INT_TYPE_INTPTR")]
pub type mp_int_t = isize;
#[cfg(micropython = "MP_INT_TYPE_INTPTR")]
pub type mp_uint_t = usize;

#[cfg(micropython = "MP_INT_TYPE_INT64")]
pub type mp_int_t = i64;
#[cfg(micropython = "MP_INT_TYPE_INT64")]
pub type mp_uint_t = u64;

#[cfg(micropython = "MP_INT_TYPE_OTHER")]
compile_error!("micropython-sys does not support port-defined MP_INT_TYPE_OTHER integer types");

#[cfg(micropython = "MICROPY_FLOAT_IMPL_FLOAT")]
pub type mp_float_t = f32;
#[cfg(micropython = "MICROPY_FLOAT_IMPL_DOUBLE")]
pub type mp_float_t = f64;
