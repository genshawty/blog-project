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
            Ok((token, _)) => Ok(Response::new(RegisterResponse {
                status: ResistrationStatus::RegistrationOk.into(),
                auth: Some(Auth { token }),
            })),
            Err(crate::domain::BlogError::UserAlreadyExists) => Ok(Response::new(RegisterResponse {
                status: ResistrationStatus::RegistrationUserAlreadyExist.into(),
                auth: None,
            })),
            Err(_) => Ok(Response::new(RegisterResponse {
                status: ResistrationStatus::RegistrationInternalError.into(),
                auth: None,
            })),
        }
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        match self.auth_service.login(&req.login, &req.password).await {
            Ok((token, _)) => Ok(Response::new(LoginResponse {
                status: LoginStatus::LoginOk.into(),
                auth: Some(Auth { token }),
            })),
            Err(crate::domain::BlogError::Unauthorized) => Ok(Response::new(LoginResponse {
                status: LoginStatus::LoginInvalidCredentials.into(),
                auth: None,
            })),
            Err(_) => Ok(Response::new(LoginResponse {
                status: LoginStatus::LoginInternalError.into(),
                auth: None,
            })),
        }
    }

    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<CreatePostResponse>, Status> {
        let req = request.into_inner();
        let user_id = match self.authenticate(req.auth.as_ref()) {
            Ok(id) => id,
            Err(_) => {
                return Ok(Response::new(CreatePostResponse {
                    status: CreatePostStatus::CreatePostUnauthorized.into(),
                    post: None,
                }));
            }
        };

        let content_text = req.content.map(|c| c.text).unwrap_or_default();
        match self
            .blog_service
            .create_post(content_text.clone(), content_text, user_id)
            .await
        {
            Ok(post) => Ok(Response::new(CreatePostResponse {
                status: CreatePostStatus::CreatePostOk.into(),
                post: Some(Post {
                    post_id: post.id.to_string(),
                    user_id: post.author_id.to_string(),
                    content: Some(Content {
                        text: post.content,
                    }),
                }),
            })),
            Err(_) => Ok(Response::new(CreatePostResponse {
                status: CreatePostStatus::CreatePostInternalError.into(),
                post: None,
            })),
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
                    content: Some(Content {
                        text: post.content,
                    }),
                }),
            })),
            Err(crate::domain::DomainError::PostNotFound(_)) => {
                Ok(Response::new(GetPostResponse {
                    status: GetPostStatus::GetPostNotFound.into(),
                    post: None,
                }))
            }
            Err(_) => Ok(Response::new(GetPostResponse {
                status: GetPostStatus::GetPostInternalError.into(),
                post: None,
            })),
        }
    }

    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<UpdatePostResponse>, Status> {
        let req = request.into_inner();
        let user_id = match self.authenticate(req.auth.as_ref()) {
            Ok(id) => id,
            Err(_) => {
                return Ok(Response::new(UpdatePostResponse {
                    status: UpdatePostStatus::UpdatePostUnauthorized.into(),
                    post: None,
                }));
            }
        };

        let post_id = Uuid::parse_str(&req.post_id)
            .map_err(|_| Status::invalid_argument("invalid post_id"))?;
        let content_text = req.content.map(|c| c.text).unwrap_or_default();

        match self
            .blog_service
            .update_post(user_id, post_id, content_text.clone(), content_text)
            .await
        {
            Ok(post) => Ok(Response::new(UpdatePostResponse {
                status: UpdatePostStatus::UpdatePostOk.into(),
                post: Some(Post {
                    post_id: post.id.to_string(),
                    user_id: post.author_id.to_string(),
                    content: Some(Content {
                        text: post.content,
                    }),
                }),
            })),
            Err(crate::domain::DomainError::PostNotFound(_)) => {
                Ok(Response::new(UpdatePostResponse {
                    status: UpdatePostStatus::UpdatePostNotFound.into(),
                    post: None,
                }))
            }
            Err(crate::domain::DomainError::Forbidden) => Ok(Response::new(UpdatePostResponse {
                status: UpdatePostStatus::UpdatePostForbidden.into(),
                post: None,
            })),
            Err(_) => Ok(Response::new(UpdatePostResponse {
                status: UpdatePostStatus::UpdatePostInternalError.into(),
                post: None,
            })),
        }
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostResponse>, Status> {
        let req = request.into_inner();
        let user_id = match self.authenticate(req.auth.as_ref()) {
            Ok(id) => id,
            Err(_) => {
                return Ok(Response::new(DeletePostResponse {
                    status: DeletePostStatus::DeletePostUnauthorized.into(),
                }));
            }
        };

        let post_id = Uuid::parse_str(&req.post_id)
            .map_err(|_| Status::invalid_argument("invalid post_id"))?;

        match self.blog_service.delete_post(user_id, post_id).await {
            Ok(()) => Ok(Response::new(DeletePostResponse {
                status: DeletePostStatus::DeletePostOk.into(),
            })),
            Err(crate::domain::DomainError::PostNotFound(_)) => {
                Ok(Response::new(DeletePostResponse {
                    status: DeletePostStatus::DeletePostNotFound.into(),
                }))
            }
            Err(crate::domain::DomainError::Forbidden) => Ok(Response::new(DeletePostResponse {
                status: DeletePostStatus::DeletePostForbidden.into(),
            })),
            Err(_) => Ok(Response::new(DeletePostResponse {
                status: DeletePostStatus::DeletePostInternalError.into(),
            })),
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
                        content: Some(Content { text: p.content }),
                    })
                    .collect(),
                total,
            })),
            Err(_) => Ok(Response::new(ListPostsResponse {
                status: ListPostsStatus::ListPostsInternalError.into(),
                posts: vec![],
                total: 0,
            })),
        }
    }
}
