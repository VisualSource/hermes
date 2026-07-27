use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    pub icon: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct ServerMember {
    pub id: String,
    pub server_id: Uuid,
    pub user_id: Uuid,
}
