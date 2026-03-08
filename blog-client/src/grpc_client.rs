use async_trait::async_trait;
use tonic::transport::Channel;

use crate::blog::blog_service_client::BlogServiceClient;
use crate::blog::{
    self, Auth, GetPostRequest as GrpcGetPostRequest,
    ListPostsRequest as GrpcListPostsRequest,
};
use crate::error::BlogClientError;
use crate::types::*;
use crate::BlogApi;

pub struct BlogGrpcClient {
    client: BlogServiceClient<Channel>,
    token: Option<String>,
}

impl BlogGrpcClient {
    pub async fn connect(addr: &str) -> Result<Self, BlogClientError> {
        let client = BlogServiceClient::connect(addr.to_string()).await?;
        Ok(Self {
            client,
            token: None,
        })
    }

    pub fn new(addr: &str) -> Self {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            Self::connect(addr)
                .await
                .expect("Failed to connect to gRPC server")
        })
    }

    fn auth(&self) -> Result<Auth, BlogClientError> {
        let token = self
            .token
            .as_ref()
            .ok_or(BlogClientError::Unauthorized)?;
        Ok(Auth {
            token: token.clone(),
        })
    }

    fn proto_post_to_response(post: blog::Post) -> PostResponse {
        PostResponse {
            id: post.post_id,
            author_id: post.user_id,
            title: post.title,
            content: post.content,
            created_at: String::new(),
            updated_at: None,
        }
    }
}

#[async_trait]
impl BlogApi for BlogGrpcClient {
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
            .register(blog::RegisterRequest {
                login: username.to_string(),
                email: email.to_string(),
                password: password.to_string(),
            })
            .await?
            .into_inner();

