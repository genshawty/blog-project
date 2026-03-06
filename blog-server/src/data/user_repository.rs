use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
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

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    email: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: row.id,
            username: row.username,
            email: row.email,
            password_hash: row.password_hash,
            created_at: row.created_at,
        }
    }
}

#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: User) -> Result<User, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            "INSERT INTO users (id, username, email, password_hash)
             VALUES ($1, $2, $3, $4)
             RETURNING id, username, email, password_hash, created_at",
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                DomainError::UserAlreadyExists
            }
            _ => DomainError::Internal(e.to_string()),
        })?;

        Ok(row.into())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(User::from))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(User::from))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(User::from))
    }
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
