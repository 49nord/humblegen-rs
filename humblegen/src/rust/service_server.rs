//! Rendering of a [`hyper`](https://hyper.rs)-based server that exposes humblespec `service`s.
//!
//! The entrypoint to this module is the `render_services` function.
//! It renders:
//!
//! - a `pub struct Builder` that users of the generated code use to instantiate an HTTP server,
//! - a `pub trait $ServiceName` handler trait that users of the generated use to implement,
//!   the functionality that the server exposes at the endpoints defined in the humblespec and
//! - a `pub enum Handler` enum with variants for each humblespec service.
//!
//! In order to mount a handler `h` implementing a handler trait for humblespec service `$ServiceName`,
//! users of generated code pass the following to `Builder::add`:
//! ```text
//! Handler::$ServiceName(Arc::new(h))
//! ```
//! See generated example code's docs for details.
//!
//! # Implementation Notes
//!
//! - In general, follow the entrypoint `render_services` to understand how this module is put together.
//!   The functions in this module are ordered top-down, i.e., big-picture first, details last.
//!
//! - First, we lower all AST service definitions into our own intermediate representation.
//!     - The 'lowered representations' barely deserve their name: they are mostly collections of identifiers,
//!       names, or generated code snippets that are used in multiple places in the generated code and thus
//!       should be defined in one central place.
//! - Then, we render the generated code, using those intermediate representations.
//!

use crate::ast;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::fmt_opt_string;
use super::render_type_ident;

/// Lowered representation of an `ast::ServiceDef`.
struct Service {
    trait_name: proc_macro2::Ident,
    trait_comment: String,
    routes_factory_name: proc_macro2::Ident,
    service_routes: Vec<ServiceRoute>,
}

/// Lowered representation of an `ast::ServiceRoute`.
struct ServiceRoute {
    doc_comment: TokenStream,
    traitfn_ident: proc_macro2::Ident,
    hyper_method: TokenStream,
    components: Vec<ServiceRouteComponent>,
    query_type: Option<TokenStream>,
    query_deser_fn: TokenStream,
    post_body_type: Option<TokenStream>,
    ret_type: TokenStream,
}

/// Lowered representation of an `ast::ServiceRouteComponent`.
enum ServiceRouteComponent {
    Literal {
        spec: String,
    },
    Param {
        spec_arg_name: String,
        rust_var_ident: proc_macro2::Ident,
        rust_var_type: TokenStream,
        url_regex_str: String,
    },
}

