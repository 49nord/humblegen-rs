mod protocol {
    include!("spec.rs");
}
use humblegen_rt::hyper;
use protocol::*;
use std::sync::Arc;

struct S;

#[derive(Default)]
struct AuthzScope {
    user_id: String,
    posting_allowed: bool,
}

#[humblegen_rt::async_trait(Sync)]
impl BlogApi for S {
    type Context = AuthzScope;

    async fn intercept_handler_pre(
        &self,
        req: &hyper::Request<hyper::Body>,
    ) -> Result<Self::Context, ServiceError> {
        if req
            .headers()
            .get(hyper::header::AUTHORIZATION)
            .ok_or(ServiceError::Authorization)?
            .to_str()
            .map_err(|_| ServiceError::Authorization)?
            == "Custom AUTHZ_TOKEN"
        {
            // .. find user id database ...
            Ok(AuthzScope {
                user_id: "alice".to_owned(),
                posting_allowed: true,
            })
        } else {
            Err(ServiceError::Authorization)
        }
    }

    async fn post_user_posts(
        &self,
        ctx: Self::Context,
        post_body: Post,
        user: String,
    ) -> Response<Post> {
        if user != ctx.user_id {
            return Err(ServiceError::Authorization);
        }
        if !ctx.posting_allowed {
            return Err(ServiceError::Authorization);
        }
        // store user's post in the database
        println!("user {:?} posted {:?}", user, post_body);
        Ok(post_body)
    }
}

#[tokio::main]
async fn main() {
    // let listen_addr: std::net::SocketAddr = "127.0.0.1:3000".parse().unwrap();
    Builder::new().add("/api", Handler::BlogApi(Arc::new(S)));
    // .listen_and_run_forever(&listen_addr)
    // .await
    // .unwrap();
}
