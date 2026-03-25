use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    #[serde(rename = "responseCode")]
    pub response_code: u16,
    #[serde(rename = "responseMessage")]
    pub response_message: String,
    #[serde(rename = "responseError")]
    pub response_error: Option<String>,
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T) -> Json<Self> {
        Json(Self {
            response_code: 200,
            response_message: "success".to_string(),
            response_error: None,
            data: Some(data),
        })
    }

    pub fn failure(response_code: u16, error: impl Into<String>) -> Json<Self> {
        Json(Self {
            response_code,
            response_message: "failed".to_string(),
            response_error: Some(error.into()),
            data: None,
        })
    }
}
