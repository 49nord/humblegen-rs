//! Elm code generator.

use crate::ast;
use inflector::cases::camelcase::to_camel_case;
use itertools::Itertools;

/// Elm module preamble
///
/// The preamble is inserted into every generated Elm module and contains shared functions used by
/// the generated code.
const PREAMBLE: &str = "module Protocol exposing (..)
import Date -- justinmimbs/date
import Dict exposing (Dict)
import Iso8601  -- rtfeldman/elm-iso8601-date-strings
-- TODO: Do not require `Iso8601`, `Time` import, have humblegen include it only when needed
import Json.Decode as D
import Json.Encode as E
import Time  -- elm/time

required : String -> D.Decoder a -> D.Decoder (a -> b) -> D.Decoder b
required fieldName itemDecoder functionDecoder =
  D.map2 (|>) (D.field fieldName itemDecoder) functionDecoder

requiredIdx : Int -> D.Decoder a -> D.Decoder (a -> b) -> D.Decoder b
requiredIdx idx itemDecoder functionDecoder =
    D.map2 (|>) (D.index idx itemDecoder) functionDecoder

enumFailDecoder : D.Decoder (Maybe t) -> D.Decoder t
enumFailDecoder =
    D.andThen
        (\\x ->
            case x of
                Just v ->
                    D.succeed v

                Nothing ->
                    D.fail \"invalid enum string value\"
        )

encMaybe : (t -> E.Value) -> Maybe t -> E.Value
encMaybe enc = Maybe.withDefault E.null << Maybe.map enc

dateDecoder : D.Decoder Date.Date
dateDecoder =
    D.map Date.fromIsoString D.string
    |> D.andThen
        (\\result ->
            case result of
                Ok v ->
                    D.succeed v

                Err errMsg ->
                    D.fail <| \"not a valid date: \" ++ errMsg
        )

encDate : Date.Date -> E.Value
encDate = Date.toIsoString >> E.string


\n\n";

/// Render a definition.
fn render_def(spec: &ast::Spec) -> String {
    spec.iter()
        .map(|spec_item| match spec_item {
            ast::SpecItem::StructDef(sdef) => render_struct_def(sdef),
            ast::SpecItem::EnumDef(edef) => render_enum_def(edef),
        })
        .join("\n\n")
}

/// Render a struct definition.
fn render_struct_def(sdef: &ast::StructDef) -> String {
    format!(
        "type alias {name} = {{ {fields} }}",
        name = sdef.name,
        fields = sdef.fields.iter().map(render_struct_field).join(", ")
    )
}

/// Render an enum definition.
fn render_enum_def(edef: &ast::EnumDef) -> String {
    let variants: Vec<_> = edef.variants.iter().map(render_variant_def).collect();

    format!(
        "type {name} = {variants}",
        name = edef.name,
        variants = variants.join(" | ")
    )
}

/// Render a struct field.
fn render_struct_field(field: &ast::FieldNode) -> String {
    format!(
        "{name}: {ty}",
        name = field_name(&field.name),
        ty = render_type_ident(&field.type_ident)
    )
}

/// Add optional parenthesis is necessary.
///
/// Wraps `s` in parentheses if it contains a space.
fn opt_parens(s: String) -> String {
    if s.contains(' ') {
        format!("({})", s)
    } else {
        s
    }
}

/// Render a variant definition.
fn render_variant_def(variant: &ast::VariantDef) -> String {
    match variant.variant_type {
        ast::VariantType::Simple => variant.name.clone(),
        ast::VariantType::Tuple(ref fields) => format!(
            "{name} {fields}",
            name = variant.name,
            fields = fields
                .components()
                .iter()
                .map(render_type_ident)
                .map(opt_parens)
                .join(" ")
        ),
        ast::VariantType::Struct(ref fields) => format!(
            "{name} {{ {fields} }}",
            name = variant.name,
            fields = fields.iter().map(render_struct_field).join(", ")
        ),
        ast::VariantType::Newtype(ref ty) => format!(
            "{name} {ty}",
            name = variant.name,
            ty = render_type_ident(ty),
        ),
    }
}

/// Render a type identifier.
fn render_type_ident(type_ident: &ast::TypeIdent) -> String {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => render_atom(atom),
        ast::TypeIdent::List(inner) => format!("List {}", opt_parens(render_type_ident(inner))),
        ast::TypeIdent::Option(inner) => format!("Maybe {}", opt_parens(render_type_ident(inner))),
        ast::TypeIdent::Map(key, value) => format!(
            "Dict {} {}",
            opt_parens(render_type_ident(key)),
            opt_parens(render_type_ident(value)),
        ),
        ast::TypeIdent::Tuple(tdef) => render_tuple_def(tdef),
        ast::TypeIdent::UserDefined(ident) => ident.to_owned(),
    }
}

/// Render a tuple definition.
fn render_tuple_def(tdef: &ast::TupleDef) -> String {
    format!(
        "({})",
        tdef.components().iter().map(render_type_ident).join(", ")
    )
}

