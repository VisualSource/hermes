use super::DatabaseError;
use sqlx::SqlitePool;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub psd_hash: String,
    pub avatar: Option<String>,
    pub created_at: time::UtcDateTime,
}

impl User {
    pub async fn insert_user(
        username: &str,
        avatar: &str,
        psd_hash: &str,
        db: &SqlitePool,
    ) -> Result<Uuid, DatabaseError> {
        let ts = Timestamp::now(NoContext);
        let id = Uuid::new_v7(ts);
        let timestamp = time::OffsetDateTime::now_utc();

        sqlx::query!(
            "INSERT INTO users VALUES (?,?,?,?,?)",
            id,
            username,
            psd_hash,
            avatar,
            timestamp
        )
        .execute(db)
        .await?;

        return Ok(id);
    }

    pub async fn find_by_uuid(id: &Uuid, db: &SqlitePool) -> Result<Option<User>, DatabaseError> {
        let result = sqlx::query_as!(User,r#"SELECT id as "id: uuid::Uuid", username, psd_hash,avatar,created_at FROM users WHERE id = ?"#,id).fetch_optional(db).await?;
        Ok(result)
    }
    pub async fn find_by_username(
        username: &str,
        db: &SqlitePool,
    ) -> Result<Option<User>, DatabaseError> {
        let result = sqlx::query_as!(User,r#"SELECT id as "id: uuid::Uuid", username, psd_hash, avatar, created_at FROM users WHERE username = ?"#,username).fetch_optional(db).await?;

        Ok(result)
    }
}

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct UserPublicKey {
    id: Uuid,
    user_id: Uuid,
    public_key: String,
}

impl UserPublicKey {
    pub async fn find_all_by_user_id<DB>(
        user_id: &Uuid,
        db: &sqlx::SqlitePool,
    ) -> Result<Vec<UserPublicKey>, DatabaseError> {
        let result = sqlx::query_as!(UserPublicKey,r#"SELECT id as "id: uuid::Uuid",user_id as "user_id: uuid::Uuid", public_key FROM keys WHERE user_id = ?"#, user_id)
            .fetch_all(db).await?;

        Ok(result)
    }
    pub async fn find_key_by_uuid<DB>(
        id: &Uuid,
        db: &sqlx::SqlitePool,
    ) -> Result<Option<UserPublicKey>, DatabaseError> {
        let result = sqlx::query_as!(UserPublicKey,r#"SELECT id as "id: uuid::Uuid", user_id as "user_id: uuid::Uuid", public_key FROM keys WHERE id = ?"#,id)
            .fetch_optional(db).await?;
        Ok(result)
    }

    pub async fn insert_key<DB>(
        key: &str,
        user: &Uuid,
        db: &sqlx::SqlitePool,
    ) -> Result<Uuid, DatabaseError> {
        let ts = Timestamp::now(NoContext);
        let id = Uuid::new_v7(ts);

        sqlx::query!("INSERT INTO keys VALUES (?,?,?)", id, user, key)
            .execute(db)
            .await?;

        Ok(id)
    }

    pub async fn remove_key_by_uuid(
        id: &Uuid,
        db: &sqlx::SqlitePool,
    ) -> Result<(), DatabaseError> {
        sqlx::query!("DELETE FROM keys WHERE id = ?", id)
            .execute(db)
            .await?;

        Ok(())
    }
}
