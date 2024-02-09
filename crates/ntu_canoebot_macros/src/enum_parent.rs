//! Derive macro implementation for EnumParent

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, DeriveInput};

use crate::utils::substitute_into_wildcard;

const PARENT_ATTR: &str = "enum_parent";

pub fn derive(input: TokenStream) -> TokenStream {
    derive_enum_parent(input).into()
}

/// Derives the CallbackPath trait
pub fn derive_enum_parent(input: TokenStream) -> proc_macro2::TokenStream {
    let DeriveInput {
        ident,
        data,
        // generics,
        attrs,
        ..
    } = match syn::parse(input) {
        Ok(res) => res,
        Err(e) => return e.to_compile_error().into(),
    };

    let _ = if let syn::Data::Enum(e) = data {
        e
    } else {
        let error = syn::Error::new(ident.span(), "Type must be enum")
            .to_compile_error()
            .into_token_stream();

        return error;
    };

    let callback_attr = match attrs.iter().find(|a| a.path().is_ident(PARENT_ATTR)) {
        Some(a) => a,
        None => {
            let error = syn::Error::new(
                ident.span(),
                format!("Attribute \"{}\" not found", PARENT_ATTR),
            )
            .to_compile_error()
            .to_token_stream();
            return error;
        }
    };

    let meta_list = if let syn::Meta::List(l) = &callback_attr.meta {
        l
    } else {
        let error = syn::Error::new(callback_attr.span(), "Attribute must be a list")
            .to_compile_error()
            .to_token_stream();
        return error;
    };

    let value_literal = quote! {value};

    let assoc_type = match meta_list.tokens.clone().into_iter().find(|_| true).unwrap() {
        proc_macro2::TokenTree::Ident(i) => {
            if i == "_" {
                ident.clone()
            } else {
                i
            }
        }
        _ => ident.clone(),
    };

    let substituted = substitute_into_wildcard(
        &meta_list.tokens,
        &value_literal,
        proc_macro2::Delimiter::None,
    );

    quote! {

        impl EnumParent for #ident {

            type Parent = #assoc_type;

            fn enum_parent(value: Self) -> Self::Parent {
                #substituted
            }
        }

    }
}
