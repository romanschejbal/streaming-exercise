use axum::{
    body::HttpBody,
    handler::Handler,
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::MethodRouter,
};
use futures::Future;
use std::{convert::Infallible, pin::Pin};

pub fn list<H, T, S, B>(handler: H) -> MethodRouter<S, B, Infallible>
where
    H: Handler<T, S, B>,
    B: HttpBody + Send + 'static,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    MethodRouter::new().fallback(ensure_list_method(handler))
}

fn ensure_list_method<H, T, S, B>(handler: H) -> ListHandler<H>
where
    H: Handler<T, S, B>,
    B: HttpBody + Send + 'static,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    ListHandler { handler }
}

#[derive(Clone)]
struct ListHandler<H> {
    handler: H,
}

impl<H, T, S, B> Handler<T, S, B> for ListHandler<H>
where
    H: Handler<T, S, B>,
    B: HttpBody + Send + 'static,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(self, req: Request<B>, state: S) -> Self::Future {
        if req.method() != Method::from_bytes(b"LIST").unwrap() {
            return Box::pin(async { StatusCode::NOT_FOUND.into_response() });
        }
        Box::pin(self.handler.call(req, state))
    }
}

pub trait ListExt<S, B> {
    fn list<H, T>(self, handler: H) -> Self
    where
        H: Handler<T, S, B>,
        T: 'static,
        S: Send + Sync + 'static;
}

impl<S, B> ListExt<S, B> for MethodRouter<S, B, Infallible>
where
    B: HttpBody + Send + 'static,
    S: Clone,
{
    fn list<H, T>(self, handler: H) -> Self
    where
        H: Handler<T, S, B>,
        T: 'static,
        S: Send + Sync + 'static,
    {
        self.fallback(ensure_list_method(handler))
    }
}
