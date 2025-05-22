use proc_macro2::Ident;
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, parse_quote,
    punctuated::Punctuated, spanned::Spanned, token::Comma,
};

pub(crate) fn inner(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let builder_ident = &ast.ident;
    if !builder_ident.to_string().contains("Builder") {
        return Err(syn::Error::new(
            builder_ident.span(),
            format!("{builder_ident}: should contains `Builder`"),
        ));
    }
    let docs = extract_docs(&ast.attrs);
    let ident = Ident::new(
        &builder_ident.to_string().replace("Builder", ""),
        builder_ident.span(),
    );
    match &ast.data {
        Data::Struct(data) => impl_struct(&ident, builder_ident, data, docs),
        Data::Enum(data) => impl_enum(&ident, builder_ident, data, docs),
        Data::Union(_data) => Err(syn::Error::new(ast.span(), "Unsupport Union".to_string())),
    }
}

fn impl_enum(
    ident: &Ident,
    builder_ident: &Ident,
    data: &DataEnum,
    docs: Vec<Attribute>,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut variants = data.variants.clone();
    let mut builds = Punctuated::<proc_macro2::TokenStream, Comma>::new();
    for variant in variants.iter_mut() {
        let var_id = &variant.ident;
        if let Some((_, expr)) = &variant.discriminant {
            return Err(syn::Error::new(
                expr.span(),
                "Unsupport discriminant".to_string(),
            ));
        }
        variant.attrs = extract_docs(&variant.attrs);
        match &mut variant.fields {
            Fields::Named(named) => {
                let mut f_ids = Punctuated::<Ident, Comma>::new();
                let mut f_builds = Punctuated::<proc_macro2::TokenStream, Comma>::new();
                for field in named.named.iter_mut() {
                    let ty = &field.ty;
                    let id = field.ident.as_ref().expect("1111");
                    field.ty = parse_quote!(<#ty as crate::builder::Builder<'s>>::Out);
                    f_ids.push(id.clone());
                    f_builds.push(quote! {#id: #id.build(file)});
                }
                builds
                    .push(quote! {#builder_ident::#var_id{#f_ids} => #ident::#var_id{#f_builds} });
            }
            Fields::Unnamed(unnamed) => {
                let mut f_ids = Punctuated::<Ident, Comma>::new();
                let mut f_builds = Punctuated::<proc_macro2::TokenStream, Comma>::new();
                for (i, field) in unnamed.unnamed.iter_mut().enumerate() {
                    let ty = &field.ty;
                    let id = Ident::new(&format!("__self_{i}"), field.span());
                    field.ty = parse_quote!(<#ty as crate::builder::Builder<'s>>::Out);
                    f_ids.push(id.clone());
                    f_builds.push(quote! {#id.build(file)});
                }
                builds
                    .push(quote! {#builder_ident::#var_id(#f_ids) => #ident::#var_id(#f_builds) });
            }
            Fields::Unit => {
                builds.push(quote! {#builder_ident::#var_id => #ident::#var_id});
            }
        }
    }
    Ok(quote! {
        #(#docs)*
        #[derive(Debug, Clone)]
        pub enum #ident<'s>{
            #variants
        }
        impl<'s> crate::builder::Builder<'s> for #builder_ident {
            type Out = #ident<'s>;
            #[inline]
            fn build(&self, file: &'s str) -> Self::Out {
                match self {
                    #builds
                }
            }
        }
    })
}

fn impl_struct(
    ident: &Ident,
    builder_ident: &Ident,
    data: &DataStruct,
    docs: Vec<Attribute>,
) -> syn::Result<proc_macro2::TokenStream> {
    match &data.fields {
        Fields::Named(named) => {
            let mut builds = Punctuated::<proc_macro2::TokenStream, Comma>::new();
            let mut fields = Punctuated::<Field, Comma>::new();
            for mut field in named.named.clone() {
                let id = field.ident.as_ref().expect("3333");
                let ty = &field.ty;
                builds.push(quote! {#id: self.#id.build(file)});
                field.ty = parse_quote!(<#ty as crate::builder::Builder<'s>>::Out);
                field.attrs = extract_docs(&field.attrs);
                fields.push(field);
            }
            Ok(quote! {
                #(#docs)*
                #[derive(Debug, Clone)]
                pub struct #ident<'s>{
                    #fields
                }
                impl<'s> crate::builder::Builder<'s> for #builder_ident {
                    type Out = #ident<'s>;
                    #[inline]
                    fn build(&self, file: &'s str) -> Self::Out {
                        #ident {
                            #builds
                        }
                    }
                }
            })
        }
        Fields::Unnamed(_unnamed) => Err(syn::Error::new(
            data.fields.span(),
            "Unsupport Unnamed".to_string(),
        )),
        Fields::Unit => Err(syn::Error::new(
            data.fields.span(),
            "Unsupport Unit".to_string(),
        )),
    }
}

pub fn extract_docs(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                Some(attr.clone())
            } else {
                None
            }
        })
        .collect()
}
