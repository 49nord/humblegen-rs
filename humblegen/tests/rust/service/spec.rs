#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = "A wandering monster"]
pub struct Monster {
    #[doc = "Monster ID."]
    pub id: i32,
    #[doc = "The monster's name"]
    pub name: String,
    #[doc = "Max hitpoints."]
    pub hp: i32,
    #[doc = ""]
    pub foo: String,
    #[doc = ""]
    pub bar: String,
}
#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = ""]
pub struct MonsterData {
    #[doc = "The monster's name"]
    pub name: String,
    #[doc = "Max hitpoints."]
    pub hp: i32,
}
#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = ""]
pub struct MonsterData2 {
    #[doc = ""]
    pub foo: String,
    #[doc = ""]
    pub bar: String,
}
#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = "patch of a monster"]
pub struct MonsterPatch {
    #[doc = ""]
    pub name: Option<String>,
    #[doc = ""]
    pub hp: Option<i32>,
    #[doc = ""]
    pub foo: Option<String>,
}
#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = ""]
pub struct MonsterData3 {
    #[doc = ""]
    pub bar: String,
}
#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = "Errors returned by the monster service."]
pub enum MonsterError {
    #[doc = ""]
    TooWeak,
    #[doc = ""]
    TooStrong {
        #[doc = ""]
        max_strength: i32,
    },
}
#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = ""]
pub struct PoliceCar {}
#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = ""]
pub enum PoliceError {}
#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = ""]
pub struct MonsterQuery {
    #[doc = ""]
    pub name: Option<String>,
    #[doc = ""]
    pub max_age: Option<i32>,
}
#[allow(unused_imports)]
use ::humblegen_rt::deser_helpers::{
    deser_param, deser_post_data, deser_query_primitive, deser_query_serde_urlencoded,
};
#[allow(unused_imports)]
pub use ::humblegen_rt::handler::{self, HandlerResponse as Response, ServiceError};
#[allow(unused_imports)]
use ::humblegen_rt::hyper;
#[allow(unused_imports)]
use ::humblegen_rt::regexset_map::RegexSetMap;
#[allow(unused_imports)]
use ::humblegen_rt::server::{self, handler_response_to_hyper_response, Route, Service};
#[allow(unused_imports)]
use ::humblegen_rt::service_protocol::ErrorResponse;
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
    Godzilla(Arc<dyn Godzilla<Context = Context> + Send + Sync>),
    Movies(Arc<dyn Movies<Context = Context> + Send + Sync>),
}
impl<Context: Default + Sized + Send + Sync + 'static> Handler<Context> {
    fn into_routes(self) -> Vec<Route> {
        match self {
            Handler::Godzilla(h) => routes_Godzilla(h),
            Handler::Movies(h) => routes_Movies(h),
        }
    }
}
impl<Context: Default + Sized + Send + Sync + 'static> std::fmt::Debug for Handler<Context> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Handler::Godzilla(_) => write!(formatter, "{}", "Godzilla")?,
            Handler::Movies(_) => write!(formatter, "{}", "Movies")?,
        }
        Ok(())
    }
}
#[doc = "service Godzilla provides services related to monsters."]
#[doc = "```\n#[humblegen_rt::async_trait(Sync)]\npub trait Godzilla {\n    type Context: Default + Sized + Send + Sync;\n    async fn intercept_handler_pre(\n        &self,\n        _req: &hyper::Request<hyper::Body>,\n    ) -> Result<Self::Context, ServiceError> {\n        Ok(Self::Context::default())\n    }\n    async fn get_foo(&self, ctx: Self::Context) -> Response<u32>;\n    async fn get_monsters_id(\n        &self,\n        ctx: Self::Context,\n        id: i32,\n    ) -> Response<Result<Monster, MonsterError>>;\n    async fn get_monsters(\n        &self,\n        ctx: Self::Context,\n        query: Option<MonsterQuery>,\n    ) -> Response<Vec<Monster>>;\n    async fn get_monsters_2(\n        &self,\n        ctx: Self::Context,\n        query: Option<String>,\n    ) -> Response<Vec<Monster>>;\n    async fn get_monsters_3(\n        &self,\n        ctx: Self::Context,\n        query: Option<i32>,\n    ) -> Response<Vec<Monster>>;\n    async fn get_monsters_4(&self, ctx: Self::Context) -> Response<Vec<Monster>>;\n    async fn post_monsters(\n        &self,\n        ctx: Self::Context,\n        post_body: MonsterData,\n    ) -> Response<Result<Monster, MonsterError>>;\n    async fn put_monsters_id(\n        &self,\n        ctx: Self::Context,\n        post_body: Monster,\n        id: String,\n    ) -> Response<Result<(), MonsterError>>;\n    async fn patch_monsters_id(\n        &self,\n        ctx: Self::Context,\n        post_body: MonsterPatch,\n        id: String,\n    ) -> Response<Result<(), MonsterError>>;\n    async fn delete_monster_id(\n        &self,\n        ctx: Self::Context,\n        id: String,\n    ) -> Response<Result<(), MonsterError>>;\n    async fn get_version(&self, ctx: Self::Context) -> Response<String>;\n    async fn get_tokio_police_locations(\n        &self,\n        ctx: Self::Context,\n    ) -> Response<Result<Vec<PoliceCar>, PoliceError>>;\n}\n\n```"]
#[humblegen_rt::async_trait(Sync)]
pub trait Godzilla {
    type Context: Default + Sized + Send + Sync;
    async fn intercept_handler_pre(
        &self,
        _req: &hyper::Request<hyper::Body>,
    ) -> Result<Self::Context, ServiceError> {
        Ok(Self::Context::default())
    }
    #[doc = "```\nasync fn get_foo(&self, ctx: Self::Context) -> Response<u32> {}\n\n```"]
    #[doc = "Get foo."]
    async fn get_foo(&self, ctx: Self::Context) -> Response<u32>;
    #[doc = "```\nasync fn get_monsters_id(\n    &self,\n    ctx: Self::Context,\n    id: i32,\n) -> Response<Result<Monster, MonsterError>> {\n}\n\n```"]
    #[doc = "Get monster by id"]
    async fn get_monsters_id(
        &self,
        ctx: Self::Context,
        id: i32,
    ) -> Response<Result<Monster, MonsterError>>;
    #[doc = "```\nasync fn get_monsters(\n    &self,\n    ctx: Self::Context,\n    query: Option<MonsterQuery>,\n) -> Response<Vec<Monster>> {\n}\n\n```"]
    #[doc = "Get monster by posting a query"]
    async fn get_monsters(
        &self,
        ctx: Self::Context,
        query: Option<MonsterQuery>,
    ) -> Response<Vec<Monster>>;
    #[doc = "```\nasync fn get_monsters_2(\n    &self,\n    ctx: Self::Context,\n    query: Option<String>,\n) -> Response<Vec<Monster>> {\n}\n\n```"]
    #[doc = ""]
    async fn get_monsters_2(
        &self,
        ctx: Self::Context,
        query: Option<String>,
    ) -> Response<Vec<Monster>>;
    #[doc = "```\nasync fn get_monsters_3(&self, ctx: Self::Context, query: Option<i32>) -> Response<Vec<Monster>> {}\n\n```"]
    #[doc = ""]
    async fn get_monsters_3(
        &self,
        ctx: Self::Context,
        query: Option<i32>,
    ) -> Response<Vec<Monster>>;
    #[doc = "```\nasync fn get_monsters_4(&self, ctx: Self::Context) -> Response<Vec<Monster>> {}\n\n```"]
    #[doc = ""]
    async fn get_monsters_4(&self, ctx: Self::Context) -> Response<Vec<Monster>>;
    #[doc = "```\nasync fn post_monsters(\n    &self,\n    ctx: Self::Context,\n    post_body: MonsterData,\n) -> Response<Result<Monster, MonsterError>> {\n}\n\n```"]
    #[doc = "Create a new monster."]
    async fn post_monsters(
        &self,
        ctx: Self::Context,
        post_body: MonsterData,
    ) -> Response<Result<Monster, MonsterError>>;
    #[doc = "```\nasync fn put_monsters_id(\n    &self,\n    ctx: Self::Context,\n    post_body: Monster,\n    id: String,\n) -> Response<Result<(), MonsterError>> {\n}\n\n```"]
    #[doc = "Overwrite a monster."]
    async fn put_monsters_id(
        &self,
        ctx: Self::Context,
        post_body: Monster,
        id: String,
    ) -> Response<Result<(), MonsterError>>;
    #[doc = "```\nasync fn patch_monsters_id(\n    &self,\n    ctx: Self::Context,\n    post_body: MonsterPatch,\n    id: String,\n) -> Response<Result<(), MonsterError>> {\n}\n\n```"]
    #[doc = "Patch a monster."]
    async fn patch_monsters_id(
        &self,
        ctx: Self::Context,
        post_body: MonsterPatch,
        id: String,
    ) -> Response<Result<(), MonsterError>>;
    #[doc = "```\nasync fn delete_monster_id(\n    &self,\n    ctx: Self::Context,\n    id: String,\n) -> Response<Result<(), MonsterError>> {\n}\n\n```"]
    #[doc = "Delete a monster"]
    async fn delete_monster_id(
        &self,
        ctx: Self::Context,
        id: String,
    ) -> Response<Result<(), MonsterError>>;
    #[doc = "```\nasync fn get_version(&self, ctx: Self::Context) -> Response<String> {}\n\n```"]
    #[doc = ""]
    async fn get_version(&self, ctx: Self::Context) -> Response<String>;
    #[doc = "```\nasync fn get_tokio_police_locations(\n    &self,\n    ctx: Self::Context,\n) -> Response<Result<Vec<PoliceCar>, PoliceError>> {\n}\n\n```"]
    #[doc = ""]
    async fn get_tokio_police_locations(
        &self,
        ctx: Self::Context,
    ) -> Response<Result<Vec<PoliceCar>, PoliceError>>;
}
#[allow(unused_variables)]
#[allow(unused_mut)]
#[allow(non_snake_case)]
#[allow(clippy::trivial_regex)]
#[allow(clippy::single_char_pattern)]
fn routes_Godzilla<Context: Default + Sized + Send + Sync + 'static>(
    handler: Arc<dyn Godzilla<Context = Context> + Send + Sync>,
) -> Vec<Route> {
    vec![
        {
            let handler = Arc::clone(&handler);
            Route {
                method: ::humblegen_rt::hyper::Method::GET,
                regex: ::humblegen_rt::regex::Regex::new("^/foo$").unwrap(),
                dispatcher: Box::new(
                    move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                          captures| {
                        let handler = Arc::clone(&handler);
                        Box::pin(async move {
                            use ::humblegen_rt::service_protocol::ToErrorResponse;
                            let ctx = handler
                                .intercept_handler_pre(&req)
                                .await
                                .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                .map_err(|e| e.to_error_response())?;
                            Ok(handler_response_to_hyper_response(
                                handler.get_foo(ctx).await,
                            ))
                        })
                    },
                ),
            }
        },
        {
            let handler = Arc::clone(&handler);
            Route {
                method: ::humblegen_rt::hyper::Method::GET,
                regex: ::humblegen_rt::regex::Regex::new("^/monsters/(?P<id>[^/]+)$").unwrap(),
                dispatcher: Box::new(
                    move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                          captures| {
                        let handler = Arc::clone(&handler);
                        let id: Result<i32, ErrorResponse> = deser_param("id", &captures["id"]);
                        Box::pin(async move {
                            let id = id?;
                            use ::humblegen_rt::service_protocol::ToErrorResponse;
                            let ctx = handler
                                .intercept_handler_pre(&req)
                                .await
                                .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                .map_err(|e| e.to_error_response())?;
                            Ok(handler_response_to_hyper_response(
                                handler.get_monsters_id(ctx, id).await,
                            ))
                        })
                    },
                ),
            }
        },
        {
            let handler = Arc::clone(&handler);
            Route {
                method: ::humblegen_rt::hyper::Method::GET,
                regex: ::humblegen_rt::regex::Regex::new("^/monsters$").unwrap(),
                dispatcher: Box::new(
                    move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                          captures| {
                        let handler = Arc::clone(&handler);
                        Box::pin(async move {
                            let query: Option<MonsterQuery> = match req.uri().query() {
                                None => None,
                                Some(q) => Some(deser_query_serde_urlencoded(q)?),
                            };
                            use ::humblegen_rt::service_protocol::ToErrorResponse;
                            let ctx = handler
                                .intercept_handler_pre(&req)
                                .await
                                .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                .map_err(|e| e.to_error_response())?;
                            Ok(handler_response_to_hyper_response(
                                handler.get_monsters(ctx, query).await,
                            ))
                        })
                    },
                ),
            }
        },
        {
            let handler = Arc::clone(&handler);
            Route {
                method: ::humblegen_rt::hyper::Method::GET,
                regex: ::humblegen_rt::regex::Regex::new("^/monsters2$").unwrap(),
                dispatcher: Box::new(
                    move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                          captures| {
                        let handler = Arc::clone(&handler);
                        Box::pin(async move {
                            let query: Option<String> = match req.uri().query() {
                                None => None,
                                Some(q) => Some(deser_query_primitive(q)?),
                            };
                            use ::humblegen_rt::service_protocol::ToErrorResponse;
                            let ctx = handler
                                .intercept_handler_pre(&req)
                                .await
                                .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                .map_err(|e| e.to_error_response())?;
                            Ok(handler_response_to_hyper_response(
                                handler.get_monsters_2(ctx, query).await,
                            ))
                        })
                    },
                ),
            }
        },
        {
            let handler = Arc::clone(&handler);
            Route {
                method: ::humblegen_rt::hyper::Method::GET,
                regex: ::humblegen_rt::regex::Regex::new("^/monsters3$").unwrap(),
                dispatcher: Box::new(
                    move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                          captures| {
                        let handler = Arc::clone(&handler);
                        Box::pin(async move {
                            let query: Option<i32> = match req.uri().query() {
                                None => None,
                                Some(q) => Some(deser_query_primitive(q)?),
                            };
                            use ::humblegen_rt::service_protocol::ToErrorResponse;
                            let ctx = handler
                                .intercept_handler_pre(&req)
                                .await
                                .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                .map_err(|e| e.to_error_response())?;
                            Ok(handler_response_to_hyper_response(
                                handler.get_monsters_3(ctx, query).await,
                            ))
                        })
                    },
                ),
            }
        },
        {
            let handler = Arc::clone(&handler);
            Route {
                method: ::humblegen_rt::hyper::Method::GET,
                regex: ::humblegen_rt::regex::Regex::new("^/monsters4$").unwrap(),
                dispatcher: Box::new(
                    move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                          captures| {
                        let handler = Arc::clone(&handler);
                        Box::pin(async move {
                            use ::humblegen_rt::service_protocol::ToErrorResponse;
                            let ctx = handler
                                .intercept_handler_pre(&req)
                                .await
                                .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                .map_err(|e| e.to_error_response())?;
                            Ok(handler_response_to_hyper_response(
                                handler.get_monsters_4(ctx).await,
                            ))
                        })
                    },
                ),
            }
        },
        {
            let handler = Arc::clone(&handler);
            Route {
                method: ::humblegen_rt::hyper::Method::POST,
                regex: ::humblegen_rt::regex::Regex::new("^/monsters$").unwrap(),
                dispatcher: Box::new(
                    move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                          captures| {
                        let handler = Arc::clone(&handler);
                        Box::pin(async move {
                            let post_body: MonsterData = deser_post_data(req.body_mut()).await?;
                            use ::humblegen_rt::service_protocol::ToErrorResponse;
                            let ctx = handler
                                .intercept_handler_pre(&req)
                                .await
                                .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                .map_err(|e| e.to_error_response())?;
                            Ok(handler_response_to_hyper_response(
                                handler.post_monsters(ctx, post_body).await,
                            ))
                        })
                    },
                ),
            }
        },
        {
            let handler = Arc::clone(&handler);
            Route {
                method: ::humblegen_rt::hyper::Method::PUT,
                regex: ::humblegen_rt::regex::Regex::new("^/monsters/(?P<id>[^/]+)$").unwrap(),
                dispatcher: Box::new(
                    move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                          captures| {
                        let handler = Arc::clone(&handler);
                        let id: Result<String, ErrorResponse> = deser_param("id", &captures["id"]);
                        Box::pin(async move {
                            let id = id?;
                            let post_body: Monster = deser_post_data(req.body_mut()).await?;
                            use ::humblegen_rt::service_protocol::ToErrorResponse;
                            let ctx = handler
                                .intercept_handler_pre(&req)
                                .await
                                .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                .map_err(|e| e.to_error_response())?;
                            Ok(handler_response_to_hyper_response(
                                handler.put_monsters_id(ctx, post_body, id).await,
                            ))
                        })
                    },
                ),
            }
        },
        {
            let handler = Arc::clone(&handler);
            Route {
                method: ::humblegen_rt::hyper::Method::PATCH,
                regex: ::humblegen_rt::regex::Regex::new("^/monsters/(?P<id>[^/]+)$").unwrap(),
                dispatcher: Box::new(
                    move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                          captures| {
                        let handler = Arc::clone(&handler);
                        let id: Result<String, ErrorResponse> = deser_param("id", &captures["id"]);
                        Box::pin(async move {
                            let id = id?;
                            let post_body: MonsterPatch = deser_post_data(req.body_mut()).await?;
                            use ::humblegen_rt::service_protocol::ToErrorResponse;
                            let ctx = handler
                                .intercept_handler_pre(&req)
                                .await
                                .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                .map_err(|e| e.to_error_response())?;
                            Ok(handler_response_to_hyper_response(
                                handler.patch_monsters_id(ctx, post_body, id).await,
                            ))
                        })
                    },
                ),
            }
        },
        {
            let handler = Arc::clone(&handler);
            Route {
                method: ::humblegen_rt::hyper::Method::DELETE,
                regex: ::humblegen_rt::regex::Regex::new("^/monster/(?P<id>[^/]+)$").unwrap(),
                dispatcher: Box::new(
                    move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                          captures| {
                        let handler = Arc::clone(&handler);
                        let id: Result<String, ErrorResponse> = deser_param("id", &captures["id"]);
                        Box::pin(async move {
                            let id = id?;
                            use ::humblegen_rt::service_protocol::ToErrorResponse;
                            let ctx = handler
                                .intercept_handler_pre(&req)
                                .await
                                .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                .map_err(|e| e.to_error_response())?;
                            Ok(handler_response_to_hyper_response(
                                handler.delete_monster_id(ctx, id).await,
                            ))
                        })
                    },
                ),
            }
        },
        {
            let handler = Arc::clone(&handler);
            Route {
                method: ::humblegen_rt::hyper::Method::GET,
                regex: ::humblegen_rt::regex::Regex::new("^/version$").unwrap(),
                dispatcher: Box::new(
                    move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                          captures| {
                        let handler = Arc::clone(&handler);
                        Box::pin(async move {
                            use ::humblegen_rt::service_protocol::ToErrorResponse;
                            let ctx = handler
                                .intercept_handler_pre(&req)
                                .await
                                .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                .map_err(|e| e.to_error_response())?;
                            Ok(handler_response_to_hyper_response(
                                handler.get_version(ctx).await,
                            ))
                        })
                    },
                ),
            }
        },
        {
            let handler = Arc::clone(&handler);
            Route {
                method: ::humblegen_rt::hyper::Method::GET,
                regex: ::humblegen_rt::regex::Regex::new("^/tokio-police-locations$").unwrap(),
                dispatcher: Box::new(
                    move |mut req: ::humblegen_rt::hyper::Request<::humblegen_rt::hyper::Body>,
                          captures| {
                        let handler = Arc::clone(&handler);
                        Box::pin(async move {
                            use ::humblegen_rt::service_protocol::ToErrorResponse;
                            let ctx = handler
                                .intercept_handler_pre(&req)
                                .await
                                .map_err(::humblegen_rt::service_protocol::ServiceError::from)
                                .map_err(|e| e.to_error_response())?;
                            Ok(handler_response_to_hyper_response(
                                handler.get_tokio_police_locations(ctx).await,
                            ))
                        })
                    },
                ),
            }
        },
    ]
}
#[doc = ""]
#[doc = "```\n#[humblegen_rt::async_trait(Sync)]\npub trait Movies {\n    type Context: Default + Sized + Send + Sync;\n    async fn intercept_handler_pre(\n        &self,\n        _req: &hyper::Request<hyper::Body>,\n    ) -> Result<Self::Context, ServiceError> {\n        Ok(Self::Context::default())\n    }\n}\n\n```"]
#[humblegen_rt::async_trait(Sync)]
pub trait Movies {
    type Context: Default + Sized + Send + Sync;
    async fn intercept_handler_pre(
        &self,
        _req: &hyper::Request<hyper::Body>,
    ) -> Result<Self::Context, ServiceError> {
        Ok(Self::Context::default())
    }
}
#[allow(unused_variables)]
#[allow(unused_mut)]
#[allow(non_snake_case)]
#[allow(clippy::trivial_regex)]
#[allow(clippy::single_char_pattern)]
fn routes_Movies<Context: Default + Sized + Send + Sync + 'static>(
    handler: Arc<dyn Movies<Context = Context> + Send + Sync>,
) -> Vec<Route> {
    vec![]
}
