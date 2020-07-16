//! `GEN` Generic parts of the humblegen HTTP service server implementation, based on [`hyper`](https://hyper.rs).

use crate::handler::HandlerResponse;
use crate::regexset_map;
use crate::regexset_map::RegexSetMap;
use crate::service_protocol::{self, RuntimeError, ToErrorResponse};
use derivative::Derivative;
use tracing_futures::Instrument;

use anyhow::Context;
use hyper::Body;
use hyper::Request;
use hyper::Response;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use rand::Rng;

/// Serve `services` via HTTP, binding to the given `addr`.
/// Invokes `handle_request`.
///
/// Invoked by generated code.
pub async fn listen_and_run_forever(
    services: RegexSetMap<Request<Body>, Service>,
    addr: &SocketAddr,
) -> anyhow::Result<()> {
    // Note: this is the standard (noisy) dance for handling hyper requests.
    let services = Arc::new(services);
    let server = hyper::Server::bind(addr).serve(hyper::service::make_service_fn(
        move |_sock: &hyper::server::conn::AddrStream| {
            let services = Arc::clone(&services);
            async move {
                Ok::<_, Infallible>(hyper::service::service_fn(
                    move |req: hyper::Request<hyper::Body>| {
                        let services = Arc::clone(&services);
                        async move {
                            let resp = handle_request(services, req).await;
                            Ok::<Response<hyper::Body>, Infallible>(resp)
                        }
                    },
                ))
            }
        },
    ));

    server.await.context("server error")?;
    Ok(())
}

const REQUEST_ID_HEADER_NAME: &'static str = "Request-ID";

/// The routine that maps an incoming hyper request to a service in `services`,
/// and invokes the service's dispatcher.
pub async fn handle_request(
    services: Arc<RegexSetMap<Request<Body>, Service>>,
    req: Request<Body>,
) -> Response<Body> {
    let request_id: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(30)
        .collect();
    let span = tracing::error_span!("handle_request", request_id = ?request_id);
    handle_request_impl(services, req, request_id)
        .instrument(span)
        .await
}

pub async fn handle_request_impl(
    services: Arc<RegexSetMap<Request<Body>, Service>>,
    req: Request<Body>,
    request_id: String,
) -> Response<Body> {
    let path = req.uri().path().to_string(); // necessary because we need to move req into dispatcher, but also need to move captures into dispatcher

    let mut response = match services.get(&path, &req) {
        regexset_map::GetResult::None => RuntimeError::NoServiceMounted
            .to_error_response()
            .to_hyper_response(),
        regexset_map::GetResult::Ambiguous => RuntimeError::ServiceMountsAmbiguous
            .to_error_response()
            .to_hyper_response(),
        regexset_map::GetResult::One(service) => {
            tracing::debug!(service_regex = (service.0).0.as_str(), "service matched");
            let tuple = &service.0;
            let service_regex_captures = tuple.0.captures(&path).unwrap();
            let service = service_regex_captures["root"].to_string();
            let suffix = &service_regex_captures["suffix"];
            match tuple.1.get(&suffix, &req) {
                regexset_map::GetResult::None => RuntimeError::NoRouteMountedInService { service }
                    .to_error_response()
                    .to_hyper_response(),
                regexset_map::GetResult::Ambiguous => {
                    RuntimeError::RouteMountsAmbiguous { service }
                        .to_error_response()
                        .to_hyper_response()
                }
                regexset_map::GetResult::One(route) => {
                    tracing::debug!(route_regex = route.regex.as_str(), "route matched");
                    let captures = route.regex.captures(suffix).unwrap();
                    let dispatcher = &route.dispatcher;

                    let dispatcher_result = {
                        let dispatcher_span = tracing::error_span!("invoke_dispatcher");
                        dispatcher(req, captures).instrument(dispatcher_span).await
                    };
                    match dispatcher_result {
                        Ok(r) => {
                            tracing::debug!("handler returned Ok");
                            r
                        }
                        Err(e) => {
                            tracing::error!(err = ?e, "handler returned error");
                            e.to_hyper_response()
                        }
                    }
                }
            }
        }
    };

    response.headers_mut().insert(
        REQUEST_ID_HEADER_NAME,
        hyper::header::HeaderValue::from_str(&request_id)
            .expect("request ID is expected to be valid header value"),
    );

    response.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );

    tracing::debug!(http_status = ?response.status(), "finished request");

    response
}

/// A service is a collection of Routes that share a common `prefix`.
///
/// Instantiated by generated code.
#[derive(Debug)]
pub struct Service(pub (regex::Regex, RegexSetMap<Request<Body>, Route>));

// helper type that avoids bloating the type signature of `DispatcherClosure`.
type BoxSyncFuture<Output> =
    std::pin::Pin<Box<dyn Send + Sync + std::future::Future<Output = Output>>>;

/// Closure with an internal reference to the handler trait object that implements a humblegen service trait.
/// It decodes request into the arguments required to invoke the trait function and then does the call.
type DispatcherClosure = dyn Fn(
        Request<Body>,
        regex::Captures,
    ) -> BoxSyncFuture<Result<Response<Body>, service_protocol::ErrorResponse>>
    + Send
    + Sync;

/// A route associates an HTTP method + URL path regex with a `DispatcherClosure`.
/// It implements `regexset_map::Entry` and only makes sense within a `Service`.
///
/// Instantiated by generated code.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Route {
    pub method: hyper::Method,
    pub regex: regex::Regex,
    #[derivative(Debug = "ignore")]
    pub dispatcher: Box<DispatcherClosure>,
}

impl<'a> regexset_map::Entry<Request<Body>> for Route {
    fn regex(&self) -> &regex::Regex {
        &self.regex
    }
    fn matches_input(&self, req: &Request<Body>) -> bool {
        self.method == req.method()
    }
}

impl<'a> regexset_map::Entry<Request<Body>> for Service {
    fn regex(&self) -> &regex::Regex {
        let pair = &self.0;
        &pair.0
    }
    fn matches_input(&self, _req: &Request<Body>) -> bool {
        true
    }
}

/// Conversion of a `HandlerResponse` to a hyper response.
/// Invoked from generated code within a `DispatcherClosure`.
pub fn handler_response_to_hyper_response<T>(handler_response: HandlerResponse<T>) -> Response<Body>
where
    T: serde::Serialize,
{
    match handler_response {
        Ok(x) => serde_json::to_string(&x)
            .map(|s| Response::new(Body::from(s)))
            .unwrap_or_else(|e| {
                tracing::error!(error = ?e, "cannot serialize handler response");
                RuntimeError::SerializeHandlerResponse(e.to_string())
                    .to_error_response()
                    .to_hyper_response()
            }),
        Err(e) => {
            tracing::error!(error = ?e, "handler returned error");
            service_protocol::ServiceError::from(e)
                .to_error_response()
                .to_hyper_response()
        }
    }
}
