extern crate bytes;
extern crate futures;
extern crate futures_fs;
extern crate gotham;
extern crate gotham_middleware_fs;
extern crate hyper;
extern crate mime;

use futures::{Future, Stream, future::ok};
use futures_fs::FsPool;
use gotham::handler::HandlerFuture;
use gotham::http::response::create_response;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use gotham_middleware_fs::{FsPoolMiddleware, FsPoolMiddlewareData, body::ReadBodyStream};
use hyper::StatusCode;

pub fn stream_body(mut state: State) -> Box<HandlerFuture> {
    let pool = FsPoolMiddlewareData::borrow_from(&state).pool();
    let body = pool.read_body(&mut state);
    let write = pool.write("/tmp/upload", Default::default());

    let f = body.forward(write).then(|res| match res {
        Ok(_contents) => {
            let res = create_response(
                &state,
                StatusCode::Ok,
                Some((
                    format!("File successfully uploaded").into_bytes(),
                    mime::TEXT_PLAIN,
                )),
            );

            ok((state, res))
        }
        Err(err) => {
            let res = create_response(
                &state,
                StatusCode::InternalServerError,
                Some((
                    format!("Error reading file: {}", err).into_bytes(),
                    mime::TEXT_PLAIN,
                )),
            );

            ok((state, res))
        }
    });

    Box::new(f)
}

fn router() -> Router {
    let fs_pool = FsPool::default();
    let fs_mw = FsPoolMiddleware::new(fs_pool);

    let (chain, pipelines) = single_pipeline(new_pipeline().add(fs_mw).build());

    build_router(chain, pipelines, |route| {
        route.post("/").to(stream_body);
    })
}

/// curl -X POST http://localhost:7878 --data-binary @<upload-file>
pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router());
}
