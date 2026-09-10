use syn::{
    Expr, Ident, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};

pub enum Elem {
    Slash(Token![/]),
    Star(Token![*]),
    Arg(Arg),
    StarArgs(StarArgs),
    StarKwargs(StarKwargs),
}

pub struct StarArgs {
    pub star: Token![*],
    pub ident: Ident,
}

pub struct StarKwargs {
    pub star0: Token![*],
    pub star1: Token![*],
    pub ident: Ident,
}

pub struct DefaultAssignment {
    pub eq: Token![=],
    pub value: Expr,
}

pub struct Arg {
    pub name: Ident,
    pub default: Option<DefaultAssignment>,
}

pub struct Signature {
    pub paren_token: token::Paren,
    pub elems: Punctuated<Elem, Token![,]>,
}

impl Parse for Elem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![/]) {
            return Ok(Self::Slash(input.parse()?));
        }

        if input.peek(Token![*]) {
            let star = input.parse()?;
            if input.peek(Token![*]) {
                return Ok(Self::StarKwargs(StarKwargs {
                    star0: star,
                    star1: input.parse()?,
                    ident: input.parse()?,
                }));
            }
            if input.peek(Ident) {
                return Ok(Self::StarArgs(StarArgs {
                    star,
                    ident: input.parse()?,
                }));
            }
            return Ok(Self::Star(star));
        }

        let name = input.parse()?;
        let default = if input.peek(Token![=]) {
            Some(DefaultAssignment {
                eq: input.parse()?,
                value: input.parse()?,
            })
        } else {
            None
        };
        Ok(Self::Arg(Arg { name, default }))
    }
}

impl Parse for Signature {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let paren_token = syn::parenthesized!(content in input);
        let elems = content.parse_terminated(Elem::parse, Token![,])?;
        Ok(Self { paren_token, elems })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_empty_signature() {
        let signature = syn::parse_str::<Signature>("()").unwrap();
        assert!(signature.elems.is_empty());
    }

    #[test]
    fn parses_all_element_kinds() {
        let signature = syn::parse_str::<Signature>("(a, /, b = 42, *args, c, **kwargs,)").unwrap();
        let elems: Vec<_> = signature.elems.iter().collect();
        assert_eq!(elems.len(), 6);
        assert!(matches!(elems[0], Elem::Arg(arg) if arg.name == "a" && arg.default.is_none()));
        assert!(matches!(elems[1], Elem::Slash(_)));
        assert!(matches!(elems[2], Elem::Arg(arg) if arg.name == "b"
            && matches!(&arg.default.as_ref().unwrap().value, Expr::Lit(_))));
        assert!(matches!(elems[3], Elem::StarArgs(arg) if arg.ident == "args"));
        assert!(matches!(elems[4], Elem::Arg(arg) if arg.name == "c"));
        assert!(matches!(elems[5], Elem::StarKwargs(arg) if arg.ident == "kwargs"));
        assert!(signature.elems.trailing_punct());
    }

    #[test]
    fn parses_bare_star_and_expression_defaults() {
        let signature = syn::parse_str::<Signature>("(*, a = call(1, 2), b = (3, 4))").unwrap();
        let elems: Vec<_> = signature.elems.iter().collect();
        assert_eq!(elems.len(), 3);
        assert!(matches!(elems[0], Elem::Star(_)));
        assert!(matches!(elems[1], Elem::Arg(arg)
            if matches!(&arg.default.as_ref().unwrap().value, Expr::Call(_))));
        assert!(matches!(elems[2], Elem::Arg(arg)
            if matches!(&arg.default.as_ref().unwrap().value, Expr::Tuple(_))));
    }

    #[test]
    fn rejects_malformed_signatures() {
        for source in [
            "a",
            "(a",
            "(a b)",
            "(,)",
            "(a,,)",
            "(a =)",
            "(**)",
            "(***args)",
            "(*args = 1)",
            "(**kwargs = 1)",
            "(/ = 1)",
            "() extra",
        ] {
            assert!(syn::parse_str::<Signature>(source).is_err(), "{source}");
        }
    }
}
