mod api_impl;
mod protocol {
    include!(concat!(env!("OUT_DIR"), "/protocol.rs"));
}

use std::net::SocketAddr;
use std::sync::Arc;

use api_impl::{MonsterApiImpl, MoviesApiImpl};

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // MonsterApiImpl implements the protocol::Gozilla handler trait.
    let monster_handler = protocol::Handler::Godzilla(Arc::new(MonsterApiImpl::default()));

    // MoviesApiImpl implements the protocol::Movies handler trait.
    let movies_handler = protocol::Handler::Movies(Arc::new(MoviesApiImpl::default()));

    protocol::Builder::new()
        // mount monster_handler at endpoint /api/godzilla
        .add("/api/godzilla", monster_handler)
        // mount movies_handler at endpoint /api/movies
        .add("/api/movies", movies_handler)
        // serve HTTP on address addr
        .listen_and_run_forever(&addr)
        .await
        .unwrap();
}