/// Render an atomic type.
fn render_atom(atom: &ast::AtomType) -> String {
    match atom {
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

/// Render decoders for a spec.
///
/// This is a top-level function similar to `render_def`.
fn render_decoders(spec: &ast::Spec) -> String {
    spec.iter()
        .map(|spec_item| match spec_item {
            ast::SpecItem::StructDef(sdef) => render_struct_decoder(sdef),
            ast::SpecItem::EnumDef(edef) => render_enum_decoder(edef),
        })
        .join("\n\n")
}

/// Render decoder for a struct.
fn render_struct_decoder(sdef: &ast::StructDef) -> String {
    format!(
        "{dec_name} : D.Decoder {name} \n\
         {dec_name} = D.succeed {name} {field_decoders}",
        dec_name = decoder_name(&sdef.name),
        name = sdef.name,
        field_decoders = sdef.fields.iter().map(render_field_decoder).join(" ")
    )
}

/// Render decoder for an enum.
fn render_enum_decoder(edef: &ast::EnumDef) -> String {
    let optional_string_decoder = if edef.simple_variants().count() > 0 {
        format!(
            "enumFailDecoder (D.map {string_enum_parser} D.string){opt_comma}",
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
            type_dec = opt_parens(render_variant_decoder(variant)),
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

/// Render decoder for a field.
fn render_field_decoder(field: &ast::FieldNode) -> String {
    format!(
        "|> required \"{name}\" {decoder}",
        name = field.name,
        decoder = opt_parens(render_type_decoder(&field.type_ident)),
    )
}

/// Render decoder for an enum variant.
fn render_variant_decoder(variant: &ast::VariantDef) -> String {
    match variant.variant_type {
        ast::VariantType::Simple => unreachable!("cannot build enum decoder for simple variant"),
        ast::VariantType::Tuple(ref components) => format!(
            "D.succeed {name} {components}",
            name = variant.name,
            components = render_components_by_index_pipeline(components)
        ),
        ast::VariantType::Struct(ref fields) => format!(
            "D.succeed (\\{unnamed_args} -> {name} {{ {struct_assignment} }}) {field_decoders}",
            name = variant.name,
            unnamed_args = (0..fields.0.len()).map(|i| format!("x{}", i)).join(" "),
            struct_assignment = fields
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    format!("{name} = x{arg}", name = field_name(&field.name), arg = idx)
                })
                .join(", "),
            field_decoders = fields.iter().map(render_field_decoder).join(" "),
        ),
        ast::VariantType::Newtype(ref ty) => format!(
            "D.map {name} {ty}",
            name = variant.name,
            ty = opt_parens(render_type_decoder(ty)),
        ),
    }
}

/// Render a decoder for a type.
fn render_type_decoder(type_ident: &ast::TypeIdent) -> String {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => render_atom_decoder(atom),
        ast::TypeIdent::List(inner) => format!("D.list {}", opt_parens(render_type_decoder(inner))),
        ast::TypeIdent::Option(inner) => {
            format!("D.maybe {}", opt_parens(render_type_decoder(inner)))
        }
        ast::TypeIdent::Map(key, value) => {
            assert_eq!(
                render_type_decoder(key),
                "D.string",
                "elm only supports dict keys"
            );
            format!("D.dict {}", opt_parens(render_type_decoder(value)))
        }
        ast::TypeIdent::Tuple(tdef) => render_tuple_decoder(tdef),
        ast::TypeIdent::UserDefined(ident) => decoder_name(ident),
    }
}

/// Render a decoder for a tuple.
fn render_tuple_decoder(tdef: &ast::TupleDef) -> String {
    let len = tdef.components().len();
    let parts: Vec<String> = (0..len).map(|i| format!("x{}", i)).collect();

    format!(
        "D.succeed (\\{tuple_from} -> ({tuple_to})) {field_decoders}",
        tuple_from = parts.iter().join(" "),
        tuple_to = parts.iter().join(", "),
        field_decoders = render_components_by_index_pipeline(tdef),
    )
}

/// Render a pipeline that decodes tuple fields by index.
fn render_components_by_index_pipeline(tdef: &ast::TupleDef) -> String {
    let len = tdef.components().len();

    (0..len)
        .map(|i| {
            format!(
                "|> requiredIdx {idx} {decoder}",
                idx = i,
                decoder = opt_parens(render_type_decoder(&tdef.components()[i]))
            )
        })
        .join(" ")
}

/// Render a decoder for an atomic type.
fn render_atom_decoder(atom: &ast::AtomType) -> String {
    match atom {
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

/// Render decoding helper functions for a spec.
fn render_helpers(spec: &ast::Spec) -> String {
    spec.iter()
        .map(|spec_item| match spec_item {
            // No helpers for structs.
            ast::SpecItem::StructDef(_) => "".to_string(),
            ast::SpecItem::EnumDef(edef) => render_enum_helpers(edef),
        })
        .join("\n\n")
}

/// Render helper functions for enum decoders.
fn render_enum_helpers(edef: &ast::EnumDef) -> String {
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
            .join("\n"),
        indent = "  ",
    )
}

/// Construct name of encoder function for specific `ident`.
fn encoder_name(ident: &str) -> String {
    to_camel_case(&format!("encode{}", ident))
}

/// Render encoder functions for `spec`.
fn render_encoders(spec: &ast::Spec) -> String {
    spec.iter()
        .map(|spec_item| match spec_item {
            ast::SpecItem::StructDef(sdef) => render_struct_encoder(sdef),
            ast::SpecItem::EnumDef(edef) => render_enum_encoder(edef),
        })
        .join("\n\n")
}

/// Render a struct encoder.
fn render_struct_encoder(sdef: &ast::StructDef) -> String {
    format!(
        "{encoder_name} : {type_name} -> E.Value\n\
         {encoder_name} obj = E.object [{fields}]",
        encoder_name = encoder_name(&sdef.name),
        type_name = sdef.name,
        fields = sdef.fields.iter().map(render_field_encoder).join(", "),
    )
}

/// Render an enum encoder.
fn render_enum_encoder(edef: &ast::EnumDef) -> String {
    format!(
        "{encoder_name} : {type_name} -> E.Value\n\
         {encoder_name} v = case v of \n\
         {variants}\n",
        encoder_name = encoder_name(&edef.name),
        type_name = edef.name,
        variants = edef
            .variants
            .iter()
            .map(render_variant_encoder_branch)
            .map(|s| format!("  {}", s))
            .join("\n"),
    )
}

/// Render a field encoder.
fn render_field_encoder(field: &ast::FieldNode) -> String {
    format!(
        "(\"{name}\", {value_encoder} obj.{field_name})",
        name = field.name,
        field_name = field_name(&field.name),
        value_encoder = opt_parens(render_type_encoder(&field.type_ident))
    )
}

/// Render encoding code for variant of enum.
fn render_variant_encoder_branch(variant: &ast::VariantDef) -> String {
    match variant.variant_type {
        ast::VariantType::Simple => format!("{name} -> E.string \"{name}\"", name = variant.name),
        ast::VariantType::Tuple(ref tdef) => format!(
            "{name} {field_names} -> E.object [ (\"{name}\", E.list identity [{field_encoders}]) ]",
            name = variant.name,
            field_names = (0..tdef.components().len())
                .map(|i| format!("x{}", i))
                .join(" "),
            field_encoders = tdef
                .components()
                .iter()
                .enumerate()
                .map(|(idx, component)| format!("{} x{}", render_type_encoder(component), idx))
                .join(", "),
        ),
        ast::VariantType::Struct(ref fields) => format!(
            "{name} obj -> E.object [ (\"{name}\", E.object [{fields}]) ]",
            name = variant.name,
            fields = fields.iter().map(render_field_encoder).join(", "),
        ),
        ast::VariantType::Newtype(ref ty) => format!(
            "{name} obj -> E.object [ (\"{name}\", {enc} obj) ]",
            name = variant.name,
            enc = render_type_encoder(ty),
        ),
    }
}

/// Render a type encoder.
fn render_type_encoder(type_ident: &ast::TypeIdent) -> String {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => render_atom_encoder(atom),
        ast::TypeIdent::List(inner) => format!("E.list {}", opt_parens(render_type_encoder(inner))),
        ast::TypeIdent::Option(inner) => {
            format!("encMaybe {}", opt_parens(render_type_encoder(inner)))
        }
        ast::TypeIdent::Map(key, value) => {
            assert_eq!(
                render_type_encoder(key),
                "E.string",
                "can only encode string keys in maps"
            );
            format!("E.dict identity {}", opt_parens(render_type_encoder(value)))
        }
        ast::TypeIdent::Tuple(tdef) => render_tuple_encoder(tdef),
        ast::TypeIdent::UserDefined(ident) => encoder_name(ident),
    }
}

/// Render an atomic type encoder.
fn render_atom_encoder(atom: &ast::AtomType) -> String {
    match atom {
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

/// Render a tuple encoder.
fn render_tuple_encoder(tdef: &ast::TupleDef) -> String {
    format!(
        "\\({field_names}) -> E.list identity [ {encode_values} ]",
        field_names = (0..tdef.components().len())
            .map(|i| format!("x{}", i))
            .join(", "),
        encode_values = tdef
            .components()
            .iter()
            .enumerate()
            .map(|(idx, component)| format!("{} x{}", render_type_encoder(component), idx))
            .join(", "),
    )
}

/// Render all code for `spec`.
pub fn render(spec: &ast::Spec) -> String {
    // Add preamble and return.
    format!(
        "{}\n{}\n{}\n{}\n{}\n",
        PREAMBLE,
        render_def(spec),
        render_helpers(spec),
        render_decoders(spec),
        render_encoders(spec),
    )
}
