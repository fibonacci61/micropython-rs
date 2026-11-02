#![no_std]

use core::marker::PhantomData;

use micropython_core::{
    except::{TYPE_ERROR_TYPE, VALUE_ERROR_TYPE, new_exception_msg},
    obj::{Bound, Class, IntoPyResult, Obj, PyResult, Rooted},
    qstr::Qstr,
    shims,
    vm::{Gc, MicroPython, MicroPythonMut},
};

#[derive(Clone, Copy)]
pub enum ParseError {
    TypeError { expected: Qstr, found: Qstr },
    // TODO: add message
    ValueError {},
    MissingArgument,
    DuplicateArgument,
    ExcessPositional,
    UnexpectedKeyword,
}

impl IntoPyResult for ParseError {
    fn into_py_result(self, mp: MicroPythonMut, gc: Gc) -> PyResult {
        let (ty, msg) = match self {
            Self::TypeError { .. } => (TYPE_ERROR_TYPE, "argument has incorrect type"),
            Self::ValueError { .. } => (VALUE_ERROR_TYPE, "invalid argument value"),
            Self::MissingArgument => (TYPE_ERROR_TYPE, "missing required argument"),
            Self::DuplicateArgument => (TYPE_ERROR_TYPE, "multiple values for argument"),
            Self::ExcessPositional => (TYPE_ERROR_TYPE, "too many positional arguments"),
            Self::UnexpectedKeyword => (TYPE_ERROR_TYPE, "unexpected keyword argument"),
        };
        let exception = new_exception_msg(mp, gc, ty, todo!());
        unsafe { PyResult::new(false, exception.as_obj().into_raw()) }
    }
}

pub trait Parser {
    type Output<'r>;

    fn parse<'r>(
        &self,
        value: Rooted<'r, Obj>,
        mp: MicroPython,
    ) -> Result<Self::Output<'r>, ParseError>;
}

pub trait Argument<'r, 'mp> {
    type Parser: Parser + Default;

    fn finalize(value: <Self::Parser as Parser>::Output<'r>, mp: MicroPython<'mp>) -> Self;
}

pub struct ClassParser<T: Class> {
    phantom: PhantomData<T>,
}

impl<T: Class> Default for ClassParser<T> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T: Class> Parser for ClassParser<T> {
    type Output<'r> = Rooted<'r, T>;

    fn parse<'r>(
        &self,
        value: Rooted<'r, Obj>,
        mp: MicroPython,
    ) -> Result<Self::Output<'r>, ParseError> {
        Ok(value
            .bind(mp)
            .try_downcast::<T>()
            .ok_or(ParseError::TypeError {
                expected: T::TYPE_OBJECT.name(),
                found: value.bind(mp).ty().name(),
            })?
            .as_rooted())
    }
}

impl<'r, 'mp, T: Class> Argument<'r, 'mp> for Bound<'r, 'mp, T> {
    type Parser = ClassParser<T>;

    fn finalize(value: <Self::Parser as Parser>::Output<'r>, mp: MicroPython<'mp>) -> Self {
        value.bind(mp)
    }
}

impl<'r, 'mp, T: Class> Argument<'r, 'mp> for Rooted<'r, T> {
    type Parser = ClassParser<T>;

    fn finalize(value: Rooted<'r, T>, _mp: MicroPython<'mp>) -> Self {
        value
    }
}

pub struct Args<'r, 'mp> {
    pos_args: Bound<'r, 'mp, [Obj]>,
    kw_args: Bound<'r, 'mp, [Obj]>,
}

pub struct ArgsReader<'r, 'mp> {
    args: Args<'r, 'mp>,
    i_pos: usize,
    keywords_used: usize,
}

pub struct KwArg<'r, 'mp> {
    pub name: Qstr,
    pub value: Bound<'r, 'mp, Obj>,
}

impl<'r, 'mp> Args<'r, 'mp> {
    pub const fn new(pos_args: Bound<'r, 'mp, [Obj]>, kw_args: Bound<'r, 'mp, [Obj]>) -> Self {
        Self { pos_args, kw_args }
    }

    pub fn n_arg(&self) -> usize {
        self.pos_args.len()
    }

    pub fn n_kw(&self) -> usize {
        self.kw_args.len() / 2
    }

    pub fn nth_pos(&self, n: usize) -> Option<Bound<'r, 'mp, Obj>> {
        Bound::get(self.pos_args, n)
    }

    pub fn find_keyword(&self, keyword: Qstr) -> Option<Bound<'r, 'mp, Obj>> {
        // naive linear search, but since most functions have few keyword args this is fine
        // TODO: use hash map once keyword count exceeds a threshold
        for kwarg in Bound::chunks_exact(&self.kw_args, 2) {
            let name = Bound::get(kwarg, 0).unwrap().qstr().unwrap();
            if name == keyword {
                let value = Bound::get(kwarg, 1).unwrap();
                return Some(value);
            }
        }

        None
    }
}

impl<'r, 'mp> ArgsReader<'r, 'mp> {
    pub fn new(args: Args<'r, 'mp>) -> Result<Self, ParseError> {
        if args.kw_args.len() % 2 != 0 {
            return Err(ParseError::ValueError {});
        }
        // No allocation is necessary for the small keyword lists typical here.
        for i in 0..args.n_kw() {
            let name = args
                .kw_args
                .get(2 * i)
                .unwrap()
                .qstr()
                .ok_or(ParseError::ValueError {})?;
            for j in 0..i {
                if args.kw_args.get(2 * j).unwrap().qstr() == Some(name) {
                    return Err(ParseError::DuplicateArgument);
                }
            }
        }
        Ok(Self {
            args,
            i_pos: 0,
            keywords_used: 0,
        })
    }

    pub fn has_keyword(&self, keyword: Qstr) -> bool {
        self.args.find_keyword(keyword).is_some()
    }

    /// Check for arguments not consumed by the generated binder.
    pub fn finish(self) -> Result<(), ParseError> {
        if self.i_pos != self.args.n_arg() {
            return Err(ParseError::ExcessPositional);
        }
        if self.keywords_used != self.args.n_kw() {
            return Err(ParseError::UnexpectedKeyword);
        }
        Ok(())
    }

    pub fn next_positional_with<T, P>(&mut self, parser: P) -> Option<Result<T, ParseError>>
    where
        P: Parser<Output<'r> = T>,
    {
        self.args
            .nth_pos(self.i_pos)
            .map(|value| parser.parse(value.as_rooted(), value.mp()))
            .inspect(|_| self.i_pos += 1)
    }

    /// Look up and count a keyword as consumed. The binder must call this at
    /// most once per keyword name; `finish` checks the resulting count.
    pub fn find_keyword_with<T, P>(
        &mut self,
        keyword: Qstr,
        parser: P,
    ) -> Option<Result<T, ParseError>>
    where
        P: Parser<Output<'r> = T>,
    {
        self.args
            .find_keyword(keyword)
            .map(|value| parser.parse(value.as_rooted(), value.mp()))
            .inspect(|_| self.keywords_used += 1)
    }
}
