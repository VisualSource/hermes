use actix_web::http::StatusCode;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::state::api_errors::ApplicationError;

pub const VIEW_CHANNELS: u64 = 1 << 0;
pub const SEND_MESSAGES: u64 = 1 << 1;
pub const MANAGE_MESSAGES: u64 = 1 << 2;
pub const MANAGE_CHANNELS: u64 = 1 << 3;
pub const MANAGE_ROLES: u64 = 1 << 4;
pub const MANAGE_SERVER: u64 = 1 << 5;
pub const CREATE_INVITE: u64 = 1 << 6;
pub const MANAGE_INVITES: u64 = 1 << 7;
pub const ADD_ROLE: u64 = 1 << 8;
pub const REMOVE_ROLE: u64 = 1 << 9;

pub const ALL_PERMS: u64 = u64::MAX;
pub const BASE_PERMS: u64 = VIEW_CHANNELS | SEND_MESSAGES;

pub async fn has_permissions(
    db: &SqlitePool,
    user: Uuid,
    server: Uuid,
    required: u64,
) -> Result<bool, sqlx::Error> {
    let permissions = effective_permissions(db, user, server).await?;

    return Ok(permissions & required == required);
}

pub async fn required_permissions(
    db: &SqlitePool,
    user: Uuid,
    server: Uuid,
    required: u64,
) -> Result<(), ApplicationError> {
    if has_permissions(db, user, server, required).await? {
        return Ok(());
    }

    Err(ApplicationError::new(
        StatusCode::FORBIDDEN,
        "user does not have required permissions",
        "user",
        Vec::default(),
        None,
    ))
}

pub async fn effective_permissions(
    db: &SqlitePool,
    user: Uuid,
    server: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT s.owner_id, sm.id AS "member_id", r.mask AS "mask!" FROM servers s 
        LEFT JOIN server_members sm ON sm.server_id = s.id AND sm.user_id = ? 
        LEFT JOIN role_members rm ON rm.member_id = sm.id 
        LEFT JOIN roles r ON r.id = rm.role_id AND r.server_id = s.id WHERE s.id = ?"#,
        server,
        user
    )
    .fetch_all(db)
    .await?;

    if result.is_empty() {
        return Ok(0);
    }

    if result[0].owner_id == user {
        return Ok(ALL_PERMS);
    }

    if result[0].member_id.is_none() {
        return Ok(0);
    }

    return Ok(BASE_PERMS | result.iter().fold(0, |acc, m| acc | m.mask as u64));
}
