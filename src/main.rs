use proc_macro2::TokenStream;
use quote::quote;
use std::fs;

mod ast;
mod parser;

fn render_rust_spec(spec: &ast::Spec) -> TokenStream {
    let items: Vec<_> = spec.iter().map(render_spec_item).collect();

    quote!(
        #(#items)*
    )
}

fn render_spec_item(spec_item: &ast::SpecItem) -> TokenStream {
    match spec_item {
        ast::SpecItem::StructDef(sdef) => render_struct_def(sdef),
        ast::SpecItem::EnumDef(edef) => render_enum_def(edef),
    }
}

fn render_enum_def(edef: &ast::EnumDef) -> TokenStream {
    let ident = quote::format_ident!("{}", edef.name);
    let variants: Vec<_> = edef.variants.iter().map(render_variant_def).collect();

    quote!(
        #[derive(Debug, Deserialize, Serialize)]
        enum #ident { #(#variants),* }
    )
}

fn render_variant_def(vdef: &ast::VariantDef) -> TokenStream {
    let ident = quote::format_ident!("{}", vdef.name);

    match vdef.variant_type {
        ast::VariantType::Simple => quote!(#ident),
        ast::VariantType::Tuple(ref tdef) => {
            let inner = render_tuple_def(tdef);
            quote!(#ident #inner)
        }
        ast::VariantType::Struct(ref fields) => {
            let inner = render_struct_fields(fields);
            quote!(#ident { #inner })
        }
    }
}

fn render_struct_def(sdef: &ast::StructDef) -> TokenStream {
    let ident = quote::format_ident!("{}", sdef.name);
    let fields = render_struct_fields(&sdef.fields);

    quote!(
        #[derive(Debug, Deserialize, Serialize)]
        struct #ident {
            #fields
        }
    )
}

fn render_struct_fields(fields: &ast::StructFields) -> TokenStream {
    let fields_rendered: Vec<_> = fields.iter().map(render_field_node).collect();
    quote!(#(#fields_rendered),*)
}

fn render_field_node(field: &ast::FieldNode) -> TokenStream {
    let name = quote::format_ident!("{}", field.name);
    let ty = render_type_ident(&field.type_ident);

    quote!(#name: #ty)
}

fn render_type_ident(type_ident: &ast::TypeIdent) -> TokenStream {
    match type_ident {
        ast::TypeIdent::BuiltIn(at) => render_atom_type(at),
        ast::TypeIdent::List(inner) => {
            let inner_rendered = render_type_ident(inner);
            quote!(Vec<#inner_rendered>)
        }
        ast::TypeIdent::Tuple(tdef) => render_tuple_def(tdef),
        ast::TypeIdent::UserDefined(id) => {
            let ident = quote::format_ident!("{}", id);
            quote!(#ident)
        }
    }
}

fn render_atom_type(atom_type: &ast::AtomType) -> TokenStream {
    match atom_type {
        ast::AtomType::Str => quote!(String),
        ast::AtomType::I32 => quote!(i32),
        ast::AtomType::U32 => quote!(u32),
        ast::AtomType::U8 => quote!(u8),
        ast::AtomType::F64 => quote!(f64),
    }
}

fn render_tuple_def(tuple_def: &ast::TupleDef) -> TokenStream {
    let components = tuple_def.components().iter().map(render_type_ident);

    quote!((#(#components),*))
}

fn main() {
    let input = fs::read_to_string("src/sample.humble").unwrap();
    let spec = parser::parse(&input);

    println!("{}", render_rust_spec(&spec));
}
