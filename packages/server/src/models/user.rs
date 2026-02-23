use sqlx::SqlitePool;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub mfa: bool,
    #[serde(skip_serializing)]
    pub psd_hash: String,
    pub avatar: Option<String>,
    pub created_at: time::UtcDateTime,
}

impl User {
    pub async fn insert_user(
        username: &str,
        email: &str,
        avatar: &str,
        psd_hash: &str,
        db: &SqlitePool,
    ) -> Result<Uuid, sqlx::Error> {
        let ts = Timestamp::now(NoContext);
        let id = Uuid::new_v7(ts);
        let timestamp = time::OffsetDateTime::now_utc();

        sqlx::query!(
            "INSERT INTO users (id,username,email,mfa,psd_hash,avatar,created_at) VALUES (?,?,?,?,?,?,?)",
            id,
            username,
            email,
            false,
            psd_hash,
            avatar,
            timestamp
        )
        .execute(db)
        .await?;

        return Ok(id);
    }

    pub async fn find_by_uuid(id: &Uuid, db: &SqlitePool) -> Result<Option<User>, sqlx::Error> {
        let result = sqlx::query_as!(User,r#"SELECT id as "id: uuid::Uuid", username, psd_hash,avatar,created_at,mfa,email FROM users WHERE id = ?"#,id).fetch_optional(db).await?;
        Ok(result)
    }
    pub async fn find_by_username(
        username: &str,
        db: &SqlitePool,
    ) -> Result<Option<User>, sqlx::Error> {
        let result = sqlx::query_as!(User,r#"SELECT id as "id: uuid::Uuid", username, psd_hash, avatar, created_at,mfa,email FROM users WHERE username = ?"#,username).fetch_optional(db).await?;

        Ok(result)
    }

    pub async fn user_already_exists(
        username: &str,
        email: &str,
        db: &SqlitePool,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE username = ? OR email = ? LIMIT 1",
            username,
            email
        )
        .fetch_one(db)
        .await?;

        Ok(result > 0)
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
    ) -> Result<Vec<UserPublicKey>, sqlx::Error> {
        let result = sqlx::query_as!(UserPublicKey,r#"SELECT id as "id: uuid::Uuid",user_id as "user_id: uuid::Uuid", public_key FROM keys WHERE user_id = ?"#, user_id)
            .fetch_all(db).await?;

        Ok(result)
    }
    pub async fn find_key_by_uuid<DB>(
        id: &Uuid,
        db: &sqlx::SqlitePool,
    ) -> Result<Option<UserPublicKey>, sqlx::Error> {
        let result = sqlx::query_as!(UserPublicKey,r#"SELECT id as "id: uuid::Uuid", user_id as "user_id: uuid::Uuid", public_key FROM keys WHERE id = ?"#,id)
            .fetch_optional(db).await?;
        Ok(result)
    }

    pub async fn insert_key<DB>(
        key: &str,
        user: &Uuid,
        db: &sqlx::SqlitePool,
    ) -> Result<Uuid, sqlx::Error> {
        let ts = Timestamp::now(NoContext);
        let id = Uuid::new_v7(ts);

        sqlx::query!("INSERT INTO keys VALUES (?,?,?)", id, user, key)
            .execute(db)
            .await?;

        Ok(id)
    }

    pub async fn remove_key_by_uuid(id: &Uuid, db: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM keys WHERE id = ?", id)
            .execute(db)
            .await?;

        Ok(())
    }
}
