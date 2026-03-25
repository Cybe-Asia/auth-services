use neo4rs::{query, Graph, Node};
use uuid::Uuid;

use crate::models::user_model::User;

fn user_from_node(node: Node) -> Result<User, String> {
    let id_str: String = node
        .get("id")
        .ok_or_else(|| "neo4j node missing id".to_string())?;
    let email: String = node
        .get("email")
        .ok_or_else(|| "neo4j node missing email".to_string())?;

    let password_hash: Option<String> = node.get("passwordHash");

    let id = Uuid::parse_str(&id_str).map_err(|_| "invalid uuid stored in neo4j".to_string())?;
    Ok(User {
        id,
        email,
        password_hash,
    })
}

pub async fn find_user_by_email(graph: &Graph, email: &str) -> Result<Option<User>, String> {
    let mut result = graph
        .execute(query("MATCH (u:User {email:$email}) RETURN u").param("email", email))
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
        .get("u")
        .ok_or_else(|| "neo4j row missing u".to_string())?;
    Ok(Some(user_from_node(node)?))
}

pub async fn create_user_with_password(
    graph: &Graph,
    email: &str,
    password_hash: &str,
) -> Result<User, String> {
    let id = Uuid::new_v4();
    let mut result = graph
        .execute(
            query(
                "CREATE (u:User { id: $id, email: $email, passwordHash: $passwordHash, createdAt: datetime() }) RETURN u",
            )
            .param("id", id.to_string())
            .param("email", email)
            .param("passwordHash", password_hash),
        )
        .await
        .map_err(|e| format!("neo4j create user failed: {e}"))?;

    let row = result
        .next()
        .await
        .map_err(|e| format!("neo4j result read failed: {e}"))?
        .ok_or_else(|| "neo4j create user returned no rows".to_string())?;

    let node: Node = row
        .get("u")
        .ok_or_else(|| "neo4j row missing u".to_string())?;
    user_from_node(node)
}

pub async fn create_user_google(graph: &Graph, email: &str) -> Result<User, String> {
    let id = Uuid::new_v4();
    let mut result = graph
        .execute(
            query("CREATE (u:User { id: $id, email: $email, createdAt: datetime() }) RETURN u")
                .param("id", id.to_string())
                .param("email", email),
        )
        .await
        .map_err(|e| format!("neo4j create user failed: {e}"))?;

    let row = result
        .next()
        .await
        .map_err(|e| format!("neo4j result read failed: {e}"))?
        .ok_or_else(|| "neo4j create user returned no rows".to_string())?;

    let node: Node = row
        .get("u")
        .ok_or_else(|| "neo4j row missing u".to_string())?;
    user_from_node(node)
}
