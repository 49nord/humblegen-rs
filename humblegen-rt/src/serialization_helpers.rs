//! `GEN` - deserialization helpers used by dispatcher.

use crate::service_protocol::ErrorResponse;
use crate::service_protocol::RuntimeError;
use crate::service_protocol::ToErrorResponse;

use serde::{Deserializer, Serializer};

pub fn deser_param<T, E>(name: &str, value: &str) -> Result<T, ErrorResponse>
where
    E: std::fmt::Display,
    T: std::str::FromStr<Err = E>,
{
    // TODO: Use std::primitive::str here, once Rust 1.43.0 has been out longer.
    str::parse(value).map_err(|e| {
        RuntimeError::RouteParamInvalid {
            param_name: name.to_owned(),
            parse_error: format!("{}", e),
        }
        .to_error_response()
    })
}

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
    str::parse(query).map_err(|e| RuntimeError::QueryInvalid(format!("{}", e)).to_error_response())
}

/// Helper function used by generate code to deserialize a humblegen `bytes` field.
pub fn deser_bytes<'de, D>(input: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct BytesSerdeVisitor;

    impl<'de> serde::de::Visitor<'de> for BytesSerdeVisitor {
        type Value = Vec<u8>;
        fn expecting(
            &self,
            formatter: &mut std::fmt::Formatter<'_>,
        ) -> std::result::Result<(), std::fmt::Error> {
            write!(formatter, "a base64-encoded byte array")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            base64::decode(v).map_err(E::custom)
        }
    }

    input.deserialize_str(BytesSerdeVisitor)
}

/// Helper function used by generate code to serialize a humblegen `bytes` field.
pub fn ser_bytes<S>(v: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&base64::encode(v))
}
