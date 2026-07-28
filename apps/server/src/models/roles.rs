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

/// a single record that links a user to a role
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleMember {
    role_id: String,
    member_id: Uuid,
}
