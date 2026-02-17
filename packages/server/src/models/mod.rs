use thiserror::Error;

pub mod user;

#[derive(Debug,Error)]
pub enum DatabaseError {
    Query(#[from] sqlx::Error),
}

