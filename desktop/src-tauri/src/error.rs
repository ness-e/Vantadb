//! Unified error type for the VantaDB desktop client.
//!
//! Designed to match the contract of DESKTOP-04 (`VantaError` `#[non_exhaustive]`,
//! variants by transport: `Native/Http/Mcp/Node/Python/Wasm` + `Lock/Timeout/Unsupported`).
//! DESK-02 owns the rich `Http` variants (kind/status); DESKTOP-04 added the shared
//! serde derives, the `Io`/`Serialization`/`Other` variants, `From` impls, and the
//! roundtrip tests so the error satisfies the multi-connection contract.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Kind of HTTP error, mirroring the status codes the real server returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HttpErrorKind {
    /// 401 — missing/invalid Bearer token.
    Unauthorized,
    /// 403 — insufficient RBAC permissions for the operation.
    Forbidden,
    /// 404 — unknown route / entity not found.
    NotFound,
    /// 429 — rate limit exceeded.
    TooManyRequests,
    /// 503 — circuit breaker open / server overloaded.
    ServiceUnavailable,
    /// 5xx — server-side failure.
    Server,
    /// 4xx other — bad request, etc.
    BadRequest,
    /// Domain failure: HTTP 200 but `success: false` in the body.
    Domain,
    /// Unknown status code.
    Other,
}

impl HttpErrorKind {
    fn from_status(status: u16) -> Self {
        match status {
            401 => Self::Unauthorized,
            403 => Self::Forbidden,
            404 => Self::NotFound,
            429 => Self::TooManyRequests,
            503 => Self::ServiceUnavailable,
            400..=499 => Self::BadRequest,
            500..=599 => Self::Server,
            _ => Self::Other,
        }
    }
}

impl fmt::Display for HttpErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not found",
            Self::TooManyRequests => "rate limited",
            Self::ServiceUnavailable => "service unavailable",
            Self::Server => "server error",
            Self::BadRequest => "bad request",
            Self::Domain => "domain error",
            Self::Other => "http error",
        };
        f.write_str(s)
    }
}

/// Unified, `#[non_exhaustive]` error for all VantaDB desktop connections.
///
/// Serde-compatible (JSON roundtrip contract): `Io`/`Serialization` carry a message
/// string instead of the raw error so the enum stays fully serializable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[non_exhaustive]
pub enum VantaError {
    /// Failure talking to the VantaDB HTTP server.
    #[error("http {kind}: {message}")]
    Http {
        /// Classification of the failure.
        kind: HttpErrorKind,
        /// Human-readable detail (server `data`/`error` field or reqwest error).
        message: String,
        /// Raw HTTP status code, when one was received.
        status: Option<u16>,
    },
    /// Locked database / writer already present for a path.
    #[error("lock: {0}")]
    Lock(String),
    /// Operation timed out.
    #[error("timeout: {0}")]
    Timeout(String),
    /// Operation not supported by this transport.
    #[error("unsupported: {0}")]
    Unsupported(String),
    /// Reserved for non-HTTP transports filled in by later tasks.
    #[error("native: {0}")]
    Native(String),
    #[error("mcp: {0}")]
    Mcp(String),
    #[error("node: {0}")]
    Node(String),
    #[error("python: {0}")]
    Python(String),
    #[error("wasm: {0}")]
    Wasm(String),
    /// I/O failure (string payload keeps the enum serde-serializable).
    #[error("io: {0}")]
    Io(String),
    /// (De)serialization failure (string payload keeps the enum serde-serializable).
    #[error("serialization: {0}")]
    Serialization(String),
    /// Anything else.
    #[error("{0}")]
    Other(String),
}

impl VantaError {
    /// Wrap an HTTP status + body message into a `VantaError::Http`.
    ///
    /// A `200` with `success:false` is classified as a domain error
    /// (`HttpErrorKind::Domain`), matching the VantaDB server contract where
    /// statement failures are returned as HTTP 200 with `success` in the JSON body.
    pub fn from_http_status(status: u16, message: impl Into<String>) -> Self {
        let kind = if status == 200 {
            HttpErrorKind::Domain
        } else {
            HttpErrorKind::from_status(status)
        };
        Self::Http {
            kind,
            message: message.into(),
            status: Some(status),
        }
    }
}

impl From<std::io::Error> for VantaError {
    fn from(e: std::io::Error) -> Self {
        VantaError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for VantaError {
    fn from(e: serde_json::Error) -> Self {
        VantaError::Serialization(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_serde_roundtrip_all_variants() {
        let all = vec![
            VantaError::Http {
                kind: HttpErrorKind::Unauthorized,
                message: "token missing".into(),
                status: Some(401),
            },
            VantaError::Http {
                kind: HttpErrorKind::Domain,
                message: "statement failed".into(),
                status: None,
            },
            VantaError::Lock("locked".into()),
            VantaError::Timeout("slow".into()),
            VantaError::Unsupported("nope".into()),
            VantaError::Native("n".into()),
            VantaError::Mcp("m".into()),
            VantaError::Node("n".into()),
            VantaError::Python("p".into()),
            VantaError::Wasm("w".into()),
            VantaError::Io("i".into()),
            VantaError::Serialization("s".into()),
            VantaError::Other("o".into()),
        ];
        for err in &all {
            let json = serde_json::to_string(err).expect("serialize");
            let back: VantaError = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&back, err);
        }
    }

    #[test]
    fn from_io_and_serde_json() {
        let io: VantaError = std::io::Error::new(std::io::ErrorKind::Other, "disk").into();
        assert!(matches!(io, VantaError::Io(_)));
        let ser: VantaError = serde_json::from_str::<serde_json::Value>("").unwrap_err().into();
        assert!(matches!(ser, VantaError::Serialization(_)));
    }

    #[test]
    fn from_http_status_classifies() {
        assert!(matches!(
            VantaError::from_http_status(200, "x"),
            VantaError::Http { kind: HttpErrorKind::Domain, status: Some(200), .. }
        ));
        assert!(matches!(
            VantaError::from_http_status(503, "x"),
            VantaError::Http { kind: HttpErrorKind::ServiceUnavailable, .. }
        ));
    }
}
