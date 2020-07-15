// TODO: Fix lints and remove this.
#![allow(clippy::write_literal)]

use super::{
    decoder_generation, encoder_generation, generate_doc_comment, to_atom, type_generation,
    IndentWriter,
};
use crate::{ast, LibError};
use inflector::Inflector;
use std::io::Write;

pub(crate) fn generate(service: &ast::ServiceDef, file: &mut IndentWriter) -> Result<(), LibError> {
    file.kill_indent();

    write!(
        file.start_line()?,
        "{}",
        generate_doc_comment(&service.doc_comment)
    )?;

    file.empty_lines(2)?;

    for endpoint in &service.endpoints {
        // Note: we currently generate a single flat function for each endpoint. This is what
        // OpenApi does. A worthfile, alternative api would generate an enum of endpoints
        // enum Endpoints = GetPet | PostMonster | etc first that is consumed by a generic
        // executeRequest function.
        write!(
            file.start_line()?,
            "{}",
            generate_doc_comment(&endpoint.doc_comment)
        )?;

        {
            let mut line_type_signature = Vec::new();
            let mut line_arguments = Vec::new();

            let endpoint_name = synthesize_endpoint_name(&endpoint.route);
            write!(line_type_signature, "{} : ", endpoint_name)?;
            write!(line_arguments, "{}", endpoint_name)?;

            for (idx, component) in endpoint.route.components().iter().enumerate() {
                if let ast::ServiceRouteComponent::Variable(arg) = component {
                    write!(
                        line_type_signature,
                        "{} -> ",
                        to_atom(type_generation::generate_type_ident(&arg.type_ident, "Ty."))
                    )?;
                    write!(
                        line_arguments,
                        " component{idx}_{name}",
                        idx = idx,
                        name = arg.name
                    )?;
                }
            }

            if let Some(body) = endpoint.route.request_body() {
                write!(
                    line_type_signature,
                    "{} -> ",
                    to_atom(type_generation::generate_type_ident(&body, "Ty."))
                )?;
                write!(line_arguments, " body")?;
            }

            // return type
            write!(
                line_type_signature,
                "Request {} {}",
                endpoint
                    .route
                    .query()
                    .as_ref()
                    .map(|q| type_generation::generate_type_ident(q, "Ty."))
                    .unwrap_or_else(|| "NoQuery".to_owned()),
                to_atom(type_generation::generate_type_ident(
                    endpoint.route.return_type(),
                    "Ty."
                ))
            )?;

            file.start_line()?.write_all(&line_type_signature)?;
            file.start_line()?.write_all(&line_arguments)?;
        }

        write!(file.handle(), " =")?;

        file.increase_indent();
        write!(file.start_line()?, "makeRequest")?;
        file.increase_indent();

        // method
        write!(
            file.start_line()?,
            "\"{}\"",
            endpoint.route.http_method_as_str()
        )?;

        // urlComponents
        {
            file.increase_indent();
            for (idx, component) in endpoint.route.components().iter().enumerate() {
                let is_first = idx == 0;
                let delimiter = if is_first { "[" } else { "," };

                match component {
                    ast::ServiceRouteComponent::Literal(literal) => {
                        // TODO: is this escape sufficient and correct for elm?
                        write!(
                            file.start_line()?,
                            "{delimiter} \"{}\"",
                            literal.escape_default(),
                            delimiter = delimiter
                        )?;
                    }

                    ast::ServiceRouteComponent::Variable(arg) => {
                        write!(
                            file.start_line()?,
                            "{delimiter} component{idx}_{name} |> {encoder}",
                            encoder =
                                to_atom(encoder_generation::generate_type_urlcomponent_encoder(
                                    &arg.type_ident,
                                    "AE."
                                )),
                            name = arg.name,
                            idx = idx,
                            delimiter = delimiter
                        )?;
                    }
                }
            }

            write!(file.start_line()?, "]")?;
        }

        // queryEncoder
        {
            if let Some(ident) = endpoint.route.query() {
                write!(
                    file.start_line()?,
                    "{}",
                    to_atom(encoder_generation::query_encoder(ident, "AE."))
                )?;
            } else {
                write!(file.start_line()?, "noQueryEncoder")?;
            }
        }

        // resolver
        write!(
            file.start_line()?,
            "(jsonResolver ({}))",
            to_atom(decoder_generation::generate_type_decoder(
                &endpoint.route.return_type(),
                "AD."
            ))
        )?;

        // |> withBody if we send a body
        if let Some(body) = endpoint.route.request_body() {
            write!(
                file.start_line()?,
                "|> withJsonBody {} body",
                to_atom(encoder_generation::generate_type_json_encoder(body, "AE."))
            )?;
        }

        file.decrease_indent();

        file.kill_indent();
    }

    file.empty_lines(2)?;

    Ok(())
}

fn synthesize_endpoint_name(route: &ast::ServiceRoute) -> String {
    // TODO: not guranteed to be collision free
    // TODO: let user specify names in humble spec file
    let mut out = vec![];

    let mut components = route.components().clone();

    while let Some(component) = components.pop() {
        match component {
            ast::ServiceRouteComponent::Literal(lit) => {
                out.push(lit.clone().to_pascal_case());
            }
            ast::ServiceRouteComponent::Variable(var) => {
                out.push(format!("By{}Of", var.name.clone().to_pascal_case()));
            }
        }
    }

    let action = out.join("");

    let verb = match &route {
        ast::ServiceRoute::Get { .. } => "get",
        ast::ServiceRoute::Post { .. } => "create",
        ast::ServiceRoute::Delete { .. } => "delete",
        ast::ServiceRoute::Put { .. } => "replace",
        ast::ServiceRoute::Patch { .. } => "modify",
    };

    format!("{}{}", verb, action)
}
