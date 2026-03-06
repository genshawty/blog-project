pub mod blog {
    tonic::include_proto!("blog");
}

pub mod error;
pub mod grpc_client;
pub mod http_client;

use grpc_client::BlogGrpcClient;
use http_client::BlogHttpClient;

use async_trait::async_trait;
use error::BlogClientError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUserInfo {
    pub username: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: AuthUserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostResponse {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostListResponse {
    pub posts: Vec<PostResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[async_trait]
pub trait BlogApi {
    async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError>;

    async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError>;

    async fn create_post(
        &self,
        title: &str,
        content: &str,
    ) -> Result<PostResponse, BlogClientError>;

    async fn get_post(&self, post_id: &str) -> Result<PostResponse, BlogClientError>;

    async fn update_post(
        &self,
        post_id: &str,
        title: &str,
        content: &str,
    ) -> Result<PostResponse, BlogClientError>;

    async fn delete_post(&self, post_id: &str) -> Result<(), BlogClientError>;

    async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<PostListResponse, BlogClientError>;
}

#[derive(strum_macros::EnumString)]
pub enum Transport {
    Http,
    Grpc,
}

pub struct BlogClient<R: BlogApi + 'static> {
    pub transport: Transport,
    pub client: R,
    pub token: Option<String>,
}

impl<R> BlogClient<R>
where
    R: BlogApi + 'static,
{
    pub fn new(transport: Transport, addr: &str) -> Self {
        match transport {
            Transport::Http => Self {
                transport,
                client: BlogHttpClient::new(addr),
                token: None,
            },
            Transport::Grpc => Self {
                transport,
                client: BlogGrpcClient::new(addr),
                token: None,
            },
        }
    }

    pub fn set_token(&mut self, token: &str) {
        self.token = Some(token.to_owned())
    }

    pub fn get_token(&self) -> Option<String> {
        self.token.clone()
    }

    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        self.client.register(username, email, password).await
    }

    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        self.client.login(username, password).await
    }

    pub async fn create_post(
        &self,
        title: &str,
        content: &str,
    ) -> Result<PostResponse, BlogClientError> {
        self.client.create_post(title, content).await
    }

    pub async fn get_post(&self, post_id: &str) -> Result<PostResponse, BlogClientError> {
        self.client.get_post(post_id).await
    }

    pub async fn update_post(
        &self,
        post_id: &str,
        title: &str,
        content: &str,
    ) -> Result<PostResponse, BlogClientError> {
        self.client.update_post(post_id, title, content).await
    }

    pub async fn delete_post(&self, post_id: &str) -> Result<(), BlogClientError> {
        self.client.delete_post(post_id).await
    }

    pub async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<PostListResponse, BlogClientError> {
        self.client.list_posts(limit, offset).await
    }
}
