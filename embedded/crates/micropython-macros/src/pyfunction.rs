use proc_macro2::TokenStream;
use syn::{Expr, FnArg, Ident, ItemFn, Pat, Safety, Type, ext::IdentExt, spanned::Spanned};

use crate::sig::{Elem, Signature};

pub struct PyfunctionArgs {
    pub signature: Option<Signature>,
}

enum ArgKind {
    Positional,
    PositionalOrKeyword,
    Keyword,
}

enum ValidatedArg {
    Token(TokenType),
    PythonArg(PythonArg),
    StarArgs(Ident),
    StarKwargs(Ident),
}

struct PythonArg {
    name: Ident,
    ty: Box<Type>,
    kind: ArgKind,
    default_value: Option<Expr>,
}

enum TokenType {
    MicroPython,
    MicroPythonMut,
    Gc,
}

fn is_token(ty: &Type) -> Option<TokenType> {
    match ty {
        Type::Path(p) => match p.path.segments.last()?.ident.to_string().as_str() {
            "MicroPython" => Some(TokenType::MicroPython),
            "MicroPythonMut" => Some(TokenType::MicroPythonMut),
            "Gc" => Some(TokenType::Gc),
            _ => None,
        },
        _ => None,
    }
}

struct ValidatedSignature {
    args: Vec<ValidatedArg>,
}

fn validate_signature(
    rust_sig: syn::Signature,
    py_sig: Option<Signature>,
) -> syn::Result<ValidatedSignature> {
    // Build Python-facing arguments first, then merge them into Rust order so
    // context tokens can appear anywhere without consuming signature entries.
    let mut python_args = Vec::new();
    if let Some(signature) = py_sig {
        let mut slash_seen = false;
        let mut keyword_only = false;
        let mut kwargs_seen = false;
        let mut default_seen = false;
        let mut bare_star = None;
        let mut keyword_count = 0;
        let mut names = std::collections::HashSet::new();

        for elem in signature.elems {
            let span = match &elem {
                Elem::Slash(t) => t.span(),
                Elem::Star(t) => t.span(),
                Elem::Arg(a) => a.name.span(),
                Elem::StarArgs(a) => a.ident.span(),
                Elem::StarKwargs(a) => a.ident.span(),
            };
            if kwargs_seen {
                return Err(syn::Error::new(span, "**kwargs must be last"));
            }
            let name = match &elem {
                Elem::Arg(a) => Some(&a.name),
                Elem::StarArgs(a) => Some(&a.ident),
                Elem::StarKwargs(a) => Some(&a.ident),
                _ => None,
            };
            if let Some(name) = name {
                if !names.insert(name.unraw().to_string()) {
                    return Err(syn::Error::new(span, "duplicate signature argument"));
                }
            }
            match elem {
                Elem::Slash(_) => {
                    if slash_seen || keyword_only || python_args.is_empty() {
                        return Err(syn::Error::new(
                            span,
                            "`/` must follow positional arguments and precede `*`",
                        ));
                    }
                    slash_seen = true;
                    for arg in &mut python_args {
                        if let ValidatedArg::PythonArg(arg) = arg {
                            arg.kind = ArgKind::Positional;
                        }
                    }
                }
                Elem::Star(_) => {
                    if keyword_only {
                        return Err(syn::Error::new(span, "`*` may only appear once"));
                    }
                    keyword_only = true;
                    bare_star = Some(span);
                }
                Elem::StarArgs(a) => {
                    if keyword_only {
                        return Err(syn::Error::new(span, "`*` may only appear once"));
                    }
                    keyword_only = true;
                    python_args.push(ValidatedArg::StarArgs(a.ident));
                }
                Elem::StarKwargs(a) => {
                    kwargs_seen = true;
                    python_args.push(ValidatedArg::StarKwargs(a.ident));
                }
                Elem::Arg(a) => {
                    if keyword_only {
                        keyword_count += 1;
                    } else {
                        if default_seen && a.default.is_none() {
                            return Err(syn::Error::new(
                                span,
                                "required positional argument follows an argument with a default",
                            ));
                        }
                        default_seen |= a.default.is_some();
                    }
                    python_args.push(ValidatedArg::PythonArg(PythonArg {
                        name: a.name,
                        // Filled from the matching Rust parameter below.
                        ty: Box::new(syn::parse_quote!(())),
                        kind: if keyword_only {
                            ArgKind::Keyword
                        } else {
                            ArgKind::PositionalOrKeyword
                        },
                        default_value: a.default.map(|d| d.value),
                    }));
                }
            }
        }
        if let Some(span) = bare_star {
            if keyword_count == 0 {
                return Err(syn::Error::new(
                    span,
                    "bare `*` must be followed by a keyword-only argument",
                ));
            }
        }
    } else {
        for input in &rust_sig.inputs {
            if let FnArg::Typed(arg) = input {
                if is_token(&arg.ty).is_none() {
                    python_args.push(ValidatedArg::PythonArg(PythonArg {
                        name: parameter_name(&arg.pat)?.clone(),
                        ty: arg.ty.clone(),
                        kind: ArgKind::PositionalOrKeyword,
                        default_value: None,
                    }));
                }
            }
        }
    }

    let mut python_args = python_args.into_iter();
    let mut args = Vec::new();
    for input in rust_sig.inputs {
        let FnArg::Typed(arg) = input else {
            return Err(syn::Error::new(
                input.span(),
                "freestanding pyfunction cannot have a receiver",
            ));
        };
        let name = parameter_name(&arg.pat)?;
        if let Some(token) = is_token(&arg.ty) {
            args.push(ValidatedArg::Token(token));
            continue;
        }
        let Some(mut validated) = python_args.next() else {
            return Err(syn::Error::new(
                name.span(),
                "Rust parameter is missing from the Python signature",
            ));
        };
        let expected = match &mut validated {
            ValidatedArg::PythonArg(a) => {
                a.ty = arg.ty;
                &a.name
            }
            ValidatedArg::StarArgs(name) | ValidatedArg::StarKwargs(name) => name,
            ValidatedArg::Token(_) => unreachable!(),
        };
        if name.unraw() != expected.unraw() {
            return Err(syn::Error::new(
                expected.span(),
                format!(
                    "expected Rust parameter `{}` in this position (context tokens must be omitted)",
                    name.unraw()
                ),
            ));
        }
        args.push(validated);
    }
    if let Some(extra) = python_args.next() {
        let name = match &extra {
            ValidatedArg::PythonArg(a) => &a.name,
            ValidatedArg::StarArgs(name) | ValidatedArg::StarKwargs(name) => name,
            ValidatedArg::Token(_) => unreachable!(),
        };
        return Err(syn::Error::new(
            name.span(),
            "signature argument has no matching Rust parameter",
        ));
    }
    Ok(ValidatedSignature { args })
}

