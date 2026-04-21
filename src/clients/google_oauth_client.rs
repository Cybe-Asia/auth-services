use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GoogleTokenInfoResponse {
    pub sub: Option<String>,
    pub email: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Clone)]
pub struct GoogleOAuthClient {
    http: Client,
    tokeninfo_endpoint: String,
}

impl GoogleOAuthClient {
    pub fn new(http: Client, tokeninfo_endpoint: String) -> Self {
        Self {
            http,
            tokeninfo_endpoint,
        }
    }

    pub async fn validate_id_token(
        &self,
        id_token: &str,
    ) -> Result<GoogleTokenInfoResponse, String> {
        let res = self
            .http
            .get(&self.tokeninfo_endpoint)
            .query(&[("id_token", id_token)])
            .send()
            .await
            .map_err(|e| format!("google tokeninfo request failed: {e}"))?;

        let status = res.status();
        let body = res
            .json::<GoogleTokenInfoResponse>()
            .await
            .map_err(|e| format!("google tokeninfo decode failed: {e}"))?;

        if !status.is_success() {
            let err = body
                .error_description
                .clone()
                .or(body.error.clone())
                .unwrap_or_else(|| "google token validation failed".to_string());
            return Err(err);
        }

        if body.error.is_some() {
            let err = body
                .error_description
                .clone()
                .or(body.error.clone())
                .unwrap_or_else(|| "google token validation failed".to_string());
            return Err(err);
        }

        Ok(body)
    }
}
