//! Elm code generator.

use anyhow::{Result};
use crate::{Spec, ast, Artifact, LibError};
use inflector::cases::camelcase::to_camel_case;
use itertools::Itertools;
use std::{fs::File, path::Path, io::Write};

const BACKEND_NAME : &str = "elm";

/// Generate elm code for a docstring.
///
/// If not present, generates an empty string.
fn generate_doc_comment(doc_comment: &Option<String>) -> String {
    match doc_comment {
        Some(ref ds) => format!("{{-| {ds}\n-}}\n", ds = ds),
        None => "".to_owned(),
    }
}

// TODO: Elm does not allow documentation on members, so the docs need to be converted to markdown
//       lists instead. This is true for `type alias` struct fields as well as enum variants.

/// Generate elm code for a user-defined type.
fn generate_def(spec: &ast::Spec) -> String {
    spec.iter()
        .filter_map(|spec_item| match spec_item {
            ast::SpecItem::StructDef(sdef) => Some(generate_struct_def(sdef)),
            ast::SpecItem::EnumDef(edef) => Some(generate_enum_def(edef)),
            ast::SpecItem::ServiceDef(_) => None,
        })
        .join("\n\n\n")
}

/// Generate elm code for a struct definition.
pub(crate) fn generate_struct_def(sdef: &ast::StructDef) -> String {
    format!(
        "{doc_comment}type alias {name} =\n    {{ {fields}\n    }}",
        doc_comment = generate_doc_comment(&sdef.doc_comment),
        name = sdef.name,
        fields = sdef.fields.iter().map(generate_struct_field).join("\n    , ")
    )
}

/// Generate elm code for an enum definition.
pub(crate) fn generate_enum_def(edef: &ast::EnumDef) -> String {
    let variants: Vec<_> = edef.variants.iter().map(generate_variant_def).collect();

    format!(
        "{doc_comment}type {name}\n    = {variants}",
        doc_comment = generate_doc_comment(&edef.doc_comment),
        name = edef.name,
        variants = variants.join("\n    | ")
    )
}

/// Generate elm code for a struct field.
fn generate_struct_field(field: &ast::FieldNode) -> String {
    format!(
        "{name}: {ty}",
        name = field_name(&field.pair.name),
        ty = generate_type_ident(&field.pair.type_ident)
    )
}

/// Add parenthesis if necessary.
///
/// Wraps `s` in parentheses if it contains a space.
fn to_atom(s: String) -> String {
    if s.contains(' ') {
        format!("({})", s)
    } else {
        s
    }
}

/// Generate elm code for a variant definition.
fn generate_variant_def(variant: &ast::VariantDef) -> String {
    match variant.variant_type {
        ast::VariantType::Simple => variant.name.clone(),
        ast::VariantType::Tuple(ref fields) => format!(
            "{name} {fields}",
            name = variant.name,
            fields = fields
                .elements()
                .iter()
                .map(generate_type_ident)
                .map(to_atom)
                .join(" ")
        ),
        ast::VariantType::Struct(ref fields) => format!(
            "{name} {{ {fields} }}",
            name = variant.name,
            fields = fields.iter().map(generate_struct_field).join(", ")
        ),
        ast::VariantType::Newtype(ref ty) => format!(
            "{name} {ty}",
            name = variant.name,
            ty = generate_type_ident(ty),
        ),
    }
}

/// Generate elm code for a type identifier.
fn generate_type_ident(type_ident: &ast::TypeIdent) -> String {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => generate_atom(atom),
        ast::TypeIdent::List(inner) => format!("List {}", to_atom(generate_type_ident(inner))),
        ast::TypeIdent::Option(inner) => format!("Maybe {}", to_atom(generate_type_ident(inner))),
        ast::TypeIdent::Result(_okk, _err) => todo!(),
        ast::TypeIdent::Map(key, value) => format!(
            "Dict {} {}",
            to_atom(generate_type_ident(key)),
            to_atom(generate_type_ident(value)),
        ),
        ast::TypeIdent::Tuple(tdef) => generate_tuple_def(tdef),
        ast::TypeIdent::UserDefined(ident) => ident.to_owned(),
    }
}

