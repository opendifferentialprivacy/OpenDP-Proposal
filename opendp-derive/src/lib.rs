extern crate proc_macro;

use proc_macro::TokenStream;

use heck::SnakeCase;
use proc_macro2::{Ident, Span};
use quote::quote;
use quote::ToTokens;
use syn::{Data, DataEnum, DeriveInput, Expr, Fields, FieldsUnnamed, Generics, ImplItem, ItemImpl, parse_macro_input, Path, PathSegment, Type, TypePath, Variant};
use syn::export::TokenStream2;

/// Derive an apply macro to accompany an enum.
/// The macro may be used to apply a generic function over all enum variants.
#[proc_macro_derive(Apply, attributes(reapply))]
pub fn apply(input: TokenStream) -> TokenStream {
    const REAPPLY_ATTR_NAME: &'static str = "reapply";

    let DeriveInput { ident: ident_enum, data, .. } = parse_macro_input!(input as DeriveInput);

    let DataEnum { variants, .. } = if let Data::Enum(d) = data { d } else {
        panic!("Apply data must be an enum")
    };

    // name for the generated macro
    let ident_map = format!("apply_{}", ident_enum.to_string().to_snake_case());

    // write unary map macro
    let unary_arms = variants.iter()
        .map(|variant| {
            let Variant { ident: ident_variant, attrs, .. } = variant;
            // if reapply attribute is set
            let body = if attrs.iter()
                .any(|attr| attr.path.is_ident(REAPPLY_ATTR_NAME)) {

                // retrieve the ident of the type contained within this variant
                let ident_field_ty = if let Type::Path(ty_path) = get_ty_singleton(variant) {
                    &ty_path.path.segments.last().unwrap().ident
                } else {
                    panic!("Invalid type on enum field")
                };

                let ident_apply = Ident::new(
                    &format!("apply_{}", ident_field_ty.to_string().to_snake_case()),
                    Span::call_site());

                Expr::Verbatim(quote!(#ident_apply!(__function, arg1; __options)).into())
            } else {
                Expr::Verbatim(quote!(__function(arg1, __options).map(|v| v.into())).into())
            };
            quote!(#ident_enum::#ident_variant(arg1) => #body)
        })
        .collect::<Vec<_>>();

    let matcher_unary = Expr::Verbatim(quote!(match __arg1 {
        #(#unary_arms,)*
    }).into());

    let binary_arms = variants.iter()
        .map(|variant| {
            let Variant { ident: ident_variant, attrs, .. } = variant;
            // if reapply attribute is set
            let body = if attrs.iter()
                .any(|attr| attr.path.is_ident(REAPPLY_ATTR_NAME)) {

                // retrieve the ident of the type contained within this variant
                let ident_field_ty = if let Type::Path(ty_path) = get_ty_singleton(variant) {
                    &ty_path.path.segments.last().unwrap().ident
                } else {
                    panic!("Invalid type on enum field")
                };

                let ident_apply = Ident::new(
                    &format!("apply_{}", ident_field_ty.to_string().to_snake_case()),
                    Span::call_site());

                Expr::Verbatim(quote!(#ident_apply!(__function, arg1, arg2; __options)).into())
            } else {
                Expr::Verbatim(quote!(__function(arg1, arg2, __options).map(|v| v.into())).into())
            };
            quote!((#ident_enum::#ident_variant(arg1), #ident_enum::#ident_variant(arg2)) => #body)
        })
        .collect::<Vec<_>>();

    let matcher_binary = Expr::Verbatim(quote!(match (__arg1, __arg2) {
        #(#binary_arms,)*
        _ => Err(Error::AtomicMismatch)
    }).into());

    let sub_macro_var = |text: String| text
        .replace("__function", "$function")
        .replace("__arg1", "$arg1")
        .replace("__arg2", "$arg2")
        .replace("__options", "$( $option ),*");

    // syn cannot seem to express declarative macros in its AST,
    //    but we can still construct a token stream directly
    let macro_string = format!(r#"
    macro_rules! {ident_map} {{
        ($function:ident, $arg1:expr) => {{
            {ident_map}!($function, $arg1;)
        }};
        ($function:ident, $arg1:expr; $( $option:expr ),* ) => {{
            {unary_match}
        }};
        ($function:ident, $arg1:expr, $arg2:expr) => {{
            {ident_map}!($function, $arg1, $arg2;)
        }};
        ($function:ident, $arg1:expr, $arg2:expr; $( $option:expr ),* ) => {{
            {binary_match}
        }};
    }}
    "#,
        ident_map=ident_map,
        unary_match=sub_macro_var(matcher_unary.to_token_stream().to_string()),
        binary_match=sub_macro_var(matcher_binary.to_token_stream().to_string())
    );

    macro_string.parse().unwrap()
}

/// retrieve the only type contained in a length-one tuple variant
fn get_ty_singleton(variant: &Variant) -> &Type {
    if let Fields::Unnamed(FieldsUnnamed { unnamed: ref fields, .. }) = variant.fields {
        if fields.len() != 1 {
            panic!("Variants must be tuples of length one")
        }
        &fields.first().unwrap().ty
    } else {
        panic!("Variants must be tuples")
    }
}

// derive From implementations for the annotated enum
#[proc_macro_derive(AutoFrom)]
pub fn auto_from(input: TokenStream) -> TokenStream {
    let DeriveInput { ident: ident_enum, data, .. } = parse_macro_input!(input as DeriveInput);
    let DataEnum { variants, .. } = if let Data::Enum(d) = data { d } else {
        panic!("Apply data must be an enum")
    };

    let mut output = TokenStream::new();
    output.extend(variants.iter().map(|variant| {
        let ident_variant = &variant.ident;
        let ty_variant = get_ty_singleton(variant);

        TokenStream::from(quote!(impl From<#ty_variant> for #ident_enum {
            fn from(v: #ty_variant) -> Self {
                #ident_enum::#ident_variant(v)
            }
        }))
    }));

    output
}

/// derive getters for the annotated enum
///
/// # Example
/// ```
/// #[derive(AutoGet)]
/// enum MyEnum {A(bool)}
/// fn main() {
///     let test_instance = MyEnum::A(true);
///     assert!(test_instance.a().unwrap())
/// }
/// ```
#[proc_macro_derive(AutoGet)]
pub fn auto_get(input: TokenStream) -> TokenStream {

    let DeriveInput { ident: ident_enum, data, .. } = parse_macro_input!(input as DeriveInput);
    let DataEnum { variants, .. } = if let Data::Enum(d) = data { d } else {
        panic!("Apply data must be an enum")
    };

    let implementation = syn::Item::Impl(ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: syn::token::Impl::default(),
        generics: Generics::default(),
        trait_: None,
        self_ty: Box::new(Type::Path(TypePath {
            qself: None,
            path: Path::from(PathSegment::from(ident_enum.clone()))
        })),
        brace_token: syn::token::Brace::default(),
        items: variants.iter().map(|variant| {
            let ident_variant = &variant.ident;
            let ty_variant = get_ty_singleton(variant);
            let ident_as_getter = Ident::new(
                &format!("as_{}", ident_variant.to_string().to_lowercase()), Span::call_site());
            let ident_to_getter = Ident::new(
                &format!("to_{}", ident_variant.to_string().to_lowercase()), Span::call_site());

            ImplItem::Verbatim(TokenStream2::from(quote! {
                pub fn #ident_as_getter(&self) -> Result<&#ty_variant, Error> {
                    if let #ident_enum::#ident_variant(v) = self {
                        Ok(v)
                    } else { Err(Error::AtomicMismatch) }
                }
                pub fn #ident_to_getter(self) -> Result<#ty_variant, Error> {
                    if let #ident_enum::#ident_variant(v) = self {
                        Ok(v)
                    } else { Err(Error::AtomicMismatch) }
                }
            }))
        }).collect()
    });

    TokenStream::from(implementation.to_token_stream())
}
