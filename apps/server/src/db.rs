use sqlx::{SqlitePool, migrate::MigrateDatabase};
use std::env;

pub async fn connect() -> Result<SqlitePool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    if !sqlx::Sqlite::database_exists(&database_url).await? {
        sqlx::Sqlite::create_database(&database_url).await?;
    }

    let pool = SqlitePool::connect(&database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