fn parameter_name(pat: &Pat) -> syn::Result<&Ident> {
    match pat {
        Pat::Ident(p) if p.by_ref.is_none() && p.subpat.is_none() => Ok(&p.ident),
        _ => Err(syn::Error::new(
            pat.span(),
            "pyfunction parameters must be identifiers (optionally `mut`)",
        )),
    }
}

pub fn generate(args: PyfunctionArgs, f: ItemFn) -> syn::Result<TokenStream> {
    if let Safety::Unsafe(unsafe_token) = f.sig.safety {
        return Err(syn::Error::new(
            unsafe_token.span,
            "function cannot be unsafe",
        ));
    }

    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate(rust: &str, python: Option<&str>) -> syn::Result<ValidatedSignature> {
        validate_signature(
            syn::parse_str(rust).unwrap(),
            python.map(|s| syn::parse_str(s).unwrap()),
        )
    }

    #[test]
    fn classifies_arguments_and_skips_tokens() {
        let result = validate(
            "fn f(py: MicroPython<'_>, a: i32, b: i32, gc: Gc<'_>, rest: Args, c: i32, kw: Kwargs, pm: MicroPythonMut<'_>)",
            Some("(a, /, b = 42, *rest, c, **kw)"),
        ).unwrap();
        assert_eq!(result.args.len(), 8);
        assert!(matches!(
            result.args[0],
            ValidatedArg::Token(TokenType::MicroPython)
        ));
        assert!(
            matches!(&result.args[1], ValidatedArg::PythonArg(a) if matches!(a.kind, ArgKind::Positional) && a.default_value.is_none())
        );
        assert!(
            matches!(&result.args[2], ValidatedArg::PythonArg(a) if matches!(a.kind, ArgKind::PositionalOrKeyword) && a.default_value.is_some())
        );
        assert!(matches!(result.args[3], ValidatedArg::Token(TokenType::Gc)));
        assert!(matches!(result.args[4], ValidatedArg::StarArgs(_)));
        assert!(
            matches!(&result.args[5], ValidatedArg::PythonArg(a) if matches!(a.kind, ArgKind::Keyword))
        );
        assert!(matches!(result.args[6], ValidatedArg::StarKwargs(_)));
        assert!(matches!(
            result.args[7],
            ValidatedArg::Token(TokenType::MicroPythonMut)
        ));
    }

    #[test]
    fn rejects_invalid_layouts() {
        for signature in [
            "(/, a, b)",
            "(a, /, /, b)",
            "(a, *, /, b)",
            "(a, *, *b)",
            "(*a, *b)",
            "(a, *)",
            "(*, **b)",
            "(**a, b)",
            "(**a, **b)",
            "(a, a)",
            "(a = 1, b)",
            "(a = 1, /, b)",
            "(b, a)",
            "(a)",
            "(a, b, c)",
        ] {
            assert!(
                validate("fn f(a: i32, b: i32)", Some(signature)).is_err(),
                "{signature}"
            );
        }
    }

    #[test]
    fn accepts_defaults_and_keyword_only_arguments() {
        for signature in [
            "(a, b)",
            "(a, /, b)",
            "(a, b, /)",
            "(a = 1, *, b)",
            "(*, a = 1, b)",
            "(*a, **b)",
        ] {
            assert!(
                validate("fn f(a: i32, b: i32)", Some(signature)).is_ok(),
                "{signature}"
            );
        }
        assert!(validate("fn f()", Some("()")).is_ok());
        assert!(validate("fn f(py: some::MicroPython<'_>)", Some("()")).is_ok());
        assert!(validate("fn f(mut r#type: i32)", Some("(r#type)")).is_ok());
    }

    #[test]
    fn validates_rust_parameters() {
        assert!(validate("fn f(a: i32, gc: Gc<'_>)", None).is_ok());
        assert!(validate("fn f(&self)", None).is_err());
        assert!(validate("fn f((a, b): (i32, i32))", None).is_err());
        assert!(validate("fn f(py: MicroPython<'_>)", Some("(py)")).is_err());
        assert!(syn::parse_str::<Signature>("(a: i32)").is_err());
    }
}
