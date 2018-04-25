extern crate bytes;
extern crate futures;
extern crate futures_fs;
extern crate gotham;
extern crate gotham_middleware_fs;
extern crate hyper;
extern crate mime;

use futures::future::{ok, Future};
use futures_fs::FsPool;
use gotham::handler::HandlerFuture;
use gotham::http::response::create_response;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use gotham_middleware_fs::{FsPoolMiddleware, FsPoolMiddlewareData};
use gotham_middleware_fs::response::WriteResponseStream;
use hyper::StatusCode;

pub fn stream_response(state: State) -> Box<HandlerFuture> {
    let pool = FsPoolMiddlewareData::borrow_from(&state).pool();
    let input = pool.read("Cargo.toml");
    let f = input.into_response().then(|res| match res {
        Ok(response) => ok((state, response)),
        Err(err) => {
            let response = create_response(
                &state,
                StatusCode::InternalServerError,
                Some((
                    format!("Error streaming file: {}", err).into_bytes(),
                    mime::TEXT_PLAIN,
                )),
            );
            ok((state, response))
        }
    });

    Box::new(f)
}

fn router() -> Router {
    let fs_pool = FsPool::default();
    let fs_mw = FsPoolMiddleware::new(fs_pool);

    let (chain, pipelines) = single_pipeline(new_pipeline().add(fs_mw).build());

    build_router(chain, pipelines, |route| {
        route.get("/").to(stream_response);
    })
}

/// curl -v http://localhost:7878
pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router());
}
