use std::{env, io::ErrorKind};

use sqlx::{SqlitePool, migrate::MigrateDatabase};

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

    let pool_result = SqlitePool::connect(&database_url).await;

    match pool_result {
        Ok(db) => Ok(db),
        Err(e) => Err(std::io::Error::new(ErrorKind::Other, e)),
    }
}
