//! The humble language parser.

use itertools::Itertools;
use pest::Parser;
use pest_derive::Parser;

/// The humble language parser, derived from the PEST grammar file.
#[derive(Parser)]
#[grammar = "humble.pest"]
pub struct HumbleParser;

use crate::ast::*;

/// Parse a doc comment.
///
/// Will peek at the `pairs` to see if the next item is a doc comment. If it is, remove it and
/// return in cleaned up string form.
fn parse_doc_comment<'i>(pairs: &mut pest::iterators::Pairs<'i, Rule>) -> Option<String> {
    match pairs.peek() {
        Some(pair) if pair.as_rule() == Rule::doc_comment => {
            let doc_comment = pairs.next().unwrap();
            let lines = doc_comment.into_inner();
            let s: String = lines
                .map(|line| line.into_inner().next().unwrap().as_span().as_str())
                .join("\n");

            Some(s)
        }
        _ => None,
    }
}

/// Parse a struct definition.
fn parse_struct_definition(pair: pest::iterators::Pair<Rule>) -> StructDef {
    let mut nodes = pair.into_inner();

    let doc_comment = parse_doc_comment(&mut nodes);

    let name = nodes.next().unwrap().as_span().as_str().to_string();
    let fields = parse_struct_fields(nodes.next().unwrap());

    StructDef {
        name,
        fields,
        doc_comment,
    }
}

/// Parse inner struct fields of struct definition.
fn parse_struct_fields(pair: pest::iterators::Pair<Rule>) -> StructFields {
    let fields: Vec<_> = pair.into_inner().map(parse_struct_field_def).collect();
    StructFields(fields)
}

/// Parse enum definition.
fn parse_enum_definition(pair: pest::iterators::Pair<Rule>) -> EnumDef {
    let mut outer_nodes = pair.into_inner();
    let doc_comment = parse_doc_comment(&mut outer_nodes);
    let mut nodes = outer_nodes.next().unwrap().into_inner();
    let name = nodes.next().unwrap().as_span().as_str().to_string();
    let variants = nodes.map(parse_enum_variant_def).collect();

    EnumDef {
        name,
        variants,
        doc_comment,
    }
}

/// Parse enum variant definitions.
fn parse_enum_variant_def(pair: pest::iterators::Pair<Rule>) -> VariantDef {
    let mut nodes = pair.into_inner();
    let doc_comment = parse_doc_comment(&mut nodes);
    let name = nodes.next().unwrap().as_span().as_str().to_string();

    if let Some(var) = nodes.next() {
        match var.as_rule() {
            Rule::struct_fields => VariantDef {
                name,
                variant_type: VariantType::Struct(parse_struct_fields(var)),
                doc_comment,
            },
            Rule::tuple_def => VariantDef {
                name,
                variant_type: VariantType::Tuple(parse_tuple_def(var)),
                doc_comment,
            },
            Rule::newtype_def => VariantDef {
                name,
                variant_type: VariantType::Newtype(parse_type_ident(
                    var.into_inner().next().unwrap(),
                )),
                doc_comment,
            },
            _ => unreachable!(dbg!(var)),
        }
    } else {
        VariantDef {
            name,
            variant_type: VariantType::Simple,
            doc_comment,
        }
    }
}

/// Parse field definitions in struct.
fn parse_struct_field_def(pair: pest::iterators::Pair<Rule>) -> FieldNode {
    let mut nodes = pair.into_inner();
    let doc_comment = parse_doc_comment(&mut nodes);
    let name = nodes.next().unwrap().as_span().as_str().to_string();
    let type_ident = parse_type_ident(nodes.next().unwrap());

    FieldNode {
        name,
        type_ident,
        doc_comment,
    }
}

/// Parse type identifier.
fn parse_type_ident(pair: pest::iterators::Pair<Rule>) -> TypeIdent {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::built_in_atom => TypeIdent::BuiltIn(parse_built_in_atom(inner)),
        Rule::list_type => parse_list_type(inner),
        Rule::option_type => parse_option_type(inner),
        Rule::map_type => parse_map_type(inner),
        Rule::tuple_def => TypeIdent::Tuple(parse_tuple_def(inner)),
        Rule::camel_case_ident => TypeIdent::UserDefined(inner.as_span().as_str().to_string()),
        _ => unreachable!(dbg!(inner)),
    }
}

/// Parse a built-in atomic type.
fn parse_built_in_atom(pair: pest::iterators::Pair<Rule>) -> AtomType {
    match pair.as_span().as_str() {
        "str" => AtomType::Str,
        "i32" => AtomType::I32,
        "u32" => AtomType::U32,
        "u8" => AtomType::U8,
        "f64" => AtomType::F64,
        "bool" => AtomType::Bool,
        "datetime" => AtomType::DateTime,
        "date" => AtomType::Date,
        _ => unreachable!(dbg!(pair)),
    }
}

/// Parse a list type.
fn parse_list_type(pair: pest::iterators::Pair<Rule>) -> TypeIdent {
    let inner = pair.into_inner().next().unwrap();

    TypeIdent::List(Box::new(parse_type_ident(inner)))
}

/// Parse a optional type.
fn parse_option_type(pair: pest::iterators::Pair<Rule>) -> TypeIdent {
    let inner = pair.into_inner().next().unwrap();

    TypeIdent::Option(Box::new(parse_type_ident(inner)))
}

/// Parse a map type.
fn parse_map_type(pair: pest::iterators::Pair<Rule>) -> TypeIdent {
    let mut inners = pair.into_inner();
    let key_type = inners.next().unwrap();
    let value_type = inners.next().unwrap();

    TypeIdent::Map(
        Box::new(parse_type_ident(key_type)),
        Box::new(parse_type_ident(value_type)),
    )
}

/// Parse a tuple definition.
fn parse_tuple_def(pair: pest::iterators::Pair<Rule>) -> TupleDef {
    TupleDef(pair.into_inner().map(parse_type_ident).collect())
}

/// Parse a spec item (`struct` or `enum`).
fn parse_spec_item(pair: pest::iterators::Pair<Rule>) -> SpecItem {
    match pair.as_rule() {
        Rule::struct_definition => SpecItem::StructDef(parse_struct_definition(pair)),
        Rule::enum_definition => SpecItem::EnumDef(parse_enum_definition(pair)),
        _ => unreachable!(dbg!(pair)),
    }
}

/// Parse complete spec.
pub fn parse(input: &str) -> Spec {
    // TODO: Returns errors proper.
    let humbled = HumbleParser::parse(Rule::doc, input)
        .unwrap()
        .next()
        .unwrap();

    Spec(humbled.into_inner().map(parse_spec_item).collect())
}
