pub mod blog {
    tonic::include_proto!("blog");
}

pub mod error;
pub mod grpc_client;
pub mod http_client;
pub mod types;

use grpc_client::BlogGrpcClient;
use http_client::BlogHttpClient;

use async_trait::async_trait;
use error::BlogClientError;
pub use types::*;

#[async_trait]
pub trait BlogApi {
    fn set_token(&mut self, token: String);
    fn get_token(&self) -> Option<&str>;

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

#[derive(Clone, Copy, strum_macros::EnumString)]
pub enum Transport {
    Http,
    Grpc,
}

pub struct BlogClient {
    pub transport: Transport,
    addr: String,
    client: Box<dyn BlogApi + Send + Sync>,
}

impl BlogClient {
    pub async fn new(transport: Transport, addr: &str) -> Self {
        let client: Box<dyn BlogApi + Send + Sync> = match &transport {
            Transport::Http => Box::new(BlogHttpClient::new(addr)),
            Transport::Grpc => Box::new(BlogGrpcClient::connect(addr).await.expect("Failed to connect to gRPC server")),
        };
        Self { transport, addr: addr.to_string(), client }
    }

    pub fn transport_kind(&self) -> Transport {
        self.transport
    }

    pub fn transport_addr(&self) -> &str {
        &self.addr
    }

    pub fn set_token(&mut self, token: String) {
        self.client.set_token(token);
    }

    pub fn get_token(&self) -> Option<&str> {
        self.client.get_token()
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