/// Entrypoint for rendering *all* services of a humblespec.
pub fn render_services<'a, I: Iterator<Item = &'a ast::ServiceDef>>(
    all_services: I,
) -> TokenStream {
    let all_services = lower_all_services(all_services);

    if all_services.is_empty() {
        return quote! {};
    }

    let mut out = TokenStream::new();

    // render imports and server builder
    out.extend(quote! {
        #[allow(unused_imports)]
        use ::humblegen_rt::deser_helpers::{
            deser_post_data, deser_query_primitive, deser_query_serde_urlencoded, deser_param,
        };
        #[allow(unused_imports)]
        use ::humblegen_rt::service_protocol::ErrorResponse;
        #[allow(unused_imports)]
        pub use ::humblegen_rt::handler::{self, HandlerResponse as Response, ServiceError};
        #[allow(unused_imports)]
        use ::humblegen_rt::regexset_map::RegexSetMap;
        #[allow(unused_imports)]
        use ::humblegen_rt::server::{self, handler_response_to_hyper_response, Route, Service};
        #[allow(unused_imports)]
        use ::std::sync::Arc;
        use std::net::SocketAddr;
        #[allow(unused_imports)]
        use ::humblegen_rt::hyper;

        /// Builds an HTTP server that exposes services implemented by handler trait objects.
        #[derive(Debug)]
        pub struct Builder {
            services: Vec<Service>,
        }

        impl Builder {
            pub fn new() -> Self {
                Self { services: vec![] }
            }

            /// Mounts `handler` at URL path prefix `root`.
            /// This means that a `handler` implementing humble service
            /// ```
            /// service S {
            ///     GET /bar -> i32,
            ///     GET /baz -> str,
            /// }
            /// ```
            /// and `root="/api"` will expose
            /// * handler method `fn bar() -> i32` at `/api/bar` and
            /// * handler method `fn baz() -> String` at `/api/baz`
            pub fn add<Context: Default + Sized + Send + Sync>(mut self, root: &str, handler: Handler<Context>) -> Self {
                if !root.starts_with('/') {
                    panic!("root must start with \"/\"")
                } else  if root.ends_with('/') {
                    panic!("root must not end with \"/\"")
                }

                let routes: Vec<Route> = handler.into_routes();
                let routes = RegexSetMap::new(routes).unwrap();
                self.services.push(Service((
                    humblegen_rt::regex::Regex::new(&format!(r"^(?P<root>{})(?P<suffix>/.*)", root))
                        .unwrap(),
                    routes,
                )));
                self
            }

            /// Starts an HTTP server bound to address `addr` and serves incoming requests using
            /// the previously `add`ed handlers.
            pub async fn listen_and_run_forever(self, addr: &SocketAddr) -> humblegen_rt::anyhow::Result<()> {
                use humblegen_rt::anyhow::Context;
                let services = RegexSetMap::new(self.services).context("invalid service configuration")?;
                server::listen_and_run_forever(services, addr).await
            }
        }

    });

    // render the `Handler` enum
    let handler_enum_variants: Vec<_> = all_services
        .iter()
        .map(|s| {
            let Service { trait_name, .. } = s;
            quote! {
                #trait_name(Arc<dyn #trait_name<Context=Context> + Send + Sync>)
            }
        })
        .collect();
    let handler_into_routes_match_arms: Vec<_> = all_services
        .iter()
        .map(|s| {
            let Service {
                trait_name,
                routes_factory_name,
                ..
            } = s;
            quote! {
                Handler::#trait_name(h) => #routes_factory_name(h)
            }
        })
        .collect();

    let handler_debug_arms: Vec<_> = all_services
        .iter()
        .map(|s| {
            let Service { trait_name, .. } = s;
            let trait_name_str = format!("{}", trait_name);
            quote! {
                Handler::#trait_name(_) => write!(formatter, "{}", #trait_name_str)?
            }
        })
        .collect();
    out.extend(quote! {

        /// Wrapper enum with one variant for each service defined in the humble spec.
        /// Used to pass instantiated handler trait objects to `Builder::add`.
        #[allow(dead_code)]
        pub enum Handler<Context: Default + Sized + Send + Sync + 'static> {
            #(#handler_enum_variants,)*
        }

        impl<Context: Default + Sized + Send + Sync + 'static> Handler<Context> {
            fn into_routes(self) -> Vec<Route> {
                match self {
                    #(#handler_into_routes_match_arms,)*
                }
            }
        }

        impl<Context: Default + Sized + Send + Sync + 'static> std::fmt::Debug for Handler<Context> {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#handler_debug_arms,)*
                }
                Ok(())
            }
        }

    });

    // render the service definitions
    out.extend(all_services.iter().map(render_service).flatten());

    out
}

