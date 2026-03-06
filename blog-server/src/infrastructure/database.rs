use sqlx::{PgPool, migrate, postgres::PgPoolOptions};

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    migrate!("./migrations").run(pool).await?;
    Ok(())
}

pub async fn create_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new().max_connections(10).connect(url).await
}
