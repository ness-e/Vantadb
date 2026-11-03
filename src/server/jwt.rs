//! HS256 JWT Bearer authentication — offline verification (SRV-06, ADR-039).
//!
//! MVP scope: symmetric HS256 tokens verified locally against
//! `Config::jwt_secret`. No network, no JWKS, no OIDC discovery
//! (DEFER — see ADR-039). Tokens carry `sub` (subject) + `exp` (expiry);
//! anything else is ignored, never trusted.
//!
//! Security notes:
//! - Pinned to `HS256` — `alg:none` and algorithm-confusion tokens are rejected
//!   as [`JwtError::Malformed`].
//! - Constant-time signature verification (`verify_slice`) is executed BEFORE
//!   parsing or trusting any claims.
//! - `exp` is mandatory and enforced; clock skew leeway is zero (fail closed).
//! - An empty secret is a misconfiguration: verification fails closed.
//! - Callers must never log the token or the secret (only `sub` on success).

use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

/// Claims accepted by the MVP verifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject the token was issued for (required, non-empty).
    pub sub: String,
    /// Expiry as seconds since Unix epoch (required, enforced).
    pub exp: u64,
    /// Issued-at as seconds since Unix epoch (optional, informational).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iat: Option<u64>,
}

/// JWT verification failure.
///
/// `#[non_exhaustive]` so OIDC-slice variants (issuer, audience, JWKS)
/// can be added later without a breaking change.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JwtError {
    /// Signature does not match the configured secret.
    InvalidSignature,
    /// `exp` is in the past (or missing).
    Expired,
    /// Not a JWT, wrong algorithm, empty subject/secret, or otherwise unusable.
    Malformed,
}

impl std::fmt::Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JwtError::InvalidSignature => write!(f, "invalid JWT signature"),
            JwtError::Expired => write!(f, "JWT expired"),
            JwtError::Malformed => write!(f, "malformed JWT"),
        }
    }
}

impl std::error::Error for JwtError {}

#[derive(Deserialize)]
struct JwtHeader {
    alg: String,
}

/// Verify an HS256 `token` offline against `secret`.
///
/// Returns the decoded [`Claims`] on success. Fails closed on every
/// error path — including an empty `secret` or an empty `sub`.
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, JwtError> {
    if token.is_empty() || secret.is_empty() {
        return Err(JwtError::Malformed);
    }

    let mut parts = token.split('.');
    let (Some(header_b64), Some(claims_b64), Some(sig_b64), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(JwtError::Malformed);
    };
    if header_b64.is_empty() || claims_b64.is_empty() || sig_b64.is_empty() {
        return Err(JwtError::Malformed);
    }

    // Decode header and verify algorithm is strictly HS256
    let header_bytes = URL_SAFE_NO_PAD
        .decode(header_b64)
        .map_err(|_| JwtError::Malformed)?;
    let header: JwtHeader =
        serde_json::from_slice(&header_bytes).map_err(|_| JwtError::Malformed)?;
    if header.alg != "HS256" {
        return Err(JwtError::Malformed);
    }

    // Decode signature
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| JwtError::Malformed)?;

    // Verify HMAC-SHA256 signature in constant time BEFORE trusting/decoding claims
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| JwtError::Malformed)?;
    mac.update(header_b64.as_bytes());
    mac.update(b".");
    mac.update(claims_b64.as_bytes());
    mac.verify_slice(&sig_bytes)
        .map_err(|_| JwtError::InvalidSignature)?;

    // Decode and parse claims
    let claims_bytes = URL_SAFE_NO_PAD
        .decode(claims_b64)
        .map_err(|_| JwtError::Malformed)?;
    let claims: Claims = serde_json::from_slice(&claims_bytes).map_err(|_| JwtError::Malformed)?;

    // Verify expiration without leeway (fail closed)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| JwtError::Malformed)?
        .as_secs();
    if claims.exp < now {
        return Err(JwtError::Expired);
    }

    if claims.sub.trim().is_empty() {
        return Err(JwtError::Malformed);
    }

    Ok(claims)
}

#[cfg(test)]
pub(crate) fn encode_hs256(claims: &Claims, secret: &str) -> String {
    let header_json = serde_json::json!({
        "alg": "HS256",
        "typ": "JWT"
    });
    let header_b64 = URL_SAFE_NO_PAD.encode(header_json.to_string().as_bytes());
    let claims_bytes = serde_json::to_vec(claims).expect("serialize claims");
    let claims_b64 = URL_SAFE_NO_PAD.encode(&claims_bytes);
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac key");
    mac.update(header_b64.as_bytes());
    mac.update(b".");
    mac.update(claims_b64.as_bytes());
    let sig = mac.finalize().into_bytes();
    let sig_b64 = URL_SAFE_NO_PAD.encode(sig);
    format!("{header_b64}.{claims_b64}.{sig_b64}")
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-with-at-least-32-bytes!!";
    const OTHER_SECRET: &str = "a-different-secret-with-32-bytes!!!";

    fn mint(sub: &str, exp_offset_secs: i64, secret: &str) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_secs() as i64;
        let claims = Claims {
            sub: sub.to_string(),
            exp: (now + exp_offset_secs).max(0) as u64,
            iat: Some(now.max(0) as u64),
        };
        encode_hs256(&claims, secret)
    }

    #[test]
    fn valid_token_returns_claims() {
        let token = mint("svc-1", 3600, SECRET);
        let claims = verify_jwt(&token, SECRET).expect("valid");
        assert_eq!(claims.sub, "svc-1");
    }

    #[test]
    fn expired_token_is_rejected() {
        let token = mint("svc-1", -60, SECRET);
        assert_eq!(verify_jwt(&token, SECRET), Err(JwtError::Expired));
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let token = mint("svc-1", 3600, OTHER_SECRET);
        assert_eq!(verify_jwt(&token, SECRET), Err(JwtError::InvalidSignature));
    }

    #[test]
    fn malformed_and_empty_inputs_fail_closed() {
        assert_eq!(verify_jwt("", SECRET), Err(JwtError::Malformed));
        assert_eq!(verify_jwt("not.a.jwt", SECRET), Err(JwtError::Malformed));
        let token = mint("svc-1", 3600, SECRET);
        assert_eq!(verify_jwt(&token, ""), Err(JwtError::Malformed));
    }

    #[test]
    fn empty_subject_is_rejected() {
        let token = mint("   ", 3600, SECRET);
        assert_eq!(verify_jwt(&token, SECRET), Err(JwtError::Malformed));
    }

    #[test]
    fn wrong_algorithm_is_rejected() {
        // HS384-signed token presented to an HS256-only validator.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_secs();
        let claims = Claims {
            sub: "svc-1".into(),
            exp: now + 3600,
            iat: None,
        };
        let header_json = serde_json::json!({
            "alg": "HS384",
            "typ": "JWT"
        });
        let header_b64 = URL_SAFE_NO_PAD.encode(header_json.to_string().as_bytes());
        let claims_bytes = serde_json::to_vec(&claims).expect("serialize claims");
        let claims_b64 = URL_SAFE_NO_PAD.encode(&claims_bytes);
        let mut mac = Hmac::<Sha256>::new_from_slice(SECRET.as_bytes()).expect("hmac key");
        mac.update(header_b64.as_bytes());
        mac.update(b".");
        mac.update(claims_b64.as_bytes());
        let sig = mac.finalize().into_bytes();
        let sig_b64 = URL_SAFE_NO_PAD.encode(sig);
        let token = format!("{header_b64}.{claims_b64}.{sig_b64}");
        assert_eq!(verify_jwt(&token, SECRET), Err(JwtError::Malformed));
    }
}
