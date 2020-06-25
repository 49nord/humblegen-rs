use super::{field_name, to_atom, to_camel_case};
use crate::ast;

use itertools::Itertools;

/// Generate elm code for encoder functions for `spec`.
pub fn generate_type_encoders(spec: &ast::Spec) -> String {
    spec.iter()
        .filter_map(|spec_item| match spec_item {
            ast::SpecItem::StructDef(sdef) => Some(generate_struct_encoder(sdef)),
            ast::SpecItem::EnumDef(edef) => Some(generate_enum_encoder(edef)),
            ast::SpecItem::ServiceDef(_) => None,
        })
        .join("\n\n\n")
}

/// Generate elm code for a struct encoder.
fn generate_struct_encoder(sdef: &ast::StructDef) -> String {
    format!(
        "{encoder_name} : {type_name} -> E.Value\n{encoder_name} obj =\n    E.object\n        [ {fields}\n        ]",
        encoder_name = encoder_name(&sdef.name),
        type_name = sdef.name,
        fields = sdef.fields.iter().map(generate_field_encoder).join("\n        , "),
    )
}

/// Generate elm code for an enum encoder.
fn generate_enum_encoder(edef: &ast::EnumDef) -> String {
    format!(
        "{encoder_name} : {type_name} -> E.Value\n{encoder_name} v =\n    case v of\n        {variants}",
        encoder_name = encoder_name(&edef.name),
        type_name = edef.name,
        variants = edef
            .variants
            .iter()
            .map(generate_variant_encoder_branch)
            .join("\n        "),
    )
}

/// Generate elm code for a field encoder.
fn generate_field_encoder(field: &ast::FieldNode) -> String {
    format!(
        "(\"{name}\", {value_encoder} obj.{field_name})",
        name = field.pair.name,
        field_name = field_name(&field.pair.name),
        value_encoder = generate_type_encoder(&field.pair.type_ident)
    )
}

/// Generate elm code for encoding code for variant of enum.
fn generate_variant_encoder_branch(variant: &ast::VariantDef) -> String {
    match variant.variant_type {
        ast::VariantType::Simple => format!("{name} -> E.string \"{name}\"", name = variant.name),
        ast::VariantType::Tuple(ref tdef) => format!(
            "{name} {field_names} -> E.object [ (\"{name}\", E.list identity [{field_encoders}]) ]",
            name = variant.name,
            field_names = (0..tdef.elements().len())
                .map(|i| format!("x{}", i))
                .join(" "),
            field_encoders = tdef
                .elements()
                .iter()
                .enumerate()
                .map(|(idx, component)| format!("{} x{}", generate_type_encoder(component), idx))
                .join(", "),
        ),
        ast::VariantType::Struct(ref fields) => format!(
            "{name} obj -> E.object [ (\"{name}\", E.object [{fields}]) ]",
            name = variant.name,
            fields = fields.iter().map(generate_field_encoder).join(", "),
        ),
        ast::VariantType::Newtype(ref ty) => format!(
            "{name} obj -> E.object [ (\"{name}\", {enc} obj) ]",
            name = variant.name,
            enc = generate_type_encoder(ty),
        ),
    }
}

/// Generate elm code for a type encoder.
fn generate_type_encoder(type_ident: &ast::TypeIdent) -> String {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => generate_atom_encoder(atom),
        ast::TypeIdent::List(inner) => {
            format!("E.list {}", to_atom(generate_type_encoder(inner)))
        }
        ast::TypeIdent::Option(inner) => {
            format!("encMaybe {}", to_atom(generate_type_encoder(inner)))
        }
        ast::TypeIdent::Result(_ok, _err) => todo!(),
        ast::TypeIdent::Map(key, value) => {
            assert_eq!(
                generate_type_encoder(key),
                "E.string",
                "can only encode string keys in maps"
            );
            format!("E.dict identity {}", to_atom(generate_type_encoder(value)))
        }
        ast::TypeIdent::Tuple(tdef) => generate_tuple_encoder(tdef),
        ast::TypeIdent::UserDefined(ident) => encoder_name(ident),
    }
}

/// Generate elm code for an atomic type encoder.
fn generate_atom_encoder(atom: &ast::AtomType) -> String {
    match atom {
        ast::AtomType::Empty => "(_ -> E.null)",
        ast::AtomType::Str => "E.string",
        ast::AtomType::I32 => "E.int",
        ast::AtomType::U32 => "E.int",
        ast::AtomType::U8 => "E.int",
        ast::AtomType::F64 => "E.float",
        ast::AtomType::Bool => "E.bool",
        ast::AtomType::DateTime => "Iso8601.encode",
        ast::AtomType::Date => "encDate",
        ast::AtomType::Uuid => "E.string",
        ast::AtomType::Bytes => "E.string",
    }
    .to_owned()
}

/// Generate elm code for a tuple encoder.
fn generate_tuple_encoder(tdef: &ast::TupleDef) -> String {
    format!(
        "\\({field_names}) -> E.list identity [ {encode_values} ]",
        field_names = (0..tdef.elements().len())
            .map(|i| format!("x{}", i))
            .join(", "),
        encode_values = tdef
            .elements()
            .iter()
            .enumerate()
            .map(|(idx, component)| format!("{} x{}", generate_type_encoder(component), idx))
            .join(", "),
    )
}

/// Construct name of encoder function for specific `ident`.
fn encoder_name(ident: &str) -> String {
    to_camel_case(&format!("encode{}", ident))
}