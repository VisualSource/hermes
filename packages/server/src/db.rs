use std::{env, io::ErrorKind};

use sqlx::SqlitePool;

pub async fn connect() -> Result<SqlitePool, std::io::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool_result = SqlitePool::connect("./sqlite.db").await;

    match pool_result {
        Ok(db) => Ok(db),
        Err(sqlx::Error::Configuration(e)) => Err(std::io::Error::new(ErrorKind::Other, e)),
        Err(sqlx::Error::Database(e)) => Err(std::io::Error::new(ErrorKind::Other, e)),
        Err(e) => Err(std::io::Error::new(ErrorKind::Other, e)),
    }
}
