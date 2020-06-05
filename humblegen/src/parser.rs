//! The humble language parser.

mod embeds;

use itertools::Itertools;
use pest::Parser;
use pest_derive::Parser;

/// The humble language parser, derived from the PEST grammar file.
#[derive(Parser)]
#[grammar = "humble.pest"]
struct HumbleParser;

use crate::ast::*;

/// Parse complete spec.
pub(crate) fn parse(input: &str) -> Result<Spec, pest::error::Error<Rule>> {
    let humbled = HumbleParser::parse(Rule::doc, input)?
        .next()
        .expect("grammar requires non-empty document");

    let mut ast = Spec(humbled.into_inner().map(parse_spec_item).collect());

    // AST transformations
    embeds::resolve_embeds(&mut ast);

    Ok(ast)
}

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
    let pair = pair;
    let fields: Vec<_> = pair
        .into_inner()
        .map(|p| {
            assert_eq!(p.as_rule(), Rule::struct_field_def);
            let mut nodes = p.into_inner();
            let struct_field_def = nodes.next().unwrap();
            assert_eq!(nodes.next(), None);
            match struct_field_def.as_rule() {
                Rule::struct_field_def_node => {
                    // let mut nodes = struct_field_def.into_inner();
                    // let field_def_node = nodes.next().unwrap();
                    // assert_eq!(nodes.next(), None);
                    parse_struct_field_def_node(struct_field_def)
                }
                Rule::struct_field_def_embed => {
                    // the grammar guarantees that struct field names are snake_case
                    // and that struct type names are PascalCase
                    // => a struct type name is never a valid field name
                    // ==> for embeds, use the struct type name as field name and do the fixup in spec_resolve_embeds
                    let mut nodes = struct_field_def.into_inner();
                    let ty = nodes.next().unwrap();
                    assert_eq!(nodes.next(), None);
                    FieldNode {
                        doc_comment: None,
                        pair: FieldDefPair {
                            name: ty.as_span().as_str().to_string(),
                            type_ident: parse_type_ident(ty),
                        },
                    }
                }
                x => panic!("unexpected token {:?}", x),
            }
        })
        .collect();
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

fn parse_struct_field_def_pair(pair: pest::iterators::Pair<Rule>) -> FieldDefPair {
    let pair = pair;
    let mut nodes = pair.into_inner();
    let name = nodes.next().unwrap().as_span().as_str().to_string();
    let type_ident = parse_type_ident(nodes.next().unwrap());
    assert_eq!(nodes.next(), None);
    FieldDefPair { name, type_ident }
}

/// Parse field definitions in struct.
fn parse_struct_field_def_node(pair: pest::iterators::Pair<Rule>) -> FieldNode {
    let pair = pair;
    let mut nodes = pair.into_inner();
    let doc_comment = parse_doc_comment(&mut nodes);
    let pair = parse_struct_field_def_pair(nodes.next().unwrap());
    FieldNode { pair, doc_comment }
}

fn parse_service_definition(pair: pest::iterators::Pair<Rule>) -> ServiceDef {
    let mut nodes = pair.into_inner();
    let doc_comment = parse_doc_comment(&mut nodes);
    let name = nodes.next().unwrap().as_span().as_str().to_string();
    let endpoints = nodes
        .next()
        .unwrap()
        .into_inner()
        .map(parse_service_rule)
        .collect();
    assert_eq!(nodes.next(), None);
    ServiceDef {
        doc_comment,
        name,
        endpoints,
    }
}

fn parse_service_rule(pair: pest::iterators::Pair<Rule>) -> ServiceEndpoint {
    let mut nodes = pair.into_inner();
    let doc_comment = parse_doc_comment(&mut nodes);
    let route = parse_service_rule_def(nodes.next().unwrap());
    assert_eq!(nodes.next(), None);
    ServiceEndpoint { doc_comment, route }
}

fn parse_service_rule_def(pair: pest::iterators::Pair<Rule>) -> ServiceRoute {
    let mut nodes = pair.into_inner();
    let parser = match nodes.peek().unwrap().as_rule() {
        Rule::http_get => parse_service_rule_get,
        Rule::http_delete => parse_service_rule_delete,
        Rule::http_post => parse_service_rule_post,
        Rule::http_put => parse_service_rule_put,
        Rule::http_patch => parse_service_rule_patch,
        x => panic!("unexpected token {:?}", x),
    };
    nodes.next().unwrap(); // consume what we peeked
    let route = parser(&mut nodes);
    assert_eq!(nodes.next(), None);
    route
}

