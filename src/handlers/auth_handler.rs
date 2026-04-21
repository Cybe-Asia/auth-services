use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    dto::{
        create_account_request::CreateAccountRequest,
        create_password_request::CreatePasswordRequest, google_login_request::GoogleLoginRequest,
        login_request::LoginRequest,
    },
    services::auth_service::CreateAccountResult,
    utils::response::ApiResponse,
    AppState,
};

#[derive(Serialize, ToSchema)]
pub struct CreateAccountData {
    pub email: String,
    #[serde(rename = "jwtSessionToken")]
    pub jwt_session_token: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreatePasswordData {
    #[serde(rename = "passwordCreated")]
    pub password_created: bool,
}

#[derive(Serialize, ToSchema)]
pub struct LoginData {
    #[serde(rename = "jwtAccessToken")]
    pub jwt_access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Serialize, ToSchema)]
pub struct GoogleLoginData {
    #[serde(rename = "identityAccountId")]
    pub identity_account_id: String,
    #[serde(rename = "accountType")]
    pub account_type: String,
    pub username: String,
    #[serde(rename = "externalSubjectId")]
    pub external_subject_id: String,
    pub status: String,
    #[serde(rename = "lastLoginAt")]
    pub last_login_at: String,
    #[serde(rename = "jwtAccessToken")]
    pub jwt_access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
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

/// Create new account
#[utoipa::path(
    post,
    path = "/api/v1/auth-service/createAccount",
    request_body = CreateAccountRequest,
    responses(
        (status = 200, description = "Account created successfully", body = ApiResponseCreateAccount),
        (status = 409, description = "Email already exists", body = ApiResponseCreateAccount),
        (status = 500, description = "Internal server error", body = ApiResponseCreateAccount)
    )
)]
pub async fn create_account(
    State(state): State<AppState>,
    Json(payload): Json<CreateAccountRequest>,
) -> axum::response::Response {
    match state.auth_service.create_account(&payload.email).await {
        Ok(CreateAccountResult::Created(tokens)) => {
            ApiResponse::success(CreateAccountData {
                email: tokens.email,
                jwt_session_token: tokens.jwt_session_token,
            })
            .into_response()
        }
        Ok(CreateAccountResult::AlreadyExists) => {
            ApiResponse::<CreateAccountData>::failure(409, "email already exist").into_response()
        }
        Err(e) => ApiResponse::<CreateAccountData>::failure(500, e).into_response(),
    }
}

/// Create new password after account creation
#[utoipa::path(
    post,
    path = "/api/v1/auth-service/createPassword",
    request_body = CreatePasswordRequest,
    responses(
        (status = 200, description = "Password created successfully", body = ApiResponseCreatePassword),
        (status = 500, description = "Internal server error", body = ApiResponseCreatePassword)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Login with email and password
#[utoipa::path(
    post,
    path = "/api/v1/auth-service/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponseLogin),
        (status = 500, description = "Login failed", body = ApiResponseLogin)
    )
)]
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

/// Login with Google OAuth token
#[utoipa::path(
    post,
    path = "/api/v1/googleLogin",
    request_body = GoogleLoginRequest,
    responses(
        (status = 200, description = "Google login successful", body = ApiResponseGoogleLogin),
        (status = 500, description = "Google login failed", body = ApiResponseGoogleLogin)
    )
)]
pub async fn google_login(
    State(state): State<AppState>,
    Json(payload): Json<GoogleLoginRequest>,
) -> Json<ApiResponse<GoogleLoginData>> {
    match state.auth_service.google_login(&payload.google_token).await {
        Ok(result) => {
            let account = result.identity_account;
            ApiResponse::success(GoogleLoginData {
                identity_account_id: account.identity_account_id,
                account_type: account.account_type.to_string(),
                username: account.username,
                external_subject_id: account
                    .external_subject_id
                    .unwrap_or_default(),
                status: account.status,
                last_login_at: account
                    .last_login_at
                    .unwrap_or_default(),
                jwt_access_token: result.jwt_access_token,
                refresh_token: result.refresh_token,
            })
        }
        Err(e) => ApiResponse::failure(500, e),
    }
}

