extern crate futures;
extern crate futures_fs;
extern crate gotham;
extern crate gotham_middleware_fs;
extern crate hyper;
extern crate mime;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;

use futures::{Future, Stream, future::{lazy, ok}};
use futures_fs::FsPool;
use gotham::handler::HandlerFuture;
use gotham::http::response::create_response;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use gotham_middleware_fs::{FsPoolMiddleware, FsPoolMiddlewareData};
use hyper::StatusCode;

pub fn hello_fs_async(state: State) -> Box<HandlerFuture> {
    let pool = FsPoolMiddlewareData::borrow_from(&state).pool();

    let f = pool.read("Cargo.toml").concat2().then(|res| match res {
        Ok(contents) => {
            let res = create_response(
                &state,
                StatusCode::Ok,
                Some((
                    format!("Successfully read {} bytes", contents.len()).into_bytes(),
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

pub fn hello_fs_sync(state: State) -> Box<HandlerFuture> {
    let f = lazy(|| {
        let file = File::open("Cargo.toml")?;
        let mut reader = BufReader::new(file);
        let mut contents = vec![];
        reader.read_to_end(&mut contents)?;
        Ok(contents)
    }).then(|res: Result<Vec<u8>, io::Error>| match res {
        Ok(contents) => {
            let res = create_response(
                &state,
                StatusCode::Ok,
                Some((
                    format!("Successfully read {} bytes", contents.len()).into_bytes(),
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
        route.get("/async").to(hello_fs_async);
        route.get("/sync").to(hello_fs_sync);
    })
}

pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router());
}
