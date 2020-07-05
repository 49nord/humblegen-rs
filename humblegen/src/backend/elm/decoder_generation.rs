use super::{to_atom, type_generation};
use crate::ast;
use inflector::Inflector;

use itertools::Itertools; // directly call join(.) on iterators

/// Generate elm code for decoders for a spec.
pub fn generate_type_decoders(spec: &ast::Spec) -> String {
    spec.iter()
        .filter_map(|spec_item| match spec_item {
            ast::SpecItem::StructDef(sdef) => Some(generate_struct_decoder(sdef)),
            ast::SpecItem::EnumDef(edef) => Some(generate_enum_decoder(edef)),
            ast::SpecItem::ServiceDef(_) => None,
        })
        .join("\n\n\n")
}

fn generate_struct_decoder(sdef: &ast::StructDef) -> String {
    let ns = "";
    format!(
        "{dec_name} : D.Decoder {name} \n\
        {dec_name} =\n   D.succeed {name}\n        {field_decoders}",
        dec_name = decoder_name(&sdef.name, ns),
        name = sdef.name,
        field_decoders = sdef
            .fields
            .iter()
            .map(|f| generate_field_decoder(f, ns))
            .join("\n        ")
    )
}

fn generate_enum_decoder(edef: &ast::EnumDef) -> String {
    let ns = "";

    let mut fields = edef.variants.iter().map(|variant| {
        match variant.variant_type {
            ast::VariantType::Simple => {
                format!(
                    "D.string |> D.andThen (\\s -> if s == \"{name}\" then D.succeed {name} else D.fail \"\")",
                    name = variant.name
                )
            }
            ast::VariantType::Tuple(ref components) => format!(
                "D.succeed {name} {components}",
                name = variant.name,
                components = generate_components_by_index_pipeline(components, ns)
            ),
            ast::VariantType::Struct(ref fields) => format!(
                "D.field \"{variantName}\" (D.succeed {name} {field_decoders} |> D.map {variantName})",
                name = type_generation::enum_anonymous_struct_constructor_name(&edef.name, &variant.name),
                variantName = variant.name,
                field_decoders = fields.iter().map(|f| generate_field_decoder(f, ns)).join(" "),
            ),
            ast::VariantType::Newtype(ref ty) => format!(
                "D.field \"{variantName}\" (D.map {name} {ty})",
                name = variant.name,
                variantName = variant.name,
                ty = to_atom(generate_type_decoder(ty, ns)),
            ),
        }
    });

    format!(
        "{dec_name} : D.Decoder {name}\n{dec_name} =\n    D.oneOf\n        [{fields}\n        ]",
        dec_name = decoder_name(&edef.name, ns),
        name = edef.name,
        fields = fields.join("\n        ,"),
    )
}

fn generate_field_decoder(field: &ast::FieldNode, ns: &str) -> String {
    format!(
        "|> required \"{name}\" {decoder}",
        name = field.pair.name,
        decoder = to_atom(generate_type_decoder(&field.pair.type_ident, ns)),
    )
}

pub(crate) fn generate_type_decoder(type_ident: &ast::TypeIdent, ns: &str) -> String {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => generate_atom_decoder(atom, ns),
        ast::TypeIdent::List(inner) => {
            format!("D.list {}", to_atom(generate_type_decoder(inner, ns)))
        }
        ast::TypeIdent::Option(inner) => format!(
            "{}builtinDecodeOption {}",
            ns,
            to_atom(generate_type_decoder(inner, ns))
        ),
        ast::TypeIdent::Result(ok, err) => format!(
            "{}builtinDecodeResult {} {}",
            ns,
            to_atom(generate_type_decoder(err, ns)),
            to_atom(generate_type_decoder(ok, ns))
        ),
        ast::TypeIdent::Map(key, value) => {
            // TODO: elm supports more than D.string, every comparable type
            assert_eq!(
                generate_type_decoder(key, ns),
                "D.string",
                "elm only supports dict keys"
            );
            format!("D.dict {}", to_atom(generate_type_decoder(value, ns)))
        }
        ast::TypeIdent::Tuple(tdef) => generate_tuple_decoder(tdef, ns),
        ast::TypeIdent::UserDefined(ident) => decoder_name(ident, ns),
    }
}

fn generate_tuple_decoder(tdef: &ast::TupleDef, ns: &str) -> String {
    let len = tdef.elements().len();
    let parts: Vec<String> = (0..len).map(|i| format!("x{}", i)).collect();

    format!(
        "D.succeed (\\{tuple_from} -> ({tuple_to})) {field_decoders}",
        tuple_from = parts.iter().join(" "),
        tuple_to = parts.iter().join(", "),
        field_decoders = generate_components_by_index_pipeline(tdef, ns),
    )
}

fn generate_components_by_index_pipeline(tuple: &ast::TupleDef, ns: &str) -> String {
    tuple
        .elements()
        .iter()
        .enumerate()
        .map(|(index, element)| {
            let decoder = to_atom(generate_type_decoder(&element, ns));
            format!("|> requiredIdx {} {}", index, decoder)
        })
        .join(" ")
}

fn generate_atom_decoder(atom: &ast::AtomType, ns: &str) -> String {
    match atom {
        ast::AtomType::Empty => "D.null ()".to_string(),
        ast::AtomType::Str => "D.string".to_string(),
        ast::AtomType::I32 => "D.int".to_string(),
        ast::AtomType::U32 => "D.int".to_string(),
        ast::AtomType::U8 => "D.int".to_string(),
        ast::AtomType::F64 => "D.float".to_string(),
        ast::AtomType::Bool => "D.bool".to_string(),
        ast::AtomType::DateTime => format!("{}builtinDecodeIso8601", ns),
        ast::AtomType::Date => format!("{}builtinDecodeDate", ns),
        ast::AtomType::Uuid => "D.string".to_string(),
        ast::AtomType::Bytes => "D.string".to_string(),
    }
}

/// Construct decoder function name.
pub(crate) fn decoder_name(ident: &str, ns: &str) -> String {
    format!("{}decode{}", ns, ident.to_pascal_case())
}
