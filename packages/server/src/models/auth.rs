use sqlx::{FromRow, Row, sqlite::SqliteRow};
use time::macros::format_description;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AuthorizationCode {
    pub id: uuid::Uuid,
    pub code: String,
    pub user_id: uuid::Uuid,
    pub created_at: time::UtcDateTime,
    pub expires_at: time::UtcDateTime,
    pub used: bool,

    pub code_challenge: String,
    pub code_challenge_method: String,
}

impl AuthorizationCode {
    pub fn is_expired(&self) -> bool {
        time::UtcDateTime::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.used && !self.is_expired()
    }
}

impl FromRow<'_, SqliteRow> for AuthorizationCode {
    fn from_row(row: &'_ SqliteRow) -> Result<Self, sqlx::Error> {
        let id: Vec<u8> = row.try_get("id")?;
        let usr: Vec<u8> = row.try_get("user_ud")?;

        let user_id =
            uuid::Uuid::from_slice(&usr).map_err(|err| sqlx::Error::Decode(err.into()))?;
        let id_uuid = uuid::Uuid::from_slice(&id).map_err(|err| sqlx::Error::Decode(err.into()))?;

        let ct: String = row.try_get("created_at")?;
        let et: String = row.try_get("expires_at")?;

        let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
        let created_at =
            time::UtcDateTime::parse(&ct, format).map_err(|err| sqlx::Error::Decode(err.into()))?;
        let expires_at =
            time::UtcDateTime::parse(&et, format).map_err(|err| sqlx::Error::Decode(err.into()))?;

        let code: String = row.try_get("code")?;
        let used: bool = row.try_get("used")?;
        let code_challenge: String = row.try_get("code_challenge")?;
        let code_challenge_method: String = row.try_get("code_challenge_method")?;

        Ok(Self {
            id: id_uuid,
            code,
            user_id,
            created_at,
            expires_at,
            used,
            code_challenge,
            code_challenge_method,
        })
    }
}

pub struct AuthorizationRequest {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    scope: Option<String>,
    state: Option<String>,
    code_challenge: String,
    code_challenge_method: String,
}
