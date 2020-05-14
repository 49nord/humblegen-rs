//! Rust code generator.

use crate::ast;
use proc_macro2::TokenStream;
use quote::quote;

/// Helper function to format an ident.
///
/// Turns a string into an ident, eases the use inside `quote!`.
fn fmt_ident(ident: &str) -> proc_macro2::Ident {
    quote::format_ident!("{}", ident)
}

/// Helper function to format an optional string as a string.
fn fmt_opt_string(s: &Option<String>) -> &str {
    s.as_ref().map(|s| s.as_str()).unwrap_or("")
}

/// Render a spec definition.
pub fn render(spec: &ast::Spec) -> TokenStream {
    spec.iter()
        .flat_map(|spec_item| match spec_item {
            ast::SpecItem::StructDef(sdef) => render_struct_def(sdef),
            ast::SpecItem::EnumDef(edef) => render_enum_def(edef),
        })
        .collect()
}

/// Render a struct definition.
fn render_struct_def(sdef: &ast::StructDef) -> TokenStream {
    let ident = fmt_ident(&sdef.name);
    let doc_comment = fmt_opt_string(&sdef.doc_comment);
    let fields: Vec<_> = sdef.fields.iter().map(render_pub_field_node).collect();

    quote!(
        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        #[doc = #doc_comment]
        pub struct #ident {
            #(#fields),*
        }
    )
}

/// Render an enum definition.
fn render_enum_def(edef: &ast::EnumDef) -> TokenStream {
    let ident = fmt_ident(&edef.name);
    let doc_comment = fmt_opt_string(&edef.doc_comment);

    let variants: Vec<_> = edef.variants.iter().map(render_variant).collect();

    quote!(
        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        #[doc = #doc_comment]
        pub enum #ident {
            #(#variants),*
    })
}

/// Render a field node.
fn render_field_def_pair(pair: &ast::FieldDefPair) -> TokenStream {
    let ident = fmt_ident(&pair.name);
    let ty = render_type_ident(&pair.type_ident);
    quote!(#ident: #ty)
}

/// Render a public field node.
///
/// Even though all fields are pub in generated code, fields in a `pub enum` cannot carry an
/// additional `pub` qualifier.
fn render_pub_field_node(field: &ast::FieldNode) -> TokenStream {
    let doc_comment = fmt_opt_string(&field.doc_comment);
    let field = render_field_def_pair(&field.pair);

    quote!(#[doc = #doc_comment] pub #field)
}

/// Render an enum variant.
fn render_variant(variant: &ast::VariantDef) -> TokenStream {
    let doc_comment = fmt_opt_string(&variant.doc_comment);
    let ident = fmt_ident(&variant.name);

    match variant.variant_type {
        ast::VariantType::Simple => quote!(#[doc = #doc_comment] #ident),
        ast::VariantType::Tuple(ref inner) => {
            let tuple = render_tuple_def(inner);
            quote!(#[doc = #doc_comment] #ident #tuple)
        }
        ast::VariantType::Struct(ref fields) => {
            let fields: Vec<_> = fields
                .iter()
                .map(|field| {
                    let doc_comment = fmt_opt_string(&field.doc_comment);
                    let fld = render_field_def_pair(&field.pair);
                    quote!(#[doc = #doc_comment] #fld)
                })
                .collect();

            quote!(#[doc = #doc_comment] #ident { #(#fields),*})
        }
        ast::VariantType::Newtype(ref ty) => {
            let inner = render_type_ident(ty);

            quote!(#[doc = #doc_comment] #ident(#inner))
        }
    }
}

/// Render a type identifier.
fn render_type_ident(type_ident: &ast::TypeIdent) -> TokenStream {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => render_atom(atom),
        ast::TypeIdent::List(inner) => {
            let inner_ty = render_type_ident(inner);
            quote!(Vec<#inner_ty>)
        }
        ast::TypeIdent::Option(inner) => {
            let inner_ty = render_type_ident(inner);
            quote!(Option<#inner_ty>)
        }
        ast::TypeIdent::Result(ok, err) => {
            let ok_ty = render_type_ident(ok);
            let err_ty = render_type_ident(err);
            quote!(Result<#ok_ty, #err_ty>)
        }
        ast::TypeIdent::Map(key, value) => {
            let key_ty = render_type_ident(key);
            let value_ty = render_type_ident(value);
            quote!(::std::collections::HashMap<#key_ty, #value_ty>)
        }
        ast::TypeIdent::Tuple(tdef) => render_tuple_def(tdef),
        ast::TypeIdent::UserDefined(ident) => {
            let id = fmt_ident(&ident);
            quote!(#id)
        }
    }
}

/// Render a tuple definition.
fn render_tuple_def(tdef: &ast::TupleDef) -> TokenStream {
    let components: Vec<_> = tdef.components().iter().map(render_type_ident).collect();

    if components.len() == 1 {
        quote!((#(#components),*,))
    } else {
        quote!((#(#components),*))
    }
}

/// Render an atomic type.
fn render_atom(atom: &ast::AtomType) -> TokenStream {
    match atom {
        ast::AtomType::Str => quote!(String),
        ast::AtomType::I32 => quote!(i32),
        ast::AtomType::U32 => quote!(u32),
        ast::AtomType::U8 => quote!(u8),
        ast::AtomType::F64 => quote!(f64),
        ast::AtomType::Bool => quote!(bool),
        ast::AtomType::DateTime => quote!(::chrono::DateTime::<::chrono::prelude::Utc>),
        // chrono::Date doesn't implement serde::Serialize / serde::Deserialize:
        // https://github.com/chronotope/chrono/issues/182#issuecomment-332382103
        ast::AtomType::Date => quote!(::chrono::NaiveDate),
    }
}
