use async_trait::async_trait;
use reqwest::Client;

use crate::error::BlogClientError;
use crate::types::*;
use crate::BlogApi;

pub struct BlogHttpClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl BlogHttpClient {
    pub fn new(addr: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: addr.trim_end_matches('/').to_string(),
            token: None,
        }
    }

    fn auth_header(&self) -> Result<String, BlogClientError> {
        self.token
            .as_ref()
            .map(|t| format!("Bearer {}", t))
            .ok_or(BlogClientError::Unauthorized)
    }
}

#[async_trait]
impl BlogApi for BlogHttpClient {
    fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let resp = self
            .client
            .post(format!("{}/api/auth/register", self.base_url))
            .json(&RegisterRequest {
                username: username.to_string(),
                email: email.to_string(),
                password: password.to_string(),
            })
            .send()
            .await?;

        match resp.status().as_u16() {
            409 => return Err(BlogClientError::UserAlreadyExists),
            201 => {
                let auth: AuthResponse = resp.json().await?;
                self.token = Some(auth.token.clone());
                Ok(auth)
            }
            status => Err(BlogClientError::Internal(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let resp = self
            .client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&LoginRequest {
                username: username.to_string(),
                password: password.to_string(),
            })
            .send()
            .await?;

        match resp.status().as_u16() {
            401 => return Err(BlogClientError::Unauthorized),
            200 => {
                let auth: AuthResponse = resp.json().await?;
                self.token = Some(auth.token.clone());
                Ok(auth)
            }
            status => Err(BlogClientError::Internal(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn create_post(
        &self,
        title: &str,
        content: &str,
    ) -> Result<PostResponse, BlogClientError> {
        let resp = self
            .client
            .post(format!("{}/api/posts", self.base_url))
            .header("Authorization", self.auth_header()?)
            .json(&CreatePostRequest {
                title: title.to_string(),
                content: content.to_string(),
            })
            .send()
            .await?;

        match resp.status().as_u16() {
            201 => Ok(resp.json().await?),
            401 => Err(BlogClientError::Unauthorized),
            status => Err(BlogClientError::Internal(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn get_post(&self, post_id: &str) -> Result<PostResponse, BlogClientError> {
        let resp = self
            .client
            .get(format!("{}/api/posts/{}", self.base_url, post_id))
            .send()
            .await?;

        match resp.status().as_u16() {
            200 => Ok(resp.json().await?),
            404 => Err(BlogClientError::NotFound),
            status => Err(BlogClientError::Internal(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn update_post(
        &self,
        post_id: &str,
        title: &str,
        content: &str,
    ) -> Result<PostResponse, BlogClientError> {
        let resp = self
            .client
            .put(format!("{}/api/posts/{}", self.base_url, post_id))
            .header("Authorization", self.auth_header()?)
            .json(&UpdatePostRequest {
                title: title.to_string(),
                content: content.to_string(),
            })
            .send()
            .await?;

        match resp.status().as_u16() {
            200 => Ok(resp.json().await?),
            401 => Err(BlogClientError::Unauthorized),
            403 => Err(BlogClientError::Forbidden),
            404 => Err(BlogClientError::NotFound),
            status => Err(BlogClientError::Internal(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn delete_post(&self, post_id: &str) -> Result<(), BlogClientError> {
        let resp = self
            .client
            .delete(format!("{}/api/posts/{}", self.base_url, post_id))
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        match resp.status().as_u16() {
            204 => Ok(()),
            401 => Err(BlogClientError::Unauthorized),
            403 => Err(BlogClientError::Forbidden),
            404 => Err(BlogClientError::NotFound),
            status => Err(BlogClientError::Internal(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<PostListResponse, BlogClientError> {
        let resp = self
            .client
            .get(format!("{}/api/posts", self.base_url))
            .query(&[("limit", limit), ("offset", offset)])
            .send()
            .await?;

        match resp.status().as_u16() {
            200 => Ok(resp.json().await?),
            status => Err(BlogClientError::Internal(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }
}
