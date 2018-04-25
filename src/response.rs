use std::io;

use bytes::Bytes;
use futures::{Future, Sink, Stream, future};
use hyper::{header, Body, Chunk, Error, Response};
use tokio::executor::{DefaultExecutor, Executor};

pub trait WriteResponseStream {
    fn into_response(self) -> Box<Future<Item = Response<Body>, Error = Error>>;
}

impl<S> WriteResponseStream for S
where
    S: Stream<Item = Bytes, Error = io::Error> + 'static + Send,
{
    fn into_response(self) -> Box<Future<Item = Response<Body>, Error = Error>> {
        // Create a streamable body
        let (sender, body) = Body::pair();
        // Create a new response and make sure the `Content-Type` header is not set
        let mut res = Response::new().with_body(body);
        res.headers_mut().remove::<header::ContentLength>();

        // Convert sink errors to Hyper errors
        let sender = sender
            .sink_map_err(|e| Error::from(io::Error::new(io::ErrorKind::Other, e)));

        let stream = self.map(|bytes| Ok(bytes.into()));

        let streaming_future = sender.send_all(stream)
            .map(|(_sink, _stream)| ())
            .map_err(|_e| println!("not much we can do to fix this")); //error!("Error streaming from Fs to body")

        DefaultExecutor::current().spawn(Box::new(streaming_future)).unwrap();

        Box::new(future::ok(res))
    }
}
