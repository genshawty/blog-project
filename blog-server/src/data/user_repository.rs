use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{DomainError, user::User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: User) -> Result<User, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError>;
}

#[derive(Default, Clone)]
pub struct InMemoryUserRepository {
    pub users: Arc<RwLock<HashMap<Uuid, User>>>,
    pub emails: Arc<RwLock<HashMap<String, Uuid>>>,
    pub usernames: Arc<RwLock<HashMap<String, Uuid>>>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn create(&self, user: User) -> Result<User, DomainError> {
        let mut users = self.users.write().await;
        let mut emails = self.emails.write().await;
        let mut usernames = self.usernames.write().await;

        if emails.contains_key(&user.email) || usernames.contains_key(&user.username) {
            return Err(DomainError::UserAlreadyExists);
        }

        emails.insert(user.email.clone(), user.id);
        usernames.insert(user.username.clone(), user.id);
        users.insert(user.id, user.clone());
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let emails = self.emails.read().await;
        let users = self.users.read().await;

        if let Some(id) = emails.get(email) {
            Ok(users.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        let usernames = self.usernames.read().await;
        let users = self.users.read().await;

        if let Some(id) = usernames.get(username) {
            Ok(users.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let users = self.users.read().await;
        Ok(users.get(&id).cloned())
    }
}
