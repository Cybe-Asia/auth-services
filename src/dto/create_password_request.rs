use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreatePasswordRequest {
    #[serde(rename = "newPassword")]
    pub new_password: String,
}
