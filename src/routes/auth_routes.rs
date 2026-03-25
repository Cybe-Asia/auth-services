use axum::{routing::post, Router};

use crate::{handlers::auth_handler, AppState};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/createPassword",
            post(auth_handler::create_password),
        )
        .route("/api/v1/login", post(auth_handler::login))
        .route("/api/v1/googleLogin", post(auth_handler::google_login))
}
