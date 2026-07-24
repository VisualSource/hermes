use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, query, query_as};
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

impl Server {
    pub async fn create(
        db: &SqlitePool,
        owner_id: Uuid,
        name: String,
        icon: Option<String>,
    ) -> Result<Server, sqlx::Error> {
        let server_id = Uuid::now_v7();
        let timestamp = time::OffsetDateTime::now_utc();

        let result = query_as!(
            Server,
            r#"INSERT INTO servers (id, name, owner_id, created_at, icon) VALUES (?,?,?,?,?)
               RETURNING id, name, owner_id, created_at, icon"#,
            server_id,
            name,
            owner_id,
            timestamp,
            icon
        )
        .fetch_one(db)
        .await?;

        Ok(result)
    }
    pub async fn patch(
        db: &SqlitePool,
        server_id: Uuid,
        name: String,
        icon: Option<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE servers SET name = ? AND icon = ? WHERE id = ?",
            name,
            icon,
            server_id
        )
        .execute(db)
        .await?;

        Ok(())
    }
    pub async fn get(db: &SqlitePool, id: Uuid) -> Result<Server, sqlx::Error> {
        let result = query_as!(
            Server,
            r#"SELECT id, name, icon, owner_id, created_at FROM servers WHERE id = ?"#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(result)
    }
    pub async fn delete(db: &SqlitePool, server_id: Uuid) -> Result<(), sqlx::Error> {
        query!("DELETE FROM servers WHERE id = ?", server_id)
            .execute(db)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServerMember {
    id: String,
    server_id: Uuid,
    user_id: Uuid,
}

impl ServerMember {
    pub async fn get(db: &SqlitePool, id: String) -> Result<ServerMember, sqlx::Error> {
        todo!()
    }
    pub async fn delete(
        db: &SqlitePool,
        user_id: Uuid,
        server_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        todo!()
    }
    pub async fn create(
        db: &SqlitePool,
        server_id: Uuid,
        user_id: Uuid,
    ) -> Result<ServerMember, sqlx::Error> {
        todo!()
    }

    pub async fn get_by_all_server(
        db: &SqlitePool,
        server_id: Uuid,
    ) -> Result<Vec<ServerMember>, sqlx::Error> {
        todo!()
    }
}
