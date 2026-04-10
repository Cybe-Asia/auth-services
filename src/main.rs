use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Router, Server};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod clients {
    pub mod google_oauth_client;
}
mod config;
mod database {
    pub mod neo4j;
}
mod dto {
    pub mod create_account_request;
    pub mod create_password_request;
    pub mod google_login_request;
    pub mod login_request;
}
mod handlers {
    pub mod auth_handler;
}
mod models {
    pub mod user_model;
}
mod repositories {
    pub mod user_repository;
}
mod routes {
    pub mod auth_routes;
}
mod services {
    pub mod auth_service;
}
mod utils {
    pub mod jwt;
    pub mod password;
    pub mod response;
}

#[derive(Clone)]
pub struct AppState {
    pub auth_service: services::auth_service::AuthService,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth_handler::create_account,
        handlers::auth_handler::create_password,
        handlers::auth_handler::login,
        handlers::auth_handler::google_login,
    ),
    components(
        schemas(
            dto::create_account_request::CreateAccountRequest,
            dto::create_password_request::CreatePasswordRequest,
            dto::login_request::LoginRequest,
            dto::google_login_request::GoogleLoginRequest,
            handlers::auth_handler::CreateAccountData,
            handlers::auth_handler::CreatePasswordData,
            handlers::auth_handler::LoginData,
            handlers::auth_handler::GoogleLoginData,
            utils::response::ApiResponseCreateAccount,
            utils::response::ApiResponseCreatePassword,
            utils::response::ApiResponseLogin,
            utils::response::ApiResponseGoogleLogin,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication and account management")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "auth_service=info,tower_http=info".into()),
        )
        .json()
        .init();

    let config = config::AppConfig::from_env().expect("invalid configuration");

    let graph = database::neo4j::connect(
        &config.neo4j_uri,
        &config.neo4j_user,
        &config.neo4j_password,
    )
    .await
    .expect("failed to connect to neo4j");
    let graph = Arc::new(graph);

    let http = reqwest::Client::new();
    let google_client = clients::google_oauth_client::GoogleOAuthClient::new(
        http,
        config.google_tokeninfo_endpoint.clone(),
    );

    let auth_service =
        services::auth_service::AuthService::new(graph, config.jwt_secret.clone(), google_client);
    let state = AppState { auth_service };

    let app = Router::new()
        .merge(routes::auth_routes::auth_routes())
        .merge(SwaggerUi::new("/api/v1/auth-service/swagger-ui").url("/api/v1/auth-service/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any),
        )
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    info!(%addr, "auth-service started");

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("server error");
}
