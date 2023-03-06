use crate::{
    error::Error,
    stream::{OwnedStream, Stream},
};
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

#[derive(Default)]
pub struct Context {
    streams: RwLock<BTreeMap<String, Arc<Stream>>>,
}

impl Context {
    pub fn get_stream(&self, k: &str) -> Result<Option<Arc<Stream>>, Error> {
        Ok(self.streams.read()?.get(k).map(Arc::clone))
    }

    pub fn create_stream(&self, k: String) -> Result<OwnedStream, Error> {
        let stream = Arc::new(Stream::new());
        self.streams.write()?.insert(k.clone(), Arc::clone(&stream));
        Ok(OwnedStream::new(k, stream, self))
    }

    pub fn drop_stream(&self, k: &str) -> Result<Option<Arc<Stream>>, Error> {
        Ok(self.streams.write()?.remove(k))
    }

    pub fn list_streams(&self) -> Result<Vec<String>, Error> {
        Ok(self.streams.read()?.keys().cloned().collect())
    }

    pub fn search_streams(&self, str: &str) -> Result<Vec<String>, Error> {
        Ok(self
            .streams
            .read()?
            .range(str.to_string()..)
            .take_while(|(k, _)| k.starts_with(str))
            .map(|(k, _)| k)
            .cloned()
            .collect())
    }
}
