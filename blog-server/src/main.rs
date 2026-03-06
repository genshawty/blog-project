pub mod application;
pub mod data;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

pub mod blog {
    tonic::include_proto!("blog");
}

use std::net::SocketAddr;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{App, HttpServer, web};
use tonic::transport::Server as TonicServer;
use tracing::info;

use application::auth_service::AuthService;
use application::blog_service::BlogService;
use blog::blog_service_server::BlogServiceServer;
use data::post_repository::{InMemoryPostRepository, PostRepository, PostgresPostRepository};
use data::user_repository::{InMemoryUserRepository, PostgresUserRepository, UserRepository};
use infrastructure::config::AppConfig;
use infrastructure::database::{create_pool, run_migrations};
use infrastructure::jwt::JwtKeys;
use infrastructure::logging::init_logging;
use presentation::grpc::grpc_server::GrpcBlogService;
use presentation::handlers;
use presentation::middleware::{JwtAuthMiddleware, RequestIdMiddleware, TimingMiddleware};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    let config = AppConfig::from_env().expect("invalid configuration");

    let storage = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "memory".into());

    match storage.as_str() {
        "postgres" => {
            let db_url = config
                .database_url
                .as_deref()
                .expect("DATABASE_URL must be set for postgres mode");
            let pool = create_pool(db_url)
                .await
                .expect("failed to connect to database");
            run_migrations(&pool)
                .await
                .expect("failed to run migrations");
            info!("Connected to PostgreSQL, migrations applied");

            let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
            let post_repo = Arc::new(PostgresPostRepository::new(pool));
            run_server(config, user_repo, post_repo).await
        }
        _ => {
            info!("Using in-memory storage");
            let user_repo = Arc::new(InMemoryUserRepository::default());
            let post_repo = Arc::new(InMemoryPostRepository::default());
            run_server(config, user_repo, post_repo).await
        }
    }
}

async fn run_server<U, P>(
    config: AppConfig,
    user_repo: Arc<U>,
    post_repo: Arc<P>,
) -> std::io::Result<()>
where
    U: UserRepository + Clone + 'static,
    P: PostRepository + Clone + 'static,
{
    let blog_service = BlogService::new(Arc::clone(&post_repo));
    let auth_service = AuthService::new(
        Arc::clone(&user_repo),
        JwtKeys::new(config.jwt_secret.clone()),
    );

    let grpc_addr: SocketAddr = format!("{}:{}", config.host, config.grpc_port)
        .parse()
        .expect("invalid gRPC address");
    let grpc_service = GrpcBlogService::new(
        auth_service.clone(),
        blog_service.clone(),
        JwtKeys::new(config.jwt_secret.clone()),
    );

    tokio::spawn(async move {
        info!("gRPC server listening on {}", grpc_addr);
        if let Err(e) = TonicServer::builder()
            .add_service(BlogServiceServer::new(grpc_service))
            .serve(grpc_addr)
            .await
        {
            tracing::error!("gRPC server failed: {:?}", e);
            std::process::exit(1);
        }
    });

    let config_data = config.clone();

    HttpServer::new(move || {
        let cors = build_cors(&config_data);
        App::new()
            .wrap(Logger::default())
            .wrap(RequestIdMiddleware)
            .wrap(TimingMiddleware)
            .wrap(
                DefaultHeaders::new()
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("Referrer-Policy", "no-referrer"))
                    .add(("Permissions-Policy", "geolocation=()"))
                    .add(("Cross-Origin-Opener-Policy", "same-origin")),
            )
            .wrap(cors)
            .app_data(web::Data::new(blog_service.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .service(
                web::scope("/api")
                    .wrap(JwtAuthMiddleware::new(auth_service.keys().clone()))
                    .service(handlers::handlers::auth_scope())
                    .service(handlers::handlers::posts_scope()),
            )
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}

fn build_cors(config: &AppConfig) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allow_any_header()
        .supports_credentials()
        .max_age(3600);

    if config.cors_origins.iter().any(|o| o == "*") {
        cors = cors.allow_any_origin();
    } else {
        for origin in &config.cors_origins {
            cors = cors.allowed_origin(origin);
        }
    }

    cors
}
