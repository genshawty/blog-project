use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

use crate::data::user_repository::UserRepository;
use crate::domain::errors::BlogError;
use crate::domain::user::{User, verify_password};
use crate::infrastructure::jwt::JwtKeys;

#[derive(Clone)]
pub struct AuthService {
    repo: Arc<dyn UserRepository>,
    keys: JwtKeys,
}

impl AuthService {
    pub fn new(repo: Arc<dyn UserRepository>, keys: JwtKeys) -> Self {
        Self { repo, keys }
    }

    pub fn keys(&self) -> &JwtKeys {
        &self.keys
    }

    #[instrument(skip(self))]
    pub async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<(String, User), BlogError> {
        let user = User::new(username, email, password)
            .map_err(|e| BlogError::Validation(e.to_string()))?;
        let user = self.repo.create(user).await.map_err(|e| {
            if matches!(e, crate::domain::DomainError::UserAlreadyExists) {
                BlogError::UserAlreadyExists
            } else {
                BlogError::Internal(e.to_string())
            }
        })?;
        let token = self
            .keys
            .generate_token(user.id, user.username.clone())
            .map_err(|e| BlogError::Internal(e.to_string()))?;
        Ok((token, user))
    }

    #[instrument(skip(self))]
    pub async fn login(&self, username: &str, password: &str) -> Result<(String, User), BlogError> {
        let user = self
            .repo
            .find_by_username(username)
            .await
            .map_err(|e| BlogError::Internal(e.to_string()))?
            .ok_or(BlogError::Unauthorized)?;

        let is_valid =
            verify_password(password, &user.password_hash).map_err(|_| BlogError::Unauthorized)?;
        if !is_valid {
            return Err(BlogError::Unauthorized);
        }

        let token = self
            .keys
            .generate_token(user.id, user.username.clone())
            .map_err(|e| BlogError::Internal(e.to_string()))?;
        Ok((token, user))
    }

    #[instrument(skip(self))]
    pub async fn get_user(&self, user_id: Uuid) -> Result<User, BlogError> {
        self.repo
            .find_by_id(user_id)
            .await
            .map_err(|e| BlogError::Internal(e.to_string()))?
            .ok_or(BlogError::UserNotFound(user_id))
    }
}
