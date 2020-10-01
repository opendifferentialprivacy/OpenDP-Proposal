extern crate proc_macro;

use proc_macro::TokenStream;
use std::iter::FromIterator;

use heck::SnakeCase;
use proc_macro2::{Ident, Span};
use quote::quote;
use quote::ToTokens;
use syn::{Arm, Data, DataEnum, DeriveInput, Expr, ExprMatch, ExprTuple, Fields, FieldsUnnamed, Generics, ImplItem, ItemImpl, parse_macro_input, Pat, Path, PathSegment, PatTuple, Result, Type, TypePath, Variant, PatTupleStruct, PatIdent, ExprMethodCall, ExprCall, ExprPath, ExprClosure, ReturnType, PatWild, ExprType};
use syn::export::TokenStream2;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Token;

const CATEGORICAL: [&'static str; 14] = ["String", "Bool", "R64", "R32", "I128", "I64", "I32", "I16", "I8", "U128", "U64", "U32", "U16", "U8"];
const OPTION_CATEGORICAL: [&'static str; 14] = ["OptionString", "OptionBool", "R64", "R32", "OptionI128", "OptionI64", "OptionI32", "OptionI16", "OptionI8", "OptionU128", "OptionU64", "OptionU32", "OptionU16", "OptionU8"];
const NUMERIC: [&'static str; 12] = ["R64", "R32", "I128", "I64", "I32", "I16", "I8", "U128", "U64", "U32", "U16", "U8"];
const FINITE_FLOAT: [&'static str; 2] = ["R64", "R32"];
const INTEGER: [&'static str; 10] = ["I128", "I64", "I32", "I16", "I8", "U128", "U64", "U32", "U16", "U8"];
const OPTION_FLOAT: [&'static str; 2] = ["F64", "F32"];
const OPTION_INTEGER: [&'static str; 10] = ["OptionI128", "OptionI64", "OptionI32", "OptionI16", "OptionI8", "OptionU128", "OptionU64", "OptionU32", "OptionU16", "OptionU8"];
const OPTION_NUMERIC: [&'static str; 12] = ["OptionI128", "OptionI64", "OptionI32", "OptionI16", "OptionI8", "OptionU128", "OptionU64", "OptionU32", "OptionU16", "OptionU8", "f64", "f32"];

#[derive(Clone, Debug)]
struct ApplySignature {
    function: Path,
    generics: Vec<ExprType>,
    literals: Vec<Expr>
}

impl Parse for ApplySignature {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ApplySignature {
            function: input.parse()?,
            generics: if input.peek(Token![,]) {
                let _ = input.parse::<Token![,]>();
                let mut generics = Vec::new();
                loop {
                    if input.is_empty() || input.peek(Token![;]) {
                        break;
                    }
                    if let Expr::Type(expr_type) = input.parse()? {
                        generics.push(expr_type)
                    } else {
                        panic!("Apply arguments must be expressions with type annotations, as in-- lower: Scalar")
                    };
                    if input.is_empty() || input.peek(Token![;]) {
                        break;
                    }
                    let _ = input.parse::<Token![,]>()?;
                }
                generics
            } else {
                Vec::new()
            },
            literals: if input.peek(Token![;]) {
                let _ = input.parse::<Token![;]>();
                let expr_punctuated: Punctuated<Expr, Token![,]> = input.parse_terminated(Expr::parse)?;
                expr_punctuated.into_iter().collect()
            } else {
                Vec::new()
            }
        })
    }
}

fn generate_matcher(apply_signature: ApplySignature, variant_idents: Vec<&str>) -> Expr {
    let ApplySignature { function, generics, literals } = apply_signature;
    Expr::Match(ExprMatch {
        attrs: vec![],
        match_token: syn::token::Match::default(),
        expr: match generics.len() {
            0 => panic!("Apply requires at least one generic argument"),
            1 => generics[0].clone().expr,
            _ => Box::new(Expr::Tuple(ExprTuple {
                attrs: vec![],
                paren_token: syn::token::Paren::default(),
                elems: Punctuated::from_iter(generics.iter().map(|v| *v.expr.clone()))
            }))
        },
        brace_token: syn::token::Brace::default(),
        arms: variant_idents.iter()
            .map(|ident_str| Ident::new(ident_str, Span::call_site()))
            .map(|ident| Arm {
                attrs: vec![],
                // one entry per generic argument
                pat: Pat::Tuple(PatTuple {
                    attrs: vec![],
                    paren_token: syn::token::Paren::default(),
                    // build each term in the tuple
                    elems: Punctuated::from_iter(generics.iter()
                        .enumerate().map(|(arg_count, generic)| {

                        let mut path_generic_arg = if let Type::Path(expr_path) = *generic.ty.clone() {
                            expr_path.path.clone()
                        } else {
                            panic!("({:?}) must be a path", generic.ty)
                        };
                        path_generic_arg.segments.push(PathSegment::from(ident.clone()));

                        Pat::TupleStruct(PatTupleStruct {
                            attrs: vec![],
                            // path identifying the variant
                            path: path_generic_arg,
                            // variable to match into
                            pat: PatTuple {
                                attrs: vec![],
                                paren_token: syn::token::Paren::default(),
                                elems: Punctuated::from_iter(vec![Pat::Ident(PatIdent {
                                    attrs: vec![],
                                    by_ref: None,
                                    mutability: None,
                                    ident: Ident::new(&format!("arg_{}", arg_count), Span::call_site()),
                                    subpat: None
                                })])
                            }
                        })
                    })),
                }),
                guard: None,
                fat_arrow_token: syn::token::FatArrow::default(),
                body: Box::new(Expr::MethodCall(ExprMethodCall {
                    attrs: vec![],
                    // 1. call the generic function
                    receiver: Box::new(Expr::Call(ExprCall {
                        attrs: vec![],
                        func: Box::new(Expr::Path(ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: function.clone()
                        })),
                        paren_token: syn::token::Paren::default(),
                        // arguments are a comma-separated punctuated list containing
                        // 1. expr arguments for each of the generics (verbose Path wrapping necessary)
                        // 2. expr arguments for each of the auxiliary literals (already exprs)
                        args: Punctuated::from_iter((0..generics.len())
                            .map(|arg_count| Expr::Path(ExprPath {
                                attrs: vec![],
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: Punctuated::from_iter(vec![PathSegment::from(
                                        Ident::new(&format!("arg_{}", arg_count), Span::call_site()))])
                                }
                            }))
                            .chain(literals.clone().into_iter()))
                    })),
                    dot_token: syn::token::Dot::default(),
                    method: Ident::new("map", Span::call_site()),
                    turbofish: None,
                    paren_token: syn::token::Paren::default(),
                    args: Punctuated::from_iter(vec![Expr::Closure(ExprClosure {
                        attrs: vec![],
                        asyncness: None,
                        movability: None,
                        capture: None,
                        or1_token: syn::token::Or::default(),
                        inputs: Punctuated::from_iter(vec![Pat::Tuple(PatTuple {
                            attrs: vec![],
                            paren_token: syn::token::Paren::default(),
                            elems: Punctuated::from_iter(vec![Pat::Ident(PatIdent {
                                attrs: vec![],
                                by_ref: None,
                                mutability: None,
                                ident: Ident::new("v", Span::call_site()),
                                subpat: None
                            })])
                        })]),
                        or2_token: syn::token::Or::default(),
                        output: ReturnType::Default,
                        body: Box::new(Expr::MethodCall(ExprMethodCall {
                            attrs: vec![],
                            receiver: Box::new(Expr::Path(ExprPath {
                                attrs: vec![],
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: Punctuated::from_iter(vec![PathSegment::from(
                                        Ident::new("v", Span::call_site()))])
                                }
                            })),
                            dot_token: syn::token::Dot::default(),
                            method: Ident::new("into", Span::call_site()),
                            turbofish: None,
                            paren_token: syn::token::Paren::default(),
                            args: Punctuated::new()
                        }))
                    })])
                })),
                comma: None,
            })
            .chain(vec![Arm {
                attrs: vec![],
                pat: Pat::Wild(PatWild { attrs: vec![], underscore_token: syn::token::Underscore::default() }),
                guard: None,
                fat_arrow_token: syn::token::FatArrow::default(),
                body: Box::new(Expr::Verbatim(quote!(Err(Error::AtomicMismatch)))),
                comma: None
            }])
            .collect()
    })

}

macro_rules! generate_apply_macro {
    ($name:ident, $variants:ident) => {
        #[proc_macro]
        pub fn $name(input: TokenStream) -> TokenStream {
            generate_matcher(parse_macro_input!(input as ApplySignature), $variants.to_vec())
                .to_token_stream().into()
        }
    }
}

generate_apply_macro!(apply_categorical, CATEGORICAL);
generate_apply_macro!(apply_option_categorical, OPTION_CATEGORICAL);
generate_apply_macro!(apply_numeric, NUMERIC);
generate_apply_macro!(apply_finite_float, FINITE_FLOAT);
generate_apply_macro!(apply_integer, INTEGER);
generate_apply_macro!(apply_option_float, OPTION_FLOAT);
generate_apply_macro!(apply_option_integer, OPTION_INTEGER);
generate_apply_macro!(apply_option_numeric, OPTION_NUMERIC);

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

        TokenStream::from(quote!{
            impl From<#ty_variant> for #ident_enum {
                fn from(v: #ty_variant) -> Self {
                    #ident_enum::#ident_variant(v)
                }
            }
            impl From<&#ty_variant> for #ident_enum {
                fn from(v: &#ty_variant) -> Self {
                    #ident_enum::#ident_variant(v.clone())
                }
            }
        })
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
///     assert!(test_instance.to_a().unwrap())
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
                &format!("as_{}", ident_variant.to_string().to_snake_case()), Span::call_site());
            let ident_to_getter = Ident::new(
                &format!("to_{}", ident_variant.to_string().to_snake_case()), Span::call_site());

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
