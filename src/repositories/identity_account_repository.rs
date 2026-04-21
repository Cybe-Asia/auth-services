use neo4rs::{query, Graph, Node};
use uuid::Uuid;

use crate::models::identity_account_model::{AccountType, IdentityAccount};

fn identity_account_from_node(node: Node) -> Result<IdentityAccount, String> {
    let identity_account_id: String = node
        .get("identity_account_id")
        .ok_or_else(|| "neo4j node missing identity_account_id".to_string())?;

    let account_type_str: String = node
        .get("account_type")
        .ok_or_else(|| "neo4j node missing account_type".to_string())?;
    let account_type = AccountType::from_str(&account_type_str)?;

    let username: String = node
        .get("username")
        .ok_or_else(|| "neo4j node missing username".to_string())?;

    let external_subject_id: Option<String> = node.get("external_subject_id");
    let status: String = node
        .get("status")
        .unwrap_or_else(|| "active".to_string());
    let last_login_at: Option<String> = node.get("last_login_at");

    Ok(IdentityAccount {
        identity_account_id,
        account_type,
        username,
        external_subject_id,
        status,
        last_login_at,
    })
}

/// Find an IdentityAccount by its external subject ID (e.g. "google-oauth2|123456")
pub async fn find_by_external_subject_id(
    graph: &Graph,
    external_subject_id: &str,
) -> Result<Option<IdentityAccount>, String> {
    let mut result = graph
        .execute(
            query("MATCH (ia:IdentityAccount {external_subject_id: $ext_id}) RETURN ia")
                .param("ext_id", external_subject_id),
        )
        .await
        .map_err(|e| format!("neo4j query failed: {e}"))?;

    let row = result
        .next()
        .await
        .map_err(|e| format!("neo4j result read failed: {e}"))?;

    let Some(row) = row else {
        return Ok(None);
    };

    let node: Node = row
        .get("ia")
        .ok_or_else(|| "neo4j row missing ia".to_string())?;
    Ok(Some(identity_account_from_node(node)?))
}

/// Find an IdentityAccount by username (email) and account type
pub async fn find_by_username_and_type(
    graph: &Graph,
    username: &str,
    account_type: &AccountType,
) -> Result<Option<IdentityAccount>, String> {
    let mut result = graph
        .execute(
            query(
                "MATCH (ia:IdentityAccount {username: $username, account_type: $account_type}) RETURN ia",
            )
            .param("username", username)
            .param("account_type", account_type.to_string()),
        )
        .await
        .map_err(|e| format!("neo4j query failed: {e}"))?;

    let row = result
        .next()
        .await
        .map_err(|e| format!("neo4j result read failed: {e}"))?;

    let Some(row) = row else {
        return Ok(None);
    };

    let node: Node = row
        .get("ia")
        .ok_or_else(|| "neo4j row missing ia".to_string())?;
    Ok(Some(identity_account_from_node(node)?))
}

/// Create a new Google-type IdentityAccount
pub async fn create_google_account(
    graph: &Graph,
    username: &str,
    external_subject_id: &str,
) -> Result<IdentityAccount, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let mut result = graph
        .execute(
            query(
                "CREATE (ia:IdentityAccount {
                    identity_account_id: $id,
                    account_type: $account_type,
                    username: $username,
                    external_subject_id: $ext_id,
                    status: $status,
                    last_login_at: $now,
                    created_at: $now
                }) RETURN ia",
            )
            .param("id", id.as_str())
            .param("account_type", "google")
            .param("username", username)
            .param("ext_id", external_subject_id)
            .param("status", "active")
            .param("now", now.as_str()),
        )
        .await
        .map_err(|e| format!("neo4j create identity account failed: {e}"))?;

    let row = result
        .next()
        .await
        .map_err(|e| format!("neo4j result read failed: {e}"))?
        .ok_or_else(|| "neo4j create identity account returned no rows".to_string())?;

    let node: Node = row
        .get("ia")
        .ok_or_else(|| "neo4j row decode failed: missing field 'ia'".to_string())?;
    identity_account_from_node(node)
}

/// Update the last_login_at timestamp for an IdentityAccount
pub async fn update_last_login(
    graph: &Graph,
    identity_account_id: &str,
) -> Result<IdentityAccount, String> {
    let now = chrono::Utc::now().to_rfc3339();

    let mut result = graph
        .execute(
            query(
                "MATCH (ia:IdentityAccount {identity_account_id: $id})
                 SET ia.last_login_at = $now
                 RETURN ia",
            )
            .param("id", identity_account_id)
            .param("now", now.as_str()),
        )
        .await
        .map_err(|e| format!("neo4j update last_login failed: {e}"))?;

    let row = result
        .next()
        .await
        .map_err(|e| format!("neo4j result read failed: {e}"))?
        .ok_or_else(|| "IdentityAccount not found".to_string())?;

    let node: Node = row
        .get("ia")
        .ok_or_else(|| "neo4j row decode failed: missing field 'ia'".to_string())?;
    identity_account_from_node(node)
}
