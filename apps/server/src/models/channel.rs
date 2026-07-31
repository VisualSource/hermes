use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq, PartialOrd)]
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
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct DmParticipant {
    pub channel_id: Uuid,
    pub user_a_id: Uuid,
    pub user_b_id: Uuid,
}

impl DmParticipant {
    pub fn canonical_pair(x: Uuid, y: Uuid) -> (Uuid, Uuid) {
        if x <= y { (x, y) } else { (y, x) }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct FriendRequest {
    pub id: Uuid,
    pub from_user: Uuid,
    pub to_user: Uuid,
    pub rejected: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Option<Uuid>,
    pub content: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub edited_at: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<time::OffsetDateTime>,
}
