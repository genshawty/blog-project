use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::errors::UserError;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, email: String, password: String) -> Result<Self, UserError> {
        if !email.contains("@") {
            return Err(UserError::WrongEmailFormat);
        }
        let password_hash = hash_password(&password).map_err(|_| UserError::InternalError)?;
        let id = Uuid::new_v4();
        let created_at = Utc::now();
        Ok(Self {
            id,
            username,
            email,
            password_hash,
            created_at,
        })
    }
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed).is_ok())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify() {
        let password = "my_secret_password";
        let hash = hash_password(password).expect("hashing failed");
        assert!(verify_password(password, &hash).expect("verify failed"));
    }

    #[test]
    fn wrong_password_fails() {
        let hash = hash_password("correct_password").expect("hashing failed");
        assert!(!verify_password("wrong_password", &hash).expect("verify failed"));
    }
}