/// renders a single service's Rust structs:
///
/// - its handler trait definition
/// - its routes factory function (called by Handler::into_routes)
fn render_service(service: &Service) -> TokenStream {
    let service_routes = &service.service_routes;
    let trait_comment = &service.trait_comment;

    let (trait_fns_with_comment, trait_fns_without_comment): (Vec<_>, Vec<_>) = service_routes
        .iter()
        .map(|r| {
            let ServiceRoute {
                traitfn_ident,
                post_body_type,
                query_type,
                components,
                ret_type,
                doc_comment,
                ..
            } = r;
            let mut param_list = vec![];
            param_list.push(quote! {&self});
            param_list.push(quote! {ctx: Self::Context});
            param_list.extend(post_body_type.iter().map(|t| quote! { post_body: #t }));
            param_list.extend(query_type.iter().map(|t| quote! { query: Option<#t> }));
            param_list.extend(components.iter().filter_map(|c| match c {
                ServiceRouteComponent::Literal { .. } => None,
                ServiceRouteComponent::Param {
                    rust_var_ident,
                    rust_var_type,
                    ..
                } => Some(quote! { #rust_var_ident : #rust_var_type }),
            }));
            let param_list = quote! { #(#param_list),* };

            let decl_without_comment = quote! {
                async fn #traitfn_ident (#param_list) -> Response<#ret_type>
            };
            let decl_as_doc_comment =
                // render with a trailing `{}` so that rustfmt 1.4.12 doesn't crash with
                // thread 'main' panicked at 'internal error: entered unreachable code', src/tools/rustfmt/src/visitor.rs:372:18
                render_as_rustdoc_comment_try_rustfmt(&quote! { #decl_without_comment {} });
            let decl_with_comment = quote! {
                #[doc = #decl_as_doc_comment ]
                #doc_comment
                #decl_without_comment
            };
            (decl_with_comment, decl_without_comment)
        })
        .unzip();
    let trait_name = &service.trait_name;
    let trait_def_interceptor_fn = quote! {
        type Context: Default + Sized + Send + Sync;
        async fn intercept_handler_pre(&self,
            _req: &hyper::Request<hyper::Body>,
        ) -> Result<Self::Context, ServiceError> {
            Ok(Self::Context::default())
        }
    };
    let trait_def_as_doc_comment = {
        let d = quote! {
            #[humblegen_rt::async_trait(Sync)]
            pub trait #trait_name {
                #trait_def_interceptor_fn
                #(#trait_fns_without_comment ;)*
            }
        };
        render_as_rustdoc_comment_try_rustfmt(&d)
    };
    let trait_def = quote! {
        #[doc = #trait_comment]
        #[doc = #trait_def_as_doc_comment ]
        #[humblegen_rt::async_trait(Sync)]
        pub trait #trait_name {
            #trait_def_interceptor_fn
            #(#trait_fns_with_comment ;)*
        }
    };

    let routes = service_routes.iter().map(|r| {
        let ServiceRoute {
            traitfn_ident,
            hyper_method,
            ..
        } = r;

        let regex_str = r
            .components
            .iter()
            .map(|c| match c {
                ServiceRouteComponent::Literal { spec } => format!("/{}", spec),
                ServiceRouteComponent::Param {
                    spec_arg_name,
                    url_regex_str,
                    ..
                } => format!("/(?P<{}>{})", spec_arg_name, url_regex_str,),
            })
            .collect::<Vec<_>>()
            .join("");
        let regex_str = format!("^{}$", regex_str);

        // post body
        let post_body_var = r.post_body_type.iter().map(|_| {
                quote! { post_body }
        }).collect::<Vec<_>>();
        let post_body_def = r.post_body_type.as_ref().map(|pbt| quote!{
            let post_body: #pbt =
            deser_post_data(req.body_mut()).await?;
        });

        // query
        let query_var = r.query_type.iter().map(|_| {
                quote!{ query }
        }).collect::<Vec<_>>();
        let query_deser_fn = &r.query_deser_fn;
        let query_def = r.query_type.as_ref().map(|qt| quote!{
            let query: Option<#qt> = match req.uri().query() {
                None => None,
                Some(q) => Some(#query_deser_fn(q)?),
            };
        });

        // route params
        let (route_param_vars, route_param_parse_stmts): (Vec<TokenStream>, Vec<TokenStream>) = r.components.iter().filter_map(|c| match c {
            ServiceRouteComponent::Literal { .. } => None,
            ServiceRouteComponent::Param {
                spec_arg_name,
                rust_var_ident,
                rust_var_type,
                ..
            } => Some((
                quote! { #rust_var_ident },
                quote! { let #rust_var_ident: Result<#rust_var_type, ErrorResponse> = deser_param( #spec_arg_name,  &captures[ #spec_arg_name ]); },
            )),
        }).unzip();

        let mut arg_list = Vec::new();
        arg_list.extend(&post_body_var);
        arg_list.extend(&query_var);
        arg_list.extend(&route_param_vars);


        let route_param_parse_stmts = route_param_parse_stmts.into_iter();
        let route_param_vars2 = route_param_vars.iter();
        let route_param_vars = route_param_vars.iter();
        let arg_list = arg_list.into_iter();
        quote! {
            {
                let handler = Arc::clone(&handler);
                Route{
                    method: #hyper_method,
                    regex: ::humblegen_rt::regex::Regex::new(#regex_str).unwrap(),
                    dispatcher: Box::new(
                        move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                        captures| {
                            let handler = Arc::clone(&handler);
                            // We cannot move the regex captures into the async closure, thus do the parsing
                            // of route params outside of the closure and move the parsing results into it.
                            // Inside the closure, `?` the results and return the param deserialization error.
                            #(#route_param_parse_stmts);*
                            Box::pin(async move {
                                #(let #route_param_vars = #route_param_vars2?;)*
                                #query_def
                                #post_body_def
                                // Invoke the interceptor
                                use ::humblegen_rt::service_protocol::ToErrorResponse;
                                let ctx = handler.intercept_handler_pre(&req).await
                                    .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                    .map_err(|e| e.to_error_response())?;
                                // Invoke handler if interceptor doesn't return a ServiceError
                                Ok(handler_response_to_hyper_response(handler.#traitfn_ident( ctx, #(#arg_list),* ).await))
                            })
                        }
                    ),
                }
            }
        }
    });

    let routes_factory_name = &service.routes_factory_name;
    quote! {
        #trait_def

        #[allow(unused_variables)]
        #[allow(unused_mut)]
        #[allow(non_snake_case)]
        #[allow(clippy::trivial_regex)]
        #[allow(clippy::single_char_pattern)]
        fn #routes_factory_name<Context: Default + Sized + Send + Sync + 'static>(handler: Arc<dyn #trait_name<Context=Context> + Send + Sync>) -> Vec<Route> {
            vec![#(#routes),*]
        }

    }
}

/// lower the `ast::ServiceDefs` into `struct Service`
fn lower_all_services<'a, I: Iterator<Item = &'a ast::ServiceDef>>(
    all_services: I,
) -> Vec<Service> {
    all_services
        .map(|sdef| Service {
            trait_name: format_ident!("{}", sdef.name),
            trait_comment: fmt_opt_string(&sdef.doc_comment).to_string(),
            routes_factory_name: format_ident!("routes_{}", sdef.name),
            service_routes: sdef
                .endpoints
                .iter()
                .map(|e| lower_service_route(&e))
                .collect(),
        })
        .collect()
}

/// Helper function for lowering an `ast::ServiceEndpoint` into a `ServiceRoute`.
fn lower_service_route(endpoint: &ast::ServiceEndpoint) -> ServiceRoute {
    let components = endpoint
        .route
        .components()
        .iter()
        .map(|c| match c {
            ast::ServiceRouteComponent::Literal(spec) => {
                ServiceRouteComponent::Literal { spec: spec.clone() }
            }
            ast::ServiceRouteComponent::Variable(ast::FieldDefPair { name, type_ident }) => {
                let rust_var_ident = format_ident!("{}", name);
                let rust_var_type = render_type_ident(type_ident);
                let url_regex_str = r"[^/]+".to_owned();
                ServiceRouteComponent::Param {
                    spec_arg_name: name.clone(),
                    url_regex_str,
                    rust_var_ident,
                    rust_var_type,
                }
            }
        })
        .collect();

    let post_body_type = match &endpoint.route {
        ast::ServiceRoute::Get { .. } => None,
        ast::ServiceRoute::Delete { .. } => None,
        ast::ServiceRoute::Post { body, .. } => Some(render_type_ident(body)),
        ast::ServiceRoute::Put { body, .. } => Some(render_type_ident(body)),
        ast::ServiceRoute::Patch { body, .. } => Some(render_type_ident(body)),
    };

    let ret_type = render_type_ident(endpoint.route.return_type());

    let (query_type, query_deser_fn) = endpoint
        .route
        .query()
        .as_ref()
        .map(|qt| {
            let deser_fn = match qt {
                ast::TypeIdent::UserDefined(_) => quote! { deser_query_serde_urlencoded },
                _ => quote! { deser_query_primitive },
            };
            (Some(render_type_ident(qt)), deser_fn)
        })
        .unwrap_or((None, quote! {}));

    let traitfn_name_stem = &endpoint
        .route
        .components()
        .iter()
        .map(|c| match c {
            ast::ServiceRouteComponent::Literal(l) => l.clone(),
            ast::ServiceRouteComponent::Variable(ast::FieldDefPair { name, .. }) => name.clone(),
        })
        .collect::<Vec<_>>()
        .join("_");

    let (traitfn_name_prefix, hyper_method) = match &endpoint.route {
        ast::ServiceRoute::Get { .. } => ("get", quote!(::humblegen_rt::hyper::Method::GET)),
        ast::ServiceRoute::Delete { .. } => {
            ("delete", quote!(::humblegen_rt::hyper::Method::DELETE))
        }
        ast::ServiceRoute::Post { .. } => ("post", quote!(::humblegen_rt::hyper::Method::POST)),
        ast::ServiceRoute::Put { .. } => ("put", quote!(::humblegen_rt::hyper::Method::PUT)),
        ast::ServiceRoute::Patch { .. } => ("patch", quote!(::humblegen_rt::hyper::Method::PATCH)),
    };
    let traitfn_ident = format_ident!(
        "{}_{}",
        traitfn_name_prefix,
        inflector::cases::snakecase::to_snake_case(&traitfn_name_stem)
    );

    let doc_comment = {
        let doc_comment = fmt_opt_string(&endpoint.doc_comment);
        quote! { #[doc = #doc_comment] }
    };

    ServiceRoute {
        doc_comment,
        traitfn_ident,
        hyper_method,
        components,
        query_type,
        query_deser_fn,
        post_body_type,
        ret_type,
    }
}

fn render_as_rustdoc_comment_try_rustfmt(s: &TokenStream) -> String {
    format!(
        "```\n{}\n```",
        super::rustfmt::try_rustfmt_2018_token_stream(s)
    )
}