fn parse_service_rule_get(pair: &mut pest::iterators::Pairs<Rule>) -> ServiceRoute {
    ServiceRoute::Get {
        components: parse_http_route(pair.next().unwrap()),
        query: parse_http_query(pair),
        ret: parse_type_ident(pair.next().unwrap()),
    }
}

fn parse_service_rule_delete(pair: &mut pest::iterators::Pairs<Rule>) -> ServiceRoute {
    ServiceRoute::Delete {
        components: parse_http_route(pair.next().unwrap()),
        query: parse_http_query(pair),
        ret: parse_type_ident(pair.next().unwrap()),
    }
}

fn parse_service_rule_post(pair: &mut pest::iterators::Pairs<Rule>) -> ServiceRoute {
    ServiceRoute::Post {
        components: parse_http_route(pair.next().unwrap()),
        query: parse_http_query(pair),
        body: parse_type_ident(pair.next().unwrap()),
        ret: parse_type_ident(pair.next().unwrap()),
    }
}

fn parse_service_rule_put(pair: &mut pest::iterators::Pairs<Rule>) -> ServiceRoute {
    ServiceRoute::Put {
        components: parse_http_route(pair.next().unwrap()),
        query: parse_http_query(pair),
        body: parse_type_ident(pair.next().unwrap()),
        ret: parse_type_ident(pair.next().unwrap()),
    }
}

fn parse_service_rule_patch(pair: &mut pest::iterators::Pairs<Rule>) -> ServiceRoute {
    ServiceRoute::Patch {
        components: parse_http_route(pair.next().unwrap()),
        query: parse_http_query(pair),
        body: parse_type_ident(pair.next().unwrap()),
        ret: parse_type_ident(pair.next().unwrap()),
    }
}

fn parse_http_route(pair: pest::iterators::Pair<Rule>) -> Vec<ServiceRouteComponent> {
    pair.into_inner().map(parse_http_route_segment).collect()
}

fn parse_http_route_segment(pair: pest::iterators::Pair<Rule>) -> ServiceRouteComponent {
    let mut nodes = pair.into_inner();
    let comp = nodes.next().unwrap();
    match comp.as_rule() {
        Rule::kebab_case_ident => {
            ServiceRouteComponent::Literal(comp.as_span().as_str().to_string())
        }
        Rule::http_route_segment_arg => {
            let mut nodes = comp.into_inner();
            let ret =
                ServiceRouteComponent::Variable(parse_struct_field_def_pair(nodes.next().unwrap()));
            assert_eq!(nodes.next(), None);
            ret
        }
        x => panic!("unexpected token {:?}", x),
    }
}

fn parse_http_query(pairs: &mut pest::iterators::Pairs<Rule>) -> Option<TypeIdent> {
    let next_peek = pairs.peek()?;
    if next_peek.as_rule() != Rule::http_query {
        return None;
    }
    let next = pairs.next().unwrap(); // consume
    let mut tokens = next.into_inner();
    let ret = Some(parse_type_ident(tokens.next().unwrap()));
    assert_eq!(tokens.next(), None);
    ret
}

/// Parse type identifier.
fn parse_type_ident(pair: pest::iterators::Pair<Rule>) -> TypeIdent {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::built_in_atom => TypeIdent::BuiltIn(parse_built_in_atom(inner)),
        Rule::list_type => parse_list_type(inner),
        Rule::option_type => parse_option_type(inner),
        Rule::result_type => parse_result_type(inner),
        Rule::map_type => parse_map_type(inner),
        Rule::tuple_def => TypeIdent::Tuple(parse_tuple_def(inner)),
        Rule::camel_case_ident => TypeIdent::UserDefined(inner.as_span().as_str().to_string()),
        _ => unreachable!(dbg!(inner)),
    }
}

/// Parse a built-in atomic type.
fn parse_built_in_atom(pair: pest::iterators::Pair<Rule>) -> AtomType {
    match pair.as_span().as_str() {
        "()" => AtomType::Empty,
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

/// Parse a result type.
fn parse_result_type(pair: pest::iterators::Pair<Rule>) -> TypeIdent {
    let mut tokens = pair.into_inner();
    let ok = tokens.next().unwrap();
    let err = tokens.next().unwrap();
    assert_eq!(tokens.next(), None);
    TypeIdent::Result(
        Box::new(parse_type_ident(ok)),
        Box::new(parse_type_ident(err)),
    )
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
        Rule::service_definition => SpecItem::ServiceDef(parse_service_definition(pair)),
        _ => unreachable!(dbg!(pair)),
    }
}
