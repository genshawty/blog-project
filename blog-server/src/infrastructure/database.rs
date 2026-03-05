use sqlx::{
    PgPool, migrate,
    pool::Pool,
    postgres::{PgPoolOptions, Postgres},
};

async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    migrate!("./migrations").run(pool).await?;
    Ok(())
}

async fn create_pool(url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    PgPoolOptions::new().max_connections(10).connect(url).await
}
