use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::visit_mut::VisitMut;
use syn::{Expr, FnArg, Ident, ItemFn, Pat, Safety, Type, ext::IdentExt, spanned::Spanned};

use crate::sig::{Elem, Signature};

pub struct PyfunctionArgs {
    pub signature: Option<Signature>,
}

impl Parse for PyfunctionArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut signature = None;
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            if key != "signature" {
                return Err(syn::Error::new(key.span(), "expected `signature = (...)`"));
            }
            if signature.is_some() {
                return Err(syn::Error::new(key.span(), "duplicate signature option"));
            }
            input.parse::<syn::Token![=]>()?;
            signature = Some(input.parse()?);
            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }
        Ok(Self { signature })
    }
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

pub fn generate(args: PyfunctionArgs, f: ItemFn, mprs: TokenStream) -> syn::Result<TokenStream> {
    if let Safety::Unsafe(unsafe_token) = f.sig.safety {
        return Err(syn::Error::new(
            unsafe_token.span,
            "function cannot be unsafe",
        ));
    }
    if f.sig.asyncness.is_some() || f.sig.variadic.is_some() || f.sig.abi.is_some() {
        return Err(syn::Error::new(
            f.sig.span(),
            "pyfunction requires a synchronous, non-variadic Rust function",
        ));
    }
    if f.sig
        .generics
        .params
        .iter()
        .any(|p| !matches!(p, syn::GenericParam::Lifetime(_)))
    {
        return Err(syn::Error::new(
            f.sig.generics.span(),
            "pyfunction does not support type or const generics",
        ));
    }
    let validated = validate_signature(f.sig.clone(), args.signature)?;
    let mutable_count = validated
        .args
        .iter()
        .filter(|a| matches!(a, ValidatedArg::Token(TokenType::MicroPythonMut)))
        .count();
    let gc_count = validated
        .args
        .iter()
        .filter(|a| matches!(a, ValidatedArg::Token(TokenType::Gc)))
        .count();
    if mutable_count > 1 || gc_count > 1 {
        return Err(syn::Error::new(
            f.sig.span(),
            "exclusive context tokens may only appear once",
        ));
    }
    if mutable_count != 0
        && validated
            .args
            .iter()
            .any(|a| matches!(a, ValidatedArg::Token(TokenType::MicroPython)))
    {
        return Err(syn::Error::new(
            f.sig.span(),
            "MicroPython and MicroPythonMut cannot be passed together",
        ));
    }

    let mut parsing = Vec::new();
    let mut bindings = Vec::new();
    let mut parsed_names = Vec::new();
    let mut call_args = Vec::new();
    for (index, arg) in validated.args.into_iter().enumerate() {
        match arg {
            ValidatedArg::Token(token) => call_args.push(match token {
                TokenType::MicroPython => quote!(__mp.borrow()),
                TokenType::MicroPythonMut => quote!(__mp.reborrow()),
                TokenType::Gc => quote!(__gc.reborrow()),
            }),
            ValidatedArg::StarArgs(name) | ValidatedArg::StarKwargs(name) => {
                return Err(syn::Error::new(
                    name.span(),
                    "variadic Python arguments are not supported yet",
                ));
            }
            ValidatedArg::PythonArg(arg) => {
                let local = format_ident!("__arg_{index}");
                let mut ty = *arg.ty;
                // The trampoline is monomorphic; infer the original function's
                // lifetime parameters from the actual rooted inputs.
                struct InferLifetimes<'a>(&'a syn::Generics);
                impl VisitMut for InferLifetimes<'_> {
                    fn visit_lifetime_mut(&mut self, lifetime: &mut syn::Lifetime) {
                        if self
                            .0
                            .lifetimes()
                            .any(|p| p.lifetime.ident == lifetime.ident)
                        {
                            *lifetime = syn::Lifetime::new("'_", lifetime.span());
                        }
                    }
                }
                InferLifetimes(&f.sig.generics).visit_type_mut(&mut ty);
                let keyword = syn::LitStr::new(&arg.name.unraw().to_string(), arg.name.span());
                let parser = quote!(<<#ty as #mprs::args::Argument<'_, '_>>::Parser as ::core::default::Default>::default());
                let lookup = match arg.kind {
                    ArgKind::Positional => quote!(__reader.next_positional_with(#parser)),
                    ArgKind::Keyword => {
                        quote!(__reader.find_keyword_with(#mprs::qstr!(#keyword), #parser))
                    }
                    ArgKind::PositionalOrKeyword => quote!({
                        let positional = __reader.next_positional_with(#parser);
                        if positional.is_some() {
                            if __reader.has_keyword(#mprs::qstr!(#keyword)) {
                                return Err(#mprs::args::ParseError::DuplicateArgument);
                            }
                            positional
                        } else {
                            __reader.find_keyword_with(#mprs::qstr!(#keyword), #parser)
                        }
                    }),
                };
                let required = if arg.default_value.is_none() {
                    quote!(if #local.is_none() { return Err(#mprs::args::ParseError::MissingArgument); })
                } else {
                    quote!()
                };
                parsing.push(quote! {
                    let #local = #lookup.transpose()?;
                    #required
                });
                let supplied = if mutable_count == 0 {
                    quote!(<#ty as #mprs::args::Argument<'_, '_>>::finalize(value, __mp.borrow()))
                } else {
                    quote!(value)
                };
                let absent = match arg.default_value {
                    Some(default) => quote!(#default),
                    None => quote!(::core::unreachable!(
                        "required argument checked during parsing"
                    )),
                };
                bindings.push(quote! {
                    let #local: #ty = match #local {
                        Some(value) => #supplied,
                        None => #absent,
                    };
                });
                parsed_names.push(local.clone());
                call_args.push(quote!(#local));
            }
        }
    }
    let name = &f.sig.ident;
    let vis = &f.vis;
    let cfg_attrs: Vec<_> = f
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
        .collect();
    Ok(quote! {
        #f

        #(#cfg_attrs)*
        #[allow(non_snake_case, unused_imports)]
        #vis mod #name {
            use super::*;

            pub const FUNCTION: #mprs::function::Function = unsafe {
                #mprs::function::Function::new(Some(__trampoline))
            };

            unsafe extern "C" fn __trampoline(
                __n_args: usize,
                __n_kw: usize,
                __args: *const #mprs::sys::mp_obj_t,
            ) -> #mprs::core::shims::mprs_function_ret_t {
                // SAFETY: called only by the VM's function-call boundary.
                let (mut __mp, mut __gc) = unsafe { #mprs::vm::trampoline_tokens() };
                let __parsed = (|| -> ::core::result::Result<_, #mprs::args::ParseError> {
                    let __borrow = __mp.borrow();
                    let __len = __n_args + 2 * __n_kw;
                    // Even an empty from_raw_parts requires a non-null pointer.
                    let __slice = if __len == 0 { &[] } else {
                        unsafe { ::core::slice::from_raw_parts(__args.cast::<#mprs::obj::Obj>(), __len) }
                    };
                    let (__pos, __kw) = __slice.split_at(__n_args);
                    let __args = unsafe {
                        #mprs::args::Args::new(
                            #mprs::obj::Bound::from_rooted_slice(__pos, __borrow),
                            #mprs::obj::Bound::from_rooted_slice(__kw, __borrow),
                        )
                    };
                    let mut __reader = #mprs::args::ArgsReader::new(__args)?;
                    #(#parsing)*
                    __reader.finish()?;
                    Ok((#(#parsed_names,)*))
                })();
                let (#(#parsed_names,)*) = match __parsed {
                    Ok(values) => values,
                    Err(error) => return #mprs::obj::IntoPyResult::into_py_result(error, __mp, __gc).inner(),
                };
                #(#bindings)*
                let __result = super::#name(#(#call_args),*);
                #mprs::obj::IntoPyResult::into_py_result(__result, __mp, __gc).inner()
            }
        }
    })
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

    fn expand(rust: &str, options: &str) -> syn::Result<TokenStream> {
        generate(
            syn::parse_str(options)?,
            syn::parse_str(rust)?,
            quote!(::micropython_rs),
        )
    }

    #[test]
    fn parses_attribute_options() {
        assert!(
            syn::parse_str::<PyfunctionArgs>("")
                .unwrap()
                .signature
                .is_none()
        );
        assert!(
            syn::parse_str::<PyfunctionArgs>("signature = (a, /),")
                .unwrap()
                .signature
                .is_some()
        );
        for options in [
            "signature = (), signature = ()",
            "name = foo",
            "signature",
            "signature = (a: i32)",
        ] {
            assert!(
                syn::parse_str::<PyfunctionArgs>(options).is_err(),
                "{options}"
            );
        }
    }

    #[test]
    fn generates_valid_rust_and_selects_finalization_mode() {
        let shared = expand(
            "pub fn f(value: Bound<'_, '_, Dict>) -> bool { true }",
            "signature = (value, /)",
        )
        .unwrap();
        syn::parse2::<syn::File>(shared.clone()).unwrap();
        assert!(shared.to_string().contains("finalize"));
        let mutable = expand(
            "fn f(mp: MicroPythonMut<'_>, value: Rooted<'_, Dict>) {}",
            "signature = (value, /)",
        )
        .unwrap();
        syn::parse2::<syn::File>(mutable.clone()).unwrap();
        assert!(!mutable.to_string().contains("finalize"));
        let empty = expand("fn f() {}", "").unwrap();
        syn::parse2::<syn::File>(empty).unwrap();
    }

    #[test]
    fn rejects_unsupported_functions_and_token_aliasing() {
        for rust in [
            "unsafe fn f() {}",
            "async fn f() {}",
            "extern \"C\" fn f() {}",
            "fn f<T>() {}",
            "fn f<const N: usize>() {}",
            "fn f(a: MicroPython<'_>, b: MicroPythonMut<'_>) {}",
            "fn f(a: MicroPythonMut<'_>, b: MicroPythonMut<'_>) {}",
            "fn f(a: Gc<'_>, b: Gc<'_>) {}",
        ] {
            assert!(expand(rust, "").is_err(), "{rust}");
        }
        for options in ["signature = (*a)", "signature = (**a)"] {
            let err = expand("fn f(a: Args) {}", options).unwrap_err();
            assert!(err.to_string().contains("not supported yet"));
        }
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