        match resp.status() {
            blog::RegistrationStatus::RegistrationOk => {
                let auth = resp.auth.ok_or(BlogClientError::Internal(
                    "Missing auth in response".into(),
                ))?;
                let user = resp.user.unwrap_or_default();
                self.token = Some(auth.token.clone());
                Ok(AuthResponse {
                    token: auth.token,
                    user: AuthUserInfo {
                        id: user.user_id,
                        username: user.login,
                        email: user.email,
                    },
                })
            }
            blog::RegistrationStatus::RegistrationUserAlreadyExist => {
                Err(BlogClientError::UserAlreadyExists)
            }
            blog::RegistrationStatus::RegistrationInternalError => {
                Err(BlogClientError::Internal("Registration failed".into()))
            }
        }
    }

    async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let resp = self
            .client
            .login(blog::LoginRequest {
                login: username.to_string(),
                password: password.to_string(),
            })
            .await?
            .into_inner();

        match resp.status() {
            blog::LoginStatus::LoginOk => {
                let auth = resp.auth.ok_or(BlogClientError::Internal(
                    "Missing auth in response".into(),
                ))?;
                let user = resp.user.unwrap_or_default();
                self.token = Some(auth.token.clone());
                Ok(AuthResponse {
                    token: auth.token,
                    user: AuthUserInfo {
                        id: user.user_id,
                        username: user.login,
                        email: user.email,
                    },
                })
            }
            blog::LoginStatus::LoginInvalidCredentials => Err(BlogClientError::Unauthorized),
            blog::LoginStatus::LoginInternalError => {
                Err(BlogClientError::Internal("Login failed".into()))
            }
        }
    }

    async fn create_post(
        &self,
        title: &str,
        content: &str,
    ) -> Result<PostResponse, BlogClientError> {
        let mut client = self.client.clone();
        let resp = client
            .create_post(blog::CreatePostRequest {
                auth: Some(self.auth()?),
                title: title.to_string(),
                content: content.to_string(),
            })
            .await?
            .into_inner();

        match resp.status() {
            blog::CreatePostStatus::CreatePostOk => {
                let post = resp
                    .post
                    .ok_or(BlogClientError::Internal("Missing post in response".into()))?;
                Ok(Self::proto_post_to_response(post))
            }
            blog::CreatePostStatus::CreatePostUnauthorized => Err(BlogClientError::Unauthorized),
            blog::CreatePostStatus::CreatePostInternalError => {
                Err(BlogClientError::Internal("Create post failed".into()))
            }
        }
    }

    async fn get_post(&self, post_id: &str) -> Result<PostResponse, BlogClientError> {
        let mut client = self.client.clone();
        let resp = client
            .get_post(GrpcGetPostRequest {
                auth: self.token.as_ref().map(|t| Auth { token: t.clone() }),
                post_id: post_id.to_string(),
            })
            .await?
            .into_inner();

        match resp.status() {
            blog::GetPostStatus::GetPostOk => {
                let post = resp
                    .post
                    .ok_or(BlogClientError::Internal("Missing post in response".into()))?;
                Ok(Self::proto_post_to_response(post))
            }
            blog::GetPostStatus::GetPostNotFound => Err(BlogClientError::NotFound),
            blog::GetPostStatus::GetPostUnauthorized => Err(BlogClientError::Unauthorized),
            blog::GetPostStatus::GetPostInternalError => {
                Err(BlogClientError::Internal("Get post failed".into()))
            }
        }
    }

    async fn update_post(
        &self,
        post_id: &str,
        title: &str,
        content: &str,
    ) -> Result<PostResponse, BlogClientError> {
        let mut client = self.client.clone();
        let resp = client
            .update_post(blog::UpdatePostRequest {
                auth: Some(self.auth()?),
                post_id: post_id.to_string(),
                title: title.to_string(),
                content: content.to_string(),
            })
            .await?
            .into_inner();

        match resp.status() {
            blog::UpdatePostStatus::UpdatePostOk => {
                let post = resp
                    .post
                    .ok_or(BlogClientError::Internal("Missing post in response".into()))?;
                Ok(Self::proto_post_to_response(post))
            }
            blog::UpdatePostStatus::UpdatePostNotFound => Err(BlogClientError::NotFound),
            blog::UpdatePostStatus::UpdatePostUnauthorized => Err(BlogClientError::Unauthorized),
            blog::UpdatePostStatus::UpdatePostForbidden => Err(BlogClientError::Forbidden),
            blog::UpdatePostStatus::UpdatePostInternalError => {
                Err(BlogClientError::Internal("Update post failed".into()))
            }
        }
    }

    async fn delete_post(&self, post_id: &str) -> Result<(), BlogClientError> {
        let mut client = self.client.clone();
        let resp = client
            .delete_post(blog::DeletePostRequest {
                auth: Some(self.auth()?),
                post_id: post_id.to_string(),
            })
            .await?
            .into_inner();

        match resp.status() {
            blog::DeletePostStatus::DeletePostOk => Ok(()),
            blog::DeletePostStatus::DeletePostNotFound => Err(BlogClientError::NotFound),
            blog::DeletePostStatus::DeletePostUnauthorized => Err(BlogClientError::Unauthorized),
            blog::DeletePostStatus::DeletePostForbidden => Err(BlogClientError::Forbidden),
            blog::DeletePostStatus::DeletePostInternalError => {
                Err(BlogClientError::Internal("Delete post failed".into()))
            }
        }
    }

    async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<PostListResponse, BlogClientError> {
        let mut client = self.client.clone();
        let resp = client
            .list_posts(GrpcListPostsRequest { limit, offset })
            .await?
            .into_inner();

        match resp.status() {
            blog::ListPostsStatus::ListPostsOk => {
                let posts = resp
                    .posts
                    .into_iter()
                    .map(Self::proto_post_to_response)
                    .collect();
                Ok(PostListResponse {
                    posts,
                    total: resp.total,
                    limit,
                    offset,
                })
            }
            blog::ListPostsStatus::ListPostsInternalError => {
                Err(BlogClientError::Internal("List posts failed".into()))
            }
        }
    }
}
