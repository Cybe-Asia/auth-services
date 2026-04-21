use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountType {
    #[serde(rename = "google")]
    Google,
    #[serde(rename = "basic_portal")]
    BasicPortal,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountType::Google => write!(f, "google"),
            AccountType::BasicPortal => write!(f, "basic_portal"),
        }
    }
}

impl AccountType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "google" => Ok(AccountType::Google),
            "basic_portal" => Ok(AccountType::BasicPortal),
            other => Err(format!("unknown account_type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityAccount {
    pub identity_account_id: String,
    pub account_type: AccountType,
    pub username: String,
    pub external_subject_id: Option<String>,
    pub status: String,
    pub last_login_at: Option<String>,
}
