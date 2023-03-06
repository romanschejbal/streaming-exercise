use crate::{context::Context, frame::Frame};
use std::{ops::Deref, sync::Arc};
use tokio::sync::broadcast::{self, error::SendError, Receiver, Sender};
use tracing::{debug, error};

pub struct Stream {
    publisher: Sender<Frame>,
    _subscriber: Receiver<Frame>,
}

impl Stream {
    pub fn new() -> Stream {
        let (publisher, _subscriber) = broadcast::channel(1);
        Stream {
            publisher,
            _subscriber,
        }
    }

    pub fn subscribe(&self) -> Receiver<Frame> {
        self.publisher.subscribe()
    }

    pub fn send(&self, chunk: Frame) -> Result<usize, SendError<Frame>> {
        self.publisher.send(chunk)
    }
}

pub struct OwnedStream<'ctx> {
    id: String,
    stream: Arc<Stream>,
    context: &'ctx Context,
}

impl<'ctx> OwnedStream<'ctx> {
    pub fn new(id: String, stream: Arc<Stream>, context: &'ctx Context) -> Self {
        Self {
            id,
            stream,
            context,
        }
    }
}

impl Deref for OwnedStream<'_> {
    type Target = Stream;

    fn deref(&self) -> &Self::Target {
        self.stream.as_ref()
    }
}

impl Drop for OwnedStream<'_> {
    fn drop(&mut self) {
        self.context
            .drop_stream(&self.id)
            .map(|_| debug!("Dropped stream {:?}", self.id))
            .map_err(|e| error!("Error while dropping stream {e:?}"))
            .ok();
    }
}
