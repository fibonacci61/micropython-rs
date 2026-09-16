use crate::obj::Restricted;

pub type RstResult<'gc, T> = Result<T, Restricted<'gc>>;