/// Generate elm code for a tuple definition.
fn generate_tuple_def(tdef: &ast::TupleDef) -> String {
    format!(
        "({})",
        tdef.elements().iter().map(generate_type_ident).join(", ")
    )
}

/// Generate elm code for an atomic type.
fn generate_atom(atom: &ast::AtomType) -> String {
    match atom {
        ast::AtomType::Empty => "()",
        ast::AtomType::Str => "String",
        ast::AtomType::I32 => "Int",
        ast::AtomType::U32 => "Int",
        ast::AtomType::U8 => "Int",
        ast::AtomType::F64 => "Float",
        ast::AtomType::Bool => "Bool",
        ast::AtomType::DateTime => "Time.Posix",
        ast::AtomType::Date => "Date.Date",
    }
    .to_owned()
}

/// Generate elm code for decoders for a spec.
fn generate_type_decoders(spec: &ast::Spec) -> String {
    spec.iter()
        .filter_map(|spec_item| match spec_item {
            ast::SpecItem::StructDef(sdef) => Some(generate_struct_decoder(sdef)),
            ast::SpecItem::EnumDef(edef) => Some(generate_enum_decoder(edef)),
            ast::SpecItem::ServiceDef(_) => None,
        })
        .join("\n\n")
}

/// Generate elm code for decoder for a struct.
fn generate_struct_decoder(sdef: &ast::StructDef) -> String {
    format!(
        "{dec_name} : D.Decoder {name} \n\
         {dec_name} = D.succeed {name} {field_decoders}",
        dec_name = decoder_name(&sdef.name),
        name = sdef.name,
        field_decoders = sdef.fields.iter().map(generate_field_decoder).join(" ")
    )
}

