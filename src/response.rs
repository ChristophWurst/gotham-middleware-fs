use std::io;

use bytes::Bytes;
use futures::{Future, Stream};
use gotham::handler::HandlerError;
use gotham::state::State;
use hyper::{Chunk, Error, Response};

type ChunkStream = Stream<Item = Chunk, Error = Error>;

pub type StreamingHandlerFuture =
    Future<Item = (State, Response<Box<ChunkStream>>), Error = (State, HandlerError)> + 'static;

pub trait WriteResponseStream {
    fn into_response(self) -> Response<Box<ChunkStream>>;
}

impl<S> WriteResponseStream for S
where
    S: Stream<Item = Bytes, Error = io::Error> + 'static,
{
    fn into_response(self) -> Response<Box<ChunkStream>> {
        let mapped: Box<ChunkStream> =
            Box::new(self.map(Chunk::from).map_err(|err| Error::Io(err)));
        Response::new().with_body(mapped)
    }
}
