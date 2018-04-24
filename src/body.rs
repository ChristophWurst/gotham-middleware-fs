use bytes::Bytes;
use futures::Stream;
use futures_fs::FsPool;
use gotham::state::{FromState, State};
use hyper::{self, Body};

pub trait ReadBodyStream {
    fn read_body(&self, state: &mut State) -> Box<Stream<Item = Bytes, Error = hyper::Error>>;
}

impl ReadBodyStream for FsPool {
    fn read_body(&self, state: &mut State) -> Box<Stream<Item = Bytes, Error = hyper::Error>> {
        let body = Body::take_from(state);

        Box::new(body.and_then(|chunk| Ok(Bytes::from(chunk.to_vec()))))
    }
}
