use jsonrpc_pubsub::Subscriber;
use log::*;
use thiserror::Error;
use tokio::sync::oneshot;

#[derive(Error, Debug)]
pub enum PubsubError {
    #[error("Failed to confirm subscription: {0:?}")]
    FailedToSendSubscription(Box<dyn std::error::Error>),

    #[error("Failed to confirm subscription: {0:?}")]
    FailedToConfirmSubscription(Box<oneshot::error::RecvError>),
}

pub type PubsubResult<T> = Result<T, PubsubError>;

// -----------------
// Subscriber Errors
// -----------------
pub fn reject_internal_error<T: std::fmt::Debug>(
    subscriber: Subscriber,
    msg: &str,
    err: Option<T>,
) {
    _reject_subscriber_error(
        subscriber,
        msg,
        err,
        jsonrpc_core::ErrorCode::InternalError,
    )
}

pub fn reject_parse_error<T: std::fmt::Debug>(
    subscriber: Subscriber,
    msg: &str,
    err: Option<T>,
) {
    _reject_subscriber_error(
        subscriber,
        msg,
        err,
        jsonrpc_core::ErrorCode::ParseError,
    )
}

fn _reject_subscriber_error<T: std::fmt::Debug>(
    subscriber: Subscriber,
    msg: &str,
    err: Option<T>,
    code: jsonrpc_core::ErrorCode,
) {
    let message = match err {
        Some(err) => format!("{msg}: {:?}", err),
        None => msg.to_string(),
    };
    if let Err(reject_err) = subscriber.reject(jsonrpc_core::Error {
        code,
        message,
        data: None,
    }) {
        error!("Failed to reject subscriber: {:?}", reject_err);
    };
}
