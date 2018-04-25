extern crate bytes;
extern crate futures;
extern crate futures_fs;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;

pub mod body;
pub mod response;

use std::io;
use std::panic::AssertUnwindSafe;

use gotham::handler::HandlerFuture;
use gotham::middleware::{Middleware, NewMiddleware};
use gotham::state::State;

use futures_fs::FsPool;

pub struct FsPoolMiddleware {
    pool: AssertUnwindSafe<FsPool>,
}

impl FsPoolMiddleware {
    pub fn new(pool: FsPool) -> Self {
        FsPoolMiddleware {
            pool: AssertUnwindSafe(pool),
        }
    }

    pub fn with_size(size: usize) -> Self {
        let pool = FsPool::new(size);
        Self::new(pool)
    }
}

impl Middleware for FsPoolMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture> + 'static,
        Self: Sized,
    {
        state.put(FsPoolMiddlewareData::new(self.pool.clone()));

        chain(state)
    }
}

impl NewMiddleware for FsPoolMiddleware {
    type Instance = FsPoolMiddleware;

    fn new_middleware(&self) -> io::Result<Self::Instance> {
        Ok(FsPoolMiddleware {
            pool: AssertUnwindSafe(self.pool.clone()),
        })
    }
}

#[derive(StateData)]
pub struct FsPoolMiddlewareData {
    pool: FsPool,
}

impl FsPoolMiddlewareData {
    pub fn new(pool: FsPool) -> Self {
        FsPoolMiddlewareData { pool: pool }
    }

    pub fn pool_ref(&self) -> &FsPool {
        &self.pool
    }

    pub fn pool(&self) -> FsPool {
        self.pool.clone()
    }
}
