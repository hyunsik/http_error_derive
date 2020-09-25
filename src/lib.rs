use proc_macro::TokenStream;
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::{parse_macro_input, Data, DeriveInput, Meta, NestedMeta, Variant};

#[proc_macro_derive(HttpError, attributes(detail, http_status))]
pub fn http_error_derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    impl_http_error_macro(&derive_input)
}

fn parse_detail(variant: &Variant) -> HashMap<&'static str, proc_macro2::TokenStream> {
    let attrs_set: HashSet<&'static str> = ["status", "message"].iter().cloned().collect();
    let mut attrs: HashMap<&'static str, proc_macro2::TokenStream> = HashMap::new();

    #[allow(clippy::for_loop_over_option)]
    for attr in variant
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("detail"))
    {
        if let Meta::List(list) = attr.parse_meta().unwrap() {
            for nested_meta in list.nested.iter() {
                if let NestedMeta::Meta(nested_meta) = &nested_meta {
                    if let Some(key) =
                        attrs_set.get(nested_meta.path().get_ident().unwrap().to_string().as_str())
                    {
                        if let Meta::NameValue(kv) = &nested_meta {
                            let _lit = &kv.lit;
                            attrs.insert(*key, quote! { #_lit });
                        }
                    }
                }
            }
        }
    }

    attrs
}

struct HttpErrorDetail {
    variant_name: proc_macro2::Ident,
    http_status: proc_macro2::TokenStream,
    message: proc_macro2::TokenStream,
}

fn impl_http_error_macro(ast: &syn::DeriveInput) -> TokenStream {
    let enum_name = &ast.ident;
    let details: Vec<HttpErrorDetail> = match &ast.data {
        Data::Enum(de) => de
            .variants
            .iter()
            .map(|variant: &Variant| {
                let variant_name = &variant.ident;
                let mut attrs = parse_detail(variant);

                HttpErrorDetail {
                    variant_name: variant_name.clone(),
                    http_status: attrs
                        .remove("status")
                        .expect("no http_status in HttpError variant"),
                    message: attrs
                        .remove("message")
                        .expect("no message attribute in HttpError variant"),
                }
            })
            .collect(),
        _ => panic!("only Enum is allowed"),
    };

    let error_code_impl: Vec<proc_macro2::TokenStream> = details
        .iter()
        .map(|detail| {
            let variant_name = &detail.variant_name;

            quote! {
                #enum_name::#variant_name => stringify!(#variant_name),
            }
        })
        .collect();

    let http_status_impl: Vec<proc_macro2::TokenStream> = details
        .iter()
        .map(|detail| {
            let variant_name = &detail.variant_name;
            let http_status = &detail.http_status;

            quote! {
                #enum_name::#variant_name => #http_status,
            }
        })
        .collect();

    let message_impl: Vec<proc_macro2::TokenStream> = details
        .iter()
        .map(|detail| {
            let variant_name = &detail.variant_name;
            let message = &detail.message;

            quote! {
                #enum_name::#variant_name => #message,
            }
        })
        .collect();

    let error_code_impl = quote! {
        impl #enum_name {
            pub fn error_code(&self) -> &str {
                match self {
                    #( #error_code_impl )*
                }
            }
        }
    };

    let http_status_impl = quote! {
        impl #enum_name {
            pub fn http_status(&self) -> u16 {
                match self {
                    #( #http_status_impl )*
                }
            }
        }
    };

    let message_impl = quote! {
        impl #enum_name {
            pub fn message(&self) -> &str {
                match self {
                    #( #message_impl )*
                }
            }
        }
    };

    let all_impls = quote! {
        #error_code_impl
        #http_status_impl
        #message_impl
    };

    all_impls.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
