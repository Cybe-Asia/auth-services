use neo4rs::Graph;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    clients::google_oauth_client::GoogleOAuthClient,
    models::identity_account_model::IdentityAccount,
    repositories::{identity_account_repository, user_repository},
    utils::{jwt, password},
};

#[derive(Clone)]
pub struct AuthService {
    graph: Arc<Graph>,
    jwt_secret: String,
    jwt_session_expiry: u64,
    google_client: GoogleOAuthClient,
}

pub struct LoginTokens {
    pub jwt_access_token: String,
    pub refresh_token: String,
}

pub struct CreateAccountTokens {
    pub email: String,
    pub jwt_session_token: String,
}

pub enum CreateAccountResult {
    Created(CreateAccountTokens),
    AlreadyExists,
}

pub struct GoogleLoginResult {
    pub identity_account: IdentityAccount,
    pub jwt_access_token: String,
    pub refresh_token: String,
}

impl AuthService {
    pub fn new(
        graph: Arc<Graph>,
        jwt_secret: String,
        jwt_session_expiry: u64,
        google_client: GoogleOAuthClient,
    ) -> Self {
        Self {
            graph,
            jwt_secret,
            jwt_session_expiry,
            google_client,
        }
    }

    pub async fn create_account(&self, email: &str) -> Result<CreateAccountResult, String> {
        info!(email = %email, "create account attempt");

        let existing = user_repository::find_user_by_email(self.graph.as_ref(), email).await?;
        if existing.is_some() {
            return Ok(CreateAccountResult::AlreadyExists);
        }

        let user = user_repository::create_account(self.graph.as_ref(), email).await?;

        let jwt_session_token = jwt::generate_access_token(
            &self.jwt_secret,
            &user.id.to_string(),
            &user.email,
            self.jwt_session_expiry,
        )?;

        Ok(CreateAccountResult::Created(CreateAccountTokens {
            email: user.email,
            jwt_session_token,
        }))
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

        let jwt_access_token = jwt::generate_access_token(
            &self.jwt_secret,
            &user.id.to_string(),
            &user.email,
            3600,
        )?;
        let refresh_token = Uuid::new_v4().to_string();

        info!(username = %username, "login success");

        Ok(LoginTokens {
            jwt_access_token,
            refresh_token,
        })
    }

    pub async fn google_login(&self, google_token: &str) -> Result<GoogleLoginResult, String> {
        info!("google login validation");

        // 1. Validate the Google ID token
        let info = self.google_client.validate_id_token(google_token).await?;

        let email = info
            .email
            .ok_or_else(|| "Google token response missing email".to_string())?;

        let sub = info
            .sub
            .ok_or_else(|| "Google token response missing sub".to_string())?;

        let external_subject_id = format!("google-oauth2|{sub}");

        // 2. Find or create IdentityAccount
        let account = match identity_account_repository::find_by_external_subject_id(
            self.graph.as_ref(),
            &external_subject_id,
        )
        .await?
        {
            Some(existing) => {
                // Update last_login_at
                identity_account_repository::update_last_login(
                    self.graph.as_ref(),
                    &existing.identity_account_id,
                )
                .await?
            }
            None => {
                // Create new Google IdentityAccount
                identity_account_repository::create_google_account(
                    self.graph.as_ref(),
                    &email,
                    &external_subject_id,
                )
                .await?
            }
        };

        // 3. Generate JWT with identity_account_id as subject
        let jwt_access_token = jwt::generate_access_token(
            &self.jwt_secret,
            &account.identity_account_id,
            &account.username,
            3600,
        )?;
        let refresh_token = Uuid::new_v4().to_string();

        info!(email = %email, identity_account_id = %account.identity_account_id, "google login success");

        Ok(GoogleLoginResult {
            identity_account: account,
            jwt_access_token,
            refresh_token,
        })
    }
}

