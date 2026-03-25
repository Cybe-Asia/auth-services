use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub server_port: u16,
    pub neo4j_uri: String,
    pub neo4j_user: String,
    pub neo4j_password: String,
    pub jwt_secret: String,
    pub google_tokeninfo_endpoint: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "local".to_string());
        let env_file = format!(".env.{app_env}");
        dotenv::from_filename(&env_file).ok();
        dotenv::dotenv().ok();

        let server_port = env::var("SERVER_PORT")
            .map_err(|_| "SERVER_PORT is required".to_string())?
            .parse::<u16>()
            .map_err(|_| "SERVER_PORT must be a valid u16".to_string())?;

        Ok(Self {
            server_port,
            neo4j_uri: env::var("NEO4J_URI").map_err(|_| "NEO4J_URI is required".to_string())?,
            neo4j_user: env::var("NEO4J_USER").map_err(|_| "NEO4J_USER is required".to_string())?,
            neo4j_password: env::var("NEO4J_PASSWORD")
                .map_err(|_| "NEO4J_PASSWORD is required".to_string())?,
            jwt_secret: env::var("JWT_SECRET").map_err(|_| "JWT_SECRET is required".to_string())?,
            google_tokeninfo_endpoint: env::var("GOOGLE_TOKENINFO_ENDPOINT")
                .map_err(|_| "GOOGLE_TOKENINFO_ENDPOINT is required".to_string())?,
        })
    }
}
