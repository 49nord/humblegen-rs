//! Generates API documentation for a humble specification file

// Reading this file, you should be aware that we use `format!(include_str!(...), ...)`
// as a simple HTML template engine. Since `format!` does not support loops,
// listings are generated using `...map(|thing| format!(include_str!(...), ...)).join("")`.
use crate::{ast, LibError};

use anyhow::Result;
use comrak::markdown_to_html;
use itertools::Itertools;

use std::io::Write;
use std::{fmt, fs::File, path::Path};

use ast::Spec;

#[derive(Default)]
struct Context {
    body: String,
}

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
                        &basic_options()
                    ),
                    serviceEndpoints = self.endpoints_to_html(&service.endpoints),
                )
            })
            .join("\n");

        self.body.push_str(&spec_html);

        let usertype_html = format!(
            include_str!("docs/user_defined_type_listing.html"),
            userDefinedTypes = self.user_defined_types_to_html(&spec)
        );

        self.body.push_str(&usertype_html);

        self
    }

    fn user_defined_types_to_html(&mut self, spec: &ast::Spec) -> String {
        spec.iter()
            .filter_map(|item| match item {
                ast::SpecItem::StructDef(struct_def) => Some(format!(
                    include_str!("docs/user_defined_type.html"),
                    kind = "structure",
                    name = Escape(&struct_def.name),
                    description = markdown_to_html(
                        struct_def.doc_comment.as_deref().unwrap_or(""),
                        &basic_options()
                    ),
                    codeSamples = Self::struct_definition_to_html(struct_def),
                    id = Self::link_to_user_defined_type(&struct_def.name)
                )),
                ast::SpecItem::EnumDef(enum_def) => Some(format!(
                    include_str!("docs/user_defined_type.html"),
                    kind = "enumeration",
                    name = Escape(&enum_def.name),
                    description = markdown_to_html(
                        enum_def.doc_comment.as_deref().unwrap_or(""),
                        &basic_options()
                    ),
                    codeSamples = Self::enum_definition_to_html(enum_def),
                    id = Self::link_to_user_defined_type(&enum_def.name)
                )),
                _ => None,
            })
            .join("\n")
    }

    fn tabbed_navigation_to_html(tabs: Vec<(&str, String)>) -> String {
        format!(
            include_str!("docs/tabs.html"),
            nav = tabs
                .iter()
                .map(|(lang, _)| format!(include_str!("docs/tabs_tab.html"), label = Escape(lang)))
                .join(""),
            bodies = tabs
                .iter()
                .map(|(lang, body)| format!(
                    include_str!("docs/tabs_body.html"),
                    label = Escape(lang),
                    body = body
                ))
                .join(""),
        )
    }

    fn generate_struct_property_table(struct_def: &ast::StructDef) -> String {
        format!(
            include_str!("docs/typedef_table_struct.html"),
            tableBody = struct_def
                .fields
                .iter()
                .map(|field_node| {
                    format!(
                        include_str!("docs/typedef_table_struct_field.html"),
                        fieldName = Escape(&field_node.pair.name),
                        fieldType = Self::type_ident_to_html(&field_node.pair.type_ident),
                        fieldComment = markdown_to_html(
                            &field_node.doc_comment.as_deref().unwrap_or(""),
                            &basic_options()
                        )
                    )
                })
                .join("")
        )
    }

    fn struct_definition_to_html(struct_def: &ast::StructDef) -> String {
        // TODO: make a common interface/trait for all languages?! why does this not exist in the first place
        let tabs = vec![(
            "Language Agnostic",
            Self::generate_struct_property_table(struct_def),
        )];

        Self::tabbed_navigation_to_html(tabs)
    }

    fn generate_enum_variant_table(struct_def: &ast::EnumDef) -> String {
        format!(
            include_str!("docs/typedef_table_enum.html"),
            tableBody = struct_def
                .variants
                .iter()
                .map(|variant| {
                    match &variant.variant_type {
                        ast::VariantType::Simple => format!(
                            include_str!("docs/typedef_table_enum_field.html"),
                            variantNestingDepth = 0,
                            variantNestingParent = "",
                            variantName = Escape(&variant.name),
                            variantValue = "<i>empty</i>",
                            variantComment = markdown_to_html(
                                &variant.doc_comment.as_deref().unwrap_or(""),
                                &basic_options()
                            )
                        ),
                        ast::VariantType::Newtype(ty) => format!(
                            include_str!("docs/typedef_table_enum_field.html"),
                            variantNestingDepth = 0,
                            variantNestingParent = "",
                            variantName = Escape(&variant.name),
                            variantValue = Self::type_ident_to_html(&ty),
                            variantComment = markdown_to_html(
                                &variant.doc_comment.as_deref().unwrap_or(""),
                                &basic_options()
                            )
                        ),

                        ast::VariantType::Tuple(tuple) => format!(
                            include_str!("docs/typedef_table_enum_field.html"),
                            variantNestingDepth = 0,
                            variantNestingParent = "",
                            variantName = Escape(&variant.name),
                            variantValue = Self::tuple_def_to_html(tuple),
                            variantComment = markdown_to_html(
                                &variant.doc_comment.as_deref().unwrap_or(""),
                                &basic_options()
                            )
                        ),
                        ast::VariantType::Struct(fields) => {
                            let mut rows = vec![format!(
                                include_str!("docs/typedef_table_enum_field.html"),
                                variantNestingDepth = 0,
                                variantNestingParent = "",
                                variantName = Escape(&variant.name),
                                variantValue = "<i>anonymous structure</i>",
                                variantComment = markdown_to_html(
                                    &variant.doc_comment.as_deref().unwrap_or(""),
                                    &basic_options()
                                )
                            )];

                            for field in fields.iter() {
                                rows.push(format!(
                                    include_str!("docs/typedef_table_enum_field.html"),
                                    variantNestingDepth = 1,
                                    variantNestingParent = struct_def.name,
                                    variantName = Escape(&field.pair.name),
                                    variantValue = Self::type_ident_to_html(&field.pair.type_ident),
                                    variantComment = markdown_to_html(
                                        &field.doc_comment.as_deref().unwrap_or(""),
                                        &basic_options(),
                                    ),
                                ));
                            }
                            rows.join("")
                        }
                    }
                })
                .join("")
        )
    }

    fn enum_definition_to_html(enum_def: &ast::EnumDef) -> String {
        // TODO: make a common interface/trait for all languages?! why does this not exist in the first place
        let tabs = vec![(
            "Language Agnostic",
            Self::generate_enum_variant_table(enum_def),
        )];

        Self::tabbed_navigation_to_html(tabs)
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
                        &basic_options()
                    ),
                    endpointSummary = markdown_to_html(
                        &markdown_get_first_line_as_summary(
                            endpoint.doc_comment.as_deref().unwrap_or("")
                        ),
                        &basic_options()
                    ),
                    endpointReturn = Self::type_ident_to_html(endpoint.route.return_type()),
                    endpointRouteQuery = endpoint
                        .route
                        .query()
                        .as_ref()
                        .map(|q| { format!("?{}", Self::type_ident_to_html(q)) })
                        .unwrap_or_default(),
                    endpointProperties = Self::properties_to_html(&endpoint.route),
                )
            })
            .join("\n")
    }

    pub fn atom_to_html(t: ast::AtomType) -> &'static str {
        match t {
            ast::AtomType::Empty => "empty",
            ast::AtomType::Str => "string",
            ast::AtomType::I32 => "int",
            ast::AtomType::U32 => "uint",
            ast::AtomType::U8 => "uint",
            ast::AtomType::F64 => "float",
            ast::AtomType::Bool => "bool",
            ast::AtomType::DateTime => "datetime",
            ast::AtomType::Date => "date",
            ast::AtomType::Uuid => "uuid",
            ast::AtomType::Bytes => "bytes",
        }
    }

    pub fn tuple_def_to_html(tuple: &ast::TupleDef) -> String {
        format!(
            "({})",
            tuple
                .elements()
                .iter()
                .map(Self::type_ident_to_html)
                .join(", ")
        )
    }

    pub fn type_ident_to_html(type_ident: &ast::TypeIdent) -> String {
        match type_ident {
            ast::TypeIdent::BuiltIn(atom) => Self::atom_to_html(*atom).to_string(),
            ast::TypeIdent::List(ty) => format!("list[{}]", Self::type_ident_to_html(&*ty)),
            ast::TypeIdent::Option(ty) => format!("option[{}]", Self::type_ident_to_html(&*ty)),
            ast::TypeIdent::Result(ty1, ty2) => format!(
                "result[{},{}]",
                Self::type_ident_to_html(&*ty1),
                Self::type_ident_to_html(&*ty2)
            ),
            ast::TypeIdent::Map(ty1, ty2) => format!(
                "map[{},{}]",
                Self::type_ident_to_html(&*ty1),
                Self::type_ident_to_html(&*ty2)
            ),
            ast::TypeIdent::Tuple(tuple) => Self::tuple_def_to_html(tuple),
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
                        "/<var><span class=\"var-bracket\">{{</span><span class=\"var-name\">{}</span><span class=\"var-ty-name-sep\">:</span><span class=\"var-ty\">{}</span><span class=\"var-bracket\">}}</span></var>",
                        Escape(&name),
                        Escape(&Self::type_ident_to_html(&type_ident))
                    )
                }
            })
            .join("")
    }

    pub fn components_to_link(route: &ast::ServiceRoute) -> String {
        let component_str = route
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

        format!("{}{}", route.http_method_as_str(), component_str)
    }

    pub fn properties_to_html(route: &ast::ServiceRoute) -> String {
        match route.request_body() {
            Some(type_ident) => format!(
                include_str!("docs/endpoint-properties.html"),
                endpointBody = Self::type_ident_to_html(type_ident),
            ),
            None => "".to_owned(),
        }
    }

    // FIXME: Consider renaming this
    #[allow(clippy::wrong_self_convention)]
    fn to_html(&mut self) -> String {
        vec![
            "<!doctype html>",
            r#"<meta charset="utf-8">"#,
            "<title>",
            &self.spec_name(),
            "</title>",
            r#"<meta name="viewport" content="width=device-width, initial-scale=1">"#,
            include_str!("docs/external_head.html"),
            "<style>",
            //include_str!("docs/prism.css"),
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
            include_str!("docs/external_body.html"),
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

#[derive(Default)]
pub struct Generator {}

impl crate::CodeGenerator for Generator {
    fn generate(&self, spec: &Spec, output: &Path) -> Result<(), LibError> {
        let docs = Context::default().add_spec(spec).to_html();

        // TODO: support folder as output path
        let mut outfile = File::create(&output).map_err(LibError::IoError)?;
        outfile
            .write_all(docs.as_bytes())
            .map_err(LibError::IoError)?;
        Ok(())
    }
}

/// Get the basic formatting options for writing markdown as HTML.
pub fn basic_options() -> comrak::ComrakOptions {
    // all options are written here to give an overview which are available.
    let mut options = comrak::ComrakOptions::default();

    options.extension.strikethrough = true;
    options.extension.tagfilter = false;
    options.extension.table = true;
    options.extension.autolink = false;
    options.extension.tasklist = true;
    options.extension.superscript = true;
    options.extension.header_ids = None;
    options.extension.footnotes = false;
    options.extension.description_lists = true;

    options.parse.smart = false;
    options.parse.default_info_string = None;

    options.render.hardbreaks = false;
    options.render.github_pre_lang = false;
    options.render.width = 0; // magic number to disable wrap column
    options.render.unsafe_ = false;
    options.render.escape = false;

    options
}
