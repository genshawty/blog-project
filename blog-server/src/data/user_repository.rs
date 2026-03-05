use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{DomainError, user::User};

#[derive(Default, Clone)]
pub struct UserRepository {
    pub users: Arc<RwLock<HashMap<Uuid, User>>>,
    pub emails: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl UserRepository {
    async fn create(&self, user: User) -> Result<User, DomainError> {
        let mut users = self.users.write().await;
        let mut emails = self.emails.write().await;

        if emails.contains_key(&user.email) {
            return Err(DomainError::Validation("email already registered".into()));
        }

        emails.insert(user.email.clone(), user.id);
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

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let users = self.users.read().await;
        Ok(users.get(&id).cloned())
    }
}
