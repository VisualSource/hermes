use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ChannelKind {
    Text,
    Voice,
    Dm,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Channel {
    pub id: Uuid,
    pub kind: ChannelKind,
    pub server_id: Option<Uuid>,
    pub name: String,
    pub category: Option<String>,
}

/// link info for a channel(dm)
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DmParticipant {
    id: String,
    channel_id: Uuid,
    user_a_id: Uuid,
    user_b_id: Uuid,
}
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Message {
    id: Uuid,
    channel_id: Uuid,
    user_id: Option<Uuid>,
    content: String,
    #[serde(with = "time::serde::rfc3339")]
    created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    edited_at: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    delete_at: Option<time::OffsetDateTime>,
}
