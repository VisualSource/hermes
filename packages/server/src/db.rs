use sqlx::{SqlitePool, migrate::MigrateDatabase};
use std::{env, io::ErrorKind};

pub async fn connect() -> Result<SqlitePool, std::io::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    if !sqlx::Sqlite::database_exists(&database_url)
        .await
        .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?
    {
        sqlx::Sqlite::create_database(&database_url)
            .await
            .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;
    }

    let pool = SqlitePool::connect(&database_url)
        .await
        .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;

    sqlx::query!("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await
        .map_err(|err| std::io::Error::new(ErrorKind::Other, err))?;

    Ok(pool)
}
