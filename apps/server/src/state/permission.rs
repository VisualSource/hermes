use actix_web::http::StatusCode;
use sqlx::{SqlitePool, query, query_scalar};
use uuid::Uuid;

use crate::{models::channel::ChannelKind, state::api_errors::ApplicationError};

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
pub const MANAGE_USERS: u64 = 1 << 10;
pub const ALL_PERMS: u64 = u64::MAX;
pub const BASE_PERMS: u64 = VIEW_CHANNELS | SEND_MESSAGES;

pub async fn has_permissions(
    db: &SqlitePool,
    user: &Uuid,
    server: &Uuid,
    required: u64,
) -> Result<bool, sqlx::Error> {
    let permissions = effective_permissions(db, user, server).await?;

    return Ok(permissions & required == required);
}

pub async fn required_permissions(
    db: &SqlitePool,
    user: &Uuid,
    server: &Uuid,
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
    user: &Uuid,
    server: &Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT s.owner_id, sm.id AS "member_id", r.mask AS "mask!" FROM servers s 
        LEFT JOIN server_members sm ON sm.server_id = s.id AND sm.user_id = ? 
        LEFT JOIN role_members rm ON rm.member_id = sm.id 
        LEFT JOIN roles r ON r.id = rm.role_id AND r.server_id = s.id WHERE s.id = ?"#,
        user,
        server
    )
    .fetch_all(db)
    .await?;

    if result.is_empty() {
        return Ok(0);
    }

    if result[0].owner_id == *user {
        return Ok(ALL_PERMS);
    }

    if result[0].member_id.is_none() {
        return Ok(0);
    }

    return Ok(BASE_PERMS | result.iter().fold(0, |acc, m| acc | m.mask as u64));
}

pub enum ChannelScope {
    Server { id: Uuid, perms: u64 },
    Dm,
}

pub struct ChannelAccess {
    pub kind: ChannelKind,
    pub scope: ChannelScope,
}

impl ChannelAccess {
    pub fn require(&self, required_perms: u64) -> Result<(), ApplicationError> {
        match self.scope {
            ChannelScope::Server { perms, .. } => {
                if perms & required_perms == required_perms {
                    return Ok(());
                }

                return Err(ApplicationError::new(
                    StatusCode::FORBIDDEN,
                    "user does not have required permissions",
                    "user",
                    Vec::default(),
                    None,
                ));
            }
            ChannelScope::Dm => Ok(()),
        }
    }
    pub fn permissions(&self) -> Option<u64> {
        match &self.scope {
            ChannelScope::Server { perms, .. } => Some(*perms),
            ChannelScope::Dm => None,
        }
    }
}

pub async fn channel_permissions(
    db: &SqlitePool,
    user: &Uuid,
    channel_id: &Uuid,
) -> Result<ChannelAccess, ApplicationError> {
    let channel = query!(
        "SELECT kind, server_id FROM channels WHERE id = ?",
        &channel_id
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| ApplicationError::not_found("channel"))?;

    if let Some(server_id) = channel.server_id {
        let bits = effective_permissions(db, user, &server_id).await?;

        if bits & VIEW_CHANNELS != VIEW_CHANNELS {
            return Err(ApplicationError::new(
                StatusCode::FORBIDDEN,
                "forbidden",
                "user",
                Vec::default(),
                None,
            ));
        }

        Ok(ChannelAccess {
            kind: channel.kind,
            scope: ChannelScope::Server {
                id: server_id,
                perms: bits,
            },
        })
    } else {
        let result = query_scalar!("SELECT COUNT(*) FROM dm_participants WHERE channel_id = ? AND (user_a_id = ? OR user_b_id = ?)",channel_id,&user,&user).fetch_one(db).await?;

        if result != 1 {
            return Err(ApplicationError::new(
                StatusCode::NOT_FOUND,
                "no channel exists for this user",
                "request",
                Vec::default(),
                None,
            ));
        }

        Ok(ChannelAccess {
            kind: channel.kind,
            scope: ChannelScope::Dm,
        })
    }
}
