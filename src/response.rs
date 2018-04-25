use std::io;

use bytes::Bytes;
use futures::{Future, Sink, Stream};
use hyper::{Body, Chunk, Error, Response};

pub trait WriteResponseStream {
    fn into_response(self) -> Box<Future<Item = Response<Body>, Error = Error>>;
}

impl<S> WriteResponseStream for S
where
    S: Stream<Item = Bytes, Error = io::Error> + 'static,
{
    fn into_response(self) -> Box<Future<Item = Response<Body>, Error = Error>> {
        let (sender, body) = Body::pair();
        let res = Response::new().with_body(body);

        let sender = sender.sink_map_err(|e| Error::from(io::Error::new(io::ErrorKind::Other, e)));
        let f = self.map(|bytes| {
            println!("read chunk of {} bytes", bytes.len());
            Ok(Chunk::from(bytes))
        }).map_err(|err| Error::Io(err))
            .forward(sender)
            .and_then(|_| Ok(res));

        Box::new(f)
    }
}
