use super::{field_name, to_atom, to_camel_case};
use crate::ast;

use itertools::Itertools;

/// Generate elm code for encoder functions for `spec`.
pub fn generate_struct_and_enum_encoders(spec: &ast::Spec) -> String {
    spec.iter()
        .filter_map(|spec_item| match spec_item {
            ast::SpecItem::StructDef(sdef) => {
                let json_encoder = generate_struct_json_encoder(sdef);
                let query_encoder = generate_struct_query_encoder(sdef);
                Some(format!("{}\n\n\n{}", json_encoder, query_encoder))
            },
            ast::SpecItem::EnumDef(edef) => Some(generate_enum_encoder(edef)),
            ast::SpecItem::ServiceDef(_) => None,
        })
        .join("\n\n\n")
}

fn generate_struct_json_encoder(sdef: &ast::StructDef) -> String {
    format!(
        "{encoder_name} : {type_name} -> E.Value\n{encoder_name} obj =\n    E.object\n        [ {fields}\n        ]",
        encoder_name = struct_or_enum_encoder_name(&sdef.name),
        type_name = sdef.name,
        fields = sdef.fields.iter().map(generate_field_json_encoder).join("\n        , "),
    )
}

fn generate_struct_query_encoder(sdef: &ast::StructDef) -> String {
    format!(
        "{encoder_name} : {type_name} -> List Url.Builder.QueryParameter\n{encoder_name} obj =\n    [ {fields}\n    ]",
        encoder_name = query_encoder_name(&sdef.name),
        type_name = sdef.name,
        fields = sdef.fields.iter().map(generate_field_query_encoder).join("\n    , "),
    )
}

fn generate_enum_encoder(edef: &ast::EnumDef) -> String {
    format!(
        "{encoder_name} : {type_name} -> E.Value\n{encoder_name} v =\n    case v of\n        {variants}",
        encoder_name = struct_or_enum_encoder_name(&edef.name),
        type_name = edef.name,
        variants = edef
            .variants
            .iter()
            .map(generate_variant_encoder_branch)
            .join("\n        "),
    )
}


fn generate_field_json_encoder(field: &ast::FieldNode) -> String {
    format!(
        "(\"{name}\", {value_encoder} obj.{field_name})",
        name = field.pair.name,
        field_name = field_name(&field.pair.name),
        value_encoder = generate_type_json_encoder(&field.pair.type_ident)
    )
}

fn generate_field_query_encoder(field: &ast::FieldNode) -> String {
    // TODO: escape strings (but we could fix this in the whole codebase)
    match field.pair.type_ident {
        ast::TypeIdent::BuiltIn(ast::AtomType::Str) |
        ast::TypeIdent::BuiltIn(ast::AtomType::Uuid) |
        ast::TypeIdent::BuiltIn(ast::AtomType::Bytes)
         => format!(
            "Url.Builder.string \"{name}\" obj.{field_name}",
            name = field.pair.name,
            field_name = field_name(&field.pair.name)
        ),
        ast::TypeIdent::BuiltIn(ast::AtomType::I32) |
        ast::TypeIdent::BuiltIn(ast::AtomType::U32) |
        ast::TypeIdent::BuiltIn(ast::AtomType::U8)
         => format!(
            "Url.Builder.int \"{name}\" obj.{field_name}",
            name = field.pair.name,
            field_name = field_name(&field.pair.name),
        ),
        _ => {
            // encode other types as json encoded strings
            format!(
                "obj.{field_name} |> {value_encoder} |> E.encode 4 |> Url.Builder.string \"{name}\"",
                name = field.pair.name,
                field_name = field_name(&field.pair.name),
                value_encoder = generate_complex_type_query_encoder(&field.pair.type_ident)
            )
        }
    }
}


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
                .map(|(idx, component)| format!("{} x{}", generate_type_json_encoder(component), idx))
                .join(", "),
        ),
        ast::VariantType::Struct(ref fields) => format!(
            "{name} obj -> E.object [ (\"{name}\", E.object [{fields}]) ]",
            name = variant.name,
            fields = fields.iter().map(generate_field_json_encoder).join(", "),
        ),
        ast::VariantType::Newtype(ref ty) => format!(
            "{name} obj -> E.object [ (\"{name}\", {enc} obj) ]",
            name = variant.name,
            enc = generate_type_json_encoder(ty),
        ),
    }
}

/// Generate elm code for a type encoder.
fn generate_type_encoder(atom_encoder: &dyn Fn(&ast::AtomType) -> String, type_ident: &ast::TypeIdent) -> String {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => atom_encoder(atom),
        ast::TypeIdent::List(inner) => {
            format!("E.list {}", to_atom(generate_type_json_encoder(inner)))
        }
        ast::TypeIdent::Option(inner) => {
            format!("builtinEncodeMaybe {}", to_atom(generate_type_json_encoder(inner)))
        }
        ast::TypeIdent::Result(ok, err) => {
            format!("builtinEncodeResult {} {}",
                to_atom(generate_type_json_encoder(err)),
                to_atom(generate_type_json_encoder(ok))
            )
        },
        ast::TypeIdent::Map(key, value) => {
            assert_eq!(
                generate_type_json_encoder(key),
                "E.string",
                "can only encode string keys in maps"
            );
            format!("E.dict identity {}", to_atom(generate_type_json_encoder(value)))
        }
        ast::TypeIdent::Tuple(tdef) => generate_tuple_encoder(tdef),
        ast::TypeIdent::UserDefined(ident) => struct_or_enum_encoder_name(ident),
    }
}

fn generate_type_json_encoder(type_ident: &ast::TypeIdent) -> String {
    generate_type_encoder(&generate_atom_json_encoder, type_ident)
}

fn generate_complex_type_query_encoder(type_ident: &ast::TypeIdent) -> String {
    generate_type_encoder(&generate_atom_query_encoder, type_ident)
}


fn generate_atom_json_encoder(atom: &ast::AtomType) -> String {
    match atom {
        ast::AtomType::Empty => "(_ -> E.null)",
        ast::AtomType::Str => "E.string",
        ast::AtomType::I32 => "E.int",
        ast::AtomType::U32 => "E.int",
        ast::AtomType::U8 => "E.int",
        ast::AtomType::F64 => "E.float",
        ast::AtomType::Bool => "E.bool",
        ast::AtomType::DateTime => "Iso8601.encode",
        ast::AtomType::Date => "builtinEncodeDate",
        ast::AtomType::Uuid => "E.string",
        ast::AtomType::Bytes => "E.string",
    }
    .to_owned()
}


fn generate_atom_query_encoder(atom: &ast::AtomType) -> String {
    match atom {
        ast::AtomType::Empty => "E.null",
        ast::AtomType::Str | ast::AtomType::Uuid | ast::AtomType::Bytes => "Url.Builder.string",
        ast::AtomType::I32 | ast::AtomType::U32 | ast::AtomType::U8 => "Url.Builder.int",
        ast::AtomType::F64 => "E.float",
        ast::AtomType::Bool => "E.bool",
        ast::AtomType::DateTime => "Iso8601.encode",
        ast::AtomType::Date => "builtinEncodeDate",
    }
    .to_owned()
}

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
            .map(|(idx, component)| format!("{} x{}", generate_type_json_encoder(component), idx))
            .join(", "),
    )
}

/// Construct name of encoder function for specific `ident`.
pub(crate) fn struct_or_enum_encoder_name(ident: &str) -> String {
    to_camel_case(&format!("encode{}", ident))
}

/// Encoder for url query
pub(crate) fn query_encoder_name(ident: &str) -> String {
    to_camel_case(&format!("buildQuery{}", ident))
}