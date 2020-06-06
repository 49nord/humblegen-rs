//! Elm code generator.

use crate::ast;
use comrak::{markdown_to_html, ComrakOptions};
use itertools::Itertools;
use std::fmt;

use base64;

struct NavigationLevel {
    label: String,
    link: String,
    children: Vec<NavigationLevel>,
}

struct SearchResult {
    term: String,
    link: NavigationLevel,
}

#[derive(Default)]
struct Context {
    navigation: Vec<NavigationLevel>,
    searchterms: Vec<SearchResult>,
    body: String,
}

// struct CloseTag<'a> {
//     name : &'a str,
//     out: &'a mut String
// }

// impl<'a> Drop for CloseTag<'a> {
//     fn drop(&mut self) {
//         self.out.push_str(&format!("</{}>", self.name));
//     }
// }

// fn html<'a>(out : &'a mut String, name :&'a str) -> CloseTag<'a> {
//     out.push_str(&format!("<{}>", name));
//     return CloseTag { name, out }
// }

/// Wrapper struct which will emit the HTML-escaped version of the contained
/// string when passed to a format string.
pub struct Escape<'a>(pub &'a str);

impl<'a> fmt::Display for Escape<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // Because the internet is always right, turns out there's not that many
        // characters to escape: http://stackoverflow.com/questions/7381974
        let Escape(s) = *self;
        let pile_o_bits = s;
        let mut last = 0;
        for (i, ch) in s.bytes().enumerate() {
            match ch as char {
                '<' | '>' | '&' | '\'' | '"' => {
                    fmt.write_str(&pile_o_bits[last..i])?;
                    let s = match ch as char {
                        '>' => "&gt;",
                        '<' => "&lt;",
                        '&' => "&amp;",
                        '\'' => "&#39;",
                        '"' => "&quot;",
                        _ => unreachable!(),
                    };
                    fmt.write_str(s)?;
                    last = i + 1;
                }
                _ => {}
            }
        }

        if last < s.len() {
            fmt.write_str(&pile_o_bits[last..])?;
        }
        Ok(())
    }
}

impl Context {
    fn add_spec(&mut self, spec: &ast::Spec) -> &mut Self {
        let spec_html = spec
            .iter()
            .map(|item| item.service_def())
            .filter_map(|service| service)
            .map(|service| {
                format!(
                    include_str!("docs/service.html"),
                    serviceName = Escape(service.name.as_str()),
                    serviceDescription = markdown_to_html(
                        service.doc_comment.as_deref().unwrap_or(""),
                        &ComrakOptions::default()
                    ),
                    serviceEndpoints = self.endpoints_to_html(&service.endpoints),
                )
            })
            .join("\n");

        self.body.push_str(&spec_html);

        let usertype_html = format!(
            include_str!("docs/UserDefinedTypeListing.html"),
            userDefinedTypes = self.user_defined_types_to_html(&spec)
        );

        self.body.push_str(&usertype_html);

        self
    }

    fn user_defined_types_to_html(&mut self, spec: &ast::Spec) -> String {
        spec.iter()
            .filter_map(|item| match item {
                ast::SpecItem::StructDef(struct_def) => Some(format!(
                    include_str!("docs/UserDefinedType.html"),
                    kind = "structure",
                    name = Escape(&struct_def.name),
                    description = markdown_to_html(
                        struct_def.doc_comment.as_deref().unwrap_or(""),
                        &ComrakOptions::default()
                    ),
                    id = Self::link_to_user_defined_type(&struct_def.name)
                )),
                ast::SpecItem::EnumDef(enum_def) => Some(format!(
                    include_str!("docs/UserDefinedType.html"),
                    kind = "enumeration",
                    name = Escape(&enum_def.name),
                    description = markdown_to_html(
                        enum_def.doc_comment.as_deref().unwrap_or(""),
                        &ComrakOptions::default()
                    ),
                    id = Self::link_to_user_defined_type(&enum_def.name)
                )),
                _ => None,
            })
            .join("\n")
    }

    fn endpoints_to_html(&mut self, endpoints: &[ast::ServiceEndpoint]) -> String {
        endpoints
            .iter()
            .map(|endpoint| {
                format!(
                    include_str!("docs/endpoint.html"),
                    httpMethod = endpoint.route.http_method_as_str(),
                    endpointRoute = Self::components_to_html(endpoint.route.components()),
                    endpointLink = Self::components_to_link(&endpoint.route),
                    endpointDescription = markdown_to_html(
                        endpoint.doc_comment.as_deref().unwrap_or(""),
                        &ComrakOptions::default()
                    ),
                    endpointSummary = markdown_to_html(
                        &markdown_get_first_line_as_summary(
                            endpoint.doc_comment.as_deref().unwrap_or("")
                        ),
                        &ComrakOptions::default()
                    ),
                    endpointReturn = Self::type_ident_to_html(endpoint.route.return_type()),
                    endpointRouteQuery = endpoint
                        .route
                        .query()
                        .as_ref()
                        .map(|q| { format!("?{}", Self::type_ident_to_html(q)) })
                        .unwrap_or_default(),
                    //endpointProperties = "",
                )
            })
            .join("\n")
    }

