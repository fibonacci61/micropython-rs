use proc_macro::TokenStream;

#[proc_macro]
pub fn qstr(input: TokenStream) -> TokenStream {
    let qstr = syn::parse_macro_input!(input as syn::LitStr);
    let name = qstr.value();
    let ident = quote::format_ident!("MP_QSTR_{name}");

    quote::quote! {
        unsafe { ::micropython_rs::qstr::Qstr::from_raw(::micropython_rs::qstr::__micropython_sys::#ident) }
    }.into()
}
