use super::{generate_doc_comment, IndentWriter};
use crate::{LibError, ast};
use inflector::Inflector;

pub(crate) fn generate(service: &ast::ServiceDef, file :&mut IndentWriter) -> Result<(), LibError> {
    
    file.kill_indent();

    write!(file.start_line()?, "{}", generate_doc_comment(&service.doc_comment))?;
    
    write!(file.start_line()?, "import Url.Builder")?;
    write!(file.start_line()?, "import Http")?;
    write!(file.start_line()?, "import Api.Data")?;
    write!(file.start_line()?, "import Api.Encode")?;
    write!(file.start_line()?, "import Api.Decode")?;

    file.empty_lines(2)?;

    for endpoint in &service.endpoints {
        // Note: we currently generate a single flat function for each endpoint. This is what
        // OpenApi does. A worthfile, alternative api would generate an enum of endpoints
        // enum Endpoints = GetPet | PostMonster | etc first that is consumed by a generic
        // executeRequest function.
        write!(file.start_line()?, "{}", generate_doc_comment(&endpoint.doc_comment))?;

        let endpoint_name = synthesize_endpoint_name(&endpoint.route);
        write!(file.start_line()?, "{} msg =", endpoint_name)?;

        file.increase_indent();
        write!(file.start_line()?,"Http.request")?;
        file.increase_indent();
        write!(file.start_line()?,"{{ method = \"{}\"", endpoint.route.http_method_as_str())?;
        write!(file.start_line()?,", headers = []")?;
        write!(file.start_line()?,", url = Url.Builder.crossOrigin baseUrl [] [] -- TODO")?;
        write!(file.start_line()?,", body = Http.emptyBody \"TODO\"")?;
        write!(file.start_line()?,", expect = Http.expectJson msg (decodeServerResponse decode.String) -- TODO")?;
        write!(file.start_line()?,", timeout = Nothing")?;
        write!(file.start_line()?,", tracker = Nothing")?;
        write!(file.start_line()?,"}}")?;

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