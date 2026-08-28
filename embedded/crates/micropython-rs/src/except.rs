use crate::obj::Restricted;

pub type RstResult<'py, T> = Result<T, Restricted<'py>>;
