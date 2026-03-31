use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: &str) -> Result<String, String> {
    hash(password, DEFAULT_COST).map_err(|e| format!("password hashing failed: {e}"))
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, String> {
    verify(password, password_hash).map_err(|e| format!("password verification failed: {e}"))
}
