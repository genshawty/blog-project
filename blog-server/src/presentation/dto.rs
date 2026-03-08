use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Post, User};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthUserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: AuthUserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            author_id: post.author_id,
            title: post.title,
            content: post.content,
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PostListResponse {
    pub posts: Vec<PostResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub timestamp: DateTime<Utc>,
}
