use jsonrpc_pubsub::Subscriber;
use log::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PubsubError {
    #[error("Failed to confirm subscription: {0}")]
    FailedToSendSubscription(String),
}

pub type PubsubResult<T> = Result<T, PubsubError>;

// -----------------
// Subscriber Errors
// -----------------
#[allow(dead_code)]
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

#[allow(dead_code)]
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
