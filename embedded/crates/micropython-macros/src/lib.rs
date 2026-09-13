use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

mod pyfunction;
mod root_project;
mod sig;

#[proc_macro_derive(RootProject, attributes(root_project))]
pub fn derive_root_project(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    root_project::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn micropython_rs_path() -> TokenStream {
    match crate_name("micropython-rs").expect("micropython-rs") {
        FoundCrate::Itself => quote! { crate },
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { ::#ident }
        }
    }
}

#[proc_macro]
pub fn qstr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mprs = micropython_rs_path();

    let qstr = syn::parse_macro_input!(input as syn::LitStr);
    let name = qstr.value();
    let ident = quote::format_ident!("MP_QSTR_{name}");

    quote::quote! {
        unsafe { #mprs::qstr::Qstr::from_raw(#mprs::sys::#ident as #mprs::sys::qstr) }
    }
    .into()
}

#[proc_macro_attribute]
pub fn pyfunction(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}
