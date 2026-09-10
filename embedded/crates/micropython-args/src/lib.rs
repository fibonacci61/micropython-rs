use micropython_rs::{
    obj::{Bound, Obj},
    qstr::Qstr,
};

pub trait Parser<'b> {
    type Output;

    fn parse(&self, value: Bound<'b, Obj>) -> Result<Self::Output, ()>;
}

pub struct Args<'b> {
    pos_args: Bound<'b, [Obj]>,
    kw_args: Bound<'b, [Obj]>,
}

pub struct ArgsReader<'b> {
    args: Args<'b>,
    i_pos: usize,
}

pub struct KwArg<'b> {
    pub name: Qstr,
    pub value: Bound<'b, Obj>,
}

impl<'b> Args<'b> {
    pub const fn new(pos_args: Bound<'b, [Obj]>, kw_args: Bound<'b, [Obj]>) -> Self {
        Self { pos_args, kw_args }
    }

    pub fn n_arg(&self) -> usize {
        self.pos_args.len()
    }

    pub fn n_kw(&self) -> usize {
        self.kw_args.len() * 2
    }

    pub fn nth_pos(&self, n: usize) -> Option<Bound<'b, Obj>> {
        Bound::get(self.pos_args, n)
    }

    pub fn find_keyword(&self, keyword: Qstr) -> Option<Bound<'b, Obj>> {
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

impl<'b> ArgsReader<'b> {
    pub fn next_positional_with<T, P>(&mut self, parser: P) -> Result<T, ()>
    where
        P: Parser<'b, Output = T>,
    {
        self.args
            .nth_pos(self.i_pos)
            .ok_or(())
            .and_then(|value| parser.parse(value))
            .inspect(|_| self.i_pos += 1)
    }

    pub fn find_keyword_with<T, P>(&mut self, keyword: Qstr, parser: P) -> Result<T, ()>
    where
        P: Parser<'b, Output = T>,
    {
        self.args
            .find_keyword(keyword)
            .ok_or(())
            .and_then(|value| parser.parse(value))
    }
}
