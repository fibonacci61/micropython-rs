use std::marker::PhantomData;

use micropython_core::{
    obj::{Bound, Class, Obj, Rooted},
    qstr::Qstr,
    vm::MicroPython,
};

pub enum ParseError {
    TypeError { expected: Qstr, found: Qstr },
    // TODO: add message
    ValueError {},
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
    type Parser: Parser;

    fn finalize(value: <Self::Parser as Parser>::Output<'r>, mp: MicroPython<'mp>) -> Self;
}

pub struct ClassParser<T: Class> {
    phantom: PhantomData<T>,
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

pub struct Args<'r, 'mp> {
    pos_args: Bound<'r, 'mp, [Obj]>,
    kw_args: Bound<'r, 'mp, [Obj]>,
}

pub struct ArgsReader<'r, 'mp> {
    args: Args<'r, 'mp>,
    i_pos: usize,
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
        self.kw_args.len() * 2
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
    pub fn next_positional_with<T, P>(&mut self, parser: P) -> Option<Result<T, ParseError>>
    where
        P: Parser<Output<'r> = T>,
    {
        self.args
            .nth_pos(self.i_pos)
            .map(|value| parser.parse(value.as_rooted(), value.mp()))
            .inspect(|_| self.i_pos += 1)
    }

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
    }
}
