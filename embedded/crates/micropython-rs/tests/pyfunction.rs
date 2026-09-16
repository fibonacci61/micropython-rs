//! Compile coverage for generated trampolines
#![allow(dead_code)]

// proc_macro_crate resolves this package to `crate` in integration tests.
use dict::Dict;
pub use micropython_rs::*;
use obj::{Bound, Rooted};
use vm::{Gc, MicroPython, MicroPythonMut};

#[pyfunction]
fn empty() {}

#[pyfunction(signature = (value, /))]
fn shared<'r, 'mp>(mp: MicroPython<'mp>, value: Bound<'r, 'mp, Dict>, gc: Gc<'_>) -> bool {
    let _ = (mp, value, gc);
    true
}

#[pyfunction(signature = (value, /))]
fn exclusive(mp: MicroPythonMut<'_>, value: Rooted<'_, Dict>, gc: Gc<'_>) -> bool {
    let _ = (mp, value, gc);
    true
}

#[pyfunction(signature = (dict))]
fn keyword(dict: Bound<'_, '_, Dict>) {
    let _ = dict;
}

// A local parser exercises default expressions without requiring built-in
// primitive parsers (which are independent of trampoline generation).
struct Count(usize);
#[derive(Default)]
struct CountParser;
impl args::Parser for CountParser {
    type Output<'r> = Count;
    fn parse<'r>(
        &self,
        _: Rooted<'r, obj::Obj>,
        _: MicroPython<'_>,
    ) -> Result<Count, args::ParseError> {
        Ok(Count(1))
    }
}
impl<'r, 'mp> args::Argument<'r, 'mp> for Count {
    type Parser = CountParser;
    fn finalize(value: Count, _: MicroPython<'mp>) -> Self {
        value
    }
}

#[pyfunction(signature = (dict = Count(2), *, key = Count(3)))]
fn defaults(dict: Count, key: Count) -> bool {
    dict.0 == key.0
}

#[pyfunction(signature = (dict = Count(2), /))]
fn mutable_default(mp: MicroPythonMut<'_>, dict: Count) -> bool {
    let _ = mp;
    dict.0 == 2
}

#[pyfunction(signature = (r#type, /))]
fn raw_identifier(r#type: Bound<'_, '_, Dict>) -> Result<bool, bool> {
    let _ = r#type;
    Ok(true)
}

// The module must follow the function's cfg, not emit an unresolved call.
#[pyfunction]
#[cfg(any())]
fn disabled() {}

const _: &function::Function = &empty::FUNCTION;
const _: &function::Function = &shared::FUNCTION;
const _: &function::Function = &exclusive::FUNCTION;
const _: &function::Function = &keyword::FUNCTION;
const _: &function::Function = &defaults::FUNCTION;
