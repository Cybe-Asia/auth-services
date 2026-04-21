use neo4rs::Graph;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    clients::google_oauth_client::GoogleOAuthClient,
    repositories::user_repository,
    utils::{jwt, password},
};

#[derive(Clone)]
pub struct AuthService {
    graph: Arc<Graph>,
    jwt_secret: String,
    google_client: GoogleOAuthClient,
}

pub struct LoginTokens {
    pub jwt_access_token: String,
    pub refresh_token: String,
}

pub enum CreateAccountResult {
    Created,
    AlreadyExists,
}

impl AuthService {
    pub fn new(graph: Arc<Graph>, jwt_secret: String, google_client: GoogleOAuthClient) -> Self {
        Self {
            graph,
            jwt_secret,
            google_client,
        }
    }

    pub async fn account_status(&self, email: &str) -> Result<(bool, bool), String> {
        let user = user_repository::find_user_by_email(self.graph.as_ref(), email).await?;
        match user {
            None => Ok((false, false)),
            Some(u) => Ok((true, u.password_hash.is_some())),
        }
    }

    pub async fn create_account(&self, email: &str) -> Result<CreateAccountResult, String> {
        info!(email = %email, "create account attempt");

        let existing = user_repository::find_user_by_email(self.graph.as_ref(), email).await?;
        if existing.is_some() {
            return Ok(CreateAccountResult::AlreadyExists);
        }

        user_repository::create_account(self.graph.as_ref(), email).await?;
        Ok(CreateAccountResult::Created)
    }

    pub async fn create_password(
        &self,
        bearer_token: &str,
        new_password: &str,
    ) -> Result<bool, String> {
        let claims = jwt::verify_access_token(&self.jwt_secret, bearer_token)?;
        let email = claims.email;

        info!(email = %email, "password creation");

        let password_hash = password::hash_password(new_password)?;
        user_repository::update_password(self.graph.as_ref(), &email, &password_hash).await?;
        Ok(true)
    }

    pub async fn login(&self, username: &str, password_plain: &str) -> Result<LoginTokens, String> {
        info!(username = %username, "login attempt");

        let user = user_repository::find_user_by_email(self.graph.as_ref(), username)
            .await?
            .ok_or_else(|| "Invalid credentials".to_string())?;

        let Some(hash) = user.password_hash.as_deref() else {
            warn!(username = %username, "login failure (no password set)");
            return Err("Invalid credentials".to_string());
        };

        let ok = password::verify_password(password_plain, hash)?;
        if !ok {
            warn!(username = %username, "login failure");
            return Err("Invalid credentials".to_string());
        }

        let jwt_access_token =
            jwt::generate_access_token(&self.jwt_secret, &user.id.to_string(), &user.email)?;
        let refresh_token = Uuid::new_v4().to_string();

        info!(username = %username, "login success");

        Ok(LoginTokens {
            jwt_access_token,
            refresh_token,
        })
    }

    pub async fn google_login(&self, google_token: &str) -> Result<String, String> {
        info!("google login validation");

        let info = self.google_client.validate_id_token(google_token).await?;
        let email = info
            .email
            .ok_or_else(|| "Google token response missing email".to_string())?;

        let user = match user_repository::find_user_by_email(self.graph.as_ref(), &email).await? {
            Some(u) => u,
            None => user_repository::create_account(self.graph.as_ref(), &email).await?,
        };

        let jwt_access_token =
            jwt::generate_access_token(&self.jwt_secret, &user.id.to_string(), &user.email)?;

        info!(email = %email, "google login success");

        Ok(jwt_access_token)
    }
}
