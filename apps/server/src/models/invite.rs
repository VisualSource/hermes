use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Invite {
    id: Uuid,
    server_id: Uuid,
    created_by: Option<Uuid>,
    #[serde(with = "time::serde::rfc3339")]
    expires_at: time::OffsetDateTime,
    max_uses: u8,
    uses: u8,
    revoked: bool,
}

impl Invite {
    pub async fn create(
        db: &SqlitePool,
        server_id: Uuid,
        created_by: Uuid,
        max_uses: u8,
    ) -> Result<Invite, sqlx::Error> {
        todo!()
    }
    pub async fn delete(db: &SqlitePool, id: Uuid) -> Result<(), sqlx::Error> {
        todo!()
    }
    pub async fn get(db: &SqlitePool) -> Result<Invite, sqlx::Error> {
        todo!()
    }
}
