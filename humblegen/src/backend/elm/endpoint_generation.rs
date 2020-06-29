use super::{generate_doc_comment, to_atom, IndentWriter, type_generation, decoder_generation, encoder_generation};
use crate::{LibError, ast};
use inflector::Inflector;
use std::io::Write;

pub(crate) fn generate(service: &ast::ServiceDef, file :&mut IndentWriter) -> Result<(), LibError> {
    
    file.kill_indent();

    write!(file.start_line()?, "{}", generate_doc_comment(&service.doc_comment))?;

    file.empty_lines(2)?;

    for endpoint in &service.endpoints {
        // Note: we currently generate a single flat function for each endpoint. This is what
        // OpenApi does. A worthfile, alternative api would generate an enum of endpoints
        // enum Endpoints = GetPet | PostMonster | etc first that is consumed by a generic
        // executeRequest function.
        write!(file.start_line()?, "{}", generate_doc_comment(&endpoint.doc_comment))?;

        // TODO: should narrow type of query parameter. According to spec query has to be a user defined struct
        let query_ty_name = endpoint.route.query().as_ref().map(|query| {
            if let ast::TypeIdent::UserDefined(query_ty_name) = query {
                query_ty_name
            } else {
                panic!("query MUST be a user defined struct");
            }
        });

        {
            // built_signature
            let mut line_type_signature = Vec::new();
            let mut line_arguments = Vec::new();

            let endpoint_name = synthesize_endpoint_name(&endpoint.route);
            write!(line_type_signature, "{} : String -> Maybe String -> (Result Http.Error {} -> msg)",
                endpoint_name,
                to_atom(type_generation::generate_type_ident(endpoint.route.return_type(), "Ty.")))?;
            write!(line_arguments, "{} baseUrl session msg ", endpoint_name)?;

            for (idx, component) in endpoint.route.components().iter().enumerate() {
                if let ast::ServiceRouteComponent::Variable(arg) = component {
                    write!(line_type_signature, " -> {}", to_atom(type_generation::generate_type_ident(&arg.type_ident, "Ty.")))?;
                    write!(line_arguments, " component{idx}_{name}", idx=idx, name=arg.name)?;
                }
            }

            // TODO: body
            
            if let Some(ident) = query_ty_name {
                
                write!(line_type_signature, " -> {}", ident)?;
                write!(line_arguments, " query")?;
            }

            // return type
            write!(line_type_signature, " -> Cmd msg")?;

            file.start_line()?.write_all(&line_type_signature)?;
            file.start_line()?.write_all(&line_arguments)?;
        }

        write!(file.handle(), " =")?;

        file.increase_indent();
        write!(file.start_line()?,"Http.request")?;
        file.increase_indent();
        write!(file.start_line()?,"{{ method = \"{}\"", endpoint.route.http_method_as_str())?;
        write!(file.start_line()?, "{}", ", headers = S.maybeWithAuthorization session")?;
        write!(file.start_line()?, "{}", ", url = Url.Builder.crossOrigin baseUrl")?;

        {
            // generate_endpoint_url
            file.increase_indent();
            for (idx, component) in endpoint.route.components().iter().enumerate() {
                let is_first = idx == 0;
                let delimiter = if is_first { "[" } else { ","};
                
                match component {
                    ast::ServiceRouteComponent::Literal(literal) => {
                        // TODO: is this escape sufficient and correct for elm?
                        write!(file.start_line()?, "{delimiter} \"{}\"", literal.escape_default(), delimiter=delimiter)?;
                    }

                    ast::ServiceRouteComponent::Variable(arg) => {
                        write!(file.start_line()?, "{delimiter} component{idx}_{name} |> {encoder} |> E.encode 0",
                            // TODO: this puts string components into quotes which is most likely not what we want
                            // Apart from json and query encoders, we need a third encoder: component encoder
                            encoder=to_atom(encoder_generation::generate_type_json_encoder(&arg.type_ident)),
                            name=arg.name,
                            idx=idx,
                            delimiter=delimiter
                        )?;
                    }
                }
            }

            write!(file.start_line()?, "]")?;
            
            if let Some(ident) = query_ty_name {
                // Note: cannot collide with other encoders since they are imported with AE prefix
                // TODO: but can colide with variable names in the route components
                write!(file.start_line()?, "(AE.{} query)", encoder_generation::query_struct_encoder_name(&ident))?;
            } else {
                write!(file.start_line()?, "[]")?;
            }
            
            file.decrease_indent();
        }

        write!(file.start_line()?,", body = Http.emptyBody -- TODO")?;
        write!(file.start_line()?,", expect = Http.expectJson (S.mapServerResponse msg) {}",
            to_atom(decoder_generation::generate_type_decoder(&endpoint.route.return_type(), "AD.")))?;
        write!(file.start_line()?,", timeout = Nothing")?;
        write!(file.start_line()?,", tracker = Nothing")?;
        write!(file.start_line()?,"}}")?;
        file.kill_indent();

        // match &endpoint.route {
        //     ast::ServiceRoute::Get { components, query, ret } => {

        //     }
        //     ast::ServiceRoute::Post { components, query, body, ret } => {
                
        //     }
        //     ast::ServiceRoute::Delete { components, query, ret } => {
                
        //     }
        //     ast::ServiceRoute::Put  { components, query, body, ret } => {
                
        //     }
        //     ast::ServiceRoute::Patch { components, query, body, ret }  => {
                
        //     }       
        // }
    }

    file.empty_lines(2)?;

    Ok(())
}

fn synthesize_endpoint_name(route :&ast::ServiceRoute) -> String {
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
        ast::ServiceRoute::Get { .. } => { "get" }
        ast::ServiceRoute::Post { .. } => { "create" }
        ast::ServiceRoute::Delete { .. } => { "delete" }
        ast::ServiceRoute::Put  { .. } => { "replace" }
        ast::ServiceRoute::Patch { .. }  => { "modify" }       
    };

    format!("{}{}", verb, action)
}