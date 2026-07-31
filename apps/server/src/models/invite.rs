use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Invite {
    pub id: String,
    pub server_id: Uuid,
    pub created_by: Option<Uuid>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub expires_at: Option<time::OffsetDateTime>,
    pub max_uses: i64,
    pub uses: i64,
    pub revoked: bool,
}
