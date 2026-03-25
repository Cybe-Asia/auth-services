use axum::{extract::State, http::HeaderMap, Json};
use serde::Serialize;

use crate::{
    dto::{
        create_password_request::CreatePasswordRequest, google_login_request::GoogleLoginRequest,
        login_request::LoginRequest,
    },
    utils::response::ApiResponse,
    AppState,
};

#[derive(Serialize)]
pub struct CreatePasswordData {
    #[serde(rename = "passwordCreated")]
    pub password_created: bool,
}

#[derive(Serialize)]
pub struct LoginData {
    #[serde(rename = "jwtAccessToken")]
    pub jwt_access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct GoogleLoginData {
    #[serde(rename = "jwtAccessToken")]
    pub jwt_access_token: String,
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<String, String> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or_else(|| "Missing Authorization header".to_string())?
        .to_str()
        .map_err(|_| "Invalid Authorization header".to_string())?;

    let value = value
        .strip_prefix("Bearer ")
        .ok_or_else(|| "Authorization header must be Bearer token".to_string())?;

    Ok(value.to_string())
}

pub async fn create_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreatePasswordRequest>,
) -> Json<ApiResponse<CreatePasswordData>> {
    let bearer = match extract_bearer_token(&headers) {
        Ok(t) => t,
        Err(e) => return ApiResponse::failure(500, e),
    };

    match state
        .auth_service
        .create_password(&bearer, &payload.new_password)
        .await
    {
        Ok(password_created) => ApiResponse::success(CreatePasswordData { password_created }),
        Err(e) => ApiResponse::failure(500, e),
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Json<ApiResponse<LoginData>> {
    match state
        .auth_service
        .login(&payload.username, &payload.password)
        .await
    {
        Ok(tokens) => ApiResponse::success(LoginData {
            jwt_access_token: tokens.jwt_access_token,
            refresh_token: tokens.refresh_token,
        }),
        Err(e) => ApiResponse::failure(500, e),
    }
}

pub async fn google_login(
    State(state): State<AppState>,
    Json(payload): Json<GoogleLoginRequest>,
) -> Json<ApiResponse<GoogleLoginData>> {
    match state.auth_service.google_login(&payload.google_token).await {
        Ok(jwt_access_token) => ApiResponse::success(GoogleLoginData { jwt_access_token }),
        Err(e) => ApiResponse::failure(500, e),
    }
}
