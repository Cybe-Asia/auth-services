use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Router, Server};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

mod clients {
    pub mod google_oauth_client;
}
mod config;
mod database {
    pub mod neo4j;
}
mod dto {
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

    let app: Router = routes::auth_routes::auth_routes()
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
