//! HS256 JWT Bearer authentication — offline verification (SRV-06, ADR-039).
//!
//! MVP scope: symmetric HS256 tokens verified locally against
//! `Config::jwt_secret`. No network, no JWKS, no OIDC discovery
//! (DEFER — see ADR-039). Tokens carry `sub` (subject) + `exp` (expiry);
//! anything else is ignored, never trusted.
//!
//! Security notes:
//! - `Algorithm::HS256` is pinned in [`Validation`] — `alg:none` and
//!   algorithm-confusion tokens are rejected as [`JwtError::Malformed`].
//! - `exp` is mandatory and enforced; clock skew leeway is zero (fail closed).
//! - An empty secret is a misconfiguration: verification fails closed.
//! - Callers must never log the token or the secret (only `sub` on success).

use serde::{Deserialize, Serialize};

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

/// Verify an HS256 `token` offline against `secret`.
///
/// Returns the decoded [`Claims`] on success. Fails closed on every
/// error path — including an empty `secret` or an empty `sub`.
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, JwtError> {
    if token.is_empty() || secret.is_empty() {
        return Err(JwtError::Malformed);
    }
    let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
    // `Validation::new(HS256)` pins the algorithm and requires + enforces `exp`.
    // Leeway is zeroed: an expired token is expired (fail closed, no clock-skew grace).
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.leeway = 0;
    let data = jsonwebtoken::decode::<Claims>(token, &key, &validation).map_err(|e| {
        use jsonwebtoken::errors::ErrorKind as K;
        match e.kind() {
            K::ExpiredSignature => JwtError::Expired,
            K::InvalidSignature => JwtError::InvalidSignature,
            _ => JwtError::Malformed,
        }
    })?;
    if data.claims.sub.trim().is_empty() {
        return Err(JwtError::Malformed);
    }
    Ok(data.claims)
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

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
        encode(
            &Header::new(jsonwebtoken::Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("mint")
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
        let token = encode(
            &Header::new(jsonwebtoken::Algorithm::HS384),
            &claims,
            &EncodingKey::from_secret(SECRET.as_bytes()),
        )
        .expect("mint hs384");
        assert_eq!(verify_jwt(&token, SECRET), Err(JwtError::Malformed));
    }
}
