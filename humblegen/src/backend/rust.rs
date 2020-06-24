//! Rust code generator.

pub(crate) mod rustfmt;
mod service_server;

use crate::{ast, Artifact, LibError, Spec};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;
use std::path::Path;
use std::{fs::File, io::Write};

const BACKEND_NAME: &str = "rust";

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

/// Generate rust code for a struct definition.
pub(crate) fn generate_struct_def(sdef: &ast::StructDef) -> TokenStream {
    let ident = fmt_ident(&sdef.name);
    let doc_comment = fmt_opt_string(&sdef.doc_comment);
    let fields: Vec<_> = sdef.fields.iter().map(generate_pub_field_node).collect();

    quote!(
        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        #[doc = #doc_comment]
        pub struct #ident {
            #(#fields),*
        }
    )
}

/// Generate rust code for an enum definition.
pub(crate) fn generate_enum_def(edef: &ast::EnumDef) -> TokenStream {
    let ident = fmt_ident(&edef.name);
    let doc_comment = fmt_opt_string(&edef.doc_comment);

    let variants: Vec<_> = edef.variants.iter().map(generate_variant).collect();

    quote!(
        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        #[doc = #doc_comment]
        pub enum #ident {
            #(#variants),*
    })
}

/// Generate rust code for a field node.
fn generate_field_def_pair(pair: &ast::FieldDefPair) -> TokenStream {
    let ident = fmt_ident(&pair.name);
    let ty = generate_type_ident(&pair.type_ident);
    quote!(#ident: #ty)
}

/// Generate rust code for a public field node.
///
/// Even though all fields are pub in generated code, fields in a `pub enum` cannot carry an
/// additional `pub` qualifier.
fn generate_pub_field_node(field: &ast::FieldNode) -> TokenStream {
    let doc_comment = fmt_opt_string(&field.doc_comment);
    let attributes = generate_field_attributes(&field.pair.type_ident);
    let field = generate_field_def_pair(&field.pair);
    quote! {
        #[doc = #doc_comment]
        #(#[#attributes])*
        pub #field
    }
}

/// Generate rust code for an enum variant.
fn generate_variant(variant: &ast::VariantDef) -> TokenStream {
    let doc_comment = fmt_opt_string(&variant.doc_comment);
    let ident = fmt_ident(&variant.name);

    match variant.variant_type {
        ast::VariantType::Simple => quote!(#[doc = #doc_comment] #ident),
        ast::VariantType::Tuple(ref inner) => {
            let tuple = generate_tuple_def(inner);
            quote!(#[doc = #doc_comment] #ident #tuple)
        }
        ast::VariantType::Struct(ref fields) => {
            let fields: Vec<_> = fields
                .iter()
                .map(|field| {
                    let doc_comment = fmt_opt_string(&field.doc_comment);
                    let fld = generate_field_def_pair(&field.pair);
                    quote!(#[doc = #doc_comment] #fld)
                })
                .collect();

            quote!(#[doc = #doc_comment] #ident { #(#fields),*})
        }
        ast::VariantType::Newtype(ref ty) => {
            let inner = generate_type_ident(ty);

            quote!(#[doc = #doc_comment] #ident(#inner))
        }
    }
}

/// Generate rust code for a type identifier.
fn generate_type_ident(type_ident: &ast::TypeIdent) -> TokenStream {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => generate_atom(atom),
        ast::TypeIdent::List(inner) => {
            let inner_ty = generate_type_ident(inner);
            quote!(Vec<#inner_ty>)
        }
        ast::TypeIdent::Option(inner) => {
            let inner_ty = generate_type_ident(inner);
            quote!(Option<#inner_ty>)
        }
        ast::TypeIdent::Result(ok, err) => {
            let ok_ty = generate_type_ident(ok);
            let err_ty = generate_type_ident(err);
            quote!(Result<#ok_ty, #err_ty>)
        }
        ast::TypeIdent::Map(key, value) => {
            let key_ty = generate_type_ident(key);
            let value_ty = generate_type_ident(value);
            quote!(::std::collections::HashMap<#key_ty, #value_ty>)
        }
        ast::TypeIdent::Tuple(tdef) => generate_tuple_def(tdef),
        ast::TypeIdent::UserDefined(ident) => {
            let id = fmt_ident(&ident);
            quote!(#id)
        }
    }
}

/// The list of attributes that are tacked onto the struct / enum field definition.
/// Without the surrounding `#[` and `]`
type FieldAttributes = Vec<TokenStream>;

/// Render the list of field attributes for the given type_ident
fn generate_field_attributes(type_ident: &ast::TypeIdent) -> FieldAttributes {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => match atom {
            ast::AtomType::Empty => vec![],
            ast::AtomType::Str => vec![],
            ast::AtomType::I32 => vec![],
            ast::AtomType::U32 => vec![],
            ast::AtomType::U8 => vec![],
            ast::AtomType::F64 => vec![],
            ast::AtomType::Bool => vec![],
            ast::AtomType::DateTime => vec![],
            ast::AtomType::Date => vec![],
            ast::AtomType::Uuid => vec![],
            ast::AtomType::Bytes => vec![
                quote! { serde(deserialize_with = "::humblegen_rt::serialization_helpers::deser_bytes") },
                quote! { serde(serialize_with = "::humblegen_rt::serialization_helpers::ser_bytes") },
            ],
        },
        ast::TypeIdent::List(_) => vec![],
        ast::TypeIdent::Option(_) => vec![],
        ast::TypeIdent::Result(_, _) => vec![],
        ast::TypeIdent::Map(_, _) => vec![],
        ast::TypeIdent::Tuple(_) => vec![],
        ast::TypeIdent::UserDefined(_) => vec![],
    }
}

/// Generate rust code for a tuple definition.
fn generate_tuple_def(tdef: &ast::TupleDef) -> TokenStream {
    let components: Vec<_> = tdef.elements().iter().map(generate_type_ident).collect();

    if components.len() == 1 {
        quote!((#(#components),*,))
    } else {
        quote!((#(#components),*))
    }
}

/// Generate rust code for an atomic type.
fn generate_atom(atom: &ast::AtomType) -> TokenStream {
    match atom {
        ast::AtomType::Empty => quote!(()),
        ast::AtomType::Str => quote!(String),
        ast::AtomType::I32 => quote!(i32),
        ast::AtomType::U32 => quote!(u32),
        ast::AtomType::U8 => quote!(u8),
        ast::AtomType::F64 => quote!(f64),
        ast::AtomType::Bool => quote!(bool),
        ast::AtomType::DateTime => {
            quote!(::humblegen_rt::chrono::DateTime::<::humblegen_rt::chrono::prelude::Utc>)
        }
        // chrono::Date doesn't implement serde::Serialize / serde::Deserialize:
        // https://github.com/chronotope/chrono/issues/182#issuecomment-332382103
        ast::AtomType::Date => quote!(::humblegen_rt::chrono::NaiveDate),
        ast::AtomType::Uuid => quote! {::humblegen_rt::uuid::Uuid},
        ast::AtomType::Bytes => quote!(Vec<u8>),
    }
}

/// Generate rust code for a spec definition.
pub fn render_spec(spec: &ast::Spec) -> TokenStream {
    let mut out = TokenStream::new();

    out.extend(spec.iter().flat_map(|spec_item| match spec_item {
        ast::SpecItem::StructDef(sdef) => generate_struct_def(sdef),
        ast::SpecItem::EnumDef(edef) => generate_enum_def(edef),
        ast::SpecItem::ServiceDef(_) => quote! {}, // done below
    }));

    out.extend(service_server::generate_services(
        spec.iter().filter_map(|si| si.service_def()),
    ));

    out
}

pub struct Generator {
    _artifact: Artifact,
}

impl Generator {
    pub fn new(artifact: Artifact) -> Result<Self, LibError> {
        match artifact {
            Artifact::TypesOnly | Artifact::ServerEndpoints => Ok(Self {
                _artifact: artifact,
            }),
            Artifact::ClientEndpoints => Err(LibError::UnsupportedArtifact {
                artifact,
                backend: BACKEND_NAME,
            }),
        }
    }
}

impl crate::CodeGenerator for Generator {
    fn generate(&self, spec: &Spec, output: &Path) -> Result<(), LibError> {
        // TODO: honor artifact field
        let generated_code_unformatted = render_spec(spec).to_string();
        let generated_code = rustfmt::rustfmt_2018_generated_string(&generated_code_unformatted)
            .map(std::borrow::Cow::into_owned)
            .unwrap_or(generated_code_unformatted);

        // TODO: support folder as output path
        let mut outfile = File::create(&output).map_err(LibError::IoError)?;
        outfile
            .write_all(generated_code.as_bytes())
            .map_err(LibError::IoError)?;
        Ok(())
    }
}
