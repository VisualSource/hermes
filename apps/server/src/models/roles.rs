use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Role {
    pub id: Uuid,
    pub server_id: Uuid,
    pub name: String,
    pub fg_color: Option<String>,
    pub bg_color: Option<String>,
    pub mask: i64,
}

/// a single record that links a role to a `server_members` row — note
/// `member_id` is a `server_members.id`, not a `users.id`.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleMember {
    pub role_id: Uuid,
    pub member_id: Uuid,
}

