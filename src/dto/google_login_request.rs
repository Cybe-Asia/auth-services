use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GoogleLoginRequest {
    #[serde(rename = "googleToken")]
    pub google_token: String,
}
