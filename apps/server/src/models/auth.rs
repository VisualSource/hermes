use sqlx::{SqlitePool, query};
use std::ops::Add;

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct AuthorizationCode {
    pub id: uuid::Uuid,
    pub code: String,
    pub user_id: uuid::Uuid,
    pub client_id: uuid::Uuid,
    pub redirect_uri: String,
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
    pub family_id: uuid::Uuid,
    pub created_at: time::UtcDateTime,
    pub expires_at: time::UtcDateTime,
    pub used: bool,
    pub revoked: bool,
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

    pub async fn mark_token_used(
        id: &uuid::Uuid,
        db: &SqlitePool,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        query!("UPDATE refresh_tokens SET used = TRUE WHERE id = ?", id)
            .execute(db)
            .await
    }

    /// Revoke every refresh token in a family. Called when reuse is detected,
    /// per OAuth 2.1 §6.1 / RFC 6819 §5.2.2.3. Sets both `used` and `revoked`
    /// so the reuse-detection predicate (`used || revoked`) short-circuits on
    /// the first check regardless of order.
    pub async fn revoke_family(
        family_id: &uuid::Uuid,
        db: &SqlitePool,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        query!(
            "UPDATE refresh_tokens SET revoked = TRUE, used = TRUE WHERE family_id = ?",
            family_id
        )
        .execute(db)
        .await
    }

    /// Delete expired refresh tokens. Called hourly by the cleanup task.
    /// Revoked-but-not-expired rows stay until natural expiry so reuse
    /// detection still fires on them.
    pub async fn remove_expired(db: &SqlitePool) -> Result<(), sqlx::Error> {
        let now = time::OffsetDateTime::now_utc();
        sqlx::query!("DELETE FROM refresh_tokens WHERE expires_at < ?;", now)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn insert_token(
        id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        family_id: &uuid::Uuid,
        expires_at: time::UtcDateTime,
        db: &SqlitePool,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let created_at = time::OffsetDateTime::now_utc();
        let expires = time::OffsetDateTime::from(expires_at);

        query!(
            "INSERT INTO refresh_tokens (id, user_id, family_id, created_at, expires_at, used, revoked) \
             VALUES (?, ?, ?, ?, ?, FALSE, FALSE)",
            id,
            user_id,
            family_id,
            created_at,
            expires,
        )
        .execute(db)
        .await
    }

    pub async fn get_token(
        jti: &uuid::Uuid,
        db: &SqlitePool,
    ) -> Result<Option<RefreshToken>, sqlx::Error> {
        sqlx::query_as!(
            RefreshToken,
            r#"SELECT
                id,
                user_id,
                family_id,
                created_at,
                expires_at,
                used,
                revoked
             FROM refresh_tokens WHERE id = ? LIMIT 1;"#,
            jti
        )
        .fetch_optional(db)
        .await
    }
}

impl AuthorizationCode {
    pub async fn get_by_code(
        code: &str,
        db: &SqlitePool,
    ) -> Result<Option<AuthorizationCode>, sqlx::Error> {
        sqlx::query_as!(
            AuthorizationCode,
            r#"SELECT
                id,
                code,
                user_id,
                client_id,
                redirect_uri,
                created_at,
                expires_at,
                used,
                code_challenge,
                code_challenge_method,
                scopes
             FROM grants WHERE code = ?"#,
            code
        )
        .fetch_optional(db)
        .await
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
        client_id: &uuid::Uuid,
        redirect_uri: &str,
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
            "INSERT INTO grants \
             (id, code, user_id, client_id, redirect_uri, created_at, expires_at, scopes, used, code_challenge, code_challenge_method) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, FALSE, ?, ?);",
            id,
            code,
            user_id,
            client_id,
            redirect_uri,
            now,
            expires,
            scopes,
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
