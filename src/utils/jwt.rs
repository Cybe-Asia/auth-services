use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
}

pub fn generate_access_token(secret: &str, sub: &str, email: &str) -> Result<String, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system time before unix epoch".to_string())?
        .as_secs();

    let claims = JwtClaims {
        sub: sub.to_string(),
        email: email.to_string(),
        exp: (now + 3600) as usize,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("jwt encode failed: {e}"))
}

pub fn verify_access_token(secret: &str, token: &str) -> Result<JwtClaims, String> {
    jsonwebtoken::decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("jwt decode failed: {e}"))
}
