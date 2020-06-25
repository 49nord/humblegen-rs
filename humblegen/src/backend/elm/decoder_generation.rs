use super::{to_atom, to_camel_case};
use crate::ast;

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

/// Generate elm code for helper functions for enum decoders.
pub fn generate_enum_helpers(edef: &ast::EnumDef) -> String {
    format!(
        "{fname} : String -> Maybe {type_name}\n\
        {fname} s = case s of \n\
        {variant_decoders}\n\
        {indent}_ -> Nothing\n",
        fname = enum_string_decoder_name(&edef.name),
        type_name = edef.name,
        variant_decoders = edef
            .simple_variants()
            .map(|variant| format!("  \"{name}\" -> Just {name}", name = variant.name))
            .join("\n\n"),
        indent = "  ",
    )
}

/// Generate elm code for decoder for a struct.
fn generate_struct_decoder(sdef: &ast::StructDef) -> String {
    format!(
        "{dec_name} : D.Decoder {name} \n\
        {dec_name} =\n   D.succeed {name}\n        {field_decoders}",
        dec_name = decoder_name(&sdef.name),
        name = sdef.name,
        field_decoders = sdef
            .fields
            .iter()
            .map(generate_field_decoder)
            .join("\n        ")
    )
}

/// Generate elm code for decoder for an enum.
fn generate_enum_decoder(edef: &ast::EnumDef) -> String {
    let optional_string_decoder = if edef.simple_variants().count() > 0 {
        format!(
            "unwrapDecoder (D.map {string_enum_parser} D.string){opt_comma}",
            string_enum_parser = enum_string_decoder_name(&edef.name),
            opt_comma = if edef.complex_variants().count() > 0 {
                "\n        ,"
            } else {
                ""
            }
        )
    } else {
        "".to_owned()
    };

    let mut fields = edef.complex_variants().map(|variant| {
        format!(
            "D.field \"{field_name}\" {type_dec}",
            field_name = variant.name,
            type_dec = to_atom(generate_variant_decoder(variant)),
        )
    });

    format!(
        "{dec_name} : D.Decoder {name}\n{dec_name} =\n    D.oneOf\n        [{optional_string_decoder} {fields}\n        ]",
        dec_name = decoder_name(&edef.name),
        name = edef.name,
        optional_string_decoder = optional_string_decoder,
        fields = fields.join("\n        ,"),
    )
}

/// Generate elm code for decoder for a field.
fn generate_field_decoder(field: &ast::FieldNode) -> String {
    format!(
        "|> required \"{name}\" {decoder}",
        name = field.pair.name,
        decoder = to_atom(generate_type_decoder(&field.pair.type_ident)),
    )
}

/// Generate elm code for decoder for an enum variant.
fn generate_variant_decoder(variant: &ast::VariantDef) -> String {
    match variant.variant_type {
        ast::VariantType::Simple => {
            unreachable!("cannot build enum decoder for simple variant")
        }
        ast::VariantType::Tuple(ref components) => format!(
            "D.succeed {name} {components}",
            name = variant.name,
            components = generate_components_by_index_pipeline(components)
        ),
        ast::VariantType::Struct(ref fields) => format!(
            "D.succeed {name} {field_decoders}",
            name = variant.name,
            field_decoders = fields.iter().map(generate_field_decoder).join(" "),
        ),
        ast::VariantType::Newtype(ref ty) => format!(
            "D.map {name} {ty}",
            name = variant.name,
            ty = to_atom(generate_type_decoder(ty)),
        ),
    }
}

/// Generate elm code for a decoder for a type.
fn generate_type_decoder(type_ident: &ast::TypeIdent) -> String {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => generate_atom_decoder(atom),
        ast::TypeIdent::List(inner) => {
            format!("D.list {}", to_atom(generate_type_decoder(inner)))
        }
        ast::TypeIdent::Option(inner) => {
            format!("D.maybe {}", to_atom(generate_type_decoder(inner)))
        }
        ast::TypeIdent::Result(_ok, _err) => todo!(),
        ast::TypeIdent::Map(key, value) => {
            assert_eq!(
                generate_type_decoder(key),
                "D.string",
                "elm only supports dict keys"
            );
            format!("D.dict {}", to_atom(generate_type_decoder(value)))
        }
        ast::TypeIdent::Tuple(tdef) => generate_tuple_decoder(tdef),
        ast::TypeIdent::UserDefined(ident) => decoder_name(ident),
    }
}

/// Generate elm code for a decoder for a tuple.
fn generate_tuple_decoder(tdef: &ast::TupleDef) -> String {
    let len = tdef.elements().len();
    let parts: Vec<String> = (0..len).map(|i| format!("x{}", i)).collect();

    format!(
        "D.succeed (\\{tuple_from} -> ({tuple_to})) {field_decoders}",
        tuple_from = parts.iter().join(" "),
        tuple_to = parts.iter().join(", "),
        field_decoders = generate_components_by_index_pipeline(tdef),
    )
}

/// Generate elm code for a pipeline that decodes tuple fields by index.
fn generate_components_by_index_pipeline(tuple: &ast::TupleDef) -> String {
    tuple
        .elements()
        .iter()
        .enumerate()
        .map(|(index, element)| {
            let decoder = to_atom(generate_type_decoder(&element));
            format!("|> requiredIdx {} {}", index, decoder)
        })
        .join(" ")
}

/// Generate elm code for a decoder for an atomic type.
fn generate_atom_decoder(atom: &ast::AtomType) -> String {
    match atom {
        ast::AtomType::Empty => "(D.succeed ())",
        ast::AtomType::Str => "D.string",
        ast::AtomType::I32 => "D.int",
        ast::AtomType::U32 => "D.int",
        ast::AtomType::U8 => "D.int",
        ast::AtomType::F64 => "D.float",
        ast::AtomType::Bool => "D.bool",
        ast::AtomType::DateTime => "Iso8601.decoder",
        ast::AtomType::Date => "dateDecoder",
        ast::AtomType::Uuid => "D.string",
        ast::AtomType::Bytes => "D.string",
    }
    .to_string()
}

/// Construct decoder function name.
fn decoder_name(ident: &str) -> String {
    to_camel_case(&format!("{}Decoder", ident))
}

/// Construct function name for an enum decoder.
fn enum_string_decoder_name(ident: &str) -> String {
    to_camel_case(&format!("parseEnum{}FromString", ident))
}