    pub fn atom_to_html(t: ast::AtomType) -> &'static str {
        match t {
            ast::AtomType::Empty => "Empty",
            ast::AtomType::Str => "String",
            ast::AtomType::I32 => "Integer",
            ast::AtomType::U32 => "Unsinged Integer",
            ast::AtomType::U8 => "Unsinged Integer",
            ast::AtomType::F64 => "Float",
            ast::AtomType::Bool => "Boolean",
            ast::AtomType::DateTime => "DateTime",
            ast::AtomType::Date => "Date",
        }
    }

    pub fn type_ident_to_html(type_ident: &ast::TypeIdent) -> String {
        match type_ident {
            ast::TypeIdent::BuiltIn(atom) => Self::atom_to_html(*atom).to_string(),
            ast::TypeIdent::List(ty) => format!("List[{}]", Self::type_ident_to_html(&*ty)),
            ast::TypeIdent::Option(ty) => format!("Option[{}]", Self::type_ident_to_html(&*ty)),
            ast::TypeIdent::Result(ty1, ty2) => format!(
                "Result[{},{}]",
                Self::type_ident_to_html(&*ty1),
                Self::type_ident_to_html(&*ty2)
            ),
            ast::TypeIdent::Map(ty1, ty2) => format!(
                "Map[{},{}]",
                Self::type_ident_to_html(&*ty1),
                Self::type_ident_to_html(&*ty2)
            ),
            ast::TypeIdent::Tuple(tuple) => format!(
                "({})",
                tuple
                    .components()
                    .iter()
                    .map(|x| Self::type_ident_to_html(x))
                    .join(", ")
            ),
            ast::TypeIdent::UserDefined(name) => format!(
                r##"<a href="#{}">{}</a>"##,
                Self::link_to_user_defined_type(name),
                name
            ),
        }
    }

    pub fn link_to_user_defined_type(name: &str) -> String {
        format!("type-{}", name)
    }

    pub fn components_to_html(components: &[ast::ServiceRouteComponent]) -> String {
        components
            .iter()
            .map(|c| match c {
                ast::ServiceRouteComponent::Literal(lit) => {
                    format!("/<span>{}</span>", Escape(&lit))
                }
                ast::ServiceRouteComponent::Variable(ast::FieldDefPair { name, type_ident }) => {
                    format!(
                        "/<var>{}:{}</var>",
                        Escape(&name),
                        Escape(&Self::type_ident_to_html(&type_ident))
                    )
                }
            })
            .join("")
    }

    pub fn components_to_link(route: &ast::ServiceRoute) -> String {
        let componentStr = route
            .components()
            .iter()
            .map(|c| match c {
                ast::ServiceRouteComponent::Literal(lit) => format!("/{}", Escape(&lit)),
                ast::ServiceRouteComponent::Variable(ast::FieldDefPair { name, type_ident }) => {
                    format!(
                        "/{}:{}",
                        Escape(&name),
                        Escape(&Self::type_ident_to_html(&type_ident))
                    )
                }
            })
            .join("");

        format!("{}{}", route.http_method_as_str(), componentStr)
    }

    fn to_html(&mut self) -> String {
        vec![
            "<!doctype html>",
            r#"<meta charset="utf-8">"#,
            "<title>",
            &self.spec_name(),
            "</title>",
            r#"<meta name="viewport" content="width=device-width, initial-scale=1">"#,
            "<style>",
            include_str!("docs/main.css"),
            &inline_svg_icon("link", include_str!("docs/unicode-symbol-1f517.svg")),
            &inline_svg_icon(
                "chevron-contract",
                include_str!("docs/bootstrap-icons/chevron-contract.svg"),
            ),
            &inline_svg_icon(
                "chevron-expand",
                include_str!("docs/bootstrap-icons/chevron-expand.svg"),
            ),
            &inline_svg_icon("search", include_str!("docs/bootstrap-icons/search.svg")),
            "</style>",
            "<body>",
            include_str!("docs/page_head.html"),
            &self.body,
            "<script>",
            include_str!("docs/script.js"),
            "</script>",
        ]
        .join("\n")
    }

    fn spec_name(&self) -> String {
        String::new()
    }
}

fn inline_svg_icon(class_name: &str, svg: &str) -> String {
    format!(
        ".icon--{} {{ background-image: url(\"data:image/svg+xml;base64,{}\") }}",
        class_name,
        base64::encode(svg)
    )
}

fn markdown_get_first_line_as_summary(markdown: &str) -> String {
    let first_sentence = markdown.split("\n\n").next().unwrap_or("");
    if first_sentence.len() > 100 {
        format!("{}...", &first_sentence[0..97])
    } else {
        first_sentence.to_string()
    }
}

pub fn render(spec: &ast::Spec) -> String {
    Context::default().add_spec(spec).to_html()
}
