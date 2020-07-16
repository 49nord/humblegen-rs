//! `HANDLER` Types used by a handler implementation, re-exported by generated code.

use core::fmt::Display;

/// The response type returned by implementors of a humblegen service trait function.
pub type HandlerResponse<T> = Result<T, ServiceError>;

/// A service-level error.
///
/// This type is returned by implementors of a humblegen service trait function
/// as part of a `HandlerResponse`.
///
/// The runtime converts it to a `super::service_protocol::ServiceError`.
#[derive(Debug)]
pub enum ServiceError {
    Authentication,
    Authorization,
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::Authentication => write!(f, "authentication error"),
            ServiceError::Authorization => write!(f, "not authorized"),
            ServiceError::Internal(e) => write!(f, "internal server error: {:?}", e),
        }
    }
}
