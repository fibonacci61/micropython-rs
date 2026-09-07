use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Fields, GenericParam, Lifetime, LifetimeParam, Meta, Token,
    ext::IdentExt,
    punctuated::Punctuated,
    visit_mut::{self, VisitMut},
};

pub fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    for attr in &input.attrs {
        if attr.path().is_ident("root_project") {
            return Err(syn::Error::new_spanned(
                attr,
                "root_project belongs on a field",
            ));
        }
        if attr.path().is_ident("repr") {
            let repr = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            if repr.iter().any(|meta| meta.path().is_ident("packed")) {
                return Err(syn::Error::new_spanned(
                    attr,
                    "RootProject does not support packed structs",
                ));
            }
        }
    }

    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "RootProject requires a struct with named fields",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "RootProject requires a struct with named fields",
        ));
    };

    let mprs = crate::micropython_rs_path();
    let name = &input.ident;
    let vis = &input.vis;
    let projection = format_ident!("__RootProject{}", name.unraw());
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let parent: syn::Type = syn::parse_quote!(#name #ty_generics);
    let mut replace_self = ReplaceSelf { parent };

    let mut lifetime_name = "__root_project".to_owned();
    while input
        .generics
        .lifetimes()
        .any(|param| param.lifetime.ident == lifetime_name)
    {
        lifetime_name.push('_');
    }
    let lifetime = Lifetime::new(&format!("'{lifetime_name}"), Span::call_site());
    let mut projection_generics = input.generics.clone();
    replace_self.visit_generics_mut(&mut projection_generics);
    projection_generics.params.insert(
        0,
        GenericParam::Lifetime(LifetimeParam::new(lifetime.clone())),
    );
    let (projection_impl_generics, projection_ty_generics, projection_where_clause) =
        projection_generics.split_for_impl();

    let mut methods = Vec::new();
    for field in &fields.named {
        let mut selected = false;
        for attr in &field.attrs {
            if attr.path().is_ident("root_project") {
                if selected || !matches!(attr.meta, Meta::Path(_)) {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "expected a single #[root_project] attribute",
                    ));
                }
                selected = true;
            }
        }
        if !selected {
            continue;
        }
        let field_name = &field.ident;
        let field_vis = &field.vis;
        let mut field_ty = field.ty.clone();
        replace_self.visit_type_mut(&mut field_ty);
        methods.push(quote! {
            #field_vis fn #field_name(&self) -> #mprs::obj::Rooted<#lifetime, #field_ty> {
                unsafe {
                    self.__root.project_unchecked(|parent| {
                        ::core::ptr::addr_of!((*parent).#field_name)
                    })
                }
            }
        });
    }

    Ok(quote! {
        #vis struct #projection #projection_generics #projection_where_clause {
            __root: #mprs::obj::Rooted<#lifetime, #name #ty_generics>,
        }

        impl #projection_impl_generics #projection #projection_ty_generics
            #projection_where_clause
        {
            #(#methods)*
        }

        impl #impl_generics #mprs::obj::RootProject for #name #ty_generics #where_clause {
            type Projection<#lifetime> = #projection #projection_ty_generics;

            fn project<#lifetime>(
                root: #mprs::obj::Rooted<#lifetime, Self>,
            ) -> Self::Projection<#lifetime> {
                #projection { __root: root }
            }
        }
    })
}

struct ReplaceSelf {
    parent: syn::Type,
}

impl VisitMut for ReplaceSelf {
    fn visit_type_mut(&mut self, ty: &mut syn::Type) {
        if let syn::Type::Path(path) = ty {
            if path.qself.is_none() && path.path.is_ident("Self") {
                *ty = self.parent.clone();
                return;
            }
        }
        visit_mut::visit_type_mut(self, ty);
    }

    fn visit_type_path_mut(&mut self, path: &mut syn::TypePath) {
        self.qualify(&mut path.qself, &mut path.path);
        visit_mut::visit_type_path_mut(self, path);
    }

    fn visit_expr_path_mut(&mut self, path: &mut syn::ExprPath) {
        self.qualify(&mut path.qself, &mut path.path);
        visit_mut::visit_expr_path_mut(self, path);
    }
}

impl ReplaceSelf {
    fn qualify(&self, qself: &mut Option<syn::QSelf>, path: &mut syn::Path) {
        if qself.is_none()
            && path.segments.len() > 1
            && path
                .segments
                .first()
                .is_some_and(|segment| segment.ident == "Self")
        {
            path.segments = path.segments.iter().skip(1).cloned().collect();
            path.leading_colon = Some(Default::default());
            *qself = Some(syn::QSelf {
                lt_token: Default::default(),
                ty: Box::new(self.parent.clone()),
                position: 0,
                as_token: None,
                gt_token: Default::default(),
            });
        }
    }
}
