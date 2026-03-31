use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct GoogleLoginRequest {
    #[serde(rename = "googleToken")]
    pub google_token: String,
}
