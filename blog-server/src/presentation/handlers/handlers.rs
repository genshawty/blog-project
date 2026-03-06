use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, ResponseError, Scope,
    delete, get, post, put, web,
};
use tracing::info;
use uuid::Uuid;

use crate::application::auth_service::AuthService;
use crate::application::blog_service::BlogService;
use crate::domain::{BlogError, DomainError};
use crate::presentation::auth::AuthenticatedUser;
use crate::presentation::dto::{
    AuthResponse, AuthUserInfo, CreatePostRequest, LoginRequest, PaginationParams,
    PostListResponse, PostResponse, RegisterRequest, UpdatePostRequest,
};
use crate::presentation::middleware::RequestId;

impl ResponseError for BlogError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            BlogError::Unauthorized => actix_web::http::StatusCode::UNAUTHORIZED,
            BlogError::Forbidden => actix_web::http::StatusCode::FORBIDDEN,
            BlogError::UserAlreadyExists => actix_web::http::StatusCode::CONFLICT,
            BlogError::UserNotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            BlogError::PostNotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            BlogError::Validation(_) => actix_web::http::StatusCode::BAD_REQUEST,
            BlogError::Internal(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub fn auth_scope() -> Scope {
    web::scope("/api/auth")
        .service(register)
        .service(login)
}

pub fn posts_scope() -> Scope {
    web::scope("/api/posts")
        .service(list_posts)
        .service(create_post)
        .service(get_post)
        .service(update_post)
        .service(delete_post)
}

fn request_id(req: &HttpRequest) -> String {
    req.extensions()
        .get::<RequestId>()
        .map(|rid| rid.0.clone())
        .unwrap_or_else(|| "unknown".into())
}

fn map_domain_err(e: DomainError) -> BlogError {
    match e {
        DomainError::PostNotFound(id) => BlogError::PostNotFound(id),
        DomainError::Forbidden => BlogError::Forbidden,
        DomainError::Validation(msg) => BlogError::Validation(msg),
        other => BlogError::Internal(other.to_string()),
    }
}

#[post("/register")]
async fn register(
    req: HttpRequest,
    auth: web::Data<AuthService>,
    payload: web::Json<RegisterRequest>,
) -> Result<HttpResponse, BlogError> {
    let (token, user) = auth
        .register(
            payload.username.clone(),
            payload.email.clone(),
            payload.password.clone(),
        )
        .await?;

    info!(request_id = %request_id(&req), user_id = %user.id, "user registered");
    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        user: AuthUserInfo {
            username: user.username,
            email: user.email,
        },
    }))
}

#[post("/login")]
async fn login(
    req: HttpRequest,
    auth: web::Data<AuthService>,
    payload: web::Json<LoginRequest>,
) -> Result<HttpResponse, BlogError> {
    let (token, user) = auth.login(&payload.username, &payload.password).await?;

    info!(request_id = %request_id(&req), "user logged in");
    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: AuthUserInfo {
            username: user.username,
            email: user.email,
        },
    }))
}

#[post("")]
async fn create_post(
    req: HttpRequest,
    user: AuthenticatedUser,
    blog: web::Data<BlogService>,
    payload: web::Json<CreatePostRequest>,
) -> Result<HttpResponse, BlogError> {
    let post = blog
        .create_post(payload.title.clone(), payload.content.clone(), user.id)
        .await
        .map_err(map_domain_err)?;

    info!(request_id = %request_id(&req), user_id = %user.id, post_id = %post.id, "post created");
    Ok(HttpResponse::Created().json(PostResponse::from(post)))
}

#[get("/{post_id}")]
async fn get_post(
    req: HttpRequest,
    blog: web::Data<BlogService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, BlogError> {
    let post_id = path.into_inner();
    let post = blog
        .get_post(post_id)
        .await
        .map_err(map_domain_err)?;

    info!(request_id = %request_id(&req), post_id = %post_id, "post fetched");
    Ok(HttpResponse::Ok().json(PostResponse::from(post)))
}

#[put("/{post_id}")]
async fn update_post(
    req: HttpRequest,
    user: AuthenticatedUser,
    blog: web::Data<BlogService>,
    path: web::Path<Uuid>,
    payload: web::Json<UpdatePostRequest>,
) -> Result<HttpResponse, BlogError> {
    let post_id = path.into_inner();
    let post = blog
        .update_post(user.id, post_id, payload.title.clone(), payload.content.clone())
        .await
        .map_err(map_domain_err)?;

    info!(request_id = %request_id(&req), user_id = %user.id, post_id = %post_id, "post updated");
    Ok(HttpResponse::Ok().json(PostResponse::from(post)))
}

#[delete("/{post_id}")]
async fn delete_post(
    req: HttpRequest,
    user: AuthenticatedUser,
    blog: web::Data<BlogService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, BlogError> {
    let post_id = path.into_inner();
    blog.delete_post(user.id, post_id)
        .await
        .map_err(map_domain_err)?;

    info!(request_id = %request_id(&req), user_id = %user.id, post_id = %post_id, "post deleted");
    Ok(HttpResponse::NoContent().finish())
}

#[get("")]
async fn list_posts(
    req: HttpRequest,
    blog: web::Data<BlogService>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, BlogError> {
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);
    let (posts, total) = blog
        .list_posts(limit, offset)
        .await
        .map_err(map_domain_err)?;

    info!(request_id = %request_id(&req), limit, offset, total, "posts listed");
    Ok(HttpResponse::Ok().json(PostListResponse {
        posts: posts.into_iter().map(PostResponse::from).collect(),
        total,
        limit,
        offset,
    }))
}
