extern crate proc_macro;

use proc_macro::TokenStream;

use heck::SnakeCase;
use proc_macro2::{Ident, Span};
use quote::quote;
use quote::ToTokens;
use syn::{Arm, Data, DataEnum, DeriveInput, Expr, ExprMatch, Fields, FieldsUnnamed, Generics, ImplItem, ItemImpl, parse_macro_input, Pat, Path, PathSegment, Type, TypePath, Variant, PatWild};
use syn::export::TokenStream2;

/// Derive an apply macro to accompany an enum.
/// The macro may be used to apply a generic function over all enum variants.
#[proc_macro_derive(Apply)]
pub fn apply(input: TokenStream) -> TokenStream {

    let DeriveInput { ident: ident_enum, data, .. } = parse_macro_input!(input as DeriveInput);

    let DataEnum { variants, .. } = if let Data::Enum(d) = data { d } else {
        panic!("Apply data must be an enum")
    };

    // name for the generated unary map macro
    let ident_map = format!("apply_{}", ident_enum.to_string().to_snake_case());

    // write unary map macro
    let matcher_unary = Expr::Match(ExprMatch {
        attrs: vec![],
        match_token: syn::token::Match::default(),
        // reference the macro variable $value in a placeholder ident
        expr: Box::new(Expr::Verbatim(quote!(__arg1).into())),
        brace_token: syn::token::Brace::default(),
        // generate each match arm from the variants of the enum
        arms: variants.iter()
            .map(|variant| &variant.ident)
            .map(|ident_variant| Arm {
                attrs: vec![],
                // left-hand side of "=>"
                pat: Pat::Verbatim(quote!(#ident_enum::#ident_variant(arg1)).into()),
                guard: None,
                fat_arrow_token: syn::token::FatArrow::default(),
                // right-hand side of "=>"
                body: Box::new(Expr::Verbatim(quote!(__function(arg1, __options).into()).into())),
                comma: Some(syn::token::Comma::default()),
            })
            .collect()
    });

    let matcher_binary = Expr::Match(ExprMatch {
        attrs: vec![],
        match_token: syn::token::Match::default(),
        // reference the macro variable $value in a placeholder ident
        expr: Box::new(Expr::Verbatim(quote!((__arg1, __arg2)).into())),
        brace_token: syn::token::Brace::default(),
        // generate each match arm from the variants of the enum
        arms: variants.iter()
            .map(|variant| &variant.ident)
            .map(|ident_variant| Arm {
                attrs: vec![],
                // left-hand side of "=>"
                pat: Pat::Verbatim(quote!((#ident_enum::#ident_variant(arg1), #ident_enum::#ident_variant(arg2))).into()),
                guard: None,
                fat_arrow_token: syn::token::FatArrow::default(),
                // right-hand side of "=>"
                body: Box::new(Expr::Verbatim(quote!(__function(arg1, arg2, __options).into()).into())),
                comma: Some(syn::token::Comma::default()),
            })
            .chain(vec![Arm {
                attrs: vec![],
                pat: Pat::Wild(PatWild { attrs: vec![], underscore_token: syn::token::Underscore::default()}),
                guard: None,
                fat_arrow_token: syn::token::FatArrow::default(),
                // TODO: switch to Error::AtomicMismatch
                body: Box::new(Expr::Verbatim(quote!(panic!("argument types must match")).into())),
                comma: None
            }])
            .collect()
    });

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

        TokenStream::from(quote!(impl From<#ty_variant> for NumericScalar {
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
            let ident_getter = Ident::new(
                &ident_variant.to_string().to_lowercase(), Span::call_site());

            ImplItem::Verbatim(TokenStream2::from(quote! {
                fn #ident_getter(self) -> Result<#ty_variant, Error> {
                    if let #ident_enum::#ident_variant(v) = self {
                        Ok(v)
                    } else { Err(Error::AtomicMismatch) }
                }
            }))
        }).collect()
    });

    TokenStream::from(implementation.to_token_stream())
}
