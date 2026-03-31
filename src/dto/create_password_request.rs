use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePasswordRequest {
    #[serde(rename = "newPassword")]
    pub new_password: String,
}
