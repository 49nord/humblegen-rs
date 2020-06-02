//! `GEN` - deserialization helpers used by dispatcher.

use crate::service_protocol::ErrorResponse;
use crate::service_protocol::RuntimeError;
use crate::service_protocol::ToErrorResponse;

/// Helper function used by generated code to deserialize POST body data.
pub async fn deser_post_data<T: serde::de::DeserializeOwned>(
    req_body: &mut hyper::Body,
) -> Result<T, ErrorResponse> {
    let bytes = hyper::body::to_bytes(req_body)
        .await
        .map_err(|e| RuntimeError::PostBodyReadError(format!("{}", e)).to_error_response())?
        .to_vec();
    match serde_json::from_slice::<T>(&bytes[..]) {
        Ok(b) => Ok(b),
        Err(e) => Err(RuntimeError::PostBodyReadError(format!("{}", e)).to_error_response()),
    }
}

/// Helper function used by generated code to deserialize the URL query from application/x-www-form-urlencoded into a type T.
pub fn deser_query_serde_urlencoded<'a, T: serde::de::Deserialize<'a>>(
    query: &'a str,
) -> Result<T, ErrorResponse> {
    match serde_urlencoded::from_str(query) {
        Ok(q) => Ok(q),
        Err(e) => Err(RuntimeError::QueryInvalid(format!("{}", e)).to_error_response()),
    }
}

/// Helper function used by generated code to deserialize the URL query into a primitive type.
pub fn deser_query_primitive<E: std::fmt::Display, T: std::str::FromStr<Err = E>>(
    query: &str,
) -> Result<T, ErrorResponse> {
    std::primitive::str::parse(query)
        .map_err(|e| RuntimeError::QueryInvalid(format!("{}", e)).to_error_response())
}
