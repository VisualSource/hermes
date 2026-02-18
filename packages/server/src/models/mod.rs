use thiserror::Error;

pub mod user;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error(transparent)]
    Query(#[from] sqlx::Error),
}