/// Generate elm code for decoder for an enum.
fn generate_enum_decoder(edef: &ast::EnumDef) -> String {
    let optional_string_decoder = if edef.simple_variants().count() > 0 {
        format!(
            "unwrapDecoder (D.map {string_enum_parser} D.string){opt_comma}",
            string_enum_parser = enum_string_decoder_name(&edef.name),
            opt_comma = if edef.complex_variants().count() > 0 {
                ", "
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
        "{dec_name} : D.Decoder {name} \n\
         {dec_name} = D.oneOf [{optional_string_decoder} {fields}]\n",
        dec_name = decoder_name(&edef.name),
        name = edef.name,
        optional_string_decoder = optional_string_decoder,
        fields = fields.join(", "),
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
        ast::VariantType::Simple => unreachable!("cannot build enum decoder for simple variant"),
        ast::VariantType::Tuple(ref components) => format!(
            "D.succeed {name} {components}",
            name = variant.name,
            components = generate_components_by_index_pipeline(components)
        ),
        ast::VariantType::Struct(ref fields) => format!(
            "D.succeed (\\{unnamed_args} -> {name} {{ {struct_assignment} }}) {field_decoders}",
            name = variant.name,
            unnamed_args = (0..fields.0.len()).map(|i| format!("x{}", i)).join(" "),
            struct_assignment = fields
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    format!(
                        "{name} = x{arg}",
                        name = field_name(&field.pair.name),
                        arg = idx
                    )
                })
                .join(", "),
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
        ast::TypeIdent::List(inner) => format!("D.list {}", to_atom(generate_type_decoder(inner))),
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
    tuple.elements().iter().enumerate().map(|(index, element)| {
            let decoder = to_atom(generate_type_decoder(&element));
            format!("|> requiredIdx {} {}", index, decoder)
    }).join(" ")
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

/// Construct name for a field.
fn field_name(ident: &str) -> String {
    to_camel_case(ident)
}

fn generate_rest_api_client_helpers(spec: &ast::Spec) -> String {
    spec.iter()
        .filter_map(|spec_item| match spec_item {
            ast::SpecItem::StructDef(_) | ast::SpecItem::ServiceDef(_) => None,
            ast::SpecItem::EnumDef(edef) => Some(generate_enum_helpers(edef)),
        })
        .join("")
}

fn generate_rest_api_clients(spec: &ast::Spec) -> String {
    generate_rest_api_client_helpers(spec);
    spec.iter()
        .filter_map(|spec_item| match spec_item {
            // No helpers for structs.
            ast::SpecItem::StructDef(_) | ast::SpecItem::EnumDef(_) => None,
            ast::SpecItem::ServiceDef(service) => Some(generate_rest_api_client(service)),
        })
        .join("")
}

fn generate_rest_api_client(spec: &ast::ServiceDef) -> String { 
    unimplemented!()
}

/// Generate elm code for helper functions for enum decoders.
fn generate_enum_helpers(edef: &ast::EnumDef) -> String {
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

/// Construct name of encoder function for specific `ident`.
fn encoder_name(ident: &str) -> String {
    to_camel_case(&format!("encode{}", ident))
}

/// Generate elm code for encoder functions for `spec`.
fn generate_type_encoders(spec: &ast::Spec) -> String {
    spec.iter()
        .filter_map(|spec_item| match spec_item {
            ast::SpecItem::StructDef(sdef) => Some(generate_struct_encoder(sdef)),
            ast::SpecItem::EnumDef(edef) => Some(generate_enum_encoder(edef)),
            ast::SpecItem::ServiceDef(_) => None,
        })
        .join("\n\n")
}

/// Generate elm code for a struct encoder.
fn generate_struct_encoder(sdef: &ast::StructDef) -> String {
    format!(
        "{encoder_name} : {type_name} -> E.Value\n\
         {encoder_name} obj = E.object [{fields}]",
        encoder_name = encoder_name(&sdef.name),
        type_name = sdef.name,
        fields = sdef.fields.iter().map(generate_field_encoder).join(", "),
    )
}

/// Generate elm code for an enum encoder.
fn generate_enum_encoder(edef: &ast::EnumDef) -> String {
    format!(
        "{encoder_name} : {type_name} -> E.Value\n\
         {encoder_name} v = case v of \n\
         {variants}\n",
        encoder_name = encoder_name(&edef.name),
        type_name = edef.name,
        variants = edef
            .variants
            .iter()
            .map(generate_variant_encoder_branch)
            .map(|s| format!("  {}", s))
            .join("\n"),
    )
}

/// Generate elm code for a field encoder.
fn generate_field_encoder(field: &ast::FieldNode) -> String {
    format!(
        "(\"{name}\", {value_encoder} obj.{field_name})",
        name = field.pair.name,
        field_name = field_name(&field.pair.name),
        value_encoder = to_atom(generate_type_encoder(&field.pair.type_ident))
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
        ast::TypeIdent::List(inner) => format!("E.list {}", to_atom(generate_type_encoder(inner))),
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

pub struct Generator {
    artifact : Artifact,
}

impl Generator {
    pub fn new(artifact :Artifact) -> Result<Self, LibError> {
        match artifact {
            Artifact::TypesOnly | Artifact::ClientEndpoints => Ok(Self { artifact }),
            Artifact::ServerEndpoints => Err(LibError::UnsupportedArtifact {
                artifact, backend: BACKEND_NAME
                
            })
        }
    }

    pub fn generate_spec(&self, spec: &Spec) -> String {
        let generate_client_side_services = self.artifact == Artifact::ClientEndpoints && spec
            .iter()
            .find(|item| item.service_def().is_some())
            .is_some();

        let defs = generate_def(spec);

        let mut outfile = vec![
            include_str!("elm/module_header.elm"),
            include_str!("elm/preamble_types.elm"),
            if generate_client_side_services {
                include_str!("elm/preamble_services.elm")
            } else {
                ""
            },
            &defs,
            include_str!("elm/utils_types.elm"),
        ];

        if generate_client_side_services {
            let decoders = generate_type_decoders(spec);
            let encoders = generate_type_encoders(spec);
            let clients = generate_rest_api_clients(spec);
            let client_side_code :Vec<&str> = vec![ &decoders, &encoders, &clients ];
            outfile.extend(client_side_code);
            outfile.join("\n")
        } else {
            outfile.join("\n")
        }
    }
}

impl crate::CodeGenerator for Generator {
    fn generate(&self, spec :&Spec, output: &Path) -> Result<(), LibError> {
        let generated_code = self.generate_spec(spec);

        // TODO: support folder as output path
        let mut outfile = File::create(&output).map_err(LibError::IoError)?;
        outfile.write_all(generated_code.as_bytes()).map_err(LibError::IoError)?;
        Ok(())
    }
}