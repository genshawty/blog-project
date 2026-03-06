use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{DomainError, Post};

// Nested map: author_id -> post_id -> Post
#[derive(Default, Clone)]
pub struct PostRepository {
    posts: Arc<RwLock<HashMap<Uuid, HashMap<Uuid, Post>>>>,
}

impl PostRepository {
    pub async fn create(&self, post: Post) -> Result<(), DomainError> {
        let mut posts = self.posts.write().await;
        let author_posts = posts.entry(post.author_id).or_default();
        if author_posts.contains_key(&post.id) {
            return Err(DomainError::Validation("post already exists".into()));
        }
        author_posts.insert(post.id, post);
        Ok(())
    }

    pub async fn get(&self, author_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        let posts = self.posts.read().await;
        posts
            .get(&author_id)
            .and_then(|author_posts| author_posts.get(&post_id))
            .cloned()
            .ok_or(DomainError::PostNotFound(post_id))
    }

    pub async fn update(&self, post: Post) -> Result<(), DomainError> {
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

    pub async fn delete(&self, author_id: Uuid, post_id: Uuid) -> Result<(), DomainError> {
        let mut posts = self.posts.write().await;
        let author_posts = posts
            .get_mut(&author_id)
            .ok_or(DomainError::PostNotFound(post_id))?;
        author_posts
            .remove(&post_id)
            .ok_or(DomainError::PostNotFound(post_id))?;
        Ok(())
    }

    pub async fn find_by_id(&self, post_id: Uuid) -> Result<Post, DomainError> {
        let posts = self.posts.read().await;
        for author_posts in posts.values() {
            if let Some(post) = author_posts.get(&post_id) {
                return Ok(post.clone());
            }
        }
        Err(DomainError::PostNotFound(post_id))
    }

    pub async fn list_all(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), DomainError> {
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

    pub async fn list_for_author(&self, author_id: Uuid) -> Result<Vec<Post>, DomainError> {
        let posts = self.posts.read().await;
        Ok(posts
            .get(&author_id)
            .map(|author_posts| author_posts.values().cloned().collect())
            .unwrap_or_default())
    }
}
