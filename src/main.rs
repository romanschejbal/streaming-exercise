mod axum_ext;
mod context;
mod error;
mod frame;
mod stream;

use axum::{
    body::StreamBody,
    extract::{BodyStream, Path, State},
    http::{header::TRANSFER_ENCODING, HeaderMap, HeaderValue, StatusCode},
    response::{AppendHeaders, IntoResponse, Response, Result as AxumResult},
    routing::get,
    Json, Router,
};
use axum_ext::{list, ListExt};
use context::Context;
use error::Error;
use frame::Frame;
use futures::StreamExt;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let services = Arc::new(Context::default());

    let router = Router::new()
        .route("/", list(list_streams))
        .route(
            "/*id",
            get(send_stream)
                .put(receive_stream)
                .list(list_streams)
                .delete(delete_stream),
        )
        .with_state(services);

    let server =
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(router.into_make_service());
    info!("Server running at {}", server.local_addr());
    server.await.unwrap();
    Ok(())
}

async fn list_streams(
    State(context): State<Arc<Context>>,
    search: Option<Path<String>>,
) -> AxumResult<Json<Vec<String>>> {
    if let Some(Path(search)) = search {
        if search.ends_with("*") {
            info!("Searching for streams starting with {search:?}");
            return Ok(Json(
                context
                    .search_streams(&search.replace("*", ""))
                    .unwrap_or(vec![]),
            ));
        } else {
            return Ok(Json(
                context
                    .get_stream(&search)?
                    .map(|_| vec![search])
                    .unwrap_or_default(),
            ));
        }
    }
    info!("Listing all streams");
    Ok(Json(context.list_streams().unwrap_or(vec![])))
}

#[axum_macros::debug_handler]
async fn receive_stream(
    State(context): State<Arc<Context>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    mut body_stream: BodyStream,
) -> AxumResult<Response> {
    if let Some(_stream) = context.get_stream(&id).map_err(Error::from)? {
        warn!("Attempt to stream to an already existing stream {id:?}");
        return Err(Error::Conflict(id))?;
    }

    info!("Receiving stream {id:?} with headers {:?}", headers);
    let stream = context.create_stream(id.clone()).map_err(|e| {
        error!("Creating stream failed with error: {e}");
        e
    })?;

    if headers.get(TRANSFER_ENCODING) == Some(&HeaderValue::from_static("chunked")) {
        while let Some(body) = body_stream.next().await {
            let body = body.map_err(|e| {
                error!("Error while receiving stream: {e}");
                Error::from(e)
            })?;

            let frame = Frame::try_from(body).map_err(|e| {
                error!("Parsing frame failed with error: {e}");
                Error::from(e)
            })?;

            stream.send(frame).map_err(|e| {
                error!("Publishing frame failed with error: {e}");
                Error::from(e)
            })?;
        }
    }

    Ok(StatusCode::OK.into_response())
}

#[axum_macros::debug_handler]
async fn send_stream(
    State(context): State<Arc<Context>>,
    Path(id): Path<String>,
) -> AxumResult<Response> {
    info!("Subscribing to stream {id:?}");
    // for convenience, we'll hold onto the connection until the stream exists
    let stream = loop {
        if let Some(stream) = context.get_stream(&id).map_err(Error::from)? {
            break stream;
        }
        tokio::task::yield_now().await;
    };
    let headers = AppendHeaders([(TRANSFER_ENCODING, "chunked")]);
    let rx_stream = stream.subscribe();
    let broadcast_stream = BroadcastStream::new(rx_stream);
    let stream = StreamBody::new(broadcast_stream);
    info!("Subscribed to stream {id:?}");
    Ok((headers, stream).into_response())
}

#[axum_macros::debug_handler]
async fn delete_stream(
    State(context): State<Arc<Context>>,
    Path(id): Path<String>,
) -> AxumResult<Response> {
    context.drop_stream(&id).map_err(|e| {
        error!("Dropping stream failed with error {e:?}");
        Error::from(e)
    })?;
    Ok(StatusCode::OK.into_response())
}
