use std::{borrow::Cow, ffi::OsStr, path::Path, process::Command};

use anyhow::{Context, bail};
use syn::{
    Expr, Ident, Lit, LitStr, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    visit::Visit,
};

use crate::generate::ScanItem;

struct FormatArgsBody {
    #[allow(dead_code)]
    format: LitStr,
    arguments: Punctuated<FormatArgument, Token![,]>,
}

enum FormatArgument {
    Positional(Expr),
    Named {
        #[allow(dead_code)]
        name: Ident,
        value: Expr,
    },
}

impl FormatArgument {
    fn value(&self) -> &Expr {
        match self {
            Self::Positional(value) | Self::Named { value, .. } => value,
        }
    }
}

impl Parse for FormatArgument {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Ident) && input.peek2(Token![=]) {
            Ok(Self::Named {
                name: input.parse()?,
                value: {
                    input.parse::<Token![=]>()?;
                    input.parse()?
                },
            })
        } else {
            Ok(Self::Positional(input.parse()?))
        }
    }
}

impl Parse for FormatArgsBody {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let format = input.parse()?;
        let arguments = if input.is_empty() {
            Punctuated::new()
        } else {
            input.parse::<Token![,]>()?;
            Punctuated::parse_terminated(input)?
        };

        Ok(Self { format, arguments })
    }
}

#[derive(Default)]
struct QstrFinder {
    qstrs: Vec<String>,
}

impl<'ast> Visit<'ast> for QstrFinder {
    fn visit_expr_tuple(&mut self, tuple: &'ast syn::ExprTuple) {
        let mut elems = tuple.elems.iter();
        let sentinel = elems.next();
        let qstr = elems.next();

        if let (Some(Expr::Lit(sentinel)), Some(Expr::Lit(qstr))) = (sentinel, qstr)
            && let (Lit::Str(sentinel), Lit::Str(qstr)) = (&sentinel.lit, &qstr.lit)
            && sentinel.value() == "__MICROPYTHON_RS_QSTR_VALUE__"
        {
            self.qstrs.push(qstr.value());
        }

        syn::visit::visit_expr_tuple(self, tuple);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if mac.path.is_ident("format_args")
            && let Ok(body) = syn::parse2::<FormatArgsBody>(mac.tokens.clone())
        {
            for argument in &body.arguments {
                self.visit_expr(argument.value());
            }
        }

        syn::visit::visit_macro(self, mac);
    }
}

pub fn scan_crate(crate_path: &Path, target: Option<&str>) -> anyhow::Result<ScanItem> {
    let cargo = std::env::var_os("CARGO")
        .map(|c| Cow::Owned(c))
        .unwrap_or(Cow::Borrowed(OsStr::new("cargo")));
    let mut command = Command::new(&cargo);
    command
        // setting in RUSTFLAGS instead of args makes it propagate
        .env("RUSTFLAGS", "--cfg micropython_rs_qstr_scan")
        .args(["rustc", "--quiet", "--", "-Zunpretty=expanded"])
        .current_dir(crate_path);

    if let Some(target) = target {
        command.args(["--target", target]);
    }

    let output = command
        .output()
        .with_context(|| format!("couldn't spawn `{}`", cargo.display()))?;

    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy_owned(output.stderr));
    }

    let expanded_crate = str::from_utf8(&output.stdout).with_context(|| {
        format!(
            "couldn't decode expanded crate `{}` as UTF-8",
            crate_path.display()
        )
    })?;
    let file = syn::parse_file(expanded_crate)
        .with_context(|| format!("couldn't parse expanded crate `{}`", crate_path.display()))?;
    let mut qstr_finder = QstrFinder::default();
    for item in file.items {
        qstr_finder.visit_item(&item);
    }

    Ok(ScanItem {
        qstrs: dbg!(qstr_finder.qstrs),
        // TODO: `micropython-rs` currently cannot define modules or root pointers, add sacnning
        // once bindings support exists
        moduledefs: Vec::new(),
        root_pointers: Vec::new(),
    })
}
