use axum::{http::StatusCode, response::IntoResponse};
use std::{num::ParseIntError, sync::PoisonError};
use thiserror::Error;
use tokio::sync::broadcast::error::SendError;

use crate::frame::Frame;

#[derive(Error, Debug)]
pub enum Error {
    #[error("axum error {0:?}")]
    Axum(#[from] axum::Error),
    #[error("stream with id {0:?} already exists")]
    Conflict(String),
    #[error("poison error")]
    PoisonError,
    #[error("parse error {0:?}")]
    ParseError(#[from] ParseIntError),
    #[error("channel send error {0:?}")]
    ChannelSendError(#[from] SendError<Frame>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Conflict(_) => (StatusCode::CONFLICT, self.to_string()).into_response(),
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_value: PoisonError<T>) -> Self {
        Self::PoisonError
    }
}
