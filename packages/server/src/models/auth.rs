use sqlx::{SqlitePool, query};
use std::ops::Add;

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct AuthorizationCode {
    pub id: uuid::Uuid,
    pub code: String,
    pub user_id: uuid::Uuid,
    pub created_at: time::UtcDateTime,
    pub expires_at: time::UtcDateTime,
    pub used: bool,

    pub scopes: Option<String>,

    pub code_challenge: String,
    pub code_challenge_method: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RefreshToken {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub created_at: time::UtcDateTime,
    pub expires_at: time::UtcDateTime,
}

impl RefreshToken {
    pub async fn remove_token(
        id: &uuid::Uuid,
        db: &SqlitePool,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        query!("DELETE FROM refresh_tokens WHERE id = ?;", id)
            .execute(db)
            .await
    }

    pub async fn insert_token(
        id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        expires_at: time::OffsetDateTime,
        db: &SqlitePool,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let created_at = time::OffsetDateTime::now_utc();

        let result = query!(
            "INSERT INTO refresh_tokens VALUES (?,?,?,?)",
            id,
            user_id,
            created_at,
            expires_at
        )
        .execute(db)
        .await;

        result
    }
    pub async fn get_token(
        jti: &uuid::Uuid,
        db: &SqlitePool,
    ) -> Result<Option<RefreshToken>, sqlx::Error> {
        let result = sqlx::query_as!(RefreshToken,r#"SELECT id as "id: uuid::Uuid", user_id as "user_id: uuid::Uuid", created_at,expires_at FROM refresh_tokens WHERE id = ? LIMIT 1;"#,jti).fetch_optional(db).await;

        result
    }
}

impl AuthorizationCode {
    pub async fn get_by_code_and_user(
        code: &str,
        db: &SqlitePool,
    ) -> Result<Option<AuthorizationCode>, sqlx::Error> {
        let result = sqlx::query_as!(AuthorizationCode,r#"SELECT id as "id: uuid::Uuid",code,user_id as "user_id: uuid::Uuid",created_at,expires_at,used,code_challenge,code_challenge_method,scopes FROM grants WHERE code = ?"#,code).fetch_optional(db).await;

        result
    }

    pub async fn mark_code_used(code: &str, db: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE grants SET used = TRUE WHERE code = ?", code)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn remove_expired(db: &SqlitePool) -> Result<(), sqlx::Error> {
        let now = time::OffsetDateTime::now_utc();

        sqlx::query!("DELETE FROM grants WHERE expires_at < ?;", now)
            .execute(db)
            .await?;

        Ok(())
    }

    pub async fn insert_request(
        code: &str,
        user_id: &uuid::Uuid,
        challenge: &str,
        method: &str,
        scopes: Option<String>,
        db: &SqlitePool,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let ts = uuid::Timestamp::now(uuid::NoContext);
        let id = uuid::Uuid::new_v7(ts);

        let now = time::OffsetDateTime::now_utc();
        let expires = now.add(time::Duration::minutes(10));

        sqlx::query!(
            "INSERT INTO grants VALUES (?,?,?,?,?,?,?,?,?);",
            id,
            code,
            user_id,
            now,
            expires,
            scopes,
            false,
            challenge,
            method
        )
        .execute(db)
        .await
    }

    pub fn is_expired(&self) -> bool {
        time::UtcDateTime::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.used && !self.is_expired()
    }
}
