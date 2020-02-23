use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "humble.pest"]
pub struct HumbleParser;

use crate::ast::*;

fn parse_struct_definition(pair: pest::iterators::Pair<Rule>) -> StructDef {
    let mut nodes = pair.into_inner().into_iter();
    let name = nodes.next().unwrap().as_span().as_str().to_string();
    let fields = parse_struct_fields(nodes.next().unwrap());

    StructDef { name, fields }
}

fn parse_struct_fields(pair: pest::iterators::Pair<Rule>) -> StructFields {
    let fields: Vec<_> = pair
        .into_inner()
        .into_iter()
        .map(parse_struct_field_def)
        .collect();
    StructFields(fields)
}

fn parse_enum_definition(pair: pest::iterators::Pair<Rule>) -> EnumDef {
    let mut nodes = pair.into_inner().into_iter();
    let name = nodes.next().unwrap().as_span().as_str().to_string();
    let variants = nodes.map(parse_enum_variant_def).collect();

    EnumDef { name, variants }
}

fn parse_enum_variant_def(pair: pest::iterators::Pair<Rule>) -> VariantDef {
    let mut nodes = pair.into_inner().into_iter();
    let name = nodes.next().unwrap().as_span().as_str().to_string();

    if let Some(var) = nodes.next() {
        match var.as_rule() {
            Rule::struct_fields => VariantDef {
                name,
                variant_type: VariantType::Struct(parse_struct_fields(var)),
            },
            Rule::tuple_def => VariantDef {
                name,
                variant_type: VariantType::Tuple(parse_tuple_def(var)),
            },
            _ => unreachable!(dbg!(var)),
        }
    } else {
        VariantDef {
            name,
            variant_type: VariantType::Simple,
        }
    }
}

fn parse_struct_field_def(pair: pest::iterators::Pair<Rule>) -> FieldNode {
    let mut nodes = pair.into_inner().into_iter();
    let name = nodes.next().unwrap().as_span().as_str().to_string();
    let type_ident = parse_type_ident(nodes.next().unwrap());

    FieldNode { name, type_ident }
}

fn parse_type_ident(pair: pest::iterators::Pair<Rule>) -> TypeIdent {
    let inner = pair.into_inner().into_iter().next().unwrap();
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

fn parse_built_in_atom(pair: pest::iterators::Pair<Rule>) -> AtomType {
    match pair.as_span().as_str() {
        "str" => AtomType::Str,
        "i32" => AtomType::I32,
        "u32" => AtomType::U32,
        "u8" => AtomType::U8,
        "f64" => AtomType::F64,
        _ => unreachable!(dbg!(pair)),
    }
}

fn parse_list_type(pair: pest::iterators::Pair<Rule>) -> TypeIdent {
    let inner = pair.into_inner().into_iter().next().unwrap();

    TypeIdent::List(Box::new(parse_type_ident(inner)))
}

fn parse_option_type(pair: pest::iterators::Pair<Rule>) -> TypeIdent {
    let inner = pair.into_inner().into_iter().next().unwrap();

    TypeIdent::Option(Box::new(parse_type_ident(inner)))
}

fn parse_map_type(pair: pest::iterators::Pair<Rule>) -> TypeIdent {
    let mut inners = pair.into_inner().into_iter();
    let key_type = inners.next().unwrap();
    let value_type = inners.next().unwrap();

    TypeIdent::Map(
        Box::new(parse_type_ident(key_type)),
        Box::new(parse_type_ident(value_type)),
    )
}

fn parse_tuple_def(pair: pest::iterators::Pair<Rule>) -> TupleDef {
    TupleDef(
        pair.into_inner()
            .into_iter()
            .map(parse_type_ident)
            .collect(),
    )
}

fn parse_spec_item(pair: pest::iterators::Pair<Rule>) -> SpecItem {
    match pair.as_rule() {
        Rule::struct_definition => SpecItem::StructDef(parse_struct_definition(pair)),
        Rule::enum_def => SpecItem::EnumDef(parse_enum_definition(pair)),
        _ => unreachable!(dbg!(pair)),
    }
}

// "parse spec"
pub fn parse(input: &str) -> Spec {
    let humbled = HumbleParser::parse(Rule::doc, input)
        .unwrap()
        .next()
        .unwrap();

    Spec(
        humbled
            .into_inner()
            .into_iter()
            .map(parse_spec_item)
            .collect(),
    )
}
