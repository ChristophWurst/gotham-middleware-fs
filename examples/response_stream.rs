extern crate bytes;
extern crate futures;
extern crate futures_fs;
extern crate gotham;
extern crate gotham_middleware_fs;
extern crate hyper;
extern crate mime;

use futures::future::ok;
use futures_fs::FsPool;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use gotham_middleware_fs::{FsPoolMiddleware, FsPoolMiddlewareData};
use gotham_middleware_fs::response::{StreamingHandlerFuture, WriteResponseStream};

pub fn stream_response(mut state: State) -> Box<StreamingHandlerFuture> {
    let pool = FsPoolMiddlewareData::borrow_from(&state).pool();
    let input = pool.read("Cargo.toml");
    let res = input.into_response();

    Box::new(ok((state, res)))
}

fn router() -> Router {
    let fs_pool = FsPool::default();
    let fs_mw = FsPoolMiddleware::new(fs_pool);

    let (chain, pipelines) = single_pipeline(new_pipeline().add(fs_mw).build());

    build_router(chain, pipelines, |route| {
        route.post("/").to(stream_response);
    })
}

/// curl -X POST http://localhost:7878 --data-binary @<upload-file>
pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router());
}
