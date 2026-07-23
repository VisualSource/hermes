use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ChannelKind {
    Text,
    Voice,
    Dm,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Channel {
    id: Uuid,
    kind: ChannelKind,
    server_id: Option<Uuid>,
    name: String,
    category: Option<String>,
}

impl Channel {
    pub async fn create(
        db: &SqlitePool,
        server_id: Uuid,
        kind: ChannelKind,
        name: String,
        category: Option<String>,
    ) -> Result<Channel, sqlx::Error> {
        todo!()
    }
    pub async fn delete(db: &SqlitePool, id: Uuid) -> Result<(), sqlx::Error> {
        todo!()
    }
    pub async fn patch(
        db: &SqlitePool,
        name: String,
        category: Option<String>,
    ) -> Result<(), sqlx::Error> {
        todo!()
    }
    pub async fn get(db: &SqlitePool, id: Uuid) -> Result<Channel, sqlx::Error> {
        todo!()
    }

    pub async fn get_All_by_server(db: &SqlitePool) -> Result<Vec<Channel>, sqlx::Error> {
        todo!()
    }
}

/// link info for a channel(dm)
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DmParticipant {
    id: String,
    channel_id: Uuid,
    user_a_id: Uuid,
    user_b_id: Uuid,
}

impl DmParticipant {
    pub async fn create(
        db: &SqlitePool,
        channel_id: Uuid,
        user_a: Uuid,
        user_b: Uuid,
    ) -> Result<DmParticipant, sqlx::Error> {
        todo!()
    }
    pub async fn delete(db: &SqlitePool, id: String) -> Result<(), sqlx::Error> {
        todo!()
    }
    pub async fn get(db: &SqlitePool, id: String) -> Result<DmParticipant, sqlx::Error> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    id: Uuid,
    channel_id: Uuid,
    user_id: Option<Uuid>,
    content: String,
    created_at: time::UtcDateTime,
    edited_at: Option<time::UtcDateTime>,
    delete_at: Option<time::UtcDateTime>,
}

impl Message {
    pub async fn create() {}
    pub async fn get() {}

    pub async fn delete() {}
    pub async fn patch() {}
}
