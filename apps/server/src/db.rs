use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use std::env;
use std::str::FromStr;
use std::time::Duration;

pub async fn connect() -> Result<SqlitePool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    // https://mort.coffee/home/sqlite-editions/?ref=dailydev
    // Set pragmas on the connection options so they apply to every pooled
    // connection, rather than running them once as a checked query.
    let options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_millis(5000))
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new().connect_with(options).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
