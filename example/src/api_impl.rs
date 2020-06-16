use core::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::AtomicU32;

use super::protocol;
use humblegen_rt::async_trait;

/// Implements the `Godzilla` service (trait `protocol::Godzilla`)
#[derive(Default)]
pub struct MonsterApiImpl {
    ctr: AtomicU32,
}

#[async_trait(Sync)]
impl protocol::Godzilla for MonsterApiImpl {
    type Context = ();

    async fn get_foo(&self, _ctx: Self::Context) -> protocol::Response<u32> {
        // simulate authorization failure for every other request
        let v = self.ctr.fetch_add(1, SeqCst);
        if v % 2 == 0 {
            Ok(v)
        } else {
            Err(protocol::ServiceError::Authorization)
        }
    }

    async fn get_monsters_id(
        &self,
        _ctx: Self::Context,
        _id: i32,
    ) -> protocol::Response<Result<protocol::Monster, protocol::MonsterError>> {
        // demonstrate how service-specific errors are handled
        Ok(Err(protocol::MonsterError::TooWeak))
    }

    async fn get_monsters(
        &self,
        _ctx: Self::Context,
        query: Option<protocol::MonsterQuery>,
    ) -> protocol::Response<Vec<protocol::Monster>> {
        // the query-part of the URL is deserialized into argument `query` if specified by the user
        dbg!(query);
        // panics do _not_ crash the entire server:
        // hyper calls close(1) on the client connection descriptor and moves on
        unimplemented!()
    }

    async fn post_monsters(
        &self,
        _ctx: Self::Context,
        post_body: protocol::MonsterData,
    ) -> protocol::Response<Result<protocol::Monster, protocol::MonsterError>> {
        // the POST body is made available as argument `post_body`
        dbg!(post_body);
        // we return an Ok(Err(MonsterError)) here:
        // - the request could be processed correctly (no auth{n,z} problems, database worked, etc)
        // - but there was a domain error: the posted monster is too weak
        Ok(Err(protocol::MonsterError::TooWeak))
    }

    async fn get_monsters_2(
        &self,
        _ctx: Self::Context,
        query: Option<String>,
    ) -> protocol::Response<Vec<protocol::Monster>> {
        dbg!(query);
        unimplemented!()
    }

    async fn get_monsters_3(
        &self,
        _ctx: Self::Context,
        query: Option<i32>,
    ) -> protocol::Response<Vec<protocol::Monster>> {
        // non-struct queries are deserialized
        dbg!(query);
        unimplemented!()
    }

    async fn get_monsters_4(
        &self,
        _ctx: Self::Context,
    ) -> protocol::Response<Vec<protocol::Monster>> {
        unimplemented!()
    }

    async fn get_version(&self, _ctx: Self::Context) -> protocol::Response<String> {
        unimplemented!()
    }

    async fn get_tokio_police_locations(
        &self,
        _ctx: Self::Context,
    ) -> protocol::Response<Result<Vec<protocol::PoliceCar>, protocol::PoliceError>> {
        unimplemented!()
    }

    async fn delete_monster_id(
        &self,
        _ctx: Self::Context,
        id: String,
    ) -> protocol::Response<Result<(), protocol::MonsterError>> {
        println!("would delete id={}", id);
        unimplemented!()
    }

    async fn put_monsters_id(
        &self,
        _ctx: Self::Context,
        monster: protocol::Monster,
        id: String,
    ) -> protocol::Response<Result<(), protocol::MonsterError>> {
        dbg!((id, monster));
        unimplemented!()
    }

    async fn patch_monsters_id(
        &self,
        _ctx: Self::Context,
        patch: protocol::MonsterPatch,
        id: String,
    ) -> protocol::Response<Result<(), protocol::MonsterError>> {
        dbg!((id, patch));
        unimplemented!()
    }
}

/// Implements the `Movies` service (trait `protocol::Movies`)
#[derive(Default)]
pub struct MoviesApiImpl {}

impl protocol::Movies for MoviesApiImpl {
    type Context = ();
}
