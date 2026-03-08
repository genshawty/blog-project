use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::application::auth_service::AuthService;
use crate::application::blog_service::BlogService;
use crate::blog::blog_service_server::BlogService as BlogServiceTrait;
use crate::blog::*;
use crate::infrastructure::jwt::JwtKeys;

pub struct GrpcBlogService {
    auth_service: AuthService,
    blog_service: BlogService,
    keys: JwtKeys,
}

impl GrpcBlogService {
    pub fn new(
        auth_service: AuthService,
        blog_service: BlogService,
        keys: JwtKeys,
    ) -> Self {
        Self {
            auth_service,
            blog_service,
            keys,
        }
    }

    fn authenticate(&self, auth: Option<&Auth>) -> Result<Uuid, Status> {
        let token = auth
            .map(|a| a.token.as_str())
            .ok_or_else(|| Status::unauthenticated("missing auth token"))?;
        let claims = self
            .keys
            .verify_token(token)
            .map_err(|_| Status::unauthenticated("invalid token"))?;
        Uuid::parse_str(&claims.user_id)
            .map_err(|_| Status::unauthenticated("invalid user id in token"))
    }
}

#[tonic::async_trait]
impl BlogServiceTrait for GrpcBlogService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        match self
            .auth_service
            .register(req.login.clone(), req.email.clone(), req.password)
            .await
        {
            Ok((token, user)) => Ok(Response::new(RegisterResponse {
                status: RegistrationStatus::RegistrationOk.into(),
                auth: Some(Auth { token }),
                user: Some(User {
                    user_id: user.id.to_string(),
                    login: user.username,
                    email: user.email,
                }),
            })),
            Err(crate::domain::BlogError::UserAlreadyExists) => {
                Err(Status::already_exists("user already exists"))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        match self.auth_service.login(&req.login, &req.password).await {
            Ok((token, user)) => Ok(Response::new(LoginResponse {
                status: LoginStatus::LoginOk.into(),
                auth: Some(Auth { token }),
                user: Some(User {
                    user_id: user.id.to_string(),
                    login: user.username,
                    email: user.email,
                }),
            })),
            Err(crate::domain::BlogError::Unauthorized) => {
                Err(Status::unauthenticated("invalid credentials"))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<CreatePostResponse>, Status> {
        let req = request.into_inner();
        let user_id = self.authenticate(req.auth.as_ref())?;

        match self
            .blog_service
            .create_post(req.title, req.content, user_id)
            .await
        {
            Ok(post) => Ok(Response::new(CreatePostResponse {
                status: CreatePostStatus::CreatePostOk.into(),
                post: Some(Post {
                    post_id: post.id.to_string(),
                    user_id: post.author_id.to_string(),
                    title: post.title,
                    content: post.content,
                }),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<Response<GetPostResponse>, Status> {
        let req = request.into_inner();
        let post_id = Uuid::parse_str(&req.post_id)
            .map_err(|_| Status::invalid_argument("invalid post_id"))?;

        match self.blog_service.get_post(post_id).await {
            Ok(post) => Ok(Response::new(GetPostResponse {
                status: GetPostStatus::GetPostOk.into(),
                post: Some(Post {
                    post_id: post.id.to_string(),
                    user_id: post.author_id.to_string(),
                    title: post.title,
                    content: post.content,
                }),
            })),
            Err(crate::domain::DomainError::PostNotFound(_)) => {
                Err(Status::not_found("post not found"))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<UpdatePostResponse>, Status> {
        let req = request.into_inner();
        let user_id = self.authenticate(req.auth.as_ref())?;

        let post_id = Uuid::parse_str(&req.post_id)
            .map_err(|_| Status::invalid_argument("invalid post_id"))?;

        match self
            .blog_service
            .update_post(user_id, post_id, req.title, req.content)
            .await
        {
            Ok(post) => Ok(Response::new(UpdatePostResponse {
                status: UpdatePostStatus::UpdatePostOk.into(),
                post: Some(Post {
                    post_id: post.id.to_string(),
                    user_id: post.author_id.to_string(),
                    title: post.title,
                    content: post.content,
                }),
            })),
            Err(crate::domain::DomainError::PostNotFound(_)) => {
                Err(Status::not_found("post not found"))
            }
            Err(crate::domain::DomainError::Forbidden) => {
                Err(Status::permission_denied("not the author"))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostResponse>, Status> {
        let req = request.into_inner();
        let user_id = self.authenticate(req.auth.as_ref())?;

        let post_id = Uuid::parse_str(&req.post_id)
            .map_err(|_| Status::invalid_argument("invalid post_id"))?;

        match self.blog_service.delete_post(user_id, post_id).await {
            Ok(()) => Ok(Response::new(DeletePostResponse {
                status: DeletePostStatus::DeletePostOk.into(),
            })),
            Err(crate::domain::DomainError::PostNotFound(_)) => {
                Err(Status::not_found("post not found"))
            }
            Err(crate::domain::DomainError::Forbidden) => {
                Err(Status::permission_denied("not the author"))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let req = request.into_inner();
        let limit = if req.limit > 0 { req.limit } else { 10 };
        let offset = if req.offset >= 0 { req.offset } else { 0 };

        match self.blog_service.list_posts(limit, offset).await {
            Ok((posts, total)) => Ok(Response::new(ListPostsResponse {
                status: ListPostsStatus::ListPostsOk.into(),
                posts: posts
                    .into_iter()
                    .map(|p| Post {
                        post_id: p.id.to_string(),
                        user_id: p.author_id.to_string(),
                        title: p.title,
                        content: p.content,
                    })
                    .collect(),
                total,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
