use axum::{routing::{get, post}, Router};

use crate::{handlers::auth_handler, AppState};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/auth-service/createAccount", post(auth_handler::create_account))
        .route("/api/v1/auth-service/createPassword", post(auth_handler::create_password))
        .route("/api/v1/auth-service/login", post(auth_handler::login))
        .route("/api/v1/auth-service/accountStatus", get(auth_handler::account_status))
        .route("/api/v1/googleLogin", post(auth_handler::google_login))
}
