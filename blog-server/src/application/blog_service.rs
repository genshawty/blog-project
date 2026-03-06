use tracing::instrument;
use uuid::Uuid;

use crate::data::post_repository::PostRepository;
use crate::domain::{DomainError, Post};

#[derive(Clone)]
pub struct BlogService {
    repo: PostRepository,
}

impl BlogService {
    pub fn new(repo: PostRepository) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn create_post(
        &self,
        title: String,
        content: String,
        author_id: Uuid,
    ) -> Result<Post, DomainError> {
        let post = Post::new(title, content, author_id)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        self.repo.create(post.clone()).await?;
        Ok(post)
    }

    #[instrument(skip(self))]
    pub async fn get_post(&self, post_id: Uuid) -> Result<Post, DomainError> {
        self.repo.find_by_id(post_id).await
    }

    #[instrument(skip(self))]
    pub async fn update_post(
        &self,
        author_id: Uuid,
        post_id: Uuid,
        new_title: String,
        new_content: String,
    ) -> Result<Post, DomainError> {
        let mut post = self.repo.find_by_id(post_id).await?;
        if post.author_id != author_id {
            return Err(DomainError::Forbidden);
        }
        post.update(new_title, new_content);
        self.repo.update(post.clone()).await?;
        Ok(post)
    }

    #[instrument(skip(self))]
    pub async fn delete_post(&self, author_id: Uuid, post_id: Uuid) -> Result<(), DomainError> {
        let post = self.repo.find_by_id(post_id).await?;
        if post.author_id != author_id {
            return Err(DomainError::Forbidden);
        }
        self.repo.delete(post.author_id, post_id).await
    }

    #[instrument(skip(self))]
    pub async fn list_posts(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), DomainError> {
        self.repo.list_all(limit, offset).await
    }
}
