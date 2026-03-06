use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{DomainError, Post};

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, post: Post) -> Result<(), DomainError>;
    async fn find_by_id(&self, post_id: Uuid) -> Result<Post, DomainError>;
    async fn update(&self, post: Post) -> Result<(), DomainError>;
    async fn delete(&self, author_id: Uuid, post_id: Uuid) -> Result<(), DomainError>;
    async fn list_all(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), DomainError>;
}

#[derive(sqlx::FromRow)]
struct PostRow {
    id: Uuid,
    title: String,
    content: String,
    author_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
}

impl From<PostRow> for Post {
    fn from(row: PostRow) -> Self {
        Post {
            id: row.id,
            title: row.title,
            content: row.content,
            author_id: row.author_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(&self, post: Post) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO posts (id, title, content, author_id)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(post.id)
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.author_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, post_id: Uuid) -> Result<Post, DomainError> {
        let row = sqlx::query_as::<_, PostRow>(
            "SELECT id, title, content, author_id, created_at, updated_at
             FROM posts WHERE id = $1",
        )
        .bind(post_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or(DomainError::PostNotFound(post_id))?;

        Ok(row.into())
    }

    async fn update(&self, post: Post) -> Result<(), DomainError> {
        let result = sqlx::query(
            "UPDATE posts SET title = $1, content = $2, updated_at = $3
             WHERE id = $4 AND author_id = $5",
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.updated_at)
        .bind(post.id)
        .bind(post.author_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::PostNotFound(post.id));
        }

        Ok(())
    }

    async fn delete(&self, author_id: Uuid, post_id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query(
            "DELETE FROM posts WHERE id = $1 AND author_id = $2",
        )
        .bind(post_id)
        .bind(author_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::PostNotFound(post_id));
        }

        Ok(())
    }

    async fn list_all(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), DomainError> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let rows = sqlx::query_as::<_, PostRow>(
            "SELECT id, title, content, author_id, created_at, updated_at
             FROM posts ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let posts = rows.into_iter().map(Post::from).collect();
        Ok((posts, total))
    }
}

// Nested map: author_id -> post_id -> Post
#[derive(Default, Clone)]
pub struct InMemoryPostRepository {
    posts: Arc<RwLock<HashMap<Uuid, HashMap<Uuid, Post>>>>,
}

#[async_trait]
impl PostRepository for InMemoryPostRepository {
    async fn create(&self, post: Post) -> Result<(), DomainError> {
        let mut posts = self.posts.write().await;
        let author_posts = posts.entry(post.author_id).or_default();
        if author_posts.contains_key(&post.id) {
            return Err(DomainError::Validation("post already exists".into()));
        }
        author_posts.insert(post.id, post);
        Ok(())
    }

    async fn find_by_id(&self, post_id: Uuid) -> Result<Post, DomainError> {
        let posts = self.posts.read().await;
        for author_posts in posts.values() {
            if let Some(post) = author_posts.get(&post_id) {
                return Ok(post.clone());
            }
        }
        Err(DomainError::PostNotFound(post_id))
    }

    async fn update(&self, post: Post) -> Result<(), DomainError> {
        let mut posts = self.posts.write().await;
        let author_posts = posts
            .get_mut(&post.author_id)
            .ok_or(DomainError::PostNotFound(post.id))?;
        if !author_posts.contains_key(&post.id) {
            return Err(DomainError::PostNotFound(post.id));
        }
        author_posts.insert(post.id, post);
        Ok(())
    }

    async fn delete(&self, author_id: Uuid, post_id: Uuid) -> Result<(), DomainError> {
        let mut posts = self.posts.write().await;
        let author_posts = posts
            .get_mut(&author_id)
            .ok_or(DomainError::PostNotFound(post_id))?;
        author_posts
            .remove(&post_id)
            .ok_or(DomainError::PostNotFound(post_id))?;
        Ok(())
    }

    async fn list_all(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), DomainError> {
        let posts = self.posts.read().await;
        let mut all_posts: Vec<Post> = posts
            .values()
            .flat_map(|author_posts| author_posts.values().cloned())
            .collect();
        all_posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let total = all_posts.len() as i64;
        let result = all_posts
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();
        Ok((result, total))
    }
}
