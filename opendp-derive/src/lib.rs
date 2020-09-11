extern crate proc_macro;

use proc_macro::{TokenStream};

use quote::{ToTokens};
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Expr, Variant, Arm, ExprMatch, ExprPath, Path, PathSegment, Pat, PatIdent, ExprCall, PatTupleStruct, PatTuple};
use syn::punctuated::Punctuated;
use heck::SnakeCase;
use proc_macro2::{Ident, Span};
use std::iter::FromIterator;

#[proc_macro_derive(Mappable)]
pub fn map_variant(input: TokenStream) -> TokenStream {

    let DeriveInput { ident: ident_enum, data, .. } = parse_macro_input!(input as DeriveInput);

    let DataEnum { variants, .. } = if let Data::Enum(d) = data { d } else {
        panic!("Mappable data must be an enum")
    };

    // name for the generated unary map macro
    let ident_map_unary = format!("map_{}_unary", ident_enum.to_string().to_snake_case());
    // let ident_map_unary = Ident::new(
    //     format!("map_{}_unary", ident_enum.to_string().to_snake_case()).as_str(),
    //     ident_enum.span());

    let make_ident_expr = |v: &str| Expr::Path(ExprPath {
        attrs: vec![], qself: None,
        path: Path::from(PathSegment::from(Ident::new(v, Span::call_site())))
    });

    // write unary map macro
    let matcher = Expr::Match(ExprMatch {
        attrs: vec![],
        match_token: syn::token::Match::default(),
        // reference the macro variable $value in a placeholder ident
        expr: Box::new(make_ident_expr("__value")),
        brace_token: syn::token::Brace::default(),
        // generate each match arm from the variants of the enum
        arms: variants.into_iter().map(|variant| {
            let Variant { ident: ident_variant, .. } = variant;
            Arm {
                attrs: vec![],
                // left-hand side of "=>"
                pat: Pat::TupleStruct(PatTupleStruct {
                    attrs: vec![],
                    // full path to enum variant
                    path: Path {
                        leading_colon: None,
                        segments: Punctuated::from_iter(vec![ident_enum.clone(), ident_variant]
                            .into_iter().map(PathSegment::from))
                    },
                    // declare "v", a temporary variable with the value in the enum variant
                    pat: PatTuple {
                        attrs: vec![],
                        paren_token: syn::token::Paren {span: Span::call_site()},
                        elems: Punctuated::from_iter(vec![Pat::Ident(PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: Ident::new("v", Span::call_site()),
                            subpat: None
                        })].into_iter())
                    }
                }),
                guard: None,
                fat_arrow_token: syn::token::FatArrow::default(),

                // right-hand side of "=>": two nested function calls
                // 1. the conversion back to ExampleEnum
                body: Box::new(Expr::Call(ExprCall {
                    attrs: vec![],
                    func: Box::new(Expr::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: Punctuated::from_iter(vec![
                                ident_enum.clone(), Ident::new("from", Span::call_site())
                            ].into_iter().map(PathSegment::from))
                        }
                    })),
                    paren_token: syn::token::Paren::default(),
                    // 2. and the generic function invocation
                    args: Punctuated::from_iter(vec![Expr::Call(ExprCall {
                        attrs: vec![],
                        // reference the macro variable $function in a placeholder ident
                        func: Box::new(make_ident_expr("__function")),
                        paren_token: syn::token::Paren::default(),
                        args: Punctuated::from_iter(vec![make_ident_expr("v")])
                    })])
                })),
                comma: Some(syn::token::Comma::default()),
            }
        }).collect()
    });

    // syn cannot seem to express declarative macros in its AST,
    //    but we can still construct a token stream directly
    let macro_string = format!("
    macro_rules! {} {{
        ($value:ident, $function:ident) => {{
            {}
        }}
    }}
    ", ident_map_unary, matcher.to_token_stream().to_string())
        .replace("__value", "$value")
        .replace("__function", "$function");

    macro_string.parse().unwrap()
}
