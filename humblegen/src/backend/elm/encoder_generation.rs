use super::{field_name, to_atom};
use crate::ast;

use inflector::Inflector;
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
    let ns = "";
    format!(
        "{encoder_name} : {type_name} -> E.Value\n{encoder_name} obj =\n    E.object\n        [ {fields}\n        ]",
        encoder_name = struct_or_enum_encoder_name(&sdef.name, ns),
        type_name = sdef.name,
        fields = sdef.fields.iter().map(|f| generate_field_json_encoder(f, ns)).join("\n        , "),
    )
}

fn generate_struct_query_encoder(sdef: &ast::StructDef) -> String {
    let ns = "";
    format!(
        "{encoder_name} : {type_name} -> List Url.Builder.QueryParameter\n{encoder_name} obj =\n    [ {fields}\n    ]",
        encoder_name = query_struct_encoder_name(&sdef.name, ns),
        type_name = sdef.name,
        fields = sdef.fields.iter().map(|f| generate_field_query_encoder(f, ns)).join("\n    , "),
    )
}

fn generate_enum_encoder(edef: &ast::EnumDef) -> String {
    let ns = "";

    format!(
        "{encoder_name} : {type_name} -> E.Value\n{encoder_name} v =\n    case v of\n        {variants}",
        encoder_name = struct_or_enum_encoder_name(&edef.name, ns),
        type_name = edef.name,
        variants = edef
            .variants
            .iter()
            .map(|v| generate_variant_encoder_branch(v, ns))
            .join("\n        "),
    )
}


fn generate_field_json_encoder(field: &ast::FieldNode, ns :&str) -> String {
    format!(
        "(\"{name}\", {value_encoder} obj.{field_name})",
        name = field.pair.name,
        field_name = field_name(&field.pair.name),
        value_encoder = generate_type_json_encoder(&field.pair.type_ident, ns)
    )
}

fn generate_field_query_encoder(field: &ast::FieldNode, ns :&str) -> String {
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
                value_encoder = generate_complex_type_query_encoder(&field.pair.type_ident, ns)
            )
        }
    }
}


fn generate_variant_encoder_branch(variant: &ast::VariantDef, ns :&str) -> String {
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
                .map(|(idx, component)| format!("{} x{}", generate_type_json_encoder(component, ns), idx))
                .join(", "),
        ),
        ast::VariantType::Struct(ref fields) => format!(
            "{name} obj -> E.object [ (\"{name}\", E.object [{fields}]) ]",
            name = variant.name,
            fields = fields.iter().map(|f| generate_field_json_encoder(f, ns)).join(", "),
        ),
        ast::VariantType::Newtype(ref ty) => format!(
            "{name} obj -> E.object [ (\"{name}\", {enc} obj) ]",
            name = variant.name,
            enc = generate_type_json_encoder(ty, ns),
        ),
    }
}

/// Generate elm code for a type encoder.
fn generate_type_encoder(atom_encoder: &dyn Fn(&ast::AtomType, &str) -> String, type_ident: &ast::TypeIdent, ns :&str) -> String {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => atom_encoder(atom, ns),
        ast::TypeIdent::List(inner) => {
            format!("E.list {}", to_atom(generate_type_json_encoder(inner, ns)))
        }
        ast::TypeIdent::Option(inner) => {
            format!("builtinEncodeMaybe {}", to_atom(generate_type_json_encoder(inner, ns)))
        }
        ast::TypeIdent::Result(ok, err) => {
            format!("builtinEncodeResult {} {}",
                to_atom(generate_type_json_encoder(err, ns)),
                to_atom(generate_type_json_encoder(ok, ns))
            )
        },
        ast::TypeIdent::Map(key, value) => {
            assert_eq!(
                generate_type_json_encoder(key, ns),
                "E.string",
                "can only encode string keys in maps"
            );
            format!("E.dict identity {}", to_atom(generate_type_json_encoder(value, ns)))
        }
        ast::TypeIdent::Tuple(tdef) => generate_tuple_encoder(tdef, ns),
        ast::TypeIdent::UserDefined(ident) => struct_or_enum_encoder_name(ident, ns),
    }
}

pub(crate) fn generate_type_json_encoder(type_ident: &ast::TypeIdent, ns :&str) -> String {
    generate_type_encoder(&generate_atom_json_encoder, type_ident, ns)
}

fn generate_complex_type_query_encoder(type_ident: &ast::TypeIdent, ns :&str) -> String {
    generate_type_encoder(&generate_atom_query_encoder, type_ident, ns)
}


fn generate_atom_json_encoder(atom: &ast::AtomType, ns :&str) -> String {
    match atom {
        ast::AtomType::Empty => "(_ -> E.null)".to_owned(),
        ast::AtomType::Str => "E.string".to_owned(),
        ast::AtomType::I32 => "E.int".to_owned(),
        ast::AtomType::U32 => "E.int".to_owned(),
        ast::AtomType::U8 => "E.int".to_owned(),
        ast::AtomType::F64 => "E.float".to_owned(),
        ast::AtomType::Bool => "E.bool".to_owned(),
        ast::AtomType::DateTime => format!("{}builtinEncodeIso8601", ns),
        ast::AtomType::Date => format!("{}builtinEncodeDate", ns),
        ast::AtomType::Uuid => "E.string".to_owned(),
        ast::AtomType::Bytes => "E.string".to_owned(),
    }
}


fn generate_atom_query_encoder(atom: &ast::AtomType, ns :&str) -> String {
    match atom {
        ast::AtomType::Empty => "E.null".to_owned(),
        ast::AtomType::Str | ast::AtomType::Uuid | ast::AtomType::Bytes => "Url.Builder.string".to_owned(),
        ast::AtomType::I32 | ast::AtomType::U32 | ast::AtomType::U8 => "Url.Builder.int".to_owned(),
        ast::AtomType::F64 => "E.float".to_owned(),
        ast::AtomType::Bool => "E.bool".to_owned(),
        ast::AtomType::DateTime => format!("{}builtinEncodeIso8601", ns),
        ast::AtomType::Date => format!("{}builtinEncodeDate", ns),
    }
}

fn generate_tuple_encoder(tdef: &ast::TupleDef, ns :&str) -> String {
    format!(
        "\\({field_names}) -> E.list identity [ {encode_values} ]",
        field_names = (0..tdef.elements().len())
            .map(|i| format!("x{}", i))
            .join(", "),
        encode_values = tdef
            .elements()
            .iter()
            .enumerate()
            .map(|(idx, component)| format!("{} x{}", generate_type_json_encoder(component, ns), idx))
            .join(", "),
    )
}

/// Construct name of encoder function for specific `ident`.
pub(crate) fn struct_or_enum_encoder_name(ident: &str, ns :&str) -> String {
    format!("{}encode{}", ns, ident.to_pascal_case())
}

pub(crate) fn query_encoder(ident :&ast::TypeIdent, ns: &str) -> String {
    // TODO: should narrow type of query parameter. According to spec query has to be a user defined struct
    if let ast::TypeIdent::UserDefined(query_ty_name) = ident {
        query_struct_encoder_name(query_ty_name, ns)
    } else {
        panic!("query MUST be a user defined struct");
    }
}

pub(crate) fn query_struct_encoder_name(ident: &str, ns :&str) -> String {
    format!("{}buildQuery{}", ns, ident.to_pascal_case())
}