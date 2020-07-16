#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = ""]
pub struct Post {
    #[doc = ""]
    pub content: String,
}
#[allow(unused_imports)]
use ::humblegen_rt::deser_helpers::{
    deser_param, deser_post_data, deser_query_primitive, deser_query_serde_urlencoded,
};
#[allow(unused_imports)]
pub use ::humblegen_rt::handler::{self, HandlerResponse as Response, ServiceError};
#[allow(unused_imports)]
use ::humblegen_rt::regexset_map::RegexSetMap;
#[allow(unused_imports)]
use ::humblegen_rt::server::{self, handler_response_to_hyper_response, Route, Service};
#[allow(unused_imports)]
use ::humblegen_rt::service_protocol::ErrorResponse;
use ::humblegen_rt::tracing_futures::Instrument;
#[allow(unused_imports)]
use ::humblegen_rt::{hyper, tracing};
#[allow(unused_imports)]
use ::std::sync::Arc;
use std::net::SocketAddr;
#[doc = r" Builds an HTTP server that exposes services implemented by handler trait objects."]
#[derive(Debug)]
pub struct Builder {
    services: Vec<Service>,
}
impl Builder {
    pub fn new() -> Self {
        Self { services: vec![] }
    }
    #[doc = r" Mounts `handler` at URL path prefix `root`."]
    #[doc = r" This means that a `handler` implementing humble service"]
    #[doc = r" ```"]
    #[doc = r" service S {"]
    #[doc = r"     GET /bar -> i32,"]
    #[doc = r"     GET /baz -> str,"]
    #[doc = r" }"]
    #[doc = r" ```"]
    #[doc = r#" and `root="/api"` will expose"#]
    #[doc = r" * handler method `fn bar() -> i32` at `/api/bar` and"]
    #[doc = r" * handler method `fn baz() -> String` at `/api/baz`"]
    pub fn add<Context: Default + Sized + Send + Sync>(
        mut self,
        root: &str,
        handler: Handler<Context>,
    ) -> Self {
        if !root.starts_with('/') {
            panic!("root must start with \"/\"")
        } else if root.ends_with('/') {
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
    #[doc = r" Starts an HTTP server bound to address `addr` and serves incoming requests using"]
    #[doc = r" the previously `add`ed handlers."]
    pub async fn listen_and_run_forever(
        self,
        addr: &SocketAddr,
    ) -> humblegen_rt::anyhow::Result<()> {
        use humblegen_rt::anyhow::Context;
        let services = RegexSetMap::new(self.services).context("invalid service configuration")?;
        server::listen_and_run_forever(services, addr).await
    }
}
#[doc = r" Wrapper enum with one variant for each service defined in the humble spec."]
#[doc = r" Used to pass instantiated handler trait objects to `Builder::add`."]
#[allow(dead_code)]
pub enum Handler<Context: Default + Sized + Send + Sync + 'static> {
    BlogApi(Arc<dyn BlogApi<Context = Context> + Send + Sync>),
}
impl<Context: Default + Sized + Send + Sync + 'static> Handler<Context> {
    fn into_routes(self) -> Vec<Route> {
        match self {
            Handler::BlogApi(h) => routes_BlogApi(h),
        }
    }
}
impl<Context: Default + Sized + Send + Sync + 'static> std::fmt::Debug for Handler<Context> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Handler::BlogApi(_) => write!(formatter, "{}", "BlogApi")?,
        }
        Ok(())
    }
}
#[doc = ""]
#[doc = "```\n#[humblegen_rt::async_trait(Sync)]\npub trait BlogApi {\n    type Context: Default + Sized + Send + Sync;\n    async fn intercept_handler_pre(\n        &self,\n        _req: &hyper::Request<hyper::Body>,\n    ) -> Result<Self::Context, ServiceError> {\n        Ok(Self::Context::default())\n    }\n    async fn post_user_posts(\n        &self,\n        ctx: Self::Context,\n        post_body: Post,\n        user: String,\n    ) -> Response<Post>;\n}\n\n```"]
#[humblegen_rt::async_trait(Sync)]
pub trait BlogApi {
    type Context: Default + Sized + Send + Sync;
    async fn intercept_handler_pre(
        &self,
        _req: &hyper::Request<hyper::Body>,
    ) -> Result<Self::Context, ServiceError> {
        Ok(Self::Context::default())
    }
    #[doc = "```\nasync fn post_user_posts(\n    &self,\n    ctx: Self::Context,\n    post_body: Post,\n    user: String,\n) -> Response<Post> {\n}\n\n```"]
    #[doc = "Must send header `Authorization: Custom AUTHZ_TOKEN`\notherwise authorization error."]
    async fn post_user_posts(
        &self,
        ctx: Self::Context,
        post_body: Post,
        user: String,
    ) -> Response<Post>;
}
#[allow(unused_variables)]
#[allow(unused_mut)]
#[allow(non_snake_case)]
#[allow(clippy::trivial_regex)]
#[allow(clippy::single_char_pattern)]
fn routes_BlogApi<Context: Default + Sized + Send + Sync + 'static>(
    handler: Arc<dyn BlogApi<Context = Context> + Send + Sync>,
) -> Vec<Route> {
    vec![{
        let handler = Arc::clone(&handler);
        Route {
            method: ::humblegen_rt::hyper::Method::POST,
            regex: ::humblegen_rt::regex::Regex::new("^/(?P<user>[^/]+)/posts$").unwrap(),
            dispatcher: Box::new(
                move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                      captures| {
                    let handler = Arc::clone(&handler);
                    let user: Result<String, ErrorResponse> =
                        deser_param("user", &captures["user"]);
                    Box::pin(async move {
                        use ::humblegen_rt::service_protocol::ToErrorResponse;
                        let ctx = {
                            let span = tracing::error_span!("interceptor");
                            handler . intercept_handler_pre ( & req ) . instrument ( span ) . await . map_err ( :: humblegen_rt :: service_protocol :: ServiceError :: from ) . map_err ( | e | { tracing :: debug ! ( service_error = ? format ! ( "{:?}" , e ) , "interceptor rejected request" ) ; e } ) . map_err ( | e | e . to_error_response ( ) ) ?
                        };
                        let user = user?;
                        let post_body: Post = deser_post_data(req.body_mut()).await?;
                        drop(req);
                        {
                            let span = tracing::error_span!("handler");
                            Ok(handler_response_to_hyper_response(
                                handler
                                    .post_user_posts(ctx, post_body, user)
                                    .instrument(span)
                                    .await,
                            ))
                        }
                    })
                },
            ),
        }
    }]
}